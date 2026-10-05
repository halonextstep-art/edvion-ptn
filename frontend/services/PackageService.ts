import type { ApiClient } from './ApiClient'
import type { ElectivePackageGroup, ElectivesResponse, PackageContentItem, PackageContentType, PackageItem, PackagePayload } from '~/types'

export class PackageService {
  constructor(private api: ApiClient) {}

  list() {
    return this.api.get<PackageItem[]>('/packages')
  }

  get(id: string) {
    return this.api.get<PackageItem>(`/packages/${id}`)
  }

  create(payload: PackagePayload) {
    return this.api.post<PackageItem>('/packages', payload)
  }

  update(id: string, payload: PackagePayload) {
    return this.api.put<PackageItem>(`/packages/${id}`, payload)
  }

  setActive(id: string, active: boolean) {
    return this.api.patch<PackageItem>(`/packages/${id}/active`, { active })
  }

  remove(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/packages/${id}`)
  }

  // ─── Isi Paket — which TryoutSessions/SimulationTemplates this package unlocks ───────────

  listContent(packageId: string) {
    return this.api.get<PackageContentItem[]>(`/packages/${packageId}/content`)
  }

  addContentItem(packageId: string, contentType: PackageContentType, contentId: string) {
    return this.api.post<PackageContentItem>(`/packages/${packageId}/content`, {
      content_type: contentType,
      content_id: contentId,
    })
  }

  removeContentItem(packageId: string, itemId: string) {
    return this.api.delete<{ deleted: boolean }>(`/packages/${packageId}/content/${itemId}`)
  }

  /** "Setujui Semua Soal" — bulk-approves every question in a Set Soal used by this package's
   *  sessions, instead of approving one-by-one in Bank Soal. See backend
   *  `PackageService::bulk_approve_content` doc comment. Admin-only. */
  bulkApproveContent(packageId: string) {
    return this.api.post<{ approved_count: number }>(`/packages/${packageId}/bulk-approve-content`)
  }

  // ─── "Mapel Pilihan" (TKA-style subject choice) — student-facing ────────────────────────

  /** Every package with an active "mapel pilihan" the student can currently pick from —
   *  powers the "Pilih Mapel Pilihan" section in DrillingZone.vue. */
  myElectivePackages() {
    return this.api.get<ElectivePackageGroup[]>('/electives')
  }

  getElectives(packageId: string) {
    return this.api.get<ElectivesResponse>(`/packages/${packageId}/electives`)
  }

  setElectiveChoices(packageId: string, sessionIds: string[]) {
    return this.api.put<{ session_ids: string[] }>(`/packages/${packageId}/electives`, { session_ids: sessionIds })
  }
}
