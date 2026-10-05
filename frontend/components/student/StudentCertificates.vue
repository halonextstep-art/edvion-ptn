<script setup lang="ts">
// Menu "Sertifikat" — daftar semua Simulasi TO (UTBK/TKA) yang sudah diselesaikan siswa ini,
// masing-masing dengan tombol unduh sertifikat + laporan hasil (lihat
// utils/simulationCertificate.ts). Terpisah dari DrillingZone's "Lihat Hasil" (yang membuka
// layar hasil di [runId].vue) — ini adalah arsip khusus buat menemukan & mengunduh ulang PDF
// tanpa harus menelusuri katalog Drilling per exam_track.
import { Award, Download, Loader2, GraduationCap, Layers, Calendar } from 'lucide-vue-next'
import type { SimulationRunItem } from '~/types'
import { exportSimulationCertificate } from '~/utils/simulationCertificate'

const { simulationService } = useApi()
const { user } = useAuth()
const toast = useToast()

const loading = ref(true)
const runs = ref<SimulationRunItem[]>([])
const completedRuns = computed(() =>
  runs.value.filter((r) => r.status === 'completed').sort((a, b) => new Date(b.completed_at || b.started_at).getTime() - new Date(a.completed_at || a.started_at).getTime()),
)

const downloadingId = ref<string | null>(null)
async function download(run: SimulationRunItem) {
  downloadingId.value = run.id
  try {
    let stats = null
    try {
      stats = await simulationService.templateStats(run.template_id)
    } catch {
      // non-critical — sertifikat tetap dibuat tanpa baris pembanding
    }
    exportSimulationCertificate(user.value?.name || 'Siswa', user.value?.school_name || null, run, stats)
  } catch (e: any) {
    toast.error('Gagal membuat sertifikat', e?.message)
  } finally {
    downloadingId.value = null
  }
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString('id-ID', { day: 'numeric', month: 'long', year: 'numeric' })
}

onMounted(async () => {
  try {
    runs.value = await simulationService.listMyRuns()
  } catch (e: any) {
    toast.error('Gagal memuat daftar sertifikat', e?.message)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="max-w-3xl mx-auto">
    <div class="mb-6">
      <h2 class="text-xl font-black text-slate-900 flex items-center gap-2"><Award class="w-6 h-6 text-indigo-600" />Sertifikat & Laporan Hasil</h2>
      <p class="text-sm text-muted-foreground mt-1">Sertifikat dan laporan hasil analisis kemampuan untuk setiap Simulasi TO yang sudah kamu selesaikan.</p>
    </div>

    <div v-if="loading" class="flex items-center justify-center py-16 text-slate-400">
      <Loader2 class="w-6 h-6 animate-spin" />
    </div>

    <Card v-else-if="!completedRuns.length" class="p-10 text-center">
      <div class="w-14 h-14 rounded-2xl bg-slate-100 flex items-center justify-center mx-auto mb-4">
        <Award class="w-7 h-7 text-slate-300" />
      </div>
      <p class="font-semibold text-slate-700 mb-1">Belum ada sertifikat</p>
      <p class="text-sm text-muted-foreground">Selesaikan sebuah Simulasi TO (UTBK/TKA) di menu Drilling untuk mendapatkan sertifikat dan laporan hasilmu di sini.</p>
    </Card>

    <div v-else class="space-y-3">
      <Card v-for="r in completedRuns" :key="r.id" class="p-4 hover:shadow-lg transition-shadow">
        <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4">
          <div class="flex items-center gap-3 sm:gap-4 min-w-0 flex-1">
            <div class="w-11 h-11 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shrink-0">
              <GraduationCap class="w-5 h-5 text-white" />
            </div>
            <div class="flex-1 min-w-0">
              <h4 class="font-bold text-sm text-slate-900 truncate">{{ r.template_title }}</h4>
              <div class="flex items-center gap-x-3 gap-y-1 text-xs text-muted-foreground mt-0.5 flex-wrap">
                <span class="flex items-center gap-1"><Calendar class="w-3 h-3" />{{ r.completed_at ? formatDate(r.completed_at) : '-' }}</span>
                <span class="flex items-center gap-1"><Layers class="w-3 h-3" />{{ r.slots.length }} subtes</span>
                <span v-if="r.template_kind === 'utbk_combined' && r.combined_estimate_score != null" class="font-semibold text-indigo-600">
                  Skor: {{ Math.round(r.combined_estimate_score) }}
                </span>
              </div>
            </div>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <NuxtLink :to="`/simulasi/${r.id}`" class="flex-1 sm:flex-none"><Button variant="outline" size="sm" class="w-full">Lihat Hasil</Button></NuxtLink>
            <Button variant="gradient" size="sm" class="flex-1 sm:flex-none" :disabled="downloadingId === r.id" @click="download(r)">
              <Download class="w-3.5 h-3.5" />{{ downloadingId === r.id ? 'Membuat...' : 'Unduh Sertifikat' }}
            </Button>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>
