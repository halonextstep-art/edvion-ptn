<script setup lang="ts">
// Editor SNBP (nilai rapor + prestasi + Pilihan Universitas 1/2 + resume peluang) —
// dipakai di DUA tempat dengan UI/alur yang PERSIS SAMA:
//  1. Portal Siswa (RasionalisasiSNBT.vue, mode "Jalur SNBP") — siswa input sendiri,
//     tampilan flat (semua bagian sekaligus). `studentId` tidak di-pass → semua request
//     lewat endpoint self-service (/me-style: /rationalization/rapor, /achievements,
//     /targets, /preview).
//  2. Portal Sekolah (SchoolRasionalisasi.vue, tombol "+ Tambah Data" di tab
//     Rasionalisasi) — PIC sekolah memverifikasi/mengoreksi data SNBP siswa yang sudah
//     dipilih, lewat wizard 4-langkah (Data Siswa → Prestasi → Nilai Rapor → Resume)
//     mengikuti alur referensi. `studentId` di-pass → request lewat endpoint
//     /rationalization/students/:id/... yang sama persis secara struktur, tinggal
//     di-scope ke siswa tertentu (backend cek ensure_owns_student).
//
// PENTING: setiap langkah wizard MENGEDIT record nyata yang sudah ada (rapor/prestasi/
// target tersimpan langsung saat diisi, bukan draft lokal yang baru dikirim di akhir)
// — beda dari referensi yang menyimpan semua sekaligus lewat "Selesai & Simpan". Ini
// justru lebih aman (tidak ada risiko kehilangan input kalau dialog tertutup tidak
// sengaja), jadi tombol terakhir di wizard hanya menutup dialog, bukan "menyimpan".
//
// Adaptasi sengaja dari referensi (data tetap nyata, tidak dikarang):
//  - "Resume Peluang" (step 4, wizard) punya DUA kategori alternatif nyata, sama persis
//    dengan referensi: "Sesuai Rumpun Program Studi" (dari katalog PTN rumpun yang sama
//    dengan pilihan siswa) dan "Sesuai Minat Jurusan" (dari field `minat_jurusan` kalau
//    diisi sendiri oleh siswa lewat kartu "Jurusan yang Diminati" di bawah — OPSIONAL;
//    kalau kosong, otomatis fallback ke `nama_prodi` pilihan yang sudah dipilih siswa
//    (Pilihan 1/2), karena siswa yang sudah memilih program itu SUDAH menyatakan minat
//    lewat pilihan itu sendiri — mewajibkan isi field terpisah untuk hal yang sama adalah
//    kerja ganda. Field manual tetap ada sebagai override kalau minat siswa berbeda dari
//    program yang sudah dipilih. Kalau BENAR-BENAR tidak ada sinyal apapun (tidak ada
//    minat_jurusan DAN tidak ada pilihan), baru tampil ajakan mengisi — bukan angka
//    kosong/karangan).
//  - Wizard hanya kelola 2 pilihan (Utama/Cadangan), sesuai referensi. Target dengan
//    prioritas "Aman" (kalau ada) tetap tersimpan di data siswa, hanya tidak dikelola
//    lewat wizard ini — kelola penuh (termasuk "Aman") tetap ada di tampilan flat
//    (portal siswa sendiri).
import { Search, Plus, Trash2, Award, Info, X, Loader2, ChevronLeft, ChevronRight, CheckCircle2, Pencil, Paperclip } from 'lucide-vue-next'
import type {
  PtnProgramItem, PtnPriority, TargetWithChanceItem, RaporScoreItem, RaporEntryPayload, AchievementItem,
  AchievementLevelValue, SnbpRosterRowItem,
} from '~/types'
import { RAPOR_SUBJECTS, raporSubjectRoot } from '~/utils/raporSubjects'
import { parseRaporImportWorkbook, downloadRaporImportTemplate, type ParsedRaporRow } from '~/utils/raporImportXlsx'

const props = defineProps<{ studentId?: string; wizard?: boolean; rosterRow?: SnbpRosterRowItem }>()
const emit = defineEmits<{ changed: []; close: [] }>()
const { rationalizationService } = useApi()
const isSelf = computed(() => !props.studentId)
const { resolve: resolveUploadUrl } = useUploadUrl()
const { user, updateProfile } = useAuth()
const toast = useToast()

// ─── Mode wizard (step-by-step, 4 langkah ala referensi) ───────────────────────────
const STEP_LABEL = ['Data Siswa', 'Prestasi', 'Nilai Rapor', 'Resume']
const step = ref(1)
watch(() => props.studentId, () => { step.value = 1 })
watch(step, (s) => {
  if (props.wizard && s === 4) { loadResumeAlternatif(); loadResumeAlternatifMinat() }
})

// Daftar mapel (RAPOR_SUBJECTS) dan pencocokan root-keyword (raporSubjectRoot) sekarang
// ditarik dari satu sumber bersama (utils/raporSubjects.ts) — dipakai identik oleh
// SchoolRasionalisasi.vue dan parser import Excel (raporImportXlsx.ts), supaya ketiganya
// tidak pernah divergen kalau daftar mapel berubah lagi ke depannya.
const SUBJECTS: readonly string[] = RAPOR_SUBJECTS
const subjectRoot = raporSubjectRoot
const TIER_LABEL: Record<string, string> = { aman: 'Sangat Aman', moderat: 'Peluang Sedang', ketat: 'Perlu Peningkatan' }
const TIER_VARIANT: Record<string, 'success' | 'warning' | 'destructive'> = {
  aman: 'success', moderat: 'warning', ketat: 'destructive',
}
const LEVEL_LABEL: Record<AchievementLevelValue, string> = {
  sekolah: 'Sekolah', 'kab-kota': 'Kab/Kota', provinsi: 'Provinsi', nasional: 'Nasional', internasional: 'Internasional',
}

// ─── Nilai Rapor (grid per semester, batch-save lewat endpoint yang sudah ada) ─────
const rapor = ref<RaporScoreItem[]>([])
const raporLoading = ref(true)
const raporSemesterTab = ref(1)
const raporGrid = reactive<Record<string, { score: number | null; is_minat: boolean }>>(
  Object.fromEntries(SUBJECTS.map((s) => [s, { score: null, is_minat: false }])),
)
const savingRaporGrid = ref(false)
/** Daftar mapel sekarang ~46 item (Kurikulum Merdeka), jauh lebih banyak dari 13 mapel
 * lama — grid yang selalu tampil semua jadi kepanjangan untuk di-scroll. Kotak cari ini
 * hanya menyaring TAMPILAN grid input; tidak memengaruhi data yang sudah tersimpan. */
const raporSubjectQuery = ref('')
const filteredSubjects = computed(() => {
  const q = raporSubjectQuery.value.trim().toLowerCase()
  return q ? SUBJECTS.filter((s) => s.toLowerCase().includes(q)) : SUBJECTS
})

function syncRaporGridFromData() {
  for (const subj of SUBJECTS) {
    const existing = rapor.value.find((r) => r.semester === raporSemesterTab.value && r.subject === subj)
    raporGrid[subj] = existing ? { score: existing.score, is_minat: existing.is_minat } : { score: null, is_minat: false }
  }
}
watch(raporSemesterTab, syncRaporGridFromData)
watch(rapor, syncRaporGridFromData)

async function loadRapor() {
  raporLoading.value = true
  try {
    rapor.value = props.studentId
      ? await rationalizationService.studentRapor(props.studentId)
      : await rationalizationService.listRapor()
  } finally {
    raporLoading.value = false
  }
}
async function saveRaporGrid() {
  const entries: RaporEntryPayload[] = SUBJECTS
    .filter((s) => raporGrid[s].score != null)
    .map((s) => ({ semester: raporSemesterTab.value, subject: s, score: raporGrid[s].score as number, is_minat: raporGrid[s].is_minat }))
  if (entries.length === 0) return
  savingRaporGrid.value = true
  try {
    if (props.studentId) await rationalizationService.upsertStudentRapor(props.studentId, entries)
    else await rationalizationService.upsertRapor(entries)
    await loadRapor()
    await refreshTargets()
    emit('changed')
  } finally {
    savingRaporGrid.value = false
  }
}
async function removeRapor(id: string) {
  if (props.studentId) await rationalizationService.deleteStudentRapor(props.studentId, id)
  else await rationalizationService.deleteRapor(id)
  await loadRapor()
  await refreshTargets()
  emit('changed')
}

