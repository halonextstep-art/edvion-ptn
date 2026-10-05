use std::sync::Arc;

use edvionptn_backend::application::jwt::JwtService;
use edvionptn_backend::application::{
    AccessService, AdmissionDeadlineService, AnalyticsService, AuthService, ElectiveService, EventService,
    FinanceService, GamificationService, InstitutionService, IrtService, NotificationService, PackageService,
    PaymentService, PlatformSettingsService, QuestionImportService, QuestionService, QuestionSetService,
    RationalizationService, ReportService, SchoolPortalService, SchoolRationalizationService, SchoolService,
    SchoolTkaService, SimulationService, TaxonomyService, TryoutService, UserService, VoucherService,
};
use edvionptn_backend::config::Config;
use edvionptn_backend::domain::payment::NotConfiguredProvider;
use edvionptn_backend::infrastructure::db;
use edvionptn_backend::infrastructure::repositories::{
    PostgresAchievementRepository, PostgresAlumniBenchmarkRepository, PostgresAnalyticsRepository,
    PostgresAttemptRepository, PostgresAuditLogRepository, PostgresBadgeRepository, PostgresChallengeRepository,
    PostgresEventRepository, PostgresLeaderboardRepository, PostgresPackageRepository,
    PostgresPackageContentRepository, PostgresPackageElectiveRepository,
    PostgresPaymentTransactionRepository, PostgresPointRuleRepository, PostgresPtnProgramRepository,
    PostgresPtnTargetRepository, PostgresQuestionRepository, PostgresQuestionSetRepository,
    PostgresRaporScoreRepository, PostgresReportRepository, PostgresSchoolEligibilityRepository,
    PostgresLockdownViolationRepository, PostgresSchoolEntitlementRepository, PostgresSchoolRepository,
    PostgresSimulationRunRepository, PostgresSimulationTemplateRepository, PostgresSnbpParticipationRepository,
    PostgresSnbtTrackingRepository,
    PostgresNotificationRepository, PostgresPlatformSettingsRepository, PostgresTaxonomyRepository,
    PostgresTryoutSessionRepository, PostgresUserRepository, PostgresVoucherRepository,
    PostgresInstitutionRepository, PostgresQuestionIrtParamRepository, PostgresAdmissionDeadlineRepository,
};
use edvionptn_backend::interfaces::http::{build_router, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    tracing::info!("connecting to database...");
    let pool = db::create_pool(&config.database_url).await?;

    tracing::info!("running migrations...");
    db::run_migrations(&pool).await?;

    // ─── Composition root: wire infrastructure (Postgres) into application services ──
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let question_repo = Arc::new(PostgresQuestionRepository::new(pool.clone()));
    let question_set_repo = Arc::new(PostgresQuestionSetRepository::new(pool.clone()));
    let session_repo = Arc::new(PostgresTryoutSessionRepository::new(pool.clone()));
    let attempt_repo = Arc::new(PostgresAttemptRepository::new(pool.clone()));
    let school_repo = Arc::new(PostgresSchoolRepository::new(pool.clone()));
    let entitlement_repo = Arc::new(PostgresSchoolEntitlementRepository::new(pool.clone()));
    let simulation_template_repo = Arc::new(PostgresSimulationTemplateRepository::new(pool.clone()));
    let simulation_run_repo = Arc::new(PostgresSimulationRunRepository::new(pool.clone()));
    let lockdown_violation_repo = Arc::new(PostgresLockdownViolationRepository::new(pool.clone()));
    let analytics_repo = Arc::new(PostgresAnalyticsRepository::new(pool.clone()));
    let event_repo = Arc::new(PostgresEventRepository::new(pool.clone()));
    let package_repo = Arc::new(PostgresPackageRepository::new(pool.clone()));
    let package_content_repo = Arc::new(PostgresPackageContentRepository::new(pool.clone()));
    let package_elective_repo = Arc::new(PostgresPackageElectiveRepository::new(pool.clone()));
    let voucher_repo = Arc::new(PostgresVoucherRepository::new(pool.clone()));
    let point_rule_repo = Arc::new(PostgresPointRuleRepository::new(pool.clone()));
    let badge_repo = Arc::new(PostgresBadgeRepository::new(pool.clone()));
    let challenge_repo = Arc::new(PostgresChallengeRepository::new(pool.clone()));
    let leaderboard_repo = Arc::new(PostgresLeaderboardRepository::new(pool.clone()));
    let ptn_program_repo = Arc::new(PostgresPtnProgramRepository::new(pool.clone()));
    let rapor_repo = Arc::new(PostgresRaporScoreRepository::new(pool.clone()));
    let ptn_target_repo = Arc::new(PostgresPtnTargetRepository::new(pool.clone()));
    let alumni_repo = Arc::new(PostgresAlumniBenchmarkRepository::new(pool.clone()));
    let eligibility_repo = Arc::new(PostgresSchoolEligibilityRepository::new(pool.clone()));
    let achievement_repo = Arc::new(PostgresAchievementRepository::new(pool.clone()));
    let snbt_tracking_repo = Arc::new(PostgresSnbtTrackingRepository::new(pool.clone()));
    let snbp_participation_repo = Arc::new(PostgresSnbpParticipationRepository::new(pool.clone()));
    let report_repo = Arc::new(PostgresReportRepository::new(pool.clone()));
    let audit_log_repo = Arc::new(PostgresAuditLogRepository::new(pool.clone()));
    let payment_transaction_repo = Arc::new(PostgresPaymentTransactionRepository::new(pool.clone()));
    let taxonomy_repo = Arc::new(PostgresTaxonomyRepository::new(pool.clone()));
    let platform_settings_repo = Arc::new(PostgresPlatformSettingsRepository::new(pool.clone()));
    let notification_repo = Arc::new(PostgresNotificationRepository::new(pool.clone()));
    let institution_repo = Arc::new(PostgresInstitutionRepository::new(pool.clone()));
    let question_irt_param_repo = Arc::new(PostgresQuestionIrtParamRepository::new(pool.clone()));
    let admission_deadline_repo = Arc::new(PostgresAdmissionDeadlineRepository::new(pool.clone()));

    let jwt = Arc::new(JwtService::new(config.jwt_secret.clone(), config.jwt_expiry_hours));

    let auth_service = Arc::new(AuthService::new(user_repo.clone(), jwt.clone()));
    let question_service = Arc::new(QuestionService::new(question_repo.clone(), question_set_repo.clone()));
    let event_service = Arc::new(EventService::new(event_repo.clone(), session_repo.clone()));
    let rationalization_service = Arc::new(RationalizationService::new(
        ptn_program_repo.clone(),
        rapor_repo.clone(),
        ptn_target_repo.clone(),
        attempt_repo.clone(),
        achievement_repo.clone(),
        simulation_run_repo.clone(),
    ));
    let school_portal_service = Arc::new(SchoolPortalService::new(
        user_repo.clone(),
        attempt_repo.clone(),
        ptn_target_repo.clone(),
        ptn_program_repo.clone(),
    ));
    let access_service = Arc::new(AccessService::new(
        entitlement_repo,
        voucher_repo.clone(),
        user_repo.clone(),
        package_repo.clone(),
        school_repo.clone(),
        package_content_repo.clone(),
    ));
    let elective_service = Arc::new(ElectiveService::new(
        session_repo.clone(),
        package_repo.clone(),
        package_content_repo.clone(),
        package_elective_repo.clone(),
    ));
    // Constructed here (rather than near the other misc services below) because
    // `tryout_service` right below now depends on both — see `TryoutService::build_result_for_
    // submitted`'s `irt_score`/`score_display_mode` fields.
    let platform_settings_service = Arc::new(PlatformSettingsService::new(platform_settings_repo));
    let irt_service = Arc::new(IrtService::new(attempt_repo.clone(), question_irt_param_repo));
    let tryout_service = Arc::new(TryoutService::new(
        session_repo.clone(),
        attempt_repo.clone(),
        question_repo,
        question_set_repo.clone(),
        access_service.clone(),
        elective_service.clone(),
        irt_service.clone(),
        platform_settings_service.clone(),
    ));
    // Depends on `tryout_service` for the "Riwayat Pengerjaan TO" drill-down's per-attempt
    // result/pembahasan (student_attempt_result/student_attempt_review) — see that struct's
    // doc comment on the `tryout` field.
    let school_rationalization_service = Arc::new(SchoolRationalizationService::new(
        user_repo.clone(),
        ptn_target_repo,
        ptn_program_repo,
        rapor_repo,
        alumni_repo,
        eligibility_repo,
        achievement_repo,
        attempt_repo.clone(),
        snbt_tracking_repo,
        snbp_participation_repo,
        simulation_run_repo.clone(),
        tryout_service.clone(),
    ));
    // Rekap TKA — school-facing monitoring for TKA content. See
    // `application::school_tka_service` doc comment.
    let school_tka_service = Arc::new(SchoolTkaService::new(
        user_repo.clone(),
        package_repo.clone(),
        package_content_repo.clone(),
        package_elective_repo,
        session_repo.clone(),
        attempt_repo.clone(),
        simulation_template_repo.clone(),
    ));
    // Moved up from below `simulation_service` so `report_service` (which needs the
    // "akreditasi" report type's per-subject accuracy) can depend on it without a forward
    // reference — `AnalyticsService` only needs `analytics_repo`/`user_repo`, both already
    // available at this point.
    let analytics_service = Arc::new(AnalyticsService::new(analytics_repo, user_repo.clone()));
    // "Laporan" report-builder — the "tka" report type reuses `school_tka_service`'s own
    // aggregate directly (see `application::report_service::build_tka_report`), the
    // "akreditasi" report type reuses `analytics_service`'s real per-subject accuracy
    // aggregate, and `sessions` lets Performance/Participation filter attempts by
    // `exam_track` (an `Attempt` doesn't denormalize that field itself). Constructed after
    // `school_tka_service`/`analytics_service` so they can be cloned in, not moved.
    let report_service = Arc::new(ReportService::new(
        user_repo.clone(),
        attempt_repo.clone(),
        session_repo.clone(),
        report_repo,
        school_rationalization_service.clone(),
        school_tka_service.clone(),
        analytics_service.clone(),
    ));
    // Simulasi UTBK — chains several TryoutSessions into one continuous, server-enforced exam
    // run, driving `tryout_service` as a collaborator. See `application::simulation_service`
    // doc comment. `session_repo`/`simulation_template_repo` are cloned (not moved) here since
    // `package_service` below also needs them (to validate content assignment targets exist).
    let simulation_service = Arc::new(SimulationService::new(
        simulation_template_repo.clone(),
        simulation_run_repo,
        session_repo.clone(),
        tryout_service.clone(),
        access_service.clone(),
        elective_service.clone(),
        package_repo.clone(),
        platform_settings_service.clone(),
        lockdown_violation_repo,
    ));
    let question_set_service = Arc::new(QuestionSetService::new(question_set_repo));
    let school_service = Arc::new(SchoolService::new(school_repo.clone()));
    let user_service = Arc::new(UserService::new(user_repo.clone()));
    let package_service = Arc::new(PackageService::new(
        package_repo.clone(),
        package_content_repo,
        session_repo,
        simulation_template_repo,
        tryout_service.clone(),
        simulation_service.clone(),
        question_service.clone(),
    ));
    let voucher_service = Arc::new(VoucherService::new(voucher_repo, package_repo.clone()));
    let gamification_service = Arc::new(GamificationService::new(point_rule_repo, badge_repo, challenge_repo, leaderboard_repo, user_repo.clone()));
    let notification_service = Arc::new(NotificationService::new(notification_repo));
    // No real payment gateway account/API key exists yet — `NotConfiguredProvider` is the
    // only `PaymentProvider` wired in, so checkout honestly fails instead of faking success.
    let payment_service = Arc::new(PaymentService::new(
        payment_transaction_repo,
        package_repo.clone(),
        Arc::new(NotConfiguredProvider),
        voucher_service.clone(),
        notification_service.clone(),
        user_repo.clone(),
    ));
    let finance_service = Arc::new(FinanceService::new(event_repo, school_repo, package_repo));
    let taxonomy_service = Arc::new(TaxonomyService::new(taxonomy_repo));
    let institution_service = Arc::new(InstitutionService::new(institution_repo));
    let question_import_service = Arc::new(QuestionImportService::new(question_service.clone()));
    let admission_deadline_service = Arc::new(AdmissionDeadlineService::new(admission_deadline_repo));

    let state = AppState {
        auth_service,
        question_service,
        question_set_service,
        tryout_service,
        school_service,
        user_service,
        analytics_service,
        event_service,
        package_service,
        voucher_service,
        gamification_service,
        finance_service,
        rationalization_service,
        school_portal_service,
        school_rationalization_service,
        report_service,
        payment_service,
        audit_log_repo,
        taxonomy_service,
        access_service,
        simulation_service,
        platform_settings_service,
        notification_service,
        institution_service,
        question_import_service,
        elective_service,
        school_tka_service,
        irt_service,
        admission_deadline_service,
        jwt,
    };

    let app = build_router(state);

    let addr = format!("{}:{}", config.server_host, config.server_port);
    tracing::info!("EdvionPTN backend listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
