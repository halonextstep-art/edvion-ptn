//! Drilling/Tryout use-cases: browse session templates, start an attempt (assigning a
//! randomized set of approved questions), autosave answers as the student progresses,
//! submit for server-side grading, and review results afterwards. This is the backend
//! counterpart to the reference `DrillingZone` + `TryoutPlayer` flow.

use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::application::access_service::AccessService;
use crate::application::elective_service::ElectiveService;
use crate::application::irt_service::IrtService;
use crate::application::platform_settings_service::PlatformSettingsService;
use crate::domain::question::{Difficulty, Question, QuestionType};
use crate::domain::repository::{
    AttemptRepository, AttemptSubmission, NewAttempt, NewSession, QuestionRepository,
    QuestionSetRepository, SessionUpdate, TryoutSessionRepository,
};
use crate::domain::package::ExamTrack;
use crate::domain::platform_settings::ScoreDisplayMode;
use crate::domain::school::SchoolType;
use crate::domain::tryout::{compute_score, Attempt, SessionType, TryoutSession};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct CreateSessionInput {
    pub title: String,
    pub session_type: SessionType,
    pub duration_minutes: i32,
    pub question_count: i32,
    pub subject_filter: Option<String>,
    pub topic_filter: Option<String>,
    pub difficulty_filter: Option<String>,
    pub is_premium: bool,
    /// `Some(set_id)` opts this session into a curated "Set Soal" fixed list instead of the
    /// random-filter draw — see `domain::question_set` doc comment.
    pub question_set_id: Option<Uuid>,
    /// See `domain::tryout::TryoutSession::is_draft` doc comment. The "Buat Paket + Subtes"
    /// wizard always sends `true` here regardless of actor role; the ordinary SessionManager
    /// form always sends `false` (published immediately), preserving pre-existing behavior.
    pub is_draft: bool,
    /// See `domain::tryout::TryoutSession::is_elective` doc comment.
    pub is_elective: bool,
    /// See `domain::package::ExamTrack` doc comment.
    pub exam_track: ExamTrack,
    /// See `domain::tryout::TryoutSession::school_type_scope` doc comment.
    pub school_type_scope: Option<SchoolType>,
}

pub struct UpdateSessionInput {
    pub title: String,
    pub session_type: SessionType,
    pub duration_minutes: i32,
    pub question_count: i32,
    pub subject_filter: Option<String>,
    pub topic_filter: Option<String>,
    pub difficulty_filter: Option<String>,
    pub is_premium: bool,
    pub question_set_id: Option<Uuid>,
    pub is_draft: bool,
    pub is_elective: bool,
    pub exam_track: ExamTrack,
    pub school_type_scope: Option<SchoolType>,
}

pub struct AttemptWithQuestions {
    pub attempt: Attempt,
    /// Questions in assigned order, with `correct_answer`/`explanation` stripped out —
    /// the player must not see the key while the attempt is in progress.
    pub questions: Vec<PlayableQuestion>,
}

/// Full resume payload for a student returning to their own in-progress attempt — typically
/// after a page refresh/close, since the player page only ever gets its state ONCE from
/// `start_attempt` (see `frontend/stores/attemptSession.ts` doc comment: in-memory only, by
/// design, so the answer key can't leak via a refetch). Same key-stripped question view as
/// `start_attempt`, PLUS every answer already saved, so the player can rebuild exactly where
/// the student left off instead of a dead-end "sesi tidak ditemukan".
pub struct AttemptResume {
    pub attempt: Attempt,
    pub questions: Vec<PlayableQuestion>,
    pub answers: Vec<crate::domain::tryout::AttemptAnswer>,
}

pub struct PlayableQuestion {
    pub id: Uuid,
    pub subject: String,
    pub topic: String,
    pub question_type: String,
    pub difficulty: String,
    pub stimulus: Option<String>,
    pub question_text: String,
    pub options: Option<Vec<String>>,
}

/// Sentinel the frontend splits on to recover the two matching sub-lists — see
/// `matching_playable_options` below.
pub const MATCHING_SPLIT_SENTINEL: &str = "__MATCH_SPLIT__";

