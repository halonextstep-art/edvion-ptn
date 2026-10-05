//! Local-disk file storage for user uploads (avatar photos, achievement certificates).
//! Deliberately the simplest thing that works: files are written straight to a local
//! `uploads/` directory next to wherever the backend binary runs, and served back by
//! `tower_http::services::ServeDir` mounted at `/uploads` in `router.rs`. No S3/cloud
//! dependency — sufficient at current scale, and swapping to S3-compatible storage later
//! only means changing this one module (callers only ever see a returned URL path).
use std::path::PathBuf;

use axum::extract::multipart::Field;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Root directory uploads are written to, relative to the process's working directory.
const UPLOAD_ROOT: &str = "uploads";

/// One allowed upload kind, with its own size cap and accepted MIME types — kept explicit
/// per call site rather than one shared limit, since a certificate scan is reasonably
/// larger than a profile photo.
pub struct UploadPolicy {
    pub subdir: &'static str,
    pub max_bytes: usize,
    pub allowed_mime: &'static [&'static str],
}

pub const AVATAR_POLICY: UploadPolicy =
    UploadPolicy { subdir: "avatars", max_bytes: 2 * 1024 * 1024, allowed_mime: &["image/png", "image/jpeg", "image/webp"] };

pub const CERTIFICATE_POLICY: UploadPolicy = UploadPolicy {
    subdir: "certificates",
    max_bytes: 8 * 1024 * 1024,
    allowed_mime: &["image/png", "image/jpeg", "image/webp", "application/pdf"],
};

/// Custom Badge/Package icon uploads (the "Upload Gambar" tab of the shared icon picker —
/// see `frontend/components/shared/IconPicker.vue`). Small cap since these render at icon
/// size, never full-page.
pub const ICON_POLICY: UploadPolicy = UploadPolicy {
    subdir: "icons",
    max_bytes: 1024 * 1024,
    allowed_mime: &["image/png", "image/jpeg", "image/webp", "image/svg+xml"],
};

/// Custom Package banner uploads (`frontend/components/shared/BannerPicker.vue`) — a wide
/// promotional image shown at the top of a package card, distinct from the small square icon
/// (`ICON_POLICY`). Larger cap than icons since it renders at near-full card width, not
/// thumbnail size, and no SVG (a promotional photo/graphic, not an icon glyph).
pub const BANNER_POLICY: UploadPolicy = UploadPolicy {
    subdir: "banners",
    max_bytes: 3 * 1024 * 1024,
    allowed_mime: &["image/png", "image/jpeg", "image/webp"],
};

/// Question authoring image uploads — backs `QuestionType::MatchingImage` ("Menjodohkan
/// Gambar"), where each side of a pair is an image URL instead of plain text (see
/// `domain::question::QuestionType::MatchingImage` doc comment). Also reusable for a
/// `stimulus` image on any question type later. Larger cap than icons since these render
/// full-size in the question player, not thumbnail-size.
pub const QUESTION_IMAGE_POLICY: UploadPolicy = UploadPolicy {
    subdir: "question-images",
    max_bytes: 4 * 1024 * 1024,
    allowed_mime: &["image/png", "image/jpeg", "image/webp"],
};

fn extension_for(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "application/pdf" => "pdf",
        "image/svg+xml" => "svg",
        _ => "bin",
    }
}

/// Reads one multipart field, validates it against `policy`, and writes it to disk under
/// `uploads/{policy.subdir}/{uuid}.{ext}`. Returns the URL path clients should use
/// (`/uploads/{policy.subdir}/{filename}`), never a filesystem path.
pub async fn save_upload(field: Field<'_>, policy: &UploadPolicy) -> AppResult<String> {
    let content_type = field.content_type().map(|s| s.to_string()).unwrap_or_default();
    let bytes = field.bytes().await.map_err(|e| AppError::Validation(format!("gagal membaca file: {e}")))?;
    save_bytes(&bytes, &content_type, policy).await
}

/// Same validation/write behavior as `save_upload`, but for bytes already in memory rather
/// than a live multipart field — used by the bulk question-`.docx`-import pipeline
/// (`application::question_import_service`), which extracts image bytes directly out of the
/// uploaded file's `word/media/` folder instead of receiving them as a separate multipart
/// part. `save_upload` is now just this function plus the multipart-specific read step, kept
/// as two functions so callers that genuinely have a `Field` don't need to buffer it
/// themselves first.
pub async fn save_bytes(bytes: &[u8], content_type: &str, policy: &UploadPolicy) -> AppResult<String> {
    if !policy.allowed_mime.contains(&content_type) {
        return Err(AppError::Validation(format!(
            "tipe file '{content_type}' tidak didukung — gunakan salah satu dari: {}",
            policy.allowed_mime.join(", ")
        )));
    }
    if bytes.is_empty() {
        return Err(AppError::Validation("file kosong".to_string()));
    }
    if bytes.len() > policy.max_bytes {
        return Err(AppError::Validation(format!(
            "file terlalu besar ({} KB) — maksimum {} KB",
            bytes.len() / 1024,
            policy.max_bytes / 1024
        )));
    }

    let dir: PathBuf = [UPLOAD_ROOT, policy.subdir].iter().collect();
    tokio::fs::create_dir_all(&dir).await.map_err(|e| AppError::Internal(anyhow::anyhow!("gagal membuat folder upload: {e}")))?;

    let filename = format!("{}.{}", Uuid::new_v4(), extension_for(content_type));
    let path = dir.join(&filename);
    tokio::fs::write(&path, bytes).await.map_err(|e| AppError::Internal(anyhow::anyhow!("gagal menyimpan file: {e}")))?;

    Ok(format!("/uploads/{}/{}", policy.subdir, filename))
}
