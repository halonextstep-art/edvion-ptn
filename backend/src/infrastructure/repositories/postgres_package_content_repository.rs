//! Postgres impl for the Package<->content join — see `20250101000031_package_content_items.sql`
//! and `domain::package::PackageContentItem` doc comments for the full rationale. `content_id`
//! has no DB-level FK (polymorphic across two tables), so `content_title` is resolved here via
//! two separate `LEFT JOIN`s keyed on `content_type`, picking whichever one matched.
//!
//! `package_id`/`content_id` are exposed to the rest of the app as `Uuid` (the `public_id` of
//! `ptn.packages`/`ptn.tryout_sessions`/`ptn.simulation_templates` respectively), but stored
//! internally as the bigint `ptn.package_content_items.content_id` — resolved against whichever
//! of the two content tables `content_type` points at. `ptn.package_content_items` itself carries
//! no `tenant_id`/RLS (see migration doc comment), but every method still opens an RLS-scoped
//! transaction since the content tables it joins against DO carry RLS.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::package::{PackageContentItem, PackageContentType};
use crate::domain::repository::PackageContentRepository;
use crate::domain::tenant::TenantCtx;
use crate::error::AppResult;

pub struct PostgresPackageContentRepository {
    pool: PgPool,
}

impl PostgresPackageContentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// See `postgres_user_repository.rs::begin_tx` doc comment.
    async fn begin_tx(&self, ctx: &TenantCtx) -> AppResult<Transaction<'static, Postgres>> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "SELECT set_config('app.tenant_id', $1, true), set_config('app.is_platform_admin', $2, true)",
        )
        .bind(ctx.tenant_id.to_string())
        .bind(if ctx.is_platform_admin { "true" } else { "false" })
        .execute(&mut *tx)
        .await?;
        Ok(tx)
    }
}

fn content_type_str(t: PackageContentType) -> &'static str {
    match t {
        PackageContentType::TryoutSession => "tryout_session",
        PackageContentType::SimulationTemplate => "simulation_template",
    }
}

fn parse_content_type(s: &str) -> AppResult<PackageContentType> {
    match s {
        "tryout_session" => Ok(PackageContentType::TryoutSession),
        "simulation_template" => Ok(PackageContentType::SimulationTemplate),
        other => Err(crate::error::AppError::Internal(anyhow::anyhow!("corrupt package content_type: {other}"))),
    }
}

#[derive(sqlx::FromRow)]
struct ContentItemRow {
    public_id: Uuid,
    package_public_id: Uuid,
    content_type: String,
    // `Option` because a dangling reference (the `ptn.tryout_sessions`/
    // `ptn.simulation_templates` row `content_id` points at was deleted without cleaning
    // up this join row first — no FK enforces it, see module doc comment) makes BOTH
    // `LEFT JOIN`s miss, and COALESCE of two NULLs is still NULL.
    content_public_id: Option<Uuid>,
    content_title: Option<String>,
    created_at: DateTime<Utc>,
}

// `content_public_id` resolves through whichever of the two content tables matches
// `content_type` — both carry `public_id` (added alongside every other `ptn.*` table with a
// `Uuid`-typed domain id), exactly mirroring `COALESCE(ts.title, st.title)` for `content_title`.
const SELECT_ITEM: &str = r#"
    SELECT pci.public_id, p.public_id AS package_public_id, pci.content_type,
           COALESCE(ts.public_id, st.public_id) AS content_public_id,
           COALESCE(ts.title, st.title) AS content_title, pci.created_at
    FROM ptn.package_content_items pci
    JOIN ptn.packages p ON p.id = pci.package_id
    LEFT JOIN ptn.tryout_sessions ts ON pci.content_type = 'tryout_session' AND ts.id = pci.content_id
    LEFT JOIN ptn.simulation_templates st ON pci.content_type = 'simulation_template' AND st.id = pci.content_id
"#;

impl TryFrom<ContentItemRow> for PackageContentItem {
    type Error = crate::error::AppError;
    fn try_from(r: ContentItemRow) -> Result<Self, Self::Error> {
        Ok(PackageContentItem {
            id: r.public_id,
            package_id: r.package_public_id,
            content_type: parse_content_type(&r.content_type)?,
            // A dangling reference (source row deleted without cleaning up the join — shouldn't
            // happen since PackageService validates before insert, but the join is a LEFT JOIN
            // so it can't be ruled out at the DB level) has no `public_id` to report either;
            // fall back to the nil UUID rather than failing the whole list.
            content_id: r.content_public_id.unwrap_or(Uuid::nil()),
            content_title: r.content_title.unwrap_or_else(|| "(konten tidak ditemukan)".to_string()),
            created_at: r.created_at,
        })
    }
}

