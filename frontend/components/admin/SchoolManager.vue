<script setup lang="ts">
import { Search, Pencil, Trash2, Plus, School as SchoolIcon, CheckCircle2, Clock, XCircle, Users as UsersIcon, DollarSign, UserPlus, Upload, Eye, X, Award, Tag, AlertTriangle, GraduationCap } from 'lucide-vue-next'
import type { PackageType, School, SchoolStatus, User, PackageItem, SchoolEntitlementItem } from '~/types'

const { schoolService, userService, analyticsService, packageService, entitlementService } = useApi()
const toast = useToast()

const schools = ref<School[]>([])
const students = ref<User[]>([])
const rankings = ref<{ school_id: string; average_score: number }[]>([])
const loading = ref(false)
const search = ref('')
const filterStatus = ref<SchoolStatus | ''>('')
const filterPkg = ref<PackageType | ''>('')

const showForm = ref(false)
const editTarget = ref<School | null>(null)
const showDelete = ref(false)
const deleteTarget = ref<School | null>(null)
const detailSchool = ref<School | null>(null)

const showWizard = ref(false)
const wizardSchool = ref<School | null>(null)
const wizardMode = ref<'approve' | 'import'>('approve')

const typeLabel: Record<string, string> = { smp: 'SMP', sma: 'SMA', smk: 'SMK', ma: 'MA' }
const packageBadge: Record<string, { label: string; variant: any }> = {
  basic: { label: 'Basic', variant: 'secondary' },
  premium: { label: 'Premium', variant: 'warning' },
  enterprise: { label: 'Enterprise', variant: 'success' },
}
const statusConfig: Record<SchoolStatus, { label: string; icon: any; class: string }> = {
  active: { label: 'Aktif', icon: CheckCircle2, class: 'bg-green-100 text-green-700 border-green-300' },
  inactive: { label: 'Tidak Aktif', icon: XCircle, class: 'bg-red-100 text-red-700 border-red-300' },
  pending: { label: 'Menunggu Persetujuan', icon: Clock, class: 'bg-amber-100 text-amber-700 border-amber-300' },
}

const filtered = computed(() => {
  let list = schools.value
  if (search.value.trim()) {
    const q = search.value.toLowerCase()
    list = list.filter((s) => s.name.toLowerCase().includes(q) || s.city.toLowerCase().includes(q))
  }
  if (filterStatus.value) list = list.filter((s) => s.status === filterStatus.value)
  if (filterPkg.value) list = list.filter((s) => s.package_type === filterPkg.value)
  return list
})

const stats = computed(() => ({
  total: schools.value.length,
  active: schools.value.filter((s) => s.status === 'active').length,
  pending: schools.value.filter((s) => s.status === 'pending').length,
  totalStudents: students.value.length,
  revenue: schools.value.reduce((sum, s) => sum + Math.round((s.monthly_revenue * s.revenue_share) / 100), 0),
}))

function studentsOf(schoolId: string) {
  return students.value.filter((u) => u.school_id === schoolId)
}
function activeStudentsOf(schoolId: string) {
  return studentsOf(schoolId).filter((u) => u.status === 'active').length
}
function avgScoreOf(schoolId: string) {
  return rankings.value.find((r) => r.school_id === schoolId)?.average_score
}

