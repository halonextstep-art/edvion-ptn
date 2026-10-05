<script setup lang="ts">
// Simulasi UTBK — student player. Unlike frontend/pages/tryout/[attemptId].vue (which fills
// a Pinia store only at navigation time and loses all state on refresh), this page derives
// 100% of its state from GET /simulations/runs/:id on every mount. The server is the sole
// authority on timing/state transitions — local countdowns here are purely a cosmetic
// ticker; when one hits zero we just ask the server for fresh state (loadRun) or ask it to
// advance past a break (advanceNow). We never mutate run.value.slots[...] locally to fake a
// transition.
import {
  Clock, Flag, ChevronLeft, ChevronRight, Send, ArrowLeftRight, Award,
  BarChart3, Home, PartyPopper, Coffee, Loader2, ShieldAlert, Maximize, PlayCircle, ListChecks, Download,
} from 'lucide-vue-next'
import type { SimulationRunItem, SchoolType } from '~/types'
import { exportSimulationCertificate } from '~/utils/simulationCertificate'

const MATCH_SPLIT = '__MATCH_SPLIT__'
function matchingLeftsRights(options?: string[] | null) {
  const opts = options || []
  const splitIdx = opts.indexOf(MATCH_SPLIT)
  if (splitIdx === -1) return { lefts: [] as string[], rights: [] as string[] }
  return { lefts: opts.slice(0, splitIdx), rights: opts.slice(splitIdx + 1) }
}

definePageMeta({ middleware: 'auth', roles: ['student'] })

const route = useRoute()
const runId = route.params.runId as string
const { simulationService, tryoutService } = useApi()
const { user } = useAuth()
const toast = useToast()

const run = ref<SimulationRunItem | null>(null)
const loading = ref(true)
const current = ref(0)
// Rebuilt from scratch on every fresh run fetch. GET /simulations/runs/:id does not return
// previously-saved answer text for the current slot's questions (no such endpoint exists),
// so a mid-quiz refresh shows blank inputs again even though the server already has the
// prior autosaved answers stored via the same attempt_answers mechanism as the single-session
// player — nothing is actually lost since only the last value submitted per question matters
// at grading time. Building a "fetch back saved answers" endpoint is out of scope here.
const answers = ref<Record<string, string>>({})
const flagged = ref<Set<string>>(new Set())
const timeLeft = ref(0)
const breakLeft = ref(0)
let ticker: ReturnType<typeof setInterval> | null = null
const submitting = ref(false)
const advancing = ref(false)
// Guards the "Menyinkronkan status simulasi..." fallback below (current slot matches neither
// active/break/finished — e.g. briefly `pending` right after the server advances to a slot
// it hasn't fully activated yet). This USED to just sit there forever with no self-heal — the
// ticker never called loadRun() again in this branch, so a student who landed here was stuck
// on a spinner with no way out short of a manual refresh. Now it retries automatically, capped
// so a genuinely broken run doesn't poll forever — see setupTicker() and the template below.
const stuckAttempts = ref(0)
const STUCK_MAX_ATTEMPTS = 10
let lastStuckPollAt = 0
const stuckGivenUp = computed(() => stuckAttempts.value >= STUCK_MAX_ATTEMPTS)

// ─── Mode Terkunci (focus/lockdown) — see composables/useLockdownMode.ts doc comment. Only
// ever armed for THIS run when the server snapshotted lockdown_enabled = true at start_run
// time (immutable for the run's lifetime, see backend domain::simulation::SimulationRun doc
// comment), and only while the run is genuinely in_progress (never during loading/finished). ──
const lockdownEnabled = computed(() => run.value?.lockdown_enabled ?? false)
// Deliberately false while `isIntro` (slot 1 still `pending`, exam clock not started yet) — the
// fullscreen requirement is folded into the Intro screen's own "Mulai Sekarang" button instead
// of showing the separate rose gate overlay before the student has even read the overview.
const lockdownActiveNow = computed(() => run.value?.status === 'in_progress' && currentSlot.value?.status !== 'pending')
const violationCount = ref(0)
const lastViolationAt: Record<string, number> = {}
const VIOLATION_LABELS: Record<string, string> = {
  tab_switch: 'Kamu berpindah tab/aplikasi',
  window_blur: 'Jendela ujian kehilangan fokus',
  fullscreen_exit: 'Kamu keluar dari mode layar penuh',
  context_menu_attempt: 'Klik-kanan tidak diizinkan selama ujian',
  copy_attempt: 'Copy/cut teks tidak diizinkan selama ujian',
  devtools_attempt: 'Shortcut ini tidak diizinkan selama ujian',
  back_navigation_attempt: 'Navigasi kembali tidak diizinkan selama ujian',
}

function handleLockdownViolation(eventType: string, detail?: string) {
  // Some browser events double-fire for the same real-world action (e.g. a tab switch fires
  // both `visibilitychange` and `blur`) — throttle per event_type so the student doesn't get
  // spammed with duplicate warnings/log rows for one action.
  const now = Date.now()
  if (now - (lastViolationAt[eventType] ?? 0) < 4000) return
  lastViolationAt[eventType] = now
  violationCount.value++
  toast.error('Pelanggaran Mode Terkunci', VIOLATION_LABELS[eventType] ?? eventType)
  simulationService.reportViolation(runId, eventType, detail).catch(() => {
    // Fire-and-forget — a logging failure must never interrupt the student's exam.
  })
}

const lockdown = useLockdownMode({
  enabled: lockdownEnabled,
  active: lockdownActiveNow,
  onViolation: handleLockdownViolation,
})