// ─── Import nilai rapor via Excel (1 siswa — ini juga dipakai School PIC saat mengedit
// satu siswa lewat `studentId`, bukan hanya siswa sendiri) ─────────────────────────────
const raporImportInputRef = ref<HTMLInputElement | null>(null)
const raporImportOpen = ref(false)
const raporImportLoading = ref(false)
const raporImportSaving = ref(false)
const raporImportRows = ref<ParsedRaporRow[]>([])

function openRaporImportPicker() {
  raporImportInputRef.value?.click()
}
async function onRaporImportFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  raporImportLoading.value = true
  try {
    const result = await parseRaporImportWorkbook(file)
    raporImportRows.value = result.rows
    raporImportOpen.value = true
  } catch (err: any) {
    toast.error('Gagal membaca file', err?.message)
  } finally {
    raporImportLoading.value = false
    input.value = ''
  }
}
// Baris tanpa error DAN punya minimal 1 nilai valid — baris kosong (semua sel dilewati)
// tidak perlu dikirim, tidak ada gunanya menimpa apa pun.
const raporImportValidRows = computed(() => raporImportRows.value.filter((r) => !r.rowError && r.entries.length > 0))
const raporImportErrorRows = computed(() => raporImportRows.value.filter((r) => r.rowError))
const raporImportTotalEntries = computed(() => raporImportValidRows.value.reduce((sum, r) => sum + r.entries.length, 0))

async function confirmRaporImport() {
  const entries: RaporEntryPayload[] = []
  for (const row of raporImportValidRows.value) {
    for (const e of row.entries) {
      entries.push({ semester: row.semester as number, subject: e.subject, score: e.score, is_minat: false })
    }
  }
  if (entries.length === 0) return
  raporImportSaving.value = true
  try {
    if (props.studentId) await rationalizationService.upsertStudentRapor(props.studentId, entries)
    else await rationalizationService.upsertRapor(entries)
    toast.success(`${entries.length} nilai rapor berhasil diimpor`)
    raporImportOpen.value = false
    raporImportRows.value = []
    await loadRapor()
    await refreshTargets()
    emit('changed')
  } catch (e: any) {
    toast.error('Gagal mengimpor nilai rapor', e?.message)
  } finally {
    raporImportSaving.value = false
  }
}
const raporAverage = computed(() => {
  if (!rapor.value.length) return null
  return Math.round((rapor.value.reduce((sum, r) => sum + r.score, 0) / rapor.value.length) * 10) / 10
})
/** Dikelompokkan per semester untuk "Riwayat Nilai Tersimpan" — supaya nilai yang sama
 * di semester BERBEDA (data valid, dua semester memang boleh punya nilai Matematika
 * masing-masing) tidak terlihat seperti baris ganda yang keliru. */
const raporGroupedBySemester = computed(() => {
  const bySemester = new Map<number, RaporScoreItem[]>()
  for (const r of rapor.value) {
    if (!bySemester.has(r.semester)) bySemester.set(r.semester, [])
    bySemester.get(r.semester)!.push(r)
  }
  return Array.from(bySemester.entries())
    .sort(([a], [b]) => a - b)
    .map(([semester, entries]) => ({ semester, entries }))
})
/** Riwayat bisa jadi sangat panjang ke bawah kalau semua semester ditampilkan sekaligus
 * (mis. 5 semester x 13 mapel = 65 baris). Default-nya cuma tampilkan semester yang lagi
 * aktif di tab S1-S5 di atas (paling relevan saat itu), dengan opsi "Tampilkan semua
 * semester" untuk yang benar-benar perlu lihat riwayat lengkap sekaligus. */
const showAllSemesters = ref(false)
const visibleRaporGroups = computed(() =>
  showAllSemesters.value
    ? raporGroupedBySemester.value
    : raporGroupedBySemester.value.filter((g) => g.semester === raporSemesterTab.value),
)

// ─── Prestasi ───────────────────────────────────────────────────────────────────────
const achievements = ref<AchievementItem[]>([])
const achievementsLoading = ref(true)
const achievementForm = reactive({ nama: '', tingkat: 'provinsi' as AchievementLevelValue, tahun: new Date().getFullYear(), juara: '' })
const savingAchievement = ref(false)

async function loadAchievements() {
  achievementsLoading.value = true
  try {
    achievements.value = props.studentId
      ? await rationalizationService.studentAchievements(props.studentId)
      : await rationalizationService.listAchievements()
  } finally {
    achievementsLoading.value = false
  }
}
async function saveAchievement() {
  if (!achievementForm.nama.trim()) return
  savingAchievement.value = true
  try {
    if (props.studentId) await rationalizationService.addStudentAchievement(props.studentId, { ...achievementForm })
    else await rationalizationService.addAchievement({ ...achievementForm })
    achievementForm.nama = ''; achievementForm.juara = ''
    await loadAchievements()
    await refreshTargets()
    emit('changed')
  } finally {
    savingAchievement.value = false
  }
}
async function removeAchievement(id: string) {
  if (props.studentId) await rationalizationService.deleteStudentAchievement(props.studentId, id)
  else await rationalizationService.deleteAchievement(id)
  await loadAchievements()
  await refreshTargets()
  emit('changed')
}
/** "Edit": belum ada endpoint update prestasi di backend, jadi revisi dilakukan dengan
 * memuat ulang nilai lama ke form lalu menghapus baris lama — user tinggal koreksi dan
 * klik "Tambah" lagi. Tetap real (bukan draft lokal), hanya jalur revisinya lewat
 * hapus+tambah ulang. */
async function editAchievement(a: AchievementItem) {
  achievementForm.nama = a.nama
  achievementForm.tingkat = a.tingkat
  achievementForm.tahun = a.tahun
  achievementForm.juara = a.juara
  await removeAchievement(a.id)
}
const prestasiIndex = computed(() => Math.min(achievements.value.reduce((sum, a) => sum + a.bonus, 0), 10))

// ── Upload sertifikat (self-service saja — sertifikat fisik cuma dipegang siswa
// sendiri, jadi tidak ada jalur "edit atas nama siswa" untuk ini seperti data lain). ──
const certificateInputs = reactive<Record<string, HTMLInputElement | null>>({})
const uploadingCertificateId = ref<string | null>(null)
const CERT_ALLOWED = ['image/png', 'image/jpeg', 'image/webp', 'application/pdf']
const CERT_MAX_BYTES = 8 * 1024 * 1024

function setCertificateInputRef(id: string, el: any) {
  certificateInputs[id] = (el as HTMLInputElement) ?? null
}
function pickCertificate(id: string) {
  certificateInputs[id]?.click()
}
async function onCertificateSelected(a: AchievementItem, e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  if (!CERT_ALLOWED.includes(file.type)) {
    toast.error('Format tidak didukung', 'Gunakan PNG, JPEG, WEBP, atau PDF.')
    return
  }
  if (file.size > CERT_MAX_BYTES) {
    toast.error('File terlalu besar', 'Maksimum 8 MB.')
    return
  }
  uploadingCertificateId.value = a.id
  try {
    const updated = await rationalizationService.uploadAchievementCertificate(a.id, file)
    const idx = achievements.value.findIndex((x) => x.id === a.id)
    if (idx !== -1) achievements.value[idx] = updated
    toast.success('Sertifikat diunggah')
  } catch (err: any) {
    toast.error('Gagal mengunggah sertifikat', err?.message)
  } finally {
    uploadingCertificateId.value = null
    const input = certificateInputs[a.id]
    if (input) input.value = ''
  }
}

