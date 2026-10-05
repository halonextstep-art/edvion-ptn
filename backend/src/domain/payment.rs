//! Payment gateway scaffolding — a provider-agnostic transaction ledger and the
//! `PaymentProvider` port that a real gateway (Midtrans/Xendit/etc.) would implement.
//!
//! This codebase has no payment gateway account or API key configured (confirmed with
//! the product owner: "belum punya akun/key sama sekali"), so `application::payment_service`
//! only ever wires in `NotConfiguredProvider` — every checkout attempt honestly fails with
//! a clear "not configured yet" error instead of faking a successful payment. The schema,
//! domain types, and service/handler plumbing are real and ready so that swapping in a
//! real provider later is a matter of implementing `PaymentProvider` and wiring it into
//! `main.rs`, not redesigning anything.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Paid,
    Failed,
    Cancelled,
    Expired,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Pending => "pending",
            PaymentStatus::Paid => "paid",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Cancelled => "cancelled",
            PaymentStatus::Expired => "expired",
        }
    }
}

impl std::str::FromStr for PaymentStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(PaymentStatus::Pending),
            "paid" => Ok(PaymentStatus::Paid),
            "failed" => Ok(PaymentStatus::Failed),
            "cancelled" => Ok(PaymentStatus::Cancelled),
            "expired" => Ok(PaymentStatus::Expired),
            other => Err(format!("unknown payment status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTransaction {
    pub id: Uuid,
    pub package_id: Uuid,
    pub buyer_user_id: Uuid,
    pub amount: i32,
    pub currency: String,
    pub provider: Option<String>,
    pub provider_ref: Option<String>,
    pub status: PaymentStatus,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

/// What a real provider integration would hand back after creating a checkout session —
/// e.g. a redirect URL the buyer completes payment at. `NotConfiguredProvider` never
/// produces this; it always errors instead.
#[derive(Debug, Clone, Serialize)]
pub struct CheckoutSession {
    pub transaction_id: Uuid,
    pub redirect_url: String,
    pub provider: String,
}

/// A verified event coming from a provider's webhook — "this provider_ref is now in this
/// state". `NotConfiguredProvider` has no webhook secret to verify against, so it can
/// never produce one of these either.
#[derive(Debug, Clone)]
pub struct WebhookOutcome {
    pub provider_ref: String,
    pub status: PaymentStatus,
    pub failure_reason: Option<String>,
}

/// Port that a real payment gateway integration implements. Kept intentionally small —
/// just "start a checkout" and "verify+parse a webhook call" — so a future Midtrans or
/// Xendit adapter is the only thing that needs writing; nothing else in the app changes.
#[async_trait::async_trait]
pub trait PaymentProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn create_checkout(&self, transaction: &PaymentTransaction) -> AppResult<CheckoutSession>;

    /// `raw_body` is the untouched webhook request body (providers sign over the exact
    /// bytes); `signature_header` is whatever header that provider uses to carry its
    /// signature (name varies per provider, hence `Option<&str>` rather than assuming one).
    async fn verify_webhook(&self, raw_body: &[u8], signature_header: Option<&str>) -> AppResult<WebhookOutcome>;
}

/// The only `PaymentProvider` wired into this app today. Every method honestly fails —
/// no fabricated checkout URL, no fabricated "payment verified" outcome — because there is
/// genuinely no gateway account/API key to talk to yet.
pub struct NotConfiguredProvider;

#[async_trait::async_trait]
impl PaymentProvider for NotConfiguredProvider {
    fn name(&self) -> &'static str {
        "not_configured"
    }

    async fn create_checkout(&self, _transaction: &PaymentTransaction) -> AppResult<CheckoutSession> {
        Err(crate::error::AppError::NotImplemented(
            "Payment gateway belum dikonfigurasi — belum ada akun/API key Midtrans/Xendit yang terhubung. \
             Transaksi tersimpan sebagai draf (status pending), tapi checkout online belum bisa diproses."
                .to_string(),
        ))
    }

    async fn verify_webhook(&self, _raw_body: &[u8], _signature_header: Option<&str>) -> AppResult<WebhookOutcome> {
        Err(crate::error::AppError::NotImplemented(
            "Payment gateway belum dikonfigurasi — tidak ada webhook secret untuk diverifikasi.".to_string(),
        ))
    }
}
