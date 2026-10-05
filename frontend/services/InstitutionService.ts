import type { ApiClient } from './ApiClient'
import type { InstitutionItem, InstitutionPayload } from '~/types'

/** Institution master data (profil institusi PTN, di luar statistik per-prodi di
 * ptn_programs) — lihat backend `domain::institution` doc comment. Read terbuka untuk
 * semua role login; create/update/delete admin-only (ditegakkan di backend). */
export class InstitutionService {
  constructor(private api: ApiClient) {}

  list(search?: string) {
    return this.api.get<InstitutionItem[]>('/institutions', search ? { search } : undefined)
  }

  get(id: string) {
    return this.api.get<InstitutionItem>(`/institutions/${id}`)
  }

  /** Resolusi nama_ptn (plain text di ptn_programs) -> profil institusi. Mengembalikan
   * `null` (bukan error) kalau data riset belum ada untuk institusi tsb — state jujur,
   * bukan placeholder. */
  lookup(namaPtn: string) {
    return this.api.get<InstitutionItem | null>('/institutions/lookup', { nama_ptn: namaPtn })
  }

  create(payload: InstitutionPayload) {
    return this.api.post<InstitutionItem>('/institutions', payload)
  }

  update(id: string, payload: InstitutionPayload) {
    return this.api.put<InstitutionItem>(`/institutions/${id}`, payload)
  }

  delete(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/institutions/${id}`)
  }
}
