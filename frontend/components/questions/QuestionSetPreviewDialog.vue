<script setup lang="ts">
// Preview satu set soal PERSIS seperti tampilan siswa saat mengerjakan (fase "quiz" di
// pages/tryout/[attemptId].vue) — dipakai PackageWizard.vue supaya penulis paket bisa cek
// format soal sebelum publish tanpa harus klik satu-satu per soal. SENGAJA selalu terang,
// tidak ikut varian `dark` admin/konten, karena tujuannya menirukan layar siswa apa adanya,
// bukan menyesuaikan tema penulis. Markup & class per tipe soal (pilihan ganda, menjodohkan,
// menjodohkan gambar, benar/salah ganda, urutkan, esai, isian bebas) disalin verbatim dari
// fase quiz tryout supaya "sama persis" — bedanya: tidak ada timer/autosave/submit, dan
// pilihan jawaban hanya state lokal (tidak dikirim ke server) murni buat lihat interaksi
// highlight-nya.
//
// PENTING soal tema: komponen ini dipakai lewat PackageWizard.vue yang bisa dirender di dalam
// portal Konten (root <div class="dark"> di pages/konten/index.vue). <Card>/<Button> dkk pakai
// warna berbasis CSS variable (--card, --background, dst — lihat assets/css/main.css) yang
// ikut berubah gelap kalau ada leluhur `.dark`, sementara semua kelas warna literal yang
// disalin dari halaman tryout (text-slate-900, border-slate-200, dst) TETAP terang — kombinasi
// keduanya bikin teks gelap di atas kartu gelap (tidak kebaca). Makanya class
// `preview-light-scope` di bawah menimpa ulang variabel-variabel itu ke nilai terang `:root`
// tepat di elemen root komponen ini, supaya seluruh subtree (termasuk Card/Button bawaan)
// selalu terang tanpa peduli ada leluhur `.dark` atau tidak — bukan menduplikasi Card/Button
// jadi div biasa, supaya tetap komponen yang sama persis dipakai di halaman siswa.
import { X, ChevronLeft, ChevronRight, ArrowLeftRight, Eye, Lightbulb, CheckCircle2 } from 'lucide-vue-next'
import type { Question } from '~/types'

const open = defineModel<boolean>({ default: false })
const props = defineProps<{ questions: Question[]; title?: string }>()

// PENTING: `options`/`correct_answer` di sini adalah data ADMIN/KONTEN mentah (dari
// questionSetService.getItems()), BUKAN format "playable" khusus yang dikirim server ke siswa
// saat live attempt (yang untuk tipe menjodohkan disembunyikan lewat sentinel __MATCH_SPLIT__
// — lihat tryout_service.rs matching_playable_options()). Untuk menjodohkan & menjodohkan
// gambar, format mentah yang tersimpan justru sudah berupa pasangan "kiri::kanan" apa adanya
// (identik dengan correct_answer, lihat QuestionFormPanel.vue baris ~305) — jadi lefts/rights
// diambil langsung dari situ, rights diacak lokal supaya tampilan dropdown tetap realistis
// seperti yang dilihat siswa (bukan demi kerahasiaan, karena ini alat review penulis).
function shuffle<T>(arr: T[]): T[] {
  const a = [...arr]
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[a[i], a[j]] = [a[j], a[i]]
  }
  return a
}
function parseMatchingPairs(options?: string[] | null) {
  const opts = options || []
  const lefts = opts.map((o) => o.split('::')[0] ?? '')
  const rights = shuffle(opts.map((o) => o.split('::')[1] ?? ''))
  return { lefts, rights }
}

const current = ref(0)
const answers = ref<Record<string, string>>({})
const orderingWorkingCopy = ref<Record<string, string[]>>({})
// Toggle "Lihat Kunci & Pembahasan" — direset tiap ganti soal supaya tidak kebawa spoiler
// tanpa sengaja saat lompat-lompat soal.
const showAnswerKey = ref(false)

watch(open, (v) => {
  if (v) {
    current.value = 0
    answers.value = {}
    orderingWorkingCopy.value = {}
    showAnswerKey.value = false
  }
})
watch(current, () => { showAnswerKey.value = false })

const currentQuestion = computed(() => props.questions[current.value] ?? null)
const currentMatching = computed(() => parseMatchingPairs(currentQuestion.value?.options))
const currentMatchingPairs = computed(() => (currentQuestion.value?.options || []).map((o) => [o.split('::')[0] ?? '', o.split('::')[1] ?? '']))
const answeredCount = computed(() => Object.keys(answers.value).length)

