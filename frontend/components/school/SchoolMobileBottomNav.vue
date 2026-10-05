<script setup lang="ts">
// Bottom navigation bar khusus mobile (lg:hidden) untuk Portal Sekolah — pola SAMA PERSIS
// dengan MobileBottomNav.vue milik Portal Siswa, disesuaikan untuk 7 tab Sekolah (vs 6 tab
// Siswa). 4 tab yang paling sering dipakai PIC sekolah sehari-hari dapat slot tetap (Beranda,
// Siswa, Analytics, Laporan) — 3 sub-menu Rasionalisasi (SNBP/Rekap SNBT/Rekap TKA), yang lebih
// jarang dibuka dan sifatnya musiman (menjelang SNBP/SNBT), dikumpulkan di sheet "Lainnya"
// bersama Profil & Keluar (dulu cuma ada di header, yang di-collapse habis di mobile). v-model
// mengikat langsung ke `activeTab` yang sama dipakai <Tabs> di sekolah.vue, jadi bottom nav
// mobile & tab atas desktop selalu sinkron satu state.
import { LayoutDashboard, Users, TrendingUp, FileText, MoreHorizontal, Target, ClipboardList, ListChecks, UserRound, LogOut, X } from 'lucide-vue-next'

const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{
  'update:modelValue': [value: string]
  openProfile: []
  logout: []
}>()

const showMore = ref(false)

const primaryItems = [
  { value: 'overview', label: 'Beranda', icon: LayoutDashboard },
  { value: 'students', label: 'Siswa', icon: Users },
  { value: 'analytics', label: 'Analytics', icon: TrendingUp },
  { value: 'reports', label: 'Laporan', icon: FileText },
] as const

const moreTabItems = [
  { value: 'snbp', label: 'Rasionalisasi SNBP', icon: Target },
  { value: 'snbt', label: 'Rekap SNBT', icon: ClipboardList },
  { value: 'tka', label: 'Rekap TKA', icon: ListChecks },
] as const

// "Lainnya" ikut ter-highlight kalau tab aktif adalah salah satu yang disembunyikan di
// dalamnya — sama seperti pola isMoreActive di MobileBottomNav.vue Siswa.
const isMoreActive = computed(() => moreTabItems.some((i) => i.value === props.modelValue))

function selectTab(value: string) {
  emit('update:modelValue', value)
  showMore.value = false
}
</script>

<template>
  <!-- Bar utama -->
  <nav
    class="lg:hidden fixed bottom-0 inset-x-0 z-40 bg-white/95 backdrop-blur-lg border-t grid grid-cols-5"
    style="padding-bottom: env(safe-area-inset-bottom)"
  >
    <button
      v-for="item in primaryItems" :key="item.value"
      class="flex flex-col items-center justify-center gap-0.5 py-2 min-h-[56px] transition-colors"
      :class="modelValue === item.value ? 'text-blue-600' : 'text-slate-400'"
      @click="selectTab(item.value)"
    >
      <component :is="item.icon" class="w-5 h-5" :class="modelValue === item.value ? 'scale-110' : ''" />
      <span class="text-[10px] font-semibold leading-none">{{ item.label }}</span>
    </button>
    <button
      class="flex flex-col items-center justify-center gap-0.5 py-2 min-h-[56px] transition-colors"
      :class="isMoreActive || showMore ? 'text-blue-600' : 'text-slate-400'"
      @click="showMore = true"
    >
      <MoreHorizontal class="w-5 h-5" />
      <span class="text-[10px] font-semibold leading-none">Lainnya</span>
    </button>
  </nav>

  <!-- Sheet "Lainnya" -->
  <Teleport to="body">
    <Transition name="sheet-fade">
      <div v-if="showMore" class="lg:hidden fixed inset-0 z-50 bg-black/40" @click="showMore = false" />
    </Transition>
    <Transition name="sheet-slide">
      <div
        v-if="showMore"
        class="lg:hidden fixed bottom-0 inset-x-0 z-50 bg-white rounded-t-3xl shadow-2xl"
        style="padding-bottom: env(safe-area-inset-bottom)"
      >
        <div class="flex items-center justify-between px-5 pt-4 pb-2">
          <p class="text-sm font-bold text-slate-900">Lainnya</p>
          <button class="p-1.5 rounded-full hover:bg-slate-100" @click="showMore = false"><X class="w-4 h-4 text-slate-400" /></button>
        </div>
        <div class="px-3 pb-2 space-y-0.5">
          <button
            v-for="item in moreTabItems" :key="item.value"
            class="w-full flex items-center gap-3 px-3 py-3 rounded-xl text-sm font-semibold transition-colors"
            :class="modelValue === item.value ? 'bg-blue-50 text-blue-700' : 'text-slate-600 hover:bg-slate-50'"
            @click="selectTab(item.value)"
          >
            <component :is="item.icon" class="w-4 h-4" /><span>{{ item.label }}</span>
          </button>
        </div>
        <div class="h-px bg-slate-100 mx-3" />
        <div class="px-3 py-2 space-y-0.5">
          <button class="w-full flex items-center gap-3 px-3 py-3 rounded-xl text-sm font-semibold text-slate-600 hover:bg-slate-50" @click="showMore = false; emit('openProfile')">
            <UserRound class="w-4 h-4" /><span>Profil Saya</span>
          </button>
          <button class="w-full flex items-center gap-3 px-3 py-3 rounded-xl text-sm font-semibold text-red-600 hover:bg-red-50" @click="showMore = false; emit('logout')">
            <LogOut class="w-4 h-4" /><span>Keluar</span>
          </button>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.sheet-fade-enter-active, .sheet-fade-leave-active { transition: opacity 0.2s ease; }
.sheet-fade-enter-from, .sheet-fade-leave-to { opacity: 0; }
.sheet-slide-enter-active, .sheet-slide-leave-active { transition: transform 0.25s cubic-bezier(0.32, 0.72, 0, 1); }
.sheet-slide-enter-from, .sheet-slide-leave-to { transform: translateY(100%); }
</style>
