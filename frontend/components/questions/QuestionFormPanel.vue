<script setup lang="ts">
// Persistent side panel for authoring soal — replaces the old modal dialog so content-role
// authors can batch-create questions without losing their place ("Simpan & Buat Lagi"),
// matching the question-input flow in referensi/ContentDashboard.tsx.
import { X, Save, RotateCcw, Plus, CheckCircle2, ArrowLeftRight, Upload, Loader2, ArrowUp, ArrowDown, ImageOff } from 'lucide-vue-next'
import type { Question, QuestionPayload, QuestionType, Difficulty, SubjectCategoryWithSubjects, QuestionSetItem } from '~/types'

const props = defineProps<{
  open: boolean
  editQuestion?: Question | null
  duplicateFrom?: Question | null
  // Dipakai QuestionSetManager.vue untuk mode "tambah soal baru langsung ke Set X" — saat
  // diberikan, dropdown Set Soal di bawah dipreseleksi ke set ini dan preseleksi itu BERTAHAN
  // setiap kali resetForm() dipanggil ulang (lihat pemakaiannya di resetForm), supaya penulis
  // bisa menyimpan banyak soal berturut-turut ke set yang sama tanpa memilih ulang. Tidak
  // dipakai/di-pass sama sekali oleh QuestionBankManager.vue, jadi alur create/edit biasa di
  // sana tidak berubah (tetap selalu reset ke '').
  defaultQuestionSetId?: string
}>()
const emit = defineEmits<{ close: []; saved: [] }>()

const { questionService, taxonomyService, questionSetService, client } = useApi()
const { user } = useAuth()
const toast = useToast()

// Katalog mata uji live dari taksonomi (SNBT/TKA-IPA/TKA-IPS/AKM), bukan lagi array
// hardcode — lihat TaxonomyManager.vue admin untuk pengelolaannya. Dropdown tetap
// mengirim nama mata uji (string polos) ke backend, bukan id.
const categories = ref<SubjectCategoryWithSubjects[]>([])
const firstSubjectName = computed(() => categories.value[0]?.subjects[0]?.name || '')
// Grup Mata Uji per kategori untuk SearchableSelect — sama datanya dengan <optgroup>
// sebelumnya, cuma bentuk objek yang combobox ini butuhkan.
const subjectGroups = computed(() =>
  categories.value.map((cat) => ({
    label: cat.name,
    options: cat.subjects.map((s) => ({ value: s.name, label: s.name })),
  })),
)
onMounted(async () => {
  try {
    categories.value = await taxonomyService.list()
  } catch (e: any) {
    toast.error('Gagal memuat daftar mata uji', e?.message)
  }
})
watch(categories, () => {
  if (!subject.value) subject.value = firstSubjectName.value
})

// Katalog Set Soal (lihat admin/QuestionSetManager.vue untuk pengelolaannya) — dropdown
// opsional "tambahkan soal ini ke Set Soal X" saat menyimpan.
const sets = ref<QuestionSetItem[]>([])
onMounted(async () => {
  try {
    sets.value = await questionSetService.list()
  } catch (e: any) {
    toast.error('Gagal memuat daftar Set Soal', e?.message)
  }
})

const QTYPE_OPTIONS: { value: QuestionType; label: string; desc: string }[] = [
  { value: 'multiple_choice', label: 'Pilihan Ganda', desc: '1 jawaban benar dari beberapa opsi' },
  { value: 'complex_multiple', label: 'PG Kompleks', desc: 'Lebih dari 1 opsi bisa benar' },
  { value: 'short_answer', label: 'Isian Singkat', desc: 'Jawaban singkat/kata kunci, tanpa opsi' },
  { value: 'true_false', label: 'Benar/Salah', desc: 'Pilih salah satu: Benar atau Salah' },
  { value: 'matching', label: 'Menjodohkan', desc: 'Pasangkan item kiri dengan item kanan' },
  { value: 'essay', label: 'Uraian', desc: 'Jawaban esai bebas — khusus latihan Drilling, dinilai mandiri oleh siswa' },
  { value: 'true_false_complex', label: 'Benar/Salah Ganda', desc: 'Beberapa pernyataan, masing-masing dinilai Benar/Salah' },
  { value: 'matching_image', label: 'Menjodohkan Gambar', desc: 'Pasangkan gambar kiri dengan gambar kanan' },
  { value: 'ordering', label: 'Urutkan', desc: 'Susun beberapa langkah ke urutan yang benar' },
]

