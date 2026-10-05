<script setup lang="ts">
// Analytics & Insights Sekolah — v2. 100% data nyata, di-scope otomatis oleh backend ke
// siswa sekolah ini saja (lihat AnalyticsService::*_for_school di backend, di-join lewat
// users.school_id). Dikembangkan dari versi sebelumnya (subject breakdown + tren skor +
// distribusi target + kekuatan/peningkatan) supaya lebih bermakna dan actionable:
//  - Benchmark: tren skor sekolah dibandingkan langsung dengan rata-rata nasional (garis
//    overlay pada chart yang sama), bukan cuma angka sekolah sendiri tanpa pembanding.
//  - Ringkasan per Kelas: skor rata-rata dipecah per rombel (dari users.rombel_code) —
//    PIC sekolah bisa lihat kelas mana yang butuh perhatian, bukan cuma agregat sekolah.
//  - Insight & Rekomendasi: aturan sederhana dari data asli (siswa belum pernah
//    mengerjakan apa pun, siswa tidak aktif >14 hari, kelas di bawah rata-rata sekolah,
//    arah tren) — supaya dashboard ini menyarankan tindak lanjut konkret, bukan cuma
//    menampilkan angka. Setiap insight HANYA muncul kalau kondisinya benar-benar terjadi
//    pada data nyata (tidak pernah ditampilkan sebagai placeholder kosong).
//  - Tabel Siswa: drill-down penuh sampai level siswa individual (nama, kelas, jumlah
//    attempt, skor rata-rata, terakhir aktif, status), bisa difilter per kelas dan
//    diurutkan — sebelumnya cuma ada agregat sekolah, tidak ada cara melihat siswa mana
//    yang dimaksud.
//  - Ekspor PDF — lihat utils/schoolAnalyticsExport.ts.
import {
  TrendingUp, BarChart3, Loader2, AlertCircle, Target, ThumbsUp, AlertTriangle, Info, LineChart,
  Users, UserX, Clock, Download, ArrowUpRight, ArrowDownRight, Lightbulb, School as SchoolIcon, FileSpreadsheet,
  CalendarRange,
} from 'lucide-vue-next'
import type { SubjectAccuracy, ScoreTrendPoint, PtnDistributionItem, StudentActivityItem, YearlyPerformancePoint } from '~/types'
import { exportSchoolAnalyticsToPdf, exportSchoolAnalyticsToExcel } from '~/utils/schoolAnalyticsExport'

const { analyticsService, schoolService } = useApi()
const { user } = useAuth()
const toast = useToast()

const subjects = ref<SubjectAccuracy[]>([])
const trend = ref<ScoreTrendPoint[]>([])
const nationalTrend = ref<ScoreTrendPoint[]>([])
const distribution = ref<PtnDistributionItem[]>([])
const activity = ref<StudentActivityItem[]>([])
const yearlyPerf = ref<YearlyPerformancePoint[]>([])
const loading = ref(true)
const loadError = ref<string | null>(null)
const exporting = ref(false)
const exportingExcel = ref(false)
const trendLoading = ref(false)

const TIME_FILTERS = [
  { label: '7 hari', days: 7 },
  { label: '30 hari', days: 30 },
  { label: '3 bulan', days: 90 },
  { label: '1 tahun', days: 365 },
]
const selectedDays = ref(30)

async function load() {
  loading.value = true
  loadError.value = null
  try {
    const [s, t, nt, d, act, yp] = await Promise.all([
      analyticsService.schoolSubjectBreakdown(),
      analyticsService.schoolScoreTrend(selectedDays.value),
      analyticsService.nationalScoreTrend(selectedDays.value),
      schoolService.ptnDistribution(),
      analyticsService.schoolStudentActivity(),
      analyticsService.schoolScoreTrendByYear(),
    ])
    subjects.value = s
    trend.value = t
    nationalTrend.value = nt
    distribution.value = d
    activity.value = act
    yearlyPerf.value = yp
  } catch (e: any) {
    loadError.value = e?.message || 'Gagal memuat data analytics.'
    toast.error('Gagal memuat analytics', loadError.value)
  } finally {
    loading.value = false
  }
}
onMounted(load)

async function changeTimeFilter(days: number) {
  selectedDays.value = days
  trendLoading.value = true
  try {
    const [t, nt] = await Promise.all([
      analyticsService.schoolScoreTrend(days),
      analyticsService.nationalScoreTrend(days),
    ])
    trend.value = t
    nationalTrend.value = nt
  } catch (e: any) {
    toast.error('Gagal memuat tren skor', e?.message)
  } finally {
    trendLoading.value = false
  }
}

