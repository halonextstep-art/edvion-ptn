//! Postgres implementation of the gamification repositories. `earned_count` and
//! `stats_for` are the load-bearing methods here: instead of ever writing a fabricated
//! "earned by" or "participants" number, they run real SQL against `attempts` /
//! `attempt_answers` at read time — same philosophy as `PostgresEventRepository`'s live
//! `participants` join.

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::gamification::{
    Badge, BadgeConditionType, BadgeWithStats, Challenge, ChallengeStatus, ChallengeType, ChallengeWithStats,
    LeaderboardEntry, LeaderboardRange, PointRule, Rarity, StudentBadgeStatus,
};
use crate::domain::repository::{
    BadgeRepository, BadgeUpdate, ChallengeRepository, ChallengeUpdate, LeaderboardParams, LeaderboardRepository,
    NewBadge, NewChallenge, NewPointRule, PointRuleRepository, PointRuleUpdate,
};
use crate::error::AppResult;

// ─── Point rules ────────────────────────────────────────────────────────────────

pub struct PostgresPointRuleRepository {
    pool: PgPool,
}

impl PostgresPointRuleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PointRuleRow {
    id: Uuid,
    action: String,
    category: String,
    base_points: i32,
    multiplier: f64,
    enabled: bool,
    icon: String,
    sort_order: i32,
    created_at: DateTime<Utc>,
}

impl From<PointRuleRow> for PointRule {
    fn from(r: PointRuleRow) -> Self {
        PointRule {
            id: r.id,
            action: r.action,
            category: r.category,
            base_points: r.base_points,
            multiplier: r.multiplier,
            enabled: r.enabled,
            icon: r.icon,
            sort_order: r.sort_order,
            created_at: r.created_at,
        }
    }
}

#[async_trait]
impl PointRuleRepository for PostgresPointRuleRepository {
    async fn create(&self, n: NewPointRule) -> AppResult<PointRule> {
        let row = sqlx::query_as::<_, PointRuleRow>(
            r#"INSERT INTO point_rules (action, category, base_points, multiplier, enabled, icon, sort_order)
               VALUES ($1,$2,$3,$4,$5,$6,$7)
               RETURNING id, action, category, base_points, multiplier, enabled, icon, sort_order, created_at"#,
        )
        .bind(&n.action)
        .bind(&n.category)
        .bind(n.base_points)
        .bind(n.multiplier)
        .bind(n.enabled)
        .bind(&n.icon)
        .bind(n.sort_order)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn list(&self) -> AppResult<Vec<PointRule>> {
        let rows = sqlx::query_as::<_, PointRuleRow>(
            "SELECT id, action, category, base_points, multiplier, enabled, icon, sort_order, created_at FROM point_rules ORDER BY sort_order ASC, created_at ASC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update(&self, id: Uuid, u: PointRuleUpdate) -> AppResult<Option<PointRule>> {
        let row = sqlx::query_as::<_, PointRuleRow>(
            r#"UPDATE point_rules SET base_points = $2, multiplier = $3, enabled = $4
               WHERE id = $1
               RETURNING id, action, category, base_points, multiplier, enabled, icon, sort_order, created_at"#,
        )
        .bind(id)
        .bind(u.base_points)
        .bind(u.multiplier)
        .bind(u.enabled)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM point_rules WHERE id = $1").bind(id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }
}

// ─── Badges ─────────────────────────────────────────────────────────────────────

pub struct PostgresBadgeRepository {
    pool: PgPool,
}

impl PostgresBadgeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct BadgeRow {
    id: Uuid,
    emoji: String,
    icon_type: String,
    icon_name: Option<String>,
    icon_url: Option<String>,
    name: String,
    description: String,
    rarity: String,
    condition_type: String,
    condition_value: i32,
    active: bool,
    created_by: Uuid,
    created_at: DateTime<Utc>,
}

impl TryFrom<BadgeRow> for Badge {
    type Error = crate::error::AppError;
    fn try_from(r: BadgeRow) -> Result<Self, Self::Error> {
        Ok(Badge {
            id: r.id,
            emoji: r.emoji,
            icon_type: r.icon_type,
            icon_name: r.icon_name,
            icon_url: r.icon_url,
            name: r.name,
            description: r.description,
            rarity: r.rarity.parse::<Rarity>().map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("corrupt rarity: {e}")))?,
            condition_type: r
                .condition_type
                .parse::<BadgeConditionType>()
                .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("corrupt condition_type: {e}")))?,
            condition_value: r.condition_value,
            active: r.active,
            created_by: r.created_by,
            created_at: r.created_at,
        })
    }
}

