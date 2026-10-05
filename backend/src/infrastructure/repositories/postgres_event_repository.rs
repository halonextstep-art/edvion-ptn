use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::event::{Event, EventStatus, EventType};
use crate::domain::repository::{EventRepository, EventUpdate, NewEvent};
use crate::error::{AppError, AppResult};

pub struct PostgresEventRepository {
    pool: PgPool,
}

impl PostgresEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    id: Uuid,
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
    created_by: Uuid,
    created_at: DateTime<Utc>,
}

impl TryFrom<EventRow> for Event {
    type Error = AppError;
    fn try_from(r: EventRow) -> Result<Self, Self::Error> {
        Ok(Event {
            id: r.id,
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
    SELECT e.id, e.name, e.event_type, e.description, e.session_id, s.title AS session_title,
           s.duration_minutes, e.start_date, e.end_date, e.max_participants,
           COALESCE(p.participants, 0) AS participants,
           e.price, e.status, e.prizes, e.target_class, e.created_by, e.created_at
    FROM events e
    JOIN tryout_sessions s ON s.id = e.session_id
    LEFT JOIN (
        SELECT session_id, COUNT(DISTINCT student_id) AS participants
        FROM attempts
        GROUP BY session_id
    ) p ON p.session_id = e.session_id
"#;

#[async_trait]
impl EventRepository for PostgresEventRepository {
    async fn create(&self, n: NewEvent) -> AppResult<Event> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO events
                (id, name, event_type, description, session_id, start_date, end_date,
                 max_participants, price, prizes, target_class, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"#,
        )
        .bind(id)
        .bind(&n.name)
        .bind(event_type_str(n.event_type))
        .bind(&n.description)
        .bind(n.session_id)
        .bind(n.start_date)
        .bind(n.end_date)
        .bind(n.max_participants)
        .bind(n.price)
        .bind(&n.prizes)
        .bind(&n.target_class)
        .bind(n.created_by)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.code().as_deref() == Some("23503") {
                    return AppError::Validation("paket soal (session) yang dipilih tidak ditemukan".to_string());
                }
            }
            AppError::Database(e)
        })?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("event vanished right after insert")))
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Event>> {
        let row = sqlx::query_as::<_, EventRow>(&format!("{SELECT_EVENT} WHERE e.id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(Event::try_from).transpose()
    }

    async fn list(&self) -> AppResult<Vec<Event>> {
        let rows = sqlx::query_as::<_, EventRow>(&format!("{SELECT_EVENT} ORDER BY e.start_date DESC, e.created_at DESC"))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Event::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: EventUpdate) -> AppResult<Option<Event>> {
        let result = sqlx::query(
            r#"UPDATE events SET
                name = $2, event_type = $3, description = $4, session_id = $5, start_date = $6,
                end_date = $7, max_participants = $8, price = $9, status = $10, prizes = $11,
                target_class = $12
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(&u.name)
        .bind(event_type_str(u.event_type))
        .bind(&u.description)
        .bind(u.session_id)
        .bind(u.start_date)
        .bind(u.end_date)
        .bind(u.max_participants)
        .bind(u.price)
        .bind(status_str(u.status))
        .bind(&u.prizes)
        .bind(&u.target_class)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.code().as_deref() == Some("23503") {
                    return AppError::Validation("paket soal (session) yang dipilih tidak ditemukan".to_string());
                }
            }
            AppError::Database(e)
        })?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn set_status(&self, id: Uuid, status: EventStatus) -> AppResult<Option<Event>> {
        let result = sqlx::query("UPDATE events SET status = $2 WHERE id = $1")
            .bind(id)
            .bind(status_str(status))
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM events WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
