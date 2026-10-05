//! Postgres implementation of `QuestionSetRepository` ("Set Soal") — mirrors the
//! `postgres_question_repository.rs` style (FromRow struct + a shared column list used by
//! both plain SELECTs and INSERT/UPDATE ... RETURNING). `SELECT_COLUMNS`/`QuestionRow` in
//! that file are private, so rather than widen their visibility for this one extra caller,
//! this file duplicates the minimal question column list it needs (less invasive than
//! touching an existing, working repository's encapsulation).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{types::Json, PgPool};
use uuid::Uuid;

use crate::domain::question::Question;
use crate::domain::question_set::QuestionSet;
use crate::domain::repository::QuestionSetRepository;
use crate::domain::tryout::SessionType;
use crate::error::{AppError, AppResult};

pub struct PostgresQuestionSetRepository {
    pool: PgPool,
}

impl PostgresQuestionSetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ─── question_sets row mapping ───────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct QuestionSetRow {
    id: Uuid,
    name: String,
    description: String,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    item_count: i64,
}

impl From<QuestionSetRow> for QuestionSet {
    fn from(r: QuestionSetRow) -> Self {
        QuestionSet {
            id: r.id,
            name: r.name,
            description: r.description,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
            item_count: r.item_count,
        }
    }
}

// `item_count` is resolved via a scalar subquery (rather than a GROUP BY) so this same
// column list works unmodified in plain SELECTs, INSERT/UPDATE ... RETURNING, and
// single-row lookups alike — same pattern as `created_by_name` in
// `postgres_question_repository.rs`.
const SELECT_SET_COLUMNS: &str = r#"
    id, name, description, created_by, created_at, updated_at,
    (SELECT COUNT(*) FROM question_set_items WHERE question_set_items.set_id = question_sets.id) AS item_count
"#;

// ─── questions row mapping (duplicated minimal subset — see module doc comment) ──

