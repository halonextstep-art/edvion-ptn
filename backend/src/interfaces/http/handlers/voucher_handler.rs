use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::voucher_dto::{
    RedeemPayload, RedemptionResponse, VoucherActivePayload, VoucherPayload, VoucherResponse,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<VoucherPayload>,
) -> AppResult<Json<Vec<VoucherResponse>>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_input()?;
    let vouchers = state.voucher_service.create(&auth, input).await?;
    Ok(Json(vouchers.into_iter().map(Into::into).collect()))
}

pub async fn list(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<VoucherResponse>>> {
    let vouchers = state.voucher_service.list(&auth).await?;
    Ok(Json(vouchers.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<VoucherResponse>> {
    let v = state.voucher_service.get(&auth, id).await?;
    Ok(Json(v.into()))
}

pub async fn set_active(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<VoucherActivePayload>,
) -> AppResult<Json<VoucherResponse>> {
    let v = state.voucher_service.set_active(&auth, id, payload.active).await?;
    Ok(Json(v.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    state.voucher_service.delete(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn redeem(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<RedeemPayload>,
) -> AppResult<Json<RedemptionResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let result = state.voucher_service.redeem(&auth, &payload.code).await?;
    Ok(Json(result.into()))
}
