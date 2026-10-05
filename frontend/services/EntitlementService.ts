import type { ApiClient } from './ApiClient'
import type { AccessStatus, SchoolEntitlementItem, SchoolEntitlementPayload } from '~/types'

export class EntitlementService {
  constructor(private api: ApiClient) {}

  listForSchool(schoolId: string) {
    return this.api.get<SchoolEntitlementItem[]>(`/school-entitlements/school/${schoolId}`)
  }

  create(payload: SchoolEntitlementPayload) {
    return this.api.post<SchoolEntitlementItem>('/school-entitlements', payload)
  }

  remove(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/school-entitlements/${id}`)
  }

  myAccessStatus() {
    return this.api.get<AccessStatus>('/school-entitlements/me/status')
  }
}
