<script setup lang="ts">
// Bank Soal — paginated list + persistent side panel for authoring (see
// QuestionFormPanel.vue), status strip filters, and inline row expansion for detail +
// review actions. Shared between the admin "Review Soal" tab and the Tim Konten dashboard;
// role-aware buttons decide what each user can do to a given row (see canEdit/canSubmit/
// isAdmin below). The side panel renders as a fixed, full-height right-side drawer so its
// height is always well-defined regardless of which page hosts this component (avoids a
// prior bug where a sticky+max-height wrapper silently clipped the form).
// Admin review is available directly on each row (no need to expand first): Approve is a
// true one-click action; Tolak/Revisi auto-expand the row and open a note panel because the
// backend requires a non-empty note for those two verdicts (see startReview/submitReviewAction).
import {
  Search, ChevronDown, ChevronLeft, ChevronRight, Plus, FileText, Clock, CheckCircle2, XCircle,
  RotateCcw, User, MessageSquare, Pencil, Trash2, Copy, Send, Tag, BarChart3, ArrowLeftRight, Upload,
} from 'lucide-vue-next'
import type { Question, QuestionStatus, QuestionType, Difficulty, SubjectCategoryWithSubjects } from '~/types'

const { questionService, taxonomyService } = useApi()
const { user } = useAuth()
const toast = useToast()

const PAGE_SIZE = 20

const questions = ref<Question[]>([])
const total = ref(0)
const page = ref(1)
const loading = ref(false)
const search = ref('')
const filterSubject = ref('')
const filterType = ref<QuestionType | ''>('')
const filterDifficulty = ref<Difficulty | ''>('')
const filterStatus = ref<QuestionStatus | 'all'>('all')

const importOpen = ref(false)
const panelOpen = ref(false)
const editTarget = ref<Question | null>(null)
const duplicateSource = ref<Question | null>(null)
const showDelete = ref(false)
const deleteTarget = ref<Question | null>(null)
const expandedId = ref<string | null>(null)
const reviewingId = ref<string | null>(null)
const reviewAction = ref<'reject' | 'revision' | null>(null)
const reviewNote = ref('')
const submittingReview = ref(false)
const submittingAgain = ref<string | null>(null)

const isAdmin = computed(() => user.value?.role === 'admin')

