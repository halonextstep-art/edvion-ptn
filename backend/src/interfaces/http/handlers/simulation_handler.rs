use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::application::simulation_service::SimulationRunWithQuestions;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::simulation_dto::{
    build_run_response, CreateTemplatePayload, LockdownViolationResponse, ReportViolationPayload, SetActivePayload,
    SetLockdownPayload, SetPremiumPayload, SimulationRunResponse, SimulationTemplateResponse, TemplateStatsResponse,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn create_template(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateTemplatePayload>,
) -> AppResult<Json<SimulationTemplateResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let template = state.simulation_service.create_template(&auth, payload.into_input()).await?;
    Ok(Json(template.into()))
}

pub async fn list_templates(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<SimulationTemplateResponse>>> {
    let templates = state.simulation_service.list_templates(&auth).await?;
    Ok(Json(templates.into_iter().map(Into::into).collect()))
}

pub async fn set_template_active(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<SetActivePayload>,
) -> AppResult<Json<SimulationTemplateResponse>> {
    let template = state.simulation_service.set_template_active(&auth, id, payload.active).await?;
    Ok(Json(template.into()))
}

pub async fn set_template_premium(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<SetPremiumPayload>,
) -> AppResult<Json<SimulationTemplateResponse>> {
    let template = state.simulation_service.set_template_premium(&auth, id, payload.is_premium).await?;
    Ok(Json(template.into()))
}

/// Admin-only — see `domain::simulation::SimulationTemplate::lockdown_override` doc comment.
pub async fn set_template_lockdown(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<SetLockdownPayload>,
) -> AppResult<Json<SimulationTemplateResponse>> {
    let template = state
        .simulation_service
        .set_template_lockdown(&auth, id, payload.lockdown_override)
        .await?;
    Ok(Json(template.into()))
}

/// Student or Admin — see `SimulationService::template_stats` doc comment. Used by the
/// sertifikat/laporan PDF to show an honest "posisi kamu dibanding peserta lain" line.
pub async fn template_stats(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<TemplateStatsResponse>> {
    let stats = state.simulation_service.template_stats(&auth, id).await?;
    Ok(Json(stats.into()))
}

pub async fn delete_template(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.simulation_service.delete_template(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn start_run(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(template_id): Path<Uuid>,
) -> AppResult<Json<SimulationRunResponse>> {
    let result = state.simulation_service.start_run(&auth, template_id).await?;
    Ok(Json(build_run_response(result)))
}

/// Called when the student clicks "Mulai Sekarang" on the pre-exam overview screen — see
/// `SimulationService::begin_run` doc comment.
pub async fn begin_run(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(run_id): Path<Uuid>,
) -> AppResult<Json<SimulationRunResponse>> {
    let result = state.simulation_service.begin_run(&auth, run_id).await?;
    Ok(Json(build_run_response(result)))
}

pub async fn get_run(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(run_id): Path<Uuid>,
) -> AppResult<Json<SimulationRunResponse>> {
    let result = state.simulation_service.get_run(&auth, run_id).await?;
    Ok(Json(build_run_response(result)))
}

pub async fn submit_current(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(run_id): Path<Uuid>,
) -> AppResult<Json<SimulationRunResponse>> {
    let result = state.simulation_service.submit_current(&auth, run_id).await?;
    Ok(Json(build_run_response(result)))
}

pub async fn advance_after_break(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(run_id): Path<Uuid>,
) -> AppResult<Json<SimulationRunResponse>> {
    let result = state.simulation_service.advance_after_break(&auth, run_id).await?;
    Ok(Json(build_run_response(result)))
}

pub async fn abandon_run(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(run_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.simulation_service.abandon_run(&auth, run_id).await?;
    Ok(Json(serde_json::json!({ "abandoned": true })))
}

pub async fn list_my_runs(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<SimulationRunResponse>>> {
    let runs = state.simulation_service.list_my_runs(&auth).await?;
    let responses = runs
        .into_iter()
        .map(|run| build_run_response(SimulationRunWithQuestions { run, current_questions: None }))
        .collect();
    Ok(Json(responses))
}

/// Admin-only — see `SimulationService::admin_list_runs` doc comment.
pub async fn admin_list_runs(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
) -> AppResult<Json<Vec<SimulationRunResponse>>> {
    let runs = state.simulation_service.admin_list_runs(&auth, student_id).await?;
    let responses = runs
        .into_iter()
        .map(|run| build_run_response(SimulationRunWithQuestions { run, current_questions: None }))
        .collect();
    Ok(Json(responses))
}

/// Student-only — see `SimulationService::record_violation` doc comment. Called by the "Mode
/// Terkunci" frontend enforcement every time it detects a breach.
pub async fn report_violation(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(run_id): Path<Uuid>,
    Json(payload): Json<ReportViolationPayload>,
) -> AppResult<Json<LockdownViolationResponse>> {
    let violation = state
        .simulation_service
        .record_violation(&auth, run_id, payload.event_type, payload.detail)
        .await?;
    Ok(Json(violation.into()))
}

/// The run owner (student) or Admin — see `SimulationService::list_violations` doc comment.
pub async fn list_violations(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(run_id): Path<Uuid>,
) -> AppResult<Json<Vec<LockdownViolationResponse>>> {
    let violations = state.simulation_service.list_violations(&auth, run_id).await?;
    Ok(Json(violations.into_iter().map(Into::into).collect()))
}
