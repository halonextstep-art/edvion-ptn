<script setup lang="ts">
import {
  Plus, Tag, Search, Copy, X, Save, ToggleLeft, ToggleRight, Ticket, Percent,
  School, Share2, CheckCircle2, TrendingUp, ChevronDown, Trash2, Clock, Info,
  ArrowRight, ArrowLeft,
} from 'lucide-vue-next'
import type { VoucherItem, VoucherPayload, VoucherType, VoucherStatus, PackageItem } from '~/types'

const { voucherService, packageService } = useApi()
const toast = useToast()

const vouchers = ref<VoucherItem[]>([])
const packages = ref<PackageItem[]>([])
const loading = ref(false)
const search = ref('')
const filterStatus = ref<VoucherStatus | 'all'>('all')
const tab = ref<VoucherType | 'semua'>('semua')
const expandedId = ref<string | null>(null)

const showForm = ref(false)
const showDelete = ref(false)
const deleteTarget = ref<VoucherItem | null>(null)
const step = ref<1 | 2>(1)

const TYPE_CONFIG: Record<VoucherType, { label: string; icon: any; color: string; bg: string; desc: string }> = {
  access: { label: 'Akses Individual', icon: Ticket, color: 'text-indigo-700', bg: 'bg-indigo-100', desc: 'Kode unik untuk 1 siswa' },
  bulk_school: { label: 'Bulk Sekolah', icon: School, color: 'text-blue-700', bg: 'bg-blue-100', desc: 'Batch kode untuk sekolah mitra' },
  promo: { label: 'Promo / Diskon', icon: Percent, color: 'text-emerald-700', bg: 'bg-emerald-100', desc: 'Diskon % atau nominal untuk kampanye' },
  referral: { label: 'Referral', icon: Share2, color: 'text-purple-700', bg: 'bg-purple-100', desc: 'Kode afiliasi dengan komisi' },
}
const STATUS_CONFIG: Record<VoucherStatus, { label: string; variant: any }> = {
  active: { label: 'Aktif', variant: 'success' },
  inactive: { label: 'Nonaktif', variant: 'secondary' },
  expired: { label: 'Kedaluwarsa', variant: 'destructive' },
  redeemed: { label: 'Terpakai', variant: 'secondary' },
}

function blankForm(): VoucherPayload & { quantity: number } {
  const defaultExpiry = new Date(Date.now() + 90 * 86400000).toISOString().split('T')[0]
  return {
    voucher_type: 'access', package_id: '', discount_type: 'full', discount_value: 0,
    max_uses: 1, expires_at: defaultExpiry, note: '', school_name: '', referrer_name: '',
    referrer_commission: 25000, quantity: 1,
  }
}
const form = ref(blankForm())

