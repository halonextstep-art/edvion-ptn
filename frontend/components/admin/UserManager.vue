<script setup lang="ts">
import { Search, Pencil, Trash2, Plus, UserCog, GraduationCap, School as SchoolIcon, Layers, ShieldCheck, UserCheck, UserX, AlertCircle, Eye, RefreshCw, X, Power, Loader2, Clock } from 'lucide-vue-next'
import type { Attempt, Role, SimulationRunItem, User, UserStatus } from '~/types'

const { userService, settingsService, tryoutService, simulationService } = useApi()
const { user: currentUser } = useAuth()
const toast = useToast()

// Platform-wide B2C (schoolless) self-registration on/off toggle — see backend
// `domain::platform_settings` doc comment. Only affects the public `/daftar` self-service
// path; never blocks B2B self-register or an admin manually creating a B2C account here.
const b2cEnabled = ref<boolean | null>(null)
const b2cLoading = ref(false)

async function loadB2cSetting() {
  try {
    const s = await settingsService.getB2cRegistration()
    b2cEnabled.value = s.b2c_registration_enabled
  } catch {
    // non-critical for the rest of the page — the toggle card just won't render
  }
}

async function toggleB2c() {
  if (b2cEnabled.value === null) return
  const next = !b2cEnabled.value
  b2cLoading.value = true
  try {
    const s = await settingsService.setB2cRegistration(next)
    b2cEnabled.value = s.b2c_registration_enabled
    toast.success(next ? 'Pendaftaran mandiri (B2C) diaktifkan' : 'Pendaftaran mandiri (B2C) dinonaktifkan')
  } catch (e: any) {
    toast.error('Gagal mengubah pengaturan', e?.message)
  } finally {
    b2cLoading.value = false
  }
}

const users = ref<User[]>([])
const allUsers = ref<User[]>([])
const loading = ref(false)
const search = ref('')
const filterRole = ref<Role | ''>('')
const filterStatus = ref<UserStatus | ''>('')
/** B2B/B2C segment — only meaningful for `role: student`, since school PIC/admin/content
 * accounts are always B2B/no-school by construction. `''` = no filtering. */
const filterSegment = ref<'' | 'b2b' | 'b2c'>('')

const showForm = ref(false)
const editTarget = ref<User | null>(null)
const showDelete = ref(false)
const deleteTarget = ref<User | null>(null)
const detailUser = ref<User | null>(null)

// "Status Pengerjaan" di drawer detail — sebelumnya admin tidak punya cara sama sekali melihat
// apakah seorang siswa sedang mengerjakan tryout/drilling/simulasi (semua endpoint listing
// attempt/run sebelumnya self-scoped ke siswa itu sendiri). Dimuat sekali tiap drawer dibuka
// untuk siswa, bukan di-poll — cukup untuk "cek status sekarang", bukan live-monitoring.
const activityLoading = ref(false)
const inProgressAttempts = ref<Attempt[]>([])
const inProgressRuns = ref<SimulationRunItem[]>([])
// Jumlah pelanggaran Mode Terkunci per run in-progress (id -> count) — hanya dimuat untuk run
// yang memang lockdown_enabled, lihat SimulationRunItem.lockdown_enabled doc comment.
const runViolationCounts = ref<Record<string, number>>({})
watch(detailUser, async (u) => {
  inProgressAttempts.value = []
  inProgressRuns.value = []
  runViolationCounts.value = {}
  if (!u || u.role !== 'student') return
  activityLoading.value = true
  try {
    const [attempts, runs] = await Promise.allSettled([
      tryoutService.adminListAttempts(u.id),
      simulationService.adminListRuns(u.id),
    ])
    if (attempts.status === 'fulfilled') inProgressAttempts.value = attempts.value.filter((a) => a.status === 'in_progress')
    if (runs.status === 'fulfilled') {
      inProgressRuns.value = runs.value.filter((r) => r.status === 'in_progress')
      const lockedRuns = inProgressRuns.value.filter((r) => r.lockdown_enabled)
      const results = await Promise.allSettled(lockedRuns.map((r) => simulationService.listViolations(r.id)))
      const counts: Record<string, number> = {}
      lockedRuns.forEach((r, i) => {
        const res = results[i]
        if (res.status === 'fulfilled') counts[r.id] = res.value.length
      })
      runViolationCounts.value = counts
    }
  } finally {
    activityLoading.value = false
  }
})

