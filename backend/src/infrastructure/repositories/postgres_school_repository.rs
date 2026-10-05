use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::repository::{NewSchool, SchoolRepository, SchoolUpdate};
use crate::domain::school::School;
use crate::error::{AppError, AppResult};

pub struct PostgresSchoolRepository {
    pool: PgPool,
}

impl PostgresSchoolRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SchoolRow {
    id: Uuid,
    name: String,
    school_type: String,
    city: String,
    province: String,
    email: String,
    phone: String,
    join_date: NaiveDate,
    package_type: String,
    contact_person: String,
    status: String,
    revenue_share: i32,
    monthly_revenue: i64,
    created_at: DateTime<Utc>,
}

impl TryFrom<SchoolRow> for School {
    type Error = AppError;
    fn try_from(r: SchoolRow) -> Result<Self, Self::Error> {
        Ok(School {
            id: r.id,
            name: r.name,
            school_type: r
                .school_type
                .parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt school_type: {e}")))?,
            city: r.city,
            province: r.province,
            email: r.email,
            phone: r.phone,
            join_date: r.join_date,
            package_type: r
                .package_type
                .parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt package_type: {e}")))?,
            contact_person: r.contact_person,
            status: r
                .status
                .parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt school status: {e}")))?,
            revenue_share: r.revenue_share,
            monthly_revenue: r.monthly_revenue,
            created_at: r.created_at,
        })
    }
}

const SELECT_SCHOOL: &str = r#"
    SELECT id, name, school_type, city, province, email, phone, join_date,
           package_type, contact_person, status, revenue_share, monthly_revenue, created_at
    FROM schools
"#;

#[async_trait]
impl SchoolRepository for PostgresSchoolRepository {
    async fn create(&self, n: NewSchool) -> AppResult<School> {
        let id = Uuid::new_v4();
        let row = sqlx::query_as::<_, SchoolRow>(
            r#"INSERT INTO schools
                (id, name, school_type, city, province, email, phone, package_type, contact_person,
                 status, revenue_share, monthly_revenue)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
               RETURNING id, name, school_type, city, province, email, phone, join_date,
                         package_type, contact_person, status, revenue_share, monthly_revenue, created_at"#,
        )
        .bind(id)
        .bind(&n.name)
        .bind(n.school_type.as_str())
        .bind(&n.city)
        .bind(&n.province)
        .bind(&n.email)
        .bind(&n.phone)
        .bind(n.package_type.as_str())
        .bind(&n.contact_person)
        .bind(n.status.as_str())
        .bind(n.revenue_share)
        .bind(n.monthly_revenue)
        .fetch_one(&self.pool)
        .await?;
        School::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<School>> {
        let row = sqlx::query_as::<_, SchoolRow>(&format!("{SELECT_SCHOOL} WHERE id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(School::try_from).transpose()
    }

    async fn list(&self) -> AppResult<Vec<School>> {
        let rows = sqlx::query_as::<_, SchoolRow>(&format!("{SELECT_SCHOOL} ORDER BY created_at DESC"))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(School::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: SchoolUpdate) -> AppResult<Option<School>> {
        let row = sqlx::query_as::<_, SchoolRow>(
            r#"UPDATE schools SET
                name = $2, school_type = $3, city = $4, province = $5, email = $6,
                phone = $7, package_type = $8, contact_person = $9, status = $10,
                revenue_share = $11, monthly_revenue = $12
               WHERE id = $1
               RETURNING id, name, school_type, city, province, email, phone, join_date,
                         package_type, contact_person, status, revenue_share, monthly_revenue, created_at"#,
        )
        .bind(id)
        .bind(&u.name)
        .bind(u.school_type.as_str())
        .bind(&u.city)
        .bind(&u.province)
        .bind(&u.email)
        .bind(&u.phone)
        .bind(u.package_type.as_str())
        .bind(&u.contact_person)
        .bind(u.status.as_str())
        .bind(u.revenue_share)
        .bind(u.monthly_revenue)
        .fetch_optional(&self.pool)
        .await?;
        row.map(School::try_from).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM schools WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::from_sqlx_delete(e, "cannot delete school: it is still referenced elsewhere"))?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_status(&self, id: Uuid, status: crate::domain::school::SchoolStatus) -> AppResult<Option<School>> {
        let result = sqlx::query("UPDATE schools SET status = $2 WHERE id = $1")
            .bind(id)
            .bind(status.as_str())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }
}
