<script setup lang="ts">
// Drilling Zone — 100% data nyata: daftar sesi dari tryoutService.listSessions() (termasuk
// topic_filter/difficulty_filter asli yang di-set admin lewat SessionFormDialog), progress
// per mata uji dari analyticsService.mySubjectBreakdown() (akurasi jawaban asli), dan
// riwayat latihan dari tryoutService.myAttempts(). Tidak ada label "X soal tersedia" yang
// dikarang — angka yang ditampilkan selalu berasal dari data yang benar-benar ada.
import { Trophy, Zap, Play, Clock, Lock, Star, Sparkles, Flame, History, ChevronDown, ChevronUp, ChevronRight, Target, CheckCircle2, Layers, ArrowRight, ListChecks, Save, GraduationCap, Loader2 } from 'lucide-vue-next'
import type { TryoutSessionTemplate, Attempt, SubjectAccuracy, SimulationTemplateItem, SimulationRunItem, ElectivePackageGroup, ExamTrack } from '~/types'

const { tryoutService, analyticsService, entitlementService, simulationService, packageService } = useApi()
const { user } = useAuth()
const toast = useToast()

// Siswa B2C (mandiri, tanpa sekolah) difokuskan ke Tryout — bukan berarti Drilling per-topik
// & Simulasi UTBK disembunyikan (masih berguna buat latihan), cuma ditonjolkan/didahulukan
// lewat urutan tampil (lihat kelas `order-*` di template).
const isB2C = computed(() => !user.value?.school_id)
const attemptSession = useAttemptSessionStore()
const router = useRouter()
// The locked-premium CTA sends the student to the Overview tab, which has the "Beli Paket"
// showcase (see StudentOverview.vue) — previously the lock badge here was a dead end with no
// path to actually buy a package.
const emit = defineEmits<{ navigate: [tab: string] }>()

const sessions = ref<TryoutSessionTemplate[]>([])
const attempts = ref<Attempt[]>([])
const subjects = ref<SubjectAccuracy[]>([])
const loading = ref(true)
const startingId = ref<string | null>(null)
const showAllHistory = ref(false)
const focusSubject = ref<string | null>(null)
// Jalur ujian yang sedang dilihat — SNBT/UTBK vs TKA. Tab ini cuma dirender kalau siswa
// benar-benar punya konten di KEDUA jalur (lihat hasSnbtContent/hasTkaContent computed di bawah)
// supaya siswa yang cuma punya satu jalur tidak melihat tab kosong yang membingungkan.
const activeTrack = ref<ExamTrack>('snbt')
// Real per-content gating: server enforces this on start_attempt/start_run too — this just
// reflects reality. Fail closed (both sets stay empty) on error since this gates paid content.
// Access is now package-scoped (a Package explicitly lists which sessions/templates it
// unlocks) rather than a single all-or-nothing flag — see backend AccessService rework.
const unlockedTryoutSessionIds = ref<Set<string>>(new Set())
const unlockedSimulationTemplateIds = ref<Set<string>>(new Set())

// "Mapel Pilihan" (TKA-style subject choice) — every package with elective_pick_count > 0 that
// has at least one eligible session, plus this student's current picks per package. Draft edits
// are kept in a local per-package Set (`electiveDraft`) so the checkboxes feel instant and only
// hit the server on "Simpan Pilihan" — matches the "boleh diganti kapan saja" product decision
// (full-replace semantics server-side, see backend ElectiveService doc comment).
const electivePackages = ref<ElectivePackageGroup[]>([])
const electiveDraft = ref<Record<string, Set<string>>>({})
const savingElectives = ref<string | null>(null)

function resetElectiveDraft() {
  electiveDraft.value = Object.fromEntries(electivePackages.value.map((g) => [g.package_id, new Set(g.my_choices)]))
}

function toggleElectiveChoice(group: ElectivePackageGroup, sessionId: string) {
  const set = electiveDraft.value[group.package_id] ?? new Set<string>()
  if (set.has(sessionId)) {
    set.delete(sessionId)
  } else {
    if (group.pick_count > 0 && set.size >= group.pick_count) {
      toast.error(`Maksimal ${group.pick_count} mapel pilihan untuk "${group.package_name}"`)
      return
    }
    set.add(sessionId)
  }
  electiveDraft.value = { ...electiveDraft.value, [group.package_id]: set }
}

async function saveElectiveChoices(group: ElectivePackageGroup) {
  const chosen = [...(electiveDraft.value[group.package_id] ?? new Set<string>())]
  savingElectives.value = group.package_id
  try {
    await packageService.setElectiveChoices(group.package_id, chosen)
    toast.success(`Pilihan mapel untuk "${group.package_name}" tersimpan`)
    electivePackages.value = await packageService.myElectivePackages()
    resetElectiveDraft()
  } catch (e: any) {
    toast.error('Gagal menyimpan pilihan mapel', e?.message)
  } finally {
    savingElectives.value = null
  }
}

// Every session id this student has actually chosen, across all packages — used to disable
// elective cards wherever they appear in the lists below (Tryout Tersedia, Mode Latihan,
// Latihan Per Topik) until picked, instead of only inside the picker itself.
const chosenElectiveSessionIds = computed(() => new Set(electivePackages.value.flatMap((g) => g.my_choices)))
function electiveGroupFor(sessionId: string) {
  return electivePackages.value.find((g) => g.sessions.some((s) => s.id === sessionId)) ?? null
}

