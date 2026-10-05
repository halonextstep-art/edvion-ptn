//! Singleton platform-wide settings row — see migration 20250101000026 doc comment. Always
//! exactly one row (`id = 1`). Starts with just the B2C (schoolless) self-registration
//! on/off toggle; more platform-wide flags can be added as columns here later.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Which score(s) are shown to students/school reports for a UTBK/SNBT attempt — see
/// `domain::irt` doc comment for what "Estimasi IRT" is (and explicitly isn't). Added by
/// migration `20250101000036`. Does NOT affect TKA scoring, which always uses the existing
/// jenjang-aware scale (0-100/Istimewa-95 for SD/SMP, 200-800/Istimewa-725 for SMA/SMK/MA — see
/// `application::school_tka_service::tka_scale_for`) regardless of this setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreDisplayMode {
    /// Only "Skor Instan" (the existing simple percentage-based score) is shown — the default,
    /// and identical to platform behavior before this feature existed.
    Instant,
    /// Only "Estimasi IRT" is shown.
    Irt,
    /// Both scores are shown side by side.
    Both,
}

impl ScoreDisplayMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ScoreDisplayMode::Instant => "instant",
            ScoreDisplayMode::Irt => "irt",
            ScoreDisplayMode::Both => "both",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "instant" => Some(Self::Instant),
            "irt" => Some(Self::Irt),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSettings {
    /// Whether `/auth/register` will accept a new *schoolless* (B2C/mandiri) student
    /// account. Never gates B2B self-registration (a student registering WITH a
    /// `school_id`) or an admin manually creating any account via `UserService` — this is
    /// specifically the public self-serve B2C entry point's on/off switch.
    pub b2c_registration_enabled: bool,
    /// See `ScoreDisplayMode` doc comment.
    pub score_display_mode: ScoreDisplayMode,
    /// Default "Mode Terkunci" (focus/lockdown mode) applied to every Simulasi TO
    /// (`domain::simulation::SimulationTemplate`) that doesn't set its own explicit
    /// `lockdown_override`. See `20250101000041_simulation_lockdown.sql` and
    /// `domain::simulation::SimulationTemplate::lockdown_override` doc comment for the full
    /// two-level (global default + per-template override) resolution rule. `false` by default —
    /// existing Simulasi TO behavior is completely unchanged until an admin opts in.
    pub simulation_lockdown_default: bool,
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}
