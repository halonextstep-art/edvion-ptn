//! Application layer — orchestrates business use-cases by composing domain entities
//! and repository ports. No SQL, no HTTP: services depend only on `domain::repository`
//! traits (injected as `Arc<dyn Trait>`), which makes them framework-agnostic and
//! testable in isolation from Postgres/Axum.

pub mod jwt;
pub mod password;
pub mod auth_service;
pub mod question_service;
pub mod question_set_service;
pub mod tryout_service;
pub mod school_service;
pub mod user_service;
pub mod analytics_service;
pub mod event_service;
pub mod package_service;
pub mod voucher_service;
pub mod gamification_service;
pub mod finance_service;
pub mod rationalization_service;
pub mod school_portal_service;
pub mod school_rationalization_service;
pub mod school_tka_service;
pub mod report_service;
pub mod payment_service;
pub mod taxonomy_service;
pub mod access_service;
pub mod elective_service;
pub mod simulation_service;
pub mod platform_settings_service;
pub mod notification_service;
pub mod institution_service;
pub mod question_import;
pub mod question_import_service;
pub mod irt_service;
pub mod admission_deadline_service;

pub use auth_service::AuthService;
pub use question_service::QuestionService;
pub use question_set_service::QuestionSetService;
pub use tryout_service::TryoutService;
pub use school_service::SchoolService;
pub use user_service::UserService;
pub use analytics_service::AnalyticsService;
pub use event_service::EventService;
pub use package_service::PackageService;
pub use voucher_service::VoucherService;
pub use gamification_service::GamificationService;
pub use finance_service::FinanceService;
pub use rationalization_service::RationalizationService;
pub use school_portal_service::SchoolPortalService;
pub use school_rationalization_service::SchoolRationalizationService;
pub use school_tka_service::SchoolTkaService;
pub use report_service::ReportService;
pub use payment_service::PaymentService;
pub use taxonomy_service::TaxonomyService;
pub use access_service::AccessService;
pub use elective_service::ElectiveService;
pub use simulation_service::SimulationService;
pub use platform_settings_service::PlatformSettingsService;
pub use notification_service::NotificationService;
pub use institution_service::InstitutionService;
pub use question_import_service::QuestionImportService;
pub use irt_service::IrtService;
pub use admission_deadline_service::AdmissionDeadlineService;