async function load() {
  loading.value = true
  try {
    const [sc, us, top] = await Promise.all([
      schoolService.list(),
      userService.list({ role: 'student' }),
      analyticsService.topSchools(50),
    ])
    schools.value = sc
    students.value = us
    rankings.value = top
  } catch (e: any) {
    toast.error('Gagal memuat daftar sekolah', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

function openCreate() {
  editTarget.value = null
  showForm.value = true
}
function openEdit(s: School) {
  editTarget.value = s
  showForm.value = true
}
function openDelete(s: School) {
  deleteTarget.value = s
  showDelete.value = true
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  try {
    await schoolService.remove(deleteTarget.value.id)
    toast.success('Sekolah berhasil dihapus!')
    detailSchool.value = null
    load()
  } catch (e: any) {
    toast.error('Gagal menghapus sekolah', e?.message)
  }
}

function openWizard(s: School, mode: 'approve' | 'import') {
  wizardSchool.value = s
  wizardMode.value = mode
  showWizard.value = true
}

async function deactivate(s: School) {
  try {
    await schoolService.setStatus(s.id, 'inactive')
    toast.success('Mitra dinonaktifkan')
    load()
  } catch (e: any) {
    toast.error('Gagal menonaktifkan mitra', e?.message)
  }
}
async function reactivate(s: School) {
  try {
    await schoolService.setStatus(s.id, 'active')
    toast.success('Mitra diaktifkan kembali')
    load()
  } catch (e: any) {
    toast.error('Gagal mengaktifkan mitra', e?.message)
  }
}

// ─── Paket Terpasang (School Package Entitlement) ────────────────────────────────
const packages = ref<PackageItem[]>([])
const entitlements = ref<SchoolEntitlementItem[]>([])
const entitlementsLoading = ref(false)

function todayStr() {
  return new Date().toISOString().slice(0, 10)
}

const entForm = ref({
  package_id: '',
  starts_at: todayStr(),
  expires_at: '',
  unlimited: true,
  note: '',
})
function resetEntForm() {
  entForm.value = { package_id: '', starts_at: todayStr(), expires_at: '', unlimited: true, note: '' }
}

// Jumlah konten (TryoutSession/SimulationTemplate) per paket — supaya admin langsung lihat
// kalau sebuah paket masih KOSONG sebelum disematkan ke sekolah. Paket kosong = sekolah/siswa
// pemiliknya tidak akan bisa membuka konten premium apa pun (lihat AccessService: akses
// sekarang per-konten, bukan lagi all-or-nothing) — ini penyebab paling umum keluhan "paket
// premium sudah di-assign tapi tidak bisa dibuka".
const packageContentCounts = ref<Record<string, number>>({})
async function loadPackageContentCounts() {
  const results = await Promise.all(
    packages.value.map(async (p) => {
      try {
        const items = await packageService.listContent(p.id)
        return [p.id, items.length] as const
      } catch {
        return [p.id, -1] as const // -1 = gagal memuat, jangan tampilkan sebagai "kosong"
      }
    }),
  )
  packageContentCounts.value = Object.fromEntries(results)
}
function contentCountLabel(packageId: string) {
  const n = packageContentCounts.value[packageId]
  if (n === undefined) return ''
  if (n < 0) return ''
  return n === 0 ? ' — KOSONG' : ` — ${n} konten`
}
function isPackageEmpty(packageId: string) {
  return packageContentCounts.value[packageId] === 0
}

async function loadPackages() {
  try {
    packages.value = await packageService.list()
    await loadPackageContentCounts()
  } catch (e: any) {
    toast.error('Gagal memuat daftar paket', e?.message)
  }
}
onMounted(loadPackages)

async function loadEntitlements(schoolId: string) {
  entitlementsLoading.value = true
  try {
    entitlements.value = await entitlementService.listForSchool(schoolId)
  } catch (e: any) {
    toast.error('Gagal memuat paket terpasang', e?.message)
  } finally {
    entitlementsLoading.value = false
  }
}
watch(detailSchool, (s) => {
  if (s) {
    resetEntForm()
    loadEntitlements(s.id)
  } else {
    entitlements.value = []
  }
})

async function createEntitlement() {
  if (!detailSchool.value || !entForm.value.package_id) return
  try {
    await entitlementService.create({
      school_id: detailSchool.value.id,
      package_id: entForm.value.package_id,
      starts_at: entForm.value.starts_at || undefined,
      expires_at: entForm.value.unlimited ? null : entForm.value.expires_at || null,
      note: entForm.value.note || null,
    })
    toast.success('Paket berhasil disematkan ke sekolah')
    resetEntForm()
    await loadEntitlements(detailSchool.value.id)
  } catch (e: any) {
    toast.error('Gagal menyematkan paket', e?.message)
  }
}

async function removeEntitlement(item: SchoolEntitlementItem) {
  if (!window.confirm(`Hapus paket "${item.package_name}" dari sekolah ini?`)) return
  try {
    await entitlementService.remove(item.id)
    toast.success('Paket berhasil dilepas dari sekolah')
    if (detailSchool.value) await loadEntitlements(detailSchool.value.id)
  } catch (e: any) {
    toast.error('Gagal menghapus paket', e?.message)
  }
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2"><SchoolIcon class="w-5 h-5 text-indigo-600" />Sekolah Mitra</h2>
        <p class="text-sm text-muted-foreground">{{ schools.length }} sekolah terdaftar</p>
      </div>
      <Button variant="gradient" @click="openCreate"><Plus class="w-4 h-4" />Tambah Sekolah</Button>
    </div>

    <div v-if="stats.pending > 0" class="bg-amber-50 border border-amber-200 rounded-xl p-4 flex items-center gap-3 flex-wrap">
      <Clock class="w-5 h-5 text-amber-600 shrink-0" />
      <p class="text-sm text-amber-800">
        <span class="font-bold">{{ stats.pending }} sekolah</span> menunggu persetujuan. Klik <strong>"Setujui & Onboarding"</strong> untuk memulai wizard import siswa.
      </p>
      <button class="ml-auto text-xs font-bold text-amber-700 underline hover:text-amber-900 shrink-0" @click="filterStatus = 'pending'">Lihat Sekarang</button>
    </div>

    <div class="grid grid-cols-2 lg:grid-cols-5 gap-3">
      <Card class="p-4 border-l-4 border-l-green-500">
        <div class="flex items-center justify-between">
          <div><p class="text-xs text-muted-foreground mb-1">Total Mitra</p><h3 class="text-xl font-bold">{{ stats.total }}</h3></div>
          <div class="w-9 h-9 rounded-lg bg-green-100 flex items-center justify-center"><SchoolIcon class="w-4 h-4 text-green-600" /></div>
        </div>
      </Card>
      <Card class="p-4 border-l-4 border-l-blue-500">
        <div class="flex items-center justify-between">
          <div><p class="text-xs text-muted-foreground mb-1">Aktif</p><h3 class="text-xl font-bold">{{ stats.active }}</h3></div>
          <div class="w-9 h-9 rounded-lg bg-blue-100 flex items-center justify-center"><CheckCircle2 class="w-4 h-4 text-blue-600" /></div>
        </div>
      </Card>
      <Card class="p-4 border-l-4 border-l-amber-500">
        <div class="flex items-center justify-between">
          <div><p class="text-xs text-muted-foreground mb-1">Menunggu</p><h3 class="text-xl font-bold">{{ stats.pending }}</h3></div>
          <div class="w-9 h-9 rounded-lg bg-amber-100 flex items-center justify-center"><Clock class="w-4 h-4 text-amber-600" /></div>
        </div>
      </Card>
      <Card class="p-4 border-l-4 border-l-purple-500">
        <div class="flex items-center justify-between">
          <div><p class="text-xs text-muted-foreground mb-1">Total Siswa</p><h3 class="text-xl font-bold">{{ stats.totalStudents.toLocaleString('id-ID') }}</h3></div>
          <div class="w-9 h-9 rounded-lg bg-purple-100 flex items-center justify-center"><UsersIcon class="w-4 h-4 text-purple-600" /></div>
        </div>
      </Card>
      <Card class="p-4 border-l-4 border-l-rose-500">
        <div class="flex items-center justify-between">
          <div><p class="text-xs text-muted-foreground mb-1">Rev. B2B/bulan</p><h3 class="text-xl font-bold">Rp {{ (stats.revenue / 1e6).toFixed(1) }}jt</h3></div>
          <div class="w-9 h-9 rounded-lg bg-rose-100 flex items-center justify-center"><DollarSign class="w-4 h-4 text-rose-600" /></div>
        </div>
      </Card>
    </div>

    <Card class="p-4">
      <div class="flex flex-col sm:flex-row gap-3">
        <div class="relative flex-1">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input v-model="search" placeholder="Cari nama sekolah atau kota..." class="w-full pl-9 pr-3 py-2 border rounded-lg text-sm bg-white" />
        </div>
        <select v-model="filterStatus" class="px-3 py-2 border rounded-xl text-sm bg-white">
          <option value="">Semua Status</option>
          <option value="active">Aktif</option>
          <option value="inactive">Tidak Aktif</option>
          <option value="pending">Menunggu</option>
        </select>
        <select v-model="filterPkg" class="px-3 py-2 border rounded-xl text-sm bg-white">
          <option value="">Semua Paket</option>
          <option value="basic">Basic</option>
          <option value="premium">Premium</option>
          <option value="enterprise">Enterprise</option>
        </select>
      </div>
    </Card>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
    <Card v-else-if="filtered.length === 0" class="p-10 text-center text-sm text-muted-foreground">Belum ada sekolah mitra yang cocok.</Card>
    <div v-else class="grid lg:grid-cols-2 gap-5">
      <Card v-for="s in filtered" :key="s.id" class="p-5" :class="s.status === 'pending' ? 'border-amber-300 border-2' : ''">
        <div class="flex items-start justify-between mb-4">
          <div class="flex items-start gap-3">
            <div class="w-11 h-11 rounded-xl bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center text-white shrink-0">
              <SchoolIcon class="w-5 h-5" />
            </div>
            <div>
              <h3 class="font-bold text-slate-900 leading-tight">{{ s.name }}</h3>
              <p class="text-xs text-muted-foreground mt-0.5">{{ typeLabel[s.school_type] }} &middot; {{ s.city }}, {{ s.province }}</p>
            </div>
          </div>
          <div class="flex flex-col items-end gap-1.5">
            <Badge variant="outline" class="text-xs gap-1" :class="statusConfig[s.status].class">
              <component :is="statusConfig[s.status].icon" class="w-3 h-3" />{{ statusConfig[s.status].label }}
            </Badge>
            <Badge :variant="packageBadge[s.package_type]?.variant">{{ packageBadge[s.package_type]?.label }}</Badge>
          </div>
        </div>

        <div class="grid grid-cols-3 gap-2 mb-4">
          <div class="p-2.5 bg-slate-50 rounded-lg text-center">
            <p class="font-bold text-sm text-slate-900">{{ studentsOf(s.id).length }}</p>
            <p class="text-[10px] text-muted-foreground mt-0.5">Total Siswa</p>
          </div>
          <div class="p-2.5 bg-slate-50 rounded-lg text-center">
            <p class="font-bold text-sm text-green-600">{{ activeStudentsOf(s.id) }}</p>
            <p class="text-[10px] text-muted-foreground mt-0.5">Aktif</p>
          </div>
          <div class="p-2.5 bg-slate-50 rounded-lg text-center">
            <p class="font-bold text-sm text-slate-900">{{ avgScoreOf(s.id) !== undefined ? Math.round(avgScoreOf(s.id)!) : '—' }}</p>
            <p class="text-[10px] text-muted-foreground mt-0.5">Avg Score</p>
          </div>
        </div>

        <div v-if="s.status === 'active' && studentsOf(s.id).length === 0" class="flex items-center gap-2 p-2.5 bg-indigo-50 border border-indigo-200 rounded-lg mb-3">
          <UserPlus class="w-4 h-4 text-indigo-600 shrink-0" />
          <p class="text-xs text-indigo-800 font-medium flex-1">Belum ada siswa diimport</p>
          <button class="text-xs text-indigo-600 font-bold underline" @click="openWizard(s, 'import')">Import Sekarang</button>
        </div>

        <div v-if="s.status === 'active' && s.monthly_revenue > 0" class="flex items-center justify-between p-3 bg-gradient-to-r from-green-50 to-emerald-50 border border-green-200 rounded-lg mb-4">
          <div class="flex items-center gap-2 text-xs text-green-700">
            <DollarSign class="w-3.5 h-3.5" /><span>Revenue share ({{ s.revenue_share }}%)</span>
          </div>
          <span class="text-sm font-black text-green-700">Rp {{ Math.round(s.monthly_revenue * s.revenue_share / 100).toLocaleString('id-ID') }}/bln</span>
        </div>

        <div class="text-xs text-slate-500 space-y-0.5 mb-4">
          <p>{{ s.contact_person }}</p>
          <p>{{ s.email }}</p>
          <p>{{ s.phone }}</p>
        </div>

        <div class="flex items-center justify-between pt-3 border-t gap-2 flex-wrap">
          <p class="text-xs text-muted-foreground shrink-0">Sejak {{ s.join_date }}</p>
          <div class="flex gap-1.5 flex-wrap justify-end">
            <template v-if="s.status === 'pending'">
              <Button size="sm" class="gap-1.5 bg-gradient-to-r from-indigo-600 to-purple-600 text-xs" @click="openWizard(s, 'approve')">
                <UserPlus class="w-3.5 h-3.5" />Setujui & Onboarding
              </Button>
              <Button size="sm" variant="outline" class="gap-1.5 text-red-600 border-red-200 text-xs" @click="openDelete(s)">
                <XCircle class="w-3.5 h-3.5" />Tolak
              </Button>
            </template>
            <template v-else-if="s.status === 'active'">
              <Button size="sm" variant="outline" class="gap-1.5 text-xs" @click="openWizard(s, 'import')">
                <Upload class="w-3.5 h-3.5" />Import Siswa
              </Button>
              <Button size="sm" variant="outline" class="text-xs" @click="deactivate(s)">Nonaktifkan</Button>
            </template>
            <template v-else>
              <Button size="sm" variant="outline" class="gap-1.5 text-green-700 border-green-300 text-xs" @click="reactivate(s)">
                <CheckCircle2 class="w-3.5 h-3.5" />Aktifkan
              </Button>
            </template>
            <button class="p-1.5 rounded hover:bg-slate-100" title="Detail" @click="detailSchool = s"><Eye class="w-4 h-4 text-slate-500" /></button>
            <button class="p-1.5 rounded hover:bg-slate-100" title="Edit" @click="openEdit(s)"><Pencil class="w-4 h-4 text-slate-500" /></button>
            <button class="p-1.5 rounded hover:bg-red-50" title="Hapus" @click="openDelete(s)"><Trash2 class="w-4 h-4 text-red-500" /></button>
          </div>
        </div>
      </Card>
    </div>

    <div class="text-center text-sm text-muted-foreground">Menampilkan {{ filtered.length }} dari {{ schools.length }} mitra</div>

    <SchoolFormDialog v-model="showForm" :edit-school="editTarget" @saved="load" />
    <SchoolOnboardingWizard v-if="wizardSchool" v-model="showWizard" :school="wizardSchool" :mode="wizardMode" @saved="load" />
    <ConfirmDialog
      v-model="showDelete"
      title="Hapus sekolah ini?"
      description="Akun sekolah/siswa yang terkait akan kehilangan afiliasi sekolahnya, bukan ikut terhapus."
      confirm-label="Hapus"
      @confirm="confirmDelete"
    />

    <!-- Detail drawer -->
    <Teleport to="body">
      <div v-if="detailSchool" class="fixed inset-0 bg-black/40 flex items-center justify-end z-50 p-4" @click="detailSchool = null">
        <div class="bg-white h-full w-full max-w-sm rounded-2xl shadow-2xl overflow-y-auto" @click.stop>
          <div class="flex items-center justify-between p-5 border-b">
            <h3 class="font-bold text-slate-900">Detail Mitra</h3>
            <button class="p-2 rounded-xl hover:bg-slate-100" @click="detailSchool = null"><X class="w-4 h-4" /></button>
          </div>
          <div class="p-5">
            <div class="flex items-center gap-3 mb-5">
              <div class="w-14 h-14 rounded-xl bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center text-white">
                <SchoolIcon class="w-7 h-7" />
              </div>
              <div>
                <h4 class="font-bold text-slate-900 text-sm">{{ detailSchool.name }}</h4>
                <p class="text-xs text-muted-foreground">{{ typeLabel[detailSchool.school_type] }} &middot; {{ detailSchool.city }}</p>
              </div>
            </div>

            <div class="grid grid-cols-3 gap-3 mb-5">
              <div class="bg-slate-50 rounded-xl p-3">
                <UsersIcon class="w-4 h-4 text-blue-500 mb-1.5" />
                <p class="text-lg font-black text-slate-900">{{ studentsOf(detailSchool.id).length }}</p>
                <p class="text-[10px] text-muted-foreground">Total Siswa</p>
              </div>
              <div class="bg-slate-50 rounded-xl p-3">
                <CheckCircle2 class="w-4 h-4 text-emerald-500 mb-1.5" />
                <p class="text-lg font-black text-slate-900">{{ activeStudentsOf(detailSchool.id) }}</p>
                <p class="text-[10px] text-muted-foreground">Siswa Aktif</p>
              </div>
              <div class="bg-slate-50 rounded-xl p-3">
                <Award class="w-4 h-4 text-amber-500 mb-1.5" />
                <p class="text-lg font-black text-slate-900">{{ avgScoreOf(detailSchool.id) !== undefined ? Math.round(avgScoreOf(detailSchool.id)!) : '—' }}</p>
                <p class="text-[10px] text-muted-foreground">Avg Score</p>
              </div>
            </div>

            <div class="space-y-3 mb-5">
              <div
                v-for="[label, val] in [
                  ['Status', statusConfig[detailSchool.status].label],
                  ['Paket', packageBadge[detailSchool.package_type]?.label],
                  ['Revenue Share', `${detailSchool.revenue_share}%`],
                  ['Nilai Kontrak/bln', detailSchool.monthly_revenue > 0 ? `Rp ${detailSchool.monthly_revenue.toLocaleString('id-ID')}` : '—'],
                  ['Bagian Sekolah/bln', detailSchool.monthly_revenue > 0 ? `Rp ${Math.round(detailSchool.monthly_revenue * detailSchool.revenue_share / 100).toLocaleString('id-ID')}` : '—'],
                  ['PIC', detailSchool.contact_person],
                  ['Email', detailSchool.email],
                  ['Telepon', detailSchool.phone],
                  ['Bergabung', detailSchool.join_date],
                ]"
                :key="label"
                class="flex justify-between py-2 border-b last:border-0"
              >
                <span class="text-xs text-muted-foreground">{{ label }}</span>
                <span class="text-xs font-semibold text-slate-800 text-right max-w-[55%]">{{ val }}</span>
              </div>
            </div>

            <div class="mb-5">
              <h5 class="text-xs font-bold text-slate-700 uppercase tracking-wide mb-2.5 flex items-center gap-1.5"><Tag class="w-3.5 h-3.5 text-indigo-500" />Paket Terpasang</h5>

              <div class="p-3 bg-slate-50 rounded-xl mb-3 space-y-2">
                <select v-model="entForm.package_id" class="w-full px-2.5 py-1.5 border rounded-lg text-xs bg-white">
                  <option value="" disabled>Pilih paket...</option>
                  <option v-for="p in packages" :key="p.id" :value="p.id">{{ p.name }}{{ contentCountLabel(p.id) }}</option>
                </select>
                <div v-if="entForm.package_id && isPackageEmpty(entForm.package_id)" class="flex items-start gap-1.5 p-2 bg-red-50 border border-red-200 rounded-lg text-[11px] text-red-700">
                  <AlertTriangle class="w-3.5 h-3.5 shrink-0 mt-0.5" />
                  <span>Paket ini belum ada isinya (belum di-assign Sesi/Simulasi TO apa pun). Sekolah tetap bisa memilikinya, tapi siswa TIDAK akan bisa membuka konten premium apa pun sampai kamu isi paket ini lewat Manajemen Paket &rarr; "Isi Paket".</span>
                </div>
                <div class="grid grid-cols-2 gap-2">
                  <div>
                    <label class="text-[10px] text-muted-foreground block mb-0.5">Berlaku Mulai</label>
                    <input v-model="entForm.starts_at" type="date" class="w-full px-2 py-1.5 border rounded-lg text-xs bg-white" />
                  </div>
                  <div>
                    <label class="text-[10px] text-muted-foreground block mb-0.5">Berlaku Sampai</label>
                    <input v-model="entForm.expires_at" type="date" :disabled="entForm.unlimited" class="w-full px-2 py-1.5 border rounded-lg text-xs bg-white disabled:bg-slate-100 disabled:text-slate-400" />
                  </div>
                </div>
                <label class="flex items-center gap-1.5 text-[11px] text-slate-600">
                  <input v-model="entForm.unlimited" type="checkbox" class="rounded" />Tanpa batas waktu
                </label>
                <input v-model="entForm.note" placeholder="Catatan (opsional)" class="w-full px-2.5 py-1.5 border rounded-lg text-xs bg-white" />
                <Button size="sm" class="w-full text-xs" :disabled="!entForm.package_id" @click="createEntitlement">Sematkan Paket</Button>
              </div>

              <div v-if="entitlementsLoading" class="text-center text-xs text-muted-foreground py-3">Memuat...</div>
              <div v-else-if="entitlements.length === 0" class="text-center text-xs text-muted-foreground py-3">Belum ada paket premium terpasang dari admin.</div>
              <div v-else class="space-y-2">
                <div v-for="item in entitlements" :key="item.id" class="p-2.5 border rounded-lg">
                  <div class="flex items-start justify-between gap-2">
                    <div class="min-w-0">
                      <p class="text-xs font-bold text-slate-800 truncate">{{ item.package_name }}</p>
                      <p class="text-[10px] text-muted-foreground">{{ item.starts_at }} &ndash; {{ item.expires_at ?? 'Tanpa batas waktu' }}</p>
                      <p v-if="item.note" class="text-[10px] text-muted-foreground italic mt-0.5">{{ item.note }}</p>
                      <p v-if="isPackageEmpty(item.package_id)" class="text-[10px] text-red-600 font-semibold mt-0.5 flex items-center gap-1"><AlertTriangle class="w-3 h-3" />Paket kosong — belum bisa dibuka siswa</p>
                    </div>
                    <div class="flex items-center gap-1.5 shrink-0">
                      <Badge :variant="item.is_active ? 'success' : 'secondary'" class="text-[10px]">{{ item.is_active ? 'Aktif' : 'Tidak Aktif' }}</Badge>
                      <button class="p-1 rounded hover:bg-red-50" title="Hapus" @click="removeEntitlement(item)"><Trash2 class="w-3.5 h-3.5 text-red-500" /></button>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div class="mb-5">
              <h5 class="text-xs font-bold text-slate-700 uppercase tracking-wide mb-2.5 flex items-center gap-1.5"><GraduationCap class="w-3.5 h-3.5 text-indigo-500" />Daftar Siswa ({{ studentsOf(detailSchool.id).length }})</h5>
              <div v-if="studentsOf(detailSchool.id).length === 0" class="text-center text-xs text-muted-foreground py-3 border border-dashed rounded-lg">
                Belum ada siswa terdaftar di sekolah ini.
              </div>
              <div v-else class="space-y-1.5 max-h-56 overflow-y-auto pr-1">
                <div v-for="u in studentsOf(detailSchool.id)" :key="u.id" class="flex items-center justify-between gap-2 p-2 border rounded-lg">
                  <div class="min-w-0">
                    <p class="text-xs font-semibold text-slate-800 truncate">{{ u.name }}</p>
                    <p class="text-[10px] text-muted-foreground truncate">{{ u.email }}{{ u.grade ? ` · ${u.grade}` : '' }}</p>
                  </div>
                  <Badge :variant="u.status === 'active' ? 'success' : 'secondary'" class="text-[10px] shrink-0">{{ u.status === 'active' ? 'Aktif' : u.status }}</Badge>
                </div>
              </div>
            </div>

            <div class="flex flex-col gap-2">
              <div class="flex gap-2">
                <Button variant="outline" size="sm" class="flex-1 gap-1.5" @click="openEdit(detailSchool); detailSchool = null"><Pencil class="w-3.5 h-3.5" />Edit</Button>
                <Button v-if="detailSchool.status === 'active'" size="sm" class="flex-1 gap-1.5 bg-indigo-600 hover:bg-indigo-700" @click="openWizard(detailSchool, 'import'); detailSchool = null">
                  <Upload class="w-3.5 h-3.5" />Import Siswa
                </Button>
              </div>
              <Button v-if="detailSchool.status === 'pending'" size="sm" class="w-full bg-gradient-to-r from-indigo-600 to-purple-600 gap-1.5" @click="openWizard(detailSchool, 'approve'); detailSchool = null">
                <UserPlus class="w-3.5 h-3.5" />Setujui & Onboarding Siswa
              </Button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
