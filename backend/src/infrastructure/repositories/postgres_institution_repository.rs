use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::institution::Institution;
use crate::domain::repository::{InstitutionRepository, InstitutionUpdate, NewInstitution};
use crate::error::AppResult;

pub struct PostgresInstitutionRepository {
    pool: PgPool,
}

impl PostgresInstitutionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct InstitutionRow {
    id: Uuid,
    nama_ptn: String,
    singkatan: String,
    website: Option<String>,
    alamat: Option<String>,
    kota: Option<String>,
    provinsi: Option<String>,
    tahun_berdiri: Option<i32>,
    status: Option<String>,
    akreditasi: Option<String>,
    logo_url: Option<String>,
    sumber: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<InstitutionRow> for Institution {
    fn from(r: InstitutionRow) -> Self {
        Institution {
            id: r.id,
            nama_ptn: r.nama_ptn,
            singkatan: r.singkatan,
            website: r.website,
            alamat: r.alamat,
            kota: r.kota,
            provinsi: r.provinsi,
            tahun_berdiri: r.tahun_berdiri,
            status: r.status,
            akreditasi: r.akreditasi,
            logo_url: r.logo_url,
            sumber: r.sumber,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

const SELECT_INSTITUTION: &str = "SELECT id, nama_ptn, singkatan, website, alamat, kota, provinsi, tahun_berdiri, status, akreditasi, logo_url, sumber, created_at, updated_at FROM institutions";

#[async_trait]
impl InstitutionRepository for PostgresInstitutionRepository {
    async fn upsert_by_nama_ptn(&self, input: NewInstitution) -> AppResult<Institution> {
        let id = Uuid::new_v4();
        let row = sqlx::query_as::<_, InstitutionRow>(&format!(
            r#"INSERT INTO institutions (id, nama_ptn, singkatan, website, alamat, kota, provinsi, tahun_berdiri, status, akreditasi, logo_url, sumber)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               ON CONFLICT (nama_ptn) DO UPDATE SET
                   singkatan = EXCLUDED.singkatan,
                   website = EXCLUDED.website,
                   alamat = EXCLUDED.alamat,
                   kota = EXCLUDED.kota,
                   provinsi = EXCLUDED.provinsi,
                   tahun_berdiri = EXCLUDED.tahun_berdiri,
                   status = EXCLUDED.status,
                   akreditasi = EXCLUDED.akreditasi,
                   logo_url = EXCLUDED.logo_url,
                   sumber = EXCLUDED.sumber,
                   updated_at = now()
               RETURNING {cols}"#,
            cols = "id, nama_ptn, singkatan, website, alamat, kota, provinsi, tahun_berdiri, status, akreditasi, logo_url, sumber, created_at, updated_at"
        ))
        .bind(id)
        .bind(&input.nama_ptn)
        .bind(&input.singkatan)
        .bind(&input.website)
        .bind(&input.alamat)
        .bind(&input.kota)
        .bind(&input.provinsi)
        .bind(input.tahun_berdiri)
        .bind(&input.status)
        .bind(&input.akreditasi)
        .bind(&input.logo_url)
        .bind(&input.sumber)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Institution>> {
        let row = sqlx::query_as::<_, InstitutionRow>(&format!("{SELECT_INSTITUTION} WHERE id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_nama_ptn(&self, nama_ptn: &str) -> AppResult<Option<Institution>> {
        let row = sqlx::query_as::<_, InstitutionRow>(&format!(
            "{SELECT_INSTITUTION} WHERE lower(nama_ptn) = lower($1)"
        ))
        .bind(nama_ptn)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn list(&self, search: Option<String>) -> AppResult<Vec<Institution>> {
        let pattern = search
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{s}%"));
        let rows = sqlx::query_as::<_, InstitutionRow>(&format!(
            "{SELECT_INSTITUTION}
             WHERE ($1::text IS NULL OR lower(nama_ptn) LIKE $1 OR lower(singkatan) LIKE $1)
             ORDER BY nama_ptn ASC"
        ))
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update(&self, id: Uuid, update: InstitutionUpdate) -> AppResult<Option<Institution>> {
        let row = sqlx::query_as::<_, InstitutionRow>(&format!(
            r#"UPDATE institutions SET
                   singkatan = $2, website = $3, alamat = $4, kota = $5, provinsi = $6,
                   tahun_berdiri = $7, status = $8, akreditasi = $9, logo_url = $10, sumber = $11,
                   updated_at = now()
               WHERE id = $1
               RETURNING {cols}"#,
            cols = "id, nama_ptn, singkatan, website, alamat, kota, provinsi, tahun_berdiri, status, akreditasi, logo_url, sumber, created_at, updated_at"
        ))
        .bind(id)
        .bind(&update.singkatan)
        .bind(&update.website)
        .bind(&update.alamat)
        .bind(&update.kota)
        .bind(&update.provinsi)
        .bind(update.tahun_berdiri)
        .bind(&update.status)
        .bind(&update.akreditasi)
        .bind(&update.logo_url)
        .bind(&update.sumber)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM institutions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
