//! One-time (idempotent) importer for the real PTN admission-statistics catalog used by
//! the Rasionalisasi SNBT/SNBP engine. Source: "Hitungan Rasionalisasi.xlsx" (sheet "Data
//! Jurusan dan Passinggrade D") provided by the user — real, published daya
//! tampung/peminat/passing-grade figures for both the SNBP and SNBT admission tracks,
//! exported to `backend/data/ptn_programs.csv`.
//!
//! Upserts on the source's own `kode` column (see migration
//! `20250101000008_rationalization.sql`), so re-running this after fixing a figure in the
//! CSV updates the existing row instead of duplicating it.
//!
//! Run with: `cargo run --bin import_ptn_catalog -- backend/data/ptn_programs.csv`
//! (path defaults to `backend/data/ptn_programs.csv` if omitted).

use std::sync::Arc;

use edvionptn_backend::config::Config;
use edvionptn_backend::domain::repository::{NewPtnProgram, PtnProgramRepository};
use edvionptn_backend::infrastructure::db;
use edvionptn_backend::infrastructure::repositories::PostgresPtnProgramRepository;

#[derive(Debug, serde::Deserialize)]
struct CsvRow {
    kode: f64,
    nama_ptn: String,
    nama_prodi: String,
    provinsi: String,
    kota: String,
    singkatan: String,
    rumpun: String,
    mapel_syarat: String,
    daya_tampung_snbp: f64,
    peminat_snbp: f64,
    daya_tampung_snbt: f64,
    peminat_snbt: f64,
    pg_snbt: f64,
    pg_snbp: f64,
    jenjang: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let path = std::env::args().nth(1).unwrap_or_else(|| "backend/data/ptn_programs.csv".to_string());
    tracing::info!("reading PTN catalog from {path}");

    let config = Config::from_env()?;
    let pool = db::create_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;

    let repo: Arc<dyn PtnProgramRepository> = Arc::new(PostgresPtnProgramRepository::new(pool));

    let mut reader = csv::Reader::from_path(&path)?;
    let mut imported = 0usize;
    let mut skipped = 0usize;

    for result in reader.deserialize::<CsvRow>() {
        let row = match result {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("skipping malformed row: {e}");
                skipped += 1;
                continue;
            }
        };

        if row.nama_ptn.trim().is_empty() || row.nama_prodi.trim().is_empty() {
            skipped += 1;
            continue;
        }

        repo.upsert_by_kode(NewPtnProgram {
            kode: row.kode.round() as i64,
            nama_ptn: row.nama_ptn,
            nama_prodi: row.nama_prodi,
            provinsi: row.provinsi,
            kota: row.kota,
            singkatan: row.singkatan,
            rumpun: row.rumpun,
            mapel_syarat: row.mapel_syarat,
            daya_tampung_snbp: row.daya_tampung_snbp.round() as i32,
            peminat_snbp: row.peminat_snbp.round() as i32,
            daya_tampung_snbt: row.daya_tampung_snbt.round() as i32,
            peminat_snbt: row.peminat_snbt.round() as i32,
            pg_snbt: row.pg_snbt,
            pg_snbp: row.pg_snbp,
            jenjang: row.jenjang,
            // Every row in this official-statistics catalog carries a real, published
            // quota/applicant/passing-grade figure.
            has_official_stats: true,
        })
        .await?;

        imported += 1;
        if imported % 500 == 0 {
            tracing::info!("...{imported} program imported");
        }
    }

    tracing::info!("selesai: {imported} program diimpor/diperbarui, {skipped} baris dilewati");
    Ok(())
}
