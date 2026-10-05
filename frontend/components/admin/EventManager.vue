<script setup lang="ts">
import {
  Plus, Calendar, Users, Clock, Pencil, Trash2, CheckCircle, Archive, Play,
  Search, TrendingUp, DollarSign, Layers, Trophy, Eye, X,
} from 'lucide-vue-next'
import type { EventItem, EventStatus, EventType } from '~/types'

const { eventService } = useApi()
const toast = useToast()

const events = ref<EventItem[]>([])
const loading = ref(false)
const search = ref('')
const filterStatus = ref<EventStatus | 'all'>('all')
const filterType = ref<EventType | 'all'>('all')

const showForm = ref(false)
const editTarget = ref<EventItem | null>(null)
const showDelete = ref(false)
const deleteTarget = ref<EventItem | null>(null)
const detailEvent = ref<EventItem | null>(null)

const typeConfig: Record<EventType, { label: string; color: string; bg: string; icon: any }> = {
  tryout: { label: 'Tryout', color: 'text-indigo-700', bg: 'bg-indigo-100', icon: Trophy },
  drilling: { label: 'Drilling', color: 'text-orange-700', bg: 'bg-orange-100', icon: Layers },
  mini: { label: 'Mini Tryout', color: 'text-blue-700', bg: 'bg-blue-100', icon: Play },
}
const statusConfig: Record<EventStatus, { label: string; variant: any }> = {
  draft: { label: 'Draft', variant: 'secondary' },
  upcoming: { label: 'Dijadwalkan', variant: 'warning' },
  ongoing: { label: 'Berlangsung', variant: 'success' },
  completed: { label: 'Selesai', variant: 'default' },
  archived: { label: 'Diarsipkan', variant: 'secondary' },
}

const filtered = computed(() =>
  events.value.filter((e) => {
    const ms = !search.value.trim() || e.name.toLowerCase().includes(search.value.toLowerCase())
    const mSt = filterStatus.value === 'all' || e.status === filterStatus.value
    const mTy = filterType.value === 'all' || e.event_type === filterType.value
    return ms && mSt && mTy
  }),
)

const stats = computed(() => ({
  ongoing: events.value.filter((e) => e.status === 'ongoing').length,
  upcoming: events.value.filter((e) => e.status === 'upcoming').length,
  totalParticipants: events.value.reduce((s, e) => s + e.participants, 0),
  revenue: events.value.reduce((s, e) => s + e.price * e.participants, 0),
}))

