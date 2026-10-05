use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::entitlement::SchoolPackageEntitlement;

#[derive(Debug, Deserialize, Validate)]
pub struct EntitlementPayload {
    pub school_id: Uuid,
    pub package_id: Uuid,
    #[serde(default = "today")]
    pub starts_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
    pub note: Option<String>,
}

fn today() -> NaiveDate {
    Utc::now().date_naive()
}

impl EntitlementPayload {
    pub fn into_input(self) -> crate::application::access_service::CreateEntitlementInput {
        crate::application::access_service::CreateEntitlementInput {
            school_id: self.school_id,
            package_id: self.package_id,
            starts_at: self.starts_at,
            expires_at: self.expires_at,
            note: self.note,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EntitlementResponse {
    pub id: Uuid,
    pub school_id: Uuid,
    pub school_name: String,
    pub package_id: Uuid,
    pub package_name: String,
    pub starts_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
    pub is_active: bool,
    pub note: Option<String>,
    pub granted_by: Uuid,
    pub granted_by_name: String,
    pub created_at: DateTime<Utc>,
}

impl From<SchoolPackageEntitlement> for EntitlementResponse {
    fn from(e: SchoolPackageEntitlement) -> Self {
        let today = Utc::now().date_naive();
        let is_active = e.is_active_on(today);
        Self {
            id: e.id, school_id: e.school_id, school_name: e.school_name,
            package_id: e.package_id, package_name: e.package_name,
            starts_at: e.starts_at, expires_at: e.expires_at, is_active,
            note: e.note, granted_by: e.granted_by, granted_by_name: e.granted_by_name,
            created_at: e.created_at,
        }
    }
}

/// `has_premium_access` is a coarse "has this student unlocked ANYTHING at all" convenience
/// flag (derived from the two id lists below) — kept for existing callers like
/// `StudentOverview.vue`'s package-showcase banner that only need a yes/no signal. Real
/// per-item gating (locking individual Tryout/Simulasi cards) must use the id lists, not this
/// flag, since access is now package-scoped rather than all-or-nothing.
#[derive(Debug, Serialize)]
pub struct AccessStatusResponse {
    pub has_premium_access: bool,
    pub unlocked_tryout_session_ids: Vec<Uuid>,
    pub unlocked_simulation_template_ids: Vec<Uuid>,
}

impl From<crate::application::access_service::UnlockedContent> for AccessStatusResponse {
    fn from(u: crate::application::access_service::UnlockedContent) -> Self {
        let has_premium_access = !u.tryout_session_ids.is_empty() || !u.simulation_template_ids.is_empty();
        Self {
            has_premium_access,
            unlocked_tryout_session_ids: u.tryout_session_ids.into_iter().collect(),
            unlocked_simulation_template_ids: u.simulation_template_ids.into_iter().collect(),
        }
    }
}
