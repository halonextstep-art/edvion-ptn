use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::question::{Difficulty, Question, QuestionType};

#[derive(Debug, Deserialize, Validate)]
pub struct QuestionPayload {
    pub question_type: String,
    #[validate(length(min = 1))]
    pub subject: String,
    #[validate(length(min = 1))]
    pub topic: String,
    #[validate(length(min = 1))]
    pub subtopic: String,
    pub difficulty: String,
    #[validate(length(min = 1))]
    pub bloom_level: String,
    pub stimulus: Option<String>,
    #[validate(length(min = 1, message = "question text is required"))]
    pub question_text: String,
    pub options: Option<Vec<String>>,
    #[validate(length(min = 1, message = "correct_answer is required"))]
    pub correct_answer: String,
    #[serde(default)]
    pub explanation: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub time_limit: Option<i32>,
    #[serde(default)]
    pub submit_for_review: bool,
    /// Optional "Set Soal" to attach this question to as part of the same save — see
    /// `CreateQuestionInput::question_set_id` doc comment.
    #[serde(default)]
    pub question_set_id: Option<Uuid>,
}

impl QuestionPayload {
    pub fn into_service_input(
        self,
    ) -> Result<crate::application::question_service::CreateQuestionInput, crate::error::AppError> {
        let question_type: QuestionType = self
            .question_type
            .parse()
            .map_err(crate::error::AppError::Validation)?;
        let difficulty: Difficulty = self.difficulty.parse().map_err(crate::error::AppError::Validation)?;

        Ok(crate::application::question_service::CreateQuestionInput {
            question_type,
            subject: self.subject,
            topic: self.topic,
            subtopic: self.subtopic,
            difficulty,
            bloom_level: self.bloom_level,
            stimulus: self.stimulus,
            question_text: self.question_text,
            options: self.options,
            correct_answer: self.correct_answer,
            explanation: self.explanation,
            tags: self.tags,
            time_limit: self.time_limit,
            submit_for_review: self.submit_for_review,
            question_set_id: self.question_set_id,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct QuestionListQuery {
    pub search: Option<String>,
    pub subject: Option<String>,
    pub status: Option<String>,
    pub difficulty: Option<String>,
    pub question_type: Option<String>,
    pub mine: Option<bool>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReviewPayload {
    /// One of: "approve" | "reject" | "revision"
    pub action: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QuestionResponse {
    pub id: Uuid,
    pub code: String,
    pub question_type: String,
    pub subject: String,
    pub topic: String,
    pub subtopic: String,
    pub difficulty: String,
    pub bloom_level: String,
    pub stimulus: Option<String>,
    pub question_text: String,
    pub options: Option<Vec<String>>,
    pub correct_answer: String,
    pub explanation: String,
    pub tags: Vec<String>,
    pub status: String,
    pub review_note: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub created_by_name: String,
    pub usage_count: i32,
    pub average_score: f64,
    pub time_limit: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Question> for QuestionResponse {
    fn from(q: Question) -> Self {
        Self {
            id: q.id,
            code: q.code,
            question_type: q.question_type.as_str().to_string(),
            subject: q.subject,
            topic: q.topic,
            subtopic: q.subtopic,
            difficulty: q.difficulty.as_str().to_string(),
            bloom_level: q.bloom_level,
            stimulus: q.stimulus,
            question_text: q.question_text,
            options: q.options,
            correct_answer: q.correct_answer,
            explanation: q.explanation,
            tags: q.tags,
            status: q.status.as_str().to_string(),
            review_note: q.review_note,
            reviewed_at: q.reviewed_at,
            created_by: q.created_by,
            created_by_name: q.created_by_name,
            usage_count: q.usage_count,
            average_score: q.average_score,
            time_limit: q.time_limit,
            created_at: q.created_at,
            updated_at: q.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct QuestionListResponse {
    pub items: Vec<QuestionResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

// ─── Bulk "Import Soal" .docx (see application::question_import_service) ──────────────────

#[derive(Debug, Serialize)]
pub struct ImportIssueResponse {
    pub question_index: usize,
    pub no_soal: Option<String>,
    pub message: String,
}

impl From<crate::application::question_import::ImportIssue> for ImportIssueResponse {
    fn from(i: crate::application::question_import::ImportIssue) -> Self {
        Self { question_index: i.question_index, no_soal: i.no_soal, message: i.message }
    }
}

#[derive(Debug, Serialize)]
pub struct PreviewedQuestionResponse {
    pub question_index: usize,
    pub no_soal: String,
    pub question_type: String,
    pub question_text: String,
    pub difficulty: String,
    pub explanation: String,
    /// For `matching_image` rows, each pair side is a throwaway `data:` URI (see
    /// `application::question_import_service` doc comment) — nothing is on disk yet.
    pub options: Option<Vec<String>>,
    pub correct_answer: String,
    /// Markdown stimulus, if the block had a `"Stimulus:"` line and/or an inline image
    /// paragraph — any embedded image is likewise a throwaway `data:` URI at preview time.
    pub stimulus: Option<String>,
}

impl From<crate::application::question_import_service::PreviewedQuestion> for PreviewedQuestionResponse {
    fn from(q: crate::application::question_import_service::PreviewedQuestion) -> Self {
        Self {
            question_index: q.question_index,
            no_soal: q.no_soal,
            question_type: q.question_type.as_str().to_string(),
            question_text: q.question_text,
            difficulty: q.difficulty.as_str().to_string(),
            explanation: q.explanation,
            options: q.options,
            correct_answer: q.correct_answer,
            stimulus: q.stimulus,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ImportPreviewResponse {
    pub questions: Vec<PreviewedQuestionResponse>,
    pub issues: Vec<ImportIssueResponse>,
}

impl From<crate::application::question_import_service::ImportPreview> for ImportPreviewResponse {
    fn from(p: crate::application::question_import_service::ImportPreview) -> Self {
        Self {
            questions: p.questions.into_iter().map(Into::into).collect(),
            issues: p.issues.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ImportCommitResponse {
    pub created: Vec<QuestionResponse>,
    pub issues: Vec<ImportIssueResponse>,
}

impl From<crate::application::question_import_service::ImportCommitResult> for ImportCommitResponse {
    fn from(r: crate::application::question_import_service::ImportCommitResult) -> Self {
        Self {
            created: r.created.into_iter().map(Into::into).collect(),
            issues: r.issues.into_iter().map(Into::into).collect(),
        }
    }
}
