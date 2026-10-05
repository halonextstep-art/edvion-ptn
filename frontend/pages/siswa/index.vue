<script setup lang="ts">
import {
  LogOut, LayoutDashboard, Zap, Trophy, Play, Tag, X, Loader2,
  CheckCircle2, GraduationCap, BarChart3, Award,
} from 'lucide-vue-next'

definePageMeta({ middleware: 'auth', roles: ['student'] })

const { user, logout } = useAuth()
const { voucherService, tryoutService, entitlementService, simulationService } = useApi()
const activeTab = ref('overview')
const showVoucher = ref(false)
const showProfile = ref(false)
const initials = computed(() => (user.value?.name || '?').trim().charAt(0).toUpperCase())
const toast = useToast()
const router = useRouter()
const attemptSession = useAttemptSessionStore()

// "Mulai Sekarang" — dulu 4 judul dikarang (tidak ada di database) yang keempatnya melakukan
// hal yang sama persis (pindah ke tab Drilling) apa pun yang diklik, jadi terasa tidak
// berfungsi jelas. Sekarang: sesi ASLI milik siswa ini (belum dikerjakan, tidak terkunci,
// bukan mapel pilihan yang belum dipilih, bukan subtes di dalam Simulasi UTBK) — klik langsung
// memulai sesi itu, sama seperti tombol "Mulai" di DrillingZone.vue.
type QuickSession = { id: string; title: string; type: 'tryout' | 'drilling' | 'mini' }
const quickSessions = ref<QuickSession[]>([])
const startingQuickId = ref<string | null>(null)

async function loadQuickSessions() {
  try {
    const [sessions, attempts, access, templates] = await Promise.all([
      tryoutService.listSessions(),
      tryoutService.myAttempts(),
      entitlementService.myAccessStatus().catch(() => ({ unlocked_tryout_session_ids: [], unlocked_simulation_template_ids: [] })),
      simulationService.listTemplates().catch(() => []),
    ])
    const unlockedIds = new Set(access.unlocked_tryout_session_ids)
    const slotSessionIds = new Set(templates.flatMap((t) => t.slots.map((s) => s.session_id).filter((id): id is string => !!id)))
    const doneIds = new Set(attempts.filter((a) => a.status === 'submitted').map((a) => a.session_id))
    const eligible = sessions.filter((s) =>
      !s.is_elective && !slotSessionIds.has(s.id) && !doneIds.has(s.id) && (!s.is_premium || unlockedIds.has(s.id)),
    )
    // Prioritaskan variasi tipe (tryout dulu, baru mini/drilling) supaya bar tidak didominasi
    // satu jenis saja — tapi tetap murni dari data asli, bukan judul yang dikarang.
    const order: Record<string, number> = { tryout: 0, mini: 1, drilling: 2 }
    eligible.sort((a, b) => (order[a.session_type] ?? 3) - (order[b.session_type] ?? 3))
    quickSessions.value = eligible.slice(0, 4).map((s) => ({ id: s.id, title: s.title, type: s.session_type as QuickSession['type'] }))
  } catch {
    // Non-critical widget — dashboard tetap jalan tanpa bar ini kalau gagal memuat.
    quickSessions.value = []
  }
}
onMounted(loadQuickSessions)

async function quickLaunch(s: QuickSession) {
  startingQuickId.value = s.id
  try {
    const result = await tryoutService.startAttempt(s.id)
    attemptSession.set(result)
    await router.push(`/tryout/${result.attempt.id}`)
  } catch (e: any) {
    toast.error('Tidak bisa memulai sesi', e?.message)
  } finally {
    startingQuickId.value = null
  }
}

// ─── Voucher redeem (real) ───────────────────────────────────────────────────────
const voucherCode = ref('')
const redeeming = ref(false)
const redeemError = ref('')
const redeemResult = ref<Awaited<ReturnType<typeof voucherService.redeem>> | null>(null)

