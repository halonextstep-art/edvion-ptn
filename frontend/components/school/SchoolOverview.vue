<script setup lang="ts">
// Overview Portal Sekolah — semua data nyata:
//  - 4 stat card: schoolService.overview() (sudah ada)
//  - Aktivitas Mingguan: analyticsService.schoolScoreTrend(7) — jumlah attempt/hari 7 hari
//    terakhir (bukan "siswa aktif" per hari karena kita belum menghitung distinct student,
//    tapi jumlah attempt adalah proxy aktivitas yang jujur & real, dilabel apa adanya).
//  - Performa per Mata Uji: analyticsService.schoolSubjectBreakdown() (top 4 mapel)
//  - Top 5 Siswa: analyticsService.schoolStudentRanking(), diambil 5 teratas
//  - Aktivitas Terkini: schoolService.recentActivity() — attempt selesai terbaru
import { Users, TrendingUp, Trophy, Target, Activity, Medal, BarChart2, Clock, Hand, Tag, ClipboardList, ArrowRight, Loader2 } from 'lucide-vue-next'
import type { SchoolOverview, ScoreTrendPoint, SubjectAccuracy, StudentRanking, RecentActivityItem, SchoolEntitlementItem } from '~/types'

const emit = defineEmits<{ navigate: [tab: string] }>()

const { schoolService, analyticsService, entitlementService } = useApi()
const toast = useToast()
const { user } = useAuth()

const overview = ref<SchoolOverview | null>(null)
const overviewLoading = ref(true)
const overviewError = ref<string | null>(null)

const trend = ref<ScoreTrendPoint[]>([])
const subjects = ref<SubjectAccuracy[]>([])
const ranking = ref<StudentRanking[]>([])
const activity = ref<RecentActivityItem[]>([])
const secondaryLoading = ref(true)

async function loadOverview() {
  overviewLoading.value = true
  overviewError.value = null
  try {
    overview.value = await schoolService.overview()
  } catch (e: any) {
    overviewError.value = e?.message || 'Gagal memuat data overview sekolah.'
    toast.error('Gagal memuat overview', overviewError.value)
  } finally {
    overviewLoading.value = false
  }
}

async function loadSecondary() {
  secondaryLoading.value = true
  try {
    const [t, s, r, a] = await Promise.all([
      analyticsService.schoolScoreTrend(7),
      analyticsService.schoolSubjectBreakdown(),
      analyticsService.schoolStudentRanking(),
      schoolService.recentActivity(8),
    ])
    trend.value = t
    subjects.value = s
    ranking.value = r
    activity.value = a
  } catch (e: any) {
    toast.error('Gagal memuat sebagian data overview', e?.message)
  } finally {
    secondaryLoading.value = false
  }
}

const activeEntitlements = ref<SchoolEntitlementItem[]>([])
const entitlementsLoading = ref(true)
async function loadEntitlements() {
  const schoolId = user.value?.school_id
  if (!schoolId) {
    entitlementsLoading.value = false
    return
  }
  entitlementsLoading.value = true
  try {
    const list = await entitlementService.listForSchool(schoolId)
    activeEntitlements.value = list.filter((e) => e.is_active)
  } catch (e: any) {
    toast.error('Gagal memuat status paket sekolah', e?.message)
  } finally {
    entitlementsLoading.value = false
  }
}

onMounted(() => {
  loadOverview()
  loadSecondary()
  loadEntitlements()
})

const topSubjects = computed(() => [...subjects.value].sort((a, b) => (b.total ? b.correct / b.total : 0) - (a.total ? a.correct / a.total : 0)).slice(0, 4))
function accuracyPct(s: SubjectAccuracy) {
  return s.total > 0 ? Math.round((s.correct / s.total) * 100) : 0
}
// Data chart untuk Aktivitas Mingguan & Performa per Mata Uji (BarLineChart)
const trendLabels = computed(() => trend.value.map((t) => fmtDay(t.day)))
const trendSeries = computed(() => [{ label: 'Attempt', data: trend.value.map((t) => t.attempts), color: '#4f46e5' }])
const subjectLabels = computed(() => topSubjects.value.map((s) => s.subject))
const subjectSeries = computed(() => [{ label: 'Akurasi', data: topSubjects.value.map((s) => accuracyPct(s)), color: '#6366f1' }])
const overallAccuracy = computed(() => {
  const totalCorrect = subjects.value.reduce((sum, s) => sum + s.correct, 0)
  const totalAll = subjects.value.reduce((sum, s) => sum + s.total, 0)
  return totalAll > 0 ? Math.round((totalCorrect / totalAll) * 100) : null
})
const top5 = computed(() => [...ranking.value].sort((a, b) => b.average_score - a.average_score).slice(0, 5))
const MEDAL = ['bg-amber-400 text-white', 'bg-slate-300 text-white', 'bg-amber-700 text-white']

