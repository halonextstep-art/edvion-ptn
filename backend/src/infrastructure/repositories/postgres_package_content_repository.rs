//! Postgres impl for the Package<->content join — see `20250101000031_package_content_items.sql`
//! and `domain::package::PackageContentItem` doc comments for the full rationale. `content_id`
//! has no DB-level FK (polymorphic across two tables), so `content_title` is resolved here via
//! two separate `LEFT JOIN`s keyed on `content_type`, picking whichever one matched.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::package::{PackageContentItem, PackageContentType};
use crate::domain::repository::PackageContentRepository;
use crate::error::AppResult;

pub struct PostgresPackageContentRepository {
    pool: PgPool,
}

impl PostgresPackageContentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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
    id: Uuid,
    package_id: Uuid,
    content_type: String,
    content_id: Uuid,
    content_title: Option<String>,
    created_at: DateTime<Utc>,
}

const SELECT_ITEM: &str = r#"
    SELECT pci.id, pci.package_id, pci.content_type, pci.content_id,
           COALESCE(ts.title, st.title) AS content_title, pci.created_at
    FROM package_content_items pci
    LEFT JOIN tryout_sessions ts ON pci.content_type = 'tryout_session' AND ts.id = pci.content_id
    LEFT JOIN simulation_templates st ON pci.content_type = 'simulation_template' AND st.id = pci.content_id
"#;

impl TryFrom<ContentItemRow> for PackageContentItem {
    type Error = crate::error::AppError;
    fn try_from(r: ContentItemRow) -> Result<Self, Self::Error> {
        Ok(PackageContentItem {
            id: r.id,
            package_id: r.package_id,
            content_type: parse_content_type(&r.content_type)?,
            content_id: r.content_id,
            // A dangling reference (source row deleted without cleaning up the join — shouldn't
            // happen since PackageService validates before insert, but the join is a LEFT JOIN
            // so it can't be ruled out at the DB level) surfaces as an honest placeholder rather
            // than a panic.
            content_title: r.content_title.unwrap_or_else(|| "(konten tidak ditemukan)".to_string()),
            created_at: r.created_at,
        })
    }
}

#[async_trait]
impl PackageContentRepository for PostgresPackageContentRepository {
    async fn list_for_package(&self, package_id: Uuid) -> AppResult<Vec<PackageContentItem>> {
        let rows = sqlx::query_as::<_, ContentItemRow>(&format!(
            "{SELECT_ITEM} WHERE pci.package_id = $1 ORDER BY pci.created_at ASC"
        ))
        .bind(package_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn add_item(&self, package_id: Uuid, content_type: PackageContentType, content_id: Uuid) -> AppResult<PackageContentItem> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO package_content_items (id, package_id, content_type, content_id)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (package_id, content_type, content_id) DO NOTHING"#,
        )
        .bind(id)
        .bind(package_id)
        .bind(content_type_str(content_type))
        .bind(content_id)
        .execute(&self.pool)
        .await?;

        // If it already existed, the INSERT above no-opped and `id` was never actually used —
        // re-fetch the real row by its natural key instead of assuming `id` is correct.
        let row = sqlx::query_as::<_, ContentItemRow>(&format!(
            "{SELECT_ITEM} WHERE pci.package_id = $1 AND pci.content_type = $2 AND pci.content_id = $3"
        ))
        .bind(package_id)
        .bind(content_type_str(content_type))
        .bind(content_id)
        .fetch_one(&self.pool)
        .await?;
        row.try_into()
    }

    async fn remove_item(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM package_content_items WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn package_ids_for_content(&self, content_type: PackageContentType, content_id: Uuid) -> AppResult<Vec<Uuid>> {
        let ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT package_id FROM package_content_items WHERE content_type = $1 AND content_id = $2",
        )
        .bind(content_type_str(content_type))
        .bind(content_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(ids)
    }

    async fn content_ids_for_packages(&self, package_ids: &[Uuid]) -> AppResult<Vec<(PackageContentType, Uuid)>> {
        if package_ids.is_empty() {
            return Ok(Vec::new());
        }
        let rows: Vec<(String, Uuid)> = sqlx::query_as(
            "SELECT DISTINCT content_type, content_id FROM package_content_items WHERE package_id = ANY($1)",
        )
        .bind(package_ids)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(t, id)| Ok((parse_content_type(&t)?, id)))
            .collect()
    }
}
