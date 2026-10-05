<script setup lang="ts">
// Analitik Konten — 100% dihitung dari data soal nyata (questionService.list), tidak ada
// modul analytics/paket terpisah di backend untuk role ini, jadi semua kartu & chart di
// bawah adalah agregasi client-side dari field yang sudah nyata di setiap Question
// (status, question_type, difficulty, subject, usage_count, average_score, created_at).
import { FileText, AlertTriangle, Layers, TrendingUp } from 'lucide-vue-next'
import type { Question, QuestionStatus, QuestionType } from '~/types'

const { questionService } = useApi()
const { user } = useAuth()
const toast = useToast()

const questions = ref<Question[]>([])
const total = ref(0)
const loading = ref(true)

// Backend clamps page_size to 100 (see postgres_question_repository.rs) — charts below are
// computed from this loaded batch, so for banks with >100 questions they reflect the most
// recent 100, not the full total. The "Total Soal" KPI still uses the real, unclamped total.
async function load() {
  loading.value = true
  try {
    const res = await questionService.list({ page: 1, page_size: 100 })
    questions.value = res.items
    total.value = res.total
  } catch (e: any) {
    toast.error('Gagal memuat analitik konten', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

const QTYPE_LABELS: Record<QuestionType, string> = {
  multiple_choice: 'Pilihan Ganda',
  complex_multiple: 'PG Kompleks',
  short_answer: 'Isian Singkat',
}
const STATUS_LABELS: Record<QuestionStatus, string> = {
  draft: 'Draft', review: 'Review', approved: 'Disetujui', rejected: 'Ditolak', revision: 'Revisi',
}

const myCount = computed(() => questions.value.filter((q) => q.created_by === user.value?.id).length)
const approvedCount = computed(() => questions.value.filter((q) => q.status === 'approved').length)
const needsAttentionCount = computed(() => questions.value.filter((q) => q.status === 'rejected' || q.status === 'revision').length)

const kpis = computed(() => [
  { label: 'Total Soal', val: total.value, sub: `${approvedCount.value} disetujui`, icon: FileText, c: 'text-indigo-600 dark:text-violet-400', bg: 'bg-indigo-50 dark:bg-violet-500/15' },
  { label: 'Soal Saya', val: myCount.value, sub: 'dibuat oleh kamu', icon: Layers, c: 'text-violet-600 dark:text-violet-400', bg: 'bg-violet-50 dark:bg-violet-500/15' },
  { label: 'Perlu Tindak Lanjut', val: needsAttentionCount.value, sub: 'ditolak/perlu revisi', icon: AlertTriangle, c: 'text-amber-600 dark:text-amber-400', bg: 'bg-amber-50 dark:bg-amber-500/15' },
  { label: 'Tipe Soal', val: Object.keys(QTYPE_LABELS).length, sub: 'format didukung', icon: FileText, c: 'text-slate-600 dark:text-white/50', bg: 'bg-slate-100 dark:bg-white/10' },
])

// Status distribution (donut)
const statusChartLabels = computed(() => {
  const present = new Set(questions.value.map((q) => q.status))
  return [...present].map((s) => STATUS_LABELS[s])
})
const statusChartValues = computed(() => {
  const counts: Record<string, number> = {}
  for (const q of questions.value) counts[q.status] = (counts[q.status] || 0) + 1
  const present = [...new Set(questions.value.map((q) => q.status))]
  return present.map((s) => counts[s])
})

// Type distribution (donut)
const typeChartLabels = computed(() => {
  const present = [...new Set(questions.value.map((q) => q.question_type))]
  return present.map((t) => QTYPE_LABELS[t])
})
const typeChartValues = computed(() => {
  const counts: Record<string, number> = {}
  for (const q of questions.value) counts[q.question_type] = (counts[q.question_type] || 0) + 1
  const present = [...new Set(questions.value.map((q) => q.question_type))]
  return present.map((t) => counts[t])
})

// Subject breakdown (horizontal bar)
const subjectChartLabels = computed(() => {
  const counts = new Map<string, number>()
  for (const q of questions.value) counts.set(q.subject, (counts.get(q.subject) || 0) + 1)
  return [...counts.keys()]
})
const subjectChartSeries = computed(() => {
  const counts = new Map<string, number>()
  for (const q of questions.value) counts.set(q.subject, (counts.get(q.subject) || 0) + 1)
  return [{ label: 'Jumlah Soal', data: [...counts.values()], color: '#6366f1' }]
})

// Weekly creation trend (last 8 weeks, based on real created_at)
const weeklyTrend = computed(() => {
  const buckets = new Map<string, { count: number; weekStart: number }>()
  for (const q of questions.value) {
    const d = new Date(q.created_at)
    const monday = new Date(d); monday.setHours(0, 0, 0, 0); monday.setDate(d.getDate() - ((d.getDay() + 6) % 7))
    const key = monday.toISOString().slice(0, 10)
    const b = buckets.get(key) ?? { count: 0, weekStart: monday.getTime() }
    b.count += 1
    buckets.set(key, b)
  }
  return [...buckets.values()].sort((a, b) => a.weekStart - b.weekStart).slice(-8)
})
const trendChartLabels = computed(() => weeklyTrend.value.map((w) => new Date(w.weekStart).toLocaleDateString('id-ID', { day: '2-digit', month: 'short' })))
const trendChartSeries = computed(() => [{ label: 'Soal Dibuat', data: weeklyTrend.value.map((w) => w.count), color: '#8b5cf6' }])

// Difficulty breakdown — always real (every question has a difficulty), unlike usage-based
// metrics: the backend never updates a question's usage_count/average_score after creation
// (verified — they stay frozen at 0), so a "soal paling sulit" ranking would be permanently
// empty. Difficulty distribution is the honest signal available today.
const DIFF_LABEL: Record<string, string> = { easy: 'Mudah', medium: 'Sedang', hard: 'Sulit' }
const difficultyChartLabels = computed(() => (['easy', 'medium', 'hard'] as const).map((d) => DIFF_LABEL[d]))
const difficultyChartSeries = computed(() => {
  const counts: Record<string, number> = { easy: 0, medium: 0, hard: 0 }
  for (const q of questions.value) counts[q.difficulty] = (counts[q.difficulty] || 0) + 1
  return [{ label: 'Jumlah Soal', data: (['easy', 'medium', 'hard'] as const).map((d) => counts[d]), color: '#f59e0b' }]
})
</script>

<template>
  <div class="space-y-6">
    <div>
      <h2 class="text-xl font-bold text-slate-900 dark:text-white">Analitik Konten</h2>
      <p class="text-sm text-muted-foreground">Performa soal, distribusi, dan statistik bank soal — seluruhnya dari data nyata.</p>
      <p v-if="total > questions.length" class="text-xs text-amber-600 dark:text-amber-400 mt-1">
        Menampilkan distribusi dari {{ questions.length }} soal terbaru dari {{ total }} total.
      </p>
    </div>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat analitik...</div>

    <template v-else>
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card v-for="k in kpis" :key="k.label" class="p-5">
          <div class="w-10 h-10 rounded-xl flex items-center justify-center mb-3" :class="k.bg">
            <component :is="k.icon" class="w-5 h-5" :class="k.c" />
          </div>
          <p class="text-2xl font-black text-slate-900 dark:text-white">{{ k.val }}</p>
          <p class="text-xs text-muted-foreground mt-0.5">{{ k.label }}</p>
          <p class="text-[10px] text-muted-foreground/70">{{ k.sub }}</p>
        </Card>
      </div>

      <div class="grid lg:grid-cols-2 gap-5">
        <Card class="p-6">
          <h3 class="text-sm font-bold text-slate-900 dark:text-white mb-1 flex items-center gap-2"><TrendingUp class="w-4 h-4 text-violet-600 dark:text-violet-400" />Tren Pembuatan Soal (8 Minggu)</h3>
          <BarLineChart :labels="trendChartLabels" :series="trendChartSeries" :height="200" />
        </Card>
        <Card class="p-6">
          <h3 class="text-sm font-bold text-slate-900 dark:text-white mb-1">Status Bank Soal</h3>
          <DonutChart :labels="statusChartLabels" :values="statusChartValues" unit-label=" soal" :height="180" />
        </Card>
        <Card class="p-6">
          <h3 class="text-sm font-bold text-slate-900 dark:text-white mb-1">Distribusi Tipe Soal</h3>
          <DonutChart :labels="typeChartLabels" :values="typeChartValues" unit-label=" soal" :height="180" />
        </Card>
        <Card class="p-6">
          <h3 class="text-sm font-bold text-slate-900 dark:text-white mb-1">Soal per Mata Uji</h3>
          <BarLineChart :labels="subjectChartLabels" :series="subjectChartSeries" :horizontal="true" :height="Math.max(subjectChartLabels.length * 32, 140)" />
        </Card>
      </div>

      <Card class="p-6">
        <h3 class="text-sm font-bold text-slate-900 dark:text-white mb-1">Sebaran Tingkat Kesulitan</h3>
        <BarLineChart :labels="difficultyChartLabels" :series="difficultyChartSeries" :height="180" />
      </Card>
    </template>
  </div>
</template>
