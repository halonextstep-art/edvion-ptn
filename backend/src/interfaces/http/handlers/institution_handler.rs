use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::institution_dto::{
    InstitutionListQuery, InstitutionLookupQuery, InstitutionPayload, InstitutionResponse,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(q): Query<InstitutionListQuery>,
) -> AppResult<Json<Vec<InstitutionResponse>>> {
    let items = state.institution_service.list(q.search).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

/// Resolves a `ptn_programs` row's plain-text `nama_ptn` to its institution profile.
/// Returns `204 No Content`-shaped `null` (not a 404) when no research data exists yet for
/// that name, since "institution has no profile yet" is an expected, honest state — not an
/// error — for institutions whose research data wasn't found/confirmed.
pub async fn lookup(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(q): Query<InstitutionLookupQuery>,
) -> AppResult<Json<Option<InstitutionResponse>>> {
    let found = state.institution_service.find_by_nama_ptn(&q.nama_ptn).await?;
    Ok(Json(found.map(Into::into)))
}

pub async fn get(State(state): State<AppState>, _auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<InstitutionResponse>> {
    let item = state.institution_service.get(id).await?;
    Ok(Json(item.into()))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<InstitutionPayload>,
) -> AppResult<Json<InstitutionResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let item = state.institution_service.create(&auth, payload.into_new()).await?;
    Ok(Json(item.into()))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<InstitutionPayload>,
) -> AppResult<Json<InstitutionResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let item = state.institution_service.update(&auth, id, payload.into_update()).await?;
    Ok(Json(item.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.institution_service.delete(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}
