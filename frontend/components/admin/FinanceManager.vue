<script setup lang="ts">
import { DollarSign, Calendar, School, Package, Info, Users, Percent } from 'lucide-vue-next'
import type { FinanceSummary, EventRevenueItem, SchoolRevenueItem } from '~/types'

const { financeService } = useApi()
const toast = useToast()

const loading = ref(false)
const summary = ref<FinanceSummary | null>(null)
const eventRows = ref<EventRevenueItem[]>([])
const schoolRows = ref<SchoolRevenueItem[]>([])

async function load() {
  loading.value = true
  try {
    const [s, ev, sc] = await Promise.all([
      financeService.summary(),
      financeService.eventBreakdown(),
      financeService.schoolBreakdown(),
    ])
    summary.value = s
    eventRows.value = ev
    schoolRows.value = sc
  } catch (e: any) {
    toast.error('Gagal memuat data keuangan', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

function fmtRp(n: number) {
  return 'Rp ' + n.toLocaleString('id-ID')
}
</script>

<template>
  <div class="space-y-6">
    <div class="bg-gradient-to-r from-emerald-600 via-teal-600 to-cyan-600 rounded-2xl p-6 text-white">
      <div class="flex items-center gap-2 mb-1"><DollarSign class="w-5 h-5 text-emerald-200" /><span class="text-sm font-semibold text-emerald-200">Ringkasan Keuangan</span></div>
      <h1 class="text-2xl font-black mb-1">Manajemen Keuangan</h1>
      <p class="text-emerald-100 text-sm">Dihitung dari data asli yang sudah ada di sistem — dashboard ringkasan, bukan buku besar per-transaksi. Untuk approve pembayaran manual siswa, lihat tab "Permintaan Pembelian".</p>
    </div>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>

    <template v-else-if="summary">
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card class="p-4 flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-emerald-50 flex items-center justify-center shrink-0"><DollarSign class="w-5 h-5 text-emerald-600" /></div>
          <div><div class="text-lg font-black text-emerald-600">{{ fmtRp(summary.total_event_revenue_estimate) }}</div><div class="text-xs text-muted-foreground">Estimasi Revenue Event</div></div>
        </Card>
        <Card class="p-4 flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-blue-50 flex items-center justify-center shrink-0"><Calendar class="w-5 h-5 text-blue-600" /></div>
          <div><div class="text-xl font-black text-blue-600">{{ summary.paid_events_count }}</div><div class="text-xs text-muted-foreground">Event Berbayar</div></div>
        </Card>
        <Card class="p-4 flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-indigo-50 flex items-center justify-center shrink-0"><Users class="w-5 h-5 text-indigo-600" /></div>
          <div><div class="text-xl font-black text-indigo-600">{{ summary.total_paid_participants.toLocaleString('id-ID') }}</div><div class="text-xs text-muted-foreground">Peserta Event Berbayar</div></div>
        </Card>
        <Card class="p-4 flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-amber-50 flex items-center justify-center shrink-0"><Percent class="w-5 h-5 text-amber-600" /></div>
          <div><div class="text-xl font-black text-amber-600">{{ summary.avg_revenue_share_percent.toFixed(1) }}%</div><div class="text-xs text-muted-foreground">Rata-rata Revenue Share</div></div>
        </Card>
        <Card class="p-4 flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-slate-50 flex items-center justify-center shrink-0"><School class="w-5 h-5 text-slate-600" /></div>
          <div><div class="text-xl font-black text-slate-700">{{ summary.school_count }}</div><div class="text-xs text-muted-foreground">Sekolah Mitra</div></div>
        </Card>
        <Card class="p-4 flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-teal-50 flex items-center justify-center shrink-0"><DollarSign class="w-5 h-5 text-teal-600" /></div>
          <div><div class="text-lg font-black text-teal-600">{{ fmtRp(summary.total_monthly_revenue) }}</div><div class="text-xs text-muted-foreground">Total Kontrak Bulanan Sekolah</div></div>
        </Card>
        <Card class="p-4 flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-violet-50 flex items-center justify-center shrink-0"><Package class="w-5 h-5 text-violet-600" /></div>
          <div><div class="text-xl font-black text-violet-600">{{ summary.active_package_count }}</div><div class="text-xs text-muted-foreground">Paket Aktif di Landing Page</div></div>
        </Card>
        <Card class="p-4 flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-rose-50 flex items-center justify-center shrink-0"><Package class="w-5 h-5 text-rose-600" /></div>
          <div><div class="text-lg font-black text-rose-600">{{ fmtRp(summary.package_catalog_value) }}</div><div class="text-xs text-muted-foreground">Total Nilai Katalog Paket</div></div>
        </Card>
      </div>

      <div class="grid lg:grid-cols-2 gap-6">
        <Card class="overflow-hidden">
          <div class="px-5 py-3 border-b bg-slate-50"><h3 class="font-bold text-slate-900 text-sm flex items-center gap-2"><Calendar class="w-4 h-4 text-indigo-500" />Estimasi Revenue per Event</h3></div>
          <div v-if="eventRows.length === 0" class="p-8 text-center text-sm text-muted-foreground">Belum ada event berbayar.</div>
          <div v-else class="divide-y">
            <div v-for="ev in eventRows" :key="ev.event_id" class="flex items-center justify-between px-5 py-3">
              <div class="min-w-0 pr-3"><p class="text-sm font-semibold text-slate-900 truncate">{{ ev.event_name }}</p><p class="text-xs text-muted-foreground">{{ fmtRp(ev.price) }} &times; {{ ev.participants.toLocaleString('id-ID') }} peserta</p></div>
              <p class="text-sm font-black text-emerald-600 shrink-0">{{ fmtRp(ev.revenue_estimate) }}</p>
            </div>
          </div>
        </Card>

        <Card class="overflow-hidden">
          <div class="px-5 py-3 border-b bg-slate-50"><h3 class="font-bold text-slate-900 text-sm flex items-center gap-2"><School class="w-4 h-4 text-indigo-500" />Revenue Share Sekolah Mitra</h3></div>
          <div v-if="schoolRows.length === 0" class="p-8 text-center text-sm text-muted-foreground">Belum ada sekolah mitra.</div>
          <div v-else class="divide-y">
            <div v-for="sc in schoolRows" :key="sc.school_id" class="flex items-center justify-between px-5 py-3">
              <div class="min-w-0 pr-3"><p class="text-sm font-semibold text-slate-900 truncate">{{ sc.school_name }}</p><p class="text-xs text-muted-foreground capitalize">{{ sc.package_type }} &middot; Share {{ sc.revenue_share_percent }}%</p></div>
              <p class="text-sm font-black text-slate-700 shrink-0">{{ fmtRp(sc.monthly_revenue) }}</p>
            </div>
          </div>
        </Card>
      </div>

      <p class="text-xs text-muted-foreground flex items-start gap-1.5"><Info class="w-3.5 h-3.5 mt-0.5 shrink-0" />Estimasi revenue event dihitung dari harga tiket × jumlah peserta asli (data yang sama dengan tab Event). Kontrak bulanan sekolah adalah nilai yang diinput admin saat onboarding — akan tetap Rp 0 sampai ada sistem billing nyata. Belum ada daftar transaksi individual karena platform ini belum terhubung ke payment gateway.</p>
    </template>
  </div>
</template>
