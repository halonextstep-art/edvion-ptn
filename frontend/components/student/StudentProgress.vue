<script setup lang="ts">
// Progress — 100% dihitung dari data nyata: attempt milik siswa (tryoutService.myAttempts),
// penguasaan per mata uji (analyticsService.mySubjectBreakdown), rata-rata nasional
// (analyticsService.nationalScoreTrend — agregat/anonim, aman dibagikan lintas role), dan
// milestone dari badge nyata (gamificationService.myBadges). Insight di bawah dirangkai
// dari angka-angka ini, bukan teks yang dikarang.
import {
  TrendingUp, Target, CheckCircle2, XCircle, HelpCircle, Loader2, Share2, Printer, Flame, Trophy, Zap,
  Lightbulb, Award, BarChart3, Download, FileSpreadsheet,
} from 'lucide-vue-next'
import type { Attempt, SubjectAccuracy, ScoreTrendPoint, StudentBadgeStatusItem } from '~/types'
import { exportStudentProgressToPdf, exportStudentProgressToExcel } from '~/utils/studentProgressExport'

const { tryoutService, analyticsService, gamificationService } = useApi()
const { user } = useAuth()
const toast = useToast()
const exporting = ref(false)
const exportingExcel = ref(false)

const attempts = ref<Attempt[]>([])
const subjects = ref<SubjectAccuracy[]>([])
const national = ref<ScoreTrendPoint[]>([])
const badges = ref<StudentBadgeStatusItem[]>([])
const loading = ref(true)

onMounted(async () => {
  try {
    attempts.value = await tryoutService.myAttempts()
  } finally {
    loading.value = false
  }
  const [s, n, b] = await Promise.allSettled([
    analyticsService.mySubjectBreakdown(),
    analyticsService.nationalScoreTrend(14),
    gamificationService.myBadges(),
  ])
  if (s.status === 'fulfilled') subjects.value = s.value
  if (n.status === 'fulfilled') national.value = n.value
  if (b.status === 'fulfilled') badges.value = b.value
})

const submitted = computed(() =>
  attempts.value
    .filter((a) => a.status === 'submitted' && a.score != null && a.submitted_at)
    .sort((a, b) => new Date(a.submitted_at!).getTime() - new Date(b.submitted_at!).getTime()),
)

const bestScore = computed(() => (submitted.value.length ? Math.max(...submitted.value.map((a) => a.score ?? 0)) : null))
const avgScore = computed(() => {
  if (!submitted.value.length) return null
  return Math.round(submitted.value.reduce((sum, a) => sum + (a.score ?? 0), 0) / submitted.value.length)
})
const avgAccuracy = computed(() => {
  const withAcc = submitted.value.filter((a) => a.accuracy != null)
  if (!withAcc.length) return null
  return Math.round((withAcc.reduce((sum, a) => sum + (a.accuracy ?? 0), 0) / withAcc.length) * 10) / 10
})
const scoreImprovement = computed(() => {
  if (submitted.value.length < 2) return null
  const half = Math.floor(submitted.value.length / 2)
  const firstHalfAvg = submitted.value.slice(0, half).reduce((s, a) => s + (a.score ?? 0), 0) / half
  const secondHalfAvg = submitted.value.slice(half).reduce((s, a) => s + (a.score ?? 0), 0) / (submitted.value.length - half)
  return Math.round(secondHalfAvg - firstHalfAvg)
})

const currentStreak = computed(() => {
  const days = [...new Set(submitted.value.map((a) => new Date(a.submitted_at!).toDateString()))].map((d) => new Date(d).getTime()).sort((a, b) => b - a)
  if (!days.length) return 0
  const today = new Date(); today.setHours(0, 0, 0, 0)
  if (Math.round((today.getTime() - days[0]) / 86400000) > 1) return 0
  let streak = 1
  for (let i = 1; i < days.length; i++) { if (Math.round((days[i - 1] - days[i]) / 86400000) === 1) streak++; else break }
  return streak
})

