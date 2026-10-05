//! Set Soal — an admin/content-curated, ordered, fixed list of specific questions. This is
//! an alternative to `TryoutSession`'s existing random-filter draw (`random_approved`), for
//! when a content team wants a guaranteed exact composition instead of trusting a random
//! pool match. See migration `20250101000021_question_sets.sql` for the full rationale.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionSet {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Live `COUNT` of `question_set_items` rows for this set — never stored, computed at
    /// read time by the repository, same "honest derived value" pattern as
    /// `Package::discount_percent()`.
    pub item_count: i64,
}
