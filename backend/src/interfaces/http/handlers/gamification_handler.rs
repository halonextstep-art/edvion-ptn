use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::interfaces::http::dto::gamification_dto::{
    ActivePayload, BadgePayload, BadgeResponse, ChallengePayload, ChallengeResponse, ChallengeStatusPayload,
    LeaderboardEntryResponse, PointRulePayload, PointRuleResponse, PointRuleUpdatePayload, StudentBadgeStatusResponse,
};
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

// ─── Point rules ────────────────────────────────────────────────────────────────

pub async fn create_point_rule(State(state): State<AppState>, auth: AuthUser, Json(payload): Json<PointRulePayload>) -> AppResult<Json<PointRuleResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let rule = state
        .gamification_service
        .create_point_rule(&auth, crate::application::gamification_service::CreatePointRuleInput {
            action: payload.action, category: payload.category, base_points: payload.base_points,
            multiplier: payload.multiplier, enabled: payload.enabled, icon: payload.icon, sort_order: payload.sort_order,
        })
        .await?;
    Ok(Json(rule.into()))
}

pub async fn list_point_rules(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<PointRuleResponse>>> {
    let rules = state.gamification_service.list_point_rules(&auth).await?;
    Ok(Json(rules.into_iter().map(Into::into).collect()))
}

pub async fn update_point_rule(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>, Json(payload): Json<PointRuleUpdatePayload>) -> AppResult<Json<PointRuleResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let rule = state.gamification_service.update_point_rule(&auth, id, payload.base_points, payload.multiplier, payload.enabled).await?;
    Ok(Json(rule.into()))
}

pub async fn delete_point_rule(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<serde_json::Value>> {
    state.gamification_service.delete_point_rule(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ─── Badges ─────────────────────────────────────────────────────────────────────

pub async fn create_badge(State(state): State<AppState>, auth: AuthUser, Json(payload): Json<BadgePayload>) -> AppResult<Json<BadgeResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_input()?;
    let badge = state.gamification_service.create_badge(&auth, input).await?;
    Ok(Json(badge.into()))
}

pub async fn list_badges(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<BadgeResponse>>> {
    let badges = state.gamification_service.list_badges(&auth).await?;
    Ok(Json(badges.into_iter().map(Into::into).collect()))
}

pub async fn update_badge(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>, Json(payload): Json<BadgePayload>) -> AppResult<Json<BadgeResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_input()?;
    let badge = state.gamification_service.update_badge(&auth, id, input).await?;
    Ok(Json(badge.into()))
}

pub async fn set_badge_active(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>, Json(payload): Json<ActivePayload>) -> AppResult<Json<BadgeResponse>> {
    let badge = state.gamification_service.set_badge_active(&auth, id, payload.active).await?;
    Ok(Json(badge.into()))
}

pub async fn delete_badge(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<serde_json::Value>> {
    state.gamification_service.delete_badge(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

pub async fn my_badges(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<StudentBadgeStatusResponse>>> {
    let statuses = state.gamification_service.my_badges(&auth).await?;
    Ok(Json(statuses.into_iter().map(Into::into).collect()))
}

// ─── Challenges ─────────────────────────────────────────────────────────────────

pub async fn create_challenge(State(state): State<AppState>, auth: AuthUser, Json(payload): Json<ChallengePayload>) -> AppResult<Json<ChallengeResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_input()?;
    let c = state.gamification_service.create_challenge(&auth, input).await?;
    Ok(Json(c.into()))
}

pub async fn list_challenges(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Vec<ChallengeResponse>>> {
    let cs = state.gamification_service.list_challenges(&auth).await?;
    Ok(Json(cs.into_iter().map(Into::into).collect()))
}

pub async fn update_challenge(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>, Json(payload): Json<ChallengePayload>) -> AppResult<Json<ChallengeResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let input = payload.into_input()?;
    let c = state.gamification_service.update_challenge(&auth, id, input).await?;
    Ok(Json(c.into()))
}

pub async fn set_challenge_status(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>, Json(payload): Json<ChallengeStatusPayload>) -> AppResult<Json<ChallengeResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let status = payload.status.parse().map_err(AppError::Validation)?;
    let c = state.gamification_service.set_challenge_status(&auth, id, status).await?;
    Ok(Json(c.into()))
}

pub async fn delete_challenge(State(state): State<AppState>, auth: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<serde_json::Value>> {
    state.gamification_service.delete_challenge(&auth, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ─── Leaderboard ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub limit: Option<i64>,
    /// "national" | "school" — defaults to national.
    pub scope: Option<String>,
    /// "today" | "week" | "all_time" — defaults to all_time.
    pub range: Option<String>,
    /// When set, ranks by real answer-accuracy % within this subject instead of overall score.
    pub subject: Option<String>,
}

pub async fn leaderboard(State(state): State<AppState>, auth: AuthUser, Query(q): Query<LeaderboardQuery>) -> AppResult<Json<Vec<LeaderboardEntryResponse>>> {
    let scope = q.scope.as_deref().unwrap_or("national").parse().map_err(AppError::Validation)?;
    let range = q.range.as_deref().unwrap_or("all_time").parse().map_err(AppError::Validation)?;
    let entries = state.gamification_service.leaderboard(&auth, q.limit.unwrap_or(20), scope, range, q.subject).await?;
    Ok(Json(entries.into_iter().map(Into::into).collect()))
}
