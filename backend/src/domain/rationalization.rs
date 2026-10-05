//! Rasionalisasi SNBT/SNBP — PTN (university) admission-chance recommendation engine.
//!
//! `PtnProgram` is a real, published catalog (source: official SNBP/SNBT admission
//! statistics — daya tampung, peminat, dan passing grade per program studi — provided by
//! the user as "Hitungan Rasionalisasi.xlsx" and imported once via
//! `cargo run --bin import_ptn_catalog`; admin can maintain/update the figures afterward
//! via CRUD, same pattern as the Package/Voucher catalogs).
//!
//! The reference prototype's full methodology (Index Nilai + Index SNBP historis 3-tahun +
//! Index SNBT + Index Universitas + Index Jurusan + Index Prestasi + Index Akreditasi)
//! requires 3 years of *per-school* historical admission outcomes (berapa siswa dari
//! sekolah ini diterima di kampus X tahun 2021/2022/2023) that do not exist anywhere in
//! this system — no such table, no such input source, no admin ever entered it. Fabricating
//! that history to complete the formula would violate the project's core rule of never
//! hardcoding/faking data, so it is intentionally **not** replicated here.
//!
//! Instead, `compute_chance` produces a simplified, fully real-data-driven estimate from
//! two inputs that genuinely exist: the student's own real performance (tryout scores for
//! SNBT, self-reported rapor scores for SNBP — see `application::rationalization_service`
//! for how each track prepares its inputs) and the program's real published passing grade
//! + competition ratio (daya tampung / peminat). This is disclosed to the user in the UI as
//! an estimate, not a guarantee — same transparency pattern as the Finance module's
//! "aggregate from real data, not a full ledger" disclosure.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtnProgram {
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
    /// `false` for programs added from descriptive research only, without a verified
    /// official daya-tampung/peminat/passing-grade figure behind them. The engine must
    /// refuse to compute a chance estimate for these (see `unavailable_chance`) rather than
    /// let the zero-default numeric fields silently read as "no competition".
    pub has_official_stats: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PtnProgram {
    /// Capacity / applicants for the SNBP track. Falls back to 1.0 (i.e. "no signal, don't
    /// penalize") when there is no real applicant figure to divide by.
    pub fn competition_ratio_snbp(&self) -> f64 {
        if self.peminat_snbp <= 0 {
            1.0
        } else {
            (self.daya_tampung_snbp as f64 / self.peminat_snbp as f64).min(1.0)
        }
    }

    /// Capacity / applicants for the SNBT track.
    pub fn competition_ratio_snbt(&self) -> f64 {
        if self.peminat_snbt <= 0 {
            1.0
        } else {
            (self.daya_tampung_snbt as f64 / self.peminat_snbt as f64).min(1.0)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Track {
    Snbp,
    Snbt,
}

impl Track {
    pub fn as_str(&self) -> &'static str {
        match self {
            Track::Snbp => "snbp",
            Track::Snbt => "snbt",
        }
    }
}

impl std::str::FromStr for Track {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "snbp" => Ok(Track::Snbp),
            "snbt" => Ok(Track::Snbt),
            other => Err(format!("unknown track: {other}")),
        }
    }
}

/// Student-chosen label for how they personally regard this target — distinct from the
/// engine's *computed* `ChanceTier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Utama,
    Cadangan,
    Aman,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Utama => "utama",
            Priority::Cadangan => "cadangan",
            Priority::Aman => "aman",
        }
    }
}

