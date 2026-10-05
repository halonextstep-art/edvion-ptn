use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::application::report_service::GenerateReportInput;
use crate::domain::analytics::SubjectAccuracy;
use crate::domain::package::ExamTrack;
use crate::domain::school::SchoolType;
use crate::domain::report::{
    AkreditasiClassRow, AkreditasiReportPayload, AkreditasiStudentRow, ClassAvgRow, MonthlyTrendPoint,
    ParticipationPayload, ParticipationRow, PerformancePayload, PtnTargetPayload, RankingRow, Report, ReportPayload,
    TkaPackageSummaryRow, TkaReportPayload, TkaStudentReportRow, UniversityCountSnapshot,
};
use crate::interfaces::http::dto::rationalization_dto::UniversityCountResponse;

// ─── Request ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct GenerateReportPayload {
    /// "performance" | "participation" | "ptn-target" | "tka" | "akreditasi"
    #[validate(length(min = 1, message = "jenis laporan wajib diisi"))]
    pub report_type: String,
    /// Human label shown on the saved report, e.g. "Juli 2026" or "Semua Waktu".
    #[validate(length(min = 1, max = 100, message = "periode wajib diisi"))]
    pub period_label: String,
    /// "YYYY-MM", `None` = whole history.
    #[serde(default)]
    pub year_month: Option<String>,
    #[serde(default)]
    pub class_filter: Option<String>,
    /// See `domain::report::Report::exam_track_filter` doc comment. `null`/omitted = "Semua
    /// Jalur". Only meaningful for `report_type` "performance"/"participation".
    #[serde(default)]
    pub exam_track_filter: Option<ExamTrack>,
    /// KKM-like cutoff, only meaningful for `report_type` "akreditasi" — see
    /// `application::report_service::GenerateReportInput::threshold` doc comment.
    #[serde(default)]
    pub threshold: Option<i32>,
}

impl GenerateReportPayload {
    pub fn into_input(self, report_type: crate::domain::report::ReportType) -> GenerateReportInput {
        GenerateReportInput {
            report_type,
            period_label: self.period_label,
            year_month: self.year_month,
            class_filter: self.class_filter,
            exam_track_filter: self.exam_track_filter,
            threshold: self.threshold,
        }
    }
}

// ─── Response ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ClassAvgRowResponse {
    pub grade: String,
    pub student_count: i64,
    pub avg_score: Option<f64>,
}
impl From<ClassAvgRow> for ClassAvgRowResponse {
    fn from(r: ClassAvgRow) -> Self {
        Self { grade: r.grade, student_count: r.student_count, avg_score: r.avg_score }
    }
}

#[derive(Debug, Serialize)]
pub struct MonthlyTrendPointResponse {
    pub month_label: String,
    pub avg_score: Option<f64>,
    pub attempt_count: i64,
}
impl From<MonthlyTrendPoint> for MonthlyTrendPointResponse {
    fn from(r: MonthlyTrendPoint) -> Self {
        Self { month_label: r.month_label, avg_score: r.avg_score, attempt_count: r.attempt_count }
    }
}

