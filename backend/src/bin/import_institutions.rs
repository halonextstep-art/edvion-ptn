//! One-time (idempotent) importer for institution master data — see `domain::institution`
//! doc comment. Source: web research across 137 institutions (every non-empty field traces
//! to a cited source in the `sumber` column; fields the research pass could not verify were
//! left blank rather than guessed), compiled into `backend/data/institutions.csv`.
//!
//! Upserts on `nama_ptn` (see migration `20250101000028_institutions.sql`), so re-running
//! this after refreshing the research data updates existing rows instead of duplicating
//! them. After upserting every institution, also backfills `ptn_programs.institution_id`
//! by matching `ptn_programs.nama_ptn` to the freshly-upserted institution (case-
//! insensitive exact match) — this is a one-shot backfill, not a live FK the DB enforces,
//! so it's redone every run to pick up newly-imported institutions.
//!
//! Run with: `cargo run --bin import_institutions -- backend/data/institutions.csv`
//! (path defaults to `backend/data/institutions.csv` if omitted).

use edvionptn_backend::config::Config;
use edvionptn_backend::domain::repository::{InstitutionRepository, NewInstitution};
use edvionptn_backend::infrastructure::db;
use edvionptn_backend::infrastructure::repositories::PostgresInstitutionRepository;

#[derive(Debug, serde::Deserialize)]
struct CsvRow {
    nama_ptn: String,
    singkatan: String,
    #[serde(default)]
    website: Option<String>,
    #[serde(default)]
    alamat: Option<String>,
    #[serde(default)]
    kota: Option<String>,
    #[serde(default)]
    provinsi: Option<String>,
    #[serde(default)]
    tahun_berdiri: Option<i32>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    akreditasi: Option<String>,
    #[serde(default)]
    logo_url: Option<String>,
    #[serde(default)]
    sumber: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let path = std::env::args().nth(1).unwrap_or_else(|| "backend/data/institutions.csv".to_string());
    tracing::info!("reading institution master data from {path}");

    let config = Config::from_env()?;
    let pool = db::create_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;

    let repo = PostgresInstitutionRepository::new(pool.clone());

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

        if row.nama_ptn.trim().is_empty() || row.singkatan.trim().is_empty() {
            skipped += 1;
            continue;
        }

        repo.upsert_by_nama_ptn(NewInstitution {
            nama_ptn: row.nama_ptn,
            singkatan: row.singkatan,
            website: non_empty(row.website),
            alamat: non_empty(row.alamat),
            kota: non_empty(row.kota),
            provinsi: non_empty(row.provinsi),
            tahun_berdiri: row.tahun_berdiri,
            status: non_empty(row.status),
            akreditasi: non_empty(row.akreditasi),
            logo_url: non_empty(row.logo_url),
            sumber: non_empty(row.sumber),
        })
        .await?;

        imported += 1;
    }

    tracing::info!("selesai: {imported} institusi diimpor/diperbarui, {skipped} baris dilewati");

    // Backfill ptn_programs.institution_id by matching nama_ptn (case-insensitive exact
    // match). This is a plain SQL UPDATE...FROM rather than routed through the repository
    // layer since it's a one-shot cross-table backfill specific to this importer, not a
    // reusable application use-case.
    let backfilled = sqlx::query(
        r#"UPDATE ptn_programs p
           SET institution_id = i.id
           FROM institutions i
           WHERE lower(p.nama_ptn) = lower(i.nama_ptn)
             AND (p.institution_id IS DISTINCT FROM i.id)"#,
    )
    .execute(&pool)
    .await?;
    tracing::info!("backfill institution_id: {} baris ptn_programs diperbarui", backfilled.rows_affected());

    Ok(())
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.trim().is_empty())
}
