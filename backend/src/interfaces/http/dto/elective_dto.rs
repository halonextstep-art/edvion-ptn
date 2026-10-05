use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::elective_service::PackageElectiveGroup;
use crate::interfaces::http::dto::tryout_dto::SessionResponse;

/// A student's current pick set for a package's "mapel pilihan" — see
/// `application::elective_service` doc comment.
#[derive(Debug, Deserialize)]
pub struct SetChoicesPayload {
    pub session_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ElectivesResponse {
    /// Every `is_elective` session attached to this package (non-draft only).
    pub sessions: Vec<SessionResponse>,
    /// Exact count the student must pick — mirrors `Package::elective_pick_count`. `0` means
    /// this package imposes no restriction (informational only; nothing to enforce).
    pub pick_count: i32,
    /// This student's current picks among `sessions` above.
    pub my_choices: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ElectivePackageGroupResponse {
    pub package_id: Uuid,
    pub package_name: String,
    pub sessions: Vec<SessionResponse>,
    pub pick_count: i32,
    pub my_choices: Vec<Uuid>,
}

impl From<PackageElectiveGroup> for ElectivePackageGroupResponse {
    fn from(g: PackageElectiveGroup) -> Self {
        Self {
            package_id: g.package.id,
            package_name: g.package.name,
            pick_count: g.package.elective_pick_count,
            sessions: g.sessions.into_iter().map(Into::into).collect(),
            my_choices: g.my_choices,
        }
    }
}
