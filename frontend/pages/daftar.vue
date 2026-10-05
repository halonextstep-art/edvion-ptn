<script setup lang="ts">
import { Rocket, Sparkles, Trophy, Target, Brain, Layers, Eye, EyeOff, ArrowLeft, Hand, PauseCircle } from 'lucide-vue-next'

definePageMeta({ layout: false })

// Public B2C (mandiri) self-registration — always creates a `Role::Student` account with no
// `school_id`. B2B students (enrolled through a partner school) are created by their school's
// admin instead (Manajemen Siswa / CSV import), never through this page.
const { register } = useAuth()
const { publicService } = useApi()
const toast = useToast()
const router = useRouter()
const route = useRoute()

const resumingPurchase = computed(() => route.query.intent === 'purchase' && typeof route.query.packageId === 'string' && !!route.query.packageId)

// Honest on/off state for the admin-controlled B2C registration toggle — see backend
// `domain::platform_settings` doc comment. `null` = still checking (brief loading state);
// once resolved, either the real form renders or an honest "dinonaktifkan" message does.
// Fails open (assume enabled) on a network error so a flaky connection doesn't block a
// legitimate registrant — the backend still enforces the real check on submit either way.
const registrationEnabled = ref<boolean | null>(null)
onMounted(async () => {
  try {
    const status = await publicService.registrationStatus()
    registrationEnabled.value = status.b2c_registration_enabled
  } catch {
    registrationEnabled.value = true
  }
})

const brandFeatures = [
  { icon: Brain, label: 'AI-Powered Analysis', sub: 'Adaptive learning system', from: 'from-purple-400', to: 'to-pink-400' },
  { icon: Target, label: 'Rasionalisasi PTN', sub: 'Prediksi peluang real-time', from: 'from-blue-400', to: 'to-cyan-400' },
  { icon: Trophy, label: '10K+ Soal Berkualitas', sub: 'Bank soal terlengkap', from: 'from-orange-400', to: 'to-red-400' },
  { icon: Layers, label: 'Simulasi UTBK Penuh', sub: '7 subtes, timer resmi', from: 'from-emerald-400', to: 'to-teal-400' },
]

const name = ref('')
const email = ref('')
const password = ref('')
const confirmPassword = ref('')
const showPassword = ref(false)
const error = ref('')
const loading = ref(false)

