<script setup lang="ts">
// Notification bell (Portal Sekolah) — dropdown fungsional, 100% data nyata dari 3 sumber:
//  - Event mendatang (eventService.upcoming) → tipe "info", tandai-dibaca via localStorage
//  - Aktivitas siswa terbaru (schoolService.recentActivity) → tipe "success", localStorage
//  - Notifikasi nyata dari tabel `notifications` backend (mis. konfirmasi pembelian paket
//    oleh admin — Sekolah juga bisa jadi pembeli, lihat PaymentService::fulfill) → tipe
//    "real", status baca disimpan di backend (bukan localStorage) karena datanya memang ada
//    baris persisten di database.
// Dua sumber pertama TIDAK punya tabel tersendiri di backend, jadi tetap di-derive live
// seperti sebelumnya — bukan di-hardcode, dan status bacanya tetap di localStorage (state UI
// murni) supaya tidak berpura-pura ada fitur backend yang sebenarnya tidak ada untuk mereka.
import { Bell, Calendar, Trophy, ShoppingBag, CheckCheck, Loader2 } from 'lucide-vue-next'

interface NotificationItem {
  id: string
  type: 'info' | 'success' | 'real'
  title: string
  detail: string
  at: string
  /** Only set for `type: 'real'` — the actual backend notification id, used to call
   * `notificationService.markRead`. Derived (event/activity) items use `readIds` instead. */
  realId?: string
  read?: boolean
  linkTab?: string | null
}

const emit = defineEmits<{ navigate: [tab: string] }>()
const { eventService, schoolService, notificationService } = useApi()

const open = ref(false)
const loading = ref(true)
const items = ref<NotificationItem[]>([])
const readIds = ref<Set<string>>(new Set())

const STORAGE_KEY = 'edvionptn_school_notif_read'

function loadReadIds() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) readIds.value = new Set(JSON.parse(raw))
  } catch {
    // ignore malformed storage
  }
}
function persistReadIds() {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify([...readIds.value]))
  } catch {
    // storage unavailable — non-critical, read state just won't persist
  }
}

async function load() {
  loading.value = true
  try {
    const [events, activity, real] = await Promise.all([
      eventService.upcoming(5),
      schoolService.recentActivity(5),
      notificationService.listMine(20).catch(() => []),
    ])
    const eventItems: NotificationItem[] = events.map((e) => ({
      id: `event-${e.id}`,
      type: 'info',
      title: 'Event mendatang',
      detail: `${e.name} — ${new Date(e.start_date).toLocaleDateString('id-ID', { day: '2-digit', month: 'short' })}`,
      at: e.start_date,
    }))
    const activityItems: NotificationItem[] = activity.map((a, i) => ({
      id: `activity-${a.submitted_at}-${i}`,
      type: 'success',
      title: `${a.student_name} menyelesaikan sesi`,
      detail: `"${a.session_title}"${a.score != null ? ` — skor ${a.score}` : ''}`,
      at: a.submitted_at,
    }))
    const realItems: NotificationItem[] = real.map((n) => ({
      id: `real-${n.id}`,
      type: 'real',
      title: n.title,
      detail: n.message,
      at: n.created_at,
      realId: n.id,
      read: n.read,
      linkTab: n.link_tab,
    }))
    items.value = [...eventItems, ...activityItems, ...realItems].sort((a, b) => new Date(b.at).getTime() - new Date(a.at).getTime())
  } catch {
    items.value = []
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadReadIds()
  load()
})

function isUnread(n: NotificationItem) {
  return n.type === 'real' ? !n.read : !readIds.value.has(n.id)
}
const unreadCount = computed(() => items.value.filter(isUnread).length)