function selectAnswer(questionId: string, value: string) {
  answers.value = { ...answers.value, [questionId]: value }
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
</script>

<template>
  <div v-if="open" class="preview-light-scope fixed inset-0 z-[100] bg-slate-50 flex flex-col">
    <!-- header preview (bukan bagian tampilan siswa — strip info + tombol tutup) -->
    <div class="bg-indigo-600 text-white px-4 py-2.5 flex items-center justify-between shrink-0">
      <div class="flex items-center gap-2 text-xs font-semibold">
        <Eye class="w-4 h-4" />Pratinjau Tampilan Siswa — {{ title }}
        <span class="opacity-70 font-normal">(jawaban di sini tidak tersimpan)</span>
      </div>
      <button class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-white/15 hover:bg-white/25" @click="open = false">
        <X class="w-3.5 h-3.5" />Tutup Preview
      </button>
    </div>

    <div v-if="questions.length === 0" class="flex-1 flex items-center justify-center text-sm text-slate-400">
      Belum ada soal untuk dipratinjau.
    </div>

    <!-- replika PERSIS fase "quiz" di pages/tryout/[attemptId].vue -->
    <div v-else class="flex-1 overflow-y-auto">
      <div class="bg-white border-b sticky top-0 z-10">
        <div class="max-w-5xl mx-auto px-4 py-3 flex items-center gap-4">
          <div class="flex-1">
            <p class="text-xs text-slate-500 font-medium mb-1">{{ title }}</p>
            <div class="flex items-center gap-2">
              <div class="flex-1 h-1.5 bg-slate-100 rounded-full overflow-hidden">
                <div class="h-full bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full" :style="{ width: `${(answeredCount / questions.length) * 100}%` }" />
              </div>
              <span class="text-xs text-slate-500 font-mono shrink-0">{{ answeredCount }}/{{ questions.length }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="max-w-5xl mx-auto w-full px-4 py-6 grid lg:grid-cols-[240px_1fr] gap-6">
        <div class="hidden lg:block">
          <Card class="p-4 sticky top-[76px]">
            <p class="text-xs font-bold text-slate-500 mb-3 uppercase tracking-wider">Daftar Soal</p>
            <div class="grid grid-cols-5 gap-1.5">
              <button
                v-for="(q, i) in questions"
                :key="q.id"
                class="w-9 h-9 rounded-lg text-xs font-bold"
                :class="i === current ? 'bg-indigo-600 text-white' : answers[q.id] ? 'bg-emerald-100 text-emerald-700 border border-emerald-300' : 'bg-slate-100 text-slate-500'"
                @click="current = i"
              >{{ i + 1 }}</button>
            </div>
          </Card>
        </div>

        <Card v-if="currentQuestion" class="overflow-hidden">
          <div class="px-6 py-4 border-b bg-gradient-to-r from-slate-50 to-indigo-50 flex items-center justify-between">
            <div class="flex items-center gap-3 text-xs">
              <span class="font-bold text-indigo-600 bg-indigo-100 px-2.5 py-1 rounded-lg">Soal {{ current + 1 }} / {{ questions.length }}</span>
              <span class="text-slate-500">{{ currentQuestion.subject }}</span>
            </div>
            <div class="flex items-center gap-3">
              <button
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold border transition-colors"
                :class="showAnswerKey ? 'bg-amber-500 border-amber-500 text-white' : 'bg-white border-amber-300 text-amber-700 hover:bg-amber-50'"
                @click="showAnswerKey = !showAnswerKey"
              >
                <Lightbulb class="w-3.5 h-3.5" />{{ showAnswerKey ? 'Sembunyikan Kunci' : 'Lihat Kunci & Pembahasan' }}
              </button>
              <span class="font-mono text-[11px] text-slate-400">{{ currentQuestion.code }}</span>
            </div>
          </div>

          <div class="p-6">
            <div v-if="currentQuestion.stimulus" class="mb-5 p-4 bg-blue-50 border border-blue-200 rounded-xl">
              <p class="text-xs font-bold text-blue-600 mb-2 uppercase tracking-wider">Stimulus</p>
              <QuestionContent :text="currentQuestion.stimulus" class="text-sm text-slate-700" />
            </div>

            <QuestionContent :text="currentQuestion.question_text" class="text-base font-semibold text-slate-900 mb-6" />

            <!-- Menjodohkan -->
            <div v-if="currentQuestion.question_type === 'matching'" class="space-y-3">
              <div v-for="(left, idx) in currentMatching.lefts" :key="idx" class="flex items-center gap-3 p-3 rounded-xl border-2 border-slate-200">
                <QuestionContent :text="left" size="sm" class="font-medium text-slate-700 flex-1" />
                <ArrowLeftRight class="w-3.5 h-3.5 text-slate-300 shrink-0" />
                <select
                  class="px-3 py-2 border-2 border-slate-200 rounded-lg text-sm focus:border-indigo-500 focus:outline-none min-w-[9rem]"
                  :value="matchingCurrentPairs(currentQuestion.id, currentMatching.lefts)[idx]"
                  @change="selectMatchingPair(currentQuestion.id, currentMatching.lefts, idx, ($event.target as HTMLSelectElement).value)"
                >
                  <option value="">Pilih pasangan...</option>
                  <option v-for="r in currentMatching.rights" :key="r" :value="r">{{ r }}</option>
                </select>
              </div>
            </div>

            <!-- Menjodohkan Gambar -->
            <div v-else-if="currentQuestion.question_type === 'matching_image'" class="space-y-3">
              <div v-for="(left, idx) in currentMatching.lefts" :key="idx" class="flex items-center gap-3 p-3 rounded-xl border-2 border-slate-200">
                <img :src="left" class="w-20 h-20 object-cover rounded-lg shrink-0" />
                <ArrowLeftRight class="w-3.5 h-3.5 text-slate-300 shrink-0" />
                <select
                  class="px-3 py-2 border-2 border-slate-200 rounded-lg text-sm focus:border-indigo-500 focus:outline-none min-w-[9rem]"
                  :value="matchingCurrentPairs(currentQuestion.id, currentMatching.lefts)[idx]"
                  @change="selectMatchingPair(currentQuestion.id, currentMatching.lefts, idx, ($event.target as HTMLSelectElement).value)"
                >
                  <option value="">Pilih pasangan...</option>
                  <option v-for="r in currentMatching.rights" :key="r" :value="r">{{ r }}</option>
                </select>
                <img v-if="matchingCurrentPairs(currentQuestion.id, currentMatching.lefts)[idx]" :src="matchingCurrentPairs(currentQuestion.id, currentMatching.lefts)[idx]" class="w-20 h-20 object-cover rounded-lg shrink-0" />
              </div>
            </div>

            <!-- Benar/Salah Ganda -->
            <div v-else-if="currentQuestion.question_type === 'true_false_complex'" class="space-y-3">
              <div v-for="(st, idx) in (currentQuestion.options || [])" :key="idx" class="flex items-center gap-3 p-3 rounded-xl border-2 border-slate-200">
                <QuestionContent :text="st" size="sm" class="font-medium text-slate-700 flex-1" />
                <div class="flex gap-1.5 shrink-0">
                  <button
                    v-for="v in (['Benar', 'Salah'] as const)" :key="v" type="button"
                    class="px-3 py-1.5 rounded-lg text-xs font-bold border-2 transition-colors"
                    :class="tfComplexCurrentAnswers(currentQuestion.id, (currentQuestion.options || []).length)[idx] === v ? 'bg-indigo-600 border-indigo-600 text-white' : 'bg-white border-slate-200 text-slate-500 hover:border-indigo-300'"
                    @click="selectTfComplexAnswer(currentQuestion.id, (currentQuestion.options || []).length, idx, v)"
                  >{{ v }}</button>
                </div>
              </div>
            </div>

            <!-- Urutkan -->
            <div v-else-if="currentQuestion.question_type === 'ordering'" class="space-y-2">
              <div
                v-for="(step, idx) in orderingCurrentOrder(currentQuestion.id, currentQuestion.options || [])"
                :key="step + idx"
                class="flex items-center gap-3 p-3 rounded-xl border-2 border-slate-200"
              >
                <span class="w-6 h-6 rounded-lg bg-indigo-100 text-indigo-700 text-xs font-black flex items-center justify-center shrink-0">{{ idx + 1 }}</span>
                <QuestionContent :text="step" size="sm" class="font-medium text-slate-700 flex-1" />
                <div class="flex gap-0.5 shrink-0">
                  <button type="button" :disabled="idx === 0" class="p-1.5 rounded-lg hover:bg-slate-100 disabled:opacity-30" @click="moveOrderingAnswer(currentQuestion.id, idx, -1)">
                    <ChevronLeft class="w-3.5 h-3.5 text-slate-500 rotate-90" />
                  </button>
                  <button type="button" :disabled="idx === (currentQuestion.options || []).length - 1" class="p-1.5 rounded-lg hover:bg-slate-100 disabled:opacity-30" @click="moveOrderingAnswer(currentQuestion.id, idx, 1)">
                    <ChevronLeft class="w-3.5 h-3.5 text-slate-500 -rotate-90" />
                  </button>
                </div>
              </div>
            </div>

            <!-- Uraian -->
            <textarea
              v-else-if="currentQuestion.question_type === 'essay'"
              :value="answers[currentQuestion.id] || ''"
              placeholder="Tulis jawaban esai kamu di sini..."
              rows="6"
              class="w-full px-4 py-3 border-2 border-slate-200 rounded-xl focus:border-indigo-500 focus:outline-none font-medium"
              @input="selectAnswer(currentQuestion.id, ($event.target as HTMLTextAreaElement).value)"
            />

            <!-- Pilihan ganda (default) -->
            <div v-else-if="currentQuestion.options && currentQuestion.options.length" class="space-y-3">
              <button
                v-for="(opt, idx) in currentQuestion.options"
                :key="opt"
                class="w-full text-left flex items-start gap-3 p-4 rounded-xl border-2 transition-all"
                :class="answers[currentQuestion.id] === opt ? 'border-indigo-500 bg-indigo-50' : 'border-slate-200 hover:border-indigo-300'"
                @click="selectAnswer(currentQuestion.id, opt)"
              >
                <span class="w-7 h-7 rounded-lg flex items-center justify-center text-xs font-black shrink-0" :class="answers[currentQuestion.id] === opt ? 'bg-indigo-600 text-white' : 'bg-slate-100 text-slate-600'">
                  {{ ['A', 'B', 'C', 'D', 'E'][idx] }}
                </span>
                <QuestionContent :text="opt" size="sm" class="leading-relaxed" />
              </button>
            </div>

            <!-- Isian bebas (fallback) -->
            <input
              v-else
              :value="answers[currentQuestion.id] || ''"
              placeholder="Ketik jawaban kamu di sini..."
              class="w-full px-4 py-3 border-2 border-slate-200 rounded-xl focus:border-indigo-500 focus:outline-none font-medium"
              @input="selectAnswer(currentQuestion.id, ($event.target as HTMLInputElement).value)"
            />

            <!-- Kunci Jawaban & Pembahasan — khusus alat review penulis, tidak ada di layar siswa asli -->
            <div v-if="showAnswerKey" class="mt-6 pt-5 border-t border-dashed border-amber-300 space-y-4">
              <!-- Menjodohkan -->
              <div v-if="currentQuestion.question_type === 'matching'" class="space-y-2">
                <p class="text-xs font-bold text-slate-500 uppercase tracking-wider">Pasangan yang Benar</p>
                <div v-for="(pair, idx) in currentMatchingPairs" :key="idx" class="flex items-center gap-3 p-3 rounded-xl border-2 border-emerald-300 bg-emerald-50">
                  <QuestionContent :text="pair[0]" size="sm" class="flex-1" />
                  <ArrowLeftRight class="w-3.5 h-3.5 text-slate-300 shrink-0" />
                  <QuestionContent :text="pair[1]" size="sm" class="flex-1 font-semibold" />
                </div>
              </div>

              <!-- Menjodohkan Gambar -->
              <div v-else-if="currentQuestion.question_type === 'matching_image'" class="space-y-2">
                <p class="text-xs font-bold text-slate-500 uppercase tracking-wider">Pasangan yang Benar</p>
                <div v-for="(pair, idx) in currentMatchingPairs" :key="idx" class="flex items-center gap-3 p-3 rounded-xl border-2 border-emerald-300 bg-emerald-50">
                  <img :src="pair[0]" class="w-16 h-16 object-cover rounded-lg shrink-0" />
                  <ArrowLeftRight class="w-3.5 h-3.5 text-slate-300 shrink-0" />
                  <img :src="pair[1]" class="w-16 h-16 object-cover rounded-lg shrink-0" />
                </div>
              </div>

              <!-- Benar/Salah Ganda -->
              <div v-else-if="currentQuestion.question_type === 'true_false_complex'" class="space-y-2">
                <p class="text-xs font-bold text-slate-500 uppercase tracking-wider">Kunci per Pernyataan</p>
                <div v-for="(st, idx) in (currentQuestion.options || [])" :key="idx" class="flex items-center gap-3 p-3 rounded-xl border-2 border-emerald-300 bg-emerald-50">
                  <QuestionContent :text="st" size="sm" class="flex-1" />
                  <span class="text-xs font-bold text-emerald-700 shrink-0">{{ (currentQuestion.correct_answer || '').split(', ')[idx] }}</span>
                </div>
              </div>

              <!-- Urutkan -->
              <div v-else-if="currentQuestion.question_type === 'ordering'" class="space-y-2">
                <p class="text-xs font-bold text-slate-500 uppercase tracking-wider">Urutan yang Benar</p>
                <div v-for="(step, idx) in (currentQuestion.correct_answer || '').split(', ')" :key="idx" class="flex items-center gap-3 p-2.5 rounded-lg border-2 border-emerald-300 bg-emerald-50 text-sm">
                  <span class="w-5 h-5 rounded bg-emerald-600 text-white text-xs font-black flex items-center justify-center shrink-0">{{ idx + 1 }}</span>
                  <QuestionContent :text="step" size="sm" />
                </div>
              </div>

              <!-- Uraian -->
              <div v-else-if="currentQuestion.question_type === 'essay'" class="p-3 bg-blue-50 border border-blue-200 rounded-xl text-sm">
                <p class="text-xs font-bold text-blue-600 uppercase mb-1">Contoh Jawaban / Rubrik</p>
                <QuestionContent :text="currentQuestion.correct_answer" size="sm" />
              </div>

              <!-- Pilihan ganda (default) -->
              <div v-else-if="currentQuestion.options && currentQuestion.options.length" class="space-y-2">
                <p class="text-xs font-bold text-slate-500 uppercase tracking-wider">Kunci Jawaban</p>
                <div
                  v-for="(opt, idx) in currentQuestion.options"
                  :key="opt"
                  class="flex items-start gap-3 p-3 rounded-xl border-2"
                  :class="opt.trim() === currentQuestion.correct_answer.trim() ? 'border-emerald-400 bg-emerald-50' : 'border-transparent bg-slate-50'"
                >
                  <span class="w-7 h-7 rounded-lg flex items-center justify-center text-xs font-black shrink-0" :class="opt.trim() === currentQuestion.correct_answer.trim() ? 'bg-emerald-500 text-white' : 'bg-slate-200 text-slate-600'">
                    {{ ['A', 'B', 'C', 'D', 'E'][idx] }}
                  </span>
                  <QuestionContent :text="opt" size="sm" class="mt-0.5" />
                  <CheckCircle2 v-if="opt.trim() === currentQuestion.correct_answer.trim()" class="w-4 h-4 text-emerald-500 ml-auto shrink-0 mt-0.5" />
                </div>
              </div>

              <!-- Fallback (mis. isian bebas / complex_multiple) -->
              <div v-else class="p-3 bg-emerald-50 border border-emerald-200 rounded-xl">
                <p class="text-xs font-bold text-emerald-600 uppercase tracking-wider mb-1">Kunci Jawaban</p>
                <QuestionContent :text="currentQuestion.correct_answer" size="sm" class="text-slate-800" />
              </div>

              <div class="p-4 bg-amber-50 border border-amber-200 rounded-xl">
                <p class="text-xs font-bold text-amber-700 mb-2 uppercase tracking-wider">Pembahasan</p>
                <QuestionContent :text="currentQuestion.explanation || '(Belum ada pembahasan)'" class="text-sm text-slate-700" />
              </div>
            </div>
          </div>

          <div class="px-6 py-4 border-t bg-slate-50 flex items-center justify-between">
            <Button variant="outline" :disabled="current === 0" @click="current = Math.max(0, current - 1)">
              <ChevronLeft class="w-4 h-4" />Sebelumnya
            </Button>
            <Button v-if="current < questions.length - 1" @click="current = Math.min(questions.length - 1, current + 1)">
              Selanjutnya <ChevronRight class="w-4 h-4" />
            </Button>
            <Button v-else variant="gradient" @click="open = false">Selesai Preview</Button>
          </div>
        </Card>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Menimpa ulang CSS variable tema (lihat komentar di atas <script>) balik ke nilai terang
   `:root` bawaan (disalin dari assets/css/main.css) — supaya Card/Button/dll yang dipakai di
   dalam dialog ini tidak ikut gelap walau dirender di dalam portal Konten yang punya leluhur
   `.dark`. */
.preview-light-scope {
  --background: 0 0% 100%;
  --foreground: 222 47% 11%;
  --card: 0 0% 100%;
  --card-foreground: 222 47% 11%;
  --popover: 0 0% 100%;
  --popover-foreground: 222 47% 11%;
  --primary: 243 75% 59%;
  --primary-foreground: 0 0% 100%;
  --secondary: 210 40% 96%;
  --secondary-foreground: 222 47% 11%;
  --muted: 210 40% 96%;
  --muted-foreground: 215 16% 47%;
  --accent: 210 40% 96%;
  --accent-foreground: 222 47% 11%;
  --destructive: 0 84% 60%;
  --destructive-foreground: 0 0% 100%;
  --border: 214 32% 91%;
  --input: 214 32% 91%;
  --ring: 243 75% 59%;
}
</style>