function accuracyPct(s: SubjectAccuracy) {
  return s.total > 0 ? Math.round((s.correct / s.total) * 100) : 0
}
function fmtDay(iso: string) {
  return new Date(iso).toLocaleDateString('id-ID', { day: '2-digit', month: 'short' })
}
function daysSince(iso: string) {
  return Math.floor((Date.now() - new Date(iso).getTime()) / 86400000)
}
function relativeActivity(iso: string | null) {
  if (!iso) return 'Belum pernah'
  const d = daysSince(iso)
  if (d <= 0) return 'Hari ini'
  if (d === 1) return 'Kemarin'
  if (d < 30) return `${d} hari lalu`
  const months = Math.floor(d / 30)
  return `${months} bulan lalu`
}

// ─── Subject mastery (existing) ────────────────────────────────────────────────────
const subjectChartLabels = computed(() => subjects.value.map((s) => s.subject))
const subjectChartSeries = computed(() => [{ label: 'Akurasi', data: subjects.value.map((s) => accuracyPct(s)), color: '#6366f1' }])
const sortedSubjects = computed(() => [...subjects.value].sort((a, b) => accuracyPct(b) - accuracyPct(a)))
const strengths = computed(() => sortedSubjects.value.filter((s) => accuracyPct(s) >= 70).slice(0, 3))
const improvements = computed(() => [...sortedSubjects.value].reverse().filter((s) => accuracyPct(s) < 70).slice(0, 3))

// ─── Distribusi target PTN (existing) ──────────────────────────────────────────────
const distributionChartLabels = computed(() => distribution.value.map((d) => d.nama_ptn))
const distributionChartValues = computed(() => distribution.value.map((d) => d.count))

// ─── Benchmark: tren skor sekolah vs rata-rata nasional, digabung dalam 1 chart ────
// `trend`/`nationalTrend` dari backend cuma berisi hari yang benar-benar ada attempt (sparse),
// jadi digabung dulu ke rentang `trendDays` hari kalender penuh supaya kedua garis sejajar
// per tanggal yang sama — hari tanpa data biarkan `null` (Chart.js melompatinya, bukan
// digambar sebagai 0 yang menyesatkan).
// Cap kalender penuh ke 90 hari maksimum meski filter "1 tahun" dipilih — sejalan dengan
// AnalyticsPanel.vue (Admin) yang membatasi bar chart harian ke 45 titik: 365 titik kalender
// per hari akan membuat setiap bar terlalu tipis untuk dibaca. Sparse points di luar jendela
// ini tetap dihitung ke rata-rata/insight lain, hanya tidak digambar per-hari di sini.
const trendDays = computed(() => {
  const windowDays = Math.min(selectedDays.value, 90)
  const days: string[] = []
  const today = new Date()
  for (let i = windowDays - 1; i >= 0; i--) {
    const d = new Date(today)
    d.setDate(d.getDate() - i)
    days.push(d.toISOString().slice(0, 10))
  }
  return days
})
function trendMapByDay(points: ScoreTrendPoint[]) {
  const m = new Map<string, number>()
  for (const p of points) m.set(p.day.slice(0, 10), Math.round(p.average_score))
  return m
}
const benchmarkLabels = computed(() => trendDays.value.map(fmtDay))
const benchmarkSeries = computed(() => {
  const schoolMap = trendMapByDay(trend.value)
  const nationalMap = trendMapByDay(nationalTrend.value)
  return [
    { label: 'Skor Sekolah Ini', data: trendDays.value.map((d) => schoolMap.get(d) ?? null), color: '#4f46e5' },
    { label: 'Rata-rata Nasional', data: trendDays.value.map((d) => nationalMap.get(d) ?? null), color: '#f59e0b', type: 'line' as const },
  ]
})

// ─── KPI: aktivitas siswa & benchmark rata-rata ────────────────────────────────────
const totalStudents = computed(() => activity.value.length)
const neverStarted = computed(() => activity.value.filter((a) => a.attempts_count === 0))
const activeStudents = computed(() => activity.value.filter((a) => a.attempts_count > 0))
const inactiveStudents = computed(() =>
  activeStudents.value.filter((a) => a.last_attempt_at && daysSince(a.last_attempt_at) > 14),
)
const schoolOverallAvg = computed(() => {
  const withScore = activity.value.filter((a): a is StudentActivityItem & { average_score: number } => a.average_score != null)
  if (!withScore.length) return null
  return Math.round(withScore.reduce((sum, a) => sum + a.average_score, 0) / withScore.length)
})
const nationalOverallAvg = computed(() => {
  if (!nationalTrend.value.length) return null
  return Math.round(nationalTrend.value.reduce((sum, t) => sum + t.average_score, 0) / nationalTrend.value.length)
})
const avgDelta = computed(() =>
  schoolOverallAvg.value != null && nationalOverallAvg.value != null ? schoolOverallAvg.value - nationalOverallAvg.value : null,
)

