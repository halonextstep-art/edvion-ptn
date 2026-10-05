<script setup lang="ts">
import {
  Plus, Package, Pencil, Trash2, Eye, EyeOff, X, Save, CheckCircle2,
  Percent, LayoutGrid, List as ListIcon, Info, ListChecks, AlertTriangle, Sparkles,
} from 'lucide-vue-next'
import type { PackageItem, PackagePayload } from '~/types'

// Diteruskan dari PackageWizard (tombol "Kelola Sesi ini"/"Kelola Set Soal ini" per subtes) ke
// ContentHub.vue, yang menangkapnya untuk membuka sub-tab Lanjutan yang sesuai.
const emit = defineEmits<{ goToSession: [sessionId: string]; goToSet: [setId: string] }>()

const { packageService } = useApi()
const toast = useToast()
const { resolve: resolveUploadUrl } = useUploadUrl()
const showWizard = ref(false)
const editingWizardId = ref<string | null>(null)
function openWizardCreate() {
  editingWizardId.value = null
  showWizard.value = true
}
function openWizardManage(p: PackageItem) {
  editingWizardId.value = p.id
  showWizard.value = true
}

const packages = ref<PackageItem[]>([])
const loading = ref(false)
const view = ref<'grid' | 'table'>('grid')

const showForm = ref(false)
const editTarget = ref<PackageItem | null>(null)
const showDelete = ref(false)
const deleteTarget = ref<PackageItem | null>(null)
const showContent = ref(false)
const contentTarget = ref<PackageItem | null>(null)
function openContent(p: PackageItem) {
  contentTarget.value = p
  showContent.value = true
}
// Refresh "kosong"/jumlah konten badge begitu dialog Isi Paket ditutup, supaya perubahan
// langsung kelihatan tanpa perlu reload manual.
watch(showContent, (isOpen) => { if (!isOpen) loadContentCounts() })

const GRADIENT_OPTIONS = [
  { label: 'Biru', value: 'from-sky-200 to-blue-100', accent: '#3b82f6' },
  { label: 'Ungu', value: 'from-violet-200 to-purple-100', accent: '#7c3aed' },
  { label: 'Kuning', value: 'from-amber-200 to-orange-100', accent: '#f59e0b' },
  { label: 'Hijau', value: 'from-emerald-200 to-teal-100', accent: '#10b981' },
  { label: 'Merah Muda', value: 'from-rose-200 to-pink-100', accent: '#ec4899' },
  { label: 'Cyan', value: 'from-cyan-200 to-sky-100', accent: '#06b6d4' },
  { label: 'Indigo', value: 'from-indigo-200 to-blue-100', accent: '#6366f1' },
]

function blankForm(): PackagePayload {
  return {
    name: '', package_type: 'SNBT', original_price: 0, sale_price: 0,
    validity: '1 tahun', features: [''], badge: '',
    icon_type: 'preset', icon_name: 'package', icon_url: null, banner_url: null,
    gradient: GRADIENT_OPTIONS[0].value, accent_color: GRADIENT_OPTIONS[0].accent,
    active: true, sort_order: packages.value.length + 1,
    elective_pick_count: 0,
  }
}
const form = ref<PackagePayload>(blankForm())

// IconPicker v-model targets just the 3 icon fields — a computed get/set so picking an
// icon mutates those fields in place on `form` instead of replacing the whole payload
// object (which would drop name/price/features/etc).
const formIcon = computed<Pick<PackagePayload, 'icon_type' | 'icon_name' | 'icon_url'>>({
  get: () => ({ icon_type: form.value.icon_type, icon_name: form.value.icon_name, icon_url: form.value.icon_url }),
  set: (v) => {
    form.value.icon_type = v.icon_type
    form.value.icon_name = v.icon_name
    form.value.icon_url = v.icon_url
  },
})

// BannerPicker v-model — same get/set-into-place pattern as formIcon above, targeting the
// single optional banner_url field.
const formBanner = computed<string | null>({
  get: () => form.value.banner_url ?? null,
  set: (v) => { form.value.banner_url = v },
})

