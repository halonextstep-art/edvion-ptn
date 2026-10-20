import type { ApiClient } from './ApiClient'
import type {
  School, SchoolPayload, SchoolStatus, SchoolOverview, StudentRosterStat, PtnDistributionItem, RecentActivityItem,
  LinkableTenant, LinkableStudent, SchoolRosterCount, InProgressActivityItem,
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

  /** School-portal-only live feed — "Pemantauan Pengerjaan": every in-progress Attempt and
   *  SimulationRun across the school's roster. Meant to be called on an interval by the
   *  caller (no realtime push infra exists in this backend — see `InProgressActivityItem`
   *  doc comment). */
  inProgressActivity() {
    return this.api.get<InProgressActivityItem[]>('/schools/in-progress-activity')
  }

  /** Force-clears one student's stuck in-progress attempt/run so they can start a fresh
   *  one. `kind`/`refId` come straight off the matching `InProgressActivityItem`. */
  forceAbandonActivity(refId: string, kind: InProgressActivityItem['kind']) {
    return this.api.post<{ abandoned: boolean }>(`/schools/in-progress-activity/${refId}/abandon`, { kind })
  }

  /** Per-school student/active-student counts for the admin list cards — one aggregate
   * query, not every student row across every school. */
  rosterCounts() {
    return this.api.get<SchoolRosterCount[]>('/schools/roster-counts')
  }

  /** "Link ke Tenant Existing" dropdown's data source — existing tenants (from other
   * modules) not yet adopted by any PTN school profile. */
  unlinkedTenants() {
    return this.api.get<LinkableTenant[]>('/schools/unlinked-tenants')
  }

  /** Import-siswa checklist's data source for a linked-tenant school. */
  linkableStudents(schoolId: string) {
    return this.api.get<LinkableStudent[]>(`/schools/${schoolId}/linkable-students`)
  }

  /** Submits the import-siswa checklist — grants PTN access to the checked existing
   * accounts without touching their role/password/username. */
  linkStudents(schoolId: string, akunIds: string[]) {
    return this.api.post<{ linked: number }>(`/schools/${schoolId}/linkable-students`, { akun_ids: akunIds })
  }
}