// ─── Jurusan yang Diminati (real, diisi sendiri oleh siswa) — dipakai sebagai basis
// pencarian "Alternatif Sesuai Minat Jurusan" di Resume (step 4). Hanya siswa sendiri yang
// boleh mengisi (isSelf) — sekolah cuma bisa MELIHAT nilai yang sudah diisi siswa (lewat
// rosterRow.minat_jurusan saat wizard, lihat loadResumeAlternatifMinat di bawah), tidak
// pernah mengedit atas nama siswa untuk field ini. Disimpan lewat mekanisme profil yang
// sama dengan ProfileDialog.vue (PUT /api/auth/me) — backend menimpa kolom ini tanpa
// syarat, jadi name/phone milik user saat ini WAJIB dikirim ulang di sini (konvensi yang
// sama seperti ProfileDialog.vue, bukan mekanisme partial-update baru). ─────────────────
const minatJurusanInput = ref(user.value?.minat_jurusan || '')
watch(() => user.value?.minat_jurusan, (v) => { minatJurusanInput.value = v || '' })
const savingMinatJurusan = ref(false)
async function saveMinatJurusan() {
  if (!user.value) return
  savingMinatJurusan.value = true
  try {
    await updateProfile({
      name: user.value.name, phone: user.value.phone ?? null,
      minat_jurusan: minatJurusanInput.value.trim() || null,
    })
    toast.success('Jurusan yang diminati disimpan')
  } catch (e: any) {
    toast.error('Gagal menyimpan', e?.message)
  } finally {
    savingMinatJurusan.value = false
  }
}

// ─── Cari & tambah target PTN (jalur SNBP) ─────────────────────────────────────────
// Di mode wizard, dipakai di dalam 2 slot "Pilihan Universitas 1/2" (step "Data Siswa");
// di mode flat (portal siswa sendiri), tetap sebagai kartu "Cari Program Studi" terpisah
// seperti sebelumnya.
const searchQuery = ref('')
const searchResults = ref<PtnProgramItem[]>([])
const searching = ref(false)
let searchTimer: ReturnType<typeof setTimeout> | null = null
watch(searchQuery, () => {
  if (searchTimer) clearTimeout(searchTimer)
  // Set status "Mencari..." SEGERA (bukan menunggu jeda 350ms selesai) begitu sudah 3+ huruf —
  // sebelumnya ada jeda kosong tanpa umpan balik apa pun antara mulai mengetik dan pencarian
  // benar-benar jalan, yang terlihat seperti kotak pencariannya tidak merespons sama sekali.
  if (searchQuery.value.trim().length >= 3) {
    searching.value = true
    searchResults.value = []
  }
  searchTimer = setTimeout(runSearch, 350)
})
async function runSearch() {
  if (searchQuery.value.trim().length < 3) {
    searchResults.value = []
    searching.value = false
    return
  }
  searching.value = true
  try {
    const res = await rationalizationService.listPrograms({ search: searchQuery.value.trim(), page_size: 15 })
    searchResults.value = res.items
  } finally {
    searching.value = false
  }
}

// Slot aktif di step "Data Siswa" ('utama' = Pilihan 1, 'cadangan' = Pilihan 2, null =
// tidak ada slot yang sedang mencari) — dua slot berbagi searchQuery/searchResults yang
// sama karena cuma satu yang tampil sekaligus.
const activeSlot = ref<'utama' | 'cadangan' | null>(null)
function openSlotSearch(slot: 'utama' | 'cadangan') {
  activeSlot.value = slot
  searchQuery.value = ''
  searchResults.value = []
}

const pickerProgram = ref<PtnProgramItem | null>(null)
const pickerPriority = ref<PtnPriority>('utama')
const pickerPreview = ref<{ chance_percent: number; tier: string } | null>(null)
const previewLoading = ref(false)
const adding = ref(false)

function openPicker(program: PtnProgramItem, priority: PtnPriority = 'utama') {
  pickerProgram.value = program
  pickerPriority.value = priority
  pickerPreview.value = null
  loadPreview()
}
function closePicker() {
  pickerProgram.value = null
}
async function loadPreview() {
  if (!pickerProgram.value) return
  previewLoading.value = true
  try {
    pickerPreview.value = props.studentId
      ? await rationalizationService.previewStudentChance(props.studentId, pickerProgram.value.id, 'snbp')
      : await rationalizationService.previewChance(pickerProgram.value.id, 'snbp')
  } catch {
    pickerPreview.value = null
  } finally {
    previewLoading.value = false
  }
}
async function confirmAddTarget() {
  if (!pickerProgram.value) return
  adding.value = true
  try {
    if (props.studentId) await rationalizationService.addStudentTarget(props.studentId, pickerProgram.value.id, 'snbp', pickerPriority.value)
    else await rationalizationService.addTarget(pickerProgram.value.id, 'snbp', pickerPriority.value)
    closePicker()
    searchQuery.value = ''
    searchResults.value = []
    activeSlot.value = null
    await refreshTargets()
    emit('changed')
  } finally {
    adding.value = false
  }
}

// ─── Target PTN saya (jalur SNBP) ───────────────────────────────────────────────────
const targets = ref<TargetWithChanceItem[]>([])
const targetsLoading = ref(true)

async function refreshTargets() {
  targetsLoading.value = true
  try {
    targets.value = props.studentId
      ? await rationalizationService.studentTargets(props.studentId)
      : await rationalizationService.myTargets()
  } finally {
    targetsLoading.value = false
  }
}
async function changePriority(t: TargetWithChanceItem, priority: PtnPriority) {
  if (props.studentId) await rationalizationService.setStudentTargetPriority(props.studentId, t.id, priority)
  else await rationalizationService.setTargetPriority(t.id, priority)
  await refreshTargets()
  emit('changed')
}
async function removeTargetItem(t: TargetWithChanceItem) {
  if (props.studentId) await rationalizationService.removeStudentTarget(props.studentId, t.id)
  else await rationalizationService.removeTarget(t.id)
  await refreshTargets()
  emit('changed')
}
const snbpTargets = computed(() => {
  const list = targets.value.filter((t) => t.track === 'snbp')
  const order: Record<PtnPriority, number> = { utama: 0, cadangan: 1, aman: 2 }
  return [...list].sort((a, b) => order[a.priority] - order[b.priority])
})
const pilihanUtama = computed(() => snbpTargets.value.find((t) => t.priority === 'utama') || null)
const pilihanCadangan = computed(() => snbpTargets.value.find((t) => t.priority === 'cadangan') || null)

// ─── Radar "Pencapaian vs Referensi" (step 4) — lihat PencapaianRadarChart.vue untuk
// penjelasan lengkap kenapa garis referensi memakai pg_snbp/pg_snbt agregat, bukan
// minimum per mapel (data itu tidak ada di katalog manapun). ─────────────────────────
function averageForSyaratSubject(entries: RaporScoreItem[], syaratSubject: string): number | null {
  if (!syaratSubject.trim()) return null
  const root = subjectRoot(syaratSubject)
  const matches = root
    ? entries.filter((r) => subjectRoot(r.subject) === root)
    : entries.filter((r) => r.subject.toLowerCase().includes(syaratSubject.trim().toLowerCase()))
  if (matches.length === 0) return null
  return Math.round((matches.reduce((sum, r) => sum + r.score, 0) / matches.length) * 10) / 10
}
const radarAnchor = computed(() => pilihanUtama.value || pilihanCadangan.value)
/** Sumbu radar = gabungan mata pelajaran syarat program (katalog nyata) + mata
 * pelajaran yang benar-benar sudah diisi nilai rapornya oleh siswa (real). Katalog
 * `mapel_syarat` sering cuma berisi 1-2 mapel (data CSV aslinya begitu), jadi kalau
 * cuma mengandalkan itu grafiknya nyaris selalu kosong — gabungan ini memastikan
 * grafik tetap tampil begitu siswa sudah mengisi rapor, tanpa menambah data karangan
 * (dua-duanya sumber nyata, cuma digabung). */
const radarSubjects = computed(() => {
  const syarat = (radarAnchor.value?.program.mapel_syarat || '').split(',').map((s) => s.trim()).filter(Boolean)
  const dariRapor = Array.from(new Set(rapor.value.map((r) => r.subject)))
  return Array.from(new Set([...syarat, ...dariRapor])).slice(0, 8)
})
const radarPencapaian = computed(() => radarSubjects.value.map((s) => averageForSyaratSubject(rapor.value, s)))
const radarReferensi = computed(() => radarAnchor.value?.program.pg_snbp ?? 0)

