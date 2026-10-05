//! Institution master data use-cases — see `domain::institution` doc comment. Reading is
//! open to any authenticated role (students/schools need it to render institution profile
//! details in Rasionalisasi views); mutating is Admin-only, same convention as
//! `TaxonomyService`.

use std::sync::Arc;

use uuid::Uuid;

use crate::domain::institution::Institution;
use crate::domain::repository::{InstitutionRepository, InstitutionUpdate, NewInstitution};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct InstitutionService {
    institutions: Arc<dyn InstitutionRepository>,
}

impl InstitutionService {
    pub fn new(institutions: Arc<dyn InstitutionRepository>) -> Self {
        Self { institutions }
    }

    /// No role restriction — any authenticated user can read institution profiles.
    pub async fn list(&self, search: Option<String>) -> AppResult<Vec<Institution>> {
        self.institutions.list(search).await
    }

    pub async fn get(&self, id: Uuid) -> AppResult<Institution> {
        self.institutions
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("institusi {id} tidak ditemukan")))
    }

    /// Case-insensitive exact match on `nama_ptn` — used by the frontend to resolve a
    /// `ptn_programs` row (which only carries the institution's name as plain text) to its
    /// full profile. Returns `None` (not an error) when no research data exists yet for
    /// that institution, so callers can render an honest "belum ada data" state.
    pub async fn find_by_nama_ptn(&self, nama_ptn: &str) -> AppResult<Option<Institution>> {
        self.institutions.find_by_nama_ptn(nama_ptn).await
    }

    pub async fn create(&self, actor: &AuthUser, input: NewInstitution) -> AppResult<Institution> {
        actor.require_role(&[Role::Admin])?;
        validate_input(&input.nama_ptn, &input.singkatan)?;
        self.institutions.upsert_by_nama_ptn(input).await
    }

    pub async fn update(&self, actor: &AuthUser, id: Uuid, update: InstitutionUpdate) -> AppResult<Institution> {
        actor.require_role(&[Role::Admin])?;
        if update.singkatan.trim().is_empty() {
            return Err(AppError::Validation("singkatan wajib diisi".into()));
        }
        self.institutions
            .update(id, update)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("institusi {id} tidak ditemukan")))
    }

    pub async fn delete(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        let deleted = self.institutions.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("institusi {id} tidak ditemukan")));
        }
        Ok(())
    }
}

fn validate_input(nama_ptn: &str, singkatan: &str) -> AppResult<()> {
    if nama_ptn.trim().is_empty() {
        return Err(AppError::Validation("nama PTN wajib diisi".into()));
    }
    if singkatan.trim().is_empty() {
        return Err(AppError::Validation("singkatan wajib diisi".into()));
    }
    Ok(())
}
