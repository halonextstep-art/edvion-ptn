//! Postgres implementation of `QuestionSetRepository` ("Set Soal") — mirrors the
//! `postgres_question_repository.rs` style (FromRow struct + a shared column list used by
//! both plain SELECTs and INSERT/UPDATE ... RETURNING). `SELECT_COLUMNS`/`QuestionRow` in
//! that file are private, so rather than widen their visibility for this one extra caller,
//! this file duplicates the minimal question column list it needs (less invasive than
//! touching an existing, working repository's encapsulation).
//!
//! `ptn.question_sets` carries RLS (`tenant_id`); `ptn.question_set_items` does not (scoped
//! transitively via `set_id -> question_sets.tenant_id`, see that table's doc comment in
//! `20260101000003_ptn_core_content.sql`) — every method still opens an RLS-scoped
//! transaction via `begin_tx` regardless, since `ptn.questions` (joined for `list_items`/
//! `approved_items_for_session`) and `ptn.question_sets` itself both need the GUCs set first.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{types::Json, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::question::Question;
use crate::domain::question_set::QuestionSet;
use crate::domain::repository::QuestionSetRepository;
use crate::domain::tenant::TenantCtx;
use crate::domain::tryout::SessionType;
use crate::error::{AppError, AppResult};

pub struct PostgresQuestionSetRepository {
    pool: PgPool,
}

impl PostgresQuestionSetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// See `postgres_user_repository.rs::begin_tx` doc comment.
    async fn begin_tx(&self, ctx: &TenantCtx) -> AppResult<Transaction<'static, Postgres>> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "SELECT set_config('app.tenant_id', $1, true), set_config('app.is_platform_admin', $2, true)",
        )
        .bind(ctx.tenant_id.to_string())
        .bind(if ctx.is_platform_admin { "true" } else { "false" })
        .execute(&mut *tx)
        .await?;
        Ok(tx)
    }
}

// ─── question_sets row mapping ───────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct QuestionSetRow {
    public_id: Uuid,
    name: String,
    description: String,
    created_by_public_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    item_count: i64,
}

