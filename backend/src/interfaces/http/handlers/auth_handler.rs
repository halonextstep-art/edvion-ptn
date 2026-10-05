use axum::{
    extract::{Multipart, State},
    Json,
};
use validator::Validate;

use crate::application::auth_service::{ChangePasswordInput, RegisterInput, UpdateProfileInput};
use crate::error::{AppError, AppResult};
use crate::infrastructure::storage::{save_upload, AVATAR_POLICY};
use crate::interfaces::http::dto::auth_dto::{
    parse_role, AuthResponse, ChangePasswordRequest, LoginRequest, MessageResponse, RegisterRequest, UpdateProfileRequest,
    UserResponse,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let role = parse_role(&payload.role)?;

    // This is the public, unauthenticated self-registration endpoint — the only role it may
    // ever create is Student (B2C/mandiri, no school required; B2B students are imported by
    // their school instead). Admin/School/Content accounts are provisioned exclusively by an
    // already-authenticated Admin via `UserService::create` (`POST /api/users`), never here —
    // otherwise anyone could self-register as Admin. `AuthService::register` itself stays
    // role-agnostic since `bin/seed.rs` also calls it directly to bootstrap non-student demo
    // accounts.
    if role != crate::domain::user::Role::Student {
        return Err(AppError::Validation(
            "pendaftaran mandiri hanya tersedia untuk akun siswa".to_string(),
        ));
    }

    // Platform-wide B2C on/off toggle (see `domain::platform_settings` doc comment) — only
    // blocks the schoolless (B2C/mandiri) case. A student registering WITH a school_id (B2B
    // self-register) is never affected, and this check lives here in the HTTP handler rather
    // than in `AuthService::register` so it can never affect `bin/seed.rs`, which calls that
    // service directly to bootstrap demo accounts.
    if payload.school_id.is_none() && !state.platform_settings_service.is_b2c_registration_enabled().await? {
        return Err(AppError::Validation(
            "pendaftaran mandiri (tanpa sekolah) sedang dinonaktifkan sementara".to_string(),
        ));
    }

    let result = state
        .auth_service
        .register(RegisterInput {
            name: payload.name,
            email: payload.email,
            password: payload.password,
            role,
            school_id: payload.school_id,
        })
        .await?;

    Ok(Json(AuthResponse {
        token: result.token,
        user: result.user.into(),
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result = state.auth_service.login(&payload.email, &payload.password).await?;

    Ok(Json(AuthResponse {
        token: result.token,
        user: result.user.into(),
    }))
}

pub async fn me(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<UserResponse>> {
    let user = state.auth_service.me(auth.user_id).await?;
    Ok(Json(user.into()))
}

/// Self-service profile edit — every logged-in role uses this to update their own name/phone
/// from the Profile dialog. Never lets the caller touch role/status/school/email (see
/// `AuthService::update_profile` doc comment).
pub async fn update_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> AppResult<Json<UserResponse>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = state
        .auth_service
        .update_profile(
            auth.user_id,
            UpdateProfileInput { name: payload.name, phone: payload.phone, minat_jurusan: payload.minat_jurusan },
        )
        .await?;

    Ok(Json(user.into()))
}

/// Self-service avatar upload — accepts a single-field multipart form (any field name; the
/// first field found is used), validates it against `AVATAR_POLICY` (size + MIME type),
/// saves it to local disk, and points the caller's own `avatar_url` at it.
pub async fn upload_avatar(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<UserResponse>> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("gagal membaca upload: {e}")))?
        .ok_or_else(|| AppError::Validation("tidak ada file yang diunggah".to_string()))?;
    let url = save_upload(field, &AVATAR_POLICY).await?;
    let user = state.auth_service.set_avatar(auth.user_id, Some(url)).await?;
    Ok(Json(user.into()))
}

/// Self-service password change — requires the caller to submit their current password.
pub async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> AppResult<Json<MessageResponse>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    state
        .auth_service
        .change_password(
            auth.user_id,
            ChangePasswordInput { current_password: payload.current_password, new_password: payload.new_password },
        )
        .await?;

    Ok(Json(MessageResponse { message: "password berhasil diubah".to_string() }))
}
