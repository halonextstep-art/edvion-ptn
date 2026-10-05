//! Tryout / Drilling domain — session templates (what a student can start) and attempts
//! (a student's in-progress or completed run through a session), mirroring the reference
//! `DrillingZone` (session catalogue) + `TryoutPlayer` (intro -> quiz -> results -> review).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::package::ExamTrack;
use crate::domain::school::SchoolType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SessionType {
    Tryout,
    Drilling,
    Mini,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TryoutSession {
    pub id: Uuid,
    pub title: String,
    pub session_type: SessionType,
    pub duration_minutes: i32,
    pub question_count: i32,
    pub subject_filter: Option<String>,
    pub topic_filter: Option<String>,
    pub difficulty_filter: Option<String>,
    pub is_premium: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    /// `Some(set_id)` opts this session into the fixed, curated "Set Soal" question list
    /// instead of the random-filter draw — see `domain::question_set` doc comment. `None`
    /// (the default for every pre-existing session) preserves the original random-filter
    /// behavior completely unchanged.
    pub question_set_id: Option<Uuid>,
    /// `true` = still being assembled (e.g. via the "Buat Paket + Subtes" wizard), hidden from
    /// the student/school-facing catalogue. Default `false` for every pre-existing session.
    /// Cleared to `false` when the owning Package is published — see
    /// `PackageService::set_active` doc comment.
    pub is_draft: bool,
    /// `true` = this session is one of an owning Package's "mapel pilihan" (elective) options
    /// rather than a mandatory one — e.g. TKA's Fisika/Kimia/Ekonomi/etc. subtests, as opposed
    /// to its always-available Bahasa Indonesia/Matematika/Bahasa Inggris "mapel wajib" ones.
    /// Only meaningful when the owning Package's `elective_pick_count > 0`; see
    /// `ElectiveService` and `TryoutService::start_attempt_impl`'s gating check. Default `false`
    /// for every pre-existing session (unrestricted, exactly as before this feature existed).
    pub is_elective: bool,
    /// See `domain::package::ExamTrack` doc comment.
    pub exam_track: ExamTrack,
    /// `None` (default) = tampil ke siswa jenjang apapun. `Some(t)` membatasi tampilan katalog
    /// (`TryoutService::list_sessions`) ke siswa yang sekolahnya berjenjang `t` saja — siswa B2C
    /// tanpa sekolah tetap melihatnya (tidak bisa ditentukan jenjangnya, jadi tidak dibatasi).
    /// Lihat migration `20250101000039_content_school_type_scope.sql`.
    pub school_type_scope: Option<SchoolType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AttemptStatus {
    InProgress,
    Submitted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attempt {
    pub id: Uuid,
    pub session_id: Uuid,
    pub session_title: String,
    pub session_type: SessionType,
    pub student_id: Uuid,
    pub status: AttemptStatus,
    pub duration_minutes: i32,
    pub started_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub score: Option<i32>,
    pub accuracy: Option<f64>,
    pub correct_count: Option<i32>,
    pub wrong_count: Option<i32>,
    pub unanswered_count: Option<i32>,
    pub time_used_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptAnswer {
    pub id: Uuid,
    pub attempt_id: Uuid,
    pub question_id: Uuid,
    pub answer_text: Option<String>,
    pub is_correct: Option<bool>,
    pub flagged: bool,
    pub answered_at: DateTime<Utc>,
}

/// Pure scoring logic — no I/O, fully unit-testable. UTBK-style score out of 1000,
/// mirroring the reference's `Math.round((correct / total) * 1000)`.
pub struct ScoreBreakdown {
    pub score: i32,
    pub accuracy: f64,
    pub correct_count: i32,
    pub wrong_count: i32,
    pub unanswered_count: i32,
}

pub fn compute_score(total: usize, correct: usize, wrong: usize, unanswered: usize) -> ScoreBreakdown {
    let score = if total == 0 {
        0
    } else {
        ((correct as f64 / total as f64) * 1000.0).round() as i32
    };
    let denom = correct + wrong;
    let accuracy = if denom == 0 {
        0.0
    } else {
        ((correct as f64 / denom as f64) * 100.0 * 100.0).round() / 100.0
    };
    ScoreBreakdown {
        score,
        accuracy,
        correct_count: correct as i32,
        wrong_count: wrong as i32,
        unanswered_count: unanswered as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_all_correct_as_1000() {
        let b = compute_score(10, 10, 0, 0);
        assert_eq!(b.score, 1000);
        assert_eq!(b.accuracy, 100.0);
    }

    #[test]
    fn scores_partial_correctly() {
        let b = compute_score(10, 7, 2, 1);
        assert_eq!(b.score, 700);
        assert_eq!(b.correct_count, 7);
        assert_eq!(b.wrong_count, 2);
        assert_eq!(b.unanswered_count, 1);
    }

    #[test]
    fn handles_zero_total_without_panicking() {
        let b = compute_score(0, 0, 0, 0);
        assert_eq!(b.score, 0);
        assert_eq!(b.accuracy, 0.0);
    }
}