// Simulasi UTBK — a bigger commitment than a single tryout (server-orchestrated, multi-slot,
// one-active-run-at-a-time), so it's surfaced as its own entry point above the regular list.
const simulationTemplates = ref<SimulationTemplateItem[]>([])
const myRuns = ref<SimulationRunItem[]>([])
const startingSimId = ref<string | null>(null)
const inProgressRun = computed(() => myRuns.value.find((r) => r.status === 'in_progress') ?? null)
// Simulasi TO can only be completed once per template (see backend `SimulationService::start_run`
// doc comment) — map template_id -> its completed run so the catalog card can swap "Mulai
// Simulasi" for "Lihat Hasil" instead of letting the student hit the server's rejection.
const completedRunByTemplateId = computed(() => {
  const map = new Map<string, SimulationRunItem>()
  for (const r of myRuns.value) {
    if (r.status === 'completed' && !map.has(r.template_id)) map.set(r.template_id, r)
  }
  return map
})
// Tryout/Drilling biasa (single-session) — sebelumnya sama sekali tidak ditampilkan di sini,
// jadi attempt yang belum disubmit (mis. siswa refresh/tutup tab di tengah jalan) menghilang
// tanpa jejak. Backend sekarang juga resume attempt yang sama alih-alih bikin baru kalau
// "Mulai" diklik lagi (lihat TryoutService::start_attempt_impl) — kartu ini melengkapi dengan
// jalur eksplisit "Lanjutkan" tanpa perlu klik "Mulai" di sesi yang sama lagi.
const inProgressAttempt = computed(() => attempts.value.find((a) => a.status === 'in_progress') ?? null)
const visibleSimulationTemplates = computed(() => simulationTemplates.value.filter((t) => t.exam_track === activeTrack.value))
// "Mapel Pilihan" cuma relevan buat jalur TKA — sembunyikan sama sekali saat tab SNBT/UTBK aktif.
const visibleElectivePackages = computed(() => (activeTrack.value === 'tka' ? electivePackages.value : []))

function formatSimDuration(minutes: number) {
  if (minutes >= 60) {
    const h = Math.floor(minutes / 60)
    const m = minutes % 60
    return m > 0 ? `${h}j ${m}m` : `${h} jam`
  }
  return `${minutes} menit`
}

// Mirrors the same "Pilih Dulu" gating pattern already used for is_elective TryoutSessions
// below — a template with `elective_package_id` needs the student to have finished picking
// their mapel pilihan for that package before its is_elective slots can be resolved. Server
// re-enforces this regardless (see SimulationService::start_run); this is just a friendlier
// pre-check so the student isn't surprised by a raw error toast.
function electiveReadyForTemplate(t: SimulationTemplateItem): boolean {
  if (!t.elective_package_id) return true
  const group = electivePackages.value.find((g) => g.package_id === t.elective_package_id)
  if (!group) return true // can't verify from what's loaded — let the server be authoritative
  return group.my_choices.length >= group.pick_count
}

async function startSimulation(template: SimulationTemplateItem) {
  startingSimId.value = template.id
  try {
    const result = await simulationService.startRun(template.id)
    await router.push(`/simulasi/${result.id}`)
  } catch (e: any) {
    toast.error('Tidak bisa memulai simulasi', e?.message)
  } finally {
    startingSimId.value = null
  }
}

const abandoningRun = ref(false)
async function abandonSimulation() {
  if (!inProgressRun.value) return
  if (!window.confirm('Batalkan simulasi yang sedang berjalan? Subtes yang sudah dikerjakan tetap tersimpan skornya, tapi simulasi ini akan ditandai selesai/batal dan tidak bisa dilanjutkan lagi.')) return
  abandoningRun.value = true
  try {
    await simulationService.abandon(inProgressRun.value.id)
    toast.success('Simulasi dibatalkan — kamu bisa mulai yang baru sekarang')
    myRuns.value = await simulationService.listMyRuns()
  } catch (e: any) {
    toast.error('Gagal membatalkan simulasi', e?.message)
  } finally {
    abandoningRun.value = false
  }
}

onMounted(async () => {
  try {
    const [s, a] = await Promise.all([tryoutService.listSessions(), tryoutService.myAttempts()])
    sessions.value = s
    attempts.value = a
  } catch (e: any) {
    toast.error('Gagal memuat daftar sesi', e?.message)
  } finally {
    loading.value = false
  }
  try {
    subjects.value = await analyticsService.mySubjectBreakdown()
  } catch {
    // non-critical for this widget
  }
  try {
    const res = await entitlementService.myAccessStatus()
    unlockedTryoutSessionIds.value = new Set(res.unlocked_tryout_session_ids)
    unlockedSimulationTemplateIds.value = new Set(res.unlocked_simulation_template_ids)
  } catch {
    // fail closed — both sets stay empty
  }
  // allSettled (bukan all) — templates dan runs independen; satu gagal tidak boleh menyembunyikan
  // yang lain (dulu satu error 500 sesaat bikin KEDUANYA hilang, termasuk kartu "Lanjutkan" run
  // yang sedang berjalan, tanpa pesan apapun ke siswa).
  const [templatesRes, runsRes] = await Promise.allSettled([simulationService.listTemplates(), simulationService.listMyRuns()])
  if (templatesRes.status === 'fulfilled') simulationTemplates.value = templatesRes.value
  if (runsRes.status === 'fulfilled') myRuns.value = runsRes.value
  try {
    electivePackages.value = await packageService.myElectivePackages()
    resetElectiveDraft()
  } catch {
    // non-critical — "Pilih Mapel Pilihan" section just won't render; server still enforces
    // the real gate on start_attempt regardless.
  }
  // Default tab: SNBT/UTBK, kecuali siswa ini sama sekali tidak punya konten SNBT tapi punya
  // konten TKA — supaya siswa yang cuma dikasih akses TKA tidak mendarat di tab kosong.
  if (!hasSnbtContent.value && hasTkaContent.value) activeTrack.value = 'tka'
})

