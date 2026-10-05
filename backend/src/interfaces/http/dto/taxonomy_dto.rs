use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::taxonomy::{CategoryWithSubjects, Subject, SubjectCategory};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCategoryPayload {
    #[validate(length(min = 1, message = "nama kategori wajib diisi"))]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCategoryPayload {
    #[validate(length(min = 1, message = "nama kategori wajib diisi"))]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSubjectPayload {
    pub category_id: Uuid,
    #[validate(length(min = 1, message = "nama mata uji wajib diisi"))]
    pub name: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSubjectPayload {
    #[validate(length(min = 1, message = "nama mata uji wajib diisi"))]
    pub name: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Serialize)]
pub struct SubjectResponse {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub code: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Subject> for SubjectResponse {
    fn from(s: Subject) -> Self {
        Self {
            id: s.id,
            category_id: s.category_id,
            name: s.name,
            code: s.code,
            sort_order: s.sort_order,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SubjectCategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SubjectCategory> for SubjectCategoryResponse {
    fn from(c: SubjectCategory) -> Self {
        Self {
            id: c.id,
            name: c.name,
            description: c.description,
            sort_order: c.sort_order,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CategoryWithSubjectsResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub subjects: Vec<SubjectResponse>,
}

impl From<CategoryWithSubjects> for CategoryWithSubjectsResponse {
    fn from(c: CategoryWithSubjects) -> Self {
        Self {
            id: c.category.id,
            name: c.category.name,
            description: c.category.description,
            sort_order: c.category.sort_order,
            created_at: c.category.created_at,
            updated_at: c.category.updated_at,
            subjects: c.subjects.into_iter().map(Into::into).collect(),
        }
    }
}
