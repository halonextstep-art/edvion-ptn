use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppResult;
use crate::interfaces::http::dto::notification_dto::{NotificationResponse, UnreadCountResponse};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    limit: Option<i64>,
}

pub async fn list_mine(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<NotificationResponse>>> {
    let items = state.notification_service.list_mine(&auth, q.limit.unwrap_or(20)).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

pub async fn unread_count(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<UnreadCountResponse>> {
    let count = state.notification_service.unread_count(&auth).await?;
    Ok(Json(UnreadCountResponse { count }))
}

pub async fn mark_read(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<serde_json::Value>> {
    state.notification_service.mark_read(&auth, id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn mark_all_read(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<serde_json::Value>> {
    state.notification_service.mark_all_read(&auth).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
