<script setup lang="ts">
import { CheckCircle, Info, Loader2, ShoppingBag, ArrowLeft, ClipboardCheck } from 'lucide-vue-next'
import type { InitiatePurchaseResult, PackageItem } from '~/types'

// Accessible by Student or School (matches backend `initiate_purchase`'s
// `require_role(&[Student, School])` gate) — anyone else gets redirected to `/` by the
// shared auth middleware.
definePageMeta({ middleware: 'auth', roles: ['student', 'school'] })

const route = useRoute()
const { publicService, paymentService } = useApi()
const { user } = useAuth()
const toast = useToast()
const { resolve: resolveUploadUrl } = useUploadUrl()

const packageId = computed(() => {
  const raw = route.query.packageId
  return typeof raw === 'string' && raw ? raw : null
})

const loadingPackage = ref(true)
const pkg = ref<PackageItem | null>(null)
const loadError = ref('')

async function loadPackage() {
  if (!packageId.value) {
    loadingPackage.value = false
    return
  }
  loadingPackage.value = true
  loadError.value = ''
  try {
    // No authenticated single-package getter exists for non-admin roles
    // (PackageService.get is admin-only) — the public catalog is the real source here.
    const all = await publicService.packages()
    const found = all.find((p) => p.id === packageId.value)
    if (!found) {
      loadError.value = 'Paket tidak ditemukan atau sudah tidak aktif.'
    } else {
      pkg.value = found
    }
  } catch (e: any) {
    loadError.value = e?.message ?? 'Gagal memuat detail paket.'
  } finally {
    loadingPackage.value = false
  }
}
onMounted(loadPackage)

const submitting = ref(false)
const submitError = ref('')
const result = ref<InitiatePurchaseResult | null>(null)

async function submitPurchase() {
  if (!packageId.value || submitting.value) return
  submitting.value = true
  submitError.value = ''
  try {
    const res = await paymentService.initiateCheckout(packageId.value)
    if (res.mode === 'checkout') {
      // Future-proofing: once a real gateway is configured, hand off to it directly.
      window.location.href = res.checkout.redirect_url
      return
    }
    result.value = res
    toast.success('Pesanan diajukan', 'Menunggu konfirmasi admin.')
  } catch (e: any) {
    submitError.value = e?.message ?? 'Gagal mengajukan pembelian.'
    toast.error('Gagal mengajukan pembelian', e?.message)
  } finally {
    submitting.value = false
  }
}

function fmtRp(n: number) {
  return 'Rp ' + n.toLocaleString('id-ID')
}

const dashboardPath = computed(() => {
  if (user.value?.role === 'school') return '/sekolah'
  return '/siswa'
})
</script>