impl std::str::FromStr for Priority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "utama" => Ok(Priority::Utama),
            "cadangan" => Ok(Priority::Cadangan),
            "aman" => Ok(Priority::Aman),
            other => Err(format!("unknown priority: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaporScore {
    pub id: Uuid,
    pub student_id: Uuid,
    pub semester: i32,
    pub subject: String,
    pub score: f64,
    pub is_minat: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Root keywords used to match a `PtnProgram::mapel_syarat` token (free text from the real
/// catalog, e.g. "Matematika Tingkat Lanjut", "Pendidikan Pancasila") against a student's
/// canonical rapor subject name (e.g. "Matematika Tingkat Lanjut", "Pendidikan Pancasila dan
/// Kewarganegaraan") — the two vocabularies don't always match character-for-character. Same
/// root-keyword strategy as `raporSubjectRoot()` in `frontend/utils/raporSubjects.ts`; keep
/// both lists in sync if either changes.
const MAPEL_SYARAT_ROOTS: &[&str] = &[
    "matematika", "fisika", "kimia", "biologi", "ekonomi", "sosiologi", "geografi", "sejarah",
    "pancasila", "ppkn", "pkn", "indonesia", "inggris", "seni", "budaya", "agama", "jasmani",
    "penjas", "pjok", "prakarya", "antropologi", "jerman", "prancis", "jepang", "informatika",
    "bali", "mandarin", "jawa", "sunda", "arab", "korea", "asing", "fiqih", "tafsir", "hadist",
    "ushul", "muatan lokal", "robotik", "riset", "conversation",
];

fn subject_root(subject: &str) -> Option<&'static str> {
    let lower = subject.to_lowercase();
    MAPEL_SYARAT_ROOTS.iter().find(|r| lower.contains(*r)).copied()
}

/// Real per-student rapor average, blended toward the *specific program's* required subjects
/// — `average(rata2-semua, rata2-syarat)`, closer to the reference "Hitungan Rasionalisasi"
/// spreadsheet's own `average(rata2-semua, rata2-minat)` blend than a single flat average
/// across every subject the student happens to have entered (which is what this function
/// replaced). Without this, a student strong in a program's actually-relevant subjects (e.g.
/// Matematika/Fisika for Teknik) but average elsewhere (Agama, PJOK, Seni Budaya, ...) had
/// that relevant strength diluted by ~40 unrelated subjects in the old flat average.
///
/// Falls back to the flat average alone — never `None`/0 — when `mapel_syarat` is blank or
/// none of its tokens match any subject the student has actually entered (e.g. they haven't
/// filled in those specific subjects yet), so this never produces a worse/missing estimate
/// than the old behavior.
pub fn average_rapor_for_program(rapor: &[RaporScore], mapel_syarat: &str) -> Option<f64> {
    if rapor.is_empty() {
        return None;
    }
    let flat_avg = rapor.iter().map(|r| r.score).sum::<f64>() / rapor.len() as f64;

    let required_roots: std::collections::HashSet<&'static str> =
        mapel_syarat.split(',').filter_map(|token| subject_root(token.trim())).collect();
    if required_roots.is_empty() {
        return Some(flat_avg);
    }

    let matched: Vec<f64> = rapor
        .iter()
        .filter(|r| subject_root(&r.subject).map(|root| required_roots.contains(root)).unwrap_or(false))
        .map(|r| r.score)
        .collect();
    if matched.is_empty() {
        return Some(flat_avg);
    }
    let syarat_avg = matched.iter().sum::<f64>() / matched.len() as f64;
    Some((flat_avg + syarat_avg) / 2.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtnTarget {
    pub id: Uuid,
    pub student_id: Uuid,
    pub ptn_program_id: Uuid,
    pub track: Track,
    pub priority: Priority,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

/// Computed chance tier shown to the student — the engine's own read of the numbers,
/// separate from the student's self-assigned `Priority` label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChanceTier {
    Ketat,
    Moderat,
    Aman,
}

impl ChanceTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChanceTier::Ketat => "ketat",
            ChanceTier::Moderat => "moderat",
            ChanceTier::Aman => "aman",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanceResult {
    pub chance_percent: f64,
    pub tier: ChanceTier,
    pub student_value: f64,
    pub program_threshold: f64,
    pub competition_ratio: f64,
    /// Percentage-point adjustment contributed by the program's competition ratio (real,
    /// derived from `daya_tampung`/`peminat`) — one of the components that sums into
    /// `chance_percent`. Exposed so the UI can show an honest breakdown bar instead of
    /// just the final number.
    pub competition_adjustment: f64,
    /// Percentage-point bonus contributed by the student's real "Index Prestasi"
    /// (achievements) — 0.0 for plain `compute_chance` (SNBT track, or when no bonus
    /// applies), non-zero only via `compute_chance_snbp`.
    pub prestasi_contribution: f64,
}

/// Sentinel result for a program with `has_official_stats == false` — callers (see
/// `rationalization_service::targets_with_chance_for`/`preview_chance_for`) must check the
/// flag and return this instead of calling `compute_chance`/`compute_chance_snbp`, whose
/// zero-defaulted `daya_tampung`/`peminat`/`pg_*` fields would otherwise be silently misread
/// as "no competition, right at an easy threshold" and produce a falsely optimistic "Aman".
/// Deliberately conservative (0%, `Ketat`) rather than optimistic, so a caller that forgets
/// to check the flag fails toward "go research this yourself", not toward false reassurance.
pub fn unavailable_chance(student_value: f64) -> ChanceResult {
    ChanceResult {
        chance_percent: 0.0,
        tier: ChanceTier::Ketat,
        student_value,
        program_threshold: 0.0,
        competition_ratio: 0.0,
        competition_adjustment: 0.0,
        prestasi_contribution: 0.0,
    }
}

/// Core estimate: how far above/below the program's real passing-grade threshold the
/// student's real score sits, blended with the program's real competition ratio
/// (capacity/applicants). Both inputs must already be prepared on a comparable 0-100-ish
/// scale by the caller (see `rationalization_service`) — this function itself is scale
/// agnostic and does no unit conversion.
///
/// `student_value` and `program_threshold` on the same scale, `competition_ratio` in
/// `[0, 1]` (capacity / applicants, capped at 1.0).
pub fn compute_chance(student_value: f64, program_threshold: f64, competition_ratio: f64) -> ChanceResult {
    let threshold = if program_threshold <= 0.0 { 1.0 } else { program_threshold };
    // Fractional gap between the student's value and the threshold: positive means above.
    let gap_fraction = (student_value - threshold) / threshold;
    // 50% right at the threshold; scales roughly +/-1pp of chance per +/-1% deviation.
    let base = 50.0 + (gap_fraction * 100.0);
    // Tight competition (low capacity/applicants ratio) pulls the estimate down; abundant
    // capacity relative to applicants lifts it. Neutral point kept at 0.15 — this is not an
    // arbitrary guess: the median daya_tampung/peminat ratio across the real SNBP catalog
    // (4588 programs, `data/ptn_programs.csv`) is ~0.17, so "typical" competition really is
    // close to 15%. What WAS wrong (fixed here): the old single linear slope `(ratio-0.15)*20`
    // gave the most selective programs (ratio near 0 — e.g. Kedokteran/top CS, ~12% of the
    // real catalog sits below a 5% ratio) almost no penalty (floor -3pp), while abundant-
    // capacity programs got up to +17pp — backwards, since the tight end is exactly where a
    // student most needs an honest warning. Now piecewise: -20pp at ratio 0 (steep below the
    // 0.15 neutral point, so truly cutthroat competition actually drags the estimate down
    // toward "Ketat"), +15pp at ratio 1 (gentler above the neutral point).
    let competition_adjustment = if competition_ratio <= 0.15 {
        (competition_ratio.clamp(0.0, 1.0) - 0.15) / 0.15 * 20.0
    } else {
        (competition_ratio.clamp(0.0, 1.0) - 0.15) / 0.85 * 15.0
    };
    let chance = (base + competition_adjustment).clamp(1.0, 99.0);
    let tier = if chance >= 65.0 {
        ChanceTier::Aman
    } else if chance >= 35.0 {
        ChanceTier::Moderat
    } else {
        ChanceTier::Ketat
    };
    ChanceResult {
        chance_percent: (chance * 10.0).round() / 10.0,
        tier,
        student_value,
        program_threshold: threshold,
        competition_ratio: competition_ratio.clamp(0.0, 1.0),
        competition_adjustment,
        prestasi_contribution: 0.0,
    }
}

/// Real per-student achievement (lomba/kompetisi), entered by the student themselves —
/// feeds `index_prestasi` below. Mirrors the reference "Hitungan Rasionalisasi" sheet's
/// `Achievement { nama, tingkat, tahun, juara }` shape exactly, including the level-tier
/// vocabulary (`sekolah` / `kab-kota` / `provinsi` / `nasional` / `internasional`) so the
/// bonus weights in `prestasi_bonus` line up with the source spreadsheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AchievementLevel {
    Sekolah,
    KabKota,
    Provinsi,
    Nasional,
    Internasional,
}

impl AchievementLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AchievementLevel::Sekolah => "sekolah",
            AchievementLevel::KabKota => "kab-kota",
            AchievementLevel::Provinsi => "provinsi",
            AchievementLevel::Nasional => "nasional",
            AchievementLevel::Internasional => "internasional",
        }
    }

    /// Point weight per achievement level — taken directly from the reference "Hitungan
    /// Rasionalisasi" spreadsheet's `PRESTASI_BONUS` table (sekolah 0.5, kab-kota 1,
    /// provinsi 2, nasional 4, internasional 8). `juara` (rank/placement) is stored for
    /// display but, matching the source, does not change the numeric weight.
    pub fn bonus(&self) -> f64 {
        match self {
            AchievementLevel::Sekolah => 0.5,
            AchievementLevel::KabKota => 1.0,
            AchievementLevel::Provinsi => 2.0,
            AchievementLevel::Nasional => 4.0,
            AchievementLevel::Internasional => 8.0,
        }
    }
}