const kpis = computed(() => [
  { label: 'Total Siswa', val: totalStudents.value, icon: Users, tone: 'indigo' as const },
  { label: 'Belum Pernah Mengerjakan', val: neverStarted.value.length, icon: UserX, tone: totalStudents.value && neverStarted.value.length > 0 ? 'amber' as const : 'slate' as const },
  { label: 'Tidak Aktif >14 Hari', val: inactiveStudents.value.length, icon: Clock, tone: inactiveStudents.value.length > 0 ? 'rose' as const : 'slate' as const },
])

// ─── Ringkasan per Kelas ────────────────────────────────────────────────────────────
interface KelasSummary { rombel: string; total: number; aktif: number; avgScore: number | null }
const kelasSummaries = computed<KelasSummary[]>(() => {
  const map = new Map<string, StudentActivityItem[]>()
  for (const a of activity.value) {
    const key = a.rombel_code || 'Tanpa Kelas'
    if (!map.has(key)) map.set(key, [])
    map.get(key)!.push(a)
  }
  return Array.from(map.entries())
    .map(([rombel, students]) => {
      const withScore = students.filter((s) => s.average_score != null)
      const avgScore = withScore.length
        ? Math.round(withScore.reduce((sum, s) => sum + (s.average_score as number), 0) / withScore.length)
        : null
      return { rombel, total: students.length, aktif: students.filter((s) => s.attempts_count > 0).length, avgScore }
    })
    .sort((a, b) => (b.avgScore ?? -1) - (a.avgScore ?? -1))
})
const kelasWithScore = computed(() => kelasSummaries.value.filter((k) => k.avgScore != null))
const kelasChartLabels = computed(() => kelasWithScore.value.map((k) => k.rombel))
const kelasChartSeries = computed(() => [{ label: 'Skor Rata-rata', data: kelasWithScore.value.map((k) => k.avgScore as number), color: '#6366f1' }])
const kelasBelowSchoolAvg = computed(() => {
  if (schoolOverallAvg.value == null) return []
  return kelasWithScore.value.filter((k) => (k.avgScore as number) < (schoolOverallAvg.value as number) - 5)
})

// ─── Perbandingan Tahun — attempts dikelompokkan per tahun kalender submitted_at (bukan
// angkatan/kohort siswa, karena belum ada field kohort yang andal di sistem — lihat doc
// comment AnalyticsService::score_trend_by_year_for_school di backend). Hanya tahun yang
// benar-benar punya minimal 1 attempt selesai yang muncul. ─────────────────────────────
const yearlyChartLabels = computed(() => yearlyPerf.value.map((y) => String(y.year)))
const yearlyChartSeries = computed(() => [{ label: 'Skor Rata-rata', data: yearlyPerf.value.map((y) => Math.round(y.average_score)), color: '#0891b2' }])
function yearOverYearDelta(index: number) {
  if (index === 0) return null
  const prev = yearlyPerf.value[index - 1]
  const cur = yearlyPerf.value[index]
  return Math.round(cur.average_score - prev.average_score)
}

// ─── Arah tren (naik/turun/stabil) — dari titik tren sekolah sendiri yang benar-benar
// punya attempt, bandingkan paruh awal vs paruh akhir periode. ─────────────────────
const trendDirection = computed(() => {
  const pts = trend.value.filter((t) => t.attempts > 0)
  if (pts.length < 4) return null
  const mid = Math.floor(pts.length / 2)
  const avg = (arr: ScoreTrendPoint[]) => arr.reduce((s, p) => s + p.average_score, 0) / arr.length
  const delta = Math.round(avg(pts.slice(mid)) - avg(pts.slice(0, mid)))
  return { delta, direction: delta > 2 ? 'naik' : delta < -2 ? 'turun' : 'stabil' as 'naik' | 'turun' | 'stabil' }
})