#[derive(sqlx::FromRow)]
struct SetQuestionRow {
    id: Uuid,
    code: String,
    question_type: String,
    subject: String,
    topic: String,
    subtopic: String,
    difficulty: String,
    bloom_level: String,
    stimulus: Option<String>,
    question_text: String,
    options: Option<Json<Vec<String>>>,
    correct_answer: String,
    explanation: String,
    tags: Vec<String>,
    status: String,
    review_note: Option<String>,
    reviewed_by: Option<Uuid>,
    reviewed_at: Option<DateTime<Utc>>,
    created_by: Uuid,
    created_by_name: String,
    usage_count: i32,
    average_score: f64,
    time_limit: Option<i32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<SetQuestionRow> for Question {
    type Error = AppError;

    fn try_from(r: SetQuestionRow) -> Result<Self, Self::Error> {
        Ok(Question {
            id: r.id,
            code: r.code,
            question_type: r
                .question_type
                .parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt question_type: {e}")))?,
            subject: r.subject,
            topic: r.topic,
            subtopic: r.subtopic,
            difficulty: r
                .difficulty
                .parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt difficulty: {e}")))?,
            bloom_level: r.bloom_level,
            stimulus: r.stimulus,
            question_text: r.question_text,
            options: r.options.map(|Json(v)| v),
            correct_answer: r.correct_answer,
            explanation: r.explanation,
            tags: r.tags,
            status: r
                .status
                .parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt status: {e}")))?,
            review_note: r.review_note,
            reviewed_by: r.reviewed_by,
            reviewed_at: r.reviewed_at,
            created_by: r.created_by,
            created_by_name: r.created_by_name,
            usage_count: r.usage_count,
            average_score: r.average_score,
            time_limit: r.time_limit,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

const SELECT_SET_QUESTION_COLUMNS: &str = r#"
    q.id, q.code, q.question_type, q.subject, q.topic, q.subtopic, q.difficulty, q.bloom_level,
    q.stimulus, q.question_text, q.options, q.correct_answer, q.explanation, q.tags, q.status,
    q.review_note, q.reviewed_by, q.reviewed_at, q.created_by,
    (SELECT name FROM users WHERE users.id = q.created_by) AS created_by_name,
    q.usage_count, q.average_score, q.time_limit, q.created_at, q.updated_at
"#;

#[async_trait]
impl QuestionSetRepository for PostgresQuestionSetRepository {
    async fn create(&self, name: String, description: String, created_by: Uuid) -> AppResult<QuestionSet> {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO question_sets (id, name, description, created_by) VALUES ($1,$2,$3,$4)")
            .bind(id)
            .bind(&name)
            .bind(&description)
            .bind(created_by)
            .execute(&self.pool)
            .await?;

        let row = sqlx::query_as::<_, QuestionSetRow>(&format!(
            "SELECT {SELECT_SET_COLUMNS} FROM question_sets WHERE id = $1"
        ))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn list(&self) -> AppResult<Vec<QuestionSet>> {
        let rows = sqlx::query_as::<_, QuestionSetRow>(&format!(
            "SELECT {SELECT_SET_COLUMNS} FROM question_sets ORDER BY created_at DESC"
        ))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(QuestionSet::from).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<QuestionSet>> {
        let row = sqlx::query_as::<_, QuestionSetRow>(&format!(
            "SELECT {SELECT_SET_COLUMNS} FROM question_sets WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(QuestionSet::from))
    }

    async fn update(&self, id: Uuid, name: String, description: String) -> AppResult<Option<QuestionSet>> {
        let result = sqlx::query("UPDATE question_sets SET name = $2, description = $3, updated_at = now() WHERE id = $1")
            .bind(id)
            .bind(&name)
            .bind(&description)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }
        let row = sqlx::query_as::<_, QuestionSetRow>(&format!(
            "SELECT {SELECT_SET_COLUMNS} FROM question_sets WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(QuestionSet::from))
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        // `tryout_sessions.question_set_id` is `ON DELETE SET NULL`, so this never fails
        // with a foreign-key conflict — a session referencing this set just falls back to
        // its original filter fields.
        let result = sqlx::query("DELETE FROM question_sets WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_items(&self, set_id: Uuid) -> AppResult<Vec<Question>> {
        let rows = sqlx::query_as::<_, SetQuestionRow>(&format!(
            r#"SELECT {SELECT_SET_QUESTION_COLUMNS}
               FROM questions q
               JOIN question_set_items qsi ON qsi.question_id = q.id
               WHERE qsi.set_id = $1
               ORDER BY qsi.sort_order ASC"#
        ))
        .bind(set_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Question::try_from).collect()
    }

    async fn add_item(&self, set_id: Uuid, question_id: Uuid, sort_order: i32) -> AppResult<()> {
        // Adding a question already in the set is a no-op success (idempotent), not an
        // error — the question authoring form calls this on every save of a question that
        // has a set assigned, including re-saves of an already-assigned question.
        sqlx::query(
            r#"INSERT INTO question_set_items (id, set_id, question_id, sort_order)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (set_id, question_id) DO NOTHING"#,
        )
        .bind(Uuid::new_v4())
        .bind(set_id)
        .bind(question_id)
        .bind(sort_order)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn remove_item(&self, set_id: Uuid, question_id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM question_set_items WHERE set_id = $1 AND question_id = $2")
            .bind(set_id)
            .bind(question_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn approved_items_for_session(&self, set_id: Uuid, session_type: SessionType) -> AppResult<Vec<Question>> {
        let mut sql = format!(
            r#"SELECT {SELECT_SET_QUESTION_COLUMNS}
               FROM questions q
               JOIN question_set_items qsi ON qsi.question_id = q.id
               WHERE qsi.set_id = $1 AND q.status = 'approved'"#
        );
        // Same essay-exclusion rule as `QuestionRepository::random_approved` — essay is
        // self-check/Drilling-only, never auto-graded, so it must never be selected into a
        // Tryout/Mini attempt, even via a curated Set Soal.
        if !matches!(session_type, SessionType::Drilling) {
            sql.push_str(" AND q.question_type != 'essay'");
        }
        // Preserve the curated `sort_order` — do NOT re-sort by id like the random path does.
        sql.push_str(" ORDER BY qsi.sort_order ASC");

        let rows = sqlx::query_as::<_, SetQuestionRow>(&sql)
            .bind(set_id)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Question::try_from).collect()
    }
}
