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
    public_id: Uuid,
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
            id: r.public_id,
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

// `ptn.school_package_entitlements` itself carries no `tenant_id`/RLS (scoped
// transitively by `school_id`, and read by both Admin (cross-tenant) and the owning
// School — see `AccessService`/`SchoolPortalService` callers) — but every method here
// JOINs to `core.tenant`/`ptn.packages`/`core.akun`, which DO carry RLS. Querying by an
// explicit `school_id`/`id` is already a precise-enough scope (no broad tenant scan
// happens here), so every method opens its own transaction with
// `app.is_platform_admin = true` to bypass that RLS check safely, rather than relying on
// ambient GUC state from whatever connection the pool hands back — a transaction-local
// `set_config` resets to an EMPTY STRING (not "unset") on commit, and an empty string fed
// into another table's `current_setting(...)::bigint` RLS check is a hard Postgres error.
async fn set_platform_admin(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> AppResult<()> {
    sqlx::query("SELECT set_config('app.tenant_id', '0', true), set_config('app.is_platform_admin', 'true', true)")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

const SELECT_ENTITLEMENT: &str = r#"
    SELECT e.public_id, t.public_id AS school_id, t.name AS school_name, p.public_id AS package_id, p.name AS package_name,
           e.starts_at, e.expires_at, e.note,
           a.public_id AS granted_by, a.full_name AS granted_by_name, e.created_at
    FROM ptn.school_package_entitlements e
    JOIN core.tenant t ON t.id = e.school_id
    JOIN ptn.packages p ON p.id = e.package_id
    JOIN core.akun a ON a.id = e.granted_by
"#;

#[async_trait]
impl SchoolEntitlementRepository for PostgresSchoolEntitlementRepository {
    async fn create(&self, input: NewSchoolEntitlement) -> AppResult<SchoolPackageEntitlement> {
        let mut tx = self.pool.begin().await?;
        set_platform_admin(&mut tx).await?;

        let school_id: i64 = sqlx::query_scalar("SELECT id FROM core.tenant WHERE public_id = $1")
            .bind(input.school_id)
            .fetch_one(&mut *tx)
            .await?;
        let package_id: i64 = sqlx::query_scalar("SELECT id FROM ptn.packages WHERE public_id = $1")
            .bind(input.package_id)
            .fetch_one(&mut *tx)
            .await?;
        let granted_by: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(input.granted_by)
            .fetch_one(&mut *tx)
            .await?;

        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.school_package_entitlements
                (school_id, package_id, starts_at, expires_at, note, granted_by)
               VALUES ($1,$2,$3,$4,$5,$6)
               RETURNING public_id"#,
        )
        .bind(school_id)
        .bind(package_id)
        .bind(input.starts_at)
        .bind(input.expires_at)
        .bind(&input.note)
        .bind(granted_by)
        .fetch_one(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, EntitlementRow>(&format!("{SELECT_ENTITLEMENT} WHERE e.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(row.into())
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SchoolPackageEntitlement>> {
        let mut tx = self.pool.begin().await?;
        set_platform_admin(&mut tx).await?;
        let row = sqlx::query_as::<_, EntitlementRow>(&format!("{SELECT_ENTITLEMENT} WHERE e.public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    async fn list_by_school(&self, school_id: Uuid) -> AppResult<Vec<SchoolPackageEntitlement>> {
        let mut tx = self.pool.begin().await?;
        set_platform_admin(&mut tx).await?;
        let rows = sqlx::query_as::<_, EntitlementRow>(&format!(
            "{SELECT_ENTITLEMENT} WHERE t.public_id = $1 ORDER BY e.created_at DESC"
        ))
        .bind(school_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM ptn.school_package_entitlements WHERE public_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn has_active_entitlement(&self, school_id: Uuid, today: NaiveDate) -> AppResult<bool> {
        let mut tx = self.pool.begin().await?;
        set_platform_admin(&mut tx).await?;
        let exists: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(
                SELECT 1 FROM ptn.school_package_entitlements e
                JOIN core.tenant t ON t.id = e.school_id
                WHERE t.public_id = $1 AND e.starts_at <= $2 AND (e.expires_at IS NULL OR e.expires_at >= $2)
            )"#,
        )
        .bind(school_id)
        .bind(today)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(exists)
    }

    async fn active_entitlement_package_ids(&self, school_id: Uuid, today: NaiveDate) -> AppResult<Vec<Uuid>> {
        let mut tx = self.pool.begin().await?;
        set_platform_admin(&mut tx).await?;
        let ids: Vec<Uuid> = sqlx::query_scalar(
            r#"SELECT DISTINCT p.public_id
               FROM ptn.school_package_entitlements e
               JOIN core.tenant t ON t.id = e.school_id
               JOIN ptn.packages p ON p.id = e.package_id
               WHERE t.public_id = $1 AND e.starts_at <= $2 AND (e.expires_at IS NULL OR e.expires_at >= $2)"#,
        )
        .bind(school_id)
        .bind(today)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(ids)
    }
}
