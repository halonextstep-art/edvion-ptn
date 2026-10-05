<script setup lang="ts">
// Rasionalisasi SNBT — tampilan utama mengikuti referensi
// (referensi/src/app/components/student/RasionalisasiSNBT.tsx): header + 3 stat pill
// (Skor Kamu / PTN Target / Peluang Terbaik), ringkasan prioritas, kartu target
// expandable dengan breakdown real, dan tips strategi. Bedanya dengan referensi (yang
// pakai skor manual & katalog universitas hardcode):
//  - "Skor Kamu" dihitung otomatis dari rata-rata skor tryout SNBT nyata yang sudah
//    diselesaikan (sama persis dengan average_tryout_score di backend yang dipakai
//    mesin compute_chance) — tidak ada input manual yang bisa memalsukan skor.
//  - Katalog & pencarian program studi pakai data resmi ~4588 prodi nyata
//    (rationalizationService), bukan daftar universitas yang ditulis manual.
//
// Jalur SNBP (nilai rapor, prestasi, target SNBP) tetap ada — dibutuhkan oleh fitur
// Rasionalisasi SNBP di portal Sekolah — tapi dipindah ke balik toggle "Jalur" di
// pojok kanan atas supaya tidak lagi tercampur dengan tampilan utama SNBT.
import {
  Search, Plus, Trash2, GraduationCap, Loader2, X, Award, Target, Users, BookOpen,
  ChevronDown, ChevronUp, BarChart3, CheckCircle2, AlertCircle, Zap, Building2, ExternalLink,
} from 'lucide-vue-next'
import type {
  PtnProgramItem, PtnTrack, PtnPriority, TargetWithChanceItem, Attempt, InstitutionItem,
} from '~/types'

const emit = defineEmits<{ navigate: [tab: string] }>()
const { rationalizationService, tryoutService, institutionService } = useApi()
const { user } = useAuth()

// Siswa B2C (mandiri, tanpa sekolah) difokuskan ke Tryout + Rasionalisasi SNBT saja —
// jalur SNBP kuotanya dialokasikan per-sekolah oleh pemerintah, jadi secara fungsional
// gak relevan buat siswa tanpa sekolah. Toggle "Jalur" disembunyikan untuk mereka dan
// mode dikunci ke 'snbt' (bukan cuma disembunyikan di UI — dipaksa balik kalau somehow
// ke-set ke 'snbp', misalnya dari state lama sebelum fitur ini ada).
const isB2C = computed(() => !user.value?.school_id)

const TIER_LABEL: Record<string, string> = { aman: 'Sangat Aman', moderat: 'Peluang Sedang', ketat: 'Perlu Peningkatan' }
const TIER_BADGE: Record<string, string> = {
  aman: 'bg-emerald-100 text-emerald-700', moderat: 'bg-amber-100 text-amber-700', ketat: 'bg-red-100 text-red-700',
}
const TIER_VARIANT: Record<string, 'success' | 'warning' | 'destructive'> = {
  aman: 'success', moderat: 'warning', ketat: 'destructive',
}
const PRIORITY_LABEL: Record<PtnPriority, string> = { utama: 'Target Utama', cadangan: 'Cadangan', aman: 'Pilihan Aman' }

// ─── Mode: SNBT (utama/default) vs SNBP (sekunder) ────────────────────────────────
const mode = ref<PtnTrack>('snbt')

// ─── Skor tryout real (rata-rata attempt tersubmit — sama dengan average_tryout_score
//     di backend) — dipakai sebagai "Skor Kamu" pada header SNBT, bukan input manual. ──
const attempts = ref<Attempt[]>([])
const attemptsLoading = ref(true)
const submittedAttempts = computed(() => attempts.value.filter((a) => a.status === 'submitted' && a.score != null))
const myTryoutScore = computed(() => {
  if (!submittedAttempts.value.length) return null
  return Math.round(submittedAttempts.value.reduce((sum, a) => sum + (a.score ?? 0), 0) / submittedAttempts.value.length)
})
async function loadAttempts() {
  attemptsLoading.value = true
  try {
    attempts.value = await tryoutService.myAttempts()
  } finally {
    attemptsLoading.value = false
  }
}
function goDrilling() {
  emit('navigate', 'drilling')
}

