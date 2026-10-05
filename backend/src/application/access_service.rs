//! Real, server-side access control for premium content. Per-package content scoping (this
//! feature) replaced the previous all-or-nothing model, where owning ANY package (via voucher
//! redemption or school entitlement) unlocked EVERY `is_premium` `TryoutSession` platform-wide,
//! and `SimulationTemplate` ("Simulasi UTBK" / TO) had NO gating at all. Now a `Package`
//! explicitly lists which `TryoutSession`s and/or `SimulationTemplate`s it grants access to (see
//! `package_content_items` / `PackageContentRepository`), and access is checked per content
//! item: does the student own — via a personally-redeemed voucher OR their school's active
//! entitlement — ANY package that includes this specific piece of content?
//!
//! `20250101000031_package_content_items.sql` backfilled every Package that existed at
//! migration time to cover every premium TryoutSession/SimulationTemplate that existed then, so
//! existing buyers/schools didn't lose access the moment this shipped. Content created AFTER
//! that migration must be explicitly assigned to a Package by an admin — if premium content
//! isn't assigned to any package yet, `has_access_to_content` fails CLOSED (no access) rather
//! than silently treating unassigned content as free.

use std::collections::HashSet;
use std::sync::Arc;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::domain::entitlement::SchoolPackageEntitlement;
use crate::domain::package::PackageContentType;
use crate::domain::repository::{
    NewSchoolEntitlement, PackageContentRepository, PackageRepository, SchoolEntitlementRepository, SchoolRepository,
    UserRepository, VoucherRepository,
};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct CreateEntitlementInput {
    pub school_id: Uuid,
    pub package_id: Uuid,
    pub starts_at: NaiveDate,
    pub expires_at: Option<NaiveDate>,
    pub note: Option<String>,
}

/// Everything a student currently has access to, computed once — powers the lock/unlock icons
/// across the whole student-facing catalog in a single round trip instead of one access check
/// per card.
#[derive(Debug, Default)]
pub struct UnlockedContent {
    pub tryout_session_ids: HashSet<Uuid>,
    pub simulation_template_ids: HashSet<Uuid>,
}

pub struct AccessService {
    entitlements: Arc<dyn SchoolEntitlementRepository>,
    vouchers: Arc<dyn VoucherRepository>,
    users: Arc<dyn UserRepository>,
    packages: Arc<dyn PackageRepository>,
    schools: Arc<dyn SchoolRepository>,
    content: Arc<dyn PackageContentRepository>,
}

impl AccessService {
    pub fn new(
        entitlements: Arc<dyn SchoolEntitlementRepository>,
        vouchers: Arc<dyn VoucherRepository>,
        users: Arc<dyn UserRepository>,
        packages: Arc<dyn PackageRepository>,
        schools: Arc<dyn SchoolRepository>,
        content: Arc<dyn PackageContentRepository>,
    ) -> Self {
        Self { entitlements, vouchers, users, packages, schools, content }
    }

    /// Admin only — "sematkan paket ke sekolah".
    pub async fn create_entitlement(&self, actor: &AuthUser, input: CreateEntitlementInput) -> AppResult<SchoolPackageEntitlement> {
        actor.require_role(&[Role::Admin])?;
        self.schools.find_by_id(input.school_id).await?
            .ok_or_else(|| AppError::NotFound(format!("school {} not found", input.school_id)))?;
        self.packages.find_by_id(input.package_id).await?
            .ok_or_else(|| AppError::NotFound(format!("package {} not found", input.package_id)))?;
        if let Some(expires) = input.expires_at {
            if expires < input.starts_at {
                return Err(AppError::Validation("tanggal berakhir tidak boleh sebelum tanggal mulai".to_string()));
            }
        }
        self.entitlements.create(NewSchoolEntitlement {
            school_id: input.school_id,
            package_id: input.package_id,
            starts_at: input.starts_at,
            expires_at: input.expires_at,
            note: input.note,
            granted_by: actor.user_id,
        }).await
    }

    /// Admin sees any school; a School PIC may only see their OWN school's entitlements.
    pub async fn list_for_school(&self, actor: &AuthUser, school_id: Uuid) -> AppResult<Vec<SchoolPackageEntitlement>> {
        match actor.role {
            Role::Admin => {}
            Role::School => {
                let user = self.users.find_by_id(actor.user_id).await?
                    .ok_or_else(|| AppError::NotFound("actor user not found".to_string()))?;
                if user.school_id != Some(school_id) {
                    return Err(AppError::Forbidden("bukan sekolah Anda".to_string()));
                }
            }
            _ => return Err(AppError::Forbidden("role tidak diizinkan".to_string())),
        }
        self.entitlements.list_by_school(school_id).await
    }

