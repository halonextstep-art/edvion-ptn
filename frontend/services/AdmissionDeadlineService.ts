import type { ApiClient } from './ApiClient'
import type { AdmissionDeadlineItem, AdmissionDeadlinePayload } from '~/types'

export class AdmissionDeadlineService {
  constructor(private api: ApiClient) {}

  list() {
    return this.api.get<AdmissionDeadlineItem[]>('/deadlines')
  }

  /** Widget "Deadline Mendatang" (Admin/Sekolah/Siswa) — hanya yang belum lewat, terurut. */
  upcoming(limit = 5) {
    return this.api.get<AdmissionDeadlineItem[]>('/deadlines/upcoming', { limit })
  }

  get(id: string) {
    return this.api.get<AdmissionDeadlineItem>(`/deadlines/${id}`)
  }

  create(payload: AdmissionDeadlinePayload) {
    return this.api.post<AdmissionDeadlineItem>('/deadlines', payload)
  }

  update(id: string, payload: AdmissionDeadlinePayload) {
    return this.api.put<AdmissionDeadlineItem>(`/deadlines/${id}`, payload)
  }

  remove(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/deadlines/${id}`)
  }
}