const SELECT_BADGE: &str = "SELECT id, emoji, icon_type, icon_name, icon_url, name, description, rarity, condition_type, condition_value, active, created_by, created_at FROM badges";
const BADGE_COLUMNS: &str = "id, emoji, icon_type, icon_name, icon_url, name, description, rarity, condition_type, condition_value, active, created_by, created_at";

#[async_trait]
impl BadgeRepository for PostgresBadgeRepository {
    async fn create(&self, n: NewBadge) -> AppResult<Badge> {
        let row = sqlx::query_as::<_, BadgeRow>(&format!(
            "INSERT INTO badges (emoji, icon_type, icon_name, icon_url, name, description, rarity, condition_type, condition_value, active, created_by) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) RETURNING {BADGE_COLUMNS}",
        ))
        .bind(&n.emoji)
        .bind(&n.icon_type)
        .bind(&n.icon_name)
        .bind(&n.icon_url)
        .bind(&n.name)
        .bind(&n.description)
        .bind(n.rarity.as_str())
        .bind(n.condition_type.as_str())
        .bind(n.condition_value)
        .bind(n.active)
        .bind(n.created_by)
        .fetch_one(&self.pool)
        .await?;
        Badge::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Badge>> {
        let row = sqlx::query_as::<_, BadgeRow>(&format!("{SELECT_BADGE} WHERE id = $1"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(Badge::try_from).transpose()
    }

    async fn list(&self) -> AppResult<Vec<Badge>> {
        let rows = sqlx::query_as::<_, BadgeRow>(&format!("{SELECT_BADGE} ORDER BY created_at DESC")).fetch_all(&self.pool).await?;
        rows.into_iter().map(Badge::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: BadgeUpdate) -> AppResult<Option<Badge>> {
        let result = sqlx::query(
            "UPDATE badges SET emoji=$2, icon_type=$3, icon_name=$4, icon_url=$5, name=$6, description=$7, rarity=$8, condition_type=$9, condition_value=$10, active=$11 WHERE id=$1",
        )
        .bind(id)
        .bind(&u.emoji)
        .bind(&u.icon_type)
        .bind(&u.icon_name)
        .bind(&u.icon_url)
        .bind(&u.name)
        .bind(&u.description)
        .bind(u.rarity.as_str())
        .bind(u.condition_type.as_str())
        .bind(u.condition_value)
        .bind(u.active)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn set_active(&self, id: Uuid, active: bool) -> AppResult<Option<Badge>> {
        let result = sqlx::query("UPDATE badges SET active = $2 WHERE id = $1").bind(id).bind(active).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM badges WHERE id = $1").bind(id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    async fn earned_count(&self, condition_type: BadgeConditionType, condition_value: i32) -> AppResult<i64> {
        let count: i64 = match condition_type {
            BadgeConditionType::ScoreGte => {
                sqlx::query_scalar("SELECT COUNT(DISTINCT student_id) FROM attempts WHERE status = 'submitted' AND score >= $1")
                    .bind(condition_value)
                    .fetch_one(&self.pool)
                    .await?
            }
            BadgeConditionType::QuestionsGte => {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM (
                        SELECT a.student_id, COUNT(DISTINCT aa.question_id) AS qcount
                        FROM attempts a JOIN attempt_answers aa ON aa.attempt_id = a.id
                        WHERE a.status = 'submitted'
                        GROUP BY a.student_id
                    ) t WHERE t.qcount >= $1"#,
                )
                .bind(condition_value)
                .fetch_one(&self.pool)
                .await?
            }
            BadgeConditionType::TryoutGte => {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM (
                        SELECT student_id, COUNT(*) AS cnt FROM attempts WHERE status = 'submitted' GROUP BY student_id
                    ) t WHERE t.cnt >= $1"#,
                )
                .bind(condition_value)
                .fetch_one(&self.pool)
                .await?
            }
            BadgeConditionType::RankLte => {
                sqlx::query_scalar(
                    r#"WITH best AS (
                        SELECT student_id, MAX(score) AS best_score FROM attempts WHERE status = 'submitted' GROUP BY student_id
                    ), ranked AS (
                        SELECT student_id, RANK() OVER (ORDER BY best_score DESC) AS rnk FROM best
                    )
                    SELECT COUNT(*) FROM ranked WHERE rnk <= $1"#,
                )
                .bind(condition_value as i64)
                .fetch_one(&self.pool)
                .await?
            }
            BadgeConditionType::StreakGte => sqlx::query_scalar(STREAK_COUNT_SQL).bind(condition_value).fetch_one(&self.pool).await?,
        };
        Ok(count)
    }

    async fn list_with_stats(&self) -> AppResult<Vec<BadgeWithStats>> {
        let badges = self.list().await?;
        let mut out = Vec::with_capacity(badges.len());
        for badge in badges {
            let earned_by = self.earned_count(badge.condition_type, badge.condition_value).await?;
            out.push(BadgeWithStats { badge, earned_by });
        }
        Ok(out)
    }

    async fn list_for_student(&self, student_id: Uuid) -> AppResult<Vec<StudentBadgeStatus>> {
        let badges = self.list().await?;
        let m: StudentMetricsRow = sqlx::query_as(STUDENT_METRICS_SQL).bind(student_id).fetch_one(&self.pool).await?;

        Ok(badges
            .into_iter()
            .filter(|b| b.active)
            .map(|badge| {
                let (earned, progress_percent) = evaluate_badge(badge.condition_type, badge.condition_value, &m);
                StudentBadgeStatus { badge, earned, progress_percent }
            })
            .collect())
    }
}

#[derive(sqlx::FromRow)]
struct StudentMetricsRow {
    best_score: i32,
    streak: i64,
    questions_answered: i64,
    tryout_count: i64,
    rank: Option<i64>,
}

/// One student's raw real metrics, all computed in a single round-trip — used to
/// evaluate every active badge's condition against that student without N+1 queries.
const STUDENT_METRICS_SQL: &str = r#"
    WITH my AS (
        SELECT COALESCE(MAX(score), 0) AS best_score, COUNT(*) AS tryout_count
        FROM attempts WHERE student_id = $1 AND status = 'submitted'
    ), q AS (
        SELECT COUNT(DISTINCT aa.question_id) AS qcount
        FROM attempts a JOIN attempt_answers aa ON aa.attempt_id = a.id
        WHERE a.student_id = $1 AND a.status = 'submitted'
    ), streak AS (
        SELECT COALESCE(MAX(run_len), 0) AS streak FROM (
            SELECT COUNT(*) AS run_len FROM (
                SELECT d, d - (ROW_NUMBER() OVER (ORDER BY d))::integer AS grp_key FROM (
                    SELECT DISTINCT submitted_at::date AS d FROM attempts
                    WHERE student_id = $1 AND status = 'submitted' AND submitted_at IS NOT NULL
                ) days
            ) grp GROUP BY grp_key
        ) runs
    ), ranked AS (
        SELECT student_id, RANK() OVER (ORDER BY MAX(score) DESC) AS rnk
        FROM attempts WHERE status = 'submitted' GROUP BY student_id
    )
    SELECT my.best_score AS best_score, streak.streak AS streak, q.qcount AS questions_answered,
           my.tryout_count AS tryout_count, (SELECT rnk FROM ranked WHERE student_id = $1) AS rank
    FROM my, q, streak
"#;

fn pct(value: i64, target: i64) -> i32 {
    if target <= 0 {
        return 100;
    }
    ((value as f64 / target as f64) * 100.0).clamp(0.0, 100.0).round() as i32
}

fn evaluate_badge(condition_type: BadgeConditionType, condition_value: i32, m: &StudentMetricsRow) -> (bool, i32) {
    let target = condition_value as i64;
    match condition_type {
        BadgeConditionType::ScoreGte => (m.best_score as i64 >= target, pct(m.best_score as i64, target)),
        BadgeConditionType::StreakGte => (m.streak >= target, pct(m.streak, target)),
        BadgeConditionType::QuestionsGte => (m.questions_answered >= target, pct(m.questions_answered, target)),
        BadgeConditionType::TryoutGte => (m.tryout_count >= target, pct(m.tryout_count, target)),
        BadgeConditionType::RankLte => match m.rank {
            Some(rnk) if rnk <= target => (true, 100),
            Some(rnk) => (false, pct(target, rnk)),
            None => (false, 0),
        },
    }
}

/// Longest run of consecutive calendar days (anywhere in history) with at least one
/// submitted attempt, per student — the classic "gaps and islands" trick: subtracting a
/// per-student row number (ordered by date) from each date collapses consecutive dates
/// onto the same group key.
const STREAK_COUNT_SQL: &str = r#"
    WITH days AS (
        SELECT DISTINCT student_id, submitted_at::date AS d FROM attempts WHERE status = 'submitted' AND submitted_at IS NOT NULL
    ), grp AS (
        SELECT student_id, d, d - (ROW_NUMBER() OVER (PARTITION BY student_id ORDER BY d))::integer AS grp_key
        FROM days
    ), runs AS (
        SELECT student_id, COUNT(*) AS run_len FROM grp GROUP BY student_id, grp_key
    ), best AS (
        SELECT student_id, MAX(run_len) AS best_run FROM runs GROUP BY student_id
    )
    SELECT COUNT(*) FROM best WHERE best_run >= $1
"#;

// ─── Challenges ─────────────────────────────────────────────────────────────────

pub struct PostgresChallengeRepository {
    pool: PgPool,
}

impl PostgresChallengeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ChallengeRow {
    id: Uuid,
    name: String,
    challenge_type: String,
    status: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    target_value: i32,
    reward_points: i32,
    reward_badge: Option<String>,
    description: String,
    created_by: Uuid,
    created_at: DateTime<Utc>,
}

impl TryFrom<ChallengeRow> for Challenge {
    type Error = crate::error::AppError;
    fn try_from(r: ChallengeRow) -> Result<Self, Self::Error> {
        Ok(Challenge {
            id: r.id,
            name: r.name,
            challenge_type: r
                .challenge_type
                .parse::<ChallengeType>()
                .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("corrupt challenge_type: {e}")))?,
            status: r.status.parse::<ChallengeStatus>().map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("corrupt status: {e}")))?,
            start_date: r.start_date,
            end_date: r.end_date,
            target_value: r.target_value,
            reward_points: r.reward_points,
            reward_badge: r.reward_badge,
            description: r.description,
            created_by: r.created_by,
            created_at: r.created_at,
        })
    }
}

