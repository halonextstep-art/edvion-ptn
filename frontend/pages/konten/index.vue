<script setup lang="ts">
// Portal Tim Konten — tema gelap penuh mengikuti referensi/ContentDashboard.tsx. Root diberi
// class="dark" (Tailwind darkMode:'class', lihat tailwind.config.ts) sehingga hanya halaman
// ini yang gelap — Admin/Sekolah/Siswa tidak terpengaruh sama sekali. Komponen yang dipakai
// bersama (QuestionBankManager, dst) punya varian dark: di setiap kelas warna pentingnya.
import { LogOut, BookOpen, BarChart3, ArrowRight, ListChecks, Package } from 'lucide-vue-next'
import type { QuestionStatus } from '~/types'

definePageMeta({ middleware: 'auth', roles: ['content'] })

const { user, logout } = useAuth()
const { questionService } = useApi()

type ActiveView = 'questions' | 'sets' | 'packages' | 'analytics'
const activeView = ref<ActiveView>('questions')
const showProfile = ref(false)

const pendingReview = ref(0)
async function loadBadge() {
  try {
    const res = await questionService.list({ status: 'review', mine: true, page: 1, page_size: 1 })
    pendingReview.value = res.total
  } catch {
    // non-critical for the sidebar badge
  }
}
onMounted(loadBadge)

const STATUS_FLOW: { key: QuestionStatus; label: string; dot: string }[] = [
  { key: 'draft', label: 'Draft', dot: 'bg-slate-400' },
  { key: 'review', label: 'Review', dot: 'bg-amber-400' },
  { key: 'approved', label: 'Disetujui', dot: 'bg-emerald-400' },
]

const initials = computed(() => (user.value?.name || 'K').charAt(0).toUpperCase())
</script>

<template>
  <div class="dark min-h-screen bg-[#09080f] flex">
    <!-- Sidebar -->
    <aside class="w-56 bg-[#0f0d1a] border-r border-white/10 flex flex-col shrink-0">
      <div class="p-4 border-b border-white/10">
        <div class="flex items-center justify-between gap-2.5">
          <div class="flex items-center gap-2.5">
            <div class="w-7 h-7 rounded-lg overflow-hidden shrink-0">
              <img src="/logo-produk.png" alt="EdvionPTN" class="w-full h-full object-cover" />
            </div>
            <div>
              <p class="text-sm font-bold text-white">Tim Konten</p>
              <p class="text-[10px] text-violet-400">EdvionPTN</p>
            </div>
          </div>
          <div class="bg-white rounded-xl shrink-0">
            <NotificationCenter />
          </div>
        </div>
      </div>

      <nav class="flex-1 p-2.5 space-y-0.5">
        <button
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-colors"
          :class="activeView === 'questions' ? 'bg-violet-600/30 text-violet-300 border border-violet-600/30' : 'text-white/50 hover:text-white hover:bg-white/5'"
          @click="activeView = 'questions'"
        >
          <BookOpen class="w-4 h-4" /><span class="flex-1 text-left">Bank Soal</span>
          <span v-if="pendingReview > 0" class="text-[10px] font-black px-1.5 py-0.5 rounded-full bg-amber-500 text-white">{{ pendingReview }}</span>
        </button>
        <button
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-colors"
          :class="activeView === 'sets' ? 'bg-violet-600/30 text-violet-300 border border-violet-600/30' : 'text-white/50 hover:text-white hover:bg-white/5'"
          @click="activeView = 'sets'"
        >
          <ListChecks class="w-4 h-4" /><span class="flex-1 text-left">Set Soal</span>
        </button>
        <button
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-colors"
          :class="activeView === 'packages' ? 'bg-violet-600/30 text-violet-300 border border-violet-600/30' : 'text-white/50 hover:text-white hover:bg-white/5'"
          @click="activeView = 'packages'"
        >
          <Package class="w-4 h-4" /><span class="flex-1 text-left">Paket</span>
        </button>
        <button
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-colors"
          :class="activeView === 'analytics' ? 'bg-violet-600/30 text-violet-300 border border-violet-600/30' : 'text-white/50 hover:text-white hover:bg-white/5'"
          @click="activeView = 'analytics'"
        >
          <BarChart3 class="w-4 h-4" /><span class="flex-1 text-left">Analitik</span>
        </button>
      </nav>

      <!-- Alur soal mini -->
      <div class="mx-2.5 mb-2.5 p-3 bg-white/[0.03] border border-white/10 rounded-xl">
        <p class="text-[9px] font-bold text-white/25 uppercase tracking-widest mb-2">Alur Soal</p>
        <div class="space-y-1">
          <div v-for="(s, i) in STATUS_FLOW" :key="s.key" class="flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full shrink-0" :class="s.dot" />
            <span class="text-[9px] text-white/35">{{ s.label }}</span>
            <ArrowRight v-if="i < STATUS_FLOW.length - 1" class="w-2 h-2 text-white/15 ml-auto" />
          </div>
        </div>
      </div>

      <div class="p-2.5 border-t border-white/10">
        <button class="w-full flex items-center gap-2 px-2 py-1.5 mb-0.5 rounded-xl hover:bg-white/5 transition-colors" title="Profil Saya" @click="showProfile = true">
          <div class="w-7 h-7 rounded-full bg-gradient-to-br from-violet-500 to-purple-600 flex items-center justify-center text-xs font-bold shrink-0 text-white">{{ initials }}</div>
          <div class="flex-1 min-w-0 text-left">
            <p class="text-xs font-semibold text-white truncate">{{ user?.name }}</p>
            <p class="text-[10px] text-white/35 truncate">{{ user?.email }}</p>
          </div>
        </button>
        <button class="w-full flex items-center gap-2 px-2 py-2 rounded-xl text-sm text-white/40 hover:text-red-400 hover:bg-red-900/20 transition-colors" @click="logout">
          <LogOut class="w-4 h-4" /> Keluar
        </button>
      </div>
    </aside>

    <!-- Main content -->
    <div class="flex-1 min-w-0 overflow-y-auto p-6 lg:p-8 bg-[#09080f]">
      <QuestionBankManager v-if="activeView === 'questions'" />
      <QuestionSetManager v-else-if="activeView === 'sets'" />
      <ContentPackagesPanel v-else-if="activeView === 'packages'" />
      <ContentAnalytics v-else />
    </div>

    <ProfileDialog v-model="showProfile" />
  </div>
</template>