async function load() {
  loading.value = true
  try {
    const [vs, ps] = await Promise.all([voucherService.list(), packageService.list()])
    vouchers.value = vs
    packages.value = ps
  } catch (e: any) {
    toast.error('Gagal memuat daftar voucher', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

const filtered = computed(() => {
  let list = tab.value === 'semua' ? vouchers.value : vouchers.value.filter((v) => v.voucher_type === tab.value)
  if (filterStatus.value !== 'all') list = list.filter((v) => v.status === filterStatus.value)
  if (search.value.trim()) {
    const q = search.value.toLowerCase()
    list = list.filter((v) =>
      v.code.toLowerCase().includes(q) || v.note?.toLowerCase().includes(q) ||
      v.school_name?.toLowerCase().includes(q) || v.referrer_name?.toLowerCase().includes(q),
    )
  }
  return list
})

const stats = computed(() => {
  const active = vouchers.value.filter((v) => v.status === 'active').length
  const usedTotal = vouchers.value.reduce((s, v) => s + v.used_count, 0)
  const quotaLeft = vouchers.value.reduce((s, v) => s + Math.max(v.max_uses - v.used_count, 0), 0)
  return { active, usedTotal, quotaLeft, total: vouchers.value.length }
})

function openCreate() {
  form.value = blankForm()
  if (packages.value[0]) form.value.package_id = packages.value[0].id
  step.value = 1
  showForm.value = true
}

const selectedPackage = computed(() => packages.value.find((p) => p.id === form.value.package_id))
const discountedPrice = computed(() => {
  const base = selectedPackage.value?.sale_price ?? 0
  if (form.value.discount_type === 'full') return 0
  if (form.value.discount_type === 'percent') return Math.round(base * (1 - form.value.discount_value / 100))
  return Math.max(0, base - form.value.discount_value)
})

async function save() {
  try {
    const payload: VoucherPayload = {
      ...form.value,
      max_uses: form.value.voucher_type === 'access' ? 1 : form.value.max_uses,
      school_name: form.value.school_name || null,
      referrer_name: form.value.referrer_name || null,
      referrer_commission: form.value.voucher_type === 'referral' ? form.value.referrer_commission : null,
    }
    const created = await voucherService.create(payload)
    toast.success(created.length > 1 ? `${created.length} voucher berhasil digenerate` : `Voucher ${created[0]?.code} berhasil dibuat`)
    showForm.value = false
    load()
  } catch (e: any) {
    toast.error('Gagal membuat voucher', e?.message)
  }
}

async function toggleActive(v: VoucherItem) {
  try {
    await voucherService.setActive(v.id, !v.active)
    toast.success(`Voucher ${v.code} diperbarui`)
    load()
  } catch (e: any) {
    toast.error('Gagal mengubah status voucher', e?.message)
  }
}

function openDelete(v: VoucherItem) {
  if (v.used_count > 0) { toast.error('Voucher yang sudah dipakai tidak bisa dihapus'); return }
  deleteTarget.value = v
  showDelete.value = true
}
async function confirmDelete() {
  if (!deleteTarget.value) return
  try {
    await voucherService.remove(deleteTarget.value.id)
    toast.success(`Voucher ${deleteTarget.value.code} dihapus`)
    load()
  } catch (e: any) {
    toast.error('Gagal menghapus voucher', e?.message)
  }
}

function copyCode(code: string) {
  navigator.clipboard?.writeText(code)
  toast.success(`Kode ${code} disalin`)
}

function fmtRp(n: number) {
  return 'Rp ' + n.toLocaleString('id-ID')
}
function fmtDiscount(v: VoucherItem) {
  if (v.discount_type === 'full') return 'Gratis'
  if (v.discount_type === 'percent') return `Diskon ${v.discount_value}%`
  return `Potongan ${fmtRp(v.discount_value)}`
}

const TABS: { id: VoucherType | 'semua'; label: string }[] = [
  { id: 'semua', label: 'Semua' },
  { id: 'access', label: 'Akses Individual' },
  { id: 'bulk_school', label: 'Bulk Sekolah' },
  { id: 'promo', label: 'Promo' },
  { id: 'referral', label: 'Referral' },
]
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2"><Tag class="w-5 h-5 text-indigo-600" />Manajemen Voucher</h2>
        <p class="text-sm text-muted-foreground">Kelola voucher akses, bulk sekolah, promo, dan program referral</p>
      </div>
      <Button variant="gradient" :disabled="packages.length === 0" @click="openCreate"><Plus class="w-4 h-4" />Buat Voucher</Button>
    </div>
    <p v-if="packages.length === 0" class="text-xs text-amber-600 flex items-center gap-1.5"><Info class="w-3.5 h-3.5" />Tambahkan paket di tab Paket terlebih dahulu sebelum membuat voucher.</p>

    <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
      <Card class="p-4 flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-indigo-50 flex items-center justify-center shrink-0"><Ticket class="w-5 h-5 text-indigo-600" /></div>
        <div><div class="text-xl font-black text-indigo-600">{{ stats.total }}</div><div class="text-xs text-muted-foreground">Total Voucher</div></div>
      </Card>
      <Card class="p-4 flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-emerald-50 flex items-center justify-center shrink-0"><CheckCircle2 class="w-5 h-5 text-emerald-600" /></div>
        <div><div class="text-xl font-black text-emerald-600">{{ stats.active }}</div><div class="text-xs text-muted-foreground">Voucher Aktif</div></div>
      </Card>
      <Card class="p-4 flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-blue-50 flex items-center justify-center shrink-0"><TrendingUp class="w-5 h-5 text-blue-600" /></div>
        <div><div class="text-xl font-black text-blue-600">{{ stats.usedTotal }}</div><div class="text-xs text-muted-foreground">Total Digunakan</div></div>
      </Card>
      <Card class="p-4 flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-amber-50 flex items-center justify-center shrink-0"><Ticket class="w-5 h-5 text-amber-600" /></div>
        <div><div class="text-xl font-black text-amber-600">{{ stats.quotaLeft }}</div><div class="text-xs text-muted-foreground">Sisa Kuota</div></div>
      </Card>
    </div>
    <div class="flex items-center gap-1 p-1 bg-slate-100 rounded-xl w-fit flex-wrap">
      <button v-for="t in TABS" :key="t.id" class="px-4 py-2 rounded-lg text-sm font-semibold transition-all" :class="tab === t.id ? 'bg-white shadow-sm text-slate-900' : 'text-slate-500 hover:text-slate-700'" @click="tab = t.id">{{ t.label }}</button>
    </div>

    <Card class="p-4">
      <div class="flex gap-2 flex-wrap">
        <div class="relative flex-1 min-w-48">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input v-model="search" placeholder="Cari kode, sekolah, catatan..." class="w-full pl-9 pr-3 py-2 border rounded-md text-sm bg-white" />
        </div>
        <select v-model="filterStatus" class="px-3 py-2 border rounded-md text-sm bg-white">
          <option value="all">Semua Status</option>
          <option v-for="(cfg, key) in STATUS_CONFIG" :key="key" :value="key">{{ cfg.label }}</option>
        </select>
      </div>
    </Card>

    <div v-if="loading" class="p-10 text-center text-sm text-muted-foreground">Memuat...</div>
    <Card v-else-if="filtered.length === 0" class="p-10 text-center">
      <Tag class="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" />
      <p class="text-muted-foreground text-sm">{{ vouchers.length === 0 ? 'Belum ada voucher. Buat voucher pertama.' : 'Tidak ada voucher yang cocok.' }}</p>
    </Card>

    <div v-else class="space-y-2.5">
      <Card v-for="v in filtered" :key="v.id" class="overflow-hidden hover:shadow-sm transition-shadow">
        <div class="flex items-center gap-4 p-4 cursor-pointer" @click="expandedId = expandedId === v.id ? null : v.id">
          <div class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0" :class="TYPE_CONFIG[v.voucher_type].bg">
            <component :is="TYPE_CONFIG[v.voucher_type].icon" class="w-5 h-5" :class="TYPE_CONFIG[v.voucher_type].color" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 mb-1 flex-wrap">
              <span class="font-mono font-black text-slate-900 tracking-wider text-sm">{{ v.code }}</span>
              <Badge variant="outline">{{ TYPE_CONFIG[v.voucher_type].label }}</Badge>
              <Badge :variant="STATUS_CONFIG[v.status].variant">{{ STATUS_CONFIG[v.status].label }}</Badge>
            </div>
            <div class="flex items-center gap-2 text-xs text-muted-foreground flex-wrap">
              <span>{{ v.package_name }}</span><span>&middot;</span><span>{{ fmtDiscount(v) }}</span>
              <template v-if="v.school_name"><span>&middot;</span><span>{{ v.school_name }}</span></template>
              <template v-if="v.referrer_name"><span>&middot;</span><span>{{ v.referrer_name }}</span></template>
              <span>&middot;</span><span class="flex items-center gap-1"><Clock class="w-3 h-3" />Exp: {{ v.expires_at }}</span>
            </div>
          </div>
          <div class="shrink-0 text-right hidden sm:block">
            <p class="font-black text-sm text-slate-900">{{ v.used_count }}/{{ v.max_uses }}</p>
            <p class="text-xs text-muted-foreground">digunakan</p>
          </div>
          <ChevronDown class="w-4 h-4 text-slate-400 shrink-0 transition-transform" :class="expandedId === v.id ? 'rotate-180' : ''" />
        </div>
        <div v-if="expandedId === v.id" class="border-t bg-slate-50 px-4 py-3 flex items-center justify-between gap-3 flex-wrap">
          <div class="flex items-center gap-2 flex-wrap text-xs text-muted-foreground">
            <span v-if="v.note" class="italic">"{{ v.note }}"</span>
            <span v-if="v.referrer_commission">Komisi: {{ fmtRp(v.referrer_commission) }}/redeem</span>
          </div>
          <div class="flex items-center gap-2">
            <Button variant="outline" size="sm" class="gap-1.5" @click="copyCode(v.code)"><Copy class="w-3.5 h-3.5" />Salin Kode</Button>
            <button v-if="v.status === 'active' || v.status === 'inactive'" @click="toggleActive(v)">
              <component :is="v.active ? ToggleRight : ToggleLeft" class="w-7 h-7" :class="v.active ? 'text-emerald-500' : 'text-slate-300'" />
            </button>
            <Button v-if="v.used_count === 0" variant="ghost" size="sm" class="text-red-500 hover:bg-red-50" @click="openDelete(v)"><Trash2 class="w-4 h-4" /></Button>
          </div>
        </div>
      </Card>
    </div>

    <ConfirmDialog v-model="showDelete" title="Hapus voucher ini?" description="Tindakan ini tidak bisa dibatalkan." confirm-label="Hapus" @confirm="confirmDelete" />

    <!-- Create modal -->
    <Teleport to="body">
      <div v-if="showForm" class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4" @click="showForm = false">
        <Card class="w-full max-w-xl shadow-2xl flex flex-col max-h-[90vh]" @click.stop>
          <div class="flex items-center justify-between px-6 py-4 border-b shrink-0">
            <div><h3 class="font-bold text-slate-900">Buat Voucher Baru</h3><p class="text-xs text-muted-foreground mt-0.5">Langkah {{ step }} dari 2</p></div>
            <button class="p-1.5 rounded-lg hover:bg-slate-100" @click="showForm = false"><X class="w-4 h-4" /></button>
          </div>

          <div class="flex-1 overflow-y-auto px-6 py-5 space-y-5">
            <template v-if="step === 1">
              <div>
                <label class="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-2">Tipe Voucher</label>
                <div class="grid grid-cols-2 gap-2">
                  <button v-for="(cfg, key) in TYPE_CONFIG" :key="key" type="button"
                    class="flex items-start gap-3 p-3 rounded-xl border-2 text-left transition-all"
                    :class="form.voucher_type === key ? 'border-indigo-500 bg-indigo-50' : 'border-slate-200 hover:border-slate-300'"
                    @click="form.voucher_type = key">
                    <div class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0" :class="form.voucher_type === key ? 'bg-indigo-600' : 'bg-slate-100'">
                      <component :is="cfg.icon" class="w-4 h-4" :class="form.voucher_type === key ? 'text-white' : 'text-slate-500'" />
                    </div>
                    <div><p class="text-sm font-bold" :class="form.voucher_type === key ? 'text-indigo-700' : 'text-slate-700'">{{ cfg.label }}</p><p class="text-xs text-muted-foreground">{{ cfg.desc }}</p></div>
                  </button>
                </div>
              </div>

              <div>
                <label class="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Paket yang Diaktifkan</label>
                <select v-model="form.package_id" class="w-full border rounded-lg px-3 py-2 text-sm">
                  <option v-for="p in packages" :key="p.id" :value="p.id">{{ p.name }} — {{ p.sale_price === 0 ? 'Gratis' : fmtRp(p.sale_price) }}</option>
                </select>
              </div>

              <div v-if="form.voucher_type === 'access' || form.voucher_type === 'bulk_school'" class="grid grid-cols-2 gap-3">
                <div>
                  <label class="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">{{ form.voucher_type === 'bulk_school' ? 'Jumlah Kode' : 'Jumlah Voucher' }}</label>
                  <input v-model.number="form.quantity" type="number" min="1" max="500" class="w-full border rounded-lg px-3 py-2 text-sm" />
                </div>
                <div v-if="form.voucher_type === 'bulk_school'">
                  <label class="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Sekolah</label>
                  <input v-model="form.school_name" placeholder="SMA Negeri 1 ..." class="w-full border rounded-lg px-3 py-2 text-sm" />
                </div>
              </div>

              <div v-if="form.voucher_type === 'promo'">
                <label class="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Batas Penggunaan</label>
                <input v-model.number="form.max_uses" type="number" min="1" class="w-full border rounded-lg px-3 py-2 text-sm" />
              </div>

              <div v-if="form.voucher_type === 'referral'" class="grid grid-cols-2 gap-3">
                <div>
                  <label class="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Referrer</label>
                  <input v-model="form.referrer_name" placeholder="Nama afiliasi" class="w-full border rounded-lg px-3 py-2 text-sm" />
                </div>
                <div>
                  <label class="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Komisi per Redeem</label>
                  <input v-model.number="form.referrer_commission" type="number" min="0" class="w-full border rounded-lg px-3 py-2 text-sm" />
                </div>
              </div>
            </template>

            <template v-else>
              <div>
                <label class="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-2">Jenis Diskon</label>
                <div class="grid grid-cols-3 gap-2 mb-3">
                  <button v-for="[dt, label] in [['full','Gratis (100%)'],['percent','Diskon %'],['fixed','Potongan Rp']]" :key="dt" type="button"
                    class="py-2.5 rounded-xl border-2 text-sm font-semibold transition-colors"
                    :class="form.discount_type === dt ? 'border-indigo-500 bg-indigo-50 text-indigo-700' : 'border-slate-200 text-slate-500 hover:border-slate-300'"
                    @click="form.discount_type = dt as any; form.discount_value = 0">{{ label }}</button>
                </div>
                <input v-if="form.discount_type !== 'full'" v-model.number="form.discount_value" type="number" min="0" :max="form.discount_type === 'percent' ? 100 : undefined"
                  :placeholder="form.discount_type === 'percent' ? 'Persentase (0-100)' : 'Nominal potongan (Rp)'" class="w-full border rounded-lg px-3 py-2 text-sm" />
              </div>

              <div>
                <label class="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Tanggal Kedaluwarsa</label>
                <input v-model="form.expires_at" type="date" class="w-full border rounded-lg px-3 py-2 text-sm" />
              </div>

              <div>
                <label class="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Catatan Internal (opsional)</label>
                <input v-model="form.note" placeholder="contoh: Hadiah lomba, Campaign IG, ..." class="w-full border rounded-lg px-3 py-2 text-sm" />
              </div>

              <div class="p-4 bg-gradient-to-br from-indigo-50 to-purple-50 border-2 border-indigo-200 rounded-xl">
                <p class="text-xs font-bold text-indigo-500 uppercase tracking-wider mb-3">Preview Voucher</p>
                <div class="grid grid-cols-2 gap-2 text-xs">
                  <div class="bg-white rounded-lg p-2"><span class="text-muted-foreground">Harga Normal:</span> <strong>{{ fmtRp(selectedPackage?.sale_price ?? 0) }}</strong></div>
                  <div class="bg-white rounded-lg p-2"><span class="text-muted-foreground">Harga Voucher:</span> <strong class="text-emerald-600">{{ discountedPrice === 0 ? 'GRATIS' : fmtRp(discountedPrice) }}</strong></div>
                </div>
              </div>
            </template>
          </div>

          <div class="px-6 py-4 border-t flex gap-2 shrink-0">
            <template v-if="step === 1">
              <Button variant="outline" class="flex-1" @click="showForm = false">Batal</Button>
              <Button variant="gradient" class="flex-1 gap-1.5" @click="step = 2">Lanjut<ArrowRight class="w-4 h-4" /></Button>
            </template>
            <template v-else>
              <Button variant="outline" class="flex-1 gap-1.5" @click="step = 1"><ArrowLeft class="w-4 h-4" />Kembali</Button>
              <Button variant="gradient" class="flex-1 gap-1.5" @click="save"><Save class="w-4 h-4" />Generate Voucher</Button>
            </template>
          </div>
        </Card>
      </div>
    </Teleport>
  </div>
</template>
