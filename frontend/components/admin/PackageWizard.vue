<script setup lang="ts">
// "Buat Paket + Subtes + Soal" — alur penuh dari nol sampai paket siap (atau siap direview):
// (1) info dasar paket, (2) tambah subtes (Simulasi UTBK yang dirangkai berurutan, atau
// Drilling berdiri sendiri), (3) isi soal tiap subtes, (4) ringkasan & publish.
//
// SENGAJA BUKAN modal — Isi Soal butuh ruang sungguhan (form soal lengkap 9 tipe, daftar soal
// yang sudah masuk, cari dari Bank Soal, import .docx), jadi komponen ini dirender penuh oleh
// parent (PackageManager.vue / ContentPackagesPanel.vue) menggantikan tampilan daftar paket
// selama wizard aktif — bukan mengambang di atasnya. Prop `dark` bikin komponen ini otomatis
// pakai varian gelap yang sama seperti QuestionBankManager dkk, supaya bisa dipakai identik di
// Admin (terang) maupun akun Konten (gelap).
//
// Menulis soal baru memakai QuestionFormPanel.vue APA ADANYA (form lengkap yang sama dipakai
// Bank Soal) lewat drawer sisi kanan — pola persis QuestionSetManager.vue "Tambah Soal Baru ke
// Set Ini" (lihat komentar di sana), supaya pengalaman menulis soal konsisten di seluruh
// aplikasi, bukan form mini terpisah. Import massal .docx memakai QuestionImportDialog.vue apa
// adanya juga, diarahkan ke Set Soal subtes yang aktif lewat prop `defaultQuestionSetId`.
//
// Setiap subtes = 1 TryoutSession + 1 QuestionSet khusus (`question_set_id`), dibuat dengan
// is_draft:true supaya tidak muncul di katalog siswa selagi masih disusun. Kalau ada subtes
// bertipe "Simulasi UTBK", semua subtes bertipe itu dirangkai jadi 1 SimulationTemplate
// (is_draft:true juga). Paket sendiri selalu dibuat active:false (draft) — publish adalah
// aksi terpisah & eksplisit (khusus Admin), yang men-cascade un-draft semua isinya sekaligus
// (lihat PackageService::set_active backend).
import {
  ChevronRight, ChevronLeft, Plus, Trash2, Search, FileText, ListChecks, AlertTriangle,
  CheckCircle2, Loader2, Sparkles, Rocket, Save, PenLine, Upload, X, ArrowLeft, Eye, ShieldCheck,
} from 'lucide-vue-next'
import type { Question, SimulationTemplatePayload, ExamTrack, SchoolType } from '~/types'

// `goToSession`/`goToSet` — deep-link ke Lanjutan > Sesi / Set Soal untuk edit lanjutan yang
// tidak tercakup wizard ini (mis. topic_filter/difficulty_filter sesi, atau menamai ulang Set
// Soal) — lihat ContentHub.vue yang menangkap event ini dan membuka sub-tab yang sesuai.
const emit = defineEmits<{ close: []; saved: []; goToSession: [sessionId: string]; goToSet: [setId: string] }>()
// `packageId` non-null = mode kelola paket yang SUDAH ada (bukan bikin baru dari nol) —
// dipicu dengan mengklik kartu paket di ContentPackagesPanel.vue / PackageManager.vue.
// Subtes-nya direkonstruksi dari PackageContentItem yang sudah tersimpan, step 1 (info
// paket) dilewati karena paketnya sudah ada, dan alur lanjut sebagai "tambah subtes baru
// / kelola soal" bukan "buat paket baru".
const props = defineProps<{ dark?: boolean; packageId?: string | null }>()
const isEditMode = computed(() => !!props.packageId)
// Local computed so the template can reference a plain `dark` binding — `<script setup>` only
// auto-exposes the `props` object itself to the template, not its individual keys.
const dark = computed(() => props.dark ?? false)

const { packageService, tryoutService, simulationService, questionSetService, questionService } = useApi()
const { user } = useAuth()
const toast = useToast()

const isContent = computed(() => user.value?.role === 'content')
const canPublish = computed(() => user.value?.role === 'admin')

// ─── Step state ─────────────────────────────────────────────────────────────────
const step = ref(1) // 1: info paket, 2: subtes, 3: isi soal, 4: ringkasan
const saving = ref(false)
const packageId = ref<string | null>(null)

// Guardrail: admin harus SADAR memilih Jalur Ujian, bukan diam-diam kepakai default 'snbt' di
// bawah (yang dulu jadi sumber bug "TKA SMP #2" ke-tag exam_track='snbt') — dua tombol di Step 1
// baru tampil "terpilih" begitu salah satunya benar-benar diklik, dan createPackageShell() menolak
// lanjut sebelum flag ini true. Reset ke false di resetAll() (bikin paket baru), tapi langsung true
// di loadExisting() (paket lama sudah punya exam_track pasti, tidak perlu dipaksa klik ulang).
const examTrackChosen = ref(false)

const pkgForm = ref({
  name: '',
  package_type: 'SNBT',
  original_price: 0,
  sale_price: 0,
  validity: '1 tahun',
  // 0 = tidak ada batasan mapel pilihan. Isi > 0 untuk paket TKA yang punya mapel pilihan
  // (mis. TKA SMA: 2) — lihat checkbox "Mapel Pilihan" per subtes di Step 2 dan backend
  // ElectiveService doc comment. TIDAK dipakai lagi untuk menentukan exam_track (lihat field
  // exam_track di bawah) — sebelumnya exam_track diturunkan dari elective_pick_count > 0, yang
  // salah untuk TKA SMP (semua wajib, elective_pick_count = 0) sehingga paket & sesinya
  // ke-tag 'snbt' padahal isinya TKA. Itu bikin siswa SMP lihat konten SNBT/UTBK tercampur di
  // tab yang sama, dan katalog gagal membedakan jalur ujian dengan benar.
  elective_pick_count: 0,
  // Jalur ujian dipilih EKSPLISIT oleh admin di Step 1, bukan lagi ditebak dari field lain.
  exam_track: 'snbt' as ExamTrack,
  // `null` = subtes/simulasi yang dibuat lewat wizard ini tampil ke siswa jenjang apapun
  // (default, cocok untuk konten umum SNBT/UTBK). Diisi kalau paket ini memang khusus satu
  // jenjang (mis. "TKA SMP") — dikirim ke TryoutSession/SimulationTemplate, BUKAN field di
  // Package itu sendiri, lihat migration 20250101000039_content_school_type_scope.sql.
  school_type_scope: null as SchoolType | null,
  // "Mode Terkunci" (focus/lockdown) untuk Simulasi TO yang dihasilkan wizard ini — `null` =
  // ikuti default global platform (lihat SimulationManager.vue "Mode Terkunci — Simulasi TO").
  // Berlaku untuk simulasi SNBT maupun TKA yang dirakit lewat wizard ini (assembleSimulation /
  // assembleTkaSimulation), bukan field di Package itu sendiri.
  lockdown_override: null as boolean | null,
})
// Mapel pilihan cuma masuk akal untuk jalur TKA — kalau admin isi elective_pick_count > 0
// tapi Jalur Ujian masih SNBT/UTBK, paksa ke 'tka' otomatis supaya paket/sesi/simulasi yang
// dihasilkan tidak pernah mismatch (mis. sesi ke-tag 'snbt' tapi template Hari1/Hari2-nya
// 'tka', lihat assembleTkaSimulation). Tidak reversible arahnya (matikan elective TIDAK
// mengubah exam_track balik ke 'snbt') karena admin boleh saja punya paket TKA tanpa pilihan.
watch(() => pkgForm.value.elective_pick_count, (count) => {
  if (count > 0) pkgForm.value.exam_track = 'tka'
})
const discountPreview = computed(() => {
  if (pkgForm.value.original_price <= 0) return 0
  return Math.round((1 - pkgForm.value.sale_price / pkgForm.value.original_price) * 100)
})

type SubtesKind = 'simulasi' | 'drilling'
interface SubtesDraft {
  key: string
  kind: SubtesKind
  title: string
  duration_minutes: number
  subject_filter: string
  is_elective: boolean
  sessionId: string | null
  setId: string | null
  creating: boolean
  soal: Question[]
  soalLoading: boolean
  pick: { open: boolean; search: string; loading: boolean; results: Question[] }
}

