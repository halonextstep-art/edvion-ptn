use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::admission_deadline::{AdmissionDeadline, AdmissionTrack};
use crate::domain::repository::{AdmissionDeadlineRepository, AdmissionDeadlineUpdate, NewAdmissionDeadline};
use crate::domain::tenant::TenantCtx;
use crate::error::AppError;
use crate::error::AppResult;

pub struct PostgresAdmissionDeadlineRepository {
    pool: PgPool,
}

impl PostgresAdmissionDeadlineRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn begin_tx(&self, ctx: &TenantCtx) -> AppResult<Transaction<'static, Postgres>> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.tenant_id', $1, true), set_config('app.is_platform_admin', $2, true)")
            .bind(ctx.tenant_id.to_string())
            .bind(if ctx.is_platform_admin { "true" } else { "false" })
            .execute(&mut *tx)
            .await?;
        Ok(tx)
    }
}

#[derive(sqlx::FromRow)]
struct AdmissionDeadlineRow {
    public_id: Uuid,
    track: String,
    year: i32,
    label: String,
    description: String,
    deadline_date: NaiveDate,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<AdmissionDeadlineRow> for AdmissionDeadline {
    type Error = AppError;
    fn try_from(r: AdmissionDeadlineRow) -> Result<Self, Self::Error> {
        Ok(AdmissionDeadline {
            id: r.public_id,
            track: r
                .track
                .parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt admission track: {e}")))?,
            year: r.year,
            label: r.label,
            description: r.description,
            deadline_date: r.deadline_date,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

fn track_str(t: AdmissionTrack) -> &'static str {
    t.as_str()
}

const SELECT_DEADLINE: &str = r#"
    SELECT d.public_id, d.track, d.year, d.label, d.description, d.deadline_date,
           (SELECT public_id FROM core.akun WHERE core.akun.id = d.created_by) AS created_by,
           d.created_at, d.updated_at
    FROM ptn.admission_deadlines d
"#;

#[async_trait]
impl AdmissionDeadlineRepository for PostgresAdmissionDeadlineRepository {
    async fn create(&self, n: NewAdmissionDeadline, ctx: &TenantCtx) -> AppResult<AdmissionDeadline> {
        let mut tx = self.begin_tx(ctx).await?;

        let created_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(n.created_by)
            .fetch_one(&mut *tx)
            .await?;

        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.admission_deadlines
                (tenant_id, track, year, label, description, deadline_date, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7)
               RETURNING public_id"#,
        )
        .bind(ctx.tenant_id)
        .bind(track_str(n.track))
        .bind(n.year)
        .bind(&n.label)
        .bind(&n.description)
        .bind(n.deadline_date)
        .bind(created_by_id)
        .fetch_one(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, AdmissionDeadlineRow>(&format!("{SELECT_DEADLINE} WHERE d.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        AdmissionDeadline::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<AdmissionDeadline>> {
        let mut tx = self.begin_tx(ctx).await?;
        let row = sqlx::query_as::<_, AdmissionDeadlineRow>(&format!("{SELECT_DEADLINE} WHERE d.public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(AdmissionDeadline::try_from).transpose()
    }

    async fn list(&self, ctx: &TenantCtx) -> AppResult<Vec<AdmissionDeadline>> {
        let mut tx = self.begin_tx(ctx).await?;
        let rows = sqlx::query_as::<_, AdmissionDeadlineRow>(&format!("{SELECT_DEADLINE} ORDER BY d.deadline_date ASC"))
            .fetch_all(&mut *tx)
            .await?;
        tx.commit().await?;
        rows.into_iter().map(AdmissionDeadline::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: AdmissionDeadlineUpdate, ctx: &TenantCtx) -> AppResult<Option<AdmissionDeadline>> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query(
            r#"UPDATE ptn.admission_deadlines SET
                track = $2, year = $3, label = $4, description = $5, deadline_date = $6, updated_at = now()
               WHERE public_id = $1"#,
        )
        .bind(id)
        .bind(track_str(u.track))
        .bind(u.year)
        .bind(&u.label)
        .bind(&u.description)
        .bind(u.deadline_date)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, AdmissionDeadlineRow>(&format!("{SELECT_DEADLINE} WHERE d.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        AdmissionDeadline::try_from(row).map(Some)
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("DELETE FROM ptn.admission_deadlines WHERE public_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }
}