    pub async fn delete_entitlement(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        let deleted = self.entitlements.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("entitlement {id} not found")));
        }
        Ok(())
    }

    /// Every `package_id` this student owns — via a personally-redeemed voucher OR (if they
    /// belong to a school) their school's active entitlement. This is the raw ownership
    /// signal; whether a given package actually includes the specific content the student
    /// wants is answered separately by `has_access_to_content`.
    async fn owned_package_ids(&self, student_id: Uuid) -> AppResult<HashSet<Uuid>> {
        let mut ids: HashSet<Uuid> = self.vouchers.student_redeemed_package_ids(student_id).await?.into_iter().collect();
        if let Some(user) = self.users.find_by_id(student_id).await? {
            if let Some(school_id) = user.school_id {
                let today = chrono::Utc::now().date_naive();
                ids.extend(self.entitlements.active_entitlement_package_ids(school_id, today).await?);
            }
        }
        Ok(ids)
    }

    /// Core per-content check: is there ANY package that (a) includes this exact content item
    /// AND (b) this student owns? Fails closed if the content isn't assigned to any package.
    async fn has_access_to_content(&self, student_id: Uuid, content_type: PackageContentType, content_id: Uuid) -> AppResult<bool> {
        let granting_packages = self.content.package_ids_for_content(content_type, content_id).await?;
        if granting_packages.is_empty() {
            return Ok(false);
        }
        let owned = self.owned_package_ids(student_id).await?;
        Ok(granting_packages.into_iter().any(|pid| owned.contains(&pid)))
    }

    /// Called by `TryoutService::start_attempt` for real server-side enforcement when a
    /// `TryoutSession.is_premium` session is started directly (NOT as part of a Simulasi UTBK
    /// run — see `SimulationService::activate_slot`, which bypasses this per-session check
    /// because access was already verified once at the template level in `start_run`).
    pub async fn has_access_to_tryout_session(&self, student_id: Uuid, session_id: Uuid) -> AppResult<bool> {
        self.has_access_to_content(student_id, PackageContentType::TryoutSession, session_id).await
    }

    /// Called by `SimulationService::start_run` for real server-side enforcement of
    /// `SimulationTemplate.is_premium` — this template previously had NO gating at all.
    pub async fn has_access_to_simulation_template(&self, student_id: Uuid, template_id: Uuid) -> AppResult<bool> {
        self.has_access_to_content(student_id, PackageContentType::SimulationTemplate, template_id).await
    }

    /// Everything this student can currently play — one query instead of one round-trip per
    /// content item. Powers `AccessStatusResponse` for the student-facing UI's lock/unlock
    /// icons across both the Tryout list and the Simulasi UTBK list.
    pub async fn list_unlocked_content(&self, student_id: Uuid) -> AppResult<UnlockedContent> {
        let owned: Vec<Uuid> = self.owned_package_ids(student_id).await?.into_iter().collect();
        let pairs = self.content.content_ids_for_packages(&owned).await?;
        let mut unlocked = UnlockedContent::default();
        for (content_type, id) in pairs {
            match content_type {
                PackageContentType::TryoutSession => { unlocked.tryout_session_ids.insert(id); }
                PackageContentType::SimulationTemplate => { unlocked.simulation_template_ids.insert(id); }
            }
        }
        Ok(unlocked)
    }

    pub async fn my_access_status(&self, actor: &AuthUser) -> AppResult<UnlockedContent> {
        actor.require_role(&[Role::Student])?;
        self.list_unlocked_content(actor.user_id).await
    }

    /// The jenjang (SMP/SMA/SMK/MA) of this student's school, if they belong to one — powers
    /// `TryoutService::list_sessions`/`SimulationService::list_templates`'s
    /// `school_type_scope` catalog filter. `None` for a B2C student (no school at all, so their
    /// jenjang can't be determined — catalog filtering is skipped for them rather than guessed).
    pub async fn student_school_type(&self, student_id: Uuid) -> AppResult<Option<crate::domain::school::SchoolType>> {
        let Some(user) = self.users.find_by_id(student_id).await? else { return Ok(None) };
        let Some(school_id) = user.school_id else { return Ok(None) };
        let school = self.schools.find_by_id(school_id).await?;
        Ok(school.map(|s| s.school_type))
    }
}
