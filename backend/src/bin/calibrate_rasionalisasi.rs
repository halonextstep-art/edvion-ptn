//! One-off diagnostic: bandingkan `chance_percent` prediksi mesin Rasionalisasi SNBP
//! terhadap hasil penerimaan NYATA yang sudah tercatat di `snbp_participation.status` —
//! langkah pertama yang direkomendasikan sebelum menyetel konstanta `compute_chance`/
//! `compute_chance_snbp` lebih jauh lagi (lihat perubahan `average_rapor_for_program` dan
//! `competition_adjustment` piecewise di `domain::rationalization`, yang sebelumnya
//! disetel berdasarkan analisis katalog saja, belum divalidasi ke hasil sungguhan).
//! Read-only — tidak pernah menulis ke database.
//!
//! Keterbatasan (didokumentasikan terang-terangan, bukan disembunyikan):
//! - Memakai data rapor/target/katalog TERKINI siswa, bukan snapshot persis saat mereka
//!   mendaftar SNBP dulu — nilai rapor jarang berubah setelah kejadian, tapi katalog PTN
//!   BISA diedit admin (sudah ada CRUD-nya), jadi hasil kalibrasi hari ini belum tentu
//!   persis mencerminkan apa yang mesin akan katakan waktu itu.
//! - `snbp_participation.status` cuma satu flag per siswa, bukan per target — skrip ini
//!   mengasumsikan itu mencerminkan target PRIORITAS UTAMA (`Priority::Utama`) siswa,
//!   karena itu yang sebenarnya dievaluasi SNBP duluan. Siswa tanpa target utama, atau
//!   yang program utamanya tidak punya data resmi terverifikasi, dilewati (dikecualikan,
//!   bukan dihitung sebagai diterima maupun ditolak).
//! - Status "diterima-snbt" dihitung sebagai "TIDAK diterima via SNBP" untuk kalibrasi
//!   ini, karena yang diuji akurasinya adalah model SNBP secara spesifik.
//!
//! Jalankan dengan: `cargo run --bin calibrate_rasionalisasi`
//! Opsional: beri path untuk juga menulis rincian per-siswa ke CSV, misalnya
//! `cargo run --bin calibrate_rasionalisasi -- calibration_report.csv`

use uuid::Uuid;

use edvionptn_backend::config::Config;
use edvionptn_backend::domain::rationalization::{
    average_rapor_for_program, compute_chance_snbp, index_prestasi, ChanceTier, Priority, Track,
};
use edvionptn_backend::domain::repository::{
    AchievementRepository, PtnProgramRepository, PtnTargetRepository, RaporScoreRepository,
};
use edvionptn_backend::infrastructure::db;
use edvionptn_backend::infrastructure::repositories::{
    PostgresAchievementRepository, PostgresPtnProgramRepository, PostgresPtnTargetRepository,
    PostgresRaporScoreRepository,
};

#[derive(Debug, sqlx::FromRow)]
struct ParticipationRow {
    student_id: Uuid,
    status: String,
}

struct Sample {
    student_id: Uuid,
    chance_percent: f64,
    tier: ChanceTier,
    accepted: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let csv_out = std::env::args().nth(1);

    let config = Config::from_env()?;
    let pool = db::create_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;

    // Hanya siswa dengan hasil SNBP FINAL nyata — status 'belum' (masih menunggu) belum
    // punya sinyal apa pun, jadi dikecualikan alih-alih ditebak.
    let rows = sqlx::query_as::<_, ParticipationRow>(
        "SELECT student_id, status FROM snbp_participation WHERE status != 'belum'",
    )
    .fetch_all(&pool)
    .await?;

    if rows.is_empty() {
        println!("Belum ada data hasil SNBP nyata (status != 'belum') di snbp_participation — belum ada yang bisa dikalibrasi.");
        println!("Kalibrasi ini baru bermakna setelah PIC sekolah mengisi status penerimaan pasca-pengumuman SNBP di tab \"Daftar Siswa\".");
        return Ok(());
    }

    let targets = PostgresPtnTargetRepository::new(pool.clone());
    let programs = PostgresPtnProgramRepository::new(pool.clone());
    let rapor_repo = PostgresRaporScoreRepository::new(pool.clone());
    let achievements_repo = PostgresAchievementRepository::new(pool.clone());

    let mut samples: Vec<Sample> = Vec::new();
    let mut skipped_no_utama = 0usize;
    let mut skipped_no_official_stats = 0usize;
    let mut skipped_unknown_status = 0usize;

