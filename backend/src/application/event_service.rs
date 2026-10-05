//! Event (Manajemen Event) CRUD + lifecycle use-cases. Admin-only, mirroring the
//! reference `EventManagement`. An event schedules an existing `TryoutSession`
//! ("paket soal") — we validate that session exists up front so callers get a clear
//! validation error instead of a raw FK-violation 500.

use std::sync::Arc;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::domain::event::{Event, EventStatus, EventType};
use crate::domain::repository::{EventRepository, EventUpdate, NewEvent, TryoutSessionRepository};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct CreateEventInput {
    pub name: String,
    pub event_type: EventType,
    pub description: String,
    pub session_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub max_participants: Option<i32>,
    pub price: i32,
    pub prizes: String,
    pub target_class: String,
}

pub struct UpdateEventInput {
    pub name: String,
    pub event_type: EventType,
    pub description: String,
    pub session_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub max_participants: Option<i32>,
    pub price: i32,
    pub status: EventStatus,
    pub prizes: String,
    pub target_class: String,
}

pub struct EventService {
    events: Arc<dyn EventRepository>,
    sessions: Arc<dyn TryoutSessionRepository>,
}

impl EventService {
    pub fn new(events: Arc<dyn EventRepository>, sessions: Arc<dyn TryoutSessionRepository>) -> Self {
        Self { events, sessions }
    }

    pub async fn create(&self, actor: &AuthUser, input: CreateEventInput) -> AppResult<Event> {
        actor.require_role(&[Role::Admin])?;
        validate_name(&input.name)?;
        self.ensure_session_exists(input.session_id).await?;
        self.events
            .create(NewEvent {
                name: input.name,
                event_type: input.event_type,
                description: input.description,
                session_id: input.session_id,
                start_date: input.start_date,
                end_date: input.end_date,
                max_participants: input.max_participants,
                price: input.price,
                prizes: input.prizes,
                target_class: input.target_class,
                created_by: actor.user_id,
            })
            .await
    }

    pub async fn list(&self, actor: &AuthUser) -> AppResult<Vec<Event>> {
        actor.require_role(&[Role::Admin])?;
        self.events.list().await
    }

    /// Student/school-portal "Event Mendatang" widget — any authenticated role can see
    /// what's upcoming, but never the full admin CRUD list. Filtered + sorted in Rust
    /// since the event catalog is small; no separate scoped SQL query needed.
    pub async fn list_upcoming(&self, actor: &AuthUser, limit: i64) -> AppResult<Vec<Event>> {
        actor.require_role(&[Role::Admin, Role::Student, Role::School])?;
        let mut events = self.events.list().await?;
        events.retain(|e| matches!(e.status, EventStatus::Upcoming | EventStatus::Ongoing));
        events.sort_by_key(|e| e.start_date);
        events.truncate(limit.clamp(1, 50) as usize);
        Ok(events)
    }

    pub async fn get(&self, actor: &AuthUser, id: Uuid) -> AppResult<Event> {
        actor.require_role(&[Role::Admin])?;
        self.events
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("event {id} not found")))
    }

    pub async fn update(&self, actor: &AuthUser, id: Uuid, input: UpdateEventInput) -> AppResult<Event> {
        actor.require_role(&[Role::Admin])?;
        validate_name(&input.name)?;
        self.ensure_session_exists(input.session_id).await?;
        self.events
            .update(
                id,
                EventUpdate {
                    name: input.name,
                    event_type: input.event_type,
                    description: input.description,
                    session_id: input.session_id,
                    start_date: input.start_date,
                    end_date: input.end_date,
                    max_participants: input.max_participants,
                    price: input.price,
                    status: input.status,
                    prizes: input.prizes,
                    target_class: input.target_class,
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("event {id} not found")))
    }

    pub async fn set_status(&self, actor: &AuthUser, id: Uuid, status: EventStatus) -> AppResult<Event> {
        actor.require_role(&[Role::Admin])?;
        self.events
            .set_status(id, status)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("event {id} not found")))
    }

    pub async fn delete(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        let deleted = self.events.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("event {id} not found")));
        }
        Ok(())
    }

    async fn ensure_session_exists(&self, session_id: Uuid) -> AppResult<()> {
        self.sessions
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| AppError::Validation("paket soal (session) yang dipilih tidak ditemukan".to_string()))?;
        Ok(())
    }
}

fn validate_name(name: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("event name is required".to_string()));
    }
    Ok(())
}
