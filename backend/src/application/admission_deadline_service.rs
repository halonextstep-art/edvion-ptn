//! Kalender Deadline SNBP/SNBT/UTBK CRUD + read use-cases. Admin-only writes (the
//! single source of official SNPMB dates); read access is shared with School/Student
//! for the "Deadline Mendatang" countdown widget — mirrors `EventService::list_upcoming`.

use std::sync::Arc;

use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::admission_deadline::{AdmissionDeadline, AdmissionTrack};
use crate::domain::repository::{AdmissionDeadlineRepository, AdmissionDeadlineUpdate, NewAdmissionDeadline};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct CreateAdmissionDeadlineInput {
    pub track: AdmissionTrack,
    pub year: i32,
    pub label: String,
    pub description: String,
    pub deadline_date: NaiveDate,
}

pub struct UpdateAdmissionDeadlineInput {
    pub track: AdmissionTrack,
    pub year: i32,
    pub label: String,
    pub description: String,
    pub deadline_date: NaiveDate,
}

pub struct AdmissionDeadlineService {
    deadlines: Arc<dyn AdmissionDeadlineRepository>,
}

impl AdmissionDeadlineService {
    pub fn new(deadlines: Arc<dyn AdmissionDeadlineRepository>) -> Self {
        Self { deadlines }
    }

    pub async fn create(&self, actor: &AuthUser, input: CreateAdmissionDeadlineInput) -> AppResult<AdmissionDeadline> {
        actor.require_role(&[Role::Admin])?;
        validate_label(&input.label)?;
        self.deadlines
            .create(NewAdmissionDeadline {
                track: input.track,
                year: input.year,
                label: input.label,
                description: input.description,
                deadline_date: input.deadline_date,
                created_by: actor.user_id,
            })
            .await
    }

    /// Full admin CRUD list — includes past deadlines too, for record-keeping.
    pub async fn list(&self, actor: &AuthUser) -> AppResult<Vec<AdmissionDeadline>> {
        actor.require_role(&[Role::Admin])?;
        self.deadlines.list().await
    }

    /// School/Student/Admin "Deadline Mendatang" widget — only deadlines that haven't
    /// passed yet (today counts as still-upcoming), soonest first. Never the full
    /// history — see `EventService::list_upcoming` for the same shared-read pattern.
    pub async fn list_upcoming(&self, actor: &AuthUser, limit: i64) -> AppResult<Vec<AdmissionDeadline>> {
        actor.require_role(&[Role::Admin, Role::School, Role::Student])?;
        let today = Utc::now().date_naive();
        let mut deadlines = self.deadlines.list().await?;
        deadlines.retain(|d| d.deadline_date >= today);
        deadlines.sort_by_key(|d| d.deadline_date);
        deadlines.truncate(limit.clamp(1, 50) as usize);
        Ok(deadlines)
    }

    pub async fn get(&self, actor: &AuthUser, id: Uuid) -> AppResult<AdmissionDeadline> {
        actor.require_role(&[Role::Admin])?;
        self.deadlines
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("admission deadline {id} not found")))
    }

    pub async fn update(
        &self,
        actor: &AuthUser,
        id: Uuid,
        input: UpdateAdmissionDeadlineInput,
    ) -> AppResult<AdmissionDeadline> {
        actor.require_role(&[Role::Admin])?;
        validate_label(&input.label)?;
        self.deadlines
            .update(
                id,
                AdmissionDeadlineUpdate {
                    track: input.track,
                    year: input.year,
                    label: input.label,
                    description: input.description,
                    deadline_date: input.deadline_date,
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("admission deadline {id} not found")))
    }

    pub async fn delete(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        let deleted = self.deadlines.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("admission deadline {id} not found")));
        }
        Ok(())
    }
}

fn validate_label(label: &str) -> AppResult<()> {
    if label.trim().is_empty() {
        return Err(AppError::Validation("label deadline wajib diisi".to_string()));
    }
    Ok(())
}
