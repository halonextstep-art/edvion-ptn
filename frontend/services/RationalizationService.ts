import type { ApiClient } from './ApiClient'
import type {
  PtnProgramItem, PtnProgramPayload, ProgramListResponse, RaporScoreItem, RaporEntryPayload,
  RaporBulkImportEntryPayload, RaporBulkImportResult,
  ChanceResponse, TargetWithChanceItem, PtnTrack, PtnPriority,
  SnbpDashboardItem, AlumniBenchmarkItem, AlumniPayload, AlumniRaporScoreItem, AlumniRaporEntryPayload,
  SchoolEligibilityItem, EligibilityPayload,
  AchievementItem, AchievementPayload, SnbpRankingEntryItem,
  SnbtDashboardItem, SnbtRosterRowItem, SnbtTrackingPayload,
  SnbpRosterRowItem, SnbpParticipationPayload, AuditLogEntryItem, Attempt, TkaPackageGroupItem,
  AttemptResult, ReviewItem,
} from '~/types'

export class RationalizationService {
  constructor(private api: ApiClient) {}

  // Catalog (read: any role; write: admin only)
  listPrograms(params: { search?: string; rumpun?: string; jenjang?: string; page?: number; page_size?: number } = {}) {
    return this.api.get<ProgramListResponse>('/rationalization/programs', params)
  }
  getProgram(id: string) {
    return this.api.get<PtnProgramItem>(`/rationalization/programs/${id}`)
  }
  distinctRumpun() {
    return this.api.get<string[]>('/rationalization/programs/rumpun')
  }
  catalogSize() {
    return this.api.get<{ total: number }>('/rationalization/programs/catalog-size')
  }
  createProgram(payload: PtnProgramPayload) {
    return this.api.post<PtnProgramItem>('/rationalization/programs', payload)
  }
  updateProgram(id: string, payload: PtnProgramPayload) {
    return this.api.put<PtnProgramItem>(`/rationalization/programs/${id}`, payload)
  }
  deleteProgram(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/rationalization/programs/${id}`)
  }

  // Rapor scores (student-owned)
  upsertRapor(entries: RaporEntryPayload[]) {
    return this.api.post<RaporScoreItem[]>('/rationalization/rapor', { entries })
  }
  listRapor() {
    return this.api.get<RaporScoreItem[]>('/rationalization/rapor')
  }
  deleteRapor(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/rationalization/rapor/${id}`)
  }
  // Import nilai rapor massal (Portal Sekolah, banyak siswa dari 1 file Excel) — dicocokkan
  // per-baris via NIS, lihat SchoolRationalizationService::bulk_import_rapor di backend.
  bulkImportRapor(entries: RaporBulkImportEntryPayload[]) {
    return this.api.post<RaporBulkImportResult>('/rationalization/rapor/bulk-import', { entries })
  }

  // Targets + chance
  addTarget(ptn_program_id: string, track: PtnTrack, priority: PtnPriority = 'utama') {
    return this.api.post<{ id: string }>('/rationalization/targets', { ptn_program_id, track, priority })
  }
  myTargets() {
    return this.api.get<TargetWithChanceItem[]>('/rationalization/targets')
  }
  setTargetPriority(id: string, priority: PtnPriority) {
    return this.api.patch<{ updated: boolean }>(`/rationalization/targets/${id}`, { priority })
  }
  removeTarget(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/rationalization/targets/${id}`)
  }
  previewChance(ptn_program_id: string, track: PtnTrack) {
    return this.api.get<ChanceResponse>('/rationalization/preview', { ptn_program_id, track })
  }

  // ── School PIC (own students) or Admin — verifikasi & edit data SNBP siswa. Endpoint
  //    yang sama persis dipakai siswa untuk input sendiri, hanya di-scope ke student_id
  //    tertentu dan lewat ownership check di backend (ensure_owns_student). ──────────
  studentTargets(studentId: string) {
    return this.api.get<TargetWithChanceItem[]>(`/rationalization/students/${studentId}/targets`)
  }
  addStudentTarget(studentId: string, ptn_program_id: string, track: PtnTrack, priority: PtnPriority = 'utama') {
    return this.api.post<{ id: string }>(`/rationalization/students/${studentId}/targets`, { ptn_program_id, track, priority })
  }
  setStudentTargetPriority(studentId: string, id: string, priority: PtnPriority) {
    return this.api.patch<{ updated: boolean }>(`/rationalization/students/${studentId}/targets/${id}`, { priority })
  }
  removeStudentTarget(studentId: string, id: string) {
    return this.api.delete<{ deleted: boolean }>(`/rationalization/students/${studentId}/targets/${id}`)
  }
  studentRapor(studentId: string) {
    return this.api.get<RaporScoreItem[]>(`/rationalization/students/${studentId}/rapor`)
  }
  upsertStudentRapor(studentId: string, entries: RaporEntryPayload[]) {
    return this.api.post<RaporScoreItem[]>(`/rationalization/students/${studentId}/rapor`, { entries })
  }
  deleteStudentRapor(studentId: string, id: string) {
    return this.api.delete<{ deleted: boolean }>(`/rationalization/students/${studentId}/rapor/${id}`)
  }
  studentAchievements(studentId: string) {
    return this.api.get<AchievementItem[]>(`/rationalization/students/${studentId}/achievements`)
  }
  addStudentAchievement(studentId: string, payload: AchievementPayload) {
    return this.api.post<AchievementItem>(`/rationalization/students/${studentId}/achievements`, payload)
  }
  deleteStudentAchievement(studentId: string, id: string) {
    return this.api.delete<{ deleted: boolean }>(`/rationalization/students/${studentId}/achievements/${id}`)
  }
  previewStudentChance(studentId: string, ptn_program_id: string, track: PtnTrack) {
    return this.api.get<ChanceResponse>(`/rationalization/students/${studentId}/preview`, { ptn_program_id, track })
  }

  // School Rasionalisasi sub-tabs: Dashboard / Alumni ("Kaka Kelas") / Eligible
  schoolDashboard() {
    return this.api.get<SnbpDashboardItem>('/rationalization/school/dashboard')
  }
  schoolFullRanking() {
    return this.api.get<SnbpRankingEntryItem[]>('/rationalization/school/ranking')
  }
  listAlumni() {
    return this.api.get<AlumniBenchmarkItem[]>('/rationalization/school/alumni')
  }
  addAlumni(payload: AlumniPayload) {
    return this.api.post<AlumniBenchmarkItem>('/rationalization/school/alumni', payload)
  }
  deleteAlumni(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/rationalization/school/alumni/${id}`)
  }
  // "Detail Rapor" alumni: snapshot nilai per mapel, tanpa semester.
  listAlumniRapor(alumniId: string) {
    return this.api.get<AlumniRaporScoreItem[]>(`/rationalization/school/alumni/${alumniId}/rapor`)
  }
  upsertAlumniRapor(alumniId: string, entries: AlumniRaporEntryPayload[]) {
    return this.api.post<AlumniRaporScoreItem[]>(`/rationalization/school/alumni/${alumniId}/rapor`, { entries })
  }
  deleteAlumniRapor(alumniId: string, scoreId: string) {
    return this.api.delete<{ deleted: boolean }>(`/rationalization/school/alumni/${alumniId}/rapor/${scoreId}`)
  }
  listEligibility() {
    return this.api.get<SchoolEligibilityItem[]>('/rationalization/school/eligibility')
  }
  upsertEligibility(payload: EligibilityPayload) {
    return this.api.post<SchoolEligibilityItem>('/rationalization/school/eligibility', payload)
  }

  // Prestasi (student-owned achievements)
  listAchievements() {
    return this.api.get<AchievementItem[]>('/rationalization/achievements')
  }
  addAchievement(payload: AchievementPayload) {
    return this.api.post<AchievementItem>('/rationalization/achievements', payload)
  }
  deleteAchievement(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/rationalization/achievements/${id}`)
  }
  uploadAchievementCertificate(id: string, file: File) {
    const form = new FormData()
    form.append('file', file)
    return this.api.upload<AchievementItem>(`/rationalization/achievements/${id}/certificate`, form)
  }

  // Rekap SNBT (School PIC only)
  schoolSnbtDashboard() {
    return this.api.get<SnbtDashboardItem>('/rationalization/school/snbt/dashboard')
  }
  schoolSnbtRoster() {
    return this.api.get<SnbtRosterRowItem[]>('/rationalization/school/snbt/roster')
  }
  upsertSnbtTracking(studentId: string, payload: SnbtTrackingPayload) {
    return this.api.post<SnbtRosterRowItem>(`/rationalization/school/snbt/students/${studentId}/tracking`, payload)
  }
  /** Riwayat pengerjaan (tryout/drilling/Simulasi UTBK) nyata siswa ini — dipakai di drill-down
   *  Rekap SNBT sekolah. */
  schoolStudentAttempts(studentId: string) {
    return this.api.get<Attempt[]>(`/rationalization/school/students/${studentId}/attempts`)
  }
  /** Detail satu TO: skor + rincian per mata uji — dipakai untuk "kekuatan/kekurangan" di
   *  drill-down Riwayat Pengerjaan TO dan ekspor PDF-nya. */
  schoolStudentAttemptResult(studentId: string, attemptId: string) {
    return this.api.get<AttemptResult>(`/rationalization/school/students/${studentId}/attempts/${attemptId}/result`)
  }
  /** Pembahasan per soal untuk satu TO — dipakai saat PIC sekolah ingin melihat detail lengkap
   *  jawaban siswa, bukan cuma ringkasan skor. */
  schoolStudentAttemptReview(studentId: string, attemptId: string) {
    return this.api.get<ReviewItem[]>(`/rationalization/school/students/${studentId}/attempts/${attemptId}/review`)
  }

  // Rekap TKA (School PIC only)
  schoolTkaRoster() {
    return this.api.get<TkaPackageGroupItem[]>('/rationalization/school/tka/roster')
  }

  // SNBP roster "Daftar Siswa" (School PIC only)
  schoolSnbpRoster() {
    return this.api.get<SnbpRosterRowItem[]>('/rationalization/school/snbp/roster')
  }
  upsertSnbpParticipation(studentId: string, payload: SnbpParticipationPayload) {
    return this.api.post<SnbpRosterRowItem>(`/rationalization/school/snbp/roster/${studentId}`, payload)
  }

  // Audit trail ("Riwayat Perubahan") — School PIC/Admin only, same ownership check as
  // the other /students/:id endpoints.
  studentAuditLog(studentId: string) {
    return this.api.get<AuditLogEntryItem[]>(`/rationalization/students/${studentId}/audit-log`)
  }
}
