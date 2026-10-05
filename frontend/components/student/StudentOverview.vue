<script setup lang="ts">
// Overview — semua data nyata: attempts milik siswa (skor, streak dihitung client-side
// dari submitted_at asli), penguasaan per mata uji (analyticsService.mySubjectBreakdown,
// di-scope ke siswa ini di backend), event mendatang (eventService.upcoming, real dari
// modul Event), dan achievement (gamificationService.myBadges, progress real dihitung di
// backend dari attempts/attempt_answers — tidak ada satupun angka yang di-hardcode).
import { Play, Trophy, TrendingUp, Target, CheckCircle2, Flame, Calendar, Users, Award, Loader2, Clock, Hand, ShoppingBag, Sparkles, Receipt, Rocket, ArrowRight } from 'lucide-vue-next'
import type { Attempt, TargetWithChanceItem, SubjectAccuracy, EventItem, StudentBadgeStatusItem, PaymentTransactionItem, PackageItem } from '~/types'

const emit = defineEmits<{ navigate: [tab: string] }>()
const { user } = useAuth()
const router = useRouter()
const toast = useToast()
const { resolve: resolveUploadUrl } = useUploadUrl()
const {
  tryoutService,
  rationalizationService,
  analyticsService,
  eventService,
  gamificationService,
  paymentService,
  publicService,
  entitlementService,
} = useApi()

const attempts = ref<Attempt[]>([])
const targets = ref<TargetWithChanceItem[]>([])
const subjects = ref<SubjectAccuracy[]>([])
const events = ref<EventItem[]>([])
const badges = ref<StudentBadgeStatusItem[]>([])
const transactions = ref<PaymentTransactionItem[]>([])
const catalogPackages = ref<PackageItem[]>([])
const loading = ref(true)
// Fail closed (stays false) on error since this only gates whether we nag a student who
// already has access to buy again — same fail-closed convention as DrillingZone.vue's
// premium check.
const hasPremiumAccess = ref(false)

onMounted(async () => {
  const results = await Promise.allSettled([
    tryoutService.myAttempts(),
    rationalizationService.myTargets(),
    analyticsService.mySubjectBreakdown(),
    eventService.upcoming(3),
    gamificationService.myBadges(),
    paymentService.myTransactions(),
    publicService.packages(),
    entitlementService.myAccessStatus(),
  ])
  if (results[0].status === 'fulfilled') attempts.value = results[0].value
  if (results[1].status === 'fulfilled') targets.value = results[1].value
  if (results[2].status === 'fulfilled') subjects.value = results[2].value
  if (results[3].status === 'fulfilled') events.value = results[3].value
  if (results[4].status === 'fulfilled') badges.value = results[4].value
  if (results[5].status === 'fulfilled') transactions.value = results[5].value
  if (results[6].status === 'fulfilled') catalogPackages.value = results[6].value
  if (results[7].status === 'fulfilled') hasPremiumAccess.value = results[7].value.has_premium_access
  loading.value = false
})

// The only "Beli Paket" entry point that previously existed was on the pre-login landing
// page — a student already logged in (B2B or B2C) had no in-app way to buy a package.
// Shown only when there's something to sell and the student doesn't already have premium
// access, to avoid nagging students who are already covered (personal voucher or, for a
// B2B student, their school's entitlement).
const showPackageShowcase = computed(() => !hasPremiumAccess.value && catalogPackages.value.length > 0)
function buyPackage(pkg: PackageItem) {
  router.push(`/checkout?packageId=${pkg.id}`)
}

// Manual purchase requests still waiting on admin confirmation (see PaymentService —
// no real gateway configured yet, so every checkout lands in this state first).
const pendingPurchases = computed(() => transactions.value.filter((t) => t.status === 'pending'))
function packageNameFor(t: PaymentTransactionItem) {
  return catalogPackages.value.find((p) => p.id === t.package_id)?.name ?? 'Paket'
}
function fmtDateTime(iso: string) {
  return new Date(iso).toLocaleDateString('id-ID', { day: '2-digit', month: 'short', year: 'numeric' })
}

