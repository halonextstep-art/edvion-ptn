use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::package::ExamTrack;
use crate::domain::repository::{
    AttemptRepository, AttemptSubmission, ItemResponseStat, NewAttempt, NewSession, SessionUpdate,
    TryoutSessionRepository,
};
use crate::domain::school::SchoolType;
use crate::domain::tenant::TenantCtx;
use crate::domain::tryout::{Attempt, AttemptAnswer, AttemptStatus, SessionType, TryoutSession};
use crate::error::{AppError, AppResult};
use std::str::FromStr;

/// Opens a transaction and sets the RLS GUCs on it — shared by both repositories in this
/// file (`ptn.tryout_sessions`, `ptn.attempts`, and their child tables all carry RLS). See
/// `postgres_user_repository.rs` for the canonical explanation of this pattern.
async fn begin_tx(pool: &PgPool, ctx: &TenantCtx) -> AppResult<Transaction<'static, Postgres>> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true), set_config('app.is_platform_admin', $2, true)")
        .bind(ctx.tenant_id.to_string())
        .bind(if ctx.is_platform_admin { "true" } else { "false" })
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}

// ─── Session templates ──────────────────────────────────────────────────────────

pub struct PostgresTryoutSessionRepository {
    pool: PgPool,
}

impl PostgresTryoutSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    public_id: Uuid,
    title: String,
    session_type: String,
    duration_minutes: i32,
    question_count: i32,
    subject_filter: Option<String>,
    topic_filter: Option<String>,
    difficulty_filter: Option<String>,
    is_premium: bool,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    question_set_id: Option<Uuid>,
    is_draft: bool,
    is_elective: bool,
    exam_track: String,
    school_type_scope: Option<String>,
}

impl TryFrom<SessionRow> for TryoutSession {
    type Error = AppError;
    fn try_from(r: SessionRow) -> Result<Self, Self::Error> {
        Ok(TryoutSession {
            id: r.public_id,
            title: r.title,
            session_type: parse_session_type(&r.session_type)?,
            duration_minutes: r.duration_minutes,
            question_count: r.question_count,
            subject_filter: r.subject_filter,
            topic_filter: r.topic_filter,
            difficulty_filter: r.difficulty_filter,
            is_premium: r.is_premium,
            created_by: r.created_by,
            created_at: r.created_at,
            question_set_id: r.question_set_id,
            is_draft: r.is_draft,
            is_elective: r.is_elective,
            exam_track: parse_exam_track(&r.exam_track)?,
            school_type_scope: parse_school_type_scope(r.school_type_scope.as_deref())?,
        })
    }
}

fn parse_school_type_scope(s: Option<&str>) -> AppResult<Option<SchoolType>> {
    s.map(SchoolType::from_str)
        .transpose()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt school_type_scope: {e}")))
}

fn school_type_scope_str(t: Option<SchoolType>) -> Option<&'static str> {
    t.map(|t| t.as_str())
}

// `created_by`/`question_set_id` are resolved to their `public_id`s via scalar subqueries
// (rather than JOINs) so this same column list works unmodified in both plain SELECTs and
// INSERT/UPDATE ... RETURNING clauses (same pattern as `postgres_question_repository.rs`).
const SESSION_COLUMNS: &str = r#"
    public_id, title, session_type, duration_minutes, question_count,
    subject_filter, topic_filter, difficulty_filter, is_premium,
    (SELECT public_id FROM core.akun WHERE core.akun.id = ptn.tryout_sessions.created_by) AS created_by,
    created_at,
    (SELECT public_id FROM ptn.question_sets WHERE ptn.question_sets.id = ptn.tryout_sessions.question_set_id) AS question_set_id,
    is_draft, is_elective, exam_track, school_type_scope
"#;

