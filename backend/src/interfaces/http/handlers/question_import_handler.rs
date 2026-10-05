//! HTTP surface for the bulk "Import Soal" `.docx` flow — see
//! `application::question_import_service` doc comment for the overall preview/commit design
//! (stateless: `commit` re-reads the same uploaded file bytes rather than relying on anything
//! cached server-side from a prior `preview` call).

use axum::{extract::Multipart, extract::State, Json};
use uuid::Uuid;

use crate::application::question_import_service::ImportBatchMeta;
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::question_dto::{ImportCommitResponse, ImportPreviewResponse};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

/// Read-only: parses the uploaded `.docx` and returns every question it could make sense of
/// (plus a reason for every one it couldn't) — nothing is written to the database or disk.
/// Multipart field: `file`.
pub async fn preview(State(state): State<AppState>, auth: AuthUser, mut multipart: Multipart) -> AppResult<Json<ImportPreviewResponse>> {
    auth.require_role(&[Role::Admin, Role::Content])?;
    let bytes = extract_file_field(&mut multipart).await?;
    let result = state.question_import_service.preview(&bytes)?;
    Ok(Json(result.into()))
}

/// Re-parses the uploaded `.docx` and actually creates every successfully-parsed question.
/// Multipart fields: `file` (required), `subject` (required), `topic`/`subtopic`/`bloom_level`
/// (optional, same defaults as manual authoring), `time_limit` (optional, default 120),
/// `submit_for_review` (`"true"`/`"false"`, default `false`), `question_set_id` (optional
/// UUID).
pub async fn commit(State(state): State<AppState>, auth: AuthUser, multipart: Multipart) -> AppResult<Json<ImportCommitResponse>> {
    let (bytes, meta) = extract_commit_request(multipart).await?;
    let result = state.question_import_service.commit(&auth, &bytes, &meta).await?;
    Ok(Json(result.into()))
}

async fn extract_file_field(multipart: &mut Multipart) -> AppResult<Vec<u8>> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Validation(format!("gagal membaca upload: {e}")))? {
        if field.name() == Some("file") {
            let bytes = field.bytes().await.map_err(|e| AppError::Validation(format!("gagal membaca file: {e}")))?;
            return Ok(bytes.to_vec());
        }
    }
    Err(AppError::Validation("tidak ada file yang diunggah (field 'file')".to_string()))
}

async fn extract_commit_request(mut multipart: Multipart) -> AppResult<(Vec<u8>, ImportBatchMeta)> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut subject: Option<String> = None;
    let mut topic: Option<String> = None;
    let mut subtopic: Option<String> = None;
    let mut bloom_level: Option<String> = None;
    let mut time_limit_raw: Option<String> = None;
    let mut submit_for_review_raw: Option<String> = None;
    let mut question_set_id_raw: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Validation(format!("gagal membaca upload: {e}")))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                let bytes = field.bytes().await.map_err(|e| AppError::Validation(format!("gagal membaca file: {e}")))?;
                file_bytes = Some(bytes.to_vec());
            }
            "subject" => subject = Some(field.text().await.unwrap_or_default()),
            "topic" => topic = Some(field.text().await.unwrap_or_default()),
            "subtopic" => subtopic = Some(field.text().await.unwrap_or_default()),
            "bloom_level" => bloom_level = Some(field.text().await.unwrap_or_default()),
            "time_limit" => time_limit_raw = Some(field.text().await.unwrap_or_default()),
            "submit_for_review" => submit_for_review_raw = Some(field.text().await.unwrap_or_default()),
            "question_set_id" => question_set_id_raw = Some(field.text().await.unwrap_or_default()),
            _ => {}
        }
    }

    let file_bytes = file_bytes.ok_or_else(|| AppError::Validation("tidak ada file yang diunggah (field 'file')".to_string()))?;
    let subject = subject
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| AppError::Validation("field 'subject' wajib diisi".to_string()))?;

    let question_set_id = match question_set_id_raw.filter(|s| !s.trim().is_empty()) {
        Some(raw) => Some(raw.trim().parse::<Uuid>().map_err(|_| AppError::Validation("question_set_id tidak valid".to_string()))?),
        None => None,
    };

    let time_limit = match time_limit_raw.filter(|s| !s.trim().is_empty()) {
        Some(raw) => Some(raw.trim().parse::<i32>().map_err(|_| AppError::Validation("time_limit harus berupa angka".to_string()))?),
        None => Some(120),
    };

    let meta = ImportBatchMeta {
        subject,
        topic: topic.filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "General".to_string()),
        subtopic: subtopic.filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "General".to_string()),
        bloom_level: bloom_level.filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "C3 - Aplikasi".to_string()),
        time_limit,
        submit_for_review: submit_for_review_raw.map(|s| s.trim().eq_ignore_ascii_case("true")).unwrap_or(false),
        question_set_id,
    };

    Ok((file_bytes, meta))
}
