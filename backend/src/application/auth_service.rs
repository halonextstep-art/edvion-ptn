//! Auth use-cases: register, login, fetch current user. Password hashing uses Argon2id;
//! session tokens are stateless JWTs (see `jwt.rs`).
//!
//! `core.akun`/`core.tenant` are RLS-protected, and PTN login is email+password only (no
//! "pick your school" step) — so the very first step of login/registration (does this
//! email already exist, and if so under which tenant?) genuinely cannot have
//! `app.tenant_id` set yet, since that's exactly what's being resolved. That one lookup
//! runs through `auth_lookup_pool`, a second `PgPool` connected as the narrow BYPASSRLS
//! `ptn_auth_lookup` role (see migration 20260101000001_core_extensions.sql) — read-only,
//! granted `SELECT` on `core.akun`/`core.role`/`core.tenant` only. Every other operation
//! here (once the tenant is known) goes through the normal RLS-scoped `UserRepository`.

use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::application::jwt::JwtService;
use crate::application::password::{hash_password, verify_password};
use crate::domain::repository::{NewUser, UserRepository, UserUpdate};
use crate::domain::tenant::TenantCtx;
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

/// Row shape for the one pre-auth lookup query — intentionally minimal (just enough to
/// verify a password and know which tenant/role to issue a token for), never exposed
/// outside this file.
#[derive(sqlx::FromRow)]
struct AuthLookupRow {
    public_id: Uuid,
    tenant_id: i64,
    password_hash: String,
    role_kode: String,
    status: String,
}

pub struct AuthService {
    users: Arc<dyn UserRepository>,
    jwt: Arc<JwtService>,
    /// Connected as the BYPASSRLS `ptn_auth_lookup` role — see module doc comment.
    auth_lookup_pool: PgPool,
}

impl AuthService {
    pub fn new(users: Arc<dyn UserRepository>, jwt: Arc<JwtService>, auth_lookup_pool: PgPool) -> Self {
        Self { users, jwt, auth_lookup_pool }
    }

    async fn find_lookup_row_by_email(&self, email: &str) -> AppResult<Option<AuthLookupRow>> {
        let row = sqlx::query_as::<_, AuthLookupRow>(
            r#"SELECT a.public_id, a.tenant_id, a.password_hash, r.kode AS role_kode, a.status
               FROM core.akun a
               JOIN core.role r ON r.id = a.role_id
               WHERE a.email = $1"#,
        )
        .bind(email)
        .fetch_optional(&self.auth_lookup_pool)
        .await
        .map_err(AppError::Database)?;
        Ok(row)
    }

    /// `core.akun.username` is only unique *within* a tenant (shared with academic/tahfiz/
    /// office/etc. — e.g. the username "admin" exists under 18 different schools there), so
    /// a tenant-less lookup can't just take the first match. With `tenant_slug` given, this
    /// resolves straight to one account. Without it, this only ever looks inside PTN's own
    /// reserved platform tenant (`tenant_id = 0` — admin/content/B2C-student accounts, which
    /// PTN itself created and knows to be uniquely-usernamed there), so it still can't
    /// silently guess across unrelated schools.
    async fn find_lookup_row_by_username(
        &self,
        username: &str,
        tenant_slug: Option<&str>,
    ) -> AppResult<Option<AuthLookupRow>> {
        let row = if let Some(slug) = tenant_slug {
            sqlx::query_as::<_, AuthLookupRow>(
                r#"SELECT a.public_id, a.tenant_id, a.password_hash, r.kode AS role_kode, a.status
                   FROM core.akun a
                   JOIN core.role r ON r.id = a.role_id
                   JOIN core.tenant t ON t.id = a.tenant_id
                   WHERE t.slug = $1 AND lower(a.username) = lower($2)"#,
            )
            .bind(slug)
            .bind(username)
            .fetch_optional(&self.auth_lookup_pool)
            .await
            .map_err(AppError::Database)?
        } else {
            sqlx::query_as::<_, AuthLookupRow>(
                r#"SELECT a.public_id, a.tenant_id, a.password_hash, r.kode AS role_kode, a.status
                   FROM core.akun a
                   JOIN core.role r ON r.id = a.role_id
                   WHERE a.tenant_id = 0 AND lower(a.username) = lower($1)"#,
            )
            .bind(username)
            .fetch_optional(&self.auth_lookup_pool)
            .await
            .map_err(AppError::Database)?
        };
        Ok(row)
    }

