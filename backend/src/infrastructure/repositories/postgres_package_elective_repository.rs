//! Postgres impl for `package_elective_choices` — see `20250101000033_package_electives.sql`
//! and `domain::repository::PackageElectiveRepository` doc comments. Always reflects the
//! student's CURRENT pick set for a package (full replace on every `set_choices`, no history).

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::repository::PackageElectiveRepository;
use crate::error::AppResult;

pub struct PostgresPackageElectiveRepository {
    pool: PgPool,
}

impl PostgresPackageElectiveRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PackageElectiveRepository for PostgresPackageElectiveRepository {
    async fn list_choices(&self, student_id: Uuid, package_id: Uuid) -> AppResult<Vec<Uuid>> {
        let ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT session_id FROM package_elective_choices WHERE student_id = $1 AND package_id = $2",
        )
        .bind(student_id)
        .bind(package_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(ids)
    }

    async fn set_choices(&self, student_id: Uuid, package_id: Uuid, session_ids: Vec<Uuid>) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM package_elective_choices WHERE student_id = $1 AND package_id = $2")
            .bind(student_id)
            .bind(package_id)
            .execute(&mut *tx)
            .await?;
        for session_id in session_ids {
            sqlx::query(
                r#"INSERT INTO package_elective_choices (id, student_id, package_id, session_id)
                   VALUES ($1, $2, $3, $4)
                   ON CONFLICT (student_id, package_id, session_id) DO NOTHING"#,
            )
            .bind(Uuid::new_v4())
            .bind(student_id)
            .bind(package_id)
            .bind(session_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