// ─── Resume (step 4): ringkasan + alternatif nyata dari katalog rumpun yang sama ───
const resumeAlternatif = ref<{ program: PtnProgramItem; chance_percent: number; tier: string }[]>([])
const loadingResumeAlternatif = ref(false)
async function loadResumeAlternatif() {
  const anchor = pilihanUtama.value || pilihanCadangan.value
  if (!anchor) {
    resumeAlternatif.value = []
    return
  }
  loadingResumeAlternatif.value = true
  try {
    const res = await rationalizationService.listPrograms({ rumpun: anchor.program.rumpun, page_size: 8 })
    const chosenIds = new Set(snbpTargets.value.map((t) => t.ptn_program_id))
    const candidates = res.items.filter((p) => !chosenIds.has(p.id)).slice(0, 3)
    const withChance = await Promise.all(candidates.map(async (p) => {
      try {
        const preview = props.studentId
          ? await rationalizationService.previewStudentChance(props.studentId, p.id, 'snbp')
          : await rationalizationService.previewChance(p.id, 'snbp')
        return { program: p, chance_percent: preview.chance_percent, tier: preview.tier }
      } catch {
        return null
      }
    }))
    resumeAlternatif.value = withChance.filter((x): x is { program: PtnProgramItem; chance_percent: number; tier: string } => x !== null)
  } catch {
    resumeAlternatif.value = []
  } finally {
    loadingResumeAlternatif.value = false
  }
}

/** Sumber nilai minat jurusan (field manual, OPSIONAL) untuk resume: kalau isSelf (jarang
 * di step 4 wizard, karena step 4 cuma pernah dipakai lewat wizard sekolah dengan
 * studentId), pakai punya sendiri; kalau via wizard sekolah, pakai `rosterRow.minat_jurusan`
 * (dibaca read-only dari data siswa nyata — sekolah tidak pernah mengisi field ini sendiri). */
const minatJurusanForResume = computed(() => (isSelf.value ? user.value?.minat_jurusan : props.rosterRow?.minat_jurusan) || '')

/** Search term EFEKTIF untuk "Alternatif Sesuai Minat Jurusan": pakai `minat_jurusan`
 * manual kalau siswa sudah mengisinya (explicit override), kalau tidak fallback ke
 * `nama_prodi` dari pilihan yang sudah dipilih siswa (anchor SAMA PERSIS dengan yang
 * dipakai `loadResumeAlternatif`/`radarAnchor` di atas — `pilihanUtama || pilihanCadangan`)
 * — karena siswa yang sudah memilih program itu sudah menyatakan minatnya lewat pilihan
 * tsb, tidak perlu mengetik ulang. Nilai fallback ini SELALU nilai nyata (nama_prodi milik
 * program yang benar-benar dipilih), tidak pernah dikarang. */
const usingMinatFallback = computed(() => !minatJurusanForResume.value.trim() && !!radarAnchor.value)
const effectiveMinatForResume = computed(() => minatJurusanForResume.value.trim() || radarAnchor.value?.program.nama_prodi || '')

// Alternatif "Sesuai Minat Jurusan" — komposisi IDENTIK dengan loadResumeAlternatif di
// atas (listPrograms + previewStudentChance/previewChance), cuma keyed pada `search:
// effectiveMinatForResume` (minat_jurusan manual kalau diisi, else fallback ke nama_prodi
// pilihan yang sudah dipilih — lihat komentar di atas) alih-alih `rumpun`. Tidak ada mesin
// pencarian/preview kedua — reuse murni.
const resumeAlternatifMinat = ref<{ program: PtnProgramItem; chance_percent: number; tier: string }[]>([])
const loadingResumeAlternatifMinat = ref(false)
async function loadResumeAlternatifMinat() {
  const minat = effectiveMinatForResume.value.trim()
  if (!minat) {
    resumeAlternatifMinat.value = []
    return
  }
  loadingResumeAlternatifMinat.value = true
  try {
    const res = await rationalizationService.listPrograms({ search: minat, page_size: 8 })
    const chosenIds = new Set(snbpTargets.value.map((t) => t.ptn_program_id))
    const candidates = res.items.filter((p) => !chosenIds.has(p.id)).slice(0, 3)
    const withChance = await Promise.all(candidates.map(async (p) => {
      try {
        const preview = props.studentId
          ? await rationalizationService.previewStudentChance(props.studentId, p.id, 'snbp')
          : await rationalizationService.previewChance(p.id, 'snbp')
        return { program: p, chance_percent: preview.chance_percent, tier: preview.tier }
      } catch {
        return null
      }
    }))
    resumeAlternatifMinat.value = withChance.filter((x): x is { program: PtnProgramItem; chance_percent: number; tier: string } => x !== null)
  } catch {
    resumeAlternatifMinat.value = []
  } finally {
    loadingResumeAlternatifMinat.value = false
  }
}

function reload() {
  loadRapor()
  loadAchievements()
  refreshTargets()
}
// Reload otomatis saat siswa yang dipilih berganti (dipakai di panel sekolah).
watch(() => props.studentId, reload)
onMounted(reload)
defineExpose({ reload })
</script>

