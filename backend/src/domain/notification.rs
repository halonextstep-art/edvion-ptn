//! Real per-user notification feed — see migration 20250101000027 doc comment.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    /// Free-form event tag, e.g. "purchase_approved" — lets the frontend pick an icon/color
    /// without needing a Rust enum for every future trigger.
    pub kind: String,
    pub title: String,
    pub message: String,
    /// Tab key the frontend should switch to when this notification is clicked, if any —
    /// matches the `activeTab`/`activeView` string values already used by each dashboard's
    /// `@navigate="activeTab = $event"` convention.
    pub link_tab: Option<String>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
