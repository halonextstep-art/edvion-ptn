use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::application::user_service::{CreateUserInput, ListUsersFilter, UpdateUserInput};
use crate::domain::user::UserStatus;
use crate::interfaces::http::dto::auth_dto::{parse_role, parse_user_status};

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub role: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    /// `true` = only B2B accounts (has a school), `false` = only B2C (mandiri, no school).
    /// Omitted = no filtering on this dimension.
    pub has_school: Option<bool>,
}

impl UserListQuery {
    pub fn into_filter(self) -> Result<ListUsersFilter, crate::error::AppError> {
        let role = self.role.map(|r| parse_role(&r)).transpose()?;
        let status = self.status.map(|s| parse_user_status(&s)).transpose()?;
        Ok(ListUsersFilter { role, status, search: self.search, has_school: self.has_school })
    }
}

fn default_status() -> String {
    "active".to_string()
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserPayload {
    #[validate(length(min = 2, message = "name must be at least 2 characters"))]
    pub name: String,
    #[validate(email(message = "must be a valid email"))]
    pub email: String,
    #[validate(length(min = 6, message = "password must be at least 6 characters"))]
    pub password: String,
    pub role: String,
    pub school_id: Option<Uuid>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub nisn: Option<String>,
    #[serde(default)]
    pub grade: Option<String>,
    /// Student-only, from the school-mitra bulk import format (see `domain::user::User` doc
    /// comments). Create-only — there is deliberately no matching field on
    /// `UpdateUserPayload`, so the general "Edit User" admin form can never null these back
    /// out on an unrelated edit.
    #[serde(default)]
    pub nis: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub birth_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub enrolled_at: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub rombel_code: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}

impl CreateUserPayload {
    pub fn into_input(self) -> Result<CreateUserInput, crate::error::AppError> {
        Ok(CreateUserInput {
            name: self.name,
            email: self.email,
            password: self.password,
            role: parse_role(&self.role)?,
            school_id: self.school_id,
            status: parse_user_status(&self.status)?,
            phone: self.phone,
            nisn: self.nisn,
            grade: self.grade,
            nis: self.nis,
            gender: self.gender,
            birth_date: self.birth_date,
            enrolled_at: self.enrolled_at,
            rombel_code: self.rombel_code,
            username: self.username,
        })
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserPayload {
    #[validate(length(min = 2, message = "name must be at least 2 characters"))]
    pub name: String,
    #[validate(email(message = "must be a valid email"))]
    pub email: String,
    pub role: String,
    pub school_id: Option<Uuid>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub nisn: Option<String>,
    #[serde(default)]
    pub grade: Option<String>,
    /// Present + non-empty only when the admin wants to reset the password.
    #[serde(default)]
    pub password: Option<String>,
}

impl UpdateUserPayload {
    pub fn into_input(self) -> Result<UpdateUserInput, crate::error::AppError> {
        Ok(UpdateUserInput {
            name: self.name,
            email: self.email,
            role: parse_role(&self.role)?,
            school_id: self.school_id,
            status: parse_user_status(&self.status)?,
            phone: self.phone,
            nisn: self.nisn,
            grade: self.grade,
            password: self.password,
        })
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserStatusPayload {
    pub status: String,
}

impl UserStatusPayload {
    pub fn into_status(self) -> Result<UserStatus, crate::error::AppError> {
        parse_user_status(&self.status)
    }
}
