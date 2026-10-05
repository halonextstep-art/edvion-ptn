use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::repository::{QuestionIrtParam, QuestionIrtParamRepository};
use crate::error::AppResult;

pub struct PostgresQuestionIrtParamRepository {
    pool: PgPool,
}

impl PostgresQuestionIrtParamRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct QuestionIrtParamRow {
    question_id: Uuid,
    difficulty_b: f64,
    sample_size: i64,
    calibrated_at: DateTime<Utc>,
}

impl From<QuestionIrtParamRow> for QuestionIrtParam {
    fn from(r: QuestionIrtParamRow) -> Self {
        QuestionIrtParam { question_id: r.question_id, difficulty_b: r.difficulty_b, sample_size: r.sample_size, calibrated_at: r.calibrated_at }
    }
}

#[async_trait]
impl QuestionIrtParamRepository for PostgresQuestionIrtParamRepository {
    async fn upsert_many(&self, params: Vec<(Uuid, f64, i64)>) -> AppResult<()> {
        // Plain per-row upsert inside one transaction — the calibration batch (bounded by the
        // platform's total distinct-question count, not attempt volume) is small enough that a
        // bulk UNNEST query isn't worth the added complexity here.
        let mut tx = self.pool.begin().await?;
        for (question_id, difficulty_b, sample_size) in params {
            sqlx::query(
                r#"INSERT INTO question_irt_params (question_id, difficulty_b, sample_size, calibrated_at)
                   VALUES ($1, $2, $3, now())
                   ON CONFLICT (question_id)
                   DO UPDATE SET difficulty_b = EXCLUDED.difficulty_b, sample_size = EXCLUDED.sample_size, calibrated_at = now()"#,
            )
            .bind(question_id)
            .bind(difficulty_b)
            .bind(sample_size)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn find_by_question_ids(&self, question_ids: &[Uuid]) -> AppResult<Vec<QuestionIrtParam>> {
        if question_ids.is_empty() {
            return Ok(Vec::new());
        }
        let rows = sqlx::query_as::<_, QuestionIrtParamRow>(
            "SELECT question_id, difficulty_b, sample_size, calibrated_at FROM question_irt_params WHERE question_id = ANY($1)",
        )
        .bind(question_ids)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn calibration_summary(&self) -> AppResult<Option<(i64, DateTime<Utc>)>> {
        let row: Option<(i64, Option<DateTime<Utc>>)> =
            sqlx::query_as("SELECT COUNT(*), MAX(calibrated_at) FROM question_irt_params")
                .fetch_optional(&self.pool)
                .await?;
        Ok(match row {
            Some((count, Some(latest))) if count > 0 => Some((count, latest)),
            _ => None,
        })
    }
}
