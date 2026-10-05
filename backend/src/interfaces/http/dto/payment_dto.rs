use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::payment::{CheckoutSession, PaymentTransaction};

#[derive(Debug, Deserialize, Validate)]
pub struct InitiatePurchasePayload {
    pub package_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct PaymentTransactionResponse {
    pub id: Uuid,
    pub package_id: Uuid,
    pub buyer_user_id: Uuid,
    pub amount: i32,
    pub currency: String,
    pub provider: Option<String>,
    pub provider_ref: Option<String>,
    pub status: String,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}
impl From<PaymentTransaction> for PaymentTransactionResponse {
    fn from(t: PaymentTransaction) -> Self {
        Self {
            id: t.id,
            package_id: t.package_id,
            buyer_user_id: t.buyer_user_id,
            amount: t.amount,
            currency: t.currency,
            provider: t.provider,
            provider_ref: t.provider_ref,
            status: t.status.as_str().to_string(),
            failure_reason: t.failure_reason,
            created_at: t.created_at,
            updated_at: t.updated_at,
            paid_at: t.paid_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CheckoutSessionResponse {
    pub transaction_id: Uuid,
    pub redirect_url: String,
    pub provider: String,
}
impl From<CheckoutSession> for CheckoutSessionResponse {
    fn from(c: CheckoutSession) -> Self {
        Self { transaction_id: c.transaction_id, redirect_url: c.redirect_url, provider: c.provider }
    }
}

/// Both shapes `initiate_purchase` can return. `mode` tells the frontend which variant it
/// got: `"checkout"` means a real provider opened an online checkout; `"manual_pending"`
/// means no provider is configured and the transaction now waits in the admin's manual
/// purchase-request queue — an expected, honest state, not an error.
#[derive(Debug, Serialize)]
#[serde(tag = "mode")]
pub enum InitiatePurchaseResponse {
    #[serde(rename = "checkout")]
    Checkout { transaction: PaymentTransactionResponse, checkout: CheckoutSessionResponse },
    #[serde(rename = "manual_pending")]
    ManualPending { transaction: PaymentTransactionResponse },
}

/// Surfaced at `GET /api/payments/provider` so the frontend can honestly show "pembayaran
/// online belum tersedia" instead of only discovering it after a failed checkout attempt.
#[derive(Debug, Serialize)]
pub struct PaymentProviderStatusResponse {
    pub provider: String,
    pub configured: bool,
}

/// Admin manual purchase-request queue row — the raw transaction enriched with buyer/package
/// names (fetched at the handler level via `UserService`/`PackageService`, not inside
/// `PaymentService`, to keep it decoupled from other domains) so the admin doesn't have to
/// cross-reference IDs by hand.
#[derive(Debug, Serialize)]
pub struct PendingPurchaseResponse {
    pub transaction: PaymentTransactionResponse,
    pub buyer_name: String,
    pub buyer_email: String,
    pub package_name: String,
}

/// Response for `POST /api/payments/:id/fulfill` — the now-`Paid` transaction plus the real
/// Access voucher code that was generated and auto-redeemed for the buyer.
#[derive(Debug, Serialize)]
pub struct FulfillResponse {
    pub transaction: PaymentTransactionResponse,
    pub voucher_code: String,
}
