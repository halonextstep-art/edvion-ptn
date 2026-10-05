//! ReportService — "Laporan" report-builder + saved reports for the School portal.
//!
//! Performance and Participation reports are computed directly from the school's own
//! roster + real tryout attempts (optionally scoped to one grade/class — real filtering,
//! not decorative). PTN Target reuses `SchoolRationalizationService`'s already-real
//! SNBP/SNBT aggregates, which are whole-school only; the class filter is therefore
//! ignored for that report type (never silently pretended to work — the frontend disables
//! the class picker when this type is selected).
//!
//! Every generated report is persisted as an immutable snapshot (`ReportRepository`), not
//! recomputed on every view — re-opening an old report always shows what was true at
//! generation time.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Datelike, Utc};
use uuid::Uuid;

use crate::application::analytics_service::AnalyticsService;
use crate::application::school_rationalization_service::SchoolRationalizationService;
use crate::application::school_tka_service::SchoolTkaService;
use crate::domain::package::ExamTrack;
use crate::domain::repository::{
    AttemptRepository, NewReport, ReportRepository, TryoutSessionRepository, UserFilter, UserRepository,
};
use crate::domain::report::{
    AkreditasiClassRow, AkreditasiReportPayload, AkreditasiStudentRow, ClassAvgRow, MonthlyTrendPoint,
    ParticipationPayload, ParticipationRow, PerformancePayload, PtnTargetPayload, RankingRow, Report, ReportPayload,
    ReportType, TkaPackageSummaryRow, TkaReportPayload, TkaStudentReportRow, UniversityCountSnapshot,
};
use crate::domain::tryout::{Attempt, AttemptStatus};
use crate::domain::user::{Role, User};
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct GenerateReportInput {
    pub report_type: ReportType,
    pub period_label: String,
    /// "YYYY-MM", `None` = whole history.
    pub year_month: Option<String>,
    /// Grade/class label matching `User.grade` exactly (e.g. "Kelas 12"). Ignored for
    /// `ReportType::PtnTarget`.
    pub class_filter: Option<String>,
    /// See `domain::report::Report::exam_track_filter` doc comment. Only applied for
    /// `Performance`/`Participation` — ignored (forced `None`) for `PtnTarget`/`Tka`/`Akreditasi`.
    pub exam_track_filter: Option<ExamTrack>,
    /// KKM-like academic cutoff used by `ReportType::Akreditasi` to compute
    /// "students_above_threshold" — not an existing platform concept (no `kkm`/
    /// `passing_grade` column anywhere), so it's supplied per-generation rather than a
    /// stored school setting. `None`/out-of-range falls back to a sane default (75).
    /// Ignored for every other report type.
    pub threshold: Option<i32>,
}

pub struct ReportService {
    users: Arc<dyn UserRepository>,
    attempts: Arc<dyn AttemptRepository>,
    sessions: Arc<dyn TryoutSessionRepository>,
    reports: Arc<dyn ReportRepository>,
    rationalization: Arc<SchoolRationalizationService>,
    tka: Arc<SchoolTkaService>,
    /// Only used by `ReportType::Akreditasi` for the real per-subject accuracy breakdown
    /// (`subject_breakdown_for_school`) — reused rather than re-deriving that SQL here.
    analytics: Arc<AnalyticsService>,
}

