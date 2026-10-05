//! Repository traits (ports). The application layer depends only on these traits;
//! `infrastructure::repositories` provides the Postgres implementations. This keeps
//! business logic decoupled from SQLx/Postgres and makes services unit-testable with
//! in-memory fakes if desired.

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::analytics::{
    AnalyticsSummary, QuestionStatusCount, SchoolRanking, ScoreTrendPoint, StudentActivity, StudentRanking,
    SubjectAccuracy, YearlyPerformancePoint,
};
use crate::domain::admission_deadline::{AdmissionDeadline, AdmissionTrack};
use crate::domain::entitlement::SchoolPackageEntitlement;
use crate::domain::event::{Event, EventStatus, EventType};
use crate::domain::gamification::{
    Badge, BadgeConditionType, BadgeWithStats, Challenge, ChallengeStatus, ChallengeType, ChallengeWithStats,
    LeaderboardEntry, LeaderboardRange, PointRule, Rarity,
};
use crate::domain::package::{ExamTrack, Package, PackageContentItem, PackageContentType};
use crate::domain::question::{Difficulty, Question, QuestionStatus, QuestionType};
use crate::domain::question_set::QuestionSet;
use crate::domain::rationalization::{
    Achievement, AchievementLevel, AlumniBenchmark, AlumniRaporScore, AuditLogEntry, Priority, PtnProgram, PtnTarget,
    RaporScore, SchoolEligibility, SnbpParticipation, SnbpStatus, SnbtTracking, SnbtTrackingStatus, Track,
};
use crate::domain::payment::{PaymentStatus, PaymentTransaction};
use crate::domain::platform_settings::{PlatformSettings, ScoreDisplayMode};
use crate::domain::notification::Notification;
use crate::domain::institution::Institution;
use crate::domain::report::{Report, ReportPayload, ReportType};
use crate::domain::school::{PackageType, School, SchoolStatus, SchoolType};
use crate::domain::simulation::{SimulationRun, SimulationRunStatus, SimulationTemplate, TemplateCompletionStats};
use crate::domain::taxonomy::{CategoryWithSubjects, Subject, SubjectCategory};
use crate::domain::tryout::{Attempt, AttemptAnswer, SessionType, TryoutSession};
use crate::domain::user::{Role, User, UserStatus};
use crate::domain::voucher::{DiscountType, Voucher, VoucherType};
use crate::error::AppResult;

// ─── Users ──────────────────────────────────────────────────────────────────────

pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub school_id: Option<Uuid>,
    pub status: UserStatus,
    pub phone: Option<String>,
    pub nisn: Option<String>,
    pub grade: Option<String>,
    /// See `domain::user::User` doc comments for what each of these means — all student-only,
    /// added for the school-mitra bulk import format (20250101000038_student_import_fields.sql).
    pub nis: Option<String>,
    pub gender: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
    pub enrolled_at: Option<chrono::NaiveDate>,
    pub rombel_code: Option<String>,
    pub username: Option<String>,
}

#[derive(Default)]
pub struct UserFilter {
    pub role: Option<Role>,
    pub status: Option<UserStatus>,
    pub search: Option<String>,
    /// Scopes results to one school — used by the School portal so a school PIC only ever
    /// sees their own students, never another school's or unaffiliated accounts.
    pub school_id: Option<Uuid>,
    /// `Some(true)` = only accounts with a `school_id` set (B2B, enrolled through a partner
    /// school). `Some(false)` = only accounts with `school_id: None` (B2C/mandiri). `None` =
    /// no filtering on this dimension. Distinct from `school_id` above, which scopes to one
    /// *specific* school rather than "has any school at all" — used by the admin "Manajemen
    /// User" screen's B2B/B2C segment toggle.
    pub has_school: Option<bool>,
}

pub struct UserUpdate {
    pub name: String,
    pub email: String,
    pub role: Role,
    pub school_id: Option<Uuid>,
    pub status: UserStatus,
    pub phone: Option<String>,
    pub nisn: Option<String>,
    pub grade: Option<String>,
    /// Self-declared "Jurusan yang Diminati" — only ever changed via `AuthService::
    /// update_profile` (the student editing their own account); every other caller of
    /// `UserUpdate` (admin `UserService::update`, password-reset paths) must pass through
    /// the user's existing value untouched.
    pub minat_jurusan: Option<String>,
    /// `Some(hash)` only when the caller also wants to change the password.
    pub password_hash: Option<String>,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, new_user: NewUser) -> AppResult<User>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn list(&self, filter: UserFilter) -> AppResult<Vec<User>>;
    async fn update(&self, id: Uuid, update: UserUpdate) -> AppResult<Option<User>>;
    /// `Ok(false)` if the row didn't exist; a foreign-key conflict (user still owns
    /// questions/sessions) surfaces as `AppError::Conflict`, not a generic 500.
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
    async fn set_status(&self, id: Uuid, status: UserStatus) -> AppResult<Option<User>>;
    /// Called by `AuthService::login` on every successful login — real activity, never faked.
    async fn touch_last_login(&self, id: Uuid) -> AppResult<()>;
    /// Sets (or clears, with `None`) the user's profile photo URL. Kept separate from the
    /// full `update()` (which requires every field) since this is a single, frequent,
    /// self-service action.
    async fn update_avatar(&self, id: Uuid, avatar_url: Option<String>) -> AppResult<Option<User>>;
}

// ─── Schools ────────────────────────────────────────────────────────────────────

pub struct NewSchool {
    pub name: String,
    pub school_type: SchoolType,
    pub city: String,
    pub province: String,
    pub email: String,
    pub phone: String,
    pub package_type: PackageType,
    pub contact_person: String,
    pub status: SchoolStatus,
    pub revenue_share: i32,
    pub monthly_revenue: i64,
}

pub struct SchoolUpdate {
    pub name: String,
    pub school_type: SchoolType,
    pub city: String,
    pub province: String,
    pub email: String,
    pub phone: String,
    pub package_type: PackageType,
    pub contact_person: String,
    pub status: SchoolStatus,
    pub revenue_share: i32,
    pub monthly_revenue: i64,
}

#[async_trait]
pub trait SchoolRepository: Send + Sync {
    async fn create(&self, new_school: NewSchool) -> AppResult<School>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<School>>;
    async fn list(&self) -> AppResult<Vec<School>>;
    async fn update(&self, id: Uuid, update: SchoolUpdate) -> AppResult<Option<School>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
    /// Cheap targeted status flip for the approve/deactivate/reactivate quick actions.
    async fn set_status(&self, id: Uuid, status: SchoolStatus) -> AppResult<Option<School>>;
}

// ─── Analytics (read-only aggregations — no CRUD) ──────────────────────────────

