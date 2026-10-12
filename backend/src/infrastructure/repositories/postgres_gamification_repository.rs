//! Postgres implementation of the gamification repositories. `earned_count` and
//! `stats_for` are the load-bearing methods here: instead of ever writing a fabricated
//! "earned by" or "participants" number, they run real SQL against `attempts` /
//! `attempt_answers` at read time — same philosophy as `PostgresEventRepository`'s live
//! `participants` join.
//!
//! Every method opens its own `begin_tx`/`set_config` transaction (see
//! `postgres_user_repository::PostgresUserRepository::begin_tx` doc comment for the
//! pattern this mirrors) since every `ptn.point_rules`/`ptn.badges`/`ptn.challenges` row
//! carries `tenant_id` + RLS. `created_by` (bigint FK to `core.akun`) is resolved back to
//! its `public_id` via a scalar subquery in each SELECT, same as
//! `postgres_event_repository`/`postgres_voucher_repository`.

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::gamification::{
    Badge, BadgeConditionType, BadgeWithStats, Challenge, ChallengeStatus, ChallengeType, ChallengeWithStats,
    LeaderboardEntry, LeaderboardRange, PointRule, Rarity, StudentBadgeStatus,
};
use crate::domain::repository::{
    BadgeRepository, BadgeUpdate, ChallengeRepository, ChallengeUpdate, LeaderboardParams, LeaderboardRepository,
    NewBadge, NewChallenge, NewPointRule, PointRuleRepository, PointRuleUpdate,
};
use crate::domain::tenant::TenantCtx;
use crate::error::AppResult;

async fn begin_tx(pool: &PgPool, ctx: &TenantCtx) -> AppResult<Transaction<'static, Postgres>> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true), set_config('app.is_platform_admin', $2, true)")
        .bind(ctx.tenant_id.to_string())
        .bind(if ctx.is_platform_admin { "true" } else { "false" })
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}

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
    public_id: Uuid,
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
            id: r.public_id,
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

const SELECT_POINT_RULE: &str =
    "SELECT public_id, action, category, base_points, multiplier, enabled, icon, sort_order, created_at FROM ptn.point_rules";

