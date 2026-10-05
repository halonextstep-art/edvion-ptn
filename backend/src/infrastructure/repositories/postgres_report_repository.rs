use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{types::Json, PgPool};
use uuid::Uuid;

use crate::domain::package::ExamTrack;
use crate::domain::report::{Report, ReportPayload, ReportType};
use crate::domain::repository::{NewReport, ReportRepository};
use crate::error::{AppError, AppResult};

pub struct PostgresReportRepository {
    pool: PgPool,
}

impl PostgresReportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ReportRow {
    id: Uuid,
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
            id: r.id,
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

const SELECT_REPORT: &str =
    "SELECT id, school_id, report_type, title, period_label, class_filter, exam_track_filter, payload, created_at FROM school_reports";

#[async_trait]
impl ReportRepository for PostgresReportRepository {
    async fn create(&self, input: NewReport) -> AppResult<Report> {
        let row = sqlx::query_as::<_, ReportRow>(
            r#"INSERT INTO school_reports (school_id, report_type, title, period_label, class_filter, exam_track_filter, payload)
               VALUES ($1,$2,$3,$4,$5,$6,$7)
               RETURNING id, school_id, report_type, title, period_label, class_filter, exam_track_filter, payload, created_at"#,
        )
        .bind(input.school_id)
        .bind(input.report_type.as_str())
        .bind(&input.title)
        .bind(&input.period_label)
        .bind(&input.class_filter)
        .bind(input.exam_track_filter)
        .bind(Json(input.payload))
        .fetch_one(&self.pool)
        .await?;
        Report::try_from(row)
    }

    async fn list_by_school(&self, school_id: Uuid) -> AppResult<Vec<Report>> {
        let rows = sqlx::query_as::<_, ReportRow>(&format!(
            "{SELECT_REPORT} WHERE school_id = $1 ORDER BY created_at DESC"
        ))
        .bind(school_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Report::try_from).collect()
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Report>> {
        let row = sqlx::query_as::<_, ReportRow>(&format!("{SELECT_REPORT} WHERE id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(Report::try_from).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM school_reports WHERE id = $1").bind(id).execute(&self.pool).await?;
        Ok(())
    }
}
