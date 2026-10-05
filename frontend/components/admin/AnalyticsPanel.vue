<script setup lang="ts">
// Analytics Platform (Admin) — v2. Dikembangkan dari versi sebelumnya (KPI + tren skor +
// status bank soal + akurasi mata uji + top sekolah) tanpa perlu endpoint backend baru —
// semua tambahan di bawah murni diturunkan dari data yang SUDAH di-fetch (summary, trend,
// subjects, statuses, topSchools), supaya bermakna & actionable tanpa memperbesar
// permukaan API:
//  - Insight & Rekomendasi: approval rate bank soal, mata uji terlemah platform-wide,
//    sekolah dengan skor di bawah rata-rata platform, arah tren skor.
//  - Tabel Top Sekolah dapat kolom "vs Rata-rata Platform" (delta berwarna).
//  - Ekspor PDF — lihat utils/adminAnalyticsExport.ts.
import { TrendingUp, Users, BookOpen, Award, ArrowUp, School as SchoolIcon, BarChart3, Trophy, Lightbulb, ThumbsUp, AlertTriangle, ArrowUpRight, ArrowDownRight, Download, Loader2, FileSpreadsheet, CalendarRange } from 'lucide-vue-next'
import type { AnalyticsSummary, QuestionStatusCount, SchoolRanking, ScoreTrendPoint, SubjectAccuracy, YearlyPerformancePoint } from '~/types'
import { exportAdminAnalyticsToPdf, exportAdminAnalyticsToExcel } from '~/utils/adminAnalyticsExport'

const { analyticsService } = useApi()
const toast = useToast()

const loading = ref(true)
const trendLoading = ref(false)
const exporting = ref(false)
const exportingExcel = ref(false)
const summary = ref<AnalyticsSummary | null>(null)
const trend = ref<ScoreTrendPoint[]>([])
const subjects = ref<SubjectAccuracy[]>([])
const statuses = ref<QuestionStatusCount[]>([])
const topSchools = ref<SchoolRanking[]>([])
const yearlyPerf = ref<YearlyPerformancePoint[]>([])

const TIME_FILTERS = [
  { label: '7 hari', days: 7 },
  { label: '30 hari', days: 30 },
  { label: '3 bulan', days: 90 },
  { label: '1 tahun', days: 365 },
]
const selectedDays = ref(14)
// Dinaikkan ke 50 (batas maksimum yang diizinkan backend, lihat analytics_service.rs
// `top_schools` — `limit.clamp(1, 50)`) supaya tabel benchmark mencakup semua sekolah
// yang punya aktivitas, dan supaya insight "sekolah tanpa aktivitas" bisa dihitung akurat
// (lihat computed `schoolsWithZeroActivity` di bawah — hanya valid kalau hasilnya TIDAK
// terpotong oleh limit ini).
const TOP_SCHOOLS_LIMIT = 50

const statusLabel: Record<string, string> = {
  draft: 'Draft', review: 'Menunggu Review', approved: 'Disetujui', rejected: 'Ditolak', revision: 'Perlu Revisi',
}