// ─── Insight & Rekomendasi — cuma tampil kalau kondisinya benar-benar terjadi pada
// data nyata, tidak pernah sebagai placeholder. ─────────────────────────────────────
interface Insight { tone: 'warning' | 'positive' | 'info'; text: string }
const insights = computed<Insight[]>(() => {
  const list: Insight[] = []
  if (neverStarted.value.length > 0) {
    list.push({
      tone: 'warning',
      text: `${neverStarted.value.length} siswa belum pernah mengerjakan tryout/drilling sama sekali${neverStarted.value.length <= 5 ? `: ${neverStarted.value.map((s) => s.student_name).join(', ')}` : ''}. Pertimbangkan follow-up langsung.`,
    })
  }
  if (inactiveStudents.value.length > 0) {
    list.push({
      tone: 'warning',
      text: `${inactiveStudents.value.length} siswa sudah pernah mulai tapi tidak aktif lebih dari 14 hari terakhir.`,
    })
  }
  for (const k of kelasBelowSchoolAvg.value) {
    list.push({
      tone: 'warning',
      text: `Kelas ${k.rombel} rata-rata skornya ${k.avgScore} — lebih dari 5 poin di bawah rata-rata sekolah (${schoolOverallAvg.value}).`,
    })
  }
  if (trendDirection.value?.direction === 'naik') {
    list.push({ tone: 'positive', text: `Tren skor sekolah naik ${trendDirection.value.delta} poin dibanding paruh awal periode ${selectedDays.value} hari ini.` })
  } else if (trendDirection.value?.direction === 'turun') {
    list.push({ tone: 'warning', text: `Tren skor sekolah turun ${Math.abs(trendDirection.value.delta)} poin dibanding paruh awal periode ${selectedDays.value} hari ini — perlu diperhatikan.` })
  }
  if (avgDelta.value != null) {
    if (avgDelta.value >= 5) list.push({ tone: 'positive', text: `Rata-rata skor sekolah ${avgDelta.value} poin di atas rata-rata nasional.` })
    else if (avgDelta.value <= -5) list.push({ tone: 'warning', text: `Rata-rata skor sekolah ${Math.abs(avgDelta.value)} poin di bawah rata-rata nasional.` })
  }
  return list
})
const insightIcon = (tone: Insight['tone']) => (tone === 'positive' ? ThumbsUp : tone === 'warning' ? AlertTriangle : Info)
const insightClass = (tone: Insight['tone']) =>
  tone === 'positive' ? 'border-emerald-200 bg-emerald-50/60 text-emerald-900'
  : tone === 'warning' ? 'border-amber-200 bg-amber-50/60 text-amber-900'
  : 'border-sky-200 bg-sky-50/60 text-sky-900'

// ─── Tabel Siswa (drill-down) — filter per kelas + urut ────────────────────────────
// Template ref pada komponen <Card> (bukan elemen DOM native) — Vue memberi instance publik
// komponen, bukan node HTML langsung, jadi diakses lewat `.$el` untuk scrollIntoView.
const studentTableAnchor = ref<{ $el: HTMLElement } | null>(null)
/** Klik bar kelas di chart "Ringkasan per Kelas" — langsung filter Tabel Siswa ke kelas
 *  itu dan scroll ke sana, supaya PIC sekolah tidak perlu cari-cari dropdown terpisah. */
function filterByKelas(rombel: string) {
  kelasFilter.value = rombel
  studentTableAnchor.value?.$el?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}
const kelasFilter = ref<string>('semua')
const kelasOptions = computed(() => ['semua', ...kelasSummaries.value.map((k) => k.rombel)])
const sortBy = ref<'nama' | 'skor_tertinggi' | 'skor_terendah' | 'aktivitas_terbaru'>('nama')
const filteredActivity = computed(() => {
  let list = activity.value
  if (kelasFilter.value !== 'semua') list = list.filter((a) => (a.rombel_code || 'Tanpa Kelas') === kelasFilter.value)
  const sorted = [...list]
  if (sortBy.value === 'skor_tertinggi') sorted.sort((a, b) => (b.average_score ?? -1) - (a.average_score ?? -1))
  else if (sortBy.value === 'skor_terendah') sorted.sort((a, b) => (a.average_score ?? 9999) - (b.average_score ?? 9999))
  else if (sortBy.value === 'aktivitas_terbaru') sorted.sort((a, b) => new Date(b.last_attempt_at || 0).getTime() - new Date(a.last_attempt_at || 0).getTime())
  else sorted.sort((a, b) => a.student_name.localeCompare(b.student_name))
  return sorted
})
function statusFor(a: StudentActivityItem): { label: string; cls: string } {
  if (a.attempts_count === 0) return { label: 'Belum Mulai', cls: 'bg-slate-100 text-slate-600' }
  if (a.last_attempt_at && daysSince(a.last_attempt_at) > 14) return { label: 'Tidak Aktif', cls: 'bg-amber-100 text-amber-700' }
  return { label: 'Aktif', cls: 'bg-emerald-100 text-emerald-700' }
}

