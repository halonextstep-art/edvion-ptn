<script setup lang="ts">
// Widget "Deadline Mendatang" — dipakai di Overview Sekolah & Siswa (Admin punya CRUD
// penuh di tab Kalender Deadline). Data real dari GET /api/deadlines/upcoming, dihitung
// live oleh backend (days_remaining) — tidak ada tanggal yang di-hardcode di sini.
import { CalendarClock, AlertTriangle, Clock } from 'lucide-vue-next'
import type { AdmissionDeadlineItem, AdmissionTrack } from '~/types'

const { admissionDeadlineService } = useApi()
const toast = useToast()

const deadlines = ref<AdmissionDeadlineItem[]>([])
const loading = ref(true)

async function load() {
  loading.value = true
  try {
    deadlines.value = await admissionDeadlineService.upcoming(4)
  } catch (e: any) {
    toast.error('Gagal memuat kalender deadline', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

const trackConfig: Record<AdmissionTrack, { label: string; color: string; bg: string }> = {
  snbp: { label: 'SNBP', color: 'text-emerald-700', bg: 'bg-emerald-100' },
  snbt: { label: 'SNBT', color: 'text-blue-700', bg: 'bg-blue-100' },
  utbk: { label: 'UTBK', color: 'text-purple-700', bg: 'bg-purple-100' },
}

function formatDate(d: string) {
  return new Date(d).toLocaleDateString('id-ID', { day: '2-digit', month: 'short', year: 'numeric' })
}
function countdownLabel(days: number) {
  if (days === 0) return 'Hari ini!'
  if (days === 1) return 'Besok'
  return `${days} hari lagi`
}
function countdownVariant(days: number): 'destructive' | 'warning' | 'success' {
  if (days <= 3) return 'destructive'
  if (days <= 14) return 'warning'
  return 'success'
}
</script>

<template>
  <Card class="p-5">
    <h4 class="font-bold text-slate-900 mb-3 flex items-center gap-2"><CalendarClock class="w-4 h-4 text-indigo-600" /> Deadline Mendatang</h4>
    <div v-if="loading" class="py-8 text-center text-sm text-muted-foreground">Memuat...</div>
    <div v-else-if="deadlines.length === 0" class="py-8 text-center text-sm text-muted-foreground">Belum ada deadline SNBP/SNBT/UTBK yang dijadwalkan.</div>
    <div v-else class="space-y-2">
      <div v-for="d in deadlines" :key="d.id" class="flex items-center gap-3 p-2.5 rounded-xl hover:bg-slate-50">
        <div class="w-9 h-9 rounded-lg flex items-center justify-center shrink-0" :class="trackConfig[d.track].bg">
          <CalendarClock class="w-4 h-4" :class="trackConfig[d.track].color" />
        </div>
        <div class="min-w-0 flex-1">
          <p class="text-sm font-semibold text-slate-900 truncate">{{ d.label }}</p>
          <p class="text-[11px] text-muted-foreground">{{ trackConfig[d.track].label }} {{ d.year }} &middot; {{ formatDate(d.deadline_date) }}</p>
        </div>
        <Badge :variant="countdownVariant(d.days_remaining)" class="gap-1 shrink-0">
          <AlertTriangle v-if="d.days_remaining <= 3" class="w-3 h-3" />
          <Clock v-else class="w-3 h-3" />
          {{ countdownLabel(d.days_remaining) }}
        </Badge>
      </div>
    </div>
  </Card>
</template>
