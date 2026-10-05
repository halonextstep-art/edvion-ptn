use axum::extract::State;
use axum::Json;

use crate::error::AppResult;
use crate::interfaces::http::dto::finance_dto::{EventRevenueResponse, FinanceSummaryResponse, SchoolRevenueResponse};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

pub async fn summary(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<FinanceSummaryResponse>> {
    let s = state.finance_service.summary(&auth).await?;
    Ok(Json(s.into()))
}

pub async fn event_breakdown(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<EventRevenueResponse>>> {
    let rows = state.finance_service.event_breakdown(&auth).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

pub async fn school_breakdown(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<SchoolRevenueResponse>>> {
    let rows = state.finance_service.school_breakdown(&auth).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}
