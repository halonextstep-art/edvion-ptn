<script setup lang="ts">
// Laporan — report-builder + saved reports, semua data nyata (tidak ada laporan/skor
// fiktif seperti di referensi). 3 jenis laporan: Performa Siswa & Partisipasi Tryout
// dihitung backend langsung dari roster+attempts nyata sekolah ini (bisa difilter per
// kelas nyata dari data siswa), dan Analisis Target PTN memakai ulang agregat
// rasionalisasi yang sudah nyata (dashboard SNBP + Rekap SNBT) — jenis ini selalu
// mencakup seluruh sekolah karena rasionalisasi belum mendukung breakdown per kelas,
// jadi filter kelas dinonaktifkan untuk jenis ini (didokumentasikan ke user, bukan
// dipura-purakan berfungsi). "Periode" dipilih dari bulan-bulan yang benar-benar ada
// datanya (diturunkan dari tren skor asli), bukan daftar bulan hardcode seperti
// referensi. Setiap laporan yang di-generate disimpan permanen di backend sebagai
// snapshot — membuka laporan lama menampilkan data persis saat dibuat, bukan
// dihitung ulang live. Selain "Cetak" (window.print), tersedia juga unduh PDF
// (jsPDF+autoTable) dan Excel (SheetJS) sungguhan — dibangun langsung dari
// `ReportItem.payload` yang sama persis dengan yang ditampilkan di dialog ini, lihat
// utils/reportExport.ts.
import {
  FileText, Printer, Loader2, AlertCircle, ListChecks, TrendingUp, GraduationCap, Target, Users, Calendar,
  Filter, Sparkles, Trash2, Eye, X, ClipboardList, BarChart2, Download, FileSpreadsheet, ShieldCheck,
} from 'lucide-vue-next'
import type {
  SchoolOverview, User, ScoreTrendPoint, ReportTypeValue, ReportItem, ReportSummaryItem, ReportExamTrackFilter,
} from '~/types'
import { exportReportToPdf, exportReportToExcel } from '~/utils/reportExport'

const { schoolService, userService, analyticsService, reportService } = useApi()
const toast = useToast()

const REPORT_TYPES: { key: ReportTypeValue; label: string; desc: string; icon: any }[] = [
  { key: 'performance', label: 'Performa Siswa', desc: 'Skor rata-rata per kelas, tren bulanan, dan ranking internal.', icon: ListChecks },
  { key: 'participation', label: 'Partisipasi Tryout', desc: 'Keaktifan siswa mengikuti tryout/drilling — siapa aktif, siapa belum.', icon: Users },
  { key: 'ptn-target', label: 'Analisis Target PTN', desc: 'Sebaran target universitas dan status rasionalisasi SNBP/SNBT.', icon: Target },
  { key: 'tka', label: 'Hasil TKA', desc: 'Skor mata uji wajib & pilihan, progres pilih mapel pilihan, kategori Istimewa.', icon: ClipboardList },
  { key: 'akreditasi', label: 'Laporan Akreditasi (EDS)', desc: 'Ringkasan prestasi akademik siswa untuk dokumentasi akreditasi — bukan export Dapodik lengkap.', icon: ShieldCheck },
]

const EXAM_TRACK_OPTIONS: { value: ReportExamTrackFilter; label: string }[] = [
  { value: null, label: 'Semua Jalur' },
  { value: 'snbt', label: 'SNBT / UTBK' },
  { value: 'tka', label: 'TKA' },
]

// ── Context data (untuk builder: opsi periode & kelas nyata, dan stat cards) ──
const overview = ref<SchoolOverview | null>(null)
const students = ref<User[]>([])
const trend = ref<ScoreTrendPoint[]>([])
const loadingContext = ref(true)
const contextError = ref<string | null>(null)

async function loadContext() {
  loadingContext.value = true
  contextError.value = null
  try {
    const [o, s, t] = await Promise.all([
      schoolService.overview(),
      userService.list({ status: 'active' }),
      analyticsService.schoolScoreTrend(180),
    ])
    overview.value = o
    students.value = s
    trend.value = t
  } catch (e: any) {
    contextError.value = e?.message || 'Gagal memuat data sekolah.'
    toast.error('Gagal memuat data', contextError.value)
  } finally {
    loadingContext.value = false
  }
}

// ── Saved reports ──
const savedReports = ref<ReportSummaryItem[]>([])
const loadingSaved = ref(true)
const savedError = ref<string | null>(null)
async function loadSaved() {
  loadingSaved.value = true
  savedError.value = null
  try {
    savedReports.value = await reportService.list()
  } catch (e: any) {
    savedError.value = e?.message || 'Gagal memuat laporan tersimpan.'
    toast.error('Gagal memuat laporan tersimpan', savedError.value)
  } finally {
    loadingSaved.value = false
  }
}

onMounted(() => {
  loadContext()
  loadSaved()
})

// ── Builder state ──
const selectedType = ref<ReportTypeValue>('performance')
const selectedYearMonth = ref<string | null>(null) // null = Semua Waktu
const selectedClass = ref<string | null>(null) // null = Semua Kelas
const selectedExamTrack = ref<ReportExamTrackFilter>(null) // null = Semua Jalur