function blankPick() {
  return { open: false, search: '', loading: false, results: [] as Question[] }
}

const subtests = ref<SubtesDraft[]>([])
let keySeq = 0
function addSubtes(kind: SubtesKind) {
  subtests.value.push({
    key: `s${++keySeq}`,
    kind,
    title: '',
    duration_minutes: kind === 'simulasi' ? 60 : 20,
    subject_filter: '',
    is_elective: false,
    sessionId: null,
    setId: null,
    creating: false,
    soal: [],
    soalLoading: false,
    pick: blankPick(),
  })
}
function removeSubtes(idx: number) {
  subtests.value.splice(idx, 1)
}

function resetAll() {
  step.value = 1
  packageId.value = null
  pkgForm.value = { name: '', package_type: 'SNBT', original_price: 0, sale_price: 0, validity: '1 tahun', elective_pick_count: 0, exam_track: 'snbt', school_type_scope: null, lockdown_override: null }
  examTrackChosen.value = false
  subtests.value = []
  activeKey.value = null
  existingTemplateId.value = null
  existingPilihanTemplateId.value = null
  pkgActive.value = false
}

// ─── Mode edit: rekonstruksi subtes dari paket yang sudah ada ─────────────────
const existingTemplateId = ref<string | null>(null)
// Template "Hari 2 (Pilihan)" TKA — dipisah dari existingTemplateId di atas karena slotnya
// tidak punya session_id tetap (is_elective:true), jadi tidak direkonstruksi jadi subtes biasa
// seperti template lain. Lihat assembleTkaSimulation() di bawah.
const existingPilihanTemplateId = ref<string | null>(null)
const pkgActive = ref(false)
const loadingExisting = ref(false)

async function loadExisting(id: string) {
  loadingExisting.value = true
  try {
    const pkg = await packageService.get(id)
    pkgForm.value = {
      name: pkg.name, package_type: pkg.package_type, original_price: pkg.original_price,
      sale_price: pkg.sale_price, validity: pkg.validity, elective_pick_count: pkg.elective_pick_count ?? 0,
      exam_track: pkg.exam_track ?? 'snbt',
      // Package tidak punya field ini sendiri (per-simulasi, bukan per-paket) — dibiarkan
      // "ikuti default global" di sini; kalau simulasi untuk paket ini sudah pernah dibuat dan
      // admin ingin override per-template, atur lewat tab Simulasi UTBK.
      lockdown_override: null,
      // Placeholder — Package sendiri tidak punya field ini (lihat catatan di deklarasi
      // pkgForm di atas). Diisi dari subtes pertama yang punya nilai, di bawah, setelah
      // rekonstruksi selesai.
      school_type_scope: null,
    }
    pkgActive.value = pkg.active
    packageId.value = pkg.id
    // Paket lama sudah punya exam_track pasti (dipilih saat dibuat) — tidak perlu paksa admin
    // klik ulang toggle-nya sebelum bisa lanjut mengelola paket ini.
    examTrackChosen.value = true

    const items = await packageService.listContent(id)
    const rebuilt: SubtesDraft[] = []
    const claimedSessionIds = new Set<string>()
    let detectedScope: SchoolType | null = null

    for (const item of items) {
      if (item.content_type !== 'simulation_template') continue
      const templates = await simulationService.listTemplates()
      const tpl = templates.find((t) => t.id === item.content_id)
      if (!tpl) continue
      // Template "Hari 2 (Pilihan)" TKA — slotnya is_elective:true, session_id selalu null di
      // level template (baru di-resolve per-siswa saat simulasi dimulai). Tidak ada subtes tetap
      // untuk direkonstruksi dari sini; sesi pilihannya sendiri sudah muncul lewat loop kedua di
      // bawah (content_type === 'tryout_session').
      if (tpl.slots.some((s) => s.is_elective)) {
        existingPilihanTemplateId.value = tpl.id
        continue
      }
      existingTemplateId.value = tpl.id
      for (const slot of tpl.slots) {
        if (!slot.session_id) continue // defensif — seharusnya selalu ada di template non-elective
        claimedSessionIds.add(slot.session_id)
        const session = await tryoutService.getSession(slot.session_id).catch(() => null)
        if (session?.school_type_scope) detectedScope = session.school_type_scope
        rebuilt.push({
          key: `s${++keySeq}`,
          kind: 'simulasi',
          title: slot.session_title || '',
          duration_minutes: slot.duration_minutes ?? 0,
          subject_filter: slot.subject_filter || '',
          is_elective: session?.is_elective ?? false,
          sessionId: slot.session_id,
          setId: session?.question_set_id ?? null,
          creating: false,
          soal: [],
          soalLoading: false,
          pick: blankPick(),
        })
      }
    }

    for (const item of items) {
      if (item.content_type !== 'tryout_session' || claimedSessionIds.has(item.content_id)) continue
      const session = await tryoutService.getSession(item.content_id).catch(() => null)
      if (!session) continue
      if (session.school_type_scope) detectedScope = session.school_type_scope
      rebuilt.push({
        key: `s${++keySeq}`,
        kind: session.session_type === 'tryout' ? 'simulasi' : 'drilling',
        title: session.title,
        duration_minutes: session.duration_minutes,
        subject_filter: session.subject_filter || '',
        is_elective: session.is_elective,
        sessionId: session.id,
        setId: session.question_set_id ?? null,
        creating: false,
        soal: [],
        soalLoading: false,
        pick: blankPick(),
      })
    }

    subtests.value = rebuilt
    pkgForm.value.school_type_scope = detectedScope
    simulationCreated.value = !!existingTemplateId.value || !!existingPilihanTemplateId.value
    for (const s of rebuilt) await loadSoal(s)
    activeKey.value = rebuilt[0]?.key ?? null
    step.value = 2
  } catch (e: any) {
    toast.error('Gagal memuat data paket', e?.message)
    emit('close')
  } finally {
    loadingExisting.value = false
  }
}

onMounted(() => {
  if (props.packageId) loadExisting(props.packageId)
  else resetAll()
})

// ─── Step 1 -> 2: create the draft package shell ───────────────────────────────
async function createPackageShell() {
  if (!pkgForm.value.name.trim()) { toast.error('Nama paket wajib diisi'); return }
  if (pkgForm.value.original_price <= 0) { toast.error('Harga asli harus lebih dari 0'); return }
  if (pkgForm.value.sale_price <= 0) { toast.error('Harga jual harus lebih dari 0'); return }
  // Guardrail — lihat catatan di deklarasi examTrackChosen di atas: cegah paket ke-tag 'snbt'
  // diam-diam cuma karena admin lupa menyentuh toggle Jalur Ujian.
  if (!examTrackChosen.value) { toast.error('Pilih Jalur Ujian dulu', 'Klik salah satu tombol "SNBT / UTBK" atau "TKA" sebelum lanjut.'); return }
  // Guardrail — paket TKA WAJIB punya jenjang spesifik (SMP, atau SMA/SMK/MA), tidak boleh
  // "Semua". Kalau dibiarkan kosong, tka_scale_for() di backend diam-diam jatuh ke skala
  // SMA/SMK/MA (200-800) — persis bug yang bikin TKA SMP salah skala sebelumnya.
  if (pkgForm.value.exam_track === 'tka' && !pkgForm.value.school_type_scope) {
    toast.error(
      'Pilih Jenjang untuk paket TKA',
      'Paket TKA harus punya jenjang spesifik (SMP, atau SMA/SMK/MA) supaya skala penilaian (0-100 vs 200-800) otomatis benar — tidak boleh dibiarkan "Semua".',
    )
    return
  }
  saving.value = true
  try {
    // Guard against membuat paket duplikat tanpa sadar — sumber nyata dari kasus "TKA SMP
    // #1/#3/#4" (3 paket beda tapi cuma 1 yang benar-benar diisi konten & diassign ke sekolah,
    // sisanya jadi paket kosong yang membingungkan). Nama sama persis (case-insensitive) kuat
    // indikasi ini percobaan ulang, bukan paket baru yang disengaja.
    const existing = await packageService.list().catch(() => [])
    const dup = existing.find((p) => p.name.trim().toLowerCase() === pkgForm.value.name.trim().toLowerCase())
    if (dup) {
      const proceed = window.confirm(
        `Sudah ada paket bernama "${dup.name}" (dibuat ${new Date(dup.created_at).toLocaleDateString('id-ID')}).\n\n` +
        `Kalau ini percobaan ulang, sebaiknya buka & lanjutkan paket yang sudah ada itu lewat "Kelola Paket" alih-alih bikin baru — supaya tidak ada paket kosong duplikat.\n\n` +
        `Tetap buat paket baru dengan nama yang sama?`,
      )
      if (!proceed) { saving.value = false; return }
    }
    const pkg = await packageService.create({
      name: pkgForm.value.name,
      package_type: pkgForm.value.package_type,
      original_price: pkgForm.value.original_price,
      sale_price: pkgForm.value.sale_price,
      validity: pkgForm.value.validity,
      features: [],
      badge: '',
      icon_type: 'preset',
      icon_name: 'package',
      icon_url: null,
      gradient: 'from-sky-200 to-blue-100',
      accent_color: '#3b82f6',
      active: false,
      sort_order: 0,
      elective_pick_count: pkgForm.value.elective_pick_count || 0,
      // Eksplisit dari pilihan admin di Step 1 (lihat radio "Jalur Ujian") — sebelumnya
      // diturunkan dari elective_pick_count > 0, yang salah untuk TKA SMP (semua wajib, jadi
      // elective_pick_count = 0) sehingga ke-tag 'snbt' padahal isinya TKA.
      exam_track: pkgForm.value.exam_track,
    })
    packageId.value = pkg.id
    if (subtests.value.length === 0) addSubtes('simulasi')
    step.value = 2
  } catch (e: any) {
    toast.error('Gagal membuat paket', e?.message)
  } finally {
    saving.value = false
  }
}

