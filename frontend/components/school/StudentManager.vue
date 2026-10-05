<script setup lang="ts">
// Manajemen Siswa (Portal Sekolah) — roster dari /api/users (di-scope otomatis ke
// role=student, school_id=sekolah ini oleh UserService di backend), digabung dengan
// statistik nyata per siswa dari schoolService.rosterStats() (skor/tryout/target PTN/
// terakhir aktif, dihitung live dari attempts & student_ptn_targets — lihat
// SchoolPortalService::roster_stats). Hard-delete sengaja tidak ada di sini: backend
// membatasi delete akun hanya untuk Admin (agar riwayat ujian siswa tidak hilang tak
// sengaja) — sekolah menonaktifkan siswa lewat toggle status, bukan menghapus.
// Pencarian/filter kelas/status semuanya client-side atas roster penuh yang sudah
// di-fetch — ini sengaja (bukan search-by-name saja ke server) supaya bisa mencari
// berdasarkan NISN/kelas/jumlah target PTN sekaligus, sesuai referensi.
import {
  Search, Plus, Pencil, UserCheck, UserX, RefreshCw, Users, Eye, EyeOff, Sparkles, ChevronUp, ChevronDown,
  Download, Upload, X, Target, Mail, Phone, Hash, GraduationCap, CheckCircle2, AlertCircle, TrendingUp,
  Copy, FileDown, Key, FileSpreadsheet, ArrowLeft,
} from 'lucide-vue-next'
import type { User, UserStatus, StudentRosterStat, TargetWithChanceItem } from '~/types'
import { parseStudentImportWorkbook, downloadStudentImportTemplate, importedRowToCreatePayload, type ImportedStudentRow } from '~/utils/studentImportXlsx'

const { userService, schoolService, rationalizationService } = useApi()
const toast = useToast()

const students = ref<User[]>([])
const stats = ref<StudentRosterStat[]>([])
const loading = ref(false)
const search = ref('')
const filterStatus = ref<UserStatus | ''>('')
const filterGrade = ref('')