const discountPreview = computed(() => {
  if (form.value.original_price <= 0) return 0
  return Math.round((1 - form.value.sale_price / form.value.original_price) * 100)
})

// Jumlah konten (TryoutSession/SimulationTemplate) per paket — paket dengan 0 konten tidak
// akan bisa dibuka siapa pun yang memilikinya (akses sekarang per-konten, bukan all-or-nothing
// — lihat AccessService), jadi ini ditandai jelas di sini supaya tidak baru ketahuan setelah
// mitra/siswa komplain "sudah punya paket premium tapi tidak bisa dibuka".
const contentCounts = ref<Record<string, number>>({})
async function loadContentCounts() {
  const results = await Promise.all(
    packages.value.map(async (p) => {
      try {
        const items = await packageService.listContent(p.id)
        return [p.id, items.length] as const
      } catch {
        return [p.id, -1] as const
      }
    }),
  )
  contentCounts.value = Object.fromEntries(results)
}
function isEmpty(id: string) {
  return contentCounts.value[id] === 0
}

async function load() {
  loading.value = true
  try {
    packages.value = await packageService.list()
    await loadContentCounts()
  } catch (e: any) {
    toast.error('Gagal memuat daftar paket', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

const sorted = computed(() => [...packages.value].sort((a, b) => a.sort_order - b.sort_order))
const stats = computed(() => {
  const active = packages.value.filter((p) => p.active).length
  const avgDiscount = packages.value.length
    ? Math.round(packages.value.reduce((s, p) => s + p.discount, 0) / packages.value.length)
    : 0
  return { total: packages.value.length, active, avgDiscount }
})

function openCreate() {
  editTarget.value = null
  form.value = blankForm()
  showForm.value = true
}
function openEdit(p: PackageItem) {
  editTarget.value = p
  form.value = {
    name: p.name, package_type: p.package_type, original_price: p.original_price,
    sale_price: p.sale_price, validity: p.validity, features: p.features.length ? [...p.features] : [''],
    badge: p.badge, icon_type: p.icon_type, icon_name: p.icon_name, icon_url: p.icon_url,
    banner_url: p.banner_url ?? null,
    gradient: p.gradient, accent_color: p.accent_color,
    active: p.active, sort_order: p.sort_order,
    elective_pick_count: p.elective_pick_count ?? 0,
  }
  showForm.value = true
}
function openDelete(p: PackageItem) {
  deleteTarget.value = p
  showDelete.value = true
}

function addFeature() {
  form.value.features.push('')
}
function removeFeature(idx: number) {
  form.value.features.splice(idx, 1)
}

async function save() {
  if (!form.value.name.trim()) { toast.error('Nama paket wajib diisi'); return }
  if (form.value.original_price <= 0) { toast.error('Harga asli harus lebih dari 0'); return }
  if (form.value.sale_price <= 0) { toast.error('Harga jual harus lebih dari 0'); return }
  const payload: PackagePayload = {
    ...form.value,
    features: form.value.features.filter((f) => f.trim() !== ''),
  }
  try {
    if (editTarget.value) {
      await packageService.update(editTarget.value.id, payload)
      toast.success(`Paket "${payload.name}" berhasil diperbarui`)
    } else {
      await packageService.create(payload)
      toast.success(`Paket "${payload.name}" berhasil dibuat`)
    }
    showForm.value = false
    load()
  } catch (e: any) {
    toast.error('Gagal menyimpan paket', e?.message)
  }
}

async function toggleActive(p: PackageItem) {
  try {
    await packageService.setActive(p.id, !p.active)
    toast.success(`Paket "${p.name}" ${!p.active ? 'ditampilkan' : 'disembunyikan'} di landing page`)
    load()
  } catch (e: any) {
    toast.error('Gagal mengubah status paket', e?.message)
  }
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  try {
    await packageService.remove(deleteTarget.value.id)
    toast.success(`Paket "${deleteTarget.value.name}" dihapus`)
    load()
  } catch (e: any) {
    toast.error('Gagal menghapus paket', e?.message)
  }
}

function fmt(n: number) {
  return n.toLocaleString('id-ID')
}
</script>

<template>
  <!-- Wizard mengambil alih seluruh area ini (bukan modal di atasnya) supaya "Isi Soal" per
       subtes punya ruang sungguhan — lihat komentar di PackageWizard.vue. -->
  <PackageWizard
    v-if="showWizard" :package-id="editingWizardId"
    @close="showWizard = false" @saved="showWizard = false; load()"
    @go-to-session="emit('goToSession', $event)" @go-to-set="emit('goToSet', $event)"
  />
  <div v-else class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2"><Package class="w-5 h-5 text-indigo-600" />Manajemen Paket</h2>
        <p class="text-sm text-muted-foreground">Kelola katalog paket kuota tryout &amp; harga jual yang tampil di landing page</p>
      </div>
      <div class="flex items-center gap-2">
        <div class="flex items-center gap-1 p-1 bg-slate-100 rounded-lg">
          <button class="p-1.5 rounded-md" :class="view === 'grid' ? 'bg-white shadow-sm' : ''" @click="view = 'grid'"><LayoutGrid class="w-4 h-4" /></button>
          <button class="p-1.5 rounded-md" :class="view === 'table' ? 'bg-white shadow-sm' : ''" @click="view = 'table'"><ListIcon class="w-4 h-4" /></button>
        </div>
        <Button variant="outline" class="gap-1.5" @click="openWizardCreate"><Sparkles class="w-4 h-4 text-violet-500" />Buat Paket + Subtes</Button>
        <Button variant="gradient" @click="openCreate"><Plus class="w-4 h-4" />Tambah Paket</Button>
      </div>
    </div>

    <div class="grid grid-cols-2 lg:grid-cols-3 gap-4">
      <Card class="p-4 flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-indigo-50 flex items-center justify-center shrink-0"><Package class="w-5 h-5 text-indigo-600" /></div>
        <div><div class="text-xl font-black text-indigo-600">{{ stats.total }}</div><div class="text-xs text-muted-foreground">Total Paket</div></div>
      </Card>
      <Card class="p-4 flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-emerald-50 flex items-center justify-center shrink-0"><CheckCircle2 class="w-5 h-5 text-emerald-600" /></div>
        <div><div class="text-xl font-black text-emerald-600">{{ stats.active }}</div><div class="text-xs text-muted-foreground">Tampil di Landing Page</div></div>
      </Card>
      <Card class="p-4 flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-amber-50 flex items-center justify-center shrink-0"><Percent class="w-5 h-5 text-amber-600" /></div>
        <div><div class="text-xl font-black text-amber-600">{{ stats.avgDiscount }}%</div><div class="text-xs text-muted-foreground">Rata-rata Diskon</div></div>
      </Card>
    </div>
    <p class="text-xs text-muted-foreground -mt-3 flex items-center gap-1.5"><Info class="w-3.5 h-3.5" />Jumlah terjual per paket belum dihitung di sini — lihat ringkasan transaksi di tab Keuangan &amp; Permintaan Pembelian.</p>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
    <Card v-else-if="packages.length === 0" class="p-10 text-center">
      <Package class="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" />
      <p class="text-muted-foreground text-sm">Belum ada paket. Tambahkan paket pertama.</p>
    </Card>

    <div v-else-if="view === 'grid'" class="grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <Card v-for="p in sorted" :key="p.id" class="overflow-hidden relative" :class="!p.active ? 'opacity-60' : ''">
        <div v-if="p.banner_url" class="relative aspect-[3/1]">
          <img :src="resolveUploadUrl(p.banner_url) ?? undefined" :alt="p.name" class="w-full h-full object-cover" />
          <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-black/10 to-transparent" />
          <Badge v-if="p.badge" class="absolute top-3 right-3 bg-white/80 text-slate-800">{{ p.badge }}</Badge>
          <div class="absolute bottom-3 left-4 right-4">
            <h3 class="font-bold text-white drop-shadow-sm">{{ p.name }}</h3>
            <p class="text-xs text-white/90 mt-0.5">{{ p.package_type }} &middot; Berlaku {{ p.validity }}</p>
          </div>
        </div>
        <div v-else class="p-5 bg-gradient-to-br" :class="p.gradient">
          <div class="flex items-start justify-between mb-3">
            <div class="w-10 h-10 rounded-xl bg-white/70 flex items-center justify-center overflow-hidden"><IconDisplay :icon-type="p.icon_type" :icon-name="p.icon_name" :icon-url="p.icon_url" size="w-5 h-5 text-slate-700" /></div>
            <Badge v-if="p.badge" class="bg-white/80 text-slate-800">{{ p.badge }}</Badge>
          </div>
          <h3 class="font-bold text-slate-900">{{ p.name }}</h3>
          <p class="text-xs text-slate-600 mt-0.5">{{ p.package_type }} &middot; Berlaku {{ p.validity }}</p>
        </div>
        <div v-if="isEmpty(p.id)" class="px-5 py-2 bg-red-50 border-b border-red-200 flex items-center gap-1.5 text-[11px] text-red-700 font-medium">
          <AlertTriangle class="w-3.5 h-3.5 shrink-0" />Paket kosong — belum ada Sesi/Simulasi TO yang di-assign, siswa tidak bisa membuka apa pun
        </div>
        <div class="p-5 space-y-3">
          <div class="flex items-baseline gap-2">
            <span class="text-lg font-black" :style="{ color: p.accent_color }">Rp {{ fmt(p.sale_price) }}</span>
            <span class="text-xs text-muted-foreground line-through">Rp {{ fmt(p.original_price) }}</span>
            <Badge variant="outline" class="text-emerald-600 border-emerald-300">-{{ p.discount }}%</Badge>
          </div>
          <ul class="space-y-1">
            <li v-for="(f, i) in p.features" :key="i" class="text-xs text-slate-600 flex items-start gap-1.5">
              <CheckCircle2 class="w-3.5 h-3.5 text-emerald-500 shrink-0 mt-0.5" />{{ f }}
            </li>
          </ul>
          <div class="flex items-center justify-between pt-2 border-t">
            <button class="flex items-center gap-1.5 text-xs font-semibold" :class="p.active ? 'text-emerald-600' : 'text-slate-400'" @click="toggleActive(p)">
              <component :is="p.active ? Eye : EyeOff" class="w-3.5 h-3.5" />{{ p.active ? 'Tampil' : 'Tersembunyi' }}
            </button>
            <div class="flex items-center gap-1">
              <button class="p-1.5 rounded hover:bg-slate-100" title="Buat/kelola subtes &amp; soal baru lewat wizard" @click="openWizardManage(p)"><Sparkles class="w-4 h-4 text-violet-500" /></button>
              <button class="p-1.5 rounded hover:bg-slate-100" title="Pasang subtes/simulasi yang sudah ada ke paket ini" @click="openContent(p)"><ListChecks class="w-4 h-4 text-indigo-600" /></button>
              <button class="p-1.5 rounded hover:bg-slate-100" title="Edit" @click="openEdit(p)"><Pencil class="w-4 h-4 text-slate-500" /></button>
              <button class="p-1.5 rounded hover:bg-red-50" title="Hapus" @click="openDelete(p)"><Trash2 class="w-4 h-4 text-red-500" /></button>
            </div>
          </div>
        </div>
      </Card>
    </div>

    <Card v-else class="overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b bg-slate-50 text-xs text-muted-foreground">
            <th class="text-left px-4 py-3 font-semibold">Paket</th>
            <th class="text-center px-4 py-3 font-semibold">Harga</th>
            <th class="text-center px-4 py-3 font-semibold">Diskon</th>
            <th class="text-center px-4 py-3 font-semibold">Urutan</th>
            <th class="text-center px-4 py-3 font-semibold">Status</th>
            <th class="px-4 py-3" />
          </tr>
        </thead>
        <tbody class="divide-y">
          <tr v-for="p in sorted" :key="p.id" :class="!p.active ? 'opacity-50' : ''">
            <td class="px-4 py-3">
              <div class="flex items-center gap-2">
                <div class="w-8 h-8 rounded-lg bg-slate-100 flex items-center justify-center overflow-hidden shrink-0">
                  <img v-if="p.banner_url" :src="resolveUploadUrl(p.banner_url) ?? undefined" :alt="p.name" class="w-full h-full object-cover" />
                  <IconDisplay v-else :icon-type="p.icon_type" :icon-name="p.icon_name" :icon-url="p.icon_url" size="w-4 h-4 text-slate-600" />
                </div>
                <div>
                  <p class="font-semibold text-slate-900 flex items-center gap-1.5">
                    {{ p.name }}
                    <span v-if="isEmpty(p.id)" class="inline-flex items-center gap-0.5 text-[10px] text-red-600 font-bold"><AlertTriangle class="w-3 h-3" />Kosong</span>
                  </p>
                  <p class="text-xs text-muted-foreground">{{ p.package_type }}</p>
                </div>
              </div>
            </td>
            <td class="px-4 py-3 text-center">
              <div class="font-bold">Rp {{ fmt(p.sale_price) }}</div>
              <div class="text-xs text-muted-foreground line-through">Rp {{ fmt(p.original_price) }}</div>
            </td>
            <td class="px-4 py-3 text-center font-bold text-emerald-600">-{{ p.discount }}%</td>
            <td class="px-4 py-3 text-center text-muted-foreground">{{ p.sort_order }}</td>
            <td class="px-4 py-3 text-center">
              <button @click="toggleActive(p)">
                <Badge :variant="p.active ? 'success' : 'secondary'">{{ p.active ? 'Tampil' : 'Tersembunyi' }}</Badge>
              </button>
            </td>
            <td class="px-4 py-3 text-right">
              <button class="p-1.5 rounded hover:bg-slate-100" title="Buat/kelola subtes &amp; soal baru lewat wizard" @click="openWizardManage(p)"><Sparkles class="w-3.5 h-3.5 text-violet-500" /></button>
              <button class="p-1.5 rounded hover:bg-slate-100" title="Pasang subtes/simulasi yang sudah ada ke paket ini" @click="openContent(p)"><ListChecks class="w-3.5 h-3.5 text-indigo-600" /></button>
              <button class="p-1.5 rounded hover:bg-slate-100" @click="openEdit(p)"><Pencil class="w-3.5 h-3.5 text-slate-500" /></button>
              <button class="p-1.5 rounded hover:bg-red-50" @click="openDelete(p)"><Trash2 class="w-3.5 h-3.5 text-red-500" /></button>
            </td>
          </tr>
        </tbody>
      </table>
    </Card>

    <ConfirmDialog
      v-model="showDelete"
      title="Hapus paket ini?"
      description="Tindakan ini tidak bisa dibatalkan."
      confirm-label="Hapus"
      @confirm="confirmDelete"
    />

    <PackageContentDialog v-model="showContent" :pkg="contentTarget" />

    <!-- Create/Edit modal -->
    <Teleport to="body">
      <div v-if="showForm" class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" @click="showForm = false">
        <Card class="w-full max-w-lg shadow-2xl flex flex-col max-h-[90vh]" @click.stop>
          <div class="flex items-center justify-between px-6 py-4 border-b shrink-0">
            <h3 class="font-bold text-slate-900">{{ editTarget ? 'Edit Paket' : 'Tambah Paket Baru' }}</h3>
            <button class="p-1.5 rounded-lg hover:bg-slate-100" @click="showForm = false"><X class="w-4 h-4" /></button>
          </div>
          <div class="flex-1 overflow-y-auto px-6 py-5 space-y-4">
            <div>
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Paket</label>
              <input v-model="form.name" placeholder="contoh: 5 Kuota Tryout SNBT" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Tipe</label>
                <input v-model="form.package_type" placeholder="TKA / Full / Premium" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
              </div>
              <div>
                <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Masa Berlaku</label>
                <input v-model="form.validity" placeholder="1 tahun" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
              </div>
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Harga Asli (Rp)</label>
                <input v-model.number="form.original_price" type="number" min="0" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
              </div>
              <div>
                <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Harga Jual (Rp)</label>
                <input v-model.number="form.sale_price" type="number" min="0" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
              </div>
            </div>
            <p class="text-xs text-muted-foreground -mt-2">Diskon otomatis: <strong class="text-emerald-600">{{ discountPreview }}%</strong></p>

            <div>
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Jumlah Mapel Pilihan (0 = tidak ada batasan)</label>
              <input v-model.number="form.elective_pick_count" type="number" min="0" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
              <p class="text-xs text-muted-foreground mt-1">Contoh TKA SMA: isi 2 — siswa harus memilih tepat 2 dari subtes yang ditandai "Mapel Pilihan" (lihat Manajemen Sesi) sebelum bisa mengerjakannya.</p>
            </div>

            <div>
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Badge (opsional)</label>
              <input v-model="form.badge" placeholder="Terpopuler / Best Value" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
            </div>

            <div>
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-2">Icon</label>
              <IconPicker v-model="formIcon" />
            </div>

            <div>
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-2">Banner Paket (opsional)</label>
              <BannerPicker v-model="formBanner" />
              <p class="text-xs text-muted-foreground mt-1">Jika diisi, gambar ini tampil di bagian atas kartu paket menggantikan tampilan gradient+icon polos.</p>
            </div>

            <div>
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-2">Warna Gradient</label>
              <div class="flex flex-wrap gap-2">
                <button v-for="g in GRADIENT_OPTIONS" :key="g.value" type="button"
                  class="px-3 py-2 rounded-lg text-xs font-semibold border-2 bg-gradient-to-br"
                  :class="[g.value, form.gradient === g.value ? 'border-indigo-500' : 'border-transparent']"
                  @click="form.gradient = g.value; form.accent_color = g.accent">{{ g.label }}</button>
              </div>
            </div>

            <div>
              <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-2">Fitur</label>
              <div class="space-y-2">
                <div v-for="(f, idx) in form.features" :key="idx" class="flex items-center gap-2">
                  <input v-model="form.features[idx]" placeholder="contoh: 5x Tryout Full Simulasi" class="flex-1 border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                  <button type="button" class="p-1.5 rounded hover:bg-red-50" @click="removeFeature(idx)"><X class="w-4 h-4 text-red-500" /></button>
                </div>
                <Button variant="outline" size="sm" class="gap-1.5" @click="addFeature"><Plus class="w-3.5 h-3.5" />Tambah Fitur</Button>
              </div>
            </div>

            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Urutan Tampil</label>
                <input v-model.number="form.sort_order" type="number" min="1" class="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
              </div>
              <div class="flex items-center gap-3 p-3 bg-slate-50 rounded-lg mt-5">
                <button type="button" @click="form.active = !form.active">
                  <component :is="form.active ? Eye : EyeOff" class="w-6 h-6" :class="form.active ? 'text-emerald-500' : 'text-slate-300'" />
                </button>
                <p class="text-sm font-medium">{{ form.active ? 'Tampil di landing page' : 'Disembunyikan' }}</p>
              </div>
            </div>
          </div>
          <div class="px-6 py-4 border-t flex gap-2 shrink-0">
            <Button variant="outline" class="flex-1" @click="showForm = false">Batal</Button>
            <Button variant="gradient" class="flex-1 gap-1.5" @click="save"><Save class="w-4 h-4" />Simpan</Button>
          </div>
        </Card>
      </div>
    </Teleport>
  </div>
</template>
