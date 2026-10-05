<script setup lang="ts">
import { Plus, Trash2, ChevronUp, ChevronDown, Timer, Layers, Clock, Lock, Unlock, ListChecks, Gauge, RefreshCw, Info, ShieldAlert } from 'lucide-vue-next'
import type { TryoutSessionTemplate, SimulationTemplateItem, SimulationTemplateKind, SchoolType, PackageItem, ScoreDisplayMode, IrtCalibrationStatus } from '~/types'

const { tryoutService, simulationService, packageService, settingsService } = useApi()
const toast = useToast()

// ─── Pengaturan Skor UTBK: "Skor Instan" vs "Estimasi IRT" ────────────────────────────
// Lihat backend domain::irt + domain::platform_settings::ScoreDisplayMode doc comment.
// "Estimasi IRT" adalah estimasi kalibrasi milik platform ini sendiri dari data historis
// pengerjaan siswa di platform — BUKAN replikasi skor UTBK resmi SNPMB (metode & data resmi
// SNPMB tidak dipublikasikan ke publik manapun).
const scoreDisplayMode = ref<ScoreDisplayMode | null>(null)
const scoreSettingsLoading = ref(true)
const scoreSettingsSaving = ref(false)
const irtStatus = ref<IrtCalibrationStatus | null>(null)
const irtStatusLoading = ref(false)
const recalibrating = ref(false)

async function loadScoreSettings() {
  scoreSettingsLoading.value = true
  try {
    const s = await settingsService.getSettings()
    scoreDisplayMode.value = s.score_display_mode
    lockdownDefault.value = s.simulation_lockdown_default
  } catch (e: any) {
    toast.error('Gagal memuat pengaturan skor', e?.message)
  } finally {
    scoreSettingsLoading.value = false
  }
}

async function loadIrtStatus() {
  irtStatusLoading.value = true
  try {
    irtStatus.value = await settingsService.getIrtStatus()
  } catch (e: any) {
    toast.error('Gagal memuat status kalibrasi IRT', e?.message)
  } finally {
    irtStatusLoading.value = false
  }
}

async function onChangeScoreDisplayMode(mode: ScoreDisplayMode) {
  scoreSettingsSaving.value = true
  try {
    const s = await settingsService.setScoreDisplayMode(mode)
    scoreDisplayMode.value = s.score_display_mode
    toast.success('Pengaturan tampilan skor berhasil disimpan')
  } catch (e: any) {
    toast.error('Gagal menyimpan pengaturan skor', e?.message)
  } finally {
    scoreSettingsSaving.value = false
  }
}

// ─── Mode Terkunci (focus/lockdown) — global default untuk Simulasi TO ────────────────
// Lihat backend domain::platform_settings::PlatformSettings::simulation_lockdown_default doc
// comment. Setiap template bisa override sendiri (lihat lockdownOverride di form + tombol per
// kartu template di bawah) — default global ini hanya berlaku untuk template yang TIDAK
// mengatur override-nya sendiri.
const lockdownDefault = ref(false)
const lockdownSettingsSaving = ref(false)

async function onChangeLockdownDefault(enabled: boolean) {
  lockdownSettingsSaving.value = true
  try {
    const s = await settingsService.setSimulationLockdownDefault(enabled)
    lockdownDefault.value = s.simulation_lockdown_default
    toast.success('Pengaturan Mode Terkunci berhasil disimpan')
  } catch (e: any) {
    toast.error('Gagal menyimpan pengaturan Mode Terkunci', e?.message)
  } finally {
    lockdownSettingsSaving.value = false
  }
}

async function triggerRecalibrate() {
  recalibrating.value = true
  try {
    const summary = await settingsService.recalibrateIrt()
    toast.success(`Kalibrasi selesai: ${summary.calibrated_count} soal terkalibrasi, ${summary.skipped_insufficient_data_count} soal dilewati (data belum cukup)`)
    await loadIrtStatus()
  } catch (e: any) {
    toast.error('Gagal menjalankan kalibrasi ulang', e?.message)
  } finally {
    recalibrating.value = false
  }
}

// `null` override = ikuti lockdownDefault global. Dipakai untuk badge status per kartu template.
function effectiveLockdown(t: SimulationTemplateItem) {
  return t.lockdown_override ?? lockdownDefault.value
}

