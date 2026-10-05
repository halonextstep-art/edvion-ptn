//! Gamification (Manajemen Gamifikasi) use-cases: point rules, badges, challenges, and
//! the national leaderboard. Admin-only for writes; all "how many/who" figures
//! (`earned_by`, `participants`, `completions`, leaderboard ranks) come straight from the
//! repository's live computation against real `attempts` data — this service never
//! invents or backfills a number itself.

use std::sync::Arc;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::domain::gamification::{
    BadgeConditionType, BadgeWithStats, ChallengeStatus, ChallengeType, ChallengeWithStats, LeaderboardEntry,
    LeaderboardRange, LeaderboardScope, PointRule, Rarity, StudentBadgeStatus,
};
use crate::domain::repository::{
    BadgeRepository, BadgeUpdate, ChallengeRepository, ChallengeUpdate, LeaderboardParams, LeaderboardRepository,
    NewBadge, NewChallenge, NewPointRule, PointRuleRepository, PointRuleUpdate, UserRepository,
};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

pub struct CreatePointRuleInput {
    pub action: String,
    pub category: String,
    pub base_points: i32,
    pub multiplier: f64,
    pub enabled: bool,
    pub icon: String,
    pub sort_order: i32,
}

pub struct CreateBadgeInput {
    pub emoji: String,
    pub icon_type: String,
    pub icon_name: Option<String>,
    pub icon_url: Option<String>,
    pub name: String,
    pub description: String,
    pub rarity: Rarity,
    pub condition_type: BadgeConditionType,
    pub condition_value: i32,
    pub active: bool,
}

pub struct CreateChallengeInput {
    pub name: String,
    pub challenge_type: ChallengeType,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub target_value: i32,
    pub reward_points: i32,
    pub reward_badge: Option<String>,
    pub description: String,
}

pub struct GamificationService {
    point_rules: Arc<dyn PointRuleRepository>,
    badges: Arc<dyn BadgeRepository>,
    challenges: Arc<dyn ChallengeRepository>,
    leaderboard: Arc<dyn LeaderboardRepository>,
    users: Arc<dyn UserRepository>,
}

impl GamificationService {
    pub fn new(
        point_rules: Arc<dyn PointRuleRepository>,
        badges: Arc<dyn BadgeRepository>,
        challenges: Arc<dyn ChallengeRepository>,
        leaderboard: Arc<dyn LeaderboardRepository>,
        users: Arc<dyn UserRepository>,
    ) -> Self {
        Self { point_rules, badges, challenges, leaderboard, users }
    }

    /// Resolves the acting user's own `school_id` — works for both Student and School
    /// actors, since both roles carry a real `school_id` on their `users` row. Used to
    /// scope the "Sekolahku" leaderboard view without trusting a client-supplied id.
    async fn resolve_actor_school_id(&self, actor: &AuthUser) -> AppResult<Uuid> {
        let me = self.users.find_by_id(actor.user_id).await?.ok_or_else(|| AppError::Unauthorized("akun tidak ditemukan".to_string()))?;
        me.school_id.ok_or_else(|| AppError::Validation("akun ini belum terhubung ke data sekolah".to_string()))
    }

    // ─── Point rules ──────────────────────────────────────────────────────────
    pub async fn create_point_rule(&self, actor: &AuthUser, input: CreatePointRuleInput) -> AppResult<PointRule> {
        actor.require_role(&[Role::Admin])?;
        if input.action.trim().is_empty() {
            return Err(AppError::Validation("nama aksi wajib diisi".to_string()));
        }
        self.point_rules
            .create(NewPointRule {
                action: input.action,
                category: input.category,
                base_points: input.base_points,
                multiplier: input.multiplier,
                enabled: input.enabled,
                icon: input.icon,
                sort_order: input.sort_order,
            })
            .await
    }

