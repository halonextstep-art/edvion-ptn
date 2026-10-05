<script setup lang="ts">
// Rekap SNBT — monitoring pasca-ujian untuk setiap siswa dengan target SNBT nyata yang
// sudah ditambahkan siswa sendiri di tab Rasionalisasi (bukan form pilihan terpisah
// seperti di referensi, karena kita sudah punya data target yang sesungguhnya — tidak
// perlu duplikasi input, makanya tombol "+ Tambah Siswa" di bawah cuma mengarahkan ke tab
// Rasionalisasi, bukan form baru). Skor aktual/tanggal ujian/status/catatan diinput
// manual oleh PIC sekolah setelah pengumuman resmi (satu record per siswa — siswa cuma
// ikut 1 ujian UTBK/SNBT nyata yang menghasilkan 1 skor, dipakai bareng untuk pilihan
// manapun yang diterima; lihat SnbtTracking di backend). Est. Skor berasal dari rata-rata
// tryout siswa (real), PG SNBT dan data persaingan (daya tampung/peminat) dari katalog
// PTN nyata yang sama dengan tab Pos. Minimum. Kolom "vs PG SNBT" membandingkan skor
// aktual terhadap PG SNBT Pilihan 1 (fallback ke Pilihan 2 bila Pilihan 1 kosong) —
// referensi tunggal yang paling relevan karena itu target utama siswa.
//
// Adaptasi sengaja vs referensi:
//  - Kolom "Terdaftar" bukan toggle yang bisa diklik: setiap baris di sini SUDAH pasti
//    terdaftar (syarat masuk roster ini adalah siswa sudah menambahkan target SNBT
//    sendiri), jadi ditampilkan sebagai indikator status saja, bukan kontrol baru tanpa
//    data asli di baliknya.
//  - Filter tahun di referensi tidak dibuat: tidak ada field "tahun pendaftaran SNBT"
//    yang independen di data manapun untuk difilter secara jujur.
import {
  ClipboardList, BookOpen, Medal, AlertCircle, Search, Loader2, CheckCircle2, TrendingDown, MinusCircle,
  Pencil, Save, GraduationCap, Users2, Download, UserPlus, History, X, ChevronDown, ChevronUp, BarChart3, FileDown, Gauge,
} from 'lucide-vue-next'
import type { SnbtDashboardItem, SnbtRosterRowItem, SnbtTrackingStatusValue, AlumniBenchmarkItem, Attempt, AttemptResult } from '~/types'
import { exportAttemptResultToPdf } from '~/utils/attemptExport'

const { rationalizationService } = useApi()
const toast = useToast()

// ── Drill-down: riwayat pengerjaan TO (tryout/drilling/Simulasi UTBK) per siswa ──
const historyRow = ref<SnbtRosterRowItem | null>(null)
const historyAttempts = ref<Attempt[]>([])
const historyLoading = ref(false)
const historyError = ref<string | null>(null)

async function openHistory(row: SnbtRosterRowItem) {
  historyRow.value = row
  historyAttempts.value = []
  historyError.value = null
  historyLoading.value = true
  expandedAttemptId.value = null
  attemptResults.value = {}
  try {
    historyAttempts.value = await rationalizationService.schoolStudentAttempts(row.student_id)
  } catch (e: any) {
    historyError.value = e?.message || 'Gagal memuat riwayat pengerjaan.'
  } finally {
    historyLoading.value = false
  }
}
function closeHistory() {
  historyRow.value = null
}

// ── Detail per-TO: rincian mata uji + kekuatan/kekurangan, dimuat sekali per attempt saat
// barisnya diklik expand (bukan langsung semua attempt sekaligus — riwayat siswa aktif bisa
// panjang, dan sebagian besar baris mungkin tidak pernah dibuka). ──
const expandedAttemptId = ref<string | null>(null)
const attemptResults = ref<Record<string, AttemptResult>>({})
const attemptResultLoading = ref<Set<string>>(new Set())
const attemptResultError = ref<Record<string, string>>({})

async function toggleAttemptDetail(a: Attempt) {
  if (expandedAttemptId.value === a.id) {
    expandedAttemptId.value = null
    return
  }
  expandedAttemptId.value = a.id
  if (attemptResults.value[a.id] || a.status !== 'submitted' || !historyRow.value) return
  const loading = new Set(attemptResultLoading.value)
  loading.add(a.id)
  attemptResultLoading.value = loading
  try {
    attemptResults.value = { ...attemptResults.value, [a.id]: await rationalizationService.schoolStudentAttemptResult(historyRow.value.student_id, a.id) }
  } catch (e: any) {
    attemptResultError.value = { ...attemptResultError.value, [a.id]: e?.message || 'Gagal memuat rincian.' }
  } finally {
    const done = new Set(attemptResultLoading.value)
    done.delete(a.id)
    attemptResultLoading.value = done
  }
}