// Read-path variant (find_by_id / list) — a session pinned to a curated Set Soal
// (question_set_id set) pulls its fixed question list from `question_set_items`, not the
// random-filter draw, so its stored `question_count` column is cosmetic and was never
// recalculated after questions were added/removed from the set. Catalog reads compute the
// live count instead of trusting the stale stored value.
const SESSION_COLUMNS_LIVE: &str = r#"
    public_id, title, session_type, duration_minutes,
    CASE WHEN question_set_id IS NOT NULL
         THEN (SELECT count(*)::int FROM ptn.question_set_items qsi WHERE qsi.set_id = ptn.tryout_sessions.question_set_id)
         ELSE question_count
    END AS question_count,
    subject_filter, topic_filter, difficulty_filter, is_premium,
    (SELECT public_id FROM core.akun WHERE core.akun.id = ptn.tryout_sessions.created_by) AS created_by,
    created_at,
    (SELECT public_id FROM ptn.question_sets WHERE ptn.question_sets.id = ptn.tryout_sessions.question_set_id) AS question_set_id,
    is_draft, is_elective, exam_track, school_type_scope
"#;

fn parse_session_type(s: &str) -> AppResult<SessionType> {
    match s {
        "tryout" => Ok(SessionType::Tryout),
        "drilling" => Ok(SessionType::Drilling),
        "mini" => Ok(SessionType::Mini),
        other => Err(AppError::Internal(anyhow::anyhow!("corrupt session_type: {other}"))),
    }
}

fn session_type_str(t: SessionType) -> &'static str {
    match t {
        SessionType::Tryout => "tryout",
        SessionType::Drilling => "drilling",
        SessionType::Mini => "mini",
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

#[async_trait]
impl TryoutSessionRepository for PostgresTryoutSessionRepository {
    async fn create(&self, n: NewSession, ctx: &TenantCtx) -> AppResult<TryoutSession> {
        let mut tx = begin_tx(&self.pool, ctx).await?;

        let created_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(n.created_by)
            .fetch_one(&mut *tx)
            .await?;
        let question_set_id: Option<i64> = match n.question_set_id {
            Some(qsid) => Some(
                sqlx::query_scalar("SELECT id FROM ptn.question_sets WHERE public_id = $1")
                    .bind(qsid)
                    .fetch_one(&mut *tx)
                    .await?,
            ),
            None => None,
        };

        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.tryout_sessions
                (title, session_type, duration_minutes, question_count, subject_filter,
                 topic_filter, difficulty_filter, is_premium, created_by, question_set_id, is_draft,
                 is_elective, exam_track, school_type_scope)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
               RETURNING public_id"#,
        )
        .bind(&n.title)
        .bind(session_type_str(n.session_type))
        .bind(n.duration_minutes)
        .bind(n.question_count)
        .bind(&n.subject_filter)
        .bind(&n.topic_filter)
        .bind(&n.difficulty_filter)
        .bind(n.is_premium)
        .bind(created_by_id)
        .bind(question_set_id)
        .bind(n.is_draft)
        .bind(n.is_elective)
        .bind(exam_track_str(n.exam_track))
        .bind(school_type_scope_str(n.school_type_scope))
        .fetch_one(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, SessionRow>(&format!("SELECT {SESSION_COLUMNS} FROM ptn.tryout_sessions WHERE public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        TryoutSession::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<TryoutSession>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let row = sqlx::query_as::<_, SessionRow>(&format!(
            "SELECT {SESSION_COLUMNS_LIVE} FROM ptn.tryout_sessions WHERE public_id = $1"
        ))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        row.map(TryoutSession::try_from).transpose()
    }

    async fn list(&self, session_type: Option<SessionType>, ctx: &TenantCtx) -> AppResult<Vec<TryoutSession>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = match session_type {
            Some(t) => {
                sqlx::query_as::<_, SessionRow>(&format!(
                    "SELECT {SESSION_COLUMNS_LIVE} FROM ptn.tryout_sessions WHERE session_type = $1 ORDER BY created_at DESC"
                ))
                .bind(session_type_str(t))
                .fetch_all(&mut *tx)
                .await?
            }
            None => {
                sqlx::query_as::<_, SessionRow>(&format!(
                    "SELECT {SESSION_COLUMNS_LIVE} FROM ptn.tryout_sessions ORDER BY created_at DESC"
                ))
                .fetch_all(&mut *tx)
                .await?
            }
        };
        tx.commit().await?;
        rows.into_iter().map(TryoutSession::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: SessionUpdate, ctx: &TenantCtx) -> AppResult<Option<TryoutSession>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;

        let question_set_id: Option<i64> = match u.question_set_id {
            Some(qsid) => Some(
                sqlx::query_scalar("SELECT id FROM ptn.question_sets WHERE public_id = $1")
                    .bind(qsid)
                    .fetch_one(&mut *tx)
                    .await?,
            ),
            None => None,
        };

        let updated = sqlx::query(
            r#"UPDATE ptn.tryout_sessions SET
                title = $2, session_type = $3, duration_minutes = $4, question_count = $5,
                subject_filter = $6, topic_filter = $7, difficulty_filter = $8, is_premium = $9,
                question_set_id = $10, is_draft = $11, is_elective = $12, exam_track = $13,
                school_type_scope = $14
               WHERE public_id = $1"#,
        )
        .bind(id)
        .bind(&u.title)
        .bind(session_type_str(u.session_type))
        .bind(u.duration_minutes)
        .bind(u.question_count)
        .bind(&u.subject_filter)
        .bind(&u.topic_filter)
        .bind(&u.difficulty_filter)
        .bind(u.is_premium)
        .bind(question_set_id)
        .bind(u.is_draft)
        .bind(u.is_elective)
        .bind(exam_track_str(u.exam_track))
        .bind(school_type_scope_str(u.school_type_scope))
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }

        let row = sqlx::query_as::<_, SessionRow>(&format!("SELECT {SESSION_COLUMNS} FROM ptn.tryout_sessions WHERE public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        TryoutSession::try_from(row).map(Some)
    }

    async fn set_draft(&self, id: Uuid, is_draft: bool, ctx: &TenantCtx) -> AppResult<Option<TryoutSession>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let updated = sqlx::query("UPDATE ptn.tryout_sessions SET is_draft = $2 WHERE public_id = $1")
            .bind(id)
            .bind(is_draft)
            .execute(&mut *tx)
            .await?;
        if updated.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, SessionRow>(&format!("SELECT {SESSION_COLUMNS} FROM ptn.tryout_sessions WHERE public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        TryoutSession::try_from(row).map(Some)
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query("DELETE FROM ptn.tryout_sessions WHERE public_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                AppError::from_sqlx_delete(
                    e,
                    "cannot delete this session: students already have attempts recorded against it",
                )
            })?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }
}

