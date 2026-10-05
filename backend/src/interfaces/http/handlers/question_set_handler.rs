use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::question_set_dto::{
    AddSetItemPayload, CreateQuestionSetPayload, QuestionSetResponse, SetItemsResponse, UpdateQuestionSetPayload,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn list(State(state): State<AppState>, _auth: AuthUser) -> AppResult<Json<Vec<QuestionSetResponse>>> {
    let sets = state.question_set_service.list().await?;
    Ok(Json(sets.into_iter().map(Into::into).collect()))
}

pub async fn get_items(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(set_id): Path<Uuid>,
) -> AppResult<Json<SetItemsResponse>> {
    let items = state.question_set_service.get_items(set_id).await?;
    Ok(Json(items.into()))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateQuestionSetPayload>,
) -> AppResult<Json<QuestionSetResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let set = state.question_set_service.create(&auth, payload.name, payload.description).await?;
    Ok(Json(set.into()))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateQuestionSetPayload>,
) -> AppResult<Json<QuestionSetResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let set = state.question_set_service.update(&auth, id, payload.name, payload.description).await?;
    Ok(Json(set.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.question_set_service.delete(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn add_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(set_id): Path<Uuid>,
    Json(payload): Json<AddSetItemPayload>,
) -> AppResult<Json<serde_json::Value>> {
    state.question_set_service.add_item(&auth, set_id, payload.question_id).await?;
    Ok(Json(serde_json::json!({ "added": true })))
}

pub async fn remove_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((set_id, question_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    state.question_set_service.remove_item(&auth, set_id, question_id).await?;
    Ok(Json(serde_json::json!({ "removed": true })))
}
