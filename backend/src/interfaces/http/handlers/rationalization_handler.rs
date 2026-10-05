use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::domain::rationalization::{AchievementLevel, Priority, SnbpStatus, SnbtTrackingStatus, Track};
use crate::domain::repository::NewAuditLogEntry;
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::rationalization_dto::{
    AchievementPayload, AchievementResponse, AlumniPayload, AlumniRaporBatchPayload, AlumniRaporScoreResponse,
    AlumniResponse, AuditLogEntryResponse, ChanceResponse, EligibilityPayload, PreviewChanceQuery, PriorityPayload,
    ProgramListQuery, ProgramListResponse, PtnProgramPayload, PtnProgramResponse, RaporBatchPayload,
    RaporBulkImportPayload, RaporBulkImportResponse, RaporScoreResponse, SchoolEligibilityResponse,
    SnbpDashboardResponse, SnbpParticipationPayload, SnbpRankingEntryResponse, SnbpRosterRowResponse,
    SnbtDashboardResponse, SnbtRosterRowResponse, SnbtTrackingPayload, TargetPayload, TargetWithChanceResponse,
};
use crate::interfaces::http::dto::school_tka_dto::TkaPackageGroupResponse;
use crate::interfaces::http::dto::tryout_dto::{AttemptResponse, AttemptResultResponse, ReviewItemResponse};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

// ─── PTN Program catalog ────────────────────────────────────────────────────────

pub async fn list_programs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ProgramListQuery>,
) -> AppResult<Json<ProgramListResponse>> {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20);
    let (items, total) = state
        .rationalization_service
        .list_programs(&auth, q.search, q.rumpun, q.jenjang, page, page_size)
        .await?;
    Ok(Json(ProgramListResponse {
        items: items.into_iter().map(Into::into).collect(),
        total,
        page,
        page_size,
    }))
}

