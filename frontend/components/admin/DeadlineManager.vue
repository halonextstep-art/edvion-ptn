<script setup lang="ts">
import { Plus, CalendarClock, Pencil, Trash2, Search, Clock, AlertTriangle } from 'lucide-vue-next'
import type { AdmissionDeadlineItem, AdmissionTrack } from '~/types'

const { admissionDeadlineService } = useApi()
const toast = useToast()

const deadlines = ref<AdmissionDeadlineItem[]>([])
const loading = ref(false)
const search = ref('')
const filterTrack = ref<AdmissionTrack | 'all'>('all')

const showForm = ref(false)
const editTarget = ref<AdmissionDeadlineItem | null>(null)
const showDelete = ref(false)
const deleteTarget = ref<AdmissionDeadlineItem | null>(null)

const trackConfig: Record<AdmissionTrack, { label: string; color: string; bg: string }> = {
  snbp: { label: 'SNBP', color: 'text-emerald-700', bg: 'bg-emerald-100' },
  snbt: { label: 'SNBT', color: 'text-blue-700', bg: 'bg-blue-100' },
  utbk: { label: 'UTBK', color: 'text-purple-700', bg: 'bg-purple-100' },
}

const filtered = computed(() =>
  deadlines.value.filter((d) => {
    const ms = !search.value.trim() || d.label.toLowerCase().includes(search.value.toLowerCase())
    const mt = filterTrack.value === 'all' || d.track === filterTrack.value
    return ms && mt
  }),
)

async function load() {
  loading.value = true
  try {
    deadlines.value = await admissionDeadlineService.list()
  } catch (e: any) {
    toast.error('Gagal memuat kalender deadline', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

function openCreate() {
  editTarget.value = null
  showForm.value = true
}
function openEdit(d: AdmissionDeadlineItem) {
  editTarget.value = d
  showForm.value = true
}
function openDelete(d: AdmissionDeadlineItem) {
  deleteTarget.value = d
  showDelete.value = true
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  try {
    await admissionDeadlineService.remove(deleteTarget.value.id)
    toast.success(`Deadline "${deleteTarget.value.label}" dihapus`)
    load()
  } catch (e: any) {
    toast.error('Gagal menghapus deadline', e?.message)
  }
}

function formatDate(d: string) {
  return new Date(d).toLocaleDateString('id-ID', { weekday: 'long', day: '2-digit', month: 'long', year: 'numeric' })
}

// days_remaining dihitung live oleh backend (bukan dihitung ulang di sini) supaya konsisten
// dengan widget "Deadline Mendatang" yang dipakai Sekolah/Siswa — lihat AdmissionDeadlineResponse.
function countdownLabel(days: number) {
  if (days < 0) return `Lewat ${Math.abs(days)} hari lalu`
  if (days === 0) return 'Hari ini!'
  if (days === 1) return 'Besok'
  return `${days} hari lagi`
}
function countdownVariant(days: number): 'destructive' | 'warning' | 'success' | 'outline' {
  if (days < 0) return 'outline'
  if (days <= 3) return 'destructive'
  if (days <= 14) return 'warning'
  return 'success'
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2"><CalendarClock class="w-5 h-5 text-indigo-600" />Kalender Deadline SNBP/SNBT/UTBK</h2>
        <p class="text-sm text-muted-foreground">Tanggal resmi jalur seleksi (dari SNPMB) — dibaca live sebagai widget countdown oleh Sekolah &amp; Siswa</p>
      </div>
      <Button variant="gradient" @click="openCreate"><Plus class="w-4 h-4" />Tambah Deadline</Button>
    </div>

    <Card class="p-4">
      <div class="flex flex-col sm:flex-row gap-3">
        <div class="relative flex-1">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input v-model="search" placeholder="Cari label deadline..." class="w-full pl-9 pr-3 py-2 border rounded-md text-sm bg-white" />
        </div>
        <select v-model="filterTrack" class="px-3 py-2 border rounded-md text-sm bg-white">
          <option value="all">Semua Jalur</option>
          <option value="snbp">SNBP</option>
          <option value="snbt">SNBT</option>
          <option value="utbk">UTBK</option>
        </select>
      </div>
    </Card>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
    <Card v-else-if="filtered.length === 0" class="p-10 text-center">
      <CalendarClock class="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" />
      <p class="text-muted-foreground text-sm">{{ deadlines.length === 0 ? 'Belum ada deadline. Tambah deadline pertama.' : 'Tidak ada deadline yang cocok dengan filter.' }}</p>
    </Card>
    <div v-else class="space-y-3">
      <Card v-for="d in filtered" :key="d.id" class="p-5 hover:shadow-md transition-shadow">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 flex-wrap mb-1.5">
              <div class="w-9 h-9 rounded-lg flex items-center justify-center shrink-0" :class="trackConfig[d.track].bg">
                <CalendarClock class="w-4 h-4" :class="trackConfig[d.track].color" />
              </div>
              <h3 class="font-bold text-slate-900">{{ d.label }}</h3>
              <Badge variant="outline">{{ trackConfig[d.track].label }} {{ d.year }}</Badge>
              <Badge :variant="countdownVariant(d.days_remaining)" class="gap-1">
                <AlertTriangle v-if="d.days_remaining >= 0 && d.days_remaining <= 3" class="w-3 h-3" />
                <Clock v-else class="w-3 h-3" />
                {{ countdownLabel(d.days_remaining) }}
              </Badge>
            </div>
            <p class="text-xs text-muted-foreground ml-11">{{ formatDate(d.deadline_date) }}</p>
            <p v-if="d.description" class="text-xs text-muted-foreground ml-11 mt-1">{{ d.description }}</p>
          </div>

          <div class="flex items-center gap-2 shrink-0">
            <button class="p-1.5 rounded hover:bg-slate-100" title="Edit" @click="openEdit(d)"><Pencil class="w-4 h-4 text-slate-500" /></button>
            <button class="p-1.5 rounded hover:bg-red-50" title="Hapus" @click="openDelete(d)"><Trash2 class="w-4 h-4 text-red-500" /></button>
          </div>
        </div>
      </Card>
    </div>

    <DeadlineFormDialog v-model="showForm" :edit-deadline="editTarget" @saved="load" />
    <ConfirmDialog
      v-model="showDelete"
      title="Hapus deadline ini?"
      description="Tindakan ini tidak bisa dibatalkan. Widget countdown Sekolah/Siswa tidak akan lagi menampilkan deadline ini."
      confirm-label="Hapus"
      @confirm="confirmDelete"
    />
  </div>
</template>
