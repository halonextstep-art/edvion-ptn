use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entitlement::SchoolPackageEntitlement;
use crate::domain::repository::{NewSchoolEntitlement, SchoolEntitlementRepository};
use crate::error::AppResult;

pub struct PostgresSchoolEntitlementRepository {
    pool: PgPool,
}

impl PostgresSchoolEntitlementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct EntitlementRow {
    id: Uuid,
    school_id: Uuid,
    school_name: String,
    package_id: Uuid,
    package_name: String,
    starts_at: NaiveDate,
    expires_at: Option<NaiveDate>,
    note: Option<String>,
    granted_by: Uuid,
    granted_by_name: String,
    created_at: DateTime<Utc>,
}

impl From<EntitlementRow> for SchoolPackageEntitlement {
    fn from(r: EntitlementRow) -> Self {
        SchoolPackageEntitlement {
            id: r.id,
            school_id: r.school_id,
            school_name: r.school_name,
            package_id: r.package_id,
            package_name: r.package_name,
            starts_at: r.starts_at,
            expires_at: r.expires_at,
            note: r.note,
            granted_by: r.granted_by,
            granted_by_name: r.granted_by_name,
            created_at: r.created_at,
        }
    }
}

const SELECT_ENTITLEMENT: &str = r#"
    SELECT e.id, e.school_id, s.name AS school_name, e.package_id, p.name AS package_name,
           e.starts_at, e.expires_at, e.note, e.granted_by, u.name AS granted_by_name, e.created_at
    FROM school_package_entitlements e
    JOIN schools s ON s.id = e.school_id
    JOIN packages p ON p.id = e.package_id
    JOIN users u ON u.id = e.granted_by
"#;

#[async_trait]
impl SchoolEntitlementRepository for PostgresSchoolEntitlementRepository {
    async fn create(&self, input: NewSchoolEntitlement) -> AppResult<SchoolPackageEntitlement> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO school_package_entitlements
                (id, school_id, package_id, starts_at, expires_at, note, granted_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7)"#,
        )
        .bind(id)
        .bind(input.school_id)
        .bind(input.package_id)
        .bind(input.starts_at)
        .bind(input.expires_at)
        .bind(&input.note)
        .bind(input.granted_by)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, EntitlementRow>(&format!("{SELECT_ENTITLEMENT} WHERE e.id = $1"))
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.into())
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SchoolPackageEntitlement>> {
        let row = sqlx::query_as::<_, EntitlementRow>(&format!("{SELECT_ENTITLEMENT} WHERE e.id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Into::into))
    }

    async fn list_by_school(&self, school_id: Uuid) -> AppResult<Vec<SchoolPackageEntitlement>> {
        let rows = sqlx::query_as::<_, EntitlementRow>(&format!(
            "{SELECT_ENTITLEMENT} WHERE e.school_id = $1 ORDER BY e.created_at DESC"
        ))
        .bind(school_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM school_package_entitlements WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn has_active_entitlement(&self, school_id: Uuid, today: NaiveDate) -> AppResult<bool> {
        let exists: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(
                SELECT 1 FROM school_package_entitlements
                WHERE school_id = $1 AND starts_at <= $2 AND (expires_at IS NULL OR expires_at >= $2)
            )"#,
        )
        .bind(school_id)
        .bind(today)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    async fn active_entitlement_package_ids(&self, school_id: Uuid, today: NaiveDate) -> AppResult<Vec<Uuid>> {
        let ids: Vec<Uuid> = sqlx::query_scalar(
            r#"SELECT DISTINCT package_id FROM school_package_entitlements
               WHERE school_id = $1 AND starts_at <= $2 AND (expires_at IS NULL OR expires_at >= $2)"#,
        )
        .bind(school_id)
        .bind(today)
        .fetch_all(&self.pool)
        .await?;
        Ok(ids)
    }
}
