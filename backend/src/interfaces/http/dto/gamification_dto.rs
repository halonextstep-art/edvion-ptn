use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::gamification::{BadgeWithStats, ChallengeWithStats, LeaderboardEntry, PointRule, StudentBadgeStatus};

// ─── Point rules ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct PointRulePayload {
    #[validate(length(min = 1, message = "nama aksi wajib diisi"))]
    pub action: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub base_points: i32,
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default)]
    pub sort_order: i32,
}
fn default_category() -> String { "Umum".to_string() }
fn default_multiplier() -> f64 { 1.0 }
fn default_true() -> bool { true }
fn default_icon() -> String { "✅".to_string() }

#[derive(Debug, Deserialize, Validate)]
pub struct PointRuleUpdatePayload {
    pub base_points: i32,
    pub multiplier: f64,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct PointRuleResponse {
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
impl From<PointRule> for PointRuleResponse {
    fn from(r: PointRule) -> Self {
        Self { id: r.id, action: r.action, category: r.category, base_points: r.base_points, multiplier: r.multiplier, enabled: r.enabled, icon: r.icon, sort_order: r.sort_order, created_at: r.created_at }
    }
}

// ─── Badges ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct BadgePayload {
    #[serde(default = "default_badge_emoji")]
    pub emoji: String,
    /// "preset" (curated lucide icon, named in `icon_name`) or "custom" (uploaded image at
    /// `icon_url`) — see `IconPicker.vue`.
    #[serde(default = "default_icon_type")]
    pub icon_type: String,
    #[serde(default = "default_badge_icon_name")]
    pub icon_name: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[validate(length(min = 1, message = "nama badge wajib diisi"))]
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// "common" | "rare" | "epic" | "legendary"
    pub rarity: String,
    /// "score_gte" | "streak_gte" | "questions_gte" | "tryout_gte" | "rank_lte"
    pub condition_type: String,
    #[serde(default)]
    pub condition_value: i32,
    #[serde(default = "default_true")]
    pub active: bool,
}
fn default_badge_emoji() -> String { "🏆".to_string() }
fn default_icon_type() -> String { "preset".to_string() }
fn default_badge_icon_name() -> Option<String> { Some("award".to_string()) }

impl BadgePayload {
    pub fn into_input(self) -> Result<crate::application::gamification_service::CreateBadgeInput, crate::error::AppError> {
        Ok(crate::application::gamification_service::CreateBadgeInput {
            emoji: self.emoji,
            icon_type: self.icon_type,
            icon_name: self.icon_name,
            icon_url: self.icon_url,
            name: self.name,
            description: self.description,
            rarity: self.rarity.parse().map_err(crate::error::AppError::Validation)?,
            condition_type: self.condition_type.parse().map_err(crate::error::AppError::Validation)?,
            condition_value: self.condition_value,
            active: self.active,
        })
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct ActivePayload {
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct BadgeResponse {
    pub id: Uuid,
    pub emoji: String,
    pub icon_type: String,
    pub icon_name: Option<String>,
    pub icon_url: Option<String>,
    pub name: String,
    pub description: String,
    pub rarity: String,
    pub condition_type: String,
    pub condition_value: i32,
    pub active: bool,
    /// Live count of students who currently satisfy the condition — computed from real
    /// attempt data at request time, never stored or fabricated.
    pub earned_by: i64,
    pub created_at: DateTime<Utc>,
}
impl From<BadgeWithStats> for BadgeResponse {
    fn from(b: BadgeWithStats) -> Self {
        Self {
            id: b.badge.id, emoji: b.badge.emoji, icon_type: b.badge.icon_type, icon_name: b.badge.icon_name,
            icon_url: b.badge.icon_url, name: b.badge.name, description: b.badge.description,
            rarity: b.badge.rarity.as_str().to_string(), condition_type: b.badge.condition_type.as_str().to_string(),
            condition_value: b.badge.condition_value, active: b.badge.active, earned_by: b.earned_by,
            created_at: b.badge.created_at,
        }
    }
}

// ─── Challenges ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct ChallengePayload {
    #[validate(length(min = 1, message = "nama challenge wajib diisi"))]
    pub name: String,
    /// "most_solved" | "highest_score" | "longest_streak" | "speed"
    pub challenge_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[serde(default)]
    pub target_value: i32,
    #[serde(default)]
    pub reward_points: i32,
    #[serde(default)]
    pub reward_badge: Option<String>,
    #[serde(default)]
    pub description: String,
}
impl ChallengePayload {
    pub fn into_input(self) -> Result<crate::application::gamification_service::CreateChallengeInput, crate::error::AppError> {
        Ok(crate::application::gamification_service::CreateChallengeInput {
            name: self.name,
            challenge_type: self.challenge_type.parse().map_err(crate::error::AppError::Validation)?,
            start_date: self.start_date,
            end_date: self.end_date,
            target_value: self.target_value,
            reward_points: self.reward_points,
            reward_badge: self.reward_badge.filter(|s| !s.trim().is_empty()),
            description: self.description,
        })
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChallengeStatusPayload {
    /// "upcoming" | "active" | "ended"
    #[validate(length(min = 1))]
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub id: Uuid,
    pub name: String,
    pub challenge_type: String,
    pub status: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub target_value: i32,
    pub reward_points: i32,
    pub reward_badge: Option<String>,
    pub description: String,
    /// Live-computed from real attempts within [start_date, end_date] — never stored.
    pub participants: i64,
    pub completions: i64,
    pub created_at: DateTime<Utc>,
}
impl From<ChallengeWithStats> for ChallengeResponse {
    fn from(c: ChallengeWithStats) -> Self {
        Self {
            id: c.challenge.id, name: c.challenge.name, challenge_type: c.challenge.challenge_type.as_str().to_string(),
            status: c.challenge.status.as_str().to_string(), start_date: c.challenge.start_date, end_date: c.challenge.end_date,
            target_value: c.challenge.target_value, reward_points: c.challenge.reward_points, reward_badge: c.challenge.reward_badge,
            description: c.challenge.description, participants: c.participants, completions: c.completions,
            created_at: c.challenge.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StudentBadgeStatusResponse {
    pub id: Uuid,
    pub emoji: String,
    pub icon_type: String,
    pub icon_name: Option<String>,
    pub icon_url: Option<String>,
    pub name: String,
    pub description: String,
    pub rarity: String,
    /// Real, computed fresh for this student every request — never stored.
    pub earned: bool,
    pub progress_percent: i32,
}
impl From<StudentBadgeStatus> for StudentBadgeStatusResponse {
    fn from(s: StudentBadgeStatus) -> Self {
        Self {
            id: s.badge.id,
            emoji: s.badge.emoji,
            icon_type: s.badge.icon_type,
            icon_name: s.badge.icon_name,
            icon_url: s.badge.icon_url,
            name: s.badge.name,
            description: s.badge.description,
            rarity: s.badge.rarity.as_str().to_string(),
            earned: s.earned,
            progress_percent: s.progress_percent,
        }
    }
}

// ─── Leaderboard ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct LeaderboardEntryResponse {
    pub rank: i64,
    pub student_id: Uuid,
    pub student_name: String,
    pub school_name: Option<String>,
    /// Best score (national/school scope) or accuracy % (per-subject scope).
    pub best_score: i32,
    pub streak: i64,
}
impl From<LeaderboardEntry> for LeaderboardEntryResponse {
    fn from(e: LeaderboardEntry) -> Self {
        Self {
            rank: e.rank,
            student_id: e.student_id,
            student_name: e.student_name,
            school_name: e.school_name,
            best_score: e.best_score,
            streak: e.streak,
        }
    }
}