async function handleSubmit() {
  error.value = ''
  if (!name.value.trim() || name.value.trim().length < 2) {
    error.value = 'Nama minimal 2 karakter.'
    return
  }
  if (password.value.length < 6) {
    error.value = 'Password minimal 6 karakter.'
    return
  }
  if (password.value !== confirmPassword.value) {
    error.value = 'Konfirmasi password tidak sama.'
    return
  }

  loading.value = true
  try {
    const user = await register({
      name: name.value.trim(),
      email: email.value.trim(),
      password: password.value,
      role: 'student',
      school_id: null,
    })
    toast.success(`Akun berhasil dibuat, selamat datang ${user.name}!`)

    // Resume a pending "Beli Paket" intent carried over from the landing page / login page
    // (see `login.vue`'s `registerLink` and `doLogin`) — a brand-new B2C student who came
    // here specifically to buy a package should land on checkout, not the dashboard.
    const intent = route.query.intent
    const packageId = route.query.packageId
    if (intent === 'purchase' && typeof packageId === 'string' && packageId) {
      await router.push(`/checkout?packageId=${packageId}`)
      return
    }
    await router.push('/siswa')
  } catch (e: any) {
    error.value = e?.message || 'Pendaftaran gagal, coba lagi.'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 flex items-center justify-center p-4 relative overflow-hidden">
    <div
      class="absolute inset-0 opacity-30 pointer-events-none"
      style="background-image: url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjAwIiBoZWlnaHQ9IjIwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZGVmcz48cGF0dGVybiBpZD0iZ3JpZCIgd2lkdGg9IjQwIiBoZWlnaHQ9IjQwIiBwYXR0ZXJuVW5pdHM9InVzZXJTcGFjZU9uVXNlIj48cGF0aCBkPSJNIDQwIDAgTCAwIDAgMCA0MCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSJ3aGl0ZSIgc3Ryb2tlLW9wYWNpdHk9IjAuMSIgc3Ryb2tlLXdpZHRoPSIxIi8+PC9wYXR0ZXJuPjwvZGVmcz48cmVjdCB3aWR0aD0iMTAwJSIgaGVpZ2h0PSIxMDAlIiBmaWxsPSJ1cmwoI2dyaWQpIi8+PC9zdmc+')"
    />

    <NuxtLink
      to="/"
      class="fixed top-4 left-4 sm:top-6 sm:left-6 z-10 flex items-center gap-2 text-white/80 hover:text-white text-sm font-medium transition-colors"
    >
      <ArrowLeft class="w-4 h-4" /> Kembali ke Beranda
    </NuxtLink>

    <div class="relative w-full max-w-6xl">
      <div class="grid lg:grid-cols-2 gap-8 items-center">
        <!-- Left: branding -->
        <div class="text-white hidden lg:block">
          <div class="mb-8">
            <div class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-sm px-4 py-2 rounded-full mb-6">
              <Sparkles class="w-4 h-4 text-yellow-300" />
              <span class="text-sm">Daftar Mandiri — Tanpa Perlu Sekolah</span>
            </div>
            <img src="/logo-edvion.png" alt="Edvion" class="h-16 sm:h-20 w-auto mb-4" />
            <p class="text-xl text-white/90 mb-8">Daftar sebagai siswa mandiri dan langsung akses drilling tryout &amp; rasionalisasi SNBT</p>
          </div>
          <div class="space-y-4">
            <div v-for="f in brandFeatures" :key="f.label" class="flex items-center gap-3 p-4 bg-white/10 backdrop-blur-sm rounded-lg">
              <div class="w-10 h-10 rounded-lg bg-gradient-to-br flex items-center justify-center" :class="[f.from, f.to]">
                <component :is="f.icon" class="w-5 h-5 text-white" />
              </div>
              <div>
                <div class="font-semibold">{{ f.label }}</div>
                <div class="text-sm text-white/70">{{ f.sub }}</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Right: register form -->
        <Card class="p-8 shadow-2xl">
          <div class="text-center mb-6">
            <h2 class="text-2xl font-bold mb-1 flex items-center justify-center gap-2">Daftar Sekarang <Hand class="w-5 h-5" /></h2>
            <p class="text-muted-foreground text-sm">Buat akun siswa mandiri (tanpa sekolah) — gratis</p>
          </div>

          <div v-if="registrationEnabled === null" class="py-10 text-center text-sm text-muted-foreground">
            Memeriksa status pendaftaran...
          </div>

          <div v-else-if="registrationEnabled === false" class="py-6 text-center">
            <div class="w-12 h-12 rounded-full bg-amber-50 border border-amber-200 flex items-center justify-center mx-auto mb-4">
              <PauseCircle class="w-6 h-6 text-amber-600" />
            </div>
            <p class="text-sm font-bold text-slate-800 mb-1">Pendaftaran mandiri sedang ditutup</p>
            <p class="text-xs text-muted-foreground mb-5">
              Pendaftaran akun siswa mandiri (tanpa sekolah) sedang dinonaktifkan sementara oleh admin EdvionPTN.
              Kalau sekolahmu sudah bermitra dengan EdvionPTN, minta akun ke PIC sekolahmu.
            </p>
            <div class="mt-5 text-center text-sm text-muted-foreground">
              Sudah punya akun? <NuxtLink to="/login" class="text-indigo-600 hover:underline">Login di sini</NuxtLink>
            </div>
          </div>

          <template v-else>
            <div v-if="resumingPurchase" class="p-3 mb-4 rounded-lg border border-orange-200 bg-orange-50 flex items-start gap-2.5">
              <Sparkles class="w-4 h-4 text-orange-600 shrink-0 mt-0.5" />
              <p class="text-xs text-orange-700">Setelah akun dibuat, kamu akan lanjut ke halaman pembelian paket.</p>
            </div>

            <div class="p-3 mb-5 rounded-lg border border-indigo-200 bg-indigo-50 flex items-start gap-2.5">
              <Rocket class="w-4 h-4 text-indigo-600 shrink-0 mt-0.5" />
              <p class="text-xs text-indigo-700">
                Akun ini untuk siswa yang belajar mandiri (B2C), tidak terdaftar lewat sekolah mitra.
                Kalau sekolahmu sudah bermitra dengan EdvionPTN, minta akun ke PIC sekolahmu supaya
                progress belajarmu ikut terekap di sana.
              </p>
            </div>

            <form class="space-y-4" @submit.prevent="handleSubmit">
              <div>
                <Label for="name">Nama Lengkap</Label>
                <Input id="name" v-model="name" placeholder="Nama kamu" required class="mt-1.5" />
              </div>
              <div>
                <Label for="email">Email</Label>
                <Input id="email" v-model="email" type="email" placeholder="nama@email.com" required class="mt-1.5" />
              </div>
              <div>
                <Label for="password">Password</Label>
                <div class="relative mt-1.5">
                  <Input id="password" v-model="password" :type="showPassword ? 'text' : 'password'" placeholder="Minimal 6 karakter" required />
                  <button
                    type="button"
                    class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                    @click="showPassword = !showPassword"
                  >
                    <EyeOff v-if="showPassword" class="w-4 h-4" />
                    <Eye v-else class="w-4 h-4" />
                  </button>
                </div>
              </div>
              <div>
                <Label for="confirmPassword">Konfirmasi Password</Label>
                <Input id="confirmPassword" v-model="confirmPassword" :type="showPassword ? 'text' : 'password'" placeholder="Ulangi password" required class="mt-1.5" />
              </div>

              <div v-if="error" class="p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-600">
                {{ error }}
              </div>

              <Button type="submit" size="lg" class="w-full bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-700 hover:to-purple-700" :disabled="loading">
                {{ loading ? 'Memproses...' : 'Buat Akun' }}
              </Button>
            </form>

            <div class="mt-5 text-center text-sm text-muted-foreground">
              Sudah punya akun? <NuxtLink to="/login" class="text-indigo-600 hover:underline">Login di sini</NuxtLink>
            </div>
          </template>
        </Card>
      </div>

      <div class="lg:hidden text-center mt-8 text-white">
        <p class="text-sm">&copy; 2025 EdvionPTN. Platform Persiapan SNBT Terlengkap</p>
      </div>
    </div>
  </div>
</template>
