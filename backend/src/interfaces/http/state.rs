use std::sync::Arc;

use crate::application::jwt::JwtService;
use crate::application::{
    AccessService, AdmissionDeadlineService, AnalyticsService, AuthService, ElectiveService, EventService,
    FinanceService, GamificationService, InstitutionService, IrtService, NotificationService, PackageService,
    PaymentService, PlatformSettingsService, QuestionImportService, QuestionService, QuestionSetService,
    RationalizationService, ReportService, SchoolPortalService, SchoolRationalizationService, SchoolService,
    SchoolTkaService, SimulationService, TaxonomyService, TryoutService, UserService, VoucherService,
};
use crate::domain::repository::AuditLogRepository;

/// Shared application state injected into every handler via `State<AppState>`.
/// Cheaply cloneable (everything is an `Arc`) so Axum can hand a copy to each request.
#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub question_service: Arc<QuestionService>,
    /// "Set Soal" — curated, fixed question lists; see `domain::question_set` doc comment.
    pub question_set_service: Arc<QuestionSetService>,
    pub tryout_service: Arc<TryoutService>,
    pub school_service: Arc<SchoolService>,
    pub user_service: Arc<UserService>,
    pub analytics_service: Arc<AnalyticsService>,
    pub event_service: Arc<EventService>,
    pub package_service: Arc<PackageService>,
    pub voucher_service: Arc<VoucherService>,
    pub gamification_service: Arc<GamificationService>,
    pub finance_service: Arc<FinanceService>,
    pub rationalization_service: Arc<RationalizationService>,
    pub school_portal_service: Arc<SchoolPortalService>,
    pub school_rationalization_service: Arc<SchoolRationalizationService>,
    pub report_service: Arc<ReportService>,
    /// Payment gateway scaffolding — see `application::payment_service` doc comment. No
    /// real provider is wired in yet (`NotConfiguredProvider`), so every checkout attempt
    /// honestly fails; this field exists so the plumbing is ready for when one is.
    pub payment_service: Arc<PaymentService>,
    /// Accountability trail for "edit-on-behalf" actions (School PIC/Admin editing a
    /// student's own academic data) — written directly from handlers at the exact
    /// `authorize_student_edit`/roster-mutation boundary, not threaded through the
    /// services, since it's purely about WHO called the endpoint, which only the HTTP
    /// layer knows.
    pub audit_log_repo: Arc<dyn AuditLogRepository>,
    /// Admin-editable SNBT/TKA-IPA/TKA-IPS/AKM category+subject catalogue — see
    /// `domain::taxonomy` doc comment. No relationship to `questions.subject`.
    pub taxonomy_service: Arc<TaxonomyService>,
    /// Real server-side premium access gate — see `application::access_service` doc
    /// comment. Backs both the School Package Entitlement admin CRUD and the
    /// `TryoutService::start_attempt` enforcement check.
    pub access_service: Arc<AccessService>,
    /// Simulasi UTBK — chains several `TryoutSession`s into one continuous, server-enforced
    /// exam run. See `application::simulation_service` doc comment.
    pub simulation_service: Arc<SimulationService>,
    /// Platform-wide settings (currently just the B2C self-registration on/off toggle) — see
    /// `application::platform_settings_service` doc comment.
    pub platform_settings_service: Arc<PlatformSettingsService>,
    /// Real per-user notification feed — see `application::notification_service` doc comment.
    pub notification_service: Arc<NotificationService>,
    /// PTN institution master data (profile-level info beyond `ptn_programs`'s per-program
    /// stats) — see `domain::institution` doc comment.
    pub institution_service: Arc<InstitutionService>,
    /// Bulk "Import Soal" `.docx` pipeline — see `application::question_import_service` doc
    /// comment. Wraps `question_service` rather than duplicating its authorization/validation
    /// logic.
    pub question_import_service: Arc<QuestionImportService>,
    /// "Mapel Pilihan" — TKA-style subject-selection use-cases. See
    /// `application::elective_service` doc comment. Backs both the student-facing pick/choose
    /// endpoints and `TryoutService::start_attempt_impl`'s gating check.
    pub elective_service: Arc<ElectiveService>,
    /// Rekap TKA — school-facing monitoring for TKA content (elective-pick progress + real
    /// per-subject scores). See `application::school_tka_service` doc comment.
    pub school_tka_service: Arc<SchoolTkaService>,
    /// "Estimasi IRT" — item calibration + recalibration use-cases. See
    /// `application::irt_service` doc comment. `TryoutService` holds its own copy (injected at
    /// construction) to compute `AttemptResult::irt_score`; this copy backs the Admin-only
    /// "Kalibrasi Ulang IRT" endpoints.
    pub irt_service: Arc<IrtService>,
    /// Kalender Deadline SNBP/SNBT/UTBK — see `application::admission_deadline_service`
    /// doc comment. Admin-only writes, shared Admin/School/Student read for the
    /// "Deadline Mendatang" countdown widget.
    pub admission_deadline_service: Arc<AdmissionDeadlineService>,
    pub jwt: Arc<JwtService>,
}
