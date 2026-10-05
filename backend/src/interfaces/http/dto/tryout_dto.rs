use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::tryout_service::{
    AttemptResult, AttemptResume, AttemptWithQuestions, PlayableQuestion, ReviewItem, SubjectBreakdown,
};
use crate::domain::package::ExamTrack;
use crate::domain::school::SchoolType;
use crate::domain::tryout::{Attempt, SessionType, TryoutSession};

fn default_exam_track() -> ExamTrack {
    ExamTrack::Snbt
}

#[derive(Debug, Deserialize)]
pub struct SessionListQuery {
    /// "tryout" | "drilling" | "mini"
    #[serde(rename = "type")]
    pub session_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionPayload {
    pub title: String,
    pub session_type: String,
    pub duration_minutes: i32,
    pub question_count: i32,
    pub subject_filter: Option<String>,
    pub topic_filter: Option<String>,
    pub difficulty_filter: Option<String>,
    #[serde(default)]
    pub is_premium: bool,
    /// `Some(set_id)` opts this session into a curated "Set Soal" fixed list instead of
    /// the random-filter draw — see `domain::question_set` doc comment.
    #[serde(default)]
    pub question_set_id: Option<Uuid>,
    /// See `domain::tryout::TryoutSession::is_draft` doc comment.
    #[serde(default)]
    pub is_draft: bool,
    /// See `domain::tryout::TryoutSession::is_elective` doc comment.
    #[serde(default)]
    pub is_elective: bool,
    /// See `domain::package::ExamTrack` doc comment.
    #[serde(default = "default_exam_track")]
    pub exam_track: ExamTrack,
    /// See `domain::tryout::TryoutSession::school_type_scope` doc comment. `None` (omitted) =
    /// tampil ke semua jenjang.
    #[serde(default)]
    pub school_type_scope: Option<SchoolType>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSessionPayload {
    pub title: String,
    pub session_type: String,
    pub duration_minutes: i32,
    pub question_count: i32,
    pub subject_filter: Option<String>,
    pub topic_filter: Option<String>,
    pub difficulty_filter: Option<String>,
    #[serde(default)]
    pub is_premium: bool,
    #[serde(default)]
    pub question_set_id: Option<Uuid>,
    #[serde(default)]
    pub is_draft: bool,
    #[serde(default)]
    pub is_elective: bool,
    #[serde(default = "default_exam_track")]
    pub exam_track: ExamTrack,
    #[serde(default)]
    pub school_type_scope: Option<SchoolType>,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub title: String,
    pub session_type: String,
    pub duration_minutes: i32,
    pub question_count: i32,
    pub subject_filter: Option<String>,
    pub topic_filter: Option<String>,
    pub difficulty_filter: Option<String>,
    pub is_premium: bool,
    pub created_at: DateTime<Utc>,
    pub question_set_id: Option<Uuid>,
    pub is_draft: bool,
    pub is_elective: bool,
    pub exam_track: ExamTrack,
    pub school_type_scope: Option<SchoolType>,
}

impl From<TryoutSession> for SessionResponse {
    fn from(s: TryoutSession) -> Self {
        Self {
            id: s.id,
            title: s.title,
            session_type: session_type_str(s.session_type).to_string(),
            duration_minutes: s.duration_minutes,
            question_count: s.question_count,
            subject_filter: s.subject_filter,
            topic_filter: s.topic_filter,
            difficulty_filter: s.difficulty_filter,
            is_premium: s.is_premium,
            created_at: s.created_at,
            question_set_id: s.question_set_id,
            is_draft: s.is_draft,
            is_elective: s.is_elective,
            exam_track: s.exam_track,
            school_type_scope: s.school_type_scope,
        }
    }
}

pub fn session_type_str(t: SessionType) -> &'static str {
    match t {
        SessionType::Tryout => "tryout",
        SessionType::Drilling => "drilling",
        SessionType::Mini => "mini",
    }
}

pub fn parse_session_type(s: &str) -> Result<SessionType, crate::error::AppError> {
    match s {
        "tryout" => Ok(SessionType::Tryout),
        "drilling" => Ok(SessionType::Drilling),
        "mini" => Ok(SessionType::Mini),
        other => Err(crate::error::AppError::Validation(format!("unknown session type: {other}"))),
    }
}

#[derive(Debug, Serialize)]
pub struct PlayableQuestionResponse {
    pub id: Uuid,
    pub subject: String,
    pub topic: String,
    pub question_type: String,
    pub difficulty: String,
    pub stimulus: Option<String>,
    pub question_text: String,
    pub options: Option<Vec<String>>,
}

impl From<PlayableQuestion> for PlayableQuestionResponse {
    fn from(q: PlayableQuestion) -> Self {
        Self {
            id: q.id,
            subject: q.subject,
            topic: q.topic,
            question_type: q.question_type,
            difficulty: q.difficulty,
            stimulus: q.stimulus,
            question_text: q.question_text,
            options: q.options,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AttemptResponse {
    pub id: Uuid,
    pub session_id: Uuid,
    pub session_title: String,
    pub session_type: String,
    pub status: String,
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

impl From<Attempt> for AttemptResponse {
    fn from(a: Attempt) -> Self {
        Self {
            id: a.id,
            session_id: a.session_id,
            session_title: a.session_title,
            session_type: session_type_str(a.session_type).to_string(),
            status: match a.status {
                crate::domain::tryout::AttemptStatus::InProgress => "in_progress".to_string(),
                crate::domain::tryout::AttemptStatus::Submitted => "submitted".to_string(),
            },
            duration_minutes: a.duration_minutes,
            started_at: a.started_at,
            submitted_at: a.submitted_at,
            score: a.score,
            accuracy: a.accuracy,
            correct_count: a.correct_count,
            wrong_count: a.wrong_count,
            unanswered_count: a.unanswered_count,
            time_used_seconds: a.time_used_seconds,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AttemptWithQuestionsResponse {
    pub attempt: AttemptResponse,
    pub questions: Vec<PlayableQuestionResponse>,
}

impl From<AttemptWithQuestions> for AttemptWithQuestionsResponse {
    fn from(a: AttemptWithQuestions) -> Self {
        Self {
            attempt: a.attempt.into(),
            questions: a.questions.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SavedAnswerResponse {
    pub question_id: Uuid,
    pub answer_text: Option<String>,
    pub flagged: bool,
}

#[derive(Debug, Serialize)]
pub struct AttemptResumeResponse {
    pub attempt: AttemptResponse,
    pub questions: Vec<PlayableQuestionResponse>,
    /// Every answer already saved for this attempt (autosaved via PUT .../answers) — lets the
    /// player restore its local `answers`/`flagged` state instead of a blank quiz.
    pub answers: Vec<SavedAnswerResponse>,
}

impl From<AttemptResume> for AttemptResumeResponse {
    fn from(r: AttemptResume) -> Self {
        Self {
            attempt: r.attempt.into(),
            questions: r.questions.into_iter().map(Into::into).collect(),
            answers: r
                .answers
                .into_iter()
                .map(|a| SavedAnswerResponse { question_id: a.question_id, answer_text: a.answer_text, flagged: a.flagged })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SubjectBreakdownResponse {
    pub subject: String,
    pub correct: i32,
    pub total: i32,
}

impl From<SubjectBreakdown> for SubjectBreakdownResponse {
    fn from(b: SubjectBreakdown) -> Self {
        Self { subject: b.subject, correct: b.correct, total: b.total }
    }
}

#[derive(Debug, Serialize)]
pub struct AttemptResultResponse {
    pub attempt: AttemptResponse,
    pub subject_breakdown: Vec<SubjectBreakdownResponse>,
    /// "Estimasi IRT" — see `domain::irt` doc comment. `null` when not enough of this attempt's
    /// questions currently have a trusted item calibration.
    pub irt_score: Option<f64>,
    /// Which score(s) the frontend should display for this attempt — "instant" | "irt" | "both".
    /// See `domain::platform_settings::ScoreDisplayMode` doc comment.
    pub score_display_mode: String,
}

impl From<AttemptResult> for AttemptResultResponse {
    fn from(r: AttemptResult) -> Self {
        Self {
            attempt: r.attempt.into(),
            subject_breakdown: r.subject_breakdown.into_iter().map(Into::into).collect(),
            irt_score: r.irt_score,
            score_display_mode: r.score_display_mode.as_str().to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveAnswerPayload {
    pub question_id: Uuid,
    pub answer_text: Option<String>,
    #[serde(default)]
    pub flagged: bool,
}

#[derive(Debug, Deserialize)]
pub struct SubmitAttemptPayload {
    pub time_used_seconds: i32,
}

#[derive(Debug, Serialize)]
pub struct ReviewItemResponse {
    pub question_id: Uuid,
    pub code: String,
    pub subject: String,
    /// Lets the review screen branch its rendering (e.g. "kiri::kanan" pair display for
    /// `matching`, no correct/incorrect verdict for `essay` since it's self-check only).
    pub question_type: String,
    pub stimulus: Option<String>,
    pub question_text: String,
    pub options: Option<Vec<String>>,
    pub correct_answer: String,
    pub explanation: String,
    pub user_answer: Option<String>,
    pub is_correct: Option<bool>,
    pub flagged: bool,
}

impl From<ReviewItem> for ReviewItemResponse {
    fn from(r: ReviewItem) -> Self {
        Self {
            question_id: r.question.id,
            code: r.question.code,
            subject: r.question.subject,
            question_type: r.question.question_type.as_str().to_string(),
            stimulus: r.question.stimulus,
            question_text: r.question.question_text,
            options: r.question.options,
            correct_answer: r.question.correct_answer,
            explanation: r.question.explanation,
            user_answer: r.user_answer,
            is_correct: r.is_correct,
            flagged: r.flagged,
        }
    }
}
