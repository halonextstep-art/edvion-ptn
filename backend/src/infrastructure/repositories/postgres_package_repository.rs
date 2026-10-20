use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::package::{ExamTrack, Package, PackageVisibility};
use crate::domain::repository::{NewPackage, PackageRepository, PackageUpdate};
use crate::domain::tenant::TenantCtx;
use crate::error::{AppError, AppResult};

pub struct PostgresPackageRepository {
    pool: PgPool,
}

impl PostgresPackageRepository {
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

fn exam_track_str(t: ExamTrack) -> &'static str {
    match t {
        ExamTrack::Snbt => "snbt",
        ExamTrack::Tka => "tka",
    }
}

fn parse_exam_track(s: &str) -> AppResult<ExamTrack> {
    match s {
        "snbt" => Ok(ExamTrack::Snbt),
        "tka" => Ok(ExamTrack::Tka),
        other => Err(AppError::Internal(anyhow::anyhow!("corrupt exam_track: {other}"))),
    }
}

fn visibility_str(v: PackageVisibility) -> &'static str {
    match v {
        PackageVisibility::Public => "public",
        PackageVisibility::Restricted => "restricted",
    }
}

fn parse_visibility(s: &str) -> AppResult<PackageVisibility> {
    match s {
        "public" => Ok(PackageVisibility::Public),
        "restricted" => Ok(PackageVisibility::Restricted),
        other => Err(AppError::Internal(anyhow::anyhow!("corrupt package visibility: {other}"))),
    }
}

#[derive(sqlx::FromRow)]
struct PackageRow {
    public_id: Uuid,
    name: String,
    package_type: String,
    original_price: i32,
    sale_price: i32,
    validity: String,
    features: Vec<String>,
    badge: String,
    emoji: String,
    icon_type: String,
    icon_name: Option<String>,
    icon_url: Option<String>,
    banner_url: Option<String>,
    gradient: String,
    accent_color: String,
    active: bool,
    sort_order: i32,
    // `Option` because the scalar subquery can return NULL when `created_by` points at a
    // `core.akun` row the current RLS context can't see — see
    // `domain::package::Package::created_by` doc comment.
    created_by_public_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    elective_pick_count: i32,
    exam_track: String,
    visibility: String,
}

impl TryFrom<PackageRow> for Package {
    type Error = AppError;
    fn try_from(r: PackageRow) -> Result<Self, Self::Error> {
        Ok(Package {
            id: r.public_id,
            name: r.name,
            package_type: r.package_type,
            original_price: r.original_price,
            sale_price: r.sale_price,
            validity: r.validity,
            features: r.features,
            badge: r.badge,
            emoji: r.emoji,
            icon_type: r.icon_type,
            icon_name: r.icon_name,
            icon_url: r.icon_url,
            banner_url: r.banner_url,
            gradient: r.gradient,
            accent_color: r.accent_color,
            active: r.active,
            sort_order: r.sort_order,
            created_by: r.created_by_public_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
            elective_pick_count: r.elective_pick_count,
            exam_track: parse_exam_track(&r.exam_track)?,
            visibility: parse_visibility(&r.visibility)?,
        })
    }
}

const SELECT_PACKAGE: &str = r#"
    SELECT public_id, name, package_type, original_price, sale_price, validity, features,
           badge, emoji, icon_type, icon_name, icon_url, banner_url, gradient, accent_color, active,
           sort_order,
           (SELECT public_id FROM core.akun WHERE core.akun.id = ptn.packages.created_by) AS created_by_public_id,
           created_at, updated_at, elective_pick_count, exam_track, visibility
    FROM ptn.packages
"#;

#[async_trait]
impl PackageRepository for PostgresPackageRepository {
    async fn create(&self, n: NewPackage, ctx: &TenantCtx) -> AppResult<Package> {
        let mut tx = self.begin_tx(ctx).await?;

        let created_by: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(n.created_by)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("created_by lookup failed: {e}")))?;

        let id: i64 = sqlx::query_scalar(
            r#"INSERT INTO ptn.packages
                (name, package_type, original_price, sale_price, validity, features,
                 badge, emoji, icon_type, icon_name, icon_url, banner_url, gradient, accent_color, active,
                 sort_order, created_by, elective_pick_count, exam_track, visibility)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
               RETURNING id"#,
        )
        .bind(&n.name)
        .bind(&n.package_type)
        .bind(n.original_price)
        .bind(n.sale_price)
        .bind(&n.validity)
        .bind(&n.features)
        .bind(&n.badge)
        .bind(&n.emoji)
        .bind(&n.icon_type)
        .bind(&n.icon_name)
        .bind(&n.icon_url)
        .bind(&n.banner_url)
        .bind(&n.gradient)
        .bind(&n.accent_color)
        .bind(n.active)
        .bind(n.sort_order)
        .bind(created_by)
        .bind(n.elective_pick_count)
        .bind(exam_track_str(n.exam_track))
        .bind(visibility_str(n.visibility))
        .fetch_one(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, PackageRow>(&format!("{SELECT_PACKAGE} WHERE ptn.packages.id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Package::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<Package>> {
        let mut tx = self.begin_tx(ctx).await?;
        let row = sqlx::query_as::<_, PackageRow>(&format!("{SELECT_PACKAGE} WHERE public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(Package::try_from).transpose()
    }

    async fn list(&self, ctx: &TenantCtx) -> AppResult<Vec<Package>> {
        let mut tx = self.begin_tx(ctx).await?;
        let rows = sqlx::query_as::<_, PackageRow>(&format!("{SELECT_PACKAGE} ORDER BY sort_order ASC, created_at ASC"))
            .fetch_all(&mut *tx)
            .await?;
        tx.commit().await?;
        rows.into_iter().map(Package::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: PackageUpdate, ctx: &TenantCtx) -> AppResult<Option<Package>> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query(
            r#"UPDATE ptn.packages SET
                name = $2, package_type = $3, original_price = $4, sale_price = $5,
                validity = $6, features = $7, badge = $8, emoji = $9, icon_type = $10,
                icon_name = $11, icon_url = $12, banner_url = $13, gradient = $14, accent_color = $15,
                active = $16, sort_order = $17, updated_at = now(), elective_pick_count = $18,
                exam_track = $19, visibility = $20
               WHERE public_id = $1"#,
        )
        .bind(id)
        .bind(&u.name)
        .bind(&u.package_type)
        .bind(u.original_price)
        .bind(u.sale_price)
        .bind(&u.validity)
        .bind(&u.features)
        .bind(&u.badge)
        .bind(&u.emoji)
        .bind(&u.icon_type)
        .bind(&u.icon_name)
        .bind(&u.icon_url)
        .bind(&u.banner_url)
        .bind(&u.gradient)
        .bind(&u.accent_color)
        .bind(u.active)
        .bind(u.sort_order)
        .bind(u.elective_pick_count)
        .bind(exam_track_str(u.exam_track))
        .bind(visibility_str(u.visibility))
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, PackageRow>(&format!("{SELECT_PACKAGE} WHERE public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(Package::try_from).transpose()
    }

    async fn set_active(&self, id: Uuid, active: bool, ctx: &TenantCtx) -> AppResult<Option<Package>> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("UPDATE ptn.packages SET active = $2, updated_at = now() WHERE public_id = $1")
            .bind(id)
            .bind(active)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, PackageRow>(&format!("{SELECT_PACKAGE} WHERE public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(Package::try_from).transpose()
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("DELETE FROM ptn.packages WHERE public_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }
}
