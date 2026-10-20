//! School-portal-only aggregate reads — powers the "Overview" tab of the School portal.
//! Every figure is computed live from the school's own real students' `attempts` and
//! `student_ptn_targets` rows (via the already-existing repositories), never stored or
//! cached separately. Distinct from `SchoolService` (which manages the `schools` entity
//! itself, admin-only) and from `UserService` (which now also handles the School-scoped
//! student roster CRUD directly).
//!
//! Note: this loops per-student to sum up attempts/targets (N+1 queries) rather than a
//! single aggregate SQL query. Acceptable for the realistic school sizes this platform
//! targets; a dedicated aggregate repository method would be the next optimization if a
//! school's roster grows into the thousands.

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::repository::{
    AttemptRepository, PtnProgramRepository, PtnTargetRepository, SimulationRunRepository, UserFilter, UserRepository,
};
use crate::domain::simulation::SimulationRunStatus;
use crate::domain::tryout::{AttemptStatus, SessionType};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct SchoolOverview {
    pub total_students: i64,
    pub avg_score: Option<f64>,
    pub tryouts_completed: i64,
    pub target_ptn_count: i64,
}

/// Per-student real stats backing the School portal's "Manajemen Siswa" table (score/
/// tryout/target-PTN columns) — same N+1-over-roster computation as `overview`, just
/// broken out per student instead of summed.
pub struct StudentRosterStat {
    pub student_id: uuid::Uuid,
    pub best_score: Option<i32>,
    pub avg_score: Option<f64>,
    pub tryouts_completed: i64,
    pub target_ptn_count: i64,
    /// Most recent real submitted attempt (any session type) — powers "Terakhir Aktif" in
    /// the student detail drawer. `None` if this student has never submitted anything.
    pub last_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// One row of the School Analytics "Distribusi Target PTN" chart — real count of this
/// school's students who added a target at that university, across any track.
pub struct PtnDistributionItem {
    pub nama_ptn: String,
    pub count: i64,
}

/// One row of the School Overview "Aktivitas Terkini" feed — a real submitted attempt
/// (tryout/drilling/mini) from one of this school's students, most recent first.
pub struct RecentActivityItem {
    pub student_name: String,
    pub session_title: String,
    pub session_type: SessionType,
    pub score: Option<i32>,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}

/// One student currently mid-exam, for the School portal's live "Pemantauan Pengerjaan"
/// view — either a plain `Attempt` (Tryout/Drilling/Mini) or a `SimulationRun` (Simulasi
/// UTBK/TKA), normalized to the same shape since the school PIC doesn't need to care which
/// kind it is to decide whether to step in. `kind`/`ref_id` together are what
/// `force_abandon_activity` takes to act on this exact row.
pub struct InProgressActivityItem {
    pub kind: InProgressActivityKind,
    /// The `Attempt.id` or `SimulationRun.id` — opaque to the caller, round-tripped back into
    /// `force_abandon_activity`.
    pub ref_id: Uuid,
    pub student_id: Uuid,
    pub student_name: String,
    pub title: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InProgressActivityKind {
    Attempt,
    SimulationRun,
}

pub struct SchoolPortalService {
    users: Arc<dyn UserRepository>,
    attempts: Arc<dyn AttemptRepository>,
    ptn_targets: Arc<dyn PtnTargetRepository>,
    programs: Arc<dyn PtnProgramRepository>,
    simulation_runs: Arc<dyn SimulationRunRepository>,
}

impl SchoolPortalService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        attempts: Arc<dyn AttemptRepository>,
        ptn_targets: Arc<dyn PtnTargetRepository>,
        programs: Arc<dyn PtnProgramRepository>,
        simulation_runs: Arc<dyn SimulationRunRepository>,
    ) -> Self {
        Self { users, attempts, ptn_targets, programs, simulation_runs }
    }

    pub async fn overview(&self, actor: &AuthUser) -> AppResult<SchoolOverview> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let students = self
            .users
            .list(UserFilter { role: Some(Role::Student), school_id: Some(school_id), ..Default::default() }, &actor.tenant)
            .await?;

        let mut total_score: i64 = 0;
        let mut score_count: i64 = 0;
        let mut tryouts_completed: i64 = 0;
        let mut target_count: i64 = 0;

        for student in &students {
            let attempts = self.attempts.list_by_student(student.id, &actor.tenant).await?;
            for a in attempts {
                if a.status == AttemptStatus::Submitted {
                    tryouts_completed += 1;
                    if let Some(score) = a.score {
                        total_score += score as i64;
                        score_count += 1;
                    }
                }
            }
            let targets = self.ptn_targets.list_by_student(student.id, &actor.tenant).await?;
            target_count += targets.len() as i64;
        }