/// For `matching` questions, the stored `options` are canonical "kiri::kanan" pairs — sending
/// that as-is to the player would hand over the answer key directly. Instead, split into two
/// flat lists (left items in authored order, right items in authored order) joined by a
/// sentinel; the player never receives which left maps to which right, only the two option
/// pools. Display-order shuffling of the right list (if any) is left to the frontend, since it's
/// purely cosmetic and doesn't need to be a trusted server-side decision.
fn matching_playable_options(options: &Option<Vec<String>>) -> Option<Vec<String>> {
    let pairs = options.as_ref()?;
    let mut lefts = Vec::with_capacity(pairs.len());
    let mut rights = Vec::with_capacity(pairs.len());
    for pair in pairs {
        if let Some((l, r)) = pair.split_once("::") {
            lefts.push(l.to_string());
            rights.push(r.to_string());
        }
    }
    // Shuffle the right-hand list — leaving it in authored order would let a student trivially
    // deduce the pairing just from matching row position (lefts[i] with rights[i] always being
    // correct). Sort by a fresh random UUID per element rather than pulling in a `rand`
    // dependency; `uuid`'s v4 generator is already a hard dependency used throughout this file.
    let mut keyed: Vec<(Uuid, String)> = rights.into_iter().map(|r| (Uuid::new_v4(), r)).collect();
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    let rights: Vec<String> = keyed.into_iter().map(|(_, r)| r).collect();

    lefts.push(MATCHING_SPLIT_SENTINEL.to_string());
    lefts.extend(rights);
    Some(lefts)
}

/// For `ordering` questions, `options` is stored in the correct sequence (that IS the answer
/// key — see `QuestionType::Ordering` doc comment). Sending it as-is to the player would hand
/// the answer over directly, so it's fully shuffled here (same random-UUID-sort trick as
/// `matching_playable_options`, for the same "avoid a new `rand` dependency" reason) before
/// being served; the student then reorders this shuffled copy and submits their arrangement,
/// which is graded server-side against the untouched original order in `correct_answer`.
fn ordering_playable_options(options: &Option<Vec<String>>) -> Option<Vec<String>> {
    let opts = options.as_ref()?;
    let mut keyed: Vec<(Uuid, String)> = opts.iter().cloned().map(|o| (Uuid::new_v4(), o)).collect();
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    Some(keyed.into_iter().map(|(_, o)| o).collect())
}

impl From<&Question> for PlayableQuestion {
    fn from(q: &Question) -> Self {
        Self {
            id: q.id,
            subject: q.subject.clone(),
            topic: q.topic.clone(),
            question_type: q.question_type.as_str().to_string(),
            difficulty: q.difficulty.as_str().to_string(),
            stimulus: q.stimulus.clone(),
            question_text: q.question_text.clone(),
            options: if matches!(
                q.question_type,
                crate::domain::question::QuestionType::Matching | crate::domain::question::QuestionType::MatchingImage
            ) {
                matching_playable_options(&q.options)
            } else if matches!(q.question_type, crate::domain::question::QuestionType::Ordering) {
                ordering_playable_options(&q.options)
            } else {
                q.options.clone()
            },
        }
    }
}

pub struct AttemptResult {
    pub attempt: Attempt,
    pub subject_breakdown: Vec<SubjectBreakdown>,
    /// "Estimasi IRT" — see `domain::irt` doc comment. `None` when too few of this attempt's
    /// questions currently have a trusted item calibration (honest "belum tersedia"), which is
    /// the common case until an Admin runs at least one recalibration with enough historical
    /// data. Computed on demand, not persisted — see `IrtService::estimate_for_attempt`.
    pub irt_score: Option<f64>,
    /// Which score(s) the caller should actually display — see
    /// `domain::platform_settings::ScoreDisplayMode` doc comment. Included on every result so
    /// every result screen (student/school) reads the same admin-configured setting instead of
    /// each page fetching platform settings separately.
    pub score_display_mode: ScoreDisplayMode,
}

pub struct SubjectBreakdown {
    pub subject: String,
    pub correct: i32,
    pub total: i32,
}

pub struct ReviewItem {
    pub question: Question,
    pub user_answer: Option<String>,
    pub is_correct: Option<bool>,
    pub flagged: bool,
}

