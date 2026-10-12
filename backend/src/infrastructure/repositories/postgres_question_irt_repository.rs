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

// `ptn.question_irt_params` carries no `tenant_id`/RLS of its own — its PK IS
// `question_id` (1:1 with `ptn.questions`, no independent identity) — but every method
// here JOINs to or looks up `ptn.questions`, which DOES carry RLS. IRT calibration is a
// genuinely platform-wide computation over every historical attempt regardless of tenant
// (see `AttemptRepository::item_response_stats` doc comment), so every method opens its
// own transaction with `app.is_platform_admin = true` explicitly — relying on ambient GUC
// state from whatever connection the pool happens to hand back is NOT safe: a
// transaction-local `set_config` resets to an EMPTY STRING (not "unset") on commit, and an
// empty string fed into `current_setting(...)::bigint` inside another table's RLS check
// is a hard Postgres error, not a harmless default.
#[async_trait]
impl QuestionIrtParamRepository for PostgresQuestionIrtParamRepository {
    async fn upsert_many(&self, params: Vec<(Uuid, f64, i64)>) -> AppResult<()> {
        // Plain per-row upsert inside one transaction — the calibration batch (bounded by the
        // platform's total distinct-question count, not attempt volume) is small enough that a
        // bulk UNNEST query isn't worth the added complexity here.
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.tenant_id', '0', true), set_config('app.is_platform_admin', 'true', true)")
            .execute(&mut *tx)
            .await?;
        for (question_public_id, difficulty_b, sample_size) in params {
            let question_id: i64 = sqlx::query_scalar("SELECT id FROM ptn.questions WHERE public_id = $1")
                .bind(question_public_id)
                .fetch_one(&mut *tx)
                .await?;
            sqlx::query(
                r#"INSERT INTO ptn.question_irt_params (question_id, difficulty_b, sample_size, calibrated_at)
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
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.tenant_id', '0', true), set_config('app.is_platform_admin', 'true', true)")
            .execute(&mut *tx)
            .await?;
        let rows = sqlx::query_as::<_, QuestionIrtParamRow>(
            r#"SELECT q.public_id AS question_id, p.difficulty_b, p.sample_size, p.calibrated_at
               FROM ptn.question_irt_params p
               JOIN ptn.questions q ON q.id = p.question_id
               WHERE q.public_id = ANY($1)"#,
        )
        .bind(question_ids)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn calibration_summary(&self) -> AppResult<Option<(i64, DateTime<Utc>)>> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.tenant_id', '0', true), set_config('app.is_platform_admin', 'true', true)")
            .execute(&mut *tx)
            .await?;
        let row: Option<(i64, Option<DateTime<Utc>>)> =
            sqlx::query_as("SELECT COUNT(*), MAX(calibrated_at) FROM ptn.question_irt_params")
                .fetch_optional(&mut *tx)
                .await?;
        tx.commit().await?;
        Ok(match row {
            Some((count, Some(latest))) if count > 0 => Some((count, latest)),
            _ => None,
        })
    }
}
