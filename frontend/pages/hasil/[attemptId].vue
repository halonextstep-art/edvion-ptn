<script setup lang="ts">
// "Lihat Hasil" — reopens a PAST, already-submitted tryout/drilling attempt's score,
// subject breakdown, and full per-question pembahasan, any time after submission. Unlike
// pages/tryout/[attemptId].vue (a live, in-memory quiz session that only ever shows results
// once, right after Submit, then loses that state on navigation/refresh), this page derives
// 100% of its state from two read-only endpoints — TryoutService.getResult()/getReview() —
// so a student can reopen it from "Riwayat Latihan" (DrillingZone.vue) as many times as they
// want. Template/markup for the two views below is intentionally near-identical to the
// "results"/"review" phases in tryout/[attemptId].vue so the two screens feel like the same
// product, just reached via a different entry point.
import {
  Clock, ChevronLeft, ChevronRight, CheckCircle, XCircle, AlertCircle, BarChart3,
  BookOpen, Home, Eye, Award, Target, ArrowLeftRight, Loader2, FileQuestion, Gauge,
} from 'lucide-vue-next'
import type { AttemptResult, ReviewItem } from '~/types'

definePageMeta({ middleware: 'auth', roles: ['student'] })

const route = useRoute()
const attemptId = route.params.attemptId as string
const { tryoutService } = useApi()
const toast = useToast()

type View = 'hasil' | 'pembahasan'
const view = ref<View>('hasil')

const loading = ref(true)
const loadError = ref<string | null>(null)
const result = ref<AttemptResult | null>(null)
const reviewItems = ref<ReviewItem[]>([])
const reviewIndex = ref(0)

onMounted(async () => {
  try {
    result.value = await tryoutService.getResult(attemptId)
  } catch (e: any) {
    loadError.value = e?.message || 'Hasil tidak ditemukan — kemungkinan attempt ini belum disubmit atau bukan milik Anda.'
    loading.value = false
    return
  }
  try {
    reviewItems.value = await tryoutService.getReview(attemptId)
  } catch {
    // Pembahasan gagal dimuat bukan alasan memblokir layar hasil — tab "Pembahasan" akan
    // menunjukkan pesan kosong sendiri jika reviewItems tetap [].
  } finally {
    loading.value = false
  }
})

