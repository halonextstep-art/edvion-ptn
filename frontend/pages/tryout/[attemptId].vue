<script setup lang="ts">
import {
  Clock, Flag, ChevronLeft, ChevronRight, CheckCircle, XCircle,
  AlertCircle, BarChart3, BookOpen, Send, Home, Eye, Award, Target,
  ArrowRight, ArrowLeftRight, PartyPopper, Loader2, Gauge,
} from 'lucide-vue-next'
import type { AttemptResult, ReviewItem } from '~/types'

// Menjodohkan (matching) wire format — see backend tryout_service.rs matching_playable_options():
// options = [...leftItems, '__MATCH_SPLIT__', ...rightItems] with the right list pre-shuffled
// server-side so positional (same-index) correspondence never reveals the correct pairing.
const MATCH_SPLIT = '__MATCH_SPLIT__'
function matchingLeftsRights(options?: string[] | null) {
  const opts = options || []
  const splitIdx = opts.indexOf(MATCH_SPLIT)
  if (splitIdx === -1) return { lefts: [] as string[], rights: [] as string[] }
  return { lefts: opts.slice(0, splitIdx), rights: opts.slice(splitIdx + 1) }
}

definePageMeta({ middleware: 'auth', roles: ['student'] })

const route = useRoute()
const router = useRouter()
const attemptId = route.params.attemptId as string

const attemptSession = useAttemptSessionStore()
const { tryoutService } = useApi()
const toast = useToast()

type Phase = 'intro' | 'quiz' | 'results' | 'review'
const phase = ref<Phase>('intro')

const session = computed(() => attemptSession.current)
const questions = computed(() => session.value?.questions ?? [])
const durationMinutes = computed(() => session.value?.attempt.duration_minutes ?? 0)
const totalSeconds = computed(() => durationMinutes.value * 60)

const current = ref(0)
const answers = ref<Record<string, string>>({})
const flagged = ref<Set<string>>(new Set())
const timeLeft = ref(0)
const startedAt = ref(0)
let timer: ReturnType<typeof setInterval> | null = null

const result = ref<AttemptResult | null>(null)
const reviewItems = ref<ReviewItem[]>([])
const reviewIndex = ref(0)
const submitting = ref(false)

function formatTime(seconds: number) {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}

function runTimer() {
  timer = setInterval(() => {
    if (timeLeft.value <= 1) {
      timeLeft.value = 0
      submit()
      return
    }
    timeLeft.value -= 1
  }, 1000)
}

function startQuiz() {
  timeLeft.value = totalSeconds.value
  startedAt.value = Date.now()
  phase.value = 'quiz'
  runTimer()
}

onUnmounted(() => {
  if (timer) clearInterval(timer)
})

// Fallback for when the player is opened/reopened with `attemptSession` (Pinia, in-memory
// only by design — see stores/attemptSession.ts) already empty, e.g. the page was refreshed
// or the student closed the tab mid-attempt and came back via "Lanjutkan" in Drilling Zone.
// Previously this always dead-ended on "sesi tidak ditemukan"; now it fetches the same
// key-stripped questions PLUS whatever answers were already autosaved, so the student resumes
// exactly where they left off instead of losing everything.
const resuming = ref(true)
onMounted(async () => {
  if (attemptSession.current) {
    resuming.value = false
    return
  }
  try {
    const r = await tryoutService.resumeAttempt(attemptId)
    attemptSession.set({ attempt: r.attempt, questions: r.questions })
    const restoredAnswers: Record<string, string> = {}
    const restoredFlags = new Set<string>()
    for (const a of r.answers) {
      if (a.answer_text != null) restoredAnswers[a.question_id] = a.answer_text
      if (a.flagged) restoredFlags.add(a.question_id)
    }
    answers.value = restoredAnswers
    flagged.value = restoredFlags
    // Countdown resumes from the server-authoritative `started_at`, not a fresh full
    // duration — otherwise every refresh would silently grant extra time.
    const elapsedSeconds = Math.max(0, Math.floor((Date.now() - new Date(r.attempt.started_at).getTime()) / 1000))
    startedAt.value = Date.now() - elapsedSeconds * 1000
    timeLeft.value = Math.max(0, totalSeconds.value - elapsedSeconds)
    phase.value = 'quiz'
    if (timeLeft.value <= 0) {
      // Attempt ini sudah lama ditinggal (started_at + durasi < sekarang) — bukan bug, tapi
      // tanpa pesan ini siswa cuma mendarat di layar hasil serba-0 tanpa penjelasan sama
      // sekali kenapa jawabannya tidak "muncul lagi".
      toast.info('Waktu sudah habis', 'Attempt ini sudah lama ditinggal dan waktunya sudah lewat, jadi otomatis disubmit dengan jawaban yang sempat tersimpan.')
      submit()
    } else {
      runTimer()
    }
  } catch {
    // No resumable attempt (already submitted, wrong owner, or truly doesn't exist) — falls
    // through to the existing "sesi tidak ditemukan" card below.
  } finally {
    resuming.value = false
  }
})

