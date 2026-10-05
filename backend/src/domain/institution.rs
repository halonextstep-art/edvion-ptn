//! Institution master data — profile-level information about each PTN (universitas/
//! institut/politeknik) that `ptn_programs` (per-program admission statistics) does not
//! carry: alamat, website, akreditasi institusi, tahun berdiri, status kelembagaan
//! (PTN-BH/BLU/Satker), logo. Keyed by `nama_ptn` (unique across the 137 institutions in
//! the SNBP/SNBT catalog), imported once via `cargo run --bin import_institutions` from
//! real, web-researched data (see `backend/data/institutions.csv` — every non-empty field
//! traces to a cited source in `sumber`; fields the research pass could not verify were
//! left blank rather than guessed, per the project's no-fabricated-data rule) and
//! maintainable afterward by Admin via CRUD, same pattern as the Package/Voucher/Taxonomy
//! catalogs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Institution {
    pub id: Uuid,
    pub nama_ptn: String,
    pub singkatan: String,
    pub website: Option<String>,
    pub alamat: Option<String>,
    pub kota: Option<String>,
    pub provinsi: Option<String>,
    pub tahun_berdiri: Option<i32>,
    pub status: Option<String>,
    pub akreditasi: Option<String>,
    pub logo_url: Option<String>,
    pub sumber: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
