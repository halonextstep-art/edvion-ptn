//! One-time, idempotent importer for program studi (prodi) discovered via web research to be
//! missing from the original `ptn_programs` catalog (see migration
//! `20250101000008_rationalization.sql` doc comment — that catalog's 4588 rows are a faithful
//! export of the official "Hitungan Rasionalisasi.xlsx" source and were confirmed NOT to be
//! missing any rows through a mis-import; any prodi absent from it genuinely never existed in
//! that source at all).
//!
//! Every row here was found via institution-by-institution research (official campus
//! websites / PDDikti) rather than the original official SNBP/SNBT admission-statistics
//! report, so none of them carry a verified daya-tampung/peminat/passing-grade triad. To
//! avoid the rationalization engine fabricating a chance estimate from absent numbers (see
//! `domain::rationalization::unavailable_chance` doc comment), every row imported by this
//! binary is unconditionally inserted with `has_official_stats = false` and all numeric stat
//! fields at `0` — the CSV intentionally has NO stat columns at all, only descriptive fields,
//! so there is no way to accidentally mark one of these rows as having real stats.
//!
//! Idempotent via an explicit existence check on the natural key (`nama_ptn`, `nama_prodi`)
//! — NOT `upsert_by_kode`, since these rows have no real-world `kode`; a synthetic one is
//! generated only the first time a given (nama_ptn, nama_prodi) pair is seen, starting above
//! the highest `kode` already in the table so it can never collide with a real imported code.
//!
//! Run with: `cargo run --bin import_prodi_additions -- backend/data/ptn_programs_additions.csv`
//! (path defaults to `backend/data/ptn_programs_additions.csv` if omitted). Safe to re-run
//! after appending more rows from a later research batch — already-imported pairs are
//! skipped, not duplicated or overwritten.

use edvionptn_backend::config::Config;
use edvionptn_backend::domain::repository::{NewPtnProgram, PtnProgramRepository};
use edvionptn_backend::infrastructure::db;
use edvionptn_backend::infrastructure::repositories::PostgresPtnProgramRepository;

#[derive(Debug, serde::Deserialize)]
struct CsvRow {
    nama_ptn: String,
    nama_prodi: String,
    provinsi: String,
    kota: String,
    singkatan: String,
    rumpun: String,
    #[serde(default)]
    mapel_syarat: String,
    jenjang: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let path = std::env::args().nth(1).unwrap_or_else(|| "backend/data/ptn_programs_additions.csv".to_string());
    tracing::info!("reading prodi additions from {path}");

    let config = Config::from_env()?;
    let pool = db::create_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;

    let repo = PostgresPtnProgramRepository::new(pool.clone());

    // Synthetic kode range starts safely above every real imported code (the official
    // catalog's codes top out at 931006 — see `backend/data/ptn_programs.csv`) so it can
    // never collide with a genuine SNPMB program code.
    let mut next_kode: i64 = sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(kode) FROM ptn_programs")
        .fetch_one(&pool)
        .await?
        .unwrap_or(9_000_000)
        .max(9_000_000)
        + 1;

    let mut reader = csv::Reader::from_path(&path)?;
    let mut imported = 0usize;
    let mut skipped_existing = 0usize;
    let mut skipped_malformed = 0usize;

    for result in reader.deserialize::<CsvRow>() {
        let row = match result {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("skipping malformed row: {e}");
                skipped_malformed += 1;
                continue;
            }
        };

        if row.nama_ptn.trim().is_empty() || row.nama_prodi.trim().is_empty() {
            skipped_malformed += 1;
            continue;
        }

        let already_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM ptn_programs WHERE nama_ptn = $1 AND nama_prodi = $2)",
        )
        .bind(&row.nama_ptn)
        .bind(&row.nama_prodi)
        .fetch_one(&pool)
        .await?;

        if already_exists {
            skipped_existing += 1;
            continue;
        }

        let kode = next_kode;
        next_kode += 1;

        repo.create(NewPtnProgram {
            kode,
            nama_ptn: row.nama_ptn,
            nama_prodi: row.nama_prodi,
            provinsi: row.provinsi,
            kota: row.kota,
            singkatan: row.singkatan,
            rumpun: row.rumpun,
            mapel_syarat: row.mapel_syarat,
            daya_tampung_snbp: 0,
            peminat_snbp: 0,
            daya_tampung_snbt: 0,
            peminat_snbt: 0,
            pg_snbt: 0.0,
            pg_snbp: 0.0,
            jenjang: row.jenjang,
            // Every row in this file was found via descriptive web research only, never a
            // verified official quota/applicant/passing-grade figure — see module doc.
            has_official_stats: false,
        })
        .await?;

        imported += 1;
    }

    tracing::info!(
        "selesai: {imported} prodi baru diimpor, {skipped_existing} dilewati (sudah ada), {skipped_malformed} baris rusak dilewati"
    );
    Ok(())
}