async function selectAnswer(questionId: string, value: string) {
  answers.value = { ...answers.value, [questionId]: value }
  try {
    await tryoutService.saveAnswer(attemptId, questionId, value, flagged.value.has(questionId))
  } catch {
    // autosave failure is non-blocking; the final submit still sends everything
  }
}

// Parses the currently-saved answer_text (a "kiri::kanan, kiri2::kanan2, ..." string) back
// into a per-row selected-right-value array, so re-visiting a matching question restores its
// dropdowns. Falls back to '' for any row the student hasn't picked yet.
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
  // Must serialize identically to how QuestionFormPanel.vue builds correct_answer
  // ("kiri::kanan" pairs joined by ", ") since grading is plain string equality.
  const serialized = lefts.map((l, i) => `${l}::${chosen[i] || ''}`).join(', ')
  selectAnswer(questionId, serialized)
}

// Benar/Salah Ganda — parses the currently-saved ", "-joined "Benar"/"Salah" answer back into
// a per-statement array so re-visiting the question restores each toggle. Defaults every
// statement to unanswered ('') until the student picks one, mirroring matchingCurrentPairs().
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

// Urutkan — options arrive from the server already shuffled (see
// tryout_service.rs::ordering_playable_options). Keeps a local per-question working order the
// student can reorder; restores from a saved answer (", "-joined) if the student already
// submitted an arrangement, otherwise starts from the server-shuffled option order.
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
  try {
    await tryoutService.saveAnswer(attemptId, questionId, answers.value[questionId] ?? null, next.has(questionId))
  } catch {
    // ignore
  }
}

const answeredCount = computed(() => Object.keys(answers.value).length)
const currentMatching = computed(() => matchingLeftsRights(questions.value[current.value]?.options))

async function submit() {
  if (submitting.value) return
  submitting.value = true
  if (timer) clearInterval(timer)
  const timeUsed = Math.max(0, totalSeconds.value - timeLeft.value)
  try {
    result.value = await tryoutService.submitAttempt(attemptId, timeUsed)
    phase.value = 'results'
  } catch (e: any) {
    toast.error('Gagal submit jawaban', e?.message)
  } finally {
    submitting.value = false
  }
}

async function openReview() {
  try {
    reviewItems.value = await tryoutService.getReview(attemptId)
    reviewIndex.value = 0
    phase.value = 'review'
  } catch (e: any) {
    toast.error('Gagal memuat pembahasan', e?.message)
  }
}

function finish() {
  attemptSession.clear()
  router.push('/siswa')
}

const grade = computed(() => {
  const score = result.value?.attempt.score ?? 0
  if (score >= 700) return 'A'
  if (score >= 600) return 'B'
  if (score >= 500) return 'C'
  return 'D'
})

// Dual-score display ("Skor Instan" vs "Estimasi IRT") — dikontrol admin lewat
// score_display_mode, lihat backend domain::platform_settings::ScoreDisplayMode +
// domain::irt doc comment.
const showInstantScore = computed(() => (result.value?.score_display_mode ?? 'instant') !== 'irt')
const irtEnabled = computed(() => {
  const mode = result.value?.score_display_mode
  return mode === 'irt' || mode === 'both'
})
const showIrtAlongsideInstant = computed(() => result.value?.score_display_mode === 'both')
</script>

