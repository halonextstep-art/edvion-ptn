//! Simulasi UTBK domain — chains several existing `TryoutSession`s into one continuous,
//! server-enforced exam run (fixed order, official per-slot timing, mandatory break between
//! slots, no going back), mirroring how the real UTBK/SNBT's 7 subtests are administered in
//! one sitting. See the migration doc comment (`20250101000025_simulation.sql`) for the full
//! rationale. `combined_estimate_score` is always an ESTIMATE — never a real IRT-calibrated
//! UTBK score — mirroring the existing `est_score` convention used elsewhere in this codebase.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::package::ExamTrack;
use crate::domain::school::SchoolType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SimulationRunStatus {
    InProgress,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SimulationSlotStatus {
    Pending,
    Active,
    Submitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SimulationTemplateKind {
    /// Existing behavior — `finish_run` computes `combined_estimate_score` as an equal-weighted
    /// average across slots, rescaled to the UTBK ballpark. Correct for Simulasi UTBK, whose
    /// real SNBT genuinely has one combined/weighted score for ranking.
    UtbkCombined,
    /// Real TKA reports a score PER MATA UJI and never combines them into one composite for the
    /// student — `finish_run` leaves `combined_estimate_score` `None` for this kind; the
    /// frontend must read each slot's own attempt score instead.
    PerSubject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationTemplateSlot {
    pub id: Uuid,
    pub template_id: Uuid,
    pub sequence_index: i32,
    /// `None` when `is_elective` — there is no single fixed session for this slot, it depends on
    /// which student is taking it. Always `Some` otherwise.
    pub session_id: Option<Uuid>,
    /// `None` when `is_elective` (no fixed session to denormalize a title from) — the frontend
    /// shows a placeholder like "Mapel Pilihan (sesuai pilihan Anda)" in that case.
    pub session_title: Option<String>,
    /// `None` when `is_elective` — the real duration/question count depend on which session each
    /// student ends up choosing, and are only knowable once resolved into a `SimulationRunSlot`.
    pub duration_minutes: Option<i32>,
    pub question_count: Option<i32>,
    pub subject_filter: Option<String>,
    pub break_seconds: i32,
    /// See `domain::simulation` doc comment and `20250101000034_simulation_elective_slots.sql`.
    /// When `true`, `session_id`/`session_title` are `None` here — `SimulationService::start_run`
    /// resolves the real session per-student from `elective_package_id`'s current elective
    /// choices before copying this slot into a `SimulationRunSlot`.
    pub is_elective: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationTemplate {
    pub id: Uuid,
    pub title: String,
    pub is_active: bool,
    /// Added by `20250101000031_package_content_items.sql` (default `true`) to close a
    /// previously-open gap: this template used to have NO access gating at all — any logged-in
    /// student could start any active template for free. Enforced in
    /// `SimulationService::start_run` via `AccessService::has_access_to_simulation_template`,
    /// same pattern as `TryoutSession::is_premium`.
    pub is_premium: bool,
    /// Same "still being assembled, hidden from students" semantics as
    /// `domain::tryout::TryoutSession::is_draft` — see that field's doc comment.
    pub is_draft: bool,
    /// Which `Package`'s "mapel pilihan" choices resolve this template's `is_elective` slots
    /// (see `application::elective_service`). `None` when the template has no elective slot at
    /// all (every pre-existing Simulasi UTBK template, and a TKA "Hari 1 (Wajib)" template).
    pub elective_package_id: Option<Uuid>,
    pub template_kind: SimulationTemplateKind,
    /// See `domain::package::ExamTrack` doc comment.
    pub exam_track: ExamTrack,
    /// See `domain::tryout::TryoutSession::school_type_scope` doc comment — same semantics,
    /// applied to `SimulationService::list_templates` instead.
    pub school_type_scope: Option<SchoolType>,
    /// "Mode Terkunci" (focus/lockdown mode) override for this specific template. `None`
    /// (default for every pre-existing template) means "inherit
    /// `PlatformSettings::simulation_lockdown_default`" — `Some(true)`/`Some(false)` force it
    /// on/off for this template regardless of the global default. See
    /// `20250101000041_simulation_lockdown.sql`. The *effective* value is resolved once, at
    /// `SimulationService::start_run` time, and snapshotted onto `SimulationRun::lockdown_enabled`
    /// — never read live from here again once a run has started.
    pub lockdown_override: Option<bool>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub slots: Vec<SimulationTemplateSlot>,
}

impl SimulationTemplate {
    /// Resolves this template's effective lockdown decision against a global default — see
    /// `lockdown_override` doc comment.
    pub fn effective_lockdown(&self, global_default: bool) -> bool {
        self.lockdown_override.unwrap_or(global_default)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRunSlot {
    pub id: Uuid,
    pub run_id: Uuid,
    pub sequence_index: i32,
    pub session_id: Uuid,
    pub session_title: String,
    pub duration_minutes: i32,
    pub subject_filter: Option<String>,
    pub break_seconds: i32,
    pub attempt_id: Option<Uuid>,
    pub attempt_score: Option<i32>,
    /// Detailed breakdown of the underlying attempt — `None` until that attempt is submitted.
    /// Only populated for the sertifikat/laporan PDF's "Rincian per Subtes" +
    /// "Analisis Kekuatan & Kelemahan" sections (see `application::simulation_service` doc
    /// comment on `finish_run`); the live player screens only ever needed `attempt_score`.
    pub attempt_accuracy: Option<f64>,
    pub attempt_correct_count: Option<i32>,
    pub attempt_wrong_count: Option<i32>,
    pub attempt_unanswered_count: Option<i32>,
    pub status: SimulationSlotStatus,
    pub deadline_at: Option<DateTime<Utc>>,
    pub break_ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRun {
    pub id: Uuid,
    pub template_id: Uuid,
    pub template_title: String,
    /// Denormalized from the template at read time (like `template_title`) so the frontend knows
    /// whether to show `combined_estimate_score` at all — see `SimulationTemplateKind` doc
    /// comment.
    pub template_kind: SimulationTemplateKind,
    /// Denormalized from the template at read time (like `template_title`) — lets the player
    /// (`pages/simulasi/[runId].vue`) and the sertifikat/laporan PDF (`simulationCertificate.ts`)
    /// pick the jenjang-appropriate TKA display scale (see
    /// `application::school_tka_service::tka_scale_for` doc comment) without a second round-trip.
    pub school_type_scope: Option<SchoolType>,
    pub student_id: Uuid,
    pub status: SimulationRunStatus,
    pub current_sequence_index: i32,
    pub combined_estimate_score: Option<f64>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    /// Snapshotted at `start_run` time from `SimulationTemplate::effective_lockdown` — see that
    /// method's doc comment. Immutable for the lifetime of this run: an admin toggling the
    /// global default or the template override afterward never changes the rules for a run
    /// already in progress.
    pub lockdown_enabled: bool,
    pub slots: Vec<SimulationRunSlot>,
}

impl SimulationRun {
    pub fn current_slot(&self) -> Option<&SimulationRunSlot> {
        self.slots.iter().find(|s| s.sequence_index == self.current_sequence_index)
    }
}

/// Honest, aggregate-only benchmark for the sertifikat/laporan PDF's "posisi kamu" line —
/// computed live from every OTHER student's `Completed` run of the same template, never
/// per-student/named (privacy-safe) and never fabricated. `avg_combined_score` is `None`
/// whenever there's no completed run with a non-null `combined_estimate_score` yet (e.g. a
/// brand new template, or every completed run so far is `per_subject` with nothing to
/// average) — the frontend must render "belum ada data pembanding" rather than a fake 0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCompletionStats {
    pub template_id: Uuid,
    pub participant_count: i64,
    pub avg_combined_score: Option<f64>,
}

/// One detected "Mode Terkunci" breach during a `SimulationRun` — e.g. the student switched
/// tabs, exited fullscreen, or tried to open devtools. Logged fire-and-forget by
/// `SimulationService::record_violation` (never blocks the exam flow on a logging failure), and
/// surfaced to the student immediately as a warning plus, afterward, to Admin/Sekolah for
/// review. See `20250101000041_simulation_lockdown.sql`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockdownViolation {
    pub id: Uuid,
    pub run_id: Uuid,
    pub student_id: Uuid,
    /// Free-form but conventionally one of: "tab_switch", "fullscreen_exit", "devtools_attempt",
    /// "copy_attempt", "context_menu_attempt", "window_blur" — the frontend decides the exact
    /// vocabulary; the backend stores whatever string it's given.
    pub event_type: String,
    pub detail: Option<String>,
    pub occurred_at: DateTime<Utc>,
}