const totals = computed(() => submitted.value.reduce(
  (acc, a) => {
    acc.correct += a.correct_count ?? 0
    acc.wrong += a.wrong_count ?? 0
    acc.unanswered += a.unanswered_count ?? 0
    return acc
  },
  { correct: 0, wrong: 0, unanswered: 0 },
))

const recentTrend = computed(() => submitted.value.slice(-14))
const recentTrendChartLabels = computed(() => recentTrend.value.map((a) => fmtDate(a.submitted_at!)))
const recentTrendChartSeries = computed(() => [{ label: 'Skor', data: recentTrend.value.map((a) => a.score ?? null), color: '#4f46e5' }])

const byType = computed(() => {
  const map: Record<string, { count: number; totalScore: number }> = {}
  for (const a of submitted.value) {
    const key = a.session_type
    if (!map[key]) map[key] = { count: 0, totalScore: 0 }
    map[key].count += 1
    map[key].totalScore += a.score ?? 0
  }
  return map
})
const TYPE_LABEL: Record<string, string> = { tryout: 'Tryout', drilling: 'Drilling', mini: 'Mini Tryout' }

// ─── Own daily avg vs national daily avg (last 14 hari) ──────────────────────────
const comparisonDays = computed(() => {
  const out: { key: string; label: string; mine: number | null; national: number | null }[] = []
  const nationalMap = new Map(national.value.map((p) => [p.day, p.average_score]))
  const ownByDay = new Map<string, { sum: number; count: number }>()
  for (const a of submitted.value) {
    const key = new Date(a.submitted_at!).toISOString().slice(0, 10)
    const b = ownByDay.get(key) ?? { sum: 0, count: 0 }
    b.sum += a.score ?? 0; b.count += 1
    ownByDay.set(key, b)
  }
  for (let i = 13; i >= 0; i--) {
    const d = new Date(); d.setDate(d.getDate() - i)
    const key = d.toISOString().slice(0, 10)
    const own = ownByDay.get(key)
    out.push({
      key,
      label: d.toLocaleDateString('id-ID', { day: '2-digit', month: 'short' }),
      mine: own ? Math.round(own.sum / own.count) : null,
      national: nationalMap.has(key) ? Math.round(nationalMap.get(key)!) : null,
    })
  }
  return out
})
const comparisonChartLabels = computed(() => comparisonDays.value.map((d) => d.label))
const comparisonChartSeries = computed(() => [
  { label: 'Kamu', data: comparisonDays.value.map((d) => d.mine), color: '#4f46e5' },
  { label: 'Nasional', data: comparisonDays.value.map((d) => d.national), color: '#cbd5e1' },
])

// ─── Aktivitas mingguan (8 minggu terakhir) ──────────────────────────────────────
const weeklyActivity = computed(() => {
  const buckets = new Map<string, { tryout: number; drilling: number; minutes: number; weekStart: number }>()
  for (const a of submitted.value) {
    const d = new Date(a.submitted_at!)
    const monday = new Date(d); monday.setHours(0, 0, 0, 0); monday.setDate(d.getDate() - ((d.getDay() + 6) % 7))
    const key = monday.toISOString().slice(0, 10)
    const b = buckets.get(key) ?? { tryout: 0, drilling: 0, minutes: 0, weekStart: monday.getTime() }
    if (a.session_type === 'tryout') b.tryout++
    else b.drilling++
    b.minutes += a.duration_minutes ?? 0
    buckets.set(key, b)
  }
  return [...buckets.values()].sort((a, b) => a.weekStart - b.weekStart).slice(-8)
})
const totalHours = computed(() => Math.round((weeklyActivity.value.reduce((s, w) => s + w.minutes, 0) / 60) * 10) / 10)
const weeklyActivityChartLabels = computed(() => weeklyActivity.value.map((_, i) => `M${i + 1}`))
const weeklyActivityChartSeries = computed(() => [
  { label: 'Tryout', data: weeklyActivity.value.map((w) => w.tryout), color: '#f97316', stack: 'sesi' },
  { label: 'Drilling/Mini', data: weeklyActivity.value.map((w) => w.drilling), color: '#6366f1', stack: 'sesi' },
])

