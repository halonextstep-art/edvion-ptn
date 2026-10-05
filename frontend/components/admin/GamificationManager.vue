<script setup lang="ts">
import {
  Zap, Award, Trophy, Sparkles, Plus, Pencil, Trash2, X, Save, ToggleLeft, ToggleRight,
  Users, CheckCircle2, Crown, Play, Pause, Clock, Calendar, ChevronRight, Info,
  Medal, Flame, BookOpen,
} from 'lucide-vue-next'
import type {
  PointRuleItem, BadgeItem, BadgePayload, Rarity, BadgeConditionType,
  ChallengeItem, ChallengePayload, ChallengeType, ChallengeStatus, LeaderboardEntryItem,
} from '~/types'

const { gamificationService } = useApi()
const toast = useToast()

type ViewId = 'poin' | 'badge' | 'leaderboard' | 'challenge'
const view = ref<ViewId>('poin')
const VIEWS: { id: ViewId; label: string; icon: any; desc: string }[] = [
  { id: 'poin', label: 'Konfigurasi Poin', icon: Zap, desc: 'Aturan perolehan poin per aksi' },
  { id: 'badge', label: 'Badge & Achievement', icon: Award, desc: 'Buat & kelola badge siswa' },
  { id: 'leaderboard', label: 'Leaderboard', icon: Trophy, desc: 'Peringkat nasional (data asli)' },
  { id: 'challenge', label: 'Challenge & Event', icon: Sparkles, desc: 'Kompetisi berbatas waktu' },
]

const loading = ref(false)

// ─── Point rules ────────────────────────────────────────────────────────────────
const pointRules = ref<PointRuleItem[]>([])
const showRuleForm = ref(false)
const editRule = ref<PointRuleItem | null>(null)
const ruleForm = ref({ action: '', category: 'Umum', base_points: 10, multiplier: 1, enabled: true, icon: '', sort_order: 0 })

async function loadRules() {
  pointRules.value = await gamificationService.listPointRules()
}
function openCreateRule() {
  editRule.value = null
  ruleForm.value = { action: '', category: 'Umum', base_points: 10, multiplier: 1, enabled: true, icon: '', sort_order: pointRules.value.length }
  showRuleForm.value = true
}
function openEditRule(r: PointRuleItem) {
  editRule.value = r
  ruleForm.value = { action: r.action, category: r.category, base_points: r.base_points, multiplier: r.multiplier, enabled: r.enabled, icon: r.icon, sort_order: r.sort_order }
  showRuleForm.value = true
}
async function saveRule() {
  if (!ruleForm.value.action.trim()) { toast.error('Nama aksi wajib diisi'); return }
  try {
    if (editRule.value) {
      await gamificationService.updatePointRule(editRule.value.id, { base_points: ruleForm.value.base_points, multiplier: ruleForm.value.multiplier, enabled: ruleForm.value.enabled })
      toast.success('Aturan poin diperbarui')
    } else {
      await gamificationService.createPointRule(ruleForm.value)
      toast.success(`Aturan "${ruleForm.value.action}" dibuat`)
    }
    showRuleForm.value = false
    loadRules()
  } catch (e: any) {
    toast.error('Gagal menyimpan aturan poin', e?.message)
  }
}
async function toggleRule(r: PointRuleItem) {
  try {
    await gamificationService.updatePointRule(r.id, { base_points: r.base_points, multiplier: r.multiplier, enabled: !r.enabled })
    loadRules()
  } catch (e: any) {
    toast.error('Gagal mengubah status aturan', e?.message)
  }
}
async function deleteRule(r: PointRuleItem) {
  try {
    await gamificationService.removePointRule(r.id)
    toast.success(`Aturan "${r.action}" dihapus`)
    loadRules()
  } catch (e: any) {
    toast.error('Gagal menghapus aturan', e?.message)
  }
}
const ruleCategories = computed(() => [...new Set(pointRules.value.map((r) => r.category))])
const rulesByCategory = (cat: string) => pointRules.value.filter((r) => r.category === cat)

