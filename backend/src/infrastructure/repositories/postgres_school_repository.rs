use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::repository::{NewSchool, SchoolRepository, SchoolUpdate};
use crate::domain::school::{School, SchoolStatus};
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
    public_id: Uuid,
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
            id: r.public_id,
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

// `School` = `core.tenant` (name/slug, shared with other modules) 1:1-joined with
// `ptn.school_profile` (PTN-only business columns). Neither table carries RLS (`core.tenant`
// has no policy at all; `school_profile` is keyed 1:1 by `tenant_id`, its own PK, so there's
// nothing to scope) — this repository is always admin-facing (see callers in
// `school_service.rs`/`finance_service.rs`), never called with a per-tenant `TenantCtx`.
const SELECT_SCHOOL: &str = r#"
    SELECT t.public_id, t.name, sp.school_type, sp.city, sp.province, sp.email, sp.phone,
           sp.join_date, sp.package_type, sp.contact_person, sp.status, sp.revenue_share,
           sp.monthly_revenue, sp.created_at
    FROM core.tenant t
    JOIN ptn.school_profile sp ON sp.tenant_id = t.id
"#;

#[async_trait]
impl SchoolRepository for PostgresSchoolRepository {
    async fn create(&self, n: NewSchool) -> AppResult<School> {
        let mut tx = self.pool.begin().await?;

        let tenant_id: i64 = sqlx::query_scalar(
            r#"INSERT INTO core.tenant (name, slug) VALUES ($1, $2) RETURNING id"#,
        )
        .bind(&n.name)
        .bind(slugify(&n.name))
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"INSERT INTO ptn.school_profile
                (tenant_id, school_type, city, province, email, phone, package_type, contact_person,
                 status, revenue_share, monthly_revenue)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"#,
        )
        .bind(tenant_id)
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
        .execute(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, SchoolRow>(&format!("{SELECT_SCHOOL} WHERE t.id = $1"))
            .bind(tenant_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        School::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<School>> {
        let row = sqlx::query_as::<_, SchoolRow>(&format!("{SELECT_SCHOOL} WHERE t.public_id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(School::try_from).transpose()
    }

    async fn list(&self) -> AppResult<Vec<School>> {
        let rows = sqlx::query_as::<_, SchoolRow>(&format!("{SELECT_SCHOOL} ORDER BY sp.created_at DESC"))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(School::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: SchoolUpdate) -> AppResult<Option<School>> {
        let mut tx = self.pool.begin().await?;

        let tenant_id: Option<i64> = sqlx::query_scalar("SELECT id FROM core.tenant WHERE public_id = $1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        let Some(tenant_id) = tenant_id else {
            tx.rollback().await.ok();
            return Ok(None);
        };

        sqlx::query("UPDATE core.tenant SET name = $2 WHERE id = $1")
            .bind(tenant_id)
            .bind(&u.name)
            .execute(&mut *tx)
            .await?;

        let updated = sqlx::query(
            r#"UPDATE ptn.school_profile SET
                school_type = $2, city = $3, province = $4, email = $5,
                phone = $6, package_type = $7, contact_person = $8, status = $9,
                revenue_share = $10, monthly_revenue = $11
               WHERE tenant_id = $1"#,
        )
        .bind(tenant_id)
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
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }

        let row = sqlx::query_as::<_, SchoolRow>(&format!("{SELECT_SCHOOL} WHERE t.id = $1"))
            .bind(tenant_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        School::try_from(row).map(Some)
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM core.tenant WHERE public_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::from_sqlx_delete(e, "cannot delete school: it is still referenced elsewhere"))?;
        Ok(result.rows_affected() > 0)
    }

    async fn set_status(&self, id: Uuid, status: SchoolStatus) -> AppResult<Option<School>> {
        let result = sqlx::query(
            r#"UPDATE ptn.school_profile SET status = $2
               WHERE tenant_id = (SELECT id FROM core.tenant WHERE public_id = $1)"#,
        )
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

/// Lowercase/hyphenate the school name into a `core.tenant.slug` candidate, suffixed with a
/// short random disambiguator (slugs are globally unique across every module's tenants, and
/// school names routinely collide — "SMA Negeri 1").
fn slugify(name: &str) -> String {
    let base: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let suffix: String = Uuid::new_v4().to_string().chars().take(8).collect();
    format!("{base}-{suffix}")
}
