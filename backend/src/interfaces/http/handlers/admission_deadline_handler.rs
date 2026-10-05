use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::admission_deadline_dto::{AdmissionDeadlinePayload, AdmissionDeadlineResponse};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<AdmissionDeadlinePayload>,
) -> AppResult<Json<AdmissionDeadlineResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_create_input()?;
    let deadline = state.admission_deadline_service.create(&auth, input).await?;
    Ok(Json(deadline.into()))
}

pub async fn list(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<AdmissionDeadlineResponse>>> {
    let deadlines = state.admission_deadline_service.list(&auth).await?;
    Ok(Json(deadlines.into_iter().map(Into::into).collect()))
}

#[derive(Debug, Deserialize)]
pub struct UpcomingQuery {
    pub limit: Option<i64>,
}

pub async fn list_upcoming(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<UpcomingQuery>,
) -> AppResult<Json<Vec<AdmissionDeadlineResponse>>> {
    let deadlines = state
        .admission_deadline_service
        .list_upcoming(&auth, q.limit.unwrap_or(5))
        .await?;
    Ok(Json(deadlines.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<AdmissionDeadlineResponse>> {
    let deadline = state.admission_deadline_service.get(&auth, id).await?;
    Ok(Json(deadline.into()))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<AdmissionDeadlinePayload>,
) -> AppResult<Json<AdmissionDeadlineResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_update_input()?;
    let deadline = state.admission_deadline_service.update(&auth, id, input).await?;
    Ok(Json(deadline.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.admission_deadline_service.delete(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}