#[async_trait]
pub trait AnalyticsRepository: Send + Sync {
    async fn summary(&self) -> AppResult<AnalyticsSummary>;
    async fn score_trend(&self, days: i32) -> AppResult<Vec<ScoreTrendPoint>>;
    async fn subject_breakdown(&self) -> AppResult<Vec<SubjectAccuracy>>;
    async fn question_status_distribution(&self) -> AppResult<Vec<QuestionStatusCount>>;
    async fn top_schools(&self, limit: i64) -> AppResult<Vec<SchoolRanking>>;
    /// Platform-wide "Perbandingan Tahun" — attempts bucketed by calendar year of
    /// `submitted_at`. See `domain::analytics::YearlyPerformancePoint` doc comment.
    async fn score_trend_by_year(&self) -> AppResult<Vec<YearlyPerformancePoint>>;
    /// School-portal-scoped variants — same aggregation logic as the admin-wide versions,
    /// but joined through `users.school_id` so a school PIC only ever sees their own
    /// students' real data.
    async fn subject_breakdown_for_school(&self, school_id: Uuid) -> AppResult<Vec<SubjectAccuracy>>;
    async fn score_trend_for_school(&self, school_id: Uuid, days: i32) -> AppResult<Vec<ScoreTrendPoint>>;
    async fn score_trend_by_year_for_school(&self, school_id: Uuid) -> AppResult<Vec<YearlyPerformancePoint>>;
    async fn student_ranking_for_school(&self, school_id: Uuid) -> AppResult<Vec<StudentRanking>>;
    /// School portal's "Analytics & Insights" drill-down — see `domain::analytics::StudentActivity`
    /// doc comment for why this is a separate query from `student_ranking_for_school` (LEFT JOIN,
    /// includes students with zero submitted attempts, plus `rombel_code`/`last_attempt_at`).
    async fn student_activity_for_school(&self, school_id: Uuid) -> AppResult<Vec<StudentActivity>>;
    /// Student portal's own "Overview" mastery breakdown — same accuracy aggregation,
    /// scoped to just this one student's real answers.
    async fn subject_breakdown_for_student(&self, student_id: Uuid) -> AppResult<Vec<SubjectAccuracy>>;
}

// ─── Questions ──────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct QuestionFilter {
    pub search: Option<String>,
    pub subject: Option<String>,
    pub status: Option<QuestionStatus>,
    pub difficulty: Option<Difficulty>,
    pub question_type: Option<QuestionType>,
    pub created_by: Option<Uuid>,
    pub page: i64,
    pub page_size: i64,
}

pub struct NewQuestion {
    pub code: String,
    pub question_type: QuestionType,
    pub subject: String,
    pub topic: String,
    pub subtopic: String,
    pub difficulty: Difficulty,
    pub bloom_level: String,
    pub stimulus: Option<String>,
    pub question_text: String,
    pub options: Option<Vec<String>>,
    pub correct_answer: String,
    pub explanation: String,
    pub tags: Vec<String>,
    pub status: QuestionStatus,
    pub created_by: Uuid,
    pub time_limit: Option<i32>,
}

pub struct QuestionUpdate {
    pub question_type: QuestionType,
    pub subject: String,
    pub topic: String,
    pub subtopic: String,
    pub difficulty: Difficulty,
    pub bloom_level: String,
    pub stimulus: Option<String>,
    pub question_text: String,
    pub options: Option<Vec<String>>,
    pub correct_answer: String,
    pub explanation: String,
    pub tags: Vec<String>,
    pub time_limit: Option<i32>,
}

#[async_trait]
pub trait QuestionRepository: Send + Sync {
    async fn create(&self, new_question: NewQuestion) -> AppResult<Question>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Question>>;
    async fn list(&self, filter: QuestionFilter) -> AppResult<(Vec<Question>, i64)>;
    async fn update(&self, id: Uuid, update: QuestionUpdate) -> AppResult<Option<Question>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
    async fn set_review_status(
        &self,
        id: Uuid,
        status: QuestionStatus,
        review_note: Option<String>,
        reviewed_by: Uuid,
    ) -> AppResult<Option<Question>>;
    /// Move a question into the review queue and clear any previous review verdict
    /// (note/reviewed_by/reviewed_at) — used for both first-time "Kirim Review" (from draft)
    /// and "Ajukan Ulang" (from rejected/revision, after addressing feedback).
    async fn resubmit(&self, id: Uuid) -> AppResult<Option<Question>>;
    /// Random sample of approved questions matching optional subject/topic/difficulty
    /// filters — used to build a tryout/drilling attempt. `session_type` gates the `essay`
    /// question type: essay is self-check/Drilling-only (never auto-graded, see
    /// `QuestionType::Essay`), so it must never be selected into a Tryout/Mini attempt.
    async fn random_approved(
        &self,
        subject: Option<&str>,
        topic: Option<&str>,
        difficulty: Option<Difficulty>,
        count: i64,
        session_type: SessionType,
    ) -> AppResult<Vec<Question>>;
}

// ─── Tryout sessions & attempts ────────────────────────────────────────────────

pub struct NewSession {
    pub title: String,
    pub session_type: SessionType,
    pub duration_minutes: i32,
    pub question_count: i32,
    pub subject_filter: Option<String>,
    pub topic_filter: Option<String>,
    pub difficulty_filter: Option<String>,
    pub is_premium: bool,
    pub created_by: Uuid,
    /// `Some(set_id)` to use a curated "Set Soal" fixed list instead of the random-filter
    /// draw — see `domain::question_set` doc comment.
    pub question_set_id: Option<Uuid>,
    /// See `domain::tryout::TryoutSession::is_draft` doc comment.
    pub is_draft: bool,
    /// See `domain::tryout::TryoutSession::is_elective` doc comment.
    pub is_elective: bool,
    /// See `domain::package::Package::exam_track` doc comment.
    pub exam_track: ExamTrack,
    /// See `domain::tryout::TryoutSession::school_type_scope` doc comment.
    pub school_type_scope: Option<SchoolType>,
}

