use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::package::{ExamTrack, Package, PackageContentItem, PackageContentType};

#[derive(Debug, Deserialize, Validate)]
pub struct PackagePayload {
    #[validate(length(min = 1, message = "nama paket wajib diisi"))]
    pub name: String,
    #[serde(default = "default_type")]
    pub package_type: String,
    pub original_price: i32,
    pub sale_price: i32,
    #[serde(default = "default_validity")]
    pub validity: String,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub badge: String,
    #[serde(default = "default_emoji")]
    pub emoji: String,
    /// "preset" (curated lucide icon, named in `icon_name`) or "custom" (uploaded image at
    /// `icon_url`) — see `IconPicker.vue`.
    #[serde(default = "default_icon_type")]
    pub icon_type: String,
    #[serde(default = "default_package_icon_name")]
    pub icon_name: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    /// Optional wide promotional image at the top of the package card — see
    /// `domain::package::Package::banner_url` doc comment.
    #[serde(default)]
    pub banner_url: Option<String>,
    #[serde(default = "default_gradient")]
    pub gradient: String,
    #[serde(default = "default_accent_color")]
    pub accent_color: String,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub sort_order: i32,
    /// See `domain::package::Package::elective_pick_count` doc comment.
    #[serde(default)]
    pub elective_pick_count: i32,
    /// See `domain::package::ExamTrack` doc comment.
    #[serde(default = "default_exam_track")]
    pub exam_track: ExamTrack,
}

fn default_type() -> String {
    "SNBT".to_string()
}
fn default_validity() -> String {
    "1 tahun".to_string()
}
fn default_emoji() -> String {
    "📘".to_string()
}
fn default_icon_type() -> String {
    "preset".to_string()
}
fn default_package_icon_name() -> Option<String> {
    Some("package".to_string())
}
fn default_gradient() -> String {
    "from-sky-200 to-blue-100".to_string()
}
fn default_accent_color() -> String {
    "#3b82f6".to_string()
}
fn default_true() -> bool {
    true
}
fn default_exam_track() -> ExamTrack {
    ExamTrack::Snbt
}

impl PackagePayload {
    pub fn into_create_input(self) -> crate::application::package_service::CreatePackageInput {
        crate::application::package_service::CreatePackageInput {
            name: self.name,
            package_type: self.package_type,
            original_price: self.original_price,
            sale_price: self.sale_price,
            validity: self.validity,
            features: self.features.into_iter().filter(|f| !f.trim().is_empty()).collect(),
            badge: self.badge,
            emoji: self.emoji,
            icon_type: self.icon_type,
            icon_name: self.icon_name,
            icon_url: self.icon_url,
            banner_url: self.banner_url,
            gradient: self.gradient,
            accent_color: self.accent_color,
            active: self.active,
            sort_order: self.sort_order,
            elective_pick_count: self.elective_pick_count,
            exam_track: self.exam_track,
        }
    }

    pub fn into_update_input(self) -> crate::application::package_service::UpdatePackageInput {
        crate::application::package_service::UpdatePackageInput {
            name: self.name,
            package_type: self.package_type,
            original_price: self.original_price,
            sale_price: self.sale_price,
            validity: self.validity,
            features: self.features.into_iter().filter(|f| !f.trim().is_empty()).collect(),
            badge: self.badge,
            emoji: self.emoji,
            icon_type: self.icon_type,
            icon_name: self.icon_name,
            icon_url: self.icon_url,
            banner_url: self.banner_url,
            gradient: self.gradient,
            accent_color: self.accent_color,
            active: self.active,
            sort_order: self.sort_order,
            elective_pick_count: self.elective_pick_count,
            exam_track: self.exam_track,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct PackageActivePayload {
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct PackageResponse {
    pub id: Uuid,
    pub name: String,
    pub package_type: String,
    pub original_price: i32,
    pub sale_price: i32,
    pub discount: i32,
    pub validity: String,
    pub features: Vec<String>,
    pub badge: String,
    pub emoji: String,
    pub icon_type: String,
    pub icon_name: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub gradient: String,
    pub accent_color: String,
    pub active: bool,
    pub sort_order: i32,
    /// See `domain::package::Package::elective_pick_count` doc comment. `0` = this package
    /// imposes no "mapel pilihan" restriction on any of its content.
    pub elective_pick_count: i32,
    /// See `domain::package::ExamTrack` doc comment.
    pub exam_track: ExamTrack,
    /// Always 0 — no purchase/checkout system exists in this codebase yet, so there is
    /// no real sales figure to report. Kept as an explicit field (rather than omitted)
    /// so the admin UI can show "belum ada data penjualan" instead of guessing.
    pub sold_count: i64,
    /// What this package actually unlocks — e.g. "Simulasi UTBK Batch 1 (Simulasi TO)". Empty
    /// by default from the plain `From<Package>` impl below (a bare `Package` row knows
    /// nothing about its content); `public_handler::public_packages` fills this in afterward
    /// via `PackageService::public_content_titles`. Distinct from `features`, a free-text
    /// marketing bullet list the admin types by hand — this one is derived live from
    /// `package_content_items` so it can never drift from what the package actually contains.
    pub content_titles: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// "tryout_session" | "simulation_template" on the wire — matches the DB's `content_type`
/// column string form (see `PackageContentType`/`postgres_package_content_repository.rs`).
fn content_type_str(t: PackageContentType) -> &'static str {
    match t {
        PackageContentType::TryoutSession => "tryout_session",
        PackageContentType::SimulationTemplate => "simulation_template",
    }
}

fn parse_content_type(s: &str) -> Result<PackageContentType, crate::error::AppError> {
    match s {
        "tryout_session" => Ok(PackageContentType::TryoutSession),
        "simulation_template" => Ok(PackageContentType::SimulationTemplate),
        other => Err(crate::error::AppError::Validation(format!("tipe konten tidak dikenal: {other}"))),
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddContentItemPayload {
    pub content_type: String,
    pub content_id: Uuid,
}

impl AddContentItemPayload {
    pub fn parsed_content_type(&self) -> Result<PackageContentType, crate::error::AppError> {
        parse_content_type(&self.content_type)
    }
}

#[derive(Debug, Serialize)]
pub struct PackageContentItemResponse {
    pub id: Uuid,
    pub package_id: Uuid,
    pub content_type: String,
    pub content_id: Uuid,
    pub content_title: String,
    pub created_at: DateTime<Utc>,
}

impl From<PackageContentItem> for PackageContentItemResponse {
    fn from(i: PackageContentItem) -> Self {
        Self {
            id: i.id,
            package_id: i.package_id,
            content_type: content_type_str(i.content_type).to_string(),
            content_id: i.content_id,
            content_title: i.content_title,
            created_at: i.created_at,
        }
    }
}

impl From<Package> for PackageResponse {
    fn from(p: Package) -> Self {
        let discount = p.discount_percent();
        Self {
            id: p.id,
            name: p.name,
            package_type: p.package_type,
            original_price: p.original_price,
            sale_price: p.sale_price,
            discount,
            validity: p.validity,
            features: p.features,
            badge: p.badge,
            emoji: p.emoji,
            icon_type: p.icon_type,
            icon_name: p.icon_name,
            icon_url: p.icon_url,
            banner_url: p.banner_url,
            gradient: p.gradient,
            accent_color: p.accent_color,
            active: p.active,
            sort_order: p.sort_order,
            elective_pick_count: p.elective_pick_count,
            exam_track: p.exam_track,
            sold_count: 0,
            content_titles: Vec::new(),
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
