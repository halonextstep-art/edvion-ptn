//! No-auth handlers backing the public marketing landing page. None of these take
//! `AuthUser` — see `middleware::auth`'s doc comment: omitting that extractor parameter is
//! what makes a handler public in this codebase.

use axum::{extract::State, Json};
use serde::Deserialize;

use crate::error::AppResult;
use crate::interfaces::http::dto::package_dto::PackageResponse;
use crate::interfaces::http::dto::public_dto::{PublicStatsResponse, PublicVoucherCheckResponse};
use crate::interfaces::http::state::AppState;

pub async fn public_stats(State(state): State<AppState>) -> AppResult<Json<PublicStatsResponse>> {
    let summary = state.analytics_service.public_summary().await?;
    Ok(Json(summary.into()))
}

pub async fn public_packages(State(state): State<AppState>) -> AppResult<Json<Vec<PackageResponse>>> {
    let packages = state.package_service.list_public().await?;
    let mut responses = Vec::with_capacity(packages.len());
    for p in packages {
        let id = p.id;
        let mut resp: PackageResponse = p.into();
        // Best-effort: a lookup failure here shouldn't take down the whole landing page, it
        // just means this one card shows no content list (still shows price/features).
        resp.content_titles = state.package_service.public_content_titles(id).await.unwrap_or_default();
        responses.push(resp);
    }
    Ok(Json(responses))
}

#[derive(Debug, Deserialize)]
pub struct VoucherCheckQuery {
    pub code: String,
}

pub async fn check_voucher(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<VoucherCheckQuery>,
) -> AppResult<Json<PublicVoucherCheckResponse>> {
    let result = state.voucher_service.check_code(&q.code).await?;
    Ok(Json(result.into()))
}
