//! Real per-user notification feed — see `domain::notification` doc comment. Two kinds of
//! callers: other application services push notifications via `notify()` (no `AuthUser`
//! needed — the recipient is an internal parameter, not attacker-controlled), while
//! `list_mine`/`unread_count`/`mark_read`/`mark_all_read` are self-service, scoped to the
//! calling `AuthUser` so nobody can read or mark another user's notifications.

use std::sync::Arc;

use uuid::Uuid;

use crate::domain::notification::Notification;
use crate::domain::repository::{NewNotification, NotificationRepository};
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct NotificationService {
    notifications: Arc<dyn NotificationRepository>,
}

impl NotificationService {
    pub fn new(notifications: Arc<dyn NotificationRepository>) -> Self {
        Self { notifications }
    }

    /// Called by other application services (e.g. `PaymentService::fulfill`) to push a
    /// notification to `user_id`. Deliberately fire-and-forget-tolerant at the call site:
    /// a notification failing to write should never fail the primary action it's attached
    /// to, so callers typically log-and-ignore the error rather than propagate it.
    pub async fn notify(
        &self,
        user_id: Uuid,
        kind: impl Into<String>,
        title: impl Into<String>,
        message: impl Into<String>,
        link_tab: Option<String>,
    ) -> AppResult<Notification> {
        self.notifications
            .create(NewNotification { user_id, kind: kind.into(), title: title.into(), message: message.into(), link_tab })
            .await
    }

    pub async fn list_mine(&self, actor: &AuthUser, limit: i64) -> AppResult<Vec<Notification>> {
        self.notifications.list_for_user(actor.user_id, limit.clamp(1, 100)).await
    }

    pub async fn unread_count(&self, actor: &AuthUser) -> AppResult<i64> {
        self.notifications.unread_count(actor.user_id).await
    }

    pub async fn mark_read(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        let ok = self.notifications.mark_read(id, actor.user_id).await?;
        if !ok {
            return Err(AppError::NotFound(format!("notifikasi {id} tidak ditemukan")));
        }
        Ok(())
    }

    pub async fn mark_all_read(&self, actor: &AuthUser) -> AppResult<()> {
        self.notifications.mark_all_read(actor.user_id).await
    }
}
