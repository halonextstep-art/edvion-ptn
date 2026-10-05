//! Question Bank use-cases: CRUD for questions plus the admin review workflow
//! (approve / reject / request revision). Authorization decisions live here (not in the
//! HTTP layer) so the rules are enforced no matter which interface calls the service.

use std::sync::Arc;

use uuid::Uuid;

use crate::domain::question::{generate_code, Difficulty, Question, QuestionStatus, QuestionType};
use crate::domain::repository::{NewQuestion, QuestionFilter, QuestionRepository, QuestionSetRepository, QuestionUpdate};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct CreateQuestionInput {
    pub question_type: QuestionType,
    pub subject: String,
    pub topic: String,
    pub subtopic: String,
    pub difficulty: Difficulty,
    pub bloom_level: String,
    pub stimulus: Option<String>,
    pub question_text: String,
    pub options: Option<Vec<String>>,
    pub correct_answer: String,
    pub explanation: String,
    pub tags: Vec<String>,
    pub time_limit: Option<i32>,
    /// If true and the caller is `content`, the question enters the review queue instead
    /// of staying a draft (mirrors "submit for review" in the reference UI).
    pub submit_for_review: bool,
    /// If `Some(set_id)`, this question is attached to that "Set Soal" as part of the same
    /// save — lets a content author assign a question to a curated set directly while
    /// writing it, instead of an admin manually curating the set afterward. See
    /// `domain::question_set` doc comment.
    pub question_set_id: Option<Uuid>,
}

pub enum ReviewAction {
    Approve,
    Reject,
    RequestRevision,
}

pub struct QuestionService {
    questions: Arc<dyn QuestionRepository>,
    question_sets: Arc<dyn QuestionSetRepository>,
}

impl QuestionService {
    pub fn new(questions: Arc<dyn QuestionRepository>, question_sets: Arc<dyn QuestionSetRepository>) -> Self {
        Self { questions, question_sets }
    }

    pub async fn create(&self, actor: &AuthUser, input: CreateQuestionInput) -> AppResult<Question> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        validate_answerable(&input)?;

        let status = if actor.role == Role::Admin {
            QuestionStatus::Approved
        } else if input.submit_for_review {
            QuestionStatus::Review
        } else {
            QuestionStatus::Draft
        };

        let code = generate_code(&input.subject, &input.topic, &short_id());
        let question_set_id = input.question_set_id;

        let question = self
            .questions
            .create(NewQuestion {
                code,
                question_type: input.question_type,
                subject: input.subject,
                topic: input.topic,
                subtopic: input.subtopic,
                difficulty: input.difficulty,
                bloom_level: input.bloom_level,
                stimulus: input.stimulus,
                question_text: input.question_text,
                options: input.options,
                correct_answer: input.correct_answer,
                explanation: input.explanation,
                tags: input.tags,
                status,
                created_by: actor.user_id,
                time_limit: input.time_limit,
            })
            .await?;

        // Design choice: surface a set-attach failure as an error instead of silently
        // swallowing it. The question row itself would already be committed at this point,
        // but silently dropping the caller's explicit set assignment would leave the
        // content author believing the question is in the set when it isn't — a confusing,
        // hard-to-notice data-consistency gap. Failing loudly here means the author sees the
        // problem immediately (the question still exists and can be re-attached via the set
        // management UI; this is not a destructive failure).
        if let Some(set_id) = question_set_id {
            let next_sort_order = self.question_sets.list_items(set_id).await?.len() as i32;
            self.question_sets.add_item(set_id, question.id, next_sort_order).await?;
        }