impl std::str::FromStr for AchievementLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sekolah" => Ok(AchievementLevel::Sekolah),
            "kab-kota" => Ok(AchievementLevel::KabKota),
            "provinsi" => Ok(AchievementLevel::Provinsi),
            "nasional" => Ok(AchievementLevel::Nasional),
            "internasional" => Ok(AchievementLevel::Internasional),
            other => Err(format!("unknown achievement level: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: Uuid,
    pub student_id: Uuid,
    pub nama: String,
    pub tingkat: AchievementLevel,
    pub tahun: i32,
    pub juara: String,
    /// URL path to a locally-stored scan/photo of the certificate, or `None` if not
    /// attached — uploaded separately after the achievement row itself is created.
    pub certificate_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Sum of a student's real achievement bonuses, capped at 10 — same cap the reference
/// spreadsheet uses (`Math.min(sum, 10)`), so a handful of strong achievements can't
/// dominate the estimate.
pub fn index_prestasi(achievements: &[Achievement]) -> f64 {
    achievements.iter().map(|a| a.tingkat.bonus()).sum::<f64>().min(10.0)
}

/// SNBP-specific chance estimate: the same gap+competition base as `compute_chance`, plus
/// a real "Index Prestasi" bonus (see `index_prestasi`) applied as direct percentage
/// points — up to +10pp for a student with strong, real, self-reported achievements.
/// SNBT keeps using plain `compute_chance` since the reference's prestasi bonus is
/// SNBP-specific (rapor + non-akademik track), not part of the UTBK/SNBT methodology.
pub fn compute_chance_snbp(student_value: f64, program_threshold: f64, competition_ratio: f64, prestasi_bonus: f64) -> ChanceResult {
    let base = compute_chance(student_value, program_threshold, competition_ratio);
    let clamped_bonus = prestasi_bonus.clamp(0.0, 10.0);
    let adjusted = (base.chance_percent + clamped_bonus).clamp(1.0, 99.0);
    let tier = if adjusted >= 65.0 {
        ChanceTier::Aman
    } else if adjusted >= 35.0 {
        ChanceTier::Moderat
    } else {
        ChanceTier::Ketat
    };
    ChanceResult {
        chance_percent: (adjusted * 10.0).round() / 10.0,
        tier,
        prestasi_contribution: clamped_bonus,
        ..base
    }
}

/// Real post-exam status for one SNBT target, entered by the School PIC — powers "Rekap
/// SNBT". Distinct from the student's self-chosen `Priority` (utama/cadangan/aman, set at
/// target-creation time): this tracks what actually happened after the real UTBK exam.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SnbtTrackingStatus {
    Terdaftar,
    SudahUjian,
    Diterima,
    TidakDiterima,
}

impl SnbtTrackingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SnbtTrackingStatus::Terdaftar => "terdaftar",
            SnbtTrackingStatus::SudahUjian => "sudah-ujian",
            SnbtTrackingStatus::Diterima => "diterima",
            SnbtTrackingStatus::TidakDiterima => "tidak-diterima",
        }
    }
}