<template>
  <div class="min-h-screen bg-slate-50 py-10 px-4">
    <div class="max-w-xl mx-auto">
      <NuxtLink :to="dashboardPath" class="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground mb-6">
        <ArrowLeft class="w-4 h-4" /> Kembali ke Dashboard
      </NuxtLink>

      <div class="mb-6 text-center">
        <div class="w-12 h-12 rounded-2xl bg-gradient-to-br from-orange-500 to-amber-500 flex items-center justify-center mx-auto mb-3">
          <ShoppingBag class="w-6 h-6 text-white" />
        </div>
        <h1 class="text-xl font-bold text-slate-900">Konfirmasi Pembelian</h1>
      </div>

      <!-- Missing/invalid packageId -->
      <Card v-if="!packageId" class="p-8 text-center">
        <Info class="w-8 h-8 mx-auto mb-3 text-amber-500" />
        <p class="text-sm text-muted-foreground">Tidak ada paket yang dipilih. Silakan kembali ke halaman utama dan pilih paket terlebih dahulu.</p>
        <NuxtLink to="/"><Button class="mt-4" variant="outline">Ke Halaman Utama</Button></NuxtLink>
      </Card>

      <div v-else-if="loadingPackage" class="text-center text-sm text-muted-foreground py-10">Memuat detail paket...</div>

      <Card v-else-if="loadError" class="p-8 text-center">
        <Info class="w-8 h-8 mx-auto mb-3 text-red-500" />
        <p class="text-sm text-muted-foreground">{{ loadError }}</p>
        <NuxtLink to="/"><Button class="mt-4" variant="outline">Ke Halaman Utama</Button></NuxtLink>
      </Card>

      <!-- Success state (manual_pending) -->
      <Card v-else-if="result" class="p-8 text-center">
        <div class="w-14 h-14 rounded-full bg-emerald-100 flex items-center justify-center mx-auto mb-4">
          <CheckCircle class="w-7 h-7 text-emerald-600" />
        </div>
        <h2 class="font-bold text-slate-900 mb-1.5">Pesanan diajukan — tunggu konfirmasi admin</h2>
        <p class="text-sm text-muted-foreground mb-4">
          Admin akan menghubungi kamu untuk konfirmasi pembayaran, lalu membuka akses paket.
        </p>
        <div class="bg-slate-50 border rounded-xl p-3 text-left text-xs space-y-1 mb-5">
          <div class="flex justify-between"><span class="text-muted-foreground">ID Transaksi</span><span class="font-mono font-semibold">{{ result.transaction.id }}</span></div>
          <div class="flex justify-between"><span class="text-muted-foreground">Status</span><Badge variant="secondary">Menunggu Konfirmasi</Badge></div>
        </div>
        <NuxtLink :to="dashboardPath"><Button class="w-full">Kembali ke Dashboard</Button></NuxtLink>
      </Card>

      <!-- Order summary + submit -->
      <Card v-else class="p-6">
        <div class="flex items-start gap-4 mb-5">
          <div class="h-16 w-16 shrink-0 rounded-xl overflow-hidden bg-gradient-to-br flex items-center justify-center" :class="pkg?.gradient">
            <img v-if="pkg?.banner_url" :src="resolveUploadUrl(pkg.banner_url) ?? undefined" :alt="pkg.name" class="w-full h-full object-cover" />
            <IconDisplay v-else :icon-type="pkg?.icon_type" :icon-name="pkg?.icon_name" :icon-url="pkg?.icon_url" size="w-7 h-7 text-slate-700" />
          </div>
          <div class="flex-1 min-w-0">
            <h3 class="font-bold text-slate-900">{{ pkg?.name }}</h3>
            <div class="mt-1">
              <span v-if="pkg && pkg.discount > 0" class="text-slate-400 text-xs line-through mr-2">{{ fmtRp(pkg.original_price) }}</span>
              <span class="font-black text-lg text-orange-500">{{ fmtRp(pkg?.sale_price ?? 0) }}</span>
            </div>
          </div>
        </div>

        <ul v-if="pkg?.features?.length" class="space-y-1.5 mb-5">
          <li v-for="f in pkg.features" :key="f" class="flex items-start gap-1.5 text-xs text-slate-600">
            <CheckCircle class="w-3.5 h-3.5 text-emerald-500 mt-0.5 shrink-0" /> {{ f }}
          </li>
        </ul>

        <div class="flex gap-2.5 p-3 bg-blue-50 border border-blue-200 rounded-xl text-xs text-blue-800 mb-5">
          <ClipboardCheck class="w-4 h-4 shrink-0 mt-0.5" />
          <p>Pembayaran online belum tersedia. Setelah kamu mengajukan pesanan ini, admin akan menghubungi kamu (mis. via WhatsApp) untuk konfirmasi pembayaran, lalu membuka akses paket secara manual.</p>
        </div>

        <p v-if="submitError" class="text-xs text-red-600 mb-3">{{ submitError }}</p>

        <Button class="w-full" :disabled="submitting" @click="submitPurchase">
          <Loader2 v-if="submitting" class="w-4 h-4 animate-spin" />
          {{ submitting ? 'Memproses...' : 'Ajukan Pembelian' }}
        </Button>
      </Card>
    </div>
  </div>
</template>