// Nilai rapor, prestasi, dan pencarian+target jalur SNBP dipindah ke komponen bersama
// <RasionalisasiSnbpEditor> (dipakai juga oleh panel verifikasi PIC sekolah) — lihat
// blok template mode SNBP di bawah.

// ─── Catalog search (khusus jalur SNBT — jalur SNBP punya instance sendiri di dalam
//     <RasionalisasiSnbpEditor>) ──────────────────────────────────────────────────
const searchQuery = ref('')
const searchResults = ref<PtnProgramItem[]>([])
const searching = ref(false)
const catalogTotal = ref<number | null>(null)

let searchTimer: ReturnType<typeof setTimeout> | null = null
watch(searchQuery, () => {
  if (searchTimer) clearTimeout(searchTimer)
  // Set status "Mencari..." SEGERA (bukan menunggu jeda 350ms selesai) begitu sudah 3+ huruf —
  // sama seperti fix di RasionalisasiSnbpEditor.vue — sebelumnya ada jeda kosong tanpa umpan
  // balik apa pun antara mulai mengetik dan pencarian benar-benar jalan.
  if (searchQuery.value.trim().length >= 3) {
    searching.value = true
    searchResults.value = []
  }
  searchTimer = setTimeout(runSearch, 350)
})
async function runSearch() {
  if (searchQuery.value.trim().length < 3) {
    searchResults.value = []
    searching.value = false
    return
  }
  searching.value = true
  try {
    const res = await rationalizationService.listPrograms({ search: searchQuery.value.trim(), page_size: 15 })
    searchResults.value = res.items
  } finally {
    searching.value = false
  }
}

onMounted(async () => {
  try {
    const size = await rationalizationService.catalogSize()
    catalogTotal.value = size.total
  } catch { /* non-critical */ }
})

// ─── Add-target picker — track mengikuti mode aktif saat modal dibuka, tidak ada lagi
//     dropdown "Jalur" (menyamakan alur dengan referensi yang implisit SNBT). ─────────
const pickerProgram = ref<PtnProgramItem | null>(null)
const pickerPriority = ref<PtnPriority>('utama')
const pickerPreview = ref<{ chance_percent: number; tier: string } | null>(null)
const previewLoading = ref(false)
const adding = ref(false)

