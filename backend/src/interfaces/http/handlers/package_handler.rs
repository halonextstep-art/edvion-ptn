use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::package_dto::{
    AddContentItemPayload, PackageActivePayload, PackageContentItemResponse, PackagePayload, PackageResponse,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<PackagePayload>,
) -> AppResult<Json<PackageResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_create_input();
    let pkg = state.package_service.create(&auth, input).await?;
    Ok(Json(pkg.into()))
}

pub async fn list(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<PackageResponse>>> {
    let packages = state.package_service.list(&auth).await?;
    Ok(Json(packages.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PackageResponse>> {
    let pkg = state.package_service.get(&auth, id).await?;
    Ok(Json(pkg.into()))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<PackagePayload>,
) -> AppResult<Json<PackageResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_update_input();
    let pkg = state.package_service.update(&auth, id, input).await?;
    Ok(Json(pkg.into()))
}

pub async fn set_active(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<PackageActivePayload>,
) -> AppResult<Json<PackageResponse>> {
    let pkg = state.package_service.set_active(&auth, id, payload.active).await?;
    Ok(Json(pkg.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.package_service.delete(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ─── Isi Paket (which TryoutSessions/SimulationTemplates this package unlocks) ──────────

pub async fn list_content(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<PackageContentItemResponse>>> {
    let items = state.package_service.list_content(&auth, id).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

pub async fn add_content(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<AddContentItemPayload>,
) -> AppResult<Json<PackageContentItemResponse>> {
    let content_type = payload.parsed_content_type()?;
    let item = state.package_service.add_content_item(&auth, id, content_type, payload.content_id).await?;
    Ok(Json(item.into()))
}

pub async fn remove_content(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((_id, item_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    state.package_service.remove_content_item(&auth, item_id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// "Setujui Semua Soal" — see `PackageService::bulk_approve_content` doc comment.
pub async fn bulk_approve_content(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let approved_count = state.package_service.bulk_approve_content(&auth, id).await?;
    Ok(Json(serde_json::json!({ "approved_count": approved_count })))
}
