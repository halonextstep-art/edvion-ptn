//! Package entity — admin-managed pricing/marketing catalogue shown on the landing page
//! (Manajemen Paket), mirroring the reference `AdminPackageManagement`. `discount` is a
//! pure derived value (never stored) so it can never drift from the two prices. `sold_count`
//! is intentionally *not* part of this entity: no purchase/checkout system exists anywhere
//! in this codebase, so there is no real figure to report — the DTO layer always returns 0
//! for it rather than storing or fabricating a number.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Which national exam track a `Package` (and the `TryoutSession`/`SimulationTemplate` content
/// tagged to match it — see those structs' `exam_track` fields) is built to prepare for.
/// Formalizes what used to be an implicit heuristic (`elective_pick_count > 0` ⇒ "treat as
/// TKA") — see `application::elective_service`/`application::school_tka_service` doc comments
/// for the heuristic this field replaces (still in place, not removed, for the features already
/// built on it). `Snbt` (UTBK) is the default for every pre-existing row, since that's what the
/// vast majority of existing content already is. See `20250101000035_exam_track.sql`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ExamTrack {
    Snbt,
    Tka,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: Uuid,
    pub name: String,
    pub package_type: String,
    pub original_price: i32,
    pub sale_price: i32,
    pub validity: String,
    pub features: Vec<String>,
    pub badge: String,
    /// Legacy raw-emoji field — kept for backward compatibility (harmless, unread by any
    /// current display) but superseded by `icon_type`/`icon_name`/`icon_url` below. See
    /// `IconPicker.vue` for the picker that manages the new fields.
    pub emoji: String,
    /// "preset" (a curated lucide-vue-next icon, named in `icon_name`) or "custom" (an
    /// admin-uploaded image, at `icon_url`).
    pub icon_type: String,
    pub icon_name: Option<String>,
    pub icon_url: Option<String>,
    /// Optional wide promotional image shown at the top of the package card, distinct from
    /// the small square icon above. `None` (default for every pre-existing row) means the
    /// card keeps rendering its gradient + icon exactly as before — purely additive, never
    /// required. See `frontend/components/shared/BannerPicker.vue`.
    pub banner_url: Option<String>,
    pub gradient: String,
    pub accent_color: String,
    pub active: bool,
    pub sort_order: i32,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// `0` = no "mapel pilihan" restriction — every `TryoutSession` in this package (whether
    /// `is_elective` or not) is freely startable, exactly as before this feature existed. A
    /// positive number is the exact count of `is_elective` sessions in this package a student
    /// must choose (see `package_elective_choices` / `ElectiveService`) before any of them
    /// become startable — mirrors real TKA rules (e.g. "choose exactly 2 of N mapel pilihan").
    pub elective_pick_count: i32,
    /// See `ExamTrack` doc comment above.
    pub exam_track: ExamTrack,
}

impl Package {
    /// Rounded percentage discount of `sale_price` off `original_price`. Mirrors the
    /// reference's `Math.round((1 - salePrice / originalPrice) * 100)`.
    pub fn discount_percent(&self) -> i32 {
        compute_discount(self.original_price, self.sale_price)
    }
}

/// Which kind of content a `PackageContentItem` points at. A single enum/column (rather than
/// two separate join tables) because the two content kinds are structurally identical from the
/// package's point of view — "this package unlocks this specific piece of content" — and every
/// place that reads this join (AccessService, the admin content-assignment UI) needs to handle
/// both kinds identically anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PackageContentType {
    TryoutSession,
    SimulationTemplate,
}

/// One row = "owning `package_id` grants access to this specific `content_id`". Replaces the
/// old all-or-nothing model where owning ANY package unlocked ALL premium content — see
/// `20250101000031_package_content_items.sql` for the migration/backfill rationale. `title` is
/// denormalized at read time (joined from `tryout_sessions.title` or `simulation_templates.title`
/// depending on `content_type`) purely for admin-UI display; it is never stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageContentItem {
    pub id: Uuid,
    pub package_id: Uuid,
    pub content_type: PackageContentType,
    pub content_id: Uuid,
    pub content_title: String,
    pub created_at: DateTime<Utc>,
}

pub fn compute_discount(original_price: i32, sale_price: i32) -> i32 {
    if original_price <= 0 {
        return 0;
    }
    ((1.0 - sale_price as f64 / original_price as f64) * 100.0).round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_forty_percent_discount() {
        assert_eq!(compute_discount(175_000, 105_000), 40);
    }

    #[test]
    fn zero_original_price_does_not_panic() {
        assert_eq!(compute_discount(0, 0), 0);
    }

    #[test]
    fn equal_prices_have_no_discount() {
        assert_eq!(compute_discount(100_000, 100_000), 0);
    }
}
