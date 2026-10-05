//! Rekap TKA — school-facing monitoring for TKA-style content, mirroring
//! `SchoolRationalizationService`'s Rekap SNBT but for TKA instead.
//!
//! A "TKA package" is defined as any `Package` with `exam_track == Tka` — the explicit,
//! admin-chosen signal (see `domain::package::ExamTrack` doc comment). This USED to be
//! `elective_pick_count > 0` (predating `exam_track`'s existence), which was wrong in two
//! directions: it silently EXCLUDED real TKA SMP packages (SMP's TKA has no mapel pilihan at
//! all, so `elective_pick_count` is legitimately 0 there), and would have INCLUDED any
//! unrelated SNBT package that happened to set an elective pick count. `elective_pick_count`
//! is still read below — it's the correct signal for "does this package's mapel pilihan
//! progress need tracking", just not for "is this a TKA package" anymore.
//!
//! Per-subject scores are read directly from real `Attempt` rows on the package's own
//! `TryoutSession`s (both `is_elective` and mandatory ones) — this covers a student's practice
//! attempts AND their Simulasi TKA slot attempts identically, since a Simulasi run's slot is
//! just a normal `Attempt` under the hood (see `application::simulation_service` doc comment).
//!
//! Sessions belonging to a package are gathered from TWO places, unioned: (1) `TryoutSession`
//! content items attached directly to the package (the older, manual-attach path), and (2) any
//! `SimulationTemplate` content item's own slots (the "Susun Simulasi TKA" wizard path, which
//! only ever attaches the template itself as a content item — never the individual sessions
//! separately). A package assembled purely through the wizard would otherwise report zero
//! sessions here and get silently skipped, which is exactly the bug this module used to have.

use std::collections::HashSet;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::package::{ExamTrack, PackageContentType};
use crate::domain::repository::{
    AttemptRepository, PackageContentRepository, PackageElectiveRepository, PackageRepository,
    SimulationTemplateRepository, TryoutSessionRepository, UserFilter, UserRepository,
};
use crate::domain::school::SchoolType;
use crate::domain::tryout::{AttemptStatus, TryoutSession};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct TkaSubjectScoreRow {
    pub session_id: Uuid,
    pub session_title: String,
    pub is_elective: bool,
    pub attempted: bool,
    /// Platform 0-1000 scale ("Skor Instan"), from the student's latest SUBMITTED attempt on
    /// this session — kept alongside `scaled_score` purely as a transparency figure (so
    /// students/schools can see the raw platform number a scaled score was derived from), never
    /// the primary/official-looking number. See module doc comment.
    pub raw_score: Option<i32>,
    /// Honest display-only rescale to the jenjang-appropriate TKA ballpark (0-100 for SD/SMP,
    /// 200-800 for SMA/SMK/MA) — see `tka_scale_for` doc comment. Never a real IRT/officially
    /// calibrated score.
    pub scaled_score: Option<i32>,
    /// Whether this subject's score meets the "Istimewa" threshold for this package's jenjang
    /// (95 for SD/SMP, 725 for SMA/SMK/MA) — precomputed here so callers never need to
    /// re-derive the jenjang-specific threshold themselves. `None` when `raw_score` is `None`.
    pub is_istimewa: Option<bool>,
    pub submitted_at: Option<DateTime<Utc>>,
}

pub struct TkaStudentRow {
    pub student_id: Uuid,
    pub student_name: String,
    pub elective_chosen_count: i32,
    pub subjects: Vec<TkaSubjectScoreRow>,
}

pub struct TkaPackageGroup {
    pub package_id: Uuid,
    pub package_name: String,
    pub elective_pick_count: i32,
    /// Jenjang paket ini, dideteksi dari `school_type_scope` sesi-sesinya (sama seperti
    /// `PackageWizard.vue`'s own reconstruction logic — sesi pertama yang punya nilai menang).
    /// `None` = tidak diset eksplisit oleh admin; jatuh ke skala SMA/SMK/MA sebagai default
    /// aman (lihat `tka_scale_for` doc comment).
    pub school_type_scope: Option<SchoolType>,
    /// Label skala tampilan untuk paket ini, mis. "0–100" atau "200–800" — dari `tka_scale_for`.
    pub score_scale_label: &'static str,
    /// Ambang nilai kategori "Istimewa" untuk paket ini — 95 atau 725, dari `tka_scale_for`.
    pub istimewa_threshold: i32,
    pub students: Vec<TkaStudentRow>,
}