function formatTime(seconds: number) {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}
function openPembahasan() {
  reviewIndex.value = 0
  view.value = 'pembahasan'
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
// domain::irt doc comment. "Estimasi IRT" bisa null (belum tersedia) meski mode
// mengaktifkannya kalau data kalibrasi belum cukup — honest-zero, bukan angka dikarang.
const showInstantScore = computed(() => (result.value?.score_display_mode ?? 'instant') !== 'irt')
const irtEnabled = computed(() => {
  const mode = result.value?.score_display_mode
  return mode === 'irt' || mode === 'both'
})
const showIrtAlongsideInstant = computed(() => result.value?.score_display_mode === 'both')
</script>

<template>
  <div class="min-h-screen bg-slate-50">
    <div v-if="loading" class="min-h-screen flex items-center justify-center p-6">
      <div class="text-center text-muted-foreground">
        <Loader2 class="w-6 h-6 animate-spin mx-auto mb-2" />
        <p class="text-sm">Memuat hasil...</p>
      </div>
    </div>

    <div v-else-if="loadError || !result" class="min-h-screen flex items-center justify-center p-6">
      <Card class="max-w-sm w-full p-8 text-center">
        <AlertCircle class="w-8 h-8 mx-auto mb-3 text-slate-300" />
        <p class="text-sm text-muted-foreground mb-4">{{ loadError }}</p>
        <NuxtLink to="/siswa"><Button variant="gradient">Kembali ke Dashboard</Button></NuxtLink>
      </Card>
    </div>

    <!-- HASIL -->
    <div v-else-if="view === 'hasil'" class="min-h-screen bg-gradient-to-br from-indigo-50 via-purple-50 to-pink-50 py-10 px-4">
      <div class="max-w-2xl mx-auto">
        <div class="bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 rounded-3xl p-8 text-white mb-6">
          <div class="flex items-center justify-between mb-6">
            <div>
              <p class="text-indigo-200 text-sm font-medium">Hasil {{ result.attempt.session_title }}</p>
              <h2 class="text-2xl font-black mt-0.5">{{ result.attempt.session_type === 'tryout' ? 'Tryout' : result.attempt.session_type === 'mini' ? 'Mini Tryout' : 'Drilling' }}</h2>
            </div>
            <div class="text-6xl font-black bg-white/20 w-20 h-20 rounded-2xl flex items-center justify-center">{{ grade }}</div>
          </div>
          <div class="grid grid-cols-4 gap-4">
            <div class="bg-white/15 rounded-xl p-3 text-center">
              <Award class="w-4 h-4 mx-auto mb-1.5 text-indigo-200" />
              <p class="text-xl font-black">{{ showInstantScore ? result.attempt.score : (result.irt_score != null ? Math.round(result.irt_score) : '—') }}</p>
              <p class="text-xs text-indigo-200 mt-0.5">{{ showInstantScore ? 'Skor Instan' : 'Estimasi IRT' }}</p>
            </div>
            <div class="bg-white/15 rounded-xl p-3 text-center">
              <Target class="w-4 h-4 mx-auto mb-1.5 text-indigo-200" />
              <p class="text-xl font-black">{{ result.attempt.accuracy != null ? Math.round(result.attempt.accuracy) : 0 }}%</p>
              <p class="text-xs text-indigo-200 mt-0.5">Akurasi</p>
            </div>
            <div class="bg-white/15 rounded-xl p-3 text-center">
              <CheckCircle class="w-4 h-4 mx-auto mb-1.5 text-indigo-200" />
              <p class="text-xl font-black">{{ result.attempt.correct_count ?? 0 }}</p>
              <p class="text-xs text-indigo-200 mt-0.5">Benar</p>
            </div>
            <div class="bg-white/15 rounded-xl p-3 text-center">
              <Clock class="w-4 h-4 mx-auto mb-1.5 text-indigo-200" />
              <p class="text-xl font-black">{{ formatTime(result.attempt.time_used_seconds || 0) }}</p>
              <p class="text-xs text-indigo-200 mt-0.5">Waktu</p>
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
          <div v-if="result.subject_breakdown.length === 0" class="py-4 text-center text-sm text-muted-foreground">Tidak ada rincian mata uji untuk attempt ini.</div>
          <div v-else class="space-y-3">
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

        <div class="grid grid-cols-3 gap-4 mb-6">
          <div class="border rounded-2xl p-4 text-center text-emerald-600 bg-emerald-50 border-emerald-200">
            <CheckCircle class="w-6 h-6 mx-auto mb-2" />
            <p class="text-3xl font-black">{{ result.attempt.correct_count ?? 0 }}</p>
            <p class="text-sm font-medium mt-0.5">Benar</p>
          </div>
          <div class="border rounded-2xl p-4 text-center text-red-500 bg-red-50 border-red-200">
            <XCircle class="w-6 h-6 mx-auto mb-2" />
            <p class="text-3xl font-black">{{ result.attempt.wrong_count ?? 0 }}</p>
            <p class="text-sm font-medium mt-0.5">Salah</p>
          </div>
          <div class="border rounded-2xl p-4 text-center text-slate-500 bg-slate-50 border-slate-200">
            <AlertCircle class="w-6 h-6 mx-auto mb-2" />
            <p class="text-3xl font-black">{{ result.attempt.unanswered_count ?? 0 }}</p>
            <p class="text-sm font-medium mt-0.5">Tidak Dijawab</p>
          </div>
        </div>

        <div class="flex gap-3">
          <Button variant="outline" class="flex-1" :disabled="reviewItems.length === 0" @click="openPembahasan">
            <Eye class="w-5 h-5" />{{ reviewItems.length === 0 ? 'Pembahasan Tidak Tersedia' : 'Lihat Pembahasan' }}
          </Button>
          <NuxtLink to="/siswa" class="flex-1"><Button variant="gradient" class="w-full"><Home class="w-5 h-5" />Kembali ke Beranda</Button></NuxtLink>
        </div>
      </div>
    </div>

    <!-- PEMBAHASAN -->
    <div v-else-if="reviewItems[reviewIndex]" class="min-h-screen bg-slate-50 py-6 px-4">
      <div class="max-w-2xl mx-auto">
        <div class="flex items-center justify-between mb-5">
          <button class="flex items-center gap-2 text-slate-600 hover:text-slate-900 font-medium text-sm" @click="view = 'hasil'">
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

            <!-- Menjodohkan -->
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

            <!-- Menjodohkan Gambar -->
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

            <!-- Benar/Salah Ganda -->
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

            <!-- Urutkan -->
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

            <!-- Uraian -->
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
          <Button v-else variant="gradient" class="flex-1" @click="view = 'hasil'"><Home class="w-4 h-4" />Selesai</Button>
        </div>
      </div>
    </div>

    <!-- Pembahasan requested but genuinely empty (shouldn't normally happen since the button
         above is disabled when reviewItems is empty — kept as a defensive fallback). -->
    <div v-else class="min-h-screen flex items-center justify-center p-6">
      <Card class="max-w-sm w-full p-8 text-center">
        <FileQuestion class="w-8 h-8 mx-auto mb-3 text-slate-300" />
        <p class="text-sm text-muted-foreground mb-4">Pembahasan tidak tersedia untuk attempt ini.</p>
        <Button variant="gradient" @click="view = 'hasil'">Kembali ke Hasil</Button>
      </Card>
    </div>
  </div>
</template>