// Scroll ke kartu "Cari Program Studi" saat tombol "Tambah Target PTN" ditekan —
// dipakai oleh mode SNBT maupun SNBP, masing-masing punya anchor id sendiri.
function scrollToCari(track: PtnTrack) {
  document.getElementById(track === 'snbt' ? 'cari-program-snbt' : 'cari-program-snbp')?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

function openPicker(program: PtnProgramItem) {
  pickerProgram.value = program
  pickerPriority.value = 'utama'
  pickerPreview.value = null
  loadPreview()
}
function closePicker() {
  pickerProgram.value = null
}
async function loadPreview() {
  if (!pickerProgram.value) return
  previewLoading.value = true
  try {
    pickerPreview.value = await rationalizationService.previewChance(pickerProgram.value.id, mode.value)
  } catch {
    pickerPreview.value = null
  } finally {
    previewLoading.value = false
  }
}
async function confirmAddTarget() {
  if (!pickerProgram.value) return
  adding.value = true
  try {
    await rationalizationService.addTarget(pickerProgram.value.id, mode.value, pickerPriority.value)
    closePicker()
    searchQuery.value = ''
    searchResults.value = []
    await refreshTargets()
  } finally {
    adding.value = false
  }
}

// ─── My targets ──────────────────────────────────────────────────────────────────
const targets = ref<TargetWithChanceItem[]>([])
const targetsLoading = ref(true)
const expandedId = ref<string | null>(null)

async function refreshTargets() {
  targetsLoading.value = true
  try {
    targets.value = await rationalizationService.myTargets()
  } finally {
    targetsLoading.value = false
  }
}
async function changePriority(t: TargetWithChanceItem, priority: PtnPriority) {
  await rationalizationService.setTargetPriority(t.id, priority)
  await refreshTargets()
}
async function removeTargetItem(t: TargetWithChanceItem) {
  await rationalizationService.removeTarget(t.id)
  await refreshTargets()
}
// Profil institusi (logo/alamat/website/akreditasi/status/tahun berdiri) — data riset
// eksternal terpisah dari ptn_programs, dimuat on-demand per kartu saat di-expand (bukan
// sekaligus untuk semua target) dan di-cache per nama_ptn supaya tidak fetch berulang.
// `null` di cache berarti "sudah dicek, memang belum ada data riset" — beda dari "belum
// pernah dicek" (key belum ada di cache) — supaya tidak retry terus-menerus.
const institutionCache = ref<Record<string, InstitutionItem | null>>({})
function institutionFor(namaPtn: string): InstitutionItem | null {
  return institutionCache.value[namaPtn] ?? null
}
async function loadInstitution(namaPtn: string) {
  if (namaPtn in institutionCache.value) return
  try {
    institutionCache.value[namaPtn] = await institutionService.lookup(namaPtn)
  } catch {
    institutionCache.value[namaPtn] = null
  }
}

function toggleExpand(t: TargetWithChanceItem) {
  const opening = expandedId.value !== t.id
  expandedId.value = opening ? t.id : null
  if (opening) loadInstitution(t.program.nama_ptn)
}

const snbtTargets = computed(() => {
  const list = targets.value.filter((t) => t.track === 'snbt')
  const order: Record<PtnPriority, number> = { utama: 0, cadangan: 1, aman: 2 }
  return [...list].sort((a, b) => order[a.priority] - order[b.priority])
})
const bestChance = computed(() => (snbtTargets.value.length ? Math.max(...snbtTargets.value.map((t) => t.chance.chance_percent)) : null))
function priorityCount(list: TargetWithChanceItem[], p: PtnPriority) {
  return list.filter((t) => t.priority === p).length
}

onMounted(() => {
  loadAttempts()
  refreshTargets()
  if (isB2C.value && mode.value === 'snbp') mode.value = 'snbt'
})
watch(isB2C, (b2c) => {
  if (b2c && mode.value === 'snbp') mode.value = 'snbt'
})
</script>

<template>
  <div class="space-y-6">
    <!-- Toggle jalur — disembunyikan untuk siswa B2C (mandiri, tanpa sekolah): jalur SNBP
         kuotanya dialokasikan per-sekolah, jadi gak relevan tanpa sekolah. -->
    <div v-if="!isB2C" class="flex justify-end">
      <div class="inline-flex p-1 rounded-xl bg-slate-100 border">
        <button
          class="px-3.5 py-1.5 rounded-lg text-xs font-bold transition-colors"
          :class="mode === 'snbt' ? 'bg-white text-indigo-700 shadow-sm' : 'text-slate-500 hover:text-slate-700'"
          @click="mode = 'snbt'"
        >
          Jalur SNBT
        </button>
        <button
          class="px-3.5 py-1.5 rounded-lg text-xs font-bold transition-colors"
          :class="mode === 'snbp' ? 'bg-white text-indigo-700 shadow-sm' : 'text-slate-500 hover:text-slate-700'"
          @click="mode = 'snbp'"
        >
          Jalur SNBP
        </button>
      </div>
    </div>

    <!-- ══════════════════════════ SNBT (tampilan utama) ══════════════════════════ -->
    <template v-if="mode === 'snbt'">
      <!-- Header -->
      <Card class="p-5 sm:p-6 bg-gradient-to-br from-blue-600 via-indigo-600 to-purple-600 text-white border-0 relative overflow-hidden">
        <div class="absolute inset-0 opacity-10" style="background-image: linear-gradient(rgba(255,255,255,.4) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,.4) 1px, transparent 1px); background-size: 40px 40px" />
        <div class="relative flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
          <div class="min-w-0">
            <div class="flex items-center gap-2 mb-1"><Target class="w-5 h-5 text-cyan-300 shrink-0" /><span class="font-bold">Rasionalisasi SNBT</span></div>
            <h2 class="text-xl sm:text-2xl font-black mb-1">Peluang Masuk PTN</h2>
            <p class="text-blue-100 text-sm">Katalog {{ catalogTotal ?? '...' }} program studi dari data resmi SNBT. Estimasi dari rata-rata skor tryoutmu, passing grade, dan daya tampung riil per prodi.</p>
          </div>
          <div class="flex items-center gap-2 sm:gap-3 shrink-0 w-full md:w-auto">
            <div class="flex-1 md:flex-none bg-white/15 backdrop-blur-sm px-2.5 sm:px-4 py-2 sm:py-2.5 rounded-xl text-center">
              <p class="text-xl sm:text-2xl font-black">{{ myTryoutScore ?? '-' }}</p>
              <p class="text-[10px] sm:text-xs text-blue-200">Skor Kamu</p>
            </div>
            <div class="flex-1 md:flex-none bg-white/15 backdrop-blur-sm px-2.5 sm:px-4 py-2 sm:py-2.5 rounded-xl text-center">
              <p class="text-xl sm:text-2xl font-black">{{ snbtTargets.length }}</p>
              <p class="text-[10px] sm:text-xs text-blue-200">PTN Target</p>
            </div>
            <div v-if="bestChance != null" class="flex-1 md:flex-none bg-white/15 backdrop-blur-sm px-2.5 sm:px-4 py-2 sm:py-2.5 rounded-xl text-center">
              <p class="text-xl sm:text-2xl font-black">{{ bestChance }}%</p>
              <p class="text-[10px] sm:text-xs text-blue-200">Peluang Terbaik</p>
            </div>
          </div>
        </div>
      </Card>

      <!-- Skor real + tombol tambah target (jelas terlihat, sama seperti referensi) -->
      <div class="flex flex-col sm:flex-row gap-4">
        <Card class="p-5 flex-1">
          <p class="text-sm font-semibold text-slate-700 mb-1">Skor Rata-rata Tryout Kamu</p>
          <p v-if="attemptsLoading" class="text-xs text-muted-foreground py-2">Memuat...</p>
          <template v-else>
            <p class="text-3xl font-black text-indigo-600 mb-1">{{ myTryoutScore ?? '-' }}<span class="text-sm text-muted-foreground font-normal"> / 1000</span></p>
            <p class="text-xs text-muted-foreground mb-3">
              {{ myTryoutScore != null
                ? `Dihitung otomatis dari rata-rata ${submittedAttempts.length} tryout SNBT yang sudah kamu selesaikan.`
                : 'Belum ada tryout yang diselesaikan — kerjakan tryout dulu supaya estimasi peluang bisa dihitung.' }}
            </p>
            <Button size="sm" variant="outline" @click="goDrilling"><Zap class="w-3.5 h-3.5" /> Kerjakan Tryout</Button>
          </template>
        </Card>
        <div class="flex items-center sm:items-stretch">
          <Button class="w-full sm:w-auto px-6 shadow-lg" variant="gradient" @click="scrollToCari('snbt')">
            <Plus class="w-4 h-4" /> Tambah Target PTN
          </Button>
        </div>
      </div>

      <!-- Cari program studi (target scroll dari tombol "Tambah Target PTN" di atas) -->
      <Card id="cari-program-snbt" class="p-5 scroll-mt-24">
        <h4 class="font-bold text-slate-900 mb-3">Cari Program Studi</h4>
        <div class="relative mb-3">
          <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
          <Input v-model="searchQuery" placeholder="Cari nama PTN atau program studi (min. 3 huruf)..." class="pl-9" />
        </div>
        <div v-if="searching" class="text-sm text-muted-foreground py-2 text-center">Mencari...</div>
        <div v-else-if="searchQuery.trim().length >= 3 && searchResults.length === 0" class="text-sm text-muted-foreground py-2 text-center">Tidak ditemukan.</div>
        <div v-else-if="searchQuery.trim().length < 3" class="text-sm text-muted-foreground py-2 text-center">Ketik minimal 3 huruf untuk mencari PTN atau program studi.</div>
        <div v-else class="space-y-1.5 max-h-64 overflow-y-auto">
          <div v-for="p in searchResults" :key="p.id" class="flex items-center justify-between gap-3 p-2 rounded-lg border hover:bg-slate-50">
            <div class="min-w-0">
              <div class="text-sm font-semibold text-slate-900 truncate">{{ p.nama_prodi }}</div>
              <div class="text-xs text-muted-foreground truncate">{{ p.nama_ptn }} · {{ p.kota }}</div>
            </div>
            <Button size="sm" variant="outline" class="shrink-0" @click="openPicker(p)"><Plus class="w-3.5 h-3.5" /> Tambah</Button>
          </div>
        </div>
      </Card>

      <!-- Summary strip -->
      <div v-if="snbtTargets.length > 0" class="grid grid-cols-3 gap-2 sm:gap-4">
        <Card v-for="p in (['utama', 'cadangan', 'aman'] as PtnPriority[])" :key="p" class="p-3 sm:p-4 text-center">
          <p class="text-xl sm:text-2xl font-black" :class="p === 'utama' ? 'text-indigo-600' : p === 'cadangan' ? 'text-amber-600' : 'text-emerald-600'">{{ priorityCount(snbtTargets, p) }}</p>
          <p class="text-[11px] sm:text-xs text-muted-foreground mt-0.5">{{ PRIORITY_LABEL[p] }}</p>
        </Card>
      </div>

      <!-- Target list -->
      <div v-if="targetsLoading" class="py-10 text-center text-sm text-muted-foreground"><Loader2 class="w-5 h-5 animate-spin mx-auto mb-2" /> Memuat target...</div>
      <Card v-else-if="snbtTargets.length === 0" class="p-10 text-center">
        <Target class="w-12 h-12 mx-auto mb-3 text-slate-300" />
        <h3 class="font-bold text-slate-600 mb-1">Belum ada target PTN jalur SNBT</h3>
        <p class="text-sm text-muted-foreground">Cari program studi di atas lalu tambahkan sebagai target untuk melihat estimasi peluang.</p>
      </Card>
      <div v-else class="space-y-4">
        <h3 class="font-bold text-slate-800 flex items-center gap-2"><BarChart3 class="w-4 h-4 text-indigo-600" /> Analisis Peluang per PTN</h3>
        <Card v-for="t in snbtTargets" :key="t.id" class="overflow-hidden border-l-4" :class="t.priority === 'utama' ? 'border-l-indigo-500' : t.priority === 'cadangan' ? 'border-l-amber-500' : 'border-l-emerald-500'">
          <div class="p-5">
            <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
              <div class="flex items-start gap-4 flex-1 min-w-0">
                <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white text-xs font-black shrink-0 text-center leading-tight px-1">
                  {{ t.program.singkatan || t.program.nama_ptn.slice(0, 4).toUpperCase() }}
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 flex-wrap mb-1">
                    <h4 class="font-bold text-slate-900 truncate">{{ t.program.nama_ptn }}</h4>
                    <Badge variant="outline">{{ PRIORITY_LABEL[t.priority] }}</Badge>
                  </div>
                  <p class="text-sm text-indigo-600 font-semibold mb-1">{{ t.program.nama_prodi }}</p>
                  <div class="flex items-center gap-4 text-xs text-muted-foreground flex-wrap">
                    <span class="flex items-center gap-1"><Users class="w-3 h-3" />{{ t.program.peminat_snbt.toLocaleString('id-ID') }} peminat</span>
                    <span class="flex items-center gap-1"><BookOpen class="w-3 h-3" />{{ t.program.daya_tampung_snbt }} kursi</span>
                    <span class="flex items-center gap-1"><Target class="w-3 h-3" />PG: {{ t.program.pg_snbt }}</span>
                  </div>
                </div>
              </div>

              <div class="flex items-center gap-4 lg:shrink-0">
                <div v-if="!t.program.has_official_stats" class="text-center max-w-[9rem]">
                  <Badge class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
                  <p class="text-[10px] text-muted-foreground mt-1 leading-snug">Belum ada angka daya tampung/peminat resmi untuk prodi ini — estimasi peluang belum bisa dihitung.</p>
                </div>
                <div v-else class="text-center">
                  <div class="text-3xl font-black mb-0.5" :class="t.chance.tier === 'aman' ? 'text-emerald-600' : t.chance.tier === 'moderat' ? 'text-amber-600' : 'text-red-500'">{{ t.chance.chance_percent }}%</div>
                  <Badge :class="TIER_BADGE[t.chance.tier]" class="border-0">{{ TIER_LABEL[t.chance.tier] }}</Badge>
                  <div class="mt-2 w-28 h-2.5 bg-slate-100 rounded-full overflow-hidden">
                    <div class="h-full rounded-full" :class="t.chance.tier === 'aman' ? 'bg-emerald-500' : t.chance.tier === 'moderat' ? 'bg-amber-500' : 'bg-red-500'" :style="{ width: `${t.chance.chance_percent}%` }" />
                  </div>
                </div>
                <div class="flex flex-col gap-1">
                  <button class="p-2 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-slate-700" @click="toggleExpand(t)">
                    <ChevronUp v-if="expandedId === t.id" class="w-4 h-4" /><ChevronDown v-else class="w-4 h-4" />
                  </button>
                  <select
                    :value="t.priority" class="text-[11px] border rounded-lg px-1.5 py-1 bg-white"
                    @change="changePriority(t, ($event.target as HTMLSelectElement).value as PtnPriority)"
                  >
                    <option value="utama">Utama</option>
                    <option value="cadangan">Cadangan</option>
                    <option value="aman">Aman</option>
                  </select>
                  <button class="p-2 rounded-lg hover:bg-red-50 text-slate-400 hover:text-red-500" @click="removeTargetItem(t)"><Trash2 class="w-4 h-4" /></button>
                </div>
              </div>
            </div>

            <!-- Expanded detail -->
            <div v-if="expandedId === t.id" class="mt-4 pt-4 border-t space-y-4">
              <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                <div class="bg-slate-50 rounded-xl p-3">
                  <p class="text-xs text-muted-foreground mb-1">Passing Grade</p>
                  <p class="text-lg font-black text-slate-900">{{ t.program.pg_snbt }}</p>
                  <p class="text-[10px] text-muted-foreground mt-0.5">referensi tahun lalu</p>
                </div>
                <div class="bg-slate-50 rounded-xl p-3">
                  <p class="text-xs text-muted-foreground mb-1">Skor Kamu</p>
                  <p class="text-lg font-black" :class="myTryoutScore != null && myTryoutScore - t.program.pg_snbt >= 0 ? 'text-emerald-600' : 'text-red-500'">{{ myTryoutScore ?? '-' }}</p>
                  <p class="text-[10px] text-muted-foreground mt-0.5">
                    {{ myTryoutScore != null ? `selisih ${myTryoutScore - t.program.pg_snbt >= 0 ? '+' : ''}${myTryoutScore - t.program.pg_snbt}` : 'belum ada skor' }}
                  </p>
                </div>
                <div class="bg-slate-50 rounded-xl p-3">
                  <p class="text-xs text-muted-foreground mb-1">Rasio Kursi</p>
                  <p class="text-lg font-black text-slate-900">{{ Math.round(t.chance.competition_ratio * 100) }}%</p>
                  <p class="text-[10px] text-muted-foreground mt-0.5">{{ t.program.daya_tampung_snbt }} kursi / {{ t.program.peminat_snbt.toLocaleString('id-ID') }} peminat</p>
                </div>
                <div class="bg-slate-50 rounded-xl p-3">
                  <p class="text-xs text-muted-foreground mb-1">Kompetisi</p>
                  <p class="text-lg font-black text-slate-900">{{ t.program.peminat_snbt > 0 ? Math.round(t.program.peminat_snbt / Math.max(t.program.daya_tampung_snbt, 1)) : '-' }}:1</p>
                  <p class="text-[10px] text-muted-foreground mt-0.5">peminat per kursi</p>
                </div>
              </div>

              <div
                v-if="myTryoutScore != null"
                class="flex items-start gap-2 p-3 rounded-xl border"
                :class="myTryoutScore - t.program.pg_snbt >= 0 ? 'bg-emerald-50 border-emerald-200' : 'bg-red-50 border-red-200'"
              >
                <CheckCircle2 v-if="myTryoutScore - t.program.pg_snbt >= 0" class="w-4 h-4 text-emerald-600 shrink-0 mt-0.5" />
                <AlertCircle v-else class="w-4 h-4 text-red-500 shrink-0 mt-0.5" />
                <p class="text-sm" :class="myTryoutScore - t.program.pg_snbt >= 0 ? 'text-emerald-800' : 'text-red-700'">
                  {{ myTryoutScore - t.program.pg_snbt >= 0
                    ? `Skor kamu sudah ${myTryoutScore - t.program.pg_snbt} poin di atas passing grade. Pertahankan dan tingkatkan konsistensi latihan.`
                    : `Skor kamu masih ${Math.abs(myTryoutScore - t.program.pg_snbt)} poin di bawah passing grade. Fokus drilling untuk meningkatkan skor.` }}
                </p>
              </div>

              <!-- Profil Institusi — hanya tampil kalau data riset tersedia untuk PTN ini;
                   diam-diam tidak ditampilkan (bukan placeholder kosong) kalau belum ada. -->
              <div v-if="institutionFor(t.program.nama_ptn)" class="flex items-start gap-3 p-3 rounded-xl border bg-white">
                <img
                  v-if="institutionFor(t.program.nama_ptn)?.logo_url"
                  :src="institutionFor(t.program.nama_ptn)!.logo_url!"
                  class="w-10 h-10 rounded-lg object-contain border shrink-0 bg-white"
                  alt=""
                />
                <div v-else class="w-10 h-10 rounded-lg bg-slate-100 flex items-center justify-center shrink-0">
                  <Building2 class="w-4 h-4 text-slate-400" />
                </div>
                <div class="min-w-0 flex-1">
                  <p class="text-xs font-bold text-slate-900 truncate">Profil Institusi</p>
                  <p class="text-xs text-muted-foreground flex flex-wrap gap-x-1.5">
                    <span v-if="institutionFor(t.program.nama_ptn)?.akreditasi">Akreditasi {{ institutionFor(t.program.nama_ptn)?.akreditasi }}</span>
                    <span v-if="institutionFor(t.program.nama_ptn)?.status">· {{ institutionFor(t.program.nama_ptn)?.status }}</span>
                    <span v-if="institutionFor(t.program.nama_ptn)?.tahun_berdiri">· Berdiri {{ institutionFor(t.program.nama_ptn)?.tahun_berdiri }}</span>
                  </p>
                  <p v-if="institutionFor(t.program.nama_ptn)?.alamat" class="text-[11px] text-muted-foreground truncate mt-0.5">{{ institutionFor(t.program.nama_ptn)?.alamat }}</p>
                </div>
                <a
                  v-if="institutionFor(t.program.nama_ptn)?.website"
                  :href="institutionFor(t.program.nama_ptn)!.website!"
                  target="_blank" rel="noopener"
                  class="flex items-center gap-1 text-xs text-indigo-600 hover:underline shrink-0"
                >
                  Website <ExternalLink class="w-3 h-3" />
                </a>
              </div>
            </div>
          </div>
        </Card>
      </div>

      <!-- Tips -->
      <Card v-if="snbtTargets.length > 0" class="p-6 bg-gradient-to-br from-indigo-50 to-purple-50 border-indigo-200">
        <div class="flex items-start gap-3 mb-4">
          <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-600 to-purple-600 flex items-center justify-center shrink-0">
            <Award class="w-5 h-5 text-white" />
          </div>
          <div>
            <h3 class="font-bold text-slate-900">Tips Strategi Pilihan PTN</h3>
            <p class="text-xs text-muted-foreground">Berdasarkan data historis SNBT dan konfigurasi targetmu</p>
          </div>
        </div>
        <ul class="space-y-2.5">
          <li
            v-for="(tip, i) in [
              'Gunakan pola 1-1-1: 1 target utama, 1 cadangan, 1 pilihan aman untuk meminimalkan risiko tidak lolos.',
              'Perhatikan daya tampung dan jumlah peminat — prodi dengan kursi banyak dan peminat stabil lebih terprediksi.',
              'Skor SNBT tidak berubah setelah ujian. Optimalkan semua subtes lewat drilling sebelum hari-H.',
              'Data passing grade bersifat referensi dari tahun sebelumnya — bisa naik atau turun tergantung kompetisi tahun ini.',
            ]"
            :key="i" class="flex items-start gap-2.5 text-sm text-slate-700"
          >
            <span class="w-5 h-5 rounded-full bg-indigo-100 text-indigo-700 text-xs font-black flex items-center justify-center shrink-0 mt-0.5">{{ i + 1 }}</span>
            {{ tip }}
          </li>
        </ul>
      </Card>
    </template>

    <!-- ══════════════════════════ SNBP (sekunder) ══════════════════════════ -->
    <template v-else>
      <Card class="p-5 bg-gradient-to-br from-emerald-600 via-teal-600 to-cyan-600 text-white border-0">
        <div class="flex items-start gap-3">
          <div class="w-11 h-11 rounded-xl bg-white/20 flex items-center justify-center shrink-0">
            <GraduationCap class="w-6 h-6" />
          </div>
          <div>
            <h3 class="font-bold text-lg">Rasionalisasi SNBP</h3>
            <p class="text-sm text-emerald-50">Estimasi peluang dari nilai rapor &amp; prestasi nyatamu dibanding passing grade dan daya tampung riil program studi — bukan jaminan kelulusan.</p>
          </div>
        </div>
      </Card>

      <div class="flex justify-end">
        <Button variant="gradient" @click="scrollToCari('snbp')"><Plus class="w-4 h-4" /> Tambah Target PTN</Button>
      </div>

      <!-- Nilai rapor, prestasi, cari program & target — komponen bersama, dipakai juga
           oleh PIC sekolah untuk verifikasi/edit data siswa yang sama di portal Sekolah. -->
      <RasionalisasiSnbpEditor />
    </template>

    <!-- Target picker modal — dipakai mode SNBT (mode SNBP punya modalnya sendiri di
         dalam <RasionalisasiSnbpEditor>) -->
    <div v-if="pickerProgram" class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4" @click.self="closePicker">
      <Card class="w-full max-w-sm p-6 relative">
        <button class="absolute top-4 right-4 text-muted-foreground hover:text-foreground" @click="closePicker"><X class="w-4 h-4" /></button>
        <h3 class="font-bold text-slate-900 mb-1">{{ pickerProgram.nama_prodi }}</h3>
        <p class="text-xs text-muted-foreground mb-4">{{ pickerProgram.nama_ptn }} · Jalur SNBT</p>

        <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Prioritas</label>
        <select v-model="pickerPriority" class="w-full border rounded-lg px-3 py-2 text-sm mb-4">
          <option value="utama">Utama</option>
          <option value="cadangan">Cadangan</option>
          <option value="aman">Aman</option>
        </select>

        <div class="rounded-xl bg-slate-50 p-3 mb-4 text-center">
          <div v-if="previewLoading" class="text-xs text-muted-foreground py-2">Menghitung estimasi...</div>
          <template v-else-if="pickerProgram && !pickerProgram.has_official_stats">
            <Badge class="bg-slate-100 text-slate-500 border-0">Data belum lengkap</Badge>
            <div class="text-xs text-muted-foreground mt-1">Belum ada angka daya tampung/peminat resmi — estimasi peluang belum bisa dihitung.</div>
          </template>
          <template v-else-if="pickerPreview">
            <div class="text-2xl font-black" :class="pickerPreview.tier === 'aman' ? 'text-emerald-600' : pickerPreview.tier === 'moderat' ? 'text-amber-600' : 'text-red-600'">
              {{ pickerPreview.chance_percent }}%
            </div>
            <div class="text-xs text-muted-foreground">Estimasi peluang · {{ TIER_LABEL[pickerPreview.tier] }}</div>
          </template>
        </div>

        <Button class="w-full" :disabled="adding" @click="confirmAddTarget">
          <Loader2 v-if="adding" class="w-4 h-4 animate-spin" /> Tambahkan sebagai Target
        </Button>
      </Card>
    </div>
  </div>
</template>
