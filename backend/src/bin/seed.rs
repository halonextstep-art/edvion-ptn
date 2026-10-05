//! Demo data seeder. Reuses the real application services (AuthService, QuestionService,
//! TryoutService, SchoolService, EventService) so seeded data goes through the exact same
//! validation/business rules as production traffic — no separate "seed-only" code path to
//! drift out of sync. The only exception is backdating attempt timestamps at the very end
//! (raw SQL against the pool) purely so the 14-day analytics trend has more than one day
//! of data to show — the scores/accuracy themselves are still 100% computed by the real
//! scoring logic in `TryoutService::submit_attempt`, nothing is fabricated.
//!
//! Run with: `cargo run --bin seed` (after `DATABASE_URL` is set and migrations have run).
//! Safe to re-run: every step either skips or looks up existing rows on conflict.
//!
//! SAFETY GUARD: this creates publicly-known demo accounts (documented plaintext passwords,
//! printed to stdout at the end) and fake schools/questions/packages — never intended to run
//! against a live production database. Requires `SEED_CONFIRM=yes-seed-demo-data` in the
//! environment, so a stray/accidental `cargo run --bin seed` (or a copy-pasted CI command)
//! against `DATABASE_URL` pointing at production can't silently reintroduce demo data. See
//! `backend/scripts/cleanup_demo_data.sql` for removing demo data that was already seeded.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Datelike, Duration, Utc};
use uuid::Uuid;

