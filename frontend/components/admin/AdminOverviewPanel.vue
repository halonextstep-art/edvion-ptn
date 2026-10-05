<script setup lang="ts">
import {
  Users, School as SchoolIcon, FileText, TrendingUp, Activity, ArrowUp, Clock, CheckCircle2, Calendar, Loader2,
} from 'lucide-vue-next'
import type { AnalyticsSummary, EventItem, ScoreTrendPoint, SubjectAccuracy } from '~/types'

const emit = defineEmits<{ navigate: [tab: string] }>()

const { questionService, schoolService, eventService, analyticsService } = useApi()
const toast = useToast()

const loading = ref(true)
const totalQuestions = ref(0)
const pendingReview = ref(0)
const schoolCount = ref(0)
const summary = ref<AnalyticsSummary | null>(null)
const trend = ref<ScoreTrendPoint[]>([])
const subjects = ref<SubjectAccuracy[]>([])
const allEvents = ref<EventItem[]>([])
const recentEvents = computed(() =>
  [...allEvents.value].sort((a, b) => new Date(b.start_date).getTime() - new Date(a.start_date).getTime()).slice(0, 4),
)
const activeEventCount = computed(() => allEvents.value.filter((e) => e.status !== 'completed' && e.status !== 'archived').length)

const eventStatusMeta: Record<string, { label: string; dot: string; text: string }> = {
  ongoing: { label: 'Berlangsung', dot: 'bg-green-500', text: 'text-green-600 bg-green-50' },
  completed: { label: 'Selesai', dot: 'bg-blue-500', text: 'text-blue-600 bg-blue-50' },
  upcoming: { label: 'Dijadwalkan', dot: 'bg-orange-500', text: 'text-orange-600 bg-orange-50' },
  draft: { label: 'Draft', dot: 'bg-slate-400', text: 'text-slate-500 bg-slate-100' },
  archived: { label: 'Diarsipkan', dot: 'bg-slate-300', text: 'text-slate-400 bg-slate-50' },
}

