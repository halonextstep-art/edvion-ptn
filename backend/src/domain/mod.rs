//! Domain layer — the innermost layer of the architecture.
//!
//! Contains pure business entities/value-objects (no framework, no SQL, no HTTP) and the
//! *repository traits* (ports) that the application layer depends on. Infrastructure
//! provides the concrete (Postgres) implementations of these traits — this is the
//! Dependency Inversion Principle: domain/application define the interface, infrastructure
//! implements it.

pub mod user;
pub mod school;
pub mod question;
pub mod question_set;
pub mod tryout;
pub mod event;
pub mod analytics;
pub mod package;
pub mod voucher;
pub mod gamification;
pub mod finance;
pub mod rationalization;
pub mod report;
pub mod payment;
pub mod taxonomy;
pub mod entitlement;
pub mod simulation;
pub mod platform_settings;
pub mod notification;
pub mod institution;
pub mod irt;
pub mod admission_deadline;
pub mod repository;

pub use user::{Role, User};
pub use school::School;
pub use question::{Question, QuestionStatus, QuestionType, Difficulty};
pub use question_set::QuestionSet;
pub use tryout::{TryoutSession, SessionType, Attempt, AttemptStatus, AttemptAnswer};
pub use event::{Event, EventStatus, EventType};
pub use analytics::{AnalyticsSummary, QuestionStatusCount, ScoreTrendPoint, SubjectAccuracy};
pub use package::{Package, PackageContentItem, PackageContentType, ExamTrack};
pub use voucher::{Voucher, VoucherType, DiscountType, VoucherStatus};
pub use gamification::{PointRule, Badge, BadgeWithStats, Rarity, BadgeConditionType, Challenge, ChallengeWithStats, ChallengeType, ChallengeStatus, LeaderboardEntry};
pub use finance::{FinanceSummary, EventRevenueBreakdown, SchoolRevenueBreakdown};
pub use rationalization::{PtnProgram, Track, Priority, RaporScore, PtnTarget, ChanceTier, ChanceResult};
pub use report::{Report, ReportType, ReportPayload};
pub use payment::{PaymentTransaction, PaymentStatus, PaymentProvider, NotConfiguredProvider};
pub use taxonomy::{SubjectCategory, Subject, CategoryWithSubjects};
pub use entitlement::SchoolPackageEntitlement;
pub use simulation::{SimulationRun, SimulationRunSlot, SimulationRunStatus, SimulationSlotStatus, SimulationTemplate, SimulationTemplateSlot};
pub use platform_settings::{PlatformSettings, ScoreDisplayMode};
pub use notification::Notification;
pub use institution::Institution;
pub use admission_deadline::{AdmissionDeadline, AdmissionTrack};
