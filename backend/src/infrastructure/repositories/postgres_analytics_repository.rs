//! Read-only aggregation queries backing the Admin Analytics screen. Every number here
//! comes straight from `attempts`/`attempt_answers`/`questions`/`users`/`schools` — no
//! seeded/mocked figures, so the dashboard only ever shows what's actually happened.

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::analytics::{
    AnalyticsSummary, QuestionStatusCount, SchoolRanking, ScoreTrendPoint, StudentActivity, StudentRanking,
    SubjectAccuracy, YearlyPerformancePoint,
};
use crate::domain::repository::AnalyticsRepository;
use crate::error::AppResult;

pub struct PostgresAnalyticsRepository {
    pool: PgPool,
}

impl PostgresAnalyticsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ScoreTrendRow {
    day: NaiveDate,
    attempts: i64,
    average_score: f64,
}

#[derive(sqlx::FromRow)]
struct YearlyPerformanceRow {
    year: i32,
    attempts: i64,
    distinct_students: i64,
    average_score: f64,
}

#[derive(sqlx::FromRow)]
struct SubjectRow {
    subject: String,
    correct: i64,
    total: i64,
}

#[derive(sqlx::FromRow)]
struct StatusRow {
    status: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
struct SchoolRankingRow {
    school_id: Uuid,
    school_name: String,
    active_students: i64,
    average_score: f64,
}

#[derive(sqlx::FromRow)]
struct StudentRankingRow {
    student_id: Uuid,
    student_name: String,
    attempts_count: i64,
    best_score: i32,
    average_score: f64,
}

#[derive(sqlx::FromRow)]
struct StudentActivityRow {
    student_id: Uuid,
    student_name: String,
    rombel_code: Option<String>,
    attempts_count: i64,
    best_score: Option<i32>,
    average_score: Option<f64>,
    last_attempt_at: Option<DateTime<Utc>>,
}

#[async_trait]
impl AnalyticsRepository for PostgresAnalyticsRepository {
    async fn summary(&self) -> AppResult<AnalyticsSummary> {
        let total_students: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'student'")
            .fetch_one(&self.pool)
            .await?;
        let total_schools: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schools")
            .fetch_one(&self.pool)
            .await?;
        let total_questions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM questions")
            .fetch_one(&self.pool)
            .await?;
        let approved_questions: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM questions WHERE status = 'approved'")
                .fetch_one(&self.pool)
                .await?;
        let total_attempts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attempts")
            .fetch_one(&self.pool)
            .await?;
        let submitted_attempts: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM attempts WHERE status = 'submitted'")
                .fetch_one(&self.pool)
                .await?;
        // AVG() on an integer column returns `numeric` in Postgres, which sqlx can't
        // decode straight into `f64` — cast to `float8` (double precision) explicitly.
        let average_score: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(score), 0)::float8 FROM attempts WHERE status = 'submitted'",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AnalyticsSummary {
            total_students,
            total_schools,
            total_questions,
            approved_questions,
            total_attempts,
            submitted_attempts,
            average_score,
        })
    }

    async fn score_trend(&self, days: i32) -> AppResult<Vec<ScoreTrendPoint>> {
        let rows = sqlx::query_as::<_, ScoreTrendRow>(
            r#"SELECT date_trunc('day', submitted_at)::date AS day,
                      COUNT(*) AS attempts,
                      COALESCE(AVG(score), 0)::float8 AS average_score
               FROM attempts
               WHERE status = 'submitted' AND submitted_at >= now() - make_interval(days => $1)
               GROUP BY day
               ORDER BY day ASC"#,
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ScoreTrendPoint { day: r.day, attempts: r.attempts, average_score: r.average_score })
            .collect())
    }

    async fn subject_breakdown(&self) -> AppResult<Vec<SubjectAccuracy>> {
        let rows = sqlx::query_as::<_, SubjectRow>(
            r#"SELECT q.subject AS subject,
                      COUNT(*) FILTER (WHERE aa.is_correct = true) AS correct,
                      COUNT(*) AS total
               FROM attempt_answers aa
               JOIN questions q ON q.id = aa.question_id
               WHERE aa.is_correct IS NOT NULL
               GROUP BY q.subject
               ORDER BY total DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SubjectAccuracy { subject: r.subject, correct: r.correct, total: r.total })
            .collect())
    }

    async fn question_status_distribution(&self) -> AppResult<Vec<QuestionStatusCount>> {
        let rows = sqlx::query_as::<_, StatusRow>(
            "SELECT status, COUNT(*) AS count FROM questions GROUP BY status ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| QuestionStatusCount { status: r.status, count: r.count }).collect())
    }

    async fn top_schools(&self, limit: i64) -> AppResult<Vec<SchoolRanking>> {
        // INNER JOINs are intentional here — a school with zero submitted attempts has
        // nothing to rank, so it's correctly left out rather than shown with a fake 0.
        let rows = sqlx::query_as::<_, SchoolRankingRow>(
            r#"SELECT s.id AS school_id, s.name AS school_name,
                      COUNT(DISTINCT a.student_id) AS active_students,
                      COALESCE(AVG(a.score), 0)::float8 AS average_score
               FROM schools s
               JOIN users u ON u.school_id = s.id AND u.role = 'student'
               JOIN attempts a ON a.student_id = u.id AND a.status = 'submitted'
               GROUP BY s.id, s.name
               ORDER BY average_score DESC
               LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SchoolRanking {
                school_id: r.school_id,
                school_name: r.school_name,
                active_students: r.active_students,
                average_score: r.average_score,
            })
            .collect())
    }

    async fn score_trend_by_year(&self) -> AppResult<Vec<YearlyPerformancePoint>> {
        let rows = sqlx::query_as::<_, YearlyPerformanceRow>(
            r#"SELECT EXTRACT(YEAR FROM submitted_at)::int AS year,
                      COUNT(*) AS attempts,
                      COUNT(DISTINCT student_id) AS distinct_students,
                      COALESCE(AVG(score), 0)::float8 AS average_score
               FROM attempts
               WHERE status = 'submitted'
               GROUP BY year
               ORDER BY year ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| YearlyPerformancePoint {
                year: r.year,
                attempts: r.attempts,
                distinct_students: r.distinct_students,
                average_score: r.average_score,
            })
            .collect())
    }

    async fn subject_breakdown_for_school(&self, school_id: Uuid) -> AppResult<Vec<SubjectAccuracy>> {
        let rows = sqlx::query_as::<_, SubjectRow>(
            r#"SELECT q.subject AS subject,
                      COUNT(*) FILTER (WHERE aa.is_correct = true) AS correct,
                      COUNT(*) AS total
               FROM attempt_answers aa
               JOIN questions q ON q.id = aa.question_id
               JOIN attempts a ON a.id = aa.attempt_id
               JOIN users u ON u.id = a.student_id
               WHERE aa.is_correct IS NOT NULL AND u.school_id = $1
               GROUP BY q.subject
               ORDER BY total DESC"#,
        )
        .bind(school_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SubjectAccuracy { subject: r.subject, correct: r.correct, total: r.total })
            .collect())
    }

    async fn score_trend_for_school(&self, school_id: Uuid, days: i32) -> AppResult<Vec<ScoreTrendPoint>> {
        let rows = sqlx::query_as::<_, ScoreTrendRow>(
            r#"SELECT date_trunc('day', a.submitted_at)::date AS day,
                      COUNT(*) AS attempts,
                      COALESCE(AVG(a.score), 0)::float8 AS average_score
               FROM attempts a
               JOIN users u ON u.id = a.student_id
               WHERE a.status = 'submitted' AND u.school_id = $1
                 AND a.submitted_at >= now() - make_interval(days => $2)
               GROUP BY day
               ORDER BY day ASC"#,
        )
        .bind(school_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ScoreTrendPoint { day: r.day, attempts: r.attempts, average_score: r.average_score })
            .collect())
    }

    async fn score_trend_by_year_for_school(&self, school_id: Uuid) -> AppResult<Vec<YearlyPerformancePoint>> {
        let rows = sqlx::query_as::<_, YearlyPerformanceRow>(
            r#"SELECT EXTRACT(YEAR FROM a.submitted_at)::int AS year,
                      COUNT(*) AS attempts,
                      COUNT(DISTINCT a.student_id) AS distinct_students,
                      COALESCE(AVG(a.score), 0)::float8 AS average_score
               FROM attempts a
               JOIN users u ON u.id = a.student_id
               WHERE a.status = 'submitted' AND u.school_id = $1
               GROUP BY year
               ORDER BY year ASC"#,
        )
        .bind(school_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| YearlyPerformancePoint {
                year: r.year,
                attempts: r.attempts,
                distinct_students: r.distinct_students,
                average_score: r.average_score,
            })
            .collect())
    }

    async fn subject_breakdown_for_student(&self, student_id: Uuid) -> AppResult<Vec<SubjectAccuracy>> {
        let rows = sqlx::query_as::<_, SubjectRow>(
            r#"SELECT q.subject AS subject,
                      COUNT(*) FILTER (WHERE aa.is_correct = true) AS correct,
                      COUNT(*) AS total
               FROM attempt_answers aa
               JOIN questions q ON q.id = aa.question_id
               JOIN attempts a ON a.id = aa.attempt_id
               WHERE aa.is_correct IS NOT NULL AND a.student_id = $1
               GROUP BY q.subject
               ORDER BY total DESC"#,
        )
        .bind(student_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SubjectAccuracy { subject: r.subject, correct: r.correct, total: r.total })
            .collect())
    }

    async fn student_ranking_for_school(&self, school_id: Uuid) -> AppResult<Vec<StudentRanking>> {
        let rows = sqlx::query_as::<_, StudentRankingRow>(
            r#"SELECT u.id AS student_id, u.name AS student_name,
                      COUNT(a.id) AS attempts_count,
                      MAX(a.score) AS best_score,
                      COALESCE(AVG(a.score), 0)::float8 AS average_score
               FROM users u
               JOIN attempts a ON a.student_id = u.id AND a.status = 'submitted'
               WHERE u.role = 'student' AND u.school_id = $1
               GROUP BY u.id, u.name
               ORDER BY average_score DESC"#,
        )
        .bind(school_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| StudentRanking {
                student_id: r.student_id,
                student_name: r.student_name,
                attempts_count: r.attempts_count,
                best_score: r.best_score,
                average_score: r.average_score,
            })
            .collect())
    }

    async fn student_activity_for_school(&self, school_id: Uuid) -> AppResult<Vec<StudentActivity>> {
        // LEFT JOIN (not INNER, unlike student_ranking_for_school above) — a student with zero
        // submitted attempts must still appear as a row with attempts_count=0, not vanish from
        // the result set. See domain::analytics::StudentActivity doc comment.
        let rows = sqlx::query_as::<_, StudentActivityRow>(
            r#"SELECT u.id AS student_id, u.name AS student_name, st.rombel_code,
                      COUNT(a.id) AS attempts_count,
                      MAX(a.score) AS best_score,
                      AVG(a.score)::float8 AS average_score,
                      MAX(a.submitted_at) AS last_attempt_at
               FROM users u
               LEFT JOIN students st ON st.user_id = u.id
               LEFT JOIN attempts a ON a.student_id = u.id AND a.status = 'submitted'
               WHERE u.role = 'student' AND u.school_id = $1
               GROUP BY u.id, u.name, st.rombel_code
               ORDER BY u.name ASC"#,
        )
        .bind(school_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| StudentActivity {
                student_id: r.student_id,
                student_name: r.student_name,
                rombel_code: r.rombel_code,
                attempts_count: r.attempts_count,
                best_score: r.best_score,
                average_score: r.average_score,
                last_attempt_at: r.last_attempt_at,
            })
            .collect())
    }
}
