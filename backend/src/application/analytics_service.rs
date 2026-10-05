//! Admin Analytics use-cases. Purely read-only aggregation over real data — no writes,
//! no CRUD, so unlike the other services this one only ever needs a role check + a
//! pass-through to the repository.

use std::sync::Arc;

use crate::domain::analytics::{
    AnalyticsSummary, QuestionStatusCount, SchoolRanking, ScoreTrendPoint, StudentActivity, StudentRanking,
    SubjectAccuracy, YearlyPerformancePoint,
};
use crate::domain::repository::{AnalyticsRepository, UserRepository};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct AnalyticsService {
    analytics: Arc<dyn AnalyticsRepository>,
    users: Arc<dyn UserRepository>,
}

impl AnalyticsService {
    pub fn new(analytics: Arc<dyn AnalyticsRepository>, users: Arc<dyn UserRepository>) -> Self {
        Self { analytics, users }
    }

    async fn resolve_own_school_id(&self, actor: &AuthUser) -> AppResult<uuid::Uuid> {
        let me = self
            .users
            .find_by_id(actor.user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("akun tidak ditemukan".to_string()))?;
        me.school_id
            .ok_or_else(|| AppError::Validation("akun sekolah ini belum terhubung ke data sekolah".to_string()))
    }

    /// School portal's own "Analytics Sekolah" tab — topic-mastery accuracy scoped to this
    /// school's real students only.
    pub async fn subject_breakdown_for_school(&self, actor: &AuthUser) -> AppResult<Vec<SubjectAccuracy>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.analytics.subject_breakdown_for_school(school_id).await
    }

    /// School portal's own score trend (used by the Laporan "Performa Siswa" report).
    pub async fn score_trend_for_school(&self, actor: &AuthUser, days: i32) -> AppResult<Vec<ScoreTrendPoint>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.analytics.score_trend_for_school(school_id, days.clamp(1, 365)).await
    }

    /// School portal's "Perbandingan Tahun" — attempts bucketed by the calendar year of
    /// `submitted_at`. Deliberately NOT student "angkatan"/cohort: there is no reliable
    /// cohort field in this system today (`enrolled_at` is optional/sparse, `grade`/
    /// `rombel_code` are current-class-level snapshots that change every academic year,
    /// not stable cohort ids) — see `domain::analytics::YearlyPerformancePoint` doc
    /// comment. Using the real, always-populated `submitted_at` avoids silently excluding
    /// or misclassifying students with missing cohort data.
    pub async fn score_trend_by_year_for_school(&self, actor: &AuthUser) -> AppResult<Vec<YearlyPerformancePoint>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.analytics.score_trend_by_year_for_school(school_id).await
    }

    /// School portal's internal student ranking (Rekap SNBT / Laporan "Ranking Internal").
    pub async fn student_ranking_for_school(&self, actor: &AuthUser) -> AppResult<Vec<StudentRanking>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.analytics.student_ranking_for_school(school_id).await
    }

    /// School portal's "Analytics & Insights" student drill-down (Ringkasan per Kelas +
    /// Tabel Siswa) — see `domain::analytics::StudentActivity` doc comment.
    pub async fn student_activity_for_school(&self, actor: &AuthUser) -> AppResult<Vec<StudentActivity>> {
        actor.require_role(&[Role::School])?;
        let school_id = self.resolve_own_school_id(actor).await?;
        self.analytics.student_activity_for_school(school_id).await
    }

    /// Student portal's own Overview mastery breakdown — scoped to the acting student's
    /// own real answers only.
    pub async fn my_subject_breakdown(&self, actor: &AuthUser) -> AppResult<Vec<SubjectAccuracy>> {
        actor.require_role(&[Role::Student])?;
        self.analytics.subject_breakdown_for_student(actor.user_id).await
    }

    /// Platform-wide average score trend, exposed to any authenticated role — used by the
    /// student portal's Progress tab to draw a "Rata-rata Nasional" comparison line next to
    /// the student's own trend. Purely aggregate/anonymized (no per-student data leaks),
    /// so unlike `score_trend` this isn't admin-only. Same underlying real query.
    pub async fn national_score_trend(&self, actor: &AuthUser, days: i32) -> AppResult<Vec<ScoreTrendPoint>> {
        actor.require_role(&[Role::Admin, Role::Student, Role::School])?;
        self.analytics.score_trend(days.clamp(1, 365)).await
    }

    pub async fn summary(&self, actor: &AuthUser) -> AppResult<AnalyticsSummary> {
        actor.require_role(&[Role::Admin])?;
        self.analytics.summary().await
    }

    /// No-auth read for the public marketing landing page — same real system-wide
    /// aggregate query as `summary()`, just without the Admin gate. Lets the landing page
    /// show real platform numbers instead of hardcoded marketing figures.
    pub async fn public_summary(&self) -> AppResult<AnalyticsSummary> {
        self.analytics.summary().await
    }

    pub async fn score_trend(&self, actor: &AuthUser, days: i32) -> AppResult<Vec<ScoreTrendPoint>> {
        actor.require_role(&[Role::Admin])?;
        let days = days.clamp(1, 365);
        self.analytics.score_trend(days).await
    }

    pub async fn subject_breakdown(&self, actor: &AuthUser) -> AppResult<Vec<SubjectAccuracy>> {
        actor.require_role(&[Role::Admin])?;
        self.analytics.subject_breakdown().await
    }

    pub async fn question_status_distribution(&self, actor: &AuthUser) -> AppResult<Vec<QuestionStatusCount>> {
        actor.require_role(&[Role::Admin])?;
        self.analytics.question_status_distribution().await
    }

    pub async fn top_schools(&self, actor: &AuthUser, limit: i64) -> AppResult<Vec<SchoolRanking>> {
        actor.require_role(&[Role::Admin])?;
        let limit = limit.clamp(1, 50);
        self.analytics.top_schools(limit).await
    }

    /// Admin's platform-wide "Perbandingan Tahun" — same year-bucketing as
    /// `score_trend_by_year_for_school`, unscoped. See that method's doc comment for why
    /// "tahun" means calendar year of `submitted_at`, not student angkatan/cohort.
    pub async fn score_trend_by_year(&self, actor: &AuthUser) -> AppResult<Vec<YearlyPerformancePoint>> {
        actor.require_role(&[Role::Admin])?;
        self.analytics.score_trend_by_year().await
    }
}
