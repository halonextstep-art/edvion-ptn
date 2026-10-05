use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;

use crate::application::tryout_service::{CreateSessionInput, UpdateSessionInput};
use crate::error::AppResult;
use crate::interfaces::http::dto::tryout_dto::*;
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn list_sessions(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<SessionListQuery>,
) -> AppResult<Json<Vec<SessionResponse>>> {
    let session_type = q.session_type.map(|s| parse_session_type(&s)).transpose()?;
    let sessions = state.tryout_service.list_sessions(&auth, session_type).await?;
    Ok(Json(sessions.into_iter().map(Into::into).collect()))
}

pub async fn create_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateSessionPayload>,
) -> AppResult<Json<SessionResponse>> {
    let session_type = parse_session_type(&payload.session_type)?;
    let session = state
        .tryout_service
        .create_session(
            &auth,
            CreateSessionInput {
                title: payload.title,
                session_type,
                duration_minutes: payload.duration_minutes,
                question_count: payload.question_count,
                subject_filter: payload.subject_filter,
                topic_filter: payload.topic_filter,
                difficulty_filter: payload.difficulty_filter,
                is_premium: payload.is_premium,
                question_set_id: payload.question_set_id,
                is_draft: payload.is_draft,
                is_elective: payload.is_elective,
                exam_track: payload.exam_track,
                school_type_scope: payload.school_type_scope,
            },
        )
        .await?;
    Ok(Json(session.into()))
}

pub async fn get_session(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<SessionResponse>> {
    let session = state.tryout_service.get_session(session_id).await?;
    Ok(Json(session.into()))
}

pub async fn update_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<UpdateSessionPayload>,
) -> AppResult<Json<SessionResponse>> {
    let session_type = parse_session_type(&payload.session_type)?;
    let session = state
        .tryout_service
        .update_session(
            &auth,
            session_id,
            UpdateSessionInput {
                title: payload.title,
                session_type,
                duration_minutes: payload.duration_minutes,
                question_count: payload.question_count,
                subject_filter: payload.subject_filter,
                topic_filter: payload.topic_filter,
                difficulty_filter: payload.difficulty_filter,
                is_premium: payload.is_premium,
                question_set_id: payload.question_set_id,
                is_draft: payload.is_draft,
                is_elective: payload.is_elective,
                exam_track: payload.exam_track,
                school_type_scope: payload.school_type_scope,
            },
        )
        .await?;
    Ok(Json(session.into()))
}

pub async fn delete_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.tryout_service.delete_session(&auth, session_id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn start_attempt(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<AttemptWithQuestionsResponse>> {
    let result = state.tryout_service.start_attempt(&auth, session_id).await?;
    Ok(Json(result.into()))
}

pub async fn save_answer(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(attempt_id): Path<Uuid>,
    Json(payload): Json<SaveAnswerPayload>,
) -> AppResult<Json<serde_json::Value>> {
    state
        .tryout_service
        .save_answer(&auth, attempt_id, payload.question_id, payload.answer_text, payload.flagged)
        .await?;
    Ok(Json(serde_json::json!({ "saved": true })))
}

pub async fn submit_attempt(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(attempt_id): Path<Uuid>,
    Json(payload): Json<SubmitAttemptPayload>,
) -> AppResult<Json<AttemptResultResponse>> {
    let result = state
        .tryout_service
        .submit_attempt(&auth, attempt_id, payload.time_used_seconds)
        .await?;
    Ok(Json(result.into()))
}

pub async fn get_attempt(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(attempt_id): Path<Uuid>,
) -> AppResult<Json<AttemptResponse>> {
    let attempt = state.tryout_service.get_attempt(&auth, attempt_id).await?;
    Ok(Json(attempt.into()))
}

pub async fn my_attempts(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<AttemptResponse>>> {
    let attempts = state.tryout_service.list_my_attempts(&auth).await?;
    Ok(Json(attempts.into_iter().map(Into::into).collect()))
}

/// See `TryoutService::resume_attempt` doc comment — used by the player page (`/tryout/:id`)
/// to rebuild its state after a refresh/reopen, since `attemptSession` (Pinia) is in-memory
/// only by design.
pub async fn resume_attempt(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(attempt_id): Path<Uuid>,
) -> AppResult<Json<AttemptResumeResponse>> {
    let result = state.tryout_service.resume_attempt(&auth, attempt_id).await?;
    Ok(Json(result.into()))
}

/// Admin-only — see `TryoutService::admin_list_attempts` doc comment.
pub async fn admin_list_attempts(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
) -> AppResult<Json<Vec<AttemptResponse>>> {
    let attempts = state.tryout_service.admin_list_attempts(&auth, student_id).await?;
    Ok(Json(attempts.into_iter().map(Into::into).collect()))
}

pub async fn get_review(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(attempt_id): Path<Uuid>,
) -> AppResult<Json<Vec<ReviewItemResponse>>> {
    let items = state.tryout_service.get_review(&auth, attempt_id).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

/// "Lihat Hasil" — re-fetches score + subject breakdown for one of the caller's own
/// already-submitted attempts, any time after submission. See `TryoutService::get_result`.
pub async fn get_result(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(attempt_id): Path<Uuid>,
) -> AppResult<Json<AttemptResultResponse>> {
    let result = state.tryout_service.get_result(&auth, attempt_id).await?;
    Ok(Json(result.into()))
}