        Ok(question)
    }

    pub async fn get(&self, id: Uuid) -> AppResult<Question> {
        self.questions
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("question {id} not found")))
    }

    pub async fn list(&self, filter: QuestionFilter) -> AppResult<(Vec<Question>, i64)> {
        self.questions.list(filter).await
    }

    pub async fn update(&self, actor: &AuthUser, id: Uuid, input: CreateQuestionInput) -> AppResult<Question> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        validate_answerable(&input)?;

        let existing = self.get(id).await?;
        if actor.role != Role::Admin && !existing.is_owned_by(actor.user_id) {
            return Err(AppError::Forbidden(
                "you can only edit questions you created".to_string(),
            ));
        }

        let question_set_id = input.question_set_id;

        let updated = self
            .questions
            .update(
                id,
                QuestionUpdate {
                    question_type: input.question_type,
                    subject: input.subject,
                    topic: input.topic,
                    subtopic: input.subtopic,
                    difficulty: input.difficulty,
                    bloom_level: input.bloom_level,
                    stimulus: input.stimulus,
                    question_text: input.question_text,
                    options: input.options,
                    correct_answer: input.correct_answer,
                    explanation: input.explanation,
                    tags: input.tags,
                    time_limit: input.time_limit,
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("question {id} not found")))?;

        // Edit-time reassignment support: only ever ADDS the question to the given set
        // (idempotent — a no-op if it's already a member, see `add_item` doc comment).
        // Deliberately does NOT remove the question from any set it previously belonged to
        // if `question_set_id` changes or is cleared — a question may belong to multiple
        // sets going forward, and silently detaching it from a curated exam package on an
        // unrelated content edit would be a surprising, destructive side effect. Removing a
        // question from a set is a separate, explicit action via the set management UI.
        if let Some(set_id) = question_set_id {
            let next_sort_order = self.question_sets.list_items(set_id).await?.len() as i32;
            self.question_sets.add_item(set_id, updated.id, next_sort_order).await?;
        }

        Ok(updated)
    }

    pub async fn delete(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        let existing = self.get(id).await?;
        if actor.role != Role::Admin && !existing.is_owned_by(actor.user_id) {
            return Err(AppError::Forbidden(
                "you can only delete questions you created".to_string(),
            ));
        }
        let deleted = self.questions.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("question {id} not found")));
        }
        Ok(())
    }

    pub async fn review(
        &self,
        actor: &AuthUser,
        id: Uuid,
        action: ReviewAction,
        note: Option<String>,
    ) -> AppResult<Question> {
        actor.require_role(&[Role::Admin])?;

        if matches!(action, ReviewAction::Reject | ReviewAction::RequestRevision) && note.is_none() {
            return Err(AppError::Validation(
                "a note is required when rejecting or requesting revision".to_string(),
            ));
        }

        let new_status = match action {
            ReviewAction::Approve => QuestionStatus::Approved,
            ReviewAction::Reject => QuestionStatus::Rejected,
            ReviewAction::RequestRevision => QuestionStatus::Revision,
        };

        self.questions
            .set_review_status(id, new_status, note, actor.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("question {id} not found")))
    }

    /// Author sends a `draft`/`rejected`/`revision` question into the review queue — covers
    /// both "Kirim Review" (from draft) and "Ajukan Ulang" (from rejected/revision, after
    /// addressing feedback) in one action, since both just mean "put this in front of an
    /// admin". Content role may only do this for their own questions; admin may do it for any.
    pub async fn submit_for_review(&self, actor: &AuthUser, id: Uuid) -> AppResult<Question> {
        actor.require_role(&[Role::Admin, Role::Content])?;

        let existing = self.get(id).await?;
        if actor.role != Role::Admin && !existing.is_owned_by(actor.user_id) {
            return Err(AppError::Forbidden(
                "you can only submit questions you created".to_string(),
            ));
        }
        if !matches!(
            existing.status,
            QuestionStatus::Draft | QuestionStatus::Rejected | QuestionStatus::Revision
        ) {
            return Err(AppError::Validation(
                "only draft, rejected, or revision-requested questions can be submitted for review".to_string(),
            ));
        }

        self.questions
            .resubmit(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("question {id} not found")))
    }

    /// Approves every question belonging to any of the given "Set Soal" (curated fixed
    /// question lists) in one shot, instead of the content author's questions sitting in
    /// `draft`/`review` status one-by-one until an admin happens to open each of them in Bank
    /// Soal. Built specifically for the "Buat Paket + Subtes" wizard workflow: a Content-role
    /// author's newly-written questions default to `draft` (see `create` above), and a whole
    /// package can easily bundle dozens of them across several subtes — reviewing every single
    /// one individually before the package is usable is real friction for a case where the
    /// admin's actual intent is "I trust everything written for this package, approve it all".
    /// Skips (does not re-touch) questions already `approved`. Returns how many were changed.
    pub async fn bulk_approve_by_sets(&self, actor: &AuthUser, set_ids: &[Uuid]) -> AppResult<usize> {
        actor.require_role(&[Role::Admin])?;

        // `list_items` returns full `Question` rows (not just ids) already carrying `status`,
        // so no extra per-question lookup is needed here.
        let mut seen: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        let mut approved_count = 0usize;
        for set_id in set_ids {
            for question in self.question_sets.list_items(*set_id).await? {
                if !seen.insert(question.id) {
                    continue; // same question reused across multiple sets in this call
                }
                if question.status == QuestionStatus::Approved {
                    continue;
                }
                self.questions
                    .set_review_status(question.id, QuestionStatus::Approved, None, actor.user_id)
                    .await?;
                approved_count += 1;
            }
        }
        Ok(approved_count)
    }
}