function openVoucher() {
  showVoucher.value = true
  voucherCode.value = ''
  redeemError.value = ''
  redeemResult.value = null
}
function closeVoucher() {
  showVoucher.value = false
}
async function submitVoucher() {
  if (!voucherCode.value.trim()) return
  redeeming.value = true
  redeemError.value = ''
  try {
    redeemResult.value = await voucherService.redeem(voucherCode.value.trim())
  } catch (e: any) {
    redeemError.value = e?.message ?? 'Gagal menukarkan voucher'
  } finally {
    redeeming.value = false
  }
}
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-purple-50">
    <!-- `padding-top: env(safe-area-inset-top)` — kalau dibuka sebagai PWA terinstall
         (display: standalone), header ini menghindari notch/status-bar alih-alih tertimpa. -->
    <div class="bg-white/80 backdrop-blur-lg border-b sticky top-0 z-40 shadow-sm" style="padding-top: env(safe-area-inset-top)">
      <div class="max-w-[1400px] mx-auto px-3 sm:px-6 py-3 sm:py-3.5 flex items-center justify-between gap-2">
        <div class="flex items-center gap-2 sm:gap-3 min-w-0">
          <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-xl overflow-hidden shrink-0">
            <img src="/logo-produk.png" alt="EdvionPTN" class="w-full h-full object-cover" />
          </div>
          <div class="min-w-0">
            <h1 class="text-sm sm:text-base font-bold leading-tight truncate">EdvionPTN</h1>
            <p class="text-[11px] sm:text-xs text-muted-foreground leading-tight">Portal Siswa</p>
          </div>
        </div>

        <div class="flex items-center gap-1.5 sm:gap-3 shrink-0">
          <!-- Disembunyikan di mobile (hidden lg:flex) — sekarang terjangkau lewat sheet
               "Lainnya" di MobileBottomNav, jadi header mobile lebih lega/app-like, bukan
               penuh ikon. Tetap tampil apa adanya di desktop yang tidak punya bottom nav. -->
          <button
            class="hidden lg:flex items-center gap-1.5 px-3 py-1.5 rounded-xl border-2 border-dashed border-indigo-300 bg-indigo-50 hover:bg-indigo-100 text-indigo-600 text-xs font-bold transition-colors shrink-0"
            title="Tukarkan Voucher"
            @click="openVoucher"
          >
            <Tag class="w-3.5 h-3.5" /> <span>Voucher</span>
          </button>

          <NotificationCenter @navigate="activeTab = $event" />

          <button class="flex items-center gap-2 hover:bg-slate-100 rounded-xl px-1.5 sm:px-2 py-1 transition-colors shrink-0" title="Profil Saya" @click="showProfile = true">
            <div class="hidden md:block text-right">
              <div class="text-sm font-semibold leading-tight">{{ user?.name }}</div>
              <div class="text-xs text-muted-foreground leading-tight">{{ user?.school_name || 'Sekolah' }}</div>
            </div>
            <div class="w-8 h-8 rounded-full bg-gradient-to-br from-purple-600 to-pink-600 flex items-center justify-center text-white text-xs font-bold shrink-0">{{ initials }}</div>
          </button>
          <!-- Disembunyikan di mobile — ada di sheet "Lainnya" (MobileBottomNav) supaya tidak
               dobel dengan tombol Keluar di sana. -->
          <Button variant="ghost" size="icon" class="hidden lg:inline-flex shrink-0" @click="logout"><LogOut class="w-4 h-4" /></Button>
        </div>
      </div>
    </div>

    <!-- Quick launch bar — sesi asli milik siswa, klik langsung mulai (lihat loadQuickSessions) -->
    <div v-if="quickSessions.length > 0" class="bg-white border-b">
      <div class="max-w-[1400px] mx-auto px-3 sm:px-6 py-2 flex items-center gap-2 overflow-x-auto">
        <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider shrink-0">Mulai Sekarang:</span>
        <button
          v-for="s in quickSessions" :key="s.id"
          :disabled="startingQuickId === s.id"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-semibold border transition-colors whitespace-nowrap shrink-0 max-w-[220px] disabled:opacity-60"
          :class="s.type === 'tryout' ? 'bg-purple-50 border-purple-200 text-purple-700 hover:bg-purple-100'
            : s.type === 'drilling' ? 'bg-orange-50 border-orange-200 text-orange-700 hover:bg-orange-100'
            : 'bg-blue-50 border-blue-200 text-blue-700 hover:bg-blue-100'"
          @click="quickLaunch(s)"
        >
          <Loader2 v-if="startingQuickId === s.id" class="w-3 h-3 animate-spin shrink-0" />
          <Play v-else class="w-3 h-3 shrink-0" />
          <span class="truncate">{{ s.title }}</span>
        </button>
      </div>
    </div>

    <div class="max-w-[1400px] mx-auto px-3 sm:px-6 py-4 sm:py-6 pb-24 lg:pb-6">
      <Tabs v-model="activeTab" class="space-y-5 sm:space-y-6">
        <!-- Navigasi tab atas ini sekarang KHUSUS desktop (hidden di bawah lg) — di mobile
             navigasi utama dipindah ke MobileBottomNav.vue (bottom bar ala aplikasi native)
             supaya portal siswa tidak terasa seperti scroll tab website di HP. Keduanya
             mengikat ke `activeTab` yang sama, jadi selalu sinkron. -->
        <TabsList class="hidden lg:inline-flex lg:max-w-full lg:overflow-x-auto lg:flex-nowrap lg:justify-start lg:px-1 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
          <TabsTrigger value="overview" class="shrink-0 whitespace-nowrap"><LayoutDashboard class="w-4 h-4" />Dashboard</TabsTrigger>
          <TabsTrigger value="drilling" class="shrink-0 whitespace-nowrap"><Zap class="w-4 h-4" />Drilling</TabsTrigger>
          <TabsTrigger value="progress" class="shrink-0 whitespace-nowrap"><BarChart3 class="w-4 h-4" />Progress</TabsTrigger>
          <TabsTrigger value="rasionalisasi" class="shrink-0 whitespace-nowrap"><GraduationCap class="w-4 h-4" />Rasionalisasi</TabsTrigger>
          <TabsTrigger value="sertifikat" class="shrink-0 whitespace-nowrap"><Award class="w-4 h-4" />Sertifikat</TabsTrigger>
          <TabsTrigger value="leaderboard" class="shrink-0 whitespace-nowrap"><Trophy class="w-4 h-4" />Leaderboard</TabsTrigger>
        </TabsList>

        <TabsContent value="overview">
          <StudentOverview @navigate="activeTab = $event" />
        </TabsContent>

        <TabsContent value="drilling">
          <DrillingZone @navigate="activeTab = $event" />
        </TabsContent>

        <TabsContent value="progress">
          <StudentProgress />
        </TabsContent>

        <TabsContent value="rasionalisasi">
          <RasionalisasiSNBT @navigate="activeTab = $event" />
        </TabsContent>

        <TabsContent value="sertifikat">
          <StudentCertificates />
        </TabsContent>

        <TabsContent value="leaderboard">
          <StudentLeaderboard @navigate="activeTab = $event" />
        </TabsContent>
      </Tabs>
    </div>

    <!-- Voucher dialog -->
    <div v-if="showVoucher" class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4" @click.self="closeVoucher">
      <Card class="w-full max-w-sm p-6 relative text-center">
        <button class="absolute top-4 right-4 text-muted-foreground hover:text-foreground" @click="closeVoucher"><X class="w-4 h-4" /></button>
        <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center mx-auto mb-4">
          <Tag class="w-6 h-6 text-white" />
        </div>

        <template v-if="!redeemResult">
          <h3 class="font-bold text-slate-900 mb-1.5">Aktivasi Voucher</h3>
          <p class="text-sm text-muted-foreground mb-4">Masukkan kode voucher untuk mendapatkan akses paket.</p>
          <Input v-model="voucherCode" placeholder="Contoh: GASPL-XXXX-XXXX" class="mb-2 text-center uppercase" @keyup.enter="submitVoucher" />
          <p v-if="redeemError" class="text-xs text-red-600 mb-2">{{ redeemError }}</p>
          <Button class="w-full" :disabled="redeeming" @click="submitVoucher">
            <Loader2 v-if="redeeming" class="w-4 h-4 animate-spin" /> Tukarkan Voucher
          </Button>
        </template>
        <template v-else>
          <div class="w-12 h-12 rounded-full bg-emerald-100 flex items-center justify-center mx-auto mb-3">
            <CheckCircle2 class="w-6 h-6 text-emerald-600" />
          </div>
          <h3 class="font-bold text-slate-900 mb-1.5">Voucher Berhasil Ditukar!</h3>
          <p class="text-sm text-muted-foreground mb-1">Kamu mendapat akses ke:</p>
          <p class="font-semibold text-slate-900 mb-4">{{ redeemResult.package_name }}</p>
          <Button class="w-full" variant="outline" @click="closeVoucher">Tutup</Button>
        </template>
      </Card>
    </div>

    <ProfileDialog v-model="showProfile" />

    <MobileBottomNav v-model="activeTab" @open-voucher="openVoucher" @open-profile="showProfile = true" @logout="logout" />
  </div>
</template>
