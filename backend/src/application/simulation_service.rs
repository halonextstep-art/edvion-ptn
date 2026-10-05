//! Simulasi UTBK — chains several existing `TryoutSession`s into one continuous, server-
//! enforced exam run. See `domain::simulation` doc comment and the
//! `20250101000025_simulation.sql` migration for the full rationale. This service is the
//! state machine: it owns which slot is active, whether time/break has elapsed, and drives
//! `TryoutService::start_attempt`/`submit_attempt`/`get_attempt`/`get_attempt_questions` as a
//! collaborator rather than duplicating any scoring/grading/ownership logic itself.
//!
//! `combined_estimate_score` is ALWAYS an estimate — an equal-weighted average across slots,
//! rescaled from the platform's 0-1000 scale to the same 0-825 UTBK ballpark used by
//! `est_score` elsewhere (see `UTBK_SCALE_MAX`/`PLATFORM_SCALE_MAX` in
//! `rationalization_service.rs`) — never a real IRT-calibrated score.

use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::application::access_service::AccessService;
use crate::application::elective_service::ElectiveService;
use crate::application::platform_settings_service::PlatformSettingsService;
use crate::application::tryout_service::{PlayableQuestion, TryoutService};
use crate::domain::package::ExamTrack;
use crate::domain::repository::{
    LockdownViolationRepository, NewLockdownViolation, NewSimulationRun, NewSimulationRunSlot, NewSimulationTemplate,
    NewSimulationTemplateSlot, PackageRepository, SimulationRunRepository, SimulationTemplateRepository,
    TryoutSessionRepository,
};
use crate::domain::school::SchoolType;
use crate::domain::simulation::{
    LockdownViolation, SimulationRun, SimulationRunSlot, SimulationRunStatus, SimulationSlotStatus,
    SimulationTemplate, SimulationTemplateKind, TemplateCompletionStats,
};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct CreateTemplateSlotInput {
    pub sequence_index: i32,
    /// `None` iff `is_elective` — see `domain::simulation::SimulationTemplateSlot` doc comment.
    pub session_id: Option<Uuid>,
    pub break_seconds: i32,
    pub is_elective: bool,
}

pub struct CreateTemplateInput {
    pub title: String,
    pub is_premium: bool,
    pub slots: Vec<CreateTemplateSlotInput>,
    /// See `domain::simulation::SimulationTemplate::is_draft` doc comment. The "Buat Paket +
    /// Subtes" wizard always sends `true`; the ordinary SimulationManager form sends `false`.
    pub is_draft: bool,
    /// Required (validated in `create_template`) iff at least one slot is `is_elective` — which
    /// Package's "mapel pilihan" choices resolve those slots per-student at `start_run` time.
    pub elective_package_id: Option<Uuid>,
    pub template_kind: SimulationTemplateKind,
    /// See `domain::package::ExamTrack` doc comment.
    pub exam_track: ExamTrack,
    /// See `domain::tryout::TryoutSession::school_type_scope` doc comment.
    pub school_type_scope: Option<SchoolType>,
    /// See `domain::simulation::SimulationTemplate::lockdown_override` doc comment.
    pub lockdown_override: Option<bool>,
}

/// Bundled so a single endpoint call always gives the frontend BOTH the run's current state
/// AND (if the current slot is actively in-progress) the playable questions to render right
/// now — the frontend never needs a second round-trip to know what to show.
pub struct SimulationRunWithQuestions {
    pub run: SimulationRun,
    pub current_questions: Option<Vec<PlayableQuestion>>,
}

pub struct SimulationService {
    templates: Arc<dyn SimulationTemplateRepository>,
    runs: Arc<dyn SimulationRunRepository>,
    sessions: Arc<dyn TryoutSessionRepository>,
    tryout: Arc<TryoutService>,
    access: Arc<AccessService>,
    /// Used two ways: (1) `create_template` validates `elective_package_id` points at a real
    /// package, (2) `start_run` reads a student's current elective picks to resolve `is_elective`
    /// slots — see `application::elective_service` doc comment.
    elective: Arc<ElectiveService>,
    packages: Arc<dyn PackageRepository>,
    /// Read once at `start_run` time to resolve a template's effective "Mode Terkunci" decision
    /// when it has no explicit `lockdown_override` — see `PlatformSettings::simulation_lockdown_default`
    /// doc comment.
    platform_settings: Arc<PlatformSettingsService>,
    violations: Arc<dyn LockdownViolationRepository>,
}