const roleConfig: Record<Role, { label: string; icon: any; bg: string; color: string }> = {
  admin: { label: 'Admin', icon: ShieldCheck, bg: 'bg-rose-50 border-rose-200', color: 'text-rose-600' },
  school: { label: 'Sekolah', icon: SchoolIcon, bg: 'bg-emerald-50 border-emerald-200', color: 'text-emerald-600' },
  student: { label: 'Siswa', icon: GraduationCap, bg: 'bg-blue-50 border-blue-200', color: 'text-blue-600' },
  content: { label: 'Tim Konten', icon: Layers, bg: 'bg-violet-50 border-violet-200', color: 'text-violet-600' },
}

const statusBadge: Record<UserStatus, { label: string; dot: string; class: string }> = {
  active: { label: 'Aktif', dot: 'bg-emerald-400', class: 'text-emerald-700 bg-emerald-50 border-emerald-200' },
  inactive: { label: 'Nonaktif', dot: 'bg-slate-400', class: 'text-slate-600 bg-slate-100 border-slate-200' },
  pending: { label: 'Pending', dot: 'bg-amber-400', class: 'text-amber-700 bg-amber-50 border-amber-200' },
}

const roleStats = computed(() => {
  const active = allUsers.value.filter((u) => u.status === 'active').length
  const pendingSchools = allUsers.value.filter((u) => u.role === 'school' && u.status === 'pending').length
  return [
    { role: null, label: 'Total User', icon: UserCog, color: 'text-slate-700', val: allUsers.value.length, extra: `${active} aktif` },
    { role: 'student' as Role, label: 'Siswa', icon: GraduationCap, color: 'text-blue-600', val: allUsers.value.filter((u) => u.role === 'student').length, extra: '' },
    { role: 'school' as Role, label: 'Sekolah', icon: SchoolIcon, color: 'text-emerald-600', val: allUsers.value.filter((u) => u.role === 'school').length, extra: pendingSchools ? `${pendingSchools} pending` : '' },
    { role: 'content' as Role, label: 'Tim Konten', icon: Layers, color: 'text-violet-600', val: allUsers.value.filter((u) => u.role === 'content').length, extra: '' },
    { role: 'admin' as Role, label: 'Admin', icon: ShieldCheck, color: 'text-rose-600', val: allUsers.value.filter((u) => u.role === 'admin').length, extra: '' },
  ]
})

const pendingCount = computed(() => allUsers.value.filter((u) => u.status === 'pending').length)

async function loadStats() {
  try {
    allUsers.value = await userService.list({})
  } catch {
    // non-critical for the stat strip
  }
}

async function load() {
  loading.value = true
  try {
    users.value = await userService.list({
      role: filterRole.value || undefined,
      status: filterStatus.value || undefined,
      search: search.value || undefined,
      has_school: filterSegment.value === 'b2b' ? true : filterSegment.value === 'b2c' ? false : undefined,
    })
  } catch (e: any) {
    toast.error('Gagal memuat daftar user', e?.message)
  } finally {
    loading.value = false
  }
}

let debounceTimer: ReturnType<typeof setTimeout>
watch([search, filterRole, filterStatus, filterSegment], () => {
  clearTimeout(debounceTimer)
  debounceTimer = setTimeout(load, 250)
})

// The B2B/B2C segment only applies to students — silently drop it if the role filter moves
// away from "student" (e.g. via the dropdown, which doesn't go through `filterByRole`).
watch(filterRole, (r) => {
  if (r !== 'student') filterSegment.value = ''
})

onMounted(() => {
  load()
  loadStats()
  loadB2cSetting()
})

function filterByRole(role: Role | null) {
  filterRole.value = role ?? ''
  if (filterRole.value !== 'student') filterSegment.value = ''
}

