//! Postgres implementation of `ReportRepository` — `ptn.school_reports` carries
//! `tenant_id`/RLS, so every method opens its own `begin_tx` + GUC transaction (same
//! pattern as `postgres_user_repository::PostgresUserRepository::begin_tx`). `school_id`
//! is a bigint FK to `core.tenant`, resolved via `core.tenant.public_id`.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{types::Json, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::package::ExamTrack;
use crate::domain::report::{Report, ReportPayload, ReportType};
use crate::domain::repository::{NewReport, ReportRepository};
use crate::domain::tenant::TenantCtx;
use crate::error::{AppError, AppResult};

pub struct PostgresReportRepository {
    pool: PgPool,
}

impl PostgresReportRepository {
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
struct ReportRow {
    public_id: Uuid,
    school_id: Uuid,
    report_type: String,
    title: String,
    period_label: String,
    class_filter: Option<String>,
    exam_track_filter: Option<ExamTrack>,
    payload: Json<ReportPayload>,
    created_at: DateTime<Utc>,
}

impl TryFrom<ReportRow> for Report {
    type Error = AppError;
    fn try_from(r: ReportRow) -> Result<Self, Self::Error> {
        Ok(Report {
            id: r.public_id,
            school_id: r.school_id,
            report_type: r
                .report_type
                .parse::<ReportType>()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt report_type: {e}")))?,
            title: r.title,
            period_label: r.period_label,
            class_filter: r.class_filter,
            exam_track_filter: r.exam_track_filter,
            payload: r.payload.0,
            created_at: r.created_at,
        })
    }
}

const SELECT_REPORT: &str = r#"
    SELECT sr.public_id, t.public_id AS school_id, sr.report_type, sr.title, sr.period_label,
           sr.class_filter, sr.exam_track_filter, sr.payload, sr.created_at
    FROM ptn.school_reports sr
    JOIN core.tenant t ON t.id = sr.school_id
"#;

#[async_trait]
impl ReportRepository for PostgresReportRepository {
    async fn create(&self, input: NewReport, ctx: &TenantCtx) -> AppResult<Report> {
        let mut tx = self.begin_tx(ctx).await?;
        let school_pk: i64 = sqlx::query_scalar("SELECT id FROM core.tenant WHERE public_id = $1")
            .bind(input.school_id)
            .fetch_one(&mut *tx)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.school_reports (school_id, report_type, title, period_label, class_filter, exam_track_filter, payload)
               VALUES ($1,$2,$3,$4,$5,$6,$7)
               RETURNING public_id"#,
        )
        .bind(school_pk)
        .bind(input.report_type.as_str())
        .bind(&input.title)
        .bind(&input.period_label)
        .bind(&input.class_filter)
        .bind(input.exam_track_filter)
        .bind(Json(input.payload))
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as::<_, ReportRow>(&format!("{SELECT_REPORT} WHERE sr.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Report::try_from(row)
    }

    async fn list_by_school(&self, school_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<Report>> {
        let mut tx = self.begin_tx(ctx).await?;
        let rows = sqlx::query_as::<_, ReportRow>(&format!(
            "{SELECT_REPORT} WHERE t.public_id = $1 ORDER BY sr.created_at DESC"
        ))
        .bind(school_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows.into_iter().map(Report::try_from).collect()
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<Report>> {
        let mut tx = self.begin_tx(ctx).await?;
        let row = sqlx::query_as::<_, ReportRow>(&format!("{SELECT_REPORT} WHERE sr.public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(Report::try_from).transpose()
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<()> {
        let mut tx = self.begin_tx(ctx).await?;
        sqlx::query("DELETE FROM ptn.school_reports WHERE public_id = $1").bind(id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
}
