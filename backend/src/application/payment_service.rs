//! PaymentService — provider-agnostic checkout orchestration + manual purchase-request
//! fulfillment.
//!
//! This service is real and fully wired end-to-end, but the only `PaymentProvider`
//! plugged into it today is `NotConfiguredProvider` (see `domain::payment`), because this
//! system has no payment gateway account or API key yet. Every purchase still goes through
//! a real state machine — nothing here fabricates a "paid" status without an explicit admin
//! action:
//!
//! 1. `initiate_purchase` always creates a real, honestly-`pending` transaction row (real
//!    buyer, real package price), then asks the provider to open a checkout.
//! 2. With a real gateway (`PurchaseOutcome::Checkout`), the buyer pays online and a webhook
//!    later confirms it — that path is untouched by this change.
//! 3. With `NotConfiguredProvider` (`PurchaseOutcome::PendingManual`), there is no online
//!    checkout available; this is an expected, valid product state, not an error. The
//!    transaction simply stays `pending` and shows up in the admin's manual purchase-request
//!    queue (`list_pending`).
//! 4. An admin manually verifies payment out-of-band (e.g. checks a bank transfer) and calls
//!    `fulfill`, which grants the package via a real, single-use Access voucher
//!    (`VoucherService::grant_access`) — reusing the same tested redemption mechanism a
//!    student would use with a manually-typed code — and only then marks the transaction
//!    `Paid`, tagged with `provider = "manual"` and `provider_ref = <voucher code>` so the
//!    ledger honestly reflects how it was actually paid.
//!
//! Once a real provider (Midtrans/Xendit/etc.) is available, swapping `NotConfiguredProvider`
//! for a real implementation in `main.rs` is the only change needed for path 1-2; the manual
//! path stays available as a fallback either way.

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::application::notification_service::NotificationService;
use crate::application::voucher_service::VoucherService;
use crate::domain::package::Package;
use crate::domain::payment::{CheckoutSession, PaymentProvider, PaymentStatus, PaymentTransaction, WebhookOutcome};
use crate::domain::repository::{
    NewPaymentTransaction, PackageRepository, PaymentStatusUpdate, PaymentTransactionRepository, UserFilter,
    UserRepository,
};
use crate::domain::user::Role;
use crate::domain::voucher::Voucher;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

/// A `Pending` transaction older than this with no admin action is treated as abandoned —
/// auto-swept to `Expired` the next time the admin queue (`list_pending`) is read, so requests
/// nobody ever followed up on don't accumulate forever with no signal to anyone. Lazy
/// sweep-on-read rather than a background job, since this codebase has no scheduler/cron
/// infrastructure at all (see audit notes) and this keeps the fix self-contained.
const PENDING_EXPIRY_DAYS: i64 = 3;

/// Outcome of `initiate_purchase` — distinguishes a real online checkout from the honest
/// "no provider configured, this now waits for an admin" state. `PendingManual` is not an
/// error: the transaction row is real and the caller should show the buyer a confirmation
/// screen, not a failure.
pub enum PurchaseOutcome {
    Checkout(CheckoutSession),
    /// Provider isn't configured — this is an expected, valid product state (manual
    /// purchase request flow), not an error. The transaction stays `pending` until an
    /// admin fulfills it via `fulfill()`.
    PendingManual,
}

pub struct PaymentService {
    transactions: Arc<dyn PaymentTransactionRepository>,
    packages: Arc<dyn PackageRepository>,
    provider: Arc<dyn PaymentProvider>,
    voucher_service: Arc<VoucherService>,
    notification_service: Arc<NotificationService>,
    users: Arc<dyn UserRepository>,
}

impl PaymentService {
    pub fn new(
        transactions: Arc<dyn PaymentTransactionRepository>,
        packages: Arc<dyn PackageRepository>,
        provider: Arc<dyn PaymentProvider>,
        voucher_service: Arc<VoucherService>,
        notification_service: Arc<NotificationService>,
        users: Arc<dyn UserRepository>,
    ) -> Self {
        Self { transactions, packages, provider, voucher_service, notification_service, users }
    }

    /// Best-effort fan-out notify to every Admin account — a notification failing to write for
    /// one admin must never block the others or the caller's primary action, so every error is
    /// logged and swallowed (same tolerance as the single-recipient `notify()` calls elsewhere
    /// in this service).
    async fn notify_all_admins(&self, kind: &str, title: &str, message: &str, link_tab: Option<&str>) {
        let admins = match self.users.list(UserFilter { role: Some(Role::Admin), ..Default::default() }).await {
            Ok(list) => list,
            Err(e) => {
                tracing::warn!(error = %e, "gagal memuat daftar admin untuk notifikasi");
                return;
            }
        };
        for admin in admins {
            if let Err(e) = self
                .notification_service
                .notify(admin.id, kind.to_string(), title.to_string(), message.to_string(), link_tab.map(str::to_string))
                .await
            {
                tracing::warn!(error = %e, admin_id = %admin.id, "gagal membuat notifikasi admin");
            }
        }
    }

