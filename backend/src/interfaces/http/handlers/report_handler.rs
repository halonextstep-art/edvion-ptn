use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::domain::report::ReportType;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::report_dto::{GenerateReportPayload, ReportResponse, ReportSummaryResponse};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

fn parse_report_type(s: &str) -> AppResult<ReportType> {
    s.parse::<ReportType>().map_err(AppError::Validation)
}

pub async fn generate_report(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<GenerateReportPayload>,
) -> AppResult<Json<ReportResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let report_type = parse_report_type(&payload.report_type)?;
    let input = payload.into_input(report_type);
    let report = state.report_service.generate(&auth, input).await?;
    Ok(Json(report.into()))
}

pub async fn list_reports(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<ReportSummaryResponse>>> {
    let reports = state.report_service.list(&auth).await?;
    Ok(Json(reports.into_iter().map(Into::into).collect()))
}

pub async fn get_report(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ReportResponse>> {
    let report = state.report_service.get(&auth, id).await?;
    Ok(Json(report.into()))
}

pub async fn delete_report(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.report_service.delete(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}