function openCreate() {
  editTarget.value = null
  showForm.value = true
}
function openEdit(u: User) {
  editTarget.value = u
  showForm.value = true
}
function openDelete(u: User) {
  deleteTarget.value = u
  showDelete.value = true
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  try {
    await userService.remove(deleteTarget.value.id)
    toast.success('Akun berhasil dihapus!')
    detailUser.value = null
    load()
    loadStats()
  } catch (e: any) {
    toast.error('Gagal menghapus akun', e?.message)
  }
}

function onSaved() {
  load()
  loadStats()
}

async function toggleStatus(u: User) {
  const next: UserStatus = u.status === 'active' ? 'inactive' : 'active'
  try {
    await userService.setStatus(u.id, next)
    toast.success(next === 'active' ? 'Akun diaktifkan' : 'Akun dinonaktifkan')
    if (detailUser.value?.id === u.id) detailUser.value = { ...detailUser.value, status: next }
    load()
    loadStats()
  } catch (e: any) {
    toast.error('Gagal mengubah status', e?.message)
  }
}

async function approve(u: User) {
  try {
    await userService.setStatus(u.id, 'active')
    toast.success('Akun disetujui dan diaktifkan')
    load()
    loadStats()
  } catch (e: any) {
    toast.error('Gagal menyetujui akun', e?.message)
  }
}

function genPassword(seed: string) {
  const part = (seed || 'user').replace(/\s+/g, '').slice(0, 4).toLowerCase() || 'user'
  return `Gspl${part}${Math.floor(1000 + Math.random() * 9000)}!`
}

async function resetPassword(u: User) {
  const newPassword = genPassword(u.name)
  try {
    await userService.update(u.id, {
      name: u.name,
      email: u.email,
      role: u.role,
      school_id: u.school_id,
      status: u.status,
      phone: u.phone,
      nisn: u.nisn,
      grade: u.grade,
      password: newPassword,
    })
    try {
      await navigator.clipboard?.writeText(newPassword)
      toast.success(`Password baru: ${newPassword} (disalin ke clipboard)`)
    } catch {
      toast.success(`Password baru untuk ${u.name}: ${newPassword}`)
    }
  } catch (e: any) {
    toast.error('Gagal reset password', e?.message)
  }
}

