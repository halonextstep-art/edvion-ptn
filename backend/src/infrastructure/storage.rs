//! S3-compatible object storage for user uploads (avatar photos, achievement certificates,
//! question-import images, package icons/banners). Uploads go straight to the configured
//! bucket (Wasabi by default — see `S3_*` env vars) instead of local disk, so files survive
//! container restarts/redeploys and don't depend on a persistent volume. Callers only ever
//! see the returned public URL, never a filesystem path or bucket key — swapping providers
//! later only means changing this one module.
use std::sync::OnceLock;

use axum::extract::multipart::Field;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

struct S3Config {
    client: Client,
    bucket: String,
    /// Public base URL files are served from (e.g. a CDN domain in front of the bucket),
    /// distinct from the S3 API endpoint used to upload — kept separate since Wasabi's
    /// upload endpoint isn't necessarily the nicest URL to hand back to end users.
    public_base_url: String,
}

static S3: OnceLock<S3Config> = OnceLock::new();

async fn s3() -> &'static S3Config {
    if let Some(cfg) = S3.get() {
        return cfg;
    }

    let endpoint = std::env::var("S3_ENDPOINT").expect("S3_ENDPOINT must be set");
    let access_key = std::env::var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY must be set");
    let secret_key = std::env::var("S3_SECRET_KEY").expect("S3_SECRET_KEY must be set");
    let bucket = std::env::var("S3_BUCKET_NAME").expect("S3_BUCKET_NAME must be set");
    let region = std::env::var("S3_REGION").expect("S3_REGION must be set");
    let public_base_url = std::env::var("S3_PUBLIC_URL").unwrap_or_else(|_| format!("{endpoint}/{bucket}"));

    let credentials = aws_sdk_s3::config::Credentials::new(access_key, secret_key, None, None, "ptn-env");
    let config = aws_sdk_s3::config::Builder::new()
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .endpoint_url(endpoint)
        .region(aws_sdk_s3::config::Region::new(region))
        .credentials_provider(credentials)
        // Wasabi (and most S3-compatible providers) expect path-style requests
        // (`endpoint/bucket/key`) rather than AWS's virtual-hosted-style default.
        .force_path_style(true)
        .build();

    let client = Client::from_conf(config);
    let _ = S3.set(S3Config { client, bucket, public_base_url });
    S3.get().expect("S3 config just set")
}

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

/// Reads one multipart field, validates it against `policy`, and uploads it to the
/// `{policy.subdir}/{uuid}.{ext}` object key. Returns the public URL clients should use.
pub async fn save_upload(field: Field<'_>, policy: &UploadPolicy) -> AppResult<String> {
    let content_type = field.content_type().map(|s| s.to_string()).unwrap_or_default();
    let bytes = field.bytes().await.map_err(|e| AppError::Validation(format!("gagal membaca file: {e}")))?;
    save_bytes(&bytes, &content_type, policy).await
}

/// Same validation/upload behavior as `save_upload`, but for bytes already in memory rather
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

    let cfg = s3().await;
    let key = format!("{}/{}.{}", policy.subdir, Uuid::new_v4(), extension_for(content_type));

    cfg.client
        .put_object()
        .bucket(&cfg.bucket)
        .key(&key)
        .body(ByteStream::from(bytes.to_vec()))
        .content_type(content_type)
        .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("gagal mengunggah file ke object storage: {e}")))?;

    Ok(format!("{}/{}", cfg.public_base_url.trim_end_matches('/'), key))
}
