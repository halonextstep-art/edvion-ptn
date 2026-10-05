//! Question entity — the core unit of the Bank Soal (question bank) module.
//! Combines the CRUD-oriented fields from the reference `QuestionEditor` with the
//! governance/review workflow from the reference `QuestionBank` (review queue).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    MultipleChoice,
    ComplexMultiple,
    ShortAnswer,
    /// Benar/Salah — a fixed two-option pick, always `["Benar", "Salah"]`, graded by exact
    /// string match just like `multiple_choice` (no repository/grading changes needed).
    TrueFalse,
    /// Menjodohkan — pairs of left/right items. Each `options` entry is one canonical
    /// "kiri::kanan" pair string (author-entered order = the correct order); `correct_answer`
    /// is the same pairs joined the same way the player must serialize a submission, so the
    /// existing generic string-equality grading in tryout_service.rs works unmodified.
    Matching,
    /// Uraian — free-text essay. Self-check only: `correct_answer` holds a sample answer /
    /// rubric shown to the student after submit, never auto-graded and never selected for
    /// Tryout sessions (see QuestionRepository::random_approved) — Drilling-only by design,
    /// per explicit product decision to avoid building a manual-grading queue.
    Essay,
    /// Benar/Salah Ganda — several independent true/false statements graded together as one
    /// question. Each `options` entry is one statement string (author order = display order);
    /// `correct_answer` is the parallel list of `"benar"`/`"salah"` tokens (one per statement,
    /// same order), comma-space-joined — e.g. `"benar, salah"`. The player must submit its
    /// answer serialized the exact same way for the existing generic string-equality grading
    /// to keep working unmodified.
    TrueFalseComplex,
    /// Menjodohkan Gambar — same pairing mechanics/encoding as `Matching` (`"kiri::kanan"`
    /// pair strings, `correct_answer` = same pairs joined), except both sides of each pair are
    /// image URLs (from the generic file-upload storage) instead of plain text — purely a
    /// player-rendering distinction (render `<img>` instead of text for each side), so it
    /// deliberately shares `Matching`'s validation branch rather than duplicating it.
    MatchingImage,
    /// Urutkan — arrange a list of steps into the correct sequence. `options` holds the step
    /// strings in their correct order (as authored); `correct_answer` is the same list
    /// comma-space-joined. The player shuffles a display copy of `options` client-side, lets
    /// the student reorder it, then submits the student's arrangement joined the same way —
    /// again plain string-equality grading, no new repository/grading machinery needed.
    Ordering,
}

impl QuestionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            QuestionType::MultipleChoice => "multiple_choice",
            QuestionType::ComplexMultiple => "complex_multiple",
            QuestionType::ShortAnswer => "short_answer",
            QuestionType::TrueFalse => "true_false",
            QuestionType::Matching => "matching",
            QuestionType::Essay => "essay",
            QuestionType::TrueFalseComplex => "true_false_complex",
            QuestionType::MatchingImage => "matching_image",
            QuestionType::Ordering => "ordering",
        }
    }
}

impl std::str::FromStr for QuestionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "multiple_choice" => Ok(QuestionType::MultipleChoice),
            "complex_multiple" => Ok(QuestionType::ComplexMultiple),
            "short_answer" => Ok(QuestionType::ShortAnswer),
            "true_false" => Ok(QuestionType::TrueFalse),
            "matching" => Ok(QuestionType::Matching),
            "essay" => Ok(QuestionType::Essay),
            "true_false_complex" => Ok(QuestionType::TrueFalseComplex),
            "matching_image" => Ok(QuestionType::MatchingImage),
            "ordering" => Ok(QuestionType::Ordering),
            other => Err(format!("unknown question_type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
        }
    }
}

impl std::str::FromStr for Difficulty {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "easy" => Ok(Difficulty::Easy),
            "medium" => Ok(Difficulty::Medium),
            "hard" => Ok(Difficulty::Hard),
            other => Err(format!("unknown difficulty: {other}")),
        }
    }
}

/// Review/governance workflow status. New questions created by `content` role start in
/// `Review`; questions created directly by `admin` may start `Approved`. Only `admin`
/// (`Role::can_review_questions`) may transition Review -> Approved/Rejected/Revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum QuestionStatus {
    Draft,
    Review,
    Approved,
    Rejected,
    Revision,
}

impl QuestionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            QuestionStatus::Draft => "draft",
            QuestionStatus::Review => "review",
            QuestionStatus::Approved => "approved",
            QuestionStatus::Rejected => "rejected",
            QuestionStatus::Revision => "revision",
        }
    }
}

impl std::str::FromStr for QuestionStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(QuestionStatus::Draft),
            "review" => Ok(QuestionStatus::Review),
            "approved" => Ok(QuestionStatus::Approved),
            "rejected" => Ok(QuestionStatus::Rejected),
            "revision" => Ok(QuestionStatus::Revision),
            other => Err(format!("unknown question status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: Uuid,
    pub code: String,
    pub question_type: QuestionType,
    pub subject: String,
    pub topic: String,
    pub subtopic: String,
    pub difficulty: Difficulty,
    pub bloom_level: String,
    pub stimulus: Option<String>,
    pub question_text: String,
    /// For multiple_choice / complex_multiple: the list of answer options.
    pub options: Option<Vec<String>>,
    /// Single answer for multiple_choice/short_answer; comma-separated keys/values for
    /// complex_multiple (kept as a plain string to mirror the reference's flexible type).
    pub correct_answer: String,
    pub explanation: String,
    pub tags: Vec<String>,
    pub status: QuestionStatus,
    pub review_note: Option<String>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    /// Denormalized author name, resolved from `users.name` at read time — lets the UI
    /// show "dibuat oleh Sari Dewi" instead of a raw UUID.
    pub created_by_name: String,
    pub usage_count: i32,
    pub average_score: f64,
    pub time_limit: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Question {
    /// Only questions in `Approved` status are eligible to be served in a tryout/drilling
    /// attempt — mirrors "Pengelolaan soal hanya Admin Pusat" governance rule.
    pub fn is_publishable(&self) -> bool {
        matches!(self.status, QuestionStatus::Approved)
    }

    pub fn is_owned_by(&self, user_id: Uuid) -> bool {
        self.created_by == user_id
    }
}

/// Auto-generate a question code from subject/topic, e.g. "MAT-ALG-a1b2c3".
/// Mirrors the reference's `${subject}-${topic}-${suffix}` convention.
pub fn generate_code(subject: &str, topic: &str, unique_suffix: &str) -> String {
    let subj = subject.chars().take(3).collect::<String>().to_uppercase();
    let top = if topic.is_empty() {
        "GEN".to_string()
    } else {
        topic.chars().take(3).collect::<String>().to_uppercase()
    };
    format!("{subj}-{top}-{unique_suffix}")
}
