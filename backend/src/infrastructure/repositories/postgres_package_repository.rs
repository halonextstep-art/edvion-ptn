use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::package::{ExamTrack, Package};
use crate::domain::repository::{NewPackage, PackageRepository, PackageUpdate};
use crate::error::{AppError, AppResult};

pub struct PostgresPackageRepository {
    pool: PgPool,
}

impl PostgresPackageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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

#[derive(sqlx::FromRow)]
struct PackageRow {
    id: Uuid,
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
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    elective_pick_count: i32,
    exam_track: String,
}

impl TryFrom<PackageRow> for Package {
    type Error = AppError;
    fn try_from(r: PackageRow) -> Result<Self, Self::Error> {
        Ok(Package {
            id: r.id,
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
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
            elective_pick_count: r.elective_pick_count,
            exam_track: parse_exam_track(&r.exam_track)?,
        })
    }
}

const SELECT_PACKAGE: &str = r#"
    SELECT id, name, package_type, original_price, sale_price, validity, features,
           badge, emoji, icon_type, icon_name, icon_url, banner_url, gradient, accent_color, active,
           sort_order, created_by, created_at, updated_at, elective_pick_count, exam_track
    FROM packages
"#;

#[async_trait]
impl PackageRepository for PostgresPackageRepository {
    async fn create(&self, n: NewPackage) -> AppResult<Package> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO packages
                (id, name, package_type, original_price, sale_price, validity, features,
                 badge, emoji, icon_type, icon_name, icon_url, banner_url, gradient, accent_color, active,
                 sort_order, created_by, elective_pick_count, exam_track)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)"#,
        )
        .bind(id)
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
        .bind(n.created_by)
        .bind(n.elective_pick_count)
        .bind(exam_track_str(n.exam_track))
        .execute(&self.pool)
        .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| crate::error::AppError::Internal(anyhow::anyhow!("package vanished right after insert")))
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Package>> {
        let row = sqlx::query_as::<_, PackageRow>(&format!("{SELECT_PACKAGE} WHERE id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(Package::try_from).transpose()
    }

    async fn list(&self) -> AppResult<Vec<Package>> {
        let rows = sqlx::query_as::<_, PackageRow>(&format!("{SELECT_PACKAGE} ORDER BY sort_order ASC, created_at ASC"))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Package::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: PackageUpdate) -> AppResult<Option<Package>> {
        let result = sqlx::query(
            r#"UPDATE packages SET
                name = $2, package_type = $3, original_price = $4, sale_price = $5,
                validity = $6, features = $7, badge = $8, emoji = $9, icon_type = $10,
                icon_name = $11, icon_url = $12, banner_url = $13, gradient = $14, accent_color = $15,
                active = $16, sort_order = $17, updated_at = now(), elective_pick_count = $18,
                exam_track = $19
               WHERE id = $1"#,
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
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn set_active(&self, id: Uuid, active: bool) -> AppResult<Option<Package>> {
        let result = sqlx::query("UPDATE packages SET active = $2, updated_at = now() WHERE id = $1")
            .bind(id)
            .bind(active)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM packages WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
