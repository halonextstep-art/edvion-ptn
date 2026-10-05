//! Admin-facing user management use-cases (list / create / update / delete any account).
//! Distinct from `AuthService`, which only covers self-service register/login/me — this
//! service is for the "Manajemen User" admin screen and always requires an Admin actor.

use std::sync::Arc;

use uuid::Uuid;

use crate::application::password::hash_password;
use crate::domain::repository::{NewUser, UserFilter, UserRepository, UserUpdate};
use crate::domain::user::{Role, User, UserStatus};
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct ListUsersFilter {
    pub role: Option<Role>,
    pub status: Option<UserStatus>,
    pub search: Option<String>,
    /// B2B (`Some(true)`, has a `school_id`) vs B2C (`Some(false)`, no `school_id`) segment
    /// toggle on the admin "Manajemen User" screen. Ignored for a `Role::School` actor, who
    /// is always scoped to their own school's students regardless (B2B by definition).
    pub has_school: Option<bool>,
}

pub struct CreateUserInput {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: Role,
    pub school_id: Option<Uuid>,
    pub status: UserStatus,
    pub phone: Option<String>,
    pub nisn: Option<String>,
    pub grade: Option<String>,
    /// Student-only, from the school-mitra bulk import format — see `domain::user::User`
    /// doc comments. Create-only (no matching field on `UpdateUserInput`).
    pub nis: Option<String>,
    pub gender: Option<String>,
    pub birth_date: Option<chrono::NaiveDate>,
    pub enrolled_at: Option<chrono::NaiveDate>,
    pub rombel_code: Option<String>,
    pub username: Option<String>,
}

pub struct UpdateUserInput {
    pub name: String,
    pub email: String,
    pub role: Role,
    pub school_id: Option<Uuid>,
    pub status: UserStatus,
    pub phone: Option<String>,
    pub nisn: Option<String>,
    pub grade: Option<String>,
    /// `Some(new_password)` only when the admin wants to reset the password.
    pub password: Option<String>,
}

pub struct UserService {
    users: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(users: Arc<dyn UserRepository>) -> Self {
        Self { users }
    }

    /// Admin sees everything (subject to the given filter). A School actor is silently
    /// forced to `role = Student, school_id = <their own school>` regardless of what the
    /// filter says — a school PIC can never browse another school's roster or non-student
    /// accounts, no matter what query params the client sends.
    pub async fn list(&self, actor: &AuthUser, filter: ListUsersFilter) -> AppResult<Vec<User>> {
        actor.require_role(&[Role::Admin, Role::School])?;
        let school_id = match actor.role {
            Role::School => Some(self.resolve_own_school_id(actor).await?),
            _ => None,
        };
        let role = if actor.role == Role::School { Some(Role::Student) } else { filter.role };
        // A School actor is already hard-scoped to `school_id = <their own school>` above, so
        // the B2B/B2C toggle only makes sense for an Admin actor browsing the full roster.
        let has_school = if actor.role == Role::School { None } else { filter.has_school };
        self.users
            .list(UserFilter { role, status: filter.status, search: filter.search, school_id, has_school })
            .await
    }

    pub async fn create(&self, actor: &AuthUser, mut input: CreateUserInput) -> AppResult<User> {
        actor.require_role(&[Role::Admin, Role::School])?;
        if actor.role == Role::School {
            input.role = Role::Student;
            input.school_id = Some(self.resolve_own_school_id(actor).await?);
        }
        validate_school_requirement(input.role, input.school_id)?;
        if input.password.len() < 6 {
            return Err(AppError::Validation("password must be at least 6 characters".to_string()));
        }
        if self.users.find_by_email(&input.email).await?.is_some() {
            return Err(AppError::Conflict("email is already registered".to_string()));
        }

        let password_hash = hash_password(&input.password)?;
        self.users
            .create(NewUser {
                name: input.name,
                email: input.email,
                password_hash,
                role: input.role,
                school_id: input.school_id,
                status: input.status,
                phone: input.phone,
                nisn: input.nisn,
                grade: input.grade,
                nis: input.nis,
                gender: input.gender,
                birth_date: input.birth_date,
                enrolled_at: input.enrolled_at,
                rombel_code: input.rombel_code,
                username: input.username,
            })
            .await
    }

