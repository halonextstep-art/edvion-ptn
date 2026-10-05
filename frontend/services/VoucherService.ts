import type { ApiClient } from './ApiClient'
import type { VoucherItem, VoucherPayload, RedemptionResult } from '~/types'

export class VoucherService {
  constructor(private api: ApiClient) {}

  list() {
    return this.api.get<VoucherItem[]>('/vouchers')
  }

  get(id: string) {
    return this.api.get<VoucherItem>(`/vouchers/${id}`)
  }

  /** Returns an array — creating with quantity > 1 generates a batch of codes at once. */
  create(payload: VoucherPayload) {
    return this.api.post<VoucherItem[]>('/vouchers', payload)
  }

  setActive(id: string, active: boolean) {
    return this.api.patch<VoucherItem>(`/vouchers/${id}/active`, { active })
  }

  remove(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/vouchers/${id}`)
  }

  /** Student-facing redeem — validates + atomically increments used_count server-side. */
  redeem(code: string) {
    return this.api.post<RedemptionResult>('/vouchers/redeem', { code })
  }
}
