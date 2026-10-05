<script setup lang="ts">
// Manajemen Katalog PTN — CRUD manual untuk tabel ptn_programs (dasar fitur Rasionalisasi
// SNBP/SNBT). Selama ini katalog cuma bisa diperbarui lewat import CSV oleh developer
// (cargo run --bin import_ptn_catalog); layar ini memberi admin jalan resmi untuk menambah,
// mengoreksi, atau menghapus satu baris prodi tanpa perlu developer, memakai endpoint CRUD
// backend yang sudah ada (rationalization_service::create_program/update_program/
// delete_program — sudah admin-only lewat require_role, lihat rationalization_dto.rs).
import { Plus, Pencil, Trash2, GraduationCap, X, Save, Search, ChevronLeft, ChevronRight, ShieldAlert } from 'lucide-vue-next'
import type { PtnProgramItem, PtnProgramPayload } from '~/types'

const { rationalizationService } = useApi()
const toast = useToast()

const PAGE_SIZE = 20
const programs = ref<PtnProgramItem[]>([])
const total = ref(0)
const page = ref(1)
const loading = ref(false)

const search = ref('')
const filterJenjang = ref('')
const filterRumpun = ref('')
const rumpunOptions = ref<string[]>([])
const JENJANG_OPTIONS = ['S1', 'D4', 'D3', 'D2', 'S2', 'S3']

const totalPages = computed(() => Math.max(1, Math.ceil(total.value / PAGE_SIZE)))
const rangeStart = computed(() => (total.value === 0 ? 0 : (page.value - 1) * PAGE_SIZE + 1))
const rangeEnd = computed(() => Math.min(page.value * PAGE_SIZE, total.value))

async function load() {
  loading.value = true
  try {
    const res = await rationalizationService.listPrograms({
      search: search.value || undefined,
      jenjang: filterJenjang.value || undefined,
      rumpun: filterRumpun.value || undefined,
      page: page.value,
      page_size: PAGE_SIZE,
    })
    programs.value = res.items
    total.value = res.total
  } catch (e: any) {
    toast.error('Gagal memuat katalog PTN', e?.message)
  } finally {
    loading.value = false
  }
}

async function loadRumpun() {
  try {
    rumpunOptions.value = await rationalizationService.distinctRumpun()
  } catch {
    // non-kritis — filter rumpun cukup kosong kalau gagal, tabel utama tetap jalan
  }
}

let debounceTimer: ReturnType<typeof setTimeout>
watch([search, filterJenjang, filterRumpun], () => {
  clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    page.value = 1
    load()
  }, 300)
})
watch(page, load)

onMounted(() => {
  load()
  loadRumpun()
})

function goToPage(p: number) {
  if (p < 1 || p > totalPages.value || p === page.value) return
  page.value = p
}

// ─── Form tambah/edit ───────────────────────────────────────────────────────
function emptyForm(): PtnProgramPayload {
  return {
    nama_ptn: '', nama_prodi: '', provinsi: '', kota: '', singkatan: '', rumpun: '',
    mapel_syarat: '', daya_tampung_snbp: 0, peminat_snbp: 0, daya_tampung_snbt: 0,
    peminat_snbt: 0, pg_snbt: 0, pg_snbp: 0, jenjang: 'S1', has_official_stats: true,
  }
}
const showForm = ref(false)
const editTarget = ref<PtnProgramItem | null>(null)
const form = ref<PtnProgramPayload>(emptyForm())
const saving = ref(false)

function openCreate() {
  editTarget.value = null
  form.value = emptyForm()
  showForm.value = true
}
function openEdit(p: PtnProgramItem) {
  editTarget.value = p
  form.value = {
    nama_ptn: p.nama_ptn, nama_prodi: p.nama_prodi, provinsi: p.provinsi, kota: p.kota,
    singkatan: p.singkatan, rumpun: p.rumpun, mapel_syarat: p.mapel_syarat,
    daya_tampung_snbp: p.daya_tampung_snbp, peminat_snbp: p.peminat_snbp,
    daya_tampung_snbt: p.daya_tampung_snbt, peminat_snbt: p.peminat_snbt,
    pg_snbt: p.pg_snbt, pg_snbp: p.pg_snbp, jenjang: p.jenjang, has_official_stats: p.has_official_stats,
  }
  showForm.value = true
}