// ─── Attempts ───────────────────────────────────────────────────────────────────

pub struct PostgresAttemptRepository {
    pool: PgPool,
}

impl PostgresAttemptRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AttemptRow {
    public_id: Uuid,
    session_id: Uuid,
    session_title: String,
    session_type: String,
    student_id: Uuid,
    status: String,
    duration_minutes: i32,
    started_at: DateTime<Utc>,
    submitted_at: Option<DateTime<Utc>>,
    score: Option<i32>,
    accuracy: Option<f64>,
    correct_count: Option<i32>,
    wrong_count: Option<i32>,
    unanswered_count: Option<i32>,
    time_used_seconds: Option<i32>,
}

impl TryFrom<AttemptRow> for Attempt {
    type Error = AppError;
    fn try_from(r: AttemptRow) -> Result<Self, Self::Error> {
        Ok(Attempt {
            id: r.public_id,
            session_id: r.session_id,
            session_title: r.session_title,
            session_type: parse_session_type(&r.session_type)?,
            student_id: r.student_id,
            status: if r.status == "submitted" { AttemptStatus::Submitted } else { AttemptStatus::InProgress },
            duration_minutes: r.duration_minutes,
            started_at: r.started_at,
            submitted_at: r.submitted_at,
            score: r.score,
            accuracy: r.accuracy,
            correct_count: r.correct_count,
            wrong_count: r.wrong_count,
            unanswered_count: r.unanswered_count,
            time_used_seconds: r.time_used_seconds,
        })
    }
}

