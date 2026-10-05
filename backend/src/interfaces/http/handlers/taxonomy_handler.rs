use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::taxonomy_dto::{
    CategoryWithSubjectsResponse, CreateCategoryPayload, CreateSubjectPayload, SubjectCategoryResponse,
    SubjectResponse, UpdateCategoryPayload, UpdateSubjectPayload,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn list(State(state): State<AppState>, _auth: AuthUser) -> AppResult<Json<Vec<CategoryWithSubjectsResponse>>> {
    let categories = state.taxonomy_service.list().await?;
    Ok(Json(categories.into_iter().map(Into::into).collect()))
}

pub async fn create_category(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateCategoryPayload>,
) -> AppResult<Json<SubjectCategoryResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let category = state
        .taxonomy_service
        .create_category(&auth, payload.name, payload.description, payload.sort_order)
        .await?;
    Ok(Json(category.into()))
}

pub async fn update_category(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryPayload>,
) -> AppResult<Json<SubjectCategoryResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let category = state
        .taxonomy_service
        .update_category(&auth, id, payload.name, payload.description, payload.sort_order)
        .await?;
    Ok(Json(category.into()))
}

pub async fn delete_category(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.taxonomy_service.delete_category(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn create_subject(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateSubjectPayload>,
) -> AppResult<Json<SubjectResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let subject = state
        .taxonomy_service
        .create_subject(&auth, payload.category_id, payload.name, payload.code, payload.sort_order)
        .await?;
    Ok(Json(subject.into()))
}

pub async fn update_subject(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSubjectPayload>,
) -> AppResult<Json<SubjectResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let subject = state
        .taxonomy_service
        .update_subject(&auth, id, payload.name, payload.code, payload.sort_order)
        .await?;
    Ok(Json(subject.into()))
}

pub async fn delete_subject(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.taxonomy_service.delete_subject(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}
