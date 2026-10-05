use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{types::Json, PgPool, QueryBuilder};
use uuid::Uuid;

use crate::domain::question::{Difficulty, Question, QuestionStatus};
use crate::domain::repository::{NewQuestion, QuestionFilter, QuestionRepository, QuestionUpdate};
use crate::domain::tryout::SessionType;
use crate::error::{AppError, AppResult};

pub struct PostgresQuestionRepository {
    pool: PgPool,
}

impl PostgresQuestionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct QuestionRow {
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

impl TryFrom<QuestionRow> for Question {
    type Error = AppError;

    fn try_from(r: QuestionRow) -> Result<Self, Self::Error> {
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

// `created_by_name` is resolved via a scalar subquery (rather than a JOIN) so this same
// column list works unmodified in both plain SELECTs and INSERT/UPDATE ... RETURNING
// clauses, where a JOIN against another table isn't available.
const SELECT_COLUMNS: &str = r#"
    id, code, question_type, subject, topic, subtopic, difficulty, bloom_level, stimulus,
    question_text, options, correct_answer, explanation, tags, status, review_note,
    reviewed_by, reviewed_at, created_by,
    (SELECT name FROM users WHERE users.id = questions.created_by) AS created_by_name,
    usage_count, average_score, time_limit, created_at, updated_at
"#;

#[async_trait]
impl QuestionRepository for PostgresQuestionRepository {
    async fn create(&self, n: NewQuestion) -> AppResult<Question> {
        let id = Uuid::new_v4();
        let options_json = n.options.map(Json);

        let row = sqlx::query_as::<_, QuestionRow>(&format!(
            r#"INSERT INTO questions
                (id, code, question_type, subject, topic, subtopic, difficulty, bloom_level,
                 stimulus, question_text, options, correct_answer, explanation, tags, status, created_by, time_limit)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
               RETURNING {SELECT_COLUMNS}"#
        ))
        .bind(id)
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
        .bind(n.created_by)
        .bind(n.time_limit)
        .fetch_one(&self.pool)
        .await?;

        Question::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Question>> {
        let row = sqlx::query_as::<_, QuestionRow>(&format!(
            "SELECT {SELECT_COLUMNS} FROM questions WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Question::try_from).transpose()
    }

    async fn list(&self, filter: QuestionFilter) -> AppResult<(Vec<Question>, i64)> {
        let page = filter.page.max(1);
        let page_size = filter.page_size.clamp(1, 100);
        let offset = (page - 1) * page_size;

        let mut qb = QueryBuilder::new(format!("SELECT {SELECT_COLUMNS} FROM questions WHERE 1=1"));
        let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM questions WHERE 1=1");

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
            qb.push(" AND created_by = ").push_bind(created_by);
            count_qb.push(" AND created_by = ").push_bind(created_by);
        }

        qb.push(" ORDER BY created_at DESC LIMIT ").push_bind(page_size).push(" OFFSET ").push_bind(offset);

        let rows: Vec<QuestionRow> = qb.build_query_as().fetch_all(&self.pool).await?;
        let total: i64 = count_qb.build_query_scalar().fetch_one(&self.pool).await?;

        let questions = rows.into_iter().map(Question::try_from).collect::<AppResult<Vec<_>>>()?;
        Ok((questions, total))
    }

    async fn update(&self, id: Uuid, u: QuestionUpdate) -> AppResult<Option<Question>> {
        let options_json = u.options.map(Json);
        let row = sqlx::query_as::<_, QuestionRow>(&format!(
            r#"UPDATE questions SET
                question_type = $2, subject = $3, topic = $4, subtopic = $5, difficulty = $6,
                bloom_level = $7, stimulus = $8, question_text = $9, options = $10,
                correct_answer = $11, explanation = $12, tags = $13, time_limit = $14,
                updated_at = now()
               WHERE id = $1
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
        .fetch_optional(&self.pool)
        .await?;

        row.map(Question::try_from).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM questions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_review_status(
        &self,
        id: Uuid,
        status: QuestionStatus,
        review_note: Option<String>,
        reviewed_by: Uuid,
    ) -> AppResult<Option<Question>> {
        let row = sqlx::query_as::<_, QuestionRow>(&format!(
            r#"UPDATE questions SET status = $2, review_note = $3, reviewed_by = $4,
                reviewed_at = now(), updated_at = now()
               WHERE id = $1
               RETURNING {SELECT_COLUMNS}"#
        ))
        .bind(id)
        .bind(status.as_str())
        .bind(&review_note)
        .bind(reviewed_by)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Question::try_from).transpose()
    }

    async fn resubmit(&self, id: Uuid) -> AppResult<Option<Question>> {
        let row = sqlx::query_as::<_, QuestionRow>(&format!(
            r#"UPDATE questions SET status = 'review', review_note = NULL, reviewed_by = NULL,
                reviewed_at = NULL, updated_at = now()
               WHERE id = $1
               RETURNING {SELECT_COLUMNS}"#
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Question::try_from).transpose()
    }

    async fn random_approved(
        &self,
        subject: Option<&str>,
        topic: Option<&str>,
        difficulty: Option<Difficulty>,
        count: i64,
        session_type: SessionType,
    ) -> AppResult<Vec<Question>> {
        let mut qb = QueryBuilder::new(format!(
            "SELECT {SELECT_COLUMNS} FROM questions WHERE status = 'approved'"
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

        let rows: Vec<QuestionRow> = qb.build_query_as().fetch_all(&self.pool).await?;
        rows.into_iter().map(Question::try_from).collect()
    }
}