    pub async fn register(&self, input: RegisterInput) -> AppResult<AuthenticatedUser> {
        if input.password.len() < 6 {
            return Err(AppError::Validation(
                "password must be at least 6 characters".to_string(),
            ));
        }
        if self.find_lookup_row_by_email(&input.email).await?.is_some() {
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

        // `school_id` here is `core.tenant.public_id` — resolving it to the internal bigint
        // `core.tenant.id` (the real `TenantCtx.tenant_id`) requires a lookup of its own,
        // through the same BYPASSRLS pool (core.tenant has RLS... actually no, `core.tenant`
        // itself has no RLS policy — but using the lookup pool keeps this one connection
        // consistent and avoids opening a third pool just for this).
        let ctx = if matches!(input.role, Role::School) {
            let tenant_id: i64 = sqlx::query_scalar("SELECT id FROM core.tenant WHERE public_id = $1")
                .bind(input.school_id.expect("validated above"))
                .fetch_optional(&self.auth_lookup_pool)
                .await
                .map_err(AppError::Database)?
                .ok_or_else(|| AppError::Validation("school_id does not match any known school".to_string()))?;
            TenantCtx::school(tenant_id)
        } else if matches!(input.role, Role::Student) {
            // B2C (mandiri) student, no partner school — still needs *a* tenant row to
            // satisfy `core.akun.tenant_id NOT NULL`/RLS. Uses the reserved platform tenant
            // but is NOT `is_platform_admin` (a B2C student has no elevated access — only
            // their own row is visible to them, same as any other student).
            TenantCtx::school(TenantCtx::PLATFORM_TENANT_ID)
        } else {
            TenantCtx::for_role(input.role, TenantCtx::PLATFORM_TENANT_ID)
        };

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

        let token = self.jwt.issue(user.id, user.email.as_deref().unwrap_or(""), user.role, ctx)?;
        Ok(AuthenticatedUser { user, token })
    }

    /// `username_or_email` is PTN's one login field — it's either a PTN account's own
    /// email (self-registered admin/content/school/student, always globally unique because
    /// PTN itself enforces that at register time) or a legacy cross-module account's
    /// `core.akun.username` (shared tables from academic/tahfiz/office/etc., never created
    /// through PTN's own register flow). Email is tried first since it's unambiguous by
    /// construction; `tenant_slug` only matters for the username fallback — see
    /// `find_lookup_row_by_username` doc comment for why.
    pub async fn login(
        &self,
        username_or_email: &str,
        tenant_slug: Option<&str>,
        password: &str,
    ) -> AppResult<AuthenticatedUser> {
        let row = match self.find_lookup_row_by_email(username_or_email).await? {
            Some(row) => row,
            None => self
                .find_lookup_row_by_username(username_or_email, tenant_slug)
                .await?
                .ok_or_else(|| AppError::Unauthorized("invalid username or password".to_string()))?,
        };

        if !verify_password(password, &row.password_hash)? {
            return Err(AppError::Unauthorized("invalid username or password".to_string()));
        }

        let status = std::str::FromStr::from_str(&row.status)
            .map_err(|e: String| AppError::Internal(anyhow::anyhow!("corrupt user status in db: {e}")))?;
        match status {
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

        // `core.role` carries roles from every module sharing this table (GURU, BK,
        // WAKA_*, ...) — an account under one of those isn't "corrupt data", it's simply
        // not a PTN account, so this is a normal login rejection, not an internal error.
        let role = Role::from_core_role_kode(&row.role_kode)
            .map_err(|_| AppError::Unauthorized("invalid username or password".to_string()))?;
        let ctx = TenantCtx::for_role(role, row.tenant_id);

        self.users.touch_last_login(row.public_id, &ctx).await?;

        let mut user = self
            .users
            .find_by_id(row.public_id, &ctx)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("user vanished right after login")))?;
        user.last_login = Some(chrono::Utc::now());

        let token = self.jwt.issue(user.id, user.email.as_deref().unwrap_or(""), user.role, ctx)?;
        Ok(AuthenticatedUser { user, token })
    }

    pub async fn me(&self, user_id: Uuid, ctx: &TenantCtx) -> AppResult<User> {
        self.users
            .find_by_id(user_id, ctx)
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
    pub async fn update_profile(&self, user_id: Uuid, input: UpdateProfileInput, ctx: &TenantCtx) -> AppResult<User> {
        if input.name.trim().is_empty() {
            return Err(AppError::Validation("name must not be empty".to_string()));
        }
        let user = self
            .users
            .find_by_id(user_id, ctx)
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
                ctx,
            )
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))
    }

    /// Self-service "change my own password" — requires the caller to prove they know the
    /// current password first (unlike `reset_password_for_seed`, which is a trusted seeder
    /// bypass, and unlike admin's `UserService::update`, where an admin can reset someone
    /// else's password without knowing the old one).
    pub async fn change_password(&self, user_id: Uuid, input: ChangePasswordInput, ctx: &TenantCtx) -> AppResult<()> {
        if input.new_password.len() < 6 {
            return Err(AppError::Validation(
                "new password must be at least 6 characters".to_string(),
            ));
        }
        let user = self
            .users
            .find_by_id(user_id, ctx)
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
                    name: user.name.unwrap_or_default(),
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
                ctx,
            )
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;
        Ok(())
    }

    /// Password-free lookup by email — used by the seeder's `register_or_skip` to fetch an
    /// already-existing account on a re-run without needing to re-authenticate. Goes through
    /// the BYPASSRLS lookup pool (see module doc comment) and then re-fetches the full
    /// `User` through the RLS-scoped repository once the tenant/role is known.
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let Some(row) = self.find_lookup_row_by_email(email).await? else {
            return Ok(None);
        };
        let role = Role::from_core_role_kode(&row.role_kode)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt role in db: {e}")))?;
        let ctx = TenantCtx::for_role(role, row.tenant_id);
        self.users.find_by_id(row.public_id, &ctx).await
    }

    /// Self-service "set/clear my own profile photo" — a dedicated single-field update
    /// (via `UserRepository::update_avatar`) rather than routing through the full
    /// `UserUpdate` struct, since this is called on every avatar upload/removal.
    pub async fn set_avatar(&self, user_id: Uuid, avatar_url: Option<String>, ctx: &TenantCtx) -> AppResult<User> {
        self.users
            .update_avatar(user_id, avatar_url, ctx)
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
    pub async fn reset_password_for_seed(&self, user: &User, new_password: &str, ctx: &TenantCtx) -> AppResult<User> {
        let password_hash = hash_password(new_password)?;
        self.users
            .update(
                user.id,
                UserUpdate {
                    name: user.name.clone().unwrap_or_default(),
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
                ctx,
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("user {} not found", user.id)))
    }
}
