//! Postgres implementations for Simulasi UTBK — see `domain::simulation` doc comment and the
//! `20250101000025_simulation.sql` migration for the full rationale. Two tightly-related
//! repositories (template CRUD, run state machine storage) live in one file since they share
//! the same join shape (both read through `tryout_sessions` for the display fields) and are
//! always wired together by `SimulationService`.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::package::ExamTrack;
use crate::domain::repository::{
    LockdownViolationRepository, NewLockdownViolation, NewSimulationRun, NewSimulationTemplate,
    SimulationRunRepository, SimulationTemplateRepository,
};
use crate::domain::school::SchoolType;
use crate::domain::simulation::{
    LockdownViolation, SimulationRun, SimulationRunSlot, SimulationRunStatus, SimulationSlotStatus,
    SimulationTemplate, SimulationTemplateKind, SimulationTemplateSlot, TemplateCompletionStats,
};
use crate::error::{AppError, AppResult};
use std::str::FromStr;

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

    async fn slots_for_template(&self, template_id: Uuid) -> AppResult<Vec<SimulationTemplateSlot>> {
        // LEFT JOIN — an `is_elective` slot has `session_id IS NULL` in the template (no fixed
        // session to join against yet; see `20250101000034_simulation_elective_slots.sql`).
        let rows = sqlx::query_as::<_, TemplateSlotRow>(
            r#"SELECT ts.id, ts.template_id, ts.sequence_index, ts.session_id, s.title AS session_title,
                      s.duration_minutes, s.question_count, s.subject_filter, ts.break_seconds, ts.is_elective
               FROM simulation_template_slots ts
               LEFT JOIN tryout_sessions s ON s.id = ts.session_id
               WHERE ts.template_id = $1
               ORDER BY ts.sequence_index ASC"#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn assemble(&self, row: TemplateRow) -> AppResult<SimulationTemplate> {
        let slots = self.slots_for_template(row.id).await?;
        Ok(SimulationTemplate {
            id: row.id,
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
        })
    }
}

#[derive(sqlx::FromRow)]
struct TemplateRow {
    id: Uuid,
    title: String,
    is_active: bool,
    is_premium: bool,
    is_draft: bool,
    elective_package_id: Option<Uuid>,
    template_kind: String,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    exam_track: String,
    school_type_scope: Option<String>,
    lockdown_override: Option<bool>,
}

// Shared column list for plain SELECTs and UPDATE ... RETURNING clauses.
const TEMPLATE_COLUMNS: &str =
    "id, title, is_active, is_premium, is_draft, elective_package_id, template_kind, created_by, created_at, exam_track, school_type_scope, lockdown_override";

#[derive(sqlx::FromRow)]
struct TemplateSlotRow {
    id: Uuid,
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
            id: r.id,
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
    async fn create(&self, input: NewSimulationTemplate) -> AppResult<SimulationTemplate> {
        let id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"INSERT INTO simulation_templates (id, title, is_premium, created_by, is_draft, elective_package_id, template_kind, exam_track, school_type_scope, lockdown_override)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
        )
        .bind(id)
        .bind(&input.title)
        .bind(input.is_premium)
        .bind(input.created_by)
        .bind(input.is_draft)
        .bind(input.elective_package_id)
        .bind(template_kind_str(input.template_kind))
        .bind(exam_track_str(input.exam_track))
        .bind(school_type_scope_str(input.school_type_scope))
        .bind(input.lockdown_override)
        .execute(&mut *tx)
        .await?;

