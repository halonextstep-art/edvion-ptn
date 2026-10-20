<script setup lang="ts">
// Pemantauan Pengerjaan (Portal Sekolah) — daftar siswa yang SEDANG mengerjakan Tryout/
// Drilling/Mini atau Simulasi UTBK/TKA, dari schoolService.inProgressActivity() (lihat
// SchoolPortalService::in_progress_activity, backend). Tidak ada infrastruktur realtime
// (WebSocket/SSE) di backend ini — "realtime" di sini berarti auto-refresh via polling
// interval, bukan push dari server. Sekolah bisa "Reset" satu baris yang macet (tab ditutup
// di tengah ujian, dst) lewat schoolService.forceAbandonActivity() — ini membekukan status
// attempt/run itu jadi 'abandoned' sehingga siswa bisa memulai percobaan baru untuk sesi
// yang sama (selama masih in_progress, start_attempt/start_run selalu resume ke yang lama).
import { Radio, Clock, FileText, Timer, RefreshCw, RotateCcw, Pause, Play } from 'lucide-vue-next'
import type { InProgressActivityItem } from '~/types'

const { schoolService } = useApi()
const toast = useToast()

const items = ref<InProgressActivityItem[]>([])
const loading = ref(true)
const resettingId = ref<string | null>(null)
const lastRefreshedAt = ref<Date | null>(null)

// Auto-refresh tiap 15 detik — cukup "live" untuk kebutuhan pemantauan (sekolah tidak perlu
// update per detik), tanpa perlu push infrastructure yang belum ada di backend manapun di
// proyek ini. Dihentikan otomatis saat tab disembunyikan agar tidak boros request kalau PIC
// membuka banyak tab/membiarkan laptop menyala semalaman dengan tab ini terbuka.
const POLL_INTERVAL_MS = 15_000
let pollHandle: ReturnType<typeof setInterval> | null = null
const autoRefresh = ref(true)

async function load() {
  try {
    items.value = await schoolService.inProgressActivity()
    lastRefreshedAt.value = new Date()
  } catch (e: any) {
    toast.error('Gagal memuat status pengerjaan siswa', e?.message)
  } finally {
    loading.value = false
  }
}

function startPolling() {
  stopPolling()
  pollHandle = setInterval(load, POLL_INTERVAL_MS)
}
function stopPolling() {
  if (pollHandle) { clearInterval(pollHandle); pollHandle = null }
}
function toggleAutoRefresh() {
  autoRefresh.value = !autoRefresh.value
  if (autoRefresh.value) { load(); startPolling() } else { stopPolling() }
}

function handleVisibilityChange() {
  if (document.hidden) stopPolling()
  else if (autoRefresh.value) { load(); startPolling() }
}

onMounted(() => {
  load()
  startPolling()
  document.addEventListener('visibilitychange', handleVisibilityChange)
})
onUnmounted(() => {
  stopPolling()
  document.removeEventListener('visibilitychange', handleVisibilityChange)
})

async function resetActivity(item: InProgressActivityItem) {
  if (!window.confirm(`Reset pengerjaan "${item.title}" milik ${item.student_name}? Siswa akan bisa memulai percobaan baru untuk sesi ini.`)) return
  resettingId.value = item.ref_id
  try {
    await schoolService.forceAbandonActivity(item.ref_id, item.kind)
    toast.success(`Pengerjaan "${item.title}" milik ${item.student_name} berhasil di-reset`)
    await load()
  } catch (e: any) {
    toast.error('Gagal reset pengerjaan', e?.message)
  } finally {
    resettingId.value = null
  }
}

function formatDateTime(iso: string) {
  return new Date(iso).toLocaleString('id-ID', { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' })
}

// Durasi berjalan sejak started_at, dihitung ulang tiap render (bukan live-ticking per
// detik — cukup akurat untuk konteks pemantauan, dan menghindari 1 timer per baris).
function elapsedLabel(startedAt: string) {
  const minutes = Math.max(0, Math.floor((Date.now() - new Date(startedAt).getTime()) / 60000))
  if (minutes < 60) return `${minutes} menit`
  const h = Math.floor(minutes / 60)
  const m = minutes % 60
  return m > 0 ? `${h}j ${m}m` : `${h}j`
}

function kindLabel(kind: InProgressActivityItem['kind']) {
  return kind === 'simulation_run' ? 'Simulasi UTBK/TKA' : 'Tryout/Drilling'
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2">
          <Radio class="w-5 h-5 text-rose-600" />Pemantauan Pengerjaan
        </h2>
        <p class="text-sm text-muted-foreground">
          {{ items.length }} siswa sedang mengerjakan sesuatu saat ini — auto-refresh tiap {{ POLL_INTERVAL_MS / 1000 }} detik
        </p>
      </div>
      <div class="flex items-center gap-2">
        <span v-if="lastRefreshedAt" class="text-[11px] text-muted-foreground">
          Diperbarui {{ formatDateTime(lastRefreshedAt.toISOString()) }}
        </span>
        <Button size="sm" variant="outline" class="text-xs gap-1.5" @click="toggleAutoRefresh">
          <Pause v-if="autoRefresh" class="w-3.5 h-3.5" /><Play v-else class="w-3.5 h-3.5" />
          {{ autoRefresh ? 'Jeda' : 'Lanjutkan' }}
        </Button>
        <Button size="sm" variant="outline" class="text-xs gap-1.5" :disabled="loading" @click="load">
          <RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': loading }" />Muat Ulang
        </Button>
      </div>
    </div>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
    <Card v-else-if="items.length === 0" class="p-10 text-center text-sm text-muted-foreground">
      <FileText class="w-8 h-8 mx-auto mb-2 text-slate-300" />
      Tidak ada siswa yang sedang mengerjakan apa pun saat ini.
    </Card>
    <div v-else class="space-y-2">
      <Card v-for="item in items" :key="`${item.kind}:${item.ref_id}`" class="p-4 flex items-center justify-between gap-3 flex-wrap">
        <div class="flex items-center gap-3 min-w-0">
          <div class="w-10 h-10 rounded-full bg-rose-100 flex items-center justify-center shrink-0">
            <Timer class="w-5 h-5 text-rose-600" />
          </div>
          <div class="min-w-0">
            <p class="font-semibold text-sm text-slate-900 truncate">{{ item.student_name }}</p>
            <p class="text-xs text-muted-foreground truncate">{{ item.title }}</p>
            <div class="flex items-center gap-2 mt-1 flex-wrap">
              <Badge variant="secondary" class="text-[10px]">{{ kindLabel(item.kind) }}</Badge>
              <span class="text-[11px] text-muted-foreground flex items-center gap-1">
                <Clock class="w-3 h-3" />Berjalan {{ elapsedLabel(item.started_at) }} — dimulai {{ formatDateTime(item.started_at) }}
              </span>
            </div>
          </div>
        </div>
        <Button
          size="sm"
          variant="outline"
          class="text-xs gap-1.5 text-rose-600 border-rose-200 hover:bg-rose-50 shrink-0"
          :disabled="resettingId === item.ref_id"
          @click="resetActivity(item)"
        >
          <RotateCcw class="w-3.5 h-3.5" />{{ resettingId === item.ref_id ? 'Mereset...' : 'Reset' }}
        </Button>
      </Card>
    </div>
  </div>
</template>
