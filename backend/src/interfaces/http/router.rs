//! Route table. Kept as one place so the whole API surface is visible at a glance.

use axum::{
    routing::{get, patch, post, put},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

use super::handlers::{
    admission_deadline_handler, analytics_handler, auth_handler, elective_handler, entitlement_handler,
    event_handler, finance_handler, gamification_handler, institution_handler, notification_handler,
    package_handler, payment_handler, platform_settings_handler, public_handler, question_handler,
    question_import_handler, question_set_handler, rationalization_handler, report_handler, school_handler,
    simulation_handler, taxonomy_handler, tryout_handler, upload_handler, user_handler, voucher_handler,
};
use super::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(auth_handler::register))
        .route("/login", post(auth_handler::login))
        .route("/me", get(auth_handler::me).put(auth_handler::update_profile))
        .route("/me/password", put(auth_handler::change_password))
        .route("/me/avatar", post(auth_handler::upload_avatar));

    let question_routes = Router::new()
        .route("/", get(question_handler::list).post(question_handler::create))
        .route(
            "/:id",
            get(question_handler::get)
                .put(question_handler::update)
                .delete(question_handler::delete),
        )
        .route("/:id/review", post(question_handler::review))
        .route("/:id/submit-review", post(question_handler::submit_for_review))
        // Bulk "Import Soal" .docx — see application::question_import_service doc comment.
        .route("/import/preview", post(question_import_handler::preview))
        .route("/import/commit", post(question_import_handler::commit));

    let tryout_routes = Router::new()
        .route(
            "/sessions",
            get(tryout_handler::list_sessions).post(tryout_handler::create_session),
        )
        .route(
            "/sessions/:session_id",
            get(tryout_handler::get_session)
                .put(tryout_handler::update_session)
                .delete(tryout_handler::delete_session),
        )
        .route("/sessions/:session_id/start", post(tryout_handler::start_attempt))
        .route("/attempts", get(tryout_handler::my_attempts))
        .route("/attempts/:attempt_id", get(tryout_handler::get_attempt))
        .route("/attempts/:attempt_id/resume", get(tryout_handler::resume_attempt))
        .route("/attempts/:attempt_id/answers", put(tryout_handler::save_answer))
        .route("/attempts/:attempt_id/submit", post(tryout_handler::submit_attempt))
        .route("/attempts/:attempt_id/review", get(tryout_handler::get_review))
        .route("/attempts/:attempt_id/result", get(tryout_handler::get_result))
        // Admin-only: "is this student stuck mid-attempt right now" — see
        // TryoutService::admin_list_attempts doc comment. Nested under /tryout (not /admin) to
        // stay next to the rest of the attempt-related routes above.
        .route("/admin/students/:student_id/attempts", get(tryout_handler::admin_list_attempts));

    let school_routes = Router::new()
        .route("/", get(school_handler::list).post(school_handler::create))
        .route("/overview", get(school_handler::overview))
        .route("/roster-stats", get(school_handler::roster_stats))
        .route("/ptn-distribution", get(school_handler::ptn_distribution))
        .route("/recent-activity", get(school_handler::recent_activity))
        .route(
            "/:id",
            get(school_handler::get)
                .put(school_handler::update)
                .delete(school_handler::delete),
        )
        .route("/:id/status", patch(school_handler::set_status));

    let user_routes = Router::new()
        .route("/", get(user_handler::list).post(user_handler::create))
        .route("/:id", put(user_handler::update).delete(user_handler::delete))
        .route("/:id/status", patch(user_handler::set_status));

    let analytics_routes = Router::new()
        .route("/summary", get(analytics_handler::summary))
        .route("/score-trend", get(analytics_handler::score_trend))
        .route("/subject-breakdown", get(analytics_handler::subject_breakdown))
        .route("/question-status", get(analytics_handler::question_status))
        .route("/top-schools", get(analytics_handler::top_schools))
        .route("/score-trend-by-year", get(analytics_handler::score_trend_by_year))
        .route("/school/subject-breakdown", get(analytics_handler::school_subject_breakdown))
        .route("/school/score-trend", get(analytics_handler::school_score_trend))
        .route("/school/score-trend-by-year", get(analytics_handler::school_score_trend_by_year))
        .route("/school/ranking", get(analytics_handler::school_student_ranking))
        .route("/school/student-activity", get(analytics_handler::school_student_activity))
        .route("/me/subject-breakdown", get(analytics_handler::my_subject_breakdown))
        .route("/national-score-trend", get(analytics_handler::national_score_trend));

    let event_routes = Router::new()
        .route("/", get(event_handler::list).post(event_handler::create))
        .route("/upcoming", get(event_handler::list_upcoming))
        .route(
            "/:id",
            get(event_handler::get).put(event_handler::update).delete(event_handler::delete),
        )
        .route("/:id/status", patch(event_handler::set_status));

    let admission_deadline_routes = Router::new()
        .route(
            "/",
            get(admission_deadline_handler::list).post(admission_deadline_handler::create),
        )
        .route("/upcoming", get(admission_deadline_handler::list_upcoming))
        .route(
            "/:id",
            get(admission_deadline_handler::get)
                .put(admission_deadline_handler::update)
                .delete(admission_deadline_handler::delete),
        );

    let package_routes = Router::new()
        .route("/", get(package_handler::list).post(package_handler::create))
        .route(
            "/:id",
            get(package_handler::get)
                .put(package_handler::update)
                .delete(package_handler::delete),
        )
        .route("/:id/active", patch(package_handler::set_active))
        .route("/:id/content", get(package_handler::list_content).post(package_handler::add_content))
        .route("/:id/content/:item_id", axum::routing::delete(package_handler::remove_content))
        .route("/:id/bulk-approve-content", post(package_handler::bulk_approve_content))
        // "Mapel Pilihan" (TKA-style subject choice) — student-facing. See
        // `application::elective_service` doc comment.
        .route(
            "/:id/electives",
            get(elective_handler::get_electives).put(elective_handler::set_choices),
        );

    // Cross-package "Pilih Mapel Pilihan" overview — separate top-level nest (not under
    // `/api/packages`) so `GET /api/electives` never risks colliding with the `/:id` path param
    // above. See `application::elective_service` doc comment.
    let elective_routes = Router::new().route("/", get(elective_handler::my_elective_packages));

    let voucher_routes = Router::new()
        .route("/", get(voucher_handler::list).post(voucher_handler::create))
        .route("/redeem", post(voucher_handler::redeem))
        .route("/:id", get(voucher_handler::get).delete(voucher_handler::delete))
        .route("/:id/active", patch(voucher_handler::set_active));

    let gamification_routes = Router::new()
        .route(
            "/point-rules",
            get(gamification_handler::list_point_rules).post(gamification_handler::create_point_rule),
        )
        .route(
            "/point-rules/:id",
            put(gamification_handler::update_point_rule).delete(gamification_handler::delete_point_rule),
        )
        .route("/badges", get(gamification_handler::list_badges).post(gamification_handler::create_badge))
        .route(
            "/badges/:id",
            put(gamification_handler::update_badge).delete(gamification_handler::delete_badge),
        )
        .route("/badges/:id/active", patch(gamification_handler::set_badge_active))
        .route("/me/badges", get(gamification_handler::my_badges))
        .route(
            "/challenges",
            get(gamification_handler::list_challenges).post(gamification_handler::create_challenge),
        )
        .route(
            "/challenges/:id",
            put(gamification_handler::update_challenge).delete(gamification_handler::delete_challenge),
        )
        .route("/challenges/:id/status", patch(gamification_handler::set_challenge_status))
        .route("/leaderboard", get(gamification_handler::leaderboard));

    let finance_routes = Router::new()
        .route("/summary", get(finance_handler::summary))
        .route("/event-breakdown", get(finance_handler::event_breakdown))
        .route("/school-breakdown", get(finance_handler::school_breakdown));

    let rationalization_routes = Router::new()
        .route(
            "/programs",
            get(rationalization_handler::list_programs).post(rationalization_handler::create_program),
        )
        .route("/programs/rumpun", get(rationalization_handler::distinct_rumpun))
        .route("/programs/catalog-size", get(rationalization_handler::catalog_size))
        .route(
            "/programs/:id",
            get(rationalization_handler::get_program)
                .put(rationalization_handler::update_program)
                .delete(rationalization_handler::delete_program),
        )
        .route(
            "/rapor",
            get(rationalization_handler::list_rapor).post(rationalization_handler::upsert_rapor),
        )
        .route("/rapor/:id", axum::routing::delete(rationalization_handler::delete_rapor))
        .route("/rapor/bulk-import", axum::routing::post(rationalization_handler::bulk_import_rapor))
        .route(
            "/targets",
            get(rationalization_handler::my_targets).post(rationalization_handler::add_target),
        )
        .route(
            "/targets/:id",
            patch(rationalization_handler::set_target_priority).delete(rationalization_handler::remove_target),
        )
        .route(
            "/students/:student_id/targets",
            get(rationalization_handler::student_targets).post(rationalization_handler::add_student_target),
        )
        .route(
            "/students/:student_id/targets/:id",
            patch(rationalization_handler::set_student_target_priority).delete(rationalization_handler::remove_student_target),
        )
        .route(
            "/students/:student_id/rapor",
            get(rationalization_handler::student_rapor).post(rationalization_handler::upsert_student_rapor),
        )
        .route("/students/:student_id/rapor/:id", axum::routing::delete(rationalization_handler::delete_student_rapor))
        .route(
            "/students/:student_id/achievements",
            get(rationalization_handler::student_achievements).post(rationalization_handler::add_student_achievement),
        )
        .route(
            "/students/:student_id/achievements/:id",
            axum::routing::delete(rationalization_handler::delete_student_achievement),
        )
        .route("/students/:student_id/preview", get(rationalization_handler::student_preview_chance))
        .route("/students/:student_id/audit-log", get(rationalization_handler::student_audit_log))
        .route("/preview", get(rationalization_handler::preview_chance))
        .route(
            "/achievements",
            get(rationalization_handler::list_achievements).post(rationalization_handler::add_achievement),
        )
        .route("/achievements/:id", axum::routing::delete(rationalization_handler::delete_achievement))
        .route("/achievements/:id/certificate", post(rationalization_handler::upload_achievement_certificate))
        .route("/school/dashboard", get(rationalization_handler::school_dashboard))
        .route("/school/ranking", get(rationalization_handler::school_full_ranking))
        .route(
            "/school/alumni",
            get(rationalization_handler::list_alumni).post(rationalization_handler::add_alumni),
        )
        .route("/school/alumni/:id", axum::routing::delete(rationalization_handler::delete_alumni))
        .route(
            "/school/alumni/:id/rapor",
            get(rationalization_handler::list_alumni_rapor).post(rationalization_handler::upsert_alumni_rapor),
        )
        .route(
            "/school/alumni/:id/rapor/:score_id",
            axum::routing::delete(rationalization_handler::delete_alumni_rapor),
        )
        .route(
            "/school/eligibility",
            get(rationalization_handler::list_eligibility).post(rationalization_handler::upsert_eligibility),
        )
        .route("/school/snbt/dashboard", get(rationalization_handler::school_snbt_dashboard))
        .route("/school/snbt/roster", get(rationalization_handler::school_snbt_roster))
        .route("/school/students/:student_id/attempts", get(rationalization_handler::school_student_attempts))
        .route(
            "/school/students/:student_id/attempts/:attempt_id/result",
            get(rationalization_handler::school_student_attempt_result),
        )
        .route(
            "/school/students/:student_id/attempts/:attempt_id/review",
            get(rationalization_handler::school_student_attempt_review),
        )
        .route("/school/tka/roster", get(rationalization_handler::school_tka_roster))
        .route(
            "/school/snbt/students/:student_id/tracking",
            axum::routing::post(rationalization_handler::upsert_snbt_tracking),
        )
        .route("/school/snbp/roster", get(rationalization_handler::school_snbp_roster))
        .route(
            "/school/snbp/roster/:student_id",
            axum::routing::post(rationalization_handler::upsert_snbp_participation),
        );

    let question_set_routes = Router::new()
        .route("/", get(question_set_handler::list).post(question_set_handler::create))
        .route(
            "/:id",
            put(question_set_handler::update).delete(question_set_handler::delete),
        )
        .route("/:id/items", get(question_set_handler::get_items).post(question_set_handler::add_item))
        .route("/:id/items/:question_id", axum::routing::delete(question_set_handler::remove_item));

    let taxonomy_routes = Router::new()
        .route("/", get(taxonomy_handler::list))
        .route("/categories", post(taxonomy_handler::create_category))
        .route(
            "/categories/:id",
            put(taxonomy_handler::update_category).delete(taxonomy_handler::delete_category),
        )
        .route("/subjects", post(taxonomy_handler::create_subject))
        .route(
            "/subjects/:id",
            put(taxonomy_handler::update_subject).delete(taxonomy_handler::delete_subject),
        );

    let report_routes = Router::new()
        .route("/", get(report_handler::list_reports).post(report_handler::generate_report))
        .route("/:id", get(report_handler::get_report).delete(report_handler::delete_report));

    // Payment gateway scaffolding — see `domain::payment` doc comment. `/webhook` is
    // deliberately public (no `AuthUser`); everything else requires a logged-in buyer.
    let payment_routes = Router::new()
        .route("/provider", get(payment_handler::provider_status))
        .route("/checkout", post(payment_handler::initiate_purchase))
        .route("/", get(payment_handler::my_transactions))
        .route("/pending", get(payment_handler::list_pending))
        .route("/:id", get(payment_handler::get_transaction))
        .route("/:id/cancel", post(payment_handler::cancel_transaction))
        .route("/:id/fulfill", post(payment_handler::fulfill))
        .route("/webhook", post(payment_handler::webhook));

    // No-auth routes backing the public marketing landing page — see public_handler.rs.
    let public_routes = Router::new()
        .route("/stats", get(public_handler::public_stats))
        .route("/packages", get(public_handler::public_packages))
        .route("/vouchers/check", get(public_handler::check_voucher))
        .route("/registration-status", get(platform_settings_handler::registration_status));

    // Admin-only platform-wide settings — see `domain::platform_settings` doc comment.
    let settings_routes = Router::new()
        .route(
            "/b2c-registration",
            get(platform_settings_handler::get_settings).patch(platform_settings_handler::set_b2c_registration),
        )
        // UTBK/SNBT "Skor Instan" vs "Estimasi IRT" display mode + manual item recalibration —
        // see `domain::irt` / `application::irt_service` doc comments.
        .route("/score-display-mode", patch(platform_settings_handler::set_score_display_mode))
        // "Mode Terkunci" (focus/lockdown) global default for Simulasi TO — see
        // `domain::platform_settings::PlatformSettings::simulation_lockdown_default` doc comment.
        .route(
            "/simulation-lockdown-default",
            patch(platform_settings_handler::set_simulation_lockdown_default),
        )
        .route("/irt/status", get(platform_settings_handler::irt_status))
        .route("/irt/recalibrate", post(platform_settings_handler::recalibrate_irt));

    // Generic small-file upload endpoints not tied to one specific entity — the Badge/Package
    // icon picker's "Upload Gambar" tab, plus question-authoring images (Menjodohkan Gambar)
    // (see upload_handler.rs).
    let upload_routes = Router::new()
        .route("/icon", post(upload_handler::upload_icon))
        .route("/question-image", post(upload_handler::upload_question_image))
        .route("/banner", post(upload_handler::upload_banner));

    // School Package Entitlements — see `application::access_service` doc comment. Lets an
    // Admin attach a Package to a School so every student there gets real premium access.
    let entitlement_routes = Router::new()
        .route("/", post(entitlement_handler::create))
        .route("/school/:school_id", get(entitlement_handler::list_for_school))
        .route("/:id", axum::routing::delete(entitlement_handler::delete))
        .route("/me/status", get(entitlement_handler::my_status));

    // Simulasi UTBK — chains several TryoutSessions into one continuous, server-enforced
    // exam run (fixed order, official timing, mandatory breaks). See
    // `application::simulation_service` doc comment.
    let simulation_routes = Router::new()
        .route("/templates", get(simulation_handler::list_templates).post(simulation_handler::create_template))
        .route("/templates/:id/active", patch(simulation_handler::set_template_active))
        .route("/templates/:id/premium", patch(simulation_handler::set_template_premium))
        // Per-template "Mode Terkunci" override — see
        // `domain::simulation::SimulationTemplate::lockdown_override` doc comment.
        .route("/templates/:id/lockdown", patch(simulation_handler::set_template_lockdown))
        .route("/templates/:id", axum::routing::delete(simulation_handler::delete_template))
        // Aggregate-only benchmark for the sertifikat/laporan PDF — see
        // `SimulationService::template_stats` doc comment.
        .route("/templates/:id/stats", get(simulation_handler::template_stats))
        .route("/templates/:id/start", post(simulation_handler::start_run))
        .route("/runs", get(simulation_handler::list_my_runs))
        .route("/runs/:id", get(simulation_handler::get_run))
        .route("/runs/:id/begin", post(simulation_handler::begin_run))
        .route("/runs/:id/submit-current", post(simulation_handler::submit_current))
        .route("/runs/:id/advance", post(simulation_handler::advance_after_break))
        .route("/runs/:id/abandon", post(simulation_handler::abandon_run))
        // "Mode Terkunci" breach reporting/review — see
        // `SimulationService::record_violation`/`list_violations` doc comments.
        .route(
            "/runs/:id/violations",
            post(simulation_handler::report_violation).get(simulation_handler::list_violations),
        )
        // Admin-only equivalent of "/runs" scoped to any student — see
        // SimulationService::admin_list_runs doc comment.
        .route("/admin/students/:student_id/runs", get(simulation_handler::admin_list_runs));

    // Real per-user notification feed — see `application::notification_service` doc comment.
    let notification_routes = Router::new()
        .route("/", get(notification_handler::list_mine))
        .route("/unread-count", get(notification_handler::unread_count))
        .route("/read-all", post(notification_handler::mark_all_read))
        .route("/:id/read", post(notification_handler::mark_read));

    // Institution master data (see `domain::institution` doc comment). `/lookup` must be
    // declared before `/:id` so `GET /api/institutions/lookup?nama_ptn=...` doesn't get
    // swallowed by the `:id` path param.
    let institution_routes = Router::new()
        .route("/", get(institution_handler::list).post(institution_handler::create))
        .route("/lookup", get(institution_handler::lookup))
        .route(
            "/:id",
            get(institution_handler::get).put(institution_handler::update).delete(institution_handler::delete),
        );

    Router::new()
        .route("/health", get(health))
        .nest("/api/auth", auth_routes)
        .nest("/api/notifications", notification_routes)
        .nest("/api/institutions", institution_routes)
        .nest("/api/questions", question_routes)
        .nest("/api/question-sets", question_set_routes)
        .nest("/api/tryout", tryout_routes)
        .nest("/api/schools", school_routes)
        .nest("/api/users", user_routes)
        .nest("/api/analytics", analytics_routes)
        .nest("/api/events", event_routes)
        .nest("/api/deadlines", admission_deadline_routes)
        .nest("/api/packages", package_routes)
        .nest("/api/electives", elective_routes)
        .nest("/api/vouchers", voucher_routes)
        .nest("/api/gamification", gamification_routes)
        .nest("/api/finance", finance_routes)
        .nest("/api/rationalization", rationalization_routes)
        .nest("/api/reports", report_routes)
        .nest("/api/taxonomy", taxonomy_routes)
        .nest("/api/public", public_routes)
        .nest("/api/payments", payment_routes)
        .nest("/api/uploads", upload_routes)
        .nest("/api/school-entitlements", entitlement_routes)
        .nest("/api/simulations", simulation_routes)
        .nest("/api/settings", settings_routes)
        // Serves locally-stored avatar/certificate uploads back — see
        // `infrastructure::storage`. Not under `/api` since these are static files, not
        // JSON endpoints.
        .nest_service("/uploads", ServeDir::new("uploads"))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
