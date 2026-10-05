use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::auth_dto::UserResponse;
use crate::interfaces::http::dto::user_dto::{
    CreateUserPayload, UpdateUserPayload, UserListQuery, UserStatusPayload,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<UserListQuery>,
) -> AppResult<Json<Vec<UserResponse>>> {
    let filter = q.into_filter()?;
    let users = state.user_service.list(&auth, filter).await?;
    Ok(Json(users.into_iter().map(Into::into).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateUserPayload>,
) -> AppResult<Json<UserResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_input()?;
    let user = state.user_service.create(&auth, input).await?;
    Ok(Json(user.into()))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserPayload>,
) -> AppResult<Json<UserResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_input()?;
    let user = state.user_service.update(&auth, id, input).await?;
    Ok(Json(user.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.user_service.delete(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn set_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UserStatusPayload>,
) -> AppResult<Json<UserResponse>> {
    let status = payload.into_status()?;
    let user = state.user_service.set_status(&auth, id, status).await?;
    Ok(Json(user.into()))
}
