import type { ApiClient } from './ApiClient'
import type { PackageItem, PublicStats, PublicVoucherCheck, RegistrationStatus } from '~/types'

/** No-auth endpoints backing the marketing landing page (frontend/pages/index.vue). */
export class PublicService {
  constructor(private api: ApiClient) {}

  stats() {
    return this.api.get<PublicStats>('/public/stats')
  }
  packages() {
    return this.api.get<PackageItem[]>('/public/packages')
  }
  checkVoucher(code: string) {
    return this.api.get<PublicVoucherCheck>('/public/vouchers/check', { code })
  }
  /** Backs /daftar's on-mount check — lets it honestly show a disabled state instead of a
   * form that would just fail on submit if B2C self-registration is currently off. */
  registrationStatus() {
    return this.api.get<RegistrationStatus>('/public/registration-status')
  }
}
