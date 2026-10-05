//! Analytics value objects — pure read-projections aggregated from questions/attempts/
//! users/schools. No CRUD, no business rules, nothing sensitive to hide — so unlike
//! `User`/`Question`/`School`, these are serialized straight to the HTTP boundary
//! without a separate DTO layer (see `interfaces/http/handlers/analytics_handler.rs`).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsSummary {
    pub total_students: i64,
    pub total_schools: i64,
    pub total_questions: i64,
    pub approved_questions: i64,
    pub total_attempts: i64,
    pub submitted_attempts: i64,
    pub average_score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreTrendPoint {
    pub day: NaiveDate,
    pub attempts: i64,
    pub average_score: f64,
}

/// One row of the "Perbandingan Tahun" comparison — attempts bucketed by the calendar
/// year of `submitted_at` (not student "angkatan"/cohort, which has no reliable field
/// in this system yet — see `application::analytics_service` doc comment on
/// `score_trend_by_year`). Includes every year that has at least one submitted attempt;
/// there's no fixed year range, since a school's real history could start any year.
#[derive(Debug, Clone, Serialize)]
pub struct YearlyPerformancePoint {
    pub year: i32,
    pub attempts: i64,
    /// Distinct students who submitted at least one attempt that year — lets a school see
    /// whether a year-over-year score change is from more/fewer students, not just noise.
    pub distinct_students: i64,
    pub average_score: f64,
}

/// `Deserialize` ditambahkan (beda dari struct analytics lain di file ini) karena struct ini
/// juga dipakai di dalam `domain::report::AkreditasiReportPayload`, yang harus bisa dibaca
/// kembali dari snapshot JSONB laporan tersimpan — lihat `domain::report::ReportPayload`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectAccuracy {
    pub subject: String,
    pub correct: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuestionStatusCount {
    pub status: String,
    pub count: i64,
}

/// One row of the "Top Sekolah Mitra" ranking — only schools with at least one submitted
/// attempt show up (there's nothing to rank otherwise).
#[derive(Debug, Clone, Serialize)]
pub struct SchoolRanking {
    pub school_id: uuid::Uuid,
    pub school_name: String,
    pub active_students: i64,
    pub average_score: f64,
}

/// One row of the School portal's internal student ranking (Rekap SNBT / Laporan
/// "Ranking Internal") — only students with at least one submitted attempt show up.
#[derive(Debug, Clone, Serialize)]
pub struct StudentRanking {
    pub student_id: uuid::Uuid,
    pub student_name: String,
    pub attempts_count: i64,
    pub best_score: i32,
    pub average_score: f64,
}

/// One row of the School portal's "Analytics & Insights" student drill-down — deliberately
/// a SEPARATE query from `StudentRanking` above (which INNER JOINs `attempts` and so silently
/// excludes any student who has never submitted anything). The whole point of this one is the
/// opposite: surface EVERY real student of this school, including those with zero activity,
/// because "belum pernah mengerjakan apa pun" is itself the most important insight a school
/// needs to see — a ranking table that just omits them would hide the signal entirely.
/// `best_score`/`average_score`/`last_attempt_at` are all `None` for a student with no
/// submitted attempts (honest-zero: never faked as `0` where `0` would look like a real score).
#[derive(Debug, Clone, Serialize)]
pub struct StudentActivity {
    pub student_id: uuid::Uuid,
    pub student_name: String,
    /// From `students.rombel_code` (a 1:1 extension table joined on `user_id`, NOT a column
    /// on `users` itself) — `None` for students imported/created before that field existed,
    /// or B2C students without a school-managed rombel.
    pub rombel_code: Option<String>,
    pub attempts_count: i64,
    pub best_score: Option<i32>,
    pub average_score: Option<f64>,
    pub last_attempt_at: Option<DateTime<Utc>>,
}