// Sebelumnya satu-satunya cara keluar dari transaksi pending adalah menunggu admin atau
// menunggu auto-expire di server — sekarang siswa bisa batalkan sendiri kalau berubah pikiran.
const cancelingTxId = ref<string | null>(null)
async function cancelPurchase(t: PaymentTransactionItem) {
  if (!window.confirm(`Batalkan pesanan "${packageNameFor(t)}"?`)) return
  cancelingTxId.value = t.id
  try {
    await paymentService.cancelTransaction(t.id)
    transactions.value = await paymentService.myTransactions()
    toast.success('Pesanan dibatalkan')
  } catch (e: any) {
    toast.error('Gagal membatalkan pesanan', e?.message)
  } finally {
    cancelingTxId.value = null
  }
}

// Riwayat Pembelian — the full transaction log (pending, paid, failed, cancelled, expired),
// not just the pending subset above. Previously `myTransactions()` was fetched but only its
// pending slice was ever shown, so a student had no way to see what they'd actually bought
// once it was approved. Note: `has_premium_access` here is a coarse derived flag ("unlocked
// ANYTHING at all") — access itself IS tracked per-package/per-content-item server-side (see
// AccessService + PackageContentItem), so a "Lunas" entry means the student unlocked whatever
// specific sessions/templates that particular package includes, not necessarily everything;
// this list is an honest payment/ownership log, not a live per-package quota meter.
const sortedTransactions = computed(() =>
  [...transactions.value].sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()),
)
const TX_STATUS_INFO: Record<PaymentTransactionItem['status'], { label: string; class: string }> = {
  pending: { label: 'Menunggu Konfirmasi', class: 'text-amber-700 bg-amber-50 border-amber-200' },
  paid: { label: 'Lunas', class: 'text-emerald-700 bg-emerald-50 border-emerald-200' },
  failed: { label: 'Gagal', class: 'text-red-700 bg-red-50 border-red-200' },
  cancelled: { label: 'Dibatalkan', class: 'text-slate-600 bg-slate-100 border-slate-200' },
  expired: { label: 'Kedaluwarsa', class: 'text-slate-600 bg-slate-100 border-slate-200' },
}

const submitted = computed(() =>
  attempts.value
    .filter((a) => a.status === 'submitted' && a.score != null && a.submitted_at)
    .sort((a, b) => new Date(a.submitted_at!).getTime() - new Date(b.submitted_at!).getTime()),
)
const lastScore = computed(() => submitted.value[submitted.value.length - 1]?.score ?? null)
const avgScore = computed(() => {
  if (!submitted.value.length) return null
  return Math.round(submitted.value.reduce((sum, a) => sum + (a.score ?? 0), 0) / submitted.value.length)
})
const totalCorrect = computed(() => submitted.value.reduce((sum, a) => sum + (a.correct_count ?? 0), 0))
const bestChance = computed(() => (targets.value.length ? Math.max(...targets.value.map((t) => t.chance.chance_percent)) : null))

// ─── Streak (real, "gaps and islands" over submitted_at, current run up to today) ────
const currentStreak = computed(() => {
  const days = [...new Set(submitted.value.map((a) => new Date(a.submitted_at!).toDateString()))]
    .map((d) => new Date(d).getTime())
    .sort((a, b) => b - a)
  if (!days.length) return 0
  const oneDay = 86400000
  const today = new Date(); today.setHours(0, 0, 0, 0)
  const mostRecentGap = Math.round((today.getTime() - days[0]) / oneDay)
  if (mostRecentGap > 1) return 0 // streak sudah putus (tidak aktif hari ini/kemarin)
  let streak = 1
  for (let i = 1; i < days.length; i++) {
    const gap = Math.round((days[i - 1] - days[i]) / oneDay)
    if (gap === 1) streak++
    else break
  }
  return streak
})