impl std::str::FromStr for SnbtTrackingStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "terdaftar" => Ok(SnbtTrackingStatus::Terdaftar),
            "sudah-ujian" => Ok(SnbtTrackingStatus::SudahUjian),
            "diterima" => Ok(SnbtTrackingStatus::Diterima),
            "tidak-diterima" => Ok(SnbtTrackingStatus::TidakDiterima),
            other => Err(format!("unknown snbt tracking status: {other}")),
        }
    }
}

/// One real UTBK/SNBT exam result per student — a student sits a single exam that
/// produces one score, applied across whichever SNBT program choices (pilihan 1/2) they
/// registered, so this is keyed by `student_id` (not per-target; see the migration doc
/// comment on `student_snbt_tracking` for why the earlier per-target model was wrong).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnbtTracking {
    pub id: Uuid,
    pub student_id: Uuid,
    pub actual_score: Option<f64>,
    pub exam_date: Option<chrono::NaiveDate>,
    pub status: SnbtTrackingStatus,
    pub notes: String,
    pub updated_at: DateTime<Utc>,
}

/// Real SNBP admission outcome for one student, entered by the School PIC — powers the
/// "Daftar Siswa" roster table. Distinct from `SnbtTrackingStatus` (about the post-exam
/// SNBT result, also per-student); this one is about the SNBP announcement result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SnbpStatus {
    Belum,
    DiterimaSnbp,
    DiterimaSnbt,
    Tidak,
}

