<script setup lang="ts">
// Tab "Rasionalisasi SNBP" — 6 sub-tab lengkap: Dashboard, Daftar Siswa, Rasionalisasi,
// Kaka Kelas, Eligible, Pos. Minimum. (Rekap SNBT punya komponen dedicated sendiri,
// SchoolRekapSnbt.vue, karena strukturnya beda: monitoring pasca-ujian per target nyata,
// bukan pick-student-lihat-peluang.) Semua data nyata:
//  - Dashboard: agregat live dari SchoolRationalizationService::dashboard (target siswa +
//    rapor asli, mesin compute_chance yang sama dengan tab siswa).
//  - Daftar Siswa: reuse mesin RationalizationService::targets_with_chance_for per siswa
//    (ownership diverifikasi backend via ensure_owns_student).
//  - Rasionalisasi: ranking penuh (SchoolRationalizationService::full_ranking) + panel
//    detail breakdown peluang per target (kontribusi nilai rapor / rasio persaingan /
//    prestasi — 3 komponen real yang menjumlah ke chance_percent, lihat ChanceResponse)
//    + cetak laporan (window.print() sungguhan). Radar chart "pencapaian vs minimum per
//    mapel" dari referensi sengaja tidak direplikasi — tidak ada data minimum real per
//    mapel per program studi di katalog, hanya pg_snbp (satu angka agregat).
//  - Kaka Kelas & Eligible: input nyata dari PIC sekolah sendiri (tidak ada dataset yang
//    bisa menurunkannya otomatis) — CRUD asli, bukan mock array seperti referensi.
//  - Pos. Minimum: reuse katalog ptn_programs nyata yang sama dengan pencarian program di
//    tab siswa, tidak perlu backend baru.
import {
  Search, Loader2, AlertCircle, BarChart2, Users, GraduationCap, Award, Building2,
  Plus, Trash2, TrendingUp, Sparkles, Calendar, Printer, ListOrdered, Pencil, Eye, Download, ChevronRight, ChevronLeft, History,
  Paperclip, X,
} from 'lucide-vue-next'
import type {
  TargetWithChanceItem, SnbpDashboardItem, AlumniBenchmarkItem, SchoolEligibilityItem,
  PtnProgramItem, AlumniPayload, AlumniRaporEntryPayload, SnbpRankingEntryItem, SnbpRosterRowItem, SnbpStatusValue,
  RaporScoreItem, AuditLogEntryItem, InstitutionItem, RaporBulkImportEntryPayload, RaporBulkImportResult,
} from '~/types'
// Dipakai HANYA untuk render grafik radar off-screen ke PNG (embed di laporan PDF) —
// registerables Chart.js sudah didaftarkan global oleh plugins/chartjs.client.ts.
import { Chart } from 'chart.js'
import { exportRationalizationReportToPdf, exportRankingListToPdf } from '~/utils/rationalizationReportExport'
import { RAPOR_SUBJECTS, raporSubjectRoot } from '~/utils/raporSubjects'
import { parseRaporImportWorkbook, downloadRaporImportTemplate, type ParsedRaporRow } from '~/utils/raporImportXlsx'

const { rationalizationService, institutionService } = useApi()
const { user } = useAuth()
const toast = useToast()

const TIER_LABEL: Record<string, string> = { aman: 'Aman', moderat: 'Moderat', ketat: 'Ketat' }
const TIER_VARIANT: Record<string, 'success' | 'warning' | 'destructive'> = {
  aman: 'success', moderat: 'warning', ketat: 'destructive',
}

// ─────────────────────────────────────────────────────────────────────────────────
// Daftar Siswa — roster administratif (No/Nama/Kelas/Konsultan/Pilihan 1/Pilihan 2/
// Status Penerimaan/Aktif/Aksi), sesuai layout referensi. Pilihan 1/2 SELALU
// diturunkan live dari target PTN nyata siswa (SchoolRationalizationService::
// snbp_roster) — hanya konsultan/status/aktif yang benar-benar diinput PIC di sini.
// Edit nilai rapor/prestasi/target sendiri dipindah ke tab "Rasionalisasi" (tombol
// "+ Tambah Data"), sesuai alur referensi.
// ─────────────────────────────────────────────────────────────────────────────────
const STATUS_LABEL: Record<SnbpStatusValue, string> = {
  belum: 'Belum', 'diterima-snbp': 'Diterima SNBP', 'diterima-snbt': 'Diterima SNBT', tidak: 'Tidak Diterima',
}
const STATUS_VARIANT: Record<SnbpStatusValue, 'success' | 'warning' | 'destructive' | 'outline'> = {
  belum: 'outline', 'diterima-snbp': 'success', 'diterima-snbt': 'success', tidak: 'destructive',
}

const roster = ref<SnbpRosterRowItem[]>([])
const loadingRoster = ref(true)
const rosterLoaded = ref(false)
const rosterError = ref<string | null>(null)
const rosterSearch = ref('')
const rosterYear = ref<number | ''>('')

async function loadRoster() {
  loadingRoster.value = true
  rosterError.value = null
  try {
    roster.value = await rationalizationService.schoolSnbpRoster()
    rosterLoaded.value = true
  } catch (e: any) {
    rosterError.value = e?.message || 'Gagal memuat daftar siswa.'
    toast.error('Gagal memuat daftar siswa', rosterError.value)
  } finally {
    loadingRoster.value = false
  }
}
onMounted(loadRoster)

const rosterYears = computed(() => Array.from(new Set(roster.value.map((r) => r.year))).sort((a, b) => b - a))

const filteredRoster = computed(() => {
  let list = roster.value
  if (rosterYear.value !== '') list = list.filter((r) => r.year === rosterYear.value)
  if (rosterSearch.value.trim()) {
    const q = rosterSearch.value.toLowerCase()
    list = list.filter((r) => r.student_name.toLowerCase().includes(q) || (r.kelas || '').toLowerCase().includes(q))
  }
  return list
})

function exportRosterCsv() {
  const header = ['No', 'Nama', 'Kelas', 'Konsultan', 'Pilihan 1', 'Pilihan 2', 'Status Penerimaan', 'Aktif']
  const rows = filteredRoster.value.map((r, i) => [
    i + 1, r.student_name, r.kelas || '-', r.konsultan || '-', r.pilihan_1 || '-', r.pilihan_2 || '-',
    STATUS_LABEL[r.status], r.aktif ? 'Aktif' : 'Nonaktif',
  ])
  const csv = [header, ...rows].map((row) => row.map((c) => `"${String(c).replace(/"/g, '""')}"`).join(',')).join('\n')
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `daftar-siswa-snbp-${Date.now()}.csv`
  a.click()
  URL.revokeObjectURL(url)
}

// ─── Import nilai rapor massal (banyak siswa sekaligus dari 1 file Excel) ──────────────
// Dicocokkan via NIS di backend (SchoolRationalizationService::bulk_import_rapor), hanya
// terhadap siswa sekolah ini sendiri. Partial success: baris yang gagal (NIS tak dikenal,
// nilai di luar rentang) dilaporkan balik satu-satu tanpa menggagalkan baris lain yang valid.
const raporBulkInputRef = ref<HTMLInputElement | null>(null)
const raporBulkOpen = ref(false)
const raporBulkLoading = ref(false)
const raporBulkSubmitting = ref(false)
const raporBulkRows = ref<ParsedRaporRow[]>([])
const raporBulkResult = ref<RaporBulkImportResult | null>(null)

function openRaporBulkImportPicker() {
  raporBulkResult.value = null
  raporBulkInputRef.value?.click()
}
async function onRaporBulkFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  raporBulkLoading.value = true
  try {
    const result = await parseRaporImportWorkbook(file)
    if (!result.hasNisColumn) {
      toast.error('Kolom "nis" tidak ditemukan', 'Untuk import banyak siswa sekaligus, file wajib punya kolom NIS untuk mencocokkan tiap baris ke siswa yang tepat.')
      return
    }
    raporBulkRows.value = result.rows
    raporBulkOpen.value = true
  } catch (err: any) {
    toast.error('Gagal membaca file', err?.message)
  } finally {
    raporBulkLoading.value = false
    input.value = ''
  }
}
// Beda dari mode 1-siswa: baris dengan NIS kosong ditolak di sisi frontend (tidak ada
// gunanya dikirim — backend pasti akan menolaknya juga karena tidak bisa dicocokkan).
const raporBulkValidRows = computed(() =>
  raporBulkRows.value.filter((r) => !r.rowError && r.entries.length > 0 && r.nis.trim() !== ''),
)
const raporBulkNisMissingRows = computed(() => raporBulkRows.value.filter((r) => !r.rowError && r.entries.length > 0 && r.nis.trim() === ''))
const raporBulkErrorRows = computed(() => raporBulkRows.value.filter((r) => r.rowError))
const raporBulkTotalEntries = computed(() => raporBulkValidRows.value.reduce((sum, r) => sum + r.entries.length, 0))

async function confirmRaporBulkImport() {
  const entries: RaporBulkImportEntryPayload[] = []
  for (const row of raporBulkValidRows.value) {
    for (const e of row.entries) {
      entries.push({ nis: row.nis, semester: row.semester as number, subject: e.subject, score: e.score, is_minat: false })
    }
  }
  if (entries.length === 0) return
  raporBulkSubmitting.value = true
  try {
    const result = await rationalizationService.bulkImportRapor(entries)
    raporBulkResult.value = result
    if (result.saved_count > 0) toast.success(`${result.saved_count} nilai rapor berhasil diimpor`)
    if (result.errors.length > 0) toast.error(`${result.errors.length} baris gagal diimpor`, 'Lihat detail di preview.')
    await loadRoster()
  } catch (e: any) {
    toast.error('Gagal mengimpor nilai rapor', e?.message)
  } finally {
    raporBulkSubmitting.value = false
  }
}
function closeRaporBulkDialog() {
  raporBulkOpen.value = false
  raporBulkRows.value = []
  raporBulkResult.value = null
}

// Edit ringkas: konsultan / status penerimaan / aktif (input administratif PIC, bukan
// data akademik — itu diedit lewat wizard "Rasionalisasi").
const showRosterForm = ref(false)
const rosterFormStudent = ref<SnbpRosterRowItem | null>(null)
const rosterFormYear = ref(new Date().getFullYear())
const rosterFormKonsultan = ref('')
const rosterFormStatus = ref<SnbpStatusValue>('belum')
const rosterFormAktif = ref(true)
const savingRoster = ref(false)

function openRosterForm(row: SnbpRosterRowItem) {
  rosterFormStudent.value = row
  rosterFormYear.value = row.year
  rosterFormKonsultan.value = row.konsultan
  rosterFormStatus.value = row.status
  rosterFormAktif.value = row.aktif
  showRosterForm.value = true
}
async function saveRosterForm() {
  if (!rosterFormStudent.value) return
  savingRoster.value = true
  try {
    await rationalizationService.upsertSnbpParticipation(rosterFormStudent.value.student_id, {
      year: rosterFormYear.value, konsultan: rosterFormKonsultan.value, status: rosterFormStatus.value, aktif: rosterFormAktif.value,
    })
    toast.success('Data siswa disimpan')
    showRosterForm.value = false
    loadRoster()
  } catch (e: any) {
    toast.error('Gagal menyimpan data siswa', e?.message)
  } finally {
    savingRoster.value = false
  }
}
async function toggleAktif(row: SnbpRosterRowItem) {
  try {
    await rationalizationService.upsertSnbpParticipation(row.student_id, {
      year: row.year, konsultan: row.konsultan, status: row.status, aktif: !row.aktif,
    })
    row.aktif = !row.aktif
  } catch (e: any) {
    toast.error('Gagal mengubah status aktif', e?.message)
  }
}

// "View" — buka wizard verifikasi data akademik (rapor/prestasi/target) siswa yang
// sama, dipakai bersama oleh tombol "+ Tambah Data" di tab Rasionalisasi di bawah.
const showWizardEditor = ref(false)
const wizardStudentId = ref('')
const wizardStudentName = computed(() => roster.value.find((r) => r.student_id === wizardStudentId.value)?.student_name || '')
function openWizardEditor(studentId: string) {
  wizardStudentId.value = studentId
  showWizardEditor.value = true
}
function onWizardChanged() {
  loadRoster()
  if (rankingLoaded.value) loadRanking()
}

// Picker siswa untuk "+ Tambah Data" — pilih siswa dari roster, lalu buka wizard yang sama.
const showWizardPicker = ref(false)
const wizardPickerSearch = ref('')
const wizardCandidates = computed(() => {
  if (!wizardPickerSearch.value.trim()) return roster.value
  const q = wizardPickerSearch.value.toLowerCase()
  return roster.value.filter((r) => r.student_name.toLowerCase().includes(q))
})
function openWizardPicker() {
  wizardPickerSearch.value = ''
  showWizardPicker.value = true
  if (!rosterLoaded.value && !loadingRoster.value) loadRoster()
}
function pickWizardStudent(studentId: string) {
  showWizardPicker.value = false
  openWizardEditor(studentId)
}

// ─────────────────────────────────────────────────────────────────────────────────
// Rasionalisasi SNBP (track === 'snbp') — 5 sub-tab.
// ─────────────────────────────────────────────────────────────────────────────────
type Section = 'dashboard' | 'siswa' | 'rasionalisasi' | 'alumni' | 'eligible' | 'posmin'
const SECTIONS: { key: Section; label: string; icon: any }[] = [
  { key: 'dashboard', label: 'Dashboard', icon: BarChart2 },
  { key: 'siswa', label: 'Daftar Siswa', icon: Users },
  { key: 'rasionalisasi', label: 'Rasionalisasi', icon: ListOrdered },
  { key: 'alumni', label: 'Kaka Kelas', icon: GraduationCap },
  { key: 'eligible', label: 'Eligible', icon: Award },
  { key: 'posmin', label: 'Pos. Minimum', icon: Building2 },
]
const section = ref<Section>('dashboard')