// ─── Step 2 -> 3: materialize each subtes as a QuestionSet + TryoutSession, attach to paket ──
async function materializeSubtests() {
  if (subtests.value.length === 0) { toast.error('Tambahkan minimal 1 subtes dulu'); return }
  for (const s of subtests.value) {
    if (!s.title.trim()) { toast.error('Judul subtes wajib diisi untuk semua subtes'); return }
  }
  saving.value = true
  try {
    for (const s of subtests.value) {
      if (s.sessionId) continue // sudah dibuat sebelumnya (mis. user maju-mundur antar step)
      s.creating = true
      const set = await questionSetService.create({ name: `${pkgForm.value.name} — ${s.title}` })
      s.setId = set.id
      const session = await tryoutService.createSession({
        title: s.title,
        session_type: s.kind === 'simulasi' ? 'tryout' : 'drilling',
        duration_minutes: s.duration_minutes,
        // Cosmetic only once question_set_id is set (session pulls the curated set's fixed
        // list, not a random draw) — backend skips its "must be positive" check in that case.
        question_count: 0,
        subject_filter: s.subject_filter || null,
        topic_filter: null,
        difficulty_filter: null,
        is_premium: true,
        question_set_id: set.id,
        is_draft: true,
        is_elective: s.is_elective,
        // Eksplisit dari pilihan admin di Step 1 — lihat catatan di pkgForm.exam_track di atas.
        exam_track: pkgForm.value.exam_track,
        school_type_scope: pkgForm.value.school_type_scope,
      })
      s.sessionId = session.id
      await packageService.addContentItem(packageId.value!, 'tryout_session', session.id)
      s.creating = false
    }
    activeKey.value = subtests.value[0]?.key ?? null
    step.value = 3
  } catch (e: any) {
    toast.error('Gagal menyiapkan subtes', e?.message)
  } finally {
    saving.value = false
  }
}

// ─── Step 3: isi soal per subtes ───────────────────────────────────────────────
const activeKey = ref<string | null>(null)
const activeSubtes = computed(() => subtests.value.find((s) => s.key === activeKey.value) || null)

async function loadSoal(s: SubtesDraft) {
  if (!s.setId) return
  s.soalLoading = true
  try {
    s.soal = await questionSetService.getItems(s.setId)
  } catch (e: any) {
    toast.error('Gagal memuat soal', e?.message)
  } finally {
    s.soalLoading = false
  }
}
watch(activeKey, () => { if (activeSubtes.value) loadSoal(activeSubtes.value) })

async function togglePick(s: SubtesDraft) {
  s.pick.open = !s.pick.open
  if (s.pick.open) await searchApproved(s)
}
async function searchApproved(s: SubtesDraft) {
  s.pick.loading = true
  try {
    const res = await questionService.list({ status: 'approved', search: s.pick.search || undefined, page: 1, page_size: 15 })
    s.pick.results = res.items
  } catch (e: any) {
    toast.error('Gagal mencari soal', e?.message)
  } finally {
    s.pick.loading = false
  }
}
async function attachExisting(s: SubtesDraft, q: Question) {
  if (!s.setId) return
  try {
    await questionSetService.addItem(s.setId, q.id)
    toast.success(`Soal "${q.code}" ditambahkan ke ${s.title}`)
    await loadSoal(s)
  } catch (e: any) {
    toast.error('Gagal menambahkan soal', e?.message)
  }
}
async function detachSoal(s: SubtesDraft, q: Question) {
  if (!s.setId) return
  try {
    await questionSetService.removeItem(s.setId, q.id)
    await loadSoal(s)
  } catch (e: any) {
    toast.error('Gagal menghapus soal dari subtes', e?.message)
  }
}

// "Tulis Soal Baru" — drawer sisi kanan berisi QuestionFormPanel.vue apa adanya (pola persis
// QuestionSetManager.vue), diarahkan ke Set Soal subtes yang sedang aktif.
const showAuthorPanel = ref(false)
function openAuthorPanel() {
  if (!activeSubtes.value) return
  showAuthorPanel.value = true
}
function closeAuthorPanel() {
  showAuthorPanel.value = false
  if (activeSubtes.value) loadSoal(activeSubtes.value)
}

// "Import Soal (.docx)" — QuestionImportDialog.vue apa adanya, diarahkan ke Set Soal subtes aktif.
const showImport = ref(false)
function onImported() {
  if (activeSubtes.value) loadSoal(activeSubtes.value)
}

// "Preview Set Soal" — replika PERSIS layar siswa saat mengerjakan (bukan dialog per-soal),
// dengan navigasi daftar soal di dalamnya sendiri — lihat QuestionSetPreviewDialog.vue.
const showSetPreview = ref(false)

async function goToSummary() {
  step.value = 4
}

// "Setujui Semua Soal" — soal yang ditulis lewat wizard ini (role Konten) berstatus draft
// sampai admin meninjaunya satu-satu di Bank Soal (lihat backend QuestionService::create).
// Untuk paket dengan banyak subtes ini jadi langkah manual yang melelahkan — tombol ini
// menyetujui sekaligus semua soal di Set Soal milik setiap subtes paket ini. Admin-only
// (backend menolak role lain); tetap dirender untuk Konten supaya jelas kenapa non-aktif.
const bulkApproving = ref(false)
const bulkApprovedCount = ref<number | null>(null)
async function bulkApproveAll() {
  if (!packageId.value) return
  bulkApproving.value = true
  try {
    const res = await packageService.bulkApproveContent(packageId.value)
    bulkApprovedCount.value = res.approved_count
    toast.success(res.approved_count > 0 ? `${res.approved_count} soal disetujui` : 'Semua soal sudah disetujui sebelumnya')
  } catch (e: any) {
    toast.error('Gagal menyetujui soal', e?.message)
  } finally {
    bulkApproving.value = false
  }
}

// ─── Step 4: rangkai Simulasi UTBK (kalau ada) + ringkasan + publish ───────────
const hasSimulasiSubtests = computed(() => subtests.value.some((s) => s.kind === 'simulasi'))
const emptySubtests = computed(() => subtests.value.filter((s) => s.soal.length === 0))
const simulationCreated = ref(false)