const questionType = ref<QuestionType>('multiple_choice')
const subject = ref('')
const topic = ref('')
const subtopic = ref('')
const difficulty = ref<Difficulty>('medium')
const bloomLevel = ref('C3 - Aplikasi')
const hasStimulus = ref(false)
const stimulus = ref('')
const questionText = ref('')
const options = ref<string[]>(['', '', '', '', ''])
const correctAnswer = ref('')
const complexAnswers = ref<Set<number>>(new Set())
// Menjodohkan — pairs of (kiri, kanan) items. Encoded on save as "kiri::kanan" option strings
// (see QuestionType::Matching on the backend) so no new schema/column was needed.
const matchingPairs = ref<{ left: string; right: string }[]>([
  { left: '', right: '' },
  { left: '', right: '' },
])
// Benar/Salah Ganda — several statements, each independently marked Benar/Salah. Encoded on
// save as `options` = statement texts, `correct_answer` = ", "-joined "Benar"/"Salah" tokens
// in the same order (see QuestionType::TrueFalseComplex on the backend).
const tfComplexStatements = ref<{ text: string; answer: 'Benar' | 'Salah' }[]>([
  { text: '', answer: 'Benar' },
  { text: '', answer: 'Benar' },
])
// Menjodohkan Gambar — identical "kiri::kanan" pairing mechanics as `matchingPairs`, except
// each side holds an uploaded image URL instead of typed text (see
// QuestionType::MatchingImage on the backend, and upload_handler::upload_question_image).
const imageMatchingPairs = ref<{ left: string; right: string }[]>([
  { left: '', right: '' },
  { left: '', right: '' },
])
// Tracks which specific upload button is busy, e.g. "0-left" / "1-right", so only that one
// button shows a spinner while others stay interactive.
const uploadingImageKey = ref<string | null>(null)
// Urutkan — steps in their correct order (as authored). `options` and `correct_answer` are
// both the same list, `correct_answer` comma-space-joined (see QuestionType::Ordering).
const orderingSteps = ref<string[]>(['', ''])
const explanation = ref('')
// '' = tidak ditambahkan ke Set Soal manapun. NOTE: ini BUKAN indikator "soal ini sedang
// ada di set X" — `Question` tidak punya field itu (satu soal secara teori bisa dipakai di
// lebih dari satu set, dan API list soal tidak men-denormalisasi keanggotaan set-nya).
// Dropdown ini murni aksi "tambahkan ke set ini saat simpan", makanya selalu direset ke ''
// baik saat create maupun edit — jangan dianggap bug kalau field ini kosong saat edit soal
// yang sebenarnya sudah tergabung dalam sebuah set.
const questionSetId = ref<string>('')
const tags = ref<string[]>([])
const newTag = ref('')
const timeLimit = ref(120)
const submitForReview = ref(false)
const saving = ref(false)

const isEditing = computed(() => !!props.editQuestion)
const canReviewSubmit = computed(() => user.value?.role === 'content')
const inp = 'w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 bg-white dark:bg-white/5 dark:border-white/10 dark:text-white'

function resetForm(q?: Question | null) {
  questionType.value = q?.question_type || 'multiple_choice'
  subject.value = q?.subject || firstSubjectName.value
  topic.value = q?.topic || ''
  subtopic.value = q?.subtopic || ''
  difficulty.value = q?.difficulty || 'medium'
  bloomLevel.value = q?.bloom_level || 'C3 - Aplikasi'
  hasStimulus.value = !!q?.stimulus
  stimulus.value = q?.stimulus || ''
  questionText.value = q?.question_text || ''
  options.value = q?.options && q.options.length > 0 ? [...q.options] : ['', '', '', '', '']
  explanation.value = q?.explanation || ''
  tags.value = q?.tags ? [...q.tags] : []
  timeLimit.value = q?.time_limit || 120
  submitForReview.value = false
  // Direset ke '' secara default — lihat komentar di deklarasi questionSetId di atas — KECUALI
  // saat defaultQuestionSetId diberikan (mode "tambah soal baru ke Set X" dari
  // QuestionSetManager.vue), di mana preseleksi itu harus bertahan lintas reset-setelah-simpan.
  questionSetId.value = props.defaultQuestionSetId || ''

  if (q?.question_type === 'complex_multiple' && q.correct_answer) {
    correctAnswer.value = ''
    complexAnswers.value = new Set(
      q.correct_answer
        .split(',')
        .map((s) => options.value.findIndex((o) => o.trim() === s.trim()))
        .filter((i) => i >= 0),
    )
  } else {
    correctAnswer.value = q?.correct_answer?.toString() || ''
    complexAnswers.value = new Set()
  }

  if (q?.question_type === 'matching' && q.options && q.options.length > 0) {
    matchingPairs.value = q.options.map((pair) => {
      const [left, right] = pair.split('::')
      return { left: left || '', right: right || '' }
    })
  } else {
    matchingPairs.value = [{ left: '', right: '' }, { left: '', right: '' }]
  }

  if (q?.question_type === 'true_false_complex' && q.options && q.options.length > 0) {
    const answers = (q.correct_answer || '').split(', ').map((s) => s.trim())
    tfComplexStatements.value = q.options.map((text, i) => ({
      text,
      answer: answers[i] === 'Salah' ? 'Salah' : 'Benar',
    }))
  } else {
    tfComplexStatements.value = [{ text: '', answer: 'Benar' }, { text: '', answer: 'Benar' }]
  }

  if (q?.question_type === 'matching_image' && q.options && q.options.length > 0) {
    imageMatchingPairs.value = q.options.map((pair) => {
      const [left, right] = pair.split('::')
      return { left: left || '', right: right || '' }
    })
  } else {
    imageMatchingPairs.value = [{ left: '', right: '' }, { left: '', right: '' }]
  }

  if (q?.question_type === 'ordering' && q.options && q.options.length > 0) {
    orderingSteps.value = [...q.options]
  } else {
    orderingSteps.value = ['', '']
  }
}

