//! School (Sekolah Mitra) CRUD use-cases. Admin-only — school/content/student accounts
//! are attached to a school via `User.school_id`, but only the central admin manages the
//! roster of partner schools itself.

use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::{NewSchool, SchoolRepository, SchoolUpdate};
use crate::domain::school::{PackageType, School, SchoolStatus, SchoolType};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct CreateSchoolInput {
    pub name: String,
    pub school_type: SchoolType,
    pub city: String,
    pub province: String,
    pub email: String,
    pub phone: String,
    pub package_type: PackageType,
    pub contact_person: String,
    pub status: SchoolStatus,
    pub revenue_share: i32,
    pub monthly_revenue: i64,
    /// See `NewSchool::linked_tenant_id` doc comment.
    pub linked_tenant_id: Option<Uuid>,
}

pub struct UpdateSchoolInput {
    pub name: String,
    pub school_type: SchoolType,
    pub city: String,
    pub province: String,
    pub email: String,
    pub phone: String,
    pub package_type: PackageType,
    pub contact_person: String,
    pub status: SchoolStatus,
    pub revenue_share: i32,
    pub monthly_revenue: i64,
}

pub struct SchoolService {
    schools: Arc<dyn SchoolRepository>,
}

impl SchoolService {
    pub fn new(schools: Arc<dyn SchoolRepository>) -> Self {
        Self { schools }
    }

    pub async fn create(&self, actor: &AuthUser, input: CreateSchoolInput) -> AppResult<School> {
        actor.require_role(&[Role::Admin])?;
        validate(&input.name, &input.email, &input.contact_person)?;
        let revenue_share = validate_revenue_share(input.revenue_share)?;
        self.schools
            .create(NewSchool {
                name: input.name,
                school_type: input.school_type,
                city: input.city,
                province: input.province,
                email: input.email,
                phone: input.phone,
                package_type: input.package_type,
                contact_person: input.contact_person,
                status: input.status,
                revenue_share,
                monthly_revenue: input.monthly_revenue.max(0),
                linked_tenant_id: input.linked_tenant_id,
            })
            .await
    }

    pub async fn list(&self, actor: &AuthUser) -> AppResult<Vec<School>> {
        actor.require_role(&[Role::Admin])?;
        self.schools.list().await
    }

    /// Per-school student counts for the "Manajemen Sekolah" list cards — see
    /// `SchoolRepository::roster_counts` doc comment.
    pub async fn roster_counts(&self, actor: &AuthUser) -> AppResult<Vec<crate::domain::school::SchoolRosterCount>> {
        actor.require_role(&[Role::Admin])?;
        self.schools.roster_counts().await
    }

    /// Data source for the "Link ke Tenant Existing" dropdown on school creation.
    pub async fn list_unlinked_tenants(&self, actor: &AuthUser) -> AppResult<Vec<crate::domain::school::LinkableTenant>> {
        actor.require_role(&[Role::Admin])?;
        self.schools.list_unlinked_tenants().await
    }

    /// For a linked-tenant school's import-siswa step: existing `core.akun` (role SISWA)
    /// at its tenant with no `ptn.student_profile` yet.
    pub async fn list_linkable_students(&self, actor: &AuthUser, school_id: Uuid) -> AppResult<Vec<crate::domain::school::LinkableStudent>> {
        actor.require_role(&[Role::Admin])?;
        self.schools.list_linkable_students(school_id).await
    }

    /// Grants PTN access to the checked existing students — see
    /// `SchoolRepository::link_existing_students` doc comment. Returns how many were
    /// actually newly linked (already-linked/non-SISWA ids are silently skipped).
    pub async fn link_existing_students(&self, actor: &AuthUser, school_id: Uuid, akun_ids: Vec<Uuid>) -> AppResult<i64> {
        actor.require_role(&[Role::Admin])?;
        self.schools.link_existing_students(school_id, &akun_ids).await
    }

    pub async fn get(&self, actor: &AuthUser, id: Uuid) -> AppResult<School> {
        actor.require_role(&[Role::Admin])?;
        self.schools
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("school {id} not found")))
    }

    pub async fn update(&self, actor: &AuthUser, id: Uuid, input: UpdateSchoolInput) -> AppResult<School> {
        actor.require_role(&[Role::Admin])?;
        validate(&input.name, &input.email, &input.contact_person)?;
        let revenue_share = validate_revenue_share(input.revenue_share)?;
        self.schools
            .update(
                id,
                SchoolUpdate {
                    name: input.name,
                    school_type: input.school_type,
                    city: input.city,
                    province: input.province,
                    email: input.email,
                    phone: input.phone,
                    package_type: input.package_type,
                    contact_person: input.contact_person,
                    status: input.status,
                    revenue_share,
                    monthly_revenue: input.monthly_revenue.max(0),
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("school {id} not found")))
    }

    pub async fn delete(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        let deleted = self.schools.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("school {id} not found")));
        }
        Ok(())
    }

    /// Powers the approve/deactivate/reactivate quick actions — including "Setujui &
    /// Onboarding" flipping a pending partner to active once the CSV import wizard finishes.
    pub async fn set_status(&self, actor: &AuthUser, id: Uuid, status: SchoolStatus) -> AppResult<School> {
        actor.require_role(&[Role::Admin])?;
        self.schools
            .set_status(id, status)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("school {id} not found")))
    }
}

fn validate(name: &str, email: &str, contact_person: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("school name is required".to_string()));
    }
    if email.trim().is_empty() || !email.contains('@') {
        return Err(AppError::Validation("a valid email is required".to_string()));
    }
    if contact_person.trim().is_empty() {
        return Err(AppError::Validation("contact person is required".to_string()));
    }
    Ok(())
}

fn validate_revenue_share(value: i32) -> AppResult<i32> {
    if !(0..=50).contains(&value) {
        return Err(AppError::Validation("revenue_share must be between 0 and 50".to_string()));
    }
    Ok(value)
}