async function markAllRead() {
  for (const n of items.value) {
    if (n.type === 'real') n.read = true
    else readIds.value.add(n.id)
  }
  persistReadIds()
  try {
    await notificationService.markAllRead()
  } catch {
    // non-critical — local state already reflects "read"
  }
}
async function markRead(n: NotificationItem) {
  if (n.type === 'real' && n.realId) {
    n.read = true
    try {
      await notificationService.markRead(n.realId)
    } catch {
      // non-critical
    }
    if (n.linkTab) emit('navigate', n.linkTab)
  } else {
    readIds.value.add(n.id)
    persistReadIds()
  }
}
function toggle() {
  open.value = !open.value
}
function fmt(iso: string) {
  return new Date(iso).toLocaleString('id-ID', { day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit' })
}
</script>

<template>
  <div class="relative">
    <button class="relative w-9 h-9 rounded-xl hover:bg-slate-100 flex items-center justify-center transition-colors" @click="toggle">
      <Bell class="w-4 h-4 text-slate-500" />
      <span v-if="unreadCount > 0" class="absolute top-1 right-1 min-w-[14px] h-[14px] px-0.5 rounded-full bg-red-500 text-white text-[9px] font-bold flex items-center justify-center">{{ unreadCount > 9 ? '9+' : unreadCount }}</span>
    </button>

    <Teleport to="body">
      <div v-if="open" class="fixed inset-0 z-40" @click="open = false" />
    </Teleport>
    <!-- `fixed left-3 right-3` di mobile (bukan `absolute right-0 w-80`) — pola sama persis
         dengan components/shared/NotificationCenter.vue milik Portal Siswa (task #178): w-80
         fixed yang di-anchor `right-0` relatif terhadap tombol bell yang mepet tepi kanan bisa
         mendorong panel meluber ke kiri viewport di HP sempit (<360px). Cuma jadi `absolute`
         w-80 anchored di sm+ ke atas. -->
    <div v-if="open" class="fixed left-3 right-3 top-[60px] sm:absolute sm:left-auto sm:right-0 sm:top-auto sm:mt-2 sm:w-80 bg-white rounded-2xl shadow-2xl border z-50 overflow-hidden">
      <div class="flex items-center justify-between px-4 py-3 border-b">
        <span class="font-bold text-sm text-slate-900">Notifikasi</span>
        <button v-if="unreadCount > 0" class="text-xs font-semibold text-indigo-600 hover:underline flex items-center gap-1" @click="markAllRead">
          <CheckCheck class="w-3.5 h-3.5" />Tandai semua dibaca
        </button>
      </div>
      <div v-if="loading" class="py-10 text-center text-sm text-muted-foreground"><Loader2 class="w-5 h-5 animate-spin mx-auto mb-1" />Memuat...</div>
      <div v-else-if="items.length === 0" class="py-10 text-center text-sm text-muted-foreground">Belum ada notifikasi.</div>
      <div v-else class="max-h-96 overflow-y-auto divide-y divide-slate-50">
        <button
          v-for="n in items" :key="n.id"
          class="w-full text-left flex items-start gap-3 px-4 py-3 hover:bg-slate-50 transition-colors"
          :class="isUnread(n) ? 'bg-indigo-50/40' : ''"
          @click="markRead(n)"
        >
          <div
            class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0 mt-0.5"
            :class="n.type === 'info' ? 'bg-blue-50 border border-blue-100' : n.type === 'success' ? 'bg-emerald-50 border border-emerald-100' : 'bg-indigo-50 border border-indigo-100'"
          >
            <Calendar v-if="n.type === 'info'" class="w-3.5 h-3.5 text-blue-500" />
            <Trophy v-else-if="n.type === 'success'" class="w-3.5 h-3.5 text-emerald-500" />
            <ShoppingBag v-else class="w-3.5 h-3.5 text-indigo-500" />
          </div>
          <div class="min-w-0 flex-1">
            <p class="text-xs font-semibold text-slate-900 truncate">{{ n.title }}</p>
            <p class="text-xs text-muted-foreground truncate">{{ n.detail }}</p>
            <p class="text-[10px] text-muted-foreground mt-0.5">{{ fmt(n.at) }}</p>
          </div>
          <span v-if="isUnread(n)" class="w-1.5 h-1.5 rounded-full bg-indigo-500 mt-1.5 shrink-0" />
        </button>
      </div>
    </div>
  </div>
</template>
