use axum::{extract::State, Json};

use crate::error::AppResult;
use crate::interfaces::http::dto::platform_settings_dto::{
    CalibrationStatusResponse, PlatformSettingsResponse, RecalibrationSummaryResponse, RegistrationStatusResponse,
    SetB2cRegistrationPayload, SetScoreDisplayModePayload, SetSimulationLockdownDefaultPayload,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

/// No-auth — backs `/daftar`'s on-mount check so it can honestly show a disabled state
/// instead of a form that would just 422 on submit.
pub async fn registration_status(State(state): State<AppState>) -> AppResult<Json<RegistrationStatusResponse>> {
    let enabled = state.platform_settings_service.is_b2c_registration_enabled().await?;
    Ok(Json(RegistrationStatusResponse { b2c_registration_enabled: enabled }))
}

pub async fn get_settings(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<PlatformSettingsResponse>> {
    let settings = state.platform_settings_service.get(&auth).await?;
    Ok(Json(settings.into()))
}

pub async fn set_b2c_registration(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<SetB2cRegistrationPayload>,
) -> AppResult<Json<PlatformSettingsResponse>> {
    let settings = state
        .platform_settings_service
        .set_b2c_registration_enabled(&auth, payload.enabled)
        .await?;
    Ok(Json(settings.into()))
}

/// Admin-only — which score(s) ("Skor Instan" / "Estimasi IRT" / both) are shown to students
/// and school reports for UTBK/SNBT attempts. See `domain::platform_settings::ScoreDisplayMode`.
pub async fn set_score_display_mode(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<SetScoreDisplayModePayload>,
) -> AppResult<Json<PlatformSettingsResponse>> {
    let mode = payload.parse()?;
    let settings = state.platform_settings_service.set_score_display_mode(&auth, mode).await?;
    Ok(Json(settings.into()))
}

/// Admin-only — global "Mode Terkunci" default applied to every Simulasi TO that doesn't set
/// its own explicit per-template override. See
/// `domain::platform_settings::PlatformSettings::simulation_lockdown_default` doc comment.
pub async fn set_simulation_lockdown_default(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<SetSimulationLockdownDefaultPayload>,
) -> AppResult<Json<PlatformSettingsResponse>> {
    let settings = state
        .platform_settings_service
        .set_simulation_lockdown_default(&auth, payload.enabled)
        .await?;
    Ok(Json(settings.into()))
}

/// Admin-only — manually re-runs IRT item difficulty calibration from all historical graded
/// answers platform-wide. See `application::irt_service::IrtService::recalibrate` doc comment
/// (no scheduler exists, this is the only way calibration ever runs).
pub async fn recalibrate_irt(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<RecalibrationSummaryResponse>> {
    let summary = state.irt_service.recalibrate(&auth).await?;
    Ok(Json(summary.into()))
}

/// Admin-only — current IRT calibration state, for the "Kalibrasi Ulang IRT" panel to show real
/// numbers instead of guessing.
pub async fn irt_status(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<CalibrationStatusResponse>> {
    let status = state.irt_service.status(&auth).await?;
    Ok(Json(status.into()))
}
