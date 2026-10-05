//! "Set Soal" use-cases — curated, ordered, fixed question lists. See
//! `domain::question_set` doc comment for the rationale vs. the random-filter draw.
//! Reading (list/get_items) is open to any authenticated role since admin and content
//! authors both need to preview membership while authoring; mutations are gated to
//! Admin/Content, mirroring `TaxonomyService`'s admin-only-mutation shape but slightly
//! wider since content authors are meant to self-serve set assembly while writing
//! questions, not just admins.

use std::sync::Arc;

use uuid::Uuid;

use crate::domain::question::Question;
use crate::domain::question_set::QuestionSet;
use crate::domain::repository::QuestionSetRepository;
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct QuestionSetService {
    sets: Arc<dyn QuestionSetRepository>,
}

impl QuestionSetService {
    pub fn new(sets: Arc<dyn QuestionSetRepository>) -> Self {
        Self { sets }
    }

    /// Any authenticated role may browse the catalogue of sets.
    pub async fn list(&self) -> AppResult<Vec<QuestionSet>> {
        self.sets.list().await
    }

    /// Any authenticated role — admin/content need to preview membership while authoring
    /// or assigning a session.
    pub async fn get_items(&self, set_id: Uuid) -> AppResult<Vec<Question>> {
        self.sets.list_items(set_id).await
    }

    pub async fn create(&self, actor: &AuthUser, name: String, description: String) -> AppResult<QuestionSet> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        validate_name(&name)?;
        self.sets.create(name, description, actor.user_id).await
    }

    pub async fn update(&self, actor: &AuthUser, id: Uuid, name: String, description: String) -> AppResult<QuestionSet> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        validate_name(&name)?;
        self.sets
            .update(id, name, description)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("question set {id} not found")))
    }

    /// v1 keeps this a simple role gate (Admin or Content) rather than also checking
    /// whether the set is actively assigned to a live session — `tryout_sessions.question_set_id`
    /// is `ON DELETE SET NULL`, so a delete can never corrupt a session either way, it would
    /// just silently fall back that session to its random-filter fields. A stronger
    /// "block delete if in use" guard is a reasonable v2 addition but is intentionally not
    /// built here to avoid over-engineering.
    pub async fn delete(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        let deleted = self.sets.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("question set {id} not found")));
        }
        Ok(())
    }

    pub async fn add_item(&self, actor: &AuthUser, set_id: Uuid, question_id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        let current_items = self.sets.list_items(set_id).await?;
        self.sets.add_item(set_id, question_id, current_items.len() as i32).await
    }

    pub async fn remove_item(&self, actor: &AuthUser, set_id: Uuid, question_id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        let removed = self.sets.remove_item(set_id, question_id).await?;
        if !removed {
            return Err(AppError::NotFound(format!(
                "question {question_id} is not a member of set {set_id}"
            )));
        }
        Ok(())
    }
}

fn validate_name(name: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("nama set soal wajib diisi".to_string()));
    }
    Ok(())
}
