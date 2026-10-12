//! Postgres impl for `ptn.package_elective_choices` — see `20250101000033_package_electives.sql`
//! and `domain::repository::PackageElectiveRepository` doc comments. Always reflects the
//! student's CURRENT pick set for a package (full replace on every `set_choices`, no history).
//!
//! `student_id`/`package_id`/`session_id` are all exposed as `Uuid` (`core.akun.public_id` /
//! `ptn.packages.public_id` / `ptn.tryout_sessions.public_id`) but stored as bigint FKs —
//! resolved via scalar subqueries, same pattern as every other repository in this migration.
//! `ptn.package_elective_choices` itself carries no `tenant_id`/RLS (scoped transitively via
//! `student_id`/`package_id`), but every method still opens an RLS-scoped transaction since
//! those lookups touch `core.akun`/`ptn.packages`/`ptn.tryout_sessions`, which DO carry RLS.

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::repository::PackageElectiveRepository;
use crate::domain::tenant::TenantCtx;
use crate::error::AppResult;

pub struct PostgresPackageElectiveRepository {
    pool: PgPool,
}

impl PostgresPackageElectiveRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// See `postgres_user_repository.rs::begin_tx` doc comment.
    async fn begin_tx(&self, ctx: &TenantCtx) -> AppResult<Transaction<'static, Postgres>> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "SELECT set_config('app.tenant_id', $1, true), set_config('app.is_platform_admin', $2, true)",
        )
        .bind(ctx.tenant_id.to_string())
        .bind(if ctx.is_platform_admin { "true" } else { "false" })
        .execute(&mut *tx)
        .await?;
        Ok(tx)
    }
}

#[async_trait]
impl PackageElectiveRepository for PostgresPackageElectiveRepository {
    async fn list_choices(&self, student_id: Uuid, package_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<Uuid>> {
        let mut tx = self.begin_tx(ctx).await?;
        let ids: Vec<Uuid> = sqlx::query_scalar(
            r#"SELECT ts.public_id
               FROM ptn.package_elective_choices pec
               JOIN ptn.tryout_sessions ts ON ts.id = pec.session_id
               WHERE pec.student_id = (SELECT id FROM core.akun WHERE public_id = $1)
                 AND pec.package_id = (SELECT id FROM ptn.packages WHERE public_id = $2)"#,
        )
        .bind(student_id)
        .bind(package_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(ids)
    }

    async fn set_choices(&self, student_id: Uuid, package_id: Uuid, session_ids: Vec<Uuid>, ctx: &TenantCtx) -> AppResult<()> {
        let mut tx = self.begin_tx(ctx).await?;

        let student_pk: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(student_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("student lookup failed: {e}")))?;
        let package_pk: i64 = sqlx::query_scalar("SELECT id FROM ptn.packages WHERE public_id = $1")
            .bind(package_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("package lookup failed: {e}")))?;

        sqlx::query("DELETE FROM ptn.package_elective_choices WHERE student_id = $1 AND package_id = $2")
            .bind(student_pk)
            .bind(package_pk)
            .execute(&mut *tx)
            .await?;

        for session_id in session_ids {
            let session_pk: i64 = sqlx::query_scalar("SELECT id FROM ptn.tryout_sessions WHERE public_id = $1")
                .bind(session_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("session lookup failed: {e}")))?;
            sqlx::query(
                r#"INSERT INTO ptn.package_elective_choices (student_id, package_id, session_id)
                   VALUES ($1, $2, $3)
                   ON CONFLICT (student_id, package_id, session_id) DO NOTHING"#,
            )
            .bind(student_pk)
            .bind(package_pk)
            .bind(session_pk)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