async function assembleSimulation() {
  const slots = subtests.value.filter((s) => s.kind === 'simulasi' && s.sessionId)
  if (slots.length === 0) return
  saving.value = true
  const oldTemplateId = existingTemplateId.value
  try {
    // Susun & pasang dulu template BARU sebelum membongkar yang lama — kalau ada langkah yang
    // gagal di tengah jalan, paket tetap punya minimal satu Simulasi UTBK yang valid terpasang
    // (tidak pernah ada jendela waktu di mana paket kehilangan konten UTBK-nya sama sekali).
    const payload: SimulationTemplatePayload = {
      title: pkgForm.value.name,
      is_premium: true,
      slots: slots.map((s, idx) => ({ sequence_index: idx + 1, session_id: s.sessionId!, break_seconds: 300 })),
      is_draft: true,
      exam_track: pkgForm.value.exam_track,
      school_type_scope: pkgForm.value.school_type_scope,
      lockdown_override: pkgForm.value.lockdown_override,
    }
    const template = await simulationService.createTemplate(payload)
    await packageService.addContentItem(packageId.value!, 'simulation_template', template.id)
    existingTemplateId.value = template.id
    simulationCreated.value = true

    // Baru sekarang bongkar template lama (kalau sebelumnya sudah ada) — lepaskan dari paket
    // lalu hapus. Kalau langkah ini gagal, paket sudah aman punya template baru yang valid;
    // template lama jadi "yatim" tapi tidak lagi tertaut ke paket mana pun (aman diabaikan/
    // dibersihkan admin belakangan), bukan sebaliknya (paket kosong).
    if (oldTemplateId) {
      const items = await packageService.listContent(packageId.value!)
      const oldItem = items.find((i) => i.content_type === 'simulation_template' && i.content_id === oldTemplateId)
      if (oldItem) await packageService.removeContentItem(packageId.value!, oldItem.id)
      await simulationService.deleteTemplate(oldTemplateId).catch(() => {
        // Non-fatal — template lama sudah lepas dari paket di atas, jadi tidak lagi
        // memengaruhi akses siswa. Gagal dihapus (mis. sudah ada run tercatat) hanya berarti
        // baris template lama tersisa yatim di DB, tidak menutupi susunan baru yang aktif.
      })
    }
    toast.success('Simulasi UTBK berhasil disusun')
  } catch (e: any) {
    toast.error('Gagal merangkai Simulasi UTBK', e?.message)
  } finally {
    saving.value = false
  }
}

// ─── Step 4 (paket TKA — elective_pick_count > 0): susun Simulasi TKA Hari 1 (wajib, slot
// tetap) + Hari 2 (pilihan, slot diisi otomatis dari pilihan siswa) sekaligus dari subtes
// yang sudah dibuat — admin tidak perlu pindah ke tab "Simulasi UTBK" terpisah sama sekali
// untuk paket TKA. Menggantikan assembleSimulation() di atas (yang tidak tahu soal mekanisme
// mapel pilihan) khusus saat isTkaPackage. ────────────────────────────────────────
// Menentukan alur susun Step 4 (Hari 1 Wajib + Hari 2 Pilihan) — khusus paket TKA yang BENAR-
// BENAR punya mapel pilihan. TKA SMP (semua wajib, elective_pick_count = 0) tetap exam_track
// 'tka' tapi lewat assembleSimulation() biasa (1 hari, tanpa split) — lihat pkgForm.exam_track.
const isTkaPackage = computed(() => pkgForm.value.elective_pick_count > 0)
// Label subtes "simulasi" mengikuti jalur ujian yang dipilih admin (bukan lagi elective_pick_count
// — TKA SMP tanpa mapel pilihan tetap harus tampil "Simulasi TKA", bukan "Simulasi UTBK").
const simulasiTag = computed(() => (pkgForm.value.exam_track === 'tka' ? 'Simulasi TKA' : 'Simulasi UTBK'))
const tkaWajibSlots = computed(() => subtests.value.filter((s) => s.kind === 'simulasi' && s.sessionId && !s.is_elective))
const tkaPilihanCount = computed(() => subtests.value.filter((s) => s.is_elective && s.sessionId).length)

async function assembleTkaSimulation() {
  const wajibSlots = tkaWajibSlots.value
  const pickCount = pkgForm.value.elective_pick_count
  if (wajibSlots.length === 0 && pickCount <= 0) return
  if (pickCount > 0 && tkaPilihanCount.value === 0) {
    toast.error('Belum ada subtes yang ditandai "Mapel Pilihan"', 'Tambahkan minimal beberapa subtes pilihan dulu sebelum menyusun Simulasi TKA Hari 2.')
    return
  }
  saving.value = true
  const oldWajibId = existingTemplateId.value
  const oldPilihanId = existingPilihanTemplateId.value
  try {
    // Sama seperti assembleSimulation(): pasang dulu template BARU sebelum membongkar yang
    // lama, supaya paket tidak pernah kehilangan konten Simulasi TKA-nya sama sekali di
    // tengah proses susun-ulang.
    if (wajibSlots.length > 0) {
      const payloadWajib: SimulationTemplatePayload = {
        title: `${pkgForm.value.name} — Hari 1 (Wajib)`,
        is_premium: true,
        // break_seconds 0 — TKA hari 1 diadministrasikan berkelanjutan tanpa jeda antar mapel
        // wajib (beda dari UTBK yang memang punya jeda resmi antar subtes).
        slots: wajibSlots.map((s, idx) => ({ sequence_index: idx + 1, session_id: s.sessionId!, break_seconds: 0, is_elective: false })),
        is_draft: true,
        template_kind: 'per_subject',
        exam_track: 'tka',
        school_type_scope: pkgForm.value.school_type_scope,
        lockdown_override: pkgForm.value.lockdown_override,
      }
      const tplWajib = await simulationService.createTemplate(payloadWajib)
      await packageService.addContentItem(packageId.value!, 'simulation_template', tplWajib.id)
      existingTemplateId.value = tplWajib.id
    }
    if (pickCount > 0) {
      const payloadPilihan: SimulationTemplatePayload = {
        title: `${pkgForm.value.name} — Hari 2 (Pilihan)`,
        is_premium: true,
        // Jumlah slot = elective_pick_count (bukan jumlah subtes pilihan yang tersedia) — tiap
        // slot diisi otomatis dari pilihan siswa saat simulasi dimulai, lihat backend
        // SimulationService::start_run.
        slots: Array.from({ length: pickCount }, (_, idx) => ({
          sequence_index: idx + 1, session_id: null, break_seconds: 0, is_elective: true,
        })),
        is_draft: true,
        elective_package_id: packageId.value!,
        template_kind: 'per_subject',
        exam_track: 'tka',
        school_type_scope: pkgForm.value.school_type_scope,
        lockdown_override: pkgForm.value.lockdown_override,
      }
      const tplPilihan = await simulationService.createTemplate(payloadPilihan)
      await packageService.addContentItem(packageId.value!, 'simulation_template', tplPilihan.id)
      existingPilihanTemplateId.value = tplPilihan.id
    }
    simulationCreated.value = true

    for (const oldId of [oldWajibId, oldPilihanId]) {
      if (!oldId || oldId === existingTemplateId.value || oldId === existingPilihanTemplateId.value) continue
      const items = await packageService.listContent(packageId.value!)
      const oldItem = items.find((i) => i.content_type === 'simulation_template' && i.content_id === oldId)
      if (oldItem) await packageService.removeContentItem(packageId.value!, oldItem.id)
      await simulationService.deleteTemplate(oldId).catch(() => {
        // Non-fatal — lihat komentar serupa di assembleSimulation().
      })
    }
    toast.success('Simulasi TKA (Hari 1 + Hari 2) berhasil disusun')
  } catch (e: any) {
    toast.error('Gagal menyusun Simulasi TKA', e?.message)
  } finally {
    saving.value = false
  }
}

async function finish(publish: boolean) {
  saving.value = true
  try {
    if (publish && packageId.value) {
      // Idempotent & aman dipanggil ulang meski paket sudah aktif sebelumnya — cascade
      // un-draft di backend berjalan lagi, jadi ini juga cara resmi "republish" supaya
      // subtes/soal baru yang ditambahkan belakangan ikut terbit.
      await packageService.setActive(packageId.value, true)
      toast.success(`Paket "${pkgForm.value.name}" dipublish — semua subtes ikut aktif`)
    } else if (pkgActive.value) {
      toast.success(`Perubahan pada "${pkgForm.value.name}" tersimpan`)
    } else {
      toast.success(`Paket "${pkgForm.value.name}" tersimpan sebagai draft`)
    }
    emit('saved')
  } catch (e: any) {
    toast.error('Gagal menyelesaikan paket', e?.message)
  } finally {
    saving.value = false
  }
}