        Ok(SchoolOverview {
            total_students: students.len() as i64,
            avg_score: if score_count > 0 { Some(total_score as f64 / score_count as f64) } else { None },
            tryouts_completed,
            target_ptn_count: target_count,
        })
    }

    /// Per-student breakdown for the "Manajemen Siswa" table — real score/tryout/target
    /// figures per student, computed the same way as `overview`'s totals.
    pub async fn roster_stats(&self, actor: &AuthUser) -> AppResult<Vec<StudentRosterStat>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let students = self
            .users
            .list(UserFilter { role: Some(Role::Student), school_id: Some(school_id), ..Default::default() }, &actor.tenant)
            .await?;

        let mut out = Vec::with_capacity(students.len());
        for student in &students {
            let attempts = self.attempts.list_by_student(student.id, &actor.tenant).await?;
            let mut best_score: Option<i32> = None;
            let mut total_score: i64 = 0;
            let mut score_count: i64 = 0;
            let mut tryouts_completed: i64 = 0;
            let mut last_attempt_at: Option<chrono::DateTime<chrono::Utc>> = None;
            for a in attempts {
                if a.status == AttemptStatus::Submitted {
                    tryouts_completed += 1;
                    if let Some(score) = a.score {
                        total_score += score as i64;
                        score_count += 1;
                        best_score = Some(best_score.map_or(score, |b| b.max(score)));
                    }
                    if let Some(submitted_at) = a.submitted_at {
                        last_attempt_at = Some(last_attempt_at.map_or(submitted_at, |t| t.max(submitted_at)));
                    }
                }
            }
            let targets = self.ptn_targets.list_by_student(student.id, &actor.tenant).await?;
            out.push(StudentRosterStat {
                student_id: student.id,
                best_score,
                avg_score: if score_count > 0 { Some(total_score as f64 / score_count as f64) } else { None },
                tryouts_completed,
                target_ptn_count: targets.len() as i64,
                last_attempt_at,
            });
        }
        Ok(out)
    }

    /// School Analytics "Distribusi Target PTN" — real count of this school's students'
    /// targets grouped by university, across all students/tracks. Top 6 by count.
    pub async fn ptn_distribution(&self, actor: &AuthUser) -> AppResult<Vec<PtnDistributionItem>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let students = self
            .users
            .list(UserFilter { role: Some(Role::Student), school_id: Some(school_id), ..Default::default() }, &actor.tenant)
            .await?;

        let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for student in &students {
            let targets = self.ptn_targets.list_by_student(student.id, &actor.tenant).await?;
            for t in targets {
                if let Some(program) = self.programs.find_by_id(t.ptn_program_id).await? {
                    *counts.entry(program.nama_ptn).or_insert(0) += 1;
                }
            }
        }

        let mut out: Vec<PtnDistributionItem> =
            counts.into_iter().map(|(nama_ptn, count)| PtnDistributionItem { nama_ptn, count }).collect();
        out.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.nama_ptn.cmp(&b.nama_ptn)));
        out.truncate(6);
        Ok(out)
    }

    /// School Overview "Aktivitas Terkini" — the school's real most-recent submitted
    /// attempts across its whole roster, newest first. N+1 over the roster like the other
    /// methods here; acceptable at this scale (see module doc).
    pub async fn recent_activity(&self, actor: &AuthUser, limit: usize) -> AppResult<Vec<RecentActivityItem>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let students = self
            .users
            .list(UserFilter { role: Some(Role::Student), school_id: Some(school_id), ..Default::default() }, &actor.tenant)
            .await?;

        let mut out = Vec::new();
        for student in &students {
            let attempts = self.attempts.list_by_student(student.id, &actor.tenant).await?;
            for a in attempts {
                if a.status == AttemptStatus::Submitted {
                    if let Some(submitted_at) = a.submitted_at {
                        out.push(RecentActivityItem {
                            student_name: student.name.clone().unwrap_or_else(|| "(tanpa nama)".to_string()),
                            session_title: a.session_title,
                            session_type: a.session_type,
                            score: a.score,
                            submitted_at,
                        });
                    }
                }
            }
        }
        out.sort_by(|a, b| b.submitted_at.cmp(&a.submitted_at));
        out.truncate(limit);
        Ok(out)
    }

    /// Live "Pemantauan Pengerjaan" feed — every `InProgress` Attempt AND SimulationRun
    /// across the school's whole roster, oldest-started first (so a student who's been stuck
    /// the longest surfaces at the top, where the school PIC is most likely to need to step
    /// in). The frontend polls this on an interval rather than this method doing anything
    /// push-based — there is no realtime/websocket infrastructure anywhere in this codebase
    /// (see `domain::tryout::AttemptStatus::Abandoned` doc comment for the companion
    /// force-abandon half of this feature). N+1 over the roster, same acceptable-at-this-scale
    /// tradeoff as every other method in this file.
    pub async fn in_progress_activity(&self, actor: &AuthUser) -> AppResult<Vec<InProgressActivityItem>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let students = self
            .users
            .list(UserFilter { role: Some(Role::Student), school_id: Some(school_id), ..Default::default() }, &actor.tenant)
            .await?;

        let mut out = Vec::new();
        for student in &students {
            let student_name = student.name.clone().unwrap_or_else(|| "(tanpa nama)".to_string());

            let attempts = self.attempts.list_by_student(student.id, &actor.tenant).await?;
            for a in attempts {
                if a.status == AttemptStatus::InProgress {
                    out.push(InProgressActivityItem {
                        kind: InProgressActivityKind::Attempt,
                        ref_id: a.id,
                        student_id: student.id,
                        student_name: student_name.clone(),
                        title: a.session_title,
                        started_at: a.started_at,
                    });
                }
            }

            let runs = self.simulation_runs.list_by_student(student.id, &actor.tenant).await?;
            for r in runs {
                if r.status == SimulationRunStatus::InProgress {
                    out.push(InProgressActivityItem {
                        kind: InProgressActivityKind::SimulationRun,
                        ref_id: r.id,
                        student_id: student.id,
                        student_name: student_name.clone(),
                        title: r.template_title,
                        started_at: r.started_at,
                    });
                }
            }
        }
        out.sort_by(|a, b| a.started_at.cmp(&b.started_at));
        Ok(out)
    }

    /// Force-clears one student's stuck in-progress activity (plain Attempt OR SimulationRun
    /// — `kind` picks which) so they can start a fresh one. Unlike the student's own
    /// self-service `SimulationService::abandon_run`, this is School-initiated and requires
    /// ownership of the activity's student, not ownership of the activity itself. Does not
    /// touch the underlying `attempt_answers`/`simulation_run_slots` rows — those keep
    /// whatever partial progress they already have, only the parent's status changes.
    pub async fn force_abandon_activity(&self, actor: &AuthUser, kind: InProgressActivityKind, ref_id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;

        match kind {
            InProgressActivityKind::Attempt => {
                let attempt = self
                    .attempts
                    .find_by_id(ref_id, &actor.tenant)
                    .await?
                    .ok_or_else(|| AppError::NotFound(format!("attempt {ref_id} not found")))?;
                self.ensure_owns_student_id(school_id, attempt.student_id, &actor.tenant).await?;
                self.attempts
                    .force_abandon(ref_id, &actor.tenant)
                    .await?
                    .ok_or_else(|| AppError::Validation("attempt ini sudah tidak sedang berjalan".to_string()))?;
                Ok(())
            }
            InProgressActivityKind::SimulationRun => {
                let run = self
                    .simulation_runs
                    .find_by_id(ref_id, &actor.tenant)
                    .await?
                    .ok_or_else(|| AppError::NotFound(format!("run {ref_id} not found")))?;
                self.ensure_owns_student_id(school_id, run.student_id, &actor.tenant).await?;
                if run.status != SimulationRunStatus::InProgress {
                    return Err(AppError::Validation("simulasi ini sudah tidak sedang berjalan".to_string()));
                }
                self.simulation_runs
                    .set_run_status(ref_id, SimulationRunStatus::Abandoned, Some(Utc::now()), None, &actor.tenant)
                    .await
            }
        }
    }

    /// Same check as `ensure_owns_student`, but takes a resolved `school_id` directly — used by
    /// `force_abandon_activity`, which already resolved its own school_id once and would
    /// otherwise re-resolve it redundantly per call via `ensure_owns_student`.
    async fn ensure_owns_student_id(&self, school_id: Uuid, student_id: Uuid, ctx: &crate::domain::tenant::TenantCtx) -> AppResult<()> {
        let student = self
            .users
            .find_by_id(student_id, ctx)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("student {student_id} not found")))?;
        if student.role != Role::Student || student.school_id != Some(school_id) {
            return Err(AppError::Forbidden("kamu hanya bisa mengelola data siswa dari sekolahmu sendiri".to_string()));
        }
        Ok(())
    }

    /// Verifies the acting School PIC's own school actually owns `student_id` (i.e. the
    /// target is a Student account with `school_id` equal to the actor's own school).
    /// Used by other handlers (e.g. rationalization's school-scoped target/rapor views)
    /// before returning another student's data.
    pub async fn ensure_owns_student(&self, actor: &AuthUser, student_id: uuid::Uuid) -> AppResult<()> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        let student = self
            .users
            .find_by_id(student_id, &actor.tenant)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("student {student_id} not found")))?;
        if student.role != Role::Student || student.school_id != Some(school_id) {
            return Err(AppError::Forbidden("kamu hanya bisa melihat data siswa dari sekolahmu sendiri".to_string()));
        }
        Ok(())
    }

    async fn resolve_own_school_id(&self, actor: &AuthUser) -> AppResult<uuid::Uuid> {
        let me = self
            .users
            .find_by_id(actor.user_id, &actor.tenant)
            .await?
            .ok_or_else(|| AppError::Unauthorized("akun tidak ditemukan".to_string()))?;
        me.school_id
            .ok_or_else(|| AppError::Validation("akun sekolah ini belum terhubung ke data sekolah".to_string()))
    }
}
