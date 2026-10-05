//! Gamification entities (Manajemen Gamifikasi) — point rules, badges, and challenges,
//! mirroring the reference `AdminGamification`. `Badge.earned_by` and
//! `Challenge.{participants,completions}` are never stored on these structs as data —
//! they are supplied separately by the repository at read time, computed live from the
//! real `attempts` table, so the numbers can never be fabricated or drift stale.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Point rules ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointRule {
    pub id: Uuid,
    pub action: String,
    pub category: String,
    pub base_points: i32,
    pub multiplier: f64,
    pub enabled: bool,
    pub icon: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

// ─── Badges ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Rarity::Common => "common",
            Rarity::Rare => "rare",
            Rarity::Epic => "epic",
            Rarity::Legendary => "legendary",
        }
    }
}

impl std::str::FromStr for Rarity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "common" => Ok(Rarity::Common),
            "rare" => Ok(Rarity::Rare),
            "epic" => Ok(Rarity::Epic),
            "legendary" => Ok(Rarity::Legendary),
            other => Err(format!("unknown rarity: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BadgeConditionType {
    ScoreGte,
    StreakGte,
    QuestionsGte,
    TryoutGte,
    RankLte,
}

impl BadgeConditionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BadgeConditionType::ScoreGte => "score_gte",
            BadgeConditionType::StreakGte => "streak_gte",
            BadgeConditionType::QuestionsGte => "questions_gte",
            BadgeConditionType::TryoutGte => "tryout_gte",
            BadgeConditionType::RankLte => "rank_lte",
        }
    }
}

impl std::str::FromStr for BadgeConditionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "score_gte" => Ok(BadgeConditionType::ScoreGte),
            "streak_gte" => Ok(BadgeConditionType::StreakGte),
            "questions_gte" => Ok(BadgeConditionType::QuestionsGte),
            "tryout_gte" => Ok(BadgeConditionType::TryoutGte),
            "rank_lte" => Ok(BadgeConditionType::RankLte),
            other => Err(format!("unknown badge condition_type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub id: Uuid,
    /// Legacy raw-emoji field — kept for backward compatibility (harmless, unread by any
    /// current display) but superseded by `icon_type`/`icon_name`/`icon_url` below. See
    /// `IconPicker.vue` for the picker that manages the new fields.
    pub emoji: String,
    /// "preset" (a curated lucide-vue-next icon, named in `icon_name`) or "custom" (an
    /// admin-uploaded image, at `icon_url`).
    pub icon_type: String,
    pub icon_name: Option<String>,
    pub icon_url: Option<String>,
    pub name: String,
    pub description: String,
    pub rarity: Rarity,
    pub condition_type: BadgeConditionType,
    pub condition_value: i32,
    pub active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// A badge paired with its live-computed earned count (never stored).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeWithStats {
    pub badge: Badge,
    pub earned_by: i64,
}

/// A badge paired with one specific student's live-computed progress toward it — backs
/// the student portal's "Achievements" card. `earned`/`progress_percent` are computed
/// fresh from that student's real `attempts`/`attempt_answers` every read, never stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentBadgeStatus {
    pub badge: Badge,
    pub earned: bool,
    /// 0-100, how close the student is to earning this badge (100 once earned).
    pub progress_percent: i32,
}

// ─── Challenges ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeType {
    MostSolved,
    HighestScore,
    LongestStreak,
    Speed,
}

impl ChallengeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChallengeType::MostSolved => "most_solved",
            ChallengeType::HighestScore => "highest_score",
            ChallengeType::LongestStreak => "longest_streak",
            ChallengeType::Speed => "speed",
        }
    }
}

impl std::str::FromStr for ChallengeType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "most_solved" => Ok(ChallengeType::MostSolved),
            "highest_score" => Ok(ChallengeType::HighestScore),
            "longest_streak" => Ok(ChallengeType::LongestStreak),
            "speed" => Ok(ChallengeType::Speed),
            other => Err(format!("unknown challenge_type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeStatus {
    Upcoming,
    Active,
    Ended,
}

impl ChallengeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChallengeStatus::Upcoming => "upcoming",
            ChallengeStatus::Active => "active",
            ChallengeStatus::Ended => "ended",
        }
    }
}

impl std::str::FromStr for ChallengeStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "upcoming" => Ok(ChallengeStatus::Upcoming),
            "active" => Ok(ChallengeStatus::Active),
            "ended" => Ok(ChallengeStatus::Ended),
            other => Err(format!("unknown challenge status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: Uuid,
    pub name: String,
    pub challenge_type: ChallengeType,
    pub status: ChallengeStatus,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub target_value: i32,
    pub reward_points: i32,
    pub reward_badge: Option<String>,
    pub description: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// A challenge paired with its live-computed participants/completions (never stored).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeWithStats {
    pub challenge: Challenge,
    pub participants: i64,
    pub completions: i64,
}

// ─── Leaderboard (read-only, always live) ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub student_id: Uuid,
    pub student_name: String,
    pub school_name: Option<String>,
    /// Best single-attempt score (national/school views) or accuracy percentage (per-subject
    /// view) the student has achieved — real, computed live from `attempts`/`attempt_answers`.
    pub best_score: i32,
    /// Longest-ever run of consecutive calendar days with at least one submitted attempt
    /// (real, "gaps and islands" over `attempts.submitted_at` — never a stored/fake counter).
    pub streak: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardScope {
    National,
    School,
}

impl LeaderboardScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            LeaderboardScope::National => "national",
            LeaderboardScope::School => "school",
        }
    }
}

impl std::str::FromStr for LeaderboardScope {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "national" => Ok(LeaderboardScope::National),
            "school" => Ok(LeaderboardScope::School),
            other => Err(format!("unknown leaderboard scope: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardRange {
    Today,
    Week,
    AllTime,
}

impl LeaderboardRange {
    pub fn as_str(&self) -> &'static str {
        match self {
            LeaderboardRange::Today => "today",
            LeaderboardRange::Week => "week",
            LeaderboardRange::AllTime => "all_time",
        }
    }
}

impl std::str::FromStr for LeaderboardRange {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "today" => Ok(LeaderboardRange::Today),
            "week" => Ok(LeaderboardRange::Week),
            "all_time" => Ok(LeaderboardRange::AllTime),
            other => Err(format!("unknown leaderboard range: {other}")),
        }
    }
}
