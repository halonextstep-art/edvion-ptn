use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::domain::analytics::{
    AnalyticsSummary, QuestionStatusCount, SchoolRanking, ScoreTrendPoint, StudentActivity, StudentRanking,
    SubjectAccuracy, YearlyPerformancePoint,
};
use crate::error::AppResult;
use crate::interfaces::http::middleware::AuthUser;
use crate::interfaces::http::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ScoreTrendQuery {
    /// How many trailing days to include (clamped to 1..=90 in the service layer).
    pub days: Option<i32>,
}

pub async fn summary(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<AnalyticsSummary>> {
    Ok(Json(state.analytics_service.summary(&auth).await?))
}

pub async fn score_trend(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ScoreTrendQuery>,
) -> AppResult<Json<Vec<ScoreTrendPoint>>> {
    let days = q.days.unwrap_or(14);
    Ok(Json(state.analytics_service.score_trend(&auth, days).await?))
}

pub async fn subject_breakdown(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<SubjectAccuracy>>> {
    Ok(Json(state.analytics_service.subject_breakdown(&auth).await?))
}

pub async fn question_status(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<QuestionStatusCount>>> {
    Ok(Json(state.analytics_service.question_status_distribution(&auth).await?))
}

#[derive(Debug, Deserialize)]
pub struct TopSchoolsQuery {
    pub limit: Option<i64>,
}

pub async fn top_schools(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<TopSchoolsQuery>,
) -> AppResult<Json<Vec<SchoolRanking>>> {
    let limit = q.limit.unwrap_or(8);
    Ok(Json(state.analytics_service.top_schools(&auth, limit).await?))
}

pub async fn score_trend_by_year(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<YearlyPerformancePoint>>> {
    Ok(Json(state.analytics_service.score_trend_by_year(&auth).await?))
}

// ─── School-portal-scoped variants ─────────────────────────────────────────────

pub async fn school_subject_breakdown(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<SubjectAccuracy>>> {
    Ok(Json(state.analytics_service.subject_breakdown_for_school(&auth).await?))
}

pub async fn school_score_trend(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ScoreTrendQuery>,
) -> AppResult<Json<Vec<ScoreTrendPoint>>> {
    let days = q.days.unwrap_or(30);
    Ok(Json(state.analytics_service.score_trend_for_school(&auth, days).await?))
}

pub async fn school_score_trend_by_year(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<YearlyPerformancePoint>>> {
    Ok(Json(state.analytics_service.score_trend_by_year_for_school(&auth).await?))
}

pub async fn school_student_ranking(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<StudentRanking>>> {
    Ok(Json(state.analytics_service.student_ranking_for_school(&auth).await?))
}

pub async fn school_student_activity(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<StudentActivity>>> {
    Ok(Json(state.analytics_service.student_activity_for_school(&auth).await?))
}

// ─── Student-portal-scoped variant ─────────────────────────────────────────────

pub async fn my_subject_breakdown(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<SubjectAccuracy>>> {
    Ok(Json(state.analytics_service.my_subject_breakdown(&auth).await?))
}

pub async fn national_score_trend(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ScoreTrendQuery>,
) -> AppResult<Json<Vec<ScoreTrendPoint>>> {
    let days = q.days.unwrap_or(14);
    Ok(Json(state.analytics_service.national_score_trend(&auth, days).await?))
}