pub struct SchoolTkaService {
    users: Arc<dyn UserRepository>,
    packages: Arc<dyn PackageRepository>,
    package_content: Arc<dyn PackageContentRepository>,
    package_elective: Arc<dyn PackageElectiveRepository>,
    sessions: Arc<dyn TryoutSessionRepository>,
    attempts: Arc<dyn AttemptRepository>,
    /// Only used to resolve a `SimulationTemplate` content item's slots into real sessions —
    /// see module doc comment.
    simulation_templates: Arc<dyn SimulationTemplateRepository>,
}

impl SchoolTkaService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        packages: Arc<dyn PackageRepository>,
        package_content: Arc<dyn PackageContentRepository>,
        package_elective: Arc<dyn PackageElectiveRepository>,
        sessions: Arc<dyn TryoutSessionRepository>,
        attempts: Arc<dyn AttemptRepository>,
        simulation_templates: Arc<dyn SimulationTemplateRepository>,
    ) -> Self {
        Self { users, packages, package_content, package_elective, sessions, attempts, simulation_templates }
    }

    /// One group per TKA package (every real one — usually "TKA SMA" and/or "TKA SMP"), each
    /// with one row per student on this school's roster: their elective-pick progress and their
    /// latest real score (if any) on every mata uji (wajib + pilihan) belonging to that package.
    pub async fn tka_roster(&self, actor: &AuthUser) -> AppResult<Vec<TkaPackageGroup>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let students = self
            .users
            .list(UserFilter { role: Some(Role::Student), school_id: Some(school_id), ..Default::default() })
            .await?;

        let all_packages = self.packages.list().await?;
        let mut groups = Vec::new();
        for pkg in all_packages {
            if pkg.exam_track != ExamTrack::Tka {
                continue;
            }
            let pkg_sessions = self.package_sessions(pkg.id).await?;
            if pkg_sessions.is_empty() {
                continue; // e.g. still a draft in the wizard — nothing real to report yet
            }
            // Jenjang paket ini — sesi pertama yang punya `school_type_scope` menang, sama
            // seperti `PackageWizard.vue`'s own reconstruction logic (`detectedScope`).
            let jenjang = pkg_sessions.iter().find_map(|s| s.school_type_scope);
            let scale = tka_scale_for(jenjang);

            let mut student_rows = Vec::with_capacity(students.len());
            for student in &students {
                let choices = self.package_elective.list_choices(student.id, pkg.id).await?;
                let student_attempts = self.attempts.list_by_student(student.id).await?;
                let mut subjects = Vec::with_capacity(pkg_sessions.len());
                for s in &pkg_sessions {
                    let latest = student_attempts
                        .iter()
                        .filter(|a| a.session_id == s.id && a.status == AttemptStatus::Submitted)
                        .max_by_key(|a| a.submitted_at);
                    let raw_score = latest.and_then(|a| a.score);
                    subjects.push(TkaSubjectScoreRow {
                        session_id: s.id,
                        session_title: s.title.clone(),
                        is_elective: s.is_elective,
                        attempted: latest.is_some(),
                        raw_score,
                        scaled_score: raw_score.map(|r| tka_scaled_score(r, jenjang)),
                        is_istimewa: raw_score.map(|r| is_istimewa(r, jenjang)),
                        submitted_at: latest.and_then(|a| a.submitted_at),
                    });
                }
                student_rows.push(TkaStudentRow {
                    student_id: student.id,
                    student_name: student.name.clone(),
                    elective_chosen_count: choices.len() as i32,
                    subjects,
                });
            }
            student_rows.sort_by(|a, b| a.student_name.cmp(&b.student_name));

            groups.push(TkaPackageGroup {
                package_id: pkg.id,
                package_name: pkg.name.clone(),
                elective_pick_count: pkg.elective_pick_count,
                school_type_scope: jenjang,
                score_scale_label: scale.label,
                istimewa_threshold: scale.istimewa_threshold,
                students: student_rows,
            });
        }
        Ok(groups)
    }

    /// Every non-draft session belonging to this package — unioned from directly-attached
    /// `TryoutSession` content items AND from any attached `SimulationTemplate`'s own slots
    /// (deduped by session id). See module doc comment for why both sources are needed.
    async fn package_sessions(&self, package_id: Uuid) -> AppResult<Vec<TryoutSession>> {
        let content_items = self.package_content.list_for_package(package_id).await?;
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for item in &content_items {
            match item.content_type {
                PackageContentType::TryoutSession => {
                    if let Some(s) = self.sessions.find_by_id(item.content_id).await? {
                        if !s.is_draft && seen.insert(s.id) {
                            out.push(s);
                        }
                    }
                }
                PackageContentType::SimulationTemplate => {
                    let Some(tpl) = self.simulation_templates.find_by_id(item.content_id).await? else { continue };
                    for slot in &tpl.slots {
                        // `None` for `is_elective` slots — no single fixed session to resolve
                        // (see `domain::simulation::SimulationTemplateSlot` doc comment); those
                        // are only ever tracked via a directly-attached `TryoutSession` item.
                        let Some(session_id) = slot.session_id else { continue };
                        if seen.contains(&session_id) {
                            continue;
                        }
                        if let Some(s) = self.sessions.find_by_id(session_id).await? {
                            if !s.is_draft && seen.insert(s.id) {
                                out.push(s);
                            }
                        }
                    }
                }
            }
        }
        Ok(out)
    }

    async fn resolve_own_school_id(&self, actor: &AuthUser) -> AppResult<Uuid> {
        let me = self
            .users
            .find_by_id(actor.user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("akun tidak ditemukan".to_string()))?;
        me.school_id
            .ok_or_else(|| AppError::Validation("akun sekolah ini belum terhubung ke data sekolah".to_string()))
    }
}