async function load() {
  loading.value = true
  try {
    const [s, st] = await Promise.all([userService.list(), schoolService.rosterStats()])
    students.value = s
    stats.value = st
  } catch (e: any) {
    toast.error('Gagal memuat daftar siswa', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

const statMap = computed(() => new Map(stats.value.map((s) => [s.student_id, s])))
const GRADES = ['Kelas 10', 'Kelas 11', 'Kelas 12']

const activeCount = computed(() => students.value.filter((s) => s.status === 'active').length)
const avgScoreAll = computed(() => {
  const withScore = stats.value.filter((s) => s.avg_score != null)
  if (!withScore.length) return null
  return Math.round(withScore.reduce((sum, s) => sum + (s.avg_score ?? 0), 0) / withScore.length)
})
const gradeCounts = computed(() => {
  const map: Record<string, number> = {}
  for (const s of students.value) { const g = s.grade || '-'; map[g] = (map[g] || 0) + 1 }
  return map
})

// ─── Filter + sort ────────────────────────────────────────────────────────────────
type SortKey = 'name' | 'avg_score' | 'tryouts'
const sortKey = ref<SortKey>('name')
const sortDesc = ref(false)
function toggleSort(key: SortKey) {
  if (sortKey.value === key) sortDesc.value = !sortDesc.value
  else { sortKey.value = key; sortDesc.value = key !== 'name' }
}

const filteredStudents = computed(() => {
  let list = students.value
  if (filterGrade.value) list = list.filter((s) => s.grade === filterGrade.value)
  if (filterStatus.value) list = list.filter((s) => s.status === filterStatus.value)
  const q = search.value.trim().toLowerCase()
  if (q) {
    list = list.filter((s) => {
      const st = statMap.value.get(s.id)
      return (
        s.name.toLowerCase().includes(q) ||
        s.email.toLowerCase().includes(q) ||
        (s.nisn || '').toLowerCase().includes(q) ||
        (s.grade || '').toLowerCase().includes(q) ||
        (st && st.target_ptn_count > 0 && (q.includes('target') || q.includes('ptn')))
      )
    })
  }
  list = [...list].sort((a, b) => {
    let cmp = 0
    if (sortKey.value === 'name') cmp = a.name.localeCompare(b.name)
    else if (sortKey.value === 'avg_score') cmp = (statMap.value.get(a.id)?.avg_score ?? -1) - (statMap.value.get(b.id)?.avg_score ?? -1)
    else if (sortKey.value === 'tryouts') cmp = (statMap.value.get(a.id)?.tryouts_completed ?? 0) - (statMap.value.get(b.id)?.tryouts_completed ?? 0)
    return sortDesc.value ? -cmp : cmp
  })
  return list
})

function gradeBadge(avg: number | null | undefined) {
  if (avg == null) return { label: 'Belum Tes', class: 'bg-slate-100 text-slate-500' }
  if (avg >= 700) return { label: 'Unggul', class: 'bg-emerald-100 text-emerald-700' }
  if (avg >= 600) return { label: 'Baik', class: 'bg-blue-100 text-blue-700' }
  if (avg >= 500) return { label: 'Cukup', class: 'bg-amber-100 text-amber-700' }
  return { label: 'Perlu Bimbingan', class: 'bg-red-100 text-red-700' }
}
function initials(name: string) {
  return name.slice(0, 2).toUpperCase()
}
function formatDate(iso: string | null | undefined) {
  if (!iso) return null
  return new Date(iso).toLocaleDateString('id-ID', { day: 'numeric', month: 'short', year: 'numeric' })
}

// ─── Export CSV (nyata, dari data yang sedang tampil) ─────────────────────────────
function exportCsv() {
  const header = 'Nama,Email,Kelas,NISN,Status,Rata-rata,Tryout Selesai,Target PTN\n'
  const rows = filteredStudents.value.map((s) => {
    const st = statMap.value.get(s.id)
    return [s.name, s.email, s.grade || '', s.nisn || '', s.status, st?.avg_score ?? '', st?.tryouts_completed ?? 0, st?.target_ptn_count ?? 0]
      .map((v) => `"${String(v).replace(/"/g, '""')}"`).join(',')
  })
  const blob = new Blob([header + rows.join('\n')], { type: 'text/csv' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url; a.download = 'siswa.csv'; a.click()
  URL.revokeObjectURL(url)
}

// ─── Detail drawer ─────────────────────────────────────────────────────────────────
const detailStudent = ref<User | null>(null)
const detailTargets = ref<TargetWithChanceItem[]>([])
const detailLoading = ref(false)
async function openDetail(s: User) {
  detailStudent.value = s
  detailTargets.value = []
  detailLoading.value = true
  try {
    detailTargets.value = await rationalizationService.studentTargets(s.id)
  } catch {
    // non-critical for the drawer
  } finally {
    detailLoading.value = false
  }
}

// ─── Create/edit form ──────────────────────────────────────────────────────────
const showForm = ref(false)
const editTarget = ref<User | null>(null)
const name = ref('')
const email = ref('')
const nisn = ref('')
const grade = ref('Kelas 12')
const status = ref<UserStatus>('active')
const password = ref('')
const showPassword = ref(false)
const saving = ref(false)

function genPassword(seed: string) {
  const part = (seed || 'siswa').replace(/\s+/g, '').slice(0, 4).toLowerCase() || 'siswa'
  return `Gspl${part}${Math.floor(1000 + Math.random() * 9000)}!`
}

function openCreate() {
  editTarget.value = null
  name.value = ''; email.value = ''; nisn.value = ''; grade.value = 'Kelas 12'; status.value = 'active'
  password.value = genPassword('siswa')
  showForm.value = true
}
function openEdit(u: User) {
  editTarget.value = u
  name.value = u.name; email.value = u.email; nisn.value = u.nisn || ''; grade.value = u.grade || 'Kelas 12'; status.value = u.status
  password.value = ''
  showForm.value = true
}

async function save() {
  if (!name.value.trim()) return toast.error('Nama tidak boleh kosong!')
  if (!email.value.trim()) return toast.error('Email tidak boleh kosong!')
  saving.value = true
  try {
    if (editTarget.value) {
      await userService.update(editTarget.value.id, {
        name: name.value, email: email.value, role: 'student', school_id: editTarget.value.school_id,
        status: status.value, nisn: nisn.value || null, grade: grade.value, password: password.value || undefined,
      })
      toast.success('Data siswa diperbarui')
      if (detailStudent.value?.id === editTarget.value.id) detailStudent.value = null
    } else {
      await userService.create({
        name: name.value, email: email.value, password: password.value, role: 'student',
        status: status.value, nisn: nisn.value || null, grade: grade.value,
      })
      toast.success(`Siswa ditambahkan — password sementara: ${password.value}`)
    }
    showForm.value = false
    load()
  } catch (e: any) {
    toast.error('Gagal menyimpan data siswa', e?.message)
  } finally {
    saving.value = false
  }
}

async function toggleStatus(u: User) {
  const next: UserStatus = u.status === 'active' ? 'inactive' : 'active'
  try {
    await userService.setStatus(u.id, next)
    toast.success(next === 'active' ? 'Siswa diaktifkan' : 'Siswa dinonaktifkan')
    if (detailStudent.value?.id === u.id) detailStudent.value.status = next
    load()
  } catch (e: any) {
    toast.error('Gagal mengubah status', e?.message)
  }
}

async function resetPassword(u: User) {
  const newPassword = genPassword(u.name)
  try {
    await userService.update(u.id, {
      name: u.name, email: u.email, role: 'student', school_id: u.school_id,
      status: u.status, nisn: u.nisn, grade: u.grade, password: newPassword,
    })
    try {
      await navigator.clipboard?.writeText(newPassword)
      toast.success(`Password baru: ${newPassword} (disalin)`)
    } catch {
      toast.success(`Password baru untuk ${u.name}: ${newPassword}`)
    }
  } catch (e: any) {
    toast.error('Gagal reset password', e?.message)
  }
}

// ─── Import .xlsx massal (2-langkah: Upload -> Preview & Konfirmasi) ──────────────
// Format & parser sama persis dengan yang dipakai Admin di SchoolOnboardingWizard.vue (lihat
// utils/studentImportXlsx.ts) — supaya file export dari sistem sekolah (nis, nisn, nama,
// jenis kelamin, tanggal lahir, tanggal masuk, kode rombel, nama rombel, username, email,
// telepon, password) bisa langsung dipakai di sini juga tanpa perlu diubah ke CSV 4-kolom
// dulu. Password diambil apa adanya dari file, bukan dibuat otomatis lagi.
const showImport = ref(false)
const importStep = ref<1 | 2>(1)
const importFileInput = ref<HTMLInputElement | null>(null)
const importFile = ref<File | null>(null)
const importParsing = ref(false)
const importParseError = ref('')
const importRows = ref<ImportedStudentRow[]>([])
const importing = ref(false)
const importDone = ref(false)
const importedCount = ref(0)
const importFailed = ref<{ name: string; reason: string }[]>([])
const readyRows = computed(() => importRows.value.filter((r) => r.status === 'ready'))

function downloadTemplate() {
  downloadStudentImportTemplate()
  toast.success('Template .xlsx diunduh')
}

function openImport() {
  showImport.value = true
  importStep.value = 1
  importFile.value = null; importParseError.value = ''; importRows.value = []
  importDone.value = false; importedCount.value = 0; importFailed.value = []
  if (importFileInput.value) importFileInput.value.value = ''
}
function pickImportFile() {
  importFileInput.value?.click()
}
async function onImportFileSelected(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  if (!/\.(xlsx|xls)$/i.test(file.name)) {
    toast.error('Format tidak didukung', 'Unggah file .xlsx atau .xls.')
    return
  }
  importFile.value = file
  importRows.value = []
  importParseError.value = ''
  importParsing.value = true
  try {
    importRows.value = await parseStudentImportWorkbook(file)
    if (importRows.value.length === 0) importParseError.value = 'Tidak ada baris data ditemukan di file ini.'
    else importStep.value = 2
  } catch (err: any) {
    importParseError.value = err?.message || 'Gagal membaca file.'
  } finally {
    importParsing.value = false
  }
}
function backToInput() {
  importStep.value = 1
}
async function copyCredentials() {
  const text = readyRows.value.map((r) => `${r.name} | ${r.email} | ${r.password}`).join('\n')
  try {
    await navigator.clipboard?.writeText(text)
    toast.success('Kredensial disalin ke clipboard')
  } catch {
    toast.error('Gagal menyalin — clipboard tidak tersedia')
  }
}
async function confirmImport() {
  importing.value = true
  importedCount.value = 0
  importFailed.value = []
  for (const r of readyRows.value) {
    try {
      // Tidak perlu kirim school_id — backend otomatis scope ke sekolah aktor School yang
      // login ini (lihat UserService::create), sama seperti alur "Tambah Siswa" manual di
      // atas.
      await userService.create(importedRowToCreatePayload(r))
      importedCount.value += 1
    } catch (e: any) {
      importFailed.value.push({ name: r.name, reason: e?.message || 'gagal dibuat' })
    }
  }
  importing.value = false
  importDone.value = true
  load()
}
</script>

<template>
  <div class="space-y-4">
    <div>
      <h2 class="text-lg font-bold flex items-center gap-2"><Users class="w-5 h-5 text-blue-600" />Manajemen Siswa</h2>
      <p class="text-sm text-muted-foreground">{{ students.length }} siswa terdaftar &middot; {{ activeCount }} aktif</p>
    </div>

    <!-- Stat cards -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
      <Card class="p-4 bg-indigo-50 border-indigo-200 flex items-center justify-between">
        <div><p class="text-xs text-muted-foreground mb-1">Total Siswa</p><p class="text-2xl font-black text-slate-900">{{ students.length }}</p></div>
        <div class="w-9 h-9 rounded-xl bg-white/70 flex items-center justify-center"><Users class="w-4 h-4 text-indigo-600" /></div>
      </Card>
      <Card class="p-4 bg-emerald-50 border-emerald-200 flex items-center justify-between">
        <div><p class="text-xs text-muted-foreground mb-1">Aktif</p><p class="text-2xl font-black text-emerald-600">{{ activeCount }}</p></div>
        <div class="w-9 h-9 rounded-xl bg-white/70 flex items-center justify-center"><UserCheck class="w-4 h-4 text-emerald-600" /></div>
      </Card>
      <Card class="p-4 bg-purple-50 border-purple-200 flex items-center justify-between">
        <div><p class="text-xs text-muted-foreground mb-1">Kelas 12</p><p class="text-2xl font-black text-slate-900">{{ gradeCounts['Kelas 12'] || 0 }}</p></div>
        <div class="w-9 h-9 rounded-xl bg-white/70 flex items-center justify-center"><GraduationCap class="w-4 h-4 text-purple-600" /></div>
      </Card>
      <Card class="p-4 bg-blue-50 border-blue-200 flex items-center justify-between">
        <div><p class="text-xs text-muted-foreground mb-1">Rata-rata Skor</p><p class="text-2xl font-black text-slate-900">{{ avgScoreAll ?? '-' }}</p></div>
        <div class="w-9 h-9 rounded-xl bg-white/70 flex items-center justify-center"><TrendingUp class="w-4 h-4 text-blue-600" /></div>
      </Card>
    </div>

    <!-- Toolbar: search + filters + actions -->
    <Card class="p-4">
      <div class="flex flex-col sm:flex-row gap-2">
        <div class="relative flex-1">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input v-model="search" placeholder="Cari nama, NISN, kelas, atau target PTN..." class="w-full pl-9 pr-3 py-2 border rounded-lg text-sm bg-white" />
        </div>
        <select v-model="filterGrade" class="px-3 py-2 border rounded-lg text-sm bg-white">
          <option value="">Semua Kelas</option>
          <option v-for="g in GRADES" :key="g" :value="g">{{ g }}</option>
        </select>
        <select v-model="filterStatus" class="px-3 py-2 border rounded-lg text-sm bg-white">
          <option value="">Semua Status</option>
          <option value="active">Aktif</option>
          <option value="inactive">Nonaktif</option>
        </select>
        <div class="flex flex-wrap items-center gap-2 sm:flex-nowrap sm:shrink-0">
          <Button variant="outline" @click="exportCsv"><Download class="w-4 h-4" />Export</Button>
          <Button variant="outline" class="text-indigo-600 border-indigo-200 hover:bg-indigo-50" @click="openImport"><Upload class="w-4 h-4" />Import Siswa</Button>
          <Button variant="gradient" @click="openCreate"><Plus class="w-4 h-4" />Tambah Siswa</Button>
        </div>
      </div>
    </Card>

    <Card class="overflow-hidden">
      <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
      <div v-else-if="filteredStudents.length === 0" class="p-10 text-center text-sm text-muted-foreground">Tidak ada siswa yang sesuai filter.</div>
      <div v-else class="overflow-x-auto">
      <table class="w-full text-sm">
        <thead class="bg-slate-50 text-xs uppercase text-slate-500">
          <tr>
            <th class="text-left px-4 py-3 cursor-pointer select-none" @click="toggleSort('name')">
              <span class="inline-flex items-center gap-1">Siswa <ChevronUp v-if="sortKey === 'name' && !sortDesc" class="w-3 h-3" /><ChevronDown v-else-if="sortKey === 'name' && sortDesc" class="w-3 h-3" /></span>
            </th>
            <th class="text-left px-4 py-3">Kelas</th>
            <th class="text-left px-4 py-3">Target PTN</th>
            <th class="text-left px-4 py-3 cursor-pointer select-none" @click="toggleSort('avg_score')">
              <span class="inline-flex items-center gap-1">Rata-rata <ChevronUp v-if="sortKey === 'avg_score' && !sortDesc" class="w-3 h-3" /><ChevronDown v-else-if="sortKey === 'avg_score' && sortDesc" class="w-3 h-3" /></span>
            </th>
            <th class="text-left px-4 py-3 cursor-pointer select-none" @click="toggleSort('tryouts')">
              <span class="inline-flex items-center gap-1">Tryout <ChevronUp v-if="sortKey === 'tryouts' && !sortDesc" class="w-3 h-3" /><ChevronDown v-else-if="sortKey === 'tryouts' && sortDesc" class="w-3 h-3" /></span>
            </th>
            <th class="text-left px-4 py-3">Status</th>
            <th class="text-right px-4 py-3">Aksi</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-50">
          <tr v-for="s in filteredStudents" :key="s.id" class="hover:bg-slate-50 transition-colors">
            <td class="px-4 py-3">
              <button class="flex items-center gap-3 text-left" @click="openDetail(s)">
                <div class="w-9 h-9 rounded-full bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shrink-0 text-white text-xs font-bold">{{ initials(s.name) }}</div>
                <div class="min-w-0">
                  <p class="text-sm font-semibold text-slate-900 truncate">{{ s.name }}</p>
                  <p class="text-xs text-slate-500 truncate">{{ s.nisn ? `NISN ${s.nisn}` : s.email }}</p>
                </div>
              </button>
            </td>
            <td class="px-4 py-3 text-slate-600">{{ s.grade || '-' }}</td>
            <td class="px-4 py-3">
              <Badge v-if="(statMap.get(s.id)?.target_ptn_count ?? 0) > 0" variant="outline">{{ statMap.get(s.id)?.target_ptn_count }} target</Badge>
              <span v-else class="text-xs text-slate-400">-</span>
            </td>
            <td class="px-4 py-3">
              <div class="flex items-center gap-1.5">
                <span class="font-semibold text-slate-900">{{ statMap.get(s.id)?.avg_score ?? '-' }}</span>
                <span class="text-[10px] px-1.5 py-0.5 rounded-full font-semibold" :class="gradeBadge(statMap.get(s.id)?.avg_score).class">{{ gradeBadge(statMap.get(s.id)?.avg_score).label }}</span>
              </div>
            </td>
            <td class="px-4 py-3 text-slate-600">{{ statMap.get(s.id)?.tryouts_completed ?? 0 }} sesi</td>
            <td class="px-4 py-3">
              <Badge :variant="s.status === 'active' ? 'success' : 'secondary'">{{ s.status === 'active' ? 'Aktif' : 'Nonaktif' }}</Badge>
            </td>
            <td class="px-4 py-3">
              <div class="flex items-center justify-end gap-1">
                <button class="p-1.5 rounded-lg border border-transparent hover:bg-slate-100 text-slate-400 hover:text-slate-700" title="Detail" @click="openDetail(s)"><Eye class="w-4 h-4" /></button>
                <button
                  class="p-1.5 rounded-lg border transition-colors"
                  :class="s.status === 'active' ? 'text-slate-500 hover:text-orange-500 hover:bg-orange-50 border-transparent hover:border-orange-200' : 'text-slate-400 hover:text-emerald-600 hover:bg-emerald-50 border-transparent hover:border-emerald-200'"
                  :title="s.status === 'active' ? 'Nonaktifkan' : 'Aktifkan'" @click="toggleStatus(s)"
                >
                  <UserX v-if="s.status === 'active'" class="w-4 h-4" /><UserCheck v-else class="w-4 h-4" />
                </button>
                <button class="p-1.5 rounded-lg border border-transparent hover:bg-slate-100 text-slate-400 hover:text-slate-700" title="Edit" @click="openEdit(s)"><Pencil class="w-4 h-4" /></button>
                <button class="p-1.5 rounded-lg border border-transparent hover:bg-amber-50 hover:border-amber-200 text-slate-400 hover:text-amber-600" title="Reset password" @click="resetPassword(s)"><RefreshCw class="w-4 h-4" /></button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
      </div>
      <div v-if="!loading && filteredStudents.length > 0" class="px-4 py-2.5 text-xs text-muted-foreground border-t bg-slate-50/50">
        Menampilkan {{ filteredStudents.length }} dari {{ students.length }} siswa
      </div>
    </Card>

    <!-- Detail drawer -->
    <div v-if="detailStudent" class="fixed inset-0 z-50 flex justify-end bg-black/40" @click.self="detailStudent = null">
      <div class="w-full max-w-sm bg-white h-full overflow-y-auto p-6 space-y-5">
        <div class="flex items-center justify-between">
          <h3 class="font-bold text-slate-900">Detail Siswa</h3>
          <button class="text-muted-foreground hover:text-foreground" @click="detailStudent = null"><X class="w-4 h-4" /></button>
        </div>
        <div class="flex items-center gap-3">
          <div class="w-14 h-14 rounded-2xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white font-bold">{{ initials(detailStudent.name) }}</div>
          <div class="min-w-0">
            <p class="font-bold text-slate-900 truncate">{{ detailStudent.name }}</p>
            <p class="text-xs text-slate-500">{{ detailStudent.nisn ? `NISN ${detailStudent.nisn}` : detailStudent.grade || '-' }}</p>
            <Badge class="mt-1" :variant="detailStudent.status === 'active' ? 'success' : 'secondary'">{{ detailStudent.status === 'active' ? 'Aktif' : 'Nonaktif' }}</Badge>
          </div>
        </div>
        <div class="grid grid-cols-3 gap-2">
          <Card class="p-3 text-center"><p class="text-lg font-black text-slate-900">{{ statMap.get(detailStudent.id)?.best_score ?? '-' }}</p><p class="text-[10px] text-muted-foreground">Skor Terbaik</p></Card>
          <Card class="p-3 text-center"><p class="text-lg font-black text-slate-900">{{ statMap.get(detailStudent.id)?.tryouts_completed ?? 0 }}</p><p class="text-[10px] text-muted-foreground">Tryout</p></Card>
          <Card class="p-3 text-center"><p class="text-xs font-black text-slate-900 leading-tight pt-1">{{ formatDate(statMap.get(detailStudent.id)?.last_attempt_at) ?? 'Belum pernah' }}</p><p class="text-[10px] text-muted-foreground">Terakhir Aktif</p></Card>
        </div>
        <div class="space-y-2 text-sm">
          <p class="text-xs font-bold text-slate-500 uppercase">Informasi Kontak</p>
          <div class="flex items-center gap-2 text-slate-600 bg-slate-50 rounded-lg px-3 py-2"><Mail class="w-4 h-4 text-slate-400 shrink-0" /><span class="truncate">{{ detailStudent.email }}</span></div>
          <div v-if="detailStudent.phone" class="flex items-center gap-2 text-slate-600 bg-slate-50 rounded-lg px-3 py-2"><Phone class="w-4 h-4 text-slate-400 shrink-0" />{{ detailStudent.phone }}</div>
          <div v-if="detailStudent.nisn" class="flex items-center gap-2 text-slate-600 bg-slate-50 rounded-lg px-3 py-2"><Hash class="w-4 h-4 text-slate-400 shrink-0" />NISN {{ detailStudent.nisn }}</div>
          <div class="flex items-center gap-2 text-slate-600 bg-slate-50 rounded-lg px-3 py-2"><GraduationCap class="w-4 h-4 text-slate-400 shrink-0" />{{ detailStudent.grade || '-' }}</div>
        </div>
        <div v-if="(statMap.get(detailStudent.id)?.avg_score ?? null) != null">
          <p class="text-xs font-bold text-slate-500 uppercase mb-2">Performa</p>
          <div class="p-3 rounded-xl border">
            <div class="flex items-center justify-between mb-1.5">
              <span class="text-[10px] px-1.5 py-0.5 rounded-full font-semibold" :class="gradeBadge(statMap.get(detailStudent.id)?.avg_score).class">{{ gradeBadge(statMap.get(detailStudent.id)?.avg_score).label }}</span>
              <span class="text-sm font-black text-slate-900">{{ statMap.get(detailStudent.id)?.avg_score }}</span>
            </div>
            <div class="h-2 rounded-full bg-slate-100 overflow-hidden">
              <div class="h-full rounded-full bg-gradient-to-r from-indigo-500 to-purple-500" :style="{ width: `${Math.min(((statMap.get(detailStudent.id)?.avg_score ?? 0) / PLATFORM_MAX_SCORE) * 100, 100)}%` }" />
            </div>
          </div>
        </div>
        <div>
          <h4 class="text-sm font-bold text-slate-900 mb-2 flex items-center gap-1.5"><Target class="w-4 h-4 text-indigo-600" />Target PTN</h4>
          <div v-if="detailLoading" class="text-xs text-muted-foreground">Memuat...</div>
          <div v-else-if="detailTargets.length === 0" class="text-xs text-muted-foreground">Belum ada target PTN.</div>
          <div v-else class="space-y-2">
            <div v-for="t in detailTargets" :key="t.id" class="p-2.5 rounded-lg border text-xs">
              <p class="font-semibold text-slate-900">{{ t.program.nama_prodi }}</p>
              <p class="text-muted-foreground">{{ t.program.nama_ptn }} &middot; {{ t.chance.chance_percent }}% peluang</p>
            </div>
          </div>
        </div>
        <div class="flex items-center gap-2 pt-3 border-t">
          <Button variant="outline" class="flex-1" @click="toggleStatus(detailStudent)">
            <UserX v-if="detailStudent.status === 'active'" class="w-4 h-4" /><UserCheck v-else class="w-4 h-4" />
            {{ detailStudent.status === 'active' ? 'Nonaktifkan' : 'Aktifkan' }}
          </Button>
          <Button variant="gradient" class="flex-1" @click="openEdit(detailStudent)"><Pencil class="w-4 h-4" />Edit Data</Button>
        </div>
      </div>
    </div>

    <!-- Form dialog -->
    <Dialog v-model="showForm" :title="editTarget ? 'Edit Siswa' : 'Tambah Siswa Baru'" max-width="max-w-md">
      <div class="space-y-4">
        <div>
          <Label>Nama</Label>
          <Input v-model="name" placeholder="Nama lengkap" class="mt-1.5" />
        </div>
        <div>
          <Label>Email</Label>
          <Input v-model="email" type="email" placeholder="nama@email.com" class="mt-1.5" />
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <Label>NISN</Label>
            <Input v-model="nisn" placeholder="0012345678" class="mt-1.5 font-mono" />
          </div>
          <div>
            <Label>Kelas</Label>
            <select v-model="grade" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
              <option v-for="g in GRADES" :key="g" :value="g">{{ g }}</option>
            </select>
          </div>
        </div>
        <div>
          <Label>Status</Label>
          <select v-model="status" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
            <option value="active">Aktif</option>
            <option value="inactive">Nonaktif</option>
          </select>
        </div>

        <div v-if="!editTarget" class="p-4 bg-amber-50 border border-amber-200 rounded-xl">
          <p class="text-xs font-bold text-amber-700 mb-2">Password Sementara</p>
          <div class="flex items-center gap-2">
            <code class="flex-1 text-sm font-mono bg-white px-3 py-2 rounded-lg border border-amber-200 text-slate-800 truncate">
              {{ showPassword ? password : '••••••••••' }}
            </code>
            <button type="button" class="p-2 rounded-lg hover:bg-amber-100 text-amber-600" @click="showPassword = !showPassword">
              <EyeOff v-if="showPassword" class="w-4 h-4" /><Eye v-else class="w-4 h-4" />
            </button>
            <button type="button" class="p-2 rounded-lg hover:bg-amber-100 text-amber-600" title="Buat ulang" @click="password = genPassword(name)"><RefreshCw class="w-4 h-4" /></button>
          </div>
        </div>

        <div class="flex items-center justify-between pt-4 border-t">
          <Button variant="outline" @click="showForm = false">Batal</Button>
          <Button variant="gradient" :disabled="saving" @click="save">
            <Sparkles class="w-4 h-4" /> {{ saving ? 'Menyimpan...' : editTarget ? 'Update' : 'Simpan' }}
          </Button>
        </div>
      </div>
    </Dialog>

    <!-- Import .xlsx dialog (2-langkah) -->
    <Dialog v-model="showImport" title="Import Siswa dari .xlsx" max-width="max-w-2xl">
      <div v-if="importDone" class="text-center py-4">
        <div class="w-16 h-16 rounded-full bg-green-100 flex items-center justify-center mx-auto mb-4"><CheckCircle2 class="w-8 h-8 text-green-600" /></div>
        <h3 class="text-xl font-black text-slate-900 mb-2">Import Selesai</h3>
        <p class="text-sm text-muted-foreground mb-4">
          <span class="font-bold text-green-600">{{ importedCount }} akun siswa</span> berhasil dibuat.
          <span v-if="importFailed.length" class="block text-red-500 mt-1">{{ importFailed.length }} baris gagal.</span>
        </p>
        <Button class="w-full" @click="showImport = false">Tutup</Button>
      </div>
      <div v-else class="space-y-4">
        <!-- Step tabs -->
        <div class="flex items-center gap-2">
          <div class="flex items-center gap-1.5 text-sm font-semibold" :class="importStep === 1 ? 'text-indigo-700' : 'text-emerald-600'">
            <span class="w-5 h-5 rounded-full flex items-center justify-center text-[11px]" :class="importStep === 1 ? 'bg-indigo-600 text-white' : 'bg-emerald-100 text-emerald-700'">
              <CheckCircle2 v-if="importStep === 2" class="w-3.5 h-3.5" /><template v-else>1</template>
            </span>Upload File
          </div>
          <div class="flex-1 h-px bg-slate-200" />
          <div class="flex items-center gap-1.5 text-sm font-semibold" :class="importStep === 2 ? 'text-indigo-700' : 'text-slate-400'">
            <span class="w-5 h-5 rounded-full flex items-center justify-center text-[11px]" :class="importStep === 2 ? 'bg-indigo-600 text-white' : 'bg-slate-100 text-slate-400'">2</span>
            Preview &amp; Konfirmasi
          </div>
        </div>

        <template v-if="importStep === 1">
          <div class="p-3 bg-slate-50 border rounded-xl">
            <div class="flex items-center justify-between mb-1.5">
              <p class="text-xs font-mono text-slate-500">Kolom: nis, nisn, nama, jenis kelamin, tanggal lahir, tanggal masuk, kode rombel, nama rombel, username, email, telepon, password</p>
              <button class="text-xs font-semibold text-indigo-600 hover:text-indigo-700 flex items-center gap-1 shrink-0" @click="downloadTemplate"><FileDown class="w-3.5 h-3.5" />Template</button>
            </div>
            <p class="text-[11px] text-muted-foreground">Wajib diisi: nisn, nama, email, password. Kolom lain boleh kosong.</p>
          </div>
          <div class="p-3 bg-blue-50 border border-blue-200 rounded-xl">
            <p class="text-xs font-bold text-blue-700 mb-1.5 flex items-center gap-1.5"><Key class="w-3.5 h-3.5" />Kredensial</p>
            <ul class="text-xs text-blue-800 space-y-1">
              <li>• <strong>Username login:</strong> email siswa</li>
              <li>• <strong>Password:</strong> persis seperti di kolom "password" pada file</li>
              <li>• Sarankan siswa mengganti password setelah login pertama</li>
            </ul>
          </div>
          <div
            class="border-2 border-dashed rounded-xl p-6 text-center cursor-pointer transition-colors"
            :class="importFile ? 'border-emerald-300 bg-emerald-50' : 'border-slate-200 hover:border-indigo-300 hover:bg-slate-50'"
            @click="pickImportFile"
          >
            <input ref="importFileInput" type="file" accept=".xlsx,.xls" class="hidden" @change="onImportFileSelected" />
            <FileSpreadsheet v-if="importFile" class="w-8 h-8 mx-auto mb-2 text-emerald-500" />
            <Upload v-else class="w-8 h-8 mx-auto mb-2 text-slate-400" />
            <p class="text-sm font-semibold text-slate-700">{{ importFile ? importFile.name : 'Klik untuk pilih file .xlsx' }}</p>
            <p v-if="importParsing" class="text-xs text-indigo-600 mt-1 flex items-center justify-center gap-1"><RefreshCw class="w-3 h-3 animate-spin" />Membaca file...</p>
          </div>
          <div v-if="importParseError" class="p-3 bg-red-50 border border-red-200 rounded-xl text-xs text-red-600 flex items-start gap-2">
            <AlertCircle class="w-3.5 h-3.5 shrink-0 mt-0.5" />{{ importParseError }}
          </div>
        </template>

        <template v-else>
          <p class="text-sm font-semibold text-slate-700">
            <span class="text-green-600">{{ readyRows.length }} siap</span>
            <span v-if="importRows.length > readyRows.length">, <span class="text-red-500">{{ importRows.length - readyRows.length }} error</span></span>
          </p>
          <div class="border rounded-xl max-h-52 overflow-y-auto overflow-x-auto">
            <table class="w-full text-xs">
              <thead class="sticky top-0 bg-slate-50"><tr class="border-b"><th class="text-left px-3 py-2">Nama</th><th class="text-left px-3 py-2">NISN</th><th class="text-left px-3 py-2">Email</th><th class="text-left px-3 py-2">Rombel</th><th class="text-left px-3 py-2">Status</th></tr></thead>
              <tbody>
                <tr v-for="(r, i) in importRows" :key="i" class="border-b" :class="r.status === 'error' ? 'bg-red-50' : ''">
                  <td class="px-3 py-2">{{ r.name || '-' }}</td>
                  <td class="px-3 py-2 font-mono">{{ r.nisn || '-' }}</td>
                  <td class="px-3 py-2 font-mono">{{ r.email || '-' }}</td>
                  <td class="px-3 py-2">{{ r.grade || '-' }}</td>
                  <td class="px-3 py-2">
                    <span v-if="r.status === 'ready'" class="text-green-600 font-semibold flex items-center gap-1"><CheckCircle2 class="w-3 h-3" />Siap</span>
                    <span v-else class="text-red-500 font-semibold flex items-center gap-1"><AlertCircle class="w-3 h-3" />{{ r.error }}</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-if="readyRows.length > 0" class="p-3 bg-emerald-50 border border-emerald-200 rounded-xl flex items-center justify-between gap-3">
            <p class="text-xs text-emerald-800">{{ readyRows.length }} akun akan langsung aktif dengan kredensial dari file.</p>
            <button class="text-xs font-semibold text-emerald-700 hover:text-emerald-800 flex items-center gap-1 shrink-0" @click="copyCredentials"><Copy class="w-3.5 h-3.5" />Salin Kredensial</button>
          </div>
          <div class="flex items-center justify-between pt-4 border-t">
            <Button variant="outline" @click="backToInput"><ArrowLeft class="w-4 h-4" />Kembali</Button>
            <Button variant="gradient" :disabled="readyRows.length === 0 || importing" @click="confirmImport">
              {{ importing ? 'Memproses...' : `Konfirmasi Import ${readyRows.length} Siswa` }}
            </Button>
          </div>
        </template>
      </div>
    </Dialog>
  </div>
</template>