<template>
  <div class="space-y-5">
    <!-- Stepper (mode wizard) — data di tiap step tetap live-tersimpan, ini cuma
         pembagian tampilan biar mengikuti alur wizard referensi. -->
    <div v-if="wizard" class="rounded-2xl bg-gradient-to-r from-indigo-600 to-purple-600 px-5 py-5">
      <div class="flex items-center">
        <template v-for="(label, i) in STEP_LABEL" :key="i">
          <button type="button" class="flex flex-col items-center gap-1.5 shrink-0" @click="step = i + 1">
            <span
              class="w-9 h-9 rounded-full flex items-center justify-center text-sm font-bold border-2 transition-colors"
              :class="step > i + 1 ? 'bg-white border-white text-indigo-600' : step === i + 1 ? 'bg-white/25 border-white text-white' : 'bg-transparent border-white/40 text-white/60'"
            >
              <CheckCircle2 v-if="step > i + 1" class="w-4 h-4" />
              <template v-else>{{ i + 1 }}</template>
            </span>
            <span class="text-[10px] font-semibold whitespace-nowrap" :class="step >= i + 1 ? 'text-white' : 'text-white/60'">{{ label }}</span>
          </button>
          <div v-if="i < STEP_LABEL.length - 1" class="flex-1 h-px mx-2 mb-4" :class="step > i + 1 ? 'bg-white' : 'bg-white/30'" />
        </template>
      </div>
    </div>

    <!-- Step 1 (wizard): Data Siswa — ringkasan roster (real, diedit lewat ikon pensil
         di tabel Daftar Siswa) + Pilihan Universitas 1 & 2 (target PTN nyata). -->
    <Card v-if="wizard && step === 1" class="p-5">
      <h4 class="font-bold text-slate-900 mb-4">Data Siswa</h4>
      <div class="grid grid-cols-2 gap-4 mb-4 text-sm">
        <div><p class="text-[10px] uppercase tracking-wide text-muted-foreground">Nama Siswa</p><p class="font-semibold text-slate-900">{{ rosterRow?.student_name || '-' }}</p></div>
        <div><p class="text-[10px] uppercase tracking-wide text-muted-foreground">Kelas</p><p class="font-semibold text-slate-900">{{ rosterRow?.kelas || '-' }}</p></div>
        <div><p class="text-[10px] uppercase tracking-wide text-muted-foreground">Konsultan / Guru BK</p><p class="font-semibold text-slate-900">{{ rosterRow?.konsultan || '-' }}</p></div>
        <div><p class="text-[10px] uppercase tracking-wide text-muted-foreground">Tahun Lulus</p><p class="font-semibold text-slate-900">{{ rosterRow?.year || '-' }}</p></div>
      </div>
      <p class="text-[11px] text-muted-foreground mb-5">Kelas, konsultan, dan tahun diedit lewat ikon pensil di tabel "Daftar Siswa".</p>

      <div class="grid sm:grid-cols-2 gap-3">
        <div class="rounded-xl border p-4">
          <p class="text-[10px] font-bold uppercase tracking-wide text-muted-foreground mb-3">Pilihan Universitas 1 (Utama)</p>
          <template v-if="pilihanUtama">
            <p class="font-semibold text-slate-900 truncate">{{ pilihanUtama.program.nama_ptn }}</p>
            <p class="text-xs text-muted-foreground truncate mb-2">{{ pilihanUtama.program.nama_prodi }}</p>
            <div class="flex items-center justify-between">
              <Badge v-if="!pilihanUtama.program.has_official_stats" class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
              <Badge v-else :variant="TIER_VARIANT[pilihanUtama.chance.tier]">{{ pilihanUtama.chance.chance_percent }}%</Badge>
              <button class="text-xs text-red-600 hover:underline" @click="removeTargetItem(pilihanUtama)">Ganti</button>
            </div>
          </template>
          <template v-else>
            <Button v-if="activeSlot !== 'utama'" variant="outline" size="sm" class="w-full" @click="openSlotSearch('utama')"><Plus class="w-3.5 h-3.5" /> Pilih Universitas</Button>
            <div v-else>
              <div class="relative mb-2">
                <Search class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
                <Input v-model="searchQuery" placeholder="Cari PTN/prodi (min. 3 huruf)..." class="pl-8 text-xs h-8" />
              </div>
              <div v-if="searching" class="text-xs text-muted-foreground text-center py-2">Mencari...</div>
              <div v-else-if="searchQuery.trim().length < 3" class="text-xs text-muted-foreground text-center py-2">Ketik minimal 3 huruf untuk mencari PTN atau program studi.</div>
              <div v-else-if="searchResults.length === 0" class="text-xs text-muted-foreground text-center py-2">Tidak ditemukan.</div>
              <div v-else class="space-y-1 max-h-40 overflow-y-auto">
                <button v-for="p in searchResults" :key="p.id" class="w-full text-left px-2 py-1.5 rounded-lg hover:bg-slate-50 text-xs" @click="openPicker(p, 'utama')">
                  <div class="font-semibold text-slate-900 truncate">{{ p.nama_prodi }}</div>
                  <div class="text-[10px] text-muted-foreground truncate">{{ p.nama_ptn }}</div>
                </button>
              </div>
            </div>
          </template>
        </div>

        <div class="rounded-xl border p-4">
          <p class="text-[10px] font-bold uppercase tracking-wide text-muted-foreground mb-3">Pilihan Universitas 2 (Cadangan)</p>
          <template v-if="pilihanCadangan">
            <p class="font-semibold text-slate-900 truncate">{{ pilihanCadangan.program.nama_ptn }}</p>
            <p class="text-xs text-muted-foreground truncate mb-2">{{ pilihanCadangan.program.nama_prodi }}</p>
            <div class="flex items-center justify-between">
              <Badge v-if="!pilihanCadangan.program.has_official_stats" class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
              <Badge v-else :variant="TIER_VARIANT[pilihanCadangan.chance.tier]">{{ pilihanCadangan.chance.chance_percent }}%</Badge>
              <button class="text-xs text-red-600 hover:underline" @click="removeTargetItem(pilihanCadangan)">Ganti</button>
            </div>
          </template>
          <template v-else>
            <Button v-if="activeSlot !== 'cadangan'" variant="outline" size="sm" class="w-full" @click="openSlotSearch('cadangan')"><Plus class="w-3.5 h-3.5" /> Pilih Universitas</Button>
            <div v-else>
              <div class="relative mb-2">
                <Search class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
                <Input v-model="searchQuery" placeholder="Cari PTN/prodi (min. 3 huruf)..." class="pl-8 text-xs h-8" />
              </div>
              <div v-if="searching" class="text-xs text-muted-foreground text-center py-2">Mencari...</div>
              <div v-else-if="searchQuery.trim().length < 3" class="text-xs text-muted-foreground text-center py-2">Ketik minimal 3 huruf untuk mencari PTN atau program studi.</div>
              <div v-else-if="searchResults.length === 0" class="text-xs text-muted-foreground text-center py-2">Tidak ditemukan.</div>
              <div v-else class="space-y-1 max-h-40 overflow-y-auto">
                <button v-for="p in searchResults" :key="p.id" class="w-full text-left px-2 py-1.5 rounded-lg hover:bg-slate-50 text-xs" @click="openPicker(p, 'cadangan')">
                  <div class="font-semibold text-slate-900 truncate">{{ p.nama_prodi }}</div>
                  <div class="text-[10px] text-muted-foreground truncate">{{ p.nama_ptn }}</div>
                </button>
              </div>
            </div>
          </template>
        </div>
      </div>
    </Card>

    <!-- Jurusan yang Diminati — real, diisi sendiri oleh siswa (isSelf saja), OPSIONAL.
         Dipakai sebagai basis pencarian "Alternatif Sesuai Minat Jurusan" di Resume (step 4,
         dilihat PIC sekolah lewat wizard) — tapi kalau kosong, otomatis fallback ke program
         studi yang sudah dipilih siswa di Pilihan 1/2, jadi field ini cuma perlu diisi kalau
         minat siswa berbeda dari program yang sudah dipilih. Sekolah tidak bisa mengedit
         field ini. -->
    <Card v-if="isSelf" class="p-5">
      <h4 class="font-bold text-slate-900 mb-1">Jurusan yang Diminati <span class="font-normal text-muted-foreground">(Opsional)</span></h4>
      <p class="text-xs text-muted-foreground mb-3">
        Opsional — kalau kosong, kami pakai program studi yang sudah kamu pilih di Pilihan 1/2 untuk mencarikan alternatif yang sesuai. Isi di sini kalau minatmu berbeda dari program yang sudah kamu pilih (misalnya sudah pilih Kedokteran tapi penasaran sama Farmasi juga).
      </p>
      <div class="flex gap-2">
        <Input v-model="minatJurusanInput" placeholder="Contoh: Kedokteran, Teknik Informatika, Psikologi... (opsional)" class="flex-1" />
        <Button size="sm" :disabled="savingMinatJurusan" @click="saveMinatJurusan">
          <Loader2 v-if="savingMinatJurusan" class="w-3.5 h-3.5 animate-spin" /> Simpan
        </Button>
      </div>
    </Card>

    <!-- Prestasi -->
    <Card v-if="!wizard || step === 2" class="p-5">
      <div class="flex items-center justify-between mb-1">
        <h4 class="font-bold text-slate-900 flex items-center gap-1.5"><Award class="w-4 h-4 text-amber-500" /> {{ isSelf ? 'Prestasi Saya' : 'Prestasi Siswa' }}</h4>
        <span class="text-xs font-semibold text-amber-600">Index Prestasi: {{ prestasiIndex }} / 10</span>
      </div>
      <p class="text-xs text-muted-foreground mb-4">Lomba/kompetisi nyata — menambah estimasi peluang jalur SNBP (maks. +10 poin).</p>

      <div class="grid grid-cols-2 md:grid-cols-5 gap-2 mb-4">
        <Input v-model="achievementForm.nama" placeholder="Nama lomba/prestasi" class="col-span-2 md:col-span-2" />
        <select v-model="achievementForm.tingkat" class="px-3 py-2 border rounded-lg text-sm bg-white">
          <option v-for="(label, key) in LEVEL_LABEL" :key="key" :value="key">{{ label }}</option>
        </select>
        <Input v-model.number="achievementForm.tahun" type="number" placeholder="Tahun" />
        <Input v-model="achievementForm.juara" placeholder="Juara (opsional)" />
        <Button size="sm" class="col-span-2 md:col-span-1" :disabled="savingAchievement" @click="saveAchievement">
          <Loader2 v-if="savingAchievement" class="w-3.5 h-3.5 animate-spin" /> Tambah
        </Button>
      </div>

      <div v-if="achievementsLoading" class="text-sm text-muted-foreground py-4 text-center">Memuat prestasi...</div>
      <div v-else-if="achievements.length === 0" class="rounded-2xl border border-dashed py-10 text-center">
        <Award class="w-8 h-8 mx-auto mb-2 text-slate-300" />
        <p class="text-sm font-semibold text-slate-500">Belum ada prestasi</p>
        <p class="text-xs text-muted-foreground">Prestasi akan meningkatkan Index Prestasi</p>
      </div>
      <div v-else class="space-y-2">
        <div v-for="a in achievements" :key="a.id" class="flex items-center justify-between gap-3 p-2.5 rounded-lg border">
          <div class="min-w-0">
            <div class="text-sm font-semibold text-slate-900 truncate">{{ a.nama }}</div>
            <div class="text-xs text-muted-foreground">{{ LEVEL_LABEL[a.tingkat] }} · {{ a.tahun }}<span v-if="a.juara"> · {{ a.juara }}</span> · +{{ a.bonus }} poin</div>
            <a v-if="a.certificate_url" :href="resolveUploadUrl(a.certificate_url) || '#'" target="_blank" rel="noopener" class="text-[11px] text-indigo-600 hover:underline inline-flex items-center gap-1 mt-0.5">
              <Paperclip class="w-3 h-3" /> Lihat sertifikat
            </a>
          </div>
          <div class="flex items-center gap-1 shrink-0">
            <template v-if="isSelf">
              <input :ref="(el) => setCertificateInputRef(a.id, el)" type="file" accept="image/png,image/jpeg,image/webp,application/pdf" class="hidden" @change="onCertificateSelected(a, $event)" />
              <button
                class="text-muted-foreground hover:text-indigo-600" :title="a.certificate_url ? 'Ganti sertifikat' : 'Unggah sertifikat'"
                :disabled="uploadingCertificateId === a.id" @click="pickCertificate(a.id)"
              >
                <Loader2 v-if="uploadingCertificateId === a.id" class="w-3.5 h-3.5 animate-spin" /><Paperclip v-else class="w-3.5 h-3.5" />
              </button>
            </template>
            <button class="text-muted-foreground hover:text-indigo-600" title="Edit" @click="editAchievement(a)"><Pencil class="w-3.5 h-3.5" /></button>
            <button class="text-muted-foreground hover:text-red-600" title="Hapus" @click="removeAchievement(a.id)"><Trash2 class="w-3.5 h-3.5" /></button>
          </div>
        </div>
      </div>
    </Card>

    <!-- Nilai Rapor: grid semua mapel per semester (tab S1-S5), batch-save. -->
    <Card v-if="!wizard || step === 3" class="p-5">
      <div class="flex items-start justify-between gap-3 mb-1 flex-wrap">
        <h4 class="font-bold text-slate-900">{{ isSelf ? 'Nilai Rapor Saya' : 'Nilai Rapor Siswa' }}</h4>
        <div class="flex items-center gap-1.5 shrink-0">
          <input ref="raporImportInputRef" type="file" accept=".xlsx,.xls" class="hidden" @change="onRaporImportFileChange" />
          <button
            type="button" class="text-[11px] font-semibold text-slate-500 hover:underline"
            @click="downloadRaporImportTemplate(undefined, { includeNis: false })"
          >Unduh Template</button>
          <Button size="sm" variant="outline" :disabled="raporImportLoading" @click="openRaporImportPicker">
            <Loader2 v-if="raporImportLoading" class="w-3.5 h-3.5 animate-spin" /><Paperclip v-else class="w-3.5 h-3.5" /> Import Excel
          </Button>
        </div>
      </div>
      <p class="text-xs text-muted-foreground mb-4">Dipakai untuk menghitung estimasi peluang jalur SNBP. Rata-rata saat ini: <strong>{{ raporAverage ?? '-' }}</strong></p>

      <div class="flex items-center gap-1.5 mb-4 overflow-x-auto pb-1">
        <button
          v-for="s in [1, 2, 3, 4, 5]" :key="s"
          class="px-4 py-1.5 rounded-full text-xs font-bold shrink-0 transition-colors"
          :class="raporSemesterTab === s ? 'bg-indigo-600 text-white' : 'bg-slate-100 text-slate-600 hover:bg-slate-200'"
          @click="raporSemesterTab = s"
        >S{{ s }}</button>
      </div>

      <Input v-model="raporSubjectQuery" placeholder="Cari mapel..." class="h-8 text-xs mb-2.5" />
      <div class="grid sm:grid-cols-2 gap-2.5 mb-4 max-h-96 overflow-y-auto pr-1">
        <div v-for="subj in filteredSubjects" :key="subj" class="flex items-center gap-2 p-2.5 rounded-lg border">
          <span class="flex-1 text-xs truncate">{{ subj }}</span>
          <Input v-model.number="raporGrid[subj].score" type="number" min="0" max="100" placeholder="-" class="w-16 h-8 text-xs" />
          <button
            type="button"
            class="w-8 h-5 rounded-full transition-colors relative shrink-0"
            :class="raporGrid[subj].is_minat ? 'bg-emerald-500' : 'bg-slate-200'"
            title="Tandai sebagai mapel minat"
            @click="raporGrid[subj].is_minat = !raporGrid[subj].is_minat"
          >
            <span class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all" :class="raporGrid[subj].is_minat ? 'left-3.5' : 'left-0.5'" />
          </button>
        </div>
        <p v-if="filteredSubjects.length === 0" class="col-span-2 text-center text-xs text-muted-foreground py-4">Tidak ada mapel yang cocok dengan "{{ raporSubjectQuery }}".</p>
      </div>
      <Button size="sm" :disabled="savingRaporGrid" @click="saveRaporGrid">
        <Loader2 v-if="savingRaporGrid" class="w-3.5 h-3.5 animate-spin" /> Simpan Nilai Semester {{ raporSemesterTab }}
      </Button>

      <div class="mt-5 pt-4 border-t">
        <div class="flex items-center justify-between mb-1">
          <p class="text-xs font-semibold text-slate-500 uppercase tracking-wide">Riwayat Nilai Tersimpan</p>
          <button
            v-if="raporGroupedBySemester.length > 1"
            type="button" class="text-[11px] font-semibold text-indigo-600 hover:underline shrink-0"
            @click="showAllSemesters = !showAllSemesters"
          >{{ showAllSemesters ? 'Tampilkan S' + raporSemesterTab + ' saja' : `Tampilkan semua semester (${raporGroupedBySemester.length})` }}</button>
        </div>
        <p class="text-[10px] text-muted-foreground -mt-1 mb-3">Satu mapel hanya ada 1 baris per semester (mengisi ulang mapel &amp; semester yang sama akan MENIMPA nilai lama, bukan menambah baris baru). {{ showAllSemesters ? '' : `Menampilkan Semester ${raporSemesterTab} saja — pilih tab semester lain di atas atau klik "Tampilkan semua semester" untuk lihat semuanya.` }}</p>
        <div v-if="raporLoading" class="text-sm text-muted-foreground py-4 text-center">Memuat nilai rapor...</div>
        <div v-else-if="rapor.length === 0" class="text-sm text-muted-foreground py-4 text-center">Belum ada nilai rapor tersimpan.</div>
        <div v-else-if="visibleRaporGroups.length === 0" class="text-sm text-muted-foreground py-4 text-center">Belum ada nilai tersimpan untuk Semester {{ raporSemesterTab }}.</div>
        <div v-else class="space-y-3" :class="showAllSemesters ? 'max-h-80 overflow-y-auto pr-1' : ''">
          <div v-for="group in visibleRaporGroups" :key="group.semester">
            <p v-if="showAllSemesters" class="text-[11px] font-bold text-slate-500 mb-1 sticky top-0 bg-white">Semester {{ group.semester }}</p>
            <div class="grid sm:grid-cols-2 gap-x-4">
              <div v-for="r in group.entries" :key="r.id" class="flex items-center justify-between gap-2 py-1.5 border-b text-sm">
                <span class="truncate">{{ r.subject }} <Badge v-if="r.is_minat" variant="secondary" class="ml-1 text-[10px]">Minat</Badge></span>
                <span class="flex items-center gap-2 shrink-0">
                  <span class="font-semibold">{{ r.score }}</span>
                  <button class="text-muted-foreground hover:text-red-600" title="Hapus" @click="removeRapor(r.id)"><Trash2 class="w-3.5 h-3.5" /></button>
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Card>

    <!-- Step 4 (wizard): Resume — ringkasan + alternatif nyata dari rumpun yang sama. -->
    <Card v-if="wizard && step === 4" class="p-5">
      <div class="rounded-xl bg-indigo-50 border border-indigo-100 p-4 mb-5 flex items-center gap-3">
        <CheckCircle2 class="w-6 h-6 text-indigo-600 shrink-0" />
        <div class="text-sm">
          <p class="font-bold text-indigo-900">Rasionalisasi siap!</p>
          <p class="text-indigo-700">Rata-rata rapor: <strong>{{ raporAverage ?? 0 }}</strong> · Prestasi: <strong>{{ achievements.length }} entri</strong></p>
        </div>
      </div>

      <h5 class="font-bold text-slate-900 mb-3">Resume Peluang</h5>
      <table class="w-full text-sm mb-5">
        <thead class="text-xs uppercase text-slate-400">
          <tr><th class="text-left py-1.5">Jurusan</th><th class="text-left py-1.5">Min. SNBP</th><th class="text-left py-1.5">Rapor Saya</th><th class="text-left py-1.5">Peluang</th></tr>
        </thead>
        <tbody class="divide-y">
          <tr v-if="pilihanUtama">
            <td class="py-2"><div class="font-semibold text-slate-900">Pil. 1 — {{ pilihanUtama.program.nama_ptn }}</div><div class="text-xs text-muted-foreground">{{ pilihanUtama.program.nama_prodi }}</div></td>
            <td class="py-2">{{ pilihanUtama.program.pg_snbp }}</td>
            <td class="py-2">{{ raporAverage ?? '-' }}</td>
            <td class="py-2">
              <Badge v-if="!pilihanUtama.program.has_official_stats" class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
              <Badge v-else :variant="TIER_VARIANT[pilihanUtama.chance.tier]">{{ pilihanUtama.chance.chance_percent }}%</Badge>
            </td>
          </tr>
          <tr v-if="pilihanCadangan">
            <td class="py-2"><div class="font-semibold text-slate-900">Pil. 2 — {{ pilihanCadangan.program.nama_ptn }}</div><div class="text-xs text-muted-foreground">{{ pilihanCadangan.program.nama_prodi }}</div></td>
            <td class="py-2">{{ pilihanCadangan.program.pg_snbp }}</td>
            <td class="py-2">{{ raporAverage ?? '-' }}</td>
            <td class="py-2">
              <Badge v-if="!pilihanCadangan.program.has_official_stats" class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
              <Badge v-else :variant="TIER_VARIANT[pilihanCadangan.chance.tier]">{{ pilihanCadangan.chance.chance_percent }}%</Badge>
            </td>
          </tr>
          <tr v-if="!pilihanUtama && !pilihanCadangan"><td colspan="4" class="py-6 text-center text-muted-foreground">Belum ada Pilihan Universitas 1/2. Kembali ke langkah "Data Siswa".</td></tr>
        </tbody>
      </table>

      <template v-if="radarAnchor">
        <h5 class="font-bold text-slate-900 mb-1">Pencapaian vs Referensi PG per Mata Pelajaran</h5>
        <p class="text-[11px] text-muted-foreground mb-3">Sumbu = mata pelajaran syarat {{ radarAnchor.program.nama_prodi }} (katalog nyata). Titik biru = rata-rata rapor asli siswa per mapel tsb. Garis oranye = PG {{ radarAnchor.program.nama_ptn }} secara keseluruhan (bukan minimum per mapel — data itu tidak tersedia).</p>
        <PencapaianRadarChart :subjects="radarSubjects" :pencapaian="radarPencapaian" :referensi="radarReferensi" />
      </template>

      <template v-if="pilihanUtama || pilihanCadangan">
        <p class="text-xs font-bold uppercase tracking-wide text-amber-700 bg-amber-50 px-3 py-1.5 rounded-lg mb-2 mt-5 inline-block">Alternatif Sesuai Rumpun Program Studi</p>
        <div v-if="loadingResumeAlternatif" class="text-xs text-muted-foreground py-3 text-center">Mencari alternatif...</div>
        <div v-else-if="resumeAlternatif.length === 0" class="text-xs text-muted-foreground py-3 text-center">Tidak ada alternatif lain di rumpun yang sama.</div>
        <table v-else class="w-full text-sm">
          <tbody class="divide-y">
            <tr v-for="alt in resumeAlternatif" :key="alt.program.id">
              <td class="py-2"><div class="font-semibold text-slate-900">{{ alt.program.nama_ptn }}</div><div class="text-xs text-muted-foreground">{{ alt.program.nama_prodi }}</div></td>
              <td class="py-2">{{ alt.program.pg_snbp }}</td>
              <td class="py-2">
                <Badge v-if="!alt.program.has_official_stats" class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
                <Badge v-else :variant="TIER_VARIANT[alt.tier]">{{ alt.chance_percent }}%</Badge>
              </td>
            </tr>
          </tbody>
        </table>
      </template>

      <template v-if="pilihanUtama || pilihanCadangan || minatJurusanForResume.trim()">
        <p class="text-xs font-bold uppercase tracking-wide text-sky-700 bg-sky-50 px-3 py-1.5 rounded-lg mb-2 mt-5 inline-block">Alternatif Sesuai Minat Jurusan</p>
        <div v-if="!effectiveMinatForResume.trim()" class="text-xs text-muted-foreground py-3 text-center">Isi "Jurusan yang Diminati" di atas atau pilih Universitas 1/2 dulu untuk melihat alternatif ini.</div>
        <template v-else>
          <p v-if="usingMinatFallback" class="text-[11px] text-sky-700 bg-sky-50/60 border border-sky-100 rounded-lg px-3 py-2 mb-2">
            Menampilkan alternatif berdasarkan program yang kamu pilih ({{ effectiveMinatForResume }}). Isi "Jurusan yang Diminati" di atas kalau minatmu berbeda.
          </p>
          <div v-if="loadingResumeAlternatifMinat" class="text-xs text-muted-foreground py-3 text-center">Mencari alternatif...</div>
          <div v-else-if="resumeAlternatifMinat.length === 0" class="text-xs text-muted-foreground py-3 text-center">Tidak ada alternatif lain untuk minat jurusan ini.</div>
          <table v-else class="w-full text-sm">
            <tbody class="divide-y">
              <tr v-for="alt in resumeAlternatifMinat" :key="alt.program.id">
                <td class="py-2"><div class="font-semibold text-slate-900">{{ alt.program.nama_ptn }}</div><div class="text-xs text-muted-foreground">{{ alt.program.nama_prodi }}</div></td>
                <td class="py-2">{{ alt.program.pg_snbp }}</td>
                <td class="py-2">
                <Badge v-if="!alt.program.has_official_stats" class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
                <Badge v-else :variant="TIER_VARIANT[alt.tier]">{{ alt.chance_percent }}%</Badge>
              </td>
              </tr>
            </tbody>
          </table>
        </template>
      </template>

      <p class="text-[10px] text-muted-foreground mt-4">Semua data di atas sudah tersimpan otomatis di setiap langkah — tidak perlu disimpan ulang. Kamu bisa kembali ke langkah manapun lewat stepper di atas untuk merevisi.</p>
    </Card>

    <!-- Cari program studi (mode flat saja — mode wizard pakai slot Pilihan Universitas 1/2 di step "Data Siswa") -->
    <Card v-if="!wizard" id="cari-program-snbp" class="p-5 scroll-mt-24">
      <h4 class="font-bold text-slate-900 mb-3">Cari Program Studi</h4>
      <div class="relative mb-3">
        <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
        <Input v-model="searchQuery" placeholder="Cari nama PTN atau program studi (min. 3 huruf)..." class="pl-9" />
      </div>
      <div v-if="searching" class="text-sm text-muted-foreground py-3 text-center">Mencari...</div>
      <div v-else-if="searchQuery.trim().length >= 3 && searchResults.length === 0" class="text-sm text-muted-foreground py-3 text-center">Tidak ditemukan.</div>
      <div v-else-if="searchQuery.trim().length < 3" class="text-sm text-muted-foreground py-3 text-center">Ketik minimal 3 huruf untuk mencari PTN atau program studi.</div>
      <div v-else class="space-y-1.5 max-h-80 overflow-y-auto">
        <div v-for="p in searchResults" :key="p.id" class="flex items-center justify-between gap-3 p-2.5 rounded-lg border hover:bg-slate-50">
          <div class="min-w-0">
            <div class="text-sm font-semibold text-slate-900 truncate">{{ p.nama_prodi }}</div>
            <div class="text-xs text-muted-foreground truncate">{{ p.nama_ptn }} · {{ p.kota }} · {{ p.jenjang }}</div>
          </div>
          <Button size="sm" variant="outline" @click="openPicker(p)"><Plus class="w-3.5 h-3.5" /> Tambah</Button>
        </div>
      </div>
    </Card>

    <!-- Target PTN (jalur SNBP) — mode flat saja. -->
    <Card v-if="!wizard" class="p-5">
      <div class="flex items-center justify-between mb-3">
        <h4 class="font-bold text-slate-900">{{ isSelf ? 'Target PTN Saya' : 'Target PTN Siswa' }} — Jalur SNBP</h4>
        <span class="text-xs text-muted-foreground flex items-center gap-1"><Info class="w-3 h-3" /> Estimasi, bukan jaminan</span>
      </div>

      <div v-if="targetsLoading" class="text-sm text-muted-foreground py-6 text-center">Memuat target...</div>
      <div v-else-if="snbpTargets.length === 0" class="text-sm text-muted-foreground py-6 text-center">
        Belum ada target jalur SNBP. Cari program studi di atas lalu tambahkan.
      </div>
      <div v-else class="space-y-3">
        <div
          v-for="t in snbpTargets" :key="t.id"
          class="p-4 rounded-xl border-2 flex flex-col sm:flex-row sm:items-center justify-between gap-3"
          :class="!t.program.has_official_stats ? 'border-slate-200 bg-slate-50/50' : t.chance.tier === 'aman' ? 'border-emerald-200 bg-emerald-50/50' : t.chance.tier === 'moderat' ? 'border-amber-200 bg-amber-50/50' : 'border-red-200 bg-red-50/50'"
        >
          <div class="min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="font-semibold text-slate-900">{{ t.program.nama_prodi }}</span>
              <Badge v-if="!t.program.has_official_stats" class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
              <Badge v-else :variant="TIER_VARIANT[t.chance.tier]">{{ TIER_LABEL[t.chance.tier] }}</Badge>
            </div>
            <div class="text-xs text-muted-foreground">{{ t.program.nama_ptn }} · {{ t.program.kota }}</div>
          </div>

          <div class="flex items-center gap-3 shrink-0">
            <div v-if="!t.program.has_official_stats" class="text-right max-w-[7rem]">
              <div class="text-[10px] text-muted-foreground leading-snug">Belum ada data resmi — peluang belum bisa dihitung</div>
            </div>
            <div v-else class="text-right">
              <div class="text-xl font-black text-slate-900">{{ t.chance.chance_percent }}%</div>
              <div class="text-[10px] text-muted-foreground">peluang</div>
            </div>
            <select
              :value="t.priority" class="text-xs border rounded-lg px-2 py-1.5 bg-white"
              @change="changePriority(t, ($event.target as HTMLSelectElement).value as PtnPriority)"
            >
              <option value="utama">Utama</option>
              <option value="cadangan">Cadangan</option>
              <option value="aman">Aman</option>
            </select>
            <button class="text-muted-foreground hover:text-red-600" @click="removeTargetItem(t)"><Trash2 class="w-4 h-4" /></button>
          </div>
        </div>
      </div>
    </Card>

    <!-- Navigasi wizard -->
    <div v-if="wizard" class="flex items-center justify-between">
      <Button variant="outline" :disabled="step === 1" @click="step--"><ChevronLeft class="w-4 h-4" /> Kembali</Button>
      <Button v-if="step < 4" variant="gradient" @click="step++">Lanjut <ChevronRight class="w-4 h-4" /></Button>
      <Button v-else variant="gradient" @click="emit('changed'); emit('close')"><CheckCircle2 class="w-4 h-4" /> Selesai &amp; Simpan</Button>
    </div>

    <!-- Target picker modal -->
    <div v-if="pickerProgram" class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4" @click.self="closePicker">
      <Card class="w-full max-w-sm p-6 relative">
        <button class="absolute top-4 right-4 text-muted-foreground hover:text-foreground" @click="closePicker"><X class="w-4 h-4" /></button>
        <h3 class="font-bold text-slate-900 mb-1">{{ pickerProgram.nama_prodi }}</h3>
        <p class="text-xs text-muted-foreground mb-4">{{ pickerProgram.nama_ptn }} · Jalur SNBP</p>

        <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Prioritas</label>
        <select v-model="pickerPriority" class="w-full border rounded-lg px-3 py-2 text-sm mb-4">
          <option value="utama">Utama</option>
          <option value="cadangan">Cadangan</option>
          <option value="aman">Aman</option>
        </select>

        <div class="rounded-xl bg-slate-50 p-3 mb-4 text-center">
          <div v-if="previewLoading" class="text-xs text-muted-foreground py-2">Menghitung estimasi...</div>
          <template v-else-if="pickerProgram && !pickerProgram.has_official_stats">
            <Badge class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
            <div class="text-xs text-muted-foreground mt-1">Belum ada angka daya tampung/peminat resmi — estimasi peluang belum bisa dihitung.</div>
          </template>
          <template v-else-if="pickerPreview">
            <div class="text-2xl font-black" :class="pickerPreview.tier === 'aman' ? 'text-emerald-600' : pickerPreview.tier === 'moderat' ? 'text-amber-600' : 'text-red-600'">
              {{ pickerPreview.chance_percent }}%
            </div>
            <div class="text-xs text-muted-foreground">Estimasi peluang · {{ TIER_LABEL[pickerPreview.tier] }}</div>
          </template>
        </div>

        <Button class="w-full" :disabled="adding" @click="confirmAddTarget">
          <Loader2 v-if="adding" class="w-4 h-4 animate-spin" /> Tambahkan sebagai Target
        </Button>
      </Card>
    </div>

    <!-- Preview import Excel nilai rapor (1 siswa) sebelum dikonfirmasi -->
    <Teleport to="body">
      <div v-if="raporImportOpen" class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" @click="raporImportOpen = false">
        <Card class="w-full max-w-lg shadow-2xl my-8 max-h-[85vh] flex flex-col" @click.stop>
          <div class="flex items-center justify-between px-6 py-4 border-b shrink-0">
            <h3 class="font-bold text-slate-900">Preview Import Nilai Rapor</h3>
            <button class="p-1.5 rounded-lg hover:bg-slate-100" @click="raporImportOpen = false"><X class="w-4 h-4" /></button>
          </div>
          <div class="px-6 py-4 overflow-y-auto space-y-3">
            <p class="text-sm">
              <strong class="text-emerald-600">{{ raporImportTotalEntries }} nilai</strong> siap diimpor dari
              <strong>{{ raporImportValidRows.length }}</strong> baris valid.
              <span v-if="raporImportErrorRows.length" class="text-red-600">{{ raporImportErrorRows.length }} baris dilewati karena error.</span>
            </p>
            <div v-if="raporImportErrorRows.length" class="rounded-lg border border-red-100 bg-red-50 p-3 space-y-1 max-h-32 overflow-y-auto">
              <p v-for="r in raporImportErrorRows" :key="r.rowNumber" class="text-xs text-red-700">Baris {{ r.rowNumber }}: {{ r.rowError }}</p>
            </div>
            <div v-if="raporImportRows.some((r) => r.cellWarnings.length)" class="rounded-lg border border-amber-100 bg-amber-50 p-3 space-y-1 max-h-32 overflow-y-auto">
              <template v-for="r in raporImportRows" :key="'w' + r.rowNumber">
                <p v-for="(w, i) in r.cellWarnings" :key="i" class="text-xs text-amber-700">Baris {{ r.rowNumber }}: {{ w }}</p>
              </template>
            </div>
            <div class="rounded-lg border divide-y max-h-64 overflow-y-auto">
              <div v-for="r in raporImportValidRows" :key="r.rowNumber" class="px-3 py-2 text-xs">
                <span class="font-semibold">Semester {{ r.semester }}</span> — {{ r.entries.length }} mapel:
                <span class="text-muted-foreground">{{ r.entries.map((e) => `${e.subject} (${e.score})`).join(', ') }}</span>
              </div>
              <p v-if="raporImportValidRows.length === 0" class="px-3 py-4 text-center text-xs text-muted-foreground">Tidak ada baris valid untuk diimpor.</p>
            </div>
          </div>
          <div class="px-6 py-4 border-t flex gap-2 shrink-0">
            <Button variant="outline" class="flex-1" @click="raporImportOpen = false">Batal</Button>
            <Button variant="gradient" class="flex-1 gap-1.5" :disabled="raporImportSaving || raporImportTotalEntries === 0" @click="confirmRaporImport">
              <Loader2 v-if="raporImportSaving" class="w-4 h-4 animate-spin" /> Impor {{ raporImportTotalEntries }} Nilai
            </Button>
          </div>
        </Card>
      </div>
    </Teleport>
  </div>
</template>
