<script setup lang="ts">
// "Import Soal" — bulk-create questions from a .docx file matching the format the user
// supplied (block-per-question: "N. Pertanyaan", "Tipe:", type-specific fields, "Poin Soal:",
// "Tingkat Kesulitan:", "Pembahasan:"). See backend application::question_import_service doc
// comment for the full DSL and the deliberate two-endpoint (preview -> commit) design: this
// component always calls Preview first (read-only, shows every parsed question plus a reason
// for every row that failed to parse), then only actually creates anything once the user
// confirms via Commit — nothing is written to the database from a preview alone. Shared by
// both Admin and Konten roles since QuestionBankManager.vue itself already is.
import { Upload, FileText, Loader2, CheckCircle2, AlertTriangle, X, ArrowLeftRight, Download } from 'lucide-vue-next'
import type { ImportPreviewResponse, SubjectCategoryWithSubjects, QuestionSetItem } from '~/types'

const open = defineModel<boolean>({ default: false })
const emit = defineEmits<{ imported: [] }>()
// Dipakai PackageWizard.vue supaya import per-subtes langsung diarahkan ke Set Soal subtes itu
// tanpa admin/konten harus ingat memilihnya manual dari dropdown. Preseleksi ini tetap bisa
// diganti — bukan dikunci — dan tidak mengubah alur QuestionBankManager.vue yang tidak
// mem-pass prop ini sama sekali (questionSetId tetap default '' seperti sebelumnya).
const props = defineProps<{ defaultQuestionSetId?: string }>()

const { questionService, taxonomyService, questionSetService } = useApi()
const { user } = useAuth()
const toast = useToast()

const categories = ref<SubjectCategoryWithSubjects[]>([])
const sets = ref<QuestionSetItem[]>([])
onMounted(async () => {
  try {
    categories.value = await taxonomyService.list()
  } catch (e: any) {
    toast.error('Gagal memuat daftar mata uji', e?.message)
  }
  try {
    sets.value = await questionSetService.list()
  } catch (e: any) {
    toast.error('Gagal memuat daftar Set Soal', e?.message)
  }
})

const canReviewSubmit = computed(() => user.value?.role === 'content')

// Grup Mata Uji per kategori (SNBT/TKA-IPA/TKA-IPS/AKM) untuk SearchableSelect — sama datanya
// dengan <optgroup> sebelumnya, cuma bentuk objek yang combobox ini butuhkan.
const subjectGroups = computed(() =>
  categories.value.map((cat) => ({
    label: cat.name,
    options: cat.subjects.map((s) => ({ value: s.name, label: s.name })),
  })),
)

type Step = 'upload' | 'preview'
const step = ref<Step>('upload')
const fileInput = ref<HTMLInputElement | null>(null)
const selectedFile = ref<File | null>(null)

const subject = ref('')
const topic = ref('')
const subtopic = ref('')
const bloomLevel = ref('C3 - Aplikasi')
const timeLimit = ref(120)
const submitForReview = ref(false)
const questionSetId = ref('')
// true = pengguna sengaja klik "Ubah" untuk membuka dropdown Set Soal biasa walau dialog ini
// dibuka dengan target terkunci (defaultQuestionSetId) — lihat pemakaiannya di template.
const overrideQuestionSet = ref(false)

const previewing = ref(false)
const committing = ref(false)
const previewResult = ref<ImportPreviewResponse | null>(null)

function resetAll() {
  step.value = 'upload'
  selectedFile.value = null
  previewResult.value = null
  previewing.value = false
  committing.value = false
  overrideQuestionSet.value = false
  if (fileInput.value) fileInput.value.value = ''
}

