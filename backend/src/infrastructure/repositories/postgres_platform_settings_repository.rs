use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::platform_settings::{PlatformSettings, ScoreDisplayMode};
use crate::domain::repository::PlatformSettingsRepository;
use crate::domain::tenant::TenantCtx;
use crate::error::{AppError, AppResult};

pub struct PostgresPlatformSettingsRepository {
    pool: PgPool,
}

impl PostgresPlatformSettingsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// `ptn.platform_settings` is a genuinely global singleton (not per-school), so it's
    /// always read/written under the reserved platform tenant — same as any other
    /// admin/content-only table.
    async fn begin_tx(&self) -> AppResult<sqlx::Transaction<'static, sqlx::Postgres>> {
        let ctx = TenantCtx::platform_admin();
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "SELECT set_config('app.tenant_id', $1, true), set_config('app.is_platform_admin', $2, true)",
        )
        .bind(ctx.tenant_id.to_string())
        .bind(if ctx.is_platform_admin { "true" } else { "false" })
        .execute(&mut *tx)
        .await?;
        Ok(tx)
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

// `updated_by` is resolved to `core.akun.public_id` via a scalar subquery rather than a
// JOIN, so this same column list works unmodified in both plain SELECTs and
// UPDATE ... RETURNING clauses (same pattern as `postgres_question_repository.rs`).
const SETTINGS_COLUMNS: &str = r#"
    b2c_registration_enabled, score_display_mode, simulation_lockdown_default,
    (SELECT public_id FROM core.akun WHERE core.akun.id = ptn.platform_settings.updated_by) AS updated_by,
    updated_at
"#;

#[async_trait]
impl PlatformSettingsRepository for PostgresPlatformSettingsRepository {
    async fn get(&self) -> AppResult<Option<PlatformSettings>> {
        let mut tx = self.begin_tx().await?;
        let row = sqlx::query_as::<_, PlatformSettingsRow>(&format!(
            "SELECT {SETTINGS_COLUMNS} FROM ptn.platform_settings WHERE id = 1"
        ))
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        row.map(TryFrom::try_from).transpose()
    }

    async fn set_b2c_registration_enabled(&self, enabled: bool, updated_by: Uuid) -> AppResult<PlatformSettings> {
        let mut tx = self.begin_tx().await?;
        let updated_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(updated_by)
            .fetch_one(&mut *tx)
            .await?;
        let row = sqlx::query_as::<_, PlatformSettingsRow>(&format!(
            r#"UPDATE ptn.platform_settings
               SET b2c_registration_enabled = $1, updated_by = $2, updated_at = now()
               WHERE id = 1
               RETURNING {SETTINGS_COLUMNS}"#
        ))
        .bind(enabled)
        .bind(updated_by_id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        row.try_into()
    }

    async fn set_score_display_mode(&self, mode: ScoreDisplayMode, updated_by: Uuid) -> AppResult<PlatformSettings> {
        let mut tx = self.begin_tx().await?;
        let updated_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(updated_by)
            .fetch_one(&mut *tx)
            .await?;
        let row = sqlx::query_as::<_, PlatformSettingsRow>(&format!(
            r#"UPDATE ptn.platform_settings
               SET score_display_mode = $1, updated_by = $2, updated_at = now()
               WHERE id = 1
               RETURNING {SETTINGS_COLUMNS}"#
        ))
        .bind(mode.as_str())
        .bind(updated_by_id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        row.try_into()
    }

    async fn set_simulation_lockdown_default(&self, enabled: bool, updated_by: Uuid) -> AppResult<PlatformSettings> {
        let mut tx = self.begin_tx().await?;
        let updated_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(updated_by)
            .fetch_one(&mut *tx)
            .await?;
        let row = sqlx::query_as::<_, PlatformSettingsRow>(&format!(
            r#"UPDATE ptn.platform_settings
               SET simulation_lockdown_default = $1, updated_by = $2, updated_at = now()
               WHERE id = 1
               RETURNING {SETTINGS_COLUMNS}"#
        ))
        .bind(enabled)
        .bind(updated_by_id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        row.try_into()
    }
}
