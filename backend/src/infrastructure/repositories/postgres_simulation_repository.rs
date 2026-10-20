//! Postgres implementations for Simulasi UTBK — see `domain::simulation` doc comment and the
//! `20260101000005_ptn_gamification_and_simulation.sql` migration for the full rationale. Two
//! tightly-related repositories (template CRUD, run state machine storage) live in one file
//! since they share the same join shape (both read through `ptn.tryout_sessions` for the
//! display fields) and are always wired together by `SimulationService`.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::package::ExamTrack;
use crate::domain::repository::{
    LockdownViolationRepository, NewLockdownViolation, NewSimulationRun, NewSimulationTemplate, SimulationRunRepository,
    SimulationTemplateRepository, UpdateSimulationTemplate,
};
use crate::domain::school::SchoolType;
use crate::domain::simulation::{
    LockdownViolation, SimulationRun, SimulationRunSlot, SimulationRunStatus, SimulationSlotStatus,
    SimulationTemplate, SimulationTemplateKind, SimulationTemplateSlot, TemplateCompletionStats,
};
use crate::domain::tenant::TenantCtx;
use crate::error::{AppError, AppResult};
use std::str::FromStr;

async fn begin_tx(pool: &PgPool, ctx: &TenantCtx) -> AppResult<Transaction<'static, Postgres>> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true), set_config('app.is_platform_admin', $2, true)")
        .bind(ctx.tenant_id.to_string())
        .bind(if ctx.is_platform_admin { "true" } else { "false" })
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}

fn parse_school_type_scope(s: Option<&str>) -> AppResult<Option<SchoolType>> {
    s.map(SchoolType::from_str)
        .transpose()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt school_type_scope: {e}")))
}

fn school_type_scope_str(t: Option<SchoolType>) -> Option<&'static str> {
    t.map(|t| t.as_str())
}

fn template_kind_str(k: SimulationTemplateKind) -> &'static str {
    match k {
        SimulationTemplateKind::UtbkCombined => "utbk_combined",
        SimulationTemplateKind::PerSubject => "per_subject",
    }
}

fn parse_template_kind(s: &str) -> AppResult<SimulationTemplateKind> {
    match s {
        "utbk_combined" => Ok(SimulationTemplateKind::UtbkCombined),
        "per_subject" => Ok(SimulationTemplateKind::PerSubject),
        other => Err(AppError::Internal(anyhow::anyhow!("corrupt simulation template kind: {other}"))),
    }
}

fn exam_track_str(t: ExamTrack) -> &'static str {
    match t {
        ExamTrack::Snbt => "snbt",
        ExamTrack::Tka => "tka",
    }
}

fn parse_exam_track(s: &str) -> AppResult<ExamTrack> {
    match s {
        "snbt" => Ok(ExamTrack::Snbt),
        "tka" => Ok(ExamTrack::Tka),
        other => Err(AppError::Internal(anyhow::anyhow!("corrupt exam_track: {other}"))),
    }
}

fn run_status_str(s: SimulationRunStatus) -> &'static str {
    match s {
        SimulationRunStatus::InProgress => "in_progress",
        SimulationRunStatus::Completed => "completed",
        SimulationRunStatus::Abandoned => "abandoned",
    }
}

fn parse_run_status(s: &str) -> AppResult<SimulationRunStatus> {
    match s {
        "in_progress" => Ok(SimulationRunStatus::InProgress),
        "completed" => Ok(SimulationRunStatus::Completed),
        "abandoned" => Ok(SimulationRunStatus::Abandoned),
        other => Err(AppError::Internal(anyhow::anyhow!("corrupt simulation run status: {other}"))),
    }
}

fn slot_status_str(s: SimulationSlotStatus) -> &'static str {
    match s {
        SimulationSlotStatus::Pending => "pending",
        SimulationSlotStatus::Active => "active",
        SimulationSlotStatus::Submitted => "submitted",
    }
}