watch(open, (isOpen) => {
  if (isOpen) {
    resetAll()
    if (!subject.value) subject.value = categories.value[0]?.subjects[0]?.name || ''
    // Selalu re-sync ke target subtes yang sedang aktif tiap dialog dibuka (bukan cuma sekali)
    // — activeSubtes di PackageWizard.vue bisa berganti antar-buka, dan "Ubah" di atas boleh
    // sengaja melepas preseleksi ini untuk SATU sesi impor tanpa mengubah default berikutnya.
    if (props.defaultQuestionSetId) questionSetId.value = props.defaultQuestionSetId
  }
})

function pickFile() {
  fileInput.value?.click()
}

function onFileSelected(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  if (!file.name.toLowerCase().endsWith('.docx')) {
    toast.error('Format tidak didukung', 'Unggah file .docx sesuai format Import Soal.')
    return
  }
  selectedFile.value = file
}

async function doPreview() {
  if (!selectedFile.value) {
    toast.error('Pilih file .docx terlebih dahulu!')
    return
  }
  previewing.value = true
  try {
    previewResult.value = await questionService.importPreview(selectedFile.value)
    step.value = 'preview'
    if (previewResult.value.questions.length === 0) {
      toast.error('Tidak ada soal yang berhasil dibaca', 'Lihat daftar masalah di bawah untuk detail.')
    }
  } catch (e: any) {
    toast.error('Gagal membaca file', e?.message)
  } finally {
    previewing.value = false
  }
}

async function doCommit() {
  if (!selectedFile.value) return
  if (!subject.value.trim()) {
    toast.error('Mata Uji harus dipilih!')
    return
  }
  committing.value = true
  try {
    const result = await questionService.importCommit(selectedFile.value, {
      subject: subject.value,
      topic: topic.value || 'General',
      subtopic: subtopic.value || 'General',
      bloom_level: bloomLevel.value,
      time_limit: timeLimit.value,
      submit_for_review: submitForReview.value,
      question_set_id: questionSetId.value || undefined,
    })
    if (result.created.length > 0) {
      toast.success(`${result.created.length} soal berhasil diimpor!`)
      emit('imported')
    }
    if (result.issues.length > 0) {
      // Surface remaining issues (rows that failed even at commit time, e.g. a race with
      // another author) so nothing silently vanishes — reuse the same preview-style list.
      previewResult.value = { questions: [], issues: result.issues }
      toast.error(`${result.issues.length} soal gagal diimpor`, 'Lihat daftar di bawah untuk detail.')
    } else {
      open.value = false
    }
  } catch (e: any) {
    toast.error('Gagal mengimpor soal', e?.message)
  } finally {
    committing.value = false
  }
}

const QTYPE_LABELS: Record<string, string> = {
  multiple_choice: 'Pilihan Ganda',
  complex_multiple: 'PG Kompleks',
  short_answer: 'Isian Singkat',
  true_false: 'Benar/Salah',
  matching: 'Menjodohkan',
  essay: 'Uraian',
  true_false_complex: 'Benar/Salah Ganda',
  matching_image: 'Menjodohkan Gambar',
  ordering: 'Urutkan',
}

function isImagePair(opt: string): boolean {
  return opt.split('::').some((side) => side.startsWith('data:image'))
}

function pairSides(opt: string): string[] {
  return opt.split('::')
}

const validCount = computed(() => previewResult.value?.questions.length || 0)
const issueCount = computed(() => previewResult.value?.issues.length || 0)
</script>