async function load() {
  loading.value = true
  try {
    const [s, t, subj, st, top, yp] = await Promise.all([
      analyticsService.summary(),
      analyticsService.scoreTrend(selectedDays.value),
      analyticsService.subjectBreakdown(),
      analyticsService.questionStatus(),
      analyticsService.topSchools(TOP_SCHOOLS_LIMIT),
      analyticsService.scoreTrendByYear(),
    ])
    summary.value = s
    trend.value = t
    subjects.value = subj
    statuses.value = st
    topSchools.value = top
    yearlyPerf.value = yp
  } catch (e: any) {
    toast.error('Gagal memuat data analytics', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

async function changeTimeFilter(days: number) {
  selectedDays.value = days
  trendLoading.value = true
  try {
    trend.value = await analyticsService.scoreTrend(days)
  } catch (e: any) {
    toast.error('Gagal memuat tren skor', e?.message)
  } finally {
    trendLoading.value = false
  }
}

// Cap the bar count so a wide filter (3 bulan/1 tahun) doesn't render one sliver per day —
// still 100% real points, just showing the most recent ones so each bar stays readable.
const MAX_VISIBLE_BARS = 45
const visibleTrend = computed(() => trend.value.slice(-MAX_VISIBLE_BARS))
const totalQuestionsForStatus = computed(() => statuses.value.reduce((sum, s) => sum + s.count, 0) || 1)

// Data chart untuk Tren Skor, Status Bank Soal, dan Akurasi per Mata Uji
const trendChartLabels = computed(() => visibleTrend.value.map((t) => formatDay(t.day)))
const trendChartSeries = computed(() => [{ label: 'Skor Rata-rata', data: visibleTrend.value.map((t) => Math.round(t.average_score)), color: '#4f46e5' }])

const statusChartLabels = computed(() => statuses.value.map((s) => statusLabel[s.status] || s.status))
const statusChartValues = computed(() => statuses.value.map((s) => s.count))

const subjectChartLabels = computed(() => subjects.value.map((s) => s.subject))
const subjectChartSeries = computed(() => [
  { label: 'Akurasi', data: subjects.value.map((s) => (s.total ? Math.round((s.correct / s.total) * 100) : 0)), color: '#6366f1' },
])

const kpis = computed(() => [
  { label: 'Siswa Terdaftar', val: summary.value?.total_students ?? 0, icon: Users },
  { label: 'Tryout/Drilling Selesai', val: summary.value?.submitted_attempts ?? 0, icon: BookOpen },
  { label: 'Sekolah Mitra', val: summary.value?.total_schools ?? 0, icon: SchoolIcon },
  { label: 'Rata-rata Skor', val: Math.round(summary.value?.average_score ?? 0), icon: Award },
])

function formatDay(d: string) {
  return new Date(d).toLocaleDateString('id-ID', { day: '2-digit', month: 'short' })
}
function subjectAccuracy(s: SubjectAccuracy) {
  return s.total > 0 ? Math.round((s.correct / s.total) * 100) : 0
}

// ─── Insight & Rekomendasi — murni diturunkan dari data yang sudah di-fetch di atas,
// tanpa endpoint tambahan. Hanya muncul kalau kondisinya benar-benar terjadi. ──────────
interface Insight { tone: 'warning' | 'positive' | 'info'; text: string }
const approvalRate = computed(() => {
  if (!summary.value || summary.value.total_questions === 0) return null
  return Math.round((summary.value.approved_questions / summary.value.total_questions) * 100)
})
const pendingReviewCount = computed(() => statuses.value.find((s) => s.status === 'review')?.count ?? 0)
const weakestSubject = computed(() => {
  if (!subjects.value.length) return null
  const withData = subjects.value.filter((s) => s.total >= 10) // cukup data biar tidak menyesatkan
  if (!withData.length) return null
  return [...withData].sort((a, b) => subjectAccuracy(a) - subjectAccuracy(b))[0]
})
const schoolsBelowPlatformAvg = computed(() => {
  if (!summary.value) return []
  return topSchools.value.filter((s) => s.average_score < summary.value!.average_score - 5)
})
// `topSchools` (top_schools endpoint) INNER JOINs attempts, jadi sekolah tanpa attempt
// tersubmit sama sekali tidak akan muncul di sana — selisihnya dengan `summary.total_schools`
// (COUNT(*) FROM schools, tanpa syarat) adalah sekolah terdaftar yang belum pernah beraktivitas.
// Cuma dianggap valid (exact, bukan perkiraan) kalau hasil topSchools TIDAK terpotong oleh
// TOP_SCHOOLS_LIMIT — kalau terpotong (persis = limit), kita tidak bisa tahu pasti berapa
// sekolah aktif sebenarnya, jadi insight ini disembunyikan daripada menampilkan angka yang
// bisa salah ("honest zero" — lebih baik tidak tampil daripada menampilkan angka dikarang).
const schoolsWithZeroActivity = computed(() => {
  if (!summary.value || topSchools.value.length >= TOP_SCHOOLS_LIMIT) return null
  const count = summary.value.total_schools - topSchools.value.length
  return count > 0 ? count : null
})
const trendDirection = computed(() => {
  const pts = trend.value.filter((t) => t.attempts > 0)
  if (pts.length < 4) return null
  const mid = Math.floor(pts.length / 2)
  const avg = (arr: ScoreTrendPoint[]) => arr.reduce((s, p) => s + p.average_score, 0) / arr.length
  const delta = Math.round(avg(pts.slice(mid)) - avg(pts.slice(0, mid)))
  return { delta, direction: delta > 2 ? 'naik' : delta < -2 ? 'turun' : 'stabil' as 'naik' | 'turun' | 'stabil' }
})
const insights = computed<Insight[]>(() => {
  const list: Insight[] = []
  if (pendingReviewCount.value > 0 && pendingReviewCount.value / totalQuestionsForStatus.value > 0.15) {
    list.push({ tone: 'warning', text: `${pendingReviewCount.value} soal (${Math.round((pendingReviewCount.value / totalQuestionsForStatus.value) * 100)}% dari bank soal) masih menunggu review — backlog cukup besar.` })
  }
  if (approvalRate.value != null && approvalRate.value < 70) {
    list.push({ tone: 'warning', text: `Tingkat persetujuan bank soal baru ${approvalRate.value}% (${summary.value?.approved_questions ?? 0} dari ${summary.value?.total_questions ?? 0} soal).` })
  }
  if (weakestSubject.value) {
    list.push({ tone: 'warning', text: `"${weakestSubject.value.subject}" adalah mata uji dengan akurasi terendah platform-wide (${subjectAccuracy(weakestSubject.value)}%) — pertimbangkan tinjau kualitas/kesulitan soalnya.` })
  }
  if (schoolsBelowPlatformAvg.value.length > 0) {
    list.push({ tone: 'info', text: `${schoolsBelowPlatformAvg.value.length} dari ${topSchools.value.length} sekolah pada daftar di bawah punya rata-rata skor lebih dari 5 poin di bawah rata-rata platform.` })
  }
  if (schoolsWithZeroActivity.value != null) {
    list.push({ tone: 'warning', text: `${schoolsWithZeroActivity.value} dari ${summary.value?.total_schools ?? 0} sekolah mitra terdaftar belum pernah punya attempt tersubmit sama sekali — pertimbangkan follow-up onboarding.` })
  }
  if (trendDirection.value?.direction === 'naik') {
    list.push({ tone: 'positive', text: `Tren skor platform naik ${trendDirection.value.delta} poin dibanding paruh awal periode yang dipilih.` })
  } else if (trendDirection.value?.direction === 'turun') {
    list.push({ tone: 'warning', text: `Tren skor platform turun ${Math.abs(trendDirection.value.delta)} poin dibanding paruh awal periode yang dipilih.` })
  }
  return list
})
const insightIcon = (tone: Insight['tone']) => (tone === 'positive' ? ThumbsUp : tone === 'warning' ? AlertTriangle : Lightbulb)
const insightClass = (tone: Insight['tone']) =>
  tone === 'positive' ? 'border-emerald-200 bg-emerald-50/60 text-emerald-900'
  : tone === 'warning' ? 'border-amber-200 bg-amber-50/60 text-amber-900'
  : 'border-sky-200 bg-sky-50/60 text-sky-900'

function schoolDelta(s: SchoolRanking): number | null {
  if (!summary.value) return null
  return Math.round(s.average_score - summary.value.average_score)
}

// ─── Perbandingan Tahun — platform-wide, attempts dikelompokkan per tahun kalender
// submitted_at (bukan angkatan/kohort siswa — lihat doc comment
// AnalyticsService::score_trend_by_year di backend). ────────────────────────────────
const yearlyChartLabels = computed(() => yearlyPerf.value.map((y) => String(y.year)))
const yearlyChartSeries = computed(() => [{ label: 'Skor Rata-rata', data: yearlyPerf.value.map((y) => Math.round(y.average_score)), color: '#0891b2' }])
function yearOverYearDelta(index: number) {
  if (index === 0) return null
  return Math.round(yearlyPerf.value[index].average_score - yearlyPerf.value[index - 1].average_score)
}

function buildExportData() {
  return {
    generatedAt: new Date().toISOString(),
    kpis: kpis.value.map((k) => ({ label: k.label, value: k.val.toLocaleString('id-ID') })),
    insights: insights.value.map((i) => i.text),
    yearlyPerformance: yearlyPerf.value.map((y) => ({ year: y.year, attempts: y.attempts, distinctStudents: y.distinct_students, avgScore: Math.round(y.average_score) })),
    subjects: subjects.value.map((s) => ({ subject: s.subject, accuracy: subjectAccuracy(s) })),
    topSchools: topSchools.value.map((s) => ({
      name: s.school_name, students: s.active_students, avgScore: Math.round(s.average_score), delta: schoolDelta(s),
    })),
  }
}
async function exportPdf() {
  exporting.value = true
  try {
    exportAdminAnalyticsToPdf(buildExportData())
  } catch (e: any) {
    toast.error('Gagal mengekspor PDF', e?.message)
  } finally {
    exporting.value = false
  }
}
async function exportExcel() {
  exportingExcel.value = true
  try {
    exportAdminAnalyticsToExcel(buildExportData())
  } catch (e: any) {
    toast.error('Gagal mengekspor Excel', e?.message)
  } finally {
    exportingExcel.value = false
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-xl font-black text-slate-900 flex items-center gap-2"><BarChart3 class="w-5 h-5 text-indigo-600" />Analytics Platform</h2>
        <p class="text-sm text-muted-foreground">Data asli dari aktivitas siswa, soal, dan sekolah</p>
      </div>
      <div class="flex items-center gap-2 flex-wrap">
        <div class="flex gap-2">
          <button
            v-for="f in TIME_FILTERS"
            :key="f.label"
            class="px-4 py-1.5 rounded-xl text-sm font-semibold transition-colors"
            :class="selectedDays === f.days ? 'bg-indigo-600 text-white' : 'bg-white border text-slate-600 hover:bg-slate-50'"
            @click="changeTimeFilter(f.days)"
          >
            {{ f.label }}
          </button>
        </div>
        <Button variant="outline" size="sm" class="gap-1.5" :disabled="exporting || loading" @click="exportPdf">
          <Loader2 v-if="exporting" class="w-3.5 h-3.5 animate-spin" /><Download v-else class="w-3.5 h-3.5" />PDF
        </Button>
        <Button variant="outline" size="sm" class="gap-1.5" :disabled="exportingExcel || loading" @click="exportExcel">
          <Loader2 v-if="exportingExcel" class="w-3.5 h-3.5 animate-spin" /><FileSpreadsheet v-else class="w-3.5 h-3.5" />Excel
        </Button>
      </div>
    </div>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat data analytics...</div>

    <template v-else>
      <!-- KPI strip -->
      <div class="grid grid-cols-2 xl:grid-cols-4 gap-4">
        <Card v-for="k in kpis" :key="k.label" class="p-5">
          <div class="flex items-start justify-between mb-3">
            <div class="w-10 h-10 rounded-xl bg-indigo-50 flex items-center justify-center">
              <component :is="k.icon" class="w-5 h-5 text-indigo-600" />
            </div>
          </div>
          <p class="text-2xl font-black text-slate-900">{{ k.val.toLocaleString('id-ID') }}</p>
          <p class="text-xs text-muted-foreground mt-0.5">{{ k.label }}</p>
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

      <div class="grid lg:grid-cols-3 gap-6">
        <!-- Score trend -->
        <Card class="lg:col-span-2 p-6">
          <div class="flex items-center justify-between mb-6">
            <div>
              <h3 class="text-lg font-bold mb-1">Tren Skor Rata-rata ({{ TIME_FILTERS.find((f) => f.days === selectedDays)?.label || `${selectedDays} hari` }} Terakhir)</h3>
              <p class="text-sm text-muted-foreground">Berdasarkan attempt yang sudah disubmit siswa</p>
            </div>
            <Badge v-if="summary" class="bg-indigo-100 text-indigo-700 gap-1">
              <TrendingUp class="w-3 h-3" />{{ summary.total_attempts }} total attempt
            </Badge>
          </div>

          <div v-if="trendLoading" class="py-16 text-center text-sm text-muted-foreground">Memuat tren...</div>
          <div v-else-if="trend.length === 0" class="py-16 text-center text-sm text-muted-foreground">
            Belum ada attempt yang disubmit dalam rentang waktu ini.
          </div>
          <template v-else>
            <p v-if="trend.length > MAX_VISIBLE_BARS" class="text-xs text-muted-foreground mb-2">
              Menampilkan {{ MAX_VISIBLE_BARS }} hari paling baru dari {{ trend.length }} hari dengan data.
            </p>
            <BarLineChart :labels="trendChartLabels" :series="trendChartSeries" :height="224" />
          </template>
        </Card>

        <!-- Question status distribution -->
        <Card class="p-6">
          <h3 class="text-lg font-bold mb-1">Status Bank Soal</h3>
          <p class="text-sm text-muted-foreground mb-6">Distribusi status semua soal</p>
          <DonutChart :labels="statusChartLabels" :values="statusChartValues" unit-label=" soal" :height="180" empty-label="Belum ada soal." />
        </Card>
      </div>

      <!-- Subject breakdown -->
      <Card class="p-6">
        <div class="flex items-center justify-between mb-6">
          <div>
            <h3 class="text-lg font-bold mb-1">Akurasi per Mata Uji</h3>
            <p class="text-sm text-muted-foreground">Dihitung dari semua jawaban siswa yang sudah dinilai</p>
          </div>
        </div>
        <BarLineChart
          :labels="subjectChartLabels"
          :series="subjectChartSeries"
          :horizontal="true"
          suffix="%"
          :y-max="100"
          :height="Math.max(subjects.length * 42, 120)"
          empty-label="Belum ada jawaban yang dinilai — mulai tryout untuk melihat data di sini."
        />
      </Card>

      <!-- Perbandingan Tahun -->
      <Card v-if="yearlyPerf.length" class="p-6">
        <h3 class="text-lg font-bold mb-1 flex items-center gap-2"><CalendarRange class="w-4 h-4 text-cyan-600" />Perbandingan Tahun</h3>
        <p class="text-sm text-muted-foreground mb-4">Rata-rata skor platform per tahun kalender attempt dikerjakan (bukan angkatan/kohort siswa — data kohort belum tersedia lengkap di sistem).</p>
        <BarLineChart
          v-if="yearlyPerf.length > 1"
          :labels="yearlyChartLabels" :series="yearlyChartSeries"
          :height="200" empty-label="Belum cukup data untuk dibandingkan."
        />
        <div v-else class="py-6 text-center text-sm text-muted-foreground">Baru ada data 1 tahun ({{ yearlyPerf[0].year }}) — perbandingan akan muncul setelah ada attempt di tahun lain.</div>
        <div class="overflow-x-auto mt-4">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b bg-slate-50">
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Tahun</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Jumlah Attempt</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Siswa Aktif</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Skor Rata-rata</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">vs Tahun Sebelumnya</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(y, i) in yearlyPerf" :key="y.year" class="border-b hover:bg-slate-50 transition-colors">
                <td class="px-4 py-3 font-semibold text-slate-900">{{ y.year }}</td>
                <td class="px-4 py-3 text-slate-700">{{ y.attempts.toLocaleString('id-ID') }}</td>
                <td class="px-4 py-3 text-slate-700">{{ y.distinct_students.toLocaleString('id-ID') }}</td>
                <td class="px-4 py-3 font-bold text-cyan-700">{{ Math.round(y.average_score) }}</td>
                <td class="px-4 py-3">
                  <span v-if="yearOverYearDelta(i) == null" class="text-xs text-slate-400">-</span>
                  <span v-else class="inline-flex items-center gap-1 text-xs font-semibold" :class="yearOverYearDelta(i)! >= 0 ? 'text-emerald-600' : 'text-rose-600'">
                    <component :is="yearOverYearDelta(i)! >= 0 ? ArrowUpRight : ArrowDownRight" class="w-3 h-3" />{{ yearOverYearDelta(i)! >= 0 ? '+' : '' }}{{ yearOverYearDelta(i) }}
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </Card>

      <!-- Top schools -->
      <Card class="overflow-hidden">
        <div class="p-5 border-b flex items-center justify-between">
          <div>
            <h3 class="font-bold text-slate-900 flex items-center gap-2"><Trophy class="w-4 h-4 text-amber-500" />Peringkat Sekolah Mitra</h3>
            <p class="text-sm text-muted-foreground">Berdasarkan rata-rata skor siswa (attempt yang sudah disubmit), dibandingkan rata-rata platform</p>
          </div>
        </div>
        <div v-if="topSchools.length === 0" class="py-10 text-center text-sm text-muted-foreground">
          Belum ada sekolah dengan attempt tersubmit untuk diranking.
        </div>
        <div v-else class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b bg-slate-50">
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Rank</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Sekolah</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Siswa Aktif</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">Rata-rata Skor</th>
                <th class="text-left px-4 py-3 font-semibold text-slate-600">vs Rata-rata Platform</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(s, i) in topSchools" :key="s.school_id" class="border-b hover:bg-slate-50 transition-colors">
                <td class="px-4 py-3">
                  <span class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-black" :class="i < 3 ? 'bg-gradient-to-br from-indigo-500 to-purple-600 text-white' : 'bg-slate-100 text-slate-700'">
                    {{ i + 1 }}
                  </span>
                </td>
                <td class="px-4 py-3 font-semibold text-slate-900">{{ s.school_name }}</td>
                <td class="px-4 py-3 text-slate-700">{{ s.active_students.toLocaleString('id-ID') }}</td>
                <td class="px-4 py-3"><span class="font-bold text-indigo-600">{{ Math.round(s.average_score) }}</span></td>
                <td class="px-4 py-3">
                  <span v-if="schoolDelta(s) != null" class="inline-flex items-center gap-1 text-xs font-semibold" :class="schoolDelta(s)! >= 0 ? 'text-emerald-600' : 'text-rose-600'">
                    <component :is="schoolDelta(s)! >= 0 ? ArrowUpRight : ArrowDownRight" class="w-3 h-3" />{{ schoolDelta(s)! >= 0 ? '+' : '' }}{{ schoolDelta(s) }}
                  </span>
                  <span v-else class="text-xs text-slate-400">-</span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </Card>

      <Card class="p-5 flex items-center gap-3 bg-slate-50">
        <div class="w-9 h-9 rounded-lg bg-indigo-100 flex items-center justify-center shrink-0">
          <ArrowUp class="w-4 h-4 text-indigo-600" />
        </div>
        <p class="text-xs text-muted-foreground">
          {{ summary?.approved_questions ?? 0 }} dari {{ summary?.total_questions ?? 0 }} soal sudah disetujui dan siap dipakai untuk tryout/drilling.
        </p>
      </Card>
    </template>
  </div>
</template>
