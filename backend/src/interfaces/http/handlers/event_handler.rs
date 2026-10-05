use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::event_dto::{EventPayload, EventResponse, EventStatusPayload};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<EventPayload>,
) -> AppResult<Json<EventResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_create_input()?;
    let event = state.event_service.create(&auth, input).await?;
    Ok(Json(event.into()))
}

pub async fn list(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<EventResponse>>> {
    let events = state.event_service.list(&auth).await?;
    Ok(Json(events.into_iter().map(Into::into).collect()))
}

#[derive(Debug, Deserialize)]
pub struct UpcomingQuery {
    pub limit: Option<i64>,
}

pub async fn list_upcoming(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<UpcomingQuery>,
) -> AppResult<Json<Vec<EventResponse>>> {
    let events = state.event_service.list_upcoming(&auth, q.limit.unwrap_or(5)).await?;
    Ok(Json(events.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<EventResponse>> {
    let event = state.event_service.get(&auth, id).await?;
    Ok(Json(event.into()))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<EventPayload>,
) -> AppResult<Json<EventResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_update_input()?;
    let event = state.event_service.update(&auth, id, input).await?;
    Ok(Json(event.into()))
}

pub async fn set_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<EventStatusPayload>,
) -> AppResult<Json<EventResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let status = payload.status.parse().map_err(AppError::Validation)?;
    let event = state.event_service.set_status(&auth, id, status).await?;
    Ok(Json(event.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.event_service.delete(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}