function fmt(n: number) {
  return n.toLocaleString('id-ID')
}
</script>

<template>
  <div class="space-y-5" :class="dark ? 'text-white' : ''">
    <!-- header -->
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <button class="text-xs flex items-center gap-1 mb-1.5" :class="dark ? 'text-white/40 hover:text-white' : 'text-muted-foreground hover:text-slate-700'" @click="emit('close')">
          <ArrowLeft class="w-3.5 h-3.5" />Kembali ke daftar paket
        </button>
        <h2 class="text-lg font-bold flex items-center gap-2" :class="dark ? 'text-white' : 'text-slate-900'">
          <Sparkles class="w-5 h-5 text-violet-500" />{{ isEditMode ? `Kelola Paket — ${pkgForm.name}` : 'Buat Paket + Subtes' }}
        </h2>
        <p class="text-sm" :class="dark ? 'text-white/40' : 'text-muted-foreground'">
          <template v-if="isEditMode">{{ step === 2 ? 'Subtes' : step === 3 ? 'Isi Soal' : 'Ringkasan & Publish' }}</template>
          <template v-else>Langkah {{ step }} dari 4 &middot; {{ step === 1 ? 'Info Paket' : step === 2 ? 'Tambah Subtes' : step === 3 ? 'Isi Soal' : 'Ringkasan & Publish' }}</template>
        </p>
      </div>
      <div v-if="!isEditMode" class="flex items-center gap-1.5 w-full sm:w-64">
        <div v-for="n in 4" :key="n" class="h-1.5 flex-1 rounded-full" :class="n <= step ? 'bg-violet-500' : (dark ? 'bg-white/10' : 'bg-slate-200')" />
      </div>
    </div>

    <div v-if="loadingExisting" class="p-10 text-center text-sm flex items-center justify-center gap-2" :class="dark ? 'text-white/40' : 'text-muted-foreground'">
      <Loader2 class="w-4 h-4 animate-spin" />Memuat data paket...
    </div>

    <p v-else-if="isEditMode && pkgActive" class="text-xs rounded-lg p-3 flex items-start gap-1.5" :class="dark ? 'bg-blue-500/10 text-blue-300' : 'bg-blue-50 text-blue-700'">
      <AlertTriangle class="w-3.5 h-3.5 shrink-0 mt-0.5" />
      Paket ini sudah live di landing page. Subtes/soal baru yang ditambahkan di sini tetap tersembunyi dari siswa sampai di-publish ulang.
    </p>

    <template v-if="!loadingExisting">
    <!-- STEP 1: info paket (hanya alur buat baru) -->
    <Card v-if="step === 1" class="p-6 max-w-xl space-y-4" :class="dark ? 'bg-white/[0.03] border-white/10' : ''">
      <div>
        <label class="text-xs font-semibold uppercase tracking-wider block mb-1.5" :class="dark ? 'text-white/40' : 'text-slate-500'">Nama Paket</label>
        <input v-model="pkgForm.name" placeholder="contoh: Simulasi UTBK Batch 1" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-violet-400" :class="dark ? 'bg-white/5 border-white/10 text-white placeholder:text-white/30' : ''" />
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="text-xs font-semibold uppercase tracking-wider block mb-1.5" :class="dark ? 'text-white/40' : 'text-slate-500'">Tipe</label>
          <input v-model="pkgForm.package_type" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-violet-400" :class="dark ? 'bg-white/5 border-white/10 text-white' : ''" />
        </div>
        <div>
          <label class="text-xs font-semibold uppercase tracking-wider block mb-1.5" :class="dark ? 'text-white/40' : 'text-slate-500'">Masa Berlaku</label>
          <input v-model="pkgForm.validity" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-violet-400" :class="dark ? 'bg-white/5 border-white/10 text-white' : ''" />
        </div>
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="text-xs font-semibold uppercase tracking-wider block mb-1.5" :class="dark ? 'text-white/40' : 'text-slate-500'">Harga Asli (Rp)</label>
          <input v-model.number="pkgForm.original_price" type="number" min="0" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-violet-400" :class="dark ? 'bg-white/5 border-white/10 text-white' : ''" />
        </div>
        <div>
          <label class="text-xs font-semibold uppercase tracking-wider block mb-1.5" :class="dark ? 'text-white/40' : 'text-slate-500'">Harga Jual (Rp)</label>
          <input v-model.number="pkgForm.sale_price" type="number" min="0" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-violet-400" :class="dark ? 'bg-white/5 border-white/10 text-white' : ''" />
        </div>
      </div>
      <p class="text-xs" :class="dark ? 'text-white/40' : 'text-muted-foreground'">Diskon otomatis: <strong class="text-emerald-500">{{ discountPreview }}%</strong></p>
      <div>
        <label class="text-xs font-semibold uppercase tracking-wider block mb-1.5" :class="dark ? 'text-white/40' : 'text-slate-500'">Jalur Ujian <span class="text-red-500">*</span> wajib dipilih</label>
        <div class="grid grid-cols-2 gap-2">
          <button
            type="button"
            class="flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-sm font-semibold border-2 transition-colors"
            :class="(examTrackChosen && pkgForm.exam_track === 'snbt') ? 'bg-indigo-600 border-indigo-600 text-white' : (dark ? 'bg-white/5 border-white/10 text-white/60' : 'bg-white border-slate-200 text-slate-500 hover:bg-slate-50')"
            @click="pkgForm.exam_track = 'snbt'; examTrackChosen = true"
          >SNBT / UTBK</button>
          <button
            type="button"
            class="flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-sm font-semibold border-2 transition-colors"
            :class="(examTrackChosen && pkgForm.exam_track === 'tka') ? 'bg-violet-600 border-violet-600 text-white' : (dark ? 'bg-white/5 border-white/10 text-white/60' : 'bg-white border-slate-200 text-slate-500 hover:bg-slate-50')"
            @click="pkgForm.exam_track = 'tka'; examTrackChosen = true"
          >TKA</button>
        </div>
        <p class="text-xs mt-1" :class="dark ? 'text-white/40' : 'text-muted-foreground'">Menentukan tab tempat siswa melihat konten ini (SNBT/UTBK vs TKA) — pilih TKA meski semua subtesnya wajib (mis. TKA SMP tanpa mapel pilihan). Belum ada yang terpilih sampai Anda klik salah satu.</p>
      </div>
      <div>
        <label class="text-xs font-semibold uppercase tracking-wider block mb-1.5" :class="dark ? 'text-white/40' : 'text-slate-500'">
          Jenjang <template v-if="pkgForm.exam_track === 'tka'"><span class="text-red-500">*</span> wajib untuk TKA</template><template v-else>(opsional)</template>
        </label>
        <div class="grid grid-cols-5 gap-2">
          <button
            type="button"
            class="px-2 py-2 rounded-lg text-xs font-semibold border-2 transition-colors"
            :class="!pkgForm.school_type_scope ? 'bg-slate-700 border-slate-700 text-white' : (dark ? 'bg-white/5 border-white/10 text-white/60' : 'bg-white border-slate-200 text-slate-500 hover:bg-slate-50')"
            @click="pkgForm.school_type_scope = null"
          >Semua</button>
          <button
            v-for="opt in (['sma', 'smk', 'ma', 'smp'] as const)" :key="opt"
            type="button"
            class="px-2 py-2 rounded-lg text-xs font-semibold border-2 uppercase transition-colors"
            :class="pkgForm.school_type_scope === opt ? 'bg-emerald-600 border-emerald-600 text-white' : (dark ? 'bg-white/5 border-white/10 text-white/60' : 'bg-white border-slate-200 text-slate-500 hover:bg-slate-50')"
            @click="pkgForm.school_type_scope = opt"
          >{{ opt }}</button>
        </div>
        <p v-if="pkgForm.exam_track === 'tka'" class="text-xs mt-1 text-amber-600">Paket TKA wajib pilih jenjang spesifik (bukan "Semua") — menentukan skala nilai yang benar: 0–100 untuk SMP, 200–800 untuk SMA/SMK/MA.</p>
        <p v-else class="text-xs mt-1" :class="dark ? 'text-white/40' : 'text-muted-foreground'">Kosongkan ("Semua") untuk konten umum. Isi kalau paket ini khusus satu jenjang — siswa jenjang lain tidak akan melihat subtes/simulasi ini sama sekali di katalognya.</p>
      </div>
      <div>
        <label class="text-xs font-semibold uppercase tracking-wider block mb-1.5" :class="dark ? 'text-white/40' : 'text-slate-500'">Mode Terkunci — Simulasi TO</label>
        <div class="grid grid-cols-3 gap-2">
          <button
            type="button"
            class="px-2 py-2 rounded-lg text-xs font-semibold border-2 transition-colors"
            :class="pkgForm.lockdown_override === null ? 'bg-slate-700 border-slate-700 text-white' : (dark ? 'bg-white/5 border-white/10 text-white/60' : 'bg-white border-slate-200 text-slate-500 hover:bg-slate-50')"
            @click="pkgForm.lockdown_override = null"
          >Ikuti Default</button>
          <button
            type="button"
            class="px-2 py-2 rounded-lg text-xs font-semibold border-2 transition-colors"
            :class="pkgForm.lockdown_override === true ? 'bg-rose-600 border-rose-600 text-white' : (dark ? 'bg-white/5 border-white/10 text-white/60' : 'bg-white border-slate-200 text-slate-500 hover:bg-slate-50')"
            @click="pkgForm.lockdown_override = true"
          >Paksa Aktif</button>
          <button
            type="button"
            class="px-2 py-2 rounded-lg text-xs font-semibold border-2 transition-colors"
            :class="pkgForm.lockdown_override === false ? 'bg-emerald-600 border-emerald-600 text-white' : (dark ? 'bg-white/5 border-white/10 text-white/60' : 'bg-white border-slate-200 text-slate-500 hover:bg-slate-50')"
            @click="pkgForm.lockdown_override = false"
          >Paksa Nonaktif</button>
        </div>
        <p class="text-xs mt-1" :class="dark ? 'text-white/40' : 'text-muted-foreground'">
          Simulasi TO (baik SNBT/UTBK maupun TKA) yang disusun lewat wizard ini akan memakai pengaturan ini — bisa diubah lagi belakangan lewat tab Simulasi UTBK. Tidak berlaku untuk subtes Drilling/Latihan biasa.
        </p>
      </div>
      <div>
        <label class="text-xs font-semibold uppercase tracking-wider block mb-1.5" :class="dark ? 'text-white/40' : 'text-slate-500'">Jumlah Mapel Pilihan (0 = tidak ada batasan)</label>
        <input v-model.number="pkgForm.elective_pick_count" type="number" min="0" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-violet-400" :class="dark ? 'bg-white/5 border-white/10 text-white' : ''" />
        <p class="text-xs mt-1" :class="dark ? 'text-white/40' : 'text-muted-foreground'">Contoh TKA SMA: isi 2 — siswa harus memilih tepat 2 subtes yang ditandai "Mapel Pilihan" di Step 2 sebelum bisa mengerjakannya.</p>
      </div>
      <p class="text-xs flex items-start gap-1.5 rounded-lg p-3" :class="dark ? 'bg-white/5 text-white/50' : 'bg-slate-50 text-slate-500'">
        <FileText class="w-3.5 h-3.5 shrink-0 mt-0.5" />
        Detail marketing lain (fitur, badge, icon, warna) bisa dilengkapi belakangan lewat Edit Paket. Paket ini dibuat sebagai draft (belum tampil di landing page) sampai dipublish.
      </p>
      <div class="flex justify-end pt-2">
        <Button variant="gradient" class="gap-1.5" :disabled="saving" @click="createPackageShell">
          <Loader2 v-if="saving" class="w-4 h-4 animate-spin" />Lanjut<ChevronRight class="w-4 h-4" />
        </Button>
      </div>
    </Card>

    <!-- STEP 2: subtes -->
    <div v-else-if="step === 2" class="space-y-4">
      <div class="flex items-center justify-between flex-wrap gap-2">
        <p class="text-sm font-semibold" :class="dark ? 'text-white' : 'text-slate-800'">Subtes dalam paket ini</p>
        <div class="flex items-center gap-2">
          <Button variant="outline" size="sm" class="gap-1.5" @click="addSubtes('simulasi')"><Plus class="w-3.5 h-3.5" />Subtes {{ simulasiTag }}</Button>
          <Button variant="outline" size="sm" class="gap-1.5" @click="addSubtes('drilling')"><Plus class="w-3.5 h-3.5" />Subtes Drilling</Button>
        </div>
      </div>
      <p class="text-xs" :class="dark ? 'text-white/40' : 'text-muted-foreground'">
        "{{ simulasiTag }}" akan dirangkai jadi 1 ujian berurutan dengan jeda istirahat antar subtes. "Drilling" berdiri sendiri sebagai latihan bebas waktu.
      </p>
      <div v-if="subtests.length === 0" class="text-center py-8 text-sm" :class="dark ? 'text-white/30' : 'text-muted-foreground'">Belum ada subtes. Tambahkan minimal satu.</div>
      <div class="grid sm:grid-cols-2 gap-4">
        <Card v-for="(s, idx) in subtests" :key="s.key" class="p-4 space-y-3" :class="dark ? 'bg-white/[0.03] border-white/10' : ''">
          <div class="flex items-center justify-between">
            <Badge :variant="s.kind === 'simulasi' ? 'default' : 'secondary'">{{ s.kind === 'simulasi' ? simulasiTag : 'Drilling' }}</Badge>
            <button class="p-1 rounded hover:bg-red-500/10" @click="removeSubtes(idx)"><Trash2 class="w-3.5 h-3.5 text-red-500" /></button>
          </div>
          <input v-model="s.title" placeholder="Judul subtes, contoh: Penalaran Umum" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-violet-400" :class="dark ? 'bg-white/5 border-white/10 text-white placeholder:text-white/30' : 'bg-white'" />
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="text-[10px] font-semibold uppercase tracking-wider block mb-1" :class="dark ? 'text-white/30' : 'text-slate-400'">Durasi (menit)</label>
              <input v-model.number="s.duration_minutes" type="number" min="1" class="w-full border rounded-lg px-3 py-2 text-sm" :class="dark ? 'bg-white/5 border-white/10 text-white' : 'bg-white'" />
            </div>
            <div>
              <label class="text-[10px] font-semibold uppercase tracking-wider block mb-1" :class="dark ? 'text-white/30' : 'text-slate-400'">Subjek (opsional)</label>
              <input v-model="s.subject_filter" placeholder="contoh: Matematika" class="w-full border rounded-lg px-3 py-2 text-sm" :class="dark ? 'bg-white/5 border-white/10 text-white placeholder:text-white/30' : 'bg-white'" />
            </div>
          </div>
          <label class="flex items-center gap-2 text-xs cursor-pointer" :class="dark ? 'text-white/60' : 'text-slate-600'">
            <input v-model="s.is_elective" type="checkbox" class="w-3.5 h-3.5" :disabled="!!s.sessionId" />
            Mapel Pilihan (bukan wajib)
            <span v-if="s.sessionId" class="text-[10px]" :class="dark ? 'text-white/30' : 'text-slate-400'">(sudah dibuat)</span>
          </label>
          <p v-if="s.creating" class="text-xs text-violet-500 flex items-center gap-1.5"><Loader2 class="w-3 h-3 animate-spin" />Menyiapkan...</p>
          <div v-if="s.sessionId" class="flex items-center gap-3 pt-2 border-t" :class="dark ? 'border-white/10' : ''">
            <button
              type="button" class="text-[11px] font-medium hover:underline"
              :class="dark ? 'text-violet-300' : 'text-indigo-600'"
              @click="emit('goToSession', s.sessionId!)"
            >Kelola Sesi ini &rarr;</button>
            <button
              v-if="s.setId" type="button" class="text-[11px] font-medium hover:underline"
              :class="dark ? 'text-violet-300' : 'text-indigo-600'"
              @click="emit('goToSet', s.setId!)"
            >Kelola Set Soal ini &rarr;</button>
          </div>
        </Card>
      </div>
      <div class="flex justify-between pt-2">
        <Button variant="outline" class="gap-1.5" @click="isEditMode ? emit('close') : (step = 1)"><ChevronLeft class="w-4 h-4" />{{ isEditMode ? 'Tutup' : 'Kembali' }}</Button>
        <Button variant="gradient" class="gap-1.5" :disabled="saving" @click="materializeSubtests">
          <Loader2 v-if="saving" class="w-4 h-4 animate-spin" />Lanjut Isi Soal<ChevronRight class="w-4 h-4" />
        </Button>
      </div>
    </div>

    <!-- STEP 3: isi soal — layout 2 kolom, ruang penuh (bukan modal) -->
    <div v-else-if="step === 3" class="space-y-4">
      <p v-if="isContent" class="text-xs rounded-lg p-3 flex items-start gap-1.5" :class="dark ? 'bg-amber-500/10 text-amber-300' : 'bg-amber-50 text-amber-700'">
        <AlertTriangle class="w-3.5 h-3.5 shrink-0 mt-0.5" />
        Soal baru yang Anda tulis/impor di sini masuk antrian review Admin dulu sebelum bisa dipakai murid — soal yang dipilih dari Bank Soal (sudah Approved) langsung terpakai.
      </p>
      <div class="grid lg:grid-cols-[280px_1fr] gap-4 items-start">
        <!-- daftar subtes -->
        <div class="space-y-2">
          <button
            v-for="s in subtests" :key="s.key"
            class="w-full text-left rounded-xl border p-3 transition-colors"
            :class="[
              activeKey === s.key ? (dark ? 'border-violet-500/50 bg-violet-600/10' : 'border-indigo-300 bg-indigo-50') : (dark ? 'border-white/10 hover:bg-white/5' : 'hover:bg-slate-50'),
            ]"
            @click="activeKey = s.key"
          >
            <div class="flex items-center justify-between gap-2">
              <p class="text-sm font-semibold truncate" :class="dark ? 'text-white' : 'text-slate-800'">{{ s.title }}</p>
              <Badge :variant="s.soal.length > 0 ? 'success' : 'destructive'" class="shrink-0 text-[10px]">{{ s.soal.length }}</Badge>
            </div>
            <p class="text-[11px]" :class="dark ? 'text-white/40' : 'text-muted-foreground'">{{ s.kind === 'simulasi' ? simulasiTag : 'Drilling' }}</p>
          </button>
        </div>

        <!-- detail subtes aktif -->
        <Card v-if="activeSubtes" class="p-5 space-y-4" :class="dark ? 'bg-white/[0.03] border-white/10' : ''">
          <div class="flex items-center justify-between flex-wrap gap-2">
            <div>
              <p class="font-semibold" :class="dark ? 'text-white' : 'text-slate-800'">{{ activeSubtes.title }}</p>
              <p class="text-xs" :class="dark ? 'text-white/40' : 'text-muted-foreground'">{{ activeSubtes.kind === 'simulasi' ? simulasiTag : 'Drilling' }} &middot; {{ activeSubtes.duration_minutes }} menit &middot; {{ activeSubtes.soal.length }} soal</p>
            </div>
            <div class="flex items-center gap-2">
              <Button variant="outline" size="sm" class="gap-1.5" @click="openAuthorPanel"><PenLine class="w-3.5 h-3.5" />Tulis Soal Baru</Button>
              <Button variant="outline" size="sm" class="gap-1.5" @click="showImport = true"><Upload class="w-3.5 h-3.5" />Import Soal (.docx)</Button>
              <Button variant="outline" size="sm" class="gap-1.5" @click="togglePick(activeSubtes)"><Search class="w-3.5 h-3.5" />Bank Soal</Button>
              <Button variant="gradient" size="sm" class="gap-1.5" :disabled="activeSubtes.soal.length === 0" @click="showSetPreview = true"><Eye class="w-3.5 h-3.5" />Preview Set Soal</Button>
            </div>
          </div>

          <!-- pick existing -->
          <div v-if="activeSubtes.pick.open" class="rounded-lg border p-3 space-y-2" :class="dark ? 'border-white/10 bg-[#0f0d1a]' : 'bg-slate-50'">
            <div class="flex items-center gap-2">
              <input v-model="activeSubtes.pick.search" placeholder="Cari kode/teks soal..." class="flex-1 border rounded-lg px-3 py-1.5 text-xs" :class="dark ? 'bg-white/5 border-white/10 text-white placeholder:text-white/30' : 'bg-white'" @keyup.enter="searchApproved(activeSubtes)" />
              <Button variant="outline" size="sm" @click="searchApproved(activeSubtes)">Cari</Button>
            </div>
            <div v-if="activeSubtes.pick.loading" class="text-xs" :class="dark ? 'text-white/40' : 'text-muted-foreground'">Memuat...</div>
            <div v-else-if="activeSubtes.pick.results.length === 0" class="text-xs" :class="dark ? 'text-white/40' : 'text-muted-foreground'">Tidak ada soal Approved yang cocok.</div>
            <div v-else class="space-y-1 max-h-48 overflow-y-auto">
              <button v-for="q in activeSubtes.pick.results" :key="q.id" class="w-full text-left px-2.5 py-1.5 rounded-lg text-xs flex items-center justify-between gap-2" :class="dark ? 'hover:bg-white/5' : 'hover:bg-white'" @click="attachExisting(activeSubtes, q)">
                <span class="truncate"><strong>{{ q.code }}</strong> — {{ q.question_text.slice(0, 60) }}</span>
                <Plus class="w-3.5 h-3.5 shrink-0 text-emerald-500" />
              </button>
            </div>
          </div>

          <!-- daftar soal yang sudah masuk -->
          <div>
            <p class="text-xs font-semibold uppercase tracking-wider mb-2" :class="dark ? 'text-white/40' : 'text-slate-500'">Soal di subtes ini</p>
            <div v-if="activeSubtes.soalLoading" class="text-xs" :class="dark ? 'text-white/40' : 'text-muted-foreground'">Memuat...</div>
            <div v-else-if="activeSubtes.soal.length === 0" class="text-xs rounded-lg p-3 flex items-center gap-1.5" :class="dark ? 'bg-red-500/10 text-red-300' : 'bg-red-50 text-red-700'">
              <AlertTriangle class="w-3.5 h-3.5 shrink-0" />Belum ada soal — tulis baru, cari dari Bank Soal, atau import .docx di atas.
            </div>
            <div v-else class="space-y-1.5 max-h-72 overflow-y-auto">
              <div v-for="q in activeSubtes.soal" :key="q.id" class="flex items-center justify-between gap-2 px-3 py-2 rounded-lg border text-xs" :class="dark ? 'border-white/10' : ''">
                <span class="truncate"><strong>{{ q.code }}</strong> — {{ q.question_text.slice(0, 70) }}</span>
                <button class="p-1 rounded hover:bg-red-500/10 shrink-0" title="Hapus dari subtes ini" @click="detachSoal(activeSubtes, q)"><X class="w-3.5 h-3.5 text-red-500" /></button>
              </div>
            </div>
          </div>
        </Card>
        <div v-else class="text-sm p-8 text-center" :class="dark ? 'text-white/30' : 'text-muted-foreground'">Pilih subtes di sebelah kiri untuk mulai mengisi soal.</div>
      </div>

      <div class="flex justify-between pt-2">
        <Button variant="outline" class="gap-1.5" @click="step = 2"><ChevronLeft class="w-4 h-4" />Kembali</Button>
        <Button variant="gradient" class="gap-1.5" @click="goToSummary">Lihat Ringkasan<ChevronRight class="w-4 h-4" /></Button>
      </div>
    </div>

    <!-- STEP 4: ringkasan & publish -->
    <Card v-else class="p-6 max-w-xl space-y-4" :class="dark ? 'bg-white/[0.03] border-white/10' : ''">
      <p class="text-sm font-semibold" :class="dark ? 'text-white' : 'text-slate-800'">Ringkasan "{{ pkgForm.name }}"</p>
      <div v-for="s in subtests" :key="s.key" class="flex items-center justify-between rounded-lg border px-3 py-2" :class="dark ? 'border-white/10' : ''">
        <div class="flex items-center gap-2 text-sm">
          <component :is="s.soal.length > 0 ? CheckCircle2 : AlertTriangle" class="w-4 h-4" :class="s.soal.length > 0 ? 'text-emerald-500' : 'text-red-500'" />
          <span :class="dark ? 'text-white' : ''">{{ s.title }}</span>
          <Badge variant="outline" class="text-[10px]">{{ s.kind === 'simulasi' ? simulasiTag : 'Drilling' }}</Badge>
        </div>
        <span class="text-xs" :class="s.soal.length > 0 ? (dark ? 'text-white/50' : 'text-muted-foreground') : 'text-red-500 font-semibold'">{{ s.soal.length }} soal</span>
      </div>

      <div v-if="emptySubtests.length > 0" class="text-xs rounded-lg p-3 flex items-start gap-1.5" :class="dark ? 'bg-red-500/10 text-red-300' : 'bg-red-50 text-red-700'">
        <AlertTriangle class="w-3.5 h-3.5 shrink-0 mt-0.5" />
        {{ emptySubtests.length }} subtes masih kosong — subtes ini tidak akan bisa dibuka murid sampai diisi soal.
      </div>

      <!-- Soal yang ditulis role Konten berstatus draft sampai ditinjau admin satu-satu di
           Bank Soal — tombol ini menyetujui sekaligus semua soal di Set Soal tiap subtes
           paket ini, supaya paket langsung bisa dipakai murid tanpa perlu review manual per
           soal. Lihat backend PackageService::bulk_approve_content doc comment. -->
      <div v-if="canPublish && packageId" class="rounded-lg border p-3 space-y-2" :class="dark ? 'border-white/10' : 'bg-emerald-50 border-emerald-100'">
        <p class="text-xs" :class="dark ? 'text-white/50' : 'text-emerald-700'">
          Soal yang ditulis di sini berstatus draft sampai ditinjau — setujui semua sekaligus supaya paket langsung bisa dikerjakan murid.
        </p>
        <Button variant="outline" size="sm" class="gap-1.5" :disabled="bulkApproving" @click="bulkApproveAll">
          <ShieldCheck class="w-3.5 h-3.5" />{{ bulkApproving ? 'Menyetujui...' : 'Setujui Semua Soal di Paket Ini' }}
        </Button>
        <p v-if="bulkApprovedCount != null" class="text-xs text-emerald-500 flex items-center gap-1.5">
          <CheckCircle2 class="w-3.5 h-3.5" />{{ bulkApprovedCount > 0 ? `${bulkApprovedCount} soal baru disetujui` : 'Semua soal sudah disetujui' }}
        </p>
      </div>

      <div v-if="hasSimulasiSubtests && isTkaPackage" class="rounded-lg border p-3 space-y-2" :class="dark ? 'border-white/10' : 'bg-violet-50 border-violet-100'">
        <p class="text-xs" :class="dark ? 'text-white/50' : 'text-violet-700'">
          Paket ini punya batas mapel pilihan ({{ pkgForm.elective_pick_count }}) — susun jadi Simulasi TKA Hari 1 (wajib) + Hari 2 (pilihan, otomatis sesuai pilihan siswa).
        </p>
        <p class="text-[11px]" :class="dark ? 'text-white/40' : 'text-muted-foreground'">
          {{ tkaWajibSlots.length }} subtes wajib · {{ tkaPilihanCount }} subtes pilihan tersedia untuk dipilih siswa
        </p>
        <Button variant="outline" size="sm" class="gap-1.5" :disabled="saving" @click="assembleTkaSimulation">
          <ListChecks class="w-3.5 h-3.5" />{{ simulationCreated ? 'Susun Ulang Simulasi TKA' : 'Susun Simulasi TKA (Hari 1 + Hari 2)' }}
        </Button>
        <p v-if="simulationCreated" class="text-xs text-emerald-500 flex items-center gap-1.5"><CheckCircle2 class="w-3.5 h-3.5" />Simulasi TKA sudah dirangkai — susun ulang kalau baru menambah/menghapus subtes</p>
      </div>
      <div v-else-if="hasSimulasiSubtests" class="rounded-lg border p-3 space-y-2" :class="dark ? 'border-white/10' : 'bg-slate-50'">
        <p class="text-xs" :class="dark ? 'text-white/50' : 'text-muted-foreground'">Ada subtes bertipe Simulasi UTBK — rangkai jadi satu ujian berurutan.</p>
        <Button variant="outline" size="sm" class="gap-1.5" :disabled="saving" @click="assembleSimulation">
          <ListChecks class="w-3.5 h-3.5" />{{ simulationCreated ? 'Susun Ulang Simulasi UTBK' : 'Rangkai Simulasi UTBK' }}
        </Button>
        <p v-if="simulationCreated" class="text-xs text-emerald-500 flex items-center gap-1.5"><CheckCircle2 class="w-3.5 h-3.5" />Simulasi UTBK sudah dirangkai — susun ulang kalau baru menambah/menghapus subtes</p>
      </div>

      <p v-if="!canPublish && !pkgActive" class="text-xs rounded-lg p-3 flex items-start gap-1.5" :class="dark ? 'bg-white/5 text-white/50' : 'bg-slate-50 text-slate-500'">
        <FileText class="w-3.5 h-3.5 shrink-0 mt-0.5" />
        Paket ini tersimpan sebagai draft. Admin akan meninjau, melengkapi harga/marketing, dan mempublish ke landing page.
      </p>

      <div class="flex justify-between pt-2">
        <Button variant="outline" class="gap-1.5" :disabled="saving" @click="step = 3"><ChevronLeft class="w-4 h-4" />Kembali</Button>
        <div class="flex gap-2">
          <Button variant="outline" class="gap-1.5" :disabled="saving" @click="finish(false)"><Save class="w-4 h-4" />{{ pkgActive ? 'Simpan Perubahan' : 'Simpan Draft' }}</Button>
          <Button v-if="canPublish" variant="gradient" class="gap-1.5" :disabled="saving" @click="finish(true)"><Rocket class="w-4 h-4" />{{ pkgActive ? 'Publish Ulang' : 'Publish Sekarang' }}</Button>
        </div>
      </div>
    </Card>
    </template>

    <!-- "Tulis Soal Baru" — drawer sisi kanan, pola persis QuestionSetManager.vue -->
    <Transition name="author-drawer">
      <div
        v-if="showAuthorPanel && activeSubtes"
        class="fixed inset-y-0 right-0 z-50 w-full sm:w-[26rem] flex flex-col bg-white shadow-2xl border-l dark:bg-[#0f0d1a] dark:border-white/10"
      >
        <div class="px-4 py-3 border-b bg-slate-50 dark:bg-white/5 dark:border-white/10 shrink-0">
          <div class="flex items-center justify-between">
            <p class="text-xs font-bold uppercase tracking-wider text-slate-500 dark:text-white/40">Tulis Soal — {{ activeSubtes.title }}</p>
            <Button variant="outline" size="sm" class="gap-1.5" @click="closeAuthorPanel"><CheckCircle2 class="w-3.5 h-3.5" />Selesai</Button>
          </div>
        </div>
        <div class="flex-1 min-h-0">
          <QuestionFormPanel
            :open="showAuthorPanel"
            :default-question-set-id="activeSubtes.setId ?? undefined"
            @close="closeAuthorPanel"
            @saved="() => activeSubtes && loadSoal(activeSubtes)"
          />
        </div>
      </div>
    </Transition>
    <div v-if="showAuthorPanel" class="fixed inset-0 z-40 bg-black/30" @click="closeAuthorPanel" />

    <!-- "Import Soal (.docx)" — dialog bawaan, diarahkan ke Set Soal subtes aktif -->
    <QuestionImportDialog v-model="showImport" :default-question-set-id="activeSubtes?.setId ?? undefined" @imported="onImported" />

    <!-- "Preview Set Soal" — replika tampilan siswa, navigasi seluruh soal di subtes ini sekaligus -->
    <QuestionSetPreviewDialog v-model="showSetPreview" :questions="activeSubtes?.soal ?? []" :title="activeSubtes?.title" />
  </div>
</template>

<style scoped>
.author-drawer-enter-active,
.author-drawer-leave-active {
  transition: transform 0.2s ease;
}
.author-drawer-enter-from,
.author-drawer-leave-to {
  transform: translateX(100%);
}
</style>
