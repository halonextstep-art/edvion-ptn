use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::question::Question;
use crate::domain::question_set::QuestionSet;
use crate::interfaces::http::dto::question_dto::QuestionResponse;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateQuestionSetPayload {
    #[validate(length(min = 1, message = "nama set soal wajib diisi"))]
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateQuestionSetPayload {
    #[validate(length(min = 1, message = "nama set soal wajib diisi"))]
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct AddSetItemPayload {
    pub question_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct QuestionSetResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub item_count: i64,
}

impl From<QuestionSet> for QuestionSetResponse {
    fn from(s: QuestionSet) -> Self {
        Self {
            id: s.id,
            name: s.name,
            description: s.description,
            created_by: s.created_by,
            created_at: s.created_at,
            updated_at: s.updated_at,
            item_count: s.item_count,
        }
    }
}

/// Full question rows for a set's membership list — reuses the same `QuestionResponse`
/// shape the Bank Soal endpoints already return, so the frontend doesn't need a second
/// question DTO just to render a set's preview.
#[derive(Debug, Serialize)]
pub struct SetItemsResponse {
    pub items: Vec<QuestionResponse>,
}

impl From<Vec<Question>> for SetItemsResponse {
    fn from(items: Vec<Question>) -> Self {
        Self { items: items.into_iter().map(Into::into).collect() }
    }
}
