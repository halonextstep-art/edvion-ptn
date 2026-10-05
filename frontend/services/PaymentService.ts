import type { ApiClient } from './ApiClient'
import type {
  CheckoutSessionItem,
  FulfillResult,
  InitiatePurchaseResult,
  PaymentProviderStatusItem,
  PaymentTransactionItem,
  PendingPurchaseItem,
} from '~/types'

// Lihat backend domain::payment. Selama belum ada provider gateway nyata yang
// dikonfigurasi, `initiateCheckout` selalu balik `mode: 'manual_pending'` — transaksi
// nyata masuk antrean manual admin (bukan gagal, bukan pura-pura sukses instan).
// `providerStatus()` dipakai UI untuk menampilkan status itu secara jujur.
export class PaymentService {
  constructor(private api: ApiClient) {}

  providerStatus() {
    return this.api.get<PaymentProviderStatusItem>('/payments/provider')
  }
  initiateCheckout(packageId: string) {
    return this.api.post<InitiatePurchaseResult>('/payments/checkout', { package_id: packageId })
  }
  myTransactions() {
    return this.api.get<PaymentTransactionItem[]>('/payments')
  }
  getTransaction(id: string) {
    return this.api.get<PaymentTransactionItem>(`/payments/${id}`)
  }
  /** Buyer self-service — batalkan transaksi milik sendiri selama masih `pending`. */
  cancelTransaction(id: string) {
    return this.api.post<PaymentTransactionItem>(`/payments/${id}/cancel`)
  }
  /** Admin-only manual purchase-request queue. */
  listPending() {
    return this.api.get<PendingPurchaseItem[]>('/payments/pending')
  }
  /** Admin-only — confirms a pending request, grants the package via an auto-redeemed
   *  Access voucher, and marks the transaction Paid. */
  fulfill(transactionId: string) {
    return this.api.post<FulfillResult>(`/payments/${transactionId}/fulfill`)
  }
}

export type { CheckoutSessionItem }
