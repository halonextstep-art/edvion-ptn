use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::platform_settings::{PlatformSettings, ScoreDisplayMode};
use crate::domain::repository::PlatformSettingsRepository;
use crate::error::{AppError, AppResult};

pub struct PostgresPlatformSettingsRepository {
    pool: PgPool,
}

impl PostgresPlatformSettingsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PlatformSettingsRow {
    b2c_registration_enabled: bool,
    score_display_mode: String,
    simulation_lockdown_default: bool,
    updated_by: Option<Uuid>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<PlatformSettingsRow> for PlatformSettings {
    type Error = AppError;
    fn try_from(r: PlatformSettingsRow) -> Result<Self, Self::Error> {
        let score_display_mode = ScoreDisplayMode::parse(&r.score_display_mode)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("corrupt score_display_mode: {}", r.score_display_mode)))?;
        Ok(PlatformSettings {
            b2c_registration_enabled: r.b2c_registration_enabled,
            score_display_mode,
            simulation_lockdown_default: r.simulation_lockdown_default,
            updated_by: r.updated_by,
            updated_at: r.updated_at,
        })
    }
}

const SETTINGS_COLUMNS: &str =
    "b2c_registration_enabled, score_display_mode, simulation_lockdown_default, updated_by, updated_at";

#[async_trait]
impl PlatformSettingsRepository for PostgresPlatformSettingsRepository {
    async fn get(&self) -> AppResult<Option<PlatformSettings>> {
        let row = sqlx::query_as::<_, PlatformSettingsRow>(&format!(
            "SELECT {SETTINGS_COLUMNS} FROM platform_settings WHERE id = 1"
        ))
        .fetch_optional(&self.pool)
        .await?;
        row.map(TryFrom::try_from).transpose()
    }

    async fn set_b2c_registration_enabled(&self, enabled: bool, updated_by: Uuid) -> AppResult<PlatformSettings> {
        let row = sqlx::query_as::<_, PlatformSettingsRow>(&format!(
            r#"UPDATE platform_settings
               SET b2c_registration_enabled = $1, updated_by = $2, updated_at = now()
               WHERE id = 1
               RETURNING {SETTINGS_COLUMNS}"#
        ))
        .bind(enabled)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await?;
        row.try_into()
    }

    async fn set_score_display_mode(&self, mode: ScoreDisplayMode, updated_by: Uuid) -> AppResult<PlatformSettings> {
        let row = sqlx::query_as::<_, PlatformSettingsRow>(&format!(
            r#"UPDATE platform_settings
               SET score_display_mode = $1, updated_by = $2, updated_at = now()
               WHERE id = 1
               RETURNING {SETTINGS_COLUMNS}"#
        ))
        .bind(mode.as_str())
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await?;
        row.try_into()
    }

    async fn set_simulation_lockdown_default(&self, enabled: bool, updated_by: Uuid) -> AppResult<PlatformSettings> {
        let row = sqlx::query_as::<_, PlatformSettingsRow>(&format!(
            r#"UPDATE platform_settings
               SET simulation_lockdown_default = $1, updated_by = $2, updated_at = now()
               WHERE id = 1
               RETURNING {SETTINGS_COLUMNS}"#
        ))
        .bind(enabled)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await?;
        row.try_into()
    }
}
