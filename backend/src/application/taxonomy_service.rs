//! Subject taxonomy (Kategori & Mata Uji) CRUD use-cases. Reading the catalogue is open to
//! any authenticated role (content authors, admins, and eventually session-config UIs all
//! need it); mutating it is Admin-only, mirroring `PackageService`'s admin-only mutations.

use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::TaxonomyRepository;
use crate::domain::taxonomy::{CategoryWithSubjects, Subject, SubjectCategory};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct TaxonomyService {
    taxonomy: Arc<dyn TaxonomyRepository>,
}

impl TaxonomyService {
    pub fn new(taxonomy: Arc<dyn TaxonomyRepository>) -> Self {
        Self { taxonomy }
    }

    /// No role restriction — any authenticated user can read the catalogue.
    pub async fn list(&self) -> AppResult<Vec<CategoryWithSubjects>> {
        self.taxonomy.list_categories_with_subjects().await
    }

    pub async fn create_category(
        &self,
        actor: &AuthUser,
        name: String,
        description: String,
        sort_order: i32,
    ) -> AppResult<SubjectCategory> {
        actor.require_role(&[Role::Admin])?;
        validate_name(&name, "nama kategori")?;
        self.taxonomy.create_category(name, description, sort_order).await
    }

    pub async fn update_category(
        &self,
        actor: &AuthUser,
        id: Uuid,
        name: String,
        description: String,
        sort_order: i32,
    ) -> AppResult<SubjectCategory> {
        actor.require_role(&[Role::Admin])?;
        validate_name(&name, "nama kategori")?;
        self.taxonomy
            .update_category(id, name, description, sort_order)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("kategori {id} tidak ditemukan")))
    }

    pub async fn delete_category(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        let deleted = self.taxonomy.delete_category(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("kategori {id} tidak ditemukan")));
        }
        Ok(())
    }

    pub async fn create_subject(
        &self,
        actor: &AuthUser,
        category_id: Uuid,
        name: String,
        code: String,
        sort_order: i32,
    ) -> AppResult<Subject> {
        actor.require_role(&[Role::Admin])?;
        validate_name(&name, "nama mata uji")?;
        self.taxonomy.create_subject(category_id, name, code, sort_order).await
    }

    pub async fn update_subject(
        &self,
        actor: &AuthUser,
        id: Uuid,
        name: String,
        code: String,
        sort_order: i32,
    ) -> AppResult<Subject> {
        actor.require_role(&[Role::Admin])?;
        validate_name(&name, "nama mata uji")?;
        self.taxonomy
            .update_subject(id, name, code, sort_order)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("mata uji {id} tidak ditemukan")))
    }

    pub async fn delete_subject(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        let deleted = self.taxonomy.delete_subject(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("mata uji {id} tidak ditemukan")));
        }
        Ok(())
    }
}

fn validate_name(name: &str, label: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::Validation(format!("{label} wajib diisi")));
    }
    Ok(())
}
