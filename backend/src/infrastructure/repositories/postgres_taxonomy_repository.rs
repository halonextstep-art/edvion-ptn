use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::repository::TaxonomyRepository;
use crate::domain::taxonomy::{CategoryWithSubjects, Subject, SubjectCategory};
use crate::error::AppResult;

pub struct PostgresTaxonomyRepository {
    pool: PgPool,
}

impl PostgresTaxonomyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SubjectCategoryRow {
    public_id: Uuid,
    name: String,
    description: String,
    sort_order: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SubjectCategoryRow> for SubjectCategory {
    fn from(r: SubjectCategoryRow) -> Self {
        SubjectCategory {
            id: r.public_id,
            name: r.name,
            description: r.description,
            sort_order: r.sort_order,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SubjectRow {
    public_id: Uuid,
    category_id: Uuid,
    name: String,
    code: String,
    sort_order: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SubjectRow> for Subject {
    fn from(r: SubjectRow) -> Self {
        Subject {
            id: r.public_id,
            category_id: r.category_id,
            name: r.name,
            code: r.code,
            sort_order: r.sort_order,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

// `ptn.subject_categories`/`ptn.subjects` carry no `tenant_id`/RLS — a shared platform-wide
// catalogue (same as `ptn.institutions`/`ptn.ptn_programs`), so plain pool queries.
const SELECT_CATEGORY: &str = r#"
    SELECT public_id, name, description, sort_order, created_at, updated_at
    FROM ptn.subject_categories
"#;

const SELECT_SUBJECT: &str = r#"
    SELECT s.public_id, c.public_id AS category_id, s.name, s.code, s.sort_order, s.created_at, s.updated_at
    FROM ptn.subjects s
    JOIN ptn.subject_categories c ON c.id = s.category_id
"#;

#[async_trait]
impl TaxonomyRepository for PostgresTaxonomyRepository {
    async fn list_categories_with_subjects(&self) -> AppResult<Vec<CategoryWithSubjects>> {
        let category_rows = sqlx::query_as::<_, SubjectCategoryRow>(&format!("{SELECT_CATEGORY} ORDER BY sort_order ASC, created_at ASC"))
            .fetch_all(&self.pool)
            .await?;
        let subject_rows = sqlx::query_as::<_, SubjectRow>(&format!("{SELECT_SUBJECT} ORDER BY s.category_id ASC, s.sort_order ASC, s.created_at ASC"))
            .fetch_all(&self.pool)
            .await?;

        // Assembled in Rust rather than a JOIN — this is a small catalogue, not a hot
        // path, so simplicity wins over a single round-trip.
        let mut subjects_by_category: HashMap<Uuid, Vec<Subject>> = HashMap::new();
        for row in subject_rows {
            let subject = Subject::from(row);
            subjects_by_category.entry(subject.category_id).or_default().push(subject);
        }

        Ok(category_rows
            .into_iter()
            .map(|row| {
                let category = SubjectCategory::from(row);
                let subjects = subjects_by_category.remove(&category.id).unwrap_or_default();
                CategoryWithSubjects { category, subjects }
            })
            .collect())
    }

    async fn create_category(&self, name: String, description: String, sort_order: i32) -> AppResult<SubjectCategory> {
        let public_id: Uuid = sqlx::query_scalar(
            "INSERT INTO ptn.subject_categories (name, description, sort_order) VALUES ($1,$2,$3) RETURNING public_id",
        )
        .bind(&name)
        .bind(&description)
        .bind(sort_order)
        .fetch_one(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, SubjectCategoryRow>(&format!("{SELECT_CATEGORY} WHERE public_id = $1"))
            .bind(public_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.into())
    }

    async fn update_category(
        &self,
        id: Uuid,
        name: String,
        description: String,
        sort_order: i32,
    ) -> AppResult<Option<SubjectCategory>> {
        let result = sqlx::query(
            "UPDATE ptn.subject_categories SET name = $2, description = $3, sort_order = $4, updated_at = now() WHERE public_id = $1",
        )
        .bind(id)
        .bind(&name)
        .bind(&description)
        .bind(sort_order)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }
        let row = sqlx::query_as::<_, SubjectCategoryRow>(&format!("{SELECT_CATEGORY} WHERE public_id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(SubjectCategory::from))
    }

    async fn delete_category(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM ptn.subject_categories WHERE public_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn create_subject(&self, category_id: Uuid, name: String, code: String, sort_order: i32) -> AppResult<Subject> {
        let category_pk: i64 = sqlx::query_scalar("SELECT id FROM ptn.subject_categories WHERE public_id = $1")
            .bind(category_id)
            .fetch_one(&self.pool)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            "INSERT INTO ptn.subjects (category_id, name, code, sort_order) VALUES ($1,$2,$3,$4) RETURNING public_id",
        )
        .bind(category_pk)
        .bind(&name)
        .bind(&code)
        .bind(sort_order)
        .fetch_one(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, SubjectRow>(&format!("{SELECT_SUBJECT} WHERE s.public_id = $1"))
            .bind(public_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.into())
    }

    async fn update_subject(&self, id: Uuid, name: String, code: String, sort_order: i32) -> AppResult<Option<Subject>> {
        let result = sqlx::query("UPDATE ptn.subjects SET name = $2, code = $3, sort_order = $4, updated_at = now() WHERE public_id = $1")
            .bind(id)
            .bind(&name)
            .bind(&code)
            .bind(sort_order)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }
        let row = sqlx::query_as::<_, SubjectRow>(&format!("{SELECT_SUBJECT} WHERE s.public_id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Subject::from))
    }

    async fn delete_subject(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM ptn.subjects WHERE public_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
