<script setup lang="ts">
// Manajemen Set Soal — daftar soal deterministik/kurasi manual yang bisa dipasang ke sesi
// tryout/drilling (lihat SessionFormDialog.vue mode "Set Soal Tetap") supaya semua siswa
// yang mengambil sesi itu mendapat soal yang persis sama & urutan yang sama, bukan hasil
// undian acak dari bank soal. Menambahkan soal ke sebuah set dilakukan lewat dropdown "Set
// Soal (opsional)" di QuestionFormPanel.vue saat membuat/mengedit soal — layar ini hanya
// mengelola set-nya sendiri (buat/ubah/hapus set) dan melihat/menghapus anggotanya.
import { Plus, Pencil, Trash2, ListChecks, Eye, EyeOff, X, Save, ListPlus, Search, FilePlus2, CheckCircle2 } from 'lucide-vue-next'
import type { QuestionSetItem, Question, SubjectCategoryWithSubjects } from '~/types'

const { questionSetService, questionService, taxonomyService } = useApi()
const toast = useToast()

// Deep-link dari kartu subtes di PackageWizard ("Kelola Set Soal ini") — lihat ContentHub.vue.
// Kalau diisi dan cocok dengan salah satu set, set itu langsung dibuka (expanded) begitu
// komponen ini mount, supaya admin tidak perlu mencari-cari sendiri di daftar.
const props = defineProps<{ focusSetId?: string | null }>()

const sets = ref<QuestionSetItem[]>([])
const loading = ref(false)