function formatDateTime(iso: string) {
  return new Date(iso).toLocaleString('id-ID', { day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' })
}

const availableSessions = ref<TryoutSessionTemplate[]>([])
const sessionsLoading = ref(false)

const packages = ref<PackageItem[]>([])
const packagesLoading = ref(false)

const templates = ref<SimulationTemplateItem[]>([])
const templatesLoading = ref(false)

const showForm = ref(false)
const saving = ref(false)
const actionId = ref<string | null>(null)

interface SlotRow {
  session_id: string
  break_seconds: number
  /** Slot "Mapel Pilihan" — sesi diisi otomatis dari pilihan siswa (fitur "Pilih Mapel
   *  Pilihan"), bukan sesi tetap. Dipakai untuk Simulasi TKA "Hari 2 (Pilihan)". */
  is_elective: boolean
}

const title = ref('')
// Defaults true — a TO template is premium content by default, matching the backend column
// default (see `20250101000031_package_content_items.sql`), unlockable only once an admin
// assigns it to a Package via the "Isi Paket" manager in PackageManager.vue.
const isPremium = ref(true)
const rows = ref<SlotRow[]>([])
// Paket sumber "mapel pilihan" — wajib diisi kalau ada baris is_elective. Lihat
// domain::simulation::SimulationTemplate::elective_package_id (backend).
const electivePackageId = ref('')
// 'utbk_combined' (default, skor gabungan seperti Simulasi UTBK) atau 'per_subject' (skor
// per-mapel saja, tanpa digabung — sesuai struktur TKA asli yang tidak pernah menggabungkan
// nilai lintas mata uji). Lihat domain::simulation::SimulationTemplateKind (backend).
const templateKind = ref<SimulationTemplateKind>('utbk_combined')
// `null` = tampil ke siswa jenjang apapun. Isi kalau template ini khusus satu jenjang (mis.
// Simulasi TKA SMP) — lihat SimulationTemplateItem.school_type_scope doc comment.
const schoolTypeScope = ref<SchoolType | null>(null)
// `null` = ikuti default global (lockdownDefault di atas). `true`/`false` memaksa Mode Terkunci
// on/off khusus untuk template ini. Lihat SimulationTemplateItem.lockdown_override doc comment.
const lockdownOverride = ref<boolean | null>(null)

const hasElectiveRow = computed(() => rows.value.some((r) => r.is_elective))
const electivePackage = computed(() => packages.value.find((p) => p.id === electivePackageId.value) ?? null)

async function loadSessions() {
  sessionsLoading.value = true
  try {
    availableSessions.value = await tryoutService.listSessions()
  } catch (e: any) {
    toast.error('Gagal memuat daftar sesi', e?.message)
  } finally {
    sessionsLoading.value = false
  }
}

async function loadPackages() {
  packagesLoading.value = true
  try {
    packages.value = await packageService.list()
  } catch (e: any) {
    toast.error('Gagal memuat daftar paket', e?.message)
  } finally {
    packagesLoading.value = false
  }
}

async function loadTemplates() {
  templatesLoading.value = true
  try {
    templates.value = await simulationService.listTemplates()
  } catch (e: any) {
    toast.error('Gagal memuat template simulasi', e?.message)
  } finally {
    templatesLoading.value = false
  }
}

onMounted(() => {
  loadSessions()
  loadPackages()
  loadTemplates()
  loadScoreSettings()
  loadIrtStatus()
})

function sessionById(id: string) {
  return availableSessions.value.find((s) => s.id === id)
}

function resetForm() {
  title.value = ''
  isPremium.value = true
  rows.value = []
  electivePackageId.value = ''
  templateKind.value = 'utbk_combined'
  schoolTypeScope.value = null
  lockdownOverride.value = null
  showForm.value = false
}

function toggleForm() {
  if (showForm.value) {
    resetForm()
  } else {
    showForm.value = true
  }
}

function addRow() {
  rows.value.push({ session_id: '', break_seconds: 300, is_elective: false })
}

function toggleRowElective(row: SlotRow) {
  row.is_elective = !row.is_elective
  if (row.is_elective) row.session_id = ''
}

function removeRow(index: number) {
  rows.value.splice(index, 1)
}

function moveUp(index: number) {
  if (index === 0) return
  const arr = rows.value
  ;[arr[index - 1], arr[index]] = [arr[index], arr[index - 1]]
}

function moveDown(index: number) {
  if (index === rows.value.length - 1) return
  const arr = rows.value
  ;[arr[index], arr[index + 1]] = [arr[index + 1], arr[index]]
}

// Baris is_elective belum punya sesi tetap — durasinya baru diketahui saat siswa memulai
// simulasi (mengikuti sesi yang mereka pilih), jadi tidak ikut dijumlah di sini.
const totalDurationMinutes = computed(() =>
  rows.value.reduce((sum, r) => (r.is_elective ? sum : sum + (sessionById(r.session_id)?.duration_minutes ?? 0)), 0),
)
const slotCount = computed(() => rows.value.length)

function formatDuration(minutes: number) {
  if (minutes >= 60) {
    const h = Math.floor(minutes / 60)
    const m = minutes % 60
    return m > 0 ? `${h}j ${m}m` : `${h}j`
  }
  return `${minutes} menit`
}

async function submitTemplate() {
  if (!title.value.trim()) {
    toast.error('Judul template wajib diisi')
    return
  }
  if (rows.value.length === 0) {
    toast.error('Tambahkan minimal 1 subtes')
    return
  }
  if (rows.value.some((r) => !r.is_elective && !r.session_id)) {
    toast.error('Setiap subtes (bukan mapel pilihan) harus memilih sesi')
    return
  }
  const sessionIds = rows.value.filter((r) => !r.is_elective).map((r) => r.session_id)
  if (new Set(sessionIds).size !== sessionIds.length) {
    toast.error('Satu sesi tidak boleh dipakai dua kali dalam template yang sama')
    return
  }
  if (hasElectiveRow.value && !electivePackageId.value) {
    toast.error('Pilih paket sumber "Mapel Pilihan" untuk slot yang ditandai Mapel Pilihan')
    return
  }
  const electiveRowCount = rows.value.filter((r) => r.is_elective).length
  if (hasElectiveRow.value && electivePackage.value && electivePackage.value.elective_pick_count !== electiveRowCount) {
    toast.error(
      `Jumlah slot Mapel Pilihan (${electiveRowCount}) harus sama dengan "Jumlah Mapel Pilihan" pada paket "${electivePackage.value.name}" (${electivePackage.value.elective_pick_count})`,
    )
    return
  }

  saving.value = true
  try {
    await simulationService.createTemplate({
      title: title.value.trim(),
      is_premium: isPremium.value,
      slots: rows.value.map((r, i) => ({
        sequence_index: i + 1,
        session_id: r.is_elective ? null : r.session_id,
        break_seconds: r.break_seconds,
        is_elective: r.is_elective,
      })),
      elective_package_id: hasElectiveRow.value ? electivePackageId.value : null,
      template_kind: templateKind.value,
      // Sama seperti backfill migration exam_track — per_subject hampir selalu berarti TKA
      // (real TKA tidak pernah menggabungkan skor lintas mata uji), utbk_combined berarti SNBT.
      exam_track: templateKind.value === 'per_subject' ? 'tka' : 'snbt',
      school_type_scope: schoolTypeScope.value,
      lockdown_override: lockdownOverride.value,
    })
    toast.success('Template simulasi berhasil dibuat!')
    resetForm()
    await loadTemplates()
  } catch (e: any) {
    toast.error('Gagal membuat template simulasi', e?.message)
  } finally {
    saving.value = false
  }
}

async function toggleActive(t: SimulationTemplateItem) {
  actionId.value = t.id
  try {
    await simulationService.setTemplateActive(t.id, !t.is_active)
    toast.success(t.is_active ? 'Template dinonaktifkan' : 'Template diaktifkan')
    await loadTemplates()
  } catch (e: any) {
    toast.error('Gagal mengubah status template', e?.message)
  } finally {
    actionId.value = null
  }
}

async function togglePremium(t: SimulationTemplateItem) {
  actionId.value = t.id
  try {
    await simulationService.setTemplatePremium(t.id, !t.is_premium)
    toast.success(t.is_premium ? 'Template dijadikan gratis' : 'Template dijadikan premium')
    await loadTemplates()
  } catch (e: any) {
    toast.error('Gagal mengubah status premium', e?.message)
  } finally {
    actionId.value = null
  }
}

async function setLockdown(t: SimulationTemplateItem, value: string) {
  const override = value === 'inherit' ? null : value === 'on'
  actionId.value = t.id
  try {
    await simulationService.setTemplateLockdown(t.id, override)
    toast.success('Pengaturan Mode Terkunci template berhasil disimpan')
    await loadTemplates()
  } catch (e: any) {
    toast.error('Gagal mengubah Mode Terkunci', e?.message)
  } finally {
    actionId.value = null
  }
}

async function deleteTemplate(t: SimulationTemplateItem) {
  if (!window.confirm(`Hapus template "${t.title}"?`)) return
  actionId.value = t.id
  try {
    await simulationService.deleteTemplate(t.id)
    toast.success('Template berhasil dihapus')
    await loadTemplates()
  } catch (e: any) {
    toast.error('Gagal menghapus template', e?.message)
  } finally {
    actionId.value = null
  }
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2"><Timer class="w-5 h-5 text-indigo-600" />Simulasi UTBK</h2>
        <p class="text-sm text-muted-foreground">{{ templates.length }} template tersedia — menggabungkan beberapa sesi tryout menjadi satu simulasi berurutan</p>
        <p class="text-xs text-muted-foreground mt-0.5">"Buat Paket + Subtes" di tab Paket sudah bisa menyusun Simulasi UTBK/TKA otomatis — tab ini untuk pengaturan manual (jeda antar sesi, gabung sesi lintas paket) atau aktif/nonaktifkan template yang sudah ada.</p>
      </div>
      <Button variant="gradient" @click="toggleForm">
        <Plus class="w-4 h-4" />{{ showForm ? 'Tutup Form' : 'Buat Template Simulasi' }}
      </Button>
    </div>

    <!-- Pengaturan Skor UTBK: Skor Instan vs Estimasi IRT — lihat backend domain::irt +
         domain::platform_settings::ScoreDisplayMode doc comment. -->
    <Card class="p-5 space-y-4">
      <div class="flex items-start gap-2">
        <Gauge class="w-5 h-5 text-indigo-600 mt-0.5 shrink-0" />
        <div>
          <h3 class="text-sm font-bold text-slate-900">Pengaturan Skor UTBK</h3>
          <p class="text-xs text-muted-foreground mt-0.5">
            Pilih skor mana yang tampil ke siswa &amp; laporan sekolah untuk hasil UTBK/SNBT. Tidak memengaruhi skema penilaian TKA
            (skala 0–100 SD/SMP atau 200–800 SMA/SMK/MA sesuai jenjang paket, kategori Istimewa), yang selalu dipakai apa adanya.
          </p>
        </div>
      </div>

      <div class="flex items-start gap-2 bg-amber-50 border border-amber-100 rounded-lg px-3 py-2.5">
        <Info class="w-3.5 h-3.5 text-amber-600 mt-0.5 shrink-0" />
        <p class="text-[11px] text-amber-800 leading-relaxed">
          "Estimasi IRT" adalah estimasi kalibrasi milik platform ini sendiri, dihitung dari data historis pengerjaan siswa di
          platform (model Rasch/1PL) — <strong>bukan</strong> replikasi skor UTBK resmi SNPMB, karena metode &amp; data kalibrasi resmi
          SNPMB tidak dipublikasikan ke publik manapun.
        </p>
      </div>

      <div v-if="scoreSettingsLoading" class="text-xs text-muted-foreground">Memuat pengaturan...</div>
      <div v-else class="flex flex-wrap items-center gap-2">
        <label class="text-xs font-medium text-slate-600">Skor yang ditampilkan:</label>
        <select
          :value="scoreDisplayMode"
          :disabled="scoreSettingsSaving"
          class="px-2.5 py-1.5 border rounded-lg text-xs bg-white disabled:opacity-50"
          @change="onChangeScoreDisplayMode(($event.target as HTMLSelectElement).value as ScoreDisplayMode)"
        >
          <option value="instant">Hanya Skor Instan (default)</option>
          <option value="irt">Hanya Estimasi IRT</option>
          <option value="both">Keduanya (Skor Instan + Estimasi IRT)</option>
        </select>
        <span v-if="scoreSettingsSaving" class="text-[11px] text-muted-foreground">Menyimpan...</span>
      </div>

      <div class="pt-3 border-t flex items-center justify-between flex-wrap gap-3">
        <div class="text-xs text-slate-600">
          <p v-if="irtStatusLoading" class="text-muted-foreground">Memuat status kalibrasi...</p>
          <template v-else-if="irtStatus && irtStatus.calibrated_item_count > 0">
            <span class="font-semibold text-slate-800">{{ irtStatus.calibrated_item_count }} soal</span> sudah terkalibrasi
            <span v-if="irtStatus.last_calibrated_at">— terakhir {{ formatDateTime(irtStatus.last_calibrated_at) }}</span>
          </template>
          <template v-else>
            <span class="text-amber-700">Belum pernah dikalibrasi.</span> Estimasi IRT belum tersedia sampai kalibrasi pertama dijalankan
            dan cukup data historis terkumpul.
          </template>
        </div>
        <Button size="sm" variant="outline" class="text-xs gap-1.5" :disabled="recalibrating" @click="triggerRecalibrate">
          <RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': recalibrating }" />
          {{ recalibrating ? 'Mengkalibrasi...' : 'Kalibrasi Ulang IRT' }}
        </Button>
      </div>
    </Card>

    <!-- Mode Terkunci: fokus/lockdown untuk Simulasi TO — sembunyikan navigasi, paksa
         fullscreen, blok copy/klik-kanan/pindah tab selama siswa mengerjakan. Tidak berlaku
         untuk Drilling/Latihan/Mini. Bisa diatur global di sini, atau per-template lewat
         dropdown pada tiap kartu template di bawah (override mengalahkan default ini). -->
    <Card class="p-5 space-y-3">
      <div class="flex items-start gap-2">
        <ShieldAlert class="w-5 h-5 text-rose-600 mt-0.5 shrink-0" />
        <div>
          <h3 class="text-sm font-bold text-slate-900">Mode Terkunci — Simulasi TO</h3>
          <p class="text-xs text-muted-foreground mt-0.5">
            Saat aktif, siswa yang sedang mengerjakan Simulasi TO tidak bisa mengakses bagian lain aplikasi: layar dipaksa fullscreen,
            navigasi disembunyikan, klik-kanan/copy/pindah tab diblokir dan dicatat sebagai pelanggaran (siswa juga diberi peringatan).
            Tidak berlaku untuk Drilling atau Latihan/Mini Tryout.
          </p>
        </div>
      </div>
      <div v-if="scoreSettingsLoading" class="text-xs text-muted-foreground">Memuat pengaturan...</div>
      <div v-else class="flex flex-wrap items-center gap-2">
        <label class="text-xs font-medium text-slate-600">Default untuk semua Simulasi TO:</label>
        <select
          :value="lockdownDefault ? 'on' : 'off'"
          :disabled="lockdownSettingsSaving"
          class="px-2.5 py-1.5 border rounded-lg text-xs bg-white disabled:opacity-50"
          @change="onChangeLockdownDefault(($event.target as HTMLSelectElement).value === 'on')"
        >
          <option value="off">Nonaktif (default)</option>
          <option value="on">Aktif</option>
        </select>
        <span v-if="lockdownSettingsSaving" class="text-[11px] text-muted-foreground">Menyimpan...</span>
        <span class="text-[11px] text-muted-foreground">— template tertentu bisa override lewat dropdown di kartunya masing-masing.</span>
      </div>
    </Card>

    <Card v-if="showForm" class="p-5 space-y-4">
      <div>
        <label class="text-xs font-medium text-slate-600 mb-1 block">Judul Template</label>
        <input
          v-model="title"
          placeholder="Simulasi UTBK Batch 1"
          class="w-full px-3 py-2 border rounded-lg text-sm bg-white"
        />
      </div>

      <div class="flex items-center gap-3 flex-wrap">
        <label class="flex items-center gap-2 text-xs text-slate-600 bg-slate-50 border rounded-lg px-3 py-2 cursor-pointer w-fit">
          <input v-model="isPremium" type="checkbox" class="rounded" />
          <span class="flex items-center gap-1.5">
            <Lock v-if="isPremium" class="w-3.5 h-3.5 text-amber-600" />
            <Unlock v-else class="w-3.5 h-3.5 text-emerald-600" />
            Konten premium (perlu di-assign ke Paket agar bisa diakses siswa)
          </span>
        </label>

        <label class="flex items-center gap-2 text-xs text-slate-600">
          <span class="font-medium">Jenis Simulasi</span>
          <select v-model="templateKind" class="px-2.5 py-1.5 border rounded-lg text-xs bg-white">
            <option value="utbk_combined">Skor gabungan (Simulasi UTBK)</option>
            <option value="per_subject">Skor per-mapel, tidak digabung (Simulasi TKA)</option>
          </select>
        </label>

        <label class="flex items-center gap-2 text-xs text-slate-600">
          <span class="font-medium">Jenjang</span>
          <select v-model="schoolTypeScope" class="px-2.5 py-1.5 border rounded-lg text-xs bg-white">
            <option :value="null">Semua jenjang</option>
            <option value="sma">SMA</option>
            <option value="smk">SMK</option>
            <option value="ma">MA</option>
            <option value="smp">SMP</option>
          </select>
        </label>

        <label class="flex items-center gap-2 text-xs text-slate-600">
          <span class="font-medium">Mode Terkunci</span>
          <select
            :value="lockdownOverride === null ? 'inherit' : lockdownOverride ? 'on' : 'off'"
            class="px-2.5 py-1.5 border rounded-lg text-xs bg-white"
            @change="lockdownOverride = ($event.target as HTMLSelectElement).value === 'inherit' ? null : ($event.target as HTMLSelectElement).value === 'on'"
          >
            <option value="inherit">Ikuti default global ({{ lockdownDefault ? 'Aktif' : 'Nonaktif' }})</option>
            <option value="on">Paksa Aktif</option>
            <option value="off">Paksa Nonaktif</option>
          </select>
        </label>
      </div>
      <p v-if="templateKind === 'per_subject'" class="text-xs text-amber-700 bg-amber-50 border border-amber-100 rounded-lg px-3 py-2">
        Skor akhir TIDAK akan digabung jadi satu angka — setiap mata uji dilaporkan terpisah, sesuai struktur TKA asli.
      </p>
      <p v-if="schoolTypeScope" class="text-xs text-emerald-700 bg-emerald-50 border border-emerald-100 rounded-lg px-3 py-2">
        Hanya tampil ke siswa jenjang {{ schoolTypeScope.toUpperCase() }} — siswa jenjang lain tidak akan melihat template ini di katalognya.
      </p>

      <div>
        <div class="flex items-center justify-between mb-2">
          <label class="text-xs font-medium text-slate-600">Urutan Subtes</label>
          <Button size="sm" variant="outline" class="text-xs gap-1.5" :disabled="sessionsLoading" @click="addRow">
            <Plus class="w-3.5 h-3.5" />Tambah Subtes
          </Button>
        </div>

        <div v-if="sessionsLoading" class="p-6 text-center text-xs text-muted-foreground">Memuat daftar sesi...</div>
        <div v-else-if="rows.length === 0" class="p-6 text-center text-xs text-muted-foreground border border-dashed rounded-lg">
          Belum ada subtes. Klik "Tambah Subtes" untuk mulai menyusun urutan.
        </div>
        <div v-else class="space-y-2">
          <div v-for="(row, i) in rows" :key="i" class="flex items-center gap-2 p-2.5 border rounded-lg bg-slate-50 flex-wrap">
            <span class="w-6 h-6 rounded-full bg-indigo-100 text-indigo-700 text-xs font-bold flex items-center justify-center shrink-0">{{ i + 1 }}</span>
            <select v-if="!row.is_elective" v-model="row.session_id" class="flex-1 min-w-0 px-2.5 py-1.5 border rounded-lg text-xs bg-white">
              <option value="" disabled>Pilih sesi...</option>
              <option v-for="s in availableSessions" :key="s.id" :value="s.id">
                {{ s.title }}{{ s.subject_filter ? ` (${s.subject_filter})` : '' }}
              </option>
            </select>
            <span
              v-else
              class="flex-1 min-w-0 px-2.5 py-1.5 border border-dashed border-violet-300 rounded-lg text-xs bg-violet-50 text-violet-700 flex items-center gap-1.5"
            >
              <ListChecks class="w-3.5 h-3.5 shrink-0" />Mapel Pilihan — sesi diisi otomatis dari pilihan siswa
            </span>
            <label class="flex items-center gap-1 text-[10px] text-violet-700 shrink-0 cursor-pointer" title="Sesi diisi otomatis dari pilihan mapel siswa, bukan sesi tetap">
              <input type="checkbox" class="rounded" :checked="row.is_elective" @change="toggleRowElective(row)" />
              Mapel Pilihan
            </label>
            <div class="flex items-center gap-1 shrink-0">
              <input
                v-model.number="row.break_seconds"
                type="number"
                min="0"
                step="30"
                class="w-20 px-2 py-1.5 border rounded-lg text-xs bg-white"
                title="Jeda setelah subtes ini (detik)"
              />
              <span class="text-[10px] text-muted-foreground">detik jeda</span>
            </div>
            <div class="flex items-center gap-0.5 shrink-0">
              <button class="p-1 rounded hover:bg-slate-200 disabled:opacity-30" :disabled="i === 0" title="Naikkan" @click="moveUp(i)">
                <ChevronUp class="w-4 h-4 text-slate-500" />
              </button>
              <button class="p-1 rounded hover:bg-slate-200 disabled:opacity-30" :disabled="i === rows.length - 1" title="Turunkan" @click="moveDown(i)">
                <ChevronDown class="w-4 h-4 text-slate-500" />
              </button>
              <button class="p-1 rounded hover:bg-red-50" title="Hapus" @click="removeRow(i)">
                <Trash2 class="w-4 h-4 text-red-500" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <div v-if="hasElectiveRow" class="p-3 border border-violet-200 bg-violet-50 rounded-lg space-y-1.5">
        <label class="text-xs font-medium text-violet-800 flex items-center gap-1.5"><ListChecks class="w-3.5 h-3.5" />Paket Sumber Mapel Pilihan</label>
        <select v-model="electivePackageId" class="w-full px-2.5 py-1.5 border rounded-lg text-xs bg-white" :disabled="packagesLoading">
          <option value="" disabled>Pilih paket...</option>
          <option v-for="p in packages" :key="p.id" :value="p.id">{{ p.name }} (Jumlah Mapel Pilihan: {{ p.elective_pick_count }})</option>
        </select>
        <p class="text-[11px] text-violet-700">
          Setiap slot "Mapel Pilihan" di atas akan diisi otomatis sesuai pilihan mapel siswa untuk paket ini. Jumlah slot Mapel Pilihan harus sama dengan "Jumlah Mapel Pilihan" paket tersebut.
        </p>
      </div>

      <div class="flex items-center gap-4 text-xs text-slate-600 bg-indigo-50 border border-indigo-100 rounded-lg px-3 py-2">
        <span class="flex items-center gap-1.5"><Layers class="w-3.5 h-3.5 text-indigo-600" />Jumlah subtes: <strong>{{ slotCount }}</strong></span>
        <span class="flex items-center gap-1.5"><Clock class="w-3.5 h-3.5 text-indigo-600" />Total durasi: <strong>{{ hasElectiveRow ? '≈' : '' }}{{ formatDuration(totalDurationMinutes) }}</strong></span>
      </div>

      <div class="flex justify-end gap-2">
        <Button variant="outline" :disabled="saving" @click="resetForm">Batal</Button>
        <Button variant="gradient" :disabled="saving" @click="submitTemplate">{{ saving ? 'Menyimpan...' : 'Simpan Template' }}</Button>
      </div>
    </Card>

    <div v-if="templatesLoading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
    <Card v-else-if="templates.length === 0" class="p-10 text-center text-sm text-muted-foreground">
      Belum ada template simulasi. Buat yang pertama di atas.
    </Card>
    <div v-else class="space-y-2.5">
      <Card v-for="t in templates" :key="t.id" class="p-4">
        <div class="flex items-start justify-between gap-3 flex-wrap mb-3">
          <div class="flex items-center gap-2 flex-wrap">
            <h3 class="font-bold text-sm text-slate-900">{{ t.title }}</h3>
            <Badge :variant="t.is_active ? 'success' : 'secondary'">{{ t.is_active ? 'Aktif' : 'Nonaktif' }}</Badge>
            <Badge :variant="t.is_premium ? 'warning' : 'secondary'" class="flex items-center gap-1">
              <Lock v-if="t.is_premium" class="w-3 h-3" /><Unlock v-else class="w-3 h-3" />
              {{ t.is_premium ? 'Premium' : 'Gratis' }}
            </Badge>
            <Badge variant="secondary" :class="t.exam_track === 'tka' ? '!bg-violet-100 !text-violet-700' : '!bg-sky-100 !text-sky-700'">
              {{ t.exam_track === 'tka' ? 'TKA' : 'SNBT/UTBK' }}
            </Badge>
            <Badge v-if="t.template_kind === 'per_subject'" variant="secondary" class="flex items-center gap-1 !bg-violet-100 !text-violet-700">
              <ListChecks class="w-3 h-3" />Skor Per-Mapel
            </Badge>
            <Badge v-if="effectiveLockdown(t)" variant="secondary" class="flex items-center gap-1 !bg-rose-100 !text-rose-700">
              <ShieldAlert class="w-3 h-3" />Mode Terkunci{{ t.lockdown_override === null || t.lockdown_override === undefined ? ' (default)' : '' }}
            </Badge>
          </div>
          <div class="flex items-center gap-1.5 shrink-0 flex-wrap">
            <select
              :value="t.lockdown_override === null || t.lockdown_override === undefined ? 'inherit' : t.lockdown_override ? 'on' : 'off'"
              :disabled="actionId === t.id"
              class="px-2 py-1.5 border rounded-lg text-[11px] bg-white disabled:opacity-50"
              title="Mode Terkunci untuk template ini"
              @change="setLockdown(t, ($event.target as HTMLSelectElement).value)"
            >
              <option value="inherit">Terkunci: ikuti default</option>
              <option value="on">Terkunci: paksa aktif</option>
              <option value="off">Terkunci: paksa nonaktif</option>
            </select>
            <Button
              size="sm"
              variant="outline"
              class="text-xs"
              :disabled="actionId === t.id"
              @click="togglePremium(t)"
            >
              {{ t.is_premium ? 'Jadikan Gratis' : 'Jadikan Premium' }}
            </Button>
            <Button
              size="sm"
              variant="outline"
              class="text-xs"
              :disabled="actionId === t.id"
              @click="toggleActive(t)"
            >
              {{ t.is_active ? 'Nonaktifkan' : 'Aktifkan' }}
            </Button>
            <button
              class="p-1.5 rounded hover:bg-red-50 disabled:opacity-40"
              title="Hapus"
              :disabled="actionId === t.id"
              @click="deleteTemplate(t)"
            >
              <Trash2 class="w-4 h-4 text-red-500" />
            </button>
          </div>
        </div>

        <div class="flex items-center gap-3 text-xs text-muted-foreground mb-3">
          <span class="flex items-center gap-1"><Clock class="w-3 h-3" />{{ formatDuration(t.total_duration_minutes) }}</span>
          <span class="flex items-center gap-1"><Layers class="w-3 h-3" />{{ t.slots.length }} subtes</span>
        </div>

        <div class="space-y-1">
          <div
            v-for="slot in t.slots"
            :key="slot.sequence_index"
            class="flex items-center gap-2 text-xs text-slate-700 bg-slate-50 rounded-lg px-2.5 py-1.5"
          >
            <span class="w-5 h-5 rounded-full bg-white border text-[10px] font-bold flex items-center justify-center shrink-0">{{ slot.sequence_index }}</span>
            <span v-if="slot.is_elective" class="font-medium flex-1 min-w-0 truncate text-violet-700 flex items-center gap-1">
              <ListChecks class="w-3.5 h-3.5 shrink-0" />Mapel Pilihan (sesuai pilihan siswa)
            </span>
            <span v-else class="font-medium flex-1 min-w-0 truncate">{{ slot.session_title }}</span>
            <span v-if="!slot.is_elective" class="shrink-0">{{ slot.duration_minutes }} menit, {{ slot.question_count }} soal</span>
            <span class="shrink-0 text-muted-foreground">Jeda {{ slot.break_seconds }}dtk</span>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>