// ─── Badges ─────────────────────────────────────────────────────────────────────
const badges = ref<BadgeItem[]>([])
const showBadgeForm = ref(false)
const editBadge = ref<BadgeItem | null>(null)
const showBadgeDelete = ref(false)
const deleteBadgeTarget = ref<BadgeItem | null>(null)
const RARITY_LABEL: Record<Rarity, string> = { common: 'Common', rare: 'Rare', epic: 'Epic', legendary: 'Legendary' }
const CONDITION_LABEL: Record<BadgeConditionType, string> = {
  score_gte: 'Skor terbaik ≥', streak_gte: 'Streak hari beruntun ≥', questions_gte: 'Total soal dijawab ≥',
  tryout_gte: 'Tryout/drilling selesai ≥', rank_lte: 'Peringkat leaderboard ≤',
}
function blankBadgeForm(): BadgePayload {
  return { icon_type: 'preset', icon_name: 'trophy', icon_url: null, name: '', description: '', rarity: 'common', condition_type: 'score_gte', condition_value: 0, active: true }
}
const badgeForm = ref<BadgePayload>(blankBadgeForm())

// IconPicker v-model targets just the 3 icon fields — a computed get/set so picking an
// icon mutates those fields in place on badgeForm instead of replacing the whole payload
// object (which would drop name/description/rarity/etc).
const badgeIcon = computed<Pick<BadgePayload, 'icon_type' | 'icon_name' | 'icon_url'>>({
  get: () => ({ icon_type: badgeForm.value.icon_type, icon_name: badgeForm.value.icon_name, icon_url: badgeForm.value.icon_url }),
  set: (v) => {
    badgeForm.value.icon_type = v.icon_type
    badgeForm.value.icon_name = v.icon_name
    badgeForm.value.icon_url = v.icon_url
  },
})

async function loadBadges() {
  badges.value = await gamificationService.listBadges()
}
function openCreateBadge() {
  editBadge.value = null
  badgeForm.value = blankBadgeForm()
  showBadgeForm.value = true
}
function openEditBadge(b: BadgeItem) {
  editBadge.value = b
  badgeForm.value = {
    icon_type: b.icon_type, icon_name: b.icon_name, icon_url: b.icon_url,
    name: b.name, description: b.description, rarity: b.rarity, condition_type: b.condition_type, condition_value: b.condition_value, active: b.active,
  }
  showBadgeForm.value = true
}
async function saveBadge() {
  if (!badgeForm.value.name.trim()) { toast.error('Nama badge wajib diisi'); return }
  try {
    if (editBadge.value) {
      await gamificationService.updateBadge(editBadge.value.id, badgeForm.value)
      toast.success(`Badge "${badgeForm.value.name}" diperbarui`)
    } else {
      await gamificationService.createBadge(badgeForm.value)
      toast.success(`Badge "${badgeForm.value.name}" dibuat`)
    }
    showBadgeForm.value = false
    loadBadges()
  } catch (e: any) {
    toast.error('Gagal menyimpan badge', e?.message)
  }
}
async function toggleBadge(b: BadgeItem) {
  try {
    await gamificationService.setBadgeActive(b.id, !b.active)
    loadBadges()
  } catch (e: any) {
    toast.error('Gagal mengubah status badge', e?.message)
  }
}
function openDeleteBadge(b: BadgeItem) {
  deleteBadgeTarget.value = b
  showBadgeDelete.value = true
}
async function confirmDeleteBadge() {
  if (!deleteBadgeTarget.value) return
  try {
    await gamificationService.removeBadge(deleteBadgeTarget.value.id)
    toast.success(`Badge "${deleteBadgeTarget.value.name}" dihapus`)
    loadBadges()
  } catch (e: any) {
    toast.error('Gagal menghapus badge', e?.message)
  }
}
const badgeStats = computed(() => ({
  total: badges.value.length,
  active: badges.value.filter((b) => b.active).length,
  totalEarned: badges.value.reduce((s, b) => s + b.earned_by, 0),
  legendary: badges.value.filter((b) => b.rarity === 'legendary').length,
}))

// ─── Leaderboard ────────────────────────────────────────────────────────────────
const leaderboard = ref<LeaderboardEntryItem[]>([])
async function loadLeaderboard() {
  leaderboard.value = await gamificationService.leaderboard({ limit: 20 })
}

// ─── Challenges ─────────────────────────────────────────────────────────────────
const challenges = ref<ChallengeItem[]>([])
const filterChallengeStatus = ref<ChallengeStatus | 'all'>('all')
const showChallengeForm = ref(false)
const editChallenge = ref<ChallengeItem | null>(null)
const CHALLENGE_TYPE_LABEL: Record<ChallengeType, { label: string; icon: any }> = {
  most_solved: { label: 'Terbanyak Soal', icon: BookOpen },
  highest_score: { label: 'Skor Tertinggi', icon: Trophy },
  longest_streak: { label: 'Streak Terpanjang', icon: Flame },
  speed: { label: 'Kecepatan', icon: Zap },
}
const CHALLENGE_STATUS_LABEL: Record<ChallengeStatus, { label: string; variant: any }> = {
  upcoming: { label: 'Akan Datang', variant: 'default' },
  active: { label: 'Aktif', variant: 'success' },
  ended: { label: 'Berakhir', variant: 'secondary' },
}
function blankChallengeForm(): ChallengePayload {
  const today = new Date().toISOString().split('T')[0]
  return { name: '', challenge_type: 'most_solved', start_date: today, end_date: today, target_value: 10, reward_points: 100, reward_badge: 'reward', description: '' }
}
const challengeForm = ref<ChallengePayload>(blankChallengeForm())