    pub async fn list_point_rules(&self, actor: &AuthUser) -> AppResult<Vec<PointRule>> {
        actor.require_role(&[Role::Admin])?;
        self.point_rules.list().await
    }

    pub async fn update_point_rule(&self, actor: &AuthUser, id: Uuid, base_points: i32, multiplier: f64, enabled: bool) -> AppResult<PointRule> {
        actor.require_role(&[Role::Admin])?;
        self.point_rules
            .update(id, PointRuleUpdate { base_points, multiplier, enabled })
            .await?
            .ok_or_else(|| AppError::NotFound(format!("point rule {id} not found")))
    }

    pub async fn delete_point_rule(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        if !self.point_rules.delete(id).await? {
            return Err(AppError::NotFound(format!("point rule {id} not found")));
        }
        Ok(())
    }

    // ─── Badges ───────────────────────────────────────────────────────────────
    pub async fn create_badge(&self, actor: &AuthUser, input: CreateBadgeInput) -> AppResult<BadgeWithStats> {
        actor.require_role(&[Role::Admin])?;
        if input.name.trim().is_empty() {
            return Err(AppError::Validation("nama badge wajib diisi".to_string()));
        }
        let badge = self
            .badges
            .create(NewBadge {
                emoji: input.emoji,
                icon_type: input.icon_type,
                icon_name: input.icon_name,
                icon_url: input.icon_url,
                name: input.name,
                description: input.description,
                rarity: input.rarity,
                condition_type: input.condition_type,
                condition_value: input.condition_value,
                active: input.active,
                created_by: actor.user_id,
            })
            .await?;
        let earned_by = self.badges.earned_count(badge.condition_type, badge.condition_value).await?;
        Ok(BadgeWithStats { badge, earned_by })
    }

    pub async fn list_badges(&self, actor: &AuthUser) -> AppResult<Vec<BadgeWithStats>> {
        actor.require_role(&[Role::Admin])?;
        self.badges.list_with_stats().await
    }

    pub async fn update_badge(&self, actor: &AuthUser, id: Uuid, input: CreateBadgeInput) -> AppResult<BadgeWithStats> {
        actor.require_role(&[Role::Admin])?;
        if input.name.trim().is_empty() {
            return Err(AppError::Validation("nama badge wajib diisi".to_string()));
        }
        let badge = self
            .badges
            .update(
                id,
                BadgeUpdate {
                    emoji: input.emoji,
                    icon_type: input.icon_type,
                    icon_name: input.icon_name,
                    icon_url: input.icon_url,
                    name: input.name,
                    description: input.description,
                    rarity: input.rarity,
                    condition_type: input.condition_type,
                    condition_value: input.condition_value,
                    active: input.active,
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("badge {id} not found")))?;
        let earned_by = self.badges.earned_count(badge.condition_type, badge.condition_value).await?;
        Ok(BadgeWithStats { badge, earned_by })
    }

    pub async fn set_badge_active(&self, actor: &AuthUser, id: Uuid, active: bool) -> AppResult<BadgeWithStats> {
        actor.require_role(&[Role::Admin])?;
        let badge = self.badges.set_active(id, active).await?.ok_or_else(|| AppError::NotFound(format!("badge {id} not found")))?;
        let earned_by = self.badges.earned_count(badge.condition_type, badge.condition_value).await?;
        Ok(BadgeWithStats { badge, earned_by })
    }

    pub async fn delete_badge(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        if !self.badges.delete(id).await? {
            return Err(AppError::NotFound(format!("badge {id} not found")));
        }
        Ok(())
    }

    /// Student portal "Achievements" card — the acting student's own real progress
    /// toward every active badge.
    pub async fn my_badges(&self, actor: &AuthUser) -> AppResult<Vec<StudentBadgeStatus>> {
        actor.require_role(&[Role::Student])?;
        self.badges.list_for_student(actor.user_id).await
    }