impl SimulationService {
    pub fn new(
        templates: Arc<dyn SimulationTemplateRepository>,
        runs: Arc<dyn SimulationRunRepository>,
        sessions: Arc<dyn TryoutSessionRepository>,
        tryout: Arc<TryoutService>,
        access: Arc<AccessService>,
        elective: Arc<ElectiveService>,
        packages: Arc<dyn PackageRepository>,
        platform_settings: Arc<PlatformSettingsService>,
        violations: Arc<dyn LockdownViolationRepository>,
    ) -> Self {
        Self { templates, runs, sessions, tryout, access, elective, packages, platform_settings, violations }
    }

    /// `Role::Content` may author templates too (the "Buat Paket + Subtes" wizard) — mirrors
    /// `QuestionService::create`'s Admin/Content split.
    pub async fn create_template(&self, actor: &AuthUser, input: CreateTemplateInput) -> AppResult<SimulationTemplate> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        if input.slots.is_empty() {
            return Err(AppError::Validation("minimal 1 subtes diperlukan".to_string()));
        }
        let mut indices: Vec<i32> = input.slots.iter().map(|s| s.sequence_index).collect();
        indices.sort_unstable();
        for (i, idx) in indices.iter().enumerate() {
            if *idx != (i as i32 + 1) {
                return Err(AppError::Validation(
                    "urutan subtes harus 1..N berurutan tanpa lubang atau duplikat".to_string(),
                ));
            }
        }
        let elective_slot_count = input.slots.iter().filter(|s| s.is_elective).count();
        for slot in &input.slots {
            if slot.break_seconds < 0 {
                return Err(AppError::Validation("jeda tidak boleh negatif".to_string()));
            }
            if slot.is_elective {
                if slot.session_id.is_some() {
                    return Err(AppError::Validation(
                        "slot mapel pilihan tidak boleh memiliki sesi tetap — sesi ditentukan otomatis dari pilihan siswa"
                            .to_string(),
                    ));
                }
            } else {
                let session_id = slot.session_id.ok_or_else(|| {
                    AppError::Validation("slot bukan mapel pilihan wajib memiliki sesi tetap".to_string())
                })?;
                self.sessions
                    .find_by_id(session_id)
                    .await?
                    .ok_or_else(|| AppError::NotFound(format!("session {session_id} not found")))?;
            }
        }
        if elective_slot_count > 0 {
            let package_id = input.elective_package_id.ok_or_else(|| {
                AppError::Validation(
                    "template dengan slot mapel pilihan wajib menentukan paket sumber pilihan (elective_package_id)"
                        .to_string(),
                )
            })?;
            let pkg = self
                .packages
                .find_by_id(package_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("package {package_id} not found")))?;
            if pkg.elective_pick_count as usize != elective_slot_count {
                return Err(AppError::Validation(format!(
                    "jumlah slot mapel pilihan ({elective_slot_count}) harus sama dengan elective_pick_count paket \"{}\" ({})",
                    pkg.name, pkg.elective_pick_count
                )));
            }
        } else if input.elective_package_id.is_some() {
            return Err(AppError::Validation(
                "elective_package_id hanya berlaku jika ada slot mapel pilihan".to_string(),
            ));
        }
        self.templates
            .create(NewSimulationTemplate {
                title: input.title,
                is_premium: input.is_premium,
                created_by: actor.user_id,
                slots: input
                    .slots
                    .into_iter()
                    .map(|s| NewSimulationTemplateSlot {
                        sequence_index: s.sequence_index,
                        session_id: s.session_id,
                        break_seconds: s.break_seconds,
                        is_elective: s.is_elective,
                    })
                    .collect(),
                is_draft: input.is_draft,
                elective_package_id: input.elective_package_id,
                template_kind: input.template_kind,
                exam_track: input.exam_track,
                school_type_scope: input.school_type_scope,
                lockdown_override: input.lockdown_override,
            })
            .await
    }

    /// See `domain::simulation::SimulationTemplate::lockdown_override` doc comment. Pass `None`
    /// to reset the template back to "inherit the global default".
    pub async fn set_template_lockdown(
        &self,
        actor: &AuthUser,
        id: Uuid,
        lockdown_override: Option<bool>,
    ) -> AppResult<SimulationTemplate> {
        actor.require_role(&[Role::Admin])?;
        self.templates
            .set_lockdown(id, lockdown_override)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("template {id} not found")))
    }

    /// Admin sees everything including every draft. Content sees every published template
    /// plus their own drafts (not colleagues' — templates are heavier/more "final" than a
    /// single session, so this is intentionally tighter than `TryoutService::list_sessions`).
    /// Student never sees a draft.
    /// See `TryoutService::list_sessions` doc comment — a `Role::Student` also never sees a
    /// template scoped to a jenjang other than their own school's, AND never sees an
    /// `exam_track: Snbt` template at all if their jenjang is SMP (hard categorical rule,
    /// independent of `school_type_scope` — SNBT/UTBK is never taken by SMP students, so this
    /// also covers every pre-existing SNBT template without needing a manual re-tag). B2C
    /// students unaffected by either rule.
    pub async fn list_templates(&self, actor: &AuthUser) -> AppResult<Vec<SimulationTemplate>> {
        match actor.role {
            Role::Admin => self.templates.list(false).await,
            Role::Content => {
                let all = self.templates.list(false).await?;
                Ok(all.into_iter().filter(|t| !t.is_draft || t.created_by == actor.user_id).collect())
            }
            Role::Student => {
                let all = self.templates.list(true).await?;
                let mut visible: Vec<SimulationTemplate> = all.into_iter().filter(|t| !t.is_draft).collect();
                if let Some(student_type) = self.access.student_school_type(actor.user_id).await? {
                    visible.retain(|t| match t.school_type_scope {
                        None => true,
                        Some(scope) => scope == student_type,
                    });
                    if student_type == SchoolType::Smp {
                        visible.retain(|t| t.exam_track != ExamTrack::Snbt);
                    }
                }
                Ok(visible)
            }
            _ => Err(AppError::Forbidden("role tidak diizinkan".to_string())),
        }
    }

    pub async fn set_template_active(&self, actor: &AuthUser, id: Uuid, active: bool) -> AppResult<SimulationTemplate> {
        actor.require_role(&[Role::Admin])?;
        self.templates
            .set_active(id, active)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("template {id} not found")))
    }

    pub async fn set_template_premium(&self, actor: &AuthUser, id: Uuid, is_premium: bool) -> AppResult<SimulationTemplate> {
        actor.require_role(&[Role::Admin])?;
        self.templates
            .set_premium(id, is_premium)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("template {id} not found")))
    }

    /// Called directly by `PackageService::set_active`'s publish cascade and by an owner
    /// manually un-hiding/re-hiding their own draft template.
    pub async fn set_template_draft(&self, id: Uuid, is_draft: bool) -> AppResult<SimulationTemplate> {
        self.templates
            .set_draft(id, is_draft)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("template {id} not found")))
    }

    pub async fn delete_template(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        if actor.role != Role::Admin {
            let existing = self
                .templates
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("template {id} not found")))?;
            if existing.created_by != actor.user_id {
                return Err(AppError::Forbidden("Anda hanya bisa menghapus simulasi yang Anda buat sendiri".to_string()));
            }
        }
        if !self.templates.delete(id).await? {
            return Err(AppError::NotFound(format!("template {id} not found")));
        }
        Ok(())
    }

    pub async fn start_run(&self, actor: &AuthUser, template_id: Uuid) -> AppResult<SimulationRunWithQuestions> {
        actor.require_role(&[Role::Student])?;
        let existing = self.runs.list_by_student(actor.user_id).await?;
        // Every Simulasi TO template can only be COMPLETED once per student — it's a formal,
        // timed mock exam (not a repeatable drill like Latihan/Drilling), so a finished run
        // permanently blocks starting a fresh one for the same template. `myRuns`/"Riwayat
        // Simulasi" is where the student reviews a completed attempt afterward.
        if existing.iter().any(|r| r.template_id == template_id && r.status == SimulationRunStatus::Completed) {
            return Err(AppError::Validation(
                "Anda sudah menyelesaikan simulasi ini sebelumnya — setiap Simulasi TO hanya bisa dikerjakan sekali. Lihat hasilnya di riwayat simulasi.".to_string(),
            ));
        }
        // `list_by_student` returns the raw DB rows as-is — it does NOT run the same
        // server-authoritative timing reconciliation `get_run` does (auto-submit an expired
        // active slot, auto-advance past an elapsed break, finish the run once the last slot
        // is past its break). So a run the student genuinely finished — but then closed the
        // tab/browser during the final break instead of waiting for auto-advance or clicking
        // "Lanjut" — stays `InProgress` in the DB FOREVER unless something happens to call
        // `get_run` on that exact run again. That stale row then permanently blocks every
        // future `start_run` call with a confusing "still have one running" 422, even though
        // from the student's perspective they already finished. Reconcile each candidate here
        // (lazy-sweep-on-read, same pattern as `PaymentService::list_pending`'s pending→expired
        // sweep) before deciding whether anything is genuinely still in progress.
        for r in existing {
            if r.status == SimulationRunStatus::InProgress {
                let reconciled = self.reconcile(actor, r).await?;
                if reconciled.status == SimulationRunStatus::InProgress {
                    // `reconcile` only ever resolves an `Active` or `Submitted` current slot —
                    // it can't do anything for a `Pending` one. Slot 1 legitimately sits
                    // `Pending` from the moment a run is created until the student confirms
                    // "Mulai Sekarang" on the pre-exam overview screen (see `begin_run`) — the
                    // exam clock deliberately doesn't start before that click. If the student
                    // never comes back to confirm (closed the tab, or `begin_run`'s own
                    // `activate_slot` call failed), this run is stuck here FOREVER with no
                    // attempt ever created and nothing ever shown to the student. Auto-abandon
                    // it instead of permanently locking them out of starting anything new — zero
                    // work was ever done on it, nothing to lose.
                    let stuck_pending = reconciled
                        .current_slot()
                        .map(|s| s.status == SimulationSlotStatus::Pending && s.attempt_id.is_none())
                        .unwrap_or(false);
                    if stuck_pending {
                        self.runs
                            .set_run_status(reconciled.id, SimulationRunStatus::Abandoned, Some(Utc::now()), None)
                            .await?;
                    } else {
                        return Err(AppError::Validation(
                            "Anda masih punya simulasi yang sedang berjalan — selesaikan dulu sebelum memulai yang baru"
                                .to_string(),
                        ));
                    }
                }
            }
        }
        let template = self
            .templates
            .find_by_id(template_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("template {template_id} not found")))?;
        if !template.is_active {
            return Err(AppError::Validation("template simulasi ini tidak aktif".to_string()));
        }
        if template.is_premium && !self.access.has_access_to_simulation_template(actor.user_id, template_id).await? {
            return Err(AppError::Forbidden(
                "simulasi ini khusus siswa dengan akses premium (voucher pribadi atau paket sekolah aktif)".to_string(),
            ));
        }
        // Resolve any `is_elective` slots (e.g. TKA "Hari 2 — Mapel Pilihan") to THIS student's
        // current picks before the run is created — a run's slots are an immutable, fully-
        // resolved snapshot (mirrors how every other slot already works), never re-resolved
        // later even if the student changes their picks afterward.
        let mut elective_session_ids: std::collections::VecDeque<Uuid> = std::collections::VecDeque::new();
        if let Some(package_id) = template.elective_package_id {
            let elective_slot_count = template.slots.iter().filter(|s| s.is_elective).count();
            let choices = self.elective.my_choices(actor, package_id).await?;
            if choices.len() != elective_slot_count {
                return Err(AppError::Validation(format!(
                    "pilih dulu {elective_slot_count} mapel pilihan Anda di halaman \"Pilih Mapel Pilihan\" sebelum memulai simulasi ini (saat ini baru {} dipilih)",
                    choices.len()
                )));
            }
            elective_session_ids = choices.into_iter().collect();
        }
        let mut run_slots = Vec::with_capacity(template.slots.len());
        for s in &template.slots {
            let session_id = if s.is_elective {
                elective_session_ids.pop_front().ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!("ran out of resolved elective choices for template {template_id}"))
                })?
            } else {
                s.session_id.ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!("non-elective template slot {} has no session_id", s.id))
                })?
            };
            run_slots.push(NewSimulationRunSlot {
                sequence_index: s.sequence_index,
                session_id,
                break_seconds: s.break_seconds,
            });
        }
        // Resolved once here and snapshotted immutably onto the run — see
        // `SimulationRun::lockdown_enabled` doc comment for why this is never re-read live.
        let global_default = self.platform_settings.simulation_lockdown_default().await?;
        let lockdown_enabled = template.effective_lockdown(global_default);
        // Deliberately does NOT activate slot 1 here — `create` leaves it `Pending` (no attempt,
        // no deadline). The frontend shows a pre-exam overview screen ("Mode Terkunci" warning,
        // subtest list, total duration) first; the exam clock only starts once the student
        // explicitly confirms via `begin_run`. This makes the overview genuinely gate the timer
        // rather than being a cosmetic card layered on top of a clock that's already ticking.
        let run = self
            .runs
            .create(NewSimulationRun { template_id, student_id: actor.user_id, slots: run_slots, lockdown_enabled })
            .await?;
        self.get_run(actor, run.id).await
    }

    /// Called when the student clicks "Mulai Sekarang" on the pre-exam overview screen —
    /// activates slot 1 for the first time, which is also the moment the exam clock genuinely
    /// starts (`activate_slot` sets `deadline_at = now() + duration`). Idempotent: calling it
    /// again on a run whose slot 1 is already past `Pending` just returns the current state
    /// instead of erroring, so a duplicate click/retry is harmless.
    pub async fn begin_run(&self, actor: &AuthUser, run_id: Uuid) -> AppResult<SimulationRunWithQuestions> {
        actor.require_role(&[Role::Student])?;
        let run = self
            .runs
            .find_by_id(run_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("run {run_id} not found")))?;
        self.ensure_owned(actor, &run)?;
        if run.status != SimulationRunStatus::InProgress {
            return Err(AppError::Validation("simulasi ini sudah tidak sedang berjalan".to_string()));
        }
        let slot = run
            .current_slot()
            .cloned()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("run {run_id} has no current slot")))?;
        if slot.status == SimulationSlotStatus::Pending {
            self.activate_slot(actor, run_id, slot.sequence_index).await?;
        }
        self.get_run(actor, run_id).await
    }

    /// Single source of truth the frontend polls: reconciles server-authoritative timing
    /// (auto-submits an expired active slot, auto-advances past an elapsed break) BEFORE
    /// returning the current state — the client's own countdown timers are cosmetic only.
    pub async fn get_run(&self, actor: &AuthUser, run_id: Uuid) -> AppResult<SimulationRunWithQuestions> {
        let mut run = self
            .runs
            .find_by_id(run_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("run {run_id} not found")))?;
        self.ensure_owned(actor, &run)?;
        if run.status == SimulationRunStatus::InProgress {
            run = self.reconcile(actor, run).await?;
        }
        let current_questions = self.load_current_questions(actor, &run).await?;
        Ok(SimulationRunWithQuestions { run, current_questions })
    }

    /// Manual early-submit of the CURRENT active slot (mirrors the existing single-session
    /// player's manual "Submit" button) — still cannot skip the break afterward.
    pub async fn submit_current(&self, actor: &AuthUser, run_id: Uuid) -> AppResult<SimulationRunWithQuestions> {
        actor.require_role(&[Role::Student])?;
        let run = self
            .runs
            .find_by_id(run_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("run {run_id} not found")))?;
        self.ensure_owned(actor, &run)?;
        if run.status != SimulationRunStatus::InProgress {
            // Idempotent, NOT an error: a duplicate/retried submit call on a run that has
            // already finished (whole simulation completed/abandoned) previously 422'd here
            // even though nothing is actually wrong — hand back the current run state instead,
            // same rationale as `TryoutService::submit_attempt`'s idempotent branch.
            return self.get_run(actor, run_id).await;
        }
        let slot = run
            .current_slot()
            .cloned()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("run {run_id} has no current slot")))?;
        if slot.status != SimulationSlotStatus::Active {
            // Same idempotency rationale, narrower case: this slot was already submitted (e.g.
            // the resume-time auto-submit and a lingering Submit-button click both fired) —
            // the underlying attempt submit is itself idempotent now, so just return current state.
            return self.get_run(actor, run_id).await;
        }
        let attempt_id = slot
            .attempt_id
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("active slot has no attempt_id")))?;
        let attempt = self.tryout.get_attempt(actor, attempt_id).await?;
        let elapsed = (Utc::now() - attempt.started_at).num_seconds().max(0) as i32;
        let capped = elapsed.min(slot.duration_minutes * 60);
        self.tryout.submit_attempt(actor, attempt_id, capped).await?;
        self.finalize_slot_submission(actor, &run, &slot).await?;
        self.get_run(actor, run_id).await
    }

    /// Called by the "Lanjut ke Subtes Berikutnya" button — server independently re-checks
    /// that the break has genuinely elapsed; a client that races ahead of its own countdown
    /// gets rejected, not silently allowed through.
    pub async fn advance_after_break(&self, actor: &AuthUser, run_id: Uuid) -> AppResult<SimulationRunWithQuestions> {
        actor.require_role(&[Role::Student])?;
        let run = self
            .runs
            .find_by_id(run_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("run {run_id} not found")))?;
        self.ensure_owned(actor, &run)?;
        if run.status != SimulationRunStatus::InProgress {
            // Idempotent, NOT an error: the run may have already finished via a concurrent
            // call (e.g. this was the last slot and `reconcile`/another request already
            // completed it) — same rationale as `submit_current`'s idempotent branch below.
            return self.get_run(actor, run_id).await;
        }
        let slot = run
            .current_slot()
            .cloned()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("run {run_id} has no current slot")))?;
        if slot.status != SimulationSlotStatus::Submitted {
            // Idempotent, NOT an error: something else already advanced this run past the
            // break we were about to skip — the client's own auto-advance ticker retrying a
            // request that raced with (or duplicated) another one, or the server's own
            // `reconcile()` auto-advancing inside a concurrent `get_run` call. The end state
            // the caller wants — "past this break" — is already true, so hand back current
            // state instead of hard-erroring on what is really a harmless duplicate call.
            return self.get_run(actor, run_id).await;
        }
        if let Some(break_ends) = slot.break_ends_at {
            if Utc::now() < break_ends {
                return Err(AppError::Validation("jeda belum selesai".to_string()));
            }
        }
        let run = self.advance_to_next(actor, run).await?;
        let current_questions = self.load_current_questions(actor, &run).await?;
        Ok(SimulationRunWithQuestions { run, current_questions })
    }

    /// Self-service escape hatch: lets a student give up on a stuck `in_progress` run (e.g.
    /// closed the tab mid-simulation days ago and never came back) so they aren't PERMANENTLY
    /// blocked from starting a new one — `start_run` refuses to start while any run is
    /// `in_progress`, and until this method existed nothing in the codebase ever transitioned
    /// a run to `Abandoned` (the variant existed in the DB/domain but was dead code). Does NOT
    /// touch the underlying `Attempt` rows for slots already played — those keep whatever
    /// score they already have, only the run's own status/completed_at changes.
    pub async fn abandon_run(&self, actor: &AuthUser, run_id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Student])?;
        let run = self
            .runs
            .find_by_id(run_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("run {run_id} not found")))?;
        self.ensure_owned(actor, &run)?;
        if run.status != SimulationRunStatus::InProgress {
            return Err(AppError::Validation("simulasi ini sudah tidak sedang berjalan".to_string()));
        }
        self.runs
            .set_run_status(run_id, SimulationRunStatus::Abandoned, Some(Utc::now()), None)
            .await
    }

    /// Honest, aggregate-only benchmark for the sertifikat/laporan PDF's "posisi kamu" line —
    /// see `domain::simulation::TemplateCompletionStats` doc comment. Open to Student (any
    /// authenticated student, not just the template's own past participants — it's aggregate,
    /// nothing per-student leaks) and Admin.
    pub async fn template_stats(&self, actor: &AuthUser, template_id: Uuid) -> AppResult<TemplateCompletionStats> {
        actor.require_role(&[Role::Student, Role::Admin])?;
        self.templates
            .find_by_id(template_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("template {template_id} not found")))?;
        self.runs.template_completion_stats(template_id).await
    }

    pub async fn list_my_runs(&self, actor: &AuthUser) -> AppResult<Vec<SimulationRun>> {
        actor.require_role(&[Role::Student])?;
        self.runs.list_by_student(actor.user_id).await
    }

    /// Admin-only equivalent of `list_my_runs`, scoped to any student — see
    /// `TryoutService::admin_list_attempts` doc comment for why this needs to exist at all
    /// (previously there was NO way for Admin to see a student's in-progress Simulasi run).
    pub async fn admin_list_runs(&self, actor: &AuthUser, student_id: Uuid) -> AppResult<Vec<SimulationRun>> {
        actor.require_role(&[Role::Admin])?;
        self.runs.list_by_student(student_id).await
    }

    /// Called by the "Mode Terkunci" frontend enforcement in the Simulasi TO player whenever it
    /// detects a breach (tab switch, fullscreen exit, devtools attempt, etc.) — student-only,
    /// scoped to their own run. Deliberately does NOT check `run.lockdown_enabled` before
    /// recording: if the frontend somehow calls this on a non-locked run, logging one harmless
    /// extra row is safer than silently dropping a signal that might matter.
    pub async fn record_violation(
        &self,
        actor: &AuthUser,
        run_id: Uuid,
        event_type: String,
        detail: Option<String>,
    ) -> AppResult<LockdownViolation> {
        actor.require_role(&[Role::Student])?;
        let run = self
            .runs
            .find_by_id(run_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("run {run_id} not found")))?;
        self.ensure_owned(actor, &run)?;
        self.violations
            .record(NewLockdownViolation { run_id, student_id: actor.user_id, event_type, detail })
            .await
    }

    /// The student can review their own run's violation history (e.g. to see the running count
    /// alongside the in-exam warning banner); Admin can review any student's — `ensure_owned`
    /// already grants both.
    pub async fn list_violations(&self, actor: &AuthUser, run_id: Uuid) -> AppResult<Vec<LockdownViolation>> {
        let run = self
            .runs
            .find_by_id(run_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("run {run_id} not found")))?;
        self.ensure_owned(actor, &run)?;
        self.violations.list_by_run(run_id).await
    }

    // ── internals ──

    fn ensure_owned(&self, actor: &AuthUser, run: &SimulationRun) -> AppResult<()> {
        if run.student_id != actor.user_id && actor.role != Role::Admin {
            return Err(AppError::Forbidden("bukan simulasi Anda".to_string()));
        }
        Ok(())
    }

    async fn activate_slot(&self, actor: &AuthUser, run_id: Uuid, sequence_index: i32) -> AppResult<()> {
        let run = self
            .runs
            .find_by_id(run_id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("run {run_id} vanished")))?;
        let slot = run
            .slots
            .iter()
            .find(|s| s.sequence_index == sequence_index)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("slot {sequence_index} not found on run {run_id}")))?;
        let attempt_with_q = self.tryout.start_attempt_for_simulation(actor, slot.session_id).await?;
        let deadline = Utc::now() + Duration::minutes(attempt_with_q.attempt.duration_minutes as i64);
        self.runs.activate_slot(run_id, sequence_index, attempt_with_q.attempt.id, deadline).await
    }

    /// Called right after an attempt has just been submitted (either manually via
    /// `submit_current`, or auto-submitted on deadline expiry inside `reconcile`) — decides
    /// whether the run needs a break before continuing. The LAST slot of a run never gets a
    /// break: there is no next subtest to prepare for, so making the student sit through a
    /// "jeda" countdown before seeing their result is pure friction with no purpose, and was
    /// confusing enough that students reported it as a bug. Every other slot still gets its
    /// configured `break_seconds` exactly as before.
    async fn finalize_slot_submission(
        &self,
        actor: &AuthUser,
        run: &SimulationRun,
        slot: &SimulationRunSlot,
    ) -> AppResult<SimulationRun> {
        let is_last_slot = slot.sequence_index >= run.slots.len() as i32;
        if is_last_slot {
            self.runs.submit_slot(run.id, slot.sequence_index, None).await?;
            let run = self
                .runs
                .find_by_id(run.id)
                .await?
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("run {} vanished after final submit", run.id)))?;
            // Go straight to finishing — no break to wait out on the last slot.
            self.advance_to_next(actor, run).await
        } else {
            let break_ends = Utc::now() + Duration::seconds(slot.break_seconds as i64);
            self.runs.submit_slot(run.id, slot.sequence_index, Some(break_ends)).await?;
            self.runs
                .find_by_id(run.id)
                .await?
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("run {} vanished after submit", run.id)))
        }
    }

    /// Re-evaluates the run's current slot against wall-clock time: auto-submits an expired
    /// `active` slot (mirroring what the client's own countdown would have done, as a
    /// server-side safety net independent of whether the client actually called submit), and
    /// auto-advances past a `submitted` slot whose break has fully elapsed. Loops (not just a
    /// single `if`) in case enough real time passed that multiple transitions are due at once
    /// (e.g. student closed the tab for an hour).
    async fn reconcile(&self, actor: &AuthUser, mut run: SimulationRun) -> AppResult<SimulationRun> {
        loop {
            let Some(slot) = run.current_slot().cloned() else { break };
            match slot.status {
                SimulationSlotStatus::Active => {
                    let Some(deadline) = slot.deadline_at else { break };
                    if Utc::now() < deadline {
                        break;
                    }
                    let attempt_id = slot
                        .attempt_id
                        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("active slot has no attempt_id")))?;
                    // Time's up — submit with the full allotted duration as time_used.
                    self.tryout.submit_attempt(actor, attempt_id, slot.duration_minutes * 60).await?;
                    run = self.finalize_slot_submission(actor, &run, &slot).await?;
                    if run.status != SimulationRunStatus::InProgress {
                        break;
                    }
                }
                SimulationSlotStatus::Submitted => {
                    let Some(break_ends) = slot.break_ends_at else { break };
                    if Utc::now() < break_ends {
                        break;
                    }
                    run = self.advance_to_next(actor, run).await?;
                    if run.status != SimulationRunStatus::InProgress {
                        break;
                    }
                }
                SimulationSlotStatus::Pending => break,
            }
        }
        Ok(run)
    }

    async fn advance_to_next(&self, actor: &AuthUser, run: SimulationRun) -> AppResult<SimulationRun> {
        let next_seq = run.current_sequence_index + 1;
        if next_seq > run.slots.len() as i32 {
            self.finish_run(actor, &run).await?;
        } else {
            self.runs.advance_current_index(run.id, next_seq).await?;
            self.activate_slot(actor, run.id, next_seq).await?;
        }
        self.runs
            .find_by_id(run.id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("run {} vanished after advance", run.id)))
    }

    /// Combined score is an equal-weighted average across slots (so a subtest with more
    /// questions doesn't dominate), computed on the platform's existing 0-1000 scale, then
    /// rescaled to the same 0-825 UTBK ballpark used by `est_score` elsewhere in this
    /// codebase — see `UTBK_SCALE_MAX`/`PLATFORM_SCALE_MAX` in `rationalization_service.rs`.
    /// ALWAYS an estimate, never presented as a real calibrated score.
    ///
    /// Only computed for `SimulationTemplateKind::UtbkCombined` templates — real TKA reports a
    /// score PER MATA UJI and never combines them into one composite for the student (see
    /// `domain::simulation` doc comment), so a `PerSubject` template's run always finishes with
    /// `combined_estimate_score = None` and the frontend must read each slot's own attempt score.
    async fn finish_run(&self, actor: &AuthUser, run: &SimulationRun) -> AppResult<()> {
        const UTBK_SCALE_MAX: f64 = 825.0;
        const PLATFORM_SCALE_MAX: f64 = 1000.0;
        let template = self.templates.find_by_id(run.template_id).await?;
        let should_combine = template
            .map(|t| t.template_kind == SimulationTemplateKind::UtbkCombined)
            .unwrap_or(true);
        let combined = if !should_combine {
            None
        } else {
            let mut scores: Vec<f64> = Vec::new();
            for slot in &run.slots {
                if let Some(attempt_id) = slot.attempt_id {
                    if let Ok(attempt) = self.tryout.get_attempt(actor, attempt_id).await {
                        if let Some(score) = attempt.score {
                            scores.push(score as f64);
                        }
                    }
                }
            }
            if scores.is_empty() {
                None
            } else {
                let avg = scores.iter().sum::<f64>() / scores.len() as f64;
                Some((avg / PLATFORM_SCALE_MAX) * UTBK_SCALE_MAX)
            }
        };
        self.runs
            .set_run_status(run.id, SimulationRunStatus::Completed, Some(Utc::now()), combined)
            .await
    }

    async fn load_current_questions(&self, actor: &AuthUser, run: &SimulationRun) -> AppResult<Option<Vec<PlayableQuestion>>> {
        let Some(slot) = run.current_slot() else { return Ok(None) };
        if slot.status != SimulationSlotStatus::Active {
            return Ok(None);
        }
        let Some(attempt_id) = slot.attempt_id else { return Ok(None) };
        Ok(Some(self.tryout.get_attempt_questions(actor, attempt_id).await?))
    }
}