fn validate_answerable(input: &CreateQuestionInput) -> AppResult<()> {
    if input.question_text.trim().is_empty() {
        return Err(AppError::Validation("question text cannot be empty".to_string()));
    }
    if input.correct_answer.trim().is_empty() {
        return Err(AppError::Validation("correct answer must be set".to_string()));
    }
    if matches!(input.question_type, QuestionType::MultipleChoice | QuestionType::ComplexMultiple) {
        match &input.options {
            Some(opts) if opts.len() >= 2 && opts.iter().all(|o| !o.trim().is_empty()) => {}
            _ => {
                return Err(AppError::Validation(
                    "multiple choice questions need at least 2 non-empty options".to_string(),
                ))
            }
        }
    }
    if matches!(input.question_type, QuestionType::TrueFalse) {
        let ans = input.correct_answer.trim();
        if ans != "Benar" && ans != "Salah" {
            return Err(AppError::Validation(
                "true/false questions require correct_answer to be exactly \"Benar\" or \"Salah\"".to_string(),
            ));
        }
    }
    if matches!(input.question_type, QuestionType::Matching | QuestionType::MatchingImage) {
        // Each `options` entry is one canonical "kiri::kanan" pair — see QuestionType::Matching.
        // `MatchingImage` shares this exact validation (both sides just happen to be image URLs
        // instead of plain text — a player-rendering distinction only). Reject embedded ", "
        // since that's the delimiter grading uses to split multiple pairs back apart
        // (tryout_service.rs normalize() + the player's matchingCurrentPairs()) — an item
        // containing it would silently mis-split and never grade correctly.
        match &input.options {
            Some(opts)
                if opts.len() >= 2
                    && opts.iter().all(|o| {
                        o.split("::").count() == 2
                            && o.split("::").all(|s| !s.trim().is_empty())
                            && !o.contains(", ")
                    }) => {}
            _ => {
                return Err(AppError::Validation(
                    "matching questions need at least 2 pairs, each formatted as \"kiri::kanan\" with both sides non-empty and without \", \"".to_string(),
                ))
            }
        }
    }
    if matches!(input.question_type, QuestionType::TrueFalseComplex) {
        // Each `options` entry is one statement; `correct_answer` is the parallel comma-space-
        // joined list of "Benar"/"Salah" tokens, same order, same count — see
        // QuestionType::TrueFalseComplex.
        let opts_len = match &input.options {
            Some(opts) if opts.len() >= 2 && opts.iter().all(|o| !o.trim().is_empty()) => opts.len(),
            _ => {
                return Err(AppError::Validation(
                    "true/false complex questions need at least 2 non-empty statements".to_string(),
                ))
            }
        };
        let answers: Vec<&str> = input.correct_answer.split(", ").map(|s| s.trim()).collect();
        if answers.len() != opts_len || !answers.iter().all(|a| *a == "Benar" || *a == "Salah") {
            return Err(AppError::Validation(
                "true/false complex correct_answer must be a \", \"-joined list of exactly \"Benar\"/\"Salah\" tokens, one per statement, in the same order".to_string(),
            ));
        }
    }
    if matches!(input.question_type, QuestionType::Ordering) {
        // `options` holds the steps in their correct order; `correct_answer` is the same list
        // comma-space-joined — see QuestionType::Ordering.
        match &input.options {
            Some(opts) if opts.len() >= 2 && opts.iter().all(|o| !o.trim().is_empty()) => {}
            _ => {
                return Err(AppError::Validation(
                    "ordering questions need at least 2 non-empty steps".to_string(),
                ))
            }
        }
    }
    Ok(())
}

fn short_id() -> String {
    Uuid::new_v4().to_string()[..6].to_uppercase()
}