async function saveForm() {
  if (!form.value.nama_ptn.trim() || !form.value.nama_prodi.trim()) {
    toast.error('Nama PTN dan Nama Program Studi wajib diisi')
    return
  }
  saving.value = true
  try {
    if (editTarget.value) {
      await rationalizationService.updateProgram(editTarget.value.id, form.value)
      toast.success(`Prodi "${form.value.nama_prodi}" berhasil diperbarui`)
    } else {
      await rationalizationService.createProgram(form.value)
      toast.success(`Prodi "${form.value.nama_prodi}" berhasil ditambahkan`)
    }
    showForm.value = false
    load()
    loadRumpun()
  } catch (e: any) {
    toast.error('Gagal menyimpan data prodi', e?.message)
  } finally {
    saving.value = false
  }
}

const showDelete = ref(false)
const deleteTarget = ref<PtnProgramItem | null>(null)
function openDelete(p: PtnProgramItem) {
  deleteTarget.value = p
  showDelete.value = true
}
async function confirmDelete() {
  if (!deleteTarget.value) return
  try {
    await rationalizationService.deleteProgram(deleteTarget.value.id)
    toast.success(`Prodi "${deleteTarget.value.nama_prodi}" dihapus`)
    load()
  } catch (e: any) {
    toast.error('Gagal menghapus data prodi', e?.message)
  }
}
</script>