    /// Which provider is currently active — surfaced to the frontend so it can show an
    /// honest "pembayaran online belum tersedia" state instead of a generic error when
    /// `provider.name() == "not_configured"`.
    pub fn provider_name(&self) -> &'static str {
        self.provider.name()
    }

    async fn find_package(&self, package_id: Uuid) -> AppResult<Package> {
        self.packages
            .find_by_id(package_id)
            .await?
            .ok_or_else(|| AppError::NotFound("paket tidak ditemukan".to_string()))
    }

    /// Creates a real pending transaction row for `package_id`, priced at the package's
    /// real `sale_price`, then attempts to open a checkout with whatever provider is
    /// configured. The transaction row is created either way (it's real, auditable state);
    /// only the checkout-session step depends on a real provider being wired in.
    pub async fn initiate_purchase(&self, actor: &AuthUser, package_id: Uuid) -> AppResult<(PaymentTransaction, PurchaseOutcome)> {
        actor.require_role(&[Role::Student, Role::School])?;
        let package = self.find_package(package_id).await?;
        if !package.active {
            return Err(AppError::Validation("paket ini sedang tidak tersedia untuk dibeli".to_string()));
        }

        let transaction = self
            .transactions
            .create(NewPaymentTransaction {
                package_id: package.id,
                buyer_user_id: actor.user_id,
                amount: package.sale_price,
                currency: "IDR".to_string(),
            })
            .await?;

        // Honest behavior: with `NotConfiguredProvider` this returns
        // `Err(AppError::NotImplemented(_))`, which is treated as the expected
        // "waits for manual admin fulfillment" state, not a failure — the transaction row
        // is real either way. Any other error from a real future provider is a genuine
        // failure and must still propagate.
        match self.provider.create_checkout(&transaction).await {
            Ok(checkout) => Ok((transaction, PurchaseOutcome::Checkout(checkout))),
            Err(AppError::NotImplemented(_)) => {
                // Previously nothing signaled a new manual purchase request existed — an admin
                // had to remember to periodically re-check "Permintaan Pembelian" manually.
                self.notify_all_admins(
                    "purchase_pending",
                    "Ada permintaan pembelian baru",
                    &format!("\"{}\" — Rp {} menunggu konfirmasi manual di tab Permintaan Pembelian.", package.name, package.sale_price),
                    Some("purchase-requests"),
                )
                .await;
                Ok((transaction, PurchaseOutcome::PendingManual))
            }
            Err(e) => Err(e),
        }
    }

    /// Admin-only queue of transactions waiting on manual confirmation (bank transfer,
    /// etc.) — everything currently `Pending`, most recent first. Lazily sweeps any `Pending`
    /// row older than `PENDING_EXPIRY_DAYS` to `Expired` first, so a request nobody ever acted
    /// on eventually falls out of the active queue instead of sitting there forever.
    pub async fn list_pending(&self, actor: &AuthUser) -> AppResult<Vec<PaymentTransaction>> {
        actor.require_role(&[Role::Admin])?;
        let pending = self.transactions.list_by_status(PaymentStatus::Pending).await?;
        let cutoff = Utc::now() - chrono::Duration::days(PENDING_EXPIRY_DAYS);
        let mut still_pending = Vec::with_capacity(pending.len());
        for tx in pending {
            if tx.created_at < cutoff {
                let _ = self
                    .transactions
                    .update_status(
                        tx.id,
                        PaymentStatusUpdate {
                            status: PaymentStatus::Expired,
                            provider: tx.provider.clone(),
                            provider_ref: tx.provider_ref.clone(),
                            failure_reason: Some(format!(
                                "Otomatis kedaluwarsa — tidak ada konfirmasi admin dalam {PENDING_EXPIRY_DAYS} hari"
                            )),
                        },
                    )
                    .await;
                // Not pushed to still_pending — it just left the active queue.
            } else {
                still_pending.push(tx);
            }
        }
        Ok(still_pending)
    }

    /// Self-service cancel for the buyer, while still `Pending` — previously the only way out
    /// of a stray pending transaction was to wait `PENDING_EXPIRY_DAYS` for the passive sweep
    /// above or ask an admin. `UserFilter`'s `Cancelled` status was already modeled but nothing
    /// could ever reach it.
    pub async fn cancel_transaction(&self, actor: &AuthUser, transaction_id: Uuid) -> AppResult<PaymentTransaction> {
        let tx = self
            .transactions
            .find_by_id(transaction_id)
            .await?
            .ok_or_else(|| AppError::NotFound("transaksi tidak ditemukan".to_string()))?;
        if tx.buyer_user_id != actor.user_id {
            return Err(AppError::Forbidden("transaksi ini bukan milik Anda".to_string()));
        }
        if tx.status != PaymentStatus::Pending {
            return Err(AppError::Conflict("transaksi ini sudah diproses, tidak bisa dibatalkan lagi".to_string()));
        }
        self.transactions
            .update_status(
                tx.id,
                PaymentStatusUpdate {
                    status: PaymentStatus::Cancelled,
                    provider: tx.provider.clone(),
                    provider_ref: tx.provider_ref.clone(),
                    failure_reason: None,
                },
            )
            .await
    }

    /// Admin confirms a manual purchase request (e.g. after verifying a bank transfer).
    /// Grants the package via a real, single-use Access voucher — auto-redeemed for the
    /// buyer — then marks the transaction `Paid`. Never marks anything paid without this
    /// explicit admin action.
    pub async fn fulfill(&self, actor: &AuthUser, transaction_id: Uuid) -> AppResult<(PaymentTransaction, Voucher)> {
        actor.require_role(&[Role::Admin])?;
        let transaction = self
            .transactions
            .find_by_id(transaction_id)
            .await?
            .ok_or_else(|| AppError::NotFound("transaksi tidak ditemukan".to_string()))?;
        if transaction.status != PaymentStatus::Pending {
            return Err(AppError::Conflict("transaksi ini sudah diproses sebelumnya".to_string()));
        }

        let voucher = self
            .voucher_service
            .grant_access(
                transaction.package_id,
                transaction.buyer_user_id,
                actor.user_id,
                format!("Konfirmasi pembelian manual — transaksi {}", transaction.id),
            )
            .await?;

        let updated = self
            .transactions
            .update_status(
                transaction.id,
                PaymentStatusUpdate {
                    status: PaymentStatus::Paid,
                    provider: Some("manual".to_string()),
                    provider_ref: Some(voucher.code.clone()),
                    failure_reason: None,
                },
            )
            .await?;

        // Real-time signal for the buyer instead of them having to keep refreshing "Riwayat
        // Pembelian" to notice access opened up — see `application::notification_service` doc
        // comment. A notification failing to write must never fail the fulfillment itself
        // (the buyer already has real access via the voucher above), so this is logged and
        // swallowed rather than propagated.
        let package_name = self
            .find_package(transaction.package_id)
            .await
            .map(|p| p.name)
            .unwrap_or_else(|_| "paket".to_string());
        if let Err(e) = self
            .notification_service
            .notify(
                transaction.buyer_user_id,
                "purchase_approved",
                "Paket berhasil dibuka!",
                format!("Pembelian \"{package_name}\" sudah dikonfirmasi — akses premium kamu aktif sekarang."),
                Some("overview".to_string()),
            )
            .await
        {
            tracing::warn!(error = %e, transaction_id = %transaction.id, "gagal membuat notifikasi purchase_approved");
        }

        Ok((updated, voucher))
    }

    pub async fn list_my_transactions(&self, actor: &AuthUser) -> AppResult<Vec<PaymentTransaction>> {
        self.transactions.list_by_buyer(actor.user_id).await
    }

    pub async fn get_transaction(&self, actor: &AuthUser, id: Uuid) -> AppResult<PaymentTransaction> {
        let tx = self
            .transactions
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("transaksi tidak ditemukan".to_string()))?;
        if tx.buyer_user_id != actor.user_id {
            actor.require_role(&[Role::Admin])?;
        }
        Ok(tx)
    }

    /// Webhook receiver — verifies the raw payload against the configured provider, then
    /// updates the matching transaction's status. With `NotConfiguredProvider` this always
    /// errors at the verification step (no secret to verify against), so no transaction
    /// can ever be marked paid through this path today. Real providers retry webhooks, so
    /// callers should treat a `NotImplemented` response as "acknowledged, nothing to do"
    /// rather than something to retry indefinitely.
    pub async fn handle_webhook(&self, raw_body: &[u8], signature_header: Option<&str>) -> AppResult<PaymentTransaction> {
        let outcome: WebhookOutcome = self.provider.verify_webhook(raw_body, signature_header).await?;
        let tx = self
            .transactions
            .find_by_provider_ref(&outcome.provider_ref)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("transaksi dengan provider_ref {} tidak ditemukan", outcome.provider_ref)))?;
        self.transactions
            .update_status(
                tx.id,
                PaymentStatusUpdate {
                    status: outcome.status,
                    provider: Some(self.provider.name().to_string()),
                    provider_ref: Some(outcome.provider_ref),
                    failure_reason: outcome.failure_reason,
                },
            )
            .await
    }
}