const SELECT_CHALLENGE: &str =
    "SELECT id, name, challenge_type, status, start_date, end_date, target_value, reward_points, reward_badge, description, created_by, created_at FROM challenges";

#[async_trait]
impl ChallengeRepository for PostgresChallengeRepository {
    async fn create(&self, n: NewChallenge) -> AppResult<Challenge> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO challenges (id, name, challenge_type, start_date, end_date, target_value, reward_points, reward_badge, description, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"#,
        )
        .bind(id)
        .bind(&n.name)
        .bind(n.challenge_type.as_str())
        .bind(n.start_date)
        .bind(n.end_date)
        .bind(n.target_value)
        .bind(n.reward_points)
        .bind(&n.reward_badge)
        .bind(&n.description)
        .bind(n.created_by)
        .execute(&self.pool)
        .await?;
        self.find_by_id(id).await?.ok_or_else(|| crate::error::AppError::Internal(anyhow::anyhow!("challenge vanished right after insert")))
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Challenge>> {
        let row = sqlx::query_as::<_, ChallengeRow>(&format!("{SELECT_CHALLENGE} WHERE id = $1")).bind(id).fetch_optional(&self.pool).await?;
        row.map(Challenge::try_from).transpose()
    }

    async fn list(&self) -> AppResult<Vec<Challenge>> {
        let rows = sqlx::query_as::<_, ChallengeRow>(&format!("{SELECT_CHALLENGE} ORDER BY start_date DESC")).fetch_all(&self.pool).await?;
        rows.into_iter().map(Challenge::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: ChallengeUpdate) -> AppResult<Option<Challenge>> {
        let result = sqlx::query(
            "UPDATE challenges SET name=$2, challenge_type=$3, start_date=$4, end_date=$5, target_value=$6, reward_points=$7, reward_badge=$8, description=$9 WHERE id=$1",
        )
        .bind(id)
        .bind(&u.name)
        .bind(u.challenge_type.as_str())
        .bind(u.start_date)
        .bind(u.end_date)
        .bind(u.target_value)
        .bind(u.reward_points)
        .bind(&u.reward_badge)
        .bind(&u.description)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn set_status(&self, id: Uuid, status: ChallengeStatus) -> AppResult<Option<Challenge>> {
        let result = sqlx::query("UPDATE challenges SET status = $2 WHERE id = $1").bind(id).bind(status.as_str()).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.find_by_id(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM challenges WHERE id = $1").bind(id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    async fn stats_for(&self, challenge: &Challenge) -> AppResult<(i64, i64)> {
        let participants: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT student_id) FROM attempts WHERE status = 'submitted' AND submitted_at::date BETWEEN $1 AND $2",
        )
        .bind(challenge.start_date)
        .bind(challenge.end_date)
        .fetch_one(&self.pool)
        .await?;

        let completions: i64 = match challenge.challenge_type {
            ChallengeType::MostSolved => {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM (
                        SELECT student_id, COUNT(*) AS cnt FROM attempts
                        WHERE status = 'submitted' AND submitted_at::date BETWEEN $1 AND $2
                        GROUP BY student_id
                    ) t WHERE t.cnt >= $3"#,
                )
                .bind(challenge.start_date)
                .bind(challenge.end_date)
                .bind(challenge.target_value)
                .fetch_one(&self.pool)
                .await?
            }
            ChallengeType::HighestScore => {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM (
                        SELECT student_id, MAX(score) AS best FROM attempts
                        WHERE status = 'submitted' AND submitted_at::date BETWEEN $1 AND $2
                        GROUP BY student_id
                    ) t WHERE t.best >= $3"#,
                )
                .bind(challenge.start_date)
                .bind(challenge.end_date)
                .bind(challenge.target_value)
                .fetch_one(&self.pool)
                .await?
            }
            ChallengeType::Speed => {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM (
                        SELECT student_id, MAX(COALESCE(correct_count, 0) + COALESCE(wrong_count, 0)) AS answered FROM attempts
                        WHERE status = 'submitted' AND submitted_at::date BETWEEN $1 AND $2
                        GROUP BY student_id
                    ) t WHERE t.answered >= $3"#,
                )
                .bind(challenge.start_date)
                .bind(challenge.end_date)
                .bind(challenge.target_value)
                .fetch_one(&self.pool)
                .await?
            }
            ChallengeType::LongestStreak => {
                sqlx::query_scalar(
                    r#"WITH days AS (
                        SELECT DISTINCT student_id, submitted_at::date AS d FROM attempts
                        WHERE status = 'submitted' AND submitted_at::date BETWEEN $1 AND $2
                    ), grp AS (
                        SELECT student_id, d, d - (ROW_NUMBER() OVER (PARTITION BY student_id ORDER BY d))::integer AS grp_key
                        FROM days
                    ), runs AS (
                        SELECT student_id, COUNT(*) AS run_len FROM grp GROUP BY student_id, grp_key
                    ), best AS (
                        SELECT student_id, MAX(run_len) AS best_run FROM runs GROUP BY student_id
                    )
                    SELECT COUNT(*) FROM best WHERE best_run >= $3"#,
                )
                .bind(challenge.start_date)
                .bind(challenge.end_date)
                .bind(challenge.target_value)
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok((participants, completions))
    }

    async fn list_with_stats(&self) -> AppResult<Vec<ChallengeWithStats>> {
        let challenges = self.list().await?;
        let mut out = Vec::with_capacity(challenges.len());
        for challenge in challenges {
            let (participants, completions) = self.stats_for(&challenge).await?;
            out.push(ChallengeWithStats { challenge, participants, completions });
        }
        Ok(out)
    }
}

