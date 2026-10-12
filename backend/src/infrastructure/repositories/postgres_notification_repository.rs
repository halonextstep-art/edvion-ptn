use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::notification::Notification;
use crate::domain::repository::{NewNotification, NotificationRepository};
use crate::error::AppResult;

pub struct PostgresNotificationRepository {
    pool: PgPool,
}

impl PostgresNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct NotificationRow {
    public_id: Uuid,
    user_id: Uuid,
    kind: String,
    title: String,
    message: String,
    link_tab: Option<String>,
    read_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<NotificationRow> for Notification {
    fn from(r: NotificationRow) -> Self {
        Notification {
            id: r.public_id,
            user_id: r.user_id,
            kind: r.kind,
            title: r.title,
            message: r.message,
            link_tab: r.link_tab,
            read_at: r.read_at,
            created_at: r.created_at,
        }
    }
}

// `ptn.notifications` itself carries no `tenant_id`/RLS — scoped purely by `user_id`
// (bigint FK to `core.akun`) — but every method here JOINs to or looks up `core.akun`,
// which DOES carry RLS (with an `OR tenant_id = 0` carve-out for platform accounts).
// Querying by `user_id`/`public_id` directly is already a precise-enough scope (no
// tenant-wide listing happens here), so each method opens its own transaction with
// `app.is_platform_admin = true` to bypass that RLS check safely, rather than relying on
// ambient GUC state from whatever connection the pool hands back — a transaction-local
// `set_config` resets to an EMPTY STRING (not "unset") on commit, and an empty string fed
// into another table's `current_setting(...)::bigint` RLS check is a hard Postgres error.
const SELECT_NOTIFICATION: &str = r#"
    SELECT n.public_id, a.public_id AS user_id, n.kind, n.title, n.message, n.link_tab, n.read_at, n.created_at
    FROM ptn.notifications n
    JOIN core.akun a ON a.id = n.user_id
"#;

async fn set_platform_admin(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> AppResult<()> {
    sqlx::query("SELECT set_config('app.tenant_id', '0', true), set_config('app.is_platform_admin', 'true', true)")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

#[async_trait]
impl NotificationRepository for PostgresNotificationRepository {
    async fn create(&self, input: NewNotification) -> AppResult<Notification> {
        let mut tx = self.pool.begin().await?;
        set_platform_admin(&mut tx).await?;
        let user_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(input.user_id)
            .fetch_one(&mut *tx)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.notifications (user_id, kind, title, message, link_tab)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING public_id"#,
        )
        .bind(user_id)
        .bind(&input.kind)
        .bind(&input.title)
        .bind(&input.message)
        .bind(&input.link_tab)
        .fetch_one(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, NotificationRow>(&format!("{SELECT_NOTIFICATION} WHERE n.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(row.into())
    }

    async fn list_for_user(&self, user_id: Uuid, limit: i64) -> AppResult<Vec<Notification>> {
        let mut tx = self.pool.begin().await?;
        set_platform_admin(&mut tx).await?;
        let rows = sqlx::query_as::<_, NotificationRow>(&format!(
            "{SELECT_NOTIFICATION} WHERE a.public_id = $1 ORDER BY n.created_at DESC LIMIT $2"
        ))
        .bind(user_id)
        .bind(limit)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn unread_count(&self, user_id: Uuid) -> AppResult<i64> {
        let mut tx = self.pool.begin().await?;
        set_platform_admin(&mut tx).await?;
        let count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM ptn.notifications n
               JOIN core.akun a ON a.id = n.user_id
               WHERE a.public_id = $1 AND n.read_at IS NULL"#,
        )
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(count)
    }

    async fn mark_read(&self, id: Uuid, user_id: Uuid) -> AppResult<bool> {
        // Idempotent: matches on id+user_id regardless of prior read state (COALESCE keeps
        // the original read_at if already set) so calling this twice — e.g. a double-click
        // or a retried request — isn't treated as an error.
        let mut tx = self.pool.begin().await?;
        set_platform_admin(&mut tx).await?;
        let result = sqlx::query(
            r#"UPDATE ptn.notifications SET read_at = COALESCE(read_at, now())
               WHERE public_id = $1 AND user_id = (SELECT id FROM core.akun WHERE public_id = $2)"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn mark_all_read(&self, user_id: Uuid) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        set_platform_admin(&mut tx).await?;
        sqlx::query(
            r#"UPDATE ptn.notifications SET read_at = now()
               WHERE user_id = (SELECT id FROM core.akun WHERE public_id = $1) AND read_at IS NULL"#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