#[derive(Debug, Serialize)]
pub struct RankingRowResponse {
    pub student_id: Uuid,
    pub student_name: String,
    pub grade: Option<String>,
    pub attempt_count: i64,
    pub best_score: Option<i32>,
    pub avg_score: Option<f64>,
}
impl From<RankingRow> for RankingRowResponse {
    fn from(r: RankingRow) -> Self {
        Self {
            student_id: r.student_id,
            student_name: r.student_name,
            grade: r.grade,
            attempt_count: r.attempt_count,
            best_score: r.best_score,
            avg_score: r.avg_score,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PerformancePayloadResponse {
    pub total_students: i64,
    pub avg_score: Option<f64>,
    pub per_class: Vec<ClassAvgRowResponse>,
    pub monthly_trend: Vec<MonthlyTrendPointResponse>,
    pub ranking: Vec<RankingRowResponse>,
}
impl From<PerformancePayload> for PerformancePayloadResponse {
    fn from(p: PerformancePayload) -> Self {
        Self {
            total_students: p.total_students,
            avg_score: p.avg_score,
            per_class: p.per_class.into_iter().map(Into::into).collect(),
            monthly_trend: p.monthly_trend.into_iter().map(Into::into).collect(),
            ranking: p.ranking.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ParticipationRowResponse {
    pub student_id: Uuid,
    pub student_name: String,
    pub grade: Option<String>,
    pub attempt_count: i64,
    pub last_attempt_at: Option<DateTime<Utc>>,
}
impl From<ParticipationRow> for ParticipationRowResponse {
    fn from(r: ParticipationRow) -> Self {
        Self {
            student_id: r.student_id,
            student_name: r.student_name,
            grade: r.grade,
            attempt_count: r.attempt_count,
            last_attempt_at: r.last_attempt_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ParticipationPayloadResponse {
    pub total_students: i64,
    pub active_students: i64,
    pub participation_rate: f64,
    pub total_attempts: i64,
    pub avg_attempts_per_active_student: f64,
    pub rows: Vec<ParticipationRowResponse>,
}
impl From<ParticipationPayload> for ParticipationPayloadResponse {
    fn from(p: ParticipationPayload) -> Self {
        Self {
            total_students: p.total_students,
            active_students: p.active_students,
            participation_rate: p.participation_rate,
            total_attempts: p.total_attempts,
            avg_attempts_per_active_student: p.avg_attempts_per_active_student,
            rows: p.rows.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<UniversityCountSnapshot> for UniversityCountResponse {
    fn from(u: UniversityCountSnapshot) -> Self {
        Self { nama_ptn: u.nama_ptn, count: u.count }
    }
}

#[derive(Debug, Serialize)]
pub struct PtnTargetPayloadResponse {
    pub total_siswa_dengan_target: i64,
    pub distribusi_universitas: Vec<UniversityCountResponse>,
    pub snbt_terdaftar: i64,
    pub snbt_sudah_ujian: i64,
    pub snbt_diterima: i64,
    pub avg_chance_snbp: Option<f64>,
}
impl From<PtnTargetPayload> for PtnTargetPayloadResponse {
    fn from(p: PtnTargetPayload) -> Self {
        Self {
            total_siswa_dengan_target: p.total_siswa_dengan_target,
            distribusi_universitas: p.distribusi_universitas.into_iter().map(Into::into).collect(),
            snbt_terdaftar: p.snbt_terdaftar,
            snbt_sudah_ujian: p.snbt_sudah_ujian,
            snbt_diterima: p.snbt_diterima,
            avg_chance_snbp: p.avg_chance_snbp,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TkaPackageSummaryRowResponse {
    pub package_id: Uuid,
    pub package_name: String,
    pub elective_pick_count: i32,
    pub student_count: i64,
    pub elective_ready_count: i64,
    pub school_type_scope: Option<SchoolType>,
    pub score_scale_label: String,
    pub istimewa_threshold: i32,
    pub avg_wajib_score: Option<f64>,
    pub avg_pilihan_score: Option<f64>,
    pub avg_raw_wajib_score: Option<f64>,
    pub avg_raw_pilihan_score: Option<f64>,
    pub avg_istimewa_count: Option<f64>,
}
impl From<TkaPackageSummaryRow> for TkaPackageSummaryRowResponse {
    fn from(r: TkaPackageSummaryRow) -> Self {
        Self {
            package_id: r.package_id,
            package_name: r.package_name,
            elective_pick_count: r.elective_pick_count,
            student_count: r.student_count,
            elective_ready_count: r.elective_ready_count,
            school_type_scope: r.school_type_scope,
            score_scale_label: r.score_scale_label,
            istimewa_threshold: r.istimewa_threshold,
            avg_wajib_score: r.avg_wajib_score,
            avg_pilihan_score: r.avg_pilihan_score,
            avg_raw_wajib_score: r.avg_raw_wajib_score,
            avg_raw_pilihan_score: r.avg_raw_pilihan_score,
            avg_istimewa_count: r.avg_istimewa_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TkaStudentReportRowResponse {
    pub student_id: Uuid,
    pub student_name: String,
    pub grade: Option<String>,
    pub package_name: String,
    pub school_type_scope: Option<SchoolType>,
    pub score_scale_label: String,
    pub istimewa_threshold: i32,
    pub elective_chosen_count: i32,
    pub elective_pick_count: i32,
    pub avg_wajib_score: Option<f64>,
    pub avg_pilihan_score: Option<f64>,
    pub avg_raw_wajib_score: Option<f64>,
    pub avg_raw_pilihan_score: Option<f64>,
    pub istimewa_count: i32,
    pub subject_count: i32,
}
impl From<TkaStudentReportRow> for TkaStudentReportRowResponse {
    fn from(r: TkaStudentReportRow) -> Self {
        Self {
            student_id: r.student_id,
            student_name: r.student_name,
            grade: r.grade,
            package_name: r.package_name,
            school_type_scope: r.school_type_scope,
            score_scale_label: r.score_scale_label,
            istimewa_threshold: r.istimewa_threshold,
            elective_chosen_count: r.elective_chosen_count,
            elective_pick_count: r.elective_pick_count,
            avg_wajib_score: r.avg_wajib_score,
            avg_pilihan_score: r.avg_pilihan_score,
            avg_raw_wajib_score: r.avg_raw_wajib_score,
            avg_raw_pilihan_score: r.avg_raw_pilihan_score,
            istimewa_count: r.istimewa_count,
            subject_count: r.subject_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TkaReportPayloadResponse {
    pub total_students: i64,
    pub packages: Vec<TkaPackageSummaryRowResponse>,
    pub students: Vec<TkaStudentReportRowResponse>,
}
impl From<TkaReportPayload> for TkaReportPayloadResponse {
    fn from(p: TkaReportPayload) -> Self {
        Self {
            total_students: p.total_students,
            packages: p.packages.into_iter().map(Into::into).collect(),
            students: p.students.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AkreditasiClassRowResponse {
    pub grade: String,
    pub student_count: i64,
    pub avg_score: Option<f64>,
    pub above_threshold_count: i64,
}
impl From<AkreditasiClassRow> for AkreditasiClassRowResponse {
    fn from(r: AkreditasiClassRow) -> Self {
        Self { grade: r.grade, student_count: r.student_count, avg_score: r.avg_score, above_threshold_count: r.above_threshold_count }
    }
}

#[derive(Debug, Serialize)]
pub struct AkreditasiStudentRowResponse {
    pub student_id: Uuid,
    pub student_name: String,
    pub grade: Option<String>,
    pub attempt_count: i64,
    pub avg_score: Option<f64>,
    pub above_threshold: bool,
    pub has_ptn_target: bool,
}
impl From<AkreditasiStudentRow> for AkreditasiStudentRowResponse {
    fn from(r: AkreditasiStudentRow) -> Self {
        Self {
            student_id: r.student_id,
            student_name: r.student_name,
            grade: r.grade,
            attempt_count: r.attempt_count,
            avg_score: r.avg_score,
            above_threshold: r.above_threshold,
            has_ptn_target: r.has_ptn_target,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AkreditasiPayloadResponse {
    pub total_students: i64,
    pub threshold_used: i32,
    pub avg_score_overall: Option<f64>,
    pub students_above_threshold: i64,
    pub students_with_ptn_target: i64,
    pub per_subject: Vec<SubjectAccuracy>,
    pub per_class: Vec<AkreditasiClassRowResponse>,
    pub students: Vec<AkreditasiStudentRowResponse>,
}
impl From<AkreditasiReportPayload> for AkreditasiPayloadResponse {
    fn from(p: AkreditasiReportPayload) -> Self {
        Self {
            total_students: p.total_students,
            threshold_used: p.threshold_used,
            avg_score_overall: p.avg_score_overall,
            students_above_threshold: p.students_above_threshold,
            students_with_ptn_target: p.students_with_ptn_target,
            per_subject: p.per_subject,
            per_class: p.per_class.into_iter().map(Into::into).collect(),
            students: p.students.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ReportPayloadResponse {
    Performance(PerformancePayloadResponse),
    Participation(ParticipationPayloadResponse),
    PtnTarget(PtnTargetPayloadResponse),
    Tka(TkaReportPayloadResponse),
    Akreditasi(AkreditasiPayloadResponse),
}
impl From<ReportPayload> for ReportPayloadResponse {
    fn from(p: ReportPayload) -> Self {
        match p {
            ReportPayload::Performance(p) => ReportPayloadResponse::Performance(p.into()),
            ReportPayload::Participation(p) => ReportPayloadResponse::Participation(p.into()),
            ReportPayload::PtnTarget(p) => ReportPayloadResponse::PtnTarget(p.into()),
            ReportPayload::Tka(p) => ReportPayloadResponse::Tka(p.into()),
            ReportPayload::Akreditasi(p) => ReportPayloadResponse::Akreditasi(p.into()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub id: Uuid,
    pub school_id: Uuid,
    pub report_type: String,
    pub title: String,
    pub period_label: String,
    pub class_filter: Option<String>,
    pub exam_track_filter: Option<ExamTrack>,
    pub payload: ReportPayloadResponse,
    pub created_at: DateTime<Utc>,
}
impl From<Report> for ReportResponse {
    fn from(r: Report) -> Self {
        Self {
            id: r.id,
            school_id: r.school_id,
            report_type: r.report_type.as_str().to_string(),
            title: r.title,
            period_label: r.period_label,
            class_filter: r.class_filter,
            exam_track_filter: r.exam_track_filter,
            payload: r.payload.into(),
            created_at: r.created_at,
        }
    }
}

/// Lightweight row for the "Laporan Tersimpan" list — omits the (potentially large)
/// `payload` since the list view only needs metadata.
#[derive(Debug, Serialize)]
pub struct ReportSummaryResponse {
    pub id: Uuid,
    pub report_type: String,
    pub title: String,
    pub period_label: String,
    pub class_filter: Option<String>,
    pub exam_track_filter: Option<ExamTrack>,
    pub created_at: DateTime<Utc>,
}
impl From<Report> for ReportSummaryResponse {
    fn from(r: Report) -> Self {
        Self {
            id: r.id,
            report_type: r.report_type.as_str().to_string(),
            title: r.title,
            period_label: r.period_label,
            class_filter: r.class_filter,
            exam_track_filter: r.exam_track_filter,
            created_at: r.created_at,
        }
    }
}
