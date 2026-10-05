//! "Laporan" (School report-builder + saved reports) domain types. Every saved `Report` is
//! a real snapshot: its `payload` is computed once at generation time from the school's own
//! roster/attempts (Performance, Participation) or from the already-real rationalization
//! aggregates (PtnTarget) — never fabricated. Re-opening an old report shows exactly what
//! was true when it was generated, not a live recomputation, which is why the payload is
//! persisted rather than derived on every read.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::analytics::SubjectAccuracy;
use crate::domain::package::ExamTrack;
use crate::domain::school::SchoolType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReportType {
    Performance,
    Participation,
    PtnTarget,
    /// See `TkaReportPayload` doc comment.
    Tka,
    /// See `AkreditasiReportPayload` doc comment.
    Akreditasi,
}

impl ReportType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportType::Performance => "performance",
            ReportType::Participation => "participation",
            ReportType::PtnTarget => "ptn-target",
            ReportType::Tka => "tka",
            ReportType::Akreditasi => "akreditasi",
        }
    }
}

impl std::str::FromStr for ReportType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "performance" => Ok(ReportType::Performance),
            "participation" => Ok(ReportType::Participation),
            "ptn-target" => Ok(ReportType::PtnTarget),
            "tka" => Ok(ReportType::Tka),
            "akreditasi" => Ok(ReportType::Akreditasi),
            other => Err(format!("jenis laporan tidak dikenal: {other}")),
        }
    }
}

// ─── Performance payload ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassAvgRow {
    pub grade: String,
    pub student_count: i64,
    pub avg_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyTrendPoint {
    pub month_label: String,
    pub avg_score: Option<f64>,
    pub attempt_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingRow {
    pub student_id: Uuid,
    pub student_name: String,
    pub grade: Option<String>,
    pub attempt_count: i64,
    pub best_score: Option<i32>,
    pub avg_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePayload {
    pub total_students: i64,
    pub avg_score: Option<f64>,
    pub per_class: Vec<ClassAvgRow>,
    /// Whole-history monthly series (up to the last 6 months with data) so the trend stays
    /// meaningful regardless of which single month/period was selected for the report.
    pub monthly_trend: Vec<MonthlyTrendPoint>,
    /// Top 10 by average score.
    pub ranking: Vec<RankingRow>,
}

// ─── Participation payload ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationRow {
    pub student_id: Uuid,
    pub student_name: String,
    pub grade: Option<String>,
    pub attempt_count: i64,
    pub last_attempt_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationPayload {
    pub total_students: i64,
    pub active_students: i64,
    pub participation_rate: f64,
    pub total_attempts: i64,
    pub avg_attempts_per_active_student: f64,
    pub rows: Vec<ParticipationRow>,
}

// ─── PTN Target payload (reuses the already-real rationalization aggregates) ────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversityCountSnapshot {
    pub nama_ptn: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtnTargetPayload {
    pub total_siswa_dengan_target: i64,
    pub distribusi_universitas: Vec<UniversityCountSnapshot>,
    pub snbt_terdaftar: i64,
    pub snbt_sudah_ujian: i64,
    pub snbt_diterima: i64,
    pub avg_chance_snbp: Option<f64>,
}

// ─── TKA payload (reuses SchoolTkaService::tka_roster's aggregate) ─────────────────

/// One row per TKA package with real content (see `application::school_tka_service` doc
/// comment for what qualifies) — package-level rollup for the report's summary section.
/// `avg_wajib_score`/`avg_pilihan_score` are on the jenjang-appropriate honest display-only
/// rescale (see `application::school_tka_service::tka_scale_for` doc comment — 0-100 for
/// SD/SMP, 200-800 for SMA/SMK/MA) — never a real IRT-calibrated score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TkaPackageSummaryRow {
    pub package_id: Uuid,
    pub package_name: String,
    pub elective_pick_count: i32,
    pub student_count: i64,
    /// Students who have picked at least `elective_pick_count` mapel pilihan (always equal to
    /// `student_count` when `elective_pick_count == 0` — nothing to pick).
    pub elective_ready_count: i64,
    /// Jenjang paket ini — lihat `application::school_tka_service::TkaPackageGroup` doc comment.
    /// `#[serde(default)]`: laporan TKA lama yang sudah tersimpan sebagai JSONB sebelum field
    /// ini ada akan tetap terbaca (jatuh ke `None`) alih-alih gagal deserialize.
    #[serde(default)]
    pub school_type_scope: Option<SchoolType>,
    /// Label skala tampilan untuk paket ini, mis. "0–100" atau "200–800". Default ke skala
    /// SMA/SMK/MA untuk kompatibilitas laporan lama (lihat `school_type_scope` doc comment).
    #[serde(default = "default_score_scale_label")]
    pub score_scale_label: String,
    /// Ambang nilai kategori "Istimewa" untuk paket ini (95 atau 725). Default 725 untuk
    /// kompatibilitas laporan lama.
    #[serde(default = "default_istimewa_threshold")]
    pub istimewa_threshold: i32,
    pub avg_wajib_score: Option<f64>,
    pub avg_pilihan_score: Option<f64>,
    /// Rata-rata skor mentah platform (skala 0-1000, "Skor Instan") — info pendukung
    /// transparansi (dari mana angka skala TKA di atas diturunkan), BUKAN skor resmi TKA.
    #[serde(default)]
    pub avg_raw_wajib_score: Option<f64>,
    #[serde(default)]
    pub avg_raw_pilihan_score: Option<f64>,
    /// Average count of mata uji per student scoring in the "Istimewa" category (see
    /// `istimewa_threshold` above) — a rough class-level signal, not a substitute for the
    /// per-student breakdown below.
    pub avg_istimewa_count: Option<f64>,
}

/// One row per student, flattened across every TKA package they belong to (a student normally
/// belongs to exactly one, for their own jenjang).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TkaStudentReportRow {
    pub student_id: Uuid,
    pub student_name: String,
    pub grade: Option<String>,
    pub package_name: String,
    /// Jenjang paket ini — sama untuk semua siswa dalam paket yang sama, diulang di sini
    /// supaya frontend tidak perlu join balik ke `TkaPackageSummaryRow`.
    #[serde(default)]
    pub school_type_scope: Option<SchoolType>,
    #[serde(default = "default_score_scale_label")]
    pub score_scale_label: String,
    #[serde(default = "default_istimewa_threshold")]
    pub istimewa_threshold: i32,
    pub elective_chosen_count: i32,
    pub elective_pick_count: i32,
    pub avg_wajib_score: Option<f64>,
    pub avg_pilihan_score: Option<f64>,
    /// Rata-rata skor mentah platform (skala 0-1000, "Skor Instan") — lihat
    /// `TkaPackageSummaryRow::avg_raw_wajib_score` doc comment.
    #[serde(default)]
    pub avg_raw_wajib_score: Option<f64>,
    #[serde(default)]
    pub avg_raw_pilihan_score: Option<f64>,
    pub istimewa_count: i32,
    pub subject_count: i32,
}

