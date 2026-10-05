//! Event entity — scheduled tryout/drilling/mini events (Manajemen Event), mirroring the
//! reference `EventManagement` component. An `Event` schedules an existing
//! `TryoutSession` ("paket soal") over a date range with capacity/pricing/lifecycle
//! metadata layered on top. Duration is always inherited from the referenced session
//! (single source of truth), and `participants` is never stored on the row — it is
//! always derived live from real `attempts` rows for that session (see
//! `PostgresEventRepository`), so there is no risk of a stale or hardcoded headcount.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Tryout,
    Drilling,
    Mini,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Tryout => "tryout",
            EventType::Drilling => "drilling",
            EventType::Mini => "mini",
        }
    }
}

impl std::str::FromStr for EventType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tryout" => Ok(EventType::Tryout),
            "drilling" => Ok(EventType::Drilling),
            "mini" => Ok(EventType::Mini),
            other => Err(format!("unknown event_type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Draft,
    Upcoming,
    Ongoing,
    Completed,
    Archived,
}

impl EventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventStatus::Draft => "draft",
            EventStatus::Upcoming => "upcoming",
            EventStatus::Ongoing => "ongoing",
            EventStatus::Completed => "completed",
            EventStatus::Archived => "archived",
        }
    }
}

impl std::str::FromStr for EventStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(EventStatus::Draft),
            "upcoming" => Ok(EventStatus::Upcoming),
            "ongoing" => Ok(EventStatus::Ongoing),
            "completed" => Ok(EventStatus::Completed),
            "archived" => Ok(EventStatus::Archived),
            other => Err(format!("unknown event status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub event_type: EventType,
    pub description: String,
    pub session_id: Uuid,
    /// Denormalized from the referenced session for display convenience.
    pub session_title: String,
    pub duration_minutes: i32,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub max_participants: Option<i32>,
    /// Real distinct-student count derived from `attempts` for this event's session —
    /// never stored, always computed at read time.
    pub participants: i64,
    pub price: i32,
    pub status: EventStatus,
    pub prizes: String,
    pub target_class: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}