function fmtDay(iso: string) {
  return new Date(iso).toLocaleDateString('id-ID', { weekday: 'short' })
}
function fmtTime(iso: string) {
  return new Date(iso).toLocaleString('id-ID', { day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit' })
}
const SESSION_LABEL: Record<string, string> = { tryout: 'menyelesaikan tryout', drilling: 'menyelesaikan drilling', mini: 'menyelesaikan mini tes' }

// Onboarding checklist — muncul selama roster sekolah masih kosong (sinyal nyata dari
// overview.total_students, bukan flag "sudah pernah lihat" di localStorage), sama seperti
// pola showFirstTimeGuide di StudentOverview.vue.
const showFirstTimeGuide = computed(() => !overviewLoading.value && overview.value?.total_students === 0)
</script>

<template>
  <div class="space-y-6">
    <Card v-if="overviewError" class="p-5 border-red-200 bg-red-50 flex items-center justify-between gap-3 flex-wrap">
      <div class="text-sm text-red-700">{{ overviewError }}</div>
      <Button variant="outline" size="sm" @click="loadOverview">Coba Lagi</Button>
    </Card>

    <Card class="p-6 bg-gradient-to-br from-blue-600 to-cyan-600 text-white border-0">
      <div class="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
        <div>
          <h2 class="text-2xl mb-2 flex items-center gap-2">Selamat Datang! <Hand class="w-5 h-5" /></h2>
          <p class="text-blue-100 text-lg">Dashboard monitoring performa siswa Anda</p>
        </div>
      </div>
    </Card>

    <Card v-if="showFirstTimeGuide" class="p-6 border-2 border-dashed border-blue-300 bg-blue-50/50">
      <div class="flex items-start gap-4">
        <div class="w-11 h-11 rounded-xl bg-blue-100 flex items-center justify-center shrink-0">
          <ClipboardList class="w-5 h-5 text-blue-600" />
        </div>
        <div class="flex-1 min-w-0">
          <h3 class="font-semibold text-slate-900 mb-1">Mulai dengan Menambahkan Siswa</h3>
          <p class="text-sm text-muted-foreground mb-3">Roster siswa Anda masih kosong. Tambahkan siswa satu per satu atau import lewat CSV untuk mulai memantau performa mereka.</p>
          <Button size="sm" class="gap-1.5" @click="emit('navigate', 'students')">
            Ke Manajemen Siswa <ArrowRight class="w-3.5 h-3.5" />
          </Button>
        </div>
      </div>
    </Card>

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <Card class="p-6 border-l-4 border-l-blue-500 hover:shadow-lg transition-shadow">
        <div class="flex items-start justify-between">
          <div>
            <p class="text-sm text-muted-foreground mb-1">Total Siswa</p>
            <h3 class="text-3xl mb-2 font-black">{{ overviewLoading ? '-' : overview?.total_students ?? 0 }}</h3>
            <div class="flex items-center gap-1 text-sm text-blue-600 font-medium"><Users class="w-3.5 h-3.5" />roster aktif</div>
          </div>
          <div class="w-12 h-12 rounded-xl bg-blue-100 flex items-center justify-center"><Users class="w-6 h-6 text-blue-600" /></div>
        </div>
      </Card>

      <Card class="p-6 border-l-4 border-l-green-500 hover:shadow-lg transition-shadow">
        <div class="flex items-start justify-between">
          <div>
            <p class="text-sm text-muted-foreground mb-1">Rata-rata Skor</p>
            <h3 class="text-3xl mb-2 font-black">{{ overviewLoading ? '-' : overview?.avg_score ?? '-' }}</h3>
            <div class="flex items-center gap-1 text-sm text-green-600 font-medium"><TrendingUp class="w-3.5 h-3.5" />semua attempt selesai</div>
          </div>
          <div class="w-12 h-12 rounded-xl bg-green-100 flex items-center justify-center"><TrendingUp class="w-6 h-6 text-green-600" /></div>
        </div>
      </Card>

      <Card class="p-6 border-l-4 border-l-purple-500 hover:shadow-lg transition-shadow">
        <div class="flex items-start justify-between">
          <div>
            <p class="text-sm text-muted-foreground mb-1">Tryout Selesai</p>
            <h3 class="text-3xl mb-2 font-black">{{ overviewLoading ? '-' : overview?.tryouts_completed ?? 0 }}</h3>
            <div class="flex items-center gap-1 text-sm text-purple-600 font-medium"><Trophy class="w-3.5 h-3.5" />total semua siswa</div>
          </div>
          <div class="w-12 h-12 rounded-xl bg-purple-100 flex items-center justify-center"><Trophy class="w-6 h-6 text-purple-600" /></div>
        </div>
      </Card>

      <Card class="p-6 border-l-4 border-l-orange-500 hover:shadow-lg transition-shadow">
        <div class="flex items-start justify-between">
          <div>
            <p class="text-sm text-muted-foreground mb-1">Target PTN</p>
            <h3 class="text-3xl mb-2 font-black">{{ overviewLoading ? '-' : overview?.target_ptn_count ?? 0 }}</h3>
            <div class="flex items-center gap-1 text-sm text-orange-600 font-medium"><Target class="w-3.5 h-3.5" />dari Rasionalisasi siswa</div>
          </div>
          <div class="w-12 h-12 rounded-xl bg-orange-100 flex items-center justify-center"><Target class="w-6 h-6 text-orange-600" /></div>
        </div>
      </Card>
    </div>

    <div class="grid lg:grid-cols-3 gap-6">
      <!-- Aktivitas Mingguan -->
      <Card class="p-5 lg:col-span-2">
        <h4 class="font-bold text-slate-900 mb-1 flex items-center gap-2"><Activity class="w-4 h-4 text-indigo-600" /> Aktivitas Siswa Mingguan</h4>
        <p class="text-xs text-muted-foreground mb-4">Jumlah attempt (tryout/drilling) yang diselesaikan siswa sekolah ini, 7 hari terakhir.</p>
        <div v-if="secondaryLoading" class="py-10 text-center text-sm text-muted-foreground"><Loader2 class="w-4 h-4 animate-spin mx-auto mb-1.5" />Memuat...</div>
        <div v-else-if="trend.length === 0" class="py-10 text-center text-sm text-muted-foreground">Belum ada aktivitas 7 hari terakhir.</div>
        <BarLineChart v-else :labels="trendLabels" :series="trendSeries" :height="180" />
      </Card>

      <!-- Performa per Mata Uji -->
      <Card class="p-5">
        <h4 class="font-bold text-slate-900 mb-3 flex items-center gap-2"><BarChart2 class="w-4 h-4 text-indigo-600" /> Performa per Mata Uji</h4>
        <div v-if="secondaryLoading" class="py-10 text-center text-sm text-muted-foreground"><Loader2 class="w-4 h-4 animate-spin mx-auto mb-1.5" />Memuat...</div>
        <div v-else-if="topSubjects.length === 0" class="py-10 text-center text-sm text-muted-foreground">Belum ada data jawaban.</div>
        <template v-else>
          <div class="mb-4">
            <BarLineChart :labels="subjectLabels" :series="subjectSeries" :horizontal="true" suffix="%" :height="160" :y-max="100" />
          </div>
          <div v-if="overallAccuracy != null" class="p-3 rounded-xl bg-gradient-to-br from-indigo-50 to-purple-50 border border-indigo-100 text-center">
            <p class="text-2xl font-black text-indigo-700">{{ overallAccuracy }}%</p>
            <p class="text-[10px] text-muted-foreground">akurasi rata-rata seluruh mapel</p>
          </div>
        </template>
      </Card>
    </div>

    <DeadlineCountdownWidget />

    <Card v-if="user?.school_id" class="p-5">
      <h4 class="font-bold text-slate-900 mb-3 flex items-center gap-2"><Tag class="w-4 h-4 text-indigo-600" /> Paket Aktif</h4>
      <div v-if="entitlementsLoading" class="py-6 text-center text-sm text-muted-foreground"><Loader2 class="w-4 h-4 animate-spin mx-auto mb-1.5" />Memuat...</div>
      <div v-else-if="activeEntitlements.length === 0" class="py-6 text-center text-sm text-muted-foreground">
        Belum ada paket premium yang aktif untuk sekolah ini. Hubungi admin platform untuk info lebih lanjut.
      </div>
      <div v-else class="space-y-2">
        <div v-for="item in activeEntitlements" :key="item.id" class="flex items-center justify-between p-3 rounded-lg border border-emerald-200 bg-emerald-50">
          <span class="text-sm font-semibold text-emerald-800">{{ item.package_name }}</span>
          <span class="text-xs text-emerald-700">berlaku sampai {{ item.expires_at ?? 'tanpa batas waktu' }}</span>
        </div>
      </div>
    </Card>

    <div class="grid lg:grid-cols-2 gap-6">
      <!-- Top 5 Siswa -->
      <Card class="p-5">
        <div class="flex items-center justify-between mb-3">
          <h4 class="font-bold text-slate-900 flex items-center gap-2"><Medal class="w-4 h-4 text-amber-500" /> Top 5 Siswa Terbaik</h4>
          <button class="text-xs font-semibold text-indigo-600 hover:underline" @click="emit('navigate', 'students')">Lihat Semua</button>
        </div>
        <div v-if="secondaryLoading" class="py-10 text-center text-sm text-muted-foreground"><Loader2 class="w-4 h-4 animate-spin mx-auto mb-1.5" />Memuat...</div>
        <div v-else-if="top5.length === 0" class="py-10 text-center text-sm text-muted-foreground">Belum ada siswa dengan attempt selesai.</div>
        <div v-else class="space-y-2">
          <div v-for="(s, i) in top5" :key="s.student_id" class="flex items-center gap-3 p-2.5 rounded-xl hover:bg-slate-50">
            <div class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold shrink-0" :class="i < 3 ? MEDAL[i] : 'bg-slate-100 text-slate-500'">{{ i + 1 }}</div>
            <div class="min-w-0 flex-1">
              <p class="text-sm font-semibold text-slate-900 truncate">{{ s.student_name }}</p>
              <p class="text-[10px] text-muted-foreground">{{ s.attempts_count }}x attempt &middot; skor terbaik {{ s.best_score }}</p>
            </div>
            <div class="text-right shrink-0">
              <p class="text-sm font-black text-slate-900">{{ Math.round(s.average_score) }}</p>
              <p class="text-[9px] text-muted-foreground">rata-rata</p>
            </div>
          </div>
        </div>
      </Card>

      <!-- Aktivitas Terkini -->
      <Card class="p-5">
        <h4 class="font-bold text-slate-900 mb-3 flex items-center gap-2"><Clock class="w-4 h-4 text-indigo-600" /> Aktivitas Terkini</h4>
        <div v-if="secondaryLoading" class="py-10 text-center text-sm text-muted-foreground"><Loader2 class="w-4 h-4 animate-spin mx-auto mb-1.5" />Memuat...</div>
        <div v-else-if="activity.length === 0" class="py-10 text-center text-sm text-muted-foreground">Belum ada aktivitas siswa.</div>
        <div v-else class="space-y-1">
          <div v-for="(a, i) in activity" :key="i" class="flex items-start gap-3 p-2 rounded-lg hover:bg-slate-50">
            <div class="w-8 h-8 rounded-lg bg-indigo-50 border border-indigo-100 flex items-center justify-center shrink-0 mt-0.5">
              <Trophy class="w-3.5 h-3.5 text-indigo-500" />
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-xs text-slate-700"><span class="font-semibold text-slate-900">{{ a.student_name }}</span> {{ SESSION_LABEL[a.session_type] || 'menyelesaikan sesi' }} <span class="font-medium">"{{ a.session_title }}"</span><span v-if="a.score != null"> — skor {{ a.score }}</span></p>
              <p class="text-[10px] text-muted-foreground">{{ fmtTime(a.submitted_at) }}</p>
            </div>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>
