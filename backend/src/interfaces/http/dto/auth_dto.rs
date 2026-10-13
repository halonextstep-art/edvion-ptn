use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::user::{Role, User, UserStatus};

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 2, message = "name must be at least 2 characters"))]
    pub name: String,
    #[validate(email(message = "must be a valid email"))]
    pub email: String,
    #[validate(length(min = 6, message = "password must be at least 6 characters"))]
    pub password: String,
    pub role: String,
    pub school_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    // Accepts either a PTN account's email or a legacy cross-module account's
    // `core.akun.username` — see `AuthService::login` doc comment for why both exist and
    // how the lookup disambiguates them. No format validation: usernames aren't emails.
    #[validate(length(min = 1, message = "username is required"))]
    pub username: String,
    // `core.akun.username` is only unique *within* a tenant (e.g. "admin" exists under 18
    // different schools) — omit this for PTN's own platform-tenant accounts (admin/content/
    // B2C students), provide it (the school's `core.tenant.slug`) to disambiguate a
    // cross-module account that belongs to a specific partner school.
    pub tenant_slug: Option<String>,
    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    // `None` for a legacy cross-module account with no `core.akun.full_name`/`email` on
    // file — see `domain::user::User` doc comments. Real-world example: bulk-imported
    // student rows at partner schools that only ever got a `username` (NISN-style id), no
    // name or email. Frontend must handle both being missing.
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub school_id: Option<Uuid>,
    pub school_name: Option<String>,
    pub status: String,
    pub last_login: Option<DateTime<Utc>>,
    pub phone: Option<String>,
    pub nisn: Option<String>,
    pub grade: Option<String>,
    pub avatar_url: Option<String>,
    pub minat_jurusan: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            name: u.name,
            email: u.email,
            role: u.role.as_str().to_string(),
            school_id: u.school_id,
            school_name: u.school_name,
            status: u.status.as_str().to_string(),
            last_login: u.last_login,
            phone: u.phone,
            nisn: u.nisn,
            grade: u.grade,
            avatar_url: u.avatar_url,
            minat_jurusan: u.minat_jurusan,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 2, message = "name must be at least 2 characters"))]
    pub name: String,
    pub phone: Option<String>,
    /// "Jurusan yang Diminati" — see `UpdateProfileInput::minat_jurusan` doc comment for the
    /// "always resend the current value" convention this field follows.
    pub minat_jurusan: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "current password is required"))]
    pub current_password: String,
    #[validate(length(min = 6, message = "new password must be at least 6 characters"))]
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

pub fn parse_role(raw: &str) -> Result<Role, crate::error::AppError> {
    raw.parse::<Role>()
        .map_err(|e| crate::error::AppError::Validation(e))
}

pub fn parse_user_status(raw: &str) -> Result<UserStatus, crate::error::AppError> {
    raw.parse::<UserStatus>()
        .map_err(crate::error::AppError::Validation)
}
