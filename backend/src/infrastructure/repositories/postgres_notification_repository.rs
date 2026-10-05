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
    id: Uuid,
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
            id: r.id,
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

const SELECT_NOTIFICATION: &str = "SELECT id, user_id, kind, title, message, link_tab, read_at, created_at FROM notifications";

#[async_trait]
impl NotificationRepository for PostgresNotificationRepository {
    async fn create(&self, input: NewNotification) -> AppResult<Notification> {
        let id = Uuid::new_v4();
        let row = sqlx::query_as::<_, NotificationRow>(
            r#"INSERT INTO notifications (id, user_id, kind, title, message, link_tab)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, user_id, kind, title, message, link_tab, read_at, created_at"#,
        )
        .bind(id)
        .bind(input.user_id)
        .bind(&input.kind)
        .bind(&input.title)
        .bind(&input.message)
        .bind(&input.link_tab)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn list_for_user(&self, user_id: Uuid, limit: i64) -> AppResult<Vec<Notification>> {
        let rows = sqlx::query_as::<_, NotificationRow>(&format!(
            "{SELECT_NOTIFICATION} WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2"
        ))
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn unread_count(&self, user_id: Uuid) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND read_at IS NULL",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn mark_read(&self, id: Uuid, user_id: Uuid) -> AppResult<bool> {
        // Idempotent: matches on id+user_id regardless of prior read state (COALESCE keeps
        // the original read_at if already set) so calling this twice — e.g. a double-click
        // or a retried request — isn't treated as an error.
        let result = sqlx::query(
            "UPDATE notifications SET read_at = COALESCE(read_at, now()) WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn mark_all_read(&self, user_id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE notifications SET read_at = now() WHERE user_id = $1 AND read_at IS NULL")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