pub async fn get_program(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PtnProgramResponse>> {
    let program = state.rationalization_service.get_program(&auth, id).await?;
    Ok(Json(program.into()))
}

pub async fn distinct_rumpun(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<String>>> {
    let rumpun = state.rationalization_service.distinct_rumpun(&auth).await?;
    Ok(Json(rumpun))
}

pub async fn catalog_size(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<serde_json::Value>> {
    let size = state.rationalization_service.catalog_size(&auth).await?;
    Ok(Json(serde_json::json!({ "total": size })))
}

pub async fn create_program(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<PtnProgramPayload>,
) -> AppResult<Json<PtnProgramResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let program = state.rationalization_service.create_program(&auth, payload.into_create_input()).await?;
    Ok(Json(program.into()))
}

pub async fn update_program(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<PtnProgramPayload>,
) -> AppResult<Json<PtnProgramResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let program = state.rationalization_service.update_program(&auth, id, payload.into_update_input()).await?;
    Ok(Json(program.into()))
}

pub async fn delete_program(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.rationalization_service.delete_program(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ─── Rapor scores ───────────────────────────────────────────────────────────────

pub async fn upsert_rapor(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<RaporBatchPayload>,
) -> AppResult<Json<Vec<RaporScoreResponse>>> {
    for entry in &payload.entries {
        entry.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    }
    let inputs = payload.entries.into_iter().map(Into::into).collect();
    let saved = state.rationalization_service.upsert_my_rapor(&auth, inputs).await?;
    Ok(Json(saved.into_iter().map(Into::into).collect()))
}

pub async fn list_rapor(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<RaporScoreResponse>>> {
    let scores = state.rationalization_service.list_my_rapor(&auth).await?;
    Ok(Json(scores.into_iter().map(Into::into).collect()))
}

pub async fn delete_rapor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.rationalization_service.delete_my_rapor(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// Import nilai rapor untuk BANYAK siswa sekaligus dari satu file Excel (Portal Sekolah) —
/// beda dari `upsert_student_rapor` di bawah (yang menyasar satu `student_id` lewat path
/// param), endpoint ini menerima banyak baris yang masing-masing membawa NIS-nya sendiri.
/// Partial success by design: baris dengan NIS tak dikenal/nilai di luar 0-100/mapel kosong
/// dikembalikan sebagai `errors` tanpa menggagalkan baris lain yang valid — lihat doc
/// comment `SchoolRationalizationService::bulk_import_rapor`.
pub async fn bulk_import_rapor(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<RaporBulkImportPayload>,
) -> AppResult<Json<RaporBulkImportResponse>> {
    let entries = payload.entries.into_iter().map(Into::into).collect();
    let result = state.school_rationalization_service.bulk_import_rapor(&auth, entries).await?;
    Ok(Json(result.into()))
}

// ─── Targets + chance ───────────────────────────────────────────────────────────

fn parse_track(s: &str) -> AppResult<Track> {
    s.parse::<Track>().map_err(AppError::Validation)
}
fn parse_priority(s: &str) -> AppResult<Priority> {
    s.parse::<Priority>().map_err(AppError::Validation)
}
fn parse_achievement_level(s: &str) -> AppResult<AchievementLevel> {
    s.parse::<AchievementLevel>().map_err(AppError::Validation)
}
fn parse_snbt_status(s: &str) -> AppResult<SnbtTrackingStatus> {
    s.parse::<SnbtTrackingStatus>().map_err(AppError::Validation)
}
fn parse_snbp_status(s: &str) -> AppResult<SnbpStatus> {
    s.parse::<SnbpStatus>().map_err(AppError::Validation)
}

pub async fn add_target(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<TargetPayload>,
) -> AppResult<Json<serde_json::Value>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let track = parse_track(&payload.track)?;
    let priority = parse_priority(&payload.priority)?;
    let target = state
        .rationalization_service
        .add_target(&auth, payload.ptn_program_id, track, priority)
        .await?;
    Ok(Json(serde_json::json!({ "id": target.id })))
}

pub async fn my_targets(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<TargetWithChanceResponse>>> {
    let results = state.rationalization_service.my_targets_with_chance(&auth).await?;
    Ok(Json(results.into_iter().map(Into::into).collect()))
}

pub async fn set_target_priority(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<PriorityPayload>,
) -> AppResult<Json<serde_json::Value>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let priority = parse_priority(&payload.priority)?;
    state.rationalization_service.set_target_priority(&auth, id, priority).await?;
    Ok(Json(serde_json::json!({ "updated": true })))
}

pub async fn remove_target(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.rationalization_service.remove_target(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// School PIC (own students only) or Admin viewing a specific student's targets+chances —
/// powers the School portal's SNBP Rasionalisasi / Rekap SNBT tabs. Ownership is verified
/// via `SchoolPortalService::ensure_owns_student` before any data is returned.
pub async fn student_targets(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
) -> AppResult<Json<Vec<TargetWithChanceResponse>>> {
    authorize_student_edit(&state, &auth, student_id).await?;
    let results = state.rationalization_service.targets_with_chance_for(student_id).await?;
    Ok(Json(results.into_iter().map(Into::into).collect()))
}

/// Shared authorization for every `/students/:student_id/...` rasionalisasi endpoint below:
/// only that student's own school PIC, or an Admin, may read/write on their behalf — never
/// another school, and never a Student/Content account reaching in for someone else. This is
/// the same check `student_targets` (read-only, pre-existing) already used.
async fn authorize_student_edit(state: &AppState, auth: &AuthUser, student_id: Uuid) -> AppResult<()> {
    if auth.role != Role::Admin {
        state.school_portal_service.ensure_owns_student(auth, student_id).await?;
    }
    Ok(())
}

/// Records one "edit-on-behalf" accountability entry. Every handler below this point is
/// only reachable by a School PIC or Admin acting on a STUDENT's data (never the student
/// themselves — they use the separate `/rapor`, `/achievements`, `/targets` self-service
/// endpoints), so every call here is by definition an on-behalf edit worth logging.
/// Fire-and-forget: a logging failure is only warned about, never allowed to fail the
/// data edit that already succeeded. `actor_name` uses the actor's email (already on
/// `AuthUser`) rather than a separate display-name lookup, to avoid an extra DB round
/// trip on every single edit just for a label.
async fn record_audit(state: &AppState, auth: &AuthUser, student_id: Uuid, action: &str, entity_type: &str, summary: String) {
    let entry = NewAuditLogEntry {
        actor_id: auth.user_id,
        actor_name: auth.email.clone(),
        actor_role: auth.role.as_str().to_string(),
        student_id,
        action: action.to_string(),
        entity_type: entity_type.to_string(),
        summary,
    };
    if let Err(e) = state.audit_log_repo.record(entry).await {
        tracing::warn!("gagal mencatat audit log ({entity_type}/{action}) untuk siswa {student_id}: {e}");
    }
}

/// "Riwayat Perubahan" — lets a student's own school PIC (or Admin) see every
/// edit-on-behalf action recorded against that student, most recent first.
pub async fn student_audit_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
) -> AppResult<Json<Vec<AuditLogEntryResponse>>> {
    authorize_student_edit(&state, &auth, student_id).await?;
    let entries = state.audit_log_repo.list_by_student(student_id, 50).await?;
    Ok(Json(entries.into_iter().map(Into::into).collect()))
}

// ─── School/Admin editing a specific student's SNBP data ────────────────────────
// Lets a school PIC cross-check and correct the exact same rapor/prestasi/target
// records the student entered themselves — not a parallel/duplicate data set. The
// student's own `/rapor`, `/achievements`, `/targets` self-service endpoints above
// remain the primary entry point; these exist purely for the "Daftar Siswa" verify/edit
// panel in the School portal.

pub async fn student_rapor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
) -> AppResult<Json<Vec<RaporScoreResponse>>> {
    authorize_student_edit(&state, &auth, student_id).await?;
    let scores = state.rationalization_service.list_rapor_for(student_id).await?;
    Ok(Json(scores.into_iter().map(Into::into).collect()))
}

pub async fn upsert_student_rapor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
    Json(payload): Json<RaporBatchPayload>,
) -> AppResult<Json<Vec<RaporScoreResponse>>> {
    authorize_student_edit(&state, &auth, student_id).await?;
    for entry in &payload.entries {
        entry.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    }
    let count = payload.entries.len();
    let semesters: Vec<i32> = {
        let mut s: Vec<i32> = payload.entries.iter().map(|e| e.semester).collect();
        s.sort_unstable();
        s.dedup();
        s
    };
    let inputs = payload.entries.into_iter().map(Into::into).collect();
    let saved = state.rationalization_service.upsert_rapor_for(student_id, inputs).await?;
    record_audit(
        &state, &auth, student_id, "upsert", "nilai_rapor",
        format!("Mengisi/memperbarui {count} nilai rapor (semester {semesters:?})"),
    ).await;
    Ok(Json(saved.into_iter().map(Into::into).collect()))
}

pub async fn delete_student_rapor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((student_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    authorize_student_edit(&state, &auth, student_id).await?;
    state.rationalization_service.delete_rapor_for(student_id, id).await?;
    record_audit(&state, &auth, student_id, "delete", "nilai_rapor", "Menghapus 1 entri nilai rapor".to_string()).await;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn student_achievements(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
) -> AppResult<Json<Vec<AchievementResponse>>> {
    authorize_student_edit(&state, &auth, student_id).await?;
    let items = state.rationalization_service.list_achievements_for(student_id).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

pub async fn add_student_achievement(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
    Json(payload): Json<AchievementPayload>,
) -> AppResult<Json<AchievementResponse>> {
    authorize_student_edit(&state, &auth, student_id).await?;
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let tingkat = parse_achievement_level(&payload.tingkat)?;
    let nama = payload.nama.clone();
    let input = payload.into_input(tingkat);
    let created = state.rationalization_service.add_achievement_for(student_id, input).await?;
    record_audit(&state, &auth, student_id, "create", "prestasi", format!("Menambahkan prestasi: {nama}")).await;
    Ok(Json(created.into()))
}

pub async fn delete_student_achievement(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((student_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    authorize_student_edit(&state, &auth, student_id).await?;
    state.rationalization_service.delete_achievement_for(student_id, id).await?;
    record_audit(&state, &auth, student_id, "delete", "prestasi", "Menghapus 1 entri prestasi".to_string()).await;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn add_student_target(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
    Json(payload): Json<TargetPayload>,
) -> AppResult<Json<serde_json::Value>> {
    authorize_student_edit(&state, &auth, student_id).await?;
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let track = parse_track(&payload.track)?;
    let priority = parse_priority(&payload.priority)?;
    let target = state
        .rationalization_service
        .add_target_for(student_id, payload.ptn_program_id, track, priority)
        .await?;
    record_audit(
        &state, &auth, student_id, "create", "target_ptn",
        format!("Menambahkan target PTN ({} — prioritas {})", payload.track, payload.priority),
    ).await;
    Ok(Json(serde_json::json!({ "id": target.id })))
}

pub async fn set_student_target_priority(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((student_id, id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<PriorityPayload>,
) -> AppResult<Json<serde_json::Value>> {
    authorize_student_edit(&state, &auth, student_id).await?;
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let priority = parse_priority(&payload.priority)?;
    state.rationalization_service.set_target_priority_for(student_id, id, priority).await?;
    record_audit(&state, &auth, student_id, "update", "target_ptn", format!("Mengubah prioritas target menjadi {}", payload.priority)).await;
    Ok(Json(serde_json::json!({ "updated": true })))
}

pub async fn remove_student_target(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((student_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    authorize_student_edit(&state, &auth, student_id).await?;
    state.rationalization_service.remove_target_for(student_id, id).await?;
    record_audit(&state, &auth, student_id, "delete", "target_ptn", "Menghapus 1 target PTN".to_string()).await;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn student_preview_chance(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
    Query(q): Query<PreviewChanceQuery>,
) -> AppResult<Json<ChanceResponse>> {
    authorize_student_edit(&state, &auth, student_id).await?;
    let track = parse_track(&q.track)?;
    let chance = state.rationalization_service.preview_chance_for(student_id, q.ptn_program_id, track).await?;
    Ok(Json(chance.into()))
}

pub async fn preview_chance(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<PreviewChanceQuery>,
) -> AppResult<Json<ChanceResponse>> {
    let track = parse_track(&q.track)?;
    let chance = state.rationalization_service.preview_chance(&auth, q.ptn_program_id, track).await?;
    Ok(Json(chance.into()))
}

// ─── School Rasionalisasi: Dashboard / Alumni ("Kaka Kelas") / Eligible ─────────

pub async fn school_dashboard(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<SnbpDashboardResponse>> {
    let dashboard = state.school_rationalization_service.dashboard(&auth).await?;
    Ok(Json(dashboard.into()))
}

/// "Rasionalisasi" sub-tab's full ranking table — every SNBP-targeting student, real chance
/// estimate, sorted best-first.
pub async fn school_full_ranking(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<SnbpRankingEntryResponse>>> {
    let ranking = state.school_rationalization_service.full_ranking(&auth).await?;
    Ok(Json(ranking.into_iter().map(Into::into).collect()))
}

pub async fn list_alumni(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<AlumniResponse>>> {
    let items = state.school_rationalization_service.list_alumni(&auth).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

pub async fn add_alumni(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<AlumniPayload>,
) -> AppResult<Json<AlumniResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let track = parse_track(&payload.track)?;
    let input = payload.into_input(track);
    let created = state.school_rationalization_service.add_alumni(&auth, input).await?;
    Ok(Json(created.into()))
}

pub async fn delete_alumni(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.school_rationalization_service.delete_alumni(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn list_alumni_rapor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(alumni_id): Path<Uuid>,
) -> AppResult<Json<Vec<AlumniRaporScoreResponse>>> {
    let items = state.school_rationalization_service.list_alumni_rapor(&auth, alumni_id).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

pub async fn upsert_alumni_rapor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(alumni_id): Path<Uuid>,
    Json(payload): Json<AlumniRaporBatchPayload>,
) -> AppResult<Json<Vec<AlumniRaporScoreResponse>>> {
    for entry in &payload.entries {
        entry.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    }
    let entries = payload.entries.into_iter().map(|e| e.into_upsert()).collect();
    let saved = state.school_rationalization_service.upsert_alumni_rapor(&auth, alumni_id, entries).await?;
    Ok(Json(saved.into_iter().map(Into::into).collect()))
}

pub async fn delete_alumni_rapor(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((alumni_id, score_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    state.school_rationalization_service.delete_alumni_rapor(&auth, alumni_id, score_id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn list_eligibility(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<SchoolEligibilityResponse>>> {
    let items = state.school_rationalization_service.list_eligibility_with_usage(&auth).await?;
    Ok(Json(items.into_iter().map(|(e, terdaftar)| SchoolEligibilityResponse::from_with_usage(e, terdaftar)).collect()))
}

pub async fn upsert_eligibility(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<EligibilityPayload>,
) -> AppResult<Json<SchoolEligibilityResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let saved = state
        .school_rationalization_service
        .upsert_eligibility(&auth, payload.year, payload.eligible_count)
        .await?;
    // Sandingkan langsung dengan angka terdaftar aktif terkini, supaya respons upsert tidak
    // menyesatkan menampilkan terdaftar_aktif_count=0 padahal roster sekolah itu sudah terisi.
    let with_usage = state.school_rationalization_service.list_eligibility_with_usage(&auth).await?;
    let terdaftar = with_usage.into_iter().find(|(e, _)| e.year == saved.year).map(|(_, t)| t).unwrap_or(0);
    Ok(Json(SchoolEligibilityResponse::from_with_usage(saved, terdaftar)))
}

// ─── Student achievements ("Prestasi") ───────────────────────────────────────────

pub async fn add_achievement(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<AchievementPayload>,
) -> AppResult<Json<AchievementResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let tingkat = parse_achievement_level(&payload.tingkat)?;
    let input = payload.into_input(tingkat);
    let created = state.rationalization_service.add_my_achievement(&auth, input).await?;
    Ok(Json(created.into()))
}

pub async fn list_achievements(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<AchievementResponse>>> {
    let items = state.rationalization_service.list_my_achievements(&auth).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

pub async fn delete_achievement(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.rationalization_service.delete_my_achievement(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// Self-service certificate scan upload for one of the caller's own achievements.
pub async fn upload_achievement_certificate(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    mut multipart: axum::extract::Multipart,
) -> AppResult<Json<AchievementResponse>> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("gagal membaca upload: {e}")))?
        .ok_or_else(|| AppError::Validation("tidak ada file yang diunggah".to_string()))?;
    let url = crate::infrastructure::storage::save_upload(field, &crate::infrastructure::storage::CERTIFICATE_POLICY).await?;
    let updated = state.rationalization_service.set_my_achievement_certificate(&auth, id, Some(url)).await?;
    Ok(Json(updated.into()))
}

// ─── Rekap SNBT ──────────────────────────────────────────────────────────────────

pub async fn school_snbt_dashboard(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<SnbtDashboardResponse>> {
    let dashboard = state.school_rationalization_service.snbt_dashboard(&auth).await?;
    Ok(Json(dashboard.into()))
}

pub async fn school_snbt_roster(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<SnbtRosterRowResponse>>> {
    let roster = state.school_rationalization_service.snbt_roster(&auth).await?;
    Ok(Json(roster.into_iter().map(Into::into).collect()))
}

/// Rekap SNBT drill-down — every real attempt (tryout/drilling/Simulasi UTBK subtes) this
/// student has made, newest first. See `SchoolRationalizationService::student_attempt_history`
/// doc comment.
pub async fn school_student_attempts(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
) -> AppResult<Json<Vec<AttemptResponse>>> {
    let attempts = state.school_rationalization_service.student_attempt_history(&auth, student_id).await?;
    Ok(Json(attempts.into_iter().map(Into::into).collect()))
}

/// Rekap SNBT drill-down, per-TO detail: score + subject breakdown for one specific attempt of
/// this student — see `SchoolRationalizationService::student_attempt_result` doc comment.
/// Powers the "kekuatan/kekurangan per mata uji" section of the "Riwayat Pengerjaan TO" dialog
/// and its "Unduh PDF" export.
pub async fn school_student_attempt_result(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((student_id, attempt_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<AttemptResultResponse>> {
    let result = state.school_rationalization_service.student_attempt_result(&auth, student_id, attempt_id).await?;
    Ok(Json(result.into()))
}

/// Same drill-down, per-question pembahasan — see
/// `SchoolRationalizationService::student_attempt_review` doc comment.
pub async fn school_student_attempt_review(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((student_id, attempt_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Vec<ReviewItemResponse>>> {
    let items = state.school_rationalization_service.student_attempt_review(&auth, student_id, attempt_id).await?;
    Ok(Json(items.into_iter().map(Into::into).collect()))
}

// ─── Rekap TKA ──────────────────────────────────────────────────────────────────

/// See `application::school_tka_service::SchoolTkaService::tka_roster` doc comment.
pub async fn school_tka_roster(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<TkaPackageGroupResponse>>> {
    let groups = state.school_tka_service.tka_roster(&auth).await?;
    Ok(Json(groups.into_iter().map(Into::into).collect()))
}

pub async fn upsert_snbt_tracking(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
    Json(payload): Json<SnbtTrackingPayload>,
) -> AppResult<Json<SnbtRosterRowResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let status = parse_snbt_status(&payload.status)?;
    let status_label = payload.status.clone();
    let actual_score = payload.actual_score;
    let input = payload.into_input(status);
    let updated = state.school_rationalization_service.upsert_snbt_tracking(&auth, student_id, input).await?;
    record_audit(
        &state, &auth, updated.student_id, "update", "snbt_tracking",
        format!("Memperbarui tracking SNBT: status {status_label}, skor aktual {}", actual_score.map(|s| s.to_string()).unwrap_or_else(|| "-".to_string())),
    ).await;
    Ok(Json(updated.into()))
}

// ─── SNBP roster ("Daftar Siswa") ──────────────────────────────────────────────

pub async fn school_snbp_roster(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<SnbpRosterRowResponse>>> {
    let roster = state.school_rationalization_service.snbp_roster(&auth).await?;
    Ok(Json(roster.into_iter().map(Into::into).collect()))
}

pub async fn upsert_snbp_participation(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(student_id): Path<Uuid>,
    Json(payload): Json<SnbpParticipationPayload>,
) -> AppResult<Json<SnbpRosterRowResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let status = parse_snbp_status(&payload.status)?;
    let status_label = payload.status.clone();
    let konsultan = payload.konsultan.clone();
    let input = payload.into_input(status);
    let updated = state.school_rationalization_service.upsert_snbp_participation(&auth, student_id, input).await?;
    record_audit(
        &state, &auth, student_id, "update", "snbp_participation",
        format!("Memperbarui partisipasi SNBP: status {status_label}, konsultan {konsultan}"),
    ).await;
    Ok(Json(updated.into()))
}