/// A jenjang-appropriate TKA display scale — base/span for the honest 0-1000 -> display
/// rescale, plus that jenjang's official "Istimewa" threshold. Two real scales exist:
/// - SD/SMP: skala 0–100, ambang Istimewa 95,00 per mata uji (Perka BSKAP No. 047/H/AN/2025).
/// - SMA/MA/SMK/MAK: skala 200–800, ambang Istimewa 725,00 per mata uji (Perka BSKAP
///   No. 045/H/AN/2025).
/// `Some(SchoolType::Smp)` is the only jenjang mapped to the SD/SMP scale (this platform has no
/// separate `Sd` `SchoolType` — see its doc comment); every other scope (including `None`, i.e.
/// no jenjang set explicitly on the package's sessions) falls back to the SMA/SMK/MA scale as a
/// safe default, since that's this platform's majority content. Baik/Memadai/Kurang cutoffs are
/// intentionally NOT reproduced anywhere in this codebase — no real standard-setting data exists
/// here to derive them honestly.
pub(crate) struct TkaScale {
    pub base: f64,
    pub span: f64,
    pub istimewa_threshold: i32,
    pub label: &'static str,
}

pub(crate) fn tka_scale_for(scope: Option<SchoolType>) -> TkaScale {
    match scope {
        Some(SchoolType::Smp) => TkaScale { base: 0.0, span: 100.0, istimewa_threshold: 95, label: "0–100" },
        _ => TkaScale { base: 200.0, span: 600.0, istimewa_threshold: 725, label: "200–800" },
    }
}

/// Honest, display-only 0-1000 -> jenjang-appropriate scale rescale — see `tka_scale_for` doc
/// comment. Mirrored in `frontend/pages/simulasi/[runId].vue` (`per_subject` Simulasi TKA runs)
/// and `frontend/utils/simulationCertificate.ts`. Kept in sync manually since one side is Rust
/// and the other TypeScript; if the scale ever changes, update all three.
pub(crate) fn tka_scaled_score(raw: i32, scope: Option<SchoolType>) -> i32 {
    let scale = tka_scale_for(scope);
    (scale.base + (raw as f64 / 1000.0) * scale.span).round() as i32
}

pub(crate) fn is_istimewa(raw_score: i32, scope: Option<SchoolType>) -> bool {
    let scale = tka_scale_for(scope);
    tka_scaled_score(raw_score, scope) >= scale.istimewa_threshold
}
