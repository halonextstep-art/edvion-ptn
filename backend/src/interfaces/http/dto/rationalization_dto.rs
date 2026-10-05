use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::application::rationalization_service::{
    AchievementInput, CreatePtnProgramInput, RaporEntryInput, TargetWithChance, UpdatePtnProgramInput,
};
use crate::application::school_rationalization_service::{
    AddAlumniInput, BulkRaporEntryInput, BulkRaporImportResult, SnbpDashboard, SnbpParticipationInput,
    SnbpRankingEntry, SnbpRosterRow, SnbtChoice, SnbtDashboard, SnbtRosterRow, SnbtTrackingInput,
    StudentChanceSummary, UniversityCount,
};
use crate::domain::rationalization::{
    Achievement, AlumniBenchmark, AuditLogEntry, ChanceResult, PtnProgram, PtnTarget, RaporScore, SchoolEligibility,
};

// ─── PTN Program catalog ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct PtnProgramPayload {
    #[validate(length(min = 1, message = "nama PTN wajib diisi"))]
    pub nama_ptn: String,
    #[validate(length(min = 1, message = "nama program studi wajib diisi"))]
    pub nama_prodi: String,
    #[serde(default)]
    pub provinsi: String,
    #[serde(default)]
    pub kota: String,
    #[serde(default)]
    pub singkatan: String,
    #[serde(default)]
    pub rumpun: String,
    #[serde(default)]
    pub mapel_syarat: String,
    #[serde(default)]
    pub daya_tampung_snbp: i32,
    #[serde(default)]
    pub peminat_snbp: i32,
    #[serde(default)]
    pub daya_tampung_snbt: i32,
    #[serde(default)]
    pub peminat_snbt: i32,
    #[serde(default)]
    pub pg_snbt: f64,
    #[serde(default)]
    pub pg_snbp: f64,
    #[serde(default = "default_jenjang")]
    pub jenjang: String,
    /// Set `false` only when adding a program purely from descriptive research, without a
    /// verified official daya-tampung/peminat/passing-grade figure. Defaults to `true`
    /// (assumes the caller is entering real statistics, matching every pre-existing row).
    #[serde(default = "default_true")]
    pub has_official_stats: bool,
}
fn default_jenjang() -> String {
    "S1".to_string()
}

impl PtnProgramPayload {
    pub fn into_create_input(self) -> CreatePtnProgramInput {
        CreatePtnProgramInput {
            nama_ptn: self.nama_ptn,
            nama_prodi: self.nama_prodi,
            provinsi: self.provinsi,
            kota: self.kota,
            singkatan: self.singkatan,
            rumpun: self.rumpun,
            mapel_syarat: self.mapel_syarat,
            daya_tampung_snbp: self.daya_tampung_snbp,
            peminat_snbp: self.peminat_snbp,
            daya_tampung_snbt: self.daya_tampung_snbt,
            peminat_snbt: self.peminat_snbt,
            pg_snbt: self.pg_snbt,
            pg_snbp: self.pg_snbp,
            jenjang: self.jenjang,
            has_official_stats: self.has_official_stats,
        }
    }