fn parse_slot_status(s: &str) -> AppResult<SimulationSlotStatus> {
    match s {
        "pending" => Ok(SimulationSlotStatus::Pending),
        "active" => Ok(SimulationSlotStatus::Active),
        "submitted" => Ok(SimulationSlotStatus::Submitted),
        other => Err(AppError::Internal(anyhow::anyhow!("corrupt simulation slot status: {other}"))),
    }
}

// ─── Templates ──────────────────────────────────────────────────────────────────

pub struct PostgresSimulationTemplateRepository {
    pool: PgPool,
}

impl PostgresSimulationTemplateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn slots_for_template(&self, template_id: i64, tx: &mut Transaction<'static, Postgres>) -> AppResult<Vec<SimulationTemplateSlot>> {
        // LEFT JOIN — an `is_elective` slot has `session_id IS NULL` in the template (no fixed
        // session to join against yet).
        let rows = sqlx::query_as::<_, TemplateSlotRow>(
            r#"SELECT ts.public_id, tt.public_id AS template_id, ts.sequence_index, s.public_id AS session_id,
                      s.title AS session_title, s.duration_minutes, s.question_count, s.subject_filter,
                      ts.break_seconds, ts.is_elective
               FROM ptn.simulation_template_slots ts
               JOIN ptn.simulation_templates tt ON tt.id = ts.template_id
               LEFT JOIN ptn.tryout_sessions s ON s.id = ts.session_id
               WHERE ts.template_id = $1
               ORDER BY ts.sequence_index ASC"#,
        )
        .bind(template_id)
        .fetch_all(&mut **tx)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn assemble(&self, row: TemplateRow, tx: &mut Transaction<'static, Postgres>) -> AppResult<SimulationTemplate> {
        let slots = self.slots_for_template(row.internal_id, tx).await?;
        let has_runs: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM ptn.simulation_runs WHERE template_id = $1)")
            .bind(row.internal_id)
            .fetch_one(&mut **tx)
            .await?;
        Ok(SimulationTemplate {
            id: row.public_id,
            title: row.title,
            is_active: row.is_active,
            is_premium: row.is_premium,
            is_draft: row.is_draft,
            elective_package_id: row.elective_package_id,
            template_kind: parse_template_kind(&row.template_kind)?,
            exam_track: parse_exam_track(&row.exam_track)?,
            school_type_scope: parse_school_type_scope(row.school_type_scope.as_deref())?,
            lockdown_override: row.lockdown_override,
            created_by: row.created_by,
            created_at: row.created_at,
            slots,
            has_runs,
        })
    }
}

#[derive(sqlx::FromRow)]
struct TemplateRow {
    #[sqlx(rename = "id")]
    internal_id: i64,
    public_id: Uuid,
    title: String,
    is_active: bool,
    is_premium: bool,
    is_draft: bool,
    elective_package_id: Option<Uuid>,
    template_kind: String,
    // `Option` because the scalar subquery can return NULL when `created_by` points at a
    // `core.akun` row the current RLS context can't see — see
    // `domain::simulation::SimulationTemplate::created_by` doc comment.
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    exam_track: String,
    school_type_scope: Option<String>,
    lockdown_override: Option<bool>,
}

// Shared column list — needs the internal `id` too (for the slots sub-query), alongside
// `public_id` and `elective_package_id`/`created_by` resolved to their own `public_id`s.
const TEMPLATE_COLUMNS: &str = r#"
    t.id, t.public_id, t.title, t.is_active, t.is_premium, t.is_draft,
    (SELECT public_id FROM ptn.packages WHERE ptn.packages.id = t.elective_package_id) AS elective_package_id,
    t.template_kind,
    (SELECT public_id FROM core.akun WHERE core.akun.id = t.created_by) AS created_by,
    t.created_at, t.exam_track, t.school_type_scope, t.lockdown_override
"#;

#[derive(sqlx::FromRow)]
struct TemplateSlotRow {
    public_id: Uuid,
    template_id: Uuid,
    sequence_index: i32,
    session_id: Option<Uuid>,
    session_title: Option<String>,
    duration_minutes: Option<i32>,
    question_count: Option<i32>,
    subject_filter: Option<String>,
    break_seconds: i32,
    is_elective: bool,
}

