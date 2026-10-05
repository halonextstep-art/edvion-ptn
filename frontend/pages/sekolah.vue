<script setup lang="ts">
import {
  LogOut, LayoutDashboard, Users, TrendingUp, FileText, School, Target,
  ClipboardList, ListChecks,
} from 'lucide-vue-next'

definePageMeta({ middleware: 'auth', roles: ['school'] })

const { user, logout } = useAuth()
const activeTab = ref('overview')
const showProfile = ref(false)

const schoolName = computed(() => user?.value?.school_name || user?.value?.name || 'Sekolah')
const initials = computed(() => (user.value?.name || '?').trim().charAt(0).toUpperCase())
</script>

<template>
  <div class="min-h-screen bg-slate-50">
    <div class="bg-white border-b sticky top-0 z-40" style="padding-top: env(safe-area-inset-top)">
      <div class="max-w-[1800px] mx-auto px-4 sm:px-6 py-3 sm:py-4">
        <div class="flex items-center justify-between gap-2">
          <div class="flex items-center gap-3 sm:gap-4 min-w-0">
            <div class="w-9 h-9 sm:w-10 sm:h-10 rounded-xl bg-gradient-to-br from-blue-600 to-cyan-600 flex items-center justify-center shrink-0">
              <School class="w-5 h-5 sm:w-6 sm:h-6 text-white" />
            </div>
            <div class="min-w-0">
              <h1 class="text-base sm:text-xl font-semibold truncate">{{ schoolName }}</h1>
              <p class="hidden sm:block text-sm text-muted-foreground">Portal Sekolah Mitra</p>
            </div>
          </div>

          <div class="flex items-center gap-2 sm:gap-3 shrink-0">
            <NotificationBell @navigate="activeTab = $event" />
            <div class="hidden lg:block w-px h-8 bg-slate-200" />
            <button class="hidden lg:flex items-center gap-3 hover:bg-slate-100 rounded-xl px-2 py-1 transition-colors" title="Profil Saya" @click="showProfile = true">
              <div class="w-9 h-9 rounded-full bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center">
                <span class="text-white text-sm font-bold">{{ initials }}</span>
              </div>
              <div class="hidden sm:block text-left">
                <div class="text-sm font-medium">{{ user?.name }}</div>
                <div class="text-xs text-muted-foreground">Admin Sekolah</div>
              </div>
            </button>
            <Button variant="ghost" size="icon" class="hidden lg:inline-flex shrink-0" @click="logout"><LogOut class="w-5 h-5" /></Button>
          </div>
        </div>
      </div>
    </div>

    <div class="max-w-[1800px] mx-auto px-3 sm:px-6 py-4 sm:py-8 pb-24 lg:pb-8">
      <Tabs v-model="activeTab" class="space-y-6">
        <TabsList class="hidden lg:inline-flex h-auto p-1 bg-white border shadow-sm flex-wrap">
          <TabsTrigger value="overview" class="gap-2">
            <LayoutDashboard class="w-4 h-4" /><span class="hidden sm:inline">Overview</span>
          </TabsTrigger>
          <TabsTrigger value="students" class="gap-2">
            <Users class="w-4 h-4" /><span class="hidden sm:inline">Siswa</span>
          </TabsTrigger>
          <TabsTrigger value="snbp" class="gap-2">
            <Target class="w-4 h-4" /><span class="hidden sm:inline">Rasionalisasi SNBP</span>
          </TabsTrigger>
          <TabsTrigger value="snbt" class="gap-2">
            <ClipboardList class="w-4 h-4" /><span class="hidden sm:inline">Rekap SNBT</span>
          </TabsTrigger>
          <TabsTrigger value="tka" class="gap-2">
            <ListChecks class="w-4 h-4" /><span class="hidden sm:inline">Rekap TKA</span>
          </TabsTrigger>
          <TabsTrigger value="analytics" class="gap-2">
            <TrendingUp class="w-4 h-4" /><span class="hidden sm:inline">Analytics</span>
          </TabsTrigger>
          <TabsTrigger value="reports" class="gap-2">
            <FileText class="w-4 h-4" /><span class="hidden sm:inline">Laporan</span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="overview"><SchoolOverview @navigate="activeTab = $event" /></TabsContent>

        <TabsContent value="students"><StudentManager /></TabsContent>
        <TabsContent value="snbp"><SchoolRasionalisasi /></TabsContent>
        <TabsContent value="snbt"><SchoolRekapSnbt /></TabsContent>
        <TabsContent value="tka"><SchoolRekapTka /></TabsContent>
        <TabsContent value="analytics"><SchoolAnalytics /></TabsContent>
        <TabsContent value="reports"><SchoolReports /></TabsContent>
      </Tabs>
    </div>

    <SchoolMobileBottomNav v-model="activeTab" @open-profile="showProfile = true" @logout="logout" />

    <ProfileDialog v-model="showProfile" />
  </div>
</template>