// ─── Leaderboard ────────────────────────────────────────────────────────────────

pub struct PostgresLeaderboardRepository {
    pool: PgPool,
}

impl PostgresLeaderboardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct LeaderboardRow {
    student_id: Uuid,
    student_name: String,
    school_name: Option<String>,
    best_score: i32,
    streak: i64,
}

/// Per-student longest-ever run of consecutive calendar days with a submitted attempt —
/// same "gaps and islands" trick as `STREAK_COUNT_SQL`, but returning the value per
/// student instead of an aggregate count, for joining onto the leaderboard.
const STREAK_PER_STUDENT_SQL: &str = r#"
    SELECT student_id, MAX(run_len) AS streak FROM (
        SELECT student_id, COUNT(*) AS run_len FROM (
            SELECT student_id, d, d - (ROW_NUMBER() OVER (PARTITION BY student_id ORDER BY d))::integer AS grp_key
            FROM (
                SELECT DISTINCT student_id, submitted_at::date AS d FROM attempts
                WHERE status = 'submitted' AND submitted_at IS NOT NULL
            ) days
        ) grp GROUP BY student_id, grp_key
    ) runs GROUP BY student_id
"#;

/// Converts a range filter into two mutually-exclusive optional bind values: a specific
/// calendar date (for "today") or a lower timestamp bound (for "week"). Both `None` means
/// all-time, no filter applied.
fn range_binds(range: LeaderboardRange) -> (Option<chrono::NaiveDate>, Option<DateTime<Utc>>) {
    match range {
        LeaderboardRange::Today => (Some(Utc::now().date_naive()), None),
        LeaderboardRange::Week => (None, Some(Utc::now() - chrono::Duration::days(7))),
        LeaderboardRange::AllTime => (None, None),
    }
}

