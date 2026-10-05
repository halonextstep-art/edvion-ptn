//! Subject taxonomy — admin-editable catalogue of exam/assessment categories (SNBT,
//! TKA-IPA, TKA-IPS, AKM) and the subjects within each. See migration
//! `20250101000020_subject_taxonomy.sql` for the full rationale: these are four distinct
//! exams/assessments (not variants of one), and the underlying policy has already changed
//! once in the last year, hence storing this as data instead of hardcoding it in the
//! frontend. This module intentionally has no relationship to `questions.subject` (which
//! stays a free-text column) or to the separate SNBP rapor subject list in
//! `domain::rationalization` — it is purely a catalogue that dropdowns read from.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectCategory {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub code: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A category with its subjects nested — the shape the frontend actually wants for
/// grouped dropdowns (category header -> subject options), so the API doesn't force every
/// consumer to re-group two flat lists client-side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryWithSubjects {
    #[serde(flatten)]
    pub category: SubjectCategory,
    pub subjects: Vec<Subject>,
}
