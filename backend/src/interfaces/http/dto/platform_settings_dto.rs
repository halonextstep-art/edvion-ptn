use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::irt_service::{CalibrationStatus, RecalibrationSummary};
use crate::domain::platform_settings::{PlatformSettings, ScoreDisplayMode};
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct PlatformSettingsResponse {
    pub b2c_registration_enabled: bool,
    /// "instant" | "irt" | "both" — see `domain::platform_settings::ScoreDisplayMode`.
    pub score_display_mode: String,
    /// See `domain::platform_settings::PlatformSettings::simulation_lockdown_default` doc comment.
    pub simulation_lockdown_default: bool,
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

impl From<PlatformSettings> for PlatformSettingsResponse {
    fn from(s: PlatformSettings) -> Self {
        Self {
            b2c_registration_enabled: s.b2c_registration_enabled,
            score_display_mode: s.score_display_mode.as_str().to_string(),
            simulation_lockdown_default: s.simulation_lockdown_default,
            updated_by: s.updated_by,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SetB2cRegistrationPayload {
    pub enabled: bool,
}

/// Response for the no-auth `GET /api/public/registration-status` — deliberately trimmed to
/// just the one boolean `/daftar` needs, not the full admin `PlatformSettingsResponse` (no
/// `updated_by`/`updated_at` leaked to an anonymous visitor).
#[derive(Debug, Serialize)]
pub struct RegistrationStatusResponse {
    pub b2c_registration_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetScoreDisplayModePayload {
    /// "instant" | "irt" | "both".
    pub mode: String,
}

impl SetScoreDisplayModePayload {
    pub fn parse(&self) -> AppResult<ScoreDisplayMode> {
        ScoreDisplayMode::parse(&self.mode)
            .ok_or_else(|| AppError::Validation(format!("mode skor tidak dikenal: {}", self.mode)))
    }
}

#[derive(Debug, Deserialize)]
pub struct SetSimulationLockdownDefaultPayload {
    pub enabled: bool,
}

/// For the Admin "Kalibrasi Ulang IRT" panel — honest summary of a just-run recalibration.
#[derive(Debug, Serialize)]
pub struct RecalibrationSummaryResponse {
    pub calibrated_count: i64,
    pub skipped_insufficient_data_count: i64,
}

impl From<RecalibrationSummary> for RecalibrationSummaryResponse {
    fn from(s: RecalibrationSummary) -> Self {
        Self { calibrated_count: s.calibrated_count, skipped_insufficient_data_count: s.skipped_insufficient_data_count }
    }
}

/// Current calibration state — `null` fields mean "belum pernah dikalibrasi", never a fabricated
/// zero-with-a-timestamp.
#[derive(Debug, Serialize)]
pub struct CalibrationStatusResponse {
    pub calibrated_item_count: i64,
    pub last_calibrated_at: Option<DateTime<Utc>>,
}

impl From<CalibrationStatus> for CalibrationStatusResponse {
    fn from(s: CalibrationStatus) -> Self {
        Self { calibrated_item_count: s.calibrated_item_count, last_calibrated_at: s.last_calibrated_at }
    }
}
