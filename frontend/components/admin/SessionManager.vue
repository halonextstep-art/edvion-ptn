<script setup lang="ts">
import { Plus, Pencil, Trash2, Trophy, Zap, Clock } from 'lucide-vue-next'
import type { TryoutSessionTemplate } from '~/types'

const { tryoutService } = useApi()
const toast = useToast()

// Deep-link dari kartu subtes di PackageWizard ("Kelola Sesi ini") — lihat ContentHub.vue.
// Kalau diisi dan cocok dengan salah satu sesi, dialog edit sesi itu langsung terbuka begitu
// komponen ini mount, supaya admin tidak perlu mencari-cari sendiri di daftar.
const props = defineProps<{ focusSessionId?: string | null }>()

const sessions = ref<TryoutSessionTemplate[]>([])
const loading = ref(false)

const showForm = ref(false)
const editTarget = ref<TryoutSessionTemplate | null>(null)
const showDelete = ref(false)
const deleteTarget = ref<TryoutSessionTemplate | null>(null)

const typeBadge: Record<string, { label: string; variant: any }> = {
  tryout: { label: 'Tryout', variant: 'default' },
  drilling: { label: 'Drilling', variant: 'warning' },
  mini: { label: 'Mini Tryout', variant: 'secondary' },
}

async function load() {
  loading.value = true
  try {
    sessions.value = await tryoutService.listSessions()
  } catch (e: any) {
    toast.error('Gagal memuat daftar sesi', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(async () => {
  await load()
  if (props.focusSessionId) {
    const target = sessions.value.find((s) => s.id === props.focusSessionId)
    if (target) openEdit(target)
  }
})

function openCreate() {
  editTarget.value = null
  showForm.value = true
}
function openEdit(s: TryoutSessionTemplate) {
  editTarget.value = s
  showForm.value = true
}
function openDelete(s: TryoutSessionTemplate) {
  deleteTarget.value = s
  showDelete.value = true
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  try {
    await tryoutService.deleteSession(deleteTarget.value.id)
    toast.success('Sesi berhasil dihapus!')
    load()
  } catch (e: any) {
    toast.error('Gagal menghapus sesi', e?.message)
  }
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2"><Zap class="w-5 h-5 text-indigo-600" />Sesi Tryout &amp; Drilling</h2>
        <p class="text-sm text-muted-foreground">{{ sessions.length }} sesi tersedia untuk siswa</p>
        <p class="text-xs text-muted-foreground mt-0.5">Untuk bikin paket baru lengkap dengan subtes+soal, pakai "Buat Paket + Subtes" di tab Paket — tab ini untuk edit detail sesi yang sudah ada (judul, durasi, mapel pilihan) atau hapus sesi.</p>
      </div>
      <Button variant="gradient" @click="openCreate"><Plus class="w-4 h-4" />Buat Sesi</Button>
    </div>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
    <Card v-else-if="sessions.length === 0" class="p-10 text-center text-sm text-muted-foreground">Belum ada sesi. Buat sesi pertama untuk siswa.</Card>
    <div v-else class="space-y-2.5">
      <Card v-for="s in sessions" :key="s.id" class="p-4">
        <div class="flex items-start gap-4">
          <div class="w-11 h-11 rounded-xl flex items-center justify-center shrink-0" :class="s.session_type === 'tryout' ? 'bg-indigo-100' : 'bg-orange-100'">
            <Trophy v-if="s.session_type === 'tryout'" class="w-5 h-5 text-indigo-600" />
            <Zap v-else class="w-5 h-5 text-orange-600" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 flex-wrap mb-1">
              <h3 class="font-bold text-sm text-slate-900">{{ s.title }}</h3>
              <Badge :variant="typeBadge[s.session_type]?.variant">{{ typeBadge[s.session_type]?.label }}</Badge>
              <Badge v-if="s.is_premium" variant="warning">Premium</Badge>
            </div>
            <div class="flex items-center gap-3 text-xs text-muted-foreground flex-wrap">
              <span class="flex items-center gap-1"><Clock class="w-3 h-3" />{{ s.duration_minutes }} menit</span>
              <span>{{ s.question_count }} soal</span>
              <span v-if="s.subject_filter">Mata uji: {{ s.subject_filter }}</span>
              <span v-if="s.topic_filter">Topik: {{ s.topic_filter }}</span>
              <span v-if="s.difficulty_filter">Kesulitan: {{ s.difficulty_filter }}</span>
            </div>
          </div>
          <div class="flex items-center gap-1 shrink-0">
            <button class="p-1.5 rounded hover:bg-slate-100" title="Edit" @click="openEdit(s)"><Pencil class="w-4 h-4 text-slate-500" /></button>
            <button class="p-1.5 rounded hover:bg-red-50" title="Hapus" @click="openDelete(s)"><Trash2 class="w-4 h-4 text-red-500" /></button>
          </div>
        </div>
      </Card>
    </div>

    <SessionFormDialog v-model="showForm" :edit-session="editTarget" @saved="load" />
    <ConfirmDialog
      v-model="showDelete"
      title="Hapus sesi ini?"
      description="Sesi yang sudah pernah dikerjakan siswa (punya riwayat attempt) tidak bisa dihapus."
      confirm-label="Hapus"
      @confirm="confirmDelete"
    />
  </div>
</template>