function subjectAccuracy(correct: number, total: number): number | null {
  return total > 0 ? Math.round((correct / total) * 100) : null
}
// Diurutkan akurasi naik supaya mata uji terlemah tampil paling atas — sama seperti urutan di
// PDF export, konsisten antara tampilan layar dan cetakan.
function sortedBreakdown(attemptId: string) {
  const bd = attemptResults.value[attemptId]?.subject_breakdown || []
  return [...bd].sort((a, b) => (subjectAccuracy(a.correct, a.total) ?? 100) - (subjectAccuracy(b.correct, b.total) ?? 100))
}

const downloadingAttemptId = ref<string | null>(null)
async function downloadAttemptPdf(a: Attempt) {
  if (!historyRow.value) return
  downloadingAttemptId.value = a.id
  try {
    const result = attemptResults.value[a.id] ?? (await rationalizationService.schoolStudentAttemptResult(historyRow.value.student_id, a.id))
    attemptResults.value = { ...attemptResults.value, [a.id]: result }
    exportAttemptResultToPdf(historyRow.value.student_name, a, result.subject_breakdown, {
      irtScore: result.irt_score,
      scoreDisplayMode: result.score_display_mode,
    })
  } catch (e: any) {
    toast.error('Gagal membuat PDF', e?.message)
  } finally {
    downloadingAttemptId.value = null
  }
}
function formatAttemptDate(iso: string | null) {
  if (!iso) return '—'
  return new Date(iso).toLocaleDateString('id-ID', { day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' })
}
const SESSION_TYPE_LABEL: Record<string, string> = {
  tryout: 'Tryout', drilling: 'Drilling', mini: 'Mini Tryout',
}

const STATUS_INFO: Record<SnbtTrackingStatusValue, { label: string; cls: string; dot: string }> = {
  terdaftar: { label: 'Terdaftar', cls: 'bg-slate-100 text-slate-600', dot: 'bg-slate-300' },
  'sudah-ujian': { label: 'Sudah Ujian', cls: 'bg-blue-100 text-blue-700', dot: 'bg-blue-500' },
  diterima: { label: 'Diterima SNBT', cls: 'bg-emerald-100 text-emerald-700', dot: 'bg-emerald-500' },
  'tidak-diterima': { label: 'Tidak Diterima', cls: 'bg-red-100 text-red-700', dot: 'bg-red-500' },
}
const STATUS_OPTIONS: SnbtTrackingStatusValue[] = ['terdaftar', 'sudah-ujian', 'diterima', 'tidak-diterima']

const dashboard = ref<SnbtDashboardItem | null>(null)
const roster = ref<SnbtRosterRowItem[]>([])
const alumni = ref<AlumniBenchmarkItem[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

async function loadAll() {
  loading.value = true
  error.value = null
  try {
    const [d, r, a] = await Promise.all([
      rationalizationService.schoolSnbtDashboard(),
      rationalizationService.schoolSnbtRoster(),
      rationalizationService.listAlumni(),
    ])
    dashboard.value = d
    roster.value = r
    alumni.value = a
  } catch (e: any) {
    error.value = e?.message || 'Gagal memuat Rekap SNBT.'
    toast.error('Gagal memuat Rekap SNBT', error.value)
  } finally {
    loading.value = false
  }
}
onMounted(loadAll)

const search = ref('')
const statusFilter = ref<'all' | SnbtTrackingStatusValue>('all')

// Sudah 1 baris = 1 siswa langsung dari backend — tidak perlu lagi dikelompokkan/
// dipilih representatifnya seperti versi sebelumnya (dulu backend nyimpen tracking per
// target sehingga siswa dengan 2 pilihan bisa punya 2 status berbeda; sekarang tracking
// betul-betul per siswa).
const sortedRoster = computed(() => [...roster.value].sort((a, b) => a.student_name.localeCompare(b.student_name)))

const filtered = computed(() => {
  let list = sortedRoster.value
  if (statusFilter.value !== 'all') list = list.filter((r) => r.status === statusFilter.value)
  const q = search.value.trim().toLowerCase()
  if (q) {
    list = list.filter(
      (r) =>
        r.student_name.toLowerCase().includes(q) ||
        r.pilihan_1?.nama_ptn.toLowerCase().includes(q) ||
        r.pilihan_1?.nama_prodi.toLowerCase().includes(q) ||
        r.pilihan_2?.nama_ptn.toLowerCase().includes(q) ||
        r.pilihan_2?.nama_prodi.toLowerCase().includes(q),
    )
  }
  return list
})

function referenceChoice(r: SnbtRosterRowItem) {
  return r.pilihan_1 ?? r.pilihan_2 ?? null
}
function gap(r: SnbtRosterRowItem): number | null {
  const ref = referenceChoice(r)
  if (!ref || r.actual_score == null) return null
  return Math.round((r.actual_score - ref.pg_snbt) * 10) / 10
}

function exportCsv() {
  const header = ['No', 'Nama', 'Pilihan 1 via SNBT', 'Pilihan 2 via SNBT', 'Est. Skor', 'Skor Aktual', 'vs PG SNBT', 'Tgl Ujian', 'Status']
  const rows = filtered.value.map((r, i) => [
    i + 1,
    r.student_name,
    r.pilihan_1 ? `${r.pilihan_1.nama_ptn} - ${r.pilihan_1.nama_prodi}` : '-',
    r.pilihan_2 ? `${r.pilihan_2.nama_ptn} - ${r.pilihan_2.nama_prodi}` : '-',
    r.est_score != null ? Math.round(r.est_score) : '-',
    r.actual_score ?? '-',
    gap(r) ?? '-',
    r.exam_date || '-',
    STATUS_INFO[r.status].label,
  ])
  const csv = [header, ...rows].map((row) => row.map((c) => `"${String(c).replace(/"/g, '""')}"`).join(',')).join('\n')
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `rekap-snbt-${Date.now()}.csv`
  a.click()
  URL.revokeObjectURL(url)
}

function goAddStudent() {
  toast.info('Tambah target SNBT', 'Target SNBT ditambahkan lewat tab Rasionalisasi (tombol "+ Tambah Data") supaya datanya konsisten dengan yang dipakai di sana — bukan form terpisah di sini.')
}

// ── Panel: Kaka Kelas (alumni SNBT) ──
const alumniSnbt = computed(() => alumni.value.filter((a) => a.track === 'snbt'))

// ── Panel: Tingkat Persaingan per Prodi (dedupe dari pilihan 1 & 2, data katalog nyata) ──
const competition = computed(() => {
  const seen = new Map<string, { nama_ptn: string; nama_prodi: string; daya_tampung: number; peminat: number }>()
  for (const r of roster.value) {
    for (const c of [r.pilihan_1, r.pilihan_2]) {
      if (!c) continue
      const key = `${c.nama_ptn}__${c.nama_prodi}`
      if (!seen.has(key)) seen.set(key, { nama_ptn: c.nama_ptn, nama_prodi: c.nama_prodi, daya_tampung: c.daya_tampung_snbt, peminat: c.peminat_snbt })
    }
  }
  return Array.from(seen.values())
})
function ratio(c: { daya_tampung: number; peminat: number }) {
  if (!c.daya_tampung) return null
  return Math.round(c.peminat / c.daya_tampung)
}
function isHardRatio(c: { daya_tampung: number; peminat: number }) {
  const r = ratio(c)
  return r != null && r > 15
}

// ── Edit modal — satu record tracking per siswa, tidak perlu lagi memilih "pilihan mana" ──
const editingRow = ref<SnbtRosterRowItem | null>(null)
const editScore = ref<number | null>(null)
const editDate = ref<string>('')
const editStatus = ref<SnbtTrackingStatusValue>('terdaftar')
const editNotes = ref('')
const saving = ref(false)

function openEdit(row: SnbtRosterRowItem) {
  editingRow.value = row
  editScore.value = row.actual_score
  editDate.value = row.exam_date || ''
  editStatus.value = row.status
  editNotes.value = row.notes
}
function closeEdit() {
  editingRow.value = null
}
const editGap = computed(() => {
  if (!editingRow.value || editScore.value == null) return null
  const ref = referenceChoice(editingRow.value)
  if (!ref) return null
  return Math.round((editScore.value - ref.pg_snbt) * 10) / 10
})

async function saveEdit() {
  if (!editingRow.value) return
  saving.value = true
  try {
    const updated = await rationalizationService.upsertSnbtTracking(editingRow.value.student_id, {
      actual_score: editScore.value,
      exam_date: editDate.value || null,
      status: editStatus.value,
      notes: editNotes.value,
    })
    const idx = roster.value.findIndex((r) => r.student_id === updated.student_id)
    if (idx !== -1) roster.value[idx] = updated
    dashboard.value = await rationalizationService.schoolSnbtDashboard()
    toast.success('Data SNBT diperbarui')
    closeEdit()
  } catch (e: any) {
    toast.error('Gagal menyimpan data SNBT', e?.message)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="space-y-5">
    <!-- Hero -->
    <div class="relative overflow-hidden rounded-2xl bg-gradient-to-br from-indigo-600 via-blue-600 to-purple-700 p-6 text-white">
      <div class="absolute inset-0 opacity-10" style="background-image: radial-gradient(circle, white 1px, transparent 1px); background-size: 16px 16px;" />
      <div class="relative flex items-center justify-between flex-wrap gap-4">
        <div>
          <p class="text-xs font-semibold uppercase tracking-wide text-indigo-100 mb-1">Monitoring Jalur Tulis</p>
          <h3 class="text-xl font-black flex items-center gap-2"><ClipboardList class="w-5 h-5" /> Rekap SNBT</h3>
          <p class="text-sm text-indigo-100 mt-1">Pantau pendaftaran, skor aktual, dan status penerimaan siswa via SNBT.</p>
        </div>
        <div v-if="dashboard" class="hidden sm:flex items-center gap-6 text-center">
          <div><p class="text-2xl font-black">{{ dashboard.terdaftar }}</p><p class="text-[11px] text-indigo-100">Terdaftar</p></div>
          <div><p class="text-2xl font-black">{{ dashboard.sudah_ujian }}</p><p class="text-[11px] text-indigo-100">Sudah Ujian</p></div>
          <div><p class="text-2xl font-black">{{ dashboard.diterima }}</p><p class="text-[11px] text-indigo-100">Diterima</p></div>
        </div>
      </div>
    </div>

    <div v-if="loading" class="py-16 text-center text-muted-foreground"><Loader2 class="w-6 h-6 animate-spin mx-auto mb-2" /> Memuat Rekap SNBT...</div>
    <Card v-else-if="error" class="p-8 text-center border-red-200 bg-red-50">
      <AlertCircle class="w-8 h-8 mx-auto mb-2 text-red-500" />
      <p class="text-sm text-red-700 mb-3">{{ error }}</p>
      <Button variant="outline" size="sm" @click="loadAll">Coba Lagi</Button>
    </Card>

    <template v-else-if="dashboard">
      <!-- Stat cards -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
        <Card class="p-4"><ClipboardList class="w-4 h-4 text-indigo-600 mb-1.5" /><p class="text-xs text-muted-foreground mb-0.5">Terdaftar SNBT</p><p class="text-2xl font-black text-slate-900">{{ dashboard.terdaftar }}</p></Card>
        <Card class="p-4"><BookOpen class="w-4 h-4 text-blue-600 mb-1.5" /><p class="text-xs text-muted-foreground mb-0.5">Sudah Ujian</p><p class="text-2xl font-black text-blue-600">{{ dashboard.sudah_ujian }}</p></Card>
        <Card class="p-4"><Medal class="w-4 h-4 text-emerald-600 mb-1.5" /><p class="text-xs text-muted-foreground mb-0.5">Diterima via SNBT</p><p class="text-2xl font-black text-emerald-600">{{ dashboard.diterima }}</p></Card>
        <Card class="p-4"><AlertCircle class="w-4 h-4 text-amber-600 mb-1.5" /><p class="text-xs text-muted-foreground mb-0.5">Skor Perlu Diisi</p><p class="text-2xl font-black text-amber-600">{{ dashboard.skor_perlu_diisi }}</p></Card>
      </div>

      <!-- Alert -->
      <div v-if="dashboard.skor_perlu_diisi > 0" class="flex items-start gap-2.5 p-3.5 rounded-xl bg-amber-50 border border-amber-200 text-sm text-amber-800">
        <AlertCircle class="w-4 h-4 mt-0.5 shrink-0" />
        <div>
          <p class="font-semibold">{{ dashboard.skor_perlu_diisi }} siswa sudah lanjut status tapi skor aktual belum diinput.</p>
          <p class="text-xs text-amber-700">Klik ikon edit pada baris siswa untuk memasukkan skor SNBT aktual mereka.</p>
        </div>
      </div>

      <!-- Toolbar -->
      <div class="flex flex-col sm:flex-row gap-2">
        <div class="relative flex-1">
          <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
          <Input v-model="search" placeholder="Cari nama, universitas, atau prodi..." class="pl-9" />
        </div>
        <select v-model="statusFilter" class="px-3 py-2 border rounded-lg text-sm bg-white">
          <option value="all">Semua Status</option>
          <option v-for="s in STATUS_OPTIONS" :key="s" :value="s">{{ STATUS_INFO[s].label }}</option>
        </select>
        <Button variant="outline" size="sm" :disabled="filtered.length === 0" @click="exportCsv"><Download class="w-4 h-4" /> Export</Button>
        <Button variant="outline" size="sm" @click="goAddStudent"><UserPlus class="w-4 h-4" /> Tambah Siswa</Button>
      </div>

      <!-- Roster table -->
      <Card class="overflow-hidden">
        <div v-if="roster.length === 0" class="p-10 text-center text-sm text-muted-foreground">
          <ClipboardList class="w-8 h-8 mx-auto mb-2 text-slate-300" />
          Belum ada siswa dengan target SNBT.
        </div>
        <div v-else-if="filtered.length === 0" class="p-10 text-center text-sm text-muted-foreground">Tidak ada data yang cocok dengan filter.</div>
        <div v-else class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead class="bg-slate-50 text-xs uppercase text-slate-500">
              <tr>
                <th class="text-left px-4 py-3">Siswa</th>
                <th class="text-center px-4 py-3">Terdaftar</th>
                <th class="text-left px-4 py-3">Pilihan 1 via SNBT</th>
                <th class="text-left px-4 py-3">Pilihan 2 via SNBT</th>
                <th class="text-left px-4 py-3">Est. Skor</th>
                <th class="text-left px-4 py-3">Skor Aktual</th>
                <th class="text-left px-4 py-3">vs PG SNBT</th>
                <th class="text-left px-4 py-3">Tgl Ujian</th>
                <th class="text-left px-4 py-3">Status</th>
                <th class="text-right px-4 py-3">Aksi</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-50">
              <tr v-for="r in filtered" :key="r.student_id" class="hover:bg-slate-50">
                <td class="px-4 py-3">
                  <div class="flex items-center gap-2.5">
                    <div class="w-8 h-8 rounded-full bg-indigo-100 text-indigo-700 flex items-center justify-center text-xs font-bold shrink-0">
                      {{ r.student_name.split(' ').map((w) => w[0]).slice(0, 2).join('').toUpperCase() }}
                    </div>
                    <div class="font-semibold text-slate-900">{{ r.student_name }}</div>
                  </div>
                </td>
                <td class="px-4 py-3 text-center">
                  <span
                    class="inline-flex w-8 h-5 rounded-full bg-emerald-500 relative align-middle"
                    title="Otomatis terdaftar — siswa sudah menambahkan target SNBT sendiri"
                  ><span class="absolute top-0.5 left-3.5 w-4 h-4 rounded-full bg-white" /></span>
                </td>
                <td class="px-4 py-3">
                  <div v-if="r.pilihan_1" class="min-w-0">
                    <div class="text-slate-900 truncate">{{ r.pilihan_1.nama_prodi }}</div>
                    <div class="text-xs text-muted-foreground truncate">{{ r.pilihan_1.nama_ptn }}</div>
                  </div>
                  <span v-else class="text-muted-foreground text-xs italic">—</span>
                </td>
                <td class="px-4 py-3">
                  <div v-if="r.pilihan_2" class="min-w-0">
                    <div class="text-slate-900 truncate">{{ r.pilihan_2.nama_prodi }}</div>
                    <div class="text-xs text-muted-foreground truncate">{{ r.pilihan_2.nama_ptn }}</div>
                  </div>
                  <span v-else class="text-muted-foreground text-xs italic">—</span>
                </td>
                <td class="px-4 py-3">
                  <span v-if="r.est_score != null" class="text-slate-900">{{ Math.round(r.est_score) }}</span>
                  <span v-else class="text-muted-foreground italic text-xs">Belum ada tryout</span>
                  <div
                    v-if="r.est_score != null"
                    class="text-[10px]"
                    :class="r.est_score_source === 'simulasi' ? 'text-emerald-600 font-medium' : 'text-muted-foreground'"
                  >
                    {{ r.est_score_source === 'simulasi' ? 'dari Simulasi UTBK' : 'dari rata-rata drilling' }}
                  </div>
                </td>
                <td class="px-4 py-3">
                  <span v-if="r.actual_score != null" class="font-bold text-slate-900">{{ r.actual_score }}</span>
                  <span v-else class="text-muted-foreground italic text-xs">Belum diisi</span>
                </td>
                <td class="px-4 py-3">
                  <div v-if="gap(r) != null" class="flex items-center gap-1" :class="gap(r)! >= 0 ? 'text-emerald-600' : 'text-red-600'">
                    <CheckCircle2 v-if="gap(r)! >= 0" class="w-3.5 h-3.5" /><TrendingDown v-else class="w-3.5 h-3.5" />
                    <span class="font-semibold">{{ gap(r)! > 0 ? '+' : '' }}{{ gap(r) }}</span>
                  </div>
                  <div v-else-if="referenceChoice(r)" class="flex items-center gap-1 text-muted-foreground">
                    <MinusCircle class="w-3.5 h-3.5" /><span class="text-xs">min {{ referenceChoice(r)!.pg_snbt }}</span>
                  </div>
                  <span v-else class="text-muted-foreground text-xs">—</span>
                </td>
                <td class="px-4 py-3 text-slate-600">{{ r.exam_date || '—' }}</td>
                <td class="px-4 py-3"><span class="px-2 py-1 rounded-full text-[11px] font-semibold" :class="STATUS_INFO[r.status].cls">{{ STATUS_INFO[r.status].label }}</span></td>
                <td class="px-4 py-3 text-right">
                  <div class="flex items-center justify-end gap-1">
                    <button class="p-1.5 rounded-lg hover:bg-indigo-50 text-slate-400 hover:text-indigo-600" title="Riwayat Pengerjaan TO" @click="openHistory(r)"><History class="w-4 h-4" /></button>
                    <button class="p-1.5 rounded-lg hover:bg-indigo-50 text-slate-400 hover:text-indigo-600" title="Edit" @click="openEdit(r)"><Pencil class="w-4 h-4" /></button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <div v-if="roster.length > 0" class="px-4 py-2.5 text-xs text-muted-foreground border-t bg-slate-50/50">
          Menampilkan {{ filtered.length }} dari {{ sortedRoster.length }} siswa.
        </div>
      </Card>

      <!-- Benchmark panels -->
      <div class="grid lg:grid-cols-2 gap-4">
        <Card class="p-5">
          <h4 class="font-bold text-slate-900 mb-1 flex items-center gap-2"><GraduationCap class="w-4 h-4 text-indigo-600" /> PM UTBK Kaka Kelas (Alumni)</h4>
          <p class="text-xs text-muted-foreground mb-4">Skor acuan alumni jalur SNBT, diinput manual oleh PIC sekolah.</p>
          <div v-if="alumniSnbt.length === 0" class="py-8 text-center text-sm text-muted-foreground">Belum ada data alumni jalur SNBT.</div>
          <div v-else class="space-y-2 max-h-80 overflow-y-auto">
            <div v-for="a in alumniSnbt" :key="a.id" class="flex items-center justify-between p-3 rounded-xl border">
              <div class="min-w-0">
                <p class="text-sm font-semibold text-slate-900 truncate">{{ a.nama_ptn }} — {{ a.nama_prodi }}</p>
                <p class="text-xs text-muted-foreground">{{ a.alumni_name }} · {{ a.graduation_year }}</p>
              </div>
              <p class="text-lg font-black text-indigo-600 shrink-0 ml-2">{{ a.benchmark_score }}</p>
            </div>
          </div>
        </Card>

        <Card class="p-5">
          <h4 class="font-bold text-slate-900 mb-1 flex items-center gap-2"><Users2 class="w-4 h-4 text-indigo-600" /> Tingkat Persaingan per Prodi</h4>
          <p class="text-xs text-muted-foreground mb-4">Daya tampung &amp; peminat nyata dari katalog untuk prodi yang jadi target siswa.</p>
          <div v-if="competition.length === 0" class="py-8 text-center text-sm text-muted-foreground">Belum ada target SNBT.</div>
          <div v-else class="space-y-2 max-h-80 overflow-y-auto">
            <div v-for="c in competition" :key="`${c.nama_ptn}__${c.nama_prodi}`" class="flex items-center justify-between p-3 rounded-xl border">
              <div class="min-w-0">
                <p class="text-sm font-semibold text-slate-900 truncate">{{ c.nama_ptn }} — {{ c.nama_prodi }}</p>
                <p class="text-xs text-muted-foreground">Kuota {{ c.daya_tampung }} · Peminat {{ c.peminat }}</p>
              </div>
              <p class="text-sm font-black shrink-0 ml-2" :class="isHardRatio(c) ? 'text-red-600' : 'text-amber-600'">
                1:{{ ratio(c) ?? '-' }}<span class="block text-[10px] font-normal text-muted-foreground text-right">persaingan</span>
              </p>
            </div>
          </div>
        </Card>
      </div>
    </template>

    <!-- Edit modal -->
    <Dialog :model-value="!!editingRow" title="Edit Data SNBT" max-width="max-w-md" @update:model-value="(v: boolean) => { if (!v) closeEdit() }">
      <div v-if="editingRow" class="space-y-4">
        <div>
          <p class="text-sm font-semibold text-slate-900">{{ editingRow.student_name }}</p>
          <p class="text-xs text-muted-foreground mt-0.5">
            <span v-if="editingRow.pilihan_1">Pilihan 1: {{ editingRow.pilihan_1.nama_prodi }} · {{ editingRow.pilihan_1.nama_ptn }} (PG SNBT {{ editingRow.pilihan_1.pg_snbt }})</span>
            <span v-if="editingRow.pilihan_2" class="block">Pilihan 2: {{ editingRow.pilihan_2.nama_prodi }} · {{ editingRow.pilihan_2.nama_ptn }} (PG SNBT {{ editingRow.pilihan_2.pg_snbt }})</span>
          </p>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <Label>Skor Aktual SNBT</Label>
            <Input v-model.number="editScore" type="number" min="0" max="1000" placeholder="cth: 698" class="mt-1.5" />
          </div>
          <div>
            <Label>Tanggal Ujian</Label>
            <Input v-model="editDate" type="date" class="mt-1.5" />
          </div>
        </div>
        <div v-if="editGap != null" class="flex items-center gap-2 p-3 rounded-xl text-sm" :class="editGap >= 0 ? 'bg-emerald-50 text-emerald-700 border border-emerald-200' : 'bg-red-50 text-red-700 border border-red-200'">
          <CheckCircle2 v-if="editGap >= 0" class="w-4 h-4 shrink-0" /><TrendingDown v-else class="w-4 h-4 shrink-0" />
          <span>{{ editGap >= 0 ? `Di atas PG SNBT (+${editGap})` : `Di bawah PG SNBT (${editGap})` }}</span>
        </div>
        <div>
          <Label>Status Penerimaan</Label>
          <div class="grid grid-cols-2 gap-2 mt-1.5">
            <button
              v-for="s in STATUS_OPTIONS" :key="s" type="button"
              class="px-3 py-2 rounded-lg text-xs font-semibold border transition-colors"
              :class="editStatus === s ? STATUS_INFO[s].cls + ' border-transparent ring-2 ring-offset-1 ring-indigo-400' : 'bg-white text-slate-500 hover:bg-slate-50'"
              @click="editStatus = s"
            >
              {{ STATUS_INFO[s].label }}
            </button>
          </div>
        </div>
        <div>
          <Label>Catatan</Label>
          <textarea v-model="editNotes" rows="2" class="mt-1.5 w-full px-3 py-2 border rounded-lg text-sm" placeholder="Catatan tambahan untuk siswa ini..." />
        </div>
        <div class="flex items-center justify-between pt-4 border-t">
          <Button variant="outline" @click="closeEdit">Batal</Button>
          <Button variant="gradient" :disabled="saving" @click="saveEdit">
            <Save class="w-4 h-4" /> {{ saving ? 'Menyimpan...' : 'Simpan' }}
          </Button>
        </div>
      </div>
    </Dialog>

    <!-- Riwayat TO drill-down -->
    <Dialog :model-value="!!historyRow" title="Riwayat Pengerjaan TO" max-width="max-w-2xl" @update:model-value="(v: boolean) => { if (!v) closeHistory() }">
      <div v-if="historyRow" class="space-y-4">
        <div class="flex items-center justify-between">
          <p class="text-sm font-semibold text-slate-900">{{ historyRow.student_name }}</p>
          <button class="p-1 rounded hover:bg-slate-100" @click="closeHistory"><X class="w-4 h-4 text-slate-400" /></button>
        </div>

        <div v-if="historyLoading" class="py-10 text-center text-sm text-muted-foreground"><Loader2 class="w-5 h-5 animate-spin mx-auto mb-2" />Memuat riwayat...</div>
        <div v-else-if="historyError" class="p-4 rounded-xl bg-red-50 border border-red-200 text-sm text-red-700">{{ historyError }}</div>
        <div v-else-if="historyAttempts.length === 0" class="py-10 text-center text-sm text-muted-foreground">
          <ClipboardList class="w-8 h-8 mx-auto mb-2 text-slate-300" />
          Siswa ini belum pernah mengerjakan tryout/drilling di aplikasi.
        </div>
        <div v-else class="space-y-2 max-h-[32rem] overflow-y-auto">
          <div v-for="a in historyAttempts" :key="a.id" class="rounded-xl border overflow-hidden">
            <button type="button" class="w-full flex items-center justify-between p-3 text-left hover:bg-slate-50 transition-colors" :disabled="a.status !== 'submitted'" @click="toggleAttemptDetail(a)">
              <div class="min-w-0">
                <p class="text-sm font-semibold text-slate-900 truncate">{{ a.session_title }}</p>
                <p class="text-xs text-muted-foreground">
                  {{ SESSION_TYPE_LABEL[a.session_type] || a.session_type }} · {{ a.status === 'submitted' ? formatAttemptDate(a.submitted_at) : 'Sedang dikerjakan' }}
                </p>
              </div>
              <div class="flex items-center gap-3 shrink-0 ml-3">
                <div class="text-right">
                  <p v-if="a.score != null" class="font-black text-slate-900">{{ a.score }}</p>
                  <p v-else class="text-xs text-slate-400 italic">Belum selesai</p>
                  <p v-if="a.accuracy != null" class="text-[10px] text-muted-foreground">{{ Math.round(a.accuracy) }}% akurasi</p>
                </div>
                <component :is="expandedAttemptId === a.id ? ChevronUp : ChevronDown" v-if="a.status === 'submitted'" class="w-4 h-4 text-slate-400" />
              </div>
            </button>

            <!-- Detail per-TO: rincian mata uji + kekuatan/kekurangan, dimuat on-demand -->
            <div v-if="expandedAttemptId === a.id && a.status === 'submitted'" class="border-t bg-slate-50/70 p-3 space-y-3">
              <div v-if="attemptResultLoading.has(a.id)" class="py-6 text-center text-xs text-muted-foreground"><Loader2 class="w-4 h-4 animate-spin mx-auto mb-1.5" />Memuat rincian...</div>
              <div v-else-if="attemptResultError[a.id]" class="text-xs text-red-600">{{ attemptResultError[a.id] }}</div>
              <template v-else-if="attemptResults[a.id]">
                <div class="grid grid-cols-3 gap-2 text-center">
                  <div class="p-2 rounded-lg bg-white border"><p class="font-black text-emerald-600 text-sm">{{ a.correct_count ?? 0 }}</p><p class="text-[10px] text-muted-foreground">Benar</p></div>
                  <div class="p-2 rounded-lg bg-white border"><p class="font-black text-red-500 text-sm">{{ a.wrong_count ?? 0 }}</p><p class="text-[10px] text-muted-foreground">Salah</p></div>
                  <div class="p-2 rounded-lg bg-white border"><p class="font-black text-slate-500 text-sm">{{ a.unanswered_count ?? 0 }}</p><p class="text-[10px] text-muted-foreground">Kosong</p></div>
                </div>

                <!-- Estimasi IRT — hanya tampil kalau admin mengaktifkan mode 'irt'/'both' di
                     Pengaturan Skor UTBK. Lihat backend domain::irt doc comment: estimasi
                     kalibrasi platform sendiri, bukan skor UTBK resmi SNPMB. -->
                <div v-if="attemptResults[a.id].score_display_mode !== 'instant'" class="flex items-center justify-between p-2 rounded-lg bg-indigo-50 border border-indigo-100 text-xs">
                  <span class="font-semibold text-indigo-700 flex items-center gap-1"><Gauge class="w-3.5 h-3.5" />Estimasi IRT</span>
                  <span class="font-black text-indigo-900">{{ attemptResults[a.id].irt_score != null ? Math.round(attemptResults[a.id].irt_score!) : 'Belum tersedia' }}</span>
                </div>

                <div v-if="sortedBreakdown(a.id).length">
                  <p class="text-[10px] font-bold text-slate-500 uppercase mb-1.5 flex items-center gap-1"><BarChart3 class="w-3 h-3" />Rincian per Mata Uji</p>
                  <div class="space-y-2">
                    <div v-for="b in sortedBreakdown(a.id)" :key="b.subject">
                      <div class="flex justify-between text-xs mb-1">
                        <span class="font-medium text-slate-700">{{ b.subject }}</span>
                        <span class="font-bold" :class="(subjectAccuracy(b.correct, b.total) ?? 100) < 60 ? 'text-red-600' : (subjectAccuracy(b.correct, b.total) ?? 0) >= 80 ? 'text-emerald-600' : 'text-slate-700'">
                          {{ b.correct }}/{{ b.total }} ({{ subjectAccuracy(b.correct, b.total) ?? '-' }}%)
                        </span>
                      </div>
                      <div class="h-1.5 bg-slate-200 rounded-full overflow-hidden">
                        <div
                          class="h-full rounded-full"
                          :class="(subjectAccuracy(b.correct, b.total) ?? 100) < 60 ? 'bg-red-500' : (subjectAccuracy(b.correct, b.total) ?? 0) >= 80 ? 'bg-emerald-500' : 'bg-amber-500'"
                          :style="{ width: `${subjectAccuracy(b.correct, b.total) ?? 0}%` }"
                        />
                      </div>
                    </div>
                  </div>
                </div>
                <p v-else class="text-xs text-muted-foreground">Tidak ada rincian mata uji untuk attempt ini.</p>

                <Button variant="outline" size="sm" class="w-full" :disabled="downloadingAttemptId === a.id" @click="downloadAttemptPdf(a)">
                  <Loader2 v-if="downloadingAttemptId === a.id" class="w-3.5 h-3.5 animate-spin" /><FileDown v-else class="w-3.5 h-3.5" />
                  {{ downloadingAttemptId === a.id ? 'Membuat PDF...' : 'Unduh PDF Hasil TO ini' }}
                </Button>
              </template>
            </div>
          </div>
        </div>
      </div>
    </Dialog>
  </div>
</template>