// ─────────────────────────────────────────────────────────────────────────────────
// Rasionalisasi — ranking penuh sebagai card list (Pilihan 1 & 2 tampil langsung,
// sesuai layout referensi) + slide-over detail. Pilihan 1/2 diambil dari target SNBP
// nyata siswa (priority utama/cadangan pertama, urut sort_order) via
// RationalizationService::studentTargets — sama persis dengan mesin compute_chance
// yang dipakai siswa sendiri, hanya di-fetch per-siswa untuk mengisi kartu ranking.
// "Skor Bobot Rasionalisasi" & "Data Detail" di panel detail HANYA memakai 3 komponen
// real (nilai rapor / rasio persaingan / index prestasi) dan field katalog PTN asli
// (pg_snbp/pg_snbt/daya_tampung/peminat/rumpun) — referensi punya 6 "index" (Index
// SNBP/SNBT/Universitas/Jurusan/Akreditasi) dan radar chart "pencapaian vs minimum per
// mapel" yang sengaja TIDAK direplikasi karena datanya fiktif (tidak ada bobot
// per-kategori atau nilai minimum per mata pelajaran per prodi di sistem ini).
// ─────────────────────────────────────────────────────────────────────────────────
type RankingPilihan = {
  target_id: string
  /** Dipakai untuk exclude alternatif yang sudah jadi target siswa — sama seperti
   * `snbpTargets.value.map((t) => t.ptn_program_id)` di RasionalisasiSnbpEditor.vue. */
  ptn_program_id: string
  nama_ptn: string
  nama_prodi: string
  rumpun: string
  mapel_syarat: string
  pg_snbp: number
  pg_snbt: number
  daya_tampung_snbp: number
  peminat_snbp: number
  chance_percent: number
  tier: string
  competition_adjustment: number
  prestasi_contribution: number
  nilai_contribution: number
  /** `false` = belum ada angka daya tampung/peminat/PG resmi untuk prodi ini — tampilkan
   * "Data belum lengkap", bukan persentase peluang (lihat `PtnProgramItem.has_official_stats`). */
  has_official_stats: boolean
}
type RankingCard = {
  student_id: string
  student_name: string
  kelas: string | null
  konsultan: string
  status: SnbpStatusValue
  year: number
  rapor_avg: number | null
  prestasi_index: number
  target_count: number
  best_chance_percent: number
  tier: string
  pilihan_1: RankingPilihan | null
  pilihan_2: RankingPilihan | null
  /** Real, student-declared "Jurusan yang Diminati" (dari roster, read-only di sisi
   * sekolah) — dipakai untuk kategori alternatif "Sesuai Minat Jurusan" di dialog detail. */
  minat_jurusan: string | null
}

function toPilihan(t: TargetWithChanceItem): RankingPilihan {
  return {
    target_id: t.id, ptn_program_id: t.ptn_program_id, nama_ptn: t.program.nama_ptn, nama_prodi: t.program.nama_prodi,
    rumpun: t.program.rumpun, mapel_syarat: t.program.mapel_syarat, pg_snbp: t.program.pg_snbp, pg_snbt: t.program.pg_snbt,
    daya_tampung_snbp: t.program.daya_tampung_snbp, peminat_snbp: t.program.peminat_snbp,
    chance_percent: t.chance.chance_percent, tier: t.chance.tier,
    competition_adjustment: t.chance.competition_adjustment, prestasi_contribution: t.chance.prestasi_contribution,
    nilai_contribution: Math.round((t.chance.chance_percent - 50 - t.chance.competition_adjustment - t.chance.prestasi_contribution) * 10) / 10,
    has_official_stats: t.program.has_official_stats,
  }
}

const ranking = ref<SnbpRankingEntryItem[]>([])
const rankingCards = ref<RankingCard[]>([])
const loadingRanking = ref(false)
const rankingLoaded = ref(false)
const rankingError = ref<string | null>(null)

async function loadRanking() {
  loadingRanking.value = true
  rankingError.value = null
  try {
    ranking.value = await rationalizationService.schoolFullRanking()
    if (!rosterLoaded.value) await loadRoster()
    const rosterMap = new Map(roster.value.map((r) => [r.student_id, r]))
    const cards: RankingCard[] = []
    for (const r of ranking.value) {
      const rr = rosterMap.get(r.student_id)
      let pilihan1: RankingPilihan | null = null
      let pilihan2: RankingPilihan | null = null
      try {
        const targets = (await rationalizationService.studentTargets(r.student_id))
          .filter((t) => t.track === 'snbp')
          .sort((a, b) => a.sort_order - b.sort_order)
        const utama = targets.find((t) => t.priority === 'utama')
        const cadangan = targets.find((t) => t.priority === 'cadangan')
        if (utama) pilihan1 = toPilihan(utama)
        if (cadangan) pilihan2 = toPilihan(cadangan)
      } catch {
        // baris tetap tampil tanpa breakdown pilihan kalau fetch per-siswa ini gagal
      }
      cards.push({
        student_id: r.student_id, student_name: r.student_name,
        kelas: rr?.kelas ?? null, konsultan: rr?.konsultan ?? '', status: rr?.status ?? 'belum',
        year: rr?.year ?? new Date().getFullYear(),
        rapor_avg: r.rapor_avg, prestasi_index: r.prestasi_index, target_count: r.target_count,
        best_chance_percent: r.best_chance_percent, tier: r.tier,
        pilihan_1: pilihan1, pilihan_2: pilihan2,
        minat_jurusan: rr?.minat_jurusan ?? null,
      })
    }
    rankingCards.value = cards
    rankingLoaded.value = true
  } catch (e: any) {
    rankingError.value = e?.message || 'Gagal memuat ranking.'
    toast.error('Gagal memuat ranking', rankingError.value)
  } finally {
    loadingRanking.value = false
  }
}
const RANK_MEDAL = ['bg-amber-400 text-white', 'bg-slate-300 text-white', 'bg-amber-700 text-white']

const rankingYear = ref<number | ''>('')
const rankingUniv = ref('')
const rankingYears = computed(() => Array.from(new Set(rankingCards.value.map((c) => c.year))).sort((a, b) => b - a))
const rankingUnivs = computed(() => {
  const set = new Set<string>()
  rankingCards.value.forEach((c) => { if (c.pilihan_1) set.add(c.pilihan_1.nama_ptn); if (c.pilihan_2) set.add(c.pilihan_2.nama_ptn) })
  return Array.from(set).sort()
})
const filteredRankingCards = computed(() => {
  let list = rankingCards.value
  if (rankingYear.value !== '') list = list.filter((c) => c.year === rankingYear.value)
  if (rankingUniv.value) list = list.filter((c) => c.pilihan_1?.nama_ptn === rankingUniv.value || c.pilihan_2?.nama_ptn === rankingUniv.value)
  return list
})

// Slide-over detail: tab antar Pilihan 1/2, breakdown real, + rekap nilai rapor asli siswa.
const detailId = ref<string>('')
const detailTab = ref<'p1' | 'p2'>('p1')
const detailCard = computed(() => rankingCards.value.find((c) => c.student_id === detailId.value) || null)
const detailPilihan = computed(() => (detailTab.value === 'p1' ? detailCard.value?.pilihan_1 : detailCard.value?.pilihan_2) || null)
const detailRapor = ref<RaporScoreItem[]>([])
const loadingDetailRapor = ref(false)
async function openDetail(studentId: string) {
  detailId.value = studentId
  const card = rankingCards.value.find((c) => c.student_id === studentId)
  detailTab.value = card && !card.pilihan_1 && card.pilihan_2 ? 'p2' : 'p1'
  detailRapor.value = []
  loadingDetailRapor.value = true
  try {
    detailRapor.value = await rationalizationService.studentRapor(studentId)
  } catch (e: any) {
    toast.error('Gagal memuat nilai rapor', e?.message)
  } finally {
    loadingDetailRapor.value = false
  }
  loadDetailAuditLog(studentId)
  loadDetailAlternatifRumpun()
  loadDetailAlternatifMinat()
}
// Rumpun anchor berubah saat tab Pilihan 1/2 diganti — reload alternatif rumpun supaya
// tetap mengikuti pilihan yang sedang aktif di layar. Minat jurusan JUGA di-reload di sini
// sekarang: sejak minat_jurusan bisa fallback ke nama_prodi pilihan yang sedang aktif
// (lihat `detailMinatEffective` di bawah), search term-nya bisa berubah saat tab diganti
// juga (kalau siswa belum mengisi minat_jurusan eksplisit) — reload tanpa syarat di sini
// lebih aman daripada mencoba membedakan kasusnya.
watch(detailTab, () => {
  if (detailId.value) { loadDetailAlternatifRumpun(); loadDetailAlternatifMinat() }
})

// Profil institusi (logo/alamat/website/akreditasi/status/tahun berdiri) — data riset
// eksternal terpisah dari ptn_programs, dimuat on-demand tiap kali pilihan aktif berubah
// (buka detail baru atau ganti tab Pilihan 1/2) dan di-cache per nama_ptn. `null` di cache
// berarti "sudah dicek, memang belum ada data riset", bukan "belum pernah dicek".
const institutionCache = ref<Record<string, InstitutionItem | null>>({})
function institutionFor(namaPtn: string): InstitutionItem | null {
  return institutionCache.value[namaPtn] ?? null
}
async function loadInstitution(namaPtn: string) {
  if (namaPtn in institutionCache.value) return
  try {
    institutionCache.value[namaPtn] = await institutionService.lookup(namaPtn)
  } catch {
    institutionCache.value[namaPtn] = null
  }
}
watch(detailPilihan, (p) => {
  if (p?.nama_ptn) loadInstitution(p.nama_ptn)
}, { immediate: true })

// ─── Alternatif Sesuai Rumpun / Sesuai Minat Jurusan — komposisi IDENTIK dengan
// loadResumeAlternatif di RasionalisasiSnbpEditor.vue (listPrograms + previewStudentChance),
// reuse murni tanpa mesin pencarian/preview kedua. Rumpun anchor = pilihan yang sedang
// aktif di tab (`detailPilihan`); minat jurusan = data real siswa dari roster (read-only di
// sisi sekolah) KALAU diisi, kalau tidak fallback ke `detailPilihan.nama_prodi` — anchor
// yang SAMA PERSIS dengan rumpun — karena siswa yang sudah punya pilihan di tab itu sudah
// menyatakan minatnya lewat pilihan tsb. Fallback ini selalu nilai nyata (nama_prodi milik
// pilihan yang sedang aktif), tidak pernah dikarang. ──────────────────────────────────
const detailAlternatifRumpun = ref<{ program: PtnProgramItem; chance_percent: number; tier: string }[]>([])
const loadingDetailAlternatifRumpun = ref(false)
function detailExcludedProgramIds(): Set<string> {
  return new Set(
    [detailCard.value?.pilihan_1?.ptn_program_id, detailCard.value?.pilihan_2?.ptn_program_id]
      .filter((x): x is string => !!x),
  )
}
/** Inti pencarian alternatif (listPrograms + previewStudentChance per kandidat) — diekstrak
 * jadi fungsi murni supaya bisa dipakai ulang oleh printStudentReport() untuk MENGHITUNG
 * alternatif Pilihan 1 & 2 SEKALIGUS (dialog on-screen hanya menghitung untuk tab yang aktif),
 * tanpa menyalin ulang logika/parameter listPrograms yang sama. Perilaku loader on-screen
 * (loadDetailAlternatifRumpun/Minat di bawah) TIDAK berubah, hanya jadi tipis wrapper. */
async function fetchAlternatifPrograms(
  studentId: string,
  searchParams: { rumpun: string } | { search: string },
  excludeIds: Set<string>,
): Promise<{ program: PtnProgramItem; chance_percent: number; tier: string }[]> {
  try {
    const res = await rationalizationService.listPrograms({ ...searchParams, page_size: 8 })
    const candidates = res.items.filter((p) => !excludeIds.has(p.id)).slice(0, 3)
    const withChance = await Promise.all(candidates.map(async (p) => {
      try {
        const preview = await rationalizationService.previewStudentChance(studentId, p.id, 'snbp')
        return { program: p, chance_percent: preview.chance_percent, tier: preview.tier }
      } catch {
        return null
      }
    }))
    return withChance.filter((x): x is { program: PtnProgramItem; chance_percent: number; tier: string } => x !== null)
  } catch {
    return []
  }
}
async function loadDetailAlternatifRumpun() {
  const studentId = detailId.value
  const anchor = detailPilihan.value
  if (!studentId || !anchor) {
    detailAlternatifRumpun.value = []
    return
  }
  loadingDetailAlternatifRumpun.value = true
  try {
    detailAlternatifRumpun.value = await fetchAlternatifPrograms(studentId, { rumpun: anchor.rumpun }, detailExcludedProgramIds())
  } finally {
    loadingDetailAlternatifRumpun.value = false
  }
}

/** Search term EFEKTIF untuk "Alternatif Sesuai Minat Jurusan": pakai `minat_jurusan`
 * manual siswa (dari roster) kalau sudah diisi, kalau tidak fallback ke `nama_prodi` milik
 * `detailPilihan` (anchor yang sama dengan rumpun) — lihat komentar di atas `watch(detailTab...)`. */
const detailMinatIsFallback = computed(() => !(detailCard.value?.minat_jurusan || '').trim() && !!detailPilihan.value)
const detailMinatEffective = computed(() => (detailCard.value?.minat_jurusan || '').trim() || detailPilihan.value?.nama_prodi || '')

const detailAlternatifMinat = ref<{ program: PtnProgramItem; chance_percent: number; tier: string }[]>([])
const loadingDetailAlternatifMinat = ref(false)
async function loadDetailAlternatifMinat() {
  const studentId = detailId.value
  const minat = detailMinatEffective.value.trim()
  if (!studentId || !minat) {
    detailAlternatifMinat.value = []
    return
  }
  loadingDetailAlternatifMinat.value = true
  try {
    detailAlternatifMinat.value = await fetchAlternatifPrograms(studentId, { search: minat }, detailExcludedProgramIds())
  } finally {
    loadingDetailAlternatifMinat.value = false
  }
}
function closeDetail() {
  detailId.value = ''
  showDetailAuditLog.value = false
}