pub struct SessionUpdate {
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

#[async_trait]
pub trait TryoutSessionRepository: Send + Sync {
    async fn create(&self, new_session: NewSession) -> AppResult<TryoutSession>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<TryoutSession>>;
    async fn list(&self, session_type: Option<SessionType>) -> AppResult<Vec<TryoutSession>>;
    async fn update(&self, id: Uuid, update: SessionUpdate) -> AppResult<Option<TryoutSession>>;
    /// `Ok(false)` if the row didn't exist; fails with `AppError::Conflict` if attempts
    /// already reference this session (they must be preserved for audit/history).
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
    /// Cheap targeted toggle — used by `PackageService::set_active`'s publish cascade and by
    /// an owner (Admin, or Content for their own session) manually flipping draft status.
    async fn set_draft(&self, id: Uuid, is_draft: bool) -> AppResult<Option<TryoutSession>>;
}

// ─── Question Sets ("Set Soal") ─────────────────────────────────────────────────

#[async_trait]
pub trait QuestionSetRepository: Send + Sync {
    async fn create(&self, name: String, description: String, created_by: Uuid) -> AppResult<QuestionSet>;
    /// Includes `item_count` (live `COUNT` join), for list views.
    async fn list(&self) -> AppResult<Vec<QuestionSet>>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<QuestionSet>>;
    async fn update(&self, id: Uuid, name: String, description: String) -> AppResult<Option<QuestionSet>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
    /// Full Question rows (not just ids), in `sort_order` order.
    async fn list_items(&self, set_id: Uuid) -> AppResult<Vec<Question>>;
    /// Upsert-style: adding a question already in the set is a no-op success (idempotent),
    /// not an error — this matters because the question authoring form calls this on every
    /// save of a question that has a set assigned, including re-saves of an
    /// already-assigned question.
    async fn add_item(&self, set_id: Uuid, question_id: Uuid, sort_order: i32) -> AppResult<()>;
    async fn remove_item(&self, set_id: Uuid, question_id: Uuid) -> AppResult<bool>;
    /// Same governance/type filtering `random_approved` applies (status='approved', and
    /// `essay` type excluded unless `session_type` is Drilling) but over this specific set's
    /// members instead of the whole bank, preserving `sort_order` (does NOT re-sort by id
    /// like the random path does — a curated set's order is intentional).
    async fn approved_items_for_session(&self, set_id: Uuid, session_type: SessionType) -> AppResult<Vec<Question>>;
}

// ─── Events (Manajemen Event) ──────────────────────────────────────────────────

pub struct NewEvent {
    pub name: String,
    pub event_type: EventType,
    pub description: String,
    pub session_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub max_participants: Option<i32>,
    pub price: i32,
    pub prizes: String,
    pub target_class: String,
    pub created_by: Uuid,
}

pub struct EventUpdate {
    pub name: String,
    pub event_type: EventType,
    pub description: String,
    pub session_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub max_participants: Option<i32>,
    pub price: i32,
    pub status: EventStatus,
    pub prizes: String,
    pub target_class: String,
}

#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn create(&self, new_event: NewEvent) -> AppResult<Event>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Event>>;
    async fn list(&self) -> AppResult<Vec<Event>>;
    async fn update(&self, id: Uuid, update: EventUpdate) -> AppResult<Option<Event>>;
    /// Cheap targeted update for the quick lifecycle actions (Publish/Mulai/Selesaikan/
    /// Arsipkan) so callers don't need to resend the full form payload just to flip status.
    async fn set_status(&self, id: Uuid, status: EventStatus) -> AppResult<Option<Event>>;
    /// `Ok(false)` if the row didn't exist. Events never block on FK conflicts from
    /// `attempts` — those belong to the underlying session, not the event row itself.
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

// ─── Admission Deadlines (Kalender Deadline SNBP/SNBT/UTBK) ────────────────────

pub struct NewAdmissionDeadline {
    pub track: AdmissionTrack,
    pub year: i32,
    pub label: String,
    pub description: String,
    pub deadline_date: NaiveDate,
    pub created_by: Uuid,
}

pub struct AdmissionDeadlineUpdate {
    pub track: AdmissionTrack,
    pub year: i32,
    pub label: String,
    pub description: String,
    pub deadline_date: NaiveDate,
}

#[async_trait]
pub trait AdmissionDeadlineRepository: Send + Sync {
    async fn create(&self, new_deadline: NewAdmissionDeadline) -> AppResult<AdmissionDeadline>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<AdmissionDeadline>>;
    /// Full admin CRUD list, newest deadline first.
    async fn list(&self) -> AppResult<Vec<AdmissionDeadline>>;
    async fn update(&self, id: Uuid, update: AdmissionDeadlineUpdate) -> AppResult<Option<AdmissionDeadline>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

pub struct NewAttempt {
    pub session_id: Uuid,
    pub student_id: Uuid,
    pub duration_minutes: i32,
    pub question_ids: Vec<Uuid>,
}

pub struct AttemptSubmission {
    pub score: i32,
    pub accuracy: f64,
    pub correct_count: i32,
    pub wrong_count: i32,
    pub unanswered_count: i32,
    pub time_used_seconds: i32,
}

/// Aggregate correct/total counts for one question, across every graded answer ever recorded
/// platform-wide — the raw input `IrtService::recalibrate` feeds into
/// `domain::irt::calibrate_difficulty`. See `AttemptRepository::item_response_stats`.
pub struct ItemResponseStat {
    pub question_id: Uuid,
    pub correct: i64,
    pub total: i64,
}

#[async_trait]
pub trait AttemptRepository: Send + Sync {
    async fn create(&self, new_attempt: NewAttempt) -> AppResult<Attempt>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Attempt>>;
    async fn list_by_student(&self, student_id: Uuid) -> AppResult<Vec<Attempt>>;
    /// Ordered list of question ids assigned to this attempt.
    async fn question_ids_for_attempt(&self, attempt_id: Uuid) -> AppResult<Vec<Uuid>>;
    async fn upsert_answer(
        &self,
        attempt_id: Uuid,
        question_id: Uuid,
        answer_text: Option<String>,
        flagged: bool,
    ) -> AppResult<AttemptAnswer>;
    async fn answers_for_attempt(&self, attempt_id: Uuid) -> AppResult<Vec<AttemptAnswer>>;
    async fn submit(&self, attempt_id: Uuid, submission: AttemptSubmission) -> AppResult<Attempt>;
    /// Persist correctness for each answer once graded (called during submit).
    async fn grade_answer(&self, answer_id: Uuid, is_correct: bool) -> AppResult<()>;
    /// Platform-wide correct/total counts per question, from every graded `attempt_answers` row
    /// (`is_correct IS NOT NULL`) ever recorded — feeds `IrtService::recalibrate`'s item
    /// difficulty calibration. A question with zero graded answers simply doesn't appear in the
    /// result (honest-zero: no fabricated `ItemResponseStat { total: 0, .. }` rows).
    async fn item_response_stats(&self) -> AppResult<Vec<ItemResponseStat>>;
}

// ─── Packages (Manajemen Paket) ────────────────────────────────────────────────

pub struct NewPackage {
    pub name: String,
    pub package_type: String,
    pub original_price: i32,
    pub sale_price: i32,
    pub validity: String,
    pub features: Vec<String>,
    pub badge: String,
    pub emoji: String,
    pub icon_type: String,
    pub icon_name: Option<String>,
    pub icon_url: Option<String>,
    /// See `domain::package::Package::banner_url` doc comment.
    pub banner_url: Option<String>,
    pub gradient: String,
    pub accent_color: String,
    pub active: bool,
    pub sort_order: i32,
    pub created_by: Uuid,
    /// See `domain::package::Package::elective_pick_count` doc comment.
    pub elective_pick_count: i32,
    /// See `domain::package::ExamTrack` doc comment.
    pub exam_track: ExamTrack,
}

pub struct PackageUpdate {
    pub name: String,
    pub package_type: String,
    pub original_price: i32,
    pub sale_price: i32,
    pub validity: String,
    pub features: Vec<String>,
    pub badge: String,
    pub emoji: String,
    pub icon_type: String,
    pub icon_name: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub gradient: String,
    pub accent_color: String,
    pub active: bool,
    pub sort_order: i32,
    pub elective_pick_count: i32,
    pub exam_track: ExamTrack,
}

#[async_trait]
pub trait PackageRepository: Send + Sync {
    async fn create(&self, new_package: NewPackage) -> AppResult<Package>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Package>>;
    async fn list(&self) -> AppResult<Vec<Package>>;
    async fn update(&self, id: Uuid, update: PackageUpdate) -> AppResult<Option<Package>>;
    /// Cheap targeted toggle for the show/hide-on-landing-page quick action.
    async fn set_active(&self, id: Uuid, active: bool) -> AppResult<Option<Package>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

// ─── Package content scoping (which TryoutSessions/SimulationTemplates a Package unlocks) ──

#[async_trait]
pub trait PackageContentRepository: Send + Sync {
    /// Denormalizes `content_title` at read time via a join keyed on `content_type`.
    async fn list_for_package(&self, package_id: Uuid) -> AppResult<Vec<PackageContentItem>>;
    /// `Ok(Err(...))`-free — caller (`PackageService`) is responsible for validating that
    /// `content_id` actually exists in the right table before calling this; the repository
    /// trusts it. Idempotent: re-adding an already-assigned item is a no-op (unique constraint).
    async fn add_item(&self, package_id: Uuid, content_type: PackageContentType, content_id: Uuid) -> AppResult<PackageContentItem>;
    /// `Ok(false)` if the row didn't exist.
    async fn remove_item(&self, id: Uuid) -> AppResult<bool>;
    /// Every `package_id` that includes this exact `(content_type, content_id)` — the core
    /// lookup `AccessService` uses to answer "which packages would unlock this content".
    async fn package_ids_for_content(&self, content_type: PackageContentType, content_id: Uuid) -> AppResult<Vec<Uuid>>;
    /// Every `(content_type, content_id)` pair unlocked by ANY of the given `package_ids` —
    /// used by `AccessService::list_my_unlocked_content` to answer "what can this student play"
    /// in one query instead of one round-trip per content item.
    async fn content_ids_for_packages(&self, package_ids: &[Uuid]) -> AppResult<Vec<(PackageContentType, Uuid)>>;
}

// ─── Package elective choices ("mapel pilihan" TKA-style subject selection) ───────

#[async_trait]
pub trait PackageElectiveRepository: Send + Sync {
    /// This student's CURRENT elective picks within this package — always reflects the latest
    /// `set_choices` call, not a history (see `ElectiveService` doc comment).
    async fn list_choices(&self, student_id: Uuid, package_id: Uuid) -> AppResult<Vec<Uuid>>;
    /// Full replace (delete-then-insert, in one transaction): `session_ids` becomes the
    /// student's entire pick set for this package, regardless of what was picked before.
    /// Passing an empty `Vec` clears all picks. Caller (`ElectiveService`) is responsible for
    /// validating count/membership before calling this; the repository trusts it.
    async fn set_choices(&self, student_id: Uuid, package_id: Uuid, session_ids: Vec<Uuid>) -> AppResult<()>;
}

// ─── Vouchers (Manajemen Voucher) ──────────────────────────────────────────────

pub struct NewVoucher {
    pub code: String,
    pub voucher_type: VoucherType,
    pub package_id: Uuid,
    pub discount_type: DiscountType,
    pub discount_value: i32,
    pub max_uses: i32,
    pub expires_at: NaiveDate,
    pub note: String,
    pub school_name: Option<String>,
    pub referrer_name: Option<String>,
    pub referrer_commission: Option<i32>,
    pub created_by: Uuid,
}

#[async_trait]
pub trait VoucherRepository: Send + Sync {
    async fn create(&self, new_voucher: NewVoucher) -> AppResult<Voucher>;
    async fn create_batch(&self, new_vouchers: Vec<NewVoucher>) -> AppResult<Vec<Voucher>>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Voucher>>;
    async fn find_by_code(&self, code: &str) -> AppResult<Option<Voucher>>;
    async fn list(&self) -> AppResult<Vec<Voucher>>;
    /// Cheap targeted toggle for the active/inactive quick action.
    async fn set_active(&self, id: Uuid, active: bool) -> AppResult<Option<Voucher>>;
    /// `Ok(false)` if the row didn't exist. Callers (service layer) are responsible for
    /// refusing to delete a voucher that has already been used — enforced above the
    /// repository so it stays a pure business rule, not a DB-level constraint.
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
    /// Atomically records a `(voucher_id, student_id)` redemption row and increments
    /// `used_count` in one transaction. Returns `AppError::Conflict` if this student already
    /// redeemed this exact voucher (unique constraint on `voucher_redemptions`).
    async fn redeem(&self, voucher_id: Uuid, student_id: Uuid) -> AppResult<Voucher>;
    /// True if this student has EVER redeemed any voucher for any package — used as the
    /// individual-access half of `AccessService::has_premium_access`'s OR check. Does not
    /// distinguish which package; today every Package functionally means the same thing
    /// (premium tryout access), since there is no per-package content-scoping data yet.
    async fn student_has_redemption(&self, student_id: Uuid) -> AppResult<bool>;
    /// Distinct `package_id`s this student has ever redeemed a voucher for — the personal-
    /// ownership half of the per-package access check (see `AccessService::owned_package_ids`).
    /// Supersedes `student_has_redemption` for real gating now that access is package-scoped
    /// rather than all-or-nothing; that method is kept for any other boolean-only callers.
    async fn student_redeemed_package_ids(&self, student_id: Uuid) -> AppResult<Vec<Uuid>>;
}

// ─── School Package Entitlements ───────────────────────────────────────────────

pub struct NewSchoolEntitlement {
    pub school_id: Uuid,
    pub package_id: Uuid,
    pub starts_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
    pub note: Option<String>,
    pub granted_by: Uuid,
}

#[async_trait]
pub trait SchoolEntitlementRepository: Send + Sync {
    async fn create(&self, input: NewSchoolEntitlement) -> AppResult<SchoolPackageEntitlement>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SchoolPackageEntitlement>>;
    async fn list_by_school(&self, school_id: Uuid) -> AppResult<Vec<SchoolPackageEntitlement>>;
    /// `Ok(false)` if the row didn't exist.
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
    /// True if `school_id` has ANY entitlement row covering `today` (starts_at <= today <=
    /// expires_at, or expires_at IS NULL meaning no end date).
    async fn has_active_entitlement(&self, school_id: Uuid, today: NaiveDate) -> AppResult<bool>;
    /// Distinct `package_id`s covered by an entitlement active on `today` for this school — the
    /// school-wide half of the per-package access check (see `AccessService::owned_package_ids`).
    async fn active_entitlement_package_ids(&self, school_id: Uuid, today: NaiveDate) -> AppResult<Vec<Uuid>>;
}

// ─── Gamification (Manajemen Gamifikasi) ───────────────────────────────────────

pub struct NewPointRule {
    pub action: String,
    pub category: String,
    pub base_points: i32,
    pub multiplier: f64,
    pub enabled: bool,
    pub icon: String,
    pub sort_order: i32,
}

pub struct PointRuleUpdate {
    pub base_points: i32,
    pub multiplier: f64,
    pub enabled: bool,
}

#[async_trait]
pub trait PointRuleRepository: Send + Sync {
    async fn create(&self, new_rule: NewPointRule) -> AppResult<PointRule>;
    async fn list(&self) -> AppResult<Vec<PointRule>>;
    async fn update(&self, id: Uuid, update: PointRuleUpdate) -> AppResult<Option<PointRule>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

pub struct NewBadge {
    pub emoji: String,
    pub icon_type: String,
    pub icon_name: Option<String>,
    pub icon_url: Option<String>,
    pub name: String,
    pub description: String,
    pub rarity: Rarity,
    pub condition_type: BadgeConditionType,
    pub condition_value: i32,
    pub active: bool,
    pub created_by: Uuid,
}

pub struct BadgeUpdate {
    pub emoji: String,
    pub icon_type: String,
    pub icon_name: Option<String>,
    pub icon_url: Option<String>,
    pub name: String,
    pub description: String,
    pub rarity: Rarity,
    pub condition_type: BadgeConditionType,
    pub condition_value: i32,
    pub active: bool,
}

#[async_trait]
pub trait BadgeRepository: Send + Sync {
    async fn create(&self, new_badge: NewBadge) -> AppResult<Badge>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Badge>>;
    async fn list(&self) -> AppResult<Vec<Badge>>;
    async fn update(&self, id: Uuid, update: BadgeUpdate) -> AppResult<Option<Badge>>;
    async fn set_active(&self, id: Uuid, active: bool) -> AppResult<Option<Badge>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
    /// Live count of distinct students who currently satisfy this badge's condition,
    /// computed against real `attempts`/`attempt_answers` rows — never stored.
    async fn earned_count(&self, condition_type: BadgeConditionType, condition_value: i32) -> AppResult<i64>;
    async fn list_with_stats(&self) -> AppResult<Vec<BadgeWithStats>>;
    /// One student's real progress toward every active badge — backs the student
    /// portal's "Achievements" card (see `StudentBadgeStatus`).
    async fn list_for_student(&self, student_id: Uuid) -> AppResult<Vec<crate::domain::gamification::StudentBadgeStatus>>;
}

pub struct NewChallenge {
    pub name: String,
    pub challenge_type: ChallengeType,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub target_value: i32,
    pub reward_points: i32,
    pub reward_badge: Option<String>,
    pub description: String,
    pub created_by: Uuid,
}

pub struct ChallengeUpdate {
    pub name: String,
    pub challenge_type: ChallengeType,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub target_value: i32,
    pub reward_points: i32,
    pub reward_badge: Option<String>,
    pub description: String,
}

#[async_trait]
pub trait ChallengeRepository: Send + Sync {
    async fn create(&self, new_challenge: NewChallenge) -> AppResult<Challenge>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Challenge>>;
    async fn list(&self) -> AppResult<Vec<Challenge>>;
    async fn update(&self, id: Uuid, update: ChallengeUpdate) -> AppResult<Option<Challenge>>;
    async fn set_status(&self, id: Uuid, status: ChallengeStatus) -> AppResult<Option<Challenge>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
    /// Live participants/completions for this challenge's date range + type, computed
    /// against real `attempts` rows — never stored.
    async fn stats_for(&self, challenge: &Challenge) -> AppResult<(i64, i64)>;
    async fn list_with_stats(&self) -> AppResult<Vec<ChallengeWithStats>>;
}

/// Optional scoping for a leaderboard read — every field is applied live in SQL, never
/// pre-filtered/cached, so a school/time-range view can never drift from the national one.
#[derive(Debug, Clone)]
pub struct LeaderboardParams {
    pub limit: i64,
    /// `Some(id)` scopes to one school's students only (used by the "Sekolahku" view).
    pub school_id: Option<Uuid>,
    pub range: LeaderboardRange,
}

#[async_trait]
pub trait LeaderboardRepository: Send + Sync {
    /// Top students ranked by their best-ever (or best-in-range) single-attempt score,
    /// joined with real school data and a real per-student streak — always computed live
    /// from `attempts`, never a stored/fake ranking.
    async fn top_students(&self, params: LeaderboardParams) -> AppResult<Vec<LeaderboardEntry>>;