// ─── Weekly score trend (last 8 weeks) ───────────────────────────────────────────
const weeklyTrend = computed(() => {
  const buckets = new Map<string, { sum: number; count: number; weekStart: number }>()
  for (const a of submitted.value) {
    const d = new Date(a.submitted_at!)
    const monday = new Date(d); monday.setHours(0, 0, 0, 0); monday.setDate(d.getDate() - ((d.getDay() + 6) % 7))
    const key = monday.toISOString().slice(0, 10)
    const b = buckets.get(key) ?? { sum: 0, count: 0, weekStart: monday.getTime() }
    b.sum += a.score ?? 0; b.count += 1
    buckets.set(key, b)
  }
  return [...buckets.values()]
    .sort((a, b) => a.weekStart - b.weekStart)
    .slice(-8)
    .map((b) => ({ weekStart: b.weekStart, score: Math.round(b.sum / b.count) }))
})
const trendGrowth = computed(() => {
  if (weeklyTrend.value.length < 2) return null
  return weeklyTrend.value[weeklyTrend.value.length - 1].score - weeklyTrend.value[0].score
})
const weeklyChartLabels = computed(() => weeklyTrend.value.map((w) => new Date(w.weekStart).toLocaleDateString('id-ID', { day: '2-digit', month: 'short' })))
const weeklyChartSeries = computed(() => [{ label: 'Skor Rata-rata', data: weeklyTrend.value.map((w) => w.score), color: '#9333ea' }])

function accuracyPct(s: SubjectAccuracy) {
  return s.total > 0 ? Math.round((s.correct / s.total) * 100) : 0
}
function barColor(pct: number) {
  if (pct >= 75) return 'from-emerald-500 to-green-500'
  if (pct >= 50) return 'from-amber-500 to-orange-500'
  return 'from-red-500 to-rose-500'
}

const earnedBadges = computed(() => badges.value.filter((b) => b.earned))
const nextBadge = computed(() => {
  const locked = badges.value.filter((b) => !b.earned).sort((a, b) => b.progress_percent - a.progress_percent)
  return locked[0] ?? null
})

function fmtDate(iso: string) {
  return new Date(iso).toLocaleDateString('id-ID', { day: '2-digit', month: 'short', year: 'numeric' })
}
const EVENT_LABEL: Record<string, string> = { tryout: 'Tryout', drilling: 'Drilling', mini: 'Mini Tryout' }

// ─── First-time guidance — real signal (zero submitted tryout AND zero rasionalisasi
//     target), not a dismissible localStorage flag, so it keeps nudging until the student
//     actually does one of the two things rather than disappearing after one glance. Siswa
//     B2C (mandiri, tanpa sekolah) diarahkan spesifik ke Tryout + Rasionalisasi SNBT (lihat
//     RasionalisasiSNBT.vue & DrillingZone.vue — jalur SNBP disembunyikan/didahulukan untuk
//     mereka juga); siswa B2B dapat pesan lebih umum karena SNBP juga relevan buat mereka. ──
const isB2C = computed(() => !user.value?.school_id)
const showFirstTimeGuide = computed(() => !loading.value && submitted.value.length === 0 && targets.value.length === 0)
</script>