function formatDate(d?: string | null) {
  if (!d) return null
  return new Date(d).toLocaleDateString('id-ID', { day: '2-digit', month: 'short', year: 'numeric' })
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2"><UserCog class="w-5 h-5 text-indigo-600" />Manajemen User</h2>
        <p class="text-sm text-muted-foreground">{{ users.length }} akun terdaftar</p>
      </div>
      <Button variant="gradient" @click="openCreate"><Plus class="w-4 h-4" />Tambah Akun</Button>
    </div>

    <div class="grid grid-cols-2 lg:grid-cols-5 gap-3">
      <button
        v-for="s in roleStats"
        :key="s.label"
        class="bg-white rounded-2xl border p-4 text-left hover:shadow-sm transition-shadow"
        :class="filterRole === (s.role ?? '') ? 'border-indigo-400 ring-1 ring-indigo-200' : 'border-slate-200'"
        @click="filterByRole(s.role)"
      >
        <component :is="s.icon" class="w-5 h-5 mb-2" :class="s.color" />
        <p class="text-2xl font-black" :class="s.color">{{ s.val }}</p>
        <p class="text-xs text-slate-500 mt-0.5">{{ s.label }}</p>
        <p v-if="s.extra" class="text-[10px] text-slate-400 mt-0.5">{{ s.extra }}</p>
      </button>
    </div>

    <div class="flex flex-col sm:flex-row gap-2">
      <div class="relative flex-1">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
        <input v-model="search" placeholder="Cari nama atau email..." class="w-full pl-9 pr-3 py-2 border rounded-lg text-sm bg-white" />
      </div>
      <select v-model="filterRole" class="px-3 py-2 border rounded-lg text-sm bg-white">
        <option value="">Semua Role</option>
        <option value="admin">Admin</option>
        <option value="school">Sekolah</option>
        <option value="student">Siswa</option>
        <option value="content">Tim Konten</option>
      </select>
      <select v-model="filterStatus" class="px-3 py-2 border rounded-lg text-sm bg-white">
        <option value="">Semua Status</option>
        <option value="active">Aktif</option>
        <option value="inactive">Nonaktif</option>
        <option value="pending">Pending</option>
      </select>
    </div>

    <div v-if="filterRole === 'student'" class="flex items-center gap-2">
      <span class="text-xs font-semibold text-slate-500">Segmen:</span>
      <div class="flex rounded-lg border bg-white p-0.5">
        <button
          type="button"
          class="px-3 py-1 rounded-md text-xs font-semibold transition-colors"
          :class="filterSegment === '' ? 'bg-indigo-600 text-white' : 'text-slate-500 hover:bg-slate-50'"
          @click="filterSegment = ''"
        >Semua</button>
        <button
          type="button"
          class="px-3 py-1 rounded-md text-xs font-semibold transition-colors"
          :class="filterSegment === 'b2b' ? 'bg-indigo-600 text-white' : 'text-slate-500 hover:bg-slate-50'"
          @click="filterSegment = 'b2b'"
        >B2B (Sekolah)</button>
        <button
          type="button"
          class="px-3 py-1 rounded-md text-xs font-semibold transition-colors"
          :class="filterSegment === 'b2c' ? 'bg-indigo-600 text-white' : 'text-slate-500 hover:bg-slate-50'"
          @click="filterSegment = 'b2c'"
        >B2C (Mandiri)</button>
      </div>
    </div>

    <Card v-if="b2cEnabled !== null" class="p-4 flex items-center justify-between gap-4 flex-wrap">
      <div class="flex items-center gap-3">
        <div
          class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0"
          :class="b2cEnabled ? 'bg-emerald-100' : 'bg-slate-100'"
        >
          <Power class="w-4 h-4" :class="b2cEnabled ? 'text-emerald-600' : 'text-slate-400'" />
        </div>
        <div>
          <p class="text-sm font-bold text-slate-800">Pendaftaran Mandiri (B2C)</p>
          <p class="text-xs text-muted-foreground">
            {{ b2cEnabled ? 'Halaman /daftar terbuka — siapa saja bisa daftar akun siswa tanpa sekolah.' : 'Halaman /daftar dinonaktifkan — pendaftaran mandiri tanpa sekolah ditutup sementara.' }}
            Tidak memengaruhi pendaftaran siswa lewat sekolah (B2B) maupun akun yang dibuat admin.
          </p>
        </div>
      </div>
      <button
        type="button"
        :disabled="b2cLoading"
        class="px-4 py-2 rounded-xl text-xs font-bold transition-colors disabled:opacity-50"
        :class="b2cEnabled ? 'bg-slate-100 hover:bg-slate-200 text-slate-700' : 'bg-emerald-600 hover:bg-emerald-500 text-white'"
        @click="toggleB2c"
      >{{ b2cEnabled ? 'Nonaktifkan' : 'Aktifkan' }}</button>
    </Card>

    <div v-if="pendingCount > 0" class="bg-amber-50 border border-amber-200 rounded-2xl p-4 flex items-center justify-between gap-4 flex-wrap">
      <div class="flex items-center gap-3">
        <div class="w-9 h-9 rounded-xl bg-amber-100 flex items-center justify-center shrink-0"><AlertCircle class="w-4 h-4 text-amber-600" /></div>
        <div>
          <p class="text-sm font-bold text-amber-800">{{ pendingCount }} akun menunggu persetujuan</p>
          <p class="text-xs text-amber-600">Tinjau dan setujui akun yang baru mendaftar</p>
        </div>
      </div>
      <button class="px-4 py-2 rounded-xl bg-amber-600 hover:bg-amber-500 text-white text-xs font-bold transition-colors" @click="filterStatus = 'pending'">
        Lihat Semua
      </button>
    </div>

    <Card class="overflow-hidden">
      <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
      <div v-else-if="users.length === 0" class="p-10 text-center text-sm text-muted-foreground">Belum ada akun yang cocok dengan filter.</div>
      <div v-else class="divide-y divide-slate-50">
        <div v-for="u in users" :key="u.id" class="flex items-center gap-4 px-5 py-3.5 hover:bg-slate-50 transition-colors">
          <div class="w-9 h-9 rounded-xl flex items-center justify-center border shrink-0" :class="roleConfig[u.role]?.bg">
            <component :is="roleConfig[u.role]?.icon" class="w-4 h-4" :class="roleConfig[u.role]?.color" />
          </div>

          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <p class="text-sm font-semibold text-slate-900">{{ u.name }}</p>
              <span class="text-[10px] px-2 py-0.5 rounded-full border font-semibold" :class="roleConfig[u.role]?.bg + ' ' + roleConfig[u.role]?.color">{{ roleConfig[u.role]?.label }}</span>
              <span class="text-[10px] px-2 py-0.5 rounded-full border flex items-center gap-1 font-semibold" :class="statusBadge[u.status]?.class">
                <span class="w-1.5 h-1.5 rounded-full" :class="statusBadge[u.status]?.dot" />{{ statusBadge[u.status]?.label }}
              </span>
            </div>
            <div class="flex items-center gap-3 mt-0.5 flex-wrap">
              <p class="text-xs text-slate-500">{{ u.email }}</p>
              <p v-if="u.school_name" class="text-xs text-slate-400">{{ u.school_name }}</p>
              <span v-else-if="u.role === 'student'" class="text-[10px] px-1.5 py-0.5 rounded border border-sky-200 bg-sky-50 text-sky-600 font-semibold">Mandiri (B2C)</span>
              <p v-if="u.grade" class="text-xs text-slate-400">{{ u.grade }}</p>
              <p v-if="u.nisn" class="text-xs text-slate-400 font-mono">NISN: {{ u.nisn }}</p>
            </div>
          </div>

          <div class="hidden lg:block text-right shrink-0">
            <p class="text-xs text-slate-400">Dibuat: {{ formatDate(u.created_at) }}</p>
            <p class="text-xs text-slate-400">{{ u.last_login ? `Login: ${formatDate(u.last_login)}` : 'Belum pernah login' }}</p>
          </div>

          <div class="flex items-center gap-1.5 shrink-0">
            <button
              v-if="u.status === 'pending'"
              class="px-3 py-1.5 rounded-lg bg-emerald-50 hover:bg-emerald-100 text-emerald-700 text-xs font-bold border border-emerald-200 transition-colors flex items-center gap-1"
              @click="approve(u)"
            ><UserCheck class="w-3.5 h-3.5" />Setujui</button>
            <button
              v-else-if="u.id !== currentUser?.id"
              class="p-1.5 rounded-lg border transition-colors"
              :class="u.status === 'active' ? 'text-slate-500 hover:text-orange-500 hover:bg-orange-50 border-transparent hover:border-orange-200' : 'text-slate-400 hover:text-emerald-600 hover:bg-emerald-50 border-transparent hover:border-emerald-200'"
              :title="u.status === 'active' ? 'Nonaktifkan' : 'Aktifkan'"
              @click="toggleStatus(u)"
            >
              <UserX v-if="u.status === 'active'" class="w-4 h-4" />
              <UserCheck v-else class="w-4 h-4" />
            </button>
            <button class="p-1.5 rounded-lg border border-transparent hover:bg-blue-50 hover:border-blue-200 text-slate-400 hover:text-blue-600 transition-colors" title="Detail" @click="detailUser = u"><Eye class="w-4 h-4" /></button>
            <button class="p-1.5 rounded-lg border border-transparent hover:bg-slate-100 text-slate-400 hover:text-slate-700 transition-colors" title="Edit" @click="openEdit(u)"><Pencil class="w-4 h-4" /></button>
            <button class="p-1.5 rounded-lg border border-transparent hover:bg-amber-50 hover:border-amber-200 text-slate-400 hover:text-amber-600 transition-colors" title="Reset password" @click="resetPassword(u)"><RefreshCw class="w-4 h-4" /></button>
            <button
              v-if="u.id !== currentUser?.id"
              class="p-1.5 rounded-lg border border-transparent hover:bg-red-50 hover:border-red-200 text-slate-400 hover:text-red-500 transition-colors"
              title="Hapus"
              @click="openDelete(u)"
            ><Trash2 class="w-4 h-4" /></button>
          </div>
        </div>
      </div>
    </Card>

    <UserFormDialog v-model="showForm" :edit-user="editTarget" @saved="onSaved" />
    <ConfirmDialog
      v-model="showDelete"
      title="Hapus akun ini?"
      description="Akun yang sudah membuat soal/sesi tidak bisa dihapus sampai konten tersebut dipindahkan atau dihapus dulu."
      confirm-label="Hapus"
      @confirm="confirmDelete"
    />

    <!-- Detail drawer -->
    <Teleport to="body">
      <div v-if="detailUser" class="fixed inset-0 bg-black/30 flex justify-end z-50" @click="detailUser = null">
        <div class="bg-white w-full max-w-sm h-full overflow-y-auto shadow-2xl border-l" @click.stop>
          <div class="sticky top-0 bg-white border-b p-5 flex items-center justify-between">
            <h3 class="font-black text-slate-900">Detail User</h3>
            <button class="p-1.5 rounded-xl hover:bg-slate-100 text-slate-400" @click="detailUser = null"><X class="w-5 h-5" /></button>
          </div>
          <div class="p-5 space-y-4">
            <div class="p-4 rounded-2xl border flex items-center gap-3" :class="roleConfig[detailUser.role]?.bg">
              <div class="w-12 h-12 rounded-xl flex items-center justify-center bg-white border" :class="roleConfig[detailUser.role]?.bg">
                <component :is="roleConfig[detailUser.role]?.icon" class="w-6 h-6" :class="roleConfig[detailUser.role]?.color" />
              </div>
              <div>
                <p class="font-bold text-slate-900">{{ detailUser.name }}</p>
                <p class="text-xs font-semibold" :class="roleConfig[detailUser.role]?.color">{{ roleConfig[detailUser.role]?.label }}</p>
              </div>
            </div>

            <div class="space-y-2.5">
              <div class="flex items-start justify-between gap-3 py-2 border-b border-slate-50">
                <span class="text-xs font-bold text-slate-400 shrink-0">Email</span>
                <span class="text-sm text-slate-700 text-right">{{ detailUser.email }}</span>
              </div>
              <div class="flex items-start justify-between gap-3 py-2 border-b border-slate-50">
                <span class="text-xs font-bold text-slate-400 shrink-0">Status</span>
                <span class="text-xs px-2 py-0.5 rounded-full border font-semibold flex items-center gap-1 w-fit" :class="statusBadge[detailUser.status]?.class">
                  <span class="w-1.5 h-1.5 rounded-full" :class="statusBadge[detailUser.status]?.dot" />{{ statusBadge[detailUser.status]?.label }}
                </span>
              </div>
              <div v-if="detailUser.school_name" class="flex items-start justify-between gap-3 py-2 border-b border-slate-50">
                <span class="text-xs font-bold text-slate-400 shrink-0">Sekolah</span>
                <span class="text-sm text-slate-700 text-right">{{ detailUser.school_name }}</span>
              </div>
              <div v-if="detailUser.grade" class="flex items-start justify-between gap-3 py-2 border-b border-slate-50">
                <span class="text-xs font-bold text-slate-400 shrink-0">Kelas</span>
                <span class="text-sm text-slate-700 text-right">{{ detailUser.grade }}</span>
              </div>
              <div v-if="detailUser.nisn" class="flex items-start justify-between gap-3 py-2 border-b border-slate-50">
                <span class="text-xs font-bold text-slate-400 shrink-0">NISN</span>
                <code class="text-sm font-mono">{{ detailUser.nisn }}</code>
              </div>
              <div v-if="detailUser.phone" class="flex items-start justify-between gap-3 py-2 border-b border-slate-50">
                <span class="text-xs font-bold text-slate-400 shrink-0">Telepon</span>
                <span class="text-sm text-slate-700 text-right">{{ detailUser.phone }}</span>
              </div>
              <div class="flex items-start justify-between gap-3 py-2 border-b border-slate-50">
                <span class="text-xs font-bold text-slate-400 shrink-0">Dibuat</span>
                <span class="text-sm text-slate-700 text-right">{{ formatDate(detailUser.created_at) }}</span>
              </div>
              <div v-if="detailUser.last_login" class="flex items-start justify-between gap-3 py-2 border-b border-slate-50">
                <span class="text-xs font-bold text-slate-400 shrink-0">Login Terakhir</span>
                <span class="text-sm text-slate-700 text-right">{{ formatDate(detailUser.last_login) }}</span>
              </div>
            </div>

            <!-- Status Pengerjaan — lihat komentar di script: sebelumnya admin sama sekali
                 tidak bisa tahu ini, harus percaya laporan siswa mentah-mentah. -->
            <div v-if="detailUser.role === 'student'" class="p-3 rounded-xl border border-slate-100 bg-slate-50/60">
              <p class="text-xs font-bold text-slate-500 uppercase tracking-wider mb-2 flex items-center gap-1.5"><Clock class="w-3.5 h-3.5" />Status Pengerjaan</p>
              <div v-if="activityLoading" class="text-xs text-slate-400 flex items-center gap-1.5"><Loader2 class="w-3.5 h-3.5 animate-spin" />Memuat...</div>
              <div v-else-if="inProgressAttempts.length === 0 && inProgressRuns.length === 0" class="text-xs text-slate-400">Tidak sedang mengerjakan apa pun saat ini.</div>
              <div v-else class="space-y-1.5">
                <div v-for="a in inProgressAttempts" :key="a.id" class="text-xs bg-amber-50 border border-amber-200 text-amber-700 rounded-lg px-2.5 py-1.5">
                  <span class="font-semibold">{{ a.session_title }}</span> — dimulai {{ formatDate(a.started_at) }}
                </div>
                <div v-for="r in inProgressRuns" :key="r.id" class="text-xs bg-indigo-50 border border-indigo-200 text-indigo-700 rounded-lg px-2.5 py-1.5">
                  <span class="font-semibold">{{ r.template_title }}</span> — subtes {{ r.current_sequence_index }}/{{ r.slots.length }}, dimulai {{ formatDate(r.started_at) }}
                  <span v-if="r.lockdown_enabled" class="ml-1.5 inline-flex items-center gap-1 text-[10px] font-bold px-1.5 py-0.5 rounded-full" :class="(runViolationCounts[r.id] ?? 0) > 0 ? 'bg-rose-100 text-rose-700' : 'bg-slate-100 text-slate-500'">
                    Terkunci{{ (runViolationCounts[r.id] ?? 0) > 0 ? ` — ${runViolationCounts[r.id]} pelanggaran` : '' }}
                  </span>
                </div>
              </div>
            </div>

            <div class="space-y-2 pt-2">
              <button class="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-indigo-50 hover:bg-indigo-100 text-indigo-700 text-sm font-semibold border border-indigo-200 transition-colors" @click="openEdit(detailUser); detailUser = null">
                <Pencil class="w-4 h-4" />Edit User
              </button>
              <button class="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-amber-50 hover:bg-amber-100 text-amber-700 text-sm font-semibold border border-amber-200 transition-colors" @click="resetPassword(detailUser)">
                <RefreshCw class="w-4 h-4" />Reset Password
              </button>
              <button
                v-if="detailUser.id !== currentUser?.id"
                class="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-slate-50 hover:bg-slate-100 text-slate-700 text-sm font-semibold border border-slate-200 transition-colors"
                @click="toggleStatus(detailUser)"
              >
                <UserX v-if="detailUser.status === 'active'" class="w-4 h-4" /><UserCheck v-else class="w-4 h-4" />
                {{ detailUser.status === 'active' ? 'Nonaktifkan' : 'Aktifkan' }}
              </button>
              <button
                v-if="detailUser.id !== currentUser?.id"
                class="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-red-50 hover:bg-red-100 text-red-600 text-sm font-semibold border border-red-200 transition-colors"
                @click="openDelete(detailUser)"
              >
                <Trash2 class="w-4 h-4" />Hapus User
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