#[async_trait]
impl PointRuleRepository for PostgresPointRuleRepository {
    async fn create(&self, n: NewPointRule, ctx: &TenantCtx) -> AppResult<PointRule> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.point_rules (tenant_id, action, category, base_points, multiplier, enabled, icon, sort_order)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
               RETURNING public_id"#,
        )
        .bind(ctx.tenant_id)
        .bind(&n.action)
        .bind(&n.category)
        .bind(n.base_points)
        .bind(n.multiplier)
        .bind(n.enabled)
        .bind(&n.icon)
        .bind(n.sort_order)
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as::<_, PointRuleRow>(&format!("{SELECT_POINT_RULE} WHERE public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(row.into())
    }

    async fn list(&self, ctx: &TenantCtx) -> AppResult<Vec<PointRule>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, PointRuleRow>(&format!("{SELECT_POINT_RULE} ORDER BY sort_order ASC, created_at ASC"))
            .fetch_all(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update(&self, id: Uuid, u: PointRuleUpdate, ctx: &TenantCtx) -> AppResult<Option<PointRule>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query("UPDATE ptn.point_rules SET base_points = $2, multiplier = $3, enabled = $4 WHERE public_id = $1")
            .bind(id)
            .bind(u.base_points)
            .bind(u.multiplier)
            .bind(u.enabled)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, PointRuleRow>(&format!("{SELECT_POINT_RULE} WHERE public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(Some(row.into()))
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query("DELETE FROM ptn.point_rules WHERE public_id = $1").bind(id).execute(&mut *tx).await?;
        tx.commit().await?;
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
    public_id: Uuid,
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
            id: r.public_id,
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

const SELECT_BADGE: &str = r#"
    SELECT b.public_id, b.emoji, b.icon_type, b.icon_name, b.icon_url, b.name, b.description, b.rarity,
           b.condition_type, b.condition_value, b.active,
           (SELECT public_id FROM core.akun WHERE core.akun.id = b.created_by) AS created_by,
           b.created_at
    FROM ptn.badges b
"#;

#[async_trait]
impl BadgeRepository for PostgresBadgeRepository {
    async fn create(&self, n: NewBadge, ctx: &TenantCtx) -> AppResult<Badge> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let created_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(n.created_by)
            .fetch_one(&mut *tx)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.badges
                (tenant_id, emoji, icon_type, icon_name, icon_url, name, description, rarity, condition_type, condition_value, active, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
               RETURNING public_id"#,
        )
        .bind(ctx.tenant_id)
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
        .bind(created_by_id)
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as::<_, BadgeRow>(&format!("{SELECT_BADGE} WHERE b.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Badge::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<Badge>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let row = sqlx::query_as::<_, BadgeRow>(&format!("{SELECT_BADGE} WHERE b.public_id = $1"))
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        row.map(Badge::try_from).transpose()
    }

    async fn list(&self, ctx: &TenantCtx) -> AppResult<Vec<Badge>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, BadgeRow>(&format!("{SELECT_BADGE} ORDER BY b.created_at DESC")).fetch_all(&mut *tx).await?;
        tx.commit().await?;
        rows.into_iter().map(Badge::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: BadgeUpdate, ctx: &TenantCtx) -> AppResult<Option<Badge>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query(
            r#"UPDATE ptn.badges SET emoji=$2, icon_type=$3, icon_name=$4, icon_url=$5, name=$6, description=$7,
                   rarity=$8, condition_type=$9, condition_value=$10, active=$11
               WHERE public_id=$1"#,
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
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, BadgeRow>(&format!("{SELECT_BADGE} WHERE b.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Badge::try_from(row).map(Some)
    }

    async fn set_active(&self, id: Uuid, active: bool, ctx: &TenantCtx) -> AppResult<Option<Badge>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query("UPDATE ptn.badges SET active = $2 WHERE public_id = $1").bind(id).bind(active).execute(&mut *tx).await?;
        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, BadgeRow>(&format!("{SELECT_BADGE} WHERE b.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Badge::try_from(row).map(Some)
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query("DELETE FROM ptn.badges WHERE public_id = $1").bind(id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn earned_count(&self, condition_type: BadgeConditionType, condition_value: i32, ctx: &TenantCtx) -> AppResult<i64> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let count: i64 = match condition_type {
            BadgeConditionType::ScoreGte => {
                sqlx::query_scalar("SELECT COUNT(DISTINCT student_id) FROM ptn.attempts WHERE status = 'submitted' AND score >= $1")
                    .bind(condition_value)
                    .fetch_one(&mut *tx)
                    .await?
            }
            BadgeConditionType::QuestionsGte => {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM (
                        SELECT a.student_id, COUNT(DISTINCT aa.question_id) AS qcount
                        FROM ptn.attempts a JOIN ptn.attempt_answers aa ON aa.attempt_id = a.id
                        WHERE a.status = 'submitted'
                        GROUP BY a.student_id
                    ) t WHERE t.qcount >= $1"#,
                )
                .bind(condition_value)
                .fetch_one(&mut *tx)
                .await?
            }
            BadgeConditionType::TryoutGte => {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM (
                        SELECT student_id, COUNT(*) AS cnt FROM ptn.attempts WHERE status = 'submitted' GROUP BY student_id
                    ) t WHERE t.cnt >= $1"#,
                )
                .bind(condition_value)
                .fetch_one(&mut *tx)
                .await?
            }
            BadgeConditionType::RankLte => {
                sqlx::query_scalar(
                    r#"WITH best AS (
                        SELECT student_id, MAX(score) AS best_score FROM ptn.attempts WHERE status = 'submitted' GROUP BY student_id
                    ), ranked AS (
                        SELECT student_id, RANK() OVER (ORDER BY best_score DESC) AS rnk FROM best
                    )
                    SELECT COUNT(*) FROM ranked WHERE rnk <= $1"#,
                )
                .bind(condition_value as i64)
                .fetch_one(&mut *tx)
                .await?
            }
            BadgeConditionType::StreakGte => sqlx::query_scalar(STREAK_COUNT_SQL).bind(condition_value).fetch_one(&mut *tx).await?,
        };
        tx.commit().await?;
        Ok(count)
    }

    async fn list_with_stats(&self, ctx: &TenantCtx) -> AppResult<Vec<BadgeWithStats>> {
        let badges = self.list(ctx).await?;
        let mut out = Vec::with_capacity(badges.len());
        for badge in badges {
            let earned_by = self.earned_count(badge.condition_type, badge.condition_value, ctx).await?;
            out.push(BadgeWithStats { badge, earned_by });
        }
        Ok(out)
    }

    async fn list_for_student(&self, student_id: Uuid, ctx: &TenantCtx) -> AppResult<Vec<StudentBadgeStatus>> {
        let badges = self.list(ctx).await?;
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let student_pk: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(student_id)
            .fetch_one(&mut *tx)
            .await?;
        let m: StudentMetricsRow = sqlx::query_as(STUDENT_METRICS_SQL).bind(student_pk).fetch_one(&mut *tx).await?;
        tx.commit().await?;

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
/// `$1` is the student's bigint `core.akun.id`, resolved from `public_id` by the caller.
const STUDENT_METRICS_SQL: &str = r#"
    WITH my AS (
        SELECT COALESCE(MAX(score), 0) AS best_score, COUNT(*) AS tryout_count
        FROM ptn.attempts WHERE student_id = $1 AND status = 'submitted'
    ), q AS (
        SELECT COUNT(DISTINCT aa.question_id) AS qcount
        FROM ptn.attempts a JOIN ptn.attempt_answers aa ON aa.attempt_id = a.id
        WHERE a.student_id = $1 AND a.status = 'submitted'
    ), streak AS (
        SELECT COALESCE(MAX(run_len), 0) AS streak FROM (
            SELECT COUNT(*) AS run_len FROM (
                SELECT d, d - (ROW_NUMBER() OVER (ORDER BY d))::integer AS grp_key FROM (
                    SELECT DISTINCT submitted_at::date AS d FROM ptn.attempts
                    WHERE student_id = $1 AND status = 'submitted' AND submitted_at IS NOT NULL
                ) days
            ) grp GROUP BY grp_key
        ) runs
    ), ranked AS (
        SELECT student_id, RANK() OVER (ORDER BY MAX(score) DESC) AS rnk
        FROM ptn.attempts WHERE status = 'submitted' GROUP BY student_id
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
        SELECT DISTINCT student_id, submitted_at::date AS d FROM ptn.attempts WHERE status = 'submitted' AND submitted_at IS NOT NULL
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
    public_id: Uuid,
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
            id: r.public_id,
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

const SELECT_CHALLENGE: &str = r#"
    SELECT c.public_id, c.name, c.challenge_type, c.status, c.start_date, c.end_date, c.target_value,
           c.reward_points, c.reward_badge, c.description,
           (SELECT public_id FROM core.akun WHERE core.akun.id = c.created_by) AS created_by,
           c.created_at
    FROM ptn.challenges c
"#;

#[async_trait]
impl ChallengeRepository for PostgresChallengeRepository {
    async fn create(&self, n: NewChallenge, ctx: &TenantCtx) -> AppResult<Challenge> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let created_by_id: i64 = sqlx::query_scalar("SELECT id FROM core.akun WHERE public_id = $1")
            .bind(n.created_by)
            .fetch_one(&mut *tx)
            .await?;
        let public_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO ptn.challenges
                (tenant_id, name, challenge_type, start_date, end_date, target_value, reward_points, reward_badge, description, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
               RETURNING public_id"#,
        )
        .bind(ctx.tenant_id)
        .bind(&n.name)
        .bind(n.challenge_type.as_str())
        .bind(n.start_date)
        .bind(n.end_date)
        .bind(n.target_value)
        .bind(n.reward_points)
        .bind(&n.reward_badge)
        .bind(&n.description)
        .bind(created_by_id)
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query_as::<_, ChallengeRow>(&format!("{SELECT_CHALLENGE} WHERE c.public_id = $1"))
            .bind(public_id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Challenge::try_from(row)
    }

    async fn find_by_id(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<Option<Challenge>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let row = sqlx::query_as::<_, ChallengeRow>(&format!("{SELECT_CHALLENGE} WHERE c.public_id = $1")).bind(id).fetch_optional(&mut *tx).await?;
        tx.commit().await?;
        row.map(Challenge::try_from).transpose()
    }

    async fn list(&self, ctx: &TenantCtx) -> AppResult<Vec<Challenge>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let rows = sqlx::query_as::<_, ChallengeRow>(&format!("{SELECT_CHALLENGE} ORDER BY c.start_date DESC")).fetch_all(&mut *tx).await?;
        tx.commit().await?;
        rows.into_iter().map(Challenge::try_from).collect()
    }

    async fn update(&self, id: Uuid, u: ChallengeUpdate, ctx: &TenantCtx) -> AppResult<Option<Challenge>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query(
            r#"UPDATE ptn.challenges SET name=$2, challenge_type=$3, start_date=$4, end_date=$5, target_value=$6,
                   reward_points=$7, reward_badge=$8, description=$9
               WHERE public_id=$1"#,
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
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, ChallengeRow>(&format!("{SELECT_CHALLENGE} WHERE c.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Challenge::try_from(row).map(Some)
    }

    async fn set_status(&self, id: Uuid, status: ChallengeStatus, ctx: &TenantCtx) -> AppResult<Option<Challenge>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result =
            sqlx::query("UPDATE ptn.challenges SET status = $2 WHERE public_id = $1").bind(id).bind(status.as_str()).execute(&mut *tx).await?;
        if result.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(None);
        }
        let row = sqlx::query_as::<_, ChallengeRow>(&format!("{SELECT_CHALLENGE} WHERE c.public_id = $1"))
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Challenge::try_from(row).map(Some)
    }

    async fn delete(&self, id: Uuid, ctx: &TenantCtx) -> AppResult<bool> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let result = sqlx::query("DELETE FROM ptn.challenges WHERE public_id = $1").bind(id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    async fn stats_for(&self, challenge: &Challenge, ctx: &TenantCtx) -> AppResult<(i64, i64)> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let participants: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT student_id) FROM ptn.attempts WHERE status = 'submitted' AND submitted_at::date BETWEEN $1 AND $2",
        )
        .bind(challenge.start_date)
        .bind(challenge.end_date)
        .fetch_one(&mut *tx)
        .await?;

        let completions: i64 = match challenge.challenge_type {
            ChallengeType::MostSolved => {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM (
                        SELECT student_id, COUNT(*) AS cnt FROM ptn.attempts
                        WHERE status = 'submitted' AND submitted_at::date BETWEEN $1 AND $2
                        GROUP BY student_id
                    ) t WHERE t.cnt >= $3"#,
                )
                .bind(challenge.start_date)
                .bind(challenge.end_date)
                .bind(challenge.target_value)
                .fetch_one(&mut *tx)
                .await?
            }
            ChallengeType::HighestScore => {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM (
                        SELECT student_id, MAX(score) AS best FROM ptn.attempts
                        WHERE status = 'submitted' AND submitted_at::date BETWEEN $1 AND $2
                        GROUP BY student_id
                    ) t WHERE t.best >= $3"#,
                )
                .bind(challenge.start_date)
                .bind(challenge.end_date)
                .bind(challenge.target_value)
                .fetch_one(&mut *tx)
                .await?
            }
            ChallengeType::Speed => {
                sqlx::query_scalar(
                    r#"SELECT COUNT(*) FROM (
                        SELECT student_id, MAX(COALESCE(correct_count, 0) + COALESCE(wrong_count, 0)) AS answered FROM ptn.attempts
                        WHERE status = 'submitted' AND submitted_at::date BETWEEN $1 AND $2
                        GROUP BY student_id
                    ) t WHERE t.answered >= $3"#,
                )
                .bind(challenge.start_date)
                .bind(challenge.end_date)
                .bind(challenge.target_value)
                .fetch_one(&mut *tx)
                .await?
            }
            ChallengeType::LongestStreak => {
                sqlx::query_scalar(
                    r#"WITH days AS (
                        SELECT DISTINCT student_id, submitted_at::date AS d FROM ptn.attempts
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
                .fetch_one(&mut *tx)
                .await?
            }
        };

        tx.commit().await?;
        Ok((participants, completions))
    }

    async fn list_with_stats(&self, ctx: &TenantCtx) -> AppResult<Vec<ChallengeWithStats>> {
        let challenges = self.list(ctx).await?;
        let mut out = Vec::with_capacity(challenges.len());
        for challenge in challenges {
            let (participants, completions) = self.stats_for(&challenge, ctx).await?;
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
                SELECT DISTINCT student_id, submitted_at::date AS d FROM ptn.attempts
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
    async fn top_students(&self, params: LeaderboardParams, ctx: &TenantCtx) -> AppResult<Vec<LeaderboardEntry>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let (today, since) = range_binds(params.range);
        let rows = sqlx::query_as::<_, LeaderboardRow>(&format!(
            r#"WITH streaks AS ({STREAK_PER_STUDENT_SQL})
               SELECT a.public_id AS student_id, a.full_name AS student_name, t.name AS school_name,
                      MAX(att.score) AS best_score, COALESCE(MAX(st.streak), 0) AS streak
               FROM ptn.attempts att
               JOIN core.akun a ON a.id = att.student_id
               LEFT JOIN core.tenant t ON t.id = a.tenant_id AND a.tenant_id <> 0
               LEFT JOIN streaks st ON st.student_id = att.student_id
               WHERE att.status = 'submitted'
                 AND ($2::uuid IS NULL OR a.tenant_id = (SELECT id FROM core.tenant WHERE public_id = $2))
                 AND ($3::date IS NULL OR att.submitted_at::date = $3)
                 AND ($4::timestamptz IS NULL OR att.submitted_at >= $4)
               GROUP BY a.public_id, a.full_name, t.name
               ORDER BY best_score DESC
               LIMIT $1"#
        ))
        .bind(params.limit)
        .bind(params.school_id)
        .bind(today)
        .bind(since)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(into_ranked_entries(rows))
    }

    async fn top_students_by_subject(&self, params: LeaderboardParams, subject: &str, ctx: &TenantCtx) -> AppResult<Vec<LeaderboardEntry>> {
        let mut tx = begin_tx(&self.pool, ctx).await?;
        let (today, since) = range_binds(params.range);
        let rows = sqlx::query_as::<_, LeaderboardRow>(&format!(
            r#"WITH streaks AS ({STREAK_PER_STUDENT_SQL}),
               acc AS (
                   SELECT att.student_id,
                          ROUND(COUNT(*) FILTER (WHERE aa.is_correct = true)::numeric / NULLIF(COUNT(*), 0) * 100)::int AS accuracy
                   FROM ptn.attempt_answers aa
                   JOIN ptn.questions q ON q.id = aa.question_id
                   JOIN ptn.attempts att ON att.id = aa.attempt_id
                   WHERE aa.is_correct IS NOT NULL
                     AND q.subject = $5
                     AND att.status = 'submitted'
                     AND ($3::date IS NULL OR att.submitted_at::date = $3)
                     AND ($4::timestamptz IS NULL OR att.submitted_at >= $4)
                   GROUP BY att.student_id
               )
               SELECT a.public_id AS student_id, a.full_name AS student_name, t.name AS school_name,
                      COALESCE(acc.accuracy, 0) AS best_score, COALESCE(st.streak, 0) AS streak
               FROM acc
               JOIN core.akun a ON a.id = acc.student_id
               LEFT JOIN core.tenant t ON t.id = a.tenant_id AND a.tenant_id <> 0
               LEFT JOIN streaks st ON st.student_id = acc.student_id
               WHERE ($2::uuid IS NULL OR a.tenant_id = (SELECT id FROM core.tenant WHERE public_id = $2))
               ORDER BY best_score DESC
               LIMIT $1"#
        ))
        .bind(params.limit)
        .bind(params.school_id)
        .bind(today)
        .bind(since)
        .bind(subject)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;

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