// Riwayat Perubahan — accountability trail untuk aksi "edit atas nama siswa" (rapor/
// prestasi/target/tracking) yang dilakukan PIC sekolah, bukan siswa sendiri. Dimuat
// bersamaan dengan detail (sekali per buka panel), tapi ditampilkan collapsed by default
// supaya tidak menambah panjang panel untuk siswa yang belum pernah diedit siapa pun.
const detailAuditLog = ref<AuditLogEntryItem[]>([])
const loadingDetailAuditLog = ref(false)
const showDetailAuditLog = ref(false)
async function loadDetailAuditLog(studentId: string) {
  loadingDetailAuditLog.value = true
  try {
    detailAuditLog.value = await rationalizationService.studentAuditLog(studentId)
  } catch (e: any) {
    detailAuditLog.value = []
  } finally {
    loadingDetailAuditLog.value = false
  }
}
function formatAuditDate(iso: string) {
  return new Date(iso).toLocaleString('id-ID', { day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' })
}
/** Rata-rata nilai rapor asli siswa per mata pelajaran (across semester) — dipakai di
 * panel detail sebagai pengganti kolom "Min. Prodi" referensi (yang butuh data minimum
 * per mata pelajaran per prodi yang tidak ada di katalog kita). */
const raporBySubject = computed(() => {
  const bySubject = new Map<string, number[]>()
  for (const r of detailRapor.value) {
    if (!bySubject.has(r.subject)) bySubject.set(r.subject, [])
    bySubject.get(r.subject)!.push(r.score)
  }
  return Array.from(bySubject.entries()).map(([subject, scores]) => ({
    subject, avg: Math.round((scores.reduce((a, b) => a + b, 0) / scores.length) * 10) / 10,
  }))
})

// Radar "Pencapaian vs Referensi" untuk pilihan yang sedang dibuka di panel detail.
// Sumbu = gabungan mapel syarat program (katalog) + mapel yang benar-benar sudah
// diisi nilai rapornya oleh siswa (real) — katalog `mapel_syarat` sering cuma berisi
// 1-2 mapel, jadi kalau cuma mengandalkan itu grafiknya nyaris selalu kosong.
/** Diekstrak jadi fungsi murni (parameter mapel_syarat bebas, bukan cuma detailPilihan aktif)
 * supaya printStudentReport() bisa menghitung radar untuk Pilihan 1 & 2 SEKALIGUS saat cetak
 * (dialog on-screen cuma menghitung untuk tab yang aktif). Perilaku computed di bawah untuk
 * layar TIDAK berubah, hanya jadi tipis wrapper di atas fungsi ini. */
function radarSubjectsFor(mapelSyarat: string | undefined): string[] {
  const syarat = (mapelSyarat || '').split(',').map((s) => s.trim()).filter(Boolean)
  const dariRapor = Array.from(new Set(detailRapor.value.map((r) => r.subject)))
  return Array.from(new Set([...syarat, ...dariRapor])).slice(0, 8)
}
const detailRadarSubjects = computed(() => radarSubjectsFor(detailPilihan.value?.mapel_syarat))
// Root-keyword matcher (root list) sekarang ditarik dari satu sumber bersama
// (utils/raporSubjects.ts) — identik dengan RasionalisasiSnbpEditor.vue, supaya tidak
// pernah divergen kalau daftar mapel berubah lagi ke depannya.
const subjectRoot = raporSubjectRoot
function radarPencapaianFor(subjects: string[]): (number | null)[] {
  return subjects.map((subj) => {
    const root = subjectRoot(subj)
    const matches = root
      ? detailRapor.value.filter((r) => subjectRoot(r.subject) === root)
      : detailRapor.value.filter((r) => r.subject.toLowerCase().includes(subj.toLowerCase()))
    if (matches.length === 0) return null
    return Math.round((matches.reduce((sum, r) => sum + r.score, 0) / matches.length) * 10) / 10
  })
}
const detailRadarPencapaian = computed(() => radarPencapaianFor(detailRadarSubjects.value))

// ─── Render grafik radar "Pencapaian vs Referensi PG" ke gambar PNG (off-screen, via
// Chart.js langsung — bukan komponen Vue PencapaianRadarChart.vue karena itu perlu ter-mount
// ke DOM) — dipakai HANYA untuk menempelkan grafik nyata ke laporan PDF (bukan tampilan
// layar). Warna & style disamakan persis dengan PencapaianRadarChart.vue supaya konsisten
// dengan yang dilihat user di dialog. `registerables` Chart.js sudah didaftarkan global oleh
// plugins/chartjs.client.ts sebelum komponen mana pun mount, jadi aman dipanggil di sini.
async function renderRadarChartImage(subjects: string[], pencapaian: (number | null)[], referensi: number): Promise<string | null> {
  if (subjects.length < 3) return null
  try {
    const canvas = document.createElement('canvas')
    canvas.width = 500
    canvas.height = 500
    const chart = new Chart(canvas, {
      type: 'radar',
      data: {
        labels: subjects,
        datasets: [
          {
            label: 'Pencapaian (rapor asli)',
            data: subjects.map((_, i) => pencapaian[i] ?? 0),
            backgroundColor: 'rgba(99, 102, 241, 0.25)',
            borderColor: '#6366f1',
            borderWidth: 2,
            pointBackgroundColor: '#6366f1',
            pointBorderColor: '#fff',
            pointRadius: 3.5,
          },
          {
            label: `Referensi PG ${referensi}`,
            data: subjects.map(() => referensi),
            backgroundColor: 'rgba(245, 158, 11, 0.12)',
            borderColor: '#f59e0b',
            borderWidth: 1.5,
            borderDash: [4, 3],
            pointRadius: 0,
          },
        ],
      },
      options: {
        responsive: false,
        animation: false,
        scales: {
          r: {
            min: 0,
            max: 100,
            ticks: { display: false },
            grid: { color: '#e2e8f0' },
            angleLines: { color: '#e2e8f0' },
            pointLabels: { font: { size: 13 }, color: '#475569' },
          },
        },
        plugins: {
          legend: { display: true, position: 'bottom', labels: { font: { size: 12 }, color: '#334155' } },
        },
      },
    })
    const dataUrl = chart.toBase64Image('image/png', 1)
    chart.destroy()
    return dataUrl
  } catch {
    // Kalau render gagal (mis. environment tanpa canvas) — laporan tetap dibuat tanpa grafik.
    return null
  }
}

// Jembatan dari panel "lihat hasil" (read-only) ke wizard editable — supaya data yang
// keliru dari hasil "Tambah Data" bisa direvisi tanpa harus lewat tabel Daftar Siswa.
function editFromDetail() {
  if (!detailId.value) return
  const studentId = detailId.value
  closeDetail()
  openWizardEditor(studentId)
}

// Baris satu-baris "kelas · konsultan" di header dialog detail — hanya gabungkan bagian
// yang benar-benar terisi (dulu ada bug: "-" tetap tampil di depan konsultan walau kelas
// kosong, jadi kelihatan seperti "- · Ibu Ratna Dewi").
const detailHeaderLine = computed(() => {
  const parts = [detailCard.value?.kelas, detailCard.value?.konsultan].filter(
    (p): p is string => !!p && p.trim() !== '',
  )
  return parts.length ? parts.join(' · ') : '-'
})

// ─── Gauge semicircle "Peluang Kompetisi" — trigonometri arc SVG standar (konvensi sudut:
// 0°=atas/12-jam, 90°=kanan/3-jam, 180°=bawah/6-jam, 270°=kiri/9-jam, searah jarum jam).
// Gauge dimulai dari kiri (270°) dan menyapu ke kanan (450° = 90°+360°) melewati atas,
// proporsional terhadap chance_percent. Murni presentasi — tidak menyentuh chance_percent
// itu sendiri, hanya menggambarnya. ──
function polarToCartesian(cx: number, cy: number, r: number, angleDeg: number) {
  const rad = ((angleDeg - 90) * Math.PI) / 180
  return { x: cx + r * Math.cos(rad), y: cy + r * Math.sin(rad) }
}
function describeArc(cx: number, cy: number, r: number, startAngle: number, endAngle: number) {
  const start = polarToCartesian(cx, cy, r, endAngle)
  const end = polarToCartesian(cx, cy, r, startAngle)
  const largeArcFlag = endAngle - startAngle <= 180 ? '0' : '1'
  return `M ${start.x} ${start.y} A ${r} ${r} 0 ${largeArcFlag} 0 ${end.x} ${end.y}`
}
const GAUGE_CX = 70
const GAUGE_CY = 68
const GAUGE_R = 56
const gaugeBackgroundPath = computed(() => describeArc(GAUGE_CX, GAUGE_CY, GAUGE_R, 270, 450))
const gaugeForegroundPath = computed(() => {
  const pct = Math.max(0, Math.min(100, detailPilihan.value?.chance_percent ?? 0))
  return describeArc(GAUGE_CX, GAUGE_CY, GAUGE_R, 270, 270 + pct * 1.8)
})

// ─── Bar diverging (bipolar) "Skor Bobot Rasionalisasi" — SAMA PERSIS rumus skala
// magnitude-ke-persen yang sudah ada sebelumnya (single-direction bar), hanya diubah
// jadi tumbuh dari tengah ke kiri/kanan sesuai tanda nilainya. Tidak mengubah data. ──
function clampPct(p: number) { return Math.max(0, Math.min(100, p)) }
const nilaiBarNegPct = computed(() => {
  const v = detailPilihan.value?.nilai_contribution ?? 0
  return v < 0 ? clampPct(Math.min(Math.abs(v), 50) * 2) : 0
})
const nilaiBarPosPct = computed(() => {
  const v = detailPilihan.value?.nilai_contribution ?? 0
  return v >= 0 ? clampPct(Math.min(Math.abs(v), 50) * 2) : 0
})
const competitionBarNegPct = computed(() => {
  const v = detailPilihan.value?.competition_adjustment ?? 0
  return v < 0 ? clampPct(Math.min(Math.abs(v), 20) * 5) : 0
})
const competitionBarPosPct = computed(() => {
  const v = detailPilihan.value?.competition_adjustment ?? 0
  return v >= 0 ? clampPct(Math.min(Math.abs(v), 20) * 5) : 0
})
const prestasiBarNegPct = computed(() => {
  const v = detailPilihan.value?.prestasi_contribution ?? 0
  return v < 0 ? clampPct((Math.abs(v) / 10) * 100) : 0
})
const prestasiBarPosPct = computed(() => {
  const v = detailPilihan.value?.prestasi_contribution ?? 0
  return v >= 0 ? clampPct((v / 10) * 100) : 0
})

// Cetak laporan ringkasan tabel ranking — dokumen ini untuk pihak sekolah/rekap internal
// (BUKAN per-siswa/ortu, itu printStudentReport() di bawah), jadi bukan window.print() (yang
// mencetak seluruh UI aplikasi) dan mencakup SEMUA siswa yang sedang tampil di
// filteredRankingCards (menghormati filter tahun/universitas yang sedang aktif di layar).
function printRankingReport() {
  exportRankingListToPdf({
    school_name: user.value?.school_name,
    year_filter: rankingYear.value,
    univ_filter: rankingUniv.value,
    rows: filteredRankingCards.value.map((c) => ({
      student_name: c.student_name,
      kelas: c.kelas,
      pilihan_1: c.pilihan_1 ? { nama_ptn: c.pilihan_1.nama_ptn, chance_percent: c.pilihan_1.chance_percent } : null,
      pilihan_2: c.pilihan_2 ? { nama_ptn: c.pilihan_2.nama_ptn, chance_percent: c.pilihan_2.chance_percent } : null,
      best_chance_percent: c.best_chance_percent,
      tier: c.tier,
      tier_label: TIER_LABEL[c.tier] ?? c.tier,
    })),
  })
}

// Cetak laporan resmi PDF untuk siswa yang sedang dibuka di panel detail — dokumen ini
// diserahkan ke ORANG TUA, jadi bukan window.print() (yang mencetak seluruh UI aplikasi).
// Menyertakan Pilihan 1 DAN 2 sekaligus (kalau ada dua-duanya) supaya orang tua langsung
// lihat kedua opsi dalam satu dokumen, bukan cuma tab yang lagi aktif di layar.
//
// Sejak update visual (grafik radar + Resume Peluang + Alternatif Rumpun/Minat per pilihan),
// fungsi ini jadi ASYNC: untuk setiap pilihan yang ada, alternatif rumpun/minat dihitung ulang
// via fetchAlternatifPrograms (mesin nyata yang sama dengan dialog on-screen, listPrograms +
// previewStudentChance) — TIDAK cukup reuse detailAlternatifRumpun/Minat karena ref itu cuma
// berisi hasil untuk tab yang lagi aktif, sedangkan laporan cetak butuh keduanya sekaligus.
const printingReport = ref(false)
async function printStudentReport() {
  const card = detailCard.value
  if (!card || printingReport.value) return
  printingReport.value = true
  try {
    const pilihanList: { key: 'p1' | 'p2'; label: string; data: RankingPilihan | null }[] = [
      { key: 'p1', label: 'Pilihan 1', data: card.pilihan_1 },
      { key: 'p2', label: 'Pilihan 2', data: card.pilihan_2 },
    ]
    const present = pilihanList.filter(
      (p): p is { key: 'p1' | 'p2'; label: string; data: RankingPilihan } => !!p.data,
    )
    const excludeIds = detailExcludedProgramIds()
    const explicitMinat = (card.minat_jurusan || '').trim()

    const pilihan = await Promise.all(present.map(async (p) => {
      const data = p.data
      const subjects = radarSubjectsFor(data.mapel_syarat)
      const pencapaian = radarPencapaianFor(subjects)
      const minatLabel = explicitMinat || data.nama_prodi
      const [altRumpun, altMinat, radarImage] = await Promise.all([
        fetchAlternatifPrograms(card.student_id, { rumpun: data.rumpun }, excludeIds),
        fetchAlternatifPrograms(card.student_id, { search: minatLabel }, excludeIds),
        renderRadarChartImage(subjects, pencapaian, data.pg_snbp),
      ])
      return {
        label: p.label,
        nama_ptn: data.nama_ptn,
        nama_prodi: data.nama_prodi,
        chance_percent: data.chance_percent,
        tier: data.tier,
        tier_label: TIER_LABEL[data.tier] ?? data.tier,
        nilai_contribution: data.nilai_contribution,
        competition_adjustment: data.competition_adjustment,
        prestasi_contribution: data.prestasi_contribution,
        pg_snbp: data.pg_snbp,
        pg_snbt: data.pg_snbt,
        daya_tampung_snbp: data.daya_tampung_snbp,
        peminat_snbp: data.peminat_snbp,
        rumpun: data.rumpun,
        alternatif_rumpun: altRumpun.map((a) => ({
          nama_ptn: a.program.nama_ptn, nama_prodi: a.program.nama_prodi,
          chance_percent: a.chance_percent, tier: a.tier, tier_label: TIER_LABEL[a.tier] ?? a.tier,
        })),
        minat_label: minatLabel,
        minat_is_fallback: !explicitMinat,
        alternatif_minat: altMinat.map((a) => ({
          nama_ptn: a.program.nama_ptn, nama_prodi: a.program.nama_prodi,
          chance_percent: a.chance_percent, tier: a.tier, tier_label: TIER_LABEL[a.tier] ?? a.tier,
        })),
        radar_chart_image: radarImage,
      }
    }))

    exportRationalizationReportToPdf({
      school_name: user.value?.school_name,
      student_name: card.student_name,
      kelas: card.kelas,
      konsultan: card.konsultan,
      pilihan,
      rapor: raporBySubject.value,
    })
  } catch (e: any) {
    toast.error('Gagal membuat laporan', e?.message)
  } finally {
    printingReport.value = false
  }
}

// ── Dashboard ──
const dashboard = ref<SnbpDashboardItem | null>(null)
const loadingDashboard = ref(true)
const dashboardError = ref<string | null>(null)
async function loadDashboard() {
  loadingDashboard.value = true
  dashboardError.value = null
  try {
    dashboard.value = await rationalizationService.schoolDashboard()
  } catch (e: any) {
    dashboardError.value = e?.message || 'Gagal memuat dashboard.'
    toast.error('Gagal memuat dashboard', dashboardError.value)
  } finally {
    loadingDashboard.value = false
  }
}
// Data chart untuk Sebaran Universitas Tujuan (SNBP)
const uniChartLabels = computed(() => (dashboard.value?.distribusi_universitas || []).map((d) => d.nama_ptn))
const uniChartValues = computed(() => (dashboard.value?.distribusi_universitas || []).map((d) => d.count))

// ── Kaka Kelas (Alumni) ──
const alumni = ref<AlumniBenchmarkItem[]>([])
const loadingAlumni = ref(false)
const alumniLoaded = ref(false)
async function loadAlumni() {
  loadingAlumni.value = true
  try {
    alumni.value = await rationalizationService.listAlumni()
    alumniLoaded.value = true
  } catch (e: any) {
    toast.error('Gagal memuat data alumni', e?.message)
  } finally {
    loadingAlumni.value = false
  }
}
const alumniCurrentYear = new Date().getFullYear()
const alumniYearOptions = Array.from({ length: 16 }, (_, i) => alumniCurrentYear - i)

const showAlumniForm = ref(false)
const alumniName = ref('')
const alumniYear = ref(alumniCurrentYear)
const alumniPtn = ref('')
const alumniProdi = ref('')
const alumniScore = ref(80)
const savingAlumni = ref(false)

// Universitas/Prodi: searchable dari katalog PTN nyata (bukan free text) — pola debounced
// search yang sama dengan RasionalisasiSnbpEditor.vue untuk target siswa, karena katalog
// PTN sudah 4600+ baris (terlalu banyak untuk combobox client-side biasa).
const alumniPickedProgram = ref<PtnProgramItem | null>(null)
const alumniSearchQuery = ref('')
const alumniSearchResults = ref<PtnProgramItem[]>([])
const alumniSearching = ref(false)
let alumniSearchTimer: ReturnType<typeof setTimeout> | null = null
watch(alumniSearchQuery, () => {
  if (alumniSearchTimer) clearTimeout(alumniSearchTimer)
  if (alumniSearchQuery.value.trim().length >= 3) {
    alumniSearching.value = true
    alumniSearchResults.value = []
  }
  alumniSearchTimer = setTimeout(runAlumniSearch, 350)
})
async function runAlumniSearch() {
  if (alumniSearchQuery.value.trim().length < 3) {
    alumniSearchResults.value = []
    alumniSearching.value = false
    return
  }
  alumniSearching.value = true
  try {
    const res = await rationalizationService.listPrograms({ search: alumniSearchQuery.value.trim(), page_size: 15 })
    alumniSearchResults.value = res.items
  } finally {
    alumniSearching.value = false
  }
}
function pickAlumniProgram(p: PtnProgramItem) {
  alumniPickedProgram.value = p
  alumniPtn.value = p.nama_ptn
  alumniProdi.value = p.nama_prodi
  alumniSearchQuery.value = ''
  alumniSearchResults.value = []
}
function clearAlumniProgram() {
  alumniPickedProgram.value = null
  alumniPtn.value = ''
  alumniProdi.value = ''
}

// "Detail Rapor" opsional — snapshot 1x per mapel (BUKAN per-semester seperti rapor siswa
// aktif, karena alumni sudah lulus) — lihat AlumniRaporScore di backend. Dipakai untuk
// fitur radar chart siswa-vs-alumni ke depannya. Diisi lewat grid per-mapel (reuse
// RAPOR_SUBJECTS, sama daftar mapel Kurikulum Merdeka dengan form nilai rapor siswa) atau
// import Excel (skope: 1 alumni saja, isi banyak mapel sekaligus — sesuai pilihan PIC).
const alumniHasRaporDetail = ref(false)
const alumniRaporValues = reactive<Record<string, number | null>>({})
function resetAlumniRaporValues() {
  for (const s of RAPOR_SUBJECTS) alumniRaporValues[s] = null
}
const alumniRaporFilledCount = computed(() =>
  RAPOR_SUBJECTS.filter((s) => alumniRaporValues[s] !== null && alumniRaporValues[s] !== undefined).length,
)

function openAlumniForm() {
  alumniName.value = ''; alumniYear.value = alumniCurrentYear
  alumniPtn.value = ''; alumniProdi.value = ''; alumniScore.value = 80
  alumniPickedProgram.value = null; alumniSearchQuery.value = ''; alumniSearchResults.value = []
  alumniHasRaporDetail.value = false
  resetAlumniRaporValues()
  showAlumniForm.value = true
}
async function saveAlumni() {
  if (!alumniName.value.trim() || !alumniPtn.value.trim() || !alumniProdi.value.trim()) {
    return toast.error('Nama, universitas, dan program studi wajib diisi')
  }
  savingAlumni.value = true
  try {
    const payload: AlumniPayload = {
      alumni_name: alumniName.value, graduation_year: alumniYear.value, track: 'snbp',
      nama_ptn: alumniPtn.value, nama_prodi: alumniProdi.value, benchmark_score: alumniScore.value,
    }
    const created = await rationalizationService.addAlumni(payload)
    // Alumni sudah tersimpan di titik ini — kegagalan upsert detail rapor di bawah ini
    // TIDAK boleh membuat user mengira seluruh penyimpanan gagal (alumni-nya tetap ada),
    // jadi ditangani dengan pesan & follow-up terpisah, bukan ikut try/catch yang sama.
    toast.success('Data alumni ditambahkan')
    showAlumniForm.value = false
    if (alumniHasRaporDetail.value) {
      const entries: AlumniRaporEntryPayload[] = RAPOR_SUBJECTS
        .filter((s) => alumniRaporValues[s] !== null && alumniRaporValues[s] !== undefined)
        .map((s) => ({ subject: s, score: alumniRaporValues[s] as number }))
      if (entries.length > 0) {
        try {
          await rationalizationService.upsertAlumniRapor(created.id, entries)
        } catch (e: any) {
          toast.error('Alumni tersimpan, tapi detail rapor gagal disimpan', e?.message)
        }
      }
    }
    loadAlumni()
  } catch (e: any) {
    toast.error('Gagal menyimpan data alumni', e?.message)
  } finally {
    savingAlumni.value = false
  }
}
const confirmDeleteAlumniId = ref<string | null>(null)
async function deleteAlumni() {
  if (!confirmDeleteAlumniId.value) return
  try {
    await rationalizationService.deleteAlumni(confirmDeleteAlumniId.value)
    toast.success('Data alumni dihapus')
    loadAlumni()
  } catch (e: any) {
    toast.error('Gagal menghapus data alumni', e?.message)
  } finally {
    confirmDeleteAlumniId.value = null
  }
}

// Import Excel Detail Rapor alumni — skope 1 alumni saja (isi banyak mapel sekaligus),
// tanpa kolom semester (snapshot, bukan riwayat). Baris pertama yang valid di file dipakai
// untuk mengisi grid nilai di form ini; pengiriman ke server terjadi bersamaan dengan
// saveAlumni di atas (alumni baru belum punya id sebelum disimpan).
const alumniRaporImportInputRef = ref<HTMLInputElement | null>(null)
const alumniRaporImportLoading = ref(false)
function openAlumniRaporImportPicker() {
  alumniRaporImportInputRef.value?.click()
}
async function onAlumniRaporImportFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  alumniRaporImportLoading.value = true
  try {
    const result = await parseRaporImportWorkbook(file, { requireSemester: false })
    const row = result.rows.find((r) => !r.rowError && r.entries.length > 0)
    if (!row) {
      toast.error('Tidak ada baris nilai yang valid di file ini')
      return
    }
    for (const entry of row.entries) alumniRaporValues[entry.subject] = entry.score
    alumniHasRaporDetail.value = true
    if (row.cellWarnings.length > 0) toast.error(`${row.cellWarnings.length} nilai diabaikan`, row.cellWarnings.join('; '))
    toast.success(`${row.entries.length} nilai mapel dimuat dari file`)
  } catch (err: any) {
    toast.error('Gagal membaca file', err?.message)
  } finally {
    alumniRaporImportLoading.value = false
    input.value = ''
  }
}

// ── Eligible ──
const eligibility = ref<SchoolEligibilityItem[]>([])
const loadingEligibility = ref(false)
const eligibilityLoaded = ref(false)
async function loadEligibility() {
  loadingEligibility.value = true
  try {
    eligibility.value = await rationalizationService.listEligibility()
    eligibilityLoaded.value = true
  } catch (e: any) {
    toast.error('Gagal memuat data eligible', e?.message)
  } finally {
    loadingEligibility.value = false
  }
}
const showEligibleForm = ref(false)
const eligibleYear = ref(new Date().getFullYear())
const eligibleCount = ref(0)
const savingEligible = ref(false)
function openEligibleForm() {
  eligibleYear.value = new Date().getFullYear()
  eligibleCount.value = 0
  showEligibleForm.value = true
}
// Sandingkan kuota eligible (input manual PIC) dengan jumlah siswa yang BENAR-BENAR
// terdaftar aktif SNBP tahun itu (dari roster nyata "Daftar Siswa", live dari backend) —
// sebelumnya dua angka ini tidak pernah dibandingkan sama sekali.
function eligibilityUsage(e: SchoolEligibilityItem): { label: string; variant: 'success' | 'warning' | 'outline' } {
  if (e.eligible_count === 0) return { label: 'Kuota belum diisi', variant: 'outline' }
  const selisih = e.terdaftar_aktif_count - e.eligible_count
  if (selisih > 0) return { label: `Melebihi kuota ${selisih}`, variant: 'warning' }
  if (selisih === 0) return { label: 'Pas dengan kuota', variant: 'success' }
  return { label: `Sisa slot ${Math.abs(selisih)}`, variant: 'outline' }
}
async function saveEligible() {
  savingEligible.value = true
  try {
    await rationalizationService.upsertEligibility({ year: eligibleYear.value, eligible_count: eligibleCount.value })
    toast.success('Data eligible disimpan')
    showEligibleForm.value = false
    loadEligibility()
  } catch (e: any) {
    toast.error('Gagal menyimpan data eligible', e?.message)
  } finally {
    savingEligible.value = false
  }
}

// ── Pos. Minimum (reuse katalog PTN nyata) ──
// Sebelumnya cuma page_size:30 tanpa kontrol halaman, jadi kalau kotak pencarian kosong
// user cuma melihat 30 baris pertama dari 4631+ prodi di katalog (bukan batasan backend —
// backend sudah dukung page/page_size + total count, lihat PtnCatalogManager.vue yang
// sudah pakai pola ini untuk CRUD Admin). Sekarang dipasang pagination sungguhan yang sama.
const POSMIN_PAGE_SIZE = 30
const posminSearch = ref('')
const posminRumpun = ref('')
const posminResults = ref<PtnProgramItem[]>([])
const posminRumpunList = ref<string[]>([])
const posminTotal = ref(0)
const posminPage = ref(1)
const loadingPosmin = ref(false)
const posminLoaded = ref(false)
const posminTotalPages = computed(() => Math.max(1, Math.ceil(posminTotal.value / POSMIN_PAGE_SIZE)))
const posminRangeStart = computed(() => (posminTotal.value === 0 ? 0 : (posminPage.value - 1) * POSMIN_PAGE_SIZE + 1))
const posminRangeEnd = computed(() => Math.min(posminPage.value * POSMIN_PAGE_SIZE, posminTotal.value))
async function loadPosminRumpun() {
  try {
    posminRumpunList.value = await rationalizationService.distinctRumpun()
  } catch {
    // non-critical filter list
  }
}
async function searchPosmin() {
  loadingPosmin.value = true
  try {
    const res = await rationalizationService.listPrograms({
      search: posminSearch.value || undefined, rumpun: posminRumpun.value || undefined,
      page: posminPage.value, page_size: POSMIN_PAGE_SIZE,
    })
    posminResults.value = res.items
    posminTotal.value = res.total
    posminLoaded.value = true
  } catch (e: any) {
    toast.error('Gagal memuat Pos. Minimum', e?.message)
  } finally {
    loadingPosmin.value = false
  }
}
function goToPosminPage(p: number) {
  if (p < 1 || p > posminTotalPages.value || p === posminPage.value) return
  posminPage.value = p
}
let posminDebounce: ReturnType<typeof setTimeout>
watch([posminSearch, posminRumpun], () => {
  clearTimeout(posminDebounce)
  posminDebounce = setTimeout(() => {
    posminPage.value = 1
    searchPosmin()
  }, 250)
})
watch(posminPage, searchPosmin)

watch(section, (s) => {
  if (s === 'rasionalisasi' && !rankingLoaded.value) loadRanking()
  if (s === 'alumni' && !alumniLoaded.value) loadAlumni()
  if (s === 'eligible' && !eligibilityLoaded.value) loadEligibility()
  if (s === 'posmin' && !posminLoaded.value) { loadPosminRumpun(); searchPosmin() }
})

onMounted(loadDashboard)
</script>

<template>
  <!-- Rasionalisasi SNBP: 6 sub-tab -->
  <div class="space-y-4">
    <div class="flex items-center gap-1.5 overflow-x-auto pb-1">
      <button
        v-for="s in SECTIONS" :key="s.key"
        class="flex items-center gap-1.5 px-3.5 py-2 rounded-full text-sm font-semibold whitespace-nowrap transition-colors shrink-0"
        :class="section === s.key ? 'bg-indigo-600 text-white' : 'bg-white border text-slate-600 hover:bg-slate-50'"
        @click="section = s.key"
      >
        <component :is="s.icon" class="w-4 h-4" />{{ s.label }}
      </button>
    </div>

    <!-- Dashboard -->
    <div v-if="section === 'dashboard'" class="space-y-4">
      <div v-if="loadingDashboard" class="py-16 text-center text-muted-foreground"><Loader2 class="w-6 h-6 animate-spin mx-auto mb-2" /> Memuat dashboard...</div>
      <Card v-else-if="dashboardError" class="p-8 text-center border-red-200 bg-red-50">
        <AlertCircle class="w-8 h-8 mx-auto mb-2 text-red-500" />
        <p class="text-sm text-red-700 mb-3">{{ dashboardError }}</p>
        <Button variant="outline" size="sm" @click="loadDashboard">Coba Lagi</Button>
      </Card>
      <template v-else-if="dashboard">
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Siswa Aktif</p><p class="text-2xl font-black text-slate-900">{{ dashboard.total_siswa_aktif }}</p></Card>
          <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Terasionalisasi</p><p class="text-2xl font-black text-indigo-600">{{ dashboard.terasionalisasi }}</p></Card>
          <Card class="p-4"><p class="text-xs text-muted-foreground mb-1">Rata-rata Peluang SNBP</p><p class="text-2xl font-black text-emerald-600">{{ dashboard.avg_chance_snbp != null ? Math.round(dashboard.avg_chance_snbp) + '%' : '-' }}</p></Card>
          <Card class="p-4">
            <p class="text-xs text-muted-foreground mb-1">Eligible {{ dashboard.eligible_terbaru?.year ?? '' }}</p>
            <p class="text-2xl font-black text-slate-900">{{ dashboard.eligible_terbaru?.eligible_count ?? '-' }}</p>
          </Card>
        </div>

        <Card class="p-5">
          <h4 class="font-bold text-slate-900 mb-1 flex items-center gap-2"><Building2 class="w-4 h-4 text-indigo-600" /> Sebaran Universitas Tujuan (SNBP)</h4>
          <p class="text-xs text-muted-foreground mb-4">Universitas paling banyak dijadikan target siswa jalur SNBP.</p>
          <DonutChart :labels="uniChartLabels" :values="uniChartValues" unit-label=" target" :height="200" empty-label="Belum ada siswa yang menambahkan target SNBP." />
        </Card>

        <Card class="p-5">
          <h4 class="font-bold text-slate-900 mb-4 flex items-center gap-2"><TrendingUp class="w-4 h-4 text-indigo-600" /> Top Peluang</h4>
          <div v-if="dashboard.top_peluang.length === 0" class="py-8 text-center text-sm text-muted-foreground">Belum ada estimasi peluang untuk dirangking.</div>
          <div v-else class="space-y-2">
            <div v-for="(t, i) in dashboard.top_peluang" :key="t.student_id" class="flex items-center justify-between p-3 rounded-xl border">
              <div class="flex items-center gap-3">
                <div class="w-7 h-7 rounded-full bg-indigo-50 border border-indigo-200 flex items-center justify-center text-xs font-bold text-indigo-600">{{ i + 1 }}</div>
                <span class="text-sm font-semibold text-slate-900">{{ t.student_name }}</span>
              </div>
              <div class="flex items-center gap-2">
                <Badge :variant="TIER_VARIANT[t.tier]">{{ TIER_LABEL[t.tier] }}</Badge>
                <span class="text-sm font-black text-slate-900">{{ t.best_chance_percent }}%</span>
              </div>
            </div>
          </div>
        </Card>
      </template>
    </div>

    <!-- Daftar Siswa: roster administratif (Konsultan/Pilihan 1-2/Status/Aktif) -->
    <div v-else-if="section === 'siswa'" class="space-y-4">
      <div class="flex flex-col sm:flex-row gap-2 sm:items-center sm:justify-between">
        <div class="flex flex-col sm:flex-row gap-2 flex-1">
          <div class="relative flex-1 sm:max-w-xs">
            <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
            <Input v-model="rosterSearch" placeholder="Cari nama atau kelas..." class="pl-9" />
          </div>
          <select v-model="rosterYear" class="px-3 py-2 border rounded-lg text-sm bg-white">
            <option value="">Semua Tahun</option>
            <option v-for="y in rosterYears" :key="y" :value="y">{{ y }}</option>
          </select>
        </div>
        <div class="flex items-center gap-1.5 shrink-0">
          <input ref="raporBulkInputRef" type="file" accept=".xlsx,.xls" class="hidden" @change="onRaporBulkFileChange" />
          <button
            type="button" class="text-[11px] font-semibold text-slate-500 hover:underline"
            @click="downloadRaporImportTemplate('template_import_nilai_rapor_massal.xlsx', { includeNis: true })"
          >Unduh Template</button>
          <Button variant="outline" size="sm" :disabled="raporBulkLoading" @click="openRaporBulkImportPicker">
            <Loader2 v-if="raporBulkLoading" class="w-4 h-4 animate-spin" /><Paperclip v-else class="w-4 h-4" /> Import Nilai Rapor
          </Button>
          <Button variant="outline" size="sm" @click="exportRosterCsv"><Download class="w-4 h-4" />Export</Button>
        </div>
      </div>

      <Card class="overflow-hidden">
        <div v-if="loadingRoster" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
        <Card v-else-if="rosterError" class="p-8 text-center border-red-200 bg-red-50 m-4">
          <AlertCircle class="w-8 h-8 mx-auto mb-2 text-red-500" />
          <p class="text-sm text-red-700 mb-3">{{ rosterError }}</p>
          <Button variant="outline" size="sm" @click="loadRoster">Coba Lagi</Button>
        </Card>
        <div v-else-if="filteredRoster.length === 0" class="p-10 text-center text-sm text-muted-foreground">Belum ada siswa yang cocok.</div>
        <div v-else class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead class="bg-slate-50 text-xs uppercase text-slate-500">
              <tr>
                <th class="text-left px-4 py-3">No</th>
                <th class="text-left px-4 py-3">Nama</th>
                <th class="text-left px-4 py-3">Kelas</th>
                <th class="text-left px-4 py-3">Konsultan</th>
                <th class="text-left px-4 py-3">Pilihan 1</th>
                <th class="text-left px-4 py-3">Pilihan 2</th>
                <th class="text-left px-4 py-3">Status Penerimaan</th>
                <th class="text-left px-4 py-3">Aktif</th>
                <th class="text-right px-4 py-3">Aksi</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-50">
              <tr v-for="(r, i) in filteredRoster" :key="r.student_id" class="hover:bg-slate-50">
                <td class="px-4 py-3 text-slate-500">{{ i + 1 }}</td>
                <td class="px-4 py-3 font-semibold text-slate-900">{{ r.student_name }}</td>
                <td class="px-4 py-3 text-slate-600">{{ r.kelas || '-' }}</td>
                <td class="px-4 py-3 text-slate-600">{{ r.konsultan || '-' }}</td>
                <td class="px-4 py-3 text-slate-600">{{ r.pilihan_1 || '-' }}</td>
                <td class="px-4 py-3 text-slate-600">{{ r.pilihan_2 || '-' }}</td>
                <td class="px-4 py-3"><Badge :variant="STATUS_VARIANT[r.status]">{{ STATUS_LABEL[r.status] }}</Badge></td>
                <td class="px-4 py-3">
                  <button
                    class="w-9 h-5 rounded-full transition-colors relative"
                    :class="r.aktif ? 'bg-emerald-500' : 'bg-slate-200'"
                    :title="r.aktif ? 'Aktif' : 'Nonaktif'"
                    @click="toggleAktif(r)"
                  >
                    <span class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all" :class="r.aktif ? 'left-4' : 'left-0.5'" />
                  </button>
                </td>
                <td class="px-4 py-3">
                  <div class="flex items-center justify-end gap-1">
                    <button class="p-1.5 rounded-lg hover:bg-indigo-50 text-slate-400 hover:text-indigo-600" title="Lihat & verifikasi data akademik" @click="openWizardEditor(r.student_id)"><Eye class="w-4 h-4" /></button>
                    <button class="p-1.5 rounded-lg hover:bg-indigo-50 text-slate-400 hover:text-indigo-600" title="Edit konsultan/status" @click="openRosterForm(r)"><Pencil class="w-4 h-4" /></button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </Card>
    </div>

    <!-- Rasionalisasi: ranking penuh (card list Pilihan 1/2) + slide-over detail -->
    <div v-else-if="section === 'rasionalisasi'" class="space-y-4">
      <div>
        <h4 class="font-bold text-slate-900 flex items-center gap-2"><ListOrdered class="w-4 h-4 text-indigo-600" /> Ranking Rasionalisasi SNBP</h4>
        <p class="text-xs text-muted-foreground">Seluruh siswa dengan target SNBP, diurutkan dari estimasi peluang terbaik.</p>
      </div>

      <div class="flex flex-wrap items-center justify-between gap-2">
        <div class="flex items-center gap-2">
          <select v-model="rankingYear" class="px-3 py-2 border rounded-lg text-sm bg-white">
            <option value="">Semua Tahun</option>
            <option v-for="y in rankingYears" :key="y" :value="y">{{ y }}</option>
          </select>
          <select v-model="rankingUniv" class="px-3 py-2 border rounded-lg text-sm bg-white">
            <option value="">Semua Universitas</option>
            <option v-for="u in rankingUnivs" :key="u" :value="u">{{ u }}</option>
          </select>
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <Button variant="outline" size="sm" :disabled="loadingRanking" @click="loadRanking"><ListOrdered class="w-4 h-4" />Generate Rank</Button>
          <Button variant="gradient" size="sm" @click="openWizardPicker"><Plus class="w-4 h-4" />Tambah Data</Button>
          <Button variant="outline" size="sm" @click="printRankingReport"><Printer class="w-4 h-4" />Cetak Laporan</Button>
        </div>
      </div>

      <div v-if="loadingRanking" class="p-10 text-center text-sm text-muted-foreground">Memuat ranking...</div>
      <Card v-else-if="rankingError" class="p-8 text-center border-red-200 bg-red-50">
        <AlertCircle class="w-8 h-8 mx-auto mb-2 text-red-500" />
        <p class="text-sm text-red-700 mb-3">{{ rankingError }}</p>
        <Button variant="outline" size="sm" @click="loadRanking">Coba Lagi</Button>
      </Card>
      <div v-else-if="filteredRankingCards.length === 0" class="p-10 text-center text-sm text-muted-foreground">Belum ada siswa dengan target SNBP.</div>
      <div v-else class="space-y-3">
        <Card v-for="(c, i) in filteredRankingCards" :key="c.student_id" class="p-4">
          <div class="flex items-start gap-3 mb-3 flex-wrap">
            <div class="shrink-0 text-center">
              <div class="w-9 h-9 rounded-full flex items-center justify-center text-sm font-bold" :class="i < 3 ? RANK_MEDAL[i] : 'bg-slate-100 text-slate-500'">{{ i + 1 }}</div>
              <p class="text-[9px] text-muted-foreground mt-0.5 uppercase tracking-wide">Rank</p>
            </div>
            <div class="flex-1 min-w-[10rem]">
              <p class="font-bold text-slate-900">{{ c.student_name }}</p>
              <p class="text-xs text-muted-foreground">{{ c.kelas || '-' }} · Tahun {{ c.year }}<template v-if="c.konsultan"> · {{ c.konsultan }}</template></p>
            </div>
            <Badge :variant="STATUS_VARIANT[c.status]" class="shrink-0 self-center">{{ STATUS_LABEL[c.status] }}</Badge>
            <Button variant="outline" size="sm" class="shrink-0" @click="openDetail(c.student_id)">Detail<ChevronRight class="w-3.5 h-3.5" /></Button>
          </div>
          <div class="grid sm:grid-cols-2 gap-3">
            <div v-if="c.pilihan_1" class="rounded-xl border bg-slate-50 p-3">
              <p class="text-[10px] uppercase tracking-wide text-muted-foreground mb-1">Pilihan 1</p>
              <div class="flex items-center justify-between gap-2">
                <div class="min-w-0">
                  <p class="font-bold text-slate-900 truncate">{{ c.pilihan_1.nama_ptn }}</p>
                  <p class="text-xs text-muted-foreground truncate">{{ c.pilihan_1.nama_prodi }}</p>
                </div>
                <div v-if="!c.pilihan_1.has_official_stats" class="text-right shrink-0 max-w-[6rem]">
                  <Badge class="bg-slate-100 text-slate-500 border-0 text-[9px]">Data belum lengkap</Badge>
                </div>
                <div v-else class="text-right shrink-0">
                  <p class="text-lg font-black" :class="c.pilihan_1.tier === 'aman' ? 'text-emerald-600' : c.pilihan_1.tier === 'moderat' ? 'text-amber-600' : 'text-red-600'">{{ c.pilihan_1.chance_percent }}%</p>
                  <Badge :variant="TIER_VARIANT[c.pilihan_1.tier]" class="text-[9px]">{{ TIER_LABEL[c.pilihan_1.tier] }}</Badge>
                </div>
              </div>
            </div>
            <div v-else class="rounded-xl border border-dashed p-3 flex items-center justify-center text-center text-xs text-muted-foreground">Belum ada Pilihan 1</div>

            <div v-if="c.pilihan_2" class="rounded-xl border bg-slate-50 p-3">
              <p class="text-[10px] uppercase tracking-wide text-muted-foreground mb-1">Pilihan 2</p>
              <div class="flex items-center justify-between gap-2">
                <div class="min-w-0">
                  <p class="font-bold text-slate-900 truncate">{{ c.pilihan_2.nama_ptn }}</p>
                  <p class="text-xs text-muted-foreground truncate">{{ c.pilihan_2.nama_prodi }}</p>
                </div>
                <div v-if="!c.pilihan_2.has_official_stats" class="text-right shrink-0 max-w-[6rem]">
                  <Badge class="bg-slate-100 text-slate-500 border-0 text-[9px]">Data belum lengkap</Badge>
                </div>
                <div v-else class="text-right shrink-0">
                  <p class="text-lg font-black" :class="c.pilihan_2.tier === 'aman' ? 'text-emerald-600' : c.pilihan_2.tier === 'moderat' ? 'text-amber-600' : 'text-red-600'">{{ c.pilihan_2.chance_percent }}%</p>
                  <Badge :variant="TIER_VARIANT[c.pilihan_2.tier]" class="text-[9px]">{{ TIER_LABEL[c.pilihan_2.tier] }}</Badge>
                </div>
              </div>
            </div>
            <div v-else class="rounded-xl border border-dashed p-3 flex items-center justify-center text-center text-xs text-muted-foreground">Belum ada Pilihan 2</div>
          </div>
        </Card>
      </div>

      <!-- Dialog detail: wide & centered (dulu slide-over sempit di tepi kanan) -->
      <Dialog
        :model-value="!!detailId"
        :title="detailCard?.student_name || ''"
        max-width="max-w-3xl"
        @update:model-value="(v) => { if (!v) closeDetail() }"
      >
        <div v-if="detailCard" class="space-y-6">
          <p class="text-xs text-muted-foreground -mt-4">{{ detailHeaderLine }}</p>

          <!-- Resume Peluang: perbandingan Pilihan 1 & 2 berdampingan — independen dari tab
               yang lagi aktif di bawah (data sudah ada di detailCard, tidak perlu fetch baru). -->
          <div>
            <p class="font-bold text-slate-900 text-sm mb-3">Resume Peluang — Pilihan 1 vs Pilihan 2</p>
            <div class="grid sm:grid-cols-2 gap-3">
              <div v-if="detailCard.pilihan_1" class="rounded-xl border p-4">
                <p class="text-[10px] uppercase tracking-wide text-muted-foreground mb-1">Pilihan 1</p>
                <p class="font-bold text-slate-900 truncate">{{ detailCard.pilihan_1.nama_ptn }}</p>
                <p class="text-xs text-muted-foreground truncate mb-2">{{ detailCard.pilihan_1.nama_prodi }}</p>
                <div class="flex items-center justify-between">
                  <Badge v-if="!detailCard.pilihan_1.has_official_stats" class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
                  <template v-else>
                    <Badge :variant="TIER_VARIANT[detailCard.pilihan_1.tier]">{{ TIER_LABEL[detailCard.pilihan_1.tier] }}</Badge>
                    <span class="text-xl font-black text-slate-900">{{ detailCard.pilihan_1.chance_percent }}%</span>
                  </template>
                </div>
              </div>
              <div v-else class="rounded-xl border border-dashed p-4 flex items-center justify-center text-center text-xs text-muted-foreground">Belum ada Pilihan 1</div>

              <div v-if="detailCard.pilihan_2" class="rounded-xl border p-4">
                <p class="text-[10px] uppercase tracking-wide text-muted-foreground mb-1">Pilihan 2</p>
                <p class="font-bold text-slate-900 truncate">{{ detailCard.pilihan_2.nama_ptn }}</p>
                <p class="text-xs text-muted-foreground truncate mb-2">{{ detailCard.pilihan_2.nama_prodi }}</p>
                <div class="flex items-center justify-between">
                  <Badge v-if="!detailCard.pilihan_2.has_official_stats" class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
                  <template v-else>
                    <Badge :variant="TIER_VARIANT[detailCard.pilihan_2.tier]">{{ TIER_LABEL[detailCard.pilihan_2.tier] }}</Badge>
                    <span class="text-xl font-black text-slate-900">{{ detailCard.pilihan_2.chance_percent }}%</span>
                  </template>
                </div>
              </div>
              <div v-else class="rounded-xl border border-dashed p-4 flex items-center justify-center text-center text-xs text-muted-foreground">Belum ada Pilihan 2</div>
            </div>
          </div>

          <div class="grid gap-2" :class="detailCard.pilihan_1 && detailCard.pilihan_2 ? 'grid-cols-2' : 'grid-cols-1'">
            <button
              v-if="detailCard.pilihan_1"
              type="button"
              class="rounded-xl border px-4 py-2.5 text-left transition-colors"
              :class="detailTab === 'p1' ? 'bg-indigo-50 border-indigo-300 text-indigo-700' : 'bg-slate-50 border-slate-200 text-slate-500'"
              @click="detailTab = 'p1'"
            >
              <p class="text-[10px] uppercase tracking-wide font-semibold opacity-70">Pilihan 1</p>
              <p class="text-sm font-bold truncate" :title="detailCard.pilihan_1.nama_ptn">{{ detailCard.pilihan_1.nama_ptn }}</p>
            </button>
            <button
              v-if="detailCard.pilihan_2"
              type="button"
              class="rounded-xl border px-4 py-2.5 text-left transition-colors"
              :class="detailTab === 'p2' ? 'bg-indigo-50 border-indigo-300 text-indigo-700' : 'bg-slate-50 border-slate-200 text-slate-500'"
              @click="detailTab = 'p2'"
            >
              <p class="text-[10px] uppercase tracking-wide font-semibold opacity-70">Pilihan 2</p>
              <p class="text-sm font-bold truncate" :title="detailCard.pilihan_2.nama_ptn">{{ detailCard.pilihan_2.nama_ptn }}</p>
            </button>
          </div>

          <div v-if="!detailPilihan" class="py-10 text-center text-sm text-muted-foreground">Belum ada target di pilihan ini.</div>
          <div v-else class="space-y-6">
            <div v-if="!detailPilihan.has_official_stats" class="rounded-2xl border border-slate-200 bg-slate-50 p-5 text-center">
              <Badge class="bg-slate-100 text-slate-500 border-0 mb-1.5">Data belum lengkap</Badge>
              <p class="text-xs text-muted-foreground">Belum ada angka daya tampung/peminat/passing-grade resmi untuk prodi ini — estimasi peluang & skor bobot belum bisa dihitung.</p>
            </div>
            <div v-else class="grid grid-cols-[180px_1fr] gap-5">
              <div class="rounded-2xl p-4 flex flex-col items-center justify-center text-center" :class="detailPilihan.tier === 'aman' ? 'bg-emerald-50' : detailPilihan.tier === 'moderat' ? 'bg-amber-50' : 'bg-red-50'">
                <svg viewBox="0 0 140 78" class="w-full max-w-[150px]">
                  <path :d="gaugeBackgroundPath" fill="none" stroke="#e2e8f0" stroke-width="14" stroke-linecap="round" />
                  <path
                    :d="gaugeForegroundPath" fill="none" stroke="currentColor" stroke-width="14" stroke-linecap="round"
                    :class="detailPilihan.tier === 'aman' ? 'text-emerald-500' : detailPilihan.tier === 'moderat' ? 'text-amber-500' : 'text-red-500'"
                  />
                </svg>
                <p class="text-[10px] uppercase tracking-wide text-muted-foreground -mt-2">Peluang Kompetisi</p>
                <p class="text-3xl font-black" :class="detailPilihan.tier === 'aman' ? 'text-emerald-700' : detailPilihan.tier === 'moderat' ? 'text-amber-700' : 'text-red-700'">{{ detailPilihan.chance_percent }}%</p>
                <Badge :variant="TIER_VARIANT[detailPilihan.tier]" class="mt-1">{{ TIER_LABEL[detailPilihan.tier] }}</Badge>
              </div>

              <div>
                <p class="font-bold text-slate-900 text-sm mb-3">Skor Bobot Rasionalisasi</p>
                <div class="space-y-4 text-xs">
                  <div>
                    <div class="flex items-center justify-between mb-1"><span>Nilai Rapor vs Passing Grade</span><span class="font-semibold">{{ detailPilihan.nilai_contribution > 0 ? '+' : '' }}{{ detailPilihan.nilai_contribution }}</span></div>
                    <div class="flex h-1.5 rounded-full bg-slate-100 overflow-hidden">
                      <div class="w-1/2 flex justify-end"><div class="h-full bg-red-400" :style="{ width: `${nilaiBarNegPct}%` }" /></div>
                      <div class="w-1/2 flex"><div class="h-full bg-indigo-500" :style="{ width: `${nilaiBarPosPct}%` }" /></div>
                    </div>
                  </div>
                  <div>
                    <div class="flex items-center justify-between mb-1"><span>Rasio Persaingan Program</span><span class="font-semibold">{{ detailPilihan.competition_adjustment > 0 ? '+' : '' }}{{ Math.round(detailPilihan.competition_adjustment * 10) / 10 }}</span></div>
                    <div class="flex h-1.5 rounded-full bg-slate-100 overflow-hidden">
                      <div class="w-1/2 flex justify-end"><div class="h-full bg-red-400" :style="{ width: `${competitionBarNegPct}%` }" /></div>
                      <div class="w-1/2 flex"><div class="h-full bg-blue-500" :style="{ width: `${competitionBarPosPct}%` }" /></div>
                    </div>
                  </div>
                  <div>
                    <div class="flex items-center justify-between mb-1"><span>Index Prestasi</span><span class="font-semibold">+{{ Math.round(detailPilihan.prestasi_contribution * 10) / 10 }}</span></div>
                    <div class="flex h-1.5 rounded-full bg-slate-100 overflow-hidden">
                      <div class="w-1/2 flex justify-end"><div class="h-full bg-red-400" :style="{ width: `${prestasiBarNegPct}%` }" /></div>
                      <div class="w-1/2 flex"><div class="h-full bg-amber-500" :style="{ width: `${prestasiBarPosPct}%` }" /></div>
                    </div>
                  </div>
                </div>
                <div class="flex items-center justify-between mt-4 pt-3 border-t text-sm font-bold text-slate-900">
                  <span>Total Index</span><span>{{ detailPilihan.chance_percent }}%</span>
                </div>
              </div>
            </div>

            <div>
              <p class="font-bold text-slate-900 text-sm mb-3">Data Detail — {{ detailPilihan.nama_ptn }} {{ detailPilihan.nama_prodi }}</p>
              <div class="grid grid-cols-3 gap-3 text-xs">
                <div class="rounded-xl border p-3"><p class="text-muted-foreground">Min. Nilai Rapor</p><p class="font-bold text-slate-900">{{ detailPilihan.pg_snbp }}</p></div>
                <div class="rounded-xl border p-3"><p class="text-muted-foreground">Min UTBK (ref. SNBT)</p><p class="font-bold text-slate-900">{{ detailPilihan.pg_snbt }}</p></div>
                <div class="rounded-xl border p-3"><p class="text-muted-foreground">Kuota SNBP</p><p class="font-bold text-slate-900">{{ detailPilihan.daya_tampung_snbp }} kursi</p></div>
                <div class="rounded-xl border p-3"><p class="text-muted-foreground">Peminat SNBP</p><p class="font-bold text-slate-900">{{ detailPilihan.peminat_snbp }} orang</p></div>
                <div class="rounded-xl border p-3"><p class="text-muted-foreground">Persaingan</p><p class="font-bold text-slate-900">1:{{ detailPilihan.daya_tampung_snbp > 0 ? Math.round(detailPilihan.peminat_snbp / detailPilihan.daya_tampung_snbp) : '-' }}</p></div>
                <div class="rounded-xl border p-3"><p class="text-muted-foreground">Rumpun</p><p class="font-bold text-slate-900">{{ detailPilihan.rumpun }}</p></div>
              </div>
            </div>

            <!-- Profil Institusi — hanya tampil kalau data riset tersedia untuk PTN ini;
                 diam-diam tidak ditampilkan (bukan placeholder kosong) kalau belum ada. -->
            <div v-if="institutionFor(detailPilihan.nama_ptn)">
              <p class="font-bold text-slate-900 text-sm mb-3">Profil Institusi</p>
              <div class="flex items-start gap-3 p-3 rounded-xl border">
                <img
                  v-if="institutionFor(detailPilihan.nama_ptn)?.logo_url"
                  :src="institutionFor(detailPilihan.nama_ptn)!.logo_url!"
                  class="w-10 h-10 rounded-lg object-contain border shrink-0 bg-white"
                  alt=""
                />
                <div v-else class="w-10 h-10 rounded-lg bg-slate-100 flex items-center justify-center shrink-0">
                  <Building2 class="w-4 h-4 text-slate-400" />
                </div>
                <div class="min-w-0 flex-1 text-xs">
                  <p class="text-muted-foreground flex flex-wrap gap-x-1.5">
                    <span v-if="institutionFor(detailPilihan.nama_ptn)?.akreditasi">Akreditasi {{ institutionFor(detailPilihan.nama_ptn)?.akreditasi }}</span>
                    <span v-if="institutionFor(detailPilihan.nama_ptn)?.status">· {{ institutionFor(detailPilihan.nama_ptn)?.status }}</span>
                    <span v-if="institutionFor(detailPilihan.nama_ptn)?.tahun_berdiri">· Berdiri {{ institutionFor(detailPilihan.nama_ptn)?.tahun_berdiri }}</span>
                  </p>
                  <p v-if="institutionFor(detailPilihan.nama_ptn)?.alamat" class="text-muted-foreground truncate mt-0.5">{{ institutionFor(detailPilihan.nama_ptn)?.alamat }}</p>
                </div>
                <a
                  v-if="institutionFor(detailPilihan.nama_ptn)?.website"
                  :href="institutionFor(detailPilihan.nama_ptn)!.website!"
                  target="_blank" rel="noopener"
                  class="text-indigo-600 hover:underline shrink-0"
                >
                  Website
                </a>
              </div>
            </div>

            <div>
              <p class="font-bold text-slate-900 text-sm mb-3">Rekap Nilai Rapor Siswa</p>
              <div v-if="loadingDetailRapor" class="text-xs text-muted-foreground text-center py-4">Memuat...</div>
              <div v-else-if="raporBySubject.length === 0" class="text-xs text-muted-foreground text-center py-4">Belum ada nilai rapor.</div>
              <div v-else class="overflow-x-auto">
                <table class="w-full text-xs">
                  <thead class="text-muted-foreground uppercase text-[10px]">
                    <tr><th class="text-left py-2">Mata Pelajaran</th><th class="text-right py-2">Rata-rata</th></tr>
                  </thead>
                  <tbody class="divide-y">
                    <tr v-for="s in raporBySubject" :key="s.subject"><td class="py-2">{{ s.subject }}</td><td class="py-2 text-right font-semibold">{{ s.avg }}</td></tr>
                  </tbody>
                </table>
              </div>
            </div>

            <div v-if="detailRadarSubjects.length">
              <p class="font-bold text-slate-900 text-sm mb-1">Pencapaian vs Referensi PG</p>
              <p class="text-[10px] text-muted-foreground mb-2">Sumbu = mapel syarat prodi ini (katalog nyata). Garis oranye = PG keseluruhan (bukan minimum per mapel).</p>
              <PencapaianRadarChart :subjects="detailRadarSubjects" :pencapaian="detailRadarPencapaian" :referensi="detailPilihan.pg_snbp" :size="460" />
            </div>

            <!-- Alternatif Sesuai Rumpun Program Studi — komposisi identik dengan
                 RasionalisasiSnbpEditor.vue (listPrograms({rumpun}) + previewStudentChance),
                 anchor = pilihan yang sedang aktif di tab (detailPilihan.rumpun). -->
            <div>
              <p class="text-xs font-bold uppercase tracking-wide text-amber-700 bg-amber-50 px-3 py-1.5 rounded-lg mb-2 inline-block">Alternatif Sesuai Rumpun Program Studi</p>
              <div v-if="loadingDetailAlternatifRumpun" class="text-xs text-muted-foreground py-3 text-center">Mencari alternatif...</div>
              <div v-else-if="detailAlternatifRumpun.length === 0" class="text-xs text-muted-foreground py-3 text-center">Tidak ada alternatif lain di rumpun yang sama.</div>
              <div v-else class="overflow-x-auto">
                <table class="w-full text-sm">
                  <tbody class="divide-y">
                    <tr v-for="alt in detailAlternatifRumpun" :key="alt.program.id">
                      <td class="py-2"><div class="font-semibold text-slate-900">{{ alt.program.nama_ptn }}</div><div class="text-xs text-muted-foreground">{{ alt.program.nama_prodi }}</div></td>
                      <td class="py-2">{{ alt.program.pg_snbp }}</td>
                      <td class="py-2">
                        <Badge v-if="!alt.program.has_official_stats" class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
                        <Badge v-else :variant="TIER_VARIANT[alt.tier]">{{ alt.chance_percent }}%</Badge>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>

            <!-- Alternatif Sesuai Minat Jurusan — keyed pada `minat_jurusan` real siswa
                 (dari roster, read-only di sisi sekolah) KALAU diisi; kalau kosong, fallback
                 ke `detailPilihan.nama_prodi` (nyata, anchor yang sama dengan rumpun di
                 atas) karena siswa yang sudah punya pilihan di tab ini sudah menyatakan
                 minatnya lewat pilihan tsb. Empty state jujur hanya kalau BENAR-BENAR tidak
                 ada sinyal (tidak ada minat_jurusan DAN tidak ada pilihan) — jarang terjadi
                 karena kartu ini hanya tampil kalau siswa sudah punya minimal satu pilihan. -->
            <div>
              <p class="text-xs font-bold uppercase tracking-wide text-sky-700 bg-sky-50 px-3 py-1.5 rounded-lg mb-2 inline-block">Alternatif Sesuai Minat Jurusan</p>
              <div v-if="!detailMinatEffective.trim()" class="text-xs text-muted-foreground py-3 text-center">Siswa belum mengisi "Jurusan yang Diminati" dan belum punya pilihan program studi.</div>
              <template v-else>
                <p v-if="detailMinatIsFallback" class="text-[11px] text-sky-700 bg-sky-50/60 border border-sky-100 rounded-lg px-3 py-2 mb-2">
                  Menampilkan alternatif berdasarkan program yang dipilih siswa ({{ detailMinatEffective }}). Siswa belum mengisi "Jurusan yang Diminati" secara eksplisit.
                </p>
                <p v-else class="text-[11px] text-muted-foreground mb-2">Minat: <strong>{{ detailCard.minat_jurusan }}</strong></p>
                <div v-if="loadingDetailAlternatifMinat" class="text-xs text-muted-foreground py-3 text-center">Mencari alternatif...</div>
                <div v-else-if="detailAlternatifMinat.length === 0" class="text-xs text-muted-foreground py-3 text-center">Tidak ada alternatif lain untuk minat jurusan ini.</div>
                <div v-else class="overflow-x-auto">
                  <table class="w-full text-sm">
                    <tbody class="divide-y">
                      <tr v-for="alt in detailAlternatifMinat" :key="alt.program.id">
                        <td class="py-2"><div class="font-semibold text-slate-900">{{ alt.program.nama_ptn }}</div><div class="text-xs text-muted-foreground">{{ alt.program.nama_prodi }}</div></td>
                        <td class="py-2">{{ alt.program.pg_snbp }}</td>
                        <td class="py-2">
                          <Badge v-if="!alt.program.has_official_stats" class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
                          <Badge v-else :variant="TIER_VARIANT[alt.tier]">{{ alt.chance_percent }}%</Badge>
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </template>
            </div>

            <div>
              <button
                type="button" class="flex items-center justify-between w-full text-left"
                @click="showDetailAuditLog = !showDetailAuditLog"
              >
                <p class="font-bold text-slate-900 text-sm flex items-center gap-1.5"><History class="w-3.5 h-3.5" /> Riwayat Perubahan <span v-if="detailAuditLog.length" class="text-xs font-normal text-muted-foreground">({{ detailAuditLog.length }})</span></p>
                <ChevronRight class="w-4 h-4 text-muted-foreground transition-transform" :class="showDetailAuditLog ? 'rotate-90' : ''" />
              </button>
              <p class="text-[10px] text-muted-foreground mt-0.5">Catatan siapa mengubah data akademik siswa ini dan kapan (bukan termasuk yang diisi siswa sendiri).</p>
              <div v-if="showDetailAuditLog" class="mt-2">
                <div v-if="loadingDetailAuditLog" class="text-xs text-muted-foreground text-center py-3">Memuat riwayat...</div>
                <div v-else-if="detailAuditLog.length === 0" class="text-xs text-muted-foreground text-center py-3">Belum ada perubahan yang tercatat.</div>
                <div v-else class="space-y-2 max-h-48 overflow-y-auto pr-1">
                  <div v-for="entry in detailAuditLog" :key="entry.id" class="text-xs p-2.5 rounded-lg border">
                    <div class="flex items-center justify-between gap-2">
                      <span class="font-semibold text-slate-900">{{ entry.actor_name }}</span>
                      <span class="text-[10px] text-muted-foreground shrink-0">{{ formatAuditDate(entry.created_at) }}</span>
                    </div>
                    <p class="text-muted-foreground mt-0.5">{{ entry.summary }}</p>
                  </div>
                </div>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <Button variant="gradient" class="flex-1" @click="editFromDetail"><Pencil class="w-4 h-4" />Edit Data</Button>
              <Button variant="outline" class="flex-1" :disabled="printingReport" @click="printStudentReport">
                <Loader2 v-if="printingReport" class="w-4 h-4 animate-spin" />
                <Printer v-else class="w-4 h-4" />
                {{ printingReport ? 'Menyiapkan Laporan...' : 'Cetak Laporan' }}
              </Button>
            </div>
          </div>
        </div>
      </Dialog>
    </div>

    <!-- Kaka Kelas (Alumni) -->
    <div v-else-if="section === 'alumni'" class="space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div>
          <h4 class="font-bold text-slate-900">Kaka Kelas — Benchmark Alumni</h4>
          <p class="text-xs text-muted-foreground">Data nyata hasil input PIC sekolah, dipakai sebagai acuan pembanding untuk siswa saat ini.</p>
        </div>
        <Button variant="gradient" class="shrink-0" @click="openAlumniForm"><Plus class="w-4 h-4" />Tambah Alumni</Button>
      </div>
      <Card class="overflow-hidden">
        <div v-if="loadingAlumni" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
        <div v-else-if="alumni.length === 0" class="p-10 text-center text-sm text-muted-foreground">Belum ada data alumni. Tambahkan yang pertama di atas.</div>
        <div v-else class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead class="bg-slate-50 text-xs uppercase text-slate-500">
              <tr>
                <th class="text-left px-4 py-3">Nama Alumni</th>
                <th class="text-left px-4 py-3">Lulus</th>
                <th class="text-left px-4 py-3">PTN / Prodi</th>
                <th class="text-left px-4 py-3">Nilai Acuan</th>
                <th class="text-right px-4 py-3">Aksi</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-50">
              <tr v-for="a in alumni" :key="a.id" class="hover:bg-slate-50">
                <td class="px-4 py-3 font-semibold text-slate-900">{{ a.alumni_name }}</td>
                <td class="px-4 py-3 text-slate-600">{{ a.graduation_year }}</td>
                <td class="px-4 py-3">
                  <div class="text-slate-900">{{ a.nama_prodi }}</div>
                  <div class="text-xs text-muted-foreground">{{ a.nama_ptn }}</div>
                </td>
                <td class="px-4 py-3 font-semibold text-slate-900">{{ a.benchmark_score }}</td>
                <td class="px-4 py-3 text-right">
                  <button class="p-1.5 rounded-lg hover:bg-red-50 text-slate-400 hover:text-red-600" title="Hapus" @click="confirmDeleteAlumniId = a.id"><Trash2 class="w-4 h-4" /></button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </Card>
    </div>

    <!-- Eligible -->
    <div v-else-if="section === 'eligible'" class="space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div>
          <h4 class="font-bold text-slate-900">Eligible SNBP per Tahun</h4>
          <p class="text-xs text-muted-foreground">Jumlah siswa yang memenuhi syarat SNBP tahun berjalan — diinput manual oleh PIC sekolah.</p>
        </div>
        <Button variant="gradient" class="shrink-0" @click="openEligibleForm"><Plus class="w-4 h-4" />Tambah/Update Tahun</Button>
      </div>
      <div v-if="loadingEligibility" class="py-10 text-center text-sm text-muted-foreground">Memuat...</div>
      <div v-else-if="eligibility.length === 0" class="py-10 text-center text-sm text-muted-foreground">Belum ada data eligible. Tambahkan tahun pertama di atas.</div>
      <div v-else class="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card v-for="e in eligibility" :key="e.id" class="p-4 text-center space-y-2">
          <p class="text-xs text-muted-foreground flex items-center justify-center gap-1"><Calendar class="w-3.5 h-3.5" />{{ e.year }}</p>
          <p class="text-2xl font-black text-slate-900">{{ e.eligible_count }}</p>
          <p class="text-[10px] text-muted-foreground">siswa eligible</p>
          <div class="pt-2 border-t space-y-1.5">
            <p class="text-xs text-slate-600">{{ e.terdaftar_aktif_count }} siswa terdaftar aktif</p>
            <Badge :variant="eligibilityUsage(e).variant" class="text-[10px]">{{ eligibilityUsage(e).label }}</Badge>
          </div>
        </Card>
      </div>
    </div>

    <!-- Pos. Minimum -->
    <div v-else-if="section === 'posmin'" class="space-y-4">
      <div class="flex flex-col sm:flex-row gap-2">
        <div class="relative flex-1">
          <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
          <Input v-model="posminSearch" placeholder="Cari PTN atau program studi..." class="pl-9" />
        </div>
        <select v-model="posminRumpun" class="px-3 py-2 border rounded-lg text-sm bg-white">
          <option value="">Semua Rumpun</option>
          <option v-for="r in posminRumpunList" :key="r" :value="r">{{ r }}</option>
        </select>
      </div>
      <Card class="overflow-hidden">
        <div v-if="loadingPosmin" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
        <div v-else-if="posminResults.length === 0" class="p-10 text-center text-sm text-muted-foreground">Tidak ada program studi ditemukan.</div>
        <div v-else class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead class="bg-slate-50 text-xs uppercase text-slate-500">
              <tr>
                <th class="text-left px-4 py-3">PTN / Prodi</th>
                <th class="text-left px-4 py-3">Jenjang</th>
                <th class="text-left px-4 py-3">Rumpun</th>
                <th class="text-left px-4 py-3">Min SNBP</th>
                <th class="text-left px-4 py-3">Min UTBK</th>
                <th class="text-left px-4 py-3">Daya Tampung</th>
                <th class="text-left px-4 py-3">Peminat</th>
                <th class="text-left px-4 py-3">Rasio</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-50">
              <tr v-for="p in posminResults" :key="p.id" class="hover:bg-slate-50">
                <td class="px-4 py-3">
                  <div class="font-semibold text-slate-900">{{ p.nama_prodi }}</div>
                  <div class="text-xs text-muted-foreground">{{ p.nama_ptn }} · {{ p.kota }}</div>
                </td>
                <td class="px-4 py-3 text-slate-600">{{ p.jenjang }}</td>
                <td class="px-4 py-3 text-slate-600">{{ p.rumpun }}</td>
                <td class="px-4 py-3 font-semibold text-slate-900">{{ p.pg_snbp }}</td>
                <td class="px-4 py-3 font-semibold text-slate-900">{{ p.pg_snbt }}</td>
                <td class="px-4 py-3 text-slate-600">{{ p.daya_tampung_snbp }}</td>
                <td class="px-4 py-3 text-slate-600">{{ p.peminat_snbp }}</td>
                <td class="px-4 py-3 text-slate-600">1:{{ p.daya_tampung_snbp > 0 ? Math.round(p.peminat_snbp / p.daya_tampung_snbp) : '-' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div v-if="posminTotalPages > 1" class="flex items-center justify-between gap-3 px-4 py-3 border-t bg-slate-50">
          <p class="text-xs text-muted-foreground">Menampilkan {{ posminRangeStart }}–{{ posminRangeEnd }} dari {{ posminTotal }} prodi</p>
          <div class="flex items-center gap-1">
            <button class="p-1.5 rounded-lg border bg-white text-slate-500 hover:text-slate-800 disabled:opacity-40" :disabled="posminPage === 1" @click="goToPosminPage(posminPage - 1)"><ChevronLeft class="w-4 h-4" /></button>
            <span class="text-xs font-semibold text-slate-600 px-2">Halaman {{ posminPage }} / {{ posminTotalPages }}</span>
            <button class="p-1.5 rounded-lg border bg-white text-slate-500 hover:text-slate-800 disabled:opacity-40" :disabled="posminPage === posminTotalPages" @click="goToPosminPage(posminPage + 1)"><ChevronRight class="w-4 h-4" /></button>
          </div>
        </div>
      </Card>
    </div>

    <!-- Dialog: Tambah Alumni -->
    <Dialog v-model="showAlumniForm" title="Tambah Data Alumni" max-width="max-w-lg">
      <div class="space-y-4 max-h-[75vh] overflow-y-auto pr-1">
        <div>
          <Label>Nama Alumni</Label>
          <Input v-model="alumniName" placeholder="Nama lengkap" class="mt-1.5" />
        </div>
        <div>
          <Label>Sekolah</Label>
          <Input :model-value="user?.school_name || ''" disabled class="mt-1.5 bg-slate-50" />
        </div>
        <div>
          <Label>Universitas &amp; Program Studi</Label>
          <div v-if="alumniPickedProgram" class="mt-1.5 flex items-center justify-between gap-2 p-2.5 rounded-lg border bg-slate-50">
            <div class="min-w-0">
              <div class="text-sm font-semibold text-slate-900 truncate">{{ alumniPickedProgram.nama_prodi }}</div>
              <div class="text-xs text-muted-foreground truncate">{{ alumniPickedProgram.nama_ptn }}</div>
            </div>
            <button type="button" class="p-1 rounded hover:bg-slate-200 text-slate-500 shrink-0" title="Ganti" @click="clearAlumniProgram"><X class="w-4 h-4" /></button>
          </div>
          <div v-else class="mt-1.5">
            <div class="relative">
              <Search class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
              <Input v-model="alumniSearchQuery" placeholder="Cari PTN/prodi (min. 3 huruf)..." class="pl-8 text-xs h-9" />
            </div>
            <div v-if="alumniSearching" class="text-xs text-muted-foreground text-center py-2">Mencari...</div>
            <div v-else-if="alumniSearchQuery.trim().length < 3" class="text-xs text-muted-foreground text-center py-2">Ketik minimal 3 huruf untuk mencari PTN atau program studi.</div>
            <div v-else-if="alumniSearchResults.length === 0" class="text-xs text-muted-foreground text-center py-2">Tidak ditemukan.</div>
            <div v-else class="space-y-1 max-h-40 overflow-y-auto mt-1.5">
              <button
                v-for="p in alumniSearchResults" :key="p.id" type="button"
                class="w-full text-left px-2 py-1.5 rounded-lg hover:bg-slate-50 border text-xs"
                @click="pickAlumniProgram(p)"
              >
                <div class="font-semibold text-slate-900 truncate">{{ p.nama_prodi }}</div>
                <div class="text-[10px] text-muted-foreground truncate">{{ p.nama_ptn }}</div>
              </button>
            </div>
          </div>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <Label>Tahun Lulus</Label>
            <select v-model.number="alumniYear" class="mt-1.5 w-full h-10 px-3 border rounded-lg text-sm bg-white">
              <option v-for="y in alumniYearOptions" :key="y" :value="y">{{ y }}{{ y === alumniCurrentYear ? ' (Tahun Sekarang)' : '' }}</option>
            </select>
          </div>
          <div>
            <Label>PM Rapor (Nilai Acuan)</Label>
            <Input v-model.number="alumniScore" type="number" step="0.1" class="mt-1.5" />
          </div>
        </div>

        <div class="flex items-center justify-between p-3 rounded-lg border bg-slate-50">
          <div>
            <p class="text-sm font-semibold text-slate-900">Detail Rapor</p>
            <p class="text-xs text-muted-foreground">Nilai per mata pelajaran (opsional), untuk acuan pembanding lebih detail.</p>
          </div>
          <div class="flex rounded-lg border overflow-hidden shrink-0">
            <button type="button" class="px-3 py-1.5 text-xs font-semibold" :class="alumniHasRaporDetail ? 'bg-emerald-600 text-white' : 'bg-white text-slate-500'" @click="alumniHasRaporDetail = true">Ada</button>
            <button type="button" class="px-3 py-1.5 text-xs font-semibold" :class="!alumniHasRaporDetail ? 'bg-slate-600 text-white' : 'bg-white text-slate-500'" @click="alumniHasRaporDetail = false">Tidak Ada</button>
          </div>
        </div>

        <div v-if="alumniHasRaporDetail" class="space-y-2.5">
          <div class="flex items-center justify-between gap-2">
            <p class="text-xs text-muted-foreground">{{ alumniRaporFilledCount }} / {{ RAPOR_SUBJECTS.length }} mapel terisi</p>
            <div class="flex items-center gap-1.5">
              <input ref="alumniRaporImportInputRef" type="file" accept=".xlsx,.xls" class="hidden" @change="onAlumniRaporImportFileChange" />
              <button
                type="button" class="text-[11px] font-semibold text-slate-500 hover:underline"
                @click="downloadRaporImportTemplate('template_import_rapor_alumni.xlsx', { includeNis: false, includeSemester: false })"
              >Unduh Template</button>
              <Button variant="outline" size="sm" :disabled="alumniRaporImportLoading" @click="openAlumniRaporImportPicker">
                <Loader2 v-if="alumniRaporImportLoading" class="w-4 h-4 animate-spin" /><Paperclip v-else class="w-4 h-4" /> Import Excel
              </Button>
            </div>
          </div>
          <div class="grid grid-cols-2 gap-2 max-h-56 overflow-y-auto p-2 border rounded-lg">
            <div v-for="s in RAPOR_SUBJECTS" :key="s">
              <label class="text-[11px] text-slate-500 block truncate" :title="s">{{ s }}</label>
              <Input v-model.number="alumniRaporValues[s]" type="number" min="0" max="100" step="0.1" placeholder="-" class="h-8 text-xs mt-0.5" />
            </div>
          </div>
        </div>

        <div class="flex items-center justify-between pt-4 border-t">
          <Button variant="outline" @click="showAlumniForm = false">Batal</Button>
          <Button variant="gradient" :disabled="savingAlumni" @click="saveAlumni">
            <Sparkles class="w-4 h-4" /> {{ savingAlumni ? 'Menyimpan...' : 'Simpan Data' }}
          </Button>
        </div>
      </div>
    </Dialog>

    <!-- Dialog: Tambah/Update Eligible -->
    <Dialog v-model="showEligibleForm" title="Tambah/Update Data Eligible" max-width="max-w-sm">
      <div class="space-y-4">
        <div>
          <Label>Tahun</Label>
          <Input v-model.number="eligibleYear" type="number" class="mt-1.5" />
        </div>
        <div>
          <Label>Jumlah Siswa Eligible</Label>
          <Input v-model.number="eligibleCount" type="number" min="0" class="mt-1.5" />
        </div>
        <div class="flex items-center justify-between pt-4 border-t">
          <Button variant="outline" @click="showEligibleForm = false">Batal</Button>
          <Button variant="gradient" :disabled="savingEligible" @click="saveEligible">
            <Sparkles class="w-4 h-4" /> {{ savingEligible ? 'Menyimpan...' : 'Simpan' }}
          </Button>
        </div>
      </div>
    </Dialog>

    <!-- Dialog: Edit roster (konsultan/status/aktif) -->
    <Dialog v-model="showRosterForm" :title="`Edit Data — ${rosterFormStudent?.student_name || ''}`" max-width="max-w-sm">
      <div class="space-y-4">
        <div>
          <Label>Konsultan</Label>
          <Input v-model="rosterFormKonsultan" placeholder="Nama konsultan/pembimbing" class="mt-1.5" />
        </div>
        <div>
          <Label>Status Penerimaan</Label>
          <select v-model="rosterFormStatus" class="w-full mt-1.5 px-3 py-2 border rounded-lg text-sm bg-white">
            <option v-for="(label, val) in STATUS_LABEL" :key="val" :value="val">{{ label }}</option>
          </select>
        </div>
        <div>
          <Label>Tahun</Label>
          <Input v-model.number="rosterFormYear" type="number" class="mt-1.5" />
        </div>
        <label class="flex items-center gap-2 text-sm">
          <input v-model="rosterFormAktif" type="checkbox" class="rounded" /> Aktif di roster SNBP
        </label>
        <div class="flex items-center justify-between pt-4 border-t">
          <Button variant="outline" @click="showRosterForm = false">Batal</Button>
          <Button variant="gradient" :disabled="savingRoster" @click="saveRosterForm">
            <Sparkles class="w-4 h-4" /> {{ savingRoster ? 'Menyimpan...' : 'Simpan' }}
          </Button>
        </div>
      </div>
    </Dialog>

    <!-- Dialog: pilih siswa untuk "+ Tambah Data" -->
    <Dialog v-model="showWizardPicker" title="Pilih Siswa" max-width="max-w-md">
      <div class="space-y-3">
        <div class="relative">
          <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
          <Input v-model="wizardPickerSearch" placeholder="Cari siswa..." class="pl-9" />
        </div>
        <div v-if="loadingRoster" class="text-xs text-muted-foreground text-center py-4">Memuat...</div>
        <div v-else-if="wizardCandidates.length === 0" class="text-xs text-muted-foreground text-center py-4">Tidak ada siswa ditemukan.</div>
        <div v-else class="space-y-1 max-h-80 overflow-y-auto">
          <button
            v-for="r in wizardCandidates" :key="r.student_id"
            class="w-full text-left px-3 py-2 rounded-lg text-sm hover:bg-slate-50 transition-colors"
            @click="pickWizardStudent(r.student_id)"
          >
            <div class="font-semibold text-slate-900">{{ r.student_name }}</div>
            <div class="text-[10px] text-muted-foreground">{{ r.kelas || '-' }}</div>
          </button>
        </div>
      </div>
    </Dialog>

    <!-- Dialog: wizard verifikasi/edit data akademik siswa (rapor/prestasi/target) —
         komponen bersama, sama persis dengan yang dipakai siswa untuk input sendiri. -->
    <Dialog v-model="showWizardEditor" :title="`Verifikasi Data SNBP — ${wizardStudentName}`" max-width="max-w-3xl">
      <p class="text-xs text-muted-foreground -mt-2 mb-4">
        Data di bawah adalah nilai rapor, prestasi, dan target PTN yang diinput siswa sendiri — kamu bisa
        mengoreksi atau melengkapinya di sini. Perubahan langsung tersimpan ke akun siswa yang sama.
      </p>
      <RasionalisasiSnbpEditor
        v-if="wizardStudentId" :key="wizardStudentId" :student-id="wizardStudentId"
        :roster-row="roster.find((r) => r.student_id === wizardStudentId)"
        wizard @changed="onWizardChanged" @close="showWizardEditor = false"
      />
    </Dialog>

    <ConfirmDialog
      :model-value="!!confirmDeleteAlumniId"
      title="Hapus data alumni?"
      description="Data benchmark alumni ini akan dihapus permanen."
      @update:model-value="(v) => { if (!v) confirmDeleteAlumniId = null }"
      @confirm="deleteAlumni"
    />

    <!-- Preview + hasil import nilai rapor massal -->
    <Teleport to="body">
      <div v-if="raporBulkOpen" class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" @click="closeRaporBulkDialog">
        <Card class="w-full max-w-2xl shadow-2xl my-8 max-h-[85vh] flex flex-col" @click.stop>
          <div class="flex items-center justify-between px-6 py-4 border-b shrink-0">
            <h3 class="font-bold text-slate-900">Import Nilai Rapor Massal</h3>
            <button class="p-1.5 rounded-lg hover:bg-slate-100" @click="closeRaporBulkDialog"><X class="w-4 h-4" /></button>
          </div>
          <div class="px-6 py-4 overflow-y-auto space-y-3">
            <template v-if="!raporBulkResult">
              <p class="text-sm">
                <strong class="text-emerald-600">{{ raporBulkTotalEntries }} nilai</strong> siap diimpor dari
                <strong>{{ raporBulkValidRows.length }}</strong> siswa/semester.
              </p>
              <div v-if="raporBulkErrorRows.length" class="rounded-lg border border-red-100 bg-red-50 p-3 space-y-1 max-h-28 overflow-y-auto">
                <p v-for="r in raporBulkErrorRows" :key="r.rowNumber" class="text-xs text-red-700">Baris {{ r.rowNumber }}: {{ r.rowError }}</p>
              </div>
              <div v-if="raporBulkNisMissingRows.length" class="rounded-lg border border-red-100 bg-red-50 p-3 space-y-1 max-h-28 overflow-y-auto">
                <p v-for="r in raporBulkNisMissingRows" :key="r.rowNumber" class="text-xs text-red-700">Baris {{ r.rowNumber }}: NIS kosong — baris dilewati</p>
              </div>
              <div v-if="raporBulkRows.some((r) => r.cellWarnings.length)" class="rounded-lg border border-amber-100 bg-amber-50 p-3 space-y-1 max-h-28 overflow-y-auto">
                <template v-for="r in raporBulkRows" :key="'w' + r.rowNumber">
                  <p v-for="(w, i) in r.cellWarnings" :key="i" class="text-xs text-amber-700">Baris {{ r.rowNumber }} (NIS {{ r.nis || '-' }}): {{ w }}</p>
                </template>
              </div>
              <div class="rounded-lg border divide-y max-h-64 overflow-y-auto">
                <div v-for="r in raporBulkValidRows" :key="r.rowNumber" class="px-3 py-2 text-xs">
                  <span class="font-semibold">NIS {{ r.nis }}<template v-if="r.nama"> — {{ r.nama }}</template> · Semester {{ r.semester }}</span> — {{ r.entries.length }} mapel:
                  <span class="text-muted-foreground">{{ r.entries.map((e) => `${e.subject} (${e.score})`).join(', ') }}</span>
                </div>
                <p v-if="raporBulkValidRows.length === 0" class="px-3 py-4 text-center text-xs text-muted-foreground">Tidak ada baris valid untuk diimpor.</p>
              </div>
            </template>
            <template v-else>
              <div class="rounded-lg border border-emerald-100 bg-emerald-50 p-3 text-sm text-emerald-800">
                <strong>{{ raporBulkResult.saved_count }} nilai</strong> berhasil diimpor.
              </div>
              <div v-if="raporBulkResult.errors.length" class="rounded-lg border border-red-100 bg-red-50 p-3 space-y-1 max-h-56 overflow-y-auto">
                <p class="text-xs font-semibold text-red-800 mb-1">{{ raporBulkResult.errors.length }} baris gagal:</p>
                <p v-for="(err, i) in raporBulkResult.errors" :key="i" class="text-xs text-red-700">NIS {{ err.nis }}, Semester {{ err.semester }}, {{ err.subject }}: {{ err.reason }}</p>
              </div>
            </template>
          </div>
          <div class="px-6 py-4 border-t flex gap-2 shrink-0">
            <template v-if="!raporBulkResult">
              <Button variant="outline" class="flex-1" @click="closeRaporBulkDialog">Batal</Button>
              <Button variant="gradient" class="flex-1 gap-1.5" :disabled="raporBulkSubmitting || raporBulkTotalEntries === 0" @click="confirmRaporBulkImport">
                <Loader2 v-if="raporBulkSubmitting" class="w-4 h-4 animate-spin" /> Impor {{ raporBulkTotalEntries }} Nilai
              </Button>
            </template>
            <Button v-else variant="gradient" class="flex-1" @click="closeRaporBulkDialog">Selesai</Button>
          </div>
        </Card>
      </div>
    </Teleport>
  </div>
</template>
