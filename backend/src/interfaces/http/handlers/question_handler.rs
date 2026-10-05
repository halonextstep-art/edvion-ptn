use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::application::question_service::ReviewAction;
use crate::domain::repository::QuestionFilter;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::question_dto::{
    QuestionListQuery, QuestionListResponse, QuestionPayload, QuestionResponse, ReviewPayload,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<QuestionPayload>,
) -> AppResult<Json<QuestionResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_service_input()?;
    let question = state.question_service.create(&auth, input).await?;
    Ok(Json(question.into()))
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<QuestionListQuery>,
) -> AppResult<Json<QuestionListResponse>> {
    let status: Option<crate::domain::question::QuestionStatus> =
        q.status.map(|s| s.parse()).transpose().map_err(AppError::Validation)?;
    let difficulty: Option<crate::domain::question::Difficulty> = q
        .difficulty
        .map(|s| s.parse())
        .transpose()
        .map_err(AppError::Validation)?;
    let question_type: Option<crate::domain::question::QuestionType> = q
        .question_type
        .map(|s| s.parse())
        .transpose()
        .map_err(AppError::Validation)?;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20);

    let filter = QuestionFilter {
        search: q.search,
        subject: q.subject,
        status,
        difficulty,
        question_type,
        created_by: if q.mine.unwrap_or(false) { Some(auth.user_id) } else { None },
        page,
        page_size,
    };

    let (items, total) = state.question_service.list(filter).await?;
    Ok(Json(QuestionListResponse {
        items: items.into_iter().map(Into::into).collect(),
        total,
        page,
        page_size,
    }))
}

pub async fn get(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<QuestionResponse>> {
    let question = state.question_service.get(id).await?;
    Ok(Json(question.into()))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<QuestionPayload>,
) -> AppResult<Json<QuestionResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_service_input()?;
    let question = state.question_service.update(&auth, id, input).await?;
    Ok(Json(question.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.question_service.delete(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn review(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReviewPayload>,
) -> AppResult<Json<QuestionResponse>> {
    let action = match payload.action.as_str() {
        "approve" => ReviewAction::Approve,
        "reject" => ReviewAction::Reject,
        "revision" => ReviewAction::RequestRevision,
        other => return Err(AppError::Validation(format!("unknown review action: {other}"))),
    };
    let question = state.question_service.review(&auth, id, action, payload.note).await?;
    Ok(Json(question.into()))
}

pub async fn submit_for_review(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<QuestionResponse>> {
    let question = state.question_service.submit_for_review(&auth, id).await?;
    Ok(Json(question.into()))
}