use edvionptn_backend::application::auth_service::RegisterInput;
use edvionptn_backend::application::event_service::CreateEventInput;
use edvionptn_backend::application::jwt::JwtService;
use edvionptn_backend::application::package_service::CreatePackageInput;
use edvionptn_backend::application::question_service::{CreateQuestionInput, ReviewAction};
use edvionptn_backend::application::rationalization_service::{AchievementInput, RaporEntryInput};
use edvionptn_backend::application::school_rationalization_service::{AddAlumniInput, SnbpParticipationInput};
use edvionptn_backend::application::school_service::CreateSchoolInput;
use edvionptn_backend::application::tryout_service::CreateSessionInput;
use edvionptn_backend::application::gamification_service::{CreateBadgeInput, CreateChallengeInput, CreatePointRuleInput};
use edvionptn_backend::application::voucher_service::CreateVoucherInput;
use edvionptn_backend::application::{
    AccessService, AuthService, ElectiveService, EventService, GamificationService, IrtService, PackageService,
    PlatformSettingsService, QuestionService, RationalizationService, SchoolRationalizationService, SchoolService,
    SimulationService, TaxonomyService, TryoutService, VoucherService,
};
use edvionptn_backend::config::Config;
use edvionptn_backend::domain::event::{EventStatus, EventType};
use edvionptn_backend::domain::gamification::{BadgeConditionType, ChallengeType, Rarity};
use edvionptn_backend::domain::package::ExamTrack;
use edvionptn_backend::domain::question::{Difficulty, Question, QuestionType};
use edvionptn_backend::domain::rationalization::{AchievementLevel, Priority, SnbpStatus, Track};
use edvionptn_backend::domain::school::{PackageType, School, SchoolStatus, SchoolType};
use edvionptn_backend::domain::tryout::SessionType;
use edvionptn_backend::domain::user::{Role, User};
use edvionptn_backend::domain::voucher::{DiscountType, VoucherType};
use edvionptn_backend::infrastructure::db;
use edvionptn_backend::infrastructure::repositories::{
    PostgresAchievementRepository, PostgresAlumniBenchmarkRepository, PostgresAttemptRepository, PostgresBadgeRepository,
    PostgresChallengeRepository, PostgresEventRepository, PostgresLeaderboardRepository, PostgresLockdownViolationRepository,
    PostgresPackageRepository, PostgresPackageContentRepository, PostgresPackageElectiveRepository,
    PostgresPlatformSettingsRepository, PostgresPointRuleRepository, PostgresPtnProgramRepository,
    PostgresPtnTargetRepository, PostgresQuestionIrtParamRepository, PostgresQuestionRepository,
    PostgresQuestionSetRepository, PostgresRaporScoreRepository, PostgresSchoolEligibilityRepository,
    PostgresSchoolEntitlementRepository, PostgresSchoolRepository, PostgresSimulationRunRepository,
    PostgresSimulationTemplateRepository,
    PostgresSnbpParticipationRepository, PostgresSnbtTrackingRepository, PostgresTaxonomyRepository,
    PostgresTryoutSessionRepository, PostgresUserRepository, PostgresVoucherRepository,
};
use edvionptn_backend::interfaces::http::middleware::AuthUser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    // See module doc comment — this binary seeds fake accounts/content with publicly-documented
    // passwords, so it refuses to run at all unless explicitly confirmed via env var.
    if std::env::var("SEED_CONFIRM").as_deref() != Ok("yes-seed-demo-data") {
        anyhow::bail!(
            "Refusing to run: this seeds demo accounts/content with publicly-documented \
             passwords (see this file's module doc comment). Set SEED_CONFIRM=yes-seed-demo-data \
             to confirm you want this (only ever appropriate for a local/dev/staging database, \
             never production)."
        );
    }

    let config = Config::from_env()?;
    let pool = db::create_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;

    // ─── Services (same composition as main.rs) ──────────────────────────────────────
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let question_repo = Arc::new(PostgresQuestionRepository::new(pool.clone()));
    let question_set_repo = Arc::new(PostgresQuestionSetRepository::new(pool.clone()));
    let session_repo = Arc::new(PostgresTryoutSessionRepository::new(pool.clone()));
    let attempt_repo = Arc::new(PostgresAttemptRepository::new(pool.clone()));
    let school_repo = Arc::new(PostgresSchoolRepository::new(pool.clone()));
    let entitlement_repo = Arc::new(PostgresSchoolEntitlementRepository::new(pool.clone()));
    let event_repo = Arc::new(PostgresEventRepository::new(pool.clone()));
    let package_repo = Arc::new(PostgresPackageRepository::new(pool.clone()));
    let package_content_repo = Arc::new(PostgresPackageContentRepository::new(pool.clone()));
    let package_elective_repo = Arc::new(PostgresPackageElectiveRepository::new(pool.clone()));
    let simulation_template_repo = Arc::new(PostgresSimulationTemplateRepository::new(pool.clone()));
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
    let simulation_run_repo = Arc::new(PostgresSimulationRunRepository::new(pool.clone()));
    let achievement_repo = Arc::new(PostgresAchievementRepository::new(pool.clone()));
    let snbt_tracking_repo = Arc::new(PostgresSnbtTrackingRepository::new(pool.clone()));
    let snbp_participation_repo = Arc::new(PostgresSnbpParticipationRepository::new(pool.clone()));
    let taxonomy_repo = Arc::new(PostgresTaxonomyRepository::new(pool.clone()));
    let platform_settings_repo = Arc::new(PostgresPlatformSettingsRepository::new(pool.clone()));
    let question_irt_param_repo = Arc::new(PostgresQuestionIrtParamRepository::new(pool.clone()));
    let lockdown_violation_repo = Arc::new(PostgresLockdownViolationRepository::new(pool.clone()));
    let jwt = Arc::new(JwtService::new(config.jwt_secret.clone(), config.jwt_expiry_hours));

    let auth_service = AuthService::new(user_repo.clone(), jwt);
    let question_service = QuestionService::new(question_repo.clone(), question_set_repo.clone());
    let event_service = EventService::new(event_repo, session_repo.clone());
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
        package_elective_repo,
    ));
    let platform_settings_service = Arc::new(PlatformSettingsService::new(platform_settings_repo));
    let irt_service = Arc::new(IrtService::new(attempt_repo.clone(), question_irt_param_repo));
    let tryout_service = Arc::new(TryoutService::new(
        session_repo.clone(),
        attempt_repo.clone(),
        question_repo,
        question_set_repo,
        access_service.clone(),
        elective_service.clone(),
        irt_service,
        platform_settings_service.clone(),
    ));
    let simulation_service = Arc::new(SimulationService::new(
        simulation_template_repo.clone(),
        simulation_run_repo.clone(),
        session_repo.clone(),
        tryout_service.clone(),
        access_service,
        elective_service,
        package_repo.clone(),
        platform_settings_service,
        lockdown_violation_repo,
    ));
    let school_service = SchoolService::new(school_repo);
    let package_service = PackageService::new(
        package_repo.clone(),
        package_content_repo,
        session_repo,
        simulation_template_repo,
        tryout_service.clone(),
        simulation_service,
    );
    let voucher_service = VoucherService::new(voucher_repo, package_repo);
    let taxonomy_service = TaxonomyService::new(taxonomy_repo);
    let gamification_service =
        GamificationService::new(point_rule_repo, badge_repo, challenge_repo, leaderboard_repo, user_repo.clone());
    // Sama persis dengan komposisi service di main.rs — dipakai untuk seed rapor,
    // prestasi, target PTN (siswa) dan alumni/eligible (sekolah) di bawah, lewat kode
    // yang identik dengan yang dipakai HTTP API (bukan raw SQL).
    let rationalization_service = RationalizationService::new(
        ptn_program_repo.clone(),
        rapor_repo.clone(),
        ptn_target_repo.clone(),
        attempt_repo.clone(),
        achievement_repo.clone(),
        simulation_run_repo.clone(),
    );
    let school_rationalization_service = SchoolRationalizationService::new(
        user_repo.clone(),
        ptn_target_repo,
        ptn_program_repo,
        rapor_repo,
        alumni_repo,
        eligibility_repo,
        achievement_repo,
        attempt_repo,
        snbt_tracking_repo,
        snbp_participation_repo,
        simulation_run_repo,
    );

    // ─── Admin account first — every other seeded row goes through a service that
    // requires an authenticated Admin actor, so there is no raw-SQL/bypass path left. ──
    let admin = register_or_skip(&auth_service, "Admin Pusat", "admin@edvionptn.id", "admin123", Role::Admin, None).await?;
    let admin_actor = AuthUser { user_id: admin.id, email: admin.email.clone(), role: Role::Admin };

    // ─── Content team (create/edit soal, submit for review) ─────────────────────────
    let content1 = register_or_skip(&auth_service, "Tim Konten Edvion", "konten@edvion.id", "konten123", Role::Content, None).await?;
    let content1_actor = AuthUser { user_id: content1.id, email: content1.email.clone(), role: Role::Content };
    let content2 = register_or_skip(&auth_service, "Rina Marlina", "rina.konten@edvion.id", "konten123", Role::Content, None).await?;
    let _content2_actor = AuthUser { user_id: content2.id, email: content2.email.clone(), role: Role::Content };

    // ─── Schools (created via SchoolService, same code path the admin UI uses) ──────
    type SchoolSeed = (&'static str, SchoolType, &'static str, &'static str, &'static str, &'static str, PackageType, &'static str);
    let school_seeds: Vec<SchoolSeed> = vec![
        ("SMA Negeri 1 Jakarta", SchoolType::Sma, "Jakarta Pusat", "DKI Jakarta", "info@sman1jakarta.sch.id", "021-3456789", PackageType::Enterprise, "Drs. Bambang Suryadi, M.Pd"),
        ("SMA Negeri 3 Surabaya", SchoolType::Sma, "Surabaya", "Jawa Timur", "sman3sby@sekolah.id", "031-8765432", PackageType::Premium, "Dr. Bambang Priyo"),
        ("MA Negeri 1 Yogyakarta", SchoolType::Ma, "Yogyakarta", "DI Yogyakarta", "man1yk@sekolah.id", "0274-567890", PackageType::Premium, "H. Ahmad Fathoni, M.Si"),
        ("SMK Negeri 2 Bandung", SchoolType::Smk, "Bandung", "Jawa Barat", "smkn2bdg@sekolah.id", "022-4567890", PackageType::Basic, "Ir. Hendra Kusuma"),
    ];

    let mut schools: HashMap<&'static str, School> = HashMap::new();
    for (name, school_type, city, province, email, phone, package_type, contact_person) in school_seeds {
        // Revenue share follows the plan tier — a real business rule, not a per-school
        // fabrication. `monthly_revenue` starts at 0: there's no billing system yet, so it's
        // left honest rather than backfilled with a fake contract value.
        let revenue_share = match package_type {
            PackageType::Enterprise => 20,
            PackageType::Premium => 15,
            PackageType::Basic => 10,
        };
        let school = match school_service
            .create(&admin_actor, CreateSchoolInput {
                name: name.to_string(), school_type, city: city.to_string(), province: province.to_string(),
                email: email.to_string(), phone: phone.to_string(), package_type, contact_person: contact_person.to_string(),
                status: SchoolStatus::Active, revenue_share, monthly_revenue: 0,
            })
            .await
        {
            Ok(s) => {
                println!("seeded school '{}'", s.name);
                s
            }
            Err(_) => school_service
                .list(&admin_actor)
                .await?
                .into_iter()
                .find(|s| s.name == name)
                .expect("school should exist if create() reported a conflict"),
        };
        schools.insert(name, school);
    }
    let sman1 = schools.get("SMA Negeri 1 Jakarta").unwrap().id;
    let sman3 = schools.get("SMA Negeri 3 Surabaya").unwrap().id;
    let man1 = schools.get("MA Negeri 1 Yogyakarta").unwrap().id;
    let smkn2 = schools.get("SMK Negeri 2 Bandung").unwrap().id;

    // ─── School PIC accounts (one per school) — captured (not discarded) so they can act
    // as the actor for seeding Kaka Kelas/Eligible below. ─────────────────────────────
    let pic_sman1 = register_or_skip(&auth_service, "SMA Negeri 1 Jakarta", "sekolah@sman1.sch.id", "sekolah123", Role::School, Some(sman1)).await?;
    let pic_sman3 = register_or_skip(&auth_service, "SMA Negeri 3 Surabaya", "pic@sman3sby.sch.id", "sekolah123", Role::School, Some(sman3)).await?;
    let pic_man1 = register_or_skip(&auth_service, "MA Negeri 1 Yogyakarta", "pic@man1yk.sch.id", "sekolah123", Role::School, Some(man1)).await?;
    let pic_smkn2 = register_or_skip(&auth_service, "SMK Negeri 2 Bandung", "pic@smkn2bdg.sch.id", "sekolah123", Role::School, Some(smkn2)).await?;
    let school_pics: Vec<User> = vec![pic_sman1, pic_sman3, pic_man1, pic_smkn2];

    // ─── Students across all 4 schools ───────────────────────────────────────────────
    type StudentSeed = (&'static str, &'static str, Uuid);
    let student_seeds: Vec<StudentSeed> = vec![
        ("Ahmad Fauzi", "siswa@student.com", sman1),
        ("Bunga Rahayu", "bunga@sman1.sch.id", sman1),
        ("Andi Pratama", "andi@sman1.sch.id", sman1),
        ("Hana Wijaya", "hana@sman1.sch.id", sman1),
        ("Rizki Maulana", "rizki@sman3sby.sch.id", sman3),
        ("Dewi Lestari", "dewi@sman3sby.sch.id", sman3),
        ("Fajar Ramadhan", "fajar@sman3sby.sch.id", sman3),
        ("Siti Nurhaliza", "siti@man1yk.sch.id", man1),
        ("Bagas Pratama", "bagas@man1yk.sch.id", man1),
        ("Putri Ayu", "putri@man1yk.sch.id", man1),
        ("Farhan Nugraha", "farhan@smkn2bdg.sch.id", smkn2),
        ("Yusuf Maulidan", "yusuf@smkn2bdg.sch.id", smkn2),
    ];
    let mut students: Vec<User> = Vec::new();
    for (name, email, school_id) in student_seeds {
        let s = register_or_skip(&auth_service, name, email, "siswa123", Role::Student, Some(school_id)).await?;
        students.push(s);
    }

    // ─── Rasionalisasi: nilai rapor, prestasi, dan target PTN (SNBP+SNBT) per siswa ───
    // Supaya Dashboard/Daftar Siswa/Rasionalisasi (ranking) di kedua portal (siswa &
    // sekolah) tidak kosong saat demo. Semua lewat RationalizationService — kode yang
    // sama persis dipakai HTTP API — bukan raw SQL. Rapor & prestasi tidak butuh
    // katalog; target PTN butuh program studi NYATA dari `cargo run --bin
    // import_ptn_catalog` — kalau katalog belum diimpor, bagian target di-skip dengan
    // pesan jelas, tidak menggagalkan seluruh seed.
    println!("\nSeeding rasionalisasi (rapor, prestasi, target PTN)...");

    async fn find_program_id(service: &RationalizationService, actor: &AuthUser, keyword: &str) -> Option<Uuid> {
        service
            .list_programs(actor, Some(keyword.to_string()), None, None, 1, 1)
            .await
            .ok()
            .and_then(|(items, _)| items.into_iter().next())
            .map(|p| p.id)
    }

    let rapor_subjects = ["Matematika Wajib", "B Indonesia", "B Inggris Wajib", "Fisika"];

    // Per-siswa (urutan sama dengan `students` di atas): skor dasar rapor, prestasi
    // opsional, dan target PTN jalur SNBP/SNBT (dicari dari katalog nyata via keyword).
    type StudentProfile = (
        f64,
        Option<(&'static str, AchievementLevel, i32, &'static str)>,
        Vec<(&'static str, Priority)>,
        Vec<(&'static str, Priority)>,
    );
    let profiles: Vec<StudentProfile> = vec![
        (88.0, Some(("Olimpiade Matematika Tingkat Provinsi", AchievementLevel::Provinsi, 2025, "Juara 2")),
            vec![("Teknik Informatika", Priority::Utama), ("Manajemen", Priority::Cadangan)],
            vec![("Teknik Informatika", Priority::Utama)]),
        (76.0, None,
            vec![("Psikologi", Priority::Utama)],
            vec![("Psikologi", Priority::Utama), ("Ilmu Komunikasi", Priority::Aman)]),
        (91.0, Some(("Lomba Karya Tulis Ilmiah Nasional", AchievementLevel::Nasional, 2025, "Juara 1")),
            vec![("Kedokteran", Priority::Utama)],
            vec![("Kedokteran", Priority::Utama)]),
        (68.0, None, vec![], vec![("Akuntansi", Priority::Aman)]),
        (82.0, None,
            vec![("Hukum", Priority::Utama)],
            vec![("Hukum", Priority::Cadangan)]),
        (73.0, Some(("Kompetisi Debat Bahasa Inggris", AchievementLevel::KabKota, 2024, "Juara 3")),
            vec![("Ilmu Komunikasi", Priority::Utama)], vec![]),
        (85.0, None,
            vec![("Teknik Sipil", Priority::Utama), ("Teknik Informatika", Priority::Aman)],
            vec![("Teknik Sipil", Priority::Utama)]),
        (79.0, Some(("Olimpiade Sains Nasional - Biologi", AchievementLevel::Nasional, 2025, "Finalis")),
            vec![("Farmasi", Priority::Utama)],
            vec![("Farmasi", Priority::Cadangan)]),
        (70.0, None, vec![], vec![("Manajemen", Priority::Utama)]),
        (93.0, Some(("Olimpiade Matematika Tingkat Nasional", AchievementLevel::Nasional, 2025, "Juara 1")),
            vec![("Teknik Informatika", Priority::Utama)],
            vec![("Teknik Informatika", Priority::Utama), ("Teknik Sipil", Priority::Aman)]),
        (65.0, None, vec![("Pendidikan Matematika", Priority::Aman)], vec![]),
        (77.0, None,
            vec![("Akuntansi", Priority::Utama)],
            vec![("Manajemen", Priority::Cadangan)]),
    ];

    let mut rapor_seeded = 0usize;
    let mut achievement_seeded = 0usize;
    let mut target_seeded = 0usize;
    let mut target_skipped_no_catalog = 0usize;

    for (idx, student) in students.iter().enumerate() {
        let Some((base, achievement, snbp_targets, snbt_targets)) = profiles.get(idx) else { continue };
        let student_actor = AuthUser { user_id: student.id, email: student.email.clone(), role: Role::Student };

        // Rapor — skip kalau siswa ini sudah punya nilai (idempotent re-run).
        if rationalization_service.list_rapor_for(student.id).await.unwrap_or_default().is_empty() {
            let entries: Vec<RaporEntryInput> = [4, 5]
                .into_iter()
                .flat_map(|sem| {
                    rapor_subjects.iter().enumerate().map(move |(j, subj)| RaporEntryInput {
                        semester: sem,
                        subject: (*subj).to_string(),
                        score: (base + (j as f64 * 1.7) - 2.0).clamp(50.0, 100.0),
                        is_minat: false,
                    })
                })
                .collect();
            match rationalization_service.upsert_rapor_for(student.id, entries).await {
                Ok(saved) => rapor_seeded += saved.len(),
                Err(e) => println!("skip rapor ({}): {e}", student.name),
            }
        }

        // Prestasi (opsional per siswa).
        if let Some((nama, tingkat, tahun, juara)) = achievement {
            let existing = rationalization_service.list_achievements_for(student.id).await.unwrap_or_default();
            if !existing.iter().any(|a| a.nama == *nama) {
                match rationalization_service
                    .add_achievement_for(
                        student.id,
                        AchievementInput { nama: (*nama).to_string(), tingkat: *tingkat, tahun: *tahun, juara: (*juara).to_string() },
                    )
                    .await
                {
                    Ok(_) => achievement_seeded += 1,
                    Err(e) => println!("skip prestasi ({}): {e}", student.name),
                }
            }
        }

        // Target PTN (SNBP & SNBT) — dicari dari katalog nyata per keyword.
        let existing_targets = rationalization_service.targets_with_chance_for(student.id).await.unwrap_or_default();
        for (track, keyword_priorities) in [(Track::Snbp, snbp_targets), (Track::Snbt, snbt_targets)] {
            for &(keyword, priority) in keyword_priorities.iter() {
                let Some(pid) = find_program_id(&rationalization_service, &admin_actor, keyword).await else {
                    target_skipped_no_catalog += 1;
                    continue;
                };
                if existing_targets.iter().any(|t| t.program.id == pid && t.target.track == track) {
                    continue;
                }
                match rationalization_service.add_target_for(student.id, pid, track, priority).await {
                    Ok(_) => target_seeded += 1,
                    Err(e) => println!("skip target {track:?} ({} / {keyword}): {e}", student.name),
                }
            }
        }
    }
    println!("rasionalisasi siswa: {rapor_seeded} nilai rapor, {achievement_seeded} prestasi, {target_seeded} target PTN disimpan");
    if target_skipped_no_catalog > 0 {
        println!(
            "  ({target_skipped_no_catalog} target PTN dilewati — katalog program studi kosong/tidak ditemukan. \
             Jalankan `cargo run --bin import_ptn_catalog` lalu ulangi seed untuk melengkapi target.)"
        );
    }

    // ─── Kaka Kelas (alumni benchmark) & Eligible per sekolah ────────────────────────
    // Dientri oleh PIC sekolah sendiri (lewat SchoolRationalizationService, sama seperti
    // form "Tambah Data Alumni" / "Eligible" di UI) — bukan array contoh yang ditulis
    // atas nama siswa manapun, murni catatan riwayat sekolah untuk benchmark internal.
    type AlumniSeed = (&'static str, i32, Track, &'static str, &'static str, f64);
    let alumni_seeds: Vec<AlumniSeed> = vec![
        ("Naya Anggraini", 2024, Track::Snbp, "Universitas Indonesia", "Ilmu Komunikasi", 88.5),
        ("Rafi Ardiansyah", 2024, Track::Snbt, "Institut Teknologi Bandung", "Teknik Informatika", 705.0),
        ("Salsabila Putri", 2023, Track::Snbp, "Universitas Gadjah Mada", "Psikologi", 91.2),
    ];
    let eligibility_seeds: Vec<(i32, i32)> = vec![(2024, 42), (2025, 38)];

    let mut alumni_seeded = 0usize;
    let mut eligibility_seeded = 0usize;
    for pic in &school_pics {
        let pic_actor = AuthUser { user_id: pic.id, email: pic.email.clone(), role: Role::School };

        let existing_alumni = school_rationalization_service.list_alumni(&pic_actor).await.unwrap_or_default();
        for (name, year, track, ptn, prodi, score) in &alumni_seeds {
            if existing_alumni.iter().any(|a| a.alumni_name == *name) {
                continue;
            }
            match school_rationalization_service
                .add_alumni(&pic_actor, AddAlumniInput {
                    alumni_name: name.to_string(), graduation_year: *year, track: *track,
                    nama_ptn: ptn.to_string(), nama_prodi: prodi.to_string(), benchmark_score: *score,
                })
                .await
            {
                Ok(_) => alumni_seeded += 1,
                Err(e) => println!("skip alumni ({name} @ {}): {e}", pic.email),
            }
        }

        let existing_eligibility = school_rationalization_service.list_eligibility(&pic_actor).await.unwrap_or_default();
        for (year, count) in &eligibility_seeds {
            if existing_eligibility.iter().any(|e| e.year == *year) {
                continue;
            }
            match school_rationalization_service.upsert_eligibility(&pic_actor, *year, *count).await {
                Ok(_) => eligibility_seeded += 1,
                Err(e) => println!("skip eligibility ({year} @ {}): {e}", pic.email),
            }
        }
    }
    println!("rasionalisasi sekolah: {alumni_seeded} data alumni, {eligibility_seeded} data eligible disimpan (per sekolah)");

    // ─── SNBP roster ("Daftar Siswa"): konsultan + status penerimaan per siswa ──────
    // Sama seperti alumni/eligible di atas — ini input administratif PIC sekolah lewat
    // SchoolRationalizationService (bukan raw SQL), bukan angka yang diklaim sebagai
    // hasil sistem. "Pilihan 1/2" TIDAK di-seed di sini karena selalu diturunkan live
    // dari target PTN nyata yang sudah diisi di atas.
    let konsultan_names = ["Ibu Ratna Dewi", "Bapak Yusuf Hidayat", "Ibu Siti Aminah"];
    let mut participation_seeded = 0usize;
    for (idx, student) in students.iter().enumerate() {
        let Some(school_id) = student.school_id else { continue };
        let Some(pic) = school_pics.iter().find(|p| p.school_id == Some(school_id)) else { continue };
        let pic_actor = AuthUser { user_id: pic.id, email: pic.email.clone(), role: Role::School };

        if school_rationalization_service.snbp_roster(&pic_actor).await.is_ok_and(|roster| {
            roster.iter().any(|r| r.student_id == student.id && !r.konsultan.is_empty())
        }) {
            continue;
        }

        let konsultan = konsultan_names[idx % konsultan_names.len()];
        // Sebagian besar masih "belum" (real — belum ada pengumuman SNBP), satu
        // contoh sudah "diterima-snbp" supaya status lain juga terlihat saat demo.
        let status = if idx == 0 { SnbpStatus::DiterimaSnbp } else { SnbpStatus::Belum };
        match school_rationalization_service
            .upsert_snbp_participation(
                &pic_actor,
                student.id,
                SnbpParticipationInput { year: Utc::now().year(), konsultan: konsultan.to_string(), status, aktif: true },
            )
            .await
        {
            Ok(_) => participation_seeded += 1,
            Err(e) => println!("skip roster snbp ({}): {e}", student.name),
        }
    }
    println!("rasionalisasi sekolah: {participation_seeded} data roster SNBP (konsultan/status) disimpan");

    // ─── Question bank: admin-authored, auto-approved, spread across 8 subjects ─────
    // (subject, topic, subtopic, difficulty, stimulus, question_text, options, correct, explanation)
    type QSeed = (QuestionType, &'static str, &'static str, &'static str, Difficulty, &'static str, &'static str, Vec<&'static str>, &'static str, &'static str);
    let approved_seeds: Vec<QSeed> = vec![
        (QuestionType::MultipleChoice, "Matematika", "Aljabar", "Persamaan Kuadrat", Difficulty::Medium,
            "Sebuah perusahaan startup teknologi memiliki pendapatan bulanan yang mengikuti pola kuadrat.",
            "Jika pendapatan pada bulan ke-t dinyatakan dengan P(t) = -2t^2 + 12t + 10 (juta rupiah), pada bulan ke berapa pendapatan maksimum tercapai?",
            vec!["Bulan ke-2", "Bulan ke-3", "Bulan ke-4", "Bulan ke-5", "Bulan ke-6"], "Bulan ke-3",
            "Puncak parabola: t = -b/2a = -12/(2*-2) = 3."),
        (QuestionType::MultipleChoice, "Matematika", "Geometri", "Bangun Ruang", Difficulty::Medium,
            "", "Sebuah tabung memiliki jari-jari 7 cm dan tinggi 10 cm. Berapa volume tabung tersebut? (gunakan pi = 22/7)",
            vec!["1.480 cm3", "1.540 cm3", "1.600 cm3", "1.680 cm3", "1.720 cm3"], "1.540 cm3",
            "V = pi r^2 t = 22/7 * 49 * 10 = 1.540 cm3."),
        (QuestionType::MultipleChoice, "Matematika", "Trigonometri", "Identitas Trigonometri", Difficulty::Hard,
            "", "Nilai dari sin(75 derajat) adalah...",
            vec!["(sqrt6 - sqrt2)/4", "(sqrt6 + sqrt2)/4", "sqrt2/2", "sqrt3/2", "1/2"], "(sqrt6 + sqrt2)/4",
            "sin(75) = sin(45+30) = sin45 cos30 + cos45 sin30 = (sqrt6+sqrt2)/4."),
        (QuestionType::MultipleChoice, "Fisika", "Mekanika", "Gerak Parabola", Difficulty::Hard,
            "Seorang atlet melempar bola dengan kecepatan awal 10 m/s pada sudut 45 derajat. g = 10 m/s^2.",
            "Berapakah jarak mendatar maksimum yang dicapai bola tersebut?",
            vec!["5 meter", "8 meter", "10 meter", "12 meter", "15 meter"], "10 meter",
            "R = v0^2 sin(2*theta) / g = (100 * 1) / 10 = 10 meter."),
        (QuestionType::MultipleChoice, "Fisika", "Listrik", "Rangkaian Seri-Paralel", Difficulty::Medium,
            "", "Dua resistor 6 ohm dan 3 ohm disusun paralel, kemudian diseri dengan resistor 4 ohm. Berapa hambatan total rangkaian?",
            vec!["4 ohm", "5 ohm", "6 ohm", "8 ohm", "10 ohm"], "6 ohm",
            "Paralel: (6*3)/(6+3) = 2 ohm; total seri = 2 + 4 = 6 ohm."),
        (QuestionType::MultipleChoice, "Fisika", "Termodinamika", "Hukum Termodinamika I", Difficulty::Medium,
            "", "Suatu gas menyerap kalor 500 J dan melakukan usaha 200 J terhadap lingkungan. Berapa perubahan energi dalam gas tersebut?",
            vec!["200 J", "300 J", "500 J", "700 J", "-300 J"], "300 J",
            "dU = Q - W = 500 - 200 = 300 J."),
        (QuestionType::ShortAnswer, "Kimia", "Stoikiometri", "Konsep Mol", Difficulty::Easy,
            "", "Berapa gram NaCl (Mr = 58,5) yang diperlukan untuk membuat 500 mL larutan NaCl 0,2 M? (jawab angka saja)",
            vec![], "5.85",
            "mol = M x V = 0,2 x 0,5 = 0,1 mol; massa = mol x Mr = 0,1 x 58,5 = 5,85 gram."),
        (QuestionType::MultipleChoice, "Kimia", "Asam Basa", "pH Larutan", Difficulty::Medium,
            "", "Berapa pH larutan HCl 0,001 M?",
            vec!["1", "2", "3", "4", "11"], "3",
            "pH = -log[H+] = -log(10^-3) = 3."),
        (QuestionType::MultipleChoice, "Kimia", "Termokimia", "Entalpi Reaksi", Difficulty::Hard,
            "", "Reaksi pembentukan air melepaskan 286 kJ/mol. Jika 2 mol air terbentuk, berapa total kalor yang dilepaskan?",
            vec!["143 kJ", "286 kJ", "429 kJ", "572 kJ", "858 kJ"], "572 kJ",
            "Kalor total = 2 x 286 kJ = 572 kJ."),
        (QuestionType::MultipleChoice, "Biologi", "Genetika", "Persilangan Monohibrid", Difficulty::Medium,
            "Tanaman berbunga merah (MM) disilangkan dengan tanaman berbunga putih (mm).",
            "Berapa persentase keturunan F2 yang berbunga merah jika merah dominan terhadap putih?",
            vec!["25%", "50%", "75%", "100%", "0%"], "75%",
            "F1 semua Mm (merah). F2: 1 MM : 2 Mm : 1 mm, sehingga 75% berbunga merah (MM+Mm)."),
        (QuestionType::MultipleChoice, "Biologi", "Ekologi", "Rantai Makanan", Difficulty::Easy,
            "", "Dalam rantai makanan padi -> tikus -> ular -> elang, tikus berperan sebagai...",
            vec!["Produsen", "Konsumen tingkat I", "Konsumen tingkat II", "Konsumen tingkat III", "Dekomposer"], "Konsumen tingkat I",
            "Tikus memakan padi (produsen) sehingga menjadi konsumen tingkat I."),
        (QuestionType::MultipleChoice, "Biologi", "Sel", "Struktur Sel", Difficulty::Easy,
            "", "Organel sel yang berfungsi sebagai penghasil energi (ATP) utama pada sel eukariotik adalah...",
            vec!["Ribosom", "Mitokondria", "Lisosom", "Badan Golgi", "Retikulum Endoplasma"], "Mitokondria",
            "Mitokondria adalah tempat respirasi seluler yang menghasilkan ATP."),
        (QuestionType::MultipleChoice, "Bahasa Indonesia", "Membaca Kritis", "Ide Pokok", Difficulty::Easy,
            "Perubahan iklim menyebabkan cuaca ekstrem yang semakin sering terjadi di berbagai wilayah Indonesia, mulai dari banjir hingga kekeringan panjang.",
            "Ide pokok paragraf tersebut adalah...",
            vec!["Banjir di Indonesia", "Kekeringan panjang", "Dampak perubahan iklim terhadap cuaca ekstrem", "Wilayah Indonesia", "Cuaca di Indonesia"], "Dampak perubahan iklim terhadap cuaca ekstrem",
            "Kalimat utama membahas perubahan iklim yang menyebabkan cuaca ekstrem."),
        (QuestionType::MultipleChoice, "Bahasa Indonesia", "Kebahasaan", "Ejaan yang Disempurnakan", Difficulty::Medium,
            "", "Penulisan kata baku yang tepat di bawah ini adalah...",
            vec!["Aktifitas", "Praktek", "Nasehat", "Analisis", "Sistim"], "Analisis",
            "'Analisis' adalah bentuk baku, sedangkan 'aktifitas', 'praktek', 'nasehat', 'sistim' tidak baku."),
        (QuestionType::MultipleChoice, "Bahasa Indonesia", "Menulis", "Teks Argumentasi", Difficulty::Medium,
            "", "Struktur teks argumentasi yang tepat secara berurutan adalah...",
            vec!["Pernyataan pendapat - argumen - penegasan ulang", "Orientasi - komplikasi - resolusi", "Abstrak - orientasi - krisis", "Tesis - isi - koda", "Judul - isi - penutup"], "Pernyataan pendapat - argumen - penegasan ulang",
            "Teks argumentasi terdiri dari pernyataan pendapat, argumen pendukung, dan penegasan ulang pendapat."),
        (QuestionType::MultipleChoice, "Bahasa Inggris", "Reading", "Reading Comprehension", Difficulty::Medium,
            "Climate change is one of the most pressing issues of our time, affecting ecosystems and human societies worldwide.",
            "What is the main topic of the passage?",
            vec!["Ecosystems", "Human societies", "Climate change", "Pressing issues", "Worldwide effects"], "Climate change",
            "The passage centers on climate change as the main topic."),
        (QuestionType::MultipleChoice, "Bahasa Inggris", "Grammar", "Tenses", Difficulty::Easy,
            "", "She ___ to the library every Saturday.",
            vec!["go", "goes", "going", "gone", "went"], "goes",
            "Subject 'she' with simple present tense requires 'goes'."),
        (QuestionType::MultipleChoice, "Bahasa Inggris", "Vocabulary", "Synonym", Difficulty::Easy,
            "", "The word 'abundant' is closest in meaning to...",
            vec!["Scarce", "Plentiful", "Limited", "Fragile", "Hidden"], "Plentiful",
            "'Abundant' means existing in large quantities, synonymous with 'plentiful'."),
        (QuestionType::MultipleChoice, "Penalaran Umum", "Logika", "Silogisme", Difficulty::Easy,
            "", "Semua siswa rajin lulus ujian. Budi lulus ujian. Kesimpulan yang tepat adalah...",
            vec!["Budi siswa rajin", "Budi bukan siswa rajin", "Semua yang lulus ujian rajin", "Tidak dapat disimpulkan", "Budi tidak rajin"], "Tidak dapat disimpulkan",
            "Pernyataan awal tidak menyatakan bahwa hanya siswa rajin yang lulus, sehingga tidak bisa ditarik simpulan pasti tentang Budi."),
        (QuestionType::MultipleChoice, "Penalaran Umum", "Analogi", "Hubungan Kata", Difficulty::Medium,
            "", "DOKTER : RUMAH SAKIT = GURU : ...",
            vec!["Buku", "Sekolah", "Murid", "Papan Tulis", "Kelas"], "Sekolah",
            "Dokter bekerja di rumah sakit, guru bekerja di sekolah."),
        (QuestionType::MultipleChoice, "Penalaran Umum", "Deduksi", "Penalaran Deduktif", Difficulty::Medium,
            "", "Jika hari hujan maka jalan basah. Jalan tidak basah. Kesimpulan yang sah adalah...",
            vec!["Hari hujan", "Hari tidak hujan", "Jalan basah", "Tidak dapat disimpulkan", "Hari mendung"], "Hari tidak hujan",
            "Modus tollens: jika P maka Q, tidak Q, maka tidak P."),
        (QuestionType::MultipleChoice, "Penalaran Matematika", "Statistika", "Rata-rata", Difficulty::Medium,
            "", "Rata-rata nilai 8 siswa adalah 75. Jika ditambah 2 siswa baru, rata-rata menjadi 73. Berapa jumlah nilai 2 siswa baru?",
            vec!["120", "125", "130", "135", "140"], "130",
            "Total 8 siswa = 600. Total 10 siswa = 730. Jumlah 2 siswa baru = 730 - 600 = 130."),
        (QuestionType::MultipleChoice, "Penalaran Matematika", "Peluang", "Peluang Kejadian", Difficulty::Hard,
            "", "Sebuah dadu dilempar sekali. Berapa peluang muncul mata dadu genap atau lebih dari 4?",
            vec!["1/3", "1/2", "2/3", "5/6", "1/6"], "2/3",
            "Genap {2,4,6}, lebih dari 4 {5,6}. Gabungan {2,4,5,6} = 4 dari 6 = 2/3."),
        (QuestionType::MultipleChoice, "Penalaran Matematika", "Deret", "Deret Aritmatika", Difficulty::Medium,
            "", "Suku ke-10 dari deret aritmatika 3, 7, 11, 15, ... adalah...",
            vec!["37", "39", "41", "43", "45"], "39",
            "Un = a + (n-1)b = 3 + 9*4 = 39."),
    ];

    let mut approved_count = 0;
    for (qtype, subject, topic, subtopic, difficulty, stimulus, question_text, options, correct, explanation) in approved_seeds {
        let input = CreateQuestionInput {
            question_type: qtype, subject: subject.to_string(), topic: topic.to_string(), subtopic: subtopic.to_string(),
            difficulty, bloom_level: "C3 - Aplikasi".to_string(),
            stimulus: if stimulus.is_empty() { None } else { Some(stimulus.to_string()) },
            question_text: question_text.to_string(),
            options: if options.is_empty() { None } else { Some(options.into_iter().map(String::from).collect()) },
            correct_answer: correct.to_string(), explanation: explanation.to_string(),
            tags: vec![topic.to_lowercase()], time_limit: Some(120), submit_for_review: false,
            question_set_id: None,
        };
        match question_service.create(&admin_actor, input).await {
            Ok(q) => { approved_count += 1; println!("seeded approved question {} ({})", q.code, q.subject); }
            Err(e) => println!("skip question ({subject}/{topic}): {e}"),
        }
    }
    println!("total approved questions seeded: {approved_count}");

    // ─── Review-queue variety: content-authored questions in every status ───────────
    let review_input = |subject: &str, topic: &str, text: &str, correct: &str, submit: bool| CreateQuestionInput {
        question_type: QuestionType::ShortAnswer, subject: subject.to_string(), topic: topic.to_string(),
        subtopic: "General".to_string(), difficulty: Difficulty::Medium, bloom_level: "C3 - Aplikasi".to_string(),
        stimulus: None, question_text: text.to_string(), options: None, correct_answer: correct.to_string(),
        explanation: "Pembahasan menyusul setelah direview.".to_string(), tags: vec![topic.to_lowercase()],
        time_limit: Some(120), submit_for_review: submit, question_set_id: None,
    };

    if let Ok(q) = question_service.create(&content1_actor, review_input("Biologi", "Ekosistem", "Jelaskan pengertian rantai makanan dalam suatu ekosistem.", "interaksi makan dan dimakan antar organisme", true)).await {
        println!("seeded review-queue question {} (status: review)", q.code);
    }
    if let Ok(q) = question_service.create(&content1_actor, review_input("Fisika", "Optik", "Sebutkan bunyi hukum pemantulan cahaya.", "sudut datang sama dengan sudut pantul", true)).await {
        match question_service.review(&admin_actor, q.id, ReviewAction::Approve, None).await {
            Ok(q2) => println!("seeded question {} (status: {:?} via review)", q2.code, q2.status),
            Err(e) => println!("skip approve: {e}"),
        }
    }
    if let Ok(q) = question_service.create(&content1_actor, review_input("Kimia", "Elektrokimia", "Jelaskan perbedaan sel volta dan sel elektrolisis.", "sel volta menghasilkan listrik dari reaksi spontan, sel elektrolisis memakai listrik untuk reaksi tidak spontan", true)).await {
        match question_service.review(&admin_actor, q.id, ReviewAction::RequestRevision, Some("Tambahkan contoh reaksi pada masing-masing sel.".to_string())).await {
            Ok(q2) => println!("seeded question {} (status: {:?})", q2.code, q2.status),
            Err(e) => println!("skip revision: {e}"),
        }
    }
    if let Ok(q) = question_service.create(&content1_actor, review_input("Matematika", "Peluang Diskrit", "Berapa banyak cara menyusun 3 orang dari 5 orang dalam satu baris?", "60", true)).await {
        match question_service.review(&admin_actor, q.id, ReviewAction::Reject, Some("Soal duplikat dengan soal Penalaran Matematika yang sudah ada.".to_string())).await {
            Ok(q2) => println!("seeded question {} (status: {:?})", q2.code, q2.status),
            Err(e) => println!("skip reject: {e}"),
        }
    }
    if let Ok(q) = question_service.create(&content1_actor, review_input("Penalaran Umum", "Deret Gambar", "Deskripsikan pola pada deret gambar berikutnya (draft, belum diajukan).", "pola berulang setiap 3 gambar", false)).await {
        println!("seeded draft question {} (status: draft)", q.code);
    }

    // ─── Sample tryout/drilling session templates ────────────────────────────────────
    type SessionSeed = (&'static str, SessionType, i32, i32, Option<&'static str>, Option<&'static str>, Option<&'static str>, bool);
    let session_seeds: Vec<SessionSeed> = vec![
        ("Tryout Nasional SNBT #45", SessionType::Tryout, 120, 5, None, None, None, false),
        ("Latihan Harian Adaptif", SessionType::Drilling, 15, 3, None, None, None, false),
        ("Mini Tryout SNBT", SessionType::Mini, 45, 4, None, None, None, false),
        ("Drilling Matematika - Level Sedang", SessionType::Drilling, 20, 2, Some("Matematika"), None, Some("medium"), false),
        ("Drilling Biologi - Genetika & Ekologi", SessionType::Drilling, 20, 3, Some("Biologi"), None, None, false),
        ("Drilling Bahasa Indonesia - Pemahaman Teks", SessionType::Drilling, 15, 3, Some("Bahasa Indonesia"), None, None, false),
        ("Drilling Bahasa Inggris - Reading & Grammar", SessionType::Drilling, 15, 3, Some("Bahasa Inggris"), None, None, false),
    ];

    let mut sessions: HashMap<&'static str, Uuid> = HashMap::new();
    for (title, stype, duration, count, subject, topic, difficulty, premium) in session_seeds {
        let input = CreateSessionInput {
            title: title.to_string(), session_type: stype, duration_minutes: duration, question_count: count,
            subject_filter: subject.map(String::from), topic_filter: topic.map(String::from),
            difficulty_filter: difficulty.map(String::from), is_premium: premium,
            question_set_id: None,
            is_draft: false,
            is_elective: false,
            exam_track: ExamTrack::Snbt,
            school_type_scope: None,
        };
        let session = match tryout_service.create_session(&admin_actor, input).await {
            Ok(s) => { println!("seeded session '{}'", s.title); s }
            Err(_) => tryout_service
                .list_sessions(&admin_actor, None)
                .await?
                .into_iter()
                .find(|s| s.title == title)
                .expect("session should exist if create reported a conflict"),
        };
        sessions.insert(title, session.id);
    }

    // ─── Events across every lifecycle status ────────────────────────────────────────
    let today = Utc::now().date_naive();
    type EventSeed = (&'static str, EventType, &'static str, &'static str, i64, Option<i64>, Option<i32>, i32, &'static str, &'static str, EventStatus);
    let event_seeds: Vec<EventSeed> = vec![
        ("Tryout Nasional SNBT #45", EventType::Tryout, "Tryout Nasional SNBT #45", "Tryout simulasi SNBT full dengan soal terbaru dan ranking nasional.", -1, Some(2), Some(500), 50000, "Voucher belanja Rp 1jt, Rp 500rb, Rp 250rb", "Kelas 12", EventStatus::Ongoing),
        ("Drilling Matematika Intensif", EventType::Drilling, "Drilling Matematika - Level Sedang", "Program drilling penalaran matematika intensif.", 5, Some(15), None, 0, "", "Kelas 11-12", EventStatus::Upcoming),
        ("Mini Tryout Weekend #12", EventType::Mini, "Mini Tryout SNBT", "Mini tryout setiap akhir pekan untuk pemanasan.", 10, Some(11), None, 0, "", "Kelas 10-12", EventStatus::Draft),
        ("Tryout PTN Premium #44", EventType::Tryout, "Tryout Nasional SNBT #45", "Tryout premium dengan pembahasan lengkap dan analitik mendalam.", -20, Some(-17), Some(6000), 75000, "Laptop, Smartphone, Tablet", "Kelas 12", EventStatus::Completed),
        ("Drilling Biologi Genetika & Ekologi", EventType::Drilling, "Drilling Biologi - Genetika & Ekologi", "Drilling fokus Genetika dan Ekologi untuk SNBT.", -2, Some(6), None, 0, "", "Kelas 11-12", EventStatus::Ongoing),
        ("Mini Tryout Bahasa Edisi Lama", EventType::Mini, "Drilling Bahasa Inggris - Reading & Grammar", "Edisi lama mini tryout kebahasaan (arsip).", -40, Some(-38), Some(300), 25000, "", "Kelas 12", EventStatus::Archived),
    ];

    let existing_events: Vec<String> = event_service
        .list(&admin_actor)
        .await
        .map(|evs| evs.into_iter().map(|e| e.name).collect())
        .unwrap_or_default();

    for (name, etype, session_title, desc, start_off, end_off, max_p, price, prizes, target_class, status) in event_seeds {
        if existing_events.iter().any(|n| n == name) {
            println!("skip event (already seeded): {name}");
            continue;
        }
        let session_id = *sessions.get(session_title).expect("referenced session must exist");
        let start_date = today + Duration::days(start_off);
        let end_date = end_off.map(|d| today + Duration::days(d));
        let created = match event_service
            .create(&admin_actor, CreateEventInput {
                name: name.to_string(), event_type: etype, description: desc.to_string(), session_id,
                start_date, end_date, max_participants: max_p, price, prizes: prizes.to_string(),
                target_class: target_class.to_string(),
            })
            .await
        {
            Ok(e) => e,
            Err(e) => { println!("skip event ({name}): {e}"); continue; }
        };
        match event_service.set_status(&admin_actor, created.id, status).await {
            Ok(e) => println!("seeded event '{}' (status: {:?})", e.name, e.status),
            Err(e) => println!("skip status update for event ({name}): {e}"),
        }
    }

    // ─── Package catalogue (Manajemen Paket / landing page pricing) ─────────────────
    // Real admin-managed pricing content, adapted from the reference's INIT_PACKAGES.
    // discount is never seeded directly — always derived from original/sale price.
    // Trailing &'static str is the lucide preset icon name — kept in lockstep with the
    // emoji->icon_name mapping in migration 20250101000022_badge_package_icons.sql so a
    // freshly-seeded database and a migrated pre-existing one always agree.
    type PackageSeed = (&'static str, &'static str, i32, i32, &'static str, Vec<&'static str>, &'static str, &'static str, &'static str, &'static str, bool, i32, &'static str);
    let package_seeds: Vec<PackageSeed> = vec![
        ("5 Kuota Tryout SNBT", "TKA", 175_000, 105_000, "1 tahun",
            vec!["5x Tryout SNBT Full Simulasi", "Kuota berlaku 1 tahun sejak pembelian", "Pembahasan lengkap tiap soal", "Analisis skor otomatis"],
            "", "📘", "from-sky-200 to-blue-100", "#3b82f6", true, 1, "book"),
        ("12 Kuota Tryout SNBT & TPS", "Full", 350_000, 210_000, "1 tahun",
            vec!["12x Tryout Full Simulasi", "TPS + Literasi + Penalaran Mat.", "Kuota berlaku 1 tahun sejak pembelian", "Video pembahasan eksklusif"],
            "Terpopuler", "🎯", "from-violet-200 to-purple-100", "#7c3aed", true, 2, "target"),
        ("20 Kuota Tryout SNBT Full", "Premium", 580_000, 348_000, "1 tahun",
            vec!["20x Tryout Full Simulasi SNBT", "Semua subtes SNBT lengkap", "Kuota berlaku 1 tahun sejak pembelian", "Rasionalisasi PTN gratis"],
            "Best Value", "🏆", "from-amber-200 to-orange-100", "#f59e0b", true, 3, "trophy"),
        ("Akses Drilling Intensif", "Drilling", 195_000, 117_000, "1 tahun",
            vec!["Drilling tanpa batas 365 hari", "10.000+ soal per kategori", "Adaptive difficulty AI", "Progress tracking real-time"],
            "", "⚡", "from-emerald-200 to-teal-100", "#10b981", true, 4, "zap"),
        ("Paket Lengkap Elite", "Elite", 750_000, 450_000, "1 tahun",
            vec!["Tryout full simulasi tanpa batas", "Drilling intensif tanpa batas", "Rasionalisasi semua PTN", "Mentoring 1-on-1 / minggu"],
            "All-In-One", "💎", "from-rose-200 to-pink-100", "#ec4899", false, 5, "gem"),
    ];

    let existing_packages: Vec<String> = package_service
        .list(&admin_actor)
        .await
        .map(|ps| ps.into_iter().map(|p| p.name).collect())
        .unwrap_or_default();

    for (name, ptype, original_price, sale_price, validity, features, badge, emoji, gradient, accent_color, active, sort_order, icon_name) in package_seeds {
        if existing_packages.iter().any(|n| n == name) {
            println!("skip package (already seeded): {name}");
            continue;
        }
        match package_service
            .create(&admin_actor, CreatePackageInput {
                name: name.to_string(), package_type: ptype.to_string(), original_price, sale_price,
                validity: validity.to_string(), features: features.into_iter().map(String::from).collect(),
                badge: badge.to_string(), emoji: emoji.to_string(),
                icon_type: "preset".to_string(), icon_name: Some(icon_name.to_string()), icon_url: None,
                banner_url: None,
                gradient: gradient.to_string(),
                accent_color: accent_color.to_string(), active, sort_order,
                elective_pick_count: 0,
                exam_track: ExamTrack::Snbt,
            })
            .await
        {
            Ok(p) => println!("seeded package '{}'", p.name),
            Err(e) => println!("skip package ({name}): {e}"),
        }
    }

    // ─── Vouchers — real codes tied to the real package catalogue above ─────────────
    // used_count always stays 0 (no checkout/redemption flow exists yet); we do not
    // fabricate redemption numbers here the way the reference does.
    let all_packages = package_service.list(&admin_actor).await.unwrap_or_default();
    let pkg_id = |name: &str| all_packages.iter().find(|p| p.name == name).map(|p| p.id);

    if let Some(pid) = pkg_id("12 Kuota Tryout SNBT & TPS") {
        let today = Utc::now().date_naive();
        type VoucherSeed = (VoucherType, DiscountType, i32, i32, i64, &'static str, Option<&'static str>, Option<&'static str>, Option<i32>, i32);
        let voucher_seeds: Vec<VoucherSeed> = vec![
            (VoucherType::Promo, DiscountType::Percent, 50, 500, 60, "Campaign promo bulanan", None, None, None, 1),
            (VoucherType::Referral, DiscountType::Percent, 20, 999, 300, "Program afiliasi", None, Some("Budi Santoso"), Some(25_000), 1),
            (VoucherType::BulkSchool, DiscountType::Full, 0, 120, 180, "Batch voucher sekolah mitra", Some("SMA Negeri 1 Jakarta"), None, None, 5),
        ];
        let existing_voucher_notes: Vec<String> = voucher_service
            .list(&admin_actor)
            .await
            .map(|vs| vs.into_iter().map(|v| v.note).collect())
            .unwrap_or_default();
        for (vtype, dtype, dvalue, max_uses, expiry_days, note, school_name, referrer_name, referrer_commission, quantity) in voucher_seeds {
            if existing_voucher_notes.iter().any(|n| n == note) {
                println!("skip voucher (already seeded): {note}");
                continue;
            }
            match voucher_service
                .create(&admin_actor, CreateVoucherInput {
                    voucher_type: vtype, package_id: pid, discount_type: dtype, discount_value: dvalue,
                    max_uses, expires_at: today + Duration::days(expiry_days), note: note.to_string(),
                    school_name: school_name.map(String::from), referrer_name: referrer_name.map(String::from),
                    referrer_commission, quantity,
                })
                .await
            {
                Ok(vs) => println!("seeded {} voucher(s) for '{note}'", vs.len()),
                Err(e) => println!("skip voucher ({note}): {e}"),
            }
        }
    }

    // ─── Subject taxonomy (Kategori & Mata Uji: SNBT/TKA-IPA/TKA-IPS/AKM) ────────────
    // Purely additive catalogue — never renames or touches any existing
    // `questions.subject` value. The 8 legacy subject names already used by the
    // question seeds below (Penalaran Umum, Penalaran Matematika, Matematika, Fisika,
    // Kimia, Biologi, Bahasa Indonesia, Bahasa Inggris) are deliberately included
    // verbatim here (spread across SNBT/TKA-IPA) so those questions stay valid/visible
    // once dropdowns switch to reading from this catalogue instead of a hardcoded array.
    type TaxonomySubjectSeed = (&'static str, &'static str);
    type TaxonomyCategorySeed = (&'static str, &'static str, Vec<TaxonomySubjectSeed>);
    let taxonomy_seeds: Vec<TaxonomyCategorySeed> = vec![
        (
            "SNBT",
            "Ujian Tes Potensi Skolastik & Literasi untuk seleksi masuk PTN (UTBK)",
            vec![
                ("Penalaran Umum", "PU"),
                ("Pemahaman Bacaan dan Menulis", "PBM"),
                ("Pengetahuan dan Pemahaman Umum", "PPU"),
                ("Pengetahuan Kuantitatif", "PK"),
                ("Literasi Bahasa Indonesia", "LBI"),
                ("Literasi Bahasa Inggris", "LBE"),
                ("Penalaran Matematika", "PM"),
            ],
        ),
        (
            "TKA-IPA",
            "Tes Kemampuan Akademik jenjang SMA/SMK — jalur IPA",
            vec![
                ("Matematika", "MTK"),
                ("Bahasa Indonesia", "BIN"),
                ("Bahasa Inggris", "BIG"),
                ("Fisika", "FIS"),
                ("Kimia", "KIM"),
                ("Biologi", "BIO"),
            ],
        ),
        (
            "TKA-IPS",
            "Tes Kemampuan Akademik jenjang SMA/SMK — jalur IPS",
            vec![
                ("Matematika", "MTK"),
                ("Bahasa Indonesia", "BIN"),
                ("Bahasa Inggris", "BIG"),
                ("Ekonomi", "EKO"),
                ("Sosiologi", "SOS"),
                ("Geografi", "GEO"),
                ("Sejarah", "SEJ"),
            ],
        ),
        (
            "AKM",
            "Asesmen Kompetensi Minimum — literasi & numerasi",
            vec![("Literasi Membaca", "LM"), ("Numerasi", "NUM")],
        ),
    ];

    let existing_taxonomy = taxonomy_service.list().await.unwrap_or_default();
    for (idx, (cat_name, cat_desc, subjects)) in taxonomy_seeds.into_iter().enumerate() {
        let existing_category = existing_taxonomy.iter().find(|c| c.category.name == cat_name);
        let category_id = if let Some(existing_category) = existing_category {
            println!("skip taxonomy category (already seeded): {cat_name}");
            existing_category.category.id
        } else {
            match taxonomy_service
                .create_category(&admin_actor, cat_name.to_string(), cat_desc.to_string(), idx as i32)
                .await
            {
                Ok(c) => {
                    println!("seeded taxonomy category '{}'", c.name);
                    c.id
                }
                Err(e) => {
                    println!("skip taxonomy category ({cat_name}): {e}");
                    continue;
                }
            }
        };

        let existing_subject_names: Vec<String> = existing_category
            .map(|c| c.subjects.iter().map(|s| s.name.clone()).collect())
            .unwrap_or_default();
        for (sidx, (subj_name, subj_code)) in subjects.into_iter().enumerate() {
            if existing_subject_names.iter().any(|n| n == subj_name) {
                println!("skip taxonomy subject (already seeded): {subj_name}");
                continue;
            }
            match taxonomy_service
                .create_subject(&admin_actor, category_id, subj_name.to_string(), subj_code.to_string(), sidx as i32)
                .await
            {
                Ok(s) => println!("seeded taxonomy subject '{}' under '{cat_name}'", s.name),
                Err(e) => println!("skip taxonomy subject ({subj_name}): {e}"),
            }
        }
    }

    // ─── Gamifikasi: point rules, badges, challenges ─────────────────────────────────
    // earned_by / participants / completions are never seeded — they're always computed
    // live from the attempts simulated further below, so run this seeder twice and the
    // numbers still reflect exactly what's in the database, nothing frozen at seed time.
    type PointRuleSeed = (&'static str, &'static str, i32, &'static str);
    let point_rule_seeds: Vec<PointRuleSeed> = vec![
        ("Jawab Benar — Pilihan Ganda", "Soal", 10, "✅"),
        ("Selesaikan Full Tryout", "Sesi", 100, "🏁"),
        ("Selesaikan Drilling", "Sesi", 30, "🏁"),
        ("Login Harian", "Aktivitas", 5, "📅"),
        ("Skor Sempurna (100%)", "Prestasi", 300, "⭐"),
    ];
    let existing_rules: Vec<String> = gamification_service.list_point_rules(&admin_actor).await.map(|rs| rs.into_iter().map(|r| r.action).collect()).unwrap_or_default();
    for (idx, (action, category, base_points, icon)) in point_rule_seeds.into_iter().enumerate() {
        if existing_rules.iter().any(|a| a == action) { continue; }
        match gamification_service
            .create_point_rule(&admin_actor, CreatePointRuleInput { action: action.to_string(), category: category.to_string(), base_points, multiplier: 1.0, enabled: true, icon: icon.to_string(), sort_order: idx as i32 })
            .await
        {
            Ok(r) => println!("seeded point rule '{}'", r.action),
            Err(e) => println!("skip point rule ({action}): {e}"),
        }
    }

    // Trailing &'static str is the lucide preset icon name — kept in lockstep with the
    // emoji->icon_name mapping in migration 20250101000022_badge_package_icons.sql so a
    // freshly-seeded database and a migrated pre-existing one always agree.
    type BadgeSeed = (&'static str, &'static str, &'static str, Rarity, BadgeConditionType, i32, &'static str);
    let badge_seeds: Vec<BadgeSeed> = vec![
        ("🔥", "7 Day Streak", "Latihan 7 hari berturut-turut tanpa jeda", Rarity::Common, BadgeConditionType::StreakGte, 7, "flame"),
        ("📚", "Soal Addict", "Menjawab 200 soal", Rarity::Common, BadgeConditionType::QuestionsGte, 200, "book-open"),
        ("🏆", "Marathon Runner", "Selesaikan 5 tryout penuh", Rarity::Epic, BadgeConditionType::TryoutGte, 5, "trophy"),
        ("👑", "Top 5 Nasional", "Masuk 5 besar leaderboard nasional", Rarity::Epic, BadgeConditionType::RankLte, 5, "crown"),
        ("⭐", "Skor Tinggi", "Meraih skor tryout ≥ 800", Rarity::Legendary, BadgeConditionType::ScoreGte, 800, "star"),
    ];
    let existing_badges: Vec<String> = gamification_service.list_badges(&admin_actor).await.map(|bs| bs.into_iter().map(|b| b.badge.name).collect()).unwrap_or_default();
    for (emoji, name, description, rarity, condition_type, condition_value, icon_name) in badge_seeds {
        if existing_badges.iter().any(|n| n == name) { continue; }
        match gamification_service
            .create_badge(&admin_actor, CreateBadgeInput {
                emoji: emoji.to_string(), icon_type: "preset".to_string(), icon_name: Some(icon_name.to_string()), icon_url: None,
                name: name.to_string(), description: description.to_string(), rarity, condition_type, condition_value, active: true,
            })
            .await
        {
            Ok(b) => println!("seeded badge '{}' (earned_by dihitung live: {})", b.badge.name, b.earned_by),
            Err(e) => println!("skip badge ({name}): {e}"),
        }
    }

    let today = Utc::now().date_naive();
    type ChallengeSeed = (&'static str, ChallengeType, i64, i64, i32, i32, &'static str, &'static str);
    let challenge_seeds: Vec<ChallengeSeed> = vec![
        ("Tryout Marathon Bulan Ini", ChallengeType::MostSolved, -10, 20, 2, 500, "🏅", "Selesaikan minimal 2 tryout/drilling bulan ini dan raih hadiah."),
        ("Speed Challenge Minggu Ini", ChallengeType::Speed, -3, 4, 10, 200, "⚡", "Jawab minimal 10 soal dalam satu sesi minggu ini."),
        ("Skor Tertinggi Kuartal Ini", ChallengeType::HighestScore, -30, 30, 700, 300, "🎯", "Raih skor tryout tertinggi sepanjang kuartal ini."),
    ];
    let existing_challenges: Vec<String> = gamification_service.list_challenges(&admin_actor).await.map(|cs| cs.into_iter().map(|c| c.challenge.name).collect()).unwrap_or_default();
    for (name, ctype, start_off, end_off, target, reward_points, reward_badge, description) in challenge_seeds {
        if existing_challenges.iter().any(|n| n == name) { continue; }
        match gamification_service
            .create_challenge(&admin_actor, CreateChallengeInput {
                name: name.to_string(), challenge_type: ctype, start_date: today + Duration::days(start_off), end_date: today + Duration::days(end_off),
                target_value: target, reward_points, reward_badge: Some(reward_badge.to_string()), description: description.to_string(),
            })
            .await
        {
            Ok(c) => println!("seeded challenge '{}' (participants/completions dihitung live: {}/{})", c.challenge.name, c.participants, c.completions),
            Err(e) => println!("skip challenge ({name}): {e}"),
        }
    }

    // ─── Simulate real student attempts so Analytics/Overview have genuine numbers ───
    // Each (student index, session title, days ago, correct ratio %) tuple runs the full
    // start -> answer -> submit flow through TryoutService, exactly like a real student
    // would via the API. Timestamps are backdated afterwards (raw SQL) purely to spread
    // activity across the last two weeks — scores/accuracy are still fully computed by
    // the real grading logic, nothing about the results themselves is fabricated.
    type AttemptSeed = (usize, &'static str, i64, u64);
    let attempt_plan: Vec<AttemptSeed> = vec![
        (0, "Tryout Nasional SNBT #45", 0, 78),
        (1, "Tryout Nasional SNBT #45", 1, 65),
        (2, "Mini Tryout SNBT", 2, 82),
        (3, "Latihan Harian Adaptif", 2, 90),
        (4, "Tryout Nasional SNBT #45", 3, 71),
        (5, "Drilling Matematika - Level Sedang", 4, 60),
        (6, "Mini Tryout SNBT", 5, 55),
        (7, "Drilling Biologi - Genetika & Ekologi", 6, 88),
        (8, "Drilling Bahasa Indonesia - Pemahaman Teks", 7, 73),
        (9, "Drilling Bahasa Inggris - Reading & Grammar", 8, 68),
        (10, "Latihan Harian Adaptif", 9, 95),
        (11, "Mini Tryout SNBT", 10, 62),
        (0, "Drilling Matematika - Level Sedang", 11, 85),
        (2, "Tryout Nasional SNBT #45", 12, 70),
        (5, "Drilling Biologi - Genetika & Ekologi", 13, 77),
        (7, "Latihan Harian Adaptif", 1, 92),
        (9, "Tryout Nasional SNBT #45", 4, 58),
        (3, "Drilling Bahasa Inggris - Reading & Grammar", 6, 80),
    ];

    let mut seeded_attempts = 0;
    for (student_idx, session_title, days_ago, correct_pct) in attempt_plan {
        let Some(student) = students.get(student_idx) else { continue };
        let student_actor = AuthUser { user_id: student.id, email: student.email.clone(), role: Role::Student };
        let Some(&session_id) = sessions.get(session_title) else { continue };

        // Re-run safety: skip if this student already has an attempt on this session
        // (e.g. from a previous `cargo run --bin seed`) instead of piling up duplicates.
        let already_attempted = tryout_service
            .list_my_attempts(&student_actor)
            .await
            .map(|attempts| attempts.iter().any(|a| a.session_id == session_id))
            .unwrap_or(false);
        if already_attempted {
            continue;
        }

        match simulate_attempt(&tryout_service, &question_service, &pool, &student_actor, session_id, correct_pct, days_ago).await {
            Ok(()) => seeded_attempts += 1,
            Err(e) => println!("skip attempt ({} / {}): {e}", student.name, session_title),
        }
    }
    println!("total simulated attempts: {seeded_attempts}");

    println!("\nSeed complete. Demo logins:");
    println!("  admin:   admin@edvionptn.id        / admin123");
    println!("  school:  sekolah@sman1.sch.id        / sekolah123");
    println!("  student: siswa@student.com           / siswa123");
    println!("  content: konten@edvion.id            / konten123");
    println!("  (+ {} more students, 3 more schools, 1 more content account — see seed.rs)", students.len().saturating_sub(1));

    Ok(())
}

/// Runs a real start -> answer -> submit cycle for one student on one session (through
/// TryoutService, same as the HTTP API would), then backdates the attempt's timestamps
/// so it lands `days_ago` days in the past. `correct_pct` controls roughly what fraction
/// of answers are graded correct — this only decides which answer text we submit, the
/// actual grading/scoring is 100% the real `TryoutService::submit_attempt` logic.
async fn simulate_attempt(
    tryout_service: &TryoutService,
    question_service: &QuestionService,
    pool: &sqlx::PgPool,
    student_actor: &AuthUser,
    session_id: Uuid,
    correct_pct: u64,
    days_ago: i64,
) -> anyhow::Result<()> {
    let started = tryout_service
        .start_attempt(student_actor, session_id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    for (idx, playable) in started.questions.iter().enumerate() {
        let full: Question = question_service.get(playable.id).await.map_err(|e| anyhow::anyhow!("{e}"))?;
        let seed = mix(session_id, student_actor.user_id, idx as u64);
        let answer_correct = (seed % 100) < correct_pct;
        let answer_text = if answer_correct {
            full.correct_answer.clone()
        } else {
            wrong_answer(&full)
        };
        tryout_service
            .save_answer(student_actor, started.attempt.id, playable.id, Some(answer_text), false)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
    }

    let time_used = (started.attempt.duration_minutes * 60) / 2;
    tryout_service
        .submit_attempt(student_actor, started.attempt.id, time_used)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Backdate — purely cosmetic for the demo trend chart, scores already computed above.
    let submitted_at = Utc::now() - Duration::days(days_ago);
    let started_at = submitted_at - Duration::minutes(time_used as i64 / 60 + 5);
    sqlx::query("UPDATE attempts SET started_at = $2, submitted_at = $3 WHERE id = $1")
        .bind(started.attempt.id)
        .bind(started_at)
        .bind(submitted_at)
        .execute(pool)
        .await?;

    Ok(())
}

fn mix(a: Uuid, b: Uuid, idx: u64) -> u64 {
    let a_bytes = a.as_u128() as u64;
    let b_bytes = b.as_u128() as u64;
    a_bytes
        .wrapping_mul(6364136223846793005)
        .wrapping_add(b_bytes.wrapping_mul(2654435761))
        .wrapping_add(idx.wrapping_mul(40503))
}

fn wrong_answer(q: &Question) -> String {
    if let Some(opts) = &q.options {
        if let Some(wrong) = opts.iter().find(|o| o.as_str() != q.correct_answer.as_str()) {
            return wrong.clone();
        }
    }
    "999999".to_string()
}

async fn register_or_skip(
    auth: &AuthService,
    name: &str,
    email: &str,
    password: &str,
    role: Role,
    school_id: Option<Uuid>,
) -> anyhow::Result<User> {
    match auth
        .register(RegisterInput { name: name.to_string(), email: email.to_string(), password: password.to_string(), role, school_id })
        .await
    {
        Ok(res) => {
            println!("created {role:?} account: {email}");
            Ok(res.user)
        }
        Err(_) => {
            // Already exists (most likely a `Conflict` from a previous run). Try logging in
            // with the password this script expects first — the common case, and it also
            // confirms the account is active. If that fails (e.g. the row was seeded by an
            // older version of this script with a different password, or was registered
            // through the real app under the same email), don't abort the whole run: fetch
            // the row directly and force its password back to the one this script documents,
            // so the "Seed complete. Demo logins:" printout at the end always stays true.
            match auth.login(email, password).await {
                Ok(res) => Ok(res.user),
                Err(_) => match auth.find_by_email(email).await? {
                    Some(user) => {
                        println!("resetting password for existing account: {email} (didn't match expected seed password)");
                        Ok(auth.reset_password_for_seed(&user, password).await?)
                    }
                    None => Err(anyhow::anyhow!("could not create or find {email}: account does not exist and creation failed")),
                },
            }
        }
    }
}
