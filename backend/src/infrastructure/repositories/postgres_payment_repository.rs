use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::payment::{PaymentStatus, PaymentTransaction};
use crate::domain::repository::{NewPaymentTransaction, PaymentStatusUpdate, PaymentTransactionRepository};
use crate::error::{AppError, AppResult};

pub struct PostgresPaymentTransactionRepository {
    pool: PgPool,
}

impl PostgresPaymentTransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PaymentTransactionRow {
    id: Uuid,
    package_id: Uuid,
    buyer_user_id: Uuid,
    amount: i32,
    currency: String,
    provider: Option<String>,
    provider_ref: Option<String>,
    status: String,
    failure_reason: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    paid_at: Option<DateTime<Utc>>,
}

impl TryFrom<PaymentTransactionRow> for PaymentTransaction {
    type Error = AppError;
    fn try_from(r: PaymentTransactionRow) -> Result<Self, Self::Error> {
        Ok(PaymentTransaction {
            id: r.id,
            package_id: r.package_id,
            buyer_user_id: r.buyer_user_id,
            amount: r.amount,
            currency: r.currency,
            provider: r.provider,
            provider_ref: r.provider_ref,
            status: r
                .status
                .parse::<PaymentStatus>()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt payment status: {e}")))?,
            failure_reason: r.failure_reason,
            created_at: r.created_at,
            updated_at: r.updated_at,
            paid_at: r.paid_at,
        })
    }
}

const SELECT_TX: &str = "SELECT id, package_id, buyer_user_id, amount, currency, provider, provider_ref, status, \
     failure_reason, created_at, updated_at, paid_at FROM payment_transactions";

#[async_trait]
impl PaymentTransactionRepository for PostgresPaymentTransactionRepository {
    async fn create(&self, input: NewPaymentTransaction) -> AppResult<PaymentTransaction> {
        let row = sqlx::query_as::<_, PaymentTransactionRow>(&format!(
            r#"INSERT INTO payment_transactions (package_id, buyer_user_id, amount, currency)
               VALUES ($1,$2,$3,$4)
               RETURNING id, package_id, buyer_user_id, amount, currency, provider, provider_ref, status,
                         failure_reason, created_at, updated_at, paid_at"#
        ))
        .bind(input.package_id)
        .bind(input.buyer_user_id)
        .bind(input.amount)
        .bind(&input.currency)
        .fetch_one(&self.pool)
        .await?;
        PaymentTransaction::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<PaymentTransaction>> {
        let row = sqlx::query_as::<_, PaymentTransactionRow>(&format!("{SELECT_TX} WHERE id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(PaymentTransaction::try_from).transpose()
    }

    async fn find_by_provider_ref(&self, provider_ref: &str) -> AppResult<Option<PaymentTransaction>> {
        let row = sqlx::query_as::<_, PaymentTransactionRow>(&format!("{SELECT_TX} WHERE provider_ref = $1"))
            .bind(provider_ref)
            .fetch_optional(&self.pool)
            .await?;
        row.map(PaymentTransaction::try_from).transpose()
    }

    async fn list_by_buyer(&self, buyer_user_id: Uuid) -> AppResult<Vec<PaymentTransaction>> {
        let rows = sqlx::query_as::<_, PaymentTransactionRow>(&format!(
            "{SELECT_TX} WHERE buyer_user_id = $1 ORDER BY created_at DESC"
        ))
        .bind(buyer_user_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(PaymentTransaction::try_from).collect()
    }

    async fn list_by_status(&self, status: PaymentStatus) -> AppResult<Vec<PaymentTransaction>> {
        let rows = sqlx::query_as::<_, PaymentTransactionRow>(&format!(
            "{SELECT_TX} WHERE status = $1 ORDER BY created_at DESC"
        ))
        .bind(status.as_str())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(PaymentTransaction::try_from).collect()
    }

    async fn update_status(&self, id: Uuid, update: PaymentStatusUpdate) -> AppResult<PaymentTransaction> {
        let paid_at_expr = if update.status == PaymentStatus::Paid { "now()" } else { "paid_at" };
        let row = sqlx::query_as::<_, PaymentTransactionRow>(&format!(
            r#"UPDATE payment_transactions SET
                 status = $2, provider = COALESCE($3, provider), provider_ref = COALESCE($4, provider_ref),
                 failure_reason = $5, updated_at = now(), paid_at = {paid_at_expr}
               WHERE id = $1
               RETURNING id, package_id, buyer_user_id, amount, currency, provider, provider_ref, status,
                         failure_reason, created_at, updated_at, paid_at"#
        ))
        .bind(id)
        .bind(update.status.as_str())
        .bind(&update.provider)
        .bind(&update.provider_ref)
        .bind(&update.failure_reason)
        .fetch_one(&self.pool)
        .await?;
        PaymentTransaction::try_from(row)
    }
}