impl From<QuestionSetRow> for QuestionSet {
    fn from(r: QuestionSetRow) -> Self {
        QuestionSet {
            id: r.public_id,
            name: r.name,
            description: r.description,
            created_by: r.created_by_public_id,
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
    public_id, name, description,
    (SELECT public_id FROM core.akun WHERE core.akun.id = ptn.question_sets.created_by) AS created_by_public_id,
    created_at, updated_at,
    (SELECT COUNT(*) FROM ptn.question_set_items WHERE ptn.question_set_items.set_id = ptn.question_sets.id) AS item_count
"#;

// ─── questions row mapping (duplicated minimal subset — see module doc comment) ──

#[derive(sqlx::FromRow)]
struct SetQuestionRow {
    public_id: Uuid,
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
    reviewed_by_public_id: Option<Uuid>,
    reviewed_at: Option<DateTime<Utc>>,
    created_by_public_id: Uuid,
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
            id: r.public_id,
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
            reviewed_by: r.reviewed_by_public_id,
            reviewed_at: r.reviewed_at,
            created_by: r.created_by_public_id,
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
    q.public_id, q.code, q.question_type, q.subject, q.topic, q.subtopic, q.difficulty, q.bloom_level,
    q.stimulus, q.question_text, q.options, q.correct_answer, q.explanation, q.tags, q.status,
    q.review_note,
    (SELECT public_id FROM core.akun WHERE core.akun.id = q.reviewed_by) AS reviewed_by_public_id,
    q.reviewed_at,
    (SELECT public_id FROM core.akun WHERE core.akun.id = q.created_by) AS created_by_public_id,
    (SELECT full_name FROM core.akun WHERE core.akun.id = q.created_by) AS created_by_name,
    q.usage_count, q.average_score, q.time_limit, q.created_at, q.updated_at
"#;

#[async_trait]
impl QuestionSetRepository for PostgresQuestionSetRepository {
    async fn create(&self, name: String, description: String, created_by: Uuid, ctx: &TenantCtx) -> AppResult<QuestionSet> {
        let mut tx = self.begin_tx(ctx).await?;

        let created_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(created_by)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("created_by lookup failed: {e}")))?;

        let set_id: i64 = sqlx::query_scalar(
            "INSERT INTO ptn.question_sets (name, description, created_by) VALUES ($1,$2,$3) RETURNING id",
        )
        .bind(&name)
        .bind(&description)
        .bind(created_by_id)
        .fetch_one(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, QuestionSetRow>(&format!(
            "SELECT {SELECT_SET_COLUMNS} FROM ptn.question_sets WHERE id = $1"
        ))
        .bind(set_id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(row.into())
    }

    async fn list(&self, ctx: &TenantCtx) -> AppResult<Vec<QuestionSet>> {
        let mut tx = self.begin_tx(ctx).await?;
        let rows = sqlx::query_as::<_, QuestionSetRow>(&format!(
            "SELECT {SELECT_SET_COLUMNS} FROM ptn.question_sets ORDER BY created_at DESC"
        ))
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(rows.into_iter().map(QuestionSet::from).collect())
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<QuestionSet>> {
        let mut tx = self.begin_tx(ctx).await?;
        let row = sqlx::query_as::<_, QuestionSetRow>(&format!(
            "SELECT {SELECT_SET_COLUMNS} FROM ptn.question_sets WHERE public_id = $1"
        ))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(row.map(QuestionSet::from))
    }

    async fn update(&self, id: Uuid, name: String, description: String, ctx: &TenantCtx) -> AppResult<Option<QuestionSet>> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("UPDATE ptn.question_sets SET name = $2, description = $3, updated_at = now() WHERE public_id = $1")
            .bind(id)
            .bind(&name)
            .bind(&description)
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, QuestionSetRow>(&format!(
            "SELECT {SELECT_SET_COLUMNS} FROM ptn.question_sets WHERE public_id = $1"
        ))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(row.map(QuestionSet::from))
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        // `tryout_sessions.question_set_id` is `ON DELETE SET NULL`, so this never fails
        // with a foreign-key conflict — a session referencing this set just falls back to
        // its original filter fields.
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("DELETE FROM ptn.question_sets WHERE public_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_items(&self, set_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<Question>> {
        let mut tx = self.begin_tx(ctx).await?;
        let rows = sqlx::query_as::<_, SetQuestionRow>(&format!(
            r#"SELECT {SELECT_SET_QUESTION_COLUMNS}
               FROM ptn.questions q
               JOIN ptn.question_set_items qsi ON qsi.question_id = q.id
               JOIN ptn.question_sets qs ON qs.id = qsi.set_id
               WHERE qs.public_id = $1
               ORDER BY qsi.sort_order ASC"#
        ))
        .bind(set_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows.into_iter().map(Question::try_from).collect()
    }

    async fn add_item(&self, set_id: Uuid, question_id: Uuid, sort_order: i32, ctx: &TenantCtx) -> AppResult<()> {
        // Adding a question already in the set is a no-op success (idempotent), not an
        // error — the question authoring form calls this on every save of a question that
        // has a set assigned, including re-saves of an already-assigned question.
        let mut tx = self.begin_tx(ctx).await?;
        sqlx::query(
            r#"INSERT INTO ptn.question_set_items (set_id, question_id, sort_order)
               VALUES (
                   (SELECT id FROM ptn.question_sets WHERE public_id = $1),
                   (SELECT id FROM ptn.questions WHERE public_id = $2),
                   $3
               )
               ON CONFLICT (set_id, question_id) DO NOTHING"#,
        )
        .bind(set_id)
        .bind(question_id)
        .bind(sort_order)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn remove_item(&self, set_id: Uuid, question_id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query(
            r#"DELETE FROM ptn.question_set_items
               WHERE set_id = (SELECT id FROM ptn.question_sets WHERE public_id = $1)
                 AND question_id = (SELECT id FROM ptn.questions WHERE public_id = $2)"#,
        )
        .bind(set_id)
        .bind(question_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn approved_items_for_session(&self, set_id: Uuid, session_type: SessionType, ctx: &TenantCtx) -> AppResult<Vec<Question>> {
        let mut tx = self.begin_tx(ctx).await?;
        let mut sql = format!(
            r#"SELECT {SELECT_SET_QUESTION_COLUMNS}
               FROM ptn.questions q
               JOIN ptn.question_set_items qsi ON qsi.question_id = q.id
               JOIN ptn.question_sets qs ON qs.id = qsi.set_id
               WHERE qs.public_id = $1 AND q.status = 'approved'"#
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
            .fetch_all(&mut *tx)
            .await?;
        tx.commit().await?;
        rows.into_iter().map(Question::try_from).collect()
    }
}
