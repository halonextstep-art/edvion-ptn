use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::school_dto::{
    parse_activity_kind, ForceAbandonActivityPayload, InProgressActivityItemResponse, PtnDistributionItemResponse,
    RecentActivityItemResponse, SchoolOverviewResponse, SchoolPayload, SchoolResponse, SchoolStatusPayload,
    StudentRosterStatResponse,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

/// School-portal-only aggregate — the school PIC's own Overview tab.
pub async fn overview(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<SchoolOverviewResponse>> {
    let data = state.school_portal_service.overview(&auth).await?;
    Ok(Json(data.into()))
}

/// School-portal-only per-student breakdown — the "Manajemen Siswa" table's score/tryout/
/// target-PTN columns.
pub async fn roster_stats(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<StudentRosterStatResponse>>> {
    let data = state.school_portal_service.roster_stats(&auth).await?;
    Ok(Json(data.into_iter().map(Into::into).collect()))
}

/// School-portal-only aggregate — Analytics tab's "Distribusi Target PTN" chart.
pub async fn ptn_distribution(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<PtnDistributionItemResponse>>> {
    let data = state.school_portal_service.ptn_distribution(&auth).await?;
    Ok(Json(data.into_iter().map(Into::into).collect()))
}

#[derive(Debug, Deserialize)]
pub struct RecentActivityQuery {
    // i64 (not usize) to match the same query-param convention used by every other
    // "?limit=" handler in this codebase (event_handler, analytics_handler,
    // gamification_handler) — keeps the Axum `Query` extractor's expected type uniform
    // across the API surface.
    pub limit: Option<i64>,
}

/// School-portal-only aggregate — Overview tab's "Aktivitas Terkini" feed.
pub async fn recent_activity(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<RecentActivityQuery>,
) -> AppResult<Json<Vec<RecentActivityItemResponse>>> {
    let limit = q.limit.unwrap_or(10).clamp(1, 50) as usize;
    let data = state.school_portal_service.recent_activity(&auth, limit).await?;
    Ok(Json(data.into_iter().map(Into::into).collect()))
}

/// School-portal-only live feed — "Pemantauan Pengerjaan": every in-progress Attempt and
/// SimulationRun across the school's roster. The frontend polls this on an interval (no
/// realtime push infra in this codebase — see `InProgressActivityItem` doc comment).
pub async fn in_progress_activity(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<InProgressActivityItemResponse>>> {
    let data = state.school_portal_service.in_progress_activity(&auth).await?;
    Ok(Json(data.into_iter().map(Into::into).collect()))
}

/// School-portal-only write — force-clears one student's stuck in-progress activity so they
/// can start a fresh attempt/run. See `SchoolPortalService::force_abandon_activity` doc
/// comment.
pub async fn force_abandon_activity(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(ref_id): Path<Uuid>,
    Json(payload): Json<ForceAbandonActivityPayload>,
) -> AppResult<Json<serde_json::Value>> {
    let kind = parse_activity_kind(&payload.kind)?;
    state.school_portal_service.force_abandon_activity(&auth, kind, ref_id).await?;
    Ok(Json(serde_json::json!({ "abandoned": true })))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<SchoolPayload>,
) -> AppResult<Json<SchoolResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_create_input()?;
    let school = state.school_service.create(&auth, input).await?;
    Ok(Json(school.into()))
}

pub async fn list(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<SchoolResponse>>> {
    let schools = state.school_service.list(&auth).await?;
    Ok(Json(schools.into_iter().map(Into::into).collect()))
}

/// Per-school student counts for the admin list cards — see `SchoolService::roster_counts`.
pub async fn roster_counts(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<crate::domain::school::SchoolRosterCount>>> {
    let counts = state.school_service.roster_counts(&auth).await?;
    Ok(Json(counts))
}

/// "Link ke Tenant Existing" dropdown's data source — see `SchoolService::list_unlinked_tenants`.
pub async fn unlinked_tenants(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<crate::domain::school::LinkableTenant>>> {
    let tenants = state.school_service.list_unlinked_tenants(&auth).await?;
    Ok(Json(tenants))
}

/// Import-siswa checklist's data source for a linked-tenant school — see
/// `SchoolService::list_linkable_students`.
pub async fn linkable_students(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<crate::domain::school::LinkableStudent>>> {
    let students = state.school_service.list_linkable_students(&auth, id).await?;
    Ok(Json(students))
}

#[derive(Debug, Deserialize)]
pub struct LinkStudentsPayload {
    pub akun_ids: Vec<Uuid>,
}

/// Submits the import-siswa checklist for a linked-tenant school — see
/// `SchoolService::link_existing_students`.
pub async fn link_students(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<LinkStudentsPayload>,
) -> AppResult<Json<serde_json::Value>> {
    let linked = state.school_service.link_existing_students(&auth, id, payload.akun_ids).await?;
    Ok(Json(serde_json::json!({ "linked": linked })))
}

pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<SchoolResponse>> {
    let school = state.school_service.get(&auth, id).await?;
    Ok(Json(school.into()))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<SchoolPayload>,
) -> AppResult<Json<SchoolResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_update_input()?;
    let school = state.school_service.update(&auth, id, input).await?;
    Ok(Json(school.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.school_service.delete(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn set_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<SchoolStatusPayload>,
) -> AppResult<Json<SchoolResponse>> {
    let status = payload.into_status()?;
    let school = state.school_service.set_status(&auth, id, status).await?;
    Ok(Json(school.into()))
}