async function load() {
  loading.value = true
  try {
    events.value = await eventService.list()
  } catch (e: any) {
    toast.error('Gagal memuat daftar event', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

function openCreate() {
  editTarget.value = null
  showForm.value = true
}
function openEdit(e: EventItem) {
  editTarget.value = e
  showForm.value = true
}
function openDelete(e: EventItem) {
  deleteTarget.value = e
  showDelete.value = true
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  try {
    await eventService.remove(deleteTarget.value.id)
    toast.success(`Event "${deleteTarget.value.name}" dihapus`)
    detailEvent.value = null
    load()
  } catch (e: any) {
    toast.error('Gagal menghapus event', e?.message)
  }
}

async function changeStatus(ev: EventItem, status: EventStatus) {
  try {
    await eventService.setStatus(ev.id, status)
    toast.success('Status event diperbarui')
    load()
  } catch (e: any) {
    toast.error('Gagal mengubah status event', e?.message)
  }
}

function formatDate(d?: string | null) {
  if (!d) return '-'
  return new Date(d).toLocaleDateString('id-ID', { day: '2-digit', month: 'short', year: 'numeric' })
}
function formatPrice(p: number) {
  return p === 0 ? 'Gratis' : `Rp ${(p / 1000).toFixed(0)}K`
}

// Kapasitas bisa >100% kalau pendaftaran melebihi kuota (over-booking nyata, bukan bug) —
// dulu label persen tidak di-clamp tapi bar visualnya di-clamp ke 100%, jadi kelihatan
// tidak sinkron (mis. label "134%" tapi bar cuma penuh). Sekarang label & warna sama-sama
// mencerminkan kondisi kelebihan kapasitas secara jujur.
function capacityPercent(participants: number, max: number) {
  return max > 0 ? Math.round((participants / max) * 100) : 0
}
function isOverCapacity(participants: number, max: number) {
  return max > 0 && participants > max
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2"><Calendar class="w-5 h-5 text-indigo-600" />Manajemen Event</h2>
        <p class="text-sm text-muted-foreground">Jadwalkan tryout, drilling, dan mini tryout berdasarkan paket soal yang sudah dibuat</p>
      </div>
      <Button variant="gradient" @click="openCreate"><Plus class="w-4 h-4" />Buat Event Baru</Button>
    </div>

    <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
      <Card class="p-4 border-l-4 border-l-green-500">
        <div class="flex items-center justify-between">
          <div><p class="text-sm text-muted-foreground mb-1">Sedang Berlangsung</p><h3 class="text-2xl font-bold">{{ stats.ongoing }}</h3></div>
          <div class="w-10 h-10 rounded-lg bg-green-100 flex items-center justify-center"><Play class="w-5 h-5 text-green-600" /></div>
        </div>
      </Card>
      <Card class="p-4 border-l-4 border-l-orange-500">
        <div class="flex items-center justify-between">
          <div><p class="text-sm text-muted-foreground mb-1">Terjadwal</p><h3 class="text-2xl font-bold">{{ stats.upcoming }}</h3></div>
          <div class="w-10 h-10 rounded-lg bg-orange-100 flex items-center justify-center"><Clock class="w-5 h-5 text-orange-600" /></div>
        </div>
      </Card>
      <Card class="p-4 border-l-4 border-l-blue-500">
        <div class="flex items-center justify-between">
          <div><p class="text-sm text-muted-foreground mb-1">Total Peserta</p><h3 class="text-2xl font-bold">{{ stats.totalParticipants.toLocaleString('id-ID') }}</h3></div>
          <div class="w-10 h-10 rounded-lg bg-blue-100 flex items-center justify-center"><Users class="w-5 h-5 text-blue-600" /></div>
        </div>
      </Card>
      <Card class="p-4 border-l-4 border-l-purple-500">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-muted-foreground mb-1">Estimasi Revenue</p>
            <h3 class="text-2xl font-bold">Rp {{ (stats.revenue / 1_000_000).toFixed(1) }}Jt</h3>
          </div>
          <div class="w-10 h-10 rounded-lg bg-purple-100 flex items-center justify-center"><DollarSign class="w-5 h-5 text-purple-600" /></div>
        </div>
      </Card>
    </div>
    <p class="text-xs text-muted-foreground -mt-3">Total peserta dihitung dari jumlah siswa unik yang benar-benar mengerjakan sesi terkait; estimasi revenue = harga × peserta.</p>

    <Card class="p-4">
      <div class="flex flex-col sm:flex-row gap-3">
        <div class="relative flex-1">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input v-model="search" placeholder="Cari event..." class="w-full pl-9 pr-3 py-2 border rounded-md text-sm bg-white" />
        </div>
        <select v-model="filterStatus" class="px-3 py-2 border rounded-md text-sm bg-white">
          <option value="all">Semua Status</option>
          <option value="draft">Draft</option>
          <option value="upcoming">Dijadwalkan</option>
          <option value="ongoing">Berlangsung</option>
          <option value="completed">Selesai</option>
          <option value="archived">Diarsipkan</option>
        </select>
        <select v-model="filterType" class="px-3 py-2 border rounded-md text-sm bg-white">
          <option value="all">Semua Tipe</option>
          <option value="tryout">Tryout</option>
          <option value="drilling">Drilling</option>
          <option value="mini">Mini Tryout</option>
        </select>
      </div>
    </Card>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
    <Card v-else-if="filtered.length === 0" class="p-10 text-center">
      <Calendar class="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" />
      <p class="text-muted-foreground text-sm">{{ events.length === 0 ? 'Belum ada event. Buat event pertama.' : 'Tidak ada event yang cocok dengan filter.' }}</p>
    </Card>
    <div v-else class="space-y-3">
      <Card v-for="ev in filtered" :key="ev.id" class="p-5 hover:shadow-md transition-shadow">
        <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
          <div class="flex-1 min-w-0">
            <div class="flex items-start gap-3">
              <div class="w-11 h-11 rounded-xl flex items-center justify-center shrink-0" :class="typeConfig[ev.event_type].bg">
                <component :is="typeConfig[ev.event_type].icon" class="w-5 h-5" :class="typeConfig[ev.event_type].color" />
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 flex-wrap mb-1">
                  <h3 class="font-bold text-slate-900">{{ ev.name }}</h3>
                  <Badge :variant="statusConfig[ev.status].variant">{{ statusConfig[ev.status].label }}</Badge>
                  <Badge variant="outline">{{ typeConfig[ev.event_type].label }}</Badge>
                </div>
                <p v-if="ev.description" class="text-xs text-muted-foreground line-clamp-1">{{ ev.description }}</p>
                <p class="text-xs text-muted-foreground mt-0.5">Paket soal: {{ ev.session_title }} &middot; {{ ev.target_class }}</p>
                <div class="flex flex-wrap items-center gap-3 mt-1.5 text-xs text-muted-foreground">
                  <span class="flex items-center gap-1"><Calendar class="w-3.5 h-3.5" />{{ formatDate(ev.start_date) }}<template v-if="ev.end_date"> – {{ formatDate(ev.end_date) }}</template></span>
                  <span class="flex items-center gap-1"><Clock class="w-3.5 h-3.5" />{{ ev.duration_minutes }} menit</span>
                  <span class="flex items-center gap-1"><Users class="w-3.5 h-3.5" />{{ ev.participants.toLocaleString('id-ID') }} peserta<template v-if="ev.max_participants"> / {{ ev.max_participants.toLocaleString('id-ID') }}</template></span>
                  <span class="text-indigo-600 font-medium">{{ formatPrice(ev.price) }}</span>
                </div>
                <div v-if="ev.max_participants" class="mt-2 max-w-xs">
                  <div class="flex justify-between text-[11px] mb-1" :class="isOverCapacity(ev.participants, ev.max_participants) ? 'text-red-600 font-semibold' : 'text-muted-foreground'">
                    <span>Kapasitas ({{ capacityPercent(ev.participants, ev.max_participants) }}%{{ isOverCapacity(ev.participants, ev.max_participants) ? ' — kelebihan kuota' : '' }})</span>
                  </div>
                  <div class="h-1.5 bg-slate-100 rounded-full overflow-hidden">
                    <div
                      class="h-full rounded-full"
                      :class="isOverCapacity(ev.participants, ev.max_participants) ? 'bg-gradient-to-r from-red-500 to-rose-500' : 'bg-gradient-to-r from-indigo-500 to-purple-500'"
                      :style="{ width: `${Math.min(capacityPercent(ev.participants, ev.max_participants), 100)}%` }"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="flex items-center gap-2 flex-wrap shrink-0">
            <Button v-if="ev.status === 'draft'" size="sm" variant="outline" class="gap-1.5 text-green-700 border-green-300 hover:bg-green-50" @click="changeStatus(ev, 'upcoming')">
              <CheckCircle class="w-3.5 h-3.5" />Publish
            </Button>
            <Button v-if="ev.status === 'upcoming'" size="sm" variant="outline" class="gap-1.5 text-blue-700 border-blue-300 hover:bg-blue-50" @click="changeStatus(ev, 'ongoing')">
              <Play class="w-3.5 h-3.5" />Mulai
            </Button>
            <Button v-if="ev.status === 'ongoing'" size="sm" variant="outline" class="gap-1.5 text-orange-700 border-orange-300 hover:bg-orange-50" @click="changeStatus(ev, 'completed')">
              <CheckCircle class="w-3.5 h-3.5" />Selesaikan
            </Button>
            <Button v-if="ev.status === 'completed' || ev.status === 'upcoming'" size="sm" variant="outline" class="gap-1.5" @click="changeStatus(ev, 'archived')">
              <Archive class="w-3.5 h-3.5" />Arsipkan
            </Button>
            <Button size="sm" variant="outline" class="gap-1.5" @click="detailEvent = ev"><Eye class="w-3.5 h-3.5" />Detail</Button>
            <button class="p-1.5 rounded hover:bg-slate-100" title="Edit" @click="openEdit(ev)"><Pencil class="w-4 h-4 text-slate-500" /></button>
            <button class="p-1.5 rounded hover:bg-red-50" title="Hapus" @click="openDelete(ev)"><Trash2 class="w-4 h-4 text-red-500" /></button>
          </div>
        </div>
      </Card>
    </div>

    <EventFormDialog v-model="showForm" :edit-event="editTarget" @saved="load" />
    <ConfirmDialog
      v-model="showDelete"
      title="Hapus event ini?"
      description="Tindakan ini tidak bisa dibatalkan. Riwayat attempt siswa pada paket soal terkait tidak ikut terhapus."
      confirm-label="Hapus"
      @confirm="confirmDelete"
    />

    <!-- Detail drawer -->
    <Teleport to="body">
      <div v-if="detailEvent" class="fixed inset-0 bg-black/40 flex items-center justify-end z-50 p-4" @click="detailEvent = null">
        <div class="bg-white h-full w-full max-w-md rounded-2xl shadow-2xl overflow-y-auto" @click.stop>
          <div class="flex items-center justify-between p-5 border-b">
            <h3 class="font-bold text-slate-900 text-base">Detail Event</h3>
            <button class="p-2 rounded-xl hover:bg-slate-100" @click="detailEvent = null"><X class="w-4 h-4" /></button>
          </div>
          <div class="p-5 space-y-5">
            <div>
              <div class="flex items-center gap-2 flex-wrap mb-2">
                <Badge :variant="statusConfig[detailEvent.status].variant">{{ statusConfig[detailEvent.status].label }}</Badge>
                <Badge variant="outline">{{ typeConfig[detailEvent.event_type].label }}</Badge>
              </div>
              <h2 class="text-xl font-bold text-slate-900">{{ detailEvent.name }}</h2>
              <p v-if="detailEvent.description" class="text-sm text-muted-foreground mt-1">{{ detailEvent.description }}</p>
            </div>

            <div
              v-for="[label, val] in [
                ['Tanggal', detailEvent.end_date ? `${formatDate(detailEvent.start_date)} – ${formatDate(detailEvent.end_date)}` : formatDate(detailEvent.start_date)],
                ['Durasi', `${detailEvent.duration_minutes} menit`],
                ['Target', detailEvent.target_class],
                ['Paket Soal', detailEvent.session_title],
                ['Harga', detailEvent.price === 0 ? 'Gratis' : `Rp ${detailEvent.price.toLocaleString('id-ID')}`],
                ['Peserta', `${detailEvent.participants.toLocaleString('id-ID')} / ${detailEvent.max_participants ? detailEvent.max_participants.toLocaleString('id-ID') : '∞'}`],
                ['Hadiah', detailEvent.prizes || '—'],
              ]"
              :key="label"
              class="flex justify-between py-2.5 border-b last:border-0"
            >
              <span class="text-sm text-muted-foreground">{{ label }}</span>
              <span class="text-sm font-semibold text-slate-800 text-right max-w-[60%]">{{ val }}</span>
            </div>

            <div v-if="detailEvent.max_participants">
              <div class="flex justify-between text-xs mb-1.5" :class="isOverCapacity(detailEvent.participants, detailEvent.max_participants) ? 'text-red-600 font-semibold' : 'text-muted-foreground'">
                <span>Kapasitas terisi{{ isOverCapacity(detailEvent.participants, detailEvent.max_participants) ? ' — kelebihan kuota' : '' }}</span>
                <span>{{ capacityPercent(detailEvent.participants, detailEvent.max_participants) }}%</span>
              </div>
              <div class="h-2 bg-slate-100 rounded-full overflow-hidden">
                <div
                  class="h-full rounded-full"
                  :class="isOverCapacity(detailEvent.participants, detailEvent.max_participants) ? 'bg-gradient-to-r from-red-500 to-rose-500' : 'bg-gradient-to-r from-indigo-500 to-purple-500'"
                  :style="{ width: `${Math.min(capacityPercent(detailEvent.participants, detailEvent.max_participants), 100)}%` }"
                />
              </div>
            </div>

            <div class="flex gap-2 pt-2">
              <Button variant="outline" class="flex-1 gap-2" @click="openEdit(detailEvent); detailEvent = null"><Pencil class="w-4 h-4" />Edit</Button>
              <Button variant="outline" class="flex-1 gap-2 text-red-600 border-red-200 hover:bg-red-50" @click="openDelete(detailEvent)"><Trash2 class="w-4 h-4" />Hapus</Button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