<template>
  <div class="space-y-5">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2"><GraduationCap class="w-5 h-5 text-indigo-600" />Katalog PTN</h2>
        <p class="text-sm text-muted-foreground">Kelola daftar PTN &amp; program studi (dipakai fitur Rasionalisasi SNBP/SNBT) — {{ total }} prodi terdaftar</p>
      </div>
      <Button variant="gradient" @click="openCreate"><Plus class="w-4 h-4" />Tambah Prodi</Button>
    </div>

    <p class="text-xs text-muted-foreground flex items-start gap-1.5 bg-amber-50 border border-amber-100 rounded-lg p-3">
      <ShieldAlert class="w-3.5 h-3.5 shrink-0 mt-0.5 text-amber-600" />
      Data daya tampung/peminat/passing grade harus berasal dari sumber resmi (bukan perkiraan) — kosongkan/nolkan dan matikan "Data Resmi Lengkap" bila prodi ini baru diketahui namanya saja, supaya sistem tidak menghitung estimasi peluang palsu untuk prodi tersebut.
    </p>

    <div class="flex flex-wrap items-center gap-2">
      <div class="relative flex-1 min-w-[220px]">
        <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
        <Input v-model="search" placeholder="Cari nama PTN, prodi, atau singkatan..." class="pl-9" />
      </div>
      <select v-model="filterJenjang" class="px-3 py-2 border rounded-lg text-sm bg-white">
        <option value="">Semua Jenjang</option>
        <option v-for="j in JENJANG_OPTIONS" :key="j" :value="j">{{ j }}</option>
      </select>
      <select v-model="filterRumpun" class="px-3 py-2 border rounded-lg text-sm bg-white">
        <option value="">Semua Rumpun</option>
        <option v-for="r in rumpunOptions" :key="r" :value="r">{{ r }}</option>
      </select>
    </div>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
    <Card v-else-if="programs.length === 0" class="p-10 text-center">
      <GraduationCap class="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" />
      <p class="text-muted-foreground text-sm">Tidak ada prodi yang cocok dengan filter ini.</p>
    </Card>

    <template v-else>
      <Card class="overflow-x-auto">
        <table class="w-full text-sm min-w-[900px]">
          <thead class="bg-slate-50 text-xs uppercase text-slate-500 border-b">
            <tr>
              <th class="text-left py-2.5 px-4">PTN</th>
              <th class="text-left py-2.5 px-4">Program Studi</th>
              <th class="text-left py-2.5 px-4">Jenjang</th>
              <th class="text-left py-2.5 px-4">Rumpun</th>
              <th class="text-left py-2.5 px-4">DT/Peminat SNBP</th>
              <th class="text-left py-2.5 px-4">DT/Peminat SNBT</th>
              <th class="text-left py-2.5 px-4">Status Data</th>
              <th class="text-right py-2.5 px-4">Aksi</th>
            </tr>
          </thead>
          <tbody class="divide-y">
            <tr v-for="p in programs" :key="p.id" class="hover:bg-slate-50">
              <td class="py-2.5 px-4">
                <div class="font-medium text-slate-900">{{ p.nama_ptn }}</div>
                <div class="text-xs text-muted-foreground">{{ p.singkatan }} · {{ p.kota }}</div>
              </td>
              <td class="py-2.5 px-4">{{ p.nama_prodi }}</td>
              <td class="py-2.5 px-4"><Badge variant="outline">{{ p.jenjang }}</Badge></td>
              <td class="py-2.5 px-4 text-xs text-muted-foreground max-w-[160px] truncate" :title="p.rumpun">{{ p.rumpun || '-' }}</td>
              <td class="py-2.5 px-4 text-xs">{{ p.daya_tampung_snbp }} / {{ p.peminat_snbp }}</td>
              <td class="py-2.5 px-4 text-xs">{{ p.daya_tampung_snbt }} / {{ p.peminat_snbt }}</td>
              <td class="py-2.5 px-4">
                <Badge v-if="p.has_official_stats" variant="success">Resmi</Badge>
                <Badge v-else class="bg-slate-100 text-slate-500 border-0">Belum Lengkap</Badge>
              </td>
              <td class="py-2.5 px-4">
                <div class="flex items-center justify-end gap-1">
                  <button class="p-1.5 rounded hover:bg-slate-200" title="Edit" @click="openEdit(p)"><Pencil class="w-4 h-4 text-slate-500" /></button>
                  <button class="p-1.5 rounded hover:bg-red-50" title="Hapus" @click="openDelete(p)"><Trash2 class="w-4 h-4 text-red-500" /></button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </Card>

      <div v-if="totalPages > 1" class="flex items-center justify-between gap-3 pt-1">
        <p class="text-xs text-muted-foreground">Menampilkan {{ rangeStart }}–{{ rangeEnd }} dari {{ total }} prodi</p>
        <div class="flex items-center gap-1.5">
          <button class="p-1.5 rounded-lg border bg-white text-slate-500 hover:text-slate-800 disabled:opacity-40" :disabled="page === 1" @click="goToPage(page - 1)"><ChevronLeft class="w-4 h-4" /></button>
          <span class="text-xs font-semibold text-slate-600 px-2">Halaman {{ page }} / {{ totalPages }}</span>
          <button class="p-1.5 rounded-lg border bg-white text-slate-500 hover:text-slate-800 disabled:opacity-40" :disabled="page === totalPages" @click="goToPage(page + 1)"><ChevronRight class="w-4 h-4" /></button>
        </div>
      </div>
    </template>

    <!-- Form tambah/edit -->
    <Teleport to="body">
      <div v-if="showForm" class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4 overflow-y-auto" @click="showForm = false">
        <Card class="w-full max-w-xl shadow-2xl my-8" @click.stop>
          <div class="flex items-center justify-between px-6 py-4 border-b">
            <h3 class="font-bold text-slate-900">{{ editTarget ? 'Edit Prodi' : 'Tambah Prodi Baru' }}</h3>
            <button class="p-1.5 rounded-lg hover:bg-slate-100" @click="showForm = false"><X class="w-4 h-4" /></button>
          </div>
          <div class="px-6 py-5 space-y-4 max-h-[70vh] overflow-y-auto">
            <div class="grid sm:grid-cols-2 gap-4">
              <div>
                <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Nama PTN</Label>
                <Input v-model="form.nama_ptn" placeholder="contoh: Universitas Gadjah Mada" />
              </div>
              <div>
                <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Singkatan</Label>
                <Input v-model="form.singkatan" placeholder="contoh: UGM" />
              </div>
            </div>
            <div>
              <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Program Studi</Label>
              <Input v-model="form.nama_prodi" placeholder="contoh: Pendidikan Dokter" />
            </div>
            <div class="grid sm:grid-cols-2 gap-4">
              <div>
                <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Provinsi</Label>
                <Input v-model="form.provinsi" placeholder="contoh: Prov. D.I. Yogyakarta" />
              </div>
              <div>
                <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Kota</Label>
                <Input v-model="form.kota" placeholder="contoh: Kota Yogyakarta" />
              </div>
            </div>
            <div class="grid sm:grid-cols-2 gap-4">
              <div>
                <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Jenjang</Label>
                <select v-model="form.jenjang" class="w-full px-3 py-2 border rounded-lg text-sm bg-white">
                  <option v-for="j in JENJANG_OPTIONS" :key="j" :value="j">{{ j }}</option>
                </select>
              </div>
              <div>
                <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Rumpun</Label>
                <Input v-model="form.rumpun" placeholder="contoh: Ilmu atau Sains Kedokteran" />
              </div>
            </div>
            <div>
              <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Mapel Syarat (pisahkan dengan koma)</Label>
              <Input v-model="form.mapel_syarat" placeholder="contoh: Biologi, Kimia" />
            </div>
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
              <div>
                <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Daya Tampung SNBP</Label>
                <Input v-model.number="form.daya_tampung_snbp" type="number" min="0" />
              </div>
              <div>
                <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Peminat SNBP</Label>
                <Input v-model.number="form.peminat_snbp" type="number" min="0" />
              </div>
              <div>
                <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Daya Tampung SNBT</Label>
                <Input v-model.number="form.daya_tampung_snbt" type="number" min="0" />
              </div>
              <div>
                <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Peminat SNBT</Label>
                <Input v-model.number="form.peminat_snbt" type="number" min="0" />
              </div>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Passing Grade SNBP</Label>
                <Input v-model.number="form.pg_snbp" type="number" min="0" max="100" step="0.01" />
              </div>
              <div>
                <Label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Passing Grade SNBT</Label>
                <Input v-model.number="form.pg_snbt" type="number" min="0" max="100" step="0.01" />
              </div>
            </div>
            <div class="flex items-center gap-2.5 pt-1">
              <button
                type="button"
                class="w-9 h-5 rounded-full transition-colors relative shrink-0"
                :class="form.has_official_stats ? 'bg-emerald-500' : 'bg-slate-200'"
                @click="form.has_official_stats = !form.has_official_stats"
              >
                <span class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all" :class="form.has_official_stats ? 'left-4' : 'left-0.5'" />
              </button>
              <div class="text-xs">
                <p class="font-semibold text-slate-700">Data Resmi Lengkap</p>
                <p class="text-muted-foreground">Matikan bila daya tampung/peminat/PG di atas belum terverifikasi resmi — sistem akan tampilkan "Data belum lengkap" ke siswa, bukan estimasi peluang.</p>
              </div>
            </div>
          </div>
          <div class="px-6 py-4 border-t flex gap-2">
            <Button variant="outline" class="flex-1" @click="showForm = false">Batal</Button>
            <Button variant="gradient" class="flex-1 gap-1.5" :disabled="saving" @click="saveForm"><Save class="w-4 h-4" />Simpan</Button>
          </div>
        </Card>
      </div>
    </Teleport>

    <ConfirmDialog
      v-model="showDelete"
      title="Hapus prodi ini?"
      description="Data prodi ini akan terhapus permanen dari katalog. Target siswa yang sudah memilih prodi ini sebagai pilihan Rasionalisasi bisa ikut terpengaruh. Tindakan ini tidak bisa dibatalkan."
      confirm-label="Hapus"
      @confirm="confirmDelete"
    />
  </div>
</template>