function addMatchingPair() {
  matchingPairs.value.push({ left: '', right: '' })
}
function removeMatchingPair(idx: number) {
  if (matchingPairs.value.length > 2) matchingPairs.value.splice(idx, 1)
}

function addTfStatement() {
  tfComplexStatements.value.push({ text: '', answer: 'Benar' })
}
function removeTfStatement(idx: number) {
  if (tfComplexStatements.value.length > 2) tfComplexStatements.value.splice(idx, 1)
}

function addImageMatchingPair() {
  imageMatchingPairs.value.push({ left: '', right: '' })
}
function removeImageMatchingPair(idx: number) {
  if (imageMatchingPairs.value.length > 2) imageMatchingPairs.value.splice(idx, 1)
}

const IMAGE_MAX_BYTES = 4 * 1024 * 1024
const IMAGE_ALLOWED_MIME = ['image/png', 'image/jpeg', 'image/webp']

async function uploadMatchingImage(idx: number, side: 'left' | 'right', e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  const key = `${idx}-${side}`
  if (!IMAGE_ALLOWED_MIME.includes(file.type)) {
    toast.error('Format tidak didukung', 'Gunakan PNG, JPEG, atau WEBP.')
    return
  }
  if (file.size > IMAGE_MAX_BYTES) {
    toast.error('File terlalu besar', 'Maksimum 4 MB.')
    return
  }
  uploadingImageKey.value = key
  try {
    const form = new FormData()
    form.append('file', file)
    const res = await client.upload<{ url: string }>('/uploads/question-image', form)
    imageMatchingPairs.value[idx][side] = res.url
    toast.success('Gambar berhasil diunggah')
  } catch (err: any) {
    toast.error('Gagal mengunggah gambar', err?.message)
  } finally {
    uploadingImageKey.value = null
    ;(e.target as HTMLInputElement).value = ''
  }
}

function addOrderingStep() {
  orderingSteps.value.push('')
}
function removeOrderingStep(idx: number) {
  if (orderingSteps.value.length > 2) orderingSteps.value.splice(idx, 1)
}
function moveOrderingStep(idx: number, dir: -1 | 1) {
  const target = idx + dir
  if (target < 0 || target >= orderingSteps.value.length) return
  const arr = orderingSteps.value
  ;[arr[idx], arr[target]] = [arr[target], arr[idx]]
}

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) resetForm(props.editQuestion || props.duplicateFrom)
  },
  { immediate: true },
)

function addTag() {
  const t = newTag.value.trim()
  if (t && !tags.value.includes(t)) tags.value.push(t)
  newTag.value = ''
}
function removeTag(t: string) {
  tags.value = tags.value.filter((x) => x !== t)
}
function toggleComplex(idx: number) {
  const s = new Set(complexAnswers.value)
  s.has(idx) ? s.delete(idx) : s.add(idx)
  complexAnswers.value = s
}