impl ReportService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        attempts: Arc<dyn AttemptRepository>,
        sessions: Arc<dyn TryoutSessionRepository>,
        reports: Arc<dyn ReportRepository>,
        rationalization: Arc<SchoolRationalizationService>,
        tka: Arc<SchoolTkaService>,
        analytics: Arc<AnalyticsService>,
    ) -> Self {
        Self { users, attempts, sessions, reports, rationalization, tka, analytics }
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

    async fn roster_for(&self, school_id: Uuid, class_filter: Option<&str>) -> AppResult<Vec<User>> {
        let all = self
            .users
            .list(UserFilter { role: Some(Role::Student), school_id: Some(school_id), ..Default::default() })
            .await?;
        Ok(match class_filter {
            Some(g) => all.into_iter().filter(|u| u.grade.as_deref() == Some(g)).collect(),
            None => all,
        })
    }

    pub async fn generate(&self, actor: &AuthUser, input: GenerateReportInput) -> AppResult<Report> {
        actor.require_role(&[Role::School])?;
        if input.period_label.trim().is_empty() {
            return Err(AppError::Validation("periode wajib diisi".to_string()));
        }
        let school_id = self.resolve_own_school_id(actor).await?;

        let (payload, type_label) = match input.report_type {
            ReportType::Performance => {
                let p = self
                    .build_performance(school_id, input.class_filter.as_deref(), input.year_month.as_deref(), input.exam_track_filter)
                    .await?;
                (ReportPayload::Performance(p), "Performa Siswa")
            }
            ReportType::Participation => {
                let p = self
                    .build_participation(school_id, input.class_filter.as_deref(), input.year_month.as_deref(), input.exam_track_filter)
                    .await?;
                (ReportPayload::Participation(p), "Partisipasi Tryout")
            }
            ReportType::PtnTarget => {
                let p = self.build_ptn_target(actor).await?;
                (ReportPayload::PtnTarget(p), "Analisis Target PTN")
            }
            ReportType::Tka => {
                let p = self.build_tka_report(actor, school_id, input.class_filter.as_deref()).await?;
                (ReportPayload::Tka(p), "Hasil TKA")
            }
            ReportType::Akreditasi => {
                let p = self
                    .build_akreditasi(actor, school_id, input.class_filter.as_deref(), input.threshold)
                    .await?;
                (ReportPayload::Akreditasi(p), "Laporan Akreditasi (EDS)")
            }
        };
        let title = format!("{type_label} — {}", input.period_label);

        // PtnTarget always covers the whole school — don't persist a class_filter that
        // wasn't actually applied.
        let class_filter = match input.report_type {
            ReportType::PtnTarget => None,
            _ => input.class_filter,
        };
        // exam_track_filter only ever really applies to Performance/Participation — PtnTarget
        // has no exam-track dimension, and a Tka report is implicitly TKA-only already.
        let exam_track_filter = match input.report_type {
            ReportType::Performance | ReportType::Participation => input.exam_track_filter,
            ReportType::PtnTarget | ReportType::Tka | ReportType::Akreditasi => None,
        };

        self.reports
            .create(NewReport {
                school_id,
                report_type: input.report_type,
                title,
                period_label: input.period_label,
                class_filter,
                exam_track_filter,
                payload,
            })
            .await
    }

    pub async fn list(&self, actor: &AuthUser) -> AppResult<Vec<Report>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.reports.list_by_school(school_id).await
    }

    pub async fn get(&self, actor: &AuthUser, id: Uuid) -> AppResult<Report> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let report =
            self.reports.find_by_id(id).await?.ok_or_else(|| AppError::NotFound("laporan tidak ditemukan".to_string()))?;
        if report.school_id != school_id {
            return Err(AppError::Forbidden("laporan ini bukan milik sekolah anda".to_string()));
        }
        Ok(report)
    }

    pub async fn delete(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        let report = self.get(actor, id).await?; // ownership check reused
        self.reports.delete(report.id).await
    }

    // ── Performance ──────────────────────────────────────────────────────────────

    /// Every session's `exam_track`, keyed by session id — built once per report generation
    /// (not cached across calls) so a filtered Performance/Participation report can tell which
    /// track each `Attempt` belongs to. `Attempt` itself doesn't denormalize `exam_track` (it
    /// only carries `session_title`/`session_type`), so this cross-reference is the cheapest
    /// way to filter without touching that widely-used struct.
    async fn session_exam_tracks(&self) -> AppResult<HashMap<Uuid, ExamTrack>> {
        let sessions = self.sessions.list(None).await?;
        Ok(sessions.into_iter().map(|s| (s.id, s.exam_track)).collect())
    }

    fn matches_track(session_id: Uuid, filter: Option<ExamTrack>, tracks: &HashMap<Uuid, ExamTrack>) -> bool {
        match filter {
            None => true,
            Some(f) => tracks.get(&session_id) == Some(&f),
        }
    }

    async fn build_performance(
        &self,
        school_id: Uuid,
        class_filter: Option<&str>,
        year_month: Option<&str>,
        exam_track_filter: Option<ExamTrack>,
    ) -> AppResult<PerformancePayload> {
        let students = self.roster_for(school_id, class_filter).await?;
        let tracks = self.session_exam_tracks().await?;

        let mut per_class: HashMap<String, (i64, f64, i64)> = HashMap::new(); // grade -> (student_count, score_sum, score_n)
        let mut monthly: HashMap<String, (f64, i64)> = HashMap::new(); // "YYYY-MM" -> (score_sum, attempt_count)
        let mut ranking: Vec<RankingRow> = Vec::new();
        let mut total_sum = 0.0;
        let mut total_n = 0i64;

        for s in &students {
            let attempts = self.attempts.list_by_student(s.id).await?;

            let in_period: Vec<&Attempt> = attempts
                .iter()
                .filter(|a| a.status == AttemptStatus::Submitted && a.score.is_some())
                .filter(|a| matches_period(a.submitted_at, year_month))
                .filter(|a| Self::matches_track(a.session_id, exam_track_filter, &tracks))
                .collect();
            let scores: Vec<i32> = in_period.iter().filter_map(|a| a.score).collect();
            let avg = avg_i32(&scores);
            let best = scores.iter().copied().max();

            ranking.push(RankingRow {
                student_id: s.id,
                student_name: s.name.clone(),
                grade: s.grade.clone(),
                attempt_count: in_period.len() as i64,
                best_score: best,
                avg_score: avg,
            });

            if let Some(a) = avg {
                let grade = s.grade.clone().unwrap_or_else(|| "Tanpa Kelas".to_string());
                let entry = per_class.entry(grade).or_insert((0, 0.0, 0));
                entry.0 += 1;
                entry.1 += a;
                entry.2 += 1;
                total_sum += a;
                total_n += 1;
            }

            // Trend always spans full history regardless of the selected period, so a
            // multi-month series stays meaningful even when one specific month is picked.
            for at in attempts
                .iter()
                .filter(|a| a.status == AttemptStatus::Submitted && a.score.is_some())
                .filter(|a| Self::matches_track(a.session_id, exam_track_filter, &tracks))
            {
                if let Some(dt) = at.submitted_at {
                    let key = year_month_key(dt);
                    let e = monthly.entry(key).or_insert((0.0, 0));
                    e.0 += at.score.unwrap() as f64;
                    e.1 += 1;
                }
            }
        }

        let mut per_class_rows: Vec<ClassAvgRow> = per_class
            .into_iter()
            .map(|(grade, (count, sum, n))| ClassAvgRow {
                grade,
                student_count: count,
                avg_score: if n > 0 { Some(sum / n as f64) } else { None },
            })
            .collect();
        per_class_rows.sort_by(|a, b| a.grade.cmp(&b.grade));

        let mut monthly_keys: Vec<String> = monthly.keys().cloned().collect();
        monthly_keys.sort();
        let last_keys: Vec<String> = {
            let mut rev: Vec<String> = monthly_keys.into_iter().rev().take(6).collect();
            rev.reverse();
            rev
        };
        let monthly_trend = last_keys
            .into_iter()
            .map(|k| {
                let (sum, n) = monthly[&k];
                MonthlyTrendPoint { month_label: month_label(&k), avg_score: if n > 0 { Some(sum / n as f64) } else { None }, attempt_count: n }
            })
            .collect();

        ranking.sort_by(|a, b| {
            b.avg_score.unwrap_or(-1.0).partial_cmp(&a.avg_score.unwrap_or(-1.0)).unwrap_or(std::cmp::Ordering::Equal)
        });
        ranking.truncate(10);

        Ok(PerformancePayload {
            total_students: students.len() as i64,
            avg_score: if total_n > 0 { Some(total_sum / total_n as f64) } else { None },
            per_class: per_class_rows,
            monthly_trend,
            ranking,
        })
    }

    // ── Participation ────────────────────────────────────────────────────────────

    async fn build_participation(
        &self,
        school_id: Uuid,
        class_filter: Option<&str>,
        year_month: Option<&str>,
        exam_track_filter: Option<ExamTrack>,
    ) -> AppResult<ParticipationPayload> {
        let students = self.roster_for(school_id, class_filter).await?;
        let tracks = self.session_exam_tracks().await?;
        let mut rows = Vec::with_capacity(students.len());
        let mut total_attempts = 0i64;
        let mut active = 0i64;

        for s in &students {
            let attempts = self.attempts.list_by_student(s.id).await?;
            let in_period: Vec<&Attempt> = attempts
                .iter()
                .filter(|a| a.status == AttemptStatus::Submitted)
                .filter(|a| matches_period(a.submitted_at, year_month))
                .filter(|a| Self::matches_track(a.session_id, exam_track_filter, &tracks))
                .collect();
            let last_attempt_at = in_period.iter().filter_map(|a| a.submitted_at).max();
            if !in_period.is_empty() {
                active += 1;
            }
            total_attempts += in_period.len() as i64;
            rows.push(ParticipationRow {
                student_id: s.id,
                student_name: s.name.clone(),
                grade: s.grade.clone(),
                attempt_count: in_period.len() as i64,
                last_attempt_at,
            });
        }
        rows.sort_by(|a, b| b.attempt_count.cmp(&a.attempt_count));

        let total = students.len() as i64;
        Ok(ParticipationPayload {
            total_students: total,
            active_students: active,
            participation_rate: if total > 0 { (active as f64 / total as f64) * 100.0 } else { 0.0 },
            total_attempts,
            avg_attempts_per_active_student: if active > 0 { total_attempts as f64 / active as f64 } else { 0.0 },
            rows,
        })
    }

    // ── PTN Target (reuses the already-real rationalization aggregates) ─────────────

    async fn build_ptn_target(&self, actor: &AuthUser) -> AppResult<PtnTargetPayload> {
        let dash = self.rationalization.dashboard(actor).await?;
        let snbt = self.rationalization.snbt_dashboard(actor).await?;
        Ok(PtnTargetPayload {
            total_siswa_dengan_target: dash.terasionalisasi,
            distribusi_universitas: dash
                .distribusi_universitas
                .into_iter()
                .map(|u| UniversityCountSnapshot { nama_ptn: u.nama_ptn, count: u.count })
                .collect(),
            snbt_terdaftar: snbt.terdaftar,
            snbt_sudah_ujian: snbt.sudah_ujian,
            snbt_diterima: snbt.diterima,
            avg_chance_snbp: dash.avg_chance_snbp,
        })
    }

    // ── TKA (reuses SchoolTkaService::tka_roster's already-real aggregate) ──────────

    async fn build_tka_report(&self, actor: &AuthUser, school_id: Uuid, class_filter: Option<&str>) -> AppResult<TkaReportPayload> {
        let groups = self.tka.tka_roster(actor).await?;
        // `TkaStudentRow` doesn't carry grade (school_tka_service has no class-filter concept
        // of its own) — cross-reference against the roster so this report can be scoped by
        // class like every other report type.
        let roster = self.roster_for(school_id, None).await?;
        let grade_by_student: HashMap<Uuid, Option<String>> = roster.into_iter().map(|u| (u.id, u.grade)).collect();

        let mut package_rows = Vec::with_capacity(groups.len());
        let mut student_rows: Vec<TkaStudentReportRow> = Vec::new();
        let mut students_in_scope: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

        for g in &groups {
            let mut in_scope_count = 0i64;
            let mut ready_count = 0i64;
            let (mut wajib_sum, mut wajib_n) = (0.0, 0i64);
            let (mut pilihan_sum, mut pilihan_n) = (0.0, 0i64);
            let (mut raw_wajib_sum, mut raw_wajib_n) = (0.0, 0i64);
            let (mut raw_pilihan_sum, mut raw_pilihan_n) = (0.0, 0i64);
            let (mut istimewa_sum, mut istimewa_n) = (0i64, 0i64);

            for st in &g.students {
                let grade = grade_by_student.get(&st.student_id).cloned().flatten();
                if let Some(cf) = class_filter {
                    if grade.as_deref() != Some(cf) {
                        continue;
                    }
                }
                in_scope_count += 1;
                students_in_scope.insert(st.student_id);
                if st.elective_chosen_count >= g.elective_pick_count {
                    ready_count += 1;
                }

                let wajib_scores: Vec<i32> = st.subjects.iter().filter(|s| !s.is_elective).filter_map(|s| s.scaled_score).collect();
                let pilihan_scores: Vec<i32> = st.subjects.iter().filter(|s| s.is_elective).filter_map(|s| s.scaled_score).collect();
                let raw_wajib_scores: Vec<i32> = st.subjects.iter().filter(|s| !s.is_elective).filter_map(|s| s.raw_score).collect();
                let raw_pilihan_scores: Vec<i32> = st.subjects.iter().filter(|s| s.is_elective).filter_map(|s| s.raw_score).collect();
                let istimewa_count = st.subjects.iter().filter(|s| s.is_istimewa == Some(true)).count() as i32;

                let avg_wajib = avg_i32(&wajib_scores);
                let avg_pilihan = avg_i32(&pilihan_scores);
                let avg_raw_wajib = avg_i32(&raw_wajib_scores);
                let avg_raw_pilihan = avg_i32(&raw_pilihan_scores);
                if let Some(a) = avg_wajib {
                    wajib_sum += a;
                    wajib_n += 1;
                }
                if let Some(a) = avg_pilihan {
                    pilihan_sum += a;
                    pilihan_n += 1;
                }
                if let Some(a) = avg_raw_wajib {
                    raw_wajib_sum += a;
                    raw_wajib_n += 1;
                }
                if let Some(a) = avg_raw_pilihan {
                    raw_pilihan_sum += a;
                    raw_pilihan_n += 1;
                }
                istimewa_sum += istimewa_count as i64;
                istimewa_n += 1;

                student_rows.push(TkaStudentReportRow {
                    student_id: st.student_id,
                    student_name: st.student_name.clone(),
                    grade,
                    package_name: g.package_name.clone(),
                    school_type_scope: g.school_type_scope,
                    score_scale_label: g.score_scale_label.to_string(),
                    istimewa_threshold: g.istimewa_threshold,
                    elective_chosen_count: st.elective_chosen_count,
                    elective_pick_count: g.elective_pick_count,
                    avg_wajib_score: avg_wajib,
                    avg_pilihan_score: avg_pilihan,
                    avg_raw_wajib_score: avg_raw_wajib,
                    avg_raw_pilihan_score: avg_raw_pilihan,
                    istimewa_count,
                    subject_count: st.subjects.len() as i32,
                });
            }

            package_rows.push(TkaPackageSummaryRow {
                package_id: g.package_id,
                package_name: g.package_name.clone(),
                elective_pick_count: g.elective_pick_count,
                student_count: in_scope_count,
                elective_ready_count: ready_count,
                school_type_scope: g.school_type_scope,
                score_scale_label: g.score_scale_label.to_string(),
                istimewa_threshold: g.istimewa_threshold,
                avg_wajib_score: if wajib_n > 0 { Some(wajib_sum / wajib_n as f64) } else { None },
                avg_pilihan_score: if pilihan_n > 0 { Some(pilihan_sum / pilihan_n as f64) } else { None },
                avg_raw_wajib_score: if raw_wajib_n > 0 { Some(raw_wajib_sum / raw_wajib_n as f64) } else { None },
                avg_raw_pilihan_score: if raw_pilihan_n > 0 { Some(raw_pilihan_sum / raw_pilihan_n as f64) } else { None },
                avg_istimewa_count: if istimewa_n > 0 { Some(istimewa_sum as f64 / istimewa_n as f64) } else { None },
            });
        }

        student_rows.sort_by(|a, b| a.student_name.cmp(&b.student_name));

        Ok(TkaReportPayload {
            total_students: students_in_scope.len() as i64,
            packages: package_rows,
            students: student_rows,
        })
    }

    // ── Akreditasi/EDS (academic-performance summary; reuses AnalyticsService's already-real
    // subject accuracy + SchoolRationalizationService's already-real target aggregates rather
    // than re-deriving new SQL) ─────────────────────────────────────────────────────────

    async fn build_akreditasi(
        &self,
        actor: &AuthUser,
        school_id: Uuid,
        class_filter: Option<&str>,
        threshold: Option<i32>,
    ) -> AppResult<AkreditasiReportPayload> {
        const DEFAULT_THRESHOLD: i32 = 75;
        let threshold_used = threshold.filter(|t| (0..=100).contains(t)).unwrap_or(DEFAULT_THRESHOLD);

        let students = self.roster_for(school_id, class_filter).await?;

        // Union of students with a real, measurable PTN target (SNBP and/or SNBT) — the
        // "kesiapan lulusan" indicator. Both calls are whole-school (no class filter of their
        // own), which is fine since we only use them to look up individual student ids below.
        let mut target_ids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        for row in self.rationalization.full_ranking(actor).await? {
            target_ids.insert(row.student_id);
        }
        for row in self.rationalization.snbt_roster(actor).await? {
            target_ids.insert(row.student_id);
        }

        let mut per_class: HashMap<String, (i64, f64, i64, i64)> = HashMap::new(); // grade -> (student_count, score_sum, score_n, above_threshold_count)
        let mut student_rows: Vec<AkreditasiStudentRow> = Vec::with_capacity(students.len());
        let mut total_sum = 0.0;
        let mut total_n = 0i64;
        let mut above_threshold_total = 0i64;
        let mut with_target_total = 0i64;

        for s in &students {
            let attempts = self.attempts.list_by_student(s.id).await?;
            let submitted: Vec<&Attempt> = attempts.iter().filter(|a| a.status == AttemptStatus::Submitted && a.score.is_some()).collect();
            let scores: Vec<i32> = submitted.iter().filter_map(|a| a.score).collect();
            let avg = avg_i32(&scores);
            let above = avg.map(|a| a >= threshold_used as f64).unwrap_or(false);
            let has_target = target_ids.contains(&s.id);

            if let Some(a) = avg {
                let grade = s.grade.clone().unwrap_or_else(|| "Tanpa Kelas".to_string());
                let entry = per_class.entry(grade).or_insert((0, 0.0, 0, 0));
                entry.0 += 1;
                entry.1 += a;
                entry.2 += 1;
                if above {
                    entry.3 += 1;
                }
                total_sum += a;
                total_n += 1;
                if above {
                    above_threshold_total += 1;
                }
            }
            if has_target {
                with_target_total += 1;
            }

            student_rows.push(AkreditasiStudentRow {
                student_id: s.id,
                student_name: s.name.clone(),
                grade: s.grade.clone(),
                attempt_count: submitted.len() as i64,
                avg_score: avg,
                above_threshold: above,
                has_ptn_target: has_target,
            });
        }
        student_rows.sort_by(|a, b| a.student_name.cmp(&b.student_name));

        let mut per_class_rows: Vec<AkreditasiClassRow> = per_class
            .into_iter()
            .map(|(grade, (count, sum, n, above))| AkreditasiClassRow {
                grade,
                student_count: count,
                avg_score: if n > 0 { Some(sum / n as f64) } else { None },
                above_threshold_count: above,
            })
            .collect();
        per_class_rows.sort_by(|a, b| a.grade.cmp(&b.grade));

        // Whole-school only — see `AkreditasiReportPayload::per_subject` doc comment.
        let per_subject = self.analytics.subject_breakdown_for_school(actor).await?;

        Ok(AkreditasiReportPayload {
            total_students: students.len() as i64,
            threshold_used,
            avg_score_overall: if total_n > 0 { Some(total_sum / total_n as f64) } else { None },
            students_above_threshold: above_threshold_total,
            students_with_ptn_target: with_target_total,
            per_subject,
            per_class: per_class_rows,
            students: student_rows,
        })
    }
}

fn avg_i32(scores: &[i32]) -> Option<f64> {
    if scores.is_empty() {
        None
    } else {
        Some(scores.iter().sum::<i32>() as f64 / scores.len() as f64)
    }
}

fn matches_period(submitted_at: Option<DateTime<Utc>>, year_month: Option<&str>) -> bool {
    match (year_month, submitted_at) {
        (Some(ym), Some(dt)) => year_month_key(dt) == ym,
        (Some(_), None) => false,
        (None, _) => true,
    }
}

fn year_month_key(dt: DateTime<Utc>) -> String {
    format!("{:04}-{:02}", dt.year(), dt.month())
}

fn month_label(ym: &str) -> String {
    const MONTHS: [&str; 12] =
        ["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"];
    let parts: Vec<&str> = ym.split('-').collect();
    if parts.len() == 2 {
        if let Ok(m) = parts[1].parse::<usize>() {
            if (1..=12).contains(&m) {
                return format!("{} {}", MONTHS[m - 1], parts[0]);
            }
        }
    }
    ym.to_string()
}
