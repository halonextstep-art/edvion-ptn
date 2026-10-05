//! Orchestrates the "Import Soal" bulk `.docx` flow end to end: read file bytes -> parse (see
//! `question_import`) -> either a read-only preview or a real batch of created questions.
//!
//! The DSL format (see `question_import` module doc) carries no per-question subject/topic —
//! a real content author uploads one file per mapel/topic batch, the same way they'd fill out
//! those fields once in `QuestionFormPanel.vue` before writing several questions in a row. So
//! `ImportBatchMeta` supplies those fields once for the whole upload.
//!
//! No server-side caching sits between preview and commit: `commit` re-reads and re-parses the
//! exact same uploaded bytes the caller sends it. This keeps both endpoints fully stateless at
//! the cost of asking the frontend to hold onto the original file and re-submit it — a
//! reasonable trade for a feature with no other need for upload-session state.

use std::sync::Arc;

use uuid::Uuid;

use crate::application::question_import::{self, ImportIssue, ParsedAnswer, ParsedImportQuestion};
use crate::application::question_service::{CreateQuestionInput, QuestionService};
use crate::domain::question::{Difficulty, Question, QuestionType};
use crate::domain::user::Role;
use crate::error::AppResult;
use crate::infrastructure::docx_reader::read_docx_paragraphs;
use crate::infrastructure::encoding::to_data_uri;
use crate::infrastructure::storage::{save_bytes, QUESTION_IMAGE_POLICY};
use crate::interfaces::http::middleware::AuthUser;

/// Fields shared by every question in one import batch.
pub struct ImportBatchMeta {
    pub subject: String,
    pub topic: String,
    pub subtopic: String,
    pub bloom_level: String,
    pub time_limit: Option<i32>,
    pub submit_for_review: bool,
    pub question_set_id: Option<Uuid>,
}

pub struct PreviewedQuestion {
    pub question_index: usize,
    pub no_soal: String,
    pub question_type: QuestionType,
    pub question_text: String,
    pub difficulty: Difficulty,
    pub explanation: String,
    /// For `MatchingImage` rows, each side of a `"kiri::kanan"` pair is a throwaway `data:`
    /// URI here — nothing has been written to disk yet (see module doc).
    pub options: Option<Vec<String>>,
    pub correct_answer: String,
    /// Markdown stimulus with any `qimg://N` image placeholders already resolved to throwaway
    /// `data:` URIs — see `question_import` module doc's "Rich content" section.
    pub stimulus: Option<String>,
}

pub struct ImportPreview {
    pub questions: Vec<PreviewedQuestion>,
    pub issues: Vec<ImportIssue>,
}

pub struct ImportCommitResult {
    pub created: Vec<Question>,
    pub issues: Vec<ImportIssue>,
}

pub struct QuestionImportService {
    questions: Arc<QuestionService>,
}

impl QuestionImportService {
    pub fn new(questions: Arc<QuestionService>) -> Self {
        Self { questions }
    }

    /// Read-only: parses the file and inlines any `MatchingImage` pair images as `data:` URIs
    /// so the caller can render full thumbnails without anything touching disk.
    pub fn preview(&self, docx_bytes: &[u8]) -> AppResult<ImportPreview> {
        let paragraphs = read_docx_paragraphs(docx_bytes)?;
        let parsed = question_import::parse_paragraphs(&paragraphs);

        let questions = parsed
            .questions
            .into_iter()
            .map(|q| {
                let (options, correct_answer) = resolve_answer_for_preview(q.answer);
                let stimulus = resolve_stimulus_for_preview(q.stimulus, &q.stimulus_images);
                PreviewedQuestion {
                    question_index: q.question_index,
                    no_soal: q.no_soal,
                    question_type: q.question_type,
                    question_text: q.question_text,
                    difficulty: q.difficulty,
                    explanation: q.explanation,
                    options,
                    correct_answer,
                    stimulus,
                }
            })
            .collect();

        Ok(ImportPreview { questions, issues: parsed.issues })
    }