impl SnbpStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SnbpStatus::Belum => "belum",
            SnbpStatus::DiterimaSnbp => "diterima-snbp",
            SnbpStatus::DiterimaSnbt => "diterima-snbt",
            SnbpStatus::Tidak => "tidak",
        }
    }
}

impl std::str::FromStr for SnbpStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "belum" => Ok(SnbpStatus::Belum),
            "diterima-snbp" => Ok(SnbpStatus::DiterimaSnbp),
            "diterima-snbt" => Ok(SnbpStatus::DiterimaSnbt),
            "tidak" => Ok(SnbpStatus::Tidak),
            other => Err(format!("unknown snbp status: {other}")),
        }
    }
}

/// Roster metadata for one student's SNBP participation (Konsultan/Status/Aktif) —
/// deliberately does NOT store "pilihan 1/2" (target university+major); those are always
/// derived live from the student's real `PtnTarget` rows (priority utama/cadangan) so the
/// roster can never drift out of sync with the actual targets/chances shown elsewhere.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnbpParticipation {
    pub id: Uuid,
    pub student_id: Uuid,
    pub school_id: Uuid,
    pub year: i32,
    pub konsultan: String,
    pub status: SnbpStatus,
    pub aktif: bool,
    pub updated_at: DateTime<Utc>,
}