async function loadChallenges() {
  challenges.value = await gamificationService.listChallenges()
}
function openCreateChallenge() {
  editChallenge.value = null
  challengeForm.value = blankChallengeForm()
  showChallengeForm.value = true
}
function openEditChallenge(c: ChallengeItem) {
  editChallenge.value = c
  challengeForm.value = { name: c.name, challenge_type: c.challenge_type, start_date: c.start_date, end_date: c.end_date, target_value: c.target_value, reward_points: c.reward_points, reward_badge: c.reward_badge, description: c.description }
  showChallengeForm.value = true
}
async function saveChallenge() {
  if (!challengeForm.value.name.trim()) { toast.error('Nama challenge wajib diisi'); return }
  try {
    if (editChallenge.value) {
      await gamificationService.updateChallenge(editChallenge.value.id, challengeForm.value)
      toast.success(`Challenge "${challengeForm.value.name}" diperbarui`)
    } else {
      await gamificationService.createChallenge(challengeForm.value)
      toast.success(`Challenge "${challengeForm.value.name}" dibuat`)
    }
    showChallengeForm.value = false
    loadChallenges()
  } catch (e: any) {
    toast.error('Gagal menyimpan challenge', e?.message)
  }
}
async function setChallengeStatus(c: ChallengeItem, status: ChallengeStatus) {
  try {
    await gamificationService.setChallengeStatus(c.id, status)
    loadChallenges()
  } catch (e: any) {
    toast.error('Gagal mengubah status challenge', e?.message)
  }
}
async function deleteChallenge(c: ChallengeItem) {
  try {
    await gamificationService.removeChallenge(c.id)
    toast.success(`Challenge "${c.name}" dihapus`)
    loadChallenges()
  } catch (e: any) {
    toast.error('Gagal menghapus challenge', e?.message)
  }
}
const filteredChallenges = computed(() => filterChallengeStatus.value === 'all' ? challenges.value : challenges.value.filter((c) => c.status === filterChallengeStatus.value))
const challengeStats = computed(() => ({
  active: challenges.value.filter((c) => c.status === 'active').length,
  upcoming: challenges.value.filter((c) => c.status === 'upcoming').length,
  totalParticipants: challenges.value.reduce((s, c) => s + c.participants, 0),
  totalCompletions: challenges.value.reduce((s, c) => s + c.completions, 0),
}))