// ─── Milestones (badge nyata) ─────────────────────────────────────────────────────
const earnedCount = computed(() => badges.value.filter((b) => b.earned).length)

// ─── Insight rule-based dari data nyata ───────────────────────────────────────────
function accuracyPct(s: SubjectAccuracy) {
  return s.total > 0 ? Math.round((s.correct / s.total) * 100) : 0
}
// `subjects` sudah di-fetch dari awal (dipakai untuk insight "Fokus Area" di bawah) tapi
// sebelumnya tidak pernah divisualisasikan — sekarang ditampilkan juga sebagai chart supaya
// siswa lihat sendiri sebaran penguasaannya per mata uji, bukan cuma 1 kalimat mata uji
// terlemah.
const subjectChartLabels = computed(() => subjects.value.map((s) => s.subject))
const subjectChartSeries = computed(() => [{ label: 'Akurasi', data: subjects.value.map((s) => accuracyPct(s)), color: '#6366f1' }])
const weakestSubject = computed(() => {
  if (!subjects.value.length) return null
  return [...subjects.value].sort((a, b) => accuracyPct(a) - accuracyPct(b))[0]
})
const insights = computed(() => {
  const list: string[] = []
  if (weakestSubject.value) {
    list.push(`Fokus Area: ${weakestSubject.value.subject} masih di akurasi ${accuracyPct(weakestSubject.value)}% — jadikan prioritas latihan minggu ini.`)
  }
  if (scoreImprovement.value != null) {
    list.push(scoreImprovement.value >= 0
      ? `Skor kamu naik ${scoreImprovement.value} poin dibanding paruh pertama sesi latihanmu — pertahankan ritmenya.`
      : `Skor kamu turun ${Math.abs(scoreImprovement.value)} poin dibanding paruh pertama sesi latihanmu — coba evaluasi jam belajar atau istirahat.`)
  }
  if (currentStreak.value >= 3) {
    list.push(`Streak ${currentStreak.value} hari berturut-turut — konsistensi seperti ini yang membangun kesiapan SNBT.`)
  } else if (submitted.value.length > 0) {
    list.push('Belum ada streak aktif — coba latihan singkat setiap hari, walau cuma 1 sesi, untuk membangun kebiasaan.')
  }
  return list
})

