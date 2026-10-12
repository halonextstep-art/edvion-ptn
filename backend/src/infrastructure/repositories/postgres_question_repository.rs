use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{types::Json, PgPool, Postgres, QueryBuilder, Transaction};
use uuid::Uuid;

use crate::domain::question::{Difficulty, Question, QuestionStatus};
use crate::domain::repository::{NewQuestion, QuestionFilter, QuestionRepository, QuestionUpdate};
use crate::domain::tenant::TenantCtx;
use crate::domain::tryout::SessionType;
use crate::error::{AppError, AppResult};

pub struct PostgresQuestionRepository {
    pool: PgPool,
}

impl PostgresQuestionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Opens this call's transaction and sets the RLS GUCs on it — see
    /// `postgres_user_repository.rs::begin_tx` doc comment for why `true` (transaction-local)
    /// matters on a pooled connection.
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

#[derive(sqlx::FromRow)]
struct QuestionRow {
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

impl TryFrom<QuestionRow> for Question {
    type Error = AppError;

    fn try_from(r: QuestionRow) -> Result<Self, Self::Error> {
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

// `created_by`/`reviewed_by` are bigint FKs to `core.akun(id)` physically, but the domain
// `Question` struct keeps exposing them (and its own `id`) as `Uuid` (API wire format is
// unchanged) — resolved here via scalar subqueries against `core.akun.public_id`/
// `ptn.questions.public_id`. This column list is shared by plain SELECTs and
// INSERT/UPDATE ... RETURNING alike (same pattern as the pre-migration version), so every
// subquery is written against `ptn.questions` column names directly rather than a join
// alias, since RETURNING has no join available.
const SELECT_COLUMNS: &str = r#"
    public_id, code, question_type, subject, topic, subtopic, difficulty, bloom_level, stimulus,
    question_text, options, correct_answer, explanation, tags, status, review_note,
    (SELECT public_id FROM core.akun WHERE core.akun.id = ptn.questions.reviewed_by) AS reviewed_by_public_id,
    reviewed_at,
    (SELECT public_id FROM core.akun WHERE core.akun.id = ptn.questions.created_by) AS created_by_public_id,
    (SELECT full_name FROM core.akun WHERE core.akun.id = ptn.questions.created_by) AS created_by_name,
    usage_count, average_score, time_limit, created_at, updated_at
"#;

#[async_trait]
impl QuestionRepository for PostgresQuestionRepository {
    async fn create(&self, n: NewQuestion, ctx: &TenantCtx) -> AppResult<Question> {
        let mut tx = self.begin_tx(ctx).await?;
        let options_json = n.options.map(Json);

        let created_by: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(n.created_by)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("created_by lookup failed: {e}")))?;

        let row = sqlx::query_as::<_, QuestionRow>(&format!(
            r#"INSERT INTO ptn.questions
                (code, question_type, subject, topic, subtopic, difficulty, bloom_level,
                 stimulus, question_text, options, correct_answer, explanation, tags, status, created_by, time_limit)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
               RETURNING {SELECT_COLUMNS}"#
        ))
        .bind(&n.code)
        .bind(n.question_type.as_str())
        .bind(&n.subject)
        .bind(&n.topic)
        .bind(&n.subtopic)
        .bind(n.difficulty.as_str())
        .bind(&n.bloom_level)
        .bind(&n.stimulus)
        .bind(&n.question_text)
        .bind(options_json)
        .bind(&n.correct_answer)
        .bind(&n.explanation)
        .bind(&n.tags)
        .bind(n.status.as_str())
        .bind(created_by)
        .bind(n.time_limit)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Question::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<Question>> {
        let mut tx = self.begin_tx(ctx).await?;
        let row = sqlx::query_as::<_, QuestionRow>(&format!(
            "SELECT {SELECT_COLUMNS} FROM ptn.questions WHERE public_id = $1"
        ))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        row.map(Question::try_from).transpose()
    }

    async fn list(&self, filter: QuestionFilter, ctx: &TenantCtx) -> AppResult<(Vec<Question>, i64)> {
        let mut tx = self.begin_tx(ctx).await?;
        let page = filter.page.max(1);
        let page_size = filter.page_size.clamp(1, 100);
        let offset = (page - 1) * page_size;

        let mut qb = QueryBuilder::new(format!("SELECT {SELECT_COLUMNS} FROM ptn.questions WHERE 1=1"));
        let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM ptn.questions WHERE 1=1");

        if let Some(search) = &filter.search {
            let pattern = format!("%{}%", search);
            qb.push(" AND (code ILIKE ").push_bind(pattern.clone())
                .push(" OR question_text ILIKE ").push_bind(pattern.clone())
                .push(" OR subject ILIKE ").push_bind(pattern.clone()).push(")");
            count_qb.push(" AND (code ILIKE ").push_bind(pattern.clone())
                .push(" OR question_text ILIKE ").push_bind(pattern.clone())
                .push(" OR subject ILIKE ").push_bind(pattern).push(")");
        }
        if let Some(subject) = &filter.subject {
            qb.push(" AND subject = ").push_bind(subject.clone());
            count_qb.push(" AND subject = ").push_bind(subject.clone());
        }
        if let Some(status) = filter.status {
            qb.push(" AND status = ").push_bind(status.as_str());
            count_qb.push(" AND status = ").push_bind(status.as_str());
        }
        if let Some(difficulty) = filter.difficulty {
            qb.push(" AND difficulty = ").push_bind(difficulty.as_str());
            count_qb.push(" AND difficulty = ").push_bind(difficulty.as_str());
        }
        if let Some(question_type) = filter.question_type {
            qb.push(" AND question_type = ").push_bind(question_type.as_str());
            count_qb.push(" AND question_type = ").push_bind(question_type.as_str());
        }
        if let Some(created_by) = filter.created_by {
            qb.push(" AND created_by = (SELECT id FROM core.akun WHERE public_id = ").push_bind(created_by).push(")");
            count_qb.push(" AND created_by = (SELECT id FROM core.akun WHERE public_id = ").push_bind(created_by).push(")");
        }

        qb.push(" ORDER BY created_at DESC LIMIT ").push_bind(page_size).push(" OFFSET ").push_bind(offset);

        let rows: Vec<QuestionRow> = qb.build_query_as().fetch_all(&mut *tx).await?;
        let total: i64 = count_qb.build_query_scalar().fetch_one(&mut *tx).await?;
        tx.commit().await?;

        let questions = rows.into_iter().map(Question::try_from).collect::<AppResult<Vec<_>>>()?;
        Ok((questions, total))
    }

    async fn update(&self, id: Uuid, u: QuestionUpdate, ctx: &TenantCtx) -> AppResult<Option<Question>> {
        let mut tx = self.begin_tx(ctx).await?;
        let options_json = u.options.map(Json);
        let row = sqlx::query_as::<_, QuestionRow>(&format!(
            r#"UPDATE ptn.questions SET
                question_type = $2, subject = $3, topic = $4, subtopic = $5, difficulty = $6,
                bloom_level = $7, stimulus = $8, question_text = $9, options = $10,
                correct_answer = $11, explanation = $12, tags = $13, time_limit = $14,
                updated_at = now()
               WHERE public_id = $1
               RETURNING {SELECT_COLUMNS}"#
        ))
        .bind(id)
        .bind(u.question_type.as_str())
        .bind(&u.subject)
        .bind(&u.topic)
        .bind(&u.subtopic)
        .bind(u.difficulty.as_str())
        .bind(&u.bloom_level)
        .bind(&u.stimulus)
        .bind(&u.question_text)
        .bind(options_json)
        .bind(&u.correct_answer)
        .bind(&u.explanation)
        .bind(&u.tags)
        .bind(u.time_limit)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        row.map(Question::try_from).transpose()
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("DELETE FROM ptn.questions WHERE public_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_review_status(
        &self,
        id: Uuid,
        status: QuestionStatus,
        review_note: Option<String>,
        reviewed_by: Uuid,
        ctx: &TenantCtx,
    ) -> AppResult<Option<Question>> {
        let mut tx = self.begin_tx(ctx).await?;

        let reviewed_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(reviewed_by)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("reviewed_by lookup failed: {e}")))?;

        let row = sqlx::query_as::<_, QuestionRow>(&format!(
            r#"UPDATE ptn.questions SET status = $2, review_note = $3, reviewed_by = $4,
                reviewed_at = now(), updated_at = now()
               WHERE public_id = $1
               RETURNING {SELECT_COLUMNS}"#
        ))
        .bind(id)
        .bind(status.as_str())
        .bind(&review_note)
        .bind(reviewed_by_id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        row.map(Question::try_from).transpose()
    }

    async fn resubmit(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<Question>> {
        let mut tx = self.begin_tx(ctx).await?;
        let row = sqlx::query_as::<_, QuestionRow>(&format!(
            r#"UPDATE ptn.questions SET status = 'review', review_note = NULL, reviewed_by = NULL,
                reviewed_at = NULL, updated_at = now()
               WHERE public_id = $1
               RETURNING {SELECT_COLUMNS}"#
        ))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        row.map(Question::try_from).transpose()
    }

    async fn random_approved(
        &self,
        subject: Option<&str>,
        topic: Option<&str>,
        difficulty: Option<Difficulty>,
        count: i64,
        session_type: SessionType,
        ctx: &TenantCtx,
    ) -> AppResult<Vec<Question>> {
        let mut tx = self.begin_tx(ctx).await?;
        let mut qb = QueryBuilder::new(format!(
            "SELECT {SELECT_COLUMNS} FROM ptn.questions WHERE status = 'approved'"
        ));
        if let Some(subject) = subject {
            qb.push(" AND subject = ").push_bind(subject.to_string());
        }
        if let Some(topic) = topic {
            qb.push(" AND topic = ").push_bind(topic.to_string());
        }
        if let Some(difficulty) = difficulty {
            qb.push(" AND difficulty = ").push_bind(difficulty.as_str());
        }
        // `essay` is self-check/Drilling-only — it's never auto-graded (see QuestionType::Essay
        // doc comment), so Tryout/Mini attempts must never draw one.
        if !matches!(session_type, SessionType::Drilling) {
            qb.push(" AND question_type != 'essay'");
        }
        qb.push(" ORDER BY random() LIMIT ").push_bind(count);

        let rows: Vec<QuestionRow> = qb.build_query_as().fetch_all(&mut *tx).await?;
        tx.commit().await?;
        rows.into_iter().map(Question::try_from).collect()
    }
}
