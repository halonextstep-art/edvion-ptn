//! Payment handlers — checkout + own-transaction history are authenticated (student/school
//! buyers only); the webhook receiver is intentionally public (see doc comment on
//! `webhook`), matching how every real payment gateway calls back.

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::application::payment_service::PurchaseOutcome;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::payment_dto::{
    FulfillResponse, InitiatePurchasePayload, InitiatePurchaseResponse, PaymentProviderStatusResponse,
    PaymentTransactionResponse, PendingPurchaseResponse,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn provider_status(State(state): State<AppState>, _auth: AuthUser) -> Json<PaymentProviderStatusResponse> {
    let provider = state.payment_service.provider_name().to_string();
    Json(PaymentProviderStatusResponse { configured: provider != "not_configured", provider })
}

pub async fn initiate_purchase(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<InitiatePurchasePayload>,
) -> AppResult<Json<InitiatePurchaseResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let (transaction, outcome) = state.payment_service.initiate_purchase(&auth, payload.package_id).await?;
    let response = match outcome {
        PurchaseOutcome::Checkout(checkout) => {
            InitiatePurchaseResponse::Checkout { transaction: transaction.into(), checkout: checkout.into() }
        }
        PurchaseOutcome::PendingManual => InitiatePurchaseResponse::ManualPending { transaction: transaction.into() },
    };
    Ok(Json(response))
}

/// Admin-only manual purchase-request queue — every `Pending` transaction, enriched with the
/// buyer's name/email and the package's name so the admin doesn't have to cross-reference IDs
/// by hand. Enrichment is deliberately done here (not inside `PaymentService`) via the
/// already-existing `UserService`/`PackageService`, per-transaction (N+1 is acceptable given
/// expected volume for a manual-review queue).
pub async fn list_pending(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<PendingPurchaseResponse>>> {
    let transactions = state.payment_service.list_pending(&auth).await?;
    let mut out = Vec::with_capacity(transactions.len());
    for tx in transactions {
        let buyer = state.user_service.get(&auth, tx.buyer_user_id).await?;
        let package = state.package_service.get(&auth, tx.package_id).await?;
        out.push(PendingPurchaseResponse {
            buyer_name: buyer.name,
            buyer_email: buyer.email,
            package_name: package.name,
            transaction: tx.into(),
        });
    }
    Ok(Json(out))
}

/// Admin confirms a manual purchase request (e.g. after checking a bank transfer) — grants
/// the package via a real, auto-redeemed Access voucher and marks the transaction `Paid`.
pub async fn fulfill(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<FulfillResponse>> {
    let (transaction, voucher) = state.payment_service.fulfill(&auth, id).await?;
    Ok(Json(FulfillResponse { transaction: transaction.into(), voucher_code: voucher.code }))
}

pub async fn my_transactions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<PaymentTransactionResponse>>> {
    let txs = state.payment_service.list_my_transactions(&auth).await?;
    Ok(Json(txs.into_iter().map(Into::into).collect()))
}

/// Self-service cancel while still `Pending` — see `PaymentService::cancel_transaction` doc
/// comment.
pub async fn cancel_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PaymentTransactionResponse>> {
    let tx = state.payment_service.cancel_transaction(&auth, id).await?;
    Ok(Json(tx.into()))
}

pub async fn get_transaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PaymentTransactionResponse>> {
    let tx = state.payment_service.get_transaction(&auth, id).await?;
    Ok(Json(tx.into()))
}

/// Deliberately takes no `AuthUser` — real payment gateways call this server-to-server
/// with no user session, authenticating instead via a provider-specific signature header
/// that `PaymentProvider::verify_webhook` checks. `NotConfiguredProvider` has no secret to
/// verify against, so this always responds with a clear "not configured" error today; it
/// never fabricates a successful payment confirmation.
pub async fn webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> AppResult<Json<PaymentTransactionResponse>> {
    let signature = headers
        .get("x-payment-signature")
        .and_then(|v| v.to_str().ok());
    let tx = state.payment_service.handle_webhook(&body, signature).await?;
    Ok(Json(tx.into()))
}
