use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::event::Event;

#[derive(Debug, Deserialize, Validate)]
pub struct EventPayload {
    #[validate(length(min = 1, message = "event name is required"))]
    pub name: String,
    /// "tryout" | "drilling" | "mini"
    pub event_type: String,
    #[serde(default)]
    pub description: String,
    pub session_id: Uuid,
    pub start_date: NaiveDate,
    #[serde(default)]
    pub end_date: Option<NaiveDate>,
    #[serde(default)]
    pub max_participants: Option<i32>,
    #[serde(default)]
    pub price: i32,
    #[serde(default)]
    pub prizes: String,
    #[serde(default = "default_target_class")]
    pub target_class: String,
    /// "draft" | "upcoming" | "ongoing" | "completed" | "archived" — ignored on create
    /// (always starts as draft).
    #[serde(default)]
    pub status: Option<String>,
}

fn default_target_class() -> String {
    "Kelas 12".to_string()
}

impl EventPayload {
    pub fn into_create_input(self) -> Result<crate::application::event_service::CreateEventInput, crate::error::AppError> {
        Ok(crate::application::event_service::CreateEventInput {
            name: self.name,
            event_type: self.event_type.parse().map_err(crate::error::AppError::Validation)?,
            description: self.description,
            session_id: self.session_id,
            start_date: self.start_date,
            end_date: self.end_date,
            max_participants: self.max_participants,
            price: self.price,
            prizes: self.prizes,
            target_class: self.target_class,
        })
    }

    pub fn into_update_input(self) -> Result<crate::application::event_service::UpdateEventInput, crate::error::AppError> {
        let status = self.status.as_deref().unwrap_or("draft");
        Ok(crate::application::event_service::UpdateEventInput {
            name: self.name,
            event_type: self.event_type.parse().map_err(crate::error::AppError::Validation)?,
            description: self.description,
            session_id: self.session_id,
            start_date: self.start_date,
            end_date: self.end_date,
            max_participants: self.max_participants,
            price: self.price,
            status: status.parse().map_err(crate::error::AppError::Validation)?,
            prizes: self.prizes,
            target_class: self.target_class,
        })
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct EventStatusPayload {
    /// "draft" | "upcoming" | "ongoing" | "completed" | "archived"
    #[validate(length(min = 1))]
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub id: Uuid,
    pub name: String,
    pub event_type: String,
    pub description: String,
    pub session_id: Uuid,
    pub session_title: String,
    pub duration_minutes: i32,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub max_participants: Option<i32>,
    pub participants: i64,
    pub price: i32,
    pub status: String,
    pub prizes: String,
    pub target_class: String,
    pub created_at: DateTime<Utc>,
}

impl From<Event> for EventResponse {
    fn from(e: Event) -> Self {
        Self {
            id: e.id,
            name: e.name,
            event_type: e.event_type.as_str().to_string(),
            description: e.description,
            session_id: e.session_id,
            session_title: e.session_title,
            duration_minutes: e.duration_minutes,
            start_date: e.start_date,
            end_date: e.end_date,
            max_participants: e.max_participants,
            participants: e.participants,
            price: e.price,
            status: e.status.as_str().to_string(),
            prizes: e.prizes,
            target_class: e.target_class,
            created_at: e.created_at,
        }
    }
}
