import type { ApiClient } from './ApiClient'
import type {
  PointRuleItem, PointRulePayload, BadgeItem, BadgePayload, ChallengeItem, ChallengePayload,
  ChallengeStatus, LeaderboardEntryItem, LeaderboardQueryParams, StudentBadgeStatusItem,
} from '~/types'

export class GamificationService {
  constructor(private api: ApiClient) {}

  // Point rules
  listPointRules() {
    return this.api.get<PointRuleItem[]>('/gamification/point-rules')
  }
  createPointRule(payload: PointRulePayload) {
    return this.api.post<PointRuleItem>('/gamification/point-rules', payload)
  }
  updatePointRule(id: string, payload: { base_points: number; multiplier: number; enabled: boolean }) {
    return this.api.put<PointRuleItem>(`/gamification/point-rules/${id}`, payload)
  }
  removePointRule(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/gamification/point-rules/${id}`)
  }

  // Badges
  listBadges() {
    return this.api.get<BadgeItem[]>('/gamification/badges')
  }
  createBadge(payload: BadgePayload) {
    return this.api.post<BadgeItem>('/gamification/badges', payload)
  }
  updateBadge(id: string, payload: BadgePayload) {
    return this.api.put<BadgeItem>(`/gamification/badges/${id}`, payload)
  }
  setBadgeActive(id: string, active: boolean) {
    return this.api.patch<BadgeItem>(`/gamification/badges/${id}/active`, { active })
  }
  removeBadge(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/gamification/badges/${id}`)
  }

  // Challenges
  listChallenges() {
    return this.api.get<ChallengeItem[]>('/gamification/challenges')
  }
  createChallenge(payload: ChallengePayload) {
    return this.api.post<ChallengeItem>('/gamification/challenges', payload)
  }
  updateChallenge(id: string, payload: ChallengePayload) {
    return this.api.put<ChallengeItem>(`/gamification/challenges/${id}`, payload)
  }
  setChallengeStatus(id: string, status: ChallengeStatus) {
    return this.api.patch<ChallengeItem>(`/gamification/challenges/${id}/status`, { status })
  }
  removeChallenge(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/gamification/challenges/${id}`)
  }

  // Leaderboard
  leaderboard(params: LeaderboardQueryParams = {}) {
    return this.api.get<LeaderboardEntryItem[]>('/gamification/leaderboard', { limit: 20, ...params })
  }

  /** Student portal "Achievements" card — real progress toward every active badge. */
  myBadges() {
    return this.api.get<StudentBadgeStatusItem[]>('/gamification/me/badges')
  }
}