<template>
  <Dialog v-model="open" title="Import Soal dari .docx" max-width="max-w-3xl">
    <div class="space-y-5">
      <!-- STEP: upload + batch metadata -->
      <div v-if="step === 'upload'" class="space-y-5">
        <div
          class="border-2 border-dashed rounded-xl p-6 text-center cursor-pointer transition-colors"
          :class="selectedFile ? 'border-emerald-300 bg-emerald-50' : 'border-slate-200 hover:border-indigo-300 hover:bg-slate-50'"
          @click="pickFile"
        >
          <input ref="fileInput" type="file" accept=".docx" class="hidden" @change="onFileSelected" />
          <FileText v-if="selectedFile" class="w-8 h-8 mx-auto mb-2 text-emerald-500" />
          <Upload v-else class="w-8 h-8 mx-auto mb-2 text-slate-400" />
          <p class="text-sm font-semibold text-slate-700">{{ selectedFile ? selectedFile.name : 'Klik untuk pilih file .docx' }}</p>
          <p class="text-xs text-muted-foreground mt-1">Format: "1. Pertanyaan", "Tipe:", opsi/jawaban sesuai tipe soal, "Tingkat Kesulitan:"</p>
        </div>

        <a
          href="/templates/format-import-soal.docx"
          download
          class="flex items-center justify-center gap-2 text-xs font-semibold text-indigo-600 hover:text-indigo-700 hover:underline -mt-3"
          @click.stop
        >
          <Download class="w-3.5 h-3.5" /> Unduh contoh format .docx
        </a>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Mata Uji *</Label>
            <SearchableSelect v-model="subject" :groups="subjectGroups" placeholder="Pilih Mata Uji" required />
          </div>
          <div>
            <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">
              {{ props.defaultQuestionSetId ? 'Set Soal' : 'Set Soal (opsional)' }}
            </Label>
            <!-- Dibuka dari dalam subtes (defaultQuestionSetId ada) — soal WAJIB masuk set ini
                 supaya muncul di subtes tersebut, jadi ditampilkan sebagai target terkunci
                 (bukan dropdown "opsional" biasa) supaya tidak ke-kosongkan tanpa sadar dan
                 soal hasil impor berakhir di Bank Soal umum tanpa terhubung ke subtes manapun.
                 "Ubah" tetap tersedia untuk kasus sengaja ingin arahkan ke set lain / tanpa set. -->
            <div v-if="props.defaultQuestionSetId && !overrideQuestionSet" class="flex items-center justify-between gap-2 px-3 py-2 rounded-lg border bg-emerald-50 border-emerald-200">
              <span class="text-xs text-emerald-700 truncate">
                <CheckCircle2 class="w-3.5 h-3.5 inline -mt-0.5 mr-1" />{{ sets.find((s) => s.id === questionSetId)?.name || 'Subtes ini' }}
              </span>
              <button type="button" class="text-[11px] font-semibold text-emerald-700 hover:underline shrink-0" @click="overrideQuestionSet = true">Ubah</button>
            </div>
            <select v-else v-model="questionSetId" class="w-full px-3 py-2 border rounded-lg text-sm">
              <option value="">Tidak ada — hanya masuk Bank Soal umum</option>
              <option v-for="s in sets" :key="s.id" :value="s.id">{{ s.name }}</option>
            </select>
          </div>
          <div>
            <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Topik</Label>
            <Input v-model="topic" placeholder="mis. Aljabar (default: General)" />
          </div>
          <div>
            <Label class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 block">Sub-topik</Label>
            <Input v-model="subtopic" placeholder="mis. Persamaan Kuadrat (default: General)" />
          </div>
        </div>
        <p class="text-xs text-muted-foreground -mt-2">
          Mata Uji/Topik/Sub-topik ini berlaku untuk SEMUA soal dalam file — format .docx tidak menyertakan field ini per soal.
        </p>

        <div v-if="canReviewSubmit" class="flex items-center gap-3 p-3 bg-blue-50 rounded-lg border border-blue-200">
          <input id="importSubmitReview" v-model="submitForReview" type="checkbox" class="w-4 h-4" />
          <label for="importSubmitReview" class="text-xs text-blue-800 cursor-pointer">Kirim semua soal hasil impor untuk review Admin Pusat (bukan simpan sebagai draft)</label>
        </div>

        <Button variant="gradient" class="w-full gap-2" :disabled="!selectedFile || previewing" @click="doPreview">
          <Loader2 v-if="previewing" class="w-4 h-4 animate-spin" /><FileText v-else class="w-4 h-4" />
          {{ previewing ? 'Membaca file...' : 'Baca & Pratinjau' }}
        </Button>
      </div>

      <!-- STEP: preview results -->
      <div v-else class="space-y-4">
        <div class="flex items-center gap-3">
          <div class="flex-1 flex items-center gap-2 px-3 py-2 rounded-lg bg-emerald-50 border border-emerald-200 text-emerald-700 text-xs font-semibold">
            <CheckCircle2 class="w-4 h-4" />{{ validCount }} soal siap diimpor
          </div>
          <div v-if="issueCount > 0" class="flex-1 flex items-center gap-2 px-3 py-2 rounded-lg bg-amber-50 border border-amber-200 text-amber-700 text-xs font-semibold">
            <AlertTriangle class="w-4 h-4" />{{ issueCount }} bermasalah
          </div>
        </div>

        <div class="max-h-96 overflow-y-auto space-y-2 pr-1">
          <div v-for="q in (previewResult?.questions || [])" :key="`ok-${q.question_index}`" class="p-3 rounded-lg border border-slate-200 bg-white">
            <div class="flex items-center gap-2 mb-1.5">
              <span class="text-[10px] font-bold px-2 py-0.5 rounded bg-indigo-100 text-indigo-700">Soal #{{ q.no_soal }}</span>
              <span class="text-[10px] font-bold px-2 py-0.5 rounded bg-slate-100 text-slate-600">{{ QTYPE_LABELS[q.question_type] || q.question_type }}</span>
              <span class="text-[10px] px-2 py-0.5 rounded bg-slate-50 text-slate-500 capitalize">{{ q.difficulty }}</span>
            </div>
            <div v-if="q.stimulus" class="mb-1.5 p-2 rounded bg-blue-50 border border-blue-200">
              <p class="text-[9px] font-bold text-blue-600 uppercase tracking-wider mb-0.5">Stimulus</p>
              <QuestionContent :text="q.stimulus" size="sm" />
            </div>
            <QuestionContent :text="q.question_text" size="sm" class="text-slate-800" />
            <div v-if="q.options && q.options.length" class="mt-2 flex flex-wrap gap-1.5">
              <template v-for="(opt, idx) in q.options" :key="idx">
                <span v-if="!isImagePair(opt)" class="text-[11px] px-2 py-1 rounded bg-slate-50 border text-slate-600">{{ opt }}</span>
                <span v-else class="inline-flex items-center gap-1 px-1.5 py-1 rounded bg-slate-50 border">
                  <img :src="pairSides(opt)[0]" class="w-8 h-8 object-cover rounded" />
                  <ArrowLeftRight class="w-3 h-3 text-slate-300" />
                  <img :src="pairSides(opt)[1]" class="w-8 h-8 object-cover rounded" />
                </span>
              </template>
            </div>
          </div>

          <div v-for="issue in (previewResult?.issues || [])" :key="`issue-${issue.question_index}`" class="p-3 rounded-lg border border-red-200 bg-red-50">
            <div class="flex items-center gap-2 mb-1">
              <X class="w-3.5 h-3.5 text-red-500 shrink-0" />
              <span class="text-xs font-bold text-red-700">{{ issue.no_soal ? `Soal #${issue.no_soal}` : 'Umum' }}</span>
            </div>
            <p class="text-xs text-red-600">{{ issue.message }}</p>
          </div>
        </div>

        <div class="flex gap-2">
          <Button variant="outline" class="flex-1" :disabled="committing" @click="step = 'upload'">Kembali</Button>
          <Button
            variant="gradient" class="flex-1 gap-2" :disabled="validCount === 0 || committing"
            @click="doCommit"
          >
            <Loader2 v-if="committing" class="w-4 h-4 animate-spin" /><Upload v-else class="w-4 h-4" />
            {{ committing ? 'Menyimpan...' : `Simpan ${validCount} Soal` }}
          </Button>
        </div>
      </div>
    </div>
  </Dialog>
</template>