<template>
  <div class="min-h-screen bg-slate-50">
    <!-- Trying to rebuild state from the server before giving up (see onMounted above) -->
    <div v-if="resuming" class="min-h-screen flex items-center justify-center p-6">
      <div class="text-center text-muted-foreground">
        <Loader2 class="w-6 h-6 animate-spin mx-auto mb-2" />
        <p class="text-sm">Memuat sesi...</p>
      </div>
    </div>

    <!-- No session data and resume failed (already submitted / truly not found) -->
    <div v-else-if="!session" class="min-h-screen flex items-center justify-center p-6">
      <Card class="max-w-sm w-full p-8 text-center">
        <p class="text-sm text-muted-foreground mb-4">
          Sesi tryout ini tidak bisa dilanjutkan — kemungkinan sudah pernah disubmit sebelumnya. Cek riwayat latihan kamu di Drilling Zone.
        </p>
        <NuxtLink to="/siswa"><Button variant="gradient">Kembali ke Dashboard</Button></NuxtLink>
      </Card>
    </div>

    <template v-else>
      <!-- INTRO -->
      <div v-if="phase === 'intro'" class="min-h-screen bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 flex items-center justify-center p-4">
        <Card class="max-w-lg w-full p-6 sm:p-10 text-center">
          <div class="w-16 h-16 sm:w-20 sm:h-20 rounded-2xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center mx-auto mb-5 sm:mb-6">
            <Target class="w-8 h-8 sm:w-10 sm:h-10 text-white" />
          </div>
          <div class="inline-block px-3 py-1 rounded-full bg-indigo-100 text-indigo-700 text-xs font-bold mb-3 uppercase tracking-wider">
            {{ session.attempt.session_type }}
          </div>
          <h1 class="text-xl sm:text-2xl font-black text-slate-900 mb-2">{{ session.attempt.session_title }}</h1>
          <p class="text-slate-500 mb-6 sm:mb-8 text-sm">Waktu berjalan setelah tombol Mulai ditekan.</p>

          <div class="grid grid-cols-3 gap-2 sm:gap-4 mb-6 sm:mb-8">
            <div class="bg-slate-50 rounded-xl p-2.5 sm:p-4">
              <BookOpen class="w-5 h-5 text-indigo-500 mx-auto mb-1.5 sm:mb-2" />
              <p class="text-lg sm:text-2xl font-black text-slate-900">{{ questions.length }}</p>
              <p class="text-[11px] sm:text-xs text-slate-500 mt-0.5">Soal</p>
            </div>
            <div class="bg-slate-50 rounded-xl p-2.5 sm:p-4">
              <Clock class="w-5 h-5 text-indigo-500 mx-auto mb-1.5 sm:mb-2" />
              <p class="text-lg sm:text-2xl font-black text-slate-900">{{ durationMinutes }}m</p>
              <p class="text-[11px] sm:text-xs text-slate-500 mt-0.5">Waktu</p>
            </div>
            <div class="bg-slate-50 rounded-xl p-2.5 sm:p-4">
              <Target class="w-5 h-5 text-indigo-500 mx-auto mb-1.5 sm:mb-2" />
              <p class="text-lg sm:text-2xl font-black text-slate-900">{{ new Set(questions.map((q) => q.subject)).size }}</p>
              <p class="text-[11px] sm:text-xs text-slate-500 mt-0.5">Subtes</p>
            </div>
          </div>

          <Button variant="gradient" size="lg" class="w-full gap-1.5" @click="startQuiz">Mulai Sekarang<ArrowRight class="w-4 h-4" /></Button>
          <NuxtLink to="/siswa" class="block mt-3 text-sm text-slate-400 hover:text-slate-600">Kembali</NuxtLink>
        </Card>
      </div>

      <!-- QUIZ -->
      <div v-else-if="phase === 'quiz'" class="min-h-screen bg-slate-50 flex flex-col">
        <div class="bg-white border-b sticky top-0 z-40">
          <div class="max-w-5xl mx-auto px-3 sm:px-4 py-2.5 sm:py-3 flex items-center gap-2 sm:gap-4">
            <div class="flex-1 min-w-0">
              <p class="text-xs text-slate-500 font-medium mb-1 truncate">{{ session.attempt.session_title }}</p>
              <div class="flex items-center gap-2">
                <div class="flex-1 h-1.5 bg-slate-100 rounded-full overflow-hidden">
                  <div class="h-full bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full" :style="{ width: `${(answeredCount / questions.length) * 100}%` }" />
                </div>
                <span class="text-xs text-slate-500 font-mono shrink-0">{{ answeredCount }}/{{ questions.length }}</span>
              </div>
            </div>
            <div class="flex items-center gap-1.5 sm:gap-2 px-2.5 sm:px-4 py-1.5 sm:py-2 rounded-xl font-mono font-bold text-sm sm:text-lg shrink-0" :class="timeLeft < 300 ? 'bg-red-50 text-red-600' : 'bg-indigo-50 text-indigo-700'">
              <Clock class="w-4 h-4 sm:w-5 sm:h-5" />{{ formatTime(timeLeft) }}
            </div>
            <Button variant="gradient" class="shrink-0 px-3 sm:px-4" :disabled="submitting" @click="submit"><Send class="w-4 h-4" /><span class="hidden sm:inline">Submit</span></Button>
          </div>
        </div>

        <!-- Navigasi soal versi mobile — sidebar di kanan cuma muncul >= lg, jadi tanpa ini
             siswa di HP tidak punya cara melompat ke nomor soal tertentu sama sekali (cuma
             Sebelumnya/Selanjutnya). Strip horizontal-scroll di bawah header, warna status sama
             persis dengan sidebar desktop. -->
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
            <Card class="p-4 sticky top-[76px]">
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

              <!-- Menjodohkan: left items fixed, each paired with a dropdown of (shuffled) right items -->
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

              <!-- Menjodohkan Gambar: identical to Menjodohkan, but each side renders as an image -->
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

              <!-- Benar/Salah Ganda: each statement independently marked Benar/Salah -->
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

              <!-- Urutkan: student reorders the server-shuffled steps into what they believe is the correct sequence -->
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

              <!-- Uraian: free-text essay, self-check only (see Drilling-only gating server-side) -->
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
              <Button v-else variant="gradient" size="sm" class="sm:h-10 sm:px-4" :disabled="submitting" @click="submit"><Send class="w-4 h-4" />Submit Semua</Button>
            </div>
          </Card>
        </div>
      </div>

      <!-- RESULTS -->
      <div v-else-if="phase === 'results' && result" class="min-h-screen bg-gradient-to-br from-indigo-50 via-purple-50 to-pink-50 py-6 sm:py-10 px-3 sm:px-4">
        <div class="max-w-2xl mx-auto">
          <div class="bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 rounded-2xl sm:rounded-3xl p-5 sm:p-8 text-white mb-6">
            <div class="flex items-center justify-between gap-3 mb-5 sm:mb-6">
              <div class="min-w-0">
                <p class="text-indigo-200 text-xs sm:text-sm font-medium truncate">Hasil {{ result.attempt.session_title }}</p>
                <h2 class="text-xl sm:text-2xl font-black mt-0.5 flex items-center gap-2">Tryout Selesai! <PartyPopper class="w-4 h-4 sm:w-5 sm:h-5 shrink-0" /></h2>
              </div>
              <div class="text-4xl sm:text-6xl font-black bg-white/20 w-14 h-14 sm:w-20 sm:h-20 rounded-2xl flex items-center justify-center shrink-0">{{ grade }}</div>
            </div>
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 sm:gap-4">
              <div class="bg-white/15 rounded-xl p-2.5 sm:p-3 text-center">
                <Award class="w-4 h-4 mx-auto mb-1.5 text-indigo-200" />
                <p class="text-lg sm:text-xl font-black">{{ showInstantScore ? result.attempt.score : (result.irt_score != null ? Math.round(result.irt_score) : '—') }}</p>
                <p class="text-[11px] sm:text-xs text-indigo-200 mt-0.5">{{ showInstantScore ? 'Skor Instan' : 'Estimasi IRT' }}</p>
              </div>
              <div class="bg-white/15 rounded-xl p-2.5 sm:p-3 text-center">
                <Target class="w-4 h-4 mx-auto mb-1.5 text-indigo-200" />
                <p class="text-lg sm:text-xl font-black">{{ result.attempt.accuracy }}%</p>
                <p class="text-[11px] sm:text-xs text-indigo-200 mt-0.5">Akurasi</p>
              </div>
              <div class="bg-white/15 rounded-xl p-2.5 sm:p-3 text-center">
                <CheckCircle class="w-4 h-4 mx-auto mb-1.5 text-indigo-200" />
                <p class="text-lg sm:text-xl font-black">{{ result.attempt.correct_count }}</p>
                <p class="text-[11px] sm:text-xs text-indigo-200 mt-0.5">Benar</p>
              </div>
              <div class="bg-white/15 rounded-xl p-2.5 sm:p-3 text-center">
                <Clock class="w-4 h-4 mx-auto mb-1.5 text-indigo-200" />
                <p class="text-lg sm:text-xl font-black">{{ formatTime(result.attempt.time_used_seconds || 0) }}</p>
                <p class="text-[11px] sm:text-xs text-indigo-200 mt-0.5">Waktu</p>
              </div>
            </div>

            <!-- Estimasi IRT berdampingan dengan Skor Instan (mode 'both') — lihat backend
                 domain::irt doc comment: estimasi kalibrasi platform sendiri, bukan skor UTBK
                 resmi SNPMB. -->
            <div v-if="showIrtAlongsideInstant" class="mt-4 pt-4 border-t border-white/20 flex items-center justify-between text-sm">
              <span class="text-indigo-200 flex items-center gap-1.5"><Gauge class="w-4 h-4" />Estimasi IRT</span>
              <span class="font-black">{{ result.irt_score != null ? Math.round(result.irt_score) : 'Belum tersedia' }}</span>
            </div>
            <p v-else-if="irtEnabled && result.irt_score == null" class="mt-4 pt-4 border-t border-white/20 text-xs text-indigo-200">
              Estimasi IRT belum tersedia untuk hasil ini (data kalibrasi belum cukup).
            </p>
          </div>

          <Card class="p-6 mb-5">
            <h3 class="font-bold text-slate-800 mb-4 flex items-center gap-2"><BarChart3 class="w-5 h-5 text-indigo-500" />Nilai per Subtes</h3>
            <div class="space-y-3">
              <div v-for="b in result.subject_breakdown" :key="b.subject">
                <div class="flex justify-between text-sm mb-1.5">
                  <span class="font-medium text-slate-700">{{ b.subject }}</span>
                  <span class="font-bold">{{ b.correct }}/{{ b.total }}</span>
                </div>
                <div class="h-2.5 bg-slate-100 rounded-full overflow-hidden">
                  <div class="h-full rounded-full bg-gradient-to-r from-emerald-500 to-green-400" :style="{ width: `${b.total > 0 ? Math.round((b.correct / b.total) * 100) : 0}%` }" />
                </div>
              </div>
            </div>
          </Card>

          <div class="grid grid-cols-3 gap-2 sm:gap-4 mb-6">
            <div class="border rounded-2xl p-2.5 sm:p-4 text-center text-emerald-600 bg-emerald-50 border-emerald-200">
              <CheckCircle class="w-5 h-5 sm:w-6 sm:h-6 mx-auto mb-1.5 sm:mb-2" />
              <p class="text-xl sm:text-3xl font-black">{{ result.attempt.correct_count }}</p>
              <p class="text-xs sm:text-sm font-medium mt-0.5">Benar</p>
            </div>
            <div class="border rounded-2xl p-2.5 sm:p-4 text-center text-red-500 bg-red-50 border-red-200">
              <XCircle class="w-5 h-5 sm:w-6 sm:h-6 mx-auto mb-1.5 sm:mb-2" />
              <p class="text-xl sm:text-3xl font-black">{{ result.attempt.wrong_count }}</p>
              <p class="text-xs sm:text-sm font-medium mt-0.5">Salah</p>
            </div>
            <div class="border rounded-2xl p-2.5 sm:p-4 text-center text-slate-500 bg-slate-50 border-slate-200">
              <AlertCircle class="w-5 h-5 sm:w-6 sm:h-6 mx-auto mb-1.5 sm:mb-2" />
              <p class="text-xl sm:text-3xl font-black">{{ result.attempt.unanswered_count }}</p>
              <p class="text-xs sm:text-sm font-medium mt-0.5">Tidak Dijawab</p>
            </div>
          </div>

          <div class="flex flex-col sm:flex-row gap-3">
            <Button variant="outline" class="flex-1" @click="openReview"><Eye class="w-5 h-5" />Review Pembahasan</Button>
            <Button variant="gradient" class="flex-1" @click="finish"><Home class="w-5 h-5" />Kembali ke Beranda</Button>
          </div>
        </div>
      </div>

      <!-- REVIEW -->
      <div v-else-if="phase === 'review' && reviewItems[reviewIndex]" class="min-h-screen bg-slate-50 py-6 px-4">
        <div class="max-w-2xl mx-auto">
          <div class="flex items-center justify-between mb-5">
            <button class="flex items-center gap-2 text-slate-600 hover:text-slate-900 font-medium text-sm" @click="phase = 'results'">
              <ChevronLeft class="w-4 h-4" />Kembali ke Hasil
            </button>
            <span class="text-sm text-slate-500 font-medium">Soal {{ reviewIndex + 1 }} dari {{ reviewItems.length }}</span>
          </div>

          <div class="flex gap-1.5 mb-5 overflow-x-auto pb-1">
            <button
              v-for="(r, i) in reviewItems"
              :key="r.question_id"
              class="w-8 h-8 rounded-lg text-xs font-bold shrink-0"
              :class="i === reviewIndex ? 'bg-indigo-600 text-white' : r.question_type === 'essay' ? (r.user_answer ? 'bg-blue-100 text-blue-700' : 'bg-slate-200 text-slate-500') : r.is_correct === true ? 'bg-emerald-100 text-emerald-700' : r.is_correct === false ? 'bg-red-100 text-red-600' : 'bg-slate-200 text-slate-500'"
              @click="reviewIndex = i"
            >{{ i + 1 }}</button>
          </div>

          <Card class="overflow-hidden mb-4">
            <div
              v-if="reviewItems[reviewIndex].question_type === 'essay'"
              class="px-6 py-4 border-b flex items-center gap-3"
              :class="reviewItems[reviewIndex].user_answer ? 'bg-blue-50' : 'bg-slate-50'"
            >
              <AlertCircle v-if="!reviewItems[reviewIndex].user_answer" class="w-5 h-5 text-slate-400" />
              <BookOpen v-else class="w-5 h-5 text-blue-600" />
              <span class="text-sm font-bold" :class="reviewItems[reviewIndex].user_answer ? 'text-blue-700' : 'text-slate-500'">
                {{ reviewItems[reviewIndex].user_answer ? 'Uraian — Nilai Mandiri' : 'Tidak Dijawab' }}
              </span>
              <span class="ml-auto text-xs text-slate-500">{{ reviewItems[reviewIndex].subject }}</span>
            </div>
            <div v-else class="px-6 py-4 border-b flex items-center gap-3" :class="reviewItems[reviewIndex].is_correct === true ? 'bg-emerald-50' : reviewItems[reviewIndex].is_correct === false ? 'bg-red-50' : 'bg-slate-50'">
              <CheckCircle v-if="reviewItems[reviewIndex].is_correct === true" class="w-5 h-5 text-emerald-600" />
              <XCircle v-else-if="reviewItems[reviewIndex].is_correct === false" class="w-5 h-5 text-red-500" />
              <AlertCircle v-else class="w-5 h-5 text-slate-400" />
              <span class="text-sm font-bold" :class="reviewItems[reviewIndex].is_correct === true ? 'text-emerald-700' : reviewItems[reviewIndex].is_correct === false ? 'text-red-600' : 'text-slate-500'">
                {{ reviewItems[reviewIndex].is_correct === true ? 'Jawaban Benar!' : reviewItems[reviewIndex].is_correct === false ? 'Jawaban Salah' : 'Tidak Dijawab' }}
              </span>
              <span class="ml-auto text-xs text-slate-500">{{ reviewItems[reviewIndex].subject }}</span>
            </div>

            <div class="p-6">
              <div v-if="reviewItems[reviewIndex].stimulus" class="mb-4 p-4 bg-blue-50 border border-blue-200 rounded-xl">
                <p class="text-xs font-bold text-blue-600 mb-1.5 uppercase tracking-wider">Stimulus</p>
                <QuestionContent :text="reviewItems[reviewIndex].stimulus" class="text-sm text-slate-700" />
              </div>

              <QuestionContent :text="reviewItems[reviewIndex].question_text" class="text-base font-semibold text-slate-900 mb-5" />

              <!-- Menjodohkan: options here are full "kiri::kanan" pairs (the review endpoint
                   only unlocks after submit, so the pairing is fine to reveal here). -->
              <div v-if="reviewItems[reviewIndex].question_type === 'matching'" class="space-y-2">
                <p class="text-xs font-bold text-slate-500 uppercase mb-1">Pasangan yang Benar</p>
                <div
                  v-for="(pair, idx) in (reviewItems[reviewIndex].options || []).map((o) => o.split('::'))"
                  :key="idx"
                  class="flex items-center gap-3 p-3 rounded-xl border-2 border-emerald-300 bg-emerald-50"
                >
                  <QuestionContent :text="pair[0]" size="sm" class="flex-1 min-w-0" />
                  <ArrowLeftRight class="w-3.5 h-3.5 text-slate-300 shrink-0" />
                  <QuestionContent :text="pair[1]" size="sm" class="flex-1 min-w-0 font-semibold" />
                </div>
                <p class="text-xs font-bold text-slate-500 uppercase mt-3 mb-1">Jawaban Kamu</p>
                <p v-if="!reviewItems[reviewIndex].user_answer" class="font-mono text-sm">(kosong)</p>
                <p v-else class="font-mono text-sm flex flex-wrap items-center gap-x-1.5 gap-y-1">
                  <template v-for="(userPart, uIdx) in reviewItems[reviewIndex].user_answer!.split(', ')" :key="uIdx">
                    <span class="inline-flex items-center gap-1">
                      <template v-for="(seg, sIdx) in userPart.split('::')" :key="sIdx">
                        <ArrowLeftRight v-if="sIdx > 0" class="w-3 h-3 text-slate-400" />{{ seg }}
                      </template>
                    </span>
                    <span v-if="uIdx < reviewItems[reviewIndex].user_answer!.split(', ').length - 1">,</span>
                  </template>
                </p>
              </div>

              <!-- Menjodohkan Gambar: same reveal pattern as Menjodohkan, images instead of text -->
              <div v-else-if="reviewItems[reviewIndex].question_type === 'matching_image'" class="space-y-2">
                <p class="text-xs font-bold text-slate-500 uppercase mb-1">Pasangan yang Benar</p>
                <div
                  v-for="(pair, idx) in (reviewItems[reviewIndex].options || []).map((o) => o.split('::'))"
                  :key="idx"
                  class="flex items-center gap-3 p-3 rounded-xl border-2 border-emerald-300 bg-emerald-50"
                >
                  <img :src="pair[0]" class="w-16 h-16 object-cover rounded-lg shrink-0" />
                  <ArrowLeftRight class="w-3.5 h-3.5 text-slate-300 shrink-0" />
                  <img :src="pair[1]" class="w-16 h-16 object-cover rounded-lg shrink-0" />
                </div>
                <p class="text-xs font-bold text-slate-500 uppercase mt-3 mb-1">Jawaban Kamu</p>
                <p v-if="!reviewItems[reviewIndex].user_answer" class="font-mono text-sm">(kosong)</p>
                <div v-else class="flex flex-wrap gap-2">
                  <div v-for="(userPart, uIdx) in reviewItems[reviewIndex].user_answer!.split(', ')" :key="uIdx" class="flex items-center gap-1">
                    <img v-for="(seg, sIdx) in userPart.split('::')" :key="sIdx" :src="seg" class="w-10 h-10 object-cover rounded" />
                  </div>
                </div>
              </div>

              <!-- Benar/Salah Ganda: per-statement key vs. student answer -->
              <div v-else-if="reviewItems[reviewIndex].question_type === 'true_false_complex'" class="space-y-2">
                <div
                  v-for="(st, idx) in (reviewItems[reviewIndex].options || [])"
                  :key="idx"
                  class="flex items-center gap-3 p-3 rounded-xl border-2"
                  :class="(reviewItems[reviewIndex].user_answer || '').split(', ')[idx] === reviewItems[reviewIndex].correct_answer.split(', ')[idx] ? 'border-emerald-300 bg-emerald-50' : 'border-red-300 bg-red-50'"
                >
                  <QuestionContent :text="st" size="sm" class="flex-1 min-w-0" />
                  <span class="text-xs font-bold text-slate-500">Kamu: {{ (reviewItems[reviewIndex].user_answer || '').split(', ')[idx] || '-' }}</span>
                  <span class="text-xs font-bold text-emerald-600">Kunci: {{ reviewItems[reviewIndex].correct_answer.split(', ')[idx] }}</span>
                </div>
              </div>

              <!-- Urutkan: correct sequence vs. the order the student submitted -->
              <div v-else-if="reviewItems[reviewIndex].question_type === 'ordering'" class="space-y-3">
                <p class="text-xs font-bold text-slate-500 uppercase mb-1">Urutan yang Benar</p>
                <ol class="space-y-1.5 list-none">
                  <li v-for="(step, idx) in reviewItems[reviewIndex].correct_answer.split(', ')" :key="idx" class="flex items-center gap-3 p-2.5 rounded-lg border-2 border-emerald-300 bg-emerald-50 text-sm">
                    <span class="w-5 h-5 rounded bg-emerald-600 text-white text-xs font-black flex items-center justify-center shrink-0">{{ idx + 1 }}</span>
                    <QuestionContent :text="step" size="sm" class="flex-1 min-w-0" />
                  </li>
                </ol>
                <p class="text-xs font-bold text-slate-500 uppercase mt-3 mb-1">Urutan Kamu</p>
                <p v-if="!reviewItems[reviewIndex].user_answer" class="font-mono text-sm">(kosong)</p>
                <ol v-else class="space-y-1.5 list-none">
                  <li v-for="(step, idx) in reviewItems[reviewIndex].user_answer!.split(', ')" :key="idx" class="flex items-center gap-3 p-2.5 rounded-lg border-2 text-sm" :class="step === reviewItems[reviewIndex].correct_answer.split(', ')[idx] ? 'border-emerald-200 bg-emerald-50/50' : 'border-red-300 bg-red-50'">
                    <span class="w-5 h-5 rounded bg-slate-300 text-white text-xs font-black flex items-center justify-center shrink-0">{{ idx + 1 }}</span>
                    <QuestionContent :text="step" size="sm" class="flex-1 min-w-0" />
                  </li>
                </ol>
              </div>

              <!-- Uraian: self-check only, no correct/incorrect verdict -->
              <div v-else-if="reviewItems[reviewIndex].question_type === 'essay'" class="space-y-3">
                <div class="p-3 bg-slate-50 border rounded-xl text-sm">
                  <p class="text-xs font-bold text-slate-500 uppercase mb-1">Jawaban Kamu</p>
                  <p class="whitespace-pre-line">{{ reviewItems[reviewIndex].user_answer || '(kosong)' }}</p>
                </div>
                <div class="p-3 bg-blue-50 border border-blue-200 rounded-xl text-sm">
                  <p class="text-xs font-bold text-blue-600 uppercase mb-1">Contoh Jawaban / Rubrik</p>
                  <QuestionContent :text="reviewItems[reviewIndex].correct_answer" size="sm" />
                </div>
              </div>

              <div v-else-if="reviewItems[reviewIndex].options" class="space-y-2">
                <div
                  v-for="(opt, idx) in reviewItems[reviewIndex].options"
                  :key="opt"
                  class="flex items-start gap-3 p-3.5 rounded-xl border-2"
                  :class="opt.trim() === reviewItems[reviewIndex].correct_answer.trim()
                    ? 'border-emerald-400 bg-emerald-50'
                    : opt === reviewItems[reviewIndex].user_answer
                      ? 'border-red-400 bg-red-50'
                      : 'border-transparent bg-slate-50'"
                >
                  <span class="w-7 h-7 rounded-lg flex items-center justify-center text-xs font-black shrink-0"
                    :class="opt.trim() === reviewItems[reviewIndex].correct_answer.trim() ? 'bg-emerald-500 text-white' : opt === reviewItems[reviewIndex].user_answer ? 'bg-red-500 text-white' : 'bg-slate-200 text-slate-600'">
                    {{ ['A', 'B', 'C', 'D', 'E'][idx] }}
                  </span>
                  <QuestionContent :text="opt" size="sm" class="mt-0.5" />
                </div>
              </div>
              <div v-else class="p-3 bg-slate-50 border rounded-xl text-sm">
                <p class="text-xs font-bold text-slate-500 uppercase mb-1">Jawaban Kamu</p>
                <p class="font-mono">{{ reviewItems[reviewIndex].user_answer || '(kosong)' }}</p>
                <p class="text-xs font-bold text-emerald-600 uppercase mt-3 mb-1">Kunci Jawaban</p>
                <p class="font-mono">{{ reviewItems[reviewIndex].correct_answer }}</p>
              </div>
            </div>

            <div class="mx-6 mb-6 p-4 bg-amber-50 border border-amber-200 rounded-xl">
              <p class="text-xs font-bold text-amber-700 mb-2 uppercase tracking-wider">Pembahasan</p>
              <QuestionContent :text="reviewItems[reviewIndex].explanation" class="text-sm text-slate-700" />
            </div>
          </Card>

          <div class="flex gap-3">
            <Button variant="outline" class="flex-1" :disabled="reviewIndex === 0" @click="reviewIndex = Math.max(0, reviewIndex - 1)">
              <ChevronLeft class="w-4 h-4" />Sebelumnya
            </Button>
            <Button v-if="reviewIndex < reviewItems.length - 1" class="flex-1" @click="reviewIndex += 1">
              Selanjutnya <ChevronRight class="w-4 h-4" />
            </Button>
            <Button v-else variant="gradient" class="flex-1" @click="finish"><Home class="w-4 h-4" />Selesai</Button>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
