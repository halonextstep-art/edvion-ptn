<script setup lang="ts">
// Manajemen Taksonomi — kategori (SNBT / TKA-IPA / TKA-IPS / AKM) + mata uji di
// bawahnya. Dipakai sebagai sumber tunggal daftar "Mata Uji" di seluruh platform
// (authoring soal, filter bank soal, sesi tryout/drilling, leaderboard per-subtes)
// menggantikan array SUBJECTS yang sebelumnya di-hardcode di masing-masing komponen.
import { Plus, Pencil, Trash2, FolderTree, ListTree, X, Save } from 'lucide-vue-next'
import type { SubjectCategoryWithSubjects, SubjectItem } from '~/types'

const { taxonomyService } = useApi()
const toast = useToast()

const categories = ref<SubjectCategoryWithSubjects[]>([])
const loading = ref(false)

async function load() {
  loading.value = true
  try {
    categories.value = await taxonomyService.list()
  } catch (e: any) {
    toast.error('Gagal memuat taksonomi', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

const sorted = computed(() => [...categories.value].sort((a, b) => a.sort_order - b.sort_order))

// ─── Category form ─────────────────────────────────────────────────────────
const showCategoryForm = ref(false)
const editCategoryTarget = ref<SubjectCategoryWithSubjects | null>(null)
const categoryForm = ref({ name: '', description: '', sort_order: 1 })
const savingCategory = ref(false)

function openCreateCategory() {
  editCategoryTarget.value = null
  categoryForm.value = { name: '', description: '', sort_order: categories.value.length + 1 }
  showCategoryForm.value = true
}
function openEditCategory(c: SubjectCategoryWithSubjects) {
  editCategoryTarget.value = c
  categoryForm.value = { name: c.name, description: c.description || '', sort_order: c.sort_order }
  showCategoryForm.value = true
}
async function saveCategory() {
  if (!categoryForm.value.name.trim()) { toast.error('Nama kategori wajib diisi'); return }
  savingCategory.value = true
  try {
    if (editCategoryTarget.value) {
      await taxonomyService.updateCategory(editCategoryTarget.value.id, categoryForm.value)
      toast.success(`Kategori "${categoryForm.value.name}" berhasil diperbarui`)
    } else {
      await taxonomyService.createCategory(categoryForm.value)
      toast.success(`Kategori "${categoryForm.value.name}" berhasil dibuat`)
    }
    showCategoryForm.value = false
    load()
  } catch (e: any) {
    toast.error('Gagal menyimpan kategori', e?.message)
  } finally {
    savingCategory.value = false
  }
}

const showCategoryDelete = ref(false)
const deleteCategoryTarget = ref<SubjectCategoryWithSubjects | null>(null)
function openDeleteCategory(c: SubjectCategoryWithSubjects) {
  deleteCategoryTarget.value = c
  showCategoryDelete.value = true
}
async function confirmDeleteCategory() {
  if (!deleteCategoryTarget.value) return
  try {
    await taxonomyService.deleteCategory(deleteCategoryTarget.value.id)
    toast.success(`Kategori "${deleteCategoryTarget.value.name}" dihapus`)
    load()
  } catch (e: any) {
    toast.error('Gagal menghapus kategori', e?.message)
  }
}

// ─── Subject form ───────────────────────────────────────────────────────────
const showSubjectForm = ref(false)
const editSubjectTarget = ref<SubjectItem | null>(null)
const subjectFormCategoryId = ref('')
const subjectForm = ref({ name: '', code: '', sort_order: 1 })
const savingSubject = ref(false)

function openCreateSubject(c: SubjectCategoryWithSubjects) {
  editSubjectTarget.value = null
  subjectFormCategoryId.value = c.id
  subjectForm.value = { name: '', code: '', sort_order: c.subjects.length + 1 }
  showSubjectForm.value = true
}
function openEditSubject(c: SubjectCategoryWithSubjects, s: SubjectItem) {
  editSubjectTarget.value = s
  subjectFormCategoryId.value = c.id
  subjectForm.value = { name: s.name, code: s.code || '', sort_order: s.sort_order }
  showSubjectForm.value = true
}
async function saveSubject() {
  if (!subjectForm.value.name.trim()) { toast.error('Nama mata uji wajib diisi'); return }
  savingSubject.value = true
  try {
    if (editSubjectTarget.value) {
      await taxonomyService.updateSubject(editSubjectTarget.value.id, subjectForm.value)
      toast.success(`Mata uji "${subjectForm.value.name}" berhasil diperbarui`)
    } else {
      await taxonomyService.createSubject({ category_id: subjectFormCategoryId.value, ...subjectForm.value })
      toast.success(`Mata uji "${subjectForm.value.name}" berhasil ditambahkan`)
    }
    showSubjectForm.value = false
    load()
  } catch (e: any) {
    toast.error('Gagal menyimpan mata uji', e?.message)
  } finally {
    savingSubject.value = false
  }
}

const showSubjectDelete = ref(false)
const deleteSubjectTarget = ref<SubjectItem | null>(null)
function openDeleteSubject(s: SubjectItem) {
  deleteSubjectTarget.value = s
  showSubjectDelete.value = true
}
async function confirmDeleteSubject() {
  if (!deleteSubjectTarget.value) return
  try {
    await taxonomyService.deleteSubject(deleteSubjectTarget.value.id)
    toast.success(`Mata uji "${deleteSubjectTarget.value.name}" dihapus`)
    load()
  } catch (e: any) {
    toast.error('Gagal menghapus mata uji', e?.message)
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2"><FolderTree class="w-5 h-5 text-indigo-600" />Manajemen Taksonomi</h2>
        <p class="text-sm text-muted-foreground">Kelola kategori (SNBT, TKA-IPA, TKA-IPS, AKM) dan mata uji di bawahnya — dipakai sebagai daftar Mata Uji di seluruh platform</p>
      </div>
      <Button variant="gradient" @click="openCreateCategory"><Plus class="w-4 h-4" />Tambah Kategori</Button>
    </div>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
    <Card v-else-if="categories.length === 0" class="p-10 text-center">
      <FolderTree class="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" />
      <p class="text-muted-foreground text-sm">Belum ada kategori. Tambahkan kategori pertama.</p>
    </Card>

    <div v-else class="space-y-4">
      <Card v-for="c in sorted" :key="c.id" class="overflow-hidden">
        <div class="p-4 flex items-start justify-between gap-3 bg-slate-50 border-b">
          <div>
            <h3 class="font-bold text-slate-900">{{ c.name }}</h3>
            <p v-if="c.description" class="text-xs text-muted-foreground mt-0.5">{{ c.description }}</p>
            <p class="text-xs text-muted-foreground mt-0.5">{{ c.subjects.length }} mata uji</p>
          </div>
          <div class="flex items-center gap-1 shrink-0">
            <button class="p-1.5 rounded hover:bg-slate-200" title="Edit Kategori" @click="openEditCategory(c)"><Pencil class="w-4 h-4 text-slate-500" /></button>
            <button class="p-1.5 rounded hover:bg-red-50" title="Hapus Kategori" @click="openDeleteCategory(c)"><Trash2 class="w-4 h-4 text-red-500" /></button>
          </div>
        </div>

        <div class="p-4 space-y-2">
          <div v-if="c.subjects.length === 0" class="text-xs text-muted-foreground italic px-1">Belum ada mata uji di kategori ini.</div>
          <div
            v-for="s in [...c.subjects].sort((a, b) => a.sort_order - b.sort_order)" :key="s.id"
            class="flex items-center justify-between gap-3 px-3 py-2 rounded-lg border bg-white hover:bg-slate-50"
          >
            <div class="flex items-center gap-2 min-w-0">
              <ListTree class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
              <span class="text-sm font-medium text-slate-800 truncate">{{ s.name }}</span>
              <Badge v-if="s.code" variant="outline" class="text-[10px]">{{ s.code }}</Badge>
            </div>
            <div class="flex items-center gap-1 shrink-0">
              <button class="p-1.5 rounded hover:bg-slate-100" title="Edit" @click="openEditSubject(c, s)"><Pencil class="w-3.5 h-3.5 text-slate-500" /></button>
              <button class="p-1.5 rounded hover:bg-red-50" title="Hapus" @click="openDeleteSubject(s)"><Trash2 class="w-3.5 h-3.5 text-red-500" /></button>
            </div>
          </div>
          <Button variant="outline" size="sm" class="gap-1.5 mt-1" @click="openCreateSubject(c)"><Plus class="w-3.5 h-3.5" />Tambah Mata Uji</Button>
        </div>
      </Card>
    </div>

    <!-- Category create/edit modal -->
    <Teleport to="body">
      <div v-if="showCategoryForm" class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" @click="showCategoryForm = false">
        <Card class="w-full max-w-md shadow-2xl" @click.stop>
          <div class="flex items-center justify-between px-6 py-4 border-b">
            <h3 class="font-bold text-slate-900">{{ editCategoryTarget ? 'Edit Kategori' : 'Tambah Kategori Baru' }}</h3>
            <button class="p-1.5 rounded-lg hover:bg-slate-100" @click="showCategoryForm = false"><X class="w-4 h-4" /></button>
          </div>
          <div class="px-6 py-5 space-y-4">
            <div>
              <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Kategori</Label>
              <Input v-model="categoryForm.name" placeholder="contoh: SNBT" />
            </div>
            <div>
              <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Deskripsi (opsional)</Label>
              <Textarea v-model="categoryForm.description" placeholder="Deskripsi singkat kategori" />
            </div>
            <div>
              <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Urutan Tampil</Label>
              <Input v-model.number="categoryForm.sort_order" type="number" min="1" />
            </div>
          </div>
          <div class="px-6 py-4 border-t flex gap-2">
            <Button variant="outline" class="flex-1" @click="showCategoryForm = false">Batal</Button>
            <Button variant="gradient" class="flex-1 gap-1.5" :disabled="savingCategory" @click="saveCategory"><Save class="w-4 h-4" />Simpan</Button>
          </div>
        </Card>
      </div>
    </Teleport>

    <!-- Subject create/edit modal -->
    <Teleport to="body">
      <div v-if="showSubjectForm" class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" @click="showSubjectForm = false">
        <Card class="w-full max-w-md shadow-2xl" @click.stop>
          <div class="flex items-center justify-between px-6 py-4 border-b">
            <h3 class="font-bold text-slate-900">{{ editSubjectTarget ? 'Edit Mata Uji' : 'Tambah Mata Uji Baru' }}</h3>
            <button class="p-1.5 rounded-lg hover:bg-slate-100" @click="showSubjectForm = false"><X class="w-4 h-4" /></button>
          </div>
          <div class="px-6 py-5 space-y-4">
            <div>
              <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Mata Uji</Label>
              <Input v-model="subjectForm.name" placeholder="contoh: Penalaran Umum" />
            </div>
            <div>
              <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Kode (opsional)</Label>
              <Input v-model="subjectForm.code" placeholder="contoh: PU" />
            </div>
            <div>
              <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Urutan Tampil</Label>
              <Input v-model.number="subjectForm.sort_order" type="number" min="1" />
            </div>
          </div>
          <div class="px-6 py-4 border-t flex gap-2">
            <Button variant="outline" class="flex-1" @click="showSubjectForm = false">Batal</Button>
            <Button variant="gradient" class="flex-1 gap-1.5" :disabled="savingSubject" @click="saveSubject"><Save class="w-4 h-4" />Simpan</Button>
          </div>
        </Card>
      </div>
    </Teleport>

    <ConfirmDialog
      v-model="showCategoryDelete"
      title="Hapus kategori ini?"
      description="Semua mata uji di dalam kategori ini akan ikut terhapus. Tindakan ini tidak bisa dibatalkan."
      confirm-label="Hapus"
      @confirm="confirmDeleteCategory"
    />
    <ConfirmDialog
      v-model="showSubjectDelete"
      title="Hapus mata uji ini?"
      description="Tindakan ini tidak bisa dibatalkan."
      confirm-label="Hapus"
      @confirm="confirmDeleteSubject"
    />
  </div>
</template>