    // ─── Challenges ───────────────────────────────────────────────────────────
    pub async fn create_challenge(&self, actor: &AuthUser, input: CreateChallengeInput) -> AppResult<ChallengeWithStats> {
        actor.require_role(&[Role::Admin])?;
        if input.name.trim().is_empty() {
            return Err(AppError::Validation("nama challenge wajib diisi".to_string()));
        }
        if input.end_date < input.start_date {
            return Err(AppError::Validation("tanggal selesai tidak boleh sebelum tanggal mulai".to_string()));
        }
        let challenge = self
            .challenges
            .create(NewChallenge {
                name: input.name,
                challenge_type: input.challenge_type,
                start_date: input.start_date,
                end_date: input.end_date,
                target_value: input.target_value,
                reward_points: input.reward_points,
                reward_badge: input.reward_badge,
                description: input.description,
                created_by: actor.user_id,
            })
            .await?;
        let (participants, completions) = self.challenges.stats_for(&challenge).await?;
        Ok(ChallengeWithStats { challenge, participants, completions })
    }

    pub async fn list_challenges(&self, actor: &AuthUser) -> AppResult<Vec<ChallengeWithStats>> {
        actor.require_role(&[Role::Admin])?;
        self.challenges.list_with_stats().await
    }

    pub async fn update_challenge(&self, actor: &AuthUser, id: Uuid, input: CreateChallengeInput) -> AppResult<ChallengeWithStats> {
        actor.require_role(&[Role::Admin])?;
        if input.name.trim().is_empty() {
            return Err(AppError::Validation("nama challenge wajib diisi".to_string()));
        }
        let challenge = self
            .challenges
            .update(
                id,
                ChallengeUpdate {
                    name: input.name,
                    challenge_type: input.challenge_type,
                    start_date: input.start_date,
                    end_date: input.end_date,
                    target_value: input.target_value,
                    reward_points: input.reward_points,
                    reward_badge: input.reward_badge,
                    description: input.description,
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("challenge {id} not found")))?;
        let (participants, completions) = self.challenges.stats_for(&challenge).await?;
        Ok(ChallengeWithStats { challenge, participants, completions })
    }

    pub async fn set_challenge_status(&self, actor: &AuthUser, id: Uuid, status: ChallengeStatus) -> AppResult<ChallengeWithStats> {
        actor.require_role(&[Role::Admin])?;
        let challenge = self.challenges.set_status(id, status).await?.ok_or_else(|| AppError::NotFound(format!("challenge {id} not found")))?;
        let (participants, completions) = self.challenges.stats_for(&challenge).await?;
        Ok(ChallengeWithStats { challenge, participants, completions })
    }

    pub async fn delete_challenge(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin])?;
        if !self.challenges.delete(id).await? {
            return Err(AppError::NotFound(format!("challenge {id} not found")));
        }
        Ok(())
    }

    // ─── Leaderboard ──────────────────────────────────────────────────────────
    /// Readable by any authenticated role — admin uses it for the Gamifikasi tab, students
    /// and school PICs use it for the student-portal Leaderboard tab. Always computed live
    /// from `attempts`, never a stored/fake ranking (see `LeaderboardRepository`).
    pub async fn leaderboard(
        &self,
        actor: &AuthUser,
        limit: i64,
        scope: LeaderboardScope,
        range: LeaderboardRange,
        subject: Option<String>,
    ) -> AppResult<Vec<LeaderboardEntry>> {
        actor.require_role(&[Role::Admin, Role::Student, Role::School])?;
        let school_id = match scope {
            LeaderboardScope::National => None,
            LeaderboardScope::School => Some(self.resolve_actor_school_id(actor).await?),
        };
        let params = LeaderboardParams { limit: limit.clamp(1, 100), school_id, range };
        match subject.filter(|s| !s.trim().is_empty()) {
            Some(subject) => self.leaderboard.top_students_by_subject(params, &subject).await,
            None => self.leaderboard.top_students(params).await,
        }
    }
}