#[async_trait]
impl LeaderboardRepository for PostgresLeaderboardRepository {
    async fn top_students(&self, params: LeaderboardParams) -> AppResult<Vec<LeaderboardEntry>> {
        let (today, since) = range_binds(params.range);
        let rows = sqlx::query_as::<_, LeaderboardRow>(&format!(
            r#"WITH streaks AS ({STREAK_PER_STUDENT_SQL})
               SELECT u.id AS student_id, u.name AS student_name, s.name AS school_name,
                      MAX(a.score) AS best_score, COALESCE(MAX(st.streak), 0) AS streak
               FROM attempts a
               JOIN users u ON u.id = a.student_id
               LEFT JOIN schools s ON s.id = u.school_id
               LEFT JOIN streaks st ON st.student_id = u.id
               WHERE a.status = 'submitted'
                 AND ($2::uuid IS NULL OR u.school_id = $2)
                 AND ($3::date IS NULL OR a.submitted_at::date = $3)
                 AND ($4::timestamptz IS NULL OR a.submitted_at >= $4)
               GROUP BY u.id, u.name, s.name
               ORDER BY best_score DESC
               LIMIT $1"#
        ))
        .bind(params.limit)
        .bind(params.school_id)
        .bind(today)
        .bind(since)
        .fetch_all(&self.pool)
        .await?;

        Ok(into_ranked_entries(rows))
    }