// ─── Ekspor PDF & Excel ─────────────────────────────────────────────────────────────
function buildExportData() {
  return {
    schoolName: user.value?.school_name || 'Sekolah',
    generatedAt: new Date().toISOString(),
    kpis: [
      { label: 'Total Siswa', value: String(totalStudents.value) },
      { label: 'Siswa Aktif', value: String(activeStudents.value.length) },
      { label: 'Belum Pernah Mengerjakan', value: String(neverStarted.value.length) },
      { label: 'Tidak Aktif >14 Hari', value: String(inactiveStudents.value.length) },
      { label: 'Rata-rata Sekolah', value: schoolOverallAvg.value != null ? String(schoolOverallAvg.value) : '-' },
      { label: 'Rata-rata Nasional', value: nationalOverallAvg.value != null ? String(nationalOverallAvg.value) : '-' },
    ],
    kelasSummaries: kelasSummaries.value,
    yearlyPerformance: yearlyPerf.value.map((y) => ({ year: y.year, attempts: y.attempts, distinctStudents: y.distinct_students, avgScore: Math.round(y.average_score) })),
    insights: insights.value.map((i) => i.text),
    subjects: subjects.value.map((s) => ({ subject: s.subject, accuracy: accuracyPct(s) })),
    students: filteredActivity.value.map((a) => ({
      name: a.student_name,
      rombel: a.rombel_code || '-',
      attempts: a.attempts_count,
      avgScore: a.average_score != null ? Math.round(a.average_score) : null,
      status: statusFor(a).label,
    })),
  }
}
async function exportPdf() {
  exporting.value = true
  try {
    exportSchoolAnalyticsToPdf(buildExportData())
  } catch (e: any) {
    toast.error('Gagal mengekspor PDF', e?.message)
  } finally {
    exporting.value = false
  }
}
async function exportExcel() {
  exportingExcel.value = true
  try {
    exportSchoolAnalyticsToExcel(buildExportData())
  } catch (e: any) {
    toast.error('Gagal mengekspor Excel', e?.message)
  } finally {
    exportingExcel.value = false
  }
}
</script>