impl From<TemplateSlotRow> for SimulationTemplateSlot {
    fn from(r: TemplateSlotRow) -> Self {
        SimulationTemplateSlot {
            id: r.public_id,
            template_id: r.template_id,
            sequence_index: r.sequence_index,
            session_id: r.session_id,
            session_title: r.session_title,
            duration_minutes: r.duration_minutes,
            question_count: r.question_count,
            subject_filter: r.subject_filter,
            break_seconds: r.break_seconds,
            is_elective: r.is_elective,
        }
    }
}

#[async_trait]
impl SimulationTemplateRepository for PostgresSimulationTemplateRepository {
    async fn create(&self, input: NewSimulationTemplate, ctx: &TenantCtx) -> AppResult<SimulationTemplate> {
        let mut tx = begin_tx(&self.pool, ctx).await?;

        let created_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(input.created_by)
            .fetch_one(&mut *tx)
            .await?;
        let elective_package_id: Option<i64> = match input.elective_package_id {
            Some(pid) => Some(
                sqlx::query_scalar("SELECT id FROM ptn.packages WHERE public_id = $1")
                    .bind(pid)
                    .fetch_one(&mut *tx)
                    .await?,
            ),
            None => None,
        };

        let template_id: i64 = sqlx::query_scalar(
            r#"INSERT INTO ptn.simulation_templates
                (tenant_id, title, is_premium, created_by, is_draft, elective_package_id, template_kind,
                 exam_track, school_type_scope, lockdown_override)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING id"#,
        )
        .bind(ctx.tenant_id)
        .bind(&input.title)
        .bind(input.is_premium)
        .bind(created_by_id)
        .bind(input.is_draft)
        .bind(elective_package_id)
        .bind(template_kind_str(input.template_kind))
        .bind(exam_track_str(input.exam_track))
        .bind(school_type_scope_str(input.school_type_scope))
        .bind(input.lockdown_override)
        .fetch_one(&mut *tx)
        .await?;

        for slot in &input.slots {
            let session_id: Option<i64> = match slot.session_id {
                Some(sid) => Some(
                    sqlx::query_scalar("SELECT id FROM ptn.tryout_sessions WHERE public_id = $1")
                        .bind(sid)
                        .fetch_one(&mut *tx)
                        .await?,
                ),
                None => None,
            };
            sqlx::query(
                r#"INSERT INTO ptn.simulation_template_slots (template_id, sequence_index, session_id, break_seconds, is_elective)
                   VALUES ($1, $2, $3, $4, $5)"#,
            )
            .bind(template_id)
            .bind(slot.sequence_index)
            .bind(session_id)
            .bind(slot.break_seconds)
            .bind(slot.is_elective)
            .execute(&mut *tx)
            .await?;
        }

        let row = sqlx::query_as::<_, TemplateRow>(&format!("SELECT {TEMPLATE_COLUMNS} FROM ptn.simulation_templates t WHERE t.id = $1"))
            .bind(template_id)
            .fetch_one(&mut *tx)
            .await?;
        let result = self.assemble(row, &mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<SimulationTemplate>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let row = sqlx::query_as::<_, TemplateRow>(&format!(
            "SELECT {TEMPLATE_COLUMNS} FROM ptn.simulation_templates t WHERE t.public_id = $1"
        ))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
        match row {
            Some(r) => {
                let result = self.assemble(r, &mut tx).await?;
                tx.commit().await?;
                Ok(Some(result))
            }
            None => {
                tx.commit().await?;
                Ok(None)
            }
        }
    }

    async fn list(&self, only_active: bool, ctx: &TenantCtx) -> AppResult<Vec<SimulationTemplate>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = if only_active {
            sqlx::query_as::<_, TemplateRow>(&format!(
                "SELECT {TEMPLATE_COLUMNS} FROM ptn.simulation_templates t WHERE t.is_active = true ORDER BY t.created_at DESC"
            ))
            .fetch_all(&mut *tx)
            .await?
        } else {
            sqlx::query_as::<_, TemplateRow>(&format!(
                "SELECT {TEMPLATE_COLUMNS} FROM ptn.simulation_templates t ORDER BY t.created_at DESC"
            ))
            .fetch_all(&mut *tx)
            .await?
        };
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(self.assemble(row, &mut tx).await?);
        }
        tx.commit().await?;
        Ok(out)
    }

    async fn set_active(&self, id: Uuid, active: bool, ctx: &TenantCtx) -> AppResult<Option<SimulationTemplate>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let updated = sqlx::query("UPDATE ptn.simulation_templates SET is_active = $2 WHERE public_id = $1")
            .bind(id)
            .bind(active)
            .execute(&mut *tx)
            .await?;
        if updated.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, TemplateRow>(&format!("SELECT {TEMPLATE_COLUMNS} FROM ptn.simulation_templates t WHERE t.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let result = self.assemble(row, &mut tx).await?;
        tx.commit().await?;
        Ok(Some(result))
    }

    async fn set_premium(&self, id: Uuid, is_premium: bool, ctx: &TenantCtx) -> AppResult<Option<SimulationTemplate>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let updated = sqlx::query("UPDATE ptn.simulation_templates SET is_premium = $2 WHERE public_id = $1")
            .bind(id)
            .bind(is_premium)
            .execute(&mut *tx)
            .await?;
        if updated.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, TemplateRow>(&format!("SELECT {TEMPLATE_COLUMNS} FROM ptn.simulation_templates t WHERE t.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let result = self.assemble(row, &mut tx).await?;
        tx.commit().await?;
        Ok(Some(result))
    }

    async fn set_draft(&self, id: Uuid, is_draft: bool, ctx: &TenantCtx) -> AppResult<Option<SimulationTemplate>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let updated = sqlx::query("UPDATE ptn.simulation_templates SET is_draft = $2 WHERE public_id = $1")
            .bind(id)
            .bind(is_draft)
            .execute(&mut *tx)
            .await?;
        if updated.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, TemplateRow>(&format!("SELECT {TEMPLATE_COLUMNS} FROM ptn.simulation_templates t WHERE t.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let result = self.assemble(row, &mut tx).await?;
        tx.commit().await?;
        Ok(Some(result))
    }

    async fn set_lockdown(&self, id: Uuid, lockdown_override: Option<bool>, ctx: &TenantCtx) -> AppResult<Option<SimulationTemplate>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let updated = sqlx::query("UPDATE ptn.simulation_templates SET lockdown_override = $2 WHERE public_id = $1")
            .bind(id)
            .bind(lockdown_override)
            .execute(&mut *tx)
            .await?;
        if updated.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, TemplateRow>(&format!("SELECT {TEMPLATE_COLUMNS} FROM ptn.simulation_templates t WHERE t.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let result = self.assemble(row, &mut tx).await?;
        tx.commit().await?;
        Ok(Some(result))
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query("DELETE FROM ptn.simulation_templates WHERE public_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                AppError::from_sqlx_delete(
                    e,
                    "cannot delete this template: students already have simulation runs recorded against it",
                )
            })?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn has_any_run(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let exists: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(
                 SELECT 1 FROM ptn.simulation_runs r
                 JOIN ptn.simulation_templates t ON t.id = r.template_id
                 WHERE t.public_id = $1
               )"#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(exists)
    }

    async fn update(&self, id: Uuid, input: UpdateSimulationTemplate, ctx: &TenantCtx) -> AppResult<Option<SimulationTemplate>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;

        let template_id: Option<i64> = sqlx::query_scalar("SELECT id FROM ptn.simulation_templates WHERE public_id = $1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        let Some(template_id) = template_id else {
            tx.rollback().await.ok();
            return Ok(None);
        };

        let elective_package_id: Option<i64> = match input.elective_package_id {
            Some(pid) => Some(
                sqlx::query_scalar("SELECT id FROM ptn.packages WHERE public_id = $1")
                    .bind(pid)
                    .fetch_one(&mut *tx)
                    .await?,
            ),
            None => None,
        };

        sqlx::query(
            r#"UPDATE ptn.simulation_templates SET
                 title = $2, is_premium = $3, elective_package_id = $4, template_kind = $5,
                 exam_track = $6, school_type_scope = $7, lockdown_override = $8
               WHERE id = $1"#,
        )
        .bind(template_id)
        .bind(&input.title)
        .bind(input.is_premium)
        .bind(elective_package_id)
        .bind(template_kind_str(input.template_kind))
        .bind(exam_track_str(input.exam_track))
        .bind(school_type_scope_str(input.school_type_scope))
        .bind(input.lockdown_override)
        .execute(&mut *tx)
        .await?;

        // Only reached once the service layer has confirmed (via `has_any_run`) that no
        // student run references this template yet — see `UpdateSimulationTemplate` doc
        // comment. Replace-all is safe here precisely because nothing downstream points at
        // the old slot rows.
        if let Some(slots) = input.slots {
            sqlx::query("DELETE FROM ptn.simulation_template_slots WHERE template_id = $1")
                .bind(template_id)
                .execute(&mut *tx)
                .await?;
            for slot in &slots {
                let session_id: Option<i64> = match slot.session_id {
                    Some(sid) => Some(
                        sqlx::query_scalar("SELECT id FROM ptn.tryout_sessions WHERE public_id = $1")
                            .bind(sid)
                            .fetch_one(&mut *tx)
                            .await?,
                    ),
                    None => None,
                };
                sqlx::query(
                    r#"INSERT INTO ptn.simulation_template_slots (template_id, sequence_index, session_id, break_seconds, is_elective)
                       VALUES ($1, $2, $3, $4, $5)"#,
                )
                .bind(template_id)
                .bind(slot.sequence_index)
                .bind(session_id)
                .bind(slot.break_seconds)
                .bind(slot.is_elective)
                .execute(&mut *tx)
                .await?;
            }
        }

        let row = sqlx::query_as::<_, TemplateRow>(&format!("SELECT {TEMPLATE_COLUMNS} FROM ptn.simulation_templates t WHERE t.id = $1"))
            .bind(template_id)
            .fetch_one(&mut *tx)
            .await?;
        let result = self.assemble(row, &mut tx).await?;
        tx.commit().await?;
        Ok(Some(result))
    }
}

// ─── Runs ───────────────────────────────────────────────────────────────────────

pub struct PostgresSimulationRunRepository {
    pool: PgPool,
}

impl PostgresSimulationRunRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn slots_for_run(&self, run_id: i64, tx: &mut Transaction<'static, Postgres>) -> AppResult<Vec<SimulationRunSlot>> {
        let rows = sqlx::query_as::<_, RunSlotRow>(
            r#"SELECT rs.public_id, r.public_id AS run_id, rs.sequence_index, s.public_id AS session_id,
                      s.title AS session_title, s.duration_minutes, s.subject_filter, rs.break_seconds,
                      a.public_id AS attempt_id,
                      a.score AS attempt_score, a.accuracy AS attempt_accuracy,
                      a.correct_count AS attempt_correct_count, a.wrong_count AS attempt_wrong_count,
                      a.unanswered_count AS attempt_unanswered_count,
                      rs.status, rs.deadline_at, rs.break_ends_at
               FROM ptn.simulation_run_slots rs
               JOIN ptn.simulation_runs r ON r.id = rs.run_id
               JOIN ptn.tryout_sessions s ON s.id = rs.session_id
               LEFT JOIN ptn.attempts a ON a.id = rs.attempt_id
               WHERE rs.run_id = $1
               ORDER BY rs.sequence_index ASC"#,
        )
        .bind(run_id)
        .fetch_all(&mut **tx)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn assemble(&self, row: RunRow, tx: &mut Transaction<'static, Postgres>) -> AppResult<SimulationRun> {
        let slots = self.slots_for_run(row.internal_id, tx).await?;
        Ok(SimulationRun {
            id: row.public_id,
            template_id: row.template_id,
            template_title: row.template_title,
            template_kind: parse_template_kind(&row.template_kind)?,
            school_type_scope: parse_school_type_scope(row.school_type_scope.as_deref())?,
            student_id: row.student_id,
            status: parse_run_status(&row.status)?,
            current_sequence_index: row.current_sequence_index,
            combined_estimate_score: row.combined_estimate_score,
            started_at: row.started_at,
            completed_at: row.completed_at,
            lockdown_enabled: row.lockdown_enabled,
            slots,
        })
    }
}

const SELECT_RUN: &str = r#"
    SELECT r.id, r.public_id, t.public_id AS template_id, t.title AS template_title, t.template_kind,
           t.school_type_scope,
           (SELECT public_id FROM core.akun WHERE core.akun.id = r.student_id) AS student_id,
           r.status, r.current_sequence_index, r.combined_estimate_score, r.started_at, r.completed_at,
           r.lockdown_enabled
    FROM ptn.simulation_runs r
    JOIN ptn.simulation_templates t ON t.id = r.template_id
"#;

#[derive(sqlx::FromRow)]
struct RunRow {
    #[sqlx(rename = "id")]
    internal_id: i64,
    public_id: Uuid,
    template_id: Uuid,
    template_title: String,
    template_kind: String,
    school_type_scope: Option<String>,
    student_id: Uuid,
    status: String,
    current_sequence_index: i32,
    combined_estimate_score: Option<f64>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    lockdown_enabled: bool,
}

#[derive(sqlx::FromRow)]
struct RunSlotRow {
    public_id: Uuid,
    run_id: Uuid,
    sequence_index: i32,
    session_id: Uuid,
    session_title: String,
    duration_minutes: i32,
    subject_filter: Option<String>,
    break_seconds: i32,
    attempt_id: Option<Uuid>,
    attempt_score: Option<i32>,
    attempt_accuracy: Option<f64>,
    attempt_correct_count: Option<i32>,
    attempt_wrong_count: Option<i32>,
    attempt_unanswered_count: Option<i32>,
    status: String,
    deadline_at: Option<DateTime<Utc>>,
    break_ends_at: Option<DateTime<Utc>>,
}

impl TryFrom<RunSlotRow> for SimulationRunSlot {
    type Error = AppError;
    fn try_from(r: RunSlotRow) -> Result<Self, Self::Error> {
        Ok(SimulationRunSlot {
            id: r.public_id,
            run_id: r.run_id,
            sequence_index: r.sequence_index,
            session_id: r.session_id,
            session_title: r.session_title,
            duration_minutes: r.duration_minutes,
            subject_filter: r.subject_filter,
            break_seconds: r.break_seconds,
            attempt_id: r.attempt_id,
            attempt_score: r.attempt_score,
            attempt_accuracy: r.attempt_accuracy,
            attempt_correct_count: r.attempt_correct_count,
            attempt_wrong_count: r.attempt_wrong_count,
            attempt_unanswered_count: r.attempt_unanswered_count,
            status: parse_slot_status(&r.status)?,
            deadline_at: r.deadline_at,
            break_ends_at: r.break_ends_at,
        })
    }
}

#[async_trait]
impl SimulationRunRepository for PostgresSimulationRunRepository {
    async fn create(&self, input: NewSimulationRun, ctx: &TenantCtx) -> AppResult<SimulationRun> {
        let mut tx = begin_tx(&self.pool, ctx).await?;

        let template_id: i64 = sqlx::query_scalar("SELECT id FROM ptn.simulation_templates WHERE public_id = $1")
            .bind(input.template_id)
            .fetch_one(&mut *tx)
            .await?;
        let student_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(input.student_id)
            .fetch_one(&mut *tx)
            .await?;

        let run_id: i64 = sqlx::query_scalar(
            r#"INSERT INTO ptn.simulation_runs (tenant_id, template_id, student_id, lockdown_enabled)
               VALUES ($1, $2, $3, $4) RETURNING id"#,
        )
        .bind(ctx.tenant_id)
        .bind(template_id)
        .bind(student_id)
        .bind(input.lockdown_enabled)
        .fetch_one(&mut *tx)
        .await?;

        for slot in &input.slots {
            let session_id: i64 = sqlx::query_scalar("SELECT id FROM ptn.tryout_sessions WHERE public_id = $1")
                .bind(slot.session_id)
                .fetch_one(&mut *tx)
                .await?;
            sqlx::query(
                r#"INSERT INTO ptn.simulation_run_slots (run_id, sequence_index, session_id, break_seconds)
                   VALUES ($1, $2, $3, $4)"#,
            )
            .bind(run_id)
            .bind(slot.sequence_index)
            .bind(session_id)
            .bind(slot.break_seconds)
            .execute(&mut *tx)
            .await?;
        }

        let row = sqlx::query_as::<_, RunRow>(&format!("{SELECT_RUN} WHERE r.id = $1"))
            .bind(run_id)
            .fetch_one(&mut *tx)
            .await?;
        let result = self.assemble(row, &mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<SimulationRun>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let row = sqlx::query_as::<_, RunRow>(&format!("{SELECT_RUN} WHERE r.public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        match row {
            Some(r) => {
                let result = self.assemble(r, &mut tx).await?;
                tx.commit().await?;
                Ok(Some(result))
            }
            None => {
                tx.commit().await?;
                Ok(None)
            }
        }
    }

    async fn list_by_student(&self, student_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<SimulationRun>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, RunRow>(&format!(
            "{SELECT_RUN} WHERE r.student_id = (SELECT id FROM core.akun WHERE public_id = $1) ORDER BY r.started_at DESC"
        ))
        .bind(student_id)
        .fetch_all(&mut *tx)
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(self.assemble(row, &mut tx).await?);
        }
        tx.commit().await?;
        Ok(out)
    }

    async fn latest_completed_for_student(&self, student_id: Uuid, ctx: &TenantCtx) -> AppResult<Option<SimulationRun>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let row = sqlx::query_as::<_, RunRow>(&format!(
            "{SELECT_RUN} WHERE r.student_id = (SELECT id FROM core.akun WHERE public_id = $1) AND r.status = 'completed'
             ORDER BY r.completed_at DESC LIMIT 1"
        ))
        .bind(student_id)
        .fetch_optional(&mut *tx)
        .await?;
        match row {
            Some(r) => {
                let result = self.assemble(r, &mut tx).await?;
                tx.commit().await?;
                Ok(Some(result))
            }
            None => {
                tx.commit().await?;
                Ok(None)
            }
        }
    }

    async fn set_run_status(
        &self,
        run_id: Uuid,
        status: SimulationRunStatus,
        completed_at: Option<DateTime<Utc>>,
        combined_estimate_score: Option<f64>,
        ctx: &TenantCtx,
    ) -> AppResult<()> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        sqlx::query(
            r#"UPDATE ptn.simulation_runs SET status = $2, completed_at = $3, combined_estimate_score = $4
               WHERE public_id = $1"#,
        )
        .bind(run_id)
        .bind(run_status_str(status))
        .bind(completed_at)
        .bind(combined_estimate_score)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn advance_current_index(&self, run_id: Uuid, new_index: i32, ctx: &TenantCtx) -> AppResult<()> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        sqlx::query("UPDATE ptn.simulation_runs SET current_sequence_index = $2 WHERE public_id = $1")
            .bind(run_id)
            .bind(new_index)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn activate_slot(&self, run_id: Uuid, sequence_index: i32, attempt_id: Uuid, deadline_at: DateTime<Utc>, ctx: &TenantCtx) -> AppResult<()> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let attempt_pk: i64 = sqlx::query_scalar("SELECT id FROM ptn.attempts WHERE public_id = $1")
            .bind(attempt_id)
            .fetch_one(&mut *tx)
            .await?;
        sqlx::query(
            r#"UPDATE ptn.simulation_run_slots SET status = $3, attempt_id = $4, deadline_at = $5
               WHERE run_id = (SELECT id FROM ptn.simulation_runs WHERE public_id = $1) AND sequence_index = $2"#,
        )
        .bind(run_id)
        .bind(sequence_index)
        .bind(slot_status_str(SimulationSlotStatus::Active))
        .bind(attempt_pk)
        .bind(deadline_at)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn submit_slot(&self, run_id: Uuid, sequence_index: i32, break_ends_at: Option<DateTime<Utc>>, ctx: &TenantCtx) -> AppResult<()> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        sqlx::query(
            r#"UPDATE ptn.simulation_run_slots SET status = $3, break_ends_at = $4
               WHERE run_id = (SELECT id FROM ptn.simulation_runs WHERE public_id = $1) AND sequence_index = $2"#,
        )
        .bind(run_id)
        .bind(sequence_index)
        .bind(slot_status_str(SimulationSlotStatus::Submitted))
        .bind(break_ends_at)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn template_completion_stats(&self, template_id: Uuid, ctx: &TenantCtx) -> AppResult<TemplateCompletionStats> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let row: (i64, Option<f64>) = sqlx::query_as(
            r#"SELECT COUNT(*), AVG(combined_estimate_score)
               FROM ptn.simulation_runs
               WHERE template_id = (SELECT id FROM ptn.simulation_templates WHERE public_id = $1) AND status = 'completed'"#,
        )
        .bind(template_id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(TemplateCompletionStats { template_id, participant_count: row.0, avg_combined_score: row.1 })
    }
}

// ─── Lockdown violations ──────────────────────────────────────────────────────────

pub struct PostgresLockdownViolationRepository {
    pool: PgPool,
}

impl PostgresLockdownViolationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct LockdownViolationRow {
    public_id: Uuid,
    run_id: Uuid,
    student_id: Uuid,
    event_type: String,
    detail: Option<String>,
    occurred_at: DateTime<Utc>,
}

impl From<LockdownViolationRow> for LockdownViolation {
    fn from(r: LockdownViolationRow) -> Self {
        LockdownViolation {
            id: r.public_id,
            run_id: r.run_id,
            student_id: r.student_id,
            event_type: r.event_type,
            detail: r.detail,
            occurred_at: r.occurred_at,
        }
    }
}

const SELECT_VIOLATION: &str = r#"
    SELECT v.public_id, r.public_id AS run_id,
           (SELECT public_id FROM core.akun WHERE core.akun.id = v.student_id) AS student_id,
           v.event_type, v.detail, v.occurred_at
    FROM ptn.simulation_lockdown_violations v
    JOIN ptn.simulation_runs r ON r.id = v.run_id
"#;

#[async_trait]
impl LockdownViolationRepository for PostgresLockdownViolationRepository {
    async fn record(&self, input: NewLockdownViolation, ctx: &TenantCtx) -> AppResult<LockdownViolation> {
        let mut tx = begin_tx(&self.pool, ctx).await?;

        let run_id: i64 = sqlx::query_scalar("SELECT id FROM ptn.simulation_runs WHERE public_id = $1")
            .bind(input.run_id)
            .fetch_one(&mut *tx)
            .await?;
        let student_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(input.student_id)
            .fetch_one(&mut *tx)
            .await?;

        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.simulation_lockdown_violations (run_id, student_id, event_type, detail)
               VALUES ($1, $2, $3, $4)
               RETURNING public_id"#,
        )
        .bind(run_id)
        .bind(student_id)
        .bind(&input.event_type)
        .bind(&input.detail)
        .fetch_one(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, LockdownViolationRow>(&format!("{SELECT_VIOLATION} WHERE v.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(row.into())
    }

    async fn list_by_run(&self, run_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<LockdownViolation>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, LockdownViolationRow>(&format!(
            "{SELECT_VIOLATION} WHERE r.public_id = $1 ORDER BY v.occurred_at DESC"
        ))
        .bind(run_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
