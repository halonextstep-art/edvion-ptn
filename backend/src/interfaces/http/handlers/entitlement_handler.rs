use axum::{extract::{Path, State}, Json};
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::entitlement_dto::{AccessStatusResponse, EntitlementPayload, EntitlementResponse};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<EntitlementPayload>,
) -> AppResult<Json<EntitlementResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_input();
    let e = state.access_service.create_entitlement(&auth, input).await?;
    Ok(Json(e.into()))
}

pub async fn list_for_school(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(school_id): Path<Uuid>,
) -> AppResult<Json<Vec<EntitlementResponse>>> {
    let list = state.access_service.list_for_school(&auth, school_id).await?;
    Ok(Json(list.into_iter().map(Into::into).collect()))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.access_service.delete_entitlement(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn my_status(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<AccessStatusResponse>> {
    let unlocked = state.access_service.my_access_status(&auth).await?;
    Ok(Json(unlocked.into()))
}