        for slot in &input.slots {
            sqlx::query(
                r#"INSERT INTO simulation_template_slots (id, template_id, sequence_index, session_id, break_seconds, is_elective)
                   VALUES ($1, $2, $3, $4, $5, $6)"#,
            )
            .bind(Uuid::new_v4())
            .bind(id)
            .bind(slot.sequence_index)
            .bind(slot.session_id)
            .bind(slot.break_seconds)
            .bind(slot.is_elective)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("simulation template vanished right after insert")))
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SimulationTemplate>> {
        let row = sqlx::query_as::<_, TemplateRow>(&format!(
            "SELECT {TEMPLATE_COLUMNS} FROM simulation_templates WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(self.assemble(r).await?)),
            None => Ok(None),
        }
    }

    async fn list(&self, only_active: bool) -> AppResult<Vec<SimulationTemplate>> {
        let rows = if only_active {
            sqlx::query_as::<_, TemplateRow>(&format!(
                "SELECT {TEMPLATE_COLUMNS} FROM simulation_templates
                 WHERE is_active = true ORDER BY created_at DESC"
            ))
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, TemplateRow>(&format!(
                "SELECT {TEMPLATE_COLUMNS} FROM simulation_templates ORDER BY created_at DESC"
            ))
            .fetch_all(&self.pool)
            .await?
        };
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(self.assemble(row).await?);
        }
        Ok(out)
    }

    async fn set_active(&self, id: Uuid, active: bool) -> AppResult<Option<SimulationTemplate>> {
        let row = sqlx::query_as::<_, TemplateRow>(&format!(
            "UPDATE simulation_templates SET is_active = $2 WHERE id = $1 RETURNING {TEMPLATE_COLUMNS}"
        ))
        .bind(id)
        .bind(active)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(self.assemble(r).await?)),
            None => Ok(None),
        }
    }

    async fn set_premium(&self, id: Uuid, is_premium: bool) -> AppResult<Option<SimulationTemplate>> {
        let row = sqlx::query_as::<_, TemplateRow>(&format!(
            "UPDATE simulation_templates SET is_premium = $2 WHERE id = $1 RETURNING {TEMPLATE_COLUMNS}"
        ))
        .bind(id)
        .bind(is_premium)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(self.assemble(r).await?)),
            None => Ok(None),
        }
    }

    async fn set_draft(&self, id: Uuid, is_draft: bool) -> AppResult<Option<SimulationTemplate>> {
        let row = sqlx::query_as::<_, TemplateRow>(&format!(
            "UPDATE simulation_templates SET is_draft = $2 WHERE id = $1 RETURNING {TEMPLATE_COLUMNS}"
        ))
        .bind(id)
        .bind(is_draft)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(self.assemble(r).await?)),
            None => Ok(None),
        }
    }

    async fn set_lockdown(&self, id: Uuid, lockdown_override: Option<bool>) -> AppResult<Option<SimulationTemplate>> {
        let row = sqlx::query_as::<_, TemplateRow>(&format!(
            "UPDATE simulation_templates SET lockdown_override = $2 WHERE id = $1 RETURNING {TEMPLATE_COLUMNS}"
        ))
        .bind(id)
        .bind(lockdown_override)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(self.assemble(r).await?)),
            None => Ok(None),
        }
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM simulation_templates WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::from_sqlx_delete(
                    e,
                    "cannot delete this template: students already have simulation runs recorded against it",
                )
            })?;
        Ok(result.rows_affected() > 0)
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

    async fn slots_for_run(&self, run_id: Uuid) -> AppResult<Vec<SimulationRunSlot>> {
        let rows = sqlx::query_as::<_, RunSlotRow>(
            r#"SELECT rs.id, rs.run_id, rs.sequence_index, rs.session_id, s.title AS session_title,
                      s.duration_minutes, s.subject_filter, rs.break_seconds, rs.attempt_id,
                      a.score AS attempt_score, a.accuracy AS attempt_accuracy,
                      a.correct_count AS attempt_correct_count, a.wrong_count AS attempt_wrong_count,
                      a.unanswered_count AS attempt_unanswered_count,
                      rs.status, rs.deadline_at, rs.break_ends_at
               FROM simulation_run_slots rs
               JOIN tryout_sessions s ON s.id = rs.session_id
               LEFT JOIN attempts a ON a.id = rs.attempt_id
               WHERE rs.run_id = $1
               ORDER BY rs.sequence_index ASC"#,
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn assemble(&self, row: RunRow) -> AppResult<SimulationRun> {
        let slots = self.slots_for_run(row.id).await?;
        Ok(SimulationRun {
            id: row.id,
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
    SELECT r.id, r.template_id, t.title AS template_title, t.template_kind, t.school_type_scope,
           r.student_id, r.status,
           r.current_sequence_index, r.combined_estimate_score, r.started_at, r.completed_at,
           r.lockdown_enabled
    FROM simulation_runs r
    JOIN simulation_templates t ON t.id = r.template_id
"#;

#[derive(sqlx::FromRow)]
struct RunRow {
    id: Uuid,
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
    id: Uuid,
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
            id: r.id,
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
    async fn create(&self, input: NewSimulationRun) -> AppResult<SimulationRun> {
        let id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"INSERT INTO simulation_runs (id, template_id, student_id, lockdown_enabled) VALUES ($1, $2, $3, $4)"#,
        )
        .bind(id)
        .bind(input.template_id)
        .bind(input.student_id)
        .bind(input.lockdown_enabled)
        .execute(&mut *tx)
        .await?;

        for slot in &input.slots {
            sqlx::query(
                r#"INSERT INTO simulation_run_slots (id, run_id, sequence_index, session_id, break_seconds)
                   VALUES ($1, $2, $3, $4, $5)"#,
            )
            .bind(Uuid::new_v4())
            .bind(id)
            .bind(slot.sequence_index)
            .bind(slot.session_id)
            .bind(slot.break_seconds)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("simulation run vanished right after insert")))
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SimulationRun>> {
        let row = sqlx::query_as::<_, RunRow>(&format!("{SELECT_RUN} WHERE r.id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(r) => Ok(Some(self.assemble(r).await?)),
            None => Ok(None),
        }
    }

    async fn list_by_student(&self, student_id: Uuid) -> AppResult<Vec<SimulationRun>> {
        let rows = sqlx::query_as::<_, RunRow>(&format!(
            "{SELECT_RUN} WHERE r.student_id = $1 ORDER BY r.started_at DESC"
        ))
        .bind(student_id)
        .fetch_all(&self.pool)
        .await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(self.assemble(row).await?);
        }
        Ok(out)
    }

    async fn latest_completed_for_student(&self, student_id: Uuid) -> AppResult<Option<SimulationRun>> {
        let row = sqlx::query_as::<_, RunRow>(&format!(
            "{SELECT_RUN} WHERE r.student_id = $1 AND r.status = 'completed'
             ORDER BY r.completed_at DESC LIMIT 1"
        ))
        .bind(student_id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(self.assemble(r).await?)),
            None => Ok(None),
        }
    }

    async fn set_run_status(
        &self,
        run_id: Uuid,
        status: SimulationRunStatus,
        completed_at: Option<DateTime<Utc>>,
        combined_estimate_score: Option<f64>,
    ) -> AppResult<()> {
        sqlx::query(
            r#"UPDATE simulation_runs SET status = $2, completed_at = $3, combined_estimate_score = $4
               WHERE id = $1"#,
        )
        .bind(run_id)
        .bind(run_status_str(status))
        .bind(completed_at)
        .bind(combined_estimate_score)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn advance_current_index(&self, run_id: Uuid, new_index: i32) -> AppResult<()> {
        sqlx::query("UPDATE simulation_runs SET current_sequence_index = $2 WHERE id = $1")
            .bind(run_id)
            .bind(new_index)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn activate_slot(&self, run_id: Uuid, sequence_index: i32, attempt_id: Uuid, deadline_at: DateTime<Utc>) -> AppResult<()> {
        sqlx::query(
            r#"UPDATE simulation_run_slots SET status = $3, attempt_id = $4, deadline_at = $5
               WHERE run_id = $1 AND sequence_index = $2"#,
        )
        .bind(run_id)
        .bind(sequence_index)
        .bind(slot_status_str(SimulationSlotStatus::Active))
        .bind(attempt_id)
        .bind(deadline_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn submit_slot(&self, run_id: Uuid, sequence_index: i32, break_ends_at: Option<DateTime<Utc>>) -> AppResult<()> {
        sqlx::query(
            r#"UPDATE simulation_run_slots SET status = $3, break_ends_at = $4
               WHERE run_id = $1 AND sequence_index = $2"#,
        )
        .bind(run_id)
        .bind(sequence_index)
        .bind(slot_status_str(SimulationSlotStatus::Submitted))
        .bind(break_ends_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn template_completion_stats(&self, template_id: Uuid) -> AppResult<TemplateCompletionStats> {
        let row: (i64, Option<f64>) = sqlx::query_as(
            r#"SELECT COUNT(*), AVG(combined_estimate_score)
               FROM simulation_runs
               WHERE template_id = $1 AND status = 'completed'"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;
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
    id: Uuid,
    run_id: Uuid,
    student_id: Uuid,
    event_type: String,
    detail: Option<String>,
    occurred_at: DateTime<Utc>,
}

impl From<LockdownViolationRow> for LockdownViolation {
    fn from(r: LockdownViolationRow) -> Self {
        LockdownViolation {
            id: r.id,
            run_id: r.run_id,
            student_id: r.student_id,
            event_type: r.event_type,
            detail: r.detail,
            occurred_at: r.occurred_at,
        }
    }
}

#[async_trait]
impl LockdownViolationRepository for PostgresLockdownViolationRepository {
    async fn record(&self, input: NewLockdownViolation) -> AppResult<LockdownViolation> {
        let row = sqlx::query_as::<_, LockdownViolationRow>(
            r#"INSERT INTO simulation_lockdown_violations (id, run_id, student_id, event_type, detail)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id, run_id, student_id, event_type, detail, occurred_at"#,
        )
        .bind(Uuid::new_v4())
        .bind(input.run_id)
        .bind(input.student_id)
        .bind(&input.event_type)
        .bind(&input.detail)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn list_by_run(&self, run_id: Uuid) -> AppResult<Vec<LockdownViolation>> {
        let rows = sqlx::query_as::<_, LockdownViolationRow>(
            r#"SELECT id, run_id, student_id, event_type, detail, occurred_at
               FROM simulation_lockdown_violations
               WHERE run_id = $1
               ORDER BY occurred_at DESC"#,
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
