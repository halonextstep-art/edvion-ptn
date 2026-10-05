import type { ApiClient } from './ApiClient'
import type {
  School, SchoolPayload, SchoolStatus, SchoolOverview, StudentRosterStat, PtnDistributionItem, RecentActivityItem,
} from '~/types'

export class SchoolService {
  constructor(private api: ApiClient) {}

  list() {
    return this.api.get<School[]>('/schools')
  }

  get(id: string) {
    return this.api.get<School>(`/schools/${id}`)
  }

  create(payload: SchoolPayload) {
    return this.api.post<School>('/schools', payload)
  }

  update(id: string, payload: SchoolPayload) {
    return this.api.put<School>(`/schools/${id}`, payload)
  }

  remove(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/schools/${id}`)
  }

  setStatus(id: string, status: SchoolStatus) {
    return this.api.patch<School>(`/schools/${id}/status`, { status })
  }

  /** School-portal-only aggregate (school PIC's own Overview tab). */
  overview() {
    return this.api.get<SchoolOverview>('/schools/overview')
  }

  /** School-portal-only per-student breakdown (Manajemen Siswa table columns). */
  rosterStats() {
    return this.api.get<StudentRosterStat[]>('/schools/roster-stats')
  }

  /** School-portal-only aggregate (Analytics tab's "Distribusi Target PTN" chart). */
  ptnDistribution() {
    return this.api.get<PtnDistributionItem[]>('/schools/ptn-distribution')
  }

  /** School-portal-only feed (Overview tab's "Aktivitas Terkini"). */
  recentActivity(limit = 10) {
    return this.api.get<RecentActivityItem[]>('/schools/recent-activity', { limit })
  }
}