async function load() {
  loading.value = true
  try {
    sets.value = await questionSetService.list()
  } catch (e: any) {
    toast.error('Gagal memuat daftar Set Soal', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(async () => {
  await load()
  if (props.focusSetId) {
    const target = sets.value.find((s) => s.id === props.focusSetId)
    if (target) await toggleMembers(target)
  }
})

// ─── Create/edit form ───────────────────────────────────────────────────────
const showForm = ref(false)
const editTarget = ref<QuestionSetItem | null>(null)
const form = ref({ name: '', description: '' })
const saving = ref(false)

function openCreate() {
  editTarget.value = null
  form.value = { name: '', description: '' }
  showForm.value = true
}
function openEdit(s: QuestionSetItem) {
  editTarget.value = s
  form.value = { name: s.name, description: s.description || '' }
  showForm.value = true
}
async function save() {
  if (!form.value.name.trim()) { toast.error('Nama Set Soal wajib diisi'); return }
  saving.value = true
  try {
    if (editTarget.value) {
      await questionSetService.update(editTarget.value.id, form.value)
      toast.success(`Set Soal "${form.value.name}" berhasil diperbarui`)
    } else {
      await questionSetService.create(form.value)
      toast.success(`Set Soal "${form.value.name}" berhasil dibuat`)
    }
    showForm.value = false
    load()
  } catch (e: any) {
    toast.error('Gagal menyimpan Set Soal', e?.message)
  } finally {
    saving.value = false
  }
}

// ─── Delete ──────────────────────────────────────────────────────────────────
const showDelete = ref(false)
const deleteTarget = ref<QuestionSetItem | null>(null)
function openDelete(s: QuestionSetItem) {
  deleteTarget.value = s
  showDelete.value = true
}
async function confirmDelete() {
  if (!deleteTarget.value) return
  try {
    await questionSetService.remove(deleteTarget.value.id)
    toast.success(`Set Soal "${deleteTarget.value.name}" dihapus`)
    if (expandedId.value === deleteTarget.value.id) expandedId.value = null
    load()
  } catch (e: any) {
    toast.error('Gagal menghapus Set Soal', e?.message)
  }
}

// ─── Lihat Soal (member list, urutan kurasi) ────────────────────────────────
const expandedId = ref<string | null>(null)
const members = ref<Question[]>([])
const loadingMembers = ref(false)

async function loadMembers(setId: string) {
  loadingMembers.value = true
  try {
    members.value = await questionSetService.getItems(setId)
  } catch (e: any) {
    toast.error('Gagal memuat soal dalam set', e?.message)
    members.value = []
  } finally {
    loadingMembers.value = false
  }
}

async function toggleMembers(s: QuestionSetItem) {
  if (expandedId.value === s.id) {
    expandedId.value = null
    return
  }
  expandedId.value = s.id
  await loadMembers(s.id)
}

async function removeMember(setId: string, questionId: string) {
  try {
    await questionSetService.removeItem(setId, questionId)
    members.value = members.value.filter((q) => q.id !== questionId)
    const target = sets.value.find((s) => s.id === setId)
    if (target) target.item_count = Math.max(0, target.item_count - 1)
    toast.success('Soal dihapus dari set')
  } catch (e: any) {
    toast.error('Gagal menghapus soal dari set', e?.message)
  }
}

// ─── Tambah dari Bank Soal (bulk picker) ────────────────────────────────────
// Lets the user search/filter the live, approved question bank and check several
// questions at once instead of opening each question's form individually just to pick a
// Set Soal from its dropdown. Reuses the exact same questionService.list() endpoint/filters
// Bank Soal itself uses (status/search/subject + page/page_size) — no new backend param.
const BANK_PAGE_SIZE = 20
const showBulkAdd = ref(false)
const bulkTargetSet = ref<QuestionSetItem | null>(null)
const bankQuestions = ref<Question[]>([])
const bankLoading = ref(false)
const bankSearch = ref('')
const bankSubject = ref('')
const bankPage = ref(1)
const bankTotal = ref(0)
const selectedIds = ref<Set<string>>(new Set())
const adding = ref(false)

const categories = ref<SubjectCategoryWithSubjects[]>([])
async function loadCategories() {
  try {
    categories.value = await taxonomyService.list()
  } catch (e: any) {
    toast.error('Gagal memuat daftar mata uji', e?.message)
  }
}

const hasMoreBank = computed(() => bankPage.value * BANK_PAGE_SIZE < bankTotal.value)

async function loadBankQuestions(reset: boolean) {
  if (reset) {
    bankPage.value = 1
    bankQuestions.value = []
  }
  bankLoading.value = true
  try {
    const res = await questionService.list({
      status: 'approved',
      search: bankSearch.value || undefined,
      subject: bankSubject.value || undefined,
      page: bankPage.value,
      page_size: BANK_PAGE_SIZE,
    })
    bankTotal.value = res.total
    // Exclude questions already in this set's member list, and anything already appended
    // from a previous page — never show a question the user could double-add.
    const memberIds = new Set(members.value.map((m) => m.id))
    const existingIds = new Set(bankQuestions.value.map((q) => q.id))
    const fresh = res.items.filter((q) => !memberIds.has(q.id) && !existingIds.has(q.id))
    bankQuestions.value = reset ? fresh : [...bankQuestions.value, ...fresh]
  } catch (e: any) {
    toast.error('Gagal memuat bank soal', e?.message)
  } finally {
    bankLoading.value = false
  }
}

let bankDebounce: ReturnType<typeof setTimeout>
watch([bankSearch, bankSubject], () => {
  if (!showBulkAdd.value) return
  clearTimeout(bankDebounce)
  bankDebounce = setTimeout(() => loadBankQuestions(true), 250)
})

function loadMoreBank() {
  bankPage.value += 1
  loadBankQuestions(false)
}

function openBulkAdd(s: QuestionSetItem) {
  bulkTargetSet.value = s
  selectedIds.value = new Set()
  bankSearch.value = ''
  bankSubject.value = ''
  showBulkAdd.value = true
  if (categories.value.length === 0) loadCategories()
  loadBankQuestions(true)
}

function closeBulkAdd() {
  showBulkAdd.value = false
  bulkTargetSet.value = null
  bankQuestions.value = []
  selectedIds.value = new Set()
}

function toggleSelect(id: string) {
  if (selectedIds.value.has(id)) selectedIds.value.delete(id)
  else selectedIds.value.add(id)
}

async function confirmBulkAdd() {
  if (!bulkTargetSet.value || selectedIds.value.size === 0) return
  const setId = bulkTargetSet.value.id
  const ids = Array.from(selectedIds.value)
  adding.value = true
  try {
    await Promise.all(ids.map((qId) => questionSetService.addItem(setId, qId)))
    const target = sets.value.find((s) => s.id === setId)
    if (target) target.item_count += ids.length
    toast.success(`${ids.length} soal berhasil ditambahkan ke set`)
    closeBulkAdd()
    if (expandedId.value === setId) await loadMembers(setId)
  } catch (e: any) {
    toast.error('Gagal menambahkan soal ke set', e?.message)
  } finally {
    adding.value = false
  }
}

// ─── Tambah Soal Baru ke Set Ini (repeat-authoring loop) ───────────────────
// Reuses QuestionFormPanel.vue as-is (the exact component QuestionBankManager.vue uses to
// author a single question) via its new optional `defaultQuestionSetId` prop, which both
// pre-selects and — critically — keeps re-selecting the Set Soal dropdown every time the
// panel resets itself after a "Simpan & Buat Lagi" save. That means the panel never has to
// close between questions: each successful `create()` call already carries question_set_id
// in its payload (see buildPayload() in QuestionFormPanel.vue), so the backend adds every new
// question straight into this set. We only close it — and only then refresh item_count /
// members — when the author explicitly clicks "Selesai" (or uses the panel's own built-in
// "Simpan & Tutup", which also emits close). The panel is mounted fresh (v-if) each time this
// is opened, so there's no stale state to reset by hand.
const showAuthorPanel = ref(false)
const authorTargetSet = ref<QuestionSetItem | null>(null)
const authorCount = ref(0)

function openAuthorPanel(s: QuestionSetItem) {
  authorTargetSet.value = s
  authorCount.value = 0
  showAuthorPanel.value = true
}

function onAuthorSaved() {
  authorCount.value += 1
}

function closeAuthorPanel() {
  const setId = authorTargetSet.value?.id
  const count = authorCount.value
  showAuthorPanel.value = false
  authorTargetSet.value = null
  authorCount.value = 0
  if (setId && count > 0) {
    const target = sets.value.find((s) => s.id === setId)
    if (target) target.item_count += count
    if (expandedId.value === setId) loadMembers(setId)
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2 dark:text-white"><ListChecks class="w-5 h-5 text-indigo-600 dark:text-violet-400" />Manajemen Set Soal</h2>
        <p class="text-sm text-muted-foreground">Kelola set soal deterministik (urutan &amp; isi tetap) — bisa dipasang ke sesi tryout/drilling sebagai pengganti undian acak dari bank soal</p>
      </div>
      <Button variant="gradient" @click="openCreate"><Plus class="w-4 h-4" />Buat Set Soal</Button>
    </div>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
    <Card v-else-if="sets.length === 0" class="p-10 text-center">
      <ListChecks class="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" />
      <p class="text-muted-foreground text-sm">Belum ada Set Soal. Buat set pertama, lalu tambahkan soal ke dalamnya lewat form Bank Soal.</p>
    </Card>

    <div v-else class="space-y-4">
      <Card v-for="s in sets" :key="s.id" class="overflow-hidden">
        <div class="p-4 flex items-start justify-between gap-3 bg-slate-50 border-b dark:bg-white/5">
          <div class="min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <h3 class="font-bold text-slate-900 dark:text-white">{{ s.name }}</h3>
              <Badge variant="outline" class="text-[10px]">{{ s.item_count }} soal</Badge>
            </div>
            <p v-if="s.description" class="text-xs text-muted-foreground mt-0.5">{{ s.description }}</p>
          </div>
          <div class="flex items-center gap-1 shrink-0">
            <Button variant="outline" size="sm" class="gap-1.5" @click="toggleMembers(s)">
              <component :is="expandedId === s.id ? EyeOff : Eye" class="w-3.5 h-3.5" />
              {{ expandedId === s.id ? 'Tutup' : 'Lihat Soal' }}
            </Button>
            <button class="p-1.5 rounded hover:bg-slate-200 dark:hover:bg-white/10" title="Edit Set Soal" @click="openEdit(s)"><Pencil class="w-4 h-4 text-slate-500 dark:text-white/50" /></button>
            <button class="p-1.5 rounded hover:bg-red-50 dark:hover:bg-red-500/10" title="Hapus Set Soal" @click="openDelete(s)"><Trash2 class="w-4 h-4 text-red-500" /></button>
          </div>
        </div>

        <div v-if="expandedId === s.id" class="p-4 space-y-2">
          <div class="flex items-center justify-end gap-2 pb-1">
            <Button variant="outline" size="sm" class="gap-1.5" @click="openAuthorPanel(s)">
              <FilePlus2 class="w-3.5 h-3.5" />Tambah Soal Baru ke Set Ini
            </Button>
            <Button variant="outline" size="sm" class="gap-1.5" @click="openBulkAdd(s)">
              <ListPlus class="w-3.5 h-3.5" />Tambah dari Bank Soal
            </Button>
          </div>
          <div v-if="loadingMembers" class="text-xs text-muted-foreground py-2">Memuat soal...</div>
          <div v-else-if="members.length === 0" class="text-xs text-muted-foreground italic px-1 py-2">
            Belum ada soal di set ini. Tambahkan lewat tombol "Tambah dari Bank Soal" di atas, atau lewat dropdown "Set Soal" saat membuat/mengedit soal di Bank Soal.
          </div>
          <div
            v-for="(q, idx) in members" :key="q.id"
            class="flex items-center justify-between gap-3 px-3 py-2 rounded-lg border bg-white hover:bg-slate-50 dark:bg-white/5 dark:hover:bg-white/10"
          >
            <div class="flex items-center gap-2 min-w-0">
              <span class="text-[10px] font-black text-slate-300 w-5 shrink-0 dark:text-white/20">{{ idx + 1 }}</span>
              <Badge variant="outline" class="text-[10px] shrink-0">{{ q.code }}</Badge>
              <Badge variant="outline" class="text-[10px] shrink-0">{{ q.subject }}</Badge>
              <span class="text-sm text-slate-700 truncate dark:text-white/70">{{ q.question_text }}</span>
            </div>
            <button
              class="p-1.5 rounded hover:bg-red-50 dark:hover:bg-red-500/10 shrink-0 flex items-center gap-1 text-red-500 text-xs font-medium"
              title="Hapus dari Set"
              @click="removeMember(s.id, q.id)"
            ><X class="w-3.5 h-3.5" />Hapus dari Set</button>
          </div>
        </div>
      </Card>
    </div>

    <!-- Create/edit modal -->
    <Teleport to="body">
      <div v-if="showForm" class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" @click="showForm = false">
        <Card class="w-full max-w-md shadow-2xl" @click.stop>
          <div class="flex items-center justify-between px-6 py-4 border-b">
            <h3 class="font-bold text-slate-900 dark:text-white">{{ editTarget ? 'Edit Set Soal' : 'Buat Set Soal Baru' }}</h3>
            <button class="p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-white/10" @click="showForm = false"><X class="w-4 h-4" /></button>
          </div>
          <div class="px-6 py-5 space-y-4">
            <div>
              <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5 dark:text-white/50">Nama Set Soal</Label>
              <Input v-model="form.name" placeholder="contoh: Tryout Nasional Batch 1" />
            </div>
            <div>
              <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5 dark:text-white/50">Deskripsi (opsional)</Label>
              <Textarea v-model="form.description" placeholder="Deskripsi singkat set soal ini" />
            </div>
          </div>
          <div class="px-6 py-4 border-t flex gap-2">
            <Button variant="outline" class="flex-1" @click="showForm = false">Batal</Button>
            <Button variant="gradient" class="flex-1 gap-1.5" :disabled="saving" @click="save"><Save class="w-4 h-4" />Simpan</Button>
          </div>
        </Card>
      </div>
    </Teleport>

    <!-- Bulk add from Bank Soal modal -->
    <Teleport to="body">
      <div v-if="showBulkAdd" class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" @click="closeBulkAdd">
        <Card class="w-full max-w-2xl max-h-[85vh] flex flex-col shadow-2xl" @click.stop>
          <div class="flex items-center justify-between px-6 py-4 border-b shrink-0">
            <div class="min-w-0">
              <h3 class="font-bold text-slate-900 dark:text-white">Tambah dari Bank Soal</h3>
              <p class="text-xs text-muted-foreground truncate">Ke set "{{ bulkTargetSet?.name }}"</p>
            </div>
            <button class="p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-white/10 shrink-0" @click="closeBulkAdd"><X class="w-4 h-4" /></button>
          </div>

          <div class="px-6 py-3 border-b shrink-0 flex gap-2 flex-wrap">
            <div class="relative flex-1 min-w-48">
              <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
              <input
                v-model="bankSearch"
                placeholder="Cari kode, soal, mata uji..."
                class="w-full pl-9 pr-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 bg-white dark:bg-white/5 dark:border-white/10 dark:text-white dark:placeholder:text-white/30 dark:focus:ring-violet-500"
              />
            </div>
            <select v-model="bankSubject" class="px-3 py-2 border rounded-lg text-sm bg-white dark:bg-[#0f0d1a] dark:border-white/10 dark:text-white">
              <option value="">Semua Mata Uji</option>
              <optgroup v-for="cat in categories" :key="cat.id" :label="cat.name">
                <option v-for="subj in cat.subjects" :key="subj.id" :value="subj.name">{{ subj.name }}</option>
              </optgroup>
            </select>
          </div>

          <div class="px-6 py-3 overflow-y-auto flex-1 space-y-1.5">
            <div v-if="bankLoading && bankQuestions.length === 0" class="text-xs text-muted-foreground text-center py-8">Memuat soal...</div>
            <div v-else-if="bankQuestions.length === 0" class="text-xs text-muted-foreground italic text-center py-8">
              Tidak ada soal disetujui yang bisa ditambahkan — soal yang cocok mungkin sudah semuanya ada di set ini, atau bank soal masih kosong.
            </div>
            <template v-else>
              <label
                v-for="q in bankQuestions" :key="q.id"
                class="flex items-center gap-3 px-3 py-2 rounded-lg border bg-white hover:bg-slate-50 dark:bg-white/5 dark:hover:bg-white/10 cursor-pointer"
              >
                <input type="checkbox" class="shrink-0" :checked="selectedIds.has(q.id)" @change="toggleSelect(q.id)" />
                <Badge variant="outline" class="text-[10px] shrink-0">{{ q.code }}</Badge>
                <Badge variant="outline" class="text-[10px] shrink-0">{{ q.subject }}</Badge>
                <span class="text-sm text-slate-700 truncate flex-1 dark:text-white/70">{{ q.question_text }}</span>
              </label>
              <div v-if="hasMoreBank" class="pt-1 text-center">
                <button
                  class="text-xs text-indigo-600 dark:text-violet-400 hover:text-indigo-700 dark:hover:text-violet-300 font-medium disabled:opacity-50"
                  :disabled="bankLoading"
                  @click="loadMoreBank"
                >{{ bankLoading ? 'Memuat...' : 'Muat Lebih Banyak' }}</button>
              </div>
            </template>
          </div>

          <div class="px-6 py-4 border-t shrink-0 flex items-center justify-between gap-2">
            <p class="text-xs text-muted-foreground">{{ selectedIds.size }} soal dipilih</p>
            <div class="flex gap-2">
              <Button variant="outline" @click="closeBulkAdd">Batal</Button>
              <Button variant="gradient" class="gap-1.5" :disabled="selectedIds.size === 0 || adding" @click="confirmBulkAdd">
                <Plus class="w-4 h-4" />Tambahkan<template v-if="selectedIds.size > 0"> ({{ selectedIds.size }})</template>
              </Button>
            </div>
          </div>
        </Card>
      </div>
    </Teleport>

    <ConfirmDialog
      v-model="showDelete"
      title="Hapus Set Soal ini?"
      description="Soal-soal anggotanya tidak ikut terhapus dari bank soal, hanya keanggotaannya di set ini yang hilang. Tindakan ini tidak bisa dibatalkan."
      confirm-label="Hapus"
      @confirm="confirmDelete"
    />

    <!-- "Tambah Soal Baru ke Set Ini" — persistent side panel, repeat-authoring mode.
         Reuses QuestionFormPanel.vue unchanged aside from its new defaultQuestionSetId prop;
         it is only mounted while showAuthorPanel is true so each open starts from a clean
         instance. The banner above it is this component's own dark-aware chrome (same
         treatment as the rest of this file) — the form panel below handles its own styling. -->
    <Transition name="author-drawer">
      <div
        v-if="showAuthorPanel"
        class="fixed inset-y-0 right-0 z-50 w-full sm:w-[26rem] flex flex-col bg-white shadow-2xl border-l dark:bg-[#0f0d1a] dark:border-white/10"
      >
        <div class="px-5 py-3 border-b shrink-0 bg-indigo-50 border-indigo-200 dark:bg-violet-500/10 dark:border-violet-500/20">
          <div class="flex items-center justify-between gap-2">
            <div class="min-w-0">
              <p class="text-[10px] font-bold uppercase tracking-wider text-indigo-600 dark:text-violet-300">Mode Tambah Soal Baru</p>
              <p class="text-xs text-indigo-800 dark:text-violet-200 truncate">
                Menambahkan soal baru ke: <span class="font-semibold">{{ authorTargetSet?.name }}</span>
              </p>
            </div>
            <Button variant="outline" size="sm" class="shrink-0 gap-1.5" @click="closeAuthorPanel">
              <CheckCircle2 class="w-3.5 h-3.5" />Selesai
            </Button>
          </div>
          <Badge variant="outline" class="mt-2 text-[10px] bg-white dark:bg-white/5">
            {{ authorCount }} soal ditambahkan ke set ini
          </Badge>
        </div>
        <div class="flex-1 min-h-0">
          <QuestionFormPanel
            :open="showAuthorPanel"
            :default-question-set-id="authorTargetSet?.id"
            @close="closeAuthorPanel"
            @saved="onAuthorSaved"
          />
        </div>
      </div>
    </Transition>
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
