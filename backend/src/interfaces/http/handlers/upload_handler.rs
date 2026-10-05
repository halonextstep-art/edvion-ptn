//! Generic small-image upload endpoint — currently backs the Badge/Package icon picker's
//! "Upload Gambar" tab (see `frontend/components/shared/IconPicker.vue` and
//! `infrastructure::storage::ICON_POLICY`). Deliberately decoupled from any specific
//! badge/package id, mirroring `auth_handler::upload_avatar`: the frontend uploads the
//! file first, gets back a URL, then submits that URL as part of the badge/package
//! create/update payload's `icon_url` field (with `icon_type: "custom"`) — no dedicated
//! service layer needed, same as the avatar upload path.

use axum::{extract::Multipart, Json};
use serde::Serialize;

use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::infrastructure::storage::{save_upload, BANNER_POLICY, ICON_POLICY, QUESTION_IMAGE_POLICY};
use crate::interfaces::http::middleware::AuthUser;

#[derive(Debug, Serialize)]
pub struct IconUploadResponse {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct QuestionImageUploadResponse {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct BannerUploadResponse {
    pub url: String,
}

/// Admin-only — badges and packages are both admin-managed entities (see
/// `GamificationService::create_badge` and `PackageService::create`, which already both
/// require `Role::Admin`), so the upload step that feeds their `icon_url` is gated the
/// same way to keep authorization consistent with who can actually attach the result.
pub async fn upload_icon(auth: AuthUser, mut multipart: Multipart) -> AppResult<Json<IconUploadResponse>> {
    auth.require_role(&[Role::Admin])?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("gagal membaca upload: {e}")))?
        .ok_or_else(|| AppError::Validation("tidak ada file yang diunggah".to_string()))?;
    let url = save_upload(field, &ICON_POLICY).await?;
    Ok(Json(IconUploadResponse { url }))
}

/// Backs the "Menjodohkan Gambar" authoring UI: an author uploads an image for one side of a
/// pair, gets back a URL, then that URL becomes one half of the `"kiri::kanan"` option string
/// submitted with the question payload — same decoupled upload-then-reference pattern as
/// `upload_icon` and `auth_handler::upload_avatar`. Gated to Admin + Content, matching who may
/// author questions at all (`QuestionService::create`/`update`).
pub async fn upload_question_image(auth: AuthUser, mut multipart: Multipart) -> AppResult<Json<QuestionImageUploadResponse>> {
    auth.require_role(&[Role::Admin, Role::Content])?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("gagal membaca upload: {e}")))?
        .ok_or_else(|| AppError::Validation("tidak ada file yang diunggah".to_string()))?;
    let url = save_upload(field, &QUESTION_IMAGE_POLICY).await?;
    Ok(Json(QuestionImageUploadResponse { url }))
}

/// Admin-only, mirroring `upload_icon` — backs the Package form's optional wide banner image
/// (`Package::banner_url`, see `BannerPicker.vue`). Same decoupled upload-then-reference
/// pattern: frontend uploads first, gets a URL, then submits it as part of the package
/// create/update payload.
pub async fn upload_banner(auth: AuthUser, mut multipart: Multipart) -> AppResult<Json<BannerUploadResponse>> {
    auth.require_role(&[Role::Admin])?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("gagal membaca upload: {e}")))?
        .ok_or_else(|| AppError::Validation("tidak ada file yang diunggah".to_string()))?;
    let url = save_upload(field, &BANNER_POLICY).await?;
    Ok(Json(BannerUploadResponse { url }))
}