function fmtDate(iso: string) {
  return new Date(iso).toLocaleDateString('id-ID', { day: '2-digit', month: 'short' })
}
function shareProgress() {
  const text = `Progress EdvionPTN saya: skor terbaik ${bestScore.value ?? '-'}, rata-rata ${avgScore.value ?? '-'}, ${submitted.value.length} sesi selesai.`
  if (navigator.clipboard) {
    navigator.clipboard.writeText(text)
    toast.success('Disalin ke clipboard', 'Ringkasan progress siap dibagikan.')
  }
}
function printReport() {
  window.print()
}
function buildExportData() {
  return {
    studentName: user.value?.name || 'Siswa',
    generatedAt: new Date().toISOString(),
    kpis: [
      { label: 'Day Streak', value: String(currentStreak.value) },
      { label: 'Tryout Selesai', value: String(submitted.value.length) },
      { label: 'Skor Terbaik', value: String(bestScore.value ?? '-') },
      { label: 'Rata-rata Skor', value: String(avgScore.value ?? '-') },
      { label: 'Rata-rata Akurasi', value: avgAccuracy.value != null ? `${avgAccuracy.value}%` : '-' },
      { label: 'Perkembangan Skor', value: scoreImprovement.value != null ? `${scoreImprovement.value >= 0 ? '+' : ''}${scoreImprovement.value}` : '-' },
    ],
    subjects: subjects.value.map((s) => ({ subject: s.subject, accuracy: accuracyPct(s) })),
    insights: insights.value,
    totals: { correct: totals.value.correct, wrong: totals.value.wrong, unanswered: totals.value.unanswered },
  }
}
async function exportPdf() {
  exporting.value = true
  try {
    exportStudentProgressToPdf(buildExportData())
  } catch (e: any) {
    toast.error('Gagal mengekspor PDF', e?.message)
  } finally {
    exporting.value = false
  }
}
async function exportExcel() {
  exportingExcel.value = true
  try {
    exportStudentProgressToExcel(buildExportData())
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
      <Loader2 class="w-6 h-6 animate-spin mx-auto mb-2" /> Memuat progress...
    </div>

    <template v-else-if="submitted.length === 0">
      <Card class="p-10 text-center">
        <Target class="w-10 h-10 text-slate-300 mx-auto mb-3" />
        <p class="text-sm text-muted-foreground">Belum ada tryout/drilling yang diselesaikan. Progress akan muncul di sini setelah kamu mulai latihan.</p>
      </Card>
    </template>

    <template v-else>
      <!-- Header hero -->
      <Card class="p-5 sm:p-6 bg-gradient-to-br from-purple-600 via-indigo-600 to-pink-600 text-white border-0">
        <div class="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
          <div class="min-w-0">
            <h2 class="text-xl sm:text-2xl font-black mb-1 flex items-center gap-2">Journey to PTN <Target class="w-5 h-5 shrink-0" /></h2>
            <p class="text-purple-100 text-sm">Semua angka di sini nyata — dihitung langsung dari hasil latihanmu.</p>
          </div>
          <!-- 4 tombol: grid 2x2 penuh-lebar di mobile (bukan 1 baris rapat yang kepotong),
               kembali ke satu baris pill compact di sm+ ke atas. -->
          <div class="grid grid-cols-2 sm:flex gap-2 w-full md:w-auto shrink-0">
            <Button variant="outline" class="bg-white/10 border-white/30 text-white hover:bg-white/20" @click="shareProgress"><Share2 class="w-4 h-4" />Share</Button>
            <Button variant="outline" class="bg-white/10 border-white/30 text-white hover:bg-white/20" :disabled="exporting" @click="exportPdf">
              <Loader2 v-if="exporting" class="w-4 h-4 animate-spin" /><Download v-else class="w-4 h-4" />PDF
            </Button>
            <Button variant="outline" class="bg-white/10 border-white/30 text-white hover:bg-white/20" :disabled="exportingExcel" @click="exportExcel">
              <Loader2 v-if="exportingExcel" class="w-4 h-4 animate-spin" /><FileSpreadsheet v-else class="w-4 h-4" />Excel
            </Button>
            <Button class="bg-white text-indigo-700 hover:bg-indigo-50" @click="printReport"><Printer class="w-4 h-4" />Cetak</Button>
          </div>
        </div>
      </Card>

      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card class="relative overflow-hidden p-5 border-2 border-amber-200">
          <Flame class="absolute -right-2 -bottom-2 w-16 h-16 text-amber-500 opacity-10" />
          <div class="w-9 h-9 rounded-lg bg-gradient-to-br from-amber-400 to-orange-500 flex items-center justify-center mb-2"><Flame class="w-4 h-4 text-white" /></div>
          <div class="text-xs text-muted-foreground mb-1">Day Streak</div>
          <div class="text-2xl font-black text-slate-900">{{ currentStreak }}</div>
        </Card>
        <Card class="relative overflow-hidden p-5 border-2 border-purple-200">
          <Trophy class="absolute -right-2 -bottom-2 w-16 h-16 text-purple-500 opacity-10" />
          <div class="w-9 h-9 rounded-lg bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center mb-2"><Trophy class="w-4 h-4 text-white" /></div>
          <div class="text-xs text-muted-foreground mb-1">Tryouts Done</div>
          <div class="text-2xl font-black text-slate-900">{{ submitted.length }}</div>
        </Card>
        <Card class="relative overflow-hidden p-5 border-2 border-blue-200">
          <Zap class="absolute -right-2 -bottom-2 w-16 h-16 text-blue-500 opacity-10" />
          <div class="w-9 h-9 rounded-lg bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center mb-2"><Zap class="w-4 h-4 text-white" /></div>
          <div class="text-xs text-muted-foreground mb-1">Questions Solved</div>
          <div class="text-2xl font-black text-slate-900">{{ totals.correct + totals.wrong }}</div>
        </Card>
        <Card class="relative overflow-hidden p-5 border-2 border-green-200">
          <TrendingUp class="absolute -right-2 -bottom-2 w-16 h-16 text-green-500 opacity-10" />
          <div class="w-9 h-9 rounded-lg bg-gradient-to-br from-green-500 to-emerald-500 flex items-center justify-center mb-2"><TrendingUp class="w-4 h-4 text-white" /></div>
          <div class="text-xs text-muted-foreground mb-1">Score Improvement</div>
          <div class="text-2xl font-black text-slate-900">{{ scoreImprovement != null ? (scoreImprovement >= 0 ? '+' : '') + scoreImprovement : '-' }}</div>
        </Card>
      </div>

      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card class="p-5">
          <div class="text-xs text-muted-foreground mb-1">Skor Terbaik</div>
          <div class="text-2xl font-black text-slate-900">{{ bestScore }}</div>
        </Card>
        <Card class="p-5">
          <div class="text-xs text-muted-foreground mb-1">Rata-rata Skor</div>
          <div class="text-2xl font-black text-slate-900">{{ avgScore }}</div>
        </Card>
        <Card class="p-5">
          <div class="text-xs text-muted-foreground mb-1">Rata-rata Akurasi</div>
          <div class="text-2xl font-black text-slate-900">{{ avgAccuracy }}%</div>
        </Card>
        <Card class="p-5">
          <div class="text-xs text-muted-foreground mb-1">Total Sesi Selesai</div>
          <div class="text-2xl font-black text-slate-900">{{ submitted.length }}</div>
        </Card>
      </div>

      <!-- Perkembangan skor vs rata-rata nasional -->
      <Card class="p-5">
        <h4 class="font-bold text-slate-900 mb-4 flex items-center gap-2"><TrendingUp class="w-4 h-4 text-indigo-600" /> Perkembangan Skor (14 hari) vs Rata-rata Nasional</h4>
        <BarLineChart :labels="comparisonChartLabels" :series="comparisonChartSeries" :height="220" />
      </Card>

      <!-- Tren skor per sesi -->
      <Card class="p-5">
        <h4 class="font-bold text-slate-900 mb-4 flex items-center gap-2"><TrendingUp class="w-4 h-4 text-indigo-600" /> Tren Skor (14 sesi terakhir)</h4>
        <BarLineChart :labels="recentTrendChartLabels" :series="recentTrendChartSeries" :height="200" />
      </Card>

      <!-- Aktivitas mingguan -->
      <Card class="p-5">
        <div class="flex items-center justify-between mb-4">
          <h4 class="font-bold text-slate-900">Aktivitas Mingguan</h4>
          <span class="text-xs text-muted-foreground">Total {{ totalHours }} jam latihan (8 minggu terakhir)</span>
        </div>
        <BarLineChart :labels="weeklyActivityChartLabels" :series="weeklyActivityChartSeries" :height="200" empty-label="Belum ada data mingguan." />
      </Card>

      <!-- Penguasaan per mata uji -->
      <Card v-if="subjects.length" class="p-5">
        <h4 class="font-bold text-slate-900 mb-1 flex items-center gap-2"><BarChart3 class="w-4 h-4 text-indigo-600" /> Penguasaan per Mata Uji</h4>
        <p class="text-xs text-muted-foreground mb-4">Akurasi jawabanmu, dihitung dari semua sesi yang sudah dinilai.</p>
        <BarLineChart
          :labels="subjectChartLabels" :series="subjectChartSeries" :horizontal="true" suffix="%" :y-max="100"
          :height="Math.max(subjects.length * 42, 120)"
        />
      </Card>

      <div class="grid md:grid-cols-2 gap-4">
        <Card class="p-5">
          <h4 class="font-bold text-slate-900 mb-3">Jawaban (akumulasi semua sesi)</h4>
          <div class="space-y-2.5">
            <div class="flex items-center justify-between text-sm">
              <span class="flex items-center gap-1.5 text-emerald-700"><CheckCircle2 class="w-4 h-4" /> Benar</span>
              <span class="font-bold">{{ totals.correct }}</span>
            </div>
            <div class="flex items-center justify-between text-sm">
              <span class="flex items-center gap-1.5 text-red-600"><XCircle class="w-4 h-4" /> Salah</span>
              <span class="font-bold">{{ totals.wrong }}</span>
            </div>
            <div class="flex items-center justify-between text-sm">
              <span class="flex items-center gap-1.5 text-slate-500"><HelpCircle class="w-4 h-4" /> Tidak dijawab</span>
              <span class="font-bold">{{ totals.unanswered }}</span>
            </div>
          </div>
        </Card>

        <Card class="p-5">
          <h4 class="font-bold text-slate-900 mb-3">Berdasarkan Tipe Sesi</h4>
          <div class="space-y-2.5">
            <div v-for="(v, k) in byType" :key="k" class="flex items-center justify-between text-sm">
              <span class="text-slate-700">{{ TYPE_LABEL[k] || k }}</span>
              <span class="text-muted-foreground">{{ v.count }}x · rata-rata <strong class="text-slate-900">{{ Math.round(v.totalScore / v.count) }}</strong></span>
            </div>
          </div>
        </Card>
      </div>

      <!-- Milestones -->
      <Card v-if="badges.length" class="p-5">
        <div class="flex items-center justify-between mb-4">
          <h4 class="font-bold text-slate-900 flex items-center gap-2"><Award class="w-4 h-4 text-amber-600" /> Milestones &amp; Achievements</h4>
          <Badge class="bg-slate-100 text-slate-600">{{ earnedCount }}/{{ badges.length }} Completed</Badge>
        </div>
        <div class="grid grid-cols-3 sm:grid-cols-6 gap-3">
          <div v-for="b in badges" :key="b.id" class="p-3 rounded-lg border text-center" :class="b.earned ? 'bg-amber-50 border-amber-200' : 'opacity-40 grayscale'">
            <IconDisplay :icon-type="b.icon_type" :icon-name="b.icon_name" :icon-url="b.icon_url" size="w-6 h-6 mx-auto mb-1 text-amber-600" />
            <div class="text-[10px] font-semibold text-slate-900 truncate">{{ b.name }}</div>
          </div>
        </div>
      </Card>

      <!-- Insight & rekomendasi -->
      <Card v-if="insights.length" class="p-5 bg-gradient-to-br from-blue-50 to-purple-50 border-blue-100">
        <h4 class="font-bold text-slate-900 mb-3 flex items-center gap-2"><Lightbulb class="w-4 h-4 text-amber-500" /> Insight &amp; Rekomendasi</h4>
        <ul class="space-y-2">
          <li v-for="(t, i) in insights" :key="i" class="flex items-start gap-2 text-sm text-slate-700">
            <span class="w-1.5 h-1.5 rounded-full bg-indigo-500 mt-1.5 shrink-0" />{{ t }}
          </li>
        </ul>
      </Card>
    </template>
  </div>
</template>
