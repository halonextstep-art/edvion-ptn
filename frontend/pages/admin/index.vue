<script setup lang="ts">
import {
  LogOut, LayoutDashboard, Calendar, DollarSign, TrendingUp,
  School, UserCog, Sparkles, Tag, Layers, ShoppingBag, Timer, CalendarClock,
} from 'lucide-vue-next'

// Tim Konten now has its own dedicated portal at /konten (see pages/konten/index.vue) —
// this page is admin-only.
definePageMeta({ middleware: 'auth', roles: ['admin'] })

const { user, logout } = useAuth()
const activeTab = ref('overview')
const showProfile = ref(false)

const initials = computed(() => (user.value?.name || 'A').charAt(0).toUpperCase())
</script>

<template>
  <div class="min-h-screen bg-slate-50">
    <div class="bg-white border-b sticky top-0 z-40">
      <div class="max-w-[1800px] mx-auto px-6 py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <div class="w-10 h-10 rounded-xl overflow-hidden shrink-0">
              <img src="/logo-produk.png" alt="EdvionPTN" class="w-full h-full object-cover" />
            </div>
            <div>
              <h1 class="text-xl font-semibold">EdvionPTN</h1>
              <p class="text-sm text-muted-foreground">Admin Dashboard</p>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <NotificationCenter @navigate="activeTab = $event" />
            <div class="w-px h-8 bg-slate-200" />
            <button class="flex items-center gap-3 hover:bg-slate-100 rounded-xl px-2 py-1 transition-colors" title="Profil Saya" @click="showProfile = true">
              <div class="w-9 h-9 rounded-full bg-gradient-to-br from-indigo-500 to-purple-500 flex items-center justify-center">
                <span class="text-white text-sm font-bold">{{ initials }}</span>
              </div>
              <div class="hidden sm:block text-left">
                <div class="text-sm font-medium">{{ user?.name }}</div>
                <div class="text-xs text-muted-foreground">Super Admin</div>
              </div>
            </button>
            <Button variant="ghost" size="icon" @click="logout"><LogOut class="w-5 h-5" /></Button>
          </div>
        </div>
      </div>
    </div>

    <div class="max-w-[1800px] mx-auto px-6 py-8">
      <Tabs v-model="activeTab" class="space-y-6">
        <!-- Urutan tab sengaja disusun mengikuti alur kerja admin sebenarnya, bukan urutan
             fitur dibangun: (1) setup dasar — user & mitra sekolah, (2) pipeline konten — satu
             tab "Konten" gabungan (Bank Soal + Paket & Subtes sebagai menu utama; Taksonomi/Set
             Soal/Sesi dikelompokkan sebagai sub-menu "Lanjutan" di dalamnya, lihat
             ContentHub.vue doc comment untuk alasannya) + Simulasi UTBK, (3) komersial —
             voucher -> persetujuan pembelian manual, (4) engagement & marketing — event,
             gamifikasi, (5) insight & pelaporan — analytics, keuangan (paling akhir, sifatnya
             ringkasan/oversight atas semua yang di atas). -->
        <TabsList class="inline-flex h-auto p-1 bg-white border shadow-sm flex-wrap">
          <TabsTrigger value="overview" class="gap-2">
            <LayoutDashboard class="w-4 h-4" /><span class="hidden sm:inline">Overview</span>
          </TabsTrigger>
          <TabsTrigger value="users" class="gap-2">
            <UserCog class="w-4 h-4" /><span class="hidden sm:inline">Manajemen User</span>
          </TabsTrigger>
          <TabsTrigger value="partners" class="gap-2">
            <School class="w-4 h-4" /><span class="hidden sm:inline">Mitra</span>
          </TabsTrigger>
          <TabsTrigger value="content" class="gap-2">
            <Layers class="w-4 h-4" /><span class="hidden sm:inline">Konten</span>
          </TabsTrigger>
          <TabsTrigger value="simulation" class="gap-2">
            <Timer class="w-4 h-4" /><span class="hidden sm:inline">Simulasi UTBK</span>
          </TabsTrigger>
          <TabsTrigger value="voucher" class="gap-2">
            <Tag class="w-4 h-4" /><span class="hidden sm:inline">Voucher</span>
          </TabsTrigger>
          <TabsTrigger value="purchase-requests" class="gap-2">
            <ShoppingBag class="w-4 h-4" /><span class="hidden sm:inline">Permintaan Pembelian</span>
          </TabsTrigger>
          <TabsTrigger value="events" class="gap-2">
            <Calendar class="w-4 h-4" /><span class="hidden sm:inline">Event</span>
          </TabsTrigger>
          <TabsTrigger value="deadlines" class="gap-2">
            <CalendarClock class="w-4 h-4" /><span class="hidden sm:inline">Kalender Deadline</span>
          </TabsTrigger>
          <TabsTrigger value="gamification" class="gap-2">
            <Sparkles class="w-4 h-4" /><span class="hidden sm:inline">Gamifikasi</span>
          </TabsTrigger>
          <TabsTrigger value="analytics" class="gap-2">
            <TrendingUp class="w-4 h-4" /><span class="hidden sm:inline">Analytics</span>
          </TabsTrigger>
          <TabsTrigger value="finance" class="gap-2">
            <DollarSign class="w-4 h-4" /><span class="hidden sm:inline">Keuangan</span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="overview"><AdminOverviewPanel @navigate="activeTab = $event" /></TabsContent>
        <TabsContent value="users"><UserManager /></TabsContent>
        <TabsContent value="partners"><SchoolManager /></TabsContent>
        <TabsContent value="content"><ContentHub /></TabsContent>
        <TabsContent value="simulation"><SimulationManager /></TabsContent>
        <TabsContent value="voucher"><VoucherManager /></TabsContent>
        <TabsContent value="purchase-requests"><PendingPurchaseManager /></TabsContent>
        <TabsContent value="events"><EventManager /></TabsContent>
        <TabsContent value="deadlines"><DeadlineManager /></TabsContent>
        <TabsContent value="gamification"><GamificationManager /></TabsContent>
        <TabsContent value="analytics"><AnalyticsPanel /></TabsContent>
        <TabsContent value="finance"><FinanceManager /></TabsContent>
      </Tabs>
    </div>

    <ProfileDialog v-model="showProfile" />
  </div>
</template>