const submitted = computed(() =>
  attempts.value.filter((a) => a.status === 'submitted' && a.submitted_at).sort((a, b) => new Date(b.submitted_at!).getTime() - new Date(a.submitted_at!).getTime()),
)
const lastScore = computed(() => submitted.value[0]?.score ?? null)
const tryoutDoneCount = computed(() => submitted.value.length)
const currentStreak = computed(() => {
  const days = [...new Set(submitted.value.map((a) => new Date(a.submitted_at!).toDateString()))].map((d) => new Date(d).getTime()).sort((a, b) => b - a)
  if (!days.length) return 0
  const today = new Date(); today.setHours(0, 0, 0, 0)
  if (Math.round((today.getTime() - days[0]) / 86400000) > 1) return 0
  let streak = 1
  for (let i = 1; i < days.length; i++) { if (Math.round((days[i - 1] - days[i]) / 86400000) === 1) streak++; else break }
  return streak
})

// Sesi yang sudah jadi subtes di dalam Simulasi UTBK manapun disembunyikan dari daftar Tryout
// berdiri sendiri — supaya siswa hanya lihat SATU kartu "Simulasi UTBK" yang sudah terisi
// subtesnya, bukan kartu simulasi DITAMBAH 7 kartu subtes yang sama lagi secara terpisah
// (keduanya sama-sama session_type "tryout" tanpa penanda pembeda di backend).
const simulationSlotSessionIds = computed(() => {
  const ids = new Set<string>()
  for (const t of simulationTemplates.value) {
    // `is_elective` slots have `session_id === null` at the template level (resolved per
    // student only once a run starts) — nothing to hide from the standalone Tryout list yet.
    for (const s of t.slots) if (s.session_id) ids.add(s.session_id)
  }
  return ids
})
// Konten yang tersedia per jalur ujian — dipakai untuk memutuskan apakah tab SNBT/TKA perlu
// ditampilkan sama sekali (lihat komentar activeTrack di atas).
const hasSnbtContent = computed(() =>
  sessions.value.some((s) => s.exam_track === 'snbt') || simulationTemplates.value.some((t) => t.exam_track === 'snbt'),
)
const hasTkaContent = computed(() =>
  sessions.value.some((s) => s.exam_track === 'tka') || simulationTemplates.value.some((t) => t.exam_track === 'tka'),
)
const showTrackTabs = computed(() => hasSnbtContent.value && hasTkaContent.value)

const tryouts = computed(() =>
  sessions.value.filter((s) => s.session_type === 'tryout' && !simulationSlotSessionIds.value.has(s.id) && s.exam_track === activeTrack.value),
)
const topicSessions = computed(() => {
  const list = sessions.value.filter((s) => s.session_type !== 'tryout' && s.topic_filter && s.exam_track === activeTrack.value)
  return focusSubject.value ? list.filter((s) => s.subject_filter === focusSubject.value) : list
})
const modeSessions = computed(() => sessions.value.filter((s) => s.session_type !== 'tryout' && !s.topic_filter && s.exam_track === activeTrack.value))

// "Mode Latihan" — supaya tidak menumpuk kalau jumlahnya banyak, cuma tampil 4 pertama dulu.
const showAllModes = ref(false)
const visibleModeSessions = computed(() => (showAllModes.value ? modeSessions.value : modeSessions.value.slice(0, 4)))

// "Latihan Per Topik" — dikelompokkan per mata uji (accordion, collapsed default) supaya daftar
// topik yang panjang tidak langsung membanjiri layar. Saat focusSubject aktif (tombol "Latihan
// Fokus Lemah"), topicSessions computed di atas sudah otomatis menyempit ke satu mata uji saja,
// jadi grup yang terbentuk di sini juga otomatis cuma satu — auto-expand grup itu di bawah.
const expandedTopicSubjects = ref<Set<string>>(new Set())
function toggleTopicSubject(subject: string) {
  const next = new Set(expandedTopicSubjects.value)
  if (next.has(subject)) next.delete(subject); else next.add(subject)
  expandedTopicSubjects.value = next
}
const topicGroups = computed(() => {
  const groups = new Map<string, TryoutSessionTemplate[]>()
  for (const s of topicSessions.value) {
    const key = s.subject_filter || 'Lainnya'
    if (!groups.has(key)) groups.set(key, [])
    groups.get(key)!.push(s)
  }
  return [...groups.entries()]
    .map(([subject, list]) => ({ subject, list }))
    .sort((a, b) => (a.subject === 'Lainnya' ? 1 : b.subject === 'Lainnya' ? -1 : a.subject.localeCompare(b.subject)))
})
// Fokus ke satu mata uji (dari "Latihan Fokus Lemah") -> grupnya cuma satu, auto-terbuka.
watch(focusSubject, (subj) => {
  if (subj) expandedTopicSubjects.value = new Set([subj])
})

const attemptedSessionIds = computed(() => new Set(submitted.value.map((a) => a.session_id)))