const SELECT_ATTEMPT: &str = r#"
    SELECT a.public_id, s.public_id AS session_id, s.title AS session_title, s.session_type,
           (SELECT public_id FROM core.akun WHERE core.akun.id = a.student_id) AS student_id,
           a.status, a.duration_minutes, a.started_at, a.submitted_at,
           a.score, a.accuracy, a.correct_count, a.wrong_count, a.unanswered_count,
           a.time_used_seconds
    FROM ptn.attempts a
    JOIN ptn.tryout_sessions s ON s.id = a.session_id
"#;

#[derive(sqlx::FromRow)]
struct AnswerRow {
    public_id: Uuid,
    attempt_id: Uuid,
    question_id: Uuid,
    answer_text: Option<String>,
    is_correct: Option<bool>,
    flagged: bool,
    answered_at: DateTime<Utc>,
}

impl From<AnswerRow> for AttemptAnswer {
    fn from(r: AnswerRow) -> Self {
        AttemptAnswer {
            id: r.public_id,
            attempt_id: r.attempt_id,
            question_id: r.question_id,
            answer_text: r.answer_text,
            is_correct: r.is_correct,
            flagged: r.flagged,
            answered_at: r.answered_at,
        }
    }
}

const SELECT_ANSWER: &str = r#"
    SELECT aa.public_id, att.public_id AS attempt_id, q.public_id AS question_id,
           aa.answer_text, aa.is_correct, aa.flagged, aa.answered_at
    FROM ptn.attempt_answers aa
    JOIN ptn.attempts att ON att.id = aa.attempt_id
    JOIN ptn.questions q ON q.id = aa.question_id
"#;