pub struct TryoutService {
    sessions: Arc<dyn TryoutSessionRepository>,
    attempts: Arc<dyn AttemptRepository>,
    questions: Arc<dyn QuestionRepository>,
    question_sets: Arc<dyn QuestionSetRepository>,
    access: Arc<AccessService>,
    electives: Arc<ElectiveService>,
    /// "Estimasi IRT" — see `application::irt_service` doc comment.
    irt: Arc<IrtService>,
    /// Admin-configurable "Skor Instan" vs "Estimasi IRT" display setting — see
    /// `domain::platform_settings::ScoreDisplayMode` doc comment.
    settings: Arc<PlatformSettingsService>,
}

impl TryoutService {
    pub fn new(
        sessions: Arc<dyn TryoutSessionRepository>,
        attempts: Arc<dyn AttemptRepository>,
        questions: Arc<dyn QuestionRepository>,
        question_sets: Arc<dyn QuestionSetRepository>,
        access: Arc<AccessService>,
        electives: Arc<ElectiveService>,
        irt: Arc<IrtService>,
        settings: Arc<PlatformSettingsService>,
    ) -> Self {
        Self { sessions, attempts, questions, question_sets, access, electives, irt, settings }
    }

    /// `Role::Admin`/`Content` get every session including drafts (Content sees drafts they
    /// don't own too — visibility only, edit/delete stays ownership-gated below — so a content
    /// author can see what a colleague is assembling). Everyone else (Student/School) never
    /// sees a draft: `is_draft` exists specifically to hide still-being-assembled subtes from
    /// the public catalogue until the owning Package is published.
    ///
    /// A `Role::Student` additionally never sees:
    ///   1. A session whose `school_type_scope` is explicitly set to a jenjang other than
    ///      their own school's (e.g. an SMP student never sees a session scoped to `sma`) —
    ///      see `AccessService::student_school_type` doc comment.
    ///   2. ANY `exam_track: Snbt` session at all, if their own jenjang is SMP — this is a
    ///      hard categorical rule, independent of `school_type_scope`: SNBT/UTBK is the
    ///      university-entrance exam taken only by graduating SMA/SMK/MA students, never SMP,
    ///      so it's never correct to show it to an SMP student regardless of whether a given
    ///      row happens to have `school_type_scope` set. This also covers every pre-existing
    ///      SNBT session created before `school_type_scope` existed (all `None` by default) —
    ///      without this rule those would keep leaking into the SMP catalog forever unless an
    ///      admin manually re-tagged every single one, which doesn't scale and is easy to miss.
    /// Neither rule applies to a B2C student (no school) — their jenjang can't be determined,
    /// so both filters are skipped entirely for them rather than hiding everything.
    pub async fn list_sessions(&self, actor: &AuthUser, session_type: Option<SessionType>) -> AppResult<Vec<TryoutSession>> {
        let sessions = self.sessions.list(session_type).await?;
        let mut visible: Vec<TryoutSession> = match actor.role {
            Role::Admin | Role::Content => sessions,
            _ => sessions.into_iter().filter(|s| !s.is_draft).collect(),
        };
        if actor.role == Role::Student {
            if let Some(student_type) = self.access.student_school_type(actor.user_id).await? {
                visible.retain(|s| match s.school_type_scope {
                    None => true,
                    Some(t) => t == student_type,
                });
                if student_type == SchoolType::Smp {
                    visible.retain(|s| s.exam_track != ExamTrack::Snbt);
                }
            }
        }
        Ok(visible)
    }