const MODE_STYLES = [
  { iconGrad: 'from-orange-500 to-red-500', cardBg: 'border-orange-100 bg-gradient-to-br from-orange-50 to-red-50', icon: Zap },
  { iconGrad: 'from-red-500 to-pink-500', cardBg: 'border-pink-100 bg-gradient-to-br from-red-50 to-pink-50', icon: Flame },
  { iconGrad: 'from-violet-500 to-purple-500', cardBg: 'border-violet-100 bg-gradient-to-br from-violet-50 to-purple-50', icon: Sparkles },
]
function modeStyle(i: number) {
  return MODE_STYLES[i % 3]
}

function accuracyPct(s: SubjectAccuracy) {
  return s.total > 0 ? Math.round((s.correct / s.total) * 100) : 0
}
function subjectColor(pct: number) {
  if (pct >= 75) return 'text-emerald-600 bg-emerald-500'
  if (pct >= 50) return 'text-amber-600 bg-amber-500'
  return 'text-red-600 bg-red-500'
}
const weakestSubject = computed(() => {
  if (!subjects.value.length) return null
  return [...subjects.value].sort((a, b) => accuracyPct(a) - accuracyPct(b))[0]
})
function focusOnWeakest() {
  if (weakestSubject.value) focusSubject.value = weakestSubject.value.subject
}

const historyList = computed(() => (showAllHistory.value ? submitted.value : submitted.value.slice(0, 3)))
function relativeTime(iso: string) {
  const diffMs = Date.now() - new Date(iso).getTime()
  const h = Math.floor(diffMs / 3600000)
  if (h < 1) return 'Baru saja'
  if (h < 24) return `${h} jam lalu`
  const d = Math.floor(h / 24)
  return `${d} hari lalu`
}

async function start(session: TryoutSessionTemplate) {
  // Preemptive client-side check so the student gets a friendly "pilih dulu" instead of a raw
  // server error — the real, authoritative gate still runs server-side in
  // TryoutService::start_attempt_impl regardless (see ElectiveService::assert_can_start).
  if (session.is_elective && !chosenElectiveSessionIds.value.has(session.id)) {
    const group = electiveGroupFor(session.id)
    toast.error(
      `"${session.title}" adalah mapel pilihan${group ? ` paket "${group.package_name}"` : ''} — pilih dulu di bagian "Pilih Mapel Pilihan" di atas`,
    )
    return
  }
  startingId.value = session.id
  try {
    const result = await tryoutService.startAttempt(session.id)
    attemptSession.set(result)
    await router.push(`/tryout/${result.attempt.id}`)
  } catch (e: any) {
    toast.error('Tidak bisa memulai sesi', e?.message)
  } finally {
    startingId.value = null
  }
}
</script>