const availablePeriods = computed(() => {
  const months = new Set<string>()
  for (const t of trend.value) {
    if (t.attempts > 0) months.add(t.day.slice(0, 7))
  }
  return [...months].sort().reverse().map((ym) => ({ value: ym, label: monthLabel(ym) }))
})
function monthLabel(ym: string): string {
  const [y, m] = ym.split('-').map(Number)
  return new Date(y, m - 1, 1).toLocaleDateString('id-ID', { month: 'long', year: 'numeric' })
}
const periodLabel = computed(() => (selectedYearMonth.value ? monthLabel(selectedYearMonth.value) : 'Semua Waktu'))

const availableClasses = computed(() => {
  const grades = new Set<string>()
  for (const s of students.value) if (s.grade) grades.add(s.grade)
  return [...grades].sort()
})
const isPtnTarget = computed(() => selectedType.value === 'ptn-target')
const isAkreditasi = computed(() => selectedType.value === 'akreditasi')
// Jalur Ujian filter only makes sense for laporan yang menghitung dari Attempt langsung
// (Performa/Partisipasi) — PtnTarget tidak punya dimensi jalur ujian sama sekali, dan laporan
// TKA sudah implisit TKA-only.
const examTrackFilterApplicable = computed(() => selectedType.value === 'performance' || selectedType.value === 'participation')

// Ambang nilai kelulusan ("KKM") — hanya untuk laporan Akreditasi, lihat backend
// `domain::report::AkreditasiReportPayload` doc comment. Bukan setting sekolah tersimpan,
// hanya parameter per-generate (default 75).
const thresholdInput = ref<number>(75)

const scopedStudentCount = computed(() => {
  if (isPtnTarget.value || !selectedClass.value) return students.value.length
  return students.value.filter((s) => s.grade === selectedClass.value).length
})

// ── Generate ──
const generating = ref(false)
const activeReport = ref<ReportItem | null>(null)
const showDetail = ref(false)

async function generate() {
  generating.value = true
  try {
    const report = await reportService.generate({
      report_type: selectedType.value,
      period_label: periodLabel.value,
      year_month: selectedYearMonth.value,
      class_filter: isPtnTarget.value ? null : selectedClass.value,
      exam_track_filter: examTrackFilterApplicable.value ? selectedExamTrack.value : null,
      threshold: isAkreditasi.value ? thresholdInput.value : null,
    })
    toast.success('Laporan berhasil dibuat')
    activeReport.value = report
    showDetail.value = true
    loadSaved()
  } catch (e: any) {
    toast.error('Gagal membuat laporan', e?.message)
  } finally {
    generating.value = false
  }
}

// ── View / delete saved report ──
const loadingDetail = ref(false)
async function viewReport(id: string) {
  loadingDetail.value = true
  try {
    activeReport.value = await reportService.get(id)
    showDetail.value = true
  } catch (e: any) {
    toast.error('Gagal memuat laporan', e?.message)
  } finally {
    loadingDetail.value = false
  }
}
function closeDetail() {
  showDetail.value = false
}
function printReport() {
  window.print()
}
function downloadPdf() {
  if (!activeReport.value) return
  try {
    exportReportToPdf(activeReport.value)
  } catch (e: any) {
    toast.error('Gagal membuat PDF', e?.message)
  }
}
function downloadExcel() {
  if (!activeReport.value) return
  try {
    exportReportToExcel(activeReport.value)
  } catch (e: any) {
    toast.error('Gagal membuat Excel', e?.message)
  }
}

// Unduh langsung dari daftar tersimpan tanpa membuka dialog detail dulu.
const downloadingRowId = ref<string | null>(null)
async function downloadRow(id: string, kind: 'pdf' | 'excel') {
  downloadingRowId.value = id
  try {
    const full = await reportService.get(id)
    if (kind === 'pdf') exportReportToPdf(full)
    else exportReportToExcel(full)
  } catch (e: any) {
    toast.error('Gagal mengunduh laporan', e?.message)
  } finally {
    downloadingRowId.value = null
  }
}

const confirmDeleteId = ref<string | null>(null)
async function deleteReport() {
  if (!confirmDeleteId.value) return
  try {
    await reportService.delete(confirmDeleteId.value)
    toast.success('Laporan dihapus')
    loadSaved()
  } catch (e: any) {
    toast.error('Gagal menghapus laporan', e?.message)
  } finally {
    confirmDeleteId.value = null
  }
}