<template>
  <div class="space-y-6">
    <div v-if="loading" class="py-16 text-center text-muted-foreground">
      <Loader2 class="w-6 h-6 animate-spin mx-auto mb-2" /> Memuat analytics...
    </div>

    <Card v-else-if="loadError" class="p-8 text-center border-red-200 bg-red-50">
      <AlertCircle class="w-8 h-8 mx-auto mb-2 text-red-500" />
      <p class="text-sm text-red-700 mb-3">{{ loadError }}</p>
      <Button variant="outline" size="sm" @click="load">Coba Lagi</Button>
    </Card>

    <template v-else>
      <Card class="p-5 flex items-center justify-between flex-wrap gap-3">
        <div>
          <h2 class="text-lg font-bold flex items-center gap-2"><LineChart class="w-5 h-5 text-blue-600" />Analytics &amp; Insights</h2>
          <p class="text-sm text-muted-foreground mt-0.5">Analisis performa siswa, kelas, dan sebaran target PTN sekolah ini — seluruhnya dari data attempt &amp; target nyata.</p>
        </div>
        <div class="flex items-center gap-2 flex-wrap">
          <div class="flex gap-1.5">
            <button
              v-for="f in TIME_FILTERS"
              :key="f.label"
              class="px-3 py-1.5 rounded-lg text-xs font-semibold transition-colors"
              :class="selectedDays === f.days ? 'bg-indigo-600 text-white' : 'bg-white border text-slate-600 hover:bg-slate-50'"
              @click="changeTimeFilter(f.days)"
            >
              {{ f.label }}
            </button>
          </div>
          <Button variant="outline" size="sm" class="gap-1.5" :disabled="exporting" @click="exportPdf">
            <Loader2 v-if="exporting" class="w-3.5 h-3.5 animate-spin" /><Download v-else class="w-3.5 h-3.5" />PDF
          </Button>
          <Button variant="outline" size="sm" class="gap-1.5" :disabled="exportingExcel" @click="exportExcel">
            <Loader2 v-if="exportingExcel" class="w-3.5 h-3.5 animate-spin" /><FileSpreadsheet v-else class="w-3.5 h-3.5" />Excel
          </Button>
        </div>
      </Card>

      <Card class="p-4 bg-gradient-to-br from-blue-50 to-cyan-50 border-blue-200">
        <p class="text-sm font-bold text-blue-800 mb-1 flex items-center gap-1.5"><Info class="w-4 h-4" />Informasi Penting</p>
        <p class="text-xs text-blue-800/80">Bank soal dan kualitas materi tryout dikelola terpusat oleh Admin Pusat. Sekolah berfokus memantau performa siswa sendiri dan menindaklanjuti hasil analitik di bawah ini.</p>
      </Card>

      <!-- KPI strip -->
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card v-for="k in kpis" :key="k.label" class="p-5">
          <div class="flex items-start justify-between mb-3">
            <div
              class="w-10 h-10 rounded-xl flex items-center justify-center"
              :class="k.tone === 'rose' ? 'bg-rose-50' : k.tone === 'amber' ? 'bg-amber-50' : k.tone === 'indigo' ? 'bg-indigo-50' : 'bg-slate-100'"
            >
              <component :is="k.icon" class="w-5 h-5" :class="k.tone === 'rose' ? 'text-rose-600' : k.tone === 'amber' ? 'text-amber-600' : k.tone === 'indigo' ? 'text-indigo-600' : 'text-slate-500'" />
            </div>
          </div>
          <p class="text-2xl font-black text-slate-900">{{ k.val.toLocaleString('id-ID') }}</p>
          <p class="text-xs text-muted-foreground mt-0.5">{{ k.label }}</p>
        </Card>
        <Card class="p-5">
          <div class="flex items-start justify-between mb-3">
            <div class="w-10 h-10 rounded-xl bg-indigo-50 flex items-center justify-center"><SchoolIcon class="w-5 h-5 text-indigo-600" /></div>
            <Badge v-if="avgDelta != null" :class="avgDelta >= 0 ? 'bg-emerald-100 text-emerald-700' : 'bg-rose-100 text-rose-700'" class="gap-1">
              <component :is="avgDelta >= 0 ? ArrowUpRight : ArrowDownRight" class="w-3 h-3" />{{ avgDelta >= 0 ? '+' : '' }}{{ avgDelta }}
            </Badge>
          </div>
          <p class="text-2xl font-black text-slate-900">{{ schoolOverallAvg ?? '-' }}</p>
          <p class="text-xs text-muted-foreground mt-0.5">Rata-rata Sekolah <span v-if="nationalOverallAvg != null">(nasional: {{ nationalOverallAvg }})</span></p>
        </Card>
      </div>

      <!-- Insight & Rekomendasi -->
      <Card v-if="insights.length" class="p-5">
        <h4 class="font-bold text-slate-900 mb-3 flex items-center gap-2"><Lightbulb class="w-4 h-4 text-amber-500" /> Insight &amp; Rekomendasi</h4>
        <div class="space-y-2">
          <div v-for="(ins, i) in insights" :key="i" class="flex items-start gap-2.5 rounded-lg border p-3 text-sm" :class="insightClass(ins.tone)">
            <component :is="insightIcon(ins.tone)" class="w-4 h-4 shrink-0 mt-0.5" />
            <span>{{ ins.text }}</span>
          </div>
        </div>
      </Card>

      <!-- Benchmark: tren sekolah vs nasional -->
      <Card class="p-5">
        <h4 class="font-bold text-slate-900 mb-1 flex items-center gap-2"><TrendingUp class="w-4 h-4 text-indigo-600" /> Tren Skor: Sekolah vs Rata-rata Nasional ({{ TIME_FILTERS.find((f) => f.days === selectedDays)?.label || `${selectedDays} hari` }})</h4>
        <p class="text-xs text-muted-foreground mb-4">Garis oranye = rata-rata nasional (anonim, semua sekolah) sebagai pembanding.</p>
        <div v-if="trendLoading" class="py-16 text-center text-sm text-muted-foreground">Memuat tren...</div>
        <BarLineChart v-else :labels="benchmarkLabels" :series="benchmarkSeries" :height="220" empty-label="Belum ada attempt selesai dalam periode ini." />
      </Card>

      <!-- Ringkasan per Kelas -->
      <Card v-if="kelasSummaries.length" class="p-5">
        <h4 class="font-bold text-slate-900 mb-1 flex items-center gap-2"><Users class="w-4 h-4 text-indigo-600" /> Ringkasan per Kelas</h4>
        <p class="text-xs text-muted-foreground mb-4">Skor rata-rata dipecah per kelas (rombel) — hanya siswa yang sudah punya minimal 1 attempt yang dihitung. Klik salah satu bar untuk memfilter Daftar Siswa ke kelas itu.</p>
        <BarLineChart
          v-if="kelasWithScore.length"
          :labels="kelasChartLabels" :series="kelasChartSeries" :horizontal="true" clickable
          :height="Math.max(kelasWithScore.length * 42, 100)" empty-label="Belum ada data skor per kelas."
          @bar-click="filterByKelas"
        />
        <div class="overflow-x-auto mt-4">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b bg-slate-50">
                <th class="text-left px-3 py-2 font-semibold text-slate-600">Kelas</th>
                <th class="text-left px-3 py-2 font-semibold text-slate-600">Total Siswa</th>
                <th class="text-left px-3 py-2 font-semibold text-slate-600">Siswa Aktif</th>
                <th class="text-left px-3 py-2 font-semibold text-slate-600">Skor Rata-rata</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="k in kelasSummaries" :key="k.rombel" class="border-b hover:bg-slate-50 cursor-pointer" @click="filterByKelas(k.rombel)">
                <td class="px-3 py-2 font-semibold text-slate-900">{{ k.rombel }}</td>
                <td class="px-3 py-2 text-slate-700">{{ k.total }}</td>
                <td class="px-3 py-2 text-slate-700">{{ k.aktif }}</td>
                <td class="px-3 py-2 font-bold" :class="k.avgScore != null ? 'text-indigo-600' : 'text-slate-400'">{{ k.avgScore ?? '-' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </Card>

      <!-- Perbandingan Tahun -->
      <Card v-if="yearlyPerf.length" class="p-5">
        <h4 class="font-bold text-slate-900 mb-1 flex items-center gap-2"><CalendarRange class="w-4 h-4 text-cyan-600" /> Perbandingan Tahun</h4>
        <p class="text-xs text-muted-foreground mb-4">Skor rata-rata per tahun kalender attempt dikerjakan (bukan angkatan/kohort siswa — data kohort belum tersedia lengkap di sistem).</p>
        <BarLineChart
          v-if="yearlyPerf.length > 1"
          :labels="yearlyChartLabels" :series="yearlyChartSeries"
          :height="180" empty-label="Belum cukup data untuk dibandingkan."
        />
        <div v-else class="py-6 text-center text-sm text-muted-foreground">Baru ada data 1 tahun ({{ yearlyPerf[0].year }}) — perbandingan akan muncul setelah ada attempt di tahun lain.</div>
        <div class="overflow-x-auto mt-4">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b bg-slate-50">
                <th class="text-left px-3 py-2 font-semibold text-slate-600">Tahun</th>
                <th class="text-left px-3 py-2 font-semibold text-slate-600">Jumlah Attempt</th>
                <th class="text-left px-3 py-2 font-semibold text-slate-600">Siswa Aktif</th>
                <th class="text-left px-3 py-2 font-semibold text-slate-600">Skor Rata-rata</th>
                <th class="text-left px-3 py-2 font-semibold text-slate-600">vs Tahun Sebelumnya</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(y, i) in yearlyPerf" :key="y.year" class="border-b">
                <td class="px-3 py-2 font-semibold text-slate-900">{{ y.year }}</td>
                <td class="px-3 py-2 text-slate-700">{{ y.attempts.toLocaleString('id-ID') }}</td>
                <td class="px-3 py-2 text-slate-700">{{ y.distinct_students.toLocaleString('id-ID') }}</td>
                <td class="px-3 py-2 font-bold text-cyan-700">{{ Math.round(y.average_score) }}</td>
                <td class="px-3 py-2">
                  <span v-if="yearOverYearDelta(i) == null" class="text-slate-400 text-xs">—</span>
                  <Badge v-else :class="yearOverYearDelta(i)! >= 0 ? 'bg-emerald-100 text-emerald-700' : 'bg-rose-100 text-rose-700'" class="gap-1">
                    <component :is="yearOverYearDelta(i)! >= 0 ? ArrowUpRight : ArrowDownRight" class="w-3 h-3" />{{ yearOverYearDelta(i)! >= 0 ? '+' : '' }}{{ yearOverYearDelta(i) }}
                  </Badge>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </Card>

      <!-- Subject mastery (existing) -->
      <Card class="p-5">
        <h4 class="font-bold text-slate-900 mb-1 flex items-center gap-2"><BarChart3 class="w-4 h-4 text-indigo-600" /> Penguasaan per Mata Uji</h4>
        <p class="text-xs text-muted-foreground mb-4">Akurasi jawaban siswa sekolah ini, dihitung dari seluruh attempt yang sudah dinilai.</p>
        <BarLineChart
          :labels="subjectChartLabels" :series="subjectChartSeries" :horizontal="true" suffix="%" :y-max="100"
          :height="Math.max(subjects.length * 42, 120)" empty-label="Belum ada data jawaban siswa."
        />
      </Card>

      <div v-if="strengths.length || improvements.length" class="grid md:grid-cols-2 gap-4">
        <Card v-if="strengths.length" class="p-5 border-emerald-200 bg-gradient-to-br from-emerald-50/60 to-teal-50/40">
          <h4 class="font-bold text-emerald-800 mb-3 flex items-center gap-2">
            <span class="w-7 h-7 rounded-lg bg-emerald-100 flex items-center justify-center"><ThumbsUp class="w-3.5 h-3.5 text-emerald-700" /></span>Kekuatan
          </h4>
          <ul class="space-y-1.5 text-sm text-emerald-900">
            <li v-for="s in strengths" :key="s.subject" class="flex items-start gap-2"><span class="w-1.5 h-1.5 rounded-full bg-emerald-500 mt-1.5 shrink-0" /><span><strong>{{ s.subject }}</strong> — akurasi {{ accuracyPct(s) }}%</span></li>
          </ul>
        </Card>
        <Card v-if="improvements.length" class="p-5 border-amber-200 bg-gradient-to-br from-amber-50/60 to-orange-50/40">
          <h4 class="font-bold text-amber-800 mb-3 flex items-center gap-2">
            <span class="w-7 h-7 rounded-lg bg-amber-100 flex items-center justify-center"><AlertTriangle class="w-3.5 h-3.5 text-amber-700" /></span>Area Peningkatan
          </h4>
          <ul class="space-y-1.5 text-sm text-amber-900">
            <li v-for="s in improvements" :key="s.subject" class="flex items-start gap-2"><span class="w-1.5 h-1.5 rounded-full bg-amber-500 mt-1.5 shrink-0" /><span><strong>{{ s.subject }}</strong> — akurasi baru {{ accuracyPct(s) }}%, perlu perhatian ekstra</span></li>
          </ul>
        </Card>
      </div>

      <!-- Distribusi Target PTN (existing) -->
      <Card class="p-5">
        <h4 class="font-bold text-slate-900 mb-1 flex items-center gap-2"><Target class="w-4 h-4 text-indigo-600" /> Distribusi Target PTN</h4>
        <p class="text-xs text-muted-foreground mb-4">Universitas yang paling banyak dijadikan target siswa sekolah ini (SNBP + SNBT).</p>
        <DonutChart :labels="distributionChartLabels" :values="distributionChartValues" unit-label=" siswa" :height="200" empty-label="Belum ada siswa yang menambahkan target PTN." />
      </Card>

      <!-- Tabel Siswa (drill-down) -->
      <Card ref="studentTableAnchor" class="overflow-hidden">
        <div class="p-5 border-b flex items-center justify-between flex-wrap gap-3">
          <div>
            <h3 class="font-bold text-slate-900 flex items-center gap-2"><Users class="w-4 h-4 text-indigo-600" />Daftar Siswa</h3>
            <p class="text-sm text-muted-foreground">{{ filteredActivity.length }} dari {{ totalStudents }} siswa ditampilkan</p>
          </div>
          <div class="flex items-center gap-2">
            <select v-model="kelasFilter" class="text-xs border rounded-lg px-2.5 py-1.5 bg-white">
              <option value="semua">Semua Kelas</option>
              <option v-for="k in kelasOptions.filter((k) => k !== 'semua')" :key="k" :value="k">{{ k }}</option>
            </select>
            <select v-model="sortBy" class="text-xs border rounded-lg px-2.5 py-1.5 bg-white">
              <option value="nama">Urutkan: Nama</option>
              <option value="skor_tertinggi">Skor Tertinggi</option>
              <option value="skor_terendah">Skor Terendah</option>
              <option value="aktivitas_terbaru">Aktivitas Terbaru</option>
            </select>
          </div>
        </div>
        <div v-if="filteredActivity.length === 0" class="py-10 text-center text-sm text-muted-foreground">Belum ada siswa untuk ditampilkan.</div>
        <div v-else class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b bg-slate-50">
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Nama</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Kelas</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Jumlah Attempt</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Skor Rata-rata</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Terakhir Aktif</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Status</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="a in filteredActivity" :key="a.student_id" class="border-b hover:bg-slate-50 transition-colors">
                <td class="px-4 py-3 font-semibold text-slate-900">{{ a.student_name }}</td>
                <td class="px-4 py-3 text-slate-700">{{ a.rombel_code || '-' }}</td>
                <td class="px-4 py-3 text-slate-700">{{ a.attempts_count }}</td>
                <td class="px-4 py-3 font-bold" :class="a.average_score != null ? 'text-indigo-600' : 'text-slate-400'">{{ a.average_score != null ? Math.round(a.average_score) : '-' }}</td>
                <td class="px-4 py-3 text-slate-500 text-xs">{{ relativeActivity(a.last_attempt_at) }}</td>
                <td class="px-4 py-3"><span class="px-2 py-1 rounded-full text-[11px] font-semibold" :class="statusFor(a).cls">{{ statusFor(a).label }}</span></td>
              </tr>
            </tbody>
          </table>
        </div>
      </Card>
    </template>
  </div>
</template>
