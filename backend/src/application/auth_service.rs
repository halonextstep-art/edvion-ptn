//! Auth use-cases: register, login, fetch current user. Password hashing uses Argon2id;
//! session tokens are stateless JWTs (see `jwt.rs`).

use std::sync::Arc;

use uuid::Uuid;

use crate::application::jwt::JwtService;
use crate::application::password::{hash_password, verify_password};
use crate::domain::repository::{NewUser, UserRepository, UserUpdate};
use crate::domain::user::{Role, User, UserStatus};
use crate::error::{AppError, AppResult};

pub struct RegisterInput {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: Role,
    pub school_id: Option<Uuid>,
}

/// Self-service "edit my own profile" — deliberately narrower than the admin-only
/// `UserService::update`: no role/status/school_id/email here, so a user editing their own
/// account can never quietly grant themselves admin or move themselves to another school.
pub struct UpdateProfileInput {
    pub name: String,
    pub phone: Option<String>,
    /// "Jurusan yang Diminati" — real, student-declared field of interest. Like `phone`,
    /// this struct has no separate "unchanged" signal: the caller (frontend) must always
    /// resend the current value for whichever of these two optional fields it isn't
    /// actively changing, since `UserRepository::update` overwrites every column
    /// unconditionally (see `ProfileDialog.vue`'s `saveProfile` and
    /// `RasionalisasiSnbpEditor.vue`'s minat-jurusan save, both of which follow this
    /// convention).
    pub minat_jurusan: Option<String>,
}

pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
}

pub struct AuthenticatedUser {
    pub user: User,
    pub token: String,
}

pub struct AuthService {
    users: Arc<dyn UserRepository>,
    jwt: Arc<JwtService>,
}

impl AuthService {
    pub fn new(users: Arc<dyn UserRepository>, jwt: Arc<JwtService>) -> Self {
        Self { users, jwt }
    }

    pub async fn register(&self, input: RegisterInput) -> AppResult<AuthenticatedUser> {
        if input.password.len() < 6 {
            return Err(AppError::Validation(
                "password must be at least 6 characters".to_string(),
            ));
        }
        if self.users.find_by_email(&input.email).await?.is_some() {
            return Err(AppError::Conflict("email is already registered".to_string()));
        }
        // A School-PIC account always represents one real partner school, so `school_id` stays
        // mandatory for `Role::School`. Students, however, may now self-register without one —
        // this is the B2C (mandiri) path: a student who isn't enrolled through a partner
        // school. B2B students still get a `school_id` (set by the school's admin-import CSV
        // or the school-scoped create flow, both of which go through `UserService::create`,
        // not here).
        if matches!(input.role, Role::School) && input.school_id.is_none() {
            return Err(AppError::Validation(
                "school_id is required for school accounts".to_string(),
            ));
        }

        let password_hash = hash_password(&input.password)?;

        // Self-service registration is always immediately usable — `pending` is reserved for
        // accounts an admin deliberately parks (e.g. a school mid-onboarding).
        let user = self
            .users
            .create(NewUser {
                name: input.name,
                email: input.email,
                password_hash,
                role: input.role,
                school_id: input.school_id,
                status: UserStatus::Active,
                phone: None,
                nisn: None,
                grade: None,
                nis: None,
                gender: None,
                birth_date: None,
                enrolled_at: None,
                rombel_code: None,
                username: None,
            })
            .await?;

        let token = self.jwt.issue(user.id, &user.email, user.role)?;
        Ok(AuthenticatedUser { user, token })
    }

