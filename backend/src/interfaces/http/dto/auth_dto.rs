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
    #[validate(email(message = "must be a valid email"))]
    pub email: String,
    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
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