    /// Same shape, but ranked by real answer accuracy (%) within one subject instead of
    /// overall score — `best_score` on the returned entries carries the accuracy percentage.
    async fn top_students_by_subject(&self, params: LeaderboardParams, subject: &str) -> AppResult<Vec<LeaderboardEntry>>;
}

// ─── Rasionalisasi SNBT/SNBP ────────────────────────────────────────────────────

#[derive(Default)]
pub struct PtnProgramFilter {
    pub search: Option<String>,
    pub rumpun: Option<String>,
    pub jenjang: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

pub struct NewPtnProgram {
    pub kode: i64,
    pub nama_ptn: String,
    pub nama_prodi: String,
    pub provinsi: String,
    pub kota: String,
    pub singkatan: String,
    pub rumpun: String,
    pub mapel_syarat: String,
    pub daya_tampung_snbp: i32,
    pub peminat_snbp: i32,
    pub daya_tampung_snbt: i32,
    pub peminat_snbt: i32,
    pub pg_snbt: f64,
    pub pg_snbp: f64,
    pub jenjang: String,
    pub has_official_stats: bool,
}

pub struct PtnProgramUpdate {
    pub nama_ptn: String,
    pub nama_prodi: String,
    pub provinsi: String,
    pub kota: String,
    pub singkatan: String,
    pub rumpun: String,
    pub mapel_syarat: String,
    pub daya_tampung_snbp: i32,
    pub peminat_snbp: i32,
    pub daya_tampung_snbt: i32,
    pub peminat_snbt: i32,
    pub pg_snbt: f64,
    pub pg_snbp: f64,
    pub jenjang: String,
    pub has_official_stats: bool,
}

#[async_trait]
pub trait PtnProgramRepository: Send + Sync {
    /// Insert-or-update keyed on the source catalog's `kode` — used by the one-time CSV
    /// importer so re-running it is idempotent rather than duplicating rows.
    async fn upsert_by_kode(&self, new_program: NewPtnProgram) -> AppResult<PtnProgram>;
    async fn create(&self, new_program: NewPtnProgram) -> AppResult<PtnProgram>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<PtnProgram>>;
    async fn list(&self, filter: PtnProgramFilter) -> AppResult<(Vec<PtnProgram>, i64)>;
    async fn update(&self, id: Uuid, update: PtnProgramUpdate) -> AppResult<Option<PtnProgram>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
    async fn count_all(&self) -> AppResult<i64>;
    async fn distinct_rumpun(&self) -> AppResult<Vec<String>>;
}

pub struct RaporScoreUpsert {
    pub semester: i32,
    pub subject: String,
    pub score: f64,
    pub is_minat: bool,
}

#[async_trait]
pub trait RaporScoreRepository: Send + Sync {
    /// Upserts on `(student_id, semester, subject)` — students edit their own rapor entry
    /// by re-submitting the same semester+subject, never accumulating duplicate rows.
    async fn upsert(&self, student_id: Uuid, entry: RaporScoreUpsert) -> AppResult<RaporScore>;
    async fn list_by_student(&self, student_id: Uuid) -> AppResult<Vec<RaporScore>>;
    async fn delete(&self, student_id: Uuid, id: Uuid) -> AppResult<bool>;
}

pub struct NewPtnTarget {
    pub student_id: Uuid,
    pub ptn_program_id: Uuid,
    pub track: Track,
    pub priority: Priority,
    pub sort_order: i32,
}

#[async_trait]
pub trait PtnTargetRepository: Send + Sync {
    async fn create(&self, new_target: NewPtnTarget) -> AppResult<PtnTarget>;
    async fn list_by_student(&self, student_id: Uuid) -> AppResult<Vec<PtnTarget>>;
    async fn set_priority(&self, student_id: Uuid, id: Uuid, priority: Priority) -> AppResult<Option<PtnTarget>>;
    async fn delete(&self, student_id: Uuid, id: Uuid) -> AppResult<bool>;
}

// ─── School Rasionalisasi sub-tabs: Alumni benchmark + SNBP eligibility ─────────

pub struct NewAlumniBenchmark {
    pub school_id: Uuid,
    pub alumni_name: String,
    pub graduation_year: i32,
    pub track: Track,
    pub nama_ptn: String,
    pub nama_prodi: String,
    pub benchmark_score: f64,
    pub created_by: Uuid,
}

pub struct AlumniRaporScoreUpsert {
    pub subject: String,
    pub score: f64,
}

#[async_trait]
pub trait AlumniBenchmarkRepository: Send + Sync {
    async fn create(&self, new_benchmark: NewAlumniBenchmark) -> AppResult<AlumniBenchmark>;
    async fn list_by_school(&self, school_id: Uuid) -> AppResult<Vec<AlumniBenchmark>>;
    /// Dipakai untuk verifikasi kepemilikan (`alumni.school_id == actor's school_id`)
    /// sebelum membaca/menulis "Detail Rapor" milik alumni ini.
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<AlumniBenchmark>>;
    async fn delete(&self, school_id: Uuid, id: Uuid) -> AppResult<bool>;
    /// "Detail Rapor" opsional per alumni — upsert on `(alumni_id, subject)`, sama pola
    /// dengan `RaporScoreRepository::upsert` milik siswa aktif tapi tanpa dimensi semester.
    async fn upsert_rapor(&self, alumni_id: Uuid, entry: AlumniRaporScoreUpsert) -> AppResult<AlumniRaporScore>;
    async fn list_rapor(&self, alumni_id: Uuid) -> AppResult<Vec<AlumniRaporScore>>;
    async fn delete_rapor(&self, alumni_id: Uuid, id: Uuid) -> AppResult<bool>;
}

pub struct SchoolEligibilityUpsert {
    pub year: i32,
    pub eligible_count: i32,
}

#[async_trait]
pub trait SchoolEligibilityRepository: Send + Sync {
    /// Upserts on `(school_id, year)` — the School PIC edits their own year's figure by
    /// re-submitting the same year, never accumulating duplicate rows.
    async fn upsert(&self, school_id: Uuid, entry: SchoolEligibilityUpsert) -> AppResult<SchoolEligibility>;
    async fn list_by_school(&self, school_id: Uuid) -> AppResult<Vec<SchoolEligibility>>;
}

// ─── Student achievements ("Prestasi") ───────────────────────────────────────────

pub struct NewAchievement {
    pub student_id: Uuid,
    pub nama: String,
    pub tingkat: AchievementLevel,
    pub tahun: i32,
    pub juara: String,
}

#[async_trait]
pub trait AchievementRepository: Send + Sync {
    async fn create(&self, new_achievement: NewAchievement) -> AppResult<Achievement>;
    async fn list_by_student(&self, student_id: Uuid) -> AppResult<Vec<Achievement>>;
    async fn delete(&self, student_id: Uuid, id: Uuid) -> AppResult<bool>;
    /// Attaches (or clears, with `None`) a certificate scan URL to an existing achievement
    /// — uploaded as a separate step after the achievement's text data is saved.
    async fn set_certificate(&self, student_id: Uuid, id: Uuid, certificate_url: Option<String>) -> AppResult<Option<Achievement>>;
}

// ─── SNBT post-exam tracking ("Rekap SNBT") ─────────────────────────────────────

pub struct SnbtTrackingUpsert {
    pub actual_score: Option<f64>,
    pub exam_date: Option<NaiveDate>,
    pub status: SnbtTrackingStatus,
    pub notes: String,
}

#[async_trait]
pub trait SnbtTrackingRepository: Send + Sync {
    /// Upserts on `student_id` (unique) — one real exam result per student, re-submitting
    /// for the same student updates it in place rather than accumulating duplicate rows.
    async fn upsert(&self, student_id: Uuid, entry: SnbtTrackingUpsert) -> AppResult<SnbtTracking>;
    async fn get_by_student(&self, student_id: Uuid) -> AppResult<Option<SnbtTracking>>;
}

// ─── SNBP roster metadata ("Daftar Siswa") ──────────────────────────────────────
// Pilihan 1/2 (target university+major) are intentionally NOT stored here — the
// service layer derives them live from the student's real `PtnTarget` rows so the
// roster never drifts out of sync with the actual Rasionalisasi data.

pub struct SnbpParticipationUpsert {
    pub school_id: Uuid,
    pub year: i32,
    pub konsultan: String,
    pub status: SnbpStatus,
    pub aktif: bool,
}

#[async_trait]
pub trait SnbpParticipationRepository: Send + Sync {
    /// Upserts on `student_id` (unique) — the School PIC edits a student's roster
    /// entry by re-submitting for the same student, never accumulating duplicate rows.
    async fn upsert(&self, student_id: Uuid, entry: SnbpParticipationUpsert) -> AppResult<SnbpParticipation>;
    async fn get_by_student(&self, student_id: Uuid) -> AppResult<Option<SnbpParticipation>>;
    async fn list_by_school(&self, school_id: Uuid) -> AppResult<Vec<SnbpParticipation>>;
}

// ─── Saved reports ("Laporan") ───────────────────────────────────────────────────

pub struct NewReport {
    pub school_id: Uuid,
    pub report_type: ReportType,
    pub title: String,
    pub period_label: String,
    pub class_filter: Option<String>,
    /// See `domain::report::Report::exam_track_filter` doc comment.
    pub exam_track_filter: Option<ExamTrack>,
    pub payload: ReportPayload,
}

#[async_trait]
pub trait ReportRepository: Send + Sync {
    async fn create(&self, input: NewReport) -> AppResult<Report>;
    async fn list_by_school(&self, school_id: Uuid) -> AppResult<Vec<Report>>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Report>>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

// ─── Audit trail ("Riwayat Perubahan") for edit-on-behalf actions ──────────────────

pub struct NewAuditLogEntry {
    pub actor_id: Uuid,
    pub actor_name: String,
    pub actor_role: String,
    pub student_id: Uuid,
    pub action: String,
    pub entity_type: String,
    pub summary: String,
}

#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn record(&self, entry: NewAuditLogEntry) -> AppResult<AuditLogEntry>;
    /// Most recent entries first, capped at `limit`.
    async fn list_by_student(&self, student_id: Uuid, limit: i64) -> AppResult<Vec<AuditLogEntry>>;
}

// ─── Payment transactions (gateway scaffolding) ────────────────────────────────────

pub struct NewPaymentTransaction {
    pub package_id: Uuid,
    pub buyer_user_id: Uuid,
    pub amount: i32,
    pub currency: String,
}

/// Fields a provider webhook (once a real one exists) is allowed to update — deliberately
/// narrow so a webhook can never touch `amount`/`package_id`/`buyer_user_id`.
pub struct PaymentStatusUpdate {
    pub status: PaymentStatus,
    pub provider: Option<String>,
    pub provider_ref: Option<String>,
    pub failure_reason: Option<String>,
}

#[async_trait]
pub trait PaymentTransactionRepository: Send + Sync {
    async fn create(&self, input: NewPaymentTransaction) -> AppResult<PaymentTransaction>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<PaymentTransaction>>;
    async fn find_by_provider_ref(&self, provider_ref: &str) -> AppResult<Option<PaymentTransaction>>;
    async fn list_by_buyer(&self, buyer_user_id: Uuid) -> AppResult<Vec<PaymentTransaction>>;
    /// Powers the admin "manual purchase request" queue — every transaction currently
    /// sitting in one status (typically `Pending`), most recent first.
    async fn list_by_status(&self, status: PaymentStatus) -> AppResult<Vec<PaymentTransaction>>;
    async fn update_status(&self, id: Uuid, update: PaymentStatusUpdate) -> AppResult<PaymentTransaction>;
}

// ─── Subject taxonomy (Kategori & Mata Uji) ────────────────────────────────────

#[async_trait]
pub trait TaxonomyRepository: Send + Sync {
    /// All categories ordered by `sort_order`, each with its subjects ordered by
    /// `sort_order` — the main read path everything else (dropdowns, question
    /// authoring) builds on.
    async fn list_categories_with_subjects(&self) -> AppResult<Vec<CategoryWithSubjects>>;
    async fn create_category(&self, name: String, description: String, sort_order: i32) -> AppResult<SubjectCategory>;
    async fn update_category(
        &self,
        id: Uuid,
        name: String,
        description: String,
        sort_order: i32,
    ) -> AppResult<Option<SubjectCategory>>;
    /// Cascades to the category's subjects via the DB's `ON DELETE CASCADE`.
    async fn delete_category(&self, id: Uuid) -> AppResult<bool>;
    async fn create_subject(&self, category_id: Uuid, name: String, code: String, sort_order: i32) -> AppResult<Subject>;
    async fn update_subject(&self, id: Uuid, name: String, code: String, sort_order: i32) -> AppResult<Option<Subject>>;
    async fn delete_subject(&self, id: Uuid) -> AppResult<bool>;
}

// ─── Simulasi UTBK ──────────────────────────────────────────────────────────────

pub struct NewSimulationTemplateSlot {
    pub sequence_index: i32,
    /// `None` iff `is_elective` — see `domain::simulation::SimulationTemplateSlot` doc comment.
    pub session_id: Option<Uuid>,
    pub break_seconds: i32,
    pub is_elective: bool,
}

pub struct NewSimulationTemplate {
    pub title: String,
    pub is_premium: bool,
    pub created_by: Uuid,
    pub slots: Vec<NewSimulationTemplateSlot>,
    /// See `domain::simulation::SimulationTemplate::is_draft` doc comment.
    pub is_draft: bool,
    pub elective_package_id: Option<Uuid>,
    pub template_kind: crate::domain::simulation::SimulationTemplateKind,
    /// See `domain::package::Package::exam_track` doc comment.
    pub exam_track: ExamTrack,
    /// See `domain::tryout::TryoutSession::school_type_scope` doc comment.
    pub school_type_scope: Option<SchoolType>,
    /// See `domain::simulation::SimulationTemplate::lockdown_override` doc comment.
    pub lockdown_override: Option<bool>,
}

#[async_trait]
pub trait SimulationTemplateRepository: Send + Sync {
    async fn create(&self, input: NewSimulationTemplate) -> AppResult<SimulationTemplate>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SimulationTemplate>>;
    /// `only_active = true` filters to `is_active = true` (used for the student-facing list).
    async fn list(&self, only_active: bool) -> AppResult<Vec<SimulationTemplate>>;
    async fn set_active(&self, id: Uuid, active: bool) -> AppResult<Option<SimulationTemplate>>;
    async fn set_premium(&self, id: Uuid, is_premium: bool) -> AppResult<Option<SimulationTemplate>>;
    /// Cheap targeted toggle — used by `PackageService::set_active`'s publish cascade and by
    /// an owner manually flipping draft status.
    async fn set_draft(&self, id: Uuid, is_draft: bool) -> AppResult<Option<SimulationTemplate>>;
    /// See `domain::simulation::SimulationTemplate::lockdown_override` doc comment. `None`
    /// resets to "inherit the global default".
    async fn set_lockdown(&self, id: Uuid, lockdown_override: Option<bool>) -> AppResult<Option<SimulationTemplate>>;
    /// `Ok(false)` if not found; fails with `AppError::Conflict` at the repository level if
    /// any `simulation_runs` already reference this template (preserve run history) — mirrors
    /// `TryoutSessionRepository::delete`'s doc comment describing the same FK-conflict pattern
    /// for attempts.
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

pub struct NewSimulationRunSlot {
    pub sequence_index: i32,
    pub session_id: Uuid,
    pub break_seconds: i32,
}

pub struct NewSimulationRun {
    pub template_id: Uuid,
    pub student_id: Uuid,
    pub slots: Vec<NewSimulationRunSlot>,
    /// See `domain::simulation::SimulationRun::lockdown_enabled` doc comment — the effective
    /// lockdown decision, already resolved by `SimulationService::start_run` before this input
    /// is constructed.
    pub lockdown_enabled: bool,
}

#[async_trait]
pub trait SimulationRunRepository: Send + Sync {
    /// Creates the run row AND all its `simulation_run_slots` rows (all `pending`, no
    /// attempt yet) in one call — implemented as a DB transaction in the Postgres impl.
    async fn create(&self, input: NewSimulationRun) -> AppResult<SimulationRun>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SimulationRun>>;
    async fn list_by_student(&self, student_id: Uuid) -> AppResult<Vec<SimulationRun>>;
    /// Student's most recent `completed` run, if any — ordered by `completed_at DESC LIMIT 1`.
    async fn latest_completed_for_student(&self, student_id: Uuid) -> AppResult<Option<SimulationRun>>;
    async fn set_run_status(
        &self,
        run_id: Uuid,
        status: SimulationRunStatus,
        completed_at: Option<DateTime<Utc>>,
        combined_estimate_score: Option<f64>,
    ) -> AppResult<()>;
    async fn advance_current_index(&self, run_id: Uuid, new_index: i32) -> AppResult<()>;
    /// Marks the slot at `sequence_index` within `run_id` as `active`, attaches `attempt_id`
    /// and `deadline_at`.
    async fn activate_slot(&self, run_id: Uuid, sequence_index: i32, attempt_id: Uuid, deadline_at: DateTime<Utc>) -> AppResult<()>;
    /// Marks the slot at `sequence_index` within `run_id` as `submitted`, sets `break_ends_at`.
    async fn submit_slot(&self, run_id: Uuid, sequence_index: i32, break_ends_at: Option<DateTime<Utc>>) -> AppResult<()>;
    /// See `domain::simulation::TemplateCompletionStats` doc comment — honest, aggregate-only
    /// benchmark for the sertifikat/laporan PDF, computed live (not cached/denormalized).
    async fn template_completion_stats(&self, template_id: Uuid) -> AppResult<TemplateCompletionStats>;
}

/// See `domain::simulation::LockdownViolation` doc comment.
pub struct NewLockdownViolation {
    pub run_id: Uuid,
    pub student_id: Uuid,
    pub event_type: String,
    pub detail: Option<String>,
}

#[async_trait]
pub trait LockdownViolationRepository: Send + Sync {
    async fn record(&self, input: NewLockdownViolation) -> AppResult<crate::domain::simulation::LockdownViolation>;
    /// Newest first — used by both the student's own live warning history and the Admin/Sekolah
    /// review view.
    async fn list_by_run(&self, run_id: Uuid) -> AppResult<Vec<crate::domain::simulation::LockdownViolation>>;
}

// ─── Platform settings ──────────────────────────────────────────────────────────

#[async_trait]
pub trait PlatformSettingsRepository: Send + Sync {
    /// The singleton row is seeded by the migration, so this should never be `None` in
    /// practice — but the return type stays honest about the theoretical empty-table case
    /// rather than panicking.
    async fn get(&self) -> AppResult<Option<PlatformSettings>>;
    async fn set_b2c_registration_enabled(&self, enabled: bool, updated_by: Uuid) -> AppResult<PlatformSettings>;
    /// See `domain::platform_settings::ScoreDisplayMode` doc comment.
    async fn set_score_display_mode(&self, mode: ScoreDisplayMode, updated_by: Uuid) -> AppResult<PlatformSettings>;
    /// See `domain::platform_settings::PlatformSettings::simulation_lockdown_default` doc comment.
    async fn set_simulation_lockdown_default(&self, enabled: bool, updated_by: Uuid) -> AppResult<PlatformSettings>;
}

// ─── IRT item calibration ───────────────────────────────────────────────────────

/// One question's calibrated Rasch difficulty parameter — see `domain::irt` doc comment.
pub struct QuestionIrtParam {
    pub question_id: Uuid,
    pub difficulty_b: f64,
    pub sample_size: i64,
    pub calibrated_at: DateTime<Utc>,
}

#[async_trait]
pub trait QuestionIrtParamRepository: Send + Sync {
    /// Upserts freshly calibrated `(question_id, difficulty_b, sample_size)` rows — called by
    /// `IrtService::recalibrate`. Every call fully overwrites the previous calibration for the
    /// questions it touches with `calibrated_at = now()`, so a re-run always reflects only the
    /// latest historical data snapshot.
    async fn upsert_many(&self, params: Vec<(Uuid, f64, i64)>) -> AppResult<()>;
    /// Fetches whatever calibration exists for these question ids — a subset may be missing (not
    /// every question has enough historical data yet); the caller must treat a missing id as
    /// "not calibrated", never assume/fabricate a value.
    async fn find_by_question_ids(&self, question_ids: &[Uuid]) -> AppResult<Vec<QuestionIrtParam>>;
    /// Honest summary of current calibration state for the Admin "Kalibrasi Ulang IRT" panel —
    /// `(calibrated item count, most recent calibrated_at)`. `None` if calibration has never run.
    async fn calibration_summary(&self) -> AppResult<Option<(i64, DateTime<Utc>)>>;
}

// ─── Notifications ──────────────────────────────────────────────────────────────

pub struct NewNotification {
    pub user_id: Uuid,
    pub kind: String,
    pub title: String,
    pub message: String,
    pub link_tab: Option<String>,
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create(&self, input: NewNotification) -> AppResult<Notification>;
    /// Most recent first.
    async fn list_for_user(&self, user_id: Uuid, limit: i64) -> AppResult<Vec<Notification>>;
    async fn unread_count(&self, user_id: Uuid) -> AppResult<i64>;
    /// Scoped to `user_id` so one user can never mark another's notification read. Returns
    /// `false` if no matching row was found (wrong id or not owned by `user_id`).
    async fn mark_read(&self, id: Uuid, user_id: Uuid) -> AppResult<bool>;
    async fn mark_all_read(&self, user_id: Uuid) -> AppResult<()>;
}

// ─── Institutions (PTN master data) ──────────────────────────────────────────────

pub struct NewInstitution {
    pub nama_ptn: String,
    pub singkatan: String,
    pub website: Option<String>,
    pub alamat: Option<String>,
    pub kota: Option<String>,
    pub provinsi: Option<String>,
    pub tahun_berdiri: Option<i32>,
    pub status: Option<String>,
    pub akreditasi: Option<String>,
    pub logo_url: Option<String>,
    pub sumber: Option<String>,
}

pub struct InstitutionUpdate {
    pub singkatan: String,
    pub website: Option<String>,
    pub alamat: Option<String>,
    pub kota: Option<String>,
    pub provinsi: Option<String>,
    pub tahun_berdiri: Option<i32>,
    pub status: Option<String>,
    pub akreditasi: Option<String>,
    pub logo_url: Option<String>,
    pub sumber: Option<String>,
}

#[async_trait]
pub trait InstitutionRepository: Send + Sync {
    /// Insert-or-update keyed on `nama_ptn` — used by the one-time CSV importer
    /// (`import_institutions`) so re-running it after refreshing the research data updates
    /// existing rows instead of duplicating them.
    async fn upsert_by_nama_ptn(&self, input: NewInstitution) -> AppResult<Institution>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Institution>>;
    /// Case-insensitive exact match — used by the frontend to resolve a `ptn_programs`
    /// row's plain-text `nama_ptn` to its institution profile without requiring the
    /// backfilled `institution_id` link to be present.
    async fn find_by_nama_ptn(&self, nama_ptn: &str) -> AppResult<Option<Institution>>;
    /// All institutions, optionally filtered by a case-insensitive substring match on
    /// `nama_ptn`/`singkatan`, ordered alphabetically. The catalog is small (137 rows) so
    /// no pagination is offered — same convention as `TaxonomyRepository::list_categories_with_subjects`.
    async fn list(&self, search: Option<String>) -> AppResult<Vec<Institution>>;
    async fn update(&self, id: Uuid, update: InstitutionUpdate) -> AppResult<Option<Institution>>;
    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}