    async fn top_students_by_subject(&self, params: LeaderboardParams, subject: &str) -> AppResult<Vec<LeaderboardEntry>> {
        let (today, since) = range_binds(params.range);
        let rows = sqlx::query_as::<_, LeaderboardRow>(&format!(
            r#"WITH streaks AS ({STREAK_PER_STUDENT_SQL}),
               acc AS (
                   SELECT a.student_id,
                          ROUND(COUNT(*) FILTER (WHERE aa.is_correct = true)::numeric / NULLIF(COUNT(*), 0) * 100)::int AS accuracy
                   FROM attempt_answers aa
                   JOIN questions q ON q.id = aa.question_id
                   JOIN attempts a ON a.id = aa.attempt_id
                   WHERE aa.is_correct IS NOT NULL
                     AND q.subject = $5
                     AND a.status = 'submitted'
                     AND ($3::date IS NULL OR a.submitted_at::date = $3)
                     AND ($4::timestamptz IS NULL OR a.submitted_at >= $4)
                   GROUP BY a.student_id
               )
               SELECT u.id AS student_id, u.name AS student_name, s.name AS school_name,
                      COALESCE(acc.accuracy, 0) AS best_score, COALESCE(st.streak, 0) AS streak
               FROM acc
               JOIN users u ON u.id = acc.student_id
               LEFT JOIN schools s ON s.id = u.school_id
               LEFT JOIN streaks st ON st.student_id = u.id
               WHERE ($2::uuid IS NULL OR u.school_id = $2)
               ORDER BY best_score DESC
               LIMIT $1"#
        ))
        .bind(params.limit)
        .bind(params.school_id)
        .bind(today)
        .bind(since)
        .bind(subject)
        .fetch_all(&self.pool)
        .await?;

        Ok(into_ranked_entries(rows))
    }
}

fn into_ranked_entries(rows: Vec<LeaderboardRow>) -> Vec<LeaderboardEntry> {
    rows.into_iter()
        .enumerate()
        .map(|(idx, r)| LeaderboardEntry {
            rank: idx as i64 + 1,
            student_id: r.student_id,
            student_name: r.student_name,
            school_name: r.school_name,
            best_score: r.best_score,
            streak: r.streak,
        })
        .collect()
}
