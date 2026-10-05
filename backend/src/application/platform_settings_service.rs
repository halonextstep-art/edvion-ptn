//! Platform-wide settings use-cases — see `domain::platform_settings` doc comment. Backs the
//! B2C/schoolless self-registration on/off flag (read publicly, no-auth, so `/daftar` can
//! honestly show a disabled state before the user even logs in, mutated Admin-only) and the
//! UTBK/SNBT "Skor Instan" vs "Estimasi IRT" display mode (read internally by `TryoutService`
//! when building an `AttemptResult`, mutated Admin-only).

use std::sync::Arc;

use uuid::Uuid;

use crate::domain::platform_settings::{PlatformSettings, ScoreDisplayMode};
use crate::domain::repository::PlatformSettingsRepository;
use crate::domain::user::Role;
use crate::error::AppResult;
use crate::interfaces::http::middleware::AuthUser;

pub struct PlatformSettingsService {
    settings: Arc<dyn PlatformSettingsRepository>,
}

impl PlatformSettingsService {
    pub fn new(settings: Arc<dyn PlatformSettingsRepository>) -> Self {
        Self { settings }
    }

    /// The singleton row is seeded by migration `20250101000026`, so a missing row would mean
    /// the migration never ran — treat that as "enabled"/"instant" (fail-open on the read side,
    /// since these only gate nice-to-have display behavior, not anything security-sensitive)
    /// rather than surfacing an opaque error.
    async fn current(&self) -> AppResult<PlatformSettings> {
        match self.settings.get().await? {
            Some(s) => Ok(s),
            None => Ok(PlatformSettings {
                b2c_registration_enabled: true,
                score_display_mode: ScoreDisplayMode::Instant,
                simulation_lockdown_default: false,
                updated_by: None,
                updated_at: chrono::Utc::now(),
            }),
        }
    }

    /// No-auth read used by both `auth_handler::register`'s enforcement check and the public
    /// `/api/public/registration-status` endpoint that `/daftar` calls on mount.
    pub async fn is_b2c_registration_enabled(&self) -> AppResult<bool> {
        Ok(self.current().await?.b2c_registration_enabled)
    }

    /// No-auth read — `TryoutService::build_result_for_submitted` calls this for every
    /// student/school result view, so it deliberately has no role check (unlike `get`/
    /// `set_score_display_mode` below, which are the Admin-only settings-page path).
    pub async fn score_display_mode(&self) -> AppResult<ScoreDisplayMode> {
        Ok(self.current().await?.score_display_mode)
    }

    pub async fn get(&self, actor: &AuthUser) -> AppResult<PlatformSettings> {
        actor.require_role(&[Role::Admin])?;
        self.current().await
    }

    pub async fn set_b2c_registration_enabled(&self, actor: &AuthUser, enabled: bool) -> AppResult<PlatformSettings> {
        actor.require_role(&[Role::Admin])?;
        let updated_by: Uuid = actor.user_id;
        self.settings.set_b2c_registration_enabled(enabled, updated_by).await
    }

    pub async fn set_score_display_mode(&self, actor: &AuthUser, mode: ScoreDisplayMode) -> AppResult<PlatformSettings> {
        actor.require_role(&[Role::Admin])?;
        let updated_by: Uuid = actor.user_id;
        self.settings.set_score_display_mode(mode, updated_by).await
    }

    /// No-auth read — `SimulationService::start_run` calls this to resolve a template's
    /// effective lockdown decision when the template has no explicit `lockdown_override`. See
    /// `PlatformSettings::simulation_lockdown_default` doc comment.
    pub async fn simulation_lockdown_default(&self) -> AppResult<bool> {
        Ok(self.current().await?.simulation_lockdown_default)
    }

    pub async fn set_simulation_lockdown_default(&self, actor: &AuthUser, enabled: bool) -> AppResult<PlatformSettings> {
        actor.require_role(&[Role::Admin])?;
        let updated_by: Uuid = actor.user_id;
        self.settings.set_simulation_lockdown_default(enabled, updated_by).await
    }
}