    /// Re-parses the same bytes (see module doc) and actually creates every successfully
    /// parsed question through the exact same `QuestionService::create` path a manually
    /// authored question goes through — same authorization, same draft/review/approved status
    /// logic, same `validate_answerable` safety net. `MatchingImage` pairs are persisted to
    /// real `/uploads/question-images/...` URLs only here, never during `preview`.
    pub async fn commit(&self, actor: &AuthUser, docx_bytes: &[u8], meta: &ImportBatchMeta) -> AppResult<ImportCommitResult> {
        actor.require_role(&[Role::Admin, Role::Content])?;

        let paragraphs = read_docx_paragraphs(docx_bytes)?;
        let parsed = question_import::parse_paragraphs(&paragraphs);

        let mut created = Vec::new();
        let mut issues = parsed.issues;

        for q in parsed.questions {
            let question_index = q.question_index;
            let no_soal = q.no_soal.clone();
            match self.create_one(actor, meta, q).await {
                Ok(question) => created.push(question),
                Err(e) => issues.push(ImportIssue { question_index, no_soal: Some(no_soal), message: format!("Gagal menyimpan: {e}") }),
            }
        }

        Ok(ImportCommitResult { created, issues })
    }

    async fn create_one(&self, actor: &AuthUser, meta: &ImportBatchMeta, q: ParsedImportQuestion) -> AppResult<Question> {
        let (options, correct_answer) = resolve_answer_for_commit(q.answer).await?;
        let stimulus = resolve_stimulus_for_commit(q.stimulus, q.stimulus_images).await?;

        self.questions
            .create(
                actor,
                CreateQuestionInput {
                    question_type: q.question_type,
                    subject: meta.subject.clone(),
                    topic: meta.topic.clone(),
                    subtopic: meta.subtopic.clone(),
                    difficulty: q.difficulty,
                    bloom_level: meta.bloom_level.clone(),
                    stimulus,
                    question_text: q.question_text,
                    options,
                    correct_answer,
                    explanation: q.explanation,
                    tags: Vec::new(),
                    time_limit: meta.time_limit,
                    submit_for_review: meta.submit_for_review,
                    question_set_id: meta.question_set_id,
                },
            )
            .await
    }
}

/// Replaces every `qimg://N` placeholder in `stimulus` (see `question_import` module doc's
/// "Rich content" section) with a throwaway `data:` URI built directly from the extracted
/// `.docx` image bytes — nothing touches disk here, mirroring `resolve_answer_for_preview`'s
/// treatment of `MatchingImage` pairs.
fn resolve_stimulus_for_preview(stimulus: Option<String>, images: &[(Vec<u8>, String)]) -> Option<String> {
    let mut text = stimulus?;
    for (idx, (bytes, mime)) in images.iter().enumerate() {
        text = text.replace(&format!("qimg://{idx}"), &to_data_uri(bytes, mime));
    }
    Some(text)
}

/// Same substitution as `resolve_stimulus_for_preview`, except each placeholder resolves to a
/// real persisted `/uploads/...` URL (via `infrastructure::storage::save_bytes`) since this
/// runs only at commit time.
async fn resolve_stimulus_for_commit(stimulus: Option<String>, images: Vec<(Vec<u8>, String)>) -> AppResult<Option<String>> {
    let Some(mut text) = stimulus else { return Ok(None) };
    for (idx, (bytes, mime)) in images.into_iter().enumerate() {
        let url = save_bytes(&bytes, &mime, &QUESTION_IMAGE_POLICY).await?;
        text = text.replace(&format!("qimg://{idx}"), &url);
    }
    Ok(Some(text))
}

fn resolve_answer_for_preview(answer: ParsedAnswer) -> (Option<Vec<String>>, String) {
    match answer {
        ParsedAnswer::Plain { options, correct_answer } => (options, correct_answer),
        ParsedAnswer::ImagePairs(pairs) => {
            let options: Vec<String> = pairs
                .into_iter()
                .map(|pair| format!("{}::{}", to_data_uri(&pair.left_bytes, &pair.left_mime), to_data_uri(&pair.right_bytes, &pair.right_mime)))
                .collect();
            let correct_answer = options.join(", ");
            (Some(options), correct_answer)
        }
    }
}

async fn resolve_answer_for_commit(answer: ParsedAnswer) -> AppResult<(Option<Vec<String>>, String)> {
    match answer {
        ParsedAnswer::Plain { options, correct_answer } => Ok((options, correct_answer)),
        ParsedAnswer::ImagePairs(pairs) => {
            let mut options = Vec::with_capacity(pairs.len());
            for pair in pairs {
                let left_url = save_bytes(&pair.left_bytes, &pair.left_mime, &QUESTION_IMAGE_POLICY).await?;
                let right_url = save_bytes(&pair.right_bytes, &pair.right_mime, &QUESTION_IMAGE_POLICY).await?;
                options.push(format!("{left_url}::{right_url}"));
            }
            let correct_answer = options.join(", ");
            Ok((Some(options), correct_answer))
        }
    }
}