#[async_trait]
impl PackageContentRepository for PostgresPackageContentRepository {
    async fn list_for_package(&self, package_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<PackageContentItem>> {
        let mut tx = self.begin_tx(ctx).await?;
        let rows = sqlx::query_as::<_, ContentItemRow>(&format!(
            "{SELECT_ITEM} WHERE p.public_id = $1 ORDER BY pci.created_at ASC"
        ))
        .bind(package_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn add_item(&self, package_id: Uuid, content_type: PackageContentType, content_id: Uuid, ctx: &TenantCtx) -> AppResult<PackageContentItem> {
        let mut tx = self.begin_tx(ctx).await?;

        let package_pk: i64 = sqlx::query_scalar("SELECT id FROM ptn.packages WHERE public_id = $1")
            .bind(package_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("package lookup failed: {e}")))?;

        let content_pk: i64 = match content_type {
            PackageContentType::TryoutSession => {
                sqlx::query_scalar("SELECT id FROM ptn.tryout_sessions WHERE public_id = $1")
                    .bind(content_id)
                    .fetch_one(&mut *tx)
                    .await
            }
            PackageContentType::SimulationTemplate => {
                sqlx::query_scalar("SELECT id FROM ptn.simulation_templates WHERE public_id = $1")
                    .bind(content_id)
                    .fetch_one(&mut *tx)
                    .await
            }
        }
        .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("content lookup failed: {e}")))?;

        sqlx::query(
            r#"INSERT INTO ptn.package_content_items (package_id, content_type, content_id)
               VALUES ($1, $2, $3)
               ON CONFLICT (package_id, content_type, content_id) DO NOTHING"#,
        )
        .bind(package_pk)
        .bind(content_type_str(content_type))
        .bind(content_pk)
        .execute(&mut *tx)
        .await?;

        // If it already existed, the INSERT above no-opped — re-fetch the real row by its
        // natural key instead of assuming a freshly generated id is correct.
        let row = sqlx::query_as::<_, ContentItemRow>(&format!(
            "{SELECT_ITEM} WHERE p.public_id = $1 AND pci.content_type = $2 AND pci.content_id = $3"
        ))
        .bind(package_id)
        .bind(content_type_str(content_type))
        .bind(content_pk)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        row.try_into()
    }

    async fn remove_item(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("DELETE FROM ptn.package_content_items WHERE public_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn package_ids_for_content(&self, content_type: PackageContentType, content_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<Uuid>> {
        let mut tx = self.begin_tx(ctx).await?;
        let ids: Vec<Uuid> = sqlx::query_scalar(
            r#"SELECT p.public_id
               FROM ptn.package_content_items pci
               JOIN ptn.packages p ON p.id = pci.package_id
               WHERE pci.content_type = $1
                 AND pci.content_id = (
                     SELECT id FROM ptn.tryout_sessions WHERE public_id = $2 AND $1 = 'tryout_session'
                     UNION ALL
                     SELECT id FROM ptn.simulation_templates WHERE public_id = $2 AND $1 = 'simulation_template'
                 )"#,
        )
        .bind(content_type_str(content_type))
        .bind(content_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(ids)
    }

    async fn content_ids_for_packages(&self, package_ids: &[Uuid], ctx: &TenantCtx) -> AppResult<Vec<(PackageContentType, Uuid)>> {
        if package_ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut tx = self.begin_tx(ctx).await?;
        // `content_public_id` is `Option` for the same dangling-reference reason as
        // `ContentItemRow` above — a row whose source session/template was deleted without
        // cleaning up this join first. Unlike the admin-facing list, there's no "(konten
        // tidak ditemukan)" placeholder to show here; this result feeds a student's
        // unlocked-content *set*, so a dangling row is simply dropped rather than polluted
        // with a nil UUID that would never match any real content anyway.
        let rows: Vec<(String, Option<Uuid>)> = sqlx::query_as(
            r#"SELECT DISTINCT pci.content_type, COALESCE(ts.public_id, st.public_id) AS content_public_id
               FROM ptn.package_content_items pci
               JOIN ptn.packages p ON p.id = pci.package_id
               LEFT JOIN ptn.tryout_sessions ts ON pci.content_type = 'tryout_session' AND ts.id = pci.content_id
               LEFT JOIN ptn.simulation_templates st ON pci.content_type = 'simulation_template' AND st.id = pci.content_id
               WHERE p.public_id = ANY($1)"#,
        )
        .bind(package_ids)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows.into_iter()
            .filter_map(|(t, id)| id.map(|id| (t, id)))
            .map(|(t, id)| Ok((parse_content_type(&t)?, id)))
            .collect()
    }
}