// Katalog mata uji live dari taksonomi — lihat TaxonomyManager.vue admin. Filter tetap
// mengirim nama mata uji (string polos) ke backend, sama seperti sebelumnya.
const categories = ref<SubjectCategoryWithSubjects[]>([])
async function loadCategories() {
  try {
    categories.value = await taxonomyService.list()
  } catch (e: any) {
    toast.error('Gagal memuat daftar mata uji', e?.message)
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

const DIFF_CONFIG: Record<string, string> = {
  easy: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-300',
  medium: 'bg-amber-100 text-amber-700 dark:bg-amber-500/20 dark:text-amber-300',
  hard: 'bg-red-100 text-red-700 dark:bg-red-500/20 dark:text-red-300',
}
const DIFF_LABEL: Record<string, string> = { easy: 'Mudah', medium: 'Sedang', hard: 'Sulit' }

const STATUS_CONFIG: Record<QuestionStatus, { label: string; color: string; dot: string; icon: any }> = {
  draft: { label: 'Draft', color: 'bg-slate-100 text-slate-600 dark:bg-white/10 dark:text-white/60', dot: 'bg-slate-400', icon: FileText },
  review: { label: 'Menunggu Review', color: 'bg-amber-100 text-amber-700 dark:bg-amber-500/20 dark:text-amber-300', dot: 'bg-amber-500', icon: Clock },
  approved: { label: 'Disetujui', color: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-300', dot: 'bg-emerald-500', icon: CheckCircle2 },
  rejected: { label: 'Ditolak', color: 'bg-red-100 text-red-700 dark:bg-red-500/20 dark:text-red-300', dot: 'bg-red-500', icon: XCircle },
  revision: { label: 'Perlu Revisi', color: 'bg-blue-100 text-blue-700 dark:bg-blue-500/20 dark:text-blue-300', dot: 'bg-blue-500', icon: RotateCcw },
}
const STATUS_ORDER: QuestionStatus[] = ['draft', 'review', 'approved', 'rejected', 'revision']

// Per-status counts, fetched as lightweight page_size=1 requests (one per status, run in
// parallel) so the strip shows accurate platform-wide numbers under the active
// search/subject/type/difficulty filters — not just whatever happens to be on the current
// page. Summing all 5 gives the exact "Semua" count too, since the statuses are exhaustive.
const statusCounts = ref<Record<QuestionStatus, number>>({ draft: 0, review: 0, approved: 0, rejected: 0, revision: 0 })
const totalAll = computed(() => STATUS_ORDER.reduce((sum, s) => sum + statusCounts.value[s], 0))

const totalPages = computed(() => Math.max(1, Math.ceil(total.value / PAGE_SIZE)))
const rangeStart = computed(() => (total.value === 0 ? 0 : (page.value - 1) * PAGE_SIZE + 1))
const rangeEnd = computed(() => Math.min(page.value * PAGE_SIZE, total.value))

function activeFilters() {
  return {
    search: search.value || undefined,
    subject: filterSubject.value || undefined,
    difficulty: filterDifficulty.value || undefined,
    question_type: filterType.value || undefined,
  }
}

async function load() {
  loading.value = true
  try {
    const res = await questionService.list({
      ...activeFilters(),
      status: filterStatus.value === 'all' ? undefined : filterStatus.value,
      page: page.value,
      page_size: PAGE_SIZE,
    })
    questions.value = res.items
    total.value = res.total
  } catch (e: any) {
    toast.error('Gagal memuat bank soal', e?.message)
  } finally {
    loading.value = false
  }
}

async function loadCounts() {
  try {
    const results = await Promise.all(
      STATUS_ORDER.map((s) => questionService.list({ ...activeFilters(), status: s, page: 1, page_size: 1 })),
    )
    STATUS_ORDER.forEach((s, i) => { statusCounts.value[s] = results[i].total })
  } catch {
    // non-critical for the status strip — list itself still loads independently
  }
}

let debounceTimer: ReturnType<typeof setTimeout>
watch([search, filterSubject, filterType, filterDifficulty, filterStatus], () => {
  clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    page.value = 1
    load()
    loadCounts()
  }, 250)
})
watch(page, load)

onMounted(() => {
  load()
  loadCounts()
  loadCategories()
})

function goToPage(p: number) {
  if (p < 1 || p > totalPages.value || p === page.value) return
  page.value = p
}

function canEdit(q: Question) {
  return isAdmin.value || q.created_by === user.value?.id
}
function canSubmit(q: Question) {
  return canEdit(q) && (q.status === 'draft' || q.status === 'rejected' || q.status === 'revision')
}

function openCreate() {
  editTarget.value = null
  duplicateSource.value = null
  panelOpen.value = true
}
function openEdit(q: Question, ev?: Event) {
  ev?.stopPropagation()
  editTarget.value = q
  duplicateSource.value = null
  panelOpen.value = true
}
function openDuplicate(q: Question, ev: Event) {
  ev.stopPropagation()
  editTarget.value = null
  duplicateSource.value = q
  panelOpen.value = true
}
function openDelete(q: Question, ev: Event) {
  ev.stopPropagation()
  deleteTarget.value = q
  showDelete.value = true
}
function toggleExpand(q: Question) {
  expandedId.value = expandedId.value === q.id ? null : q.id
  reviewingId.value = null
  reviewAction.value = null
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  try {
    await questionService.remove(deleteTarget.value.id)
    toast.success('Soal berhasil dihapus!')
    load()
    loadCounts()
  } catch (e: any) {
    toast.error('Gagal menghapus soal', e?.message)
  }
}

async function submitForReview(q: Question, ev: Event) {
  ev.stopPropagation()
  submittingAgain.value = q.id
  try {
    await questionService.submitForReview(q.id)
    toast.success(q.status === 'draft' ? 'Soal dikirim untuk review!' : 'Soal diajukan ulang untuk review!')
    load()
    loadCounts()
  } catch (e: any) {
    toast.error('Gagal mengirim soal', e?.message)
  } finally {
    submittingAgain.value = null
  }
}

function startReview(q: Question, action: 'reject' | 'revision', ev: Event) {
  ev.stopPropagation()
  // Auto-expand the row so the note panel (required — backend rejects an empty note for
  // reject/revision) is immediately visible, whether this was triggered from the row's quick
  // action button or from inside an already-expanded row.
  expandedId.value = q.id
  reviewingId.value = q.id
  reviewAction.value = action
  reviewNote.value = ''
}

async function submitReviewAction(q: Question, action: 'approve' | 'reject' | 'revision', ev: Event) {
  ev.stopPropagation()
  if (action !== 'approve' && !reviewNote.value.trim()) {
    toast.error('Catatan wajib diisi')
    return
  }
  submittingReview.value = true
  try {
    await questionService.review(q.id, action, reviewNote.value || undefined)
    toast.success(action === 'approve' ? 'Soal disetujui!' : action === 'reject' ? 'Soal ditolak.' : 'Soal dikembalikan untuk revisi.')
    reviewingId.value = null
    reviewAction.value = null
    load()
    loadCounts()
  } catch (e: any) {
    toast.error('Gagal memproses review', e?.message)
  } finally {
    submittingReview.value = false
  }
}

function correctSet(q: Question) {
  return new Set(q.correct_answer.split(',').map((s) => s.trim()))
}

function closePanel() {
  panelOpen.value = false
}
function onSaved() {
  load()
  loadCounts()
}
function onImported() {
  load()
  loadCounts()
}
</script>

<template>
  <div class="relative">
    <!-- List pane -->
    <div class="space-y-4 transition-[padding]" :class="panelOpen ? 'sm:pr-[27rem]' : ''">
      <div class="flex items-center justify-between gap-4 flex-wrap">
        <div>
          <h2 class="text-xl font-bold text-slate-900 dark:text-white">Bank Soal</h2>
          <p class="text-sm text-muted-foreground">{{ total }} soal ditemukan</p>
        </div>
        <div class="flex items-center gap-2">
          <Button variant="outline" @click="importOpen = true"><Upload class="w-4 h-4" />Import Soal</Button>
          <Button variant="gradient" @click="openCreate"><Plus class="w-4 h-4" />Tambah Soal</Button>
        </div>
      </div>

      <QuestionImportDialog v-model="importOpen" @imported="onImported" />

      <!-- Status strip -->
      <div class="flex gap-2 overflow-x-auto pb-1">
        <button
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-bold border transition-colors whitespace-nowrap"
          :class="filterStatus === 'all' ? 'border-indigo-400 bg-indigo-50 text-indigo-700 dark:border-violet-500/40 dark:bg-violet-600/20 dark:text-violet-200' : 'border-slate-200 bg-white text-slate-500 hover:text-slate-800 dark:border-white/10 dark:bg-white/5 dark:text-white/50 dark:hover:text-white'"
          @click="filterStatus = 'all'"
        >Semua ({{ totalAll }})</button>
        <button
          v-for="s in STATUS_ORDER" :key="s"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-bold border transition-colors whitespace-nowrap"
          :class="filterStatus === s ? 'border-indigo-400 bg-indigo-50 text-indigo-700 dark:border-violet-500/40 dark:bg-violet-600/20 dark:text-violet-200' : 'border-slate-200 bg-white text-slate-500 hover:text-slate-800 dark:border-white/10 dark:bg-white/5 dark:text-white/50 dark:hover:text-white'"
          @click="filterStatus = s"
        >
          <span class="w-1.5 h-1.5 rounded-full" :class="STATUS_CONFIG[s].dot" />
          {{ STATUS_CONFIG[s].label }} ({{ statusCounts[s] }})
        </button>
      </div>

      <!-- Search + filters -->
      <div class="flex gap-2 flex-wrap">
        <div class="relative flex-1 min-w-48">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input v-model="search" placeholder="Cari kode, soal, mata uji..." class="w-full pl-9 pr-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 bg-white dark:bg-white/5 dark:border-white/10 dark:text-white dark:placeholder:text-white/30 dark:focus:ring-violet-500" />
        </div>
        <select v-model="filterSubject" class="px-3 py-2 border rounded-lg text-sm bg-white dark:bg-[#0f0d1a] dark:border-white/10 dark:text-white">
          <option value="">Semua Mata Uji</option>
          <optgroup v-for="cat in categories" :key="cat.id" :label="cat.name">
            <option v-for="s in cat.subjects" :key="s.id" :value="s.name">{{ s.name }}</option>
          </optgroup>
        </select>
        <select v-model="filterType" class="px-3 py-2 border rounded-lg text-sm bg-white dark:bg-[#0f0d1a] dark:border-white/10 dark:text-white">
          <option value="">Semua Tipe</option>
          <option v-for="(label, t) in QTYPE_LABELS" :key="t" :value="t">{{ label }}</option>
        </select>
        <select v-model="filterDifficulty" class="px-3 py-2 border rounded-lg text-sm bg-white dark:bg-[#0f0d1a] dark:border-white/10 dark:text-white">
          <option value="">Semua Kesulitan</option>
          <option value="easy">Mudah</option>
          <option value="medium">Sedang</option>
          <option value="hard">Sulit</option>
        </select>
      </div>

      <!-- List -->
      <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
      <Card v-else-if="questions.length === 0" class="p-16 text-center">
        <FileText class="w-10 h-10 mx-auto mb-3 text-slate-200 dark:text-white/10" />
        <h3 class="font-bold text-slate-700 dark:text-white/70 mb-1">Tidak ada soal ditemukan</h3>
        <button class="mt-2 text-xs text-indigo-600 dark:text-violet-400 hover:text-indigo-700 dark:hover:text-violet-300 flex items-center gap-1.5 mx-auto" @click="openCreate">
          <Plus class="w-3.5 h-3.5" />Buat soal pertama
        </button>
      </Card>
      <template v-else>
        <div class="space-y-2.5">
          <Card
            v-for="q in questions" :key="q.id"
            class="p-4 cursor-pointer hover:shadow-md hover:border-indigo-300 dark:hover:border-violet-500/40 transition-all"
            @click="toggleExpand(q)"
          >
            <div class="flex items-start gap-4">
              <div class="shrink-0 w-24">
                <p class="font-mono text-xs font-bold text-indigo-600 dark:text-violet-400 truncate">{{ q.code }}</p>
                <p class="text-[10px] text-muted-foreground mt-0.5">{{ new Date(q.created_at).toLocaleDateString('id-ID') }}</p>
              </div>

              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 flex-wrap mb-1.5">
                  <span class="text-[10px] font-bold px-2 py-0.5 rounded-full bg-violet-100 text-violet-700 dark:bg-violet-500/20 dark:text-violet-300">{{ q.subject }}</span>
                  <span class="text-[10px] font-bold px-2 py-0.5 rounded-full bg-slate-100 text-slate-600 dark:bg-white/10 dark:text-white/60">{{ QTYPE_LABELS[q.question_type] }}</span>
                  <span class="text-[10px] font-bold px-2 py-0.5 rounded-full" :class="DIFF_CONFIG[q.difficulty]">{{ DIFF_LABEL[q.difficulty] }}</span>
                  <span v-if="q.topic" class="text-xs text-muted-foreground">{{ q.topic }}</span>
                </div>
                <p class="text-sm text-slate-800 dark:text-white/80 line-clamp-2">{{ q.question_text }}</p>
                <div class="flex items-center gap-3 mt-1 flex-wrap">
                  <p class="text-xs text-muted-foreground flex items-center gap-1"><User class="w-3 h-3" />{{ q.created_by_name }}</p>
                  <p v-if="q.usage_count > 0" class="text-xs text-muted-foreground flex items-center gap-1"><BarChart3 class="w-3 h-3" />{{ q.usage_count }}x dipakai</p>
                </div>
                <p v-if="q.review_note" class="text-xs mt-1 flex items-start gap-1" :class="q.status === 'rejected' ? 'text-red-500 dark:text-red-400' : 'text-blue-500 dark:text-blue-400'">
                  <MessageSquare class="w-3 h-3 shrink-0 mt-0.5" /><span class="italic line-clamp-1">"{{ q.review_note }}"</span>
                </p>
              </div>

              <div class="shrink-0 flex items-center gap-1.5" @click.stop>
                <span class="text-xs font-semibold px-2.5 py-1 rounded-full flex items-center gap-1" :class="STATUS_CONFIG[q.status].color">
                  <span class="w-1.5 h-1.5 rounded-full" :class="STATUS_CONFIG[q.status].dot" />
                  {{ STATUS_CONFIG[q.status].label }}
                </span>
                <button
                  v-if="canSubmit(q)"
                  class="p-1.5 rounded-lg bg-indigo-50 hover:bg-indigo-100 text-indigo-600 disabled:opacity-50 dark:bg-violet-500/20 dark:hover:bg-violet-500/30 dark:text-violet-300"
                  :title="q.status === 'draft' ? 'Kirim Review' : 'Ajukan Ulang'"
                  :disabled="submittingAgain === q.id"
                  @click="submitForReview(q, $event)"
                ><Send class="w-3.5 h-3.5" /></button>
                <button v-if="canEdit(q)" class="p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-white/10" title="Edit" @click="openEdit(q, $event)"><Pencil class="w-3.5 h-3.5 text-slate-500 dark:text-white/50" /></button>
                <button class="p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-white/10" title="Duplikat" @click="openDuplicate(q, $event)"><Copy class="w-3.5 h-3.5 text-slate-500 dark:text-white/50" /></button>
                <button v-if="canEdit(q)" class="p-1.5 rounded-lg hover:bg-red-50 dark:hover:bg-red-500/10" title="Hapus" @click="openDelete(q, $event)"><Trash2 class="w-3.5 h-3.5 text-red-500" /></button>
                <template v-if="isAdmin && q.status === 'review'">
                  <button
                    class="p-1.5 rounded-lg bg-emerald-50 hover:bg-emerald-100 text-emerald-600 disabled:opacity-50 dark:bg-emerald-500/20 dark:hover:bg-emerald-500/30 dark:text-emerald-300"
                    title="Setujui" :disabled="submittingReview" @click="submitReviewAction(q, 'approve', $event)"
                  ><CheckCircle2 class="w-3.5 h-3.5" /></button>
                  <button
                    class="p-1.5 rounded-lg bg-blue-50 hover:bg-blue-100 text-blue-600 dark:bg-blue-500/20 dark:hover:bg-blue-500/30 dark:text-blue-300"
                    title="Minta Revisi" @click="startReview(q, 'revision', $event)"
                  ><RotateCcw class="w-3.5 h-3.5" /></button>
                  <button
                    class="p-1.5 rounded-lg bg-red-50 hover:bg-red-100 text-red-600 dark:bg-red-500/20 dark:hover:bg-red-500/30 dark:text-red-300"
                    title="Tolak" @click="startReview(q, 'reject', $event)"
                  ><XCircle class="w-3.5 h-3.5" /></button>
                </template>
                <ChevronDown class="w-4 h-4 text-slate-400 dark:text-white/30 transition-transform" :class="expandedId === q.id ? 'rotate-180' : ''" />
              </div>
            </div>

            <!-- Expanded detail -->
            <div v-if="expandedId === q.id" class="mt-3 pt-3 border-t space-y-3 text-sm" @click.stop>
              <div v-if="q.stimulus" class="p-3 bg-blue-50 border border-blue-200 rounded-lg dark:bg-blue-500/10 dark:border-blue-500/20">
                <p class="text-[10px] font-bold text-blue-600 dark:text-blue-300 uppercase tracking-wider mb-1.5">Stimulus</p>
                <QuestionContent :text="q.stimulus" size="sm" class="text-slate-700 dark:text-white/70" />
              </div>

              <!-- Menjodohkan: each option is a "kiri::kanan" pair, not a pickable choice -->
              <div v-if="q.question_type === 'matching' && q.options && q.options.length" class="space-y-1.5">
                <div
                  v-for="(pair, idx) in q.options.map((o) => o.split('::'))" :key="idx"
                  class="flex items-center gap-2 px-3 py-1.5 rounded-lg border bg-emerald-50 border-emerald-300 text-emerald-800 dark:bg-emerald-500/10 dark:border-emerald-500/30 dark:text-emerald-300"
                >
                  <QuestionContent :text="pair[0]" size="sm" class="flex-1" />
                  <ArrowLeftRight class="w-3.5 h-3.5 text-muted-foreground shrink-0" />
                  <QuestionContent :text="pair[1]" size="sm" class="flex-1" />
                </div>
              </div>
              <!-- Menjodohkan Gambar: same pair layout as Menjodohkan, images instead of text -->
              <div v-else-if="q.question_type === 'matching_image' && q.options && q.options.length" class="space-y-1.5">
                <div
                  v-for="(pair, idx) in q.options.map((o) => o.split('::'))" :key="idx"
                  class="flex items-center gap-2 px-3 py-1.5 rounded-lg border bg-emerald-50 border-emerald-300 dark:bg-emerald-500/10 dark:border-emerald-500/30"
                >
                  <img :src="pair[0]" class="w-10 h-10 object-cover rounded shrink-0" />
                  <ArrowLeftRight class="w-3.5 h-3.5 text-muted-foreground shrink-0" />
                  <img :src="pair[1]" class="w-10 h-10 object-cover rounded shrink-0" />
                </div>
              </div>

              <!-- Benar/Salah Ganda: each statement paired with its own Benar/Salah key -->
              <div v-else-if="q.question_type === 'true_false_complex' && q.options && q.options.length" class="space-y-1.5">
                <div
                  v-for="(st, idx) in q.options" :key="idx"
                  class="flex items-center gap-2 px-3 py-1.5 rounded-lg border bg-emerald-50 border-emerald-300 text-emerald-800 dark:bg-emerald-500/10 dark:border-emerald-500/30 dark:text-emerald-300"
                >
                  <QuestionContent :text="st" size="sm" class="flex-1" />
                  <Badge variant="outline" class="text-[10px] shrink-0">{{ q.correct_answer.split(', ')[idx] }}</Badge>
                </div>
              </div>

              <!-- Urutkan: steps shown in the authored (correct) order -->
              <div v-else-if="q.question_type === 'ordering' && q.options && q.options.length" class="space-y-1.5">
                <div
                  v-for="(step, idx) in q.options" :key="idx"
                  class="flex items-center gap-2 px-3 py-1.5 rounded-lg border bg-emerald-50 border-emerald-300 text-emerald-800 dark:bg-emerald-500/10 dark:border-emerald-500/30 dark:text-emerald-300"
                >
                  <span class="font-black w-4 text-xs">{{ idx + 1 }}</span><QuestionContent :text="step" size="sm" class="flex-1" />
                </div>
              </div>

              <div v-else-if="q.options && q.options.length" class="space-y-1.5">
                <div
                  v-for="(opt, idx) in q.options" :key="idx"
                  class="flex items-center gap-2 px-3 py-1.5 rounded-lg border"
                  :class="correctSet(q).has(opt.trim()) ? 'bg-emerald-50 border-emerald-300 text-emerald-800 dark:bg-emerald-500/10 dark:border-emerald-500/30 dark:text-emerald-300' : 'bg-slate-50 border-transparent text-slate-600 dark:bg-white/5 dark:text-white/50'"
                >
                  <span class="font-black w-4 text-xs">{{ String.fromCharCode(65 + idx) }}</span><QuestionContent :text="opt" size="sm" class="flex-1" />
                  <CheckCircle2 v-if="correctSet(q).has(opt.trim())" class="w-3.5 h-3.5 shrink-0" />
                </div>
              </div>
              <div v-else class="p-3 bg-emerald-50 border border-emerald-200 rounded-lg dark:bg-emerald-500/10 dark:border-emerald-500/20">
                <p class="text-[10px] font-bold text-emerald-600 dark:text-emerald-300 uppercase tracking-wider mb-1">{{ q.question_type === 'essay' ? 'Contoh Jawaban / Rubrik' : 'Kunci Jawaban' }}</p>
                <QuestionContent :text="q.correct_answer" size="sm" class="text-slate-800 dark:text-white/80" />
              </div>

              <div v-if="q.explanation" class="p-3 bg-amber-50 border border-amber-200 rounded-lg dark:bg-amber-500/10 dark:border-amber-500/20">
                <p class="text-[10px] font-bold text-amber-700 dark:text-amber-300 uppercase tracking-wider mb-1">Pembahasan</p>
                <QuestionContent :text="q.explanation" size="sm" class="text-slate-700 dark:text-white/70" />
              </div>

              <div v-if="q.tags.length" class="flex items-center gap-1.5 flex-wrap">
                <Tag class="w-3 h-3 text-slate-400 dark:text-white/30" />
                <span v-for="t in q.tags" :key="t" class="text-[10px] bg-slate-100 text-slate-600 dark:bg-white/10 dark:text-white/50 px-2 py-0.5 rounded-full">{{ t }}</span>
              </div>

              <!-- Admin review note panel — opened directly by the row's quick Tolak/Revisi
                   buttons above (backend requires a non-empty note for those two actions, so
                   they can't be fully one-click like Approve). -->
              <div v-if="isAdmin && q.status === 'review' && reviewingId === q.id && reviewAction" class="pt-2 border-t dark:border-white/10">
                <div class="p-3 rounded-lg border-2" :class="reviewAction === 'reject' ? 'border-red-300 bg-red-50 dark:border-red-500/30 dark:bg-red-500/10' : 'border-blue-300 bg-blue-50 dark:border-blue-500/30 dark:bg-blue-500/10'">
                  <p class="text-[10px] font-bold uppercase tracking-wider mb-1.5" :class="reviewAction === 'reject' ? 'text-red-600 dark:text-red-300' : 'text-blue-600 dark:text-blue-300'">
                    {{ reviewAction === 'reject' ? 'Alasan Penolakan' : 'Catatan Revisi' }}
                  </p>
                  <Textarea v-model="reviewNote" rows="2" class="bg-white dark:bg-white/5 dark:text-white text-xs" autofocus />
                  <div class="flex gap-2 mt-2">
                    <Button variant="outline" size="sm" @click.stop="reviewingId = null; reviewAction = null">Batal</Button>
                    <Button
                      size="sm" :disabled="submittingReview"
                      :class="reviewAction === 'reject' ? 'bg-red-600 hover:bg-red-700 text-white' : 'bg-blue-600 hover:bg-blue-700 text-white'"
                      @click="submitReviewAction(q, reviewAction, $event)"
                    >Kirim</Button>
                  </div>
                </div>
              </div>
            </div>
          </Card>
        </div>

        <!-- Pagination -->
        <div v-if="totalPages > 1" class="flex items-center justify-between gap-3 pt-1">
          <p class="text-xs text-muted-foreground">Menampilkan {{ rangeStart }}–{{ rangeEnd }} dari {{ total }} soal</p>
          <div class="flex items-center gap-1.5">
            <button
              class="p-1.5 rounded-lg border bg-white text-slate-500 hover:text-slate-800 disabled:opacity-40 disabled:hover:text-slate-500 dark:bg-white/5 dark:border-white/10 dark:text-white/50 dark:hover:text-white dark:disabled:hover:text-white/50"
              :disabled="page === 1" @click="goToPage(page - 1)"
            ><ChevronLeft class="w-4 h-4" /></button>
            <span class="text-xs font-semibold text-slate-600 dark:text-white/60 px-2">Halaman {{ page }} / {{ totalPages }}</span>
            <button
              class="p-1.5 rounded-lg border bg-white text-slate-500 hover:text-slate-800 disabled:opacity-40 disabled:hover:text-slate-500 dark:bg-white/5 dark:border-white/10 dark:text-white/50 dark:hover:text-white dark:disabled:hover:text-white/50"
              :disabled="page === totalPages" @click="goToPage(page + 1)"
            ><ChevronRight class="w-4 h-4" /></button>
          </div>
        </div>
      </template>
    </div>

    <!-- Persistent side panel — fixed full-height drawer, not affected by host page's own
         scroll/height quirks (previously clipped when embedded via sticky+max-height). -->
    <Transition name="qfp-drawer">
      <div v-if="panelOpen" class="fixed inset-y-0 right-0 z-50 w-full sm:w-[26rem] bg-white shadow-2xl border-l dark:bg-[#0f0d1a] dark:border-white/10">
        <QuestionFormPanel
          :open="panelOpen" :edit-question="editTarget" :duplicate-from="duplicateSource"
          @close="closePanel"
          @saved="onSaved"
        />
      </div>
    </Transition>

    <ConfirmDialog
      v-model="showDelete"
      title="Hapus soal ini?"
      description="Tindakan ini tidak dapat dibatalkan."
      confirm-label="Hapus"
      @confirm="confirmDelete"
    />
  </div>
</template>

<style scoped>
.qfp-drawer-enter-active,
.qfp-drawer-leave-active {
  transition: transform 0.2s ease;
}
.qfp-drawer-enter-from,
.qfp-drawer-leave-to {
  transform: translateX(100%);
}
</style>