function buildPayload(): QuestionPayload | null {
  if (!questionText.value.trim()) {
    toast.error('Pertanyaan tidak boleh kosong!')
    return null
  }

  let finalCorrectAnswer = correctAnswer.value
  let finalOptions: string[] | null = null

  if (questionType.value === 'multiple_choice') {
    if (options.value.some((o) => !o.trim())) { toast.error('Semua opsi jawaban harus diisi!'); return null }
    if (!correctAnswer.value) { toast.error('Jawaban yang benar harus ditentukan!'); return null }
    finalOptions = options.value
  } else if (questionType.value === 'complex_multiple') {
    if (options.value.some((o) => !o.trim())) { toast.error('Semua opsi jawaban harus diisi!'); return null }
    if (complexAnswers.value.size === 0) { toast.error('Pilih minimal satu jawaban benar!'); return null }
    finalOptions = options.value
    finalCorrectAnswer = [...complexAnswers.value].sort().map((i) => options.value[i]).join(', ')
  } else if (questionType.value === 'true_false') {
    if (correctAnswer.value !== 'Benar' && correctAnswer.value !== 'Salah') { toast.error('Pilih Benar atau Salah!'); return null }
    finalOptions = ['Benar', 'Salah']
  } else if (questionType.value === 'matching') {
    if (matchingPairs.value.length < 2 || matchingPairs.value.some((p) => !p.left.trim() || !p.right.trim())) {
      toast.error('Isi minimal 2 pasangan, kiri dan kanan tidak boleh kosong!')
      return null
    }
    // Both `options` and `correct_answer` use the identical "kiri::kanan" join so the exact
    // same serialization the tryout player must produce for a fully-correct submission
    // matches this string byte-for-byte (grading is plain string equality — see
    // tryout_service.rs normalize()).
    finalOptions = matchingPairs.value.map((p) => `${p.left.trim()}::${p.right.trim()}`)
    finalCorrectAnswer = finalOptions.join(', ')
  } else if (questionType.value === 'true_false_complex') {
    if (tfComplexStatements.value.length < 2 || tfComplexStatements.value.some((s) => !s.text.trim())) {
      toast.error('Isi minimal 2 pernyataan, tidak boleh kosong!')
      return null
    }
    finalOptions = tfComplexStatements.value.map((s) => s.text.trim())
    finalCorrectAnswer = tfComplexStatements.value.map((s) => s.answer).join(', ')
  } else if (questionType.value === 'matching_image') {
    if (imageMatchingPairs.value.length < 2 || imageMatchingPairs.value.some((p) => !p.left.trim() || !p.right.trim())) {
      toast.error('Unggah minimal 2 pasangan gambar, kiri dan kanan tidak boleh kosong!')
      return null
    }
    finalOptions = imageMatchingPairs.value.map((p) => `${p.left.trim()}::${p.right.trim()}`)
    finalCorrectAnswer = finalOptions.join(', ')
  } else if (questionType.value === 'ordering') {
    if (orderingSteps.value.length < 2 || orderingSteps.value.some((s) => !s.trim())) {
      toast.error('Isi minimal 2 langkah, tidak boleh kosong!')
      return null
    }
    finalOptions = orderingSteps.value.map((s) => s.trim())
    finalCorrectAnswer = finalOptions.join(', ')
  } else {
    if (!correctAnswer.value.trim()) {
      toast.error(questionType.value === 'essay' ? 'Contoh jawaban/rubrik harus diisi!' : 'Jawaban yang benar harus ditentukan!')
      return null
    }
  }

  return {
    question_type: questionType.value,
    subject: subject.value,
    topic: topic.value || 'General',
    subtopic: subtopic.value || 'General',
    difficulty: difficulty.value,
    bloom_level: bloomLevel.value,
    stimulus: hasStimulus.value ? stimulus.value : null,
    question_text: questionText.value,
    options: finalOptions,
    correct_answer: finalCorrectAnswer,
    explanation: explanation.value,
    tags: tags.value,
    time_limit: timeLimit.value,
    submit_for_review: submitForReview.value,
    question_set_id: questionSetId.value || null,
  }
}