function formatTime(seconds: number) {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

const currentSlot = computed(() => run.value?.slots.find((s) => s.sequence_index === run.value!.current_sequence_index) ?? null)
const questions = computed(() => run.value?.current_questions ?? [])
const isActive = computed(() => currentSlot.value?.status === 'active')
const isBreak = computed(() => currentSlot.value?.status === 'submitted' && run.value?.status === 'in_progress')
const isFinished = computed(() => run.value?.status === 'completed')
// The run was just created and slot 1 is still `pending` — the exam clock hasn't started yet.
// Only ever true for slot 1: every later slot goes straight `pending` → `active` inside a
// single `advance_after_break` request, so it's never observable as `pending` from the client.
const isIntro = computed(() => run.value?.status === 'in_progress' && currentSlot.value?.status === 'pending')
const nextSlot = computed(() => run.value?.slots.find((s) => s.sequence_index === (run.value?.current_sequence_index ?? -1) + 1) ?? null)
const totalDurationMinutes = computed(() => run.value?.slots.reduce((sum, s) => sum + (s.duration_minutes || 0), 0) ?? 0)

const answeredCount = computed(() => Object.keys(answers.value).length)
const currentMatching = computed(() => matchingLeftsRights(questions.value[current.value]?.options))

function setupTicker() {
  if (ticker) clearInterval(ticker)
  ticker = setInterval(() => {
    if (!run.value) return
    // This ticker NEVER performs a state transition itself — it only ever reads the
    // server-provided deadline_at/break_ends_at locally to drive a cosmetic countdown, and
    // when that countdown reaches zero it asks the server for the real state (loadRun) or
    // asks the server to advance (advanceNow). The server independently re-verifies elapsed
    // time before honoring either call.
    if (isActive.value && currentSlot.value?.deadline_at) {
      stuckAttempts.value = 0
      const secs = Math.max(0, Math.floor((new Date(currentSlot.value.deadline_at).getTime() - Date.now()) / 1000))
      timeLeft.value = secs
      if (secs <= 0) loadRun()
    } else if (isBreak.value && currentSlot.value?.break_ends_at) {
      stuckAttempts.value = 0
      const secs = Math.max(0, Math.floor((new Date(currentSlot.value.break_ends_at).getTime() - Date.now()) / 1000))
      breakLeft.value = secs
      if (secs <= 0) advanceNow(true)
    } else if (!isFinished.value && !isIntro.value && !stuckGivenUp.value) {
      // Neither active/break/finished — the "Menyinkronkan status simulasi..." fallback below
      // is showing. Poll again every ~3s instead of sitting here forever with no self-heal.
      const now = Date.now()
      if (now - lastStuckPollAt > 3000) {
        lastStuckPollAt = now
        stuckAttempts.value++
        loadRun()
      }
    }
  }, 1000)
}

async function loadRun() {
  loading.value = true
  try {
    run.value = await simulationService.getRun(runId)
    current.value = 0
    answers.value = {}
    flagged.value = new Set()
    orderingWorkingCopy.value = {}
    setupTicker()
  } catch (e: any) {
    toast.error('Gagal memuat simulasi', e?.message)
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  // Awaited (not fire-and-forget) so `run.value`/`lockdownEnabled` are populated BEFORE
  // `lockdown.start()` reads them — `start()` decides synchronously, once, whether to show the
  // fullscreen gate and push the back-navigation history buffer, so calling it while `run` is
  // still `null` would silently skip both for a lockdown-enabled run.
  await loadRun()
  lockdown.start()
})
onUnmounted(() => {
  if (ticker) clearInterval(ticker)
  lockdown.stop()
})

async function selectAnswer(questionId: string, value: string) {
  answers.value = { ...answers.value, [questionId]: value }
  const attemptId = currentSlot.value?.attempt_id
  if (!attemptId) return
  try {
    await tryoutService.saveAnswer(attemptId, questionId, value, flagged.value.has(questionId))
  } catch {
    // autosave failure is non-blocking, mirrors the single-session player
  }
}

function matchingCurrentPairs(questionId: string, lefts: string[]): string[] {
  const chosen = new Array(lefts.length).fill('')
  const raw = answers.value[questionId]
  if (!raw) return chosen
  raw.split(', ').forEach((entry) => {
    const [l, r] = entry.split('::')
    const idx = lefts.indexOf(l)
    if (idx >= 0) chosen[idx] = r || ''
  })
  return chosen
}

function selectMatchingPair(questionId: string, lefts: string[], leftIdx: number, rightValue: string) {
  const chosen = matchingCurrentPairs(questionId, lefts)
  chosen[leftIdx] = rightValue
  const serialized = lefts.map((l, i) => `${l}::${chosen[i] || ''}`).join(', ')
  selectAnswer(questionId, serialized)
}

// See frontend/pages/tryout/[attemptId].vue for the doc comments behind these three helpers —
// identical mechanics, duplicated here since this page derives all state from a separate
// GET /simulations/runs/:id endpoint rather than sharing the tryout attempt store.
function tfComplexCurrentAnswers(questionId: string, count: number): string[] {
  const chosen = new Array(count).fill('')
  const raw = answers.value[questionId]
  if (!raw) return chosen
  raw.split(', ').forEach((v, i) => { if (i < count) chosen[i] = v })
  return chosen
}
function selectTfComplexAnswer(questionId: string, count: number, idx: number, value: string) {
  const chosen = tfComplexCurrentAnswers(questionId, count)
  chosen[idx] = value
  selectAnswer(questionId, chosen.join(', '))
}

const orderingWorkingCopy = ref<Record<string, string[]>>({})
function orderingCurrentOrder(questionId: string, shuffledOptions: string[]): string[] {
  if (orderingWorkingCopy.value[questionId]) return orderingWorkingCopy.value[questionId]
  const raw = answers.value[questionId]
  const initial = raw ? raw.split(', ') : [...shuffledOptions]
  orderingWorkingCopy.value = { ...orderingWorkingCopy.value, [questionId]: initial }
  return initial
}
function moveOrderingAnswer(questionId: string, idx: number, dir: -1 | 1) {
  const arr = [...orderingCurrentOrder(questionId, [])]
  const target = idx + dir
  if (target < 0 || target >= arr.length) return
  ;[arr[idx], arr[target]] = [arr[target], arr[idx]]
  orderingWorkingCopy.value = { ...orderingWorkingCopy.value, [questionId]: arr }
  selectAnswer(questionId, arr.join(', '))
}

async function toggleFlag(questionId: string) {
  const next = new Set(flagged.value)
  next.has(questionId) ? next.delete(questionId) : next.add(questionId)
  flagged.value = next
  const attemptId = currentSlot.value?.attempt_id
  if (!attemptId) return
  try {
    await tryoutService.saveAnswer(attemptId, questionId, answers.value[questionId] ?? null, next.has(questionId))
  } catch {
    // ignore
  }
}

async function submitCurrent() {
  if (submitting.value) return
  submitting.value = true
  try {
    run.value = await simulationService.submitCurrent(runId)
    current.value = 0
    answers.value = {}
    flagged.value = new Set()
    orderingWorkingCopy.value = {}
    setupTicker()
  } catch (e: any) {
    toast.error('Gagal submit subtes', e?.message)
  } finally {
    submitting.value = false
  }
}

// `silent` is true when called automatically by the ticker (breakLeft hit 0) and false for
// the manual "Lanjut ke Subtes Berikutnya" button click. The ticker re-evaluates every 1s
// off `currentSlot.value.break_ends_at`, which does NOT change on a failed call — so a
// genuine failure here (e.g. clock skew vs. the server's "jeda belum selesai" check, or the
// backend having already advanced this run via a concurrent request) would otherwise repeat
// every single second and stack an error toast per tick. The backend's own advance endpoint
// is idempotent for the "already advanced" case, so the only failures that still reach here
// are the rare clock-skew race — resync silently via loadRun() instead of spamming; the next
// tick picks up whatever the server actually thinks the state is.
async function advanceNow(silent = false) {
  if (advancing.value) return
  advancing.value = true
  try {
    run.value = await simulationService.advance(runId)
    current.value = 0
    answers.value = {}
    flagged.value = new Set()
    orderingWorkingCopy.value = {}
    setupTicker()
  } catch (e: any) {
    if (silent) {
      loadRun().catch(() => {})
    } else {
      toast.error('Gagal lanjut ke subtes berikutnya', e?.message)
    }
  } finally {
    advancing.value = false
  }
}

const beginning = ref(false)
// Called by the Intro screen's "Mulai Sekarang" button — this is the exact moment the exam
// clock starts (server activates slot 1 only once this call lands). When lockdown is enabled,
// the fullscreen request is folded into this same click so the student only ever has to click
// once, instead of a second separate gate overlay immediately after.
async function beginNow() {
  if (beginning.value) return
  beginning.value = true
  try {
    if (lockdownEnabled.value) {
      await lockdown.enterFullscreen()
    }
    run.value = await simulationService.beginRun(runId)
    current.value = 0
    setupTicker()
  } catch (e: any) {
    toast.error('Gagal memulai simulasi', e?.message)
  } finally {
    beginning.value = false
  }
}

const downloadingCertificate = ref(false)
// Fetches the honest peer benchmark (see TemplateStatsItem doc comment) fresh every download
// rather than caching it on `run` — it changes as more students complete the same template.
// A failed fetch degrades gracefully: the certificate still generates, just without that line.
async function downloadCertificate() {
  const currentRun = run.value
  if (!currentRun || downloadingCertificate.value) return
  downloadingCertificate.value = true
  try {
    let stats = null
    try {
      stats = await simulationService.templateStats(currentRun.template_id)
    } catch {
      // non-critical — certificate still generates without the peer-comparison line
    }
    exportSimulationCertificate(user.value?.name || 'Siswa', user.value?.school_name || null, currentRun, stats)
  } catch (e: any) {
    toast.error('Gagal membuat sertifikat', e?.message)
  } finally {
    downloadingCertificate.value = false
  }
}

function slotStepClass(status: string) {
  if (status === 'submitted') return 'bg-emerald-500 text-white'
  if (status === 'active') return 'bg-indigo-600 text-white'
  return 'bg-slate-200 text-slate-500'
}

// TKA presentation-layer rescaling — ONLY for `template_kind === 'per_subject'` runs (e.g.
// Simulasi TKA). Real TKA reports each mata uji via IRT on a jenjang-specific scale (0-100 for
// SD/SMP, 200-800 for SMA/SMK/MA — Perka BSKAP No. 047/2025 & No. 045/2025), which this codebase
// has no real calibrated item-parameter data to reproduce — rescaling the existing platform
// 0-1000 score is an honest DISPLAY-ONLY approximation, never presented as the real official
// score. Only "Istimewa" is shown as a category (the one publicly documented threshold per
// jenjang); Baik/Memadai/Kurang cutoffs are determined by post-exam national standard-setting
// statistics this codebase doesn't have, so anything below Istimewa gets a neutral label instead
// of a fabricated category. Mirrored in `backend::application::school_tka_service::tka_scale_for`
// and `utils/simulationCertificate.ts` — kept in sync manually, update all three if it changes.
function tkaScaleFor(scope: SchoolType | null | undefined): { base: number; span: number; istimewaThreshold: number; label: string } {
  if (scope === 'smp') return { base: 0, span: 100, istimewaThreshold: 95, label: '0–100' }
  return { base: 200, span: 600, istimewaThreshold: 725, label: '200–800' }
}
function tkaScaledScore(rawScore: number | null, scope: SchoolType | null | undefined): number | null {
  if (rawScore == null) return null
  const scale = tkaScaleFor(scope)
  return Math.round(scale.base + (rawScore / 1000) * scale.span)
}
function tkaCategoryLabel(rawScore: number | null, scope: SchoolType | null | undefined): string | null {
  const scaled = tkaScaledScore(rawScore, scope)
  if (scaled == null) return null
  return scaled >= tkaScaleFor(scope).istimewaThreshold ? 'Istimewa' : 'Di bawah Istimewa'
}

function formatDateTimeRange(startIso: string, endIso: string | null) {
  if (!endIso) return null
  const ms = new Date(endIso).getTime() - new Date(startIso).getTime()
  if (ms <= 0) return null
  const totalMin = Math.round(ms / 60000)
  const h = Math.floor(totalMin / 60)
  const m = totalMin % 60
  return h > 0 ? `${h} jam ${m} menit` : `${m} menit`
}
</script>

<template>
  <div class="min-h-screen bg-slate-50" :class="{ 'select-none': lockdownEnabled && lockdownActiveNow }">
    <!-- Mode Terkunci gate — Fullscreen API requires a user gesture, so lockdown can't be
         force-entered on page load. Shown whenever this run requires lockdown, is in_progress,
         and the browser isn't (or is no longer) in fullscreen — including right after a
         detected fullscreen-exit violation. Covers everything else on screen. -->
    <div
      v-if="lockdownEnabled && lockdownActiveNow && lockdown.showGate.value"
      class="fixed inset-0 z-[100] bg-slate-900/95 flex items-center justify-center p-6"
    >
      <Card class="max-w-md w-full p-8 text-center">
        <div class="w-14 h-14 rounded-2xl bg-rose-100 flex items-center justify-center mx-auto mb-4">
          <ShieldAlert class="w-7 h-7 text-rose-600" />
        </div>
        <h1 class="text-lg font-black text-slate-900 mb-2">Simulasi Ini Menggunakan Mode Terkunci</h1>
        <p class="text-sm text-muted-foreground mb-1">
          Selama mengerjakan, kamu tidak bisa mengakses halaman lain, berpindah tab, atau keluar dari layar penuh. Setiap pelanggaran
          akan tercatat dan bisa dilihat oleh sekolah/admin.
        </p>
        <p v-if="violationCount > 0" class="text-xs font-semibold text-rose-600 mb-4">{{ violationCount }} pelanggaran sudah tercatat pada percobaan ini.</p>
        <Button variant="gradient" class="w-full mt-4" @click="lockdown.enterFullscreen()">
          <Maximize class="w-4 h-4" />Masuk Mode Layar Penuh &amp; Lanjutkan
        </Button>
      </Card>
    </div>

    <!-- Loading -->
    <div v-if="loading && !run" class="min-h-screen flex items-center justify-center">
      <div class="flex flex-col items-center gap-3 text-slate-400">
        <Loader2 class="w-8 h-8 animate-spin" />
        <p class="text-sm font-medium">Memuat simulasi...</p>
      </div>
    </div>

    <!-- Not found / not owned -->
    <div v-else-if="!run" class="min-h-screen flex items-center justify-center p-6">
      <Card class="max-w-sm w-full p-8 text-center">
        <p class="text-sm text-muted-foreground mb-4">Simulasi tidak ditemukan.</p>
        <NuxtLink to="/siswa"><Button variant="gradient">Kembali ke Dashboard</Button></NuxtLink>
      </Card>
    </div>

    <template v-else>
      <!-- INTRO / OVERVIEW — shown once, right after the run is created and before slot 1 is
           activated. The exam clock genuinely has not started yet: clicking "Mulai Sekarang"
           below is what calls beginRun() and activates slot 1 server-side. -->
      <div v-if="isIntro" class="min-h-screen bg-gradient-to-br from-indigo-50 via-purple-50 to-pink-50 flex items-center justify-center p-4">
        <Card class="max-w-xl w-full p-5 sm:p-8">
          <div class="text-center mb-6">
            <div class="w-12 h-12 sm:w-14 sm:h-14 rounded-2xl bg-gradient-to-br from-indigo-500 to-purple-500 flex items-center justify-center mx-auto mb-4">
              <ListChecks class="w-6 h-6 sm:w-7 sm:h-7 text-white" />
            </div>
            <h1 class="text-lg sm:text-xl font-black text-slate-900">{{ run.template_title }}</h1>
            <p class="text-sm text-slate-500 mt-1">Baca ikhtisar di bawah sebelum memulai. Waktu pengerjaan baru mulai berjalan setelah kamu menekan tombol "Mulai Sekarang".</p>
          </div>

          <div class="grid grid-cols-2 gap-2 sm:gap-3 mb-6">
            <div class="bg-white rounded-xl p-2.5 sm:p-4 border text-center">
              <p class="text-lg sm:text-2xl font-black text-indigo-600">{{ run.slots.length }}</p>
              <p class="text-[11px] sm:text-xs text-slate-500 font-medium">Subtes</p>
            </div>
            <div class="bg-white rounded-xl p-2.5 sm:p-4 border text-center">
              <p class="text-lg sm:text-2xl font-black text-indigo-600">{{ totalDurationMinutes }}</p>
              <p class="text-[11px] sm:text-xs text-slate-500 font-medium">Menit Pengerjaan</p>
            </div>
          </div>

          <div class="bg-white rounded-xl border divide-y mb-6">
            <div v-for="s in run.slots" :key="s.sequence_index" class="flex items-center justify-between px-4 py-3">
              <div class="flex items-center gap-3 min-w-0">
                <span class="w-6 h-6 rounded-full bg-slate-100 text-slate-500 text-xs font-bold flex items-center justify-center shrink-0">{{ s.sequence_index }}</span>
                <p class="text-sm font-semibold text-slate-800 truncate">{{ s.session_title }}</p>
              </div>
              <span class="text-xs text-slate-500 shrink-0 ml-3">{{ s.duration_minutes }} menit</span>
            </div>
          </div>

          <div v-if="lockdownEnabled" class="flex items-start gap-3 p-4 rounded-xl bg-rose-50 border border-rose-200 mb-6">
            <ShieldAlert class="w-5 h-5 text-rose-600 shrink-0 mt-0.5" />
            <div class="text-xs text-rose-700">
              <p class="font-bold mb-1">Simulasi Ini Menggunakan Mode Terkunci</p>
              <p>Layar akan masuk mode layar penuh dan kamu tidak bisa berpindah tab, klik-kanan, atau copy-paste selama mengerjakan. Setiap pelanggaran akan tercatat dan bisa dilihat oleh sekolah/admin.</p>
            </div>
          </div>

          <Button variant="gradient" class="w-full" size="lg" :disabled="beginning" @click="beginNow">
            <PlayCircle class="w-5 h-5" />{{ beginning ? 'Memulai...' : (lockdownEnabled ? 'Masuk Layar Penuh & Mulai Sekarang' : 'Mulai Sekarang') }}
          </Button>
          <!-- Aman diklik kapan saja di layar Intro ini — waktu pengerjaan belum berjalan sama
               sekali (baru dimulai server saat "Mulai Sekarang" ditekan), jadi kembali ke
               dashboard tidak menggugurkan kesempatan. Simulasi ini tetap tersimpan sebagai
               "Lanjutkan" di Drilling Zone dan siswa akan kembali persis ke layar ini. -->
          <NuxtLink to="/siswa" class="block mt-3">
            <Button variant="outline" class="w-full" :disabled="beginning">Kembali, Tidak Jadi Sekarang</Button>
          </NuxtLink>
        </Card>
      </div>

      <!-- ACTIVE QUIZ -->
      <div v-else-if="isActive" class="min-h-screen bg-slate-50 flex flex-col">
        <div class="bg-white border-b sticky top-0 z-40">
          <div class="max-w-5xl mx-auto px-3 sm:px-4 py-2.5 sm:py-3 flex items-center gap-2 sm:gap-4">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1 flex-wrap">
                <p class="text-xs text-slate-500 font-medium truncate">{{ currentSlot?.session_title }}</p>
                <span class="text-[10px] font-bold px-1.5 py-0.5 rounded bg-indigo-100 text-indigo-700 shrink-0">
                  Subtes {{ currentSlot?.sequence_index }} / {{ run.slots.length }}
                </span>
                <span
                  v-if="lockdownEnabled"
                  class="text-[10px] font-bold px-1.5 py-0.5 rounded bg-rose-100 text-rose-700 shrink-0 flex items-center gap-1"
                  :title="violationCount > 0 ? `${violationCount} pelanggaran tercatat` : 'Mode Terkunci aktif'"
                >
                  <ShieldAlert class="w-3 h-3" />Terkunci{{ violationCount > 0 ? ` (${violationCount})` : '' }}
                </span>
              </div>
              <div class="flex items-center gap-2">
                <div class="flex-1 h-1.5 bg-slate-100 rounded-full overflow-hidden">
                  <div class="h-full bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full" :style="{ width: `${questions.length ? (answeredCount / questions.length) * 100 : 0}%` }" />
                </div>
                <span class="text-xs text-slate-500 font-mono shrink-0">{{ answeredCount }}/{{ questions.length }}</span>
              </div>
            </div>
            <div class="flex items-center gap-1.5 sm:gap-2 px-2.5 sm:px-4 py-1.5 sm:py-2 rounded-xl font-mono font-bold text-sm sm:text-lg shrink-0" :class="timeLeft < 300 ? 'bg-red-50 text-red-600' : 'bg-indigo-50 text-indigo-700'">
              <Clock class="w-4 h-4 sm:w-5 sm:h-5" />{{ formatTime(timeLeft) }}
            </div>
            <Button variant="gradient" class="shrink-0 px-3 sm:px-4" :disabled="submitting" @click="submitCurrent"><Send class="w-4 h-4" /><span class="hidden sm:inline">Submit Subtes</span></Button>
          </div>
        </div>

        <!-- Navigasi soal versi mobile — sidebar di kanan cuma muncul >= lg, sama seperti tryout
             player. Strip horizontal-scroll di bawah header, warna status sama persis dengan
             sidebar desktop. -->
        <div class="lg:hidden max-w-5xl mx-auto w-full px-4 pt-3">
          <div class="flex gap-1.5 overflow-x-auto pb-1 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
            <button
              v-for="(q, i) in questions"
              :key="q.id"
              class="w-9 h-9 rounded-lg text-xs font-bold relative shrink-0"
              :class="i === current ? 'bg-indigo-600 text-white' : answers[q.id] ? 'bg-emerald-100 text-emerald-700 border border-emerald-300' : 'bg-slate-100 text-slate-500'"
              @click="current = i"
            >
              {{ i + 1 }}
              <Flag v-if="flagged.has(q.id)" class="absolute -top-1.5 -right-1.5 w-3 h-3 text-amber-500 fill-amber-500" />
            </button>
          </div>
        </div>

        <div class="flex-1 max-w-5xl mx-auto w-full px-4 py-4 sm:py-6 grid lg:grid-cols-[240px_1fr] gap-6">
          <div class="hidden lg:block">
            <Card class="p-4 sticky top-[86px]">
              <p class="text-xs font-bold text-slate-500 mb-3 uppercase tracking-wider">Navigasi Soal</p>
              <div class="grid grid-cols-5 gap-1.5">
                <button
                  v-for="(q, i) in questions"
                  :key="q.id"
                  class="w-9 h-9 rounded-lg text-xs font-bold relative"
                  :class="i === current ? 'bg-indigo-600 text-white' : answers[q.id] ? 'bg-emerald-100 text-emerald-700 border border-emerald-300' : 'bg-slate-100 text-slate-500'"
                  @click="current = i"
                >
                  {{ i + 1 }}
                  <Flag v-if="flagged.has(q.id)" class="absolute -top-1.5 -right-1.5 w-3 h-3 text-amber-500 fill-amber-500" />
                </button>
              </div>
            </Card>
          </div>

          <Card v-if="questions[current]" class="overflow-hidden">
            <div class="px-4 sm:px-6 py-3 sm:py-4 border-b bg-gradient-to-r from-slate-50 to-indigo-50 flex items-center justify-between flex-wrap gap-2">
              <div class="flex items-center gap-3 text-xs">
                <span class="font-bold text-indigo-600 bg-indigo-100 px-2.5 py-1 rounded-lg">Soal {{ current + 1 }} / {{ questions.length }}</span>
                <span class="text-slate-500">{{ questions[current].subject }}</span>
              </div>
              <button
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold"
                :class="flagged.has(questions[current].id) ? 'bg-amber-100 text-amber-700' : 'bg-slate-100 text-slate-500'"
                @click="toggleFlag(questions[current].id)"
              >
                <Flag class="w-3.5 h-3.5" />{{ flagged.has(questions[current].id) ? 'Ditandai' : 'Tandai' }}
              </button>
            </div>

            <div class="p-4 sm:p-6">
              <div v-if="questions[current].stimulus" class="mb-5 p-4 bg-blue-50 border border-blue-200 rounded-xl">
                <p class="text-xs font-bold text-blue-600 mb-2 uppercase tracking-wider">Stimulus</p>
                <QuestionContent :text="questions[current].stimulus" class="text-sm text-slate-700" />
              </div>

              <QuestionContent :text="questions[current].question_text" class="text-base font-semibold text-slate-900 mb-6" />

              <!-- Menjodohkan -->
              <div v-if="questions[current].question_type === 'matching'" class="space-y-3">
                <div v-for="(left, idx) in currentMatching.lefts" :key="idx" class="flex items-center gap-3 p-3 rounded-xl border-2 border-slate-200">
                  <QuestionContent :text="left" size="sm" class="font-medium text-slate-700 flex-1 min-w-0" />
                  <ArrowLeftRight class="w-3.5 h-3.5 text-slate-300 shrink-0" />
                  <select
                    class="px-3 py-2 border-2 border-slate-200 rounded-lg text-sm focus:border-indigo-500 focus:outline-none min-w-[9rem]"
                    :value="matchingCurrentPairs(questions[current].id, currentMatching.lefts)[idx]"
                    @change="selectMatchingPair(questions[current].id, currentMatching.lefts, idx, ($event.target as HTMLSelectElement).value)"
                  >
                    <option value="">Pilih pasangan...</option>
                    <option v-for="r in currentMatching.rights" :key="r" :value="r">{{ r }}</option>
                  </select>
                </div>
              </div>

              <!-- Menjodohkan Gambar -->
              <div v-else-if="questions[current].question_type === 'matching_image'" class="space-y-3">
                <div v-for="(left, idx) in currentMatching.lefts" :key="idx" class="flex items-center gap-3 p-3 rounded-xl border-2 border-slate-200">
                  <img :src="left" class="w-20 h-20 object-cover rounded-lg shrink-0" />
                  <ArrowLeftRight class="w-3.5 h-3.5 text-slate-300 shrink-0" />
                  <select
                    class="px-3 py-2 border-2 border-slate-200 rounded-lg text-sm focus:border-indigo-500 focus:outline-none min-w-[9rem]"
                    :value="matchingCurrentPairs(questions[current].id, currentMatching.lefts)[idx]"
                    @change="selectMatchingPair(questions[current].id, currentMatching.lefts, idx, ($event.target as HTMLSelectElement).value)"
                  >
                    <option value="">Pilih pasangan...</option>
                    <option v-for="r in currentMatching.rights" :key="r" :value="r">{{ r }}</option>
                  </select>
                  <img v-if="matchingCurrentPairs(questions[current].id, currentMatching.lefts)[idx]" :src="matchingCurrentPairs(questions[current].id, currentMatching.lefts)[idx]" class="w-20 h-20 object-cover rounded-lg shrink-0" />
                </div>
              </div>

              <!-- Benar/Salah Ganda -->
              <div v-else-if="questions[current].question_type === 'true_false_complex'" class="space-y-3">
                <div v-for="(st, idx) in (questions[current].options || [])" :key="idx" class="flex items-center gap-3 p-3 rounded-xl border-2 border-slate-200">
                  <QuestionContent :text="st" size="sm" class="font-medium text-slate-700 flex-1 min-w-0" />
                  <div class="flex gap-1.5 shrink-0">
                    <button
                      v-for="v in (['Benar', 'Salah'] as const)" :key="v" type="button"
                      class="px-3 py-1.5 rounded-lg text-xs font-bold border-2 transition-colors"
                      :class="tfComplexCurrentAnswers(questions[current].id, (questions[current].options || []).length)[idx] === v ? 'bg-indigo-600 border-indigo-600 text-white' : 'bg-white border-slate-200 text-slate-500 hover:border-indigo-300'"
                      @click="selectTfComplexAnswer(questions[current].id, (questions[current].options || []).length, idx, v)"
                    >{{ v }}</button>
                  </div>
                </div>
              </div>

              <!-- Urutkan -->
              <div v-else-if="questions[current].question_type === 'ordering'" class="space-y-2">
                <div
                  v-for="(step, idx) in orderingCurrentOrder(questions[current].id, questions[current].options || [])"
                  :key="step + idx"
                  class="flex items-center gap-3 p-3 rounded-xl border-2 border-slate-200"
                >
                  <span class="w-6 h-6 rounded-lg bg-indigo-100 text-indigo-700 text-xs font-black flex items-center justify-center shrink-0">{{ idx + 1 }}</span>
                  <QuestionContent :text="step" size="sm" class="font-medium text-slate-700 flex-1 min-w-0" />
                  <div class="flex gap-0.5 shrink-0">
                    <button type="button" :disabled="idx === 0" class="p-1.5 rounded-lg hover:bg-slate-100 disabled:opacity-30" @click="moveOrderingAnswer(questions[current].id, idx, -1)">
                      <ChevronLeft class="w-3.5 h-3.5 text-slate-500 rotate-90" />
                    </button>
                    <button type="button" :disabled="idx === (questions[current].options || []).length - 1" class="p-1.5 rounded-lg hover:bg-slate-100 disabled:opacity-30" @click="moveOrderingAnswer(questions[current].id, idx, 1)">
                      <ChevronLeft class="w-3.5 h-3.5 text-slate-500 -rotate-90" />
                    </button>
                  </div>
                </div>
              </div>

              <!-- Uraian -->
              <textarea
                v-else-if="questions[current].question_type === 'essay'"
                :value="answers[questions[current].id] || ''"
                placeholder="Tulis jawaban esai kamu di sini..."
                rows="6"
                class="w-full px-4 py-3 border-2 border-slate-200 rounded-xl focus:border-indigo-500 focus:outline-none font-medium"
                @input="selectAnswer(questions[current].id, ($event.target as HTMLTextAreaElement).value)"
              />

              <div v-else-if="questions[current].options && questions[current].options!.length" class="space-y-3">
                <button
                  v-for="(opt, idx) in questions[current].options"
                  :key="opt"
                  class="w-full text-left flex items-start gap-3 p-4 rounded-xl border-2 transition-all"
                  :class="answers[questions[current].id] === opt ? 'border-indigo-500 bg-indigo-50' : 'border-slate-200 hover:border-indigo-300'"
                  @click="selectAnswer(questions[current].id, opt)"
                >
                  <span class="w-7 h-7 rounded-lg flex items-center justify-center text-xs font-black shrink-0" :class="answers[questions[current].id] === opt ? 'bg-indigo-600 text-white' : 'bg-slate-100 text-slate-600'">
                    {{ ['A', 'B', 'C', 'D', 'E'][idx] }}
                  </span>
                  <QuestionContent :text="opt" size="sm" class="leading-relaxed" />
                </button>
              </div>

              <input
                v-else
                :value="answers[questions[current].id] || ''"
                placeholder="Ketik jawaban kamu di sini..."
                class="w-full px-4 py-3 border-2 border-slate-200 rounded-xl focus:border-indigo-500 focus:outline-none font-medium"
                @input="selectAnswer(questions[current].id, ($event.target as HTMLInputElement).value)"
              />
            </div>

            <div class="px-4 sm:px-6 py-3 sm:py-4 border-t bg-slate-50 flex items-center justify-between gap-2">
              <Button variant="outline" size="sm" class="sm:h-10 sm:px-4" :disabled="current === 0" @click="current = Math.max(0, current - 1)">
                <ChevronLeft class="w-4 h-4" />Sebelumnya
              </Button>
              <Button v-if="current < questions.length - 1" size="sm" class="sm:h-10 sm:px-4" @click="current = Math.min(questions.length - 1, current + 1)">
                Selanjutnya <ChevronRight class="w-4 h-4" />
              </Button>
              <Button v-else variant="gradient" size="sm" class="sm:h-10 sm:px-4" :disabled="submitting" @click="submitCurrent"><Send class="w-4 h-4" />Submit Subtes</Button>
            </div>
          </Card>
        </div>
      </div>

      <!-- BREAK -->
      <div v-else-if="isBreak" class="min-h-screen bg-gradient-to-br from-indigo-50 via-purple-50 to-pink-50 flex items-center justify-center p-4">
        <Card class="max-w-lg w-full p-5 sm:p-8 text-center">
          <div class="w-14 h-14 sm:w-16 sm:h-16 rounded-2xl bg-gradient-to-br from-emerald-500 to-teal-500 flex items-center justify-center mx-auto mb-5">
            <Coffee class="w-7 h-7 sm:w-8 sm:h-8 text-white" />
          </div>
          <span
            v-if="lockdownEnabled"
            class="inline-flex items-center gap-1 text-[10px] font-bold px-2 py-0.5 rounded-full bg-rose-100 text-rose-700 mb-2"
          >
            <ShieldAlert class="w-3 h-3" />Mode Terkunci aktif{{ violationCount > 0 ? ` — ${violationCount} pelanggaran tercatat` : '' }}
          </span>
          <h1 class="text-xl font-black text-slate-900 mb-1">Subtes {{ currentSlot?.sequence_index }} Selesai!</h1>
          <p v-if="currentSlot?.score != null && run.template_kind === 'per_subject'" class="text-sm text-slate-500 mb-1">
            Skor subtes ini: <span class="font-bold text-slate-700">{{ tkaScaledScore(currentSlot.score, run.school_type_scope) }}</span> <span class="text-xs">(skala {{ tkaScaleFor(run.school_type_scope).label }})</span>
          </p>
          <p v-else-if="currentSlot?.score != null" class="text-sm text-slate-500 mb-1">Skor subtes ini: <span class="font-bold text-slate-700">{{ currentSlot.score }}</span></p>
          <p class="text-slate-500 mb-6 text-sm">Waktu istirahat sebelum lanjut ke subtes berikutnya.</p>

          <div class="bg-white rounded-2xl p-4 sm:p-6 mb-6 border">
            <p class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-2">Sisa Istirahat</p>
            <p class="text-3xl sm:text-4xl font-black font-mono text-indigo-600">{{ formatTime(breakLeft) }}</p>
          </div>

          <p v-if="nextSlot" class="text-sm text-slate-600 mb-6">
            Subtes berikutnya: <span class="font-bold">{{ nextSlot.session_title }}</span>
          </p>
          <p v-else class="text-sm text-slate-600 mb-6">Ini adalah subtes terakhir — simulasi akan selesai setelah ini.</p>

          <!-- Stepper: overall progress across all slots in the run -->
          <div class="flex items-center justify-center gap-2 mb-6 flex-wrap">
            <div v-for="s in run.slots" :key="s.sequence_index" class="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold" :class="slotStepClass(s.status)">
              {{ s.sequence_index }}
            </div>
          </div>

          <Button variant="gradient" class="w-full" :disabled="breakLeft > 0 || advancing" @click="advanceNow()">
            {{ breakLeft > 0 ? `Lanjut Otomatis dalam ${formatTime(breakLeft)}` : 'Lanjut ke Subtes Berikutnya' }}
          </Button>
        </Card>
      </div>

      <!-- FINISHED -->
      <div v-else-if="isFinished" class="min-h-screen bg-gradient-to-br from-indigo-50 via-purple-50 to-pink-50 py-6 sm:py-10 px-3 sm:px-4">
        <div class="max-w-2xl mx-auto">
          <div class="bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 rounded-2xl sm:rounded-3xl p-5 sm:p-8 text-white mb-6">
            <div class="flex items-center justify-between gap-3 mb-4">
              <div class="min-w-0">
                <p class="text-indigo-200 text-xs sm:text-sm font-medium truncate">{{ run.template_title }}</p>
                <h2 class="text-xl sm:text-2xl font-black mt-0.5 flex items-center gap-2">Simulasi Selesai! <PartyPopper class="w-4 h-4 sm:w-5 sm:h-5 shrink-0" /></h2>
              </div>
              <Award class="w-8 h-8 sm:w-10 sm:h-10 text-white/70 shrink-0" />
            </div>

            <div v-if="run.template_kind === 'per_subject'" class="bg-white/15 rounded-2xl p-4 sm:p-5 text-center">
              <p class="text-xs font-bold text-indigo-100 uppercase tracking-wider mb-1">Skor Per Mata Uji</p>
              <p class="text-sm text-indigo-100">TKA tidak menggabungkan skor lintas mata uji — lihat rincian per subtes di bawah.</p>
            </div>
            <div v-else class="bg-white/15 rounded-2xl p-4 sm:p-5 text-center">
              <p class="text-xs font-bold text-indigo-100 uppercase tracking-wider mb-1">Estimasi Skor UTBK Gabungan</p>
              <p v-if="run.combined_estimate_score != null" class="text-3xl sm:text-4xl font-black">{{ run.combined_estimate_score }}</p>
              <p v-else class="text-sm text-indigo-100">Belum ada skor yang dapat dihitung untuk simulasi ini.</p>
            </div>
            <p v-if="formatDateTimeRange(run.started_at, run.completed_at)" class="text-center text-xs text-indigo-100 mt-3">
              Total waktu pengerjaan: {{ formatDateTimeRange(run.started_at, run.completed_at) }}
            </p>
          </div>

          <Card class="p-4 sm:p-6 mb-6">
            <h3 class="font-bold text-slate-800 mb-4 flex items-center gap-2"><BarChart3 class="w-5 h-5 text-indigo-500" />Rincian per Subtes</h3>
            <div class="space-y-3">
              <div v-for="s in run.slots" :key="s.sequence_index" class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 p-3 rounded-xl border">
                <div class="min-w-0">
                  <p class="font-semibold text-sm text-slate-900 truncate">{{ s.session_title }}</p>
                  <p class="text-xs text-slate-500">{{ s.subject_filter || 'Campuran' }}</p>
                </div>
                <div class="text-left sm:text-right shrink-0 sm:ml-3">
                  <template v-if="run.template_kind === 'per_subject'">
                    <p v-if="s.score != null" class="font-black text-slate-900">
                      {{ tkaScaledScore(s.score, run.school_type_scope) }}
                      <span class="block text-[10px] font-normal text-slate-400">skala {{ tkaScaleFor(run.school_type_scope).label }} · skor mentah {{ s.score }}/1000</span>
                      <span
                        class="inline-block mt-0.5 text-[10px] font-bold px-1.5 py-0.5 rounded"
                        :class="tkaCategoryLabel(s.score, run.school_type_scope) === 'Istimewa' ? 'bg-amber-100 text-amber-700' : 'bg-slate-100 text-slate-500'"
                      >{{ tkaCategoryLabel(s.score, run.school_type_scope) }}</span>
                    </p>
                    <p v-else class="text-xs text-slate-400 italic">Belum dinilai</p>
                  </template>
                  <template v-else>
                    <p v-if="s.score != null" class="font-black text-slate-900">{{ s.score }}</p>
                    <p v-else class="text-xs text-slate-400 italic">Belum dinilai</p>
                  </template>
                </div>
              </div>
            </div>
          </Card>

          <div class="flex flex-col sm:flex-row gap-3">
            <Button variant="gradient" class="flex-1" :disabled="downloadingCertificate" @click="downloadCertificate">
              <Download class="w-4 h-4" />{{ downloadingCertificate ? 'Membuat Sertifikat...' : 'Unduh Sertifikat & Laporan' }}
            </Button>
            <NuxtLink to="/siswa" class="flex-1"><Button variant="outline" class="w-full"><Home class="w-4 h-4" />Kembali ke Dashboard</Button></NuxtLink>
          </div>
        </div>
      </div>

      <!-- Fallback: run in an unexpected/transitional slot state — ticker polls loadRun() again
           automatically (see setupTicker's stuck-state branch) every ~3s; after
           STUCK_MAX_ATTEMPTS with no resolution, offer a real escape hatch instead of a
           spinner with no way out. -->
      <div v-else class="min-h-screen flex items-center justify-center p-6">
        <div v-if="!stuckGivenUp" class="flex flex-col items-center gap-3 text-slate-400">
          <Loader2 class="w-8 h-8 animate-spin" />
          <p class="text-sm font-medium">Menyinkronkan status simulasi...</p>
        </div>
        <Card v-else class="max-w-sm w-full p-8 text-center">
          <p class="text-sm text-muted-foreground mb-4">Simulasi ini butuh waktu lebih lama dari biasanya untuk sinkron. Coba muat ulang, atau kembali dulu — progres yang sudah tersimpan tidak hilang.</p>
          <div class="flex flex-col gap-2">
            <Button variant="gradient" @click="stuckAttempts = 0; loadRun()">Coba Lagi</Button>
            <NuxtLink to="/siswa"><Button variant="outline" class="w-full">Kembali ke Dashboard</Button></NuxtLink>
          </div>
        </Card>
      </div>
    </template>
  </div>
</template>