    /// `Role::Content` may author subtes too (the "Buat Paket + Subtes" wizard) — mirrors
    /// `QuestionService::create`'s Admin/Content split.
    pub async fn create_session(&self, actor: &AuthUser, input: CreateSessionInput) -> AppResult<TryoutSession> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        // Only enforced for the random-filter draw path — a session pinned to a curated "Set
        // Soal" (question_set_id) serves that set's fixed list regardless of this number, so
        // it's purely cosmetic there (see the "Buat Paket + Subtes" wizard, which creates the
        // session before any soal exist in the set yet, i.e. legitimately 0 at creation time).
        if input.question_set_id.is_none() && input.question_count <= 0 {
            return Err(AppError::Validation("question_count must be positive".to_string()));
        }
        self.sessions
            .create(NewSession {
                title: input.title,
                session_type: input.session_type,
                duration_minutes: input.duration_minutes,
                question_count: input.question_count,
                subject_filter: input.subject_filter,
                topic_filter: input.topic_filter,
                difficulty_filter: input.difficulty_filter,
                is_premium: input.is_premium,
                created_by: actor.user_id,
                question_set_id: input.question_set_id,
                is_draft: input.is_draft,
                is_elective: input.is_elective,
                exam_track: input.exam_track,
                school_type_scope: input.school_type_scope,
            })
            .await
    }

    pub async fn get_session(&self, session_id: Uuid) -> AppResult<TryoutSession> {
        self.sessions
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("session {session_id} not found")))
    }

    pub async fn update_session(
        &self,
        actor: &AuthUser,
        session_id: Uuid,
        input: UpdateSessionInput,
    ) -> AppResult<TryoutSession> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        if actor.role != Role::Admin {
            let existing = self.get_session(session_id).await?;
            if existing.created_by != actor.user_id {
                return Err(AppError::Forbidden("Anda hanya bisa mengubah sesi yang Anda buat sendiri".to_string()));
            }
        }
        if input.question_set_id.is_none() && input.question_count <= 0 {
            return Err(AppError::Validation("question_count must be positive".to_string()));
        }
        self.sessions
            .update(
                session_id,
                SessionUpdate {
                    title: input.title,
                    session_type: input.session_type,
                    duration_minutes: input.duration_minutes,
                    question_count: input.question_count,
                    subject_filter: input.subject_filter,
                    topic_filter: input.topic_filter,
                    difficulty_filter: input.difficulty_filter,
                    is_premium: input.is_premium,
                    question_set_id: input.question_set_id,
                    is_draft: input.is_draft,
                    is_elective: input.is_elective,
                    exam_track: input.exam_track,
                    school_type_scope: input.school_type_scope,
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("session {session_id} not found")))
    }

    pub async fn delete_session(&self, actor: &AuthUser, session_id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        if actor.role != Role::Admin {
            let existing = self.get_session(session_id).await?;
            if existing.created_by != actor.user_id {
                return Err(AppError::Forbidden("Anda hanya bisa menghapus sesi yang Anda buat sendiri".to_string()));
            }
        }
        let deleted = self.sessions.delete(session_id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("session {session_id} not found")));
        }
        Ok(())
    }

    /// Called directly by `PackageService::set_active`'s publish cascade (crosses the
    /// module boundary intentionally — see that method's doc comment) and by an owner
    /// manually un-hiding/re-hiding their own session from the catalogue.
    pub async fn set_draft(&self, session_id: Uuid, is_draft: bool) -> AppResult<TryoutSession> {
        self.sessions
            .set_draft(session_id, is_draft)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("session {session_id} not found")))
    }

    pub async fn start_attempt(&self, actor: &AuthUser, session_id: Uuid) -> AppResult<AttemptWithQuestions> {
        self.start_attempt_impl(actor, session_id, false).await
    }

    /// Used ONLY by `SimulationService::activate_slot` when starting the underlying
    /// `TryoutSession` for a slot within an already-authorized Simulasi UTBK run — access was
    /// already checked once, at the TEMPLATE level, in `SimulationService::start_run`. Without
    /// this bypass, a slot's session could independently be marked `is_premium` (e.g. it's also
    /// sold standalone) without being in the SAME package as the template, which would wrongly
    /// block a student who legitimately paid for the template.
    pub(crate) async fn start_attempt_for_simulation(&self, actor: &AuthUser, session_id: Uuid) -> AppResult<AttemptWithQuestions> {
        self.start_attempt_impl(actor, session_id, true).await
    }

    async fn start_attempt_impl(&self, actor: &AuthUser, session_id: Uuid, bypass_premium_check: bool) -> AppResult<AttemptWithQuestions> {
        actor.require_role(&[Role::Student])?;

        let session = self
            .sessions
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("session {session_id} not found")))?;

        if !bypass_premium_check
            && session.is_premium
            && !self.access.has_access_to_tryout_session(actor.user_id, session_id).await?
        {
            return Err(AppError::Forbidden(
                "sesi ini khusus siswa dengan akses premium (voucher pribadi atau paket sekolah aktif)".to_string(),
            ));
        }

        // "Mapel pilihan" gating (TKA-style choose-N-of-M) — layered independently on top of
        // the premium check above, so it applies even when `bypass_premium_check` is set (a
        // Simulasi UTBK slot could in principle also be an elective session). See
        // `ElectiveService::assert_can_start` doc comment.
        self.electives.assert_can_start(actor.user_id, &session).await?;

        // Resume instead of duplicating: a student who already has an in-progress attempt for
        // this exact session (closed the tab, refreshed, clicked "Mulai"/"Ulangi" again) gets
        // that SAME attempt handed back rather than silently creating a second one — without
        // this, the first attempt is orphaned forever as an unreachable `in_progress` row (no
        // screen ever showed it again). Mirrors the analogous one-active-run guard in
        // `SimulationService::start_run`; resume-instead-of-reject here since handing back an
        // already-owned attempt is always safe.
        let existing_in_progress = self
            .attempts
            .list_by_student(actor.user_id)
            .await?
            .into_iter()
            .find(|a| a.session_id == session_id && a.status == crate::domain::tryout::AttemptStatus::InProgress);
        if let Some(attempt) = existing_in_progress {
            let questions = self.load_playable_questions(attempt.id).await?;
            return Ok(AttemptWithQuestions { attempt, questions });
        }

        // A session with `question_set_id` set uses that fixed, curated list (in its
        // authored order) instead of drawing randomly — see `domain::question_set` doc
        // comment. Every pre-existing session has `question_set_id = None`, so this `else`
        // branch is the exact same random-filter logic as before this feature existed,
        // completely unchanged.
        let questions = if let Some(set_id) = session.question_set_id {
            self.question_sets
                .approved_items_for_session(set_id, session.session_type)
                .await?
            // NOTE: do NOT sort_by_key(|q| q.id) here — a fixed set's curated order must
            // be preserved, unlike the random path below.
        } else {
            let difficulty = session
                .difficulty_filter
                .as_deref()
                .and_then(|d| d.parse::<Difficulty>().ok());

            let mut random = self
                .questions
                .random_approved(
                    session.subject_filter.as_deref(),
                    session.topic_filter.as_deref(),
                    difficulty,
                    session.question_count as i64,
                    session.session_type,
                )
                .await?;
            random.sort_by_key(|q| q.id); // deterministic-ish ordering after the random SELECT
            random
        };

        if questions.is_empty() {
            return Err(AppError::Validation(format!(
                "belum ada soal berstatus Approved untuk sesi \"{}\" — lengkapi/setujui soalnya dulu di Bank Soal",
                session.title
            )));
        }

        let attempt = self
            .attempts
            .create(NewAttempt {
                session_id,
                student_id: actor.user_id,
                duration_minutes: session.duration_minutes,
                question_ids: questions.iter().map(|q| q.id).collect(),
            })
            .await?;

        let playable = questions.iter().map(PlayableQuestion::from).collect();

        Ok(AttemptWithQuestions { attempt, questions: playable })
    }

    pub async fn save_answer(
        &self,
        actor: &AuthUser,
        attempt_id: Uuid,
        question_id: Uuid,
        answer_text: Option<String>,
        flagged: bool,
    ) -> AppResult<()> {
        let attempt = self.get_owned_attempt(actor, attempt_id).await?;
        if attempt.status != crate::domain::tryout::AttemptStatus::InProgress {
            return Err(AppError::Validation("attempt has already been submitted".to_string()));
        }
        self.attempts
            .upsert_answer(attempt_id, question_id, answer_text, flagged)
            .await?;
        Ok(())
    }

    pub async fn submit_attempt(
        &self,
        actor: &AuthUser,
        attempt_id: Uuid,
        time_used_seconds: i32,
    ) -> AppResult<AttemptResult> {
        let attempt = self.get_owned_attempt(actor, attempt_id).await?;
        if attempt.status != crate::domain::tryout::AttemptStatus::InProgress {
            // Idempotent, NOT an error: a duplicate/retried submit (client retried after a
            // network hiccup, a race between the resume-time auto-submit and a lingering
            // Submit-button click, etc.) previously 422'd here even though the FIRST submit had
            // already succeeded — confusing the student into thinking something failed when
            // their attempt was actually already safely scored. Hand back the same
            // already-computed result instead.
            return self.build_result_for_submitted(attempt).await;
        }

        let question_ids = self.attempts.question_ids_for_attempt(attempt_id).await?;
        let answers = self.attempts.answers_for_attempt(attempt_id).await?;
        let answer_by_question: HashMap<Uuid, &crate::domain::tryout::AttemptAnswer> =
            answers.iter().map(|a| (a.question_id, a)).collect();

        let mut correct = 0usize;
        let mut wrong = 0usize;
        let mut unanswered = 0usize;
        let mut scoreable_total = 0usize;
        let mut breakdown: HashMap<String, (i32, i32)> = HashMap::new();

        for qid in &question_ids {
            let question = self
                .questions
                .find_by_id(*qid)
                .await?
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("question {qid} vanished from bank")))?;

            // Essay (Uraian) is self-check only — never auto-graded and excluded from the
            // score/accuracy denominator entirely (see QuestionType::Essay). The student's
            // text is still saved via save_answer(), just never passed to grade_answer(), so
            // `is_correct` stays NULL and the review screen can show it as "belum dinilai"
            // alongside the sample answer instead of a correct/incorrect verdict.
            if matches!(question.question_type, QuestionType::Essay) {
                continue;
            }

            scoreable_total += 1;
            let entry = breakdown.entry(question.subject.clone()).or_insert((0, 0));
            entry.1 += 1;

            match answer_by_question.get(qid) {
                None => unanswered += 1,
                Some(ans) => match &ans.answer_text {
                    None => unanswered += 1,
                    Some(text) => {
                        let is_correct = normalize(text) == normalize(&question.correct_answer);
                        self.attempts.grade_answer(ans.id, is_correct).await?;
                        if is_correct {
                            correct += 1;
                            entry.0 += 1;
                        } else {
                            wrong += 1;
                        }
                    }
                },
            }
        }

        let score = compute_score(scoreable_total, correct, wrong, unanswered);

        let attempt = self
            .attempts
            .submit(
                attempt_id,
                AttemptSubmission {
                    score: score.score,
                    accuracy: score.accuracy,
                    correct_count: score.correct_count,
                    wrong_count: score.wrong_count,
                    unanswered_count: score.unanswered_count,
                    time_used_seconds,
                },
            )
            .await?;

        // Rebuild via the shared read path (re-reads the answers/is_correct just graded above)
        // so score_display_mode + irt_score are computed exactly once, in exactly one place —
        // same as the idempotent duplicate-submit branch above.
        self.build_result_for_submitted(attempt).await
    }

    /// Public, ownership-checked entry point for "Lihat Hasil" — re-fetches the score +
    /// subject breakdown of one of the caller's OWN already-submitted attempts, any time after
    /// submission (not just right after, unlike `submit_attempt`'s return value, which the
    /// student only ever sees once per attempt in the live player flow). Powers the student's
    /// "Riwayat Latihan" history reopening a past result at `/hasil/:attemptId`.
    pub async fn get_result(&self, actor: &AuthUser, attempt_id: Uuid) -> AppResult<AttemptResult> {
        let attempt = self.get_owned_attempt(actor, attempt_id).await?;
        if attempt.status != crate::domain::tryout::AttemptStatus::Submitted {
            return Err(AppError::Validation("hasil hanya tersedia setelah attempt disubmit".to_string()));
        }
        self.build_result_for_submitted(attempt).await
    }

    /// Same computation as `get_result`, but with NO ownership check on `attempt` — the caller
    /// is responsible for verifying it's allowed to see this attempt before calling this (e.g.
    /// `SchoolRationalizationService::student_attempt_result`, which verifies the attempt
    /// belongs to a student on the caller's own school roster). Kept separate from
    /// `get_result` rather than adding a bypass flag there, so the ownership-checked path stays
    /// impossible to accidentally skip.
    pub async fn result_for_submitted_attempt(&self, attempt_id: Uuid) -> AppResult<AttemptResult> {
        let attempt = self
            .attempts
            .find_by_id(attempt_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("attempt {attempt_id} not found")))?;
        if attempt.status != crate::domain::tryout::AttemptStatus::Submitted {
            return Err(AppError::Validation("hasil hanya tersedia setelah attempt disubmit".to_string()));
        }
        self.build_result_for_submitted(attempt).await
    }

    /// Rebuilds the subject breakdown for an ALREADY-submitted attempt purely by reading its
    /// already-graded answers (`AttemptAnswer::is_correct`, written once during the real
    /// `submit_attempt` above) — never re-grades or re-writes anything. Used by
    /// `submit_attempt`'s idempotent branch so a duplicate submit is a safe no-op read, not a
    /// second grading pass, and by `get_result`/`result_for_submitted_attempt` above.
    async fn build_result_for_submitted(&self, attempt: Attempt) -> AppResult<AttemptResult> {
        let question_ids = self.attempts.question_ids_for_attempt(attempt.id).await?;
        let answers = self.attempts.answers_for_attempt(attempt.id).await?;
        let answer_by_question: HashMap<Uuid, &crate::domain::tryout::AttemptAnswer> =
            answers.iter().map(|a| (a.question_id, a)).collect();

        let mut breakdown: HashMap<String, (i32, i32)> = HashMap::new();
        // (question_id, is_correct) for every scoreable (non-essay) question — feeds
        // `IrtService::estimate_for_attempt`. An unanswered question counts as incorrect here,
        // matching how `compute_score` also gives it zero credit toward the score.
        let mut irt_responses: Vec<(Uuid, bool)> = Vec::new();
        for qid in &question_ids {
            let question = self
                .questions
                .find_by_id(*qid)
                .await?
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("question {qid} vanished from bank")))?;
            if matches!(question.question_type, QuestionType::Essay) {
                continue;
            }
            let is_correct = answer_by_question.get(qid).and_then(|a| a.is_correct).unwrap_or(false);
            let entry = breakdown.entry(question.subject.clone()).or_insert((0, 0));
            entry.1 += 1;
            if is_correct {
                entry.0 += 1;
            }
            irt_responses.push((*qid, is_correct));
        }

        let subject_breakdown = breakdown
            .into_iter()
            .map(|(subject, (correct, total))| SubjectBreakdown { subject, correct, total })
            .collect();

        let irt_score = self.irt.estimate_for_attempt(&irt_responses).await?;
        let score_display_mode = self.settings.score_display_mode().await?;

        Ok(AttemptResult { attempt, subject_breakdown, irt_score, score_display_mode })
    }

    pub async fn get_attempt(&self, actor: &AuthUser, attempt_id: Uuid) -> AppResult<Attempt> {
        self.get_owned_attempt(actor, attempt_id).await
    }

    pub async fn list_my_attempts(&self, actor: &AuthUser) -> AppResult<Vec<Attempt>> {
        actor.require_role(&[Role::Student])?;
        self.attempts.list_by_student(actor.user_id).await
    }

    /// Admin-only: every attempt for `student_id`, including `in_progress` ones — the only way
    /// for the platform team to see "is this student actually stuck mid-tryout right now" when
    /// they report an issue, since attempts are otherwise fully self-scoped
    /// (`list_my_attempts`). Mirrors `SchoolRationalizationService::student_attempt_history`
    /// (School-only, same shape of need) but platform-wide instead of school-scoped.
    pub async fn admin_list_attempts(&self, actor: &AuthUser, student_id: Uuid) -> AppResult<Vec<Attempt>> {
        actor.require_role(&[Role::Admin])?;
        self.attempts.list_by_student(student_id).await
    }

    pub async fn get_review(&self, actor: &AuthUser, attempt_id: Uuid) -> AppResult<Vec<ReviewItem>> {
        let attempt = self.get_owned_attempt(actor, attempt_id).await?;
        if attempt.status != crate::domain::tryout::AttemptStatus::Submitted {
            return Err(AppError::Validation(
                "review is only available after the attempt is submitted".to_string(),
            ));
        }
        self.build_review(attempt_id).await
    }

    /// Same as `get_review`, but with NO ownership check — see
    /// `result_for_submitted_attempt`'s doc comment for why this exists as a separate method
    /// rather than a bypass flag on `get_review`. Used by
    /// `SchoolRationalizationService::student_attempt_review`.
    pub async fn review_for_submitted_attempt(&self, attempt_id: Uuid) -> AppResult<Vec<ReviewItem>> {
        let attempt = self
            .attempts
            .find_by_id(attempt_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("attempt {attempt_id} not found")))?;
        if attempt.status != crate::domain::tryout::AttemptStatus::Submitted {
            return Err(AppError::Validation(
                "review is only available after the attempt is submitted".to_string(),
            ));
        }
        self.build_review(attempt_id).await
    }

    async fn build_review(&self, attempt_id: Uuid) -> AppResult<Vec<ReviewItem>> {
        let question_ids = self.attempts.question_ids_for_attempt(attempt_id).await?;
        let answers = self.attempts.answers_for_attempt(attempt_id).await?;
        let answer_by_question: HashMap<Uuid, &crate::domain::tryout::AttemptAnswer> =
            answers.iter().map(|a| (a.question_id, a)).collect();

        let mut items = Vec::with_capacity(question_ids.len());
        for qid in question_ids {
            let question = self
                .questions
                .find_by_id(qid)
                .await?
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("question {qid} vanished from bank")))?;
            let ans = answer_by_question.get(&qid);
            items.push(ReviewItem {
                question,
                user_answer: ans.and_then(|a| a.answer_text.clone()),
                is_correct: ans.and_then(|a| a.is_correct),
                flagged: ans.map(|a| a.flagged).unwrap_or(false),
            });
        }
        Ok(items)
    }

    async fn load_playable_questions(&self, attempt_id: Uuid) -> AppResult<Vec<PlayableQuestion>> {
        let question_ids = self.attempts.question_ids_for_attempt(attempt_id).await?;
        let mut playable = Vec::with_capacity(question_ids.len());
        for qid in question_ids {
            let question = self
                .questions
                .find_by_id(qid)
                .await?
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("question {qid} vanished from bank")))?;
            playable.push(PlayableQuestion::from(&question));
        }
        Ok(playable)
    }

    /// See `AttemptResume` doc comment. `Admin` may also call this (via `get_owned_attempt`'s
    /// existing Admin bypass) to inspect one specific stuck attempt if they already know its id
    /// — this alone doesn't solve admin *discovery* of in-progress attempts platform-wide, see
    /// `SchoolRationalizationService::student_attempt_history` for the (School-only) listing
    /// equivalent.
    pub async fn resume_attempt(&self, actor: &AuthUser, attempt_id: Uuid) -> AppResult<AttemptResume> {
        let attempt = self.get_owned_attempt(actor, attempt_id).await?;
        if attempt.status != crate::domain::tryout::AttemptStatus::InProgress {
            return Err(AppError::Validation("attempt sudah disubmit".to_string()));
        }
        let questions = self.load_playable_questions(attempt.id).await?;
        let answers = self.attempts.answers_for_attempt(attempt.id).await?;
        Ok(AttemptResume { attempt, questions, answers })
    }

    /// Cross-module collaborator for `SimulationService` (`load_current_questions`, called
    /// from `get_run`'s `reconcile` path and `advance_after_break`) — fetches the key-stripped
    /// question list for the underlying `TryoutSession` attempt of the currently active slot in
    /// a Simulasi UTBK/TKA run. Same ownership+status semantics as `resume_attempt` (must be
    /// `InProgress`, must belong to `actor` or `actor` is Admin), just questions only — a
    /// simulation slot's attempt has no separate "saved answers" UI to restore since the
    /// Simulasi player (`simulasi/[runId].vue`) already re-derives everything from `get_run` on
    /// every mount, unlike the standalone tryout player.
    pub async fn get_attempt_questions(&self, actor: &AuthUser, attempt_id: Uuid) -> AppResult<Vec<PlayableQuestion>> {
        let attempt = self.get_owned_attempt(actor, attempt_id).await?;
        if attempt.status != crate::domain::tryout::AttemptStatus::InProgress {
            return Err(AppError::Validation("attempt sudah disubmit".to_string()));
        }
        self.load_playable_questions(attempt.id).await
    }

    async fn get_owned_attempt(&self, actor: &AuthUser, attempt_id: Uuid) -> AppResult<Attempt> {
        let attempt = self
            .attempts
            .find_by_id(attempt_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("attempt {attempt_id} not found")))?;
        if attempt.student_id != actor.user_id && actor.role != Role::Admin {
            return Err(AppError::Forbidden("this attempt does not belong to you".to_string()));
        }
        Ok(attempt)
    }
}

/// Loose text normalization so "5.85", " 5.85 ", "5,85" style discrepancies don't
/// unfairly fail short-answer grading. Kept intentionally simple for the MVP.
fn normalize(s: &str) -> String {
    s.trim().to_lowercase().replace(',', ".")
}