async function loadAll() {
  loading.value = true
  try {
    await Promise.all([loadRules(), loadBadges(), loadLeaderboard(), loadChallenges()])
  } catch (e: any) {
    toast.error('Gagal memuat data gamifikasi', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(loadAll)
</script>

<template>
  <div class="space-y-6">
    <div class="bg-gradient-to-r from-indigo-600 via-purple-600 to-pink-600 rounded-2xl p-6 text-white">
      <div class="flex items-center gap-2 mb-1"><Sparkles class="w-5 h-5 text-yellow-300" /><span class="text-sm font-semibold text-indigo-200">Gamification Engine</span></div>
      <h1 class="text-2xl font-black mb-1">Manajemen Gamifikasi</h1>
      <p class="text-indigo-200 text-sm">Konfigurasi poin, badge, leaderboard, dan challenge — seluruhnya dihitung dari data asli hasil tryout &amp; drilling siswa</p>
    </div>

    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
      <button v-for="v in VIEWS" :key="v.id" class="p-4 rounded-xl border-2 text-left transition-all hover:shadow-sm" :class="view === v.id ? 'border-indigo-500 bg-indigo-50' : 'border-slate-200 bg-white hover:border-slate-300'" @click="view = v.id">
        <div class="w-9 h-9 rounded-lg flex items-center justify-center mb-2.5" :class="view === v.id ? 'bg-indigo-600' : 'bg-slate-100'">
          <component :is="v.icon" class="w-4 h-4" :class="view === v.id ? 'text-white' : 'text-slate-500'" />
        </div>
        <p class="text-sm font-bold mb-0.5" :class="view === v.id ? 'text-indigo-700' : 'text-slate-900'">{{ v.label }}</p>
        <p class="text-xs text-muted-foreground">{{ v.desc }}</p>
      </button>
    </div>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>

    <template v-else-if="view === 'poin'">
      <div class="flex items-center justify-between flex-wrap gap-3">
        <div><h2 class="text-lg font-bold">Konfigurasi Poin</h2><p class="text-sm text-muted-foreground">Atur berapa poin yang diperoleh siswa untuk setiap aksi</p></div>
        <Button variant="gradient" @click="openCreateRule"><Plus class="w-4 h-4" />Tambah Aturan</Button>
      </div>
      <Card v-if="pointRules.length === 0" class="p-10 text-center"><Zap class="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" /><p class="text-sm text-muted-foreground">Belum ada aturan poin.</p></Card>
      <div v-for="cat in ruleCategories" :key="cat" class="space-y-3">
        <span class="inline-flex text-xs font-bold px-3 py-1 rounded-full border bg-indigo-50 text-indigo-700 border-indigo-200">{{ cat }}</span>
        <Card class="overflow-hidden">
          <table class="w-full text-sm">
            <thead><tr class="border-b bg-slate-50 text-xs text-muted-foreground"><th class="text-left px-4 py-3 font-semibold">Aksi</th><th class="text-center px-4 py-3 font-semibold">Poin</th><th class="text-center px-4 py-3 font-semibold">Multiplier</th><th class="text-center px-4 py-3 font-semibold">Status</th><th class="px-4 py-3" /></tr></thead>
            <tbody class="divide-y">
              <tr v-for="r in rulesByCategory(cat)" :key="r.id" :class="r.enabled ? '' : 'opacity-40 bg-slate-50'">
                <td class="px-4 py-3"><span class="inline-flex items-center gap-2"><CheckCircle2 class="w-4 h-4 text-emerald-500 shrink-0" />{{ r.action }}</span></td>
                <td class="px-4 py-3 text-center font-black text-indigo-600">+{{ r.base_points }}</td>
                <td class="px-4 py-3 text-center text-slate-500">×{{ r.multiplier }}</td>
                <td class="px-4 py-3 text-center">
                  <button @click="toggleRule(r)"><component :is="r.enabled ? ToggleRight : ToggleLeft" class="w-7 h-7 mx-auto" :class="r.enabled ? 'text-emerald-500' : 'text-slate-300'" /></button>
                </td>
                <td class="px-4 py-3 text-right">
                  <button class="p-1.5 rounded hover:bg-slate-100" @click="openEditRule(r)"><Pencil class="w-3.5 h-3.5 text-slate-500" /></button>
                  <button class="p-1.5 rounded hover:bg-red-50" @click="deleteRule(r)"><Trash2 class="w-3.5 h-3.5 text-red-500" /></button>
                </td>
              </tr>
            </tbody>
          </table>
        </Card>
      </div>
    </template>

    <template v-else-if="view === 'badge'">
      <div class="flex items-center justify-between flex-wrap gap-3">
        <div><h2 class="text-lg font-bold">Badge &amp; Achievement</h2><p class="text-sm text-muted-foreground">Jumlah "diraih" dihitung langsung dari data hasil tryout siswa, bukan angka tetap</p></div>
        <Button variant="gradient" @click="openCreateBadge"><Plus class="w-4 h-4" />Buat Badge</Button>
      </div>
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card class="p-4 flex items-center gap-3"><div class="w-10 h-10 rounded-xl bg-indigo-50 flex items-center justify-center"><Award class="w-5 h-5 text-indigo-600" /></div><div><div class="text-xl font-black text-indigo-600">{{ badgeStats.total }}</div><div class="text-xs text-muted-foreground">Total Badge</div></div></Card>
        <Card class="p-4 flex items-center gap-3"><div class="w-10 h-10 rounded-xl bg-emerald-50 flex items-center justify-center"><CheckCircle2 class="w-5 h-5 text-emerald-600" /></div><div><div class="text-xl font-black text-emerald-600">{{ badgeStats.active }}</div><div class="text-xs text-muted-foreground">Badge Aktif</div></div></Card>
        <Card class="p-4 flex items-center gap-3"><div class="w-10 h-10 rounded-xl bg-blue-50 flex items-center justify-center"><Users class="w-5 h-5 text-blue-600" /></div><div><div class="text-xl font-black text-blue-600">{{ badgeStats.totalEarned }}</div><div class="text-xs text-muted-foreground">Total Diraih</div></div></Card>
        <Card class="p-4 flex items-center gap-3"><div class="w-10 h-10 rounded-xl bg-amber-50 flex items-center justify-center"><Crown class="w-5 h-5 text-amber-600" /></div><div><div class="text-xl font-black text-amber-600">{{ badgeStats.legendary }}</div><div class="text-xs text-muted-foreground">Legendary</div></div></Card>
      </div>
      <Card v-if="badges.length === 0" class="p-10 text-center"><Award class="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" /><p class="text-sm text-muted-foreground">Belum ada badge.</p></Card>
      <div v-else class="grid sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        <Card v-for="b in badges" :key="b.id" class="p-4" :class="!b.active ? 'opacity-50' : ''">
          <div class="flex items-start justify-between mb-3">
            <div class="w-12 h-12 rounded-2xl bg-slate-100 flex items-center justify-center overflow-hidden"><IconDisplay :icon-type="b.icon_type" :icon-name="b.icon_name" :icon-url="b.icon_url" size="w-6 h-6 text-indigo-600" /></div>
            <div class="flex items-center gap-1">
              <button class="p-1 rounded hover:bg-slate-100" @click="toggleBadge(b)"><component :is="b.active ? ToggleRight : ToggleLeft" class="w-5 h-5" :class="b.active ? 'text-emerald-500' : 'text-slate-300'" /></button>
              <button class="p-1 rounded hover:bg-slate-100" @click="openEditBadge(b)"><Pencil class="w-4 h-4 text-slate-400" /></button>
              <button class="p-1 rounded hover:bg-red-50" @click="openDeleteBadge(b)"><Trash2 class="w-4 h-4 text-slate-400 hover:text-red-500" /></button>
            </div>
          </div>
          <h4 class="font-bold text-slate-900 text-sm mb-0.5">{{ b.name }}</h4>
          <p class="text-xs text-muted-foreground mb-3 leading-relaxed">{{ b.description }}</p>
          <div class="flex items-center justify-between">
            <Badge variant="outline">{{ RARITY_LABEL[b.rarity] }}</Badge>
            <span class="text-xs text-muted-foreground flex items-center gap-1"><Users class="w-3 h-3" />{{ b.earned_by }}</span>
          </div>
          <div class="mt-2 pt-2 border-t text-xs text-slate-500">{{ CONDITION_LABEL[b.condition_type] }} <strong>{{ b.condition_value }}</strong></div>
        </Card>
      </div>
    </template>

    <template v-else-if="view === 'leaderboard'">
      <div><h2 class="text-lg font-bold flex items-center gap-2"><Trophy class="w-5 h-5 text-amber-500" />Leaderboard Nasional</h2><p class="text-sm text-muted-foreground">Diurutkan dari skor tryout terbaik tiap siswa — diperbarui otomatis, tidak ada override manual</p></div>
      <Card v-if="leaderboard.length === 0" class="p-10 text-center"><Trophy class="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" /><p class="text-sm text-muted-foreground">Belum ada siswa yang menyelesaikan tryout.</p></Card>
      <Card v-else class="p-5">
        <div class="space-y-2">
          <div v-for="entry in leaderboard" :key="entry.student_id" class="flex items-center gap-3 p-3 rounded-xl" :class="entry.rank <= 3 ? 'bg-amber-50 border border-amber-200' : 'bg-slate-50'">
            <div class="w-8 h-8 rounded-lg flex items-center justify-center text-sm font-black shrink-0" :class="entry.rank === 1 ? 'bg-amber-400 text-white' : entry.rank === 2 ? 'bg-slate-300 text-slate-700' : entry.rank === 3 ? 'bg-orange-300 text-white' : 'bg-white border text-slate-500'">
              <Medal v-if="entry.rank <= 3" class="w-4 h-4" />
              <span v-else>#{{ entry.rank }}</span>
            </div>
            <div class="flex-1 min-w-0"><p class="font-semibold text-sm text-slate-900 truncate">{{ entry.student_name }}</p><p class="text-xs text-muted-foreground truncate">{{ entry.school_name || '—' }}</p></div>
            <p class="font-black text-sm text-slate-900">{{ entry.best_score }}</p>
          </div>
        </div>
      </Card>
    </template>

    <template v-else-if="view === 'challenge'">
      <div class="flex items-center justify-between flex-wrap gap-3">
        <div><h2 class="text-lg font-bold">Challenge &amp; Event</h2><p class="text-sm text-muted-foreground">Peserta &amp; penyelesai dihitung langsung dari aktivitas tryout/drilling siswa dalam rentang tanggal challenge</p></div>
        <Button variant="gradient" @click="openCreateChallenge"><Plus class="w-4 h-4" />Buat Challenge</Button>
      </div>
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Card class="p-4 flex items-center gap-3"><div class="w-10 h-10 rounded-xl bg-emerald-50 flex items-center justify-center"><Play class="w-5 h-5 text-emerald-600" /></div><div><div class="text-xl font-black text-emerald-600">{{ challengeStats.active }}</div><div class="text-xs text-muted-foreground">Aktif</div></div></Card>
        <Card class="p-4 flex items-center gap-3"><div class="w-10 h-10 rounded-xl bg-blue-50 flex items-center justify-center"><Clock class="w-5 h-5 text-blue-600" /></div><div><div class="text-xl font-black text-blue-600">{{ challengeStats.upcoming }}</div><div class="text-xs text-muted-foreground">Akan Datang</div></div></Card>
        <Card class="p-4 flex items-center gap-3"><div class="w-10 h-10 rounded-xl bg-indigo-50 flex items-center justify-center"><Users class="w-5 h-5 text-indigo-600" /></div><div><div class="text-xl font-black text-indigo-600">{{ challengeStats.totalParticipants }}</div><div class="text-xs text-muted-foreground">Total Peserta</div></div></Card>
        <Card class="p-4 flex items-center gap-3"><div class="w-10 h-10 rounded-xl bg-amber-50 flex items-center justify-center"><CheckCircle2 class="w-5 h-5 text-amber-600" /></div><div><div class="text-xl font-black text-amber-600">{{ challengeStats.totalCompletions }}</div><div class="text-xs text-muted-foreground">Total Selesai</div></div></Card>
      </div>
      <div class="flex gap-2 flex-wrap">
        <button v-for="s in ['all','active','upcoming','ended']" :key="s" class="px-4 py-1.5 rounded-full text-sm font-semibold border-2 transition-colors" :class="filterChallengeStatus === s ? 'border-indigo-500 bg-indigo-50 text-indigo-700' : 'border-transparent bg-slate-100 text-slate-500 hover:bg-slate-200'" @click="filterChallengeStatus = s as any">
          {{ s === 'all' ? 'Semua' : CHALLENGE_STATUS_LABEL[s as ChallengeStatus].label }}
        </button>
      </div>
      <Card v-if="filteredChallenges.length === 0" class="p-10 text-center"><Sparkles class="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" /><p class="text-sm text-muted-foreground">Belum ada challenge.</p></Card>
      <div v-else class="grid lg:grid-cols-2 gap-5">
        <Card v-for="c in filteredChallenges" :key="c.id" class="p-5" :class="c.status === 'ended' ? 'opacity-70' : ''">
          <div class="flex items-start justify-between mb-3">
            <div class="flex-1 min-w-0 pr-3">
              <div class="flex items-center gap-2 mb-1 flex-wrap"><Badge :variant="CHALLENGE_STATUS_LABEL[c.status].variant">{{ CHALLENGE_STATUS_LABEL[c.status].label }}</Badge><Badge variant="outline" class="inline-flex items-center gap-1"><component :is="CHALLENGE_TYPE_LABEL[c.challenge_type].icon" class="w-3 h-3" />{{ CHALLENGE_TYPE_LABEL[c.challenge_type].label }}</Badge></div>
              <h4 class="font-bold text-slate-900">{{ c.name }}</h4>
              <p class="text-xs text-muted-foreground mt-0.5">{{ c.description }}</p>
            </div>
            <div class="flex items-center gap-1 shrink-0">
              <button class="p-1.5 rounded-lg hover:bg-slate-100" @click="openEditChallenge(c)"><Pencil class="w-4 h-4 text-slate-400" /></button>
              <button class="p-1.5 rounded-lg hover:bg-red-50" @click="deleteChallenge(c)"><Trash2 class="w-4 h-4 text-slate-400 hover:text-red-500" /></button>
            </div>
          </div>
          <div class="grid grid-cols-3 gap-3 mb-3 text-center">
            <div class="bg-slate-50 rounded-xl p-2.5"><p class="text-lg font-black text-slate-900">{{ c.participants }}</p><p class="text-[10px] text-muted-foreground">Peserta</p></div>
            <div class="bg-emerald-50 rounded-xl p-2.5"><p class="text-lg font-black text-emerald-700">{{ c.completions }}</p><p class="text-[10px] text-muted-foreground">Selesai</p></div>
            <div class="bg-indigo-50 rounded-xl p-2.5"><p class="text-lg font-black text-indigo-700">{{ c.reward_points }}</p><p class="text-[10px] text-muted-foreground">Poin Reward</p></div>
          </div>
          <div class="flex items-center justify-between pt-3 border-t">
            <div class="flex items-center gap-3 text-xs text-muted-foreground"><span class="flex items-center gap-1"><Calendar class="w-3.5 h-3.5" />{{ c.start_date }}</span><ChevronRight class="w-3 h-3" /><span>{{ c.end_date }}</span></div>
            <Award v-if="c.reward_badge" class="w-5 h-5 text-amber-500" />
            <button v-if="c.status !== 'ended'" class="text-xs font-semibold px-3 py-1.5 rounded-lg transition-colors" :class="c.status === 'active' ? 'bg-red-50 text-red-600 hover:bg-red-100' : 'bg-emerald-50 text-emerald-700 hover:bg-emerald-100'" @click="setChallengeStatus(c, c.status === 'active' ? 'ended' : 'active')">
              <span class="flex items-center gap-1"><component :is="c.status === 'active' ? Pause : Play" class="w-3 h-3" />{{ c.status === 'active' ? 'Akhiri' : 'Aktifkan' }}</span>
            </button>
          </div>
        </Card>
      </div>
    </template>

    <!-- Point rule modal -->
    <Teleport to="body">
      <div v-if="showRuleForm" class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" @click="showRuleForm = false">
        <Card class="w-full max-w-md p-6 shadow-2xl" @click.stop>
          <div class="flex items-center justify-between mb-5"><h3 class="font-bold text-slate-900">{{ editRule ? 'Edit Aturan Poin' : 'Aturan Poin Baru' }}</h3><button class="p-1.5 rounded-lg hover:bg-slate-100" @click="showRuleForm = false"><X class="w-4 h-4" /></button></div>
          <div class="space-y-4">
            <div v-if="!editRule">
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Aksi</label>
              <input v-model="ruleForm.action" placeholder="contoh: Jawab Benar — Pilihan Ganda" class="w-full border rounded-lg px-3 py-2 text-sm" />
            </div>
            <div v-if="!editRule">
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Kategori</label><input v-model="ruleForm.category" class="w-full border rounded-lg px-3 py-2 text-sm" />
            </div>
            <div>
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Poin Dasar</label>
              <input v-model.number="ruleForm.base_points" type="number" min="0" class="w-full border rounded-lg px-3 py-2 text-sm" />
            </div>
            <div>
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Multiplier</label>
              <input v-model.number="ruleForm.multiplier" type="number" min="0.1" step="0.1" class="w-full border rounded-lg px-3 py-2 text-sm" />
              <p class="text-xs text-muted-foreground mt-1">Total per aksi: <strong class="text-emerald-600">+{{ Math.round(ruleForm.base_points * ruleForm.multiplier) }}</strong></p>
            </div>
            <div class="flex items-center gap-3 p-3 bg-slate-50 rounded-lg">
              <button @click="ruleForm.enabled = !ruleForm.enabled"><component :is="ruleForm.enabled ? ToggleRight : ToggleLeft" class="w-7 h-7" :class="ruleForm.enabled ? 'text-emerald-500' : 'text-slate-300'" /></button>
              <p class="text-sm font-medium">{{ ruleForm.enabled ? 'Aktif' : 'Nonaktif' }}</p>
            </div>
          </div>
          <div class="flex gap-2 mt-6"><Button variant="outline" class="flex-1" @click="showRuleForm = false">Batal</Button><Button variant="gradient" class="flex-1 gap-1.5" @click="saveRule"><Save class="w-4 h-4" />Simpan</Button></div>
        </Card>
      </div>
    </Teleport>

    <!-- Badge modal -->
    <Teleport to="body">
      <div v-if="showBadgeForm" class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" @click="showBadgeForm = false">
        <Card class="w-full max-w-lg p-6 shadow-2xl max-h-[90vh] overflow-y-auto" @click.stop>
          <div class="flex items-center justify-between mb-5"><h3 class="font-bold text-slate-900">{{ editBadge ? 'Edit Badge' : 'Buat Badge Baru' }}</h3><button class="p-1.5 rounded-lg hover:bg-slate-100" @click="showBadgeForm = false"><X class="w-4 h-4" /></button></div>
          <div class="space-y-4">
            <div>
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-2">Icon Badge</label>
              <IconPicker v-model="badgeIcon" />
            </div>
            <div><label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Badge</label><input v-model="badgeForm.name" placeholder="contoh: 7 Day Streak" class="w-full border rounded-lg px-3 py-2 text-sm" /></div>
            <div><label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Deskripsi</label><textarea v-model="badgeForm.description" rows="2" class="w-full border rounded-lg px-3 py-2 text-sm resize-none" /></div>
            <div class="grid grid-cols-2 gap-3">
              <div><label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Rarity</label>
                <select v-model="badgeForm.rarity" class="w-full border rounded-lg px-3 py-2 text-sm"><option value="common">Common</option><option value="rare">Rare</option><option value="epic">Epic</option><option value="legendary">Legendary</option></select>
              </div>
              <div><label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Tipe Kondisi</label>
                <select v-model="badgeForm.condition_type" class="w-full border rounded-lg px-3 py-2 text-sm"><option v-for="(label, key) in CONDITION_LABEL" :key="key" :value="key">{{ label }}</option></select>
              </div>
            </div>
            <div><label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Nilai Kondisi</label><input v-model.number="badgeForm.condition_value" type="number" min="0" class="w-full border rounded-lg px-3 py-2 text-sm" /><p class="text-xs text-muted-foreground mt-1">{{ CONDITION_LABEL[badgeForm.condition_type] }} <strong>{{ badgeForm.condition_value }}</strong></p></div>
          </div>
          <div class="flex gap-2 mt-6"><Button variant="outline" class="flex-1" @click="showBadgeForm = false">Batal</Button><Button variant="gradient" class="flex-1 gap-1.5" @click="saveBadge"><Save class="w-4 h-4" />Simpan Badge</Button></div>
        </Card>
      </div>
    </Teleport>
    <ConfirmDialog v-model="showBadgeDelete" title="Hapus badge ini?" description="Tindakan ini tidak bisa dibatalkan." confirm-label="Hapus" @confirm="confirmDeleteBadge" />

    <!-- Challenge modal -->
    <Teleport to="body">
      <div v-if="showChallengeForm" class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" @click="showChallengeForm = false">
        <Card class="w-full max-w-lg p-6 shadow-2xl max-h-[90vh] overflow-y-auto" @click.stop>
          <div class="flex items-center justify-between mb-5"><h3 class="font-bold text-slate-900">{{ editChallenge ? 'Edit Challenge' : 'Buat Challenge Baru' }}</h3><button class="p-1.5 rounded-lg hover:bg-slate-100" @click="showChallengeForm = false"><X class="w-4 h-4" /></button></div>
          <div class="space-y-4">
            <div><label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Challenge</label><input v-model="challengeForm.name" placeholder="contoh: Tryout Marathon Januari" class="w-full border rounded-lg px-3 py-2 text-sm" /></div>
            <div><label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Deskripsi</label><textarea v-model="challengeForm.description" rows="2" class="w-full border rounded-lg px-3 py-2 text-sm resize-none" /></div>
            <div>
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-2">Tipe Challenge</label>
              <div class="grid grid-cols-2 gap-2">
                <button v-for="(cfg, key) in CHALLENGE_TYPE_LABEL" :key="key" type="button" class="flex items-center gap-2 px-3 py-2.5 rounded-xl border-2 text-sm font-semibold" :class="challengeForm.challenge_type === key ? 'border-indigo-500 bg-indigo-50 text-indigo-700' : 'border-slate-200 text-slate-500 hover:border-slate-300'" @click="challengeForm.challenge_type = key as ChallengeType">
                  <component :is="cfg.icon" class="w-4 h-4" />{{ cfg.label }}
                </button>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div><label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Mulai</label><input v-model="challengeForm.start_date" type="date" class="w-full border rounded-lg px-3 py-2 text-sm" /></div>
              <div><label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Selesai</label><input v-model="challengeForm.end_date" type="date" class="w-full border rounded-lg px-3 py-2 text-sm" /></div>
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div><label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Target</label><input v-model.number="challengeForm.target_value" type="number" min="1" class="w-full border rounded-lg px-3 py-2 text-sm" /></div>
              <div><label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Reward Poin</label><input v-model.number="challengeForm.reward_points" type="number" min="0" class="w-full border rounded-lg px-3 py-2 text-sm" /></div>
            </div>
          </div>
          <div class="flex gap-2 mt-6"><Button variant="outline" class="flex-1" @click="showChallengeForm = false">Batal</Button><Button variant="gradient" class="flex-1 gap-1.5" @click="saveChallenge"><Save class="w-4 h-4" />Simpan</Button></div>
        </Card>
      </div>
    </Teleport>

    <p class="text-xs text-muted-foreground flex items-center gap-1.5"><Info class="w-3.5 h-3.5" />Semua angka "diraih/peserta/selesai" pada halaman ini dihitung langsung dari data attempts asli, bukan simulasi.</p>
  </div>
</template>
