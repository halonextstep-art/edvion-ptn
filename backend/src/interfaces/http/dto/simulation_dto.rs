use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::application::simulation_service::{CreateTemplateInput, CreateTemplateSlotInput, SimulationRunWithQuestions};
use crate::domain::package::ExamTrack;
use crate::domain::school::SchoolType;
use crate::domain::simulation::{
    LockdownViolation, SimulationRunStatus, SimulationSlotStatus, SimulationTemplate, SimulationTemplateKind,
    TemplateCompletionStats,
};
use crate::interfaces::http::dto::tryout_dto::PlayableQuestionResponse;

fn default_break() -> i32 {
    300
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateSlotPayload {
    pub sequence_index: i32,
    /// Required unless `is_elective` — see `domain::simulation::SimulationTemplateSlot` doc
    /// comment.
    #[serde(default)]
    pub session_id: Option<Uuid>,
    #[serde(default = "default_break")]
    pub break_seconds: i32,
    /// See `domain::simulation::SimulationTemplateSlot::is_elective` doc comment. `false` for
    /// every pre-existing Simulasi UTBK slot (unaffected).
    #[serde(default)]
    pub is_elective: bool,
}

fn default_premium() -> bool {
    true
}

fn default_template_kind() -> SimulationTemplateKind {
    SimulationTemplateKind::UtbkCombined
}

fn default_exam_track() -> ExamTrack {
    ExamTrack::Snbt
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTemplatePayload {
    #[validate(length(min = 1, message = "judul wajib diisi"))]
    pub title: String,
    /// Defaults `true` — a Simulasi UTBK / TO template is premium content by default (matching
    /// `20250101000031_package_content_items.sql`'s default for existing rows), unlockable
    /// only once an admin assigns it to a Package. Admin can flip this to `false` for a free
    /// promotional template via `set_template_premium`.
    #[serde(default = "default_premium")]
    pub is_premium: bool,
    pub slots: Vec<CreateTemplateSlotPayload>,
    /// See `domain::simulation::SimulationTemplate::is_draft` doc comment.
    #[serde(default)]
    pub is_draft: bool,
    /// Required (validated in `SimulationService::create_template`) iff at least one slot is
    /// `is_elective`.
    #[serde(default)]
    pub elective_package_id: Option<Uuid>,
    /// `"utbk_combined"` (default) or `"per_subject"` — see
    /// `domain::simulation::SimulationTemplateKind` doc comment.
    #[serde(default = "default_template_kind")]
    pub template_kind: SimulationTemplateKind,
    /// See `domain::package::ExamTrack` doc comment.
    #[serde(default = "default_exam_track")]
    pub exam_track: ExamTrack,
    /// See `domain::tryout::TryoutSession::school_type_scope` doc comment. `None` (omitted) =
    /// tampil ke semua jenjang.
    #[serde(default)]
    pub school_type_scope: Option<SchoolType>,
    /// See `domain::simulation::SimulationTemplate::lockdown_override` doc comment. Omitted or
    /// explicit `null` = inherit the platform-wide default.
    #[serde(default)]
    pub lockdown_override: Option<bool>,
}

impl CreateTemplatePayload {
    pub fn into_input(self) -> CreateTemplateInput {
        CreateTemplateInput {
            title: self.title,
            is_premium: self.is_premium,
            slots: self
                .slots
                .into_iter()
                .map(|s| CreateTemplateSlotInput {
                    sequence_index: s.sequence_index,
                    session_id: s.session_id,
                    break_seconds: s.break_seconds,
                    is_elective: s.is_elective,
                })
                .collect(),
            is_draft: self.is_draft,
            elective_package_id: self.elective_package_id,
            template_kind: self.template_kind,
            exam_track: self.exam_track,
            school_type_scope: self.school_type_scope,
            lockdown_override: self.lockdown_override,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SetActivePayload {
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetPremiumPayload {
    pub is_premium: bool,
}

/// See `domain::simulation::SimulationTemplate::lockdown_override` doc comment. The field is
/// required (no `#[serde(default)]`) so a request body must send either `true`, `false`, or
/// explicit `null` — `null` resets the template to "inherit the global default".
#[derive(Debug, Deserialize)]
pub struct SetLockdownPayload {
    pub lockdown_override: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SimulationTemplateSlotResponse {
    pub sequence_index: i32,
    /// `None` when `is_elective` — no fixed session at the template level, see
    /// `domain::simulation::SimulationTemplateSlot` doc comment.
    pub session_id: Option<Uuid>,
    pub session_title: Option<String>,
    pub duration_minutes: Option<i32>,
    pub question_count: Option<i32>,
    pub subject_filter: Option<String>,
    pub break_seconds: i32,
    pub is_elective: bool,
}

#[derive(Debug, Serialize)]
pub struct SimulationTemplateResponse {
    pub id: Uuid,
    pub title: String,
    pub is_active: bool,
    pub is_premium: bool,
    pub is_draft: bool,
    pub elective_package_id: Option<Uuid>,
    pub template_kind: SimulationTemplateKind,
    /// See `domain::package::ExamTrack` doc comment.
    pub exam_track: ExamTrack,
    /// See `domain::tryout::TryoutSession::school_type_scope` doc comment.
    pub school_type_scope: Option<SchoolType>,
    /// See `domain::simulation::SimulationTemplate::lockdown_override` doc comment. `null` =
    /// inherit the platform-wide default (see `PlatformSettingsResponse::simulation_lockdown_default`).
    pub lockdown_override: Option<bool>,
    /// Sum of every slot's known `duration_minutes` only — `is_elective` slots contribute 0
    /// here (their real duration is only known once a student's run resolves them), so this is
    /// an UNDER-estimate for any template with elective slots. Frontend should say "≈" for such
    /// templates rather than presenting this as exact.
    pub total_duration_minutes: i32,
    pub slots: Vec<SimulationTemplateSlotResponse>,
    pub created_at: DateTime<Utc>,
}

impl From<SimulationTemplate> for SimulationTemplateResponse {
    fn from(t: SimulationTemplate) -> Self {
        let slots: Vec<SimulationTemplateSlotResponse> = t
            .slots
            .into_iter()
            .map(|s| SimulationTemplateSlotResponse {
                sequence_index: s.sequence_index,
                session_id: s.session_id,
                session_title: s.session_title,
                duration_minutes: s.duration_minutes,
                question_count: s.question_count,
                subject_filter: s.subject_filter,
                break_seconds: s.break_seconds,
                is_elective: s.is_elective,
            })
            .collect();
        let total_duration_minutes = slots.iter().filter_map(|s| s.duration_minutes).sum();
        Self {
            id: t.id,
            title: t.title,
            is_active: t.is_active,
            is_premium: t.is_premium,
            is_draft: t.is_draft,
            elective_package_id: t.elective_package_id,
            template_kind: t.template_kind,
            exam_track: t.exam_track,
            school_type_scope: t.school_type_scope,
            lockdown_override: t.lockdown_override,
            total_duration_minutes,
            slots,
            created_at: t.created_at,
        }
    }
}

fn run_status_str(s: SimulationRunStatus) -> &'static str {
    match s {
        SimulationRunStatus::InProgress => "in_progress",
        SimulationRunStatus::Completed => "completed",
        SimulationRunStatus::Abandoned => "abandoned",
    }
}

fn slot_status_str(s: SimulationSlotStatus) -> &'static str {
    match s {
        SimulationSlotStatus::Pending => "pending",
        SimulationSlotStatus::Active => "active",
        SimulationSlotStatus::Submitted => "submitted",
    }
}

#[derive(Debug, Serialize)]
pub struct SimulationRunSlotResponse {
    pub sequence_index: i32,
    pub session_id: Uuid,
    pub session_title: String,
    pub subject_filter: Option<String>,
    pub duration_minutes: i32,
    pub break_seconds: i32,
    pub status: String, // "pending" | "active" | "submitted"
    pub attempt_id: Option<Uuid>,
    pub score: Option<i32>,
    /// Detail breakdown for the sertifikat/laporan PDF's per-subtest table + strength/weakness
    /// analysis — `None` until this slot's attempt is submitted.
    pub accuracy: Option<f64>,
    pub correct_count: Option<i32>,
    pub wrong_count: Option<i32>,
    pub unanswered_count: Option<i32>,
    pub deadline_at: Option<DateTime<Utc>>,
    pub break_ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct SimulationRunResponse {
    pub id: Uuid,
    /// Lets the frontend catalog match a completed run back to the template it belongs to
    /// (e.g. to swap "Mulai Simulasi" for "Lihat Hasil") — `template_title` alone isn't a safe
    /// join key since titles aren't guaranteed unique.
    pub template_id: Uuid,
    pub template_title: String,
    pub template_kind: SimulationTemplateKind,
    /// See `domain::simulation::SimulationRun::school_type_scope` doc comment.
    pub school_type_scope: Option<SchoolType>,
    pub status: String, // "in_progress" | "completed" | "abandoned"
    pub current_sequence_index: i32,
    /// Always an ESTIMATE — never a real IRT-calibrated UTBK score. ALWAYS `None` when
    /// `template_kind = "per_subject"` (e.g. Simulasi TKA) — real TKA reports a score PER MATA
    /// UJI and never combines them; the frontend must read each slot's own `score` instead. See
    /// `application::simulation_service` doc comment.
    pub combined_estimate_score: Option<f64>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    /// See `domain::simulation::SimulationRun::lockdown_enabled` doc comment — immutable
    /// snapshot taken at `start_run` time. When `true`, the frontend player must enforce "Mode
    /// Terkunci" (fullscreen, hide navigation, block copy/tab-switch, report violations).
    pub lockdown_enabled: bool,
    pub slots: Vec<SimulationRunSlotResponse>,
    /// Only populated when the current slot is actively in-progress — the exact questions to
    /// render right now. Reuses the same `PlayableQuestionResponse` shape already used by the
    /// existing single-session tryout player, so the frontend player component can be shared.
    pub current_questions: Option<Vec<PlayableQuestionResponse>>,
}

/// `SimulationRunWithQuestions` combines a `SimulationRun` (domain) with an
/// `Option<Vec<PlayableQuestion>>` from a different source type, so a plain function is used
/// instead of `From` for clarity.
pub fn build_run_response(x: SimulationRunWithQuestions) -> SimulationRunResponse {
    let run = x.run;
    let slots = run
        .slots
        .into_iter()
        .map(|s| SimulationRunSlotResponse {
            sequence_index: s.sequence_index,
            session_id: s.session_id,
            session_title: s.session_title,
            subject_filter: s.subject_filter,
            duration_minutes: s.duration_minutes,
            break_seconds: s.break_seconds,
            status: slot_status_str(s.status).to_string(),
            attempt_id: s.attempt_id,
            score: s.attempt_score,
            accuracy: s.attempt_accuracy,
            correct_count: s.attempt_correct_count,
            wrong_count: s.attempt_wrong_count,
            unanswered_count: s.attempt_unanswered_count,
            deadline_at: s.deadline_at,
            break_ends_at: s.break_ends_at,
        })
        .collect();
    SimulationRunResponse {
        id: run.id,
        template_id: run.template_id,
        template_title: run.template_title,
        template_kind: run.template_kind,
        school_type_scope: run.school_type_scope,
        status: run_status_str(run.status).to_string(),
        current_sequence_index: run.current_sequence_index,
        combined_estimate_score: run.combined_estimate_score,
        started_at: run.started_at,
        completed_at: run.completed_at,
        lockdown_enabled: run.lockdown_enabled,
        slots,
        current_questions: x.current_questions.map(|qs| qs.into_iter().map(Into::into).collect()),
    }
}

#[derive(Debug, Deserialize)]
pub struct ReportViolationPayload {
    /// Free-form but conventionally one of: "tab_switch", "fullscreen_exit",
    /// "devtools_attempt", "copy_attempt", "context_menu_attempt", "window_blur" — see
    /// `domain::simulation::LockdownViolation::event_type` doc comment.
    pub event_type: String,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LockdownViolationResponse {
    pub id: Uuid,
    pub event_type: String,
    pub detail: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

impl From<LockdownViolation> for LockdownViolationResponse {
    fn from(v: LockdownViolation) -> Self {
        Self { id: v.id, event_type: v.event_type, detail: v.detail, occurred_at: v.occurred_at }
    }
}

/// See `domain::simulation::TemplateCompletionStats` doc comment.
#[derive(Debug, Serialize)]
pub struct TemplateStatsResponse {
    pub template_id: Uuid,
    pub participant_count: i64,
    pub avg_combined_score: Option<f64>,
}

impl From<TemplateCompletionStats> for TemplateStatsResponse {
    fn from(s: TemplateCompletionStats) -> Self {
        Self { template_id: s.template_id, participant_count: s.participant_count, avg_combined_score: s.avg_combined_score }
    }
}