<template>
  <div class="space-y-6">
    <!-- Welcome hero -->
    <Card class="relative overflow-hidden p-5 sm:p-6 bg-gradient-to-br from-purple-600 via-pink-600 to-orange-600 text-white border-0">
      <div class="relative flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
        <div class="min-w-0">
          <div v-if="currentStreak > 0" class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-white/20 text-xs font-bold mb-2">
            <Flame class="w-3.5 h-3.5" /> {{ currentStreak }} hari streak
          </div>
          <h2 class="text-2xl sm:text-3xl mb-2 flex items-center gap-2">Hai, {{ user?.name || 'Pejuang PTN' }}! <Hand class="w-5 h-5 sm:w-6 sm:h-6 shrink-0" /></h2>
          <p class="text-purple-100 text-base sm:text-lg">Terus semangat! Yuk lanjutkan latihan hari ini.</p>
        </div>
        <Button size="lg" class="w-full md:w-auto bg-white text-purple-600 hover:bg-purple-50 gap-2 shrink-0" @click="emit('navigate', 'drilling')">
          <Play class="w-5 h-5" /> Mulai Drilling
        </Button>
      </div>
    </Card>

    <!-- Penuntun langkah pertama — hanya muncul selama siswa belum pernah kerjakan tryout
         ATAU belum punya target rasionalisasi (sinyal nyata, bukan flag "sudah pernah lihat"
         di localStorage), supaya siswa baru gak bingung mulai dari mana. -->
    <Card v-if="showFirstTimeGuide" class="p-5 border-2 border-indigo-200 bg-gradient-to-br from-indigo-50 to-purple-50">
      <div class="flex items-center gap-2 mb-4">
        <div class="w-9 h-9 rounded-xl bg-gradient-to-br from-indigo-600 to-purple-600 flex items-center justify-center shrink-0">
          <Rocket class="w-4 h-4 text-white" />
        </div>
        <div>
          <h3 class="font-bold text-slate-900 text-sm">Mulai di Sini</h3>
          <p class="text-xs text-muted-foreground">Dua langkah cepat buat mulai persiapan PTN kamu</p>
        </div>
      </div>
      <div class="grid sm:grid-cols-2 gap-3">
        <button
          type="button"
          class="text-left p-4 bg-white rounded-xl border hover:border-indigo-300 hover:shadow-sm transition-all flex items-start gap-3"
          @click="emit('navigate', 'drilling')"
        >
          <div class="w-8 h-8 rounded-lg bg-indigo-100 flex items-center justify-center shrink-0 text-indigo-700 font-black text-sm">1</div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-semibold text-slate-900">Kerjakan Tryout Pertamamu</p>
            <p class="text-xs text-muted-foreground mt-0.5">Skor tryout jadi dasar estimasi peluang PTN kamu.</p>
          </div>
          <ArrowRight class="w-4 h-4 text-slate-300 shrink-0 mt-1" />
        </button>
        <button
          type="button"
          class="text-left p-4 bg-white rounded-xl border hover:border-indigo-300 hover:shadow-sm transition-all flex items-start gap-3"
          @click="emit('navigate', 'rasionalisasi')"
        >
          <div class="w-8 h-8 rounded-lg bg-indigo-100 flex items-center justify-center shrink-0 text-indigo-700 font-black text-sm">2</div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-semibold text-slate-900">{{ isB2C ? 'Tambah Target Rasionalisasi SNBT' : 'Tambah Target PTN' }}</p>
            <p class="text-xs text-muted-foreground mt-0.5">
              {{ isB2C ? 'Cari program studi dan lihat estimasi peluangmu lewat jalur SNBT.' : 'Cari program studi dan lihat estimasi peluangmu lewat jalur SNBT atau SNBP.' }}
            </p>
          </div>
          <ArrowRight class="w-4 h-4 text-slate-300 shrink-0 mt-1" />
        </button>
      </div>
    </Card>

    <!-- Pesanan pembelian menunggu konfirmasi admin (manual purchase flow — belum ada
         gateway pembayaran online, lihat PaymentService) -->
    <Card v-for="t in pendingPurchases" :key="t.id" class="p-4 border-2 border-amber-200 bg-amber-50 flex flex-col sm:flex-row sm:items-center gap-3">
      <div class="flex items-center gap-3 min-w-0 flex-1">
        <div class="w-10 h-10 rounded-xl bg-amber-100 flex items-center justify-center shrink-0">
          <Clock class="w-5 h-5 text-amber-600" />
        </div>
        <div class="min-w-0 flex-1">
          <p class="text-sm font-semibold text-amber-900">Menunggu konfirmasi pembayaran — {{ packageNameFor(t) }}</p>
          <p class="text-xs text-amber-700">Diajukan {{ fmtDateTime(t.created_at) }}. Admin akan menghubungimu untuk konfirmasi.</p>
        </div>
      </div>
      <div class="flex items-center justify-between sm:justify-end gap-3 shrink-0 pl-[52px] sm:pl-0">
        <button
          class="text-xs text-amber-700 hover:text-red-600 underline shrink-0 disabled:opacity-50"
          :disabled="cancelingTxId === t.id"
          @click="cancelPurchase(t)"
        >{{ cancelingTxId === t.id ? 'Membatalkan...' : 'Batalkan' }}</button>
        <Badge class="bg-amber-100 text-amber-700 shrink-0">Pending</Badge>
      </div>
    </Card>

    <!-- In-app "Beli Paket" entry point — previously this only existed on the pre-login
         landing page, so a student already logged in (B2C especially, since they have no
         school entitlement route to premium) had no way to buy from inside the app. -->
    <Card v-if="showPackageShowcase" class="p-5 border-2 border-orange-200 bg-gradient-to-br from-orange-50 to-amber-50">
      <div class="flex items-center gap-2 mb-3">
        <div class="w-9 h-9 rounded-xl bg-gradient-to-br from-orange-500 to-amber-500 flex items-center justify-center shrink-0">
          <Sparkles class="w-4 h-4 text-white" />
        </div>
        <div>
          <h3 class="font-bold text-slate-900 text-sm">Upgrade ke Premium</h3>
          <p class="text-xs text-muted-foreground">Buka semua soal & simulasi premium dengan salah satu paket berikut</p>
        </div>
      </div>
      <div class="grid sm:grid-cols-2 gap-3">
        <div v-for="pkg in catalogPackages" :key="pkg.id" class="bg-white rounded-xl border overflow-hidden">
          <div v-if="pkg.banner_url" class="relative aspect-[3/1]">
            <img :src="resolveUploadUrl(pkg.banner_url) ?? undefined" :alt="pkg.name" class="w-full h-full object-cover" />
            <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-black/5 to-transparent" />
            <p class="absolute bottom-2 left-3 right-3 text-sm font-semibold text-white truncate drop-shadow-sm">{{ pkg.name }}</p>
          </div>
          <div class="flex items-center gap-3 p-3">
            <div v-if="!pkg.banner_url" class="h-10 w-10 shrink-0 rounded-lg bg-gradient-to-br flex items-center justify-center" :class="pkg.gradient">
              <IconDisplay :icon-type="pkg.icon_type" :icon-name="pkg.icon_name" :icon-url="pkg.icon_url" size="w-5 h-5 text-slate-700" />
            </div>
            <div class="flex-1 min-w-0">
              <p v-if="!pkg.banner_url" class="text-sm font-semibold text-slate-900 truncate">{{ pkg.name }}</p>
              <p class="text-xs font-bold text-orange-500">Rp {{ pkg.sale_price.toLocaleString('id-ID') }}</p>
            </div>
            <Button size="sm" variant="gradient" @click="buyPackage(pkg)"><ShoppingBag class="w-3.5 h-3.5" />Beli</Button>
          </div>
        </div>
      </div>
    </Card>

    <!-- Riwayat Pembelian — full transaction log, not just the pending nudge above. This is
         the only place a student can see what they've actually bought and its real status. -->
    <Card v-if="sortedTransactions.length > 0" class="p-5">
      <div class="flex items-center gap-2 mb-3">
        <div class="w-9 h-9 rounded-xl bg-slate-100 flex items-center justify-center shrink-0">
          <Receipt class="w-4 h-4 text-slate-600" />
        </div>
        <h3 class="font-bold text-slate-900 text-sm">Riwayat Pembelian</h3>
      </div>
      <div class="divide-y divide-slate-100">
        <div v-for="t in sortedTransactions" :key="t.id" class="flex items-center gap-3 py-3 first:pt-0 last:pb-0">
          <div class="flex-1 min-w-0">
            <p class="text-sm font-semibold text-slate-900 truncate">{{ packageNameFor(t) }}</p>
            <p class="text-xs text-muted-foreground">
              Rp {{ t.amount.toLocaleString('id-ID') }} &middot; Diajukan {{ fmtDateTime(t.created_at) }}
              <span v-if="t.status === 'paid' && t.paid_at"> &middot; Lunas {{ fmtDateTime(t.paid_at) }}</span>
            </p>
            <p v-if="t.status === 'paid' && t.provider_ref" class="text-[10px] text-muted-foreground font-mono mt-0.5">Kode voucher: {{ t.provider_ref }}</p>
            <p v-if="t.status === 'failed' && t.failure_reason" class="text-[10px] text-red-500 mt-0.5">{{ t.failure_reason }}</p>
          </div>
          <span class="text-[10px] px-2 py-0.5 rounded-full border font-semibold shrink-0" :class="TX_STATUS_INFO[t.status].class">
            {{ TX_STATUS_INFO[t.status].label }}
          </span>
        </div>
      </div>
    </Card>

    <!-- Quick stats -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4">
      <Card class="p-4 sm:p-5 border-2 border-purple-200 bg-gradient-to-br from-purple-50 to-white hover:shadow-lg transition-shadow">
        <div class="flex items-start justify-between mb-2 sm:mb-3">
          <div class="w-10 h-10 sm:w-12 sm:h-12 rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center">
            <Trophy class="w-5 h-5 sm:w-6 sm:h-6 text-white" />
          </div>
          <Badge class="bg-green-100 text-green-700 text-[10px] sm:text-xs">Live</Badge>
        </div>
        <div class="text-2xl sm:text-3xl mb-1 font-black text-slate-900">{{ lastScore ?? '-' }}</div>
        <div class="text-xs sm:text-sm text-muted-foreground">Skor Terakhir</div>
      </Card>

      <Card class="p-4 sm:p-5 border-2 border-blue-200 bg-gradient-to-br from-blue-50 to-white hover:shadow-lg transition-shadow">
        <div class="flex items-start justify-between mb-2 sm:mb-3">
          <div class="w-10 h-10 sm:w-12 sm:h-12 rounded-xl bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center">
            <TrendingUp class="w-5 h-5 sm:w-6 sm:h-6 text-white" />
          </div>
          <Badge class="bg-blue-100 text-blue-700 text-[10px] sm:text-xs">{{ submitted.length }}x</Badge>
        </div>
        <div class="text-2xl sm:text-3xl mb-1 font-black text-slate-900">{{ avgScore ?? '-' }}</div>
        <div class="text-xs sm:text-sm text-muted-foreground">Rata-rata Skor</div>
      </Card>

      <Card class="p-4 sm:p-5 border-2 border-green-200 bg-gradient-to-br from-green-50 to-white hover:shadow-lg transition-shadow cursor-pointer" @click="emit('navigate', 'rasionalisasi')">
        <div class="flex items-start justify-between mb-2 sm:mb-3">
          <div class="w-10 h-10 sm:w-12 sm:h-12 rounded-xl bg-gradient-to-br from-green-500 to-emerald-500 flex items-center justify-center">
            <Target class="w-5 h-5 sm:w-6 sm:h-6 text-white" />
          </div>
          <Badge v-if="bestChance != null" class="bg-green-100 text-green-700 text-[10px] sm:text-xs">{{ targets.length }} target</Badge>
          <Badge v-else class="bg-slate-100 text-slate-500 text-[10px] sm:text-xs">Kosong</Badge>
        </div>
        <div class="text-2xl sm:text-3xl mb-1 font-black text-slate-900">{{ bestChance != null ? `${bestChance}%` : '-' }}</div>
        <div class="text-xs sm:text-sm text-muted-foreground">Peluang PTN Tertinggi</div>
      </Card>

      <Card class="p-4 sm:p-5 border-2 border-orange-200 bg-gradient-to-br from-orange-50 to-white hover:shadow-lg transition-shadow">
        <div class="flex items-start justify-between mb-2 sm:mb-3">
          <div class="w-10 h-10 sm:w-12 sm:h-12 rounded-xl bg-gradient-to-br from-orange-500 to-red-500 flex items-center justify-center">
            <CheckCircle2 class="w-5 h-5 sm:w-6 sm:h-6 text-white" />
          </div>
          <Badge class="bg-orange-100 text-orange-700 text-[10px] sm:text-xs">Total</Badge>
        </div>
        <div class="text-2xl sm:text-3xl mb-1 font-black text-slate-900">{{ totalCorrect }}</div>
        <div class="text-xs sm:text-sm text-muted-foreground">Soal Terjawab Benar</div>
      </Card>
    </div>

    <div v-if="loading" class="py-10 text-center text-muted-foreground">
      <Loader2 class="w-6 h-6 animate-spin mx-auto mb-2" /> Memuat data...
    </div>

    <template v-else>
      <div class="grid lg:grid-cols-2 gap-6">
        <!-- Progres skor mingguan -->
        <Card class="p-5">
          <div class="flex items-center justify-between mb-4">
            <h4 class="font-bold text-slate-900">Progres Skor (8 Minggu Terakhir)</h4>
            <Badge v-if="trendGrowth != null" :class="trendGrowth >= 0 ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'">
              {{ trendGrowth >= 0 ? '+' : '' }}{{ trendGrowth }} poin
            </Badge>
          </div>
          <BarLineChart :labels="weeklyChartLabels" :series="weeklyChartSeries" :height="150" empty-label="Belum ada attempt selesai." />
        </Card>

        <!-- Penguasaan mata uji -->
        <Card class="p-5">
          <h4 class="font-bold text-slate-900 mb-4">Analisis Kemampuan</h4>
          <div v-if="subjects.length === 0" class="py-10 text-center text-sm text-muted-foreground">Belum ada data jawaban.</div>
          <div v-else class="space-y-3">
            <div v-for="s in subjects" :key="s.subject">
              <div class="flex items-center justify-between text-sm mb-1">
                <span class="font-medium text-slate-700">{{ s.subject }}</span>
                <span class="text-muted-foreground">{{ accuracyPct(s) }}%</span>
              </div>
              <div class="h-2.5 rounded-full bg-slate-100 overflow-hidden">
                <div class="h-full rounded-full bg-gradient-to-r" :class="barColor(accuracyPct(s))" :style="{ width: `${accuracyPct(s)}%` }" />
              </div>
            </div>
          </div>
        </Card>
      </div>

      <DeadlineCountdownWidget />

      <div class="grid lg:grid-cols-2 gap-6">
        <!-- Event mendatang -->
        <Card class="p-5">
          <div class="flex items-center justify-between mb-4">
            <h4 class="font-bold text-slate-900 flex items-center gap-2"><Calendar class="w-4 h-4 text-indigo-600" />Event Mendatang</h4>
          </div>
          <div v-if="events.length === 0" class="py-10 text-center text-sm text-muted-foreground">Belum ada event terjadwal.</div>
          <div v-else class="space-y-3">
            <div v-for="e in events" :key="e.id" class="p-3 rounded-lg border flex items-center justify-between gap-3">
              <div class="min-w-0">
                <div class="font-semibold text-sm text-slate-900 truncate">{{ e.name }}</div>
                <div class="text-xs text-muted-foreground flex items-center gap-2 flex-wrap mt-0.5">
                  <span>{{ fmtDate(e.start_date) }}</span>
                  <Badge variant="outline" class="text-[10px]">{{ EVENT_LABEL[e.event_type] }}</Badge>
                  <span class="inline-flex items-center gap-1"><Users class="w-3 h-3" />{{ e.participants }}</span>
                </div>
              </div>
              <Button size="sm" variant="outline" class="shrink-0" @click="emit('navigate', 'drilling')">Lihat</Button>
            </div>
          </div>
        </Card>

        <!-- Achievements -->
        <Card class="p-5">
          <div class="flex items-center justify-between mb-4">
            <h4 class="font-bold text-slate-900 flex items-center gap-2"><Award class="w-4 h-4 text-amber-600" />Achievements</h4>
            <Badge class="bg-slate-100 text-slate-600">{{ earnedBadges.length }}/{{ badges.length }}</Badge>
          </div>
          <div v-if="badges.length === 0" class="py-10 text-center text-sm text-muted-foreground">Belum ada badge tersedia.</div>
          <template v-else>
            <div class="grid grid-cols-2 gap-3 mb-4">
              <div
                v-for="b in badges.slice(0, 4)" :key="b.id"
                class="p-3 rounded-lg border text-center transition-opacity"
                :class="b.earned ? 'bg-amber-50 border-amber-200' : 'opacity-40 grayscale'"
              >
                <IconDisplay :icon-type="b.icon_type" :icon-name="b.icon_name" :icon-url="b.icon_url" size="w-6 h-6 mx-auto mb-1 text-amber-600" />
                <div class="text-xs font-semibold text-slate-900 truncate">{{ b.name }}</div>
              </div>
            </div>
            <div v-if="nextBadge" class="p-3 rounded-lg bg-slate-50 border">
              <div class="flex items-center justify-between text-xs mb-1.5">
                <span class="font-semibold text-slate-700 flex items-center gap-1.5">Selanjutnya: <IconDisplay :icon-type="nextBadge.icon_type" :icon-name="nextBadge.icon_name" :icon-url="nextBadge.icon_url" size="w-3.5 h-3.5" /> {{ nextBadge.name }}</span>
                <span class="text-muted-foreground">{{ nextBadge.progress_percent }}%</span>
              </div>
              <div class="h-1.5 rounded-full bg-slate-200 overflow-hidden">
                <div class="h-full rounded-full bg-gradient-to-r from-amber-500 to-orange-500" :style="{ width: `${nextBadge.progress_percent}%` }" />
              </div>
            </div>
          </template>
        </Card>
      </div>
    </template>
  </div>
</template>
