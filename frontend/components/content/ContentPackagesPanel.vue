<script setup lang="ts">
// "Paket Saya" — akun Konten melihat draft paket yang mereka susun sendiri lewat wizard
// "Buat Paket + Subtes" (backend PackageService::list menyaring hanya paket milik actor untuk
// role content — lihat doc comment di sana). Konten tidak bisa mempublish/atur harga di sini —
// itu wewenang Admin (lihat PackageWizard.vue `canPublish`).
import { Sparkles, Package, AlertTriangle, Clock, CheckCircle2, Pencil, Trash2 } from 'lucide-vue-next'
import type { PackageItem } from '~/types'

const { packageService } = useApi()
const toast = useToast()

const packages = ref<PackageItem[]>([])
const loading = ref(false)
const showWizard = ref(false)
// null = mode "buat baru"; berisi id = mode "kelola paket yang sudah ada" (klik kartu) —
// lihat prop packageId di PackageWizard.vue.
const editingId = ref<string | null>(null)
function openCreate() {
  editingId.value = null
  showWizard.value = true
}
function openManage(p: PackageItem) {
  editingId.value = p.id
  showWizard.value = true
}

// Hapus paket — hanya untuk draft milik sendiri (backend: PackageService::delete menolak kalau
// paket sudah di-publish Admin/`active`, lihat komentar di sana). Dipakai untuk beres-beres
// draft kosong/duplikat sebelum diajukan ke Admin.
const showDelete = ref(false)
const deleteTarget = ref<PackageItem | null>(null)
function openDelete(p: PackageItem) {
  deleteTarget.value = p
  showDelete.value = true
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

async function load() {
  loading.value = true
  try {
    packages.value = await packageService.list()
    await loadContentCounts()
  } catch (e: any) {
    toast.error('Gagal memuat paket', e?.message)
  } finally {
    loading.value = false
  }
}
onMounted(load)

function fmt(n: number) {
  return n.toLocaleString('id-ID')
}
</script>

<template>
  <!-- Wizard mengambil alih seluruh area ini (bukan modal) — lihat komentar di PackageWizard.vue. -->
  <PackageWizard v-if="showWizard" :dark="true" :package-id="editingId" @close="showWizard = false" @saved="showWizard = false; load()" />
  <div v-else class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-3">
      <div>
        <h2 class="text-lg font-bold flex items-center gap-2 text-white"><Package class="w-5 h-5 text-violet-400" />Paket Saya</h2>
        <p class="text-sm text-white/40">Paket + subtes + soal yang Anda susun — Admin yang meninjau, melengkapi harga, dan mempublish ke landing page. Klik kartu untuk mengelola.</p>
      </div>
      <Button variant="gradient" class="gap-1.5" @click="openCreate"><Sparkles class="w-4 h-4" />Buat Paket + Subtes</Button>
    </div>

    <div v-if="loading" class="p-10 text-center text-sm text-white/40">Memuat...</div>
    <Card v-else-if="packages.length === 0" class="p-10 text-center bg-white/[0.03] border-white/10">
      <Package class="w-10 h-10 mx-auto mb-3 text-white/20" />
      <p class="text-white/40 text-sm">Belum ada paket yang Anda susun. Mulai dari "Buat Paket + Subtes".</p>
    </Card>

    <div v-else class="grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <Card
        v-for="p in packages" :key="p.id"
        class="p-4 space-y-3 bg-white/[0.03] border-white/10 cursor-pointer hover:border-violet-500/40 transition-colors"
        @click="openManage(p)"
      >
        <div class="flex items-start justify-between gap-2">
          <p class="font-semibold text-white">{{ p.name }}</p>
          <Badge :variant="p.active ? 'success' : 'warning'" class="shrink-0">
            <component :is="p.active ? CheckCircle2 : Clock" class="w-3 h-3 mr-1" />{{ p.active ? 'Live' : 'Menunggu Review Admin' }}
          </Badge>
        </div>
        <p class="text-xs text-white/40">{{ p.package_type }} &middot; Rp {{ fmt(p.sale_price) }}</p>
        <p v-if="contentCounts[p.id] === 0" class="text-[11px] text-red-300 flex items-center gap-1.5">
          <AlertTriangle class="w-3.5 h-3.5 shrink-0" />Belum ada subtes terpasang
        </p>
        <p v-else class="text-[11px] text-white/40">{{ contentCounts[p.id] ?? '…' }} subtes/konten terpasang</p>
        <div class="flex items-center justify-between gap-2 pt-1 border-t border-white/5">
          <p class="text-[11px] text-violet-300 flex items-center gap-1"><Pencil class="w-3 h-3" />Kelola subtes &amp; soal</p>
          <button
            v-if="!p.active"
            class="p-1 rounded hover:bg-red-500/10 shrink-0"
            title="Hapus paket draft ini"
            @click.stop="openDelete(p)"
          >
            <Trash2 class="w-3.5 h-3.5 text-red-400" />
          </button>
          <span v-else class="text-[10px] text-white/25" title="Sudah live — minta Admin untuk menghapus">Live, tak bisa dihapus</span>
        </div>
      </Card>
    </div>

    <ConfirmDialog
      v-model="showDelete"
      title="Hapus paket ini?"
      :description="`'${deleteTarget?.name}' beserta kaitannya ke subtes/simulasi akan dihapus. Subtes/soal yang sudah dibuat tidak ikut terhapus, hanya tautannya ke paket ini. Tindakan ini tidak bisa dibatalkan.`"
      confirm-label="Hapus"
      @confirm="confirmDelete"
    />
  </div>
</template>