    pub async fn login(&self, email: &str, password: &str) -> AppResult<AuthenticatedUser> {
        let mut user = self
            .users
            .find_by_email(email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid email or password".to_string()))?;

        if !verify_password(password, &user.password_hash)? {
            return Err(AppError::Unauthorized("invalid email or password".to_string()));
        }

        // `status` actually gates access — not just a display label.
        match user.status {
            UserStatus::Inactive => {
                return Err(AppError::Unauthorized("this account has been deactivated".to_string()))
            }
            UserStatus::Pending => {
                return Err(AppError::Unauthorized(
                    "this account is still pending admin approval".to_string(),
                ))
            }
            UserStatus::Active => {}
        }

        self.users.touch_last_login(user.id).await?;
        user.last_login = Some(chrono::Utc::now());

        let token = self.jwt.issue(user.id, &user.email, user.role)?;
        Ok(AuthenticatedUser { user, token })
    }

    pub async fn me(&self, user_id: Uuid) -> AppResult<User> {
        self.users
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))
    }

    /// Self-service "edit my own name/phone" — every role (Admin/Sekolah/Siswa/Konten) can
    /// call this on their own account via `PUT /api/auth/me`. Email, role, status, school_id,
    /// nisn, and grade are all deliberately left untouched here: email doubles as the login
    /// identity (changing it needs its own re-verification flow, not built yet), and
    /// role/status/school_id/nisn/grade are either security-sensitive or sourced from a
    /// school's roster import — a student self-editing their own NISN or grade would be
    /// exactly the kind of fabricated data this project's rules forbid.
    pub async fn update_profile(&self, user_id: Uuid, input: UpdateProfileInput) -> AppResult<User> {
        if input.name.trim().is_empty() {
            return Err(AppError::Validation("name must not be empty".to_string()));
        }
        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        self.users
            .update(
                user_id,
                UserUpdate {
                    name: input.name.trim().to_string(),
                    email: user.email,
                    role: user.role,
                    school_id: user.school_id,
                    status: user.status,
                    phone: input.phone,
                    nisn: user.nisn,
                    grade: user.grade,
                    minat_jurusan: input.minat_jurusan,
                    password_hash: None,
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))
    }

    /// Self-service "change my own password" — requires the caller to prove they know the
    /// current password first (unlike `reset_password_for_seed`, which is a trusted seeder
    /// bypass, and unlike admin's `UserService::update`, where an admin can reset someone
    /// else's password without knowing the old one).
    pub async fn change_password(&self, user_id: Uuid, input: ChangePasswordInput) -> AppResult<()> {
        if input.new_password.len() < 6 {
            return Err(AppError::Validation(
                "new password must be at least 6 characters".to_string(),
            ));
        }
        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        if !verify_password(&input.current_password, &user.password_hash)? {
            return Err(AppError::Unauthorized("current password is incorrect".to_string()));
        }

        let password_hash = hash_password(&input.new_password)?;
        self.users
            .update(
                user_id,
                UserUpdate {
                    name: user.name,
                    email: user.email,
                    role: user.role,
                    school_id: user.school_id,
                    status: user.status,
                    phone: user.phone,
                    nisn: user.nisn,
                    grade: user.grade,
                    minat_jurusan: user.minat_jurusan,
                    password_hash: Some(password_hash),
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;
        Ok(())
    }

    /// Password-free lookup by email — used by the seeder's `register_or_skip` to fetch an
    /// already-existing account on a re-run without needing to re-authenticate.
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        self.users.find_by_email(email).await
    }

    /// Self-service "set/clear my own profile photo" — a dedicated single-field update
    /// (via `UserRepository::update_avatar`) rather than routing through the full
    /// `UserUpdate` struct, since this is called on every avatar upload/removal.
    pub async fn set_avatar(&self, user_id: Uuid, avatar_url: Option<String>) -> AppResult<User> {
        self.users
            .update_avatar(user_id, avatar_url)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))
    }

    /// Force-sets an existing user's password to a known value — used *only* by the seeder
    /// (`register_or_skip`) so the demo credentials it prints out at the end of a run are
    /// always guaranteed to work. This matters because a re-run of the seeder can encounter
    /// an account that already exists under a seed email but with a different password (e.g.
    /// seeded by an older version of this script before a demo password was changed, or
    /// created through the real registration flow with the same email). Rather than aborting
    /// the whole seed run over a stale/mismatched password on a row that's clearly meant to
    /// be a reusable demo account, this brings it back in line. Not exposed via any HTTP
    /// route — there is no user-facing "reset a stranger's password by email" capability.
    pub async fn reset_password_for_seed(&self, user: &User, new_password: &str) -> AppResult<User> {
        let password_hash = hash_password(new_password)?;
        self.users
            .update(
                user.id,
                UserUpdate {
                    name: user.name.clone(),
                    email: user.email.clone(),
                    role: user.role,
                    school_id: user.school_id,
                    status: user.status,
                    phone: user.phone.clone(),
                    nisn: user.nisn.clone(),
                    grade: user.grade.clone(),
                    minat_jurusan: user.minat_jurusan.clone(),
                    password_hash: Some(password_hash),
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("user {} not found", user.id)))
    }
}
