use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::event::{Event, EventStatus, EventType};
use crate::domain::repository::{EventRepository, EventUpdate, NewEvent};
use crate::domain::tenant::TenantCtx;
use crate::error::{AppError, AppResult};

pub struct PostgresEventRepository {
    pool: PgPool,
}

impl PostgresEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn begin_tx(&self, ctx: &TenantCtx) -> AppResult<Transaction<'static, Postgres>> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.tenant_id', $1, true), set_config('app.is_platform_admin', $2, true)")
            .bind(ctx.tenant_id.to_string())
            .bind(if ctx.is_platform_admin { "true" } else { "false" })
            .execute(&mut *tx)
            .await?;
        Ok(tx)
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    public_id: Uuid,
    name: String,
    event_type: String,
    description: String,
    session_id: Uuid,
    session_title: String,
    duration_minutes: i32,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    max_participants: Option<i32>,
    participants: i64,
    price: i32,
    status: String,
    prizes: String,
    target_class: String,
    // `Option` because the scalar subquery can return NULL when `created_by` points at a
    // `core.akun` row the current RLS context can't see — see
    // `domain::event::Event::created_by` doc comment.
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
}

impl TryFrom<EventRow> for Event {
    type Error = AppError;
    fn try_from(r: EventRow) -> Result<Self, Self::Error> {
        Ok(Event {
            id: r.public_id,
            name: r.name,
            event_type: r
                .event_type
                .parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt event_type: {e}")))?,
            description: r.description,
            session_id: r.session_id,
            session_title: r.session_title,
            duration_minutes: r.duration_minutes,
            start_date: r.start_date,
            end_date: r.end_date,
            max_participants: r.max_participants,
            participants: r.participants,
            price: r.price,
            status: r
                .status
                .parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt event status: {e}")))?,
            prizes: r.prizes,
            target_class: r.target_class,
            created_by: r.created_by,
            created_at: r.created_at,
        })
    }
}

fn event_type_str(t: EventType) -> &'static str {
    t.as_str()
}

fn status_str(s: EventStatus) -> &'static str {
    s.as_str()
}

// `participants` is always derived live from real `attempts` rows for the event's
// session — joined here, never stored on the `events` row itself.
const SELECT_EVENT: &str = r#"
    SELECT e.public_id, e.name, e.event_type, e.description, s.public_id AS session_id, s.title AS session_title,
           s.duration_minutes, e.start_date, e.end_date, e.max_participants,
           COALESCE(p.participants, 0) AS participants,
           e.price, e.status, e.prizes, e.target_class,
           (SELECT public_id FROM core.akun WHERE core.akun.id = e.created_by) AS created_by,
           e.created_at
    FROM ptn.events e
    JOIN ptn.tryout_sessions s ON s.id = e.session_id
    LEFT JOIN (
        SELECT session_id, COUNT(DISTINCT student_id) AS participants
        FROM ptn.attempts
        GROUP BY session_id
    ) p ON p.session_id = e.session_id
"#;

#[async_trait]
impl EventRepository for PostgresEventRepository {
    async fn create(&self, n: NewEvent, ctx: &TenantCtx) -> AppResult<Event> {
        let mut tx = self.begin_tx(ctx).await?;

        let session_id: i64 = sqlx::query_scalar("SELECT id FROM ptn.tryout_sessions WHERE public_id = $1")
            .bind(n.session_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::Validation("paket soal (session) yang dipilih tidak ditemukan".to_string()))?;
        let created_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(n.created_by)
            .fetch_one(&mut *tx)
            .await?;

        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.events
                (tenant_id, name, event_type, description, session_id, start_date, end_date,
                 max_participants, price, prizes, target_class, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
               RETURNING public_id"#,
        )
        .bind(ctx.tenant_id)
        .bind(&n.name)
        .bind(event_type_str(n.event_type))
        .bind(&n.description)
        .bind(session_id)
        .bind(n.start_date)
        .bind(n.end_date)
        .bind(n.max_participants)
        .bind(n.price)
        .bind(&n.prizes)
        .bind(&n.target_class)
        .bind(created_by_id)
        .fetch_one(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, EventRow>(&format!("{SELECT_EVENT} WHERE e.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Event::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<Event>> {
        let mut tx = self.begin_tx(ctx).await?;
        let row = sqlx::query_as::<_, EventRow>(&format!("{SELECT_EVENT} WHERE e.public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(Event::try_from).transpose()
    }

    async fn list(&self, ctx: &TenantCtx) -> AppResult<Vec<Event>> {
        let mut tx = self.begin_tx(ctx).await?;
        let rows = sqlx::query_as::<_, EventRow>(&format!("{SELECT_EVENT} ORDER BY e.start_date DESC, e.created_at DESC"))
            .fetch_all(&mut *tx)
            .await?;
        tx.commit().await?;
        rows.into_iter().map(Event::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: EventUpdate, ctx: &TenantCtx) -> AppResult<Option<Event>> {
        let mut tx = self.begin_tx(ctx).await?;

        let session_id: i64 = match sqlx::query_scalar("SELECT id FROM ptn.tryout_sessions WHERE public_id = $1")
            .bind(u.session_id)
            .fetch_optional(&mut *tx)
            .await?
        {
            Some(id) => id,
            None => {
                tx.rollback().await.ok();
                return Err(AppError::Validation("paket soal (session) yang dipilih tidak ditemukan".to_string()));
            }
        };

        let result = sqlx::query(
            r#"UPDATE ptn.events SET
                name = $2, event_type = $3, description = $4, session_id = $5, start_date = $6,
                end_date = $7, max_participants = $8, price = $9, status = $10, prizes = $11,
                target_class = $12
               WHERE public_id = $1"#,
        )
        .bind(id)
        .bind(&u.name)
        .bind(event_type_str(u.event_type))
        .bind(&u.description)
        .bind(session_id)
        .bind(u.start_date)
        .bind(u.end_date)
        .bind(u.max_participants)
        .bind(u.price)
        .bind(status_str(u.status))
        .bind(&u.prizes)
        .bind(&u.target_class)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, EventRow>(&format!("{SELECT_EVENT} WHERE e.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Event::try_from(row).map(Some)
    }

    async fn set_status(&self, id: Uuid, status: EventStatus, ctx: &TenantCtx) -> AppResult<Option<Event>> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("UPDATE ptn.events SET status = $2 WHERE public_id = $1")
            .bind(id)
            .bind(status_str(status))
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, EventRow>(&format!("{SELECT_EVENT} WHERE e.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Event::try_from(row).map(Some)
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = self.begin_tx(ctx).await?;
        let result = sqlx::query("DELETE FROM ptn.events WHERE public_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }
}
