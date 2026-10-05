use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::package::ExamTrack;
use crate::domain::repository::{
    AttemptRepository, AttemptSubmission, ItemResponseStat, NewAttempt, NewSession, SessionUpdate,
    TryoutSessionRepository,
};
use crate::domain::school::SchoolType;
use crate::domain::tryout::{Attempt, AttemptAnswer, AttemptStatus, SessionType, TryoutSession};
use crate::error::{AppError, AppResult};
use std::str::FromStr;

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
    id: Uuid,
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
            id: r.id,
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

// Shared column list for INSERT/UPDATE ... RETURNING clauses — plain `question_count`, since
// right after a write the caller already knows what it just set (see NewSession::question_count
// doc comment: cosmetic-only once question_set_id is set, never meant to be trusted long-term).
const SESSION_COLUMNS: &str = r#"id, title, session_type, duration_minutes, question_count,
                         subject_filter, topic_filter, difficulty_filter, is_premium,
                         created_by, created_at, question_set_id, is_draft, is_elective, exam_track,
                         school_type_scope"#;

// Read-path variant (find_by_id / list) — a session pinned to a curated Set Soal
// (question_set_id set) pulls its fixed question list from `question_set_items`, not the
// random-filter draw, so its stored `question_count` column is cosmetic and was never
// recalculated after questions were added/removed from the set (root cause of the "0 soal"
// display bug on Set-Soal-based subtes cards even when the set has real questions). Catalog
// reads compute the live count instead of trusting the stale stored value; sessions without a
// question_set_id (random-filter draw) keep using the stored column as before.
const SESSION_COLUMNS_LIVE: &str = r#"id, title, session_type, duration_minutes,
                         CASE WHEN question_set_id IS NOT NULL
                              THEN (SELECT count(*)::int FROM question_set_items qsi WHERE qsi.set_id = tryout_sessions.question_set_id)
                              ELSE question_count
                         END AS question_count,
                         subject_filter, topic_filter, difficulty_filter, is_premium,
                         created_by, created_at, question_set_id, is_draft, is_elective, exam_track,
                         school_type_scope"#;

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
    async fn create(&self, n: NewSession) -> AppResult<TryoutSession> {
        let id = Uuid::new_v4();
        let row = sqlx::query_as::<_, SessionRow>(&format!(
            r#"INSERT INTO tryout_sessions
                (id, title, session_type, duration_minutes, question_count, subject_filter,
                 topic_filter, difficulty_filter, is_premium, created_by, question_set_id, is_draft, is_elective, exam_track,
                 school_type_scope)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
               RETURNING {SESSION_COLUMNS}"#
        ))
        .bind(id)
        .bind(&n.title)
        .bind(session_type_str(n.session_type))
        .bind(n.duration_minutes)
        .bind(n.question_count)
        .bind(&n.subject_filter)
        .bind(&n.topic_filter)
        .bind(&n.difficulty_filter)
        .bind(n.is_premium)
        .bind(n.created_by)
        .bind(n.question_set_id)
        .bind(n.is_draft)
        .bind(n.is_elective)
        .bind(exam_track_str(n.exam_track))
        .bind(school_type_scope_str(n.school_type_scope))
        .fetch_one(&self.pool)
        .await?;

        TryoutSession::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<TryoutSession>> {
        let row = sqlx::query_as::<_, SessionRow>(&format!(
            "SELECT {SESSION_COLUMNS_LIVE} FROM tryout_sessions WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(TryoutSession::try_from).transpose()
    }

    async fn list(&self, session_type: Option<SessionType>) -> AppResult<Vec<TryoutSession>> {
        let rows = match session_type {
            Some(t) => {
                sqlx::query_as::<_, SessionRow>(&format!(
                    "SELECT {SESSION_COLUMNS_LIVE} FROM tryout_sessions WHERE session_type = $1 ORDER BY created_at DESC"
                ))
                .bind(session_type_str(t))
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, SessionRow>(&format!(
                    "SELECT {SESSION_COLUMNS_LIVE} FROM tryout_sessions ORDER BY created_at DESC"
                ))
                .fetch_all(&self.pool)
                .await?
            }
        };
        rows.into_iter().map(TryoutSession::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: SessionUpdate) -> AppResult<Option<TryoutSession>> {
        let row = sqlx::query_as::<_, SessionRow>(&format!(
            r#"UPDATE tryout_sessions SET
                title = $2, session_type = $3, duration_minutes = $4, question_count = $5,
                subject_filter = $6, topic_filter = $7, difficulty_filter = $8, is_premium = $9,
                question_set_id = $10, is_draft = $11, is_elective = $12, exam_track = $13,
                school_type_scope = $14
               WHERE id = $1
               RETURNING {SESSION_COLUMNS}"#
        ))
        .bind(id)
        .bind(&u.title)
        .bind(session_type_str(u.session_type))
        .bind(u.duration_minutes)
        .bind(u.question_count)
        .bind(&u.subject_filter)
        .bind(&u.topic_filter)
        .bind(&u.difficulty_filter)
        .bind(u.is_premium)
        .bind(u.question_set_id)
        .bind(u.is_draft)
        .bind(u.is_elective)
        .bind(exam_track_str(u.exam_track))
        .bind(school_type_scope_str(u.school_type_scope))
        .fetch_optional(&self.pool)
        .await?;
        row.map(TryoutSession::try_from).transpose()
    }

    async fn set_draft(&self, id: Uuid, is_draft: bool) -> AppResult<Option<TryoutSession>> {
        let row = sqlx::query_as::<_, SessionRow>(&format!(
            "UPDATE tryout_sessions SET is_draft = $2 WHERE id = $1 RETURNING {SESSION_COLUMNS}"
        ))
        .bind(id)
        .bind(is_draft)
        .fetch_optional(&self.pool)
        .await?;
        row.map(TryoutSession::try_from).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM tryout_sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::from_sqlx_delete(
                    e,
                    "cannot delete this session: students already have attempts recorded against it",
                )
            })?;
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
    id: Uuid,
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
            id: r.id,
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
    SELECT a.id, a.session_id, s.title AS session_title, s.session_type,
           a.student_id, a.status, a.duration_minutes, a.started_at, a.submitted_at,
           a.score, a.accuracy, a.correct_count, a.wrong_count, a.unanswered_count,
           a.time_used_seconds
    FROM attempts a
    JOIN tryout_sessions s ON s.id = a.session_id
"#;

#[derive(sqlx::FromRow)]
struct AnswerRow {
    id: Uuid,
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
            id: r.id,
            attempt_id: r.attempt_id,
            question_id: r.question_id,
            answer_text: r.answer_text,
            is_correct: r.is_correct,
            flagged: r.flagged,
            answered_at: r.answered_at,
        }
    }
}

#[async_trait]
impl AttemptRepository for PostgresAttemptRepository {
    async fn create(&self, n: NewAttempt) -> AppResult<Attempt> {
        let id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"INSERT INTO attempts (id, session_id, student_id, duration_minutes)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(id)
        .bind(n.session_id)
        .bind(n.student_id)
        .bind(n.duration_minutes)
        .execute(&mut *tx)
        .await?;

        for (idx, qid) in n.question_ids.iter().enumerate() {
            sqlx::query(
                r#"INSERT INTO attempt_questions (attempt_id, question_id, order_index)
                   VALUES ($1, $2, $3)"#,
            )
            .bind(id)
            .bind(qid)
            .bind(idx as i32)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("attempt vanished right after insert")))
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Attempt>> {
        let row = sqlx::query_as::<_, AttemptRow>(&format!("{SELECT_ATTEMPT} WHERE a.id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(Attempt::try_from).transpose()
    }

    async fn list_by_student(&self, student_id: Uuid) -> AppResult<Vec<Attempt>> {
        let rows = sqlx::query_as::<_, AttemptRow>(&format!(
            "{SELECT_ATTEMPT} WHERE a.student_id = $1 ORDER BY a.started_at DESC"
        ))
        .bind(student_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Attempt::try_from).collect()
    }

    async fn question_ids_for_attempt(&self, attempt_id: Uuid) -> AppResult<Vec<Uuid>> {
        let ids: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT question_id FROM attempt_questions WHERE attempt_id = $1 ORDER BY order_index ASC",
        )
        .bind(attempt_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(ids.into_iter().map(|(id,)| id).collect())
    }

    async fn upsert_answer(
        &self,
        attempt_id: Uuid,
        question_id: Uuid,
        answer_text: Option<String>,
        flagged: bool,
    ) -> AppResult<AttemptAnswer> {
        let row = sqlx::query_as::<_, AnswerRow>(
            r#"INSERT INTO attempt_answers (id, attempt_id, question_id, answer_text, flagged, answered_at)
               VALUES ($1, $2, $3, $4, $5, now())
               ON CONFLICT (attempt_id, question_id)
               DO UPDATE SET answer_text = EXCLUDED.answer_text, flagged = EXCLUDED.flagged, answered_at = now()
               RETURNING id, attempt_id, question_id, answer_text, is_correct, flagged, answered_at"#,
        )
        .bind(Uuid::new_v4())
        .bind(attempt_id)
        .bind(question_id)
        .bind(&answer_text)
        .bind(flagged)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn answers_for_attempt(&self, attempt_id: Uuid) -> AppResult<Vec<AttemptAnswer>> {
        let rows = sqlx::query_as::<_, AnswerRow>(
            "SELECT id, attempt_id, question_id, answer_text, is_correct, flagged, answered_at
             FROM attempt_answers WHERE attempt_id = $1",
        )
        .bind(attempt_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn submit(&self, attempt_id: Uuid, s: AttemptSubmission) -> AppResult<Attempt> {
        sqlx::query(
            r#"UPDATE attempts SET
                status = 'submitted', submitted_at = now(), score = $2, accuracy = $3,
                correct_count = $4, wrong_count = $5, unanswered_count = $6, time_used_seconds = $7
               WHERE id = $1"#,
        )
        .bind(attempt_id)
        .bind(s.score)
        .bind(s.accuracy)
        .bind(s.correct_count)
        .bind(s.wrong_count)
        .bind(s.unanswered_count)
        .bind(s.time_used_seconds)
        .execute(&self.pool)
        .await?;

        self.find_by_id(attempt_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("attempt {attempt_id} not found")))
    }

    async fn grade_answer(&self, answer_id: Uuid, is_correct: bool) -> AppResult<()> {
        sqlx::query("UPDATE attempt_answers SET is_correct = $2 WHERE id = $1")
            .bind(answer_id)
            .bind(is_correct)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn item_response_stats(&self) -> AppResult<Vec<ItemResponseStat>> {
        let rows = sqlx::query_as::<_, ItemResponseStatRow>(
            r#"SELECT question_id,
                      COUNT(*) FILTER (WHERE is_correct = true) AS correct,
                      COUNT(*) AS total
               FROM attempt_answers
               WHERE is_correct IS NOT NULL
               GROUP BY question_id"#,
        )
        .fetch_all(&self.pool)
        .await?;
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