<template>
  <div class="space-y-6">
    <Card class="p-6 bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 text-white">
      <div class="flex items-center gap-2 mb-1"><Zap class="w-5 h-5 text-yellow-300" /><span class="font-bold">Drilling Zone</span></div>
      <h2 class="text-2xl font-black mb-1">Latihan Intensif &amp; Tryout</h2>
      <p class="text-purple-100 text-sm mb-4">Pilih sesi di bawah ini untuk mulai berlatih.</p>
      <div class="flex flex-wrap gap-2">
        <div class="px-3 py-1.5 rounded-full bg-white/15 text-xs font-semibold flex items-center gap-1.5"><Trophy class="w-3.5 h-3.5" />{{ lastScore ?? '-' }} Skor Terakhir</div>
        <div class="px-3 py-1.5 rounded-full bg-white/15 text-xs font-semibold flex items-center gap-1.5"><Star class="w-3.5 h-3.5" />{{ tryoutDoneCount }} Tryout Selesai</div>
        <div class="px-3 py-1.5 rounded-full bg-white/15 text-xs font-semibold flex items-center gap-1.5"><Flame class="w-3.5 h-3.5" />{{ currentStreak }} Day Streak</div>
      </div>
    </Card>

    <!-- Tab jalur ujian — cuma muncul kalau siswa ini benar-benar punya konten di KEDUA jalur
         (lihat showTrackTabs computed), supaya siswa yang cuma punya satu jalur tidak melihat
         tab yang percuma. -->
    <div v-if="!loading && showTrackTabs" class="flex gap-2">
      <button
        type="button"
        class="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl text-sm font-bold border-2 transition-colors"
        :class="activeTrack === 'snbt' ? 'bg-indigo-600 border-indigo-600 text-white' : 'bg-white border-slate-200 text-slate-500 hover:bg-slate-50'"
        @click="activeTrack = 'snbt'"
      >
        <Trophy class="w-4 h-4" />SNBT / UTBK
      </button>
      <button
        type="button"
        class="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl text-sm font-bold border-2 transition-colors"
        :class="activeTrack === 'tka' ? 'bg-violet-600 border-violet-600 text-white' : 'bg-white border-slate-200 text-slate-500 hover:bg-slate-50'"
        @click="activeTrack = 'tka'"
      >
        <GraduationCap class="w-4 h-4" />TKA
      </button>
    </div>

    <div v-if="loading" class="py-10 text-center text-muted-foreground">
      <Loader2 class="w-6 h-6 animate-spin mx-auto mb-2" /> Memuat sesi...
    </div>

    <!-- v-else section pakai flex column + kelas `order-*` (bukan cuma urutan dokumen) supaya
         siswa B2C bisa lihat "Tryout Tersedia" duluan tanpa menyembunyikan section lain
         (Simulasi UTBK & Drilling per-topik tetap berguna buat latihan, cuma didahulukan
         Tryout sesuai fokus B2C). -->
    <template v-else>
      <div class="flex flex-col gap-6">
        <!-- Tryout/Drilling biasa yang belum disubmit — selalu paling atas kalau ada, terlepas
             dari isB2C, karena ini paling sering kejadian (single-session, bukan Simulasi). -->
        <Card v-if="inProgressAttempt" class="p-4 sm:p-5 border-2 border-amber-200 bg-gradient-to-br from-amber-50 to-orange-50">
          <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4">
            <div class="flex items-center gap-3 sm:gap-4 min-w-0 flex-1">
              <div class="w-11 h-11 sm:w-12 sm:h-12 rounded-xl bg-gradient-to-br from-amber-500 to-orange-500 flex items-center justify-center shrink-0">
                <Clock class="w-5 h-5 sm:w-6 sm:h-6 text-white" />
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-xs font-bold text-amber-600 uppercase tracking-wider mb-0.5">Belum Disubmit</p>
                <h4 class="font-black text-slate-900 truncate">{{ inProgressAttempt.session_title }}</h4>
                <p class="text-xs text-slate-500 mt-0.5">Dimulai {{ new Date(inProgressAttempt.started_at).toLocaleString('id-ID', { day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit' }) }}</p>
              </div>
            </div>
            <Button variant="gradient" class="w-full sm:w-auto shrink-0" @click="router.push(`/tryout/${inProgressAttempt!.id}`)">
              Lanjutkan <ArrowRight class="w-4 h-4" />
            </Button>
          </div>
        </Card>

        <!-- Simulasi UTBK — in-progress run takes priority; hides the "start new" list to mirror
             the backend's own one-active-run-at-a-time rule and avoid ever hitting its rejection. -->
        <Card
          v-if="inProgressRun"
          class="p-4 sm:p-5 border-2 border-indigo-200 bg-gradient-to-br from-indigo-50 to-purple-50"
          :class="isB2C ? 'order-2' : 'order-1'"
        >
          <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4">
            <div class="flex items-center gap-3 sm:gap-4 min-w-0 flex-1">
              <div class="w-11 h-11 sm:w-12 sm:h-12 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shrink-0">
                <Layers class="w-5 h-5 sm:w-6 sm:h-6 text-white" />
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-xs font-bold text-indigo-600 uppercase tracking-wider mb-0.5">Simulasi Sedang Berjalan</p>
                <h4 class="font-black text-slate-900 truncate">{{ inProgressRun.template_title }}</h4>
                <p class="text-xs text-slate-500 mt-0.5">Subtes {{ inProgressRun.current_sequence_index }} dari {{ inProgressRun.slots.length }}</p>
              </div>
            </div>
            <div class="flex flex-row sm:flex-col items-center sm:items-end justify-between sm:justify-start gap-1.5 shrink-0">
              <Button variant="gradient" class="w-full sm:w-auto" @click="router.push(`/simulasi/${inProgressRun!.id}`)">
                Lanjutkan <ArrowRight class="w-4 h-4" />
              </Button>
              <button
                type="button"
                class="text-[11px] text-slate-400 hover:text-red-500 underline disabled:opacity-50 shrink-0"
                :disabled="abandoningRun"
                @click="abandonSimulation"
              >
                {{ abandoningRun ? 'Membatalkan...' : 'Simulasi ini macet? Batalkan' }}
              </button>
            </div>
          </div>
        </Card>

        <div v-else-if="visibleSimulationTemplates.length" :class="isB2C ? 'order-2' : 'order-1'">
          <h3 class="text-lg font-bold mb-3 flex items-center gap-2"><Layers class="w-5 h-5 text-indigo-600" />{{ activeTrack === 'tka' ? 'Simulasi TKA Tersedia' : 'Simulasi UTBK Tersedia' }}</h3>
          <div class="space-y-3">
            <Card v-for="t in visibleSimulationTemplates" :key="t.id" class="p-4 hover:shadow-lg transition-shadow">
              <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4">
                <div class="flex items-center gap-3 sm:gap-4 min-w-0 flex-1">
                  <div class="w-11 h-11 rounded-xl flex items-center justify-center shrink-0" :class="t.is_premium ? 'bg-amber-100' : 'bg-indigo-100'">
                    <Layers class="w-5 h-5" :class="t.is_premium ? 'text-amber-600' : 'text-indigo-600'" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-0.5 flex-wrap">
                      <h4 class="font-bold text-sm text-slate-900 truncate">{{ t.title }}</h4>
                      <Badge v-if="t.is_premium" variant="warning" class="text-[10px]">Premium</Badge>
                      <Badge v-if="t.template_kind === 'per_subject'" variant="secondary" class="text-[10px] !bg-violet-100 !text-violet-700">Skor Per-Mapel</Badge>
                      <Badge v-if="completedRunByTemplateId.has(t.id)" variant="secondary" class="text-[10px] !bg-emerald-100 !text-emerald-700 gap-1">
                        <CheckCircle2 class="w-3 h-3" /> Selesai
                      </Badge>
                    </div>
                    <div class="flex items-center gap-3 text-xs text-muted-foreground">
                      <span class="flex items-center gap-1"><Clock class="w-3 h-3" />{{ t.elective_package_id ? '≈' : '' }}{{ formatSimDuration(t.total_duration_minutes) }}</span>
                      <span>{{ t.slots.length }} subtes</span>
                    </div>
                  </div>
                </div>
                <!-- Simulasi TO cuma bisa dikerjakan sekali — sekali selesai, tombol jadi
                     "Lihat Hasil" mengarah ke run yang sudah selesai, bukan memulai baru. -->
                <Button
                  v-if="completedRunByTemplateId.has(t.id)"
                  variant="outline"
                  class="w-full sm:w-auto shrink-0"
                  @click="router.push(`/simulasi/${completedRunByTemplateId.get(t.id)!.id}`)"
                >
                  <History class="w-4 h-4" />Lihat Hasil
                </Button>
                <button
                  v-else-if="t.is_premium && !unlockedSimulationTemplateIds.has(t.id)"
                  type="button"
                  class="w-full sm:w-auto shrink-0 flex items-center justify-center gap-1.5 px-4 py-2 rounded-lg bg-amber-50 hover:bg-amber-100 text-amber-700 border border-amber-200 text-sm font-semibold transition-colors"
                  @click="emit('navigate', 'overview')"
                >
                  <Lock class="w-3.5 h-3.5" /> Buka Premium
                </button>
                <button
                  v-else-if="!electiveReadyForTemplate(t)"
                  type="button"
                  class="w-full sm:w-auto shrink-0 flex items-center justify-center gap-1.5 px-4 py-2 rounded-lg bg-violet-50 hover:bg-violet-100 text-violet-700 border border-violet-200 text-sm font-semibold transition-colors"
                  title="Pilih dulu mapel pilihan Anda di bawah sebelum memulai simulasi ini"
                >
                  <ListChecks class="w-3.5 h-3.5" /> Pilih Dulu
                </button>
                <Button v-else variant="gradient" class="w-full sm:w-auto shrink-0" :disabled="startingSimId === t.id" @click="startSimulation(t)">
                  <Play class="w-4 h-4" />{{ startingSimId === t.id ? 'Memulai...' : 'Mulai Simulasi' }}
                </Button>
              </div>
            </Card>
          </div>
        </div>

        <!-- "Pilih Mapel Pilihan" (TKA-style) — muncul kalau ada paket dengan mapel pilihan
             (elective_pick_count > 0). Boleh diganti kapan saja (full-replace di backend),
             jadi tombol "Simpan Pilihan" selalu aktif, bukan cuma sekali klik lalu terkunci. -->
        <!-- Sengaja tanpa kelas order-* — default order:0 flexbox bikin ini tampil PALING ATAS
             (sebelum semua section lain yang punya order-1..7), supaya siswa lihat & selesaikan
             pilihan mapel dulu sebelum mulai mengerjakan subtes yang bergantung padanya. -->
        <div v-if="visibleElectivePackages.length" class="space-y-4">
          <Card v-for="g in visibleElectivePackages" :key="g.package_id" class="p-5 border-2 border-violet-200 bg-gradient-to-br from-violet-50 to-fuchsia-50">
            <div class="flex items-center justify-between flex-wrap gap-2 mb-3">
              <div class="flex items-center gap-2">
                <div class="w-9 h-9 rounded-xl bg-gradient-to-br from-violet-500 to-fuchsia-600 flex items-center justify-center shrink-0">
                  <ListChecks class="w-4.5 h-4.5 text-white" />
                </div>
                <div>
                  <h3 class="font-bold text-sm text-slate-900">Pilih Mapel Pilihan — {{ g.package_name }}</h3>
                  <p class="text-xs text-muted-foreground">Pilih tepat {{ g.pick_count }} mapel &middot; {{ (electiveDraft[g.package_id]?.size ?? 0) }}/{{ g.pick_count }} terpilih</p>
                </div>
              </div>
              <Button
                size="sm" variant="gradient" class="gap-1.5"
                :disabled="savingElectives === g.package_id"
                @click="saveElectiveChoices(g)"
              >
                <Save class="w-3.5 h-3.5" />{{ savingElectives === g.package_id ? 'Menyimpan...' : 'Simpan Pilihan' }}
              </Button>
            </div>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="s in g.sessions" :key="s.id"
                type="button"
                class="px-3 py-1.5 rounded-lg text-xs font-semibold border transition-colors"
                :class="electiveDraft[g.package_id]?.has(s.id) ? 'bg-violet-600 border-violet-600 text-white' : 'bg-white border-violet-200 text-slate-600 hover:bg-violet-50'"
                @click="toggleElectiveChoice(g, s.id)"
              >
                {{ s.title }}
              </button>
            </div>
          </Card>
        </div>

        <!-- Progress per subtes -->
        <Card v-if="subjects.length" class="p-5" :class="isB2C ? 'order-3' : 'order-2'">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-bold flex items-center gap-2"><Target class="w-5 h-5 text-indigo-600" />Progress Per Subtes</h3>
            <Button v-if="weakestSubject" size="sm" variant="outline" @click="focusOnWeakest">Latihan Fokus Lemah</Button>
          </div>
          <div class="grid sm:grid-cols-2 gap-3">
            <div v-for="s in subjects" :key="s.subject" class="p-3 rounded-lg border" :class="focusSubject === s.subject ? 'border-indigo-300 bg-indigo-50' : ''">
              <div class="flex items-center justify-between text-sm mb-1.5">
                <span class="font-semibold text-slate-800">{{ s.subject }}</span>
                <span class="text-xs font-bold" :class="subjectColor(accuracyPct(s)).split(' ')[0]">{{ accuracyPct(s) }}%</span>
              </div>
              <div class="h-2 rounded-full bg-slate-100 overflow-hidden mb-1.5">
                <div class="h-full rounded-full" :class="subjectColor(accuracyPct(s)).split(' ')[1]" :style="{ width: `${accuracyPct(s)}%` }" />
              </div>
              <div class="text-[11px] text-muted-foreground">{{ s.total }} soal dikerjakan</div>
            </div>
          </div>
        </Card>

        <div v-if="tryouts.length" :class="isB2C ? 'order-1' : 'order-3'">
          <h3 class="text-lg font-bold mb-3 flex items-center gap-2">
            <Trophy class="w-5 h-5 text-indigo-600" />Tryout Tersedia
            <Badge v-if="isB2C" variant="secondary" class="text-[10px]">Fokus Kamu</Badge>
          </h3>
          <div class="space-y-3">
            <Card v-for="t in tryouts" :key="t.id" class="p-4 hover:shadow-lg transition-shadow">
              <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4">
                <div class="flex items-center gap-3 sm:gap-4 min-w-0 flex-1">
                  <div class="w-11 h-11 rounded-xl flex items-center justify-center shrink-0" :class="t.is_premium ? 'bg-amber-100' : 'bg-indigo-100'">
                    <Trophy class="w-5 h-5" :class="t.is_premium ? 'text-amber-600' : 'text-indigo-600'" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-0.5 flex-wrap">
                      <h4 class="font-bold text-sm text-slate-900 truncate">{{ t.title }}</h4>
                      <Badge v-if="t.is_premium" variant="warning" class="text-[10px]">Premium</Badge>
                      <Badge v-if="t.is_elective" variant="outline" class="text-[10px]">Pilihan</Badge>
                    </div>
                    <div class="flex items-center gap-3 text-xs text-muted-foreground">
                      <span class="flex items-center gap-1"><Clock class="w-3 h-3" />{{ t.duration_minutes }} menit</span>
                      <span>{{ t.question_count }} soal</span>
                    </div>
                  </div>
                </div>
                <button
                  v-if="t.is_premium && !unlockedTryoutSessionIds.has(t.id)"
                  type="button"
                  class="w-full sm:w-auto shrink-0 flex items-center justify-center gap-1.5 px-4 py-2 rounded-lg bg-amber-50 hover:bg-amber-100 text-amber-700 border border-amber-200 text-sm font-semibold transition-colors"
                  @click="emit('navigate', 'overview')"
                >
                  <Lock class="w-3.5 h-3.5" /> Buka Premium
                </button>
                <Button
                  v-else variant="gradient" class="w-full sm:w-auto shrink-0"
                  :disabled="startingId === t.id || (t.is_elective && !chosenElectiveSessionIds.has(t.id))"
                  @click="start(t)"
                >
                  <Play class="w-4 h-4" />{{ startingId === t.id ? 'Memulai...' : (t.is_elective && !chosenElectiveSessionIds.has(t.id)) ? 'Pilih Dulu' : 'Mulai' }}
                </Button>
              </div>
            </Card>
          </div>
        </div>

        <div v-if="modeSessions.length" class="order-4">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-lg font-bold flex items-center gap-2"><Zap class="w-5 h-5 text-orange-500" />Mode Latihan</h3>
            <button v-if="modeSessions.length > 4" class="text-xs font-semibold text-indigo-600 flex items-center gap-1 hover:underline" @click="showAllModes = !showAllModes">
              {{ showAllModes ? 'Sembunyikan' : `Lihat Semua (${modeSessions.length})` }}
              <component :is="showAllModes ? ChevronUp : ChevronDown" class="w-3.5 h-3.5" />
            </button>
          </div>
          <div class="space-y-2">
            <Card v-for="(s, i) in visibleModeSessions" :key="s.id" class="p-3.5 flex items-center gap-3 flex-wrap hover:shadow-lg transition-shadow">
              <div class="w-9 h-9 rounded-lg bg-gradient-to-br flex items-center justify-center shrink-0" :class="modeStyle(i).iconGrad">
                <component :is="modeStyle(i).icon" class="w-4 h-4 text-white" />
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-1.5 mb-0.5 flex-wrap">
                  <h4 class="font-semibold text-sm text-slate-900 truncate">{{ s.title }}</h4>
                  <Badge v-if="s.is_elective" variant="outline" class="text-[10px] shrink-0">Pilihan</Badge>
                </div>
                <div class="text-xs text-muted-foreground">{{ s.question_count }} soal &middot; {{ s.duration_minutes }} menit</div>
              </div>
              <Button
                size="sm" variant="gradient" class="shrink-0"
                :disabled="startingId === s.id || (s.is_elective && !chosenElectiveSessionIds.has(s.id))"
                @click="start(s)"
              >
                <Play class="w-3.5 h-3.5" />{{ startingId === s.id ? 'Memulai...' : (s.is_elective && !chosenElectiveSessionIds.has(s.id)) ? 'Pilih Dulu' : 'Mulai' }}
              </Button>
            </Card>
          </div>
        </div>

        <!-- Latihan per topik — dikelompokkan per mata uji (accordion) supaya daftar topik yang
             panjang tidak langsung membanjiri layar. Lihat topicGroups/expandedTopicSubjects. -->
        <div v-if="topicSessions.length" class="order-5">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-lg font-bold flex items-center gap-2"><Target class="w-5 h-5 text-emerald-600" />Latihan Per Topik</h3>
            <button v-if="focusSubject" class="text-xs text-indigo-600 font-semibold hover:underline" @click="focusSubject = null">Hapus filter: {{ focusSubject }}</button>
          </div>
          <div class="space-y-2">
            <div v-for="g in topicGroups" :key="g.subject" class="border rounded-xl overflow-hidden">
              <button
                type="button"
                class="w-full flex items-center justify-between px-4 py-3 bg-slate-50 hover:bg-slate-100 transition-colors"
                @click="toggleTopicSubject(g.subject)"
              >
                <span class="text-sm font-bold text-slate-800 flex items-center gap-2">
                  {{ g.subject }}
                  <span class="text-[10px] font-semibold px-1.5 py-0.5 rounded-full bg-slate-200 text-slate-600">{{ g.list.length }}</span>
                </span>
                <component :is="expandedTopicSubjects.has(g.subject) ? ChevronUp : ChevronDown" class="w-4 h-4 text-slate-400 shrink-0" />
              </button>
              <div v-if="expandedTopicSubjects.has(g.subject)" class="p-2 space-y-2 bg-white">
                <Card v-for="s in g.list" :key="s.id" class="p-3.5 flex items-center gap-3 flex-wrap hover:shadow-lg transition-shadow">
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 flex-wrap mb-1">
                      <h4 class="font-semibold text-sm text-slate-900">{{ s.topic_filter }}</h4>
                      <Badge v-if="s.difficulty_filter" variant="outline" class="text-[10px] capitalize">{{ s.difficulty_filter }}</Badge>
                      <Badge v-if="s.is_elective" variant="outline" class="text-[10px]">Pilihan</Badge>
                      <CheckCircle2 v-if="attemptedSessionIds.has(s.id)" class="w-3.5 h-3.5 text-emerald-500" />
                    </div>
                    <div class="text-xs text-muted-foreground">{{ s.question_count }} soal &middot; {{ s.duration_minutes }} menit</div>
                  </div>
                  <Button
                    size="sm" variant="outline" class="shrink-0"
                    :disabled="startingId === s.id || (s.is_elective && !chosenElectiveSessionIds.has(s.id))"
                    @click="start(s)"
                  >
                    <Play class="w-3.5 h-3.5" />{{ (s.is_elective && !chosenElectiveSessionIds.has(s.id)) ? 'Pilih Dulu' : attemptedSessionIds.has(s.id) ? 'Ulangi' : 'Mulai' }}
                  </Button>
                </Card>
              </div>
            </div>
          </div>
        </div>

        <!-- Riwayat latihan -->
        <Card v-if="submitted.length" class="p-5 order-6">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-lg font-bold flex items-center gap-2"><History class="w-5 h-5 text-slate-500" />Riwayat Latihan</h3>
            <button v-if="submitted.length > 3" class="text-xs font-semibold text-indigo-600 flex items-center gap-1 hover:underline" @click="showAllHistory = !showAllHistory">
              {{ showAllHistory ? 'Sembunyikan' : 'Lihat Semua' }}
              <component :is="showAllHistory ? ChevronUp : ChevronDown" class="w-3.5 h-3.5" />
            </button>
          </div>
          <div class="space-y-2">
            <!-- Di layar sempit: judul+waktu di baris atas, 4 metrik jadi mini-grid rata di
                 bawahnya (bukan dipaksa satu baris penuh dengan teks di kiri seperti desktop) —
                 sebelumnya ini gampang kepotong/terlalu rapat di lebar <640px. -->
            <NuxtLink
              v-for="a in historyList" :key="a.id" :to="`/hasil/${a.id}`"
              class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2.5 sm:gap-4 p-3 rounded-lg border text-sm hover:border-indigo-300 hover:bg-indigo-50/50 transition-colors"
            >
              <div class="min-w-0 flex items-center justify-between gap-2 sm:block">
                <div>
                  <div class="font-semibold text-slate-900 truncate">{{ a.session_title }}</div>
                  <div class="text-xs text-muted-foreground">{{ relativeTime(a.submitted_at!) }} &middot; {{ a.duration_minutes }} menit</div>
                </div>
                <ChevronRight class="w-4 h-4 text-slate-300 shrink-0 sm:hidden" />
              </div>
              <div class="grid grid-cols-4 sm:flex sm:items-center gap-2 sm:gap-4 shrink-0 text-xs">
                <div class="text-center sm:text-right"><div class="font-black text-slate-900">{{ a.score }}</div><div class="text-muted-foreground">Skor</div></div>
                <div class="text-center sm:text-right"><div class="font-black text-slate-900">{{ a.accuracy != null ? Math.round(a.accuracy) + '%' : '-' }}</div><div class="text-muted-foreground">Akurasi</div></div>
                <div class="text-center sm:text-right"><div class="font-black text-emerald-600">{{ a.correct_count ?? 0 }}</div><div class="text-muted-foreground">Benar</div></div>
                <div class="text-center sm:text-right"><div class="font-black text-red-500">{{ a.wrong_count ?? 0 }}</div><div class="text-muted-foreground">Salah</div></div>
                <ChevronRight class="hidden sm:block w-4 h-4 text-slate-300 shrink-0" />
              </div>
            </NuxtLink>
          </div>
        </Card>

        <Card v-if="sessions.length === 0" class="p-10 text-center text-sm text-muted-foreground order-7">
          Belum ada sesi tryout/drilling tersedia. Hubungi Admin Pusat untuk menambahkan sesi.
        </Card>
      </div>
    </template>
  </div>
</template>