async function doSave(andNew: boolean) {
  const payload = buildPayload()
  if (!payload) return

  saving.value = true
  try {
    if (isEditing.value && props.editQuestion) {
      await questionService.update(props.editQuestion.id, payload)
      toast.success('Soal berhasil diperbarui!')
      emit('saved')
      emit('close')
    } else {
      await questionService.create(payload)
      toast.success(payload.submit_for_review ? 'Soal disimpan & dikirim untuk review!' : 'Soal baru berhasil ditambahkan!')
      emit('saved')
      if (andNew) {
        resetForm(null)
        subject.value = payload.subject
      } else {
        emit('close')
      }
    }
  } catch (e: any) {
    toast.error('Gagal menyimpan soal', e?.message)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="w-full h-full max-h-full flex flex-col bg-white dark:bg-[#0f0d1a] overflow-hidden">
    <!-- Header -->
    <div class="px-5 py-4 border-b shrink-0 flex items-center justify-between bg-white dark:bg-[#0f0d1a] dark:border-white/10">
      <h3 class="font-bold text-slate-900 dark:text-white">{{ isEditing ? 'Edit Soal' : 'Buat Soal Baru' }}</h3>
      <button class="p-1.5 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-slate-700 transition-colors dark:hover:bg-white/10 dark:text-white/40 dark:hover:text-white" @click="emit('close')">
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Scrollable body -->
    <div class="flex-1 min-h-0 overflow-y-auto px-5 py-4 space-y-4">
      <!-- Question type -->
      <div>
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Tipe Soal</Label>
        <div class="space-y-1.5">
          <button
            v-for="t in QTYPE_OPTIONS" :key="t.value" type="button"
            class="w-full flex items-center gap-3 px-3 py-2 rounded-xl border text-left transition-colors"
            :class="questionType === t.value ? 'bg-indigo-50 border-indigo-300 dark:bg-violet-500/20 dark:border-violet-500/40' : 'bg-white border-slate-200 hover:bg-slate-50 dark:bg-white/5 dark:border-white/10 dark:hover:bg-white/10'"
            @click="questionType = t.value"
          >
            <div class="w-4 h-4 rounded-full border-2 shrink-0 flex items-center justify-center" :class="questionType === t.value ? 'border-indigo-500 dark:border-violet-400' : 'border-slate-300 dark:border-white/20'">
              <div v-if="questionType === t.value" class="w-2 h-2 rounded-full bg-indigo-500 dark:bg-violet-400" />
            </div>
            <div>
              <p class="text-xs font-semibold" :class="questionType === t.value ? 'text-indigo-700 dark:text-violet-300' : 'text-slate-700 dark:text-white/70'">{{ t.label }}</p>
              <p class="text-[10px] text-muted-foreground">{{ t.desc }}</p>
            </div>
          </button>
        </div>
      </div>

      <!-- Subject + Difficulty -->
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Mata Uji</Label>
          <SearchableSelect v-model="subject" :groups="subjectGroups" placeholder="Pilih Mata Uji" />
        </div>
        <div>
          <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Kesulitan</Label>
          <div class="flex gap-1">
            <button
              v-for="d in (['easy', 'medium', 'hard'] as Difficulty[])" :key="d" type="button"
              class="flex-1 py-2 rounded-lg text-[10px] font-bold border transition-colors"
              :class="difficulty === d ? 'bg-indigo-600 border-indigo-600 text-white dark:bg-violet-600 dark:border-violet-600' : 'bg-white border-slate-200 text-slate-500 hover:text-slate-800 dark:bg-white/5 dark:border-white/10 dark:text-white/50 dark:hover:text-white'"
              @click="difficulty = d"
            >{{ d === 'easy' ? 'Mudah' : d === 'medium' ? 'Sedang' : 'Sulit' }}</button>
          </div>
        </div>
      </div>

      <!-- Topic + Subtopic -->
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Topik</Label>
          <Input v-model="topic" placeholder="mis. Aljabar" />
        </div>
        <div>
          <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Sub-topik</Label>
          <Input v-model="subtopic" placeholder="mis. Persamaan Kuadrat" />
        </div>
      </div>

      <div>
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Level Bloom</Label>
        <Input v-model="bloomLevel" placeholder="mis. C3 - Aplikasi" />
      </div>

      <!-- Set Soal (opsional) — lihat catatan di deklarasi questionSetId: ini aksi
           "tambahkan ke set", bukan tampilan keanggotaan set saat ini. -->
      <div>
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Set Soal (opsional)</Label>
        <select v-model="questionSetId" :class="inp">
          <option value="">Tidak ada</option>
          <option v-for="s in sets" :key="s.id" :value="s.id">{{ s.name }} ({{ s.item_count }} soal)</option>
        </select>
      </div>

      <!-- Stimulus -->
      <div class="flex items-center gap-3 p-3 bg-purple-50 rounded-lg border border-purple-200 dark:bg-purple-500/10 dark:border-purple-500/20">
        <input id="hasStimulus" v-model="hasStimulus" type="checkbox" class="w-4 h-4" />
        <label for="hasStimulus" class="flex-1 cursor-pointer text-xs text-purple-800 dark:text-purple-300">Soal dengan stimulus/bacaan panjang</label>
      </div>
      <MarkdownEditorField v-if="hasStimulus" v-model="stimulus" placeholder="Teks bacaan, data, atau konteks soal... (mendukung gambar/tabel/rumus)" :rows="4" />

      <!-- Question text -->
      <div>
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Teks Soal *</Label>
        <MarkdownEditorField v-model="questionText" placeholder="Tulis pertanyaan di sini... (mendukung gambar/tabel/rumus)" :rows="4" />
      </div>

      <!-- MC: single correct -->
      <div v-if="questionType === 'multiple_choice'">
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-2 block">Pilihan Jawaban — klik huruf untuk tandai benar *</Label>
        <div class="space-y-2">
          <div v-for="(opt, idx) in options" :key="idx" class="flex items-center gap-2">
            <button
              type="button"
              class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-black shrink-0 border-2 transition-colors"
              :class="correctAnswer === opt && opt !== '' ? 'bg-emerald-600 border-emerald-600 text-white' : 'bg-white border-slate-300 text-slate-400 hover:border-slate-400 dark:bg-white/5 dark:border-white/20 dark:text-white/30 dark:hover:border-white/40'"
              @click="correctAnswer = opt"
            >{{ String.fromCharCode(65 + idx) }}</button>
            <Input v-model="options[idx]" :placeholder="`Pilihan ${String.fromCharCode(65 + idx)}`" class="flex-1" />
          </div>
        </div>
      </div>

      <!-- PGK: multi correct -->
      <div v-else-if="questionType === 'complex_multiple'">
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-2 block">Pilihan Jawaban — centang semua yang benar *</Label>
        <div class="space-y-2">
          <div v-for="(opt, idx) in options" :key="idx" class="flex items-center gap-2">
            <button
              type="button"
              class="w-6 h-6 rounded flex items-center justify-center shrink-0 border-2 transition-colors"
              :class="complexAnswers.has(idx) ? 'bg-emerald-600 border-emerald-600' : 'bg-white border-slate-300 hover:border-slate-400 dark:bg-white/5 dark:border-white/20 dark:hover:border-white/40'"
              @click="toggleComplex(idx)"
            >
              <CheckCircle2 v-if="complexAnswers.has(idx)" class="w-3.5 h-3.5 text-white" />
            </button>
            <span class="text-xs font-black w-4" :class="complexAnswers.has(idx) ? 'text-emerald-600' : 'text-slate-300 dark:text-white/20'">{{ String.fromCharCode(65 + idx) }}</span>
            <Input v-model="options[idx]" :placeholder="`Pilihan ${String.fromCharCode(65 + idx)}`" class="flex-1" />
          </div>
        </div>
      </div>

      <!-- Short answer -->
      <div v-else-if="questionType === 'short_answer'">
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Jawaban yang Benar *</Label>
        <Input v-model="correctAnswer" placeholder="Masukkan jawaban yang benar" />
      </div>

      <!-- True/False -->
      <div v-else-if="questionType === 'true_false'">
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-2 block">Jawaban yang Benar *</Label>
        <div class="flex gap-2">
          <button
            v-for="v in (['Benar', 'Salah'] as const)" :key="v" type="button"
            class="flex-1 py-2.5 rounded-lg text-sm font-bold border-2 transition-colors"
            :class="correctAnswer === v ? 'bg-emerald-600 border-emerald-600 text-white' : 'bg-white border-slate-200 text-slate-500 hover:border-slate-400 dark:bg-white/5 dark:border-white/10 dark:text-white/50 dark:hover:border-white/30'"
            @click="correctAnswer = v"
          >{{ v }}</button>
        </div>
      </div>

      <!-- Matching (Menjodohkan) -->
      <div v-else-if="questionType === 'matching'">
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-2 block">Pasangan Kiri — Kanan (yang benar) *</Label>
        <div class="space-y-2">
          <div v-for="(pair, idx) in matchingPairs" :key="idx" class="flex items-center gap-2">
            <span class="text-xs font-black w-4 text-slate-400 dark:text-white/30 shrink-0">{{ idx + 1 }}</span>
            <Input v-model="pair.left" placeholder="Item kiri" class="flex-1" />
            <ArrowLeftRight class="w-3.5 h-3.5 text-muted-foreground shrink-0" />
            <Input v-model="pair.right" placeholder="Pasangan kanan" class="flex-1" />
            <button
              v-if="matchingPairs.length > 2" type="button"
              class="p-1.5 rounded-lg hover:bg-red-50 dark:hover:bg-red-500/10 shrink-0"
              @click="removeMatchingPair(idx)"
            ><X class="w-3.5 h-3.5 text-red-500" /></button>
          </div>
        </div>
        <button type="button" class="mt-2 text-xs text-indigo-600 dark:text-violet-400 hover:text-indigo-700 dark:hover:text-violet-300 flex items-center gap-1" @click="addMatchingPair">
          <Plus class="w-3.5 h-3.5" />Tambah pasangan
        </button>
      </div>

      <!-- True/False Complex (Benar/Salah Ganda) -->
      <div v-else-if="questionType === 'true_false_complex'">
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-2 block">Pernyataan — tandai Benar/Salah untuk masing-masing *</Label>
        <div class="space-y-2">
          <div v-for="(st, idx) in tfComplexStatements" :key="idx" class="flex items-center gap-2">
            <span class="text-xs font-black w-4 text-slate-400 dark:text-white/30 shrink-0">{{ idx + 1 }}</span>
            <Input v-model="st.text" placeholder="Tulis pernyataan..." class="flex-1" />
            <div class="flex gap-1 shrink-0">
              <button
                v-for="v in (['Benar', 'Salah'] as const)" :key="v" type="button"
                class="px-2.5 py-1.5 rounded-lg text-[10px] font-bold border-2 transition-colors"
                :class="st.answer === v ? 'bg-emerald-600 border-emerald-600 text-white' : 'bg-white border-slate-200 text-slate-500 hover:border-slate-400 dark:bg-white/5 dark:border-white/10 dark:text-white/50 dark:hover:border-white/30'"
                @click="st.answer = v"
              >{{ v }}</button>
            </div>
            <button
              v-if="tfComplexStatements.length > 2" type="button"
              class="p-1.5 rounded-lg hover:bg-red-50 dark:hover:bg-red-500/10 shrink-0"
              @click="removeTfStatement(idx)"
            ><X class="w-3.5 h-3.5 text-red-500" /></button>
          </div>
        </div>
        <button type="button" class="mt-2 text-xs text-indigo-600 dark:text-violet-400 hover:text-indigo-700 dark:hover:text-violet-300 flex items-center gap-1" @click="addTfStatement">
          <Plus class="w-3.5 h-3.5" />Tambah pernyataan
        </button>
      </div>

      <!-- Matching Image (Menjodohkan Gambar) -->
      <div v-else-if="questionType === 'matching_image'">
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-2 block">Pasangan Gambar Kiri — Kanan (yang benar) *</Label>
        <div class="space-y-3">
          <div v-for="(pair, idx) in imageMatchingPairs" :key="idx" class="flex items-center gap-2">
            <span class="text-xs font-black w-4 text-slate-400 dark:text-white/30 shrink-0">{{ idx + 1 }}</span>
            <label v-for="side in (['left', 'right'] as const)" :key="side" class="flex-1 relative cursor-pointer group">
              <div class="w-full aspect-video rounded-lg border-2 border-dashed border-slate-200 dark:border-white/10 flex items-center justify-center overflow-hidden bg-slate-50 dark:bg-white/5 group-hover:border-indigo-300 dark:group-hover:border-violet-400 transition-colors">
                <Loader2 v-if="uploadingImageKey === `${idx}-${side}`" class="w-5 h-5 animate-spin text-indigo-500" />
                <img v-else-if="pair[side]" :src="pair[side]" class="w-full h-full object-cover" />
                <div v-else class="flex flex-col items-center gap-1 text-slate-400 dark:text-white/30">
                  <Upload class="w-4 h-4" />
                  <span class="text-[9px]">{{ side === 'left' ? 'Gambar kiri' : 'Gambar kanan' }}</span>
                </div>
              </div>
              <input type="file" accept="image/png,image/jpeg,image/webp" class="hidden" @change="uploadMatchingImage(idx, side, $event)" />
            </label>
            <button
              v-if="imageMatchingPairs.length > 2" type="button"
              class="p-1.5 rounded-lg hover:bg-red-50 dark:hover:bg-red-500/10 shrink-0"
              @click="removeImageMatchingPair(idx)"
            ><X class="w-3.5 h-3.5 text-red-500" /></button>
          </div>
        </div>
        <p class="mt-1.5 text-[10px] text-muted-foreground flex items-center gap-1"><ImageOff class="w-3 h-3" />PNG/JPEG/WEBP, maksimum 4 MB per gambar</p>
        <button type="button" class="mt-2 text-xs text-indigo-600 dark:text-violet-400 hover:text-indigo-700 dark:hover:text-violet-300 flex items-center gap-1" @click="addImageMatchingPair">
          <Plus class="w-3.5 h-3.5" />Tambah pasangan
        </button>
      </div>

      <!-- Ordering (Urutkan) -->
      <div v-else-if="questionType === 'ordering'">
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-2 block">Langkah — susun sesuai urutan yang benar *</Label>
        <div class="space-y-2">
          <div v-for="(step, idx) in orderingSteps" :key="idx" class="flex items-center gap-2">
            <span class="text-xs font-black w-4 text-slate-400 dark:text-white/30 shrink-0">{{ idx + 1 }}</span>
            <Input v-model="orderingSteps[idx]" :placeholder="`Langkah ${idx + 1}`" class="flex-1" />
            <div class="flex gap-0.5 shrink-0">
              <button type="button" :disabled="idx === 0" class="p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-white/10 disabled:opacity-30" @click="moveOrderingStep(idx, -1)">
                <ArrowUp class="w-3.5 h-3.5 text-slate-500 dark:text-white/50" />
              </button>
              <button type="button" :disabled="idx === orderingSteps.length - 1" class="p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-white/10 disabled:opacity-30" @click="moveOrderingStep(idx, 1)">
                <ArrowDown class="w-3.5 h-3.5 text-slate-500 dark:text-white/50" />
              </button>
            </div>
            <button
              v-if="orderingSteps.length > 2" type="button"
              class="p-1.5 rounded-lg hover:bg-red-50 dark:hover:bg-red-500/10 shrink-0"
              @click="removeOrderingStep(idx)"
            ><X class="w-3.5 h-3.5 text-red-500" /></button>
          </div>
        </div>
        <button type="button" class="mt-2 text-xs text-indigo-600 dark:text-violet-400 hover:text-indigo-700 dark:hover:text-violet-300 flex items-center gap-1" @click="addOrderingStep">
          <Plus class="w-3.5 h-3.5" />Tambah langkah
        </button>
      </div>

      <!-- Essay (Uraian) -->
      <div v-else-if="questionType === 'essay'">
        <div class="mb-3 p-3 bg-amber-50 border border-amber-200 rounded-lg dark:bg-amber-500/10 dark:border-amber-500/20">
          <p class="text-xs text-amber-800 dark:text-amber-300">Uraian hanya tersedia di sesi Drilling (latihan mandiri) dan tidak dinilai otomatis. Setelah siswa menjawab, contoh jawaban di bawah ini akan ditampilkan supaya siswa bisa menilai jawabannya sendiri.</p>
        </div>
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Contoh Jawaban / Rubrik Penilaian *</Label>
        <MarkdownEditorField v-model="correctAnswer" placeholder="Tulis contoh jawaban ideal atau poin-poin rubrik penilaian..." :rows="5" />
      </div>

      <!-- Explanation -->
      <div>
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Pembahasan</Label>
        <MarkdownEditorField v-model="explanation" placeholder="Jelaskan mengapa jawaban tersebut benar... (mendukung gambar/tabel/rumus)" :rows="4" />
      </div>

      <!-- Tags -->
      <div>
        <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Tags</Label>
        <div class="flex flex-wrap gap-1.5 items-center">
          <Badge v-for="t in tags" :key="t" variant="outline" class="gap-1">
            {{ t }}<button type="button" @click="removeTag(t)"><X class="w-3 h-3" /></button>
          </Badge>
          <Input v-model="newTag" placeholder="Tag + Enter" class="w-28 h-7 text-xs" @keydown.enter.prevent="addTag" />
          <button type="button" class="px-2.5 py-1 rounded-lg bg-indigo-50 text-indigo-600 border border-indigo-200 text-xs hover:bg-indigo-100 dark:bg-violet-500/20 dark:text-violet-300 dark:border-violet-500/30 dark:hover:bg-violet-500/30" @click="addTag">
            <Plus class="w-3 h-3" />
          </button>
        </div>
      </div>

      <!-- Submit for review (content role, create only) -->
      <div v-if="canReviewSubmit && !isEditing" class="flex items-center gap-3 p-3 bg-blue-50 rounded-lg border border-blue-200 dark:bg-blue-500/10 dark:border-blue-500/20">
        <input id="submitReview" v-model="submitForReview" type="checkbox" class="w-4 h-4" />
        <label for="submitReview" class="text-xs text-blue-800 dark:text-blue-300 cursor-pointer">Kirim untuk review Admin Pusat (bukan simpan sebagai draft)</label>
      </div>
    </div>

    <!-- Footer -->
    <div class="px-5 py-4 border-t shrink-0 space-y-2 dark:border-white/10">
      <Button v-if="!isEditing" variant="outline" class="w-full gap-2" :disabled="saving" @click="doSave(true)">
        <RotateCcw class="w-4 h-4" />Simpan &amp; Buat Lagi
      </Button>
      <Button variant="gradient" class="w-full gap-2" :disabled="saving" @click="doSave(false)">
        <Save class="w-4 h-4" />{{ saving ? 'Menyimpan...' : isEditing ? 'Simpan Perubahan' : 'Simpan & Tutup' }}
      </Button>
    </div>
  </div>
</template>
