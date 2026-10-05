<script setup lang="ts">
// Admin manual purchase-request queue — see backend PaymentService::list_pending/fulfill.
// No real payment gateway is configured, so every checkout lands here as `Pending` until
// an admin manually confirms payment (e.g. after checking a bank transfer) and clicks
// "Konfirmasi & Kirim Kode Akses", which grants the package via a real, auto-redeemed
// Access voucher — the admin then relays that code to the buyer manually (WhatsApp/etc).
import { ShoppingBag, Loader2, Copy, CheckCircle2, Clock, X, Mail } from 'lucide-vue-next'
import type { PendingPurchaseItem, FulfillResult } from '~/types'

const { paymentService } = useApi()
const toast = useToast()

const pending = ref<PendingPurchaseItem[]>([])
const loading = ref(true)
const loadError = ref('')
const fulfillingId = ref<string | null>(null)
const fulfilled = ref<(FulfillResult & { package_name: string; buyer_name: string })[]>([])

async function load() {
  loading.value = true
  loadError.value = ''
  try {
    pending.value = await paymentService.listPending()
  } catch (e: any) {
    loadError.value = e?.message ?? 'Gagal memuat antrean permintaan pembelian.'
    toast.error('Gagal memuat antrean permintaan pembelian', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

async function confirmFulfill(item: PendingPurchaseItem) {
  if (fulfillingId.value) return
  fulfillingId.value = item.transaction.id
  try {
    const res = await paymentService.fulfill(item.transaction.id)
    fulfilled.value.unshift({ ...res, package_name: item.package_name, buyer_name: item.buyer_name })
    pending.value = pending.value.filter((p) => p.transaction.id !== item.transaction.id)
    toast.success(`Pembelian ${item.buyer_name} dikonfirmasi`, `Kode akses: ${res.voucher_code}`)
  } catch (e: any) {
    toast.error('Gagal mengonfirmasi pembelian', e?.message)
  } finally {
    fulfillingId.value = null
  }
}

function dismissFulfilled(voucherCode: string) {
  fulfilled.value = fulfilled.value.filter((f) => f.voucher_code !== voucherCode)
}

async function copyCode(code: string) {
  await navigator.clipboard?.writeText(code)
  toast.success(`Kode ${code} disalin`)
}

function fmtRp(n: number) {
  return 'Rp ' + n.toLocaleString('id-ID')
}
function fmtDate(iso: string) {
  return new Date(iso).toLocaleString('id-ID', { day: '2-digit', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' })
}
</script>

<template>
  <div class="space-y-6">
    <div>
      <h2 class="text-lg font-bold flex items-center gap-2"><ShoppingBag class="w-5 h-5 text-indigo-600" />Permintaan Pembelian</h2>
      <p class="text-sm text-muted-foreground">Konfirmasi pembayaran manual (transfer/dsb) lalu kirim kode akses ke pembeli.</p>
    </div>

    <!-- Just-fulfilled results — kept visible (not auto-dismissed like a toast) so the
         admin has time to copy the code before relaying it to the buyer. -->
    <Card v-for="f in fulfilled" :key="f.voucher_code" class="p-4 border-2 border-emerald-200 bg-emerald-50 flex items-center gap-3 flex-wrap">
      <div class="w-10 h-10 rounded-xl bg-emerald-100 flex items-center justify-center shrink-0">
        <CheckCircle2 class="w-5 h-5 text-emerald-600" />
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-sm font-semibold text-emerald-900">{{ f.buyer_name }} — {{ f.package_name }} dikonfirmasi</p>
        <div class="flex items-center gap-2 mt-1">
          <span class="font-mono font-black tracking-wider text-emerald-800 bg-white px-2 py-0.5 rounded border border-emerald-200 text-sm">{{ f.voucher_code }}</span>
          <Button variant="outline" size="sm" class="gap-1.5" @click="copyCode(f.voucher_code)"><Copy class="w-3.5 h-3.5" />Salin Kode</Button>
        </div>
      </div>
      <button class="p-1.5 rounded-lg hover:bg-emerald-100 text-emerald-700 shrink-0" @click="dismissFulfilled(f.voucher_code)"><X class="w-4 h-4" /></button>
    </Card>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
    <Card v-else-if="loadError" class="p-10 text-center">
      <p class="text-sm text-red-600">{{ loadError }}</p>
      <Button variant="outline" size="sm" class="mt-3" @click="load">Coba Lagi</Button>
    </Card>
    <Card v-else-if="pending.length === 0" class="p-10 text-center">
      <ShoppingBag class="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" />
      <p class="text-muted-foreground text-sm">Tidak ada permintaan pembelian yang menunggu konfirmasi.</p>
    </Card>

    <div v-else class="space-y-2.5">
      <Card v-for="item in pending" :key="item.transaction.id" class="p-4 flex items-center gap-4 flex-wrap">
        <div class="w-10 h-10 rounded-xl bg-amber-50 flex items-center justify-center shrink-0">
          <Clock class="w-5 h-5 text-amber-600" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 mb-1 flex-wrap">
            <span class="font-semibold text-slate-900 text-sm">{{ item.buyer_name }}</span>
            <Badge variant="outline" class="text-[10px]">{{ item.package_name }}</Badge>
            <Badge class="bg-amber-100 text-amber-700 text-[10px]">Menunggu Konfirmasi</Badge>
          </div>
          <div class="flex items-center gap-2 text-xs text-muted-foreground flex-wrap">
            <span class="inline-flex items-center gap-1"><Mail class="w-3 h-3" />{{ item.buyer_email }}</span>
            <span>&middot;</span><span>Diajukan {{ fmtDate(item.transaction.created_at) }}</span>
          </div>
        </div>
        <div class="shrink-0 text-right">
          <p class="font-black text-sm text-slate-900">{{ fmtRp(item.transaction.amount) }}</p>
          <p class="text-xs text-muted-foreground">{{ item.transaction.currency }}</p>
        </div>
        <Button size="sm" class="shrink-0 gap-1.5" :disabled="fulfillingId === item.transaction.id" @click="confirmFulfill(item)">
          <Loader2 v-if="fulfillingId === item.transaction.id" class="w-4 h-4 animate-spin" />
          <CheckCircle2 v-else class="w-4 h-4" />
          Konfirmasi &amp; Kirim Kode Akses
        </Button>
      </Card>
    </div>
  </div>
</template>
