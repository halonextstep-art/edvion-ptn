import type { ApiClient } from './ApiClient'
import type {
  AnalyticsSummary, QuestionStatusCount, SchoolRanking, ScoreTrendPoint, StudentActivityItem, StudentRanking,
  SubjectAccuracy, YearlyPerformancePoint,
} from '~/types'

export class AnalyticsService {
  constructor(private api: ApiClient) {}

  summary() {
    return this.api.get<AnalyticsSummary>('/analytics/summary')
  }

  scoreTrend(days = 14) {
    return this.api.get<ScoreTrendPoint[]>('/analytics/score-trend', { days })
  }

  subjectBreakdown() {
    return this.api.get<SubjectAccuracy[]>('/analytics/subject-breakdown')
  }

  questionStatus() {
    return this.api.get<QuestionStatusCount[]>('/analytics/question-status')
  }

  topSchools(limit = 8) {
    return this.api.get<SchoolRanking[]>('/analytics/top-schools', { limit })
  }
  /** "Perbandingan Tahun" platform-wide — attempts per tahun kalender submitted_at. */
  scoreTrendByYear() {
    return this.api.get<YearlyPerformancePoint[]>('/analytics/score-trend-by-year')
  }

  // ─── School-portal-scoped variants ──────────────────────────────────────────
  schoolSubjectBreakdown() {
    return this.api.get<SubjectAccuracy[]>('/analytics/school/subject-breakdown')
  }
  schoolScoreTrend(days = 30) {
    return this.api.get<ScoreTrendPoint[]>('/analytics/school/score-trend', { days })
  }
  /** "Perbandingan Tahun" milik sekolah ini — sama seperti scoreTrendByYear tapi di-scope. */
  schoolScoreTrendByYear() {
    return this.api.get<YearlyPerformancePoint[]>('/analytics/school/score-trend-by-year')
  }
  schoolStudentRanking() {
    return this.api.get<StudentRanking[]>('/analytics/school/ranking')
  }
  /** "Analytics & Insights" drill-down — includes students with zero attempts (unlike
   *  schoolStudentRanking above), plus rombel_code/last_attempt_at for per-kelas grouping
   *  and inactivity detection. */
  schoolStudentActivity() {
    return this.api.get<StudentActivityItem[]>('/analytics/school/student-activity')
  }

  // ─── Student-portal-scoped variant ──────────────────────────────────────────
  mySubjectBreakdown() {
    return this.api.get<SubjectAccuracy[]>('/analytics/me/subject-breakdown')
  }
  /** Platform-wide average trend — real, aggregate/anonymized, open to any role. */
  nationalScoreTrend(days = 14) {
    return this.api.get<ScoreTrendPoint[]>('/analytics/national-score-trend', { days })
  }
}
