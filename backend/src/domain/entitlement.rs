//! School Package Entitlements — see migration 20250101000024 doc comment. Links a School to
//! a Package it has subscribed to; `AccessService` treats every student at that school as
//! owning that package (no individual voucher needed), which in turn grants access to whatever
//! specific TryoutSessions/SimulationTemplates that package includes — see
//! `20250101000031_package_content_items.sql` and `domain::package::PackageContentItem` for the
//! per-package content-scoping rules (access is no longer all-or-nothing platform-wide).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolPackageEntitlement {
    pub id: Uuid,
    pub school_id: Uuid,
    pub school_name: String,
    pub package_id: Uuid,
    pub package_name: String,
    pub starts_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
    pub note: Option<String>,
    pub granted_by: Uuid,
    pub granted_by_name: String,
    pub created_at: DateTime<Utc>,
}

impl SchoolPackageEntitlement {
    pub fn is_active_on(&self, today: NaiveDate) -> bool {
        self.starts_at <= today && self.expires_at.map(|e| today <= e).unwrap_or(true)
    }
}