/// A real past graduate's admission outcome, entered by the School PIC — powers the
/// "Kaka Kelas" (senior alumni) benchmark sub-tab so students can compare their own rapor
/// average against a genuine prior admit at the same program, instead of a fabricated
/// number. Deliberately simple (one score, self-reported by the school) rather than the
/// reference's full per-alumni index breakdown, for the same reason `compute_chance` is
/// simplified — no 3-years-of-history dataset exists to populate anything richer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlumniBenchmark {
    pub id: Uuid,
    pub school_id: Uuid,
    pub alumni_name: String,
    pub graduation_year: i32,
    pub track: Track,
    pub nama_ptn: String,
    pub nama_prodi: String,
    pub benchmark_score: f64,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// "Detail Rapor" opsional untuk satu `AlumniBenchmark` — snapshot nilai per mata pelajaran
/// alumni tsb (bukan riwayat multi-semester seperti `RaporScore` milik siswa aktif; alumni
/// sudah lulus, jadi cukup satu nilai akhir per mapel). Dipakai untuk radar chart
/// pencapaian siswa aktif vs alumni nyata — lihat komentar `AlumniBenchmark` di atas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlumniRaporScore {
    pub id: Uuid,
    pub alumni_id: Uuid,
    pub subject: String,
    pub score: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Real per-year count of how many of the school's students were SNBP-eligible (rapor +
/// akreditasi requirements met), entered directly by the School PIC — the "Eligible"
/// sub-tab. Not derived automatically because eligibility rules (akreditasi sekolah,
/// jalur minat, dst.) live outside this system's data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolEligibility {
    pub id: Uuid,
    pub school_id: Uuid,
    pub year: i32,
    pub eligible_count: i32,
    pub updated_at: DateTime<Utc>,
}

/// One accountability-trail entry for an "edit-on-behalf" action — recorded whenever a
/// School PIC (or Admin) creates/edits/deletes a STUDENT's own academic data (rapor,
/// prestasi, target PTN, SNBP/SNBT tracking) rather than the student doing it themselves.
/// Students editing their own data are never logged here; this exists so a student (or
/// another PIC) can later see exactly who touched their record, when, and what changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub actor_id: Uuid,
    pub actor_name: String,
    pub actor_role: String,
    pub student_id: Uuid,
    pub action: String,
    pub entity_type: String,
    pub summary: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_threshold_with_neutral_competition_is_near_fifty_percent() {
        let r = compute_chance(80.0, 80.0, 0.15);
        assert!((r.chance_percent - 50.0).abs() < 0.5);
        assert_eq!(r.tier, ChanceTier::Moderat);
    }

    #[test]
    fn well_above_threshold_and_good_capacity_is_aman() {
        let r = compute_chance(95.0, 80.0, 0.4);
        assert!(r.chance_percent >= 65.0);
        assert_eq!(r.tier, ChanceTier::Aman);
    }

    #[test]
    fn well_below_threshold_and_tight_competition_is_ketat() {
        let r = compute_chance(60.0, 90.0, 0.05);
        assert!(r.chance_percent < 35.0);
        assert_eq!(r.tier, ChanceTier::Ketat);
    }

    #[test]
    fn chance_never_leaves_one_to_ninety_nine_range() {
        let extreme_high = compute_chance(1000.0, 1.0, 1.0);
        let extreme_low = compute_chance(0.0, 1000.0, 0.0);
        assert!(extreme_high.chance_percent <= 99.0);
        assert!(extreme_low.chance_percent >= 1.0);
    }

    #[test]
    fn zero_applicants_does_not_panic_and_treats_ratio_as_neutral() {
        let p = PtnProgram {
            id: Uuid::nil(),
            kode: 1,
            nama_ptn: "X".into(),
            nama_prodi: "Y".into(),
            provinsi: "".into(),
            kota: "".into(),
            singkatan: "".into(),
            rumpun: "".into(),
            mapel_syarat: "".into(),
            daya_tampung_snbp: 10,
            peminat_snbp: 0,
            daya_tampung_snbt: 10,
            peminat_snbt: 0,
            pg_snbt: 0.0,
            pg_snbp: 0.0,
            jenjang: "S1".into(),
            has_official_stats: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(p.competition_ratio_snbp(), 1.0);
        assert_eq!(p.competition_ratio_snbt(), 1.0);
    }
}
