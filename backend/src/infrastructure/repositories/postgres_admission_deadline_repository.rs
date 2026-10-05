use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::admission_deadline::{AdmissionDeadline, AdmissionTrack};
use crate::domain::repository::{AdmissionDeadlineRepository, AdmissionDeadlineUpdate, NewAdmissionDeadline};
use crate::error::AppError;
use crate::error::AppResult;

pub struct PostgresAdmissionDeadlineRepository {
    pool: PgPool,
}

impl PostgresAdmissionDeadlineRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AdmissionDeadlineRow {
    id: Uuid,
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
            id: r.id,
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
    SELECT id, track, year, label, description, deadline_date, created_by, created_at, updated_at
    FROM admission_deadlines
"#;

#[async_trait]
impl AdmissionDeadlineRepository for PostgresAdmissionDeadlineRepository {
    async fn create(&self, n: NewAdmissionDeadline) -> AppResult<AdmissionDeadline> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO admission_deadlines
                (id, track, year, label, description, deadline_date, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7)"#,
        )
        .bind(id)
        .bind(track_str(n.track))
        .bind(n.year)
        .bind(&n.label)
        .bind(&n.description)
        .bind(n.deadline_date)
        .bind(n.created_by)
        .execute(&self.pool)
        .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("admission deadline vanished right after insert")))
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<AdmissionDeadline>> {
        let row = sqlx::query_as::<_, AdmissionDeadlineRow>(&format!("{SELECT_DEADLINE} WHERE id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(AdmissionDeadline::try_from).transpose()
    }

    async fn list(&self) -> AppResult<Vec<AdmissionDeadline>> {
        let rows = sqlx::query_as::<_, AdmissionDeadlineRow>(&format!("{SELECT_DEADLINE} ORDER BY deadline_date ASC"))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(AdmissionDeadline::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: AdmissionDeadlineUpdate) -> AppResult<Option<AdmissionDeadline>> {
        let result = sqlx::query(
            r#"UPDATE admission_deadlines SET
                track = $2, year = $3, label = $4, description = $5, deadline_date = $6, updated_at = now()
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(track_str(u.track))
        .bind(u.year)
        .bind(&u.label)
        .bind(&u.description)
        .bind(u.deadline_date)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM admission_deadlines WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
