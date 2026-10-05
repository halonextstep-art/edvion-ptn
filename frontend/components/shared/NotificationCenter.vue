<script setup lang="ts">
// Real notification bell backed by the actual `notifications` table (see backend
// `application::notification_service` doc comment) — used by Admin, Siswa, and Konten
// headers. Portal Sekolah keeps its own richer `NotificationBell` (which also derives
// notifications from upcoming events + recent activity) rather than this simpler one, since
// merging both sources there would need a bigger rework; this component only shows real
// persisted notifications, honestly empty until something actually triggers one (e.g. an
// admin approving a manual purchase request).
import { Bell, CheckCheck, Loader2, ShoppingBag, Sparkles } from 'lucide-vue-next'
import type { NotificationItem } from '~/types'

const emit = defineEmits<{ navigate: [tab: string] }>()
const { notificationService } = useApi()

const open = ref(false)
const loading = ref(true)
const items = ref<NotificationItem[]>([])
const unreadCount = ref(0)

async function loadCount() {
  try {
    const res = await notificationService.unreadCount()
    unreadCount.value = res.count
  } catch {
    // non-critical — badge just won't show
  }
}
async function loadList() {
  loading.value = true
  try {
    items.value = await notificationService.listMine(20)
  } catch {
    items.value = []
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadCount()
  loadList()
})

async function toggle() {
  open.value = !open.value
  if (open.value) {
    await loadList()
    await loadCount()
  }
}

async function markAllRead() {
  try {
    await notificationService.markAllRead()
    items.value = items.value.map((n) => ({ ...n, read: true }))
    unreadCount.value = 0
  } catch {
    // non-critical
  }
}

async function handleClick(n: NotificationItem) {
  if (!n.read) {
    try {
      await notificationService.markRead(n.id)
      n.read = true
      unreadCount.value = Math.max(0, unreadCount.value - 1)
    } catch {
      // non-critical — item stays visually unread, no functional harm
    }
  }
  if (n.link_tab) emit('navigate', n.link_tab)
}

function iconFor(kind: string) {
  return kind === 'purchase_approved' || kind === 'purchase_pending' ? ShoppingBag : Sparkles
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
    <!-- `fixed` + `left-3 right-3` di mobile (bukan `absolute right-0 w-80`) — bel ini biasanya
         TIDAK jadi elemen paling kanan di header (masih ada avatar & tombol keluar sesudahnya),
         jadi lebar tetap yang di-anchor dari kanan bel bisa terpotong di layar sempit. Dipin ke
         tepi viewport langsung di bawah header supaya selalu muat berapa pun posisi bel-nya;
         kembali ke pola absolute+lebar tetap di layar >= sm yang ruangnya sudah cukup lapang. -->
    <div
      v-if="open"
      class="fixed left-3 right-3 top-[60px] sm:absolute sm:left-auto sm:right-0 sm:top-auto sm:mt-2 sm:w-80 bg-white rounded-2xl shadow-2xl border z-50 overflow-hidden"
    >
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
          :class="!n.read ? 'bg-indigo-50/40' : ''"
          @click="handleClick(n)"
        >
          <div class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0 mt-0.5 bg-indigo-50 border border-indigo-100">
            <component :is="iconFor(n.kind)" class="w-3.5 h-3.5 text-indigo-500" />
          </div>
          <div class="min-w-0 flex-1">
            <p class="text-xs font-semibold text-slate-900 truncate">{{ n.title }}</p>
            <p class="text-xs text-muted-foreground line-clamp-2">{{ n.message }}</p>
            <p class="text-[10px] text-muted-foreground mt-0.5">{{ fmt(n.created_at) }}</p>
          </div>
          <span v-if="!n.read" class="w-1.5 h-1.5 rounded-full bg-indigo-500 mt-1.5 shrink-0" />
        </button>
      </div>
    </div>
  </div>
</template>