function typeLabel(t: ReportTypeValue) {
  return REPORT_TYPES.find((r) => r.key === t)?.label || t
}
function typeIcon(t: ReportTypeValue) {
  return REPORT_TYPES.find((r) => r.key === t)?.icon || FileText
}
function examTrackLabel(v: ReportExamTrackFilter) {
  return EXAM_TRACK_OPTIONS.find((o) => o.value === v)?.label ?? null
}
function formatDateTime(iso: string) {
  return new Date(iso).toLocaleString('id-ID', { day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' })
}

// Data chart untuk Tren Skor Bulanan (performance) & Sebaran Universitas Tujuan (ptn-target)
const monthlyChartLabels = computed(() => {
  if (!activeReport.value || activeReport.value.payload.kind !== 'performance') return []
  return activeReport.value.payload.monthly_trend.map((m) => m.month_label)
})
const monthlyChartSeries = computed(() => {
  if (!activeReport.value || activeReport.value.payload.kind !== 'performance') return []
  return [{ label: 'Rata-rata Skor', data: activeReport.value.payload.monthly_trend.map((m) => (m.avg_score != null ? Math.round(m.avg_score) : null)), color: '#4f46e5' }]
})
const uniChartLabels = computed(() => {
  if (!activeReport.value || activeReport.value.payload.kind !== 'ptn-target') return []
  return activeReport.value.payload.distribusi_universitas.map((d) => d.nama_ptn)
})
const uniChartValues = computed(() => {
  if (!activeReport.value || activeReport.value.payload.kind !== 'ptn-target') return []
  return activeReport.value.payload.distribusi_universitas.map((d) => d.count)
})
</script>

<template>
  <div class="space-y-6">
    <div>
      <h2 class="text-lg font-bold flex items-center gap-2"><FileText class="w-5 h-5 text-blue-600" />Laporan</h2>
      <p class="text-sm text-muted-foreground">Buat dan simpan laporan performa, partisipasi, atau target PTN — semua dari data nyata sekolah ini.</p>
    </div>

    <div v-if="loadingContext" class="py-10 text-center text-muted-foreground"><Loader2 class="w-6 h-6 animate-spin mx-auto mb-2" /> Memuat data sekolah...</div>
    <Card v-else-if="contextError" class="p-8 text-center border-red-200 bg-red-50">
      <AlertCircle class="w-8 h-8 mx-auto mb-2 text-red-500" />
      <p class="text-sm text-red-700 mb-3">{{ contextError }}</p>
      <Button variant="outline" size="sm" @click="loadContext">Coba Lagi</Button>
    </Card>

    <template v-else>
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card class="p-5"><p class="text-xs text-muted-foreground mb-1">Total Siswa</p><p class="text-2xl font-black text-slate-900">{{ overview?.total_students ?? 0 }}</p></Card>
        <Card class="p-5"><p class="text-xs text-muted-foreground mb-1">Rata-rata Skor</p><p class="text-2xl font-black text-slate-900">{{ overview?.avg_score ?? '-' }}</p></Card>
        <Card class="p-5"><p class="text-xs text-muted-foreground mb-1">Tryout Selesai</p><p class="text-2xl font-black text-slate-900">{{ overview?.tryouts_completed ?? 0 }}</p></Card>
        <Card class="p-5"><p class="text-xs text-muted-foreground mb-1">Siswa Punya Target PTN</p><p class="text-2xl font-black text-slate-900">{{ overview?.target_ptn_count ?? 0 }}</p></Card>
      </div>

      <!-- Builder -->
      <Card class="p-5">
        <h4 class="font-bold text-slate-900 mb-4 flex items-center gap-2"><Sparkles class="w-4 h-4 text-indigo-600" /> Generate Laporan Baru</h4>
        <div class="grid lg:grid-cols-3 gap-5">
          <div>
            <Label class="mb-2 block">Jenis Laporan</Label>
            <div class="space-y-2">
              <button
                v-for="rt in REPORT_TYPES" :key="rt.key" type="button"
                class="w-full text-left p-3 rounded-xl border-2 transition-colors"
                :class="selectedType === rt.key ? 'border-indigo-500 bg-indigo-50' : 'border-slate-200 hover:bg-slate-50'"
                @click="selectedType = rt.key"
              >
                <div class="flex items-center gap-2 font-semibold text-sm" :class="selectedType === rt.key ? 'text-indigo-700' : 'text-slate-800'">
                  <component :is="rt.icon" class="w-4 h-4" />{{ rt.label }}
                </div>
                <p class="text-xs text-muted-foreground mt-0.5">{{ rt.desc }}</p>
              </button>
            </div>
          </div>

          <div class="space-y-4">
            <div>
              <Label class="mb-2 flex items-center gap-1.5"><Calendar class="w-3.5 h-3.5" />Periode</Label>
              <select v-model="selectedYearMonth" class="w-full px-3 py-2 border rounded-lg text-sm bg-white">
                <option :value="null">Semua Waktu</option>
                <option v-for="p in availablePeriods" :key="p.value" :value="p.value">{{ p.label }}</option>
              </select>
              <p v-if="availablePeriods.length === 0" class="text-[11px] text-muted-foreground mt-1">Belum ada data bulanan — hanya "Semua Waktu" yang tersedia.</p>
            </div>
            <div>
              <Label class="mb-2 flex items-center gap-1.5"><Filter class="w-3.5 h-3.5" />Filter Kelas</Label>
              <select v-model="selectedClass" class="w-full px-3 py-2 border rounded-lg text-sm bg-white" :disabled="isPtnTarget">
                <option :value="null">Semua Kelas</option>
                <option v-for="g in availableClasses" :key="g" :value="g">{{ g }}</option>
              </select>
              <p v-if="isPtnTarget" class="text-[11px] text-amber-600 mt-1">Analisis Target PTN selalu mencakup seluruh sekolah (belum mendukung filter kelas).</p>
            </div>
            <div>
              <Label class="mb-2 flex items-center gap-1.5"><Filter class="w-3.5 h-3.5" />Filter Jalur Ujian</Label>
              <select v-model="selectedExamTrack" class="w-full px-3 py-2 border rounded-lg text-sm bg-white" :disabled="!examTrackFilterApplicable">
                <option v-for="opt in EXAM_TRACK_OPTIONS" :key="opt.label" :value="opt.value">{{ opt.label }}</option>
              </select>
              <p v-if="!examTrackFilterApplicable" class="text-[11px] text-amber-600 mt-1">
                {{ selectedType === 'tka' ? 'Laporan Hasil TKA sudah otomatis khusus jalur TKA.' : 'Analisis Target PTN tidak punya dimensi jalur ujian.' }}
              </p>
              <p v-else class="text-[11px] text-muted-foreground mt-1">Skor SNBT dan TKA dihitung terpisah — pilih salah satu supaya rata-rata tidak tercampur.</p>
            </div>
            <div v-if="isAkreditasi">
              <Label class="mb-2 flex items-center gap-1.5"><ShieldCheck class="w-3.5 h-3.5" />Ambang Nilai Kelulusan (KKM)</Label>
              <input v-model.number="thresholdInput" type="number" min="0" max="100" class="w-full px-3 py-2 border rounded-lg text-sm bg-white" />
              <p class="text-[11px] text-muted-foreground mt-1">Rata-rata skor siswa ≥ nilai ini dianggap "di atas ambang". Default 75, bisa diubah per laporan.</p>
            </div>
          </div>

          <div class="p-4 rounded-xl bg-slate-50 border flex flex-col justify-between">
            <div>
              <p class="text-xs font-bold text-slate-500 uppercase mb-2">Ringkasan Laporan</p>
              <dl class="space-y-1.5 text-sm">
                <div class="flex justify-between"><dt class="text-muted-foreground">Jenis:</dt><dd class="font-semibold text-slate-900">{{ typeLabel(selectedType) }}</dd></div>
                <div class="flex justify-between"><dt class="text-muted-foreground">Periode:</dt><dd class="font-semibold text-slate-900">{{ periodLabel }}</dd></div>
                <div class="flex justify-between"><dt class="text-muted-foreground">Cakupan:</dt><dd class="font-semibold text-slate-900">{{ isPtnTarget ? 'Seluruh Sekolah' : (selectedClass || 'Semua Kelas') }}</dd></div>
                <div class="flex justify-between">
                  <dt class="text-muted-foreground">Jalur Ujian:</dt>
                  <dd class="font-semibold text-slate-900">{{ selectedType === 'tka' ? 'TKA' : (examTrackFilterApplicable ? (EXAM_TRACK_OPTIONS.find((o) => o.value === selectedExamTrack)?.label ?? 'Semua Jalur') : '—') }}</dd>
                </div>
                <div class="flex justify-between"><dt class="text-muted-foreground">Siswa:</dt><dd class="font-semibold text-slate-900">{{ scopedStudentCount }} siswa</dd></div>
                <div v-if="isAkreditasi" class="flex justify-between"><dt class="text-muted-foreground">KKM:</dt><dd class="font-semibold text-slate-900">{{ thresholdInput }}</dd></div>
              </dl>
            </div>
            <Button variant="gradient" class="mt-4 w-full" :disabled="generating" @click="generate">
              <Sparkles class="w-4 h-4" /> {{ generating ? 'Generating...' : 'Generate Laporan' }}
            </Button>
          </div>
        </div>
      </Card>

      <!-- Saved reports -->
      <Card class="overflow-hidden">
        <div class="p-5 border-b">
          <h4 class="font-bold text-slate-900 flex items-center gap-2"><ClipboardList class="w-4 h-4 text-indigo-600" /> Laporan Tersimpan</h4>
          <p class="text-xs text-muted-foreground mt-0.5">Laporan yang sudah pernah dibuat.</p>
        </div>
        <div v-if="loadingSaved" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
        <Card v-else-if="savedError" class="p-8 text-center border-red-200 bg-red-50 m-4">
          <AlertCircle class="w-8 h-8 mx-auto mb-2 text-red-500" />
          <p class="text-sm text-red-700 mb-3">{{ savedError }}</p>
          <Button variant="outline" size="sm" @click="loadSaved">Coba Lagi</Button>
        </Card>
        <div v-else-if="savedReports.length === 0" class="p-10 text-center text-sm text-muted-foreground">Belum ada laporan. Generate laporan pertama di atas.</div>
        <div v-else class="divide-y divide-slate-50">
          <div v-for="r in savedReports" :key="r.id" class="flex items-center justify-between gap-3 p-4 hover:bg-slate-50">
            <div class="flex items-center gap-3 min-w-0">
              <div class="w-9 h-9 rounded-lg bg-indigo-50 flex items-center justify-center shrink-0">
                <component :is="typeIcon(r.report_type)" class="w-4 h-4 text-indigo-600" />
              </div>
              <div class="min-w-0">
                <p class="text-sm font-semibold text-slate-900 truncate">{{ r.title }}</p>
                <div class="flex items-center gap-2 mt-0.5 flex-wrap">
                  <Badge variant="outline" class="text-[10px]">{{ typeLabel(r.report_type) }}</Badge>
                  <Badge v-if="r.class_filter" variant="outline" class="text-[10px]">{{ r.class_filter }}</Badge>
                  <Badge v-if="r.exam_track_filter" variant="outline" class="text-[10px]">{{ examTrackLabel(r.exam_track_filter) }}</Badge>
                  <span class="text-[11px] text-muted-foreground flex items-center gap-1"><Calendar class="w-3 h-3" />{{ formatDateTime(r.created_at) }}</span>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-1 shrink-0">
              <button class="p-2 rounded-lg hover:bg-indigo-50 text-slate-400 hover:text-indigo-600" title="Lihat" @click="viewReport(r.id)"><Eye class="w-4 h-4" /></button>
              <button v-if="r.report_type !== 'akreditasi'" class="p-2 rounded-lg hover:bg-indigo-50 text-slate-400 hover:text-indigo-600 disabled:opacity-50" title="Unduh PDF" :disabled="downloadingRowId === r.id" @click="downloadRow(r.id, 'pdf')">
                <Loader2 v-if="downloadingRowId === r.id" class="w-4 h-4 animate-spin" /><Download v-else class="w-4 h-4" />
              </button>
              <button class="p-2 rounded-lg hover:bg-emerald-50 text-slate-400 hover:text-emerald-600 disabled:opacity-50" title="Unduh Excel" :disabled="downloadingRowId === r.id" @click="downloadRow(r.id, 'excel')">
                <Loader2 v-if="downloadingRowId === r.id" class="w-4 h-4 animate-spin" /><FileSpreadsheet v-else class="w-4 h-4" />
              </button>
              <button class="p-2 rounded-lg hover:bg-red-50 text-slate-400 hover:text-red-600" title="Hapus" @click="confirmDeleteId = r.id"><Trash2 class="w-4 h-4" /></button>
            </div>
          </div>
        </div>
      </Card>
    </template>

    <!-- Detail dialog -->
    <Dialog v-model="showDetail" :title="activeReport?.title || 'Laporan'" max-width="max-w-3xl">
      <div v-if="loadingDetail" class="py-16 text-center text-muted-foreground"><Loader2 class="w-6 h-6 animate-spin mx-auto" /></div>
      <div v-else-if="activeReport" class="space-y-5">
        <div class="flex items-center justify-between flex-wrap gap-2">
          <div class="flex items-center gap-2 flex-wrap">
            <Badge variant="outline">{{ typeLabel(activeReport.report_type) }}</Badge>
            <Badge variant="outline">{{ activeReport.period_label }}</Badge>
            <Badge v-if="activeReport.class_filter" variant="outline">{{ activeReport.class_filter }}</Badge>
            <Badge v-if="activeReport.exam_track_filter" variant="outline">{{ examTrackLabel(activeReport.exam_track_filter) }}</Badge>
            <span class="text-xs text-muted-foreground">Dibuat {{ formatDateTime(activeReport.created_at) }}</span>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <Button v-if="activeReport.payload.kind !== 'akreditasi'" variant="outline" size="sm" @click="downloadPdf"><Download class="w-4 h-4" />Unduh PDF</Button>
            <Button variant="outline" size="sm" @click="downloadExcel"><FileSpreadsheet class="w-4 h-4" />Unduh Excel</Button>
            <Button variant="outline" size="sm" @click="printReport"><Printer class="w-4 h-4" />Cetak</Button>
          </div>
        </div>

        <!-- Performance -->
        <template v-if="activeReport.payload.kind === 'performance'">
          <div class="grid grid-cols-2 gap-4">
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Total Siswa</p><p class="text-xl font-black text-slate-900">{{ activeReport.payload.total_students }}</p></Card>
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Rata-rata Skor</p><p class="text-xl font-black text-indigo-600">{{ activeReport.payload.avg_score != null ? Math.round(activeReport.payload.avg_score) : '-' }}</p></Card>
          </div>
          <div>
            <h5 class="font-bold text-slate-900 mb-2 flex items-center gap-2 text-sm"><GraduationCap class="w-4 h-4 text-indigo-600" />Rata-rata per Kelas</h5>
            <div v-if="activeReport.payload.per_class.length === 0" class="py-6 text-center text-sm text-muted-foreground">Belum ada data.</div>
            <div v-else class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead class="text-xs uppercase text-slate-500 border-b"><tr><th class="text-left py-2">Kelas</th><th class="text-right py-2">Jumlah Siswa</th><th class="text-right py-2">Rata-rata</th></tr></thead>
                <tbody class="divide-y">
                  <tr v-for="c in activeReport.payload.per_class" :key="c.grade">
                    <td class="py-2 font-medium text-slate-800">{{ c.grade }}</td>
                    <td class="py-2 text-right text-muted-foreground">{{ c.student_count }}</td>
                    <td class="py-2 text-right font-bold text-slate-900">{{ c.avg_score != null ? Math.round(c.avg_score) : '-' }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
          <div>
            <h5 class="font-bold text-slate-900 mb-2 flex items-center gap-2 text-sm"><TrendingUp class="w-4 h-4 text-indigo-600" />Tren Skor Bulanan</h5>
            <BarLineChart :labels="monthlyChartLabels" :series="monthlyChartSeries" :height="180" empty-label="Belum cukup data." />
          </div>
          <div>
            <h5 class="font-bold text-slate-900 mb-2 flex items-center gap-2 text-sm"><BarChart2 class="w-4 h-4 text-indigo-600" />Ranking Internal (Top 10)</h5>
            <div v-if="activeReport.payload.ranking.length === 0" class="py-6 text-center text-sm text-muted-foreground">Belum ada siswa dengan attempt selesai.</div>
            <div v-else class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead class="text-xs uppercase text-slate-500 border-b"><tr><th class="text-left py-2">Siswa</th><th class="text-left py-2">Kelas</th><th class="text-right py-2">Attempt</th><th class="text-right py-2">Terbaik</th><th class="text-right py-2">Rata-rata</th></tr></thead>
                <tbody class="divide-y">
                  <tr v-for="s in activeReport.payload.ranking" :key="s.student_id">
                    <td class="py-2 font-medium text-slate-900">{{ s.student_name }}</td>
                    <td class="py-2 text-muted-foreground">{{ s.grade || '-' }}</td>
                    <td class="py-2 text-right text-muted-foreground">{{ s.attempt_count }}</td>
                    <td class="py-2 text-right font-semibold text-slate-900">{{ s.best_score ?? '-' }}</td>
                    <td class="py-2 text-right text-muted-foreground">{{ s.avg_score != null ? Math.round(s.avg_score) : '-' }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </template>

        <!-- Participation -->
        <template v-else-if="activeReport.payload.kind === 'participation'">
          <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Total Siswa</p><p class="text-xl font-black text-slate-900">{{ activeReport.payload.total_students }}</p></Card>
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Siswa Aktif</p><p class="text-xl font-black text-emerald-600">{{ activeReport.payload.active_students }}</p></Card>
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Tingkat Partisipasi</p><p class="text-xl font-black text-indigo-600">{{ Math.round(activeReport.payload.participation_rate) }}%</p></Card>
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Total Attempt</p><p class="text-xl font-black text-slate-900">{{ activeReport.payload.total_attempts }}</p></Card>
          </div>
          <div>
            <h5 class="font-bold text-slate-900 mb-2 flex items-center gap-2 text-sm"><Users class="w-4 h-4 text-indigo-600" />Detail per Siswa</h5>
            <div v-if="activeReport.payload.rows.length === 0" class="py-6 text-center text-sm text-muted-foreground">Belum ada siswa.</div>
            <div v-else class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead class="text-xs uppercase text-slate-500 border-b"><tr><th class="text-left py-2">Siswa</th><th class="text-left py-2">Kelas</th><th class="text-right py-2">Jumlah Attempt</th><th class="text-right py-2">Terakhir Aktif</th></tr></thead>
                <tbody class="divide-y">
                  <tr v-for="row in activeReport.payload.rows" :key="row.student_id">
                    <td class="py-2 font-medium text-slate-900">{{ row.student_name }}</td>
                    <td class="py-2 text-muted-foreground">{{ row.grade || '-' }}</td>
                    <td class="py-2 text-right" :class="row.attempt_count === 0 ? 'text-red-500' : 'text-slate-900 font-semibold'">{{ row.attempt_count }}</td>
                    <td class="py-2 text-right text-muted-foreground">{{ row.last_attempt_at ? formatDateTime(row.last_attempt_at) : '-' }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </template>

        <!-- PTN Target -->
        <template v-else-if="activeReport.payload.kind === 'ptn-target'">
          <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Siswa Punya Target</p><p class="text-xl font-black text-slate-900">{{ activeReport.payload.total_siswa_dengan_target }}</p></Card>
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Rata-rata Peluang SNBP</p><p class="text-xl font-black text-indigo-600">{{ activeReport.payload.avg_chance_snbp != null ? Math.round(activeReport.payload.avg_chance_snbp) + '%' : '-' }}</p></Card>
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">SNBT Sudah Ujian</p><p class="text-xl font-black text-blue-600">{{ activeReport.payload.snbt_sudah_ujian }}</p></Card>
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">SNBT Diterima</p><p class="text-xl font-black text-emerald-600">{{ activeReport.payload.snbt_diterima }}</p></Card>
          </div>
          <div>
            <h5 class="font-bold text-slate-900 mb-2 flex items-center gap-2 text-sm"><Target class="w-4 h-4 text-indigo-600" />Sebaran Universitas Tujuan (SNBP)</h5>
            <DonutChart :labels="uniChartLabels" :values="uniChartValues" unit-label=" siswa" :height="180" empty-label="Belum ada siswa yang menambahkan target SNBP." />
          </div>
        </template>

        <!-- TKA -->
        <template v-else-if="activeReport.payload.kind === 'tka'">
          <div v-if="activeReport.payload.packages.length === 0" class="py-6 text-center text-sm text-muted-foreground">
            Belum ada paket TKA dengan konten nyata untuk dilaporkan.
          </div>
          <template v-else>
            <div class="grid grid-cols-2 gap-4">
              <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Total Siswa</p><p class="text-xl font-black text-slate-900">{{ activeReport.payload.total_students }}</p></Card>
              <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Jumlah Paket TKA</p><p class="text-xl font-black text-indigo-600">{{ activeReport.payload.packages.length }}</p></Card>
            </div>

            <div>
              <h5 class="font-bold text-slate-900 mb-2 flex items-center gap-2 text-sm"><ListChecks class="w-4 h-4 text-indigo-600" />Ringkasan per Paket</h5>
              <div class="overflow-x-auto">
                <table class="w-full text-sm">
                  <thead class="text-xs uppercase text-slate-500 border-b">
                    <tr>
                      <th class="text-left py-2">Paket</th>
                      <th class="text-left py-2">Skala Nilai</th>
                      <th class="text-right py-2">Siswa</th>
                      <th class="text-right py-2">Sudah Pilih Lengkap</th>
                      <th class="text-right py-2">Rata² Wajib</th>
                      <th class="text-right py-2">Rata² Pilihan</th>
                      <th class="text-right py-2">Rata² Mata Uji Istimewa</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y">
                    <tr v-for="p in activeReport.payload.packages" :key="p.package_id">
                      <td class="py-2 font-medium text-slate-800">{{ p.package_name }}</td>
                      <td class="py-2 text-muted-foreground">{{ p.score_scale_label }} <span class="text-[10px]">(Istimewa ≥{{ p.istimewa_threshold }})</span></td>
                      <td class="py-2 text-right text-muted-foreground">{{ p.student_count }}</td>
                      <td class="py-2 text-right text-muted-foreground">{{ p.elective_ready_count }}/{{ p.student_count }}</td>
                      <td class="py-2 text-right">
                        <span class="font-bold text-slate-900">{{ p.avg_wajib_score != null ? Math.round(p.avg_wajib_score) : '-' }}</span>
                        <span v-if="p.avg_raw_wajib_score != null" class="block text-[10px] text-muted-foreground">skor mentah {{ Math.round(p.avg_raw_wajib_score) }}/1000</span>
                      </td>
                      <td class="py-2 text-right">
                        <span class="font-bold text-slate-900">{{ p.avg_pilihan_score != null ? Math.round(p.avg_pilihan_score) : '-' }}</span>
                        <span v-if="p.avg_raw_pilihan_score != null" class="block text-[10px] text-muted-foreground">skor mentah {{ Math.round(p.avg_raw_pilihan_score) }}/1000</span>
                      </td>
                      <td class="py-2 text-right text-muted-foreground">{{ p.avg_istimewa_count != null ? p.avg_istimewa_count.toFixed(1) : '-' }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <p class="text-[11px] text-muted-foreground mt-2">
                Skala nilai mengikuti jenjang tiap paket (0–100 untuk SD/SMP, 200–800 untuk SMA/SMK/MA per Perka BSKAP) — perkiraan platform dari hasil pengerjaan siswa,
                bukan skor resmi TKA Kemendikdasmen. Angka "skor mentah" kecil menunjukkan skor asli platform (skala 0–1000) sebelum dikonversi.
              </p>
            </div>

            <div>
              <h5 class="font-bold text-slate-900 mb-2 flex items-center gap-2 text-sm"><Users class="w-4 h-4 text-indigo-600" />Detail per Siswa</h5>
              <div v-if="activeReport.payload.students.length === 0" class="py-6 text-center text-sm text-muted-foreground">Belum ada siswa.</div>
              <div v-else class="overflow-x-auto">
                <table class="w-full text-sm">
                  <thead class="text-xs uppercase text-slate-500 border-b">
                    <tr>
                      <th class="text-left py-2">Siswa</th>
                      <th class="text-left py-2">Kelas</th>
                      <th class="text-left py-2">Paket</th>
                      <th class="text-right py-2">Mata Uji Pilihan</th>
                      <th class="text-right py-2">Rata² Wajib</th>
                      <th class="text-right py-2">Rata² Pilihan</th>
                      <th class="text-right py-2">Istimewa</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y">
                    <tr v-for="s in activeReport.payload.students" :key="s.student_id + s.package_name">
                      <td class="py-2 font-medium text-slate-900">{{ s.student_name }}</td>
                      <td class="py-2 text-muted-foreground">{{ s.grade || '-' }}</td>
                      <td class="py-2 text-muted-foreground">{{ s.package_name }}</td>
                      <td class="py-2 text-right text-muted-foreground">{{ s.elective_chosen_count }}/{{ s.elective_pick_count }}</td>
                      <td class="py-2 text-right">
                        <span class="font-semibold text-slate-900">{{ s.avg_wajib_score != null ? Math.round(s.avg_wajib_score) : '-' }}</span>
                        <span v-if="s.avg_raw_wajib_score != null" class="block text-[10px] text-muted-foreground">skor mentah {{ Math.round(s.avg_raw_wajib_score) }}/1000</span>
                      </td>
                      <td class="py-2 text-right">
                        <span class="font-semibold text-slate-900">{{ s.avg_pilihan_score != null ? Math.round(s.avg_pilihan_score) : '-' }}</span>
                        <span v-if="s.avg_raw_pilihan_score != null" class="block text-[10px] text-muted-foreground">skor mentah {{ Math.round(s.avg_raw_pilihan_score) }}/1000</span>
                      </td>
                      <td class="py-2 text-right">
                        <span v-if="s.istimewa_count > 0" class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-semibold bg-amber-100 text-amber-700">{{ s.istimewa_count }}/{{ s.subject_count }}</span>
                        <span v-else class="text-muted-foreground">{{ s.istimewa_count }}/{{ s.subject_count }}</span>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          </template>
        </template>

        <!-- Akreditasi (EDS) -->
        <template v-else-if="activeReport.payload.kind === 'akreditasi'">
          <p class="text-[11px] text-amber-600 bg-amber-50 border border-amber-200 rounded-lg px-3 py-2">
            Laporan ini merangkum prestasi akademik siswa (skor tryout/simulasi & target PTN) untuk membantu pengisian dokumentasi akreditasi (EDS) — bukan export identitas/Dapodik lengkap. Ambang kelulusan (KKM) yang dipakai: <strong>{{ activeReport.payload.threshold_used }}</strong>.
          </p>
          <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Total Siswa</p><p class="text-xl font-black text-slate-900">{{ activeReport.payload.total_students }}</p></Card>
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Rata-rata Skor</p><p class="text-xl font-black text-indigo-600">{{ activeReport.payload.avg_score_overall != null ? Math.round(activeReport.payload.avg_score_overall) : '-' }}</p></Card>
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Siswa di Atas KKM</p><p class="text-xl font-black text-emerald-600">{{ activeReport.payload.students_above_threshold }}</p></Card>
            <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Siswa Punya Target PTN</p><p class="text-xl font-black text-blue-600">{{ activeReport.payload.students_with_ptn_target }}</p></Card>
          </div>

          <div>
            <h5 class="font-bold text-slate-900 mb-2 flex items-center gap-2 text-sm"><BarChart2 class="w-4 h-4 text-indigo-600" />Akurasi per Mata Pelajaran (Seluruh Sekolah)</h5>
            <div v-if="activeReport.payload.per_subject.length === 0" class="py-6 text-center text-sm text-muted-foreground">Belum ada data.</div>
            <div v-else class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead class="text-xs uppercase text-slate-500 border-b"><tr><th class="text-left py-2">Mata Pelajaran</th><th class="text-right py-2">Jumlah Soal Dikerjakan</th><th class="text-right py-2">Akurasi</th></tr></thead>
                <tbody class="divide-y">
                  <tr v-for="sub in activeReport.payload.per_subject" :key="sub.subject">
                    <td class="py-2 font-medium text-slate-800">{{ sub.subject }}</td>
                    <td class="py-2 text-right text-muted-foreground">{{ sub.total }}</td>
                    <td class="py-2 text-right font-bold text-slate-900">{{ sub.total > 0 ? Math.round((sub.correct / sub.total) * 100) : 0 }}%</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div>
            <h5 class="font-bold text-slate-900 mb-2 flex items-center gap-2 text-sm"><GraduationCap class="w-4 h-4 text-indigo-600" />Rata-rata per Kelas</h5>
            <div v-if="activeReport.payload.per_class.length === 0" class="py-6 text-center text-sm text-muted-foreground">Belum ada data.</div>
            <div v-else class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead class="text-xs uppercase text-slate-500 border-b"><tr><th class="text-left py-2">Kelas</th><th class="text-right py-2">Jumlah Siswa</th><th class="text-right py-2">Rata-rata</th><th class="text-right py-2">Di Atas KKM</th></tr></thead>
                <tbody class="divide-y">
                  <tr v-for="c in activeReport.payload.per_class" :key="c.grade">
                    <td class="py-2 font-medium text-slate-800">{{ c.grade }}</td>
                    <td class="py-2 text-right text-muted-foreground">{{ c.student_count }}</td>
                    <td class="py-2 text-right font-bold text-slate-900">{{ c.avg_score != null ? Math.round(c.avg_score) : '-' }}</td>
                    <td class="py-2 text-right text-muted-foreground">{{ c.above_threshold_count }}/{{ c.student_count }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div>
            <h5 class="font-bold text-slate-900 mb-2 flex items-center gap-2 text-sm"><Users class="w-4 h-4 text-indigo-600" />Detail per Siswa</h5>
            <div v-if="activeReport.payload.students.length === 0" class="py-6 text-center text-sm text-muted-foreground">Belum ada siswa.</div>
            <div v-else class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead class="text-xs uppercase text-slate-500 border-b"><tr><th class="text-left py-2">Siswa</th><th class="text-left py-2">Kelas</th><th class="text-right py-2">Attempt</th><th class="text-right py-2">Rata-rata</th><th class="text-right py-2">Status</th><th class="text-right py-2">Target PTN</th></tr></thead>
                <tbody class="divide-y">
                  <tr v-for="s in activeReport.payload.students" :key="s.student_id">
                    <td class="py-2 font-medium text-slate-900">{{ s.student_name }}</td>
                    <td class="py-2 text-muted-foreground">{{ s.grade || '-' }}</td>
                    <td class="py-2 text-right text-muted-foreground">{{ s.attempt_count }}</td>
                    <td class="py-2 text-right font-semibold text-slate-900">{{ s.avg_score != null ? Math.round(s.avg_score) : '-' }}</td>
                    <td class="py-2 text-right">
                      <span v-if="s.avg_score == null" class="text-muted-foreground">-</span>
                      <span v-else-if="s.above_threshold" class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-semibold bg-emerald-100 text-emerald-700">Di Atas KKM</span>
                      <span v-else class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-semibold bg-red-100 text-red-700">Di Bawah KKM</span>
                    </td>
                    <td class="py-2 text-right">
                      <span v-if="s.has_ptn_target" class="text-emerald-600 font-semibold">Ya</span>
                      <span v-else class="text-muted-foreground">-</span>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </template>

        <div class="flex justify-end pt-2 border-t">
          <Button variant="outline" @click="closeDetail"><X class="w-4 h-4" />Tutup</Button>
        </div>
      </div>
    </Dialog>

    <ConfirmDialog
      :model-value="!!confirmDeleteId"
      title="Hapus laporan ini?"
      description="Laporan yang sudah dihapus tidak bisa dikembalikan."
      @update:model-value="(v) => { if (!v) confirmDeleteId = null }"
      @confirm="deleteReport"
    />
  </div>
</template>
