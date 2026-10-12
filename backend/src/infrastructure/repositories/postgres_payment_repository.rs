use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::payment::{PaymentStatus, PaymentTransaction};
use crate::domain::repository::{NewPaymentTransaction, PaymentStatusUpdate, PaymentTransactionRepository};
use crate::domain::tenant::TenantCtx;
use crate::error::{AppError, AppResult};

pub struct PostgresPaymentTransactionRepository {
    pool: PgPool,
}

impl PostgresPaymentTransactionRepository {
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
struct PaymentTransactionRow {
    public_id: Uuid,
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
            id: r.public_id,
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

const SELECT_TX: &str = r#"
    SELECT t.public_id, p.public_id AS package_id,
           (SELECT public_id FROM core.akun WHERE core.akun.id = t.buyer_user_id) AS buyer_user_id,
           t.amount, t.currency, t.provider, t.provider_ref, t.status,
           t.failure_reason, t.created_at, t.updated_at, t.paid_at
    FROM ptn.payment_transactions t
    JOIN ptn.packages p ON p.id = t.package_id
"#;

#[async_trait]
impl PaymentTransactionRepository for PostgresPaymentTransactionRepository {
    async fn create(&self, input: NewPaymentTransaction, ctx: &TenantCtx) -> AppResult<PaymentTransaction> {
        let mut tx = self.begin_tx(ctx).await?;

        let package_id: i64 = sqlx::query_scalar("SELECT id FROM ptn.packages WHERE public_id = $1")
            .bind(input.package_id)
            .fetch_one(&mut *tx)
            .await?;
        let buyer_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(input.buyer_user_id)
            .fetch_one(&mut *tx)
            .await?;

        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.payment_transactions (tenant_id, package_id, buyer_user_id, amount, currency)
               VALUES ($1,$2,$3,$4,$5)
               RETURNING public_id"#,
        )
        .bind(ctx.tenant_id)
        .bind(package_id)
        .bind(buyer_id)
        .bind(input.amount)
        .bind(&input.currency)
        .fetch_one(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, PaymentTransactionRow>(&format!("{SELECT_TX} WHERE t.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        PaymentTransaction::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<PaymentTransaction>> {
        let mut tx = self.begin_tx(ctx).await?;
        let row = sqlx::query_as::<_, PaymentTransactionRow>(&format!("{SELECT_TX} WHERE t.public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(PaymentTransaction::try_from).transpose()
    }

    async fn find_by_provider_ref(&self, provider_ref: &str, ctx: &TenantCtx) -> AppResult<Option<PaymentTransaction>> {
        let mut tx = self.begin_tx(ctx).await?;
        let row = sqlx::query_as::<_, PaymentTransactionRow>(&format!("{SELECT_TX} WHERE t.provider_ref = $1"))
            .bind(provider_ref)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(PaymentTransaction::try_from).transpose()
    }

    async fn list_by_buyer(&self, buyer_user_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<PaymentTransaction>> {
        let mut tx = self.begin_tx(ctx).await?;
        let rows = sqlx::query_as::<_, PaymentTransactionRow>(&format!(
            "{SELECT_TX} WHERE t.buyer_user_id = (SELECT id FROM core.akun WHERE public_id = $1) ORDER BY t.created_at DESC"
        ))
        .bind(buyer_user_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows.into_iter().map(PaymentTransaction::try_from).collect()
    }

    async fn list_by_status(&self, status: PaymentStatus, ctx: &TenantCtx) -> AppResult<Vec<PaymentTransaction>> {
        let mut tx = self.begin_tx(ctx).await?;
        let rows = sqlx::query_as::<_, PaymentTransactionRow>(&format!(
            "{SELECT_TX} WHERE t.status = $1 ORDER BY t.created_at DESC"
        ))
        .bind(status.as_str())
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        rows.into_iter().map(PaymentTransaction::try_from).collect()
    }

    async fn update_status(&self, id: Uuid, update: PaymentStatusUpdate, ctx: &TenantCtx) -> AppResult<PaymentTransaction> {
        let mut tx = self.begin_tx(ctx).await?;
        let paid_at_expr = if update.status == PaymentStatus::Paid { "now()" } else { "paid_at" };
        sqlx::query(&format!(
            r#"UPDATE ptn.payment_transactions SET
                 status = $2, provider = COALESCE($3, provider), provider_ref = COALESCE($4, provider_ref),
                 failure_reason = $5, updated_at = now(), paid_at = {paid_at_expr}
               WHERE public_id = $1"#
        ))
        .bind(id)
        .bind(update.status.as_str())
        .bind(&update.provider)
        .bind(&update.provider_ref)
        .bind(&update.failure_reason)
        .execute(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, PaymentTransactionRow>(&format!("{SELECT_TX} WHERE t.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        PaymentTransaction::try_from(row)
    }
}