fn default_score_scale_label() -> String {
    "200–800".to_string()
}

fn default_istimewa_threshold() -> i32 {
    725
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TkaReportPayload {
    pub total_students: i64,
    pub packages: Vec<TkaPackageSummaryRow>,
    pub students: Vec<TkaStudentReportRow>,
}

// ─── Akreditasi (EDS) payload ────────────────────────────────────────────────────────
// Real academic-performance summary a school can use to fill in accreditation (EDS)
// documentation — NOT a full Dapodik student-identity export (this system doesn't hold
// facilities/teacher/curriculum data needed for that). "KKM"/passing-grade is not an
// existing platform concept, so `threshold_used` is supplied per-generation (see
// `application::report_service::build_akreditasi` doc comment) and persisted here so an
// old snapshot stays self-explanatory even if a school later regenerates with a different
// cutoff. `per_subject` reuses the real, already-scoped `SubjectAccuracy` aggregate from
// `AnalyticsService::subject_breakdown_for_school` (accuracy, not average score — there is
// no per-subject average *score* query in this system) rather than re-deriving it.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AkreditasiClassRow {
    pub grade: String,
    pub student_count: i64,
    pub avg_score: Option<f64>,
    pub above_threshold_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AkreditasiStudentRow {
    pub student_id: Uuid,
    pub student_name: String,
    pub grade: Option<String>,
    pub attempt_count: i64,
    pub avg_score: Option<f64>,
    pub above_threshold: bool,
    /// Punya minimal 1 target PTN terukur (SNBP dan/atau SNBT) — indikator "kesiapan
    /// lulusan", dari `SchoolRationalizationService::full_ranking`/`snbt_roster`.
    pub has_ptn_target: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AkreditasiReportPayload {
    pub total_students: i64,
    pub threshold_used: i32,
    pub avg_score_overall: Option<f64>,
    pub students_above_threshold: i64,
    /// Union of students with an SNBP and/or SNBT target — see `AkreditasiStudentRow::has_ptn_target`.
    pub students_with_ptn_target: i64,
    /// Whole-school only (no class breakdown) — see module doc comment above.
    pub per_subject: Vec<SubjectAccuracy>,
    pub per_class: Vec<AkreditasiClassRow>,
    pub students: Vec<AkreditasiStudentRow>,
}

// ─── Envelope + saved report ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ReportPayload {
    Performance(PerformancePayload),
    Participation(ParticipationPayload),
    PtnTarget(PtnTargetPayload),
    Tka(TkaReportPayload),
    Akreditasi(AkreditasiReportPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub school_id: Uuid,
    pub report_type: ReportType,
    pub title: String,
    pub period_label: String,
    /// `None` = whole school. PtnTarget reports always ignore this (see module doc).
    pub class_filter: Option<String>,
    /// `None` = "Semua Jalur" (SNBT + TKA blended, the historical default). Only meaningful
    /// for `Performance`/`Participation` — `PtnTarget`, `Tka`, and `Akreditasi` reports
    /// always ignore this (a TKA report is implicitly TKA-only; PtnTarget/Akreditasi have
    /// no exam-track dimension at all).
    pub exam_track_filter: Option<ExamTrack>,
    pub payload: ReportPayload,
    pub created_at: DateTime<Utc>,
}