#[async_trait]
impl AttemptRepository for PostgresAttemptRepository {
    async fn create(&self, n: NewAttempt, ctx: &TenantCtx) -> AppResult<Attempt> {
        let mut tx = begin_tx(&self.pool, ctx).await?;

        let session_id: i64 = sqlx::query_scalar("SELECT id FROM ptn.tryout_sessions WHERE public_id = $1")
            .bind(n.session_id)
            .fetch_one(&mut *tx)
            .await?;
        let student_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(n.student_id)
            .fetch_one(&mut *tx)
            .await?;

        let attempt_id: i64 = sqlx::query_scalar(
            r#"INSERT INTO ptn.attempts (tenant_id, session_id, student_id, duration_minutes)
               VALUES ($1, $2, $3, $4) RETURNING id"#,
        )
        .bind(ctx.tenant_id)
        .bind(session_id)
        .bind(student_id)
        .bind(n.duration_minutes)
        .fetch_one(&mut *tx)
        .await?;

        for (idx, qid) in n.question_ids.iter().enumerate() {
            let question_id: i64 = sqlx::query_scalar("SELECT id FROM ptn.questions WHERE public_id = $1")
                .bind(qid)
                .fetch_one(&mut *tx)
                .await?;
            sqlx::query(
                r#"INSERT INTO ptn.attempt_questions (attempt_id, question_id, order_index)
                   VALUES ($1, $2, $3)"#,
            )
            .bind(attempt_id)
            .bind(question_id)
            .bind(idx as i32)
            .execute(&mut *tx)
            .await?;
        }

        let row = sqlx::query_as::<_, AttemptRow>(&format!("{SELECT_ATTEMPT} WHERE a.id = $1"))
            .bind(attempt_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Attempt::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<Attempt>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let row = sqlx::query_as::<_, AttemptRow>(&format!("{SELECT_ATTEMPT} WHERE a.public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(Attempt::try_from).transpose()
    }

    async fn list_by_student(&self, student_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<Attempt>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, AttemptRow>(&format!(
            "{SELECT_ATTEMPT} WHERE a.student_id = (SELECT id FROM core.akun WHERE public_id = $1) ORDER BY a.started_at DESC"
        ))
        .bind(student_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows.into_iter().map(Attempt::try_from).collect()
    }

    async fn question_ids_for_attempt(&self, attempt_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<Uuid>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let ids: Vec<(Uuid,)> = sqlx::query_as(
            r#"SELECT q.public_id FROM ptn.attempt_questions aq
               JOIN ptn.questions q ON q.id = aq.question_id
               JOIN ptn.attempts att ON att.id = aq.attempt_id
               WHERE att.public_id = $1 ORDER BY aq.order_index ASC"#,
        )
        .bind(attempt_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(ids.into_iter().map(|(id,)| id).collect())
    }

    async fn upsert_answer(
        &self,
        attempt_id: Uuid,
        question_id: Uuid,
        answer_text: Option<String>,
        flagged: bool,
        ctx: &TenantCtx,
    ) -> AppResult<AttemptAnswer> {
        let mut tx = begin_tx(&self.pool, ctx).await?;

        let attempt_pk: i64 = sqlx::query_scalar("SELECT id FROM ptn.attempts WHERE public_id = $1")
            .bind(attempt_id)
            .fetch_one(&mut *tx)
            .await?;
        let question_pk: i64 = sqlx::query_scalar("SELECT id FROM ptn.questions WHERE public_id = $1")
            .bind(question_id)
            .fetch_one(&mut *tx)
            .await?;

        let answer_public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.attempt_answers (attempt_id, question_id, answer_text, flagged, answered_at)
               VALUES ($1, $2, $3, $4, now())
               ON CONFLICT (attempt_id, question_id)
               DO UPDATE SET answer_text = EXCLUDED.answer_text, flagged = EXCLUDED.flagged, answered_at = now()
               RETURNING public_id"#,
        )
        .bind(attempt_pk)
        .bind(question_pk)
        .bind(&answer_text)
        .bind(flagged)
        .fetch_one(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, AnswerRow>(&format!("{SELECT_ANSWER} WHERE aa.public_id = $1"))
            .bind(answer_public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(row.into())
    }

    async fn answers_for_attempt(&self, attempt_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<AttemptAnswer>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, AnswerRow>(&format!("{SELECT_ANSWER} WHERE att.public_id = $1"))
            .bind(attempt_id)
            .fetch_all(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn submit(&self, attempt_id: Uuid, s: AttemptSubmission, ctx: &TenantCtx) -> AppResult<Attempt> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        sqlx::query(
            r#"UPDATE ptn.attempts SET
                status = 'submitted', submitted_at = now(), score = $2, accuracy = $3,
                correct_count = $4, wrong_count = $5, unanswered_count = $6, time_used_seconds = $7
               WHERE public_id = $1"#,
        )
        .bind(attempt_id)
        .bind(s.score)
        .bind(s.accuracy)
        .bind(s.correct_count)
        .bind(s.wrong_count)
        .bind(s.unanswered_count)
        .bind(s.time_used_seconds)
        .execute(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, AttemptRow>(&format!("{SELECT_ATTEMPT} WHERE a.public_id = $1"))
            .bind(attempt_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("attempt {attempt_id} not found")))?;
        tx.commit().await?;
        Attempt::try_from(row)
    }

    async fn grade_answer(&self, answer_id: Uuid, is_correct: bool, ctx: &TenantCtx) -> AppResult<()> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        sqlx::query("UPDATE ptn.attempt_answers SET is_correct = $2 WHERE public_id = $1")
            .bind(answer_id)
            .bind(is_correct)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn item_response_stats(&self, ctx: &TenantCtx) -> AppResult<Vec<ItemResponseStat>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, ItemResponseStatRow>(
            r#"SELECT q.public_id AS question_id,
                      COUNT(*) FILTER (WHERE aa.is_correct = true) AS correct,
                      COUNT(*) AS total
               FROM ptn.attempt_answers aa
               JOIN ptn.questions q ON q.id = aa.question_id
               WHERE aa.is_correct IS NOT NULL
               GROUP BY q.public_id"#,
        )
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(sqlx::FromRow)]
struct ItemResponseStatRow {
    question_id: Uuid,
    correct: Option<i64>,
    total: Option<i64>,
}

impl From<ItemResponseStatRow> for ItemResponseStat {
    fn from(r: ItemResponseStatRow) -> Self {
        // `COUNT(*)` never actually returns NULL for a row that GROUP BY produced, but sqlx
        // still types aggregate columns as nullable by default — default to 0 rather than
        // unwrapping, since "vanished" data should read as "no data" (honest-zero), not panic.
        ItemResponseStat { question_id: r.question_id, correct: r.correct.unwrap_or(0), total: r.total.unwrap_or(0) }
    }
}
