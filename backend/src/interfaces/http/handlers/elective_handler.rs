//! "Mapel Pilihan" — student-facing pick/choose endpoints. See
//! `application::elective_service` doc comment.

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::error::AppResult;
use crate::interfaces::http::dto::elective_dto::{ElectivePackageGroupResponse, ElectivesResponse, SetChoicesPayload};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

/// Every package with a "mapel pilihan" the student can currently pick from — powers the
/// "Pilih Mapel Pilihan" section in `DrillingZone.vue` without the frontend needing to already
/// know which package ids to ask about.
pub async fn my_elective_packages(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<ElectivePackageGroupResponse>>> {
    let groups = state.elective_service.list_my_elective_packages(&auth).await?;
    Ok(Json(groups.into_iter().map(Into::into).collect()))
}

pub async fn get_electives(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(package_id): Path<Uuid>,
) -> AppResult<Json<ElectivesResponse>> {
    let (sessions, pick_count, my_choices) = state.elective_service.get_electives_view(&auth, package_id).await?;
    Ok(Json(ElectivesResponse {
        sessions: sessions.into_iter().map(Into::into).collect(),
        pick_count,
        my_choices,
    }))
}

pub async fn set_choices(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(package_id): Path<Uuid>,
    Json(payload): Json<SetChoicesPayload>,
) -> AppResult<Json<serde_json::Value>> {
    let choices = state.elective_service.set_choices(&auth, package_id, payload.session_ids).await?;
    Ok(Json(serde_json::json!({ "session_ids": choices })))
}