    for row in rows {
        let accepted = match row.status.as_str() {
            "diterima-snbp" => true,
            "diterima-snbt" | "tidak" => false,
            other => {
                tracing::warn!("status snbp_participation tidak dikenal: {other}, dilewati");
                skipped_unknown_status += 1;
                continue;
            }
        };

        let student_targets = targets.list_by_student(row.student_id).await?;
        let Some(utama) = student_targets.into_iter().find(|t| t.track == Track::Snbp && t.priority == Priority::Utama) else {
            skipped_no_utama += 1;
            continue;
        };

        let Some(program) = programs.find_by_id(utama.ptn_program_id).await? else {
            skipped_no_utama += 1;
            continue;
        };
        if !program.has_official_stats {
            skipped_no_official_stats += 1;
            continue;
        }

        let rapor = rapor_repo.list_by_student(row.student_id).await?;
        let achievements = achievements_repo.list_by_student(row.student_id).await?;
        let student_value = average_rapor_for_program(&rapor, &program.mapel_syarat).unwrap_or(0.0);
        let prestasi_bonus = index_prestasi(&achievements);
        let chance = compute_chance_snbp(student_value, program.pg_snbp, program.competition_ratio_snbp(), prestasi_bonus);

        samples.push(Sample {
            student_id: row.student_id,
            chance_percent: chance.chance_percent,
            tier: chance.tier,
            accepted,
        });
    }

    println!("\n=== Kalibrasi Model Rasionalisasi SNBP ===");
    println!("Total sampel dipakai                                : {}", samples.len());
    println!("Dilewati (tanpa target utama / program tak ketemu)  : {skipped_no_utama}");
    println!("Dilewati (program tanpa data resmi terverifikasi)   : {skipped_no_official_stats}");
    if skipped_unknown_status > 0 {
        println!("Dilewati (status tidak dikenal)                     : {skipped_unknown_status}");
    }

    if samples.is_empty() {
        println!("\nTidak ada sampel valid untuk dianalisis (semua siswa berstatus final tidak punya target utama dengan data resmi).");
        return Ok(());
    }

    // Brier score: rata-rata (prediksi/100 - hasil_asli)^2 — 0 = prediksi sempurna,
    // 0.25 = setara buruknya dengan menebak 50% terus-menerus, mendekati 1 = konsisten salah arah.
    let brier: f64 = samples
        .iter()
        .map(|s| {
            let p = s.chance_percent / 100.0;
            let o = if s.accepted { 1.0 } else { 0.0 };
            (p - o).powi(2)
        })
        .sum::<f64>()
        / samples.len() as f64;
    println!("\nBrier score (0=sempurna, 0.25=setara nebak 50% terus): {brier:.4}");

    println!("\n--- Per Tier (Ketat <35%, Moderat 35-64.9%, Aman >=65%) ---");
    for tier in [ChanceTier::Ketat, ChanceTier::Moderat, ChanceTier::Aman] {
        let bucket: Vec<&Sample> = samples.iter().filter(|s| s.tier == tier).collect();
        if bucket.is_empty() {
            println!("{:<8} n=0", tier.as_str());
            continue;
        }
        let n = bucket.len();
        let avg_predicted = bucket.iter().map(|s| s.chance_percent).sum::<f64>() / n as f64;
        let actual_rate = bucket.iter().filter(|s| s.accepted).count() as f64 / n as f64 * 100.0;
        println!(
            "{:<8} n={n:<5} rata2 prediksi={avg_predicted:>5.1}%  tingkat diterima sungguhan={actual_rate:>5.1}%",
            tier.as_str()
        );
    }

    println!("\n--- Per Rentang 10 Poin (potret lebih halus daripada 3 tier) ---");
    for lo in (0..100).step_by(10) {
        let hi = lo + 10;
        let bucket: Vec<&Sample> =
            samples.iter().filter(|s| s.chance_percent >= lo as f64 && s.chance_percent < hi as f64).collect();
        if bucket.is_empty() {
            continue;
        }
        let n = bucket.len();
        let actual_rate = bucket.iter().filter(|s| s.accepted).count() as f64 / n as f64 * 100.0;
        println!("{lo:>3}-{hi:<3}%  n={n:<5} tingkat diterima sungguhan={actual_rate:>5.1}%");
    }

    if let Some(path) = csv_out {
        let mut wtr = csv::Writer::from_path(&path)?;
        wtr.write_record(["student_id", "chance_percent", "tier", "accepted"])?;
        for s in &samples {
            wtr.write_record([
                s.student_id.to_string(),
                format!("{:.1}", s.chance_percent),
                s.tier.as_str().to_string(),
                s.accepted.to_string(),
            ])?;
        }
        wtr.flush()?;
        println!("\nDetail per-siswa ditulis ke {path}");
    }

    println!("\nCatatan: hasil di atas pakai data rapor/katalog TERKINI, bukan snapshot persis saat siswa mendaftar SNBP —");
    println!("cukup akurat kalau nilai rapor jarang diubah setelah kejadian, tapi bisa melenceng kalau katalog PTN sudah diedit sejak saat itu.");
    println!("Kalau n per tier/rentang masih kecil (di bawah puluhan), jangan buru-buru menyetel ulang konstanta dari angka ini saja —");
    println!("tunggu datanya terkumpul lebih banyak dulu (biasanya bertambah tiap tahun setelah pengumuman SNBP).");

    Ok(())
}