async function load() {
  loading.value = true
  try {
    const [qRes, reviewRes, schools, s, t, subj, events] = await Promise.all([
      questionService.list({ page: 1, page_size: 1 }),
      questionService.list({ status: 'review', page: 1, page_size: 1 }),
      schoolService.list(),
      analyticsService.summary(),
      analyticsService.scoreTrend(7),
      analyticsService.subjectBreakdown(),
      eventService.list(),
    ])
    totalQuestions.value = qRes.total
    pendingReview.value = reviewRes.total
    schoolCount.value = schools.length
    summary.value = s
    trend.value = t
    subjects.value = subj
    allEvents.value = events
  } catch (e: any) {
    toast.error('Gagal memuat ringkasan overview', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

// Data chart untuk Aktivitas Pengerjaan & Distribusi Mata Uji
const trendChartLabels = computed(() => trend.value.map((t) => formatDay(t.day)))
const trendChartSeries = computed(() => [{ label: 'Attempt', data: trend.value.map((t) => t.attempts), color: '#4f46e5' }])
const subjectChartLabels = computed(() => subjects.value.map((s) => s.subject))
const subjectChartValues = computed(() => subjects.value.map((s) => s.total))

function formatDay(d: string) {
  return new Date(d).toLocaleDateString('id-ID', { weekday: 'short' })
}
function formatDate(d: string) {
  return new Date(d).toLocaleDateString('id-ID', { day: '2-digit', month: 'short' })
}
</script>

<template>
  <div class="space-y-6">
    <div v-if="loading" class="py-10 text-center text-muted-foreground">
      <Loader2 class="w-6 h-6 animate-spin mx-auto mb-2" /> Memuat overview...
    </div>

    <template v-else>
      <!-- Quick Stats -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card class="p-6 border-l-4 border-l-indigo-500 hover:shadow-lg transition-shadow">
          <div class="flex items-start justify-between">
            <div>
              <p class="text-sm text-muted-foreground mb-1">Siswa Terdaftar</p>
              <h3 class="text-3xl mb-2 font-black">{{ (summary?.total_students ?? 0).toLocaleString('id-ID') }}</h3>
              <div class="flex items-center gap-1 text-sm text-green-600">
                <ArrowUp class="w-4 h-4" /><span>{{ summary?.submitted_attempts ?? 0 }} tryout/drilling selesai</span>
              </div>
            </div>
            <div class="w-12 h-12 rounded-xl bg-indigo-100 flex items-center justify-center"><Users class="w-6 h-6 text-indigo-600" /></div>
          </div>
        </Card>

        <Card class="p-6 border-l-4 border-l-purple-500 hover:shadow-lg transition-shadow">
          <div class="flex items-start justify-between">
            <div>
              <p class="text-sm text-muted-foreground mb-1">Sekolah Mitra</p>
              <h3 class="text-3xl mb-2 font-black">{{ schoolCount }}</h3>
              <div class="flex items-center gap-1 text-sm text-muted-foreground"><span>terdaftar</span></div>
            </div>
            <div class="w-12 h-12 rounded-xl bg-purple-100 flex items-center justify-center"><SchoolIcon class="w-6 h-6 text-purple-600" /></div>
          </div>
        </Card>

        <Card class="p-6 border-l-4 border-l-pink-500 hover:shadow-lg transition-shadow">
          <div class="flex items-start justify-between">
            <div>
              <p class="text-sm text-muted-foreground mb-1">Bank Soal</p>
              <h3 class="text-3xl mb-2 font-black">{{ totalQuestions }}</h3>
              <div class="flex items-center gap-1 text-sm text-amber-600"><Activity class="w-4 h-4" /><span>{{ pendingReview }} menunggu review</span></div>
            </div>
            <div class="w-12 h-12 rounded-xl bg-pink-100 flex items-center justify-center"><FileText class="w-6 h-6 text-pink-600" /></div>
          </div>
        </Card>

        <Card class="p-6 border-l-4 border-l-orange-500 hover:shadow-lg transition-shadow">
          <div class="flex items-start justify-between">
            <div>
              <p class="text-sm text-muted-foreground mb-1">Event Aktif</p>
              <h3 class="text-3xl mb-2 font-black">{{ activeEventCount }}</h3>
              <div class="flex items-center gap-1 text-sm text-orange-600"><TrendingUp class="w-4 h-4" /><span>lihat semua di tab Event</span></div>
            </div>
            <div class="w-12 h-12 rounded-xl bg-orange-100 flex items-center justify-center"><TrendingUp class="w-6 h-6 text-orange-600" /></div>
          </div>
        </Card>
      </div>

      <!-- Charts row -->
      <div class="grid lg:grid-cols-3 gap-6">
        <Card class="lg:col-span-2 p-6">
          <div class="flex items-center justify-between mb-6">
            <div>
              <h3 class="text-lg font-bold mb-1">Aktivitas Pengerjaan (7 Hari Terakhir)</h3>
              <p class="text-sm text-muted-foreground">Jumlah attempt tryout/drilling yang disubmit per hari</p>
            </div>
            <Button variant="outline" size="sm" @click="emit('navigate', 'analytics')">Lihat Detail</Button>
          </div>

          <div v-if="trend.every((t) => t.attempts === 0)" class="py-16 text-center text-sm text-muted-foreground">
            Belum ada attempt yang disubmit dalam 7 hari terakhir.
          </div>
          <BarLineChart v-else :labels="trendChartLabels" :series="trendChartSeries" :height="224" />
        </Card>

        <Card class="p-6">
          <h3 class="text-lg font-bold mb-1">Distribusi Mata Uji</h3>
          <p class="text-sm text-muted-foreground mb-6">Berdasarkan jawaban siswa yang sudah dinilai</p>

          <DonutChart :labels="subjectChartLabels" :values="subjectChartValues" unit-label=" jawaban" :height="170" empty-label="Belum ada data." />
        </Card>
      </div>

      <!-- Recent events -->
      <Card class="p-6">
        <div class="flex items-center justify-between mb-6">
          <div>
            <h3 class="text-lg font-bold mb-1">Event Terkini</h3>
            <p class="text-sm text-muted-foreground">Status dan performa event terbaru</p>
          </div>
          <Button variant="outline" size="sm" @click="emit('navigate', 'events')">Lihat Semua</Button>
        </div>

        <div v-if="recentEvents.length === 0" class="py-10 text-center text-sm text-muted-foreground">
          Belum ada event. Buat event pertama di tab Event.
        </div>
        <div v-else class="space-y-3">
          <div v-for="ev in recentEvents" :key="ev.id" class="flex items-center justify-between p-4 bg-slate-50 rounded-lg hover:bg-slate-100 transition-colors">
            <div class="flex items-center gap-4 min-w-0">
              <div class="w-2 h-2 rounded-full shrink-0" :class="eventStatusMeta[ev.status]?.dot" />
              <div class="min-w-0">
                <h4 class="mb-1 font-semibold text-sm truncate">{{ ev.name }}</h4>
                <div class="flex items-center gap-3 text-sm text-muted-foreground flex-wrap">
                  <span class="flex items-center gap-1"><Users class="w-3.5 h-3.5" />{{ ev.participants.toLocaleString('id-ID') }} peserta</span>
                  <span class="px-2 py-0.5 bg-white rounded text-xs">{{ ev.event_type }}</span>
                  <span class="flex items-center gap-1 text-xs"><Calendar class="w-3 h-3" />{{ formatDate(ev.start_date) }}</span>
                </div>
              </div>
            </div>

            <div class="flex items-center gap-2 shrink-0">
              <div class="flex items-center gap-1 text-sm px-3 py-1.5 rounded-full" :class="eventStatusMeta[ev.status]?.text">
                <Activity v-if="ev.status === 'ongoing'" class="w-3.5 h-3.5" />
                <CheckCircle2 v-else-if="ev.status === 'completed'" class="w-3.5 h-3.5" />
                <Clock v-else class="w-3.5 h-3.5" />
                <span>{{ eventStatusMeta[ev.status]?.label }}</span>
              </div>
              <Button variant="ghost" size="sm" @click="emit('navigate', 'events')">Detail</Button>
            </div>
          </div>
        </div>
      </Card>
    </template>
  </div>
</template>