    pub async fn update(&self, actor: &AuthUser, id: Uuid, mut input: UpdateUserInput) -> AppResult<User> {
        actor.require_role(&[Role::Admin, Role::School])?;
        if actor.role == Role::School {
            let school_id = self.resolve_own_school_id(actor).await?;
            self.ensure_owns_student(school_id, id).await?;
            input.role = Role::Student;
            input.school_id = Some(school_id);
        }
        validate_school_requirement(input.role, input.school_id)?;

        if let Some(existing) = self.users.find_by_email(&input.email).await? {
            if existing.id != id {
                return Err(AppError::Conflict("email is already used by another account".to_string()));
            }
        }

        // Admin/school never edit `minat_jurusan` (self-declared by the student only) — fetch
        // the current value so this admin-facing update path can't accidentally null it out.
        let current = self
            .users
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("user {id} not found")))?;

        let password_hash = match &input.password {
            Some(p) if !p.is_empty() => {
                if p.len() < 6 {
                    return Err(AppError::Validation("password must be at least 6 characters".to_string()));
                }
                Some(hash_password(p)?)
            }
            _ => None,
        };

        self.users
            .update(
                id,
                UserUpdate {
                    name: input.name,
                    email: input.email,
                    role: input.role,
                    school_id: input.school_id,
                    status: input.status,
                    phone: input.phone,
                    nisn: input.nisn,
                    grade: input.grade,
                    minat_jurusan: current.minat_jurusan,
                    password_hash,
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("user {id} not found")))
    }

    /// Admin-only — a School actor can deactivate a student via `set_status` instead of a
    /// hard delete, which keeps their attempt history intact and is safer to expose.
    pub async fn delete(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        if actor.user_id == id {
            return Err(AppError::Validation("you cannot delete your own account".to_string()));
        }
        let deleted = self.users.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("user {id} not found")));
        }
        Ok(())
    }

    /// Powers the toggle-active/deactivate/approve-pending actions on the admin screen, and
    /// the activate/deactivate action on the School portal's own student roster.
    pub async fn set_status(&self, actor: &AuthUser, id: Uuid, status: UserStatus) -> AppResult<User> {
        actor.require_role(&[Role::Admin, Role::School])?;
        if actor.role == Role::School {
            let school_id = self.resolve_own_school_id(actor).await?;
            self.ensure_owns_student(school_id, id).await?;
        }
        if actor.user_id == id && status != UserStatus::Active {
            return Err(AppError::Validation("you cannot deactivate your own account".to_string()));
        }
        self.users
            .set_status(id, status)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("user {id} not found")))
    }

    /// Single-account lookup for admin screens that need to enrich another record with a
    /// user's name/email (e.g. the manual purchase-request queue showing who the buyer is).
    pub async fn get(&self, actor: &AuthUser, id: Uuid) -> AppResult<User> {
        actor.require_role(&[Role::Admin])?;
        self.users.find_by_id(id).await?.ok_or_else(|| AppError::NotFound(format!("user {id} not found")))
    }

    async fn resolve_own_school_id(&self, actor: &AuthUser) -> AppResult<Uuid> {
        let me = self
            .users
            .find_by_id(actor.user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("akun tidak ditemukan".to_string()))?;
        me.school_id
            .ok_or_else(|| AppError::Validation("akun sekolah ini belum terhubung ke data sekolah".to_string()))
    }

    async fn ensure_owns_student(&self, school_id: Uuid, student_id: Uuid) -> AppResult<()> {
        let student = self
            .users
            .find_by_id(student_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("user {student_id} not found")))?;
        if student.role != Role::Student || student.school_id != Some(school_id) {
            return Err(AppError::Forbidden("kamu hanya bisa mengelola siswa dari sekolahmu sendiri".to_string()));
        }
        Ok(())
    }
}

/// A School-PIC account always represents one real partner school, so `school_id` stays
/// mandatory for `Role::School`. `Role::Student` may now be created/edited with
/// `school_id: None` — a B2C (mandiri) student, not enrolled through any partner school.
fn validate_school_requirement(role: Role, school_id: Option<Uuid>) -> AppResult<()> {
    if matches!(role, Role::School) && school_id.is_none() {
        return Err(AppError::Validation("school_id is required for school accounts".to_string()));
    }
    Ok(())
}