    pub fn into_update_input(self) -> UpdatePtnProgramInput {
        UpdatePtnProgramInput {
            nama_ptn: self.nama_ptn,
            nama_prodi: self.nama_prodi,
            provinsi: self.provinsi,
            kota: self.kota,
            singkatan: self.singkatan,
            rumpun: self.rumpun,
            mapel_syarat: self.mapel_syarat,
            daya_tampung_snbp: self.daya_tampung_snbp,
            peminat_snbp: self.peminat_snbp,
            daya_tampung_snbt: self.daya_tampung_snbt,
            peminat_snbt: self.peminat_snbt,
            pg_snbt: self.pg_snbt,
            pg_snbp: self.pg_snbp,
            jenjang: self.jenjang,
            has_official_stats: self.has_official_stats,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PtnProgramResponse {
    pub id: Uuid,
    pub kode: i64,
    pub nama_ptn: String,
    pub nama_prodi: String,
    pub provinsi: String,
    pub kota: String,
    pub singkatan: String,
    pub rumpun: String,
    pub mapel_syarat: String,
    pub daya_tampung_snbp: i32,
    pub peminat_snbp: i32,
    pub daya_tampung_snbt: i32,
    pub peminat_snbt: i32,
    pub pg_snbt: f64,
    pub pg_snbp: f64,
    pub jenjang: String,
    pub has_official_stats: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PtnProgram> for PtnProgramResponse {
    fn from(p: PtnProgram) -> Self {
        Self {
            id: p.id,
            kode: p.kode,
            nama_ptn: p.nama_ptn,
            nama_prodi: p.nama_prodi,
            provinsi: p.provinsi,
            kota: p.kota,
            singkatan: p.singkatan,
            rumpun: p.rumpun,
            mapel_syarat: p.mapel_syarat,
            daya_tampung_snbp: p.daya_tampung_snbp,
            peminat_snbp: p.peminat_snbp,
            daya_tampung_snbt: p.daya_tampung_snbt,
            peminat_snbt: p.peminat_snbt,
            pg_snbt: p.pg_snbt,
            pg_snbp: p.pg_snbp,
            jenjang: p.jenjang,
            has_official_stats: p.has_official_stats,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ProgramListQuery {
    pub search: Option<String>,
    pub rumpun: Option<String>,
    pub jenjang: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ProgramListResponse {
    pub items: Vec<PtnProgramResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

// ─── Rapor scores ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct RaporEntryPayload {
    #[validate(range(min = 1, max = 6, message = "semester harus antara 1-6"))]
    pub semester: i32,
    #[validate(length(min = 1, message = "nama mata pelajaran wajib diisi"))]
    pub subject: String,
    #[validate(range(min = 0.0, max = 100.0, message = "nilai harus 0-100"))]
    pub score: f64,
    #[serde(default)]
    pub is_minat: bool,
}

impl From<RaporEntryPayload> for RaporEntryInput {
    fn from(p: RaporEntryPayload) -> Self {
        Self { semester: p.semester, subject: p.subject, score: p.score, is_minat: p.is_minat }
    }
}

#[derive(Debug, Deserialize)]
pub struct RaporBatchPayload {
    pub entries: Vec<RaporEntryPayload>,
}

// ─── Import nilai rapor massal (Portal Sekolah, banyak siswa dari 1 file Excel) ────────

/// Beda dari `RaporEntryPayload`: tidak lewat `Path<student_id>` (satu file bisa berisi
/// banyak siswa sekaligus), jadi tiap baris membawa `nis`-nya sendiri untuk dicocokkan ke
/// siswa di sekolah milik actor. Validasi rentang nilai/semester dilakukan di service layer
/// (bukan di sini via `validator`) supaya satu baris tidak valid tidak menggagalkan
/// deserialisasi seluruh payload — baris itu perlu tetap sampai ke service untuk
/// dilaporkan balik sebagai satu entri gagal, bukan mengagalkan HTTP request-nya.
#[derive(Debug, Deserialize)]
pub struct RaporBulkImportEntryPayload {
    pub nis: String,
    pub semester: i32,
    pub subject: String,
    pub score: f64,
    #[serde(default)]
    pub is_minat: bool,
}

impl From<RaporBulkImportEntryPayload> for BulkRaporEntryInput {
    fn from(p: RaporBulkImportEntryPayload) -> Self {
        Self { nis: p.nis, semester: p.semester, subject: p.subject, score: p.score, is_minat: p.is_minat }
    }
}

#[derive(Debug, Deserialize)]
pub struct RaporBulkImportPayload {
    pub entries: Vec<RaporBulkImportEntryPayload>,
}

#[derive(Debug, Serialize)]
pub struct RaporBulkImportErrorResponse {
    pub nis: String,
    pub semester: i32,
    pub subject: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct RaporBulkImportResponse {
    pub saved_count: i64,
    pub errors: Vec<RaporBulkImportErrorResponse>,
}

impl From<BulkRaporImportResult> for RaporBulkImportResponse {
    fn from(r: BulkRaporImportResult) -> Self {
        Self {
            saved_count: r.saved_count,
            errors: r
                .errors
                .into_iter()
                .map(|e| RaporBulkImportErrorResponse { nis: e.nis, semester: e.semester, subject: e.subject, reason: e.reason })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RaporScoreResponse {
    pub id: Uuid,
    pub semester: i32,
    pub subject: String,
    pub score: f64,
    pub is_minat: bool,
}

impl From<RaporScore> for RaporScoreResponse {
    fn from(r: RaporScore) -> Self {
        Self { id: r.id, semester: r.semester, subject: r.subject, score: r.score, is_minat: r.is_minat }
    }
}

// ─── Targets + chance ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct TargetPayload {
    pub ptn_program_id: Uuid,
    /// "snbp" | "snbt"
    #[validate(length(min = 1))]
    pub track: String,
    /// "utama" | "cadangan" | "aman"
    #[serde(default = "default_priority")]
    pub priority: String,
}
fn default_priority() -> String {
    "utama".to_string()
}

#[derive(Debug, Deserialize, Validate)]
pub struct PreviewChanceQuery {
    pub ptn_program_id: Uuid,
    pub track: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PriorityPayload {
    #[validate(length(min = 1))]
    pub priority: String,
}

#[derive(Debug, Serialize)]
pub struct ChanceResponse {
    pub chance_percent: f64,
    pub tier: String,
    pub student_value: f64,
    pub program_threshold: f64,
    pub competition_ratio: f64,
    pub competition_adjustment: f64,
    pub prestasi_contribution: f64,
}

impl From<ChanceResult> for ChanceResponse {
    fn from(c: ChanceResult) -> Self {
        Self {
            chance_percent: c.chance_percent,
            tier: c.tier.as_str().to_string(),
            student_value: c.student_value,
            program_threshold: c.program_threshold,
            competition_ratio: c.competition_ratio,
            competition_adjustment: c.competition_adjustment,
            prestasi_contribution: c.prestasi_contribution,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TargetWithChanceResponse {
    pub id: Uuid,
    pub ptn_program_id: Uuid,
    pub track: String,
    pub priority: String,
    pub sort_order: i32,
    pub program: PtnProgramResponse,
    pub chance: ChanceResponse,
}

impl From<TargetWithChance> for TargetWithChanceResponse {
    fn from(t: TargetWithChance) -> Self {
        let target: PtnTarget = t.target;
        Self {
            id: target.id,
            ptn_program_id: target.ptn_program_id,
            track: target.track.as_str().to_string(),
            priority: target.priority.as_str().to_string(),
            sort_order: target.sort_order,
            program: t.program.into(),
            chance: t.chance.into(),
        }
    }
}

// ─── School Rasionalisasi: Dashboard / Alumni / Eligibility ─────────────────────

#[derive(Debug, Serialize)]
pub struct UniversityCountResponse {
    pub nama_ptn: String,
    pub count: i64,
}
impl From<UniversityCount> for UniversityCountResponse {
    fn from(u: UniversityCount) -> Self {
        Self { nama_ptn: u.nama_ptn, count: u.count }
    }
}

#[derive(Debug, Serialize)]
pub struct StudentChanceSummaryResponse {
    pub student_id: Uuid,
    pub student_name: String,
    pub best_chance_percent: f64,
    pub tier: String,
}
impl From<StudentChanceSummary> for StudentChanceSummaryResponse {
    fn from(s: StudentChanceSummary) -> Self {
        Self {
            student_id: s.student_id,
            student_name: s.student_name,
            best_chance_percent: s.best_chance_percent,
            tier: s.tier.as_str().to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SchoolEligibilityResponse {
    pub id: Uuid,
    pub year: i32,
    pub eligible_count: i32,
    pub updated_at: DateTime<Utc>,
    /// Jumlah siswa yang BENAR-BENAR terdaftar aktif di SNBP tahun ini
    /// (`snbp_participation.aktif = true`), real dari roster "Daftar Siswa" — bukan input
    /// manual seperti `eligible_count`. Disandingkan supaya PIC bisa lihat langsung kalau
    /// pendaftaran sudah melebihi (atau masih di bawah) kuota eligible yang mereka input.
    pub terdaftar_aktif_count: i64,
}
impl From<SchoolEligibility> for SchoolEligibilityResponse {
    fn from(e: SchoolEligibility) -> Self {
        Self { id: e.id, year: e.year, eligible_count: e.eligible_count, updated_at: e.updated_at, terdaftar_aktif_count: 0 }
    }
}
impl SchoolEligibilityResponse {
    pub fn from_with_usage(e: SchoolEligibility, terdaftar_aktif_count: i64) -> Self {
        Self { id: e.id, year: e.year, eligible_count: e.eligible_count, updated_at: e.updated_at, terdaftar_aktif_count }
    }
}

#[derive(Debug, Serialize)]
pub struct SnbpDashboardResponse {
    pub total_siswa_aktif: i64,
    pub terasionalisasi: i64,
    pub avg_chance_snbp: Option<f64>,
    pub eligible_terbaru: Option<SchoolEligibilityResponse>,
    pub distribusi_universitas: Vec<UniversityCountResponse>,
    pub top_peluang: Vec<StudentChanceSummaryResponse>,
}
impl From<SnbpDashboard> for SnbpDashboardResponse {
    fn from(d: SnbpDashboard) -> Self {
        Self {
            total_siswa_aktif: d.total_siswa_aktif,
            terasionalisasi: d.terasionalisasi,
            avg_chance_snbp: d.avg_chance_snbp,
            eligible_terbaru: d.eligible_terbaru.map(Into::into),
            distribusi_universitas: d.distribusi_universitas.into_iter().map(Into::into).collect(),
            top_peluang: d.top_peluang.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SnbpRankingEntryResponse {
    pub student_id: Uuid,
    pub student_name: String,
    pub rapor_avg: Option<f64>,
    pub prestasi_index: f64,
    pub target_count: i64,
    pub best_chance_percent: f64,
    pub tier: String,
}
impl From<SnbpRankingEntry> for SnbpRankingEntryResponse {
    fn from(r: SnbpRankingEntry) -> Self {
        Self {
            student_id: r.student_id,
            student_name: r.student_name,
            rapor_avg: r.rapor_avg,
            prestasi_index: r.prestasi_index,
            target_count: r.target_count,
            best_chance_percent: r.best_chance_percent,
            tier: r.tier.as_str().to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct AlumniPayload {
    #[validate(length(min = 1, message = "nama alumni wajib diisi"))]
    pub alumni_name: String,
    #[validate(range(min = 2000, max = 2100, message = "tahun lulus tidak valid"))]
    pub graduation_year: i32,
    /// "snbp" | "snbt"
    #[serde(default = "default_alumni_track")]
    pub track: String,
    #[validate(length(min = 1, message = "nama PTN wajib diisi"))]
    pub nama_ptn: String,
    #[validate(length(min = 1, message = "nama program studi wajib diisi"))]
    pub nama_prodi: String,
    #[validate(range(min = 0.0, max = 100.0, message = "nilai acuan harus 0-100"))]
    pub benchmark_score: f64,
}
fn default_alumni_track() -> String {
    "snbp".to_string()
}

#[derive(Debug, Serialize)]
pub struct AlumniResponse {
    pub id: Uuid,
    pub alumni_name: String,
    pub graduation_year: i32,
    pub track: String,
    pub nama_ptn: String,
    pub nama_prodi: String,
    pub benchmark_score: f64,
    pub created_at: DateTime<Utc>,
}
impl From<AlumniBenchmark> for AlumniResponse {
    fn from(a: AlumniBenchmark) -> Self {
        Self {
            id: a.id,
            alumni_name: a.alumni_name,
            graduation_year: a.graduation_year,
            track: a.track.as_str().to_string(),
            nama_ptn: a.nama_ptn,
            nama_prodi: a.nama_prodi,
            benchmark_score: a.benchmark_score,
            created_at: a.created_at,
        }
    }
}

impl AlumniPayload {
    pub fn into_input(self, track: crate::domain::rationalization::Track) -> AddAlumniInput {
        AddAlumniInput {
            alumni_name: self.alumni_name,
            graduation_year: self.graduation_year,
            track,
            nama_ptn: self.nama_ptn,
            nama_prodi: self.nama_prodi,
            benchmark_score: self.benchmark_score,
        }
    }
}

/// "Detail Rapor" alumni — satu entri nilai per mata pelajaran (snapshot, tanpa semester).
#[derive(Debug, Deserialize, Validate)]
pub struct AlumniRaporEntryPayload {
    #[validate(length(min = 1, message = "nama mata pelajaran wajib diisi"))]
    pub subject: String,
    #[validate(range(min = 0.0, max = 100.0, message = "nilai harus di antara 0 dan 100"))]
    pub score: f64,
}

#[derive(Debug, Deserialize)]
pub struct AlumniRaporBatchPayload {
    pub entries: Vec<AlumniRaporEntryPayload>,
}

#[derive(Debug, Serialize)]
pub struct AlumniRaporScoreResponse {
    pub id: Uuid,
    pub alumni_id: Uuid,
    pub subject: String,
    pub score: f64,
}

impl From<crate::domain::rationalization::AlumniRaporScore> for AlumniRaporScoreResponse {
    fn from(value: crate::domain::rationalization::AlumniRaporScore) -> Self {
        Self { id: value.id, alumni_id: value.alumni_id, subject: value.subject, score: value.score }
    }
}

impl AlumniRaporEntryPayload {
    pub fn into_upsert(self) -> crate::domain::repository::AlumniRaporScoreUpsert {
        crate::domain::repository::AlumniRaporScoreUpsert { subject: self.subject, score: self.score }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct EligibilityPayload {
    #[validate(range(min = 2000, max = 2100, message = "tahun tidak valid"))]
    pub year: i32,
    #[validate(range(min = 0, message = "jumlah eligible tidak boleh negatif"))]
    pub eligible_count: i32,
}

// ─── Student achievements ("Prestasi") ───────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct AchievementPayload {
    #[validate(length(min = 1, message = "nama prestasi wajib diisi"))]
    pub nama: String,
    /// "sekolah" | "kab-kota" | "provinsi" | "nasional" | "internasional"
    #[validate(length(min = 1, message = "tingkat wajib diisi"))]
    pub tingkat: String,
    #[validate(range(min = 2000, max = 2100, message = "tahun tidak valid"))]
    pub tahun: i32,
    #[serde(default)]
    pub juara: String,
}

#[derive(Debug, Serialize)]
pub struct AchievementResponse {
    pub id: Uuid,
    pub nama: String,
    pub tingkat: String,
    pub tahun: i32,
    pub juara: String,
    pub bonus: f64,
    pub certificate_url: Option<String>,
    pub created_at: DateTime<Utc>,
}
impl From<Achievement> for AchievementResponse {
    fn from(a: Achievement) -> Self {
        Self {
            id: a.id,
            nama: a.nama,
            tingkat: a.tingkat.as_str().to_string(),
            tahun: a.tahun,
            juara: a.juara,
            bonus: a.tingkat.bonus(),
            certificate_url: a.certificate_url,
            created_at: a.created_at,
        }
    }
}

impl AchievementPayload {
    pub fn into_input(self, tingkat: crate::domain::rationalization::AchievementLevel) -> AchievementInput {
        AchievementInput { nama: self.nama, tingkat, tahun: self.tahun, juara: self.juara }
    }
}

// ─── Rekap SNBT ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SnbtChoiceResponse {
    pub nama_ptn: String,
    pub nama_prodi: String,
    pub priority: String,
    pub pg_snbt: f64,
    pub daya_tampung_snbt: i32,
    pub peminat_snbt: i32,
}
impl From<SnbtChoice> for SnbtChoiceResponse {
    fn from(c: SnbtChoice) -> Self {
        Self {
            nama_ptn: c.nama_ptn,
            nama_prodi: c.nama_prodi,
            priority: c.priority.as_str().to_string(),
            pg_snbt: c.pg_snbt,
            daya_tampung_snbt: c.daya_tampung_snbt,
            peminat_snbt: c.peminat_snbt,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SnbtRosterRowResponse {
    pub student_id: Uuid,
    pub student_name: String,
    pub pilihan_1: Option<SnbtChoiceResponse>,
    pub pilihan_2: Option<SnbtChoiceResponse>,
    pub est_score: Option<f64>,
    /// `"simulasi"` | `"drilling"` | `null` — where `est_score` came from, so the frontend
    /// can caption its reliability. See `SnbtRosterRow::est_score_source` doc comment.
    pub est_score_source: Option<String>,
    pub actual_score: Option<f64>,
    pub exam_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub notes: String,
}
impl From<SnbtRosterRow> for SnbtRosterRowResponse {
    fn from(r: SnbtRosterRow) -> Self {
        Self {
            student_id: r.student_id,
            student_name: r.student_name,
            pilihan_1: r.pilihan_1.map(Into::into),
            pilihan_2: r.pilihan_2.map(Into::into),
            est_score: r.est_score,
            est_score_source: r.est_score_source.map(|s| s.to_string()),
            actual_score: r.actual_score,
            exam_date: r.exam_date,
            status: r.status.as_str().to_string(),
            notes: r.notes,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SnbtDashboardResponse {
    pub terdaftar: i64,
    pub sudah_ujian: i64,
    pub diterima: i64,
    pub skor_perlu_diisi: i64,
}
impl From<SnbtDashboard> for SnbtDashboardResponse {
    fn from(d: SnbtDashboard) -> Self {
        Self {
            terdaftar: d.terdaftar,
            sudah_ujian: d.sudah_ujian,
            diterima: d.diterima,
            skor_perlu_diisi: d.skor_perlu_diisi,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct SnbtTrackingPayload {
    #[validate(range(min = 0.0, max = 1000.0, message = "skor aktual harus di antara 0 dan 1000"))]
    pub actual_score: Option<f64>,
    pub exam_date: Option<chrono::NaiveDate>,
    /// "terdaftar" | "sudah-ujian" | "diterima" | "tidak-diterima"
    #[validate(length(min = 1, message = "status wajib diisi"))]
    pub status: String,
    #[serde(default)]
    pub notes: String,
}

impl SnbtTrackingPayload {
    pub fn into_input(
        self,
        status: crate::domain::rationalization::SnbtTrackingStatus,
    ) -> SnbtTrackingInput {
        SnbtTrackingInput { actual_score: self.actual_score, exam_date: self.exam_date, status, notes: self.notes }
    }
}

// ─── SNBP roster ("Daftar Siswa") ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SnbpRosterRowResponse {
    pub student_id: Uuid,
    pub student_name: String,
    pub kelas: Option<String>,
    pub konsultan: String,
    pub pilihan_1: Option<String>,
    pub pilihan_2: Option<String>,
    pub status: String,
    pub aktif: bool,
    pub year: i32,
    pub minat_jurusan: Option<String>,
}
impl From<SnbpRosterRow> for SnbpRosterRowResponse {
    fn from(r: SnbpRosterRow) -> Self {
        Self {
            student_id: r.student_id,
            student_name: r.student_name,
            kelas: r.kelas,
            konsultan: r.konsultan,
            pilihan_1: r.pilihan_1,
            pilihan_2: r.pilihan_2,
            status: r.status.as_str().to_string(),
            aktif: r.aktif,
            year: r.year,
            minat_jurusan: r.minat_jurusan,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct SnbpParticipationPayload {
    pub year: i32,
    #[serde(default)]
    pub konsultan: String,
    /// "belum" | "diterima-snbp" | "diterima-snbt" | "tidak"
    #[validate(length(min = 1, message = "status wajib diisi"))]
    pub status: String,
    #[serde(default = "default_true")]
    pub aktif: bool,
}

fn default_true() -> bool {
    true
}

impl SnbpParticipationPayload {
    pub fn into_input(self, status: crate::domain::rationalization::SnbpStatus) -> SnbpParticipationInput {
        SnbpParticipationInput { year: self.year, konsultan: self.konsultan, status, aktif: self.aktif }
    }
}

// ─── Audit trail ("Riwayat Perubahan") ──────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AuditLogEntryResponse {
    pub id: Uuid,
    pub actor_name: String,
    pub actor_role: String,
    pub action: String,
    pub entity_type: String,
    pub summary: String,
    pub created_at: DateTime<Utc>,
}

impl From<AuditLogEntry> for AuditLogEntryResponse {
    fn from(e: AuditLogEntry) -> Self {
        Self {
            id: e.id,
            actor_name: e.actor_name,
            actor_role: e.actor_role,
            action: e.action,
            entity_type: e.entity_type,
            summary: e.summary,
            created_at: e.created_at,
        }
    }
}
