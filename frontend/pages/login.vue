<script setup lang="ts">
import { Eye, EyeOff, ArrowLeft, Sparkles, Trophy, Target, LineChart, Layers, Hand, Mail, Lock } from 'lucide-vue-next'
import type { Role } from '~/types'

definePageMeta({ layout: false })

const { login } = useAuth()
const toast = useToast()
const router = useRouter()
const route = useRoute()

const brandFeatures = [
  { icon: LineChart, label: 'Analisis Mendalam', sub: 'Insight progres belajar', from: 'from-purple-400', to: 'to-pink-400' },
  { icon: Target, label: 'Rasionalisasi PTN', sub: 'Prediksi peluang real-time', from: 'from-blue-400', to: 'to-cyan-400' },
  { icon: Trophy, label: '10K+ Soal Berkualitas', sub: 'Bank soal terlengkap', from: 'from-orange-400', to: 'to-red-400' },
  { icon: Layers, label: 'Tim Konten Internal', sub: 'Manajemen paket soal', from: 'from-emerald-400', to: 'to-teal-400' },
]

const email = ref('')
const password = ref('')
const showPassword = ref(false)
const error = ref('')
const loading = ref(false)

// Satu form login untuk semua role — role akun ditentukan sepenuhnya oleh backend saat
// login berhasil (lihat roleRedirect di bawah), bukan oleh pilihan apa pun di sisi UI. Dulu
// ada 4 tab (Siswa/Sekolah/Admin/Konten) di sini, tapi tab itu cuma kosmetik (ubah placeholder
// & kartu info) dan tidak benar-benar menggerbang apa pun, jadi dihapus supaya tidak
// menyesatkan pengguna mengira harus pilih tab yang "benar" untuk bisa login.
function roleRedirect(role: Role) {
  if (role === 'admin') return '/admin'
  if (role === 'content') return '/konten'
  if (role === 'student') return '/siswa'
  return '/sekolah'
}

async function doLogin(useEmail: string, usePassword: string) {
  error.value = ''
  loading.value = true
  try {
    const user = await login({ email: useEmail, password: usePassword })
    toast.success(`Selamat datang, ${user.name}!`)

    // "Beli Paket" on the landing page redirects here with ?intent=purchase&packageId=...
    // so the checkout flow can resume right after auth — only honored for buyer roles
    // (matches backend `initiate_purchase`'s `require_role(&[Student, School])` gate);
    // any other role falls back to its normal dashboard redirect.
    const intent = route.query.intent
    const packageId = route.query.packageId
    if (intent === 'purchase' && typeof packageId === 'string' && packageId && (user.role === 'student' || user.role === 'school')) {
      await router.push(`/checkout?packageId=${packageId}`)
      return
    }

    await router.push(roleRedirect(user.role))
  } catch (e: any) {
    error.value = e?.message || 'Email atau password salah!'
  } finally {
    loading.value = false
  }
}

function handleSubmit() {
  doLogin(email.value, password.value)
}

// Forward a pending "Beli Paket" intent through to the B2C registration page too, so a
// brand-new student who arrived via the landing page's buy button doesn't lose that context
// just because they don't have an account yet — `/daftar` resumes it the same way this page
// does after login (see `doLogin` above).
const registerLink = computed(() => {
  const intent = route.query.intent
  const packageId = route.query.packageId
  if (intent === 'purchase' && typeof packageId === 'string' && packageId) {
    return `/daftar?intent=purchase&packageId=${packageId}`
  }
  return '/daftar'
})
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-indigo-700 via-purple-700 to-fuchsia-600 flex items-center justify-center p-4 relative overflow-hidden">
    <!-- Ambient glow blobs — kedalaman visual tambahan dibanding gradient flat sebelumnya -->
    <div class="absolute -top-24 -left-24 w-[420px] h-[420px] bg-yellow-300/20 rounded-full blur-[110px] pointer-events-none" />
    <div class="absolute -bottom-32 -right-16 w-[480px] h-[480px] bg-pink-400/25 rounded-full blur-[120px] pointer-events-none" />
    <div class="absolute top-1/3 left-1/2 w-[520px] h-[520px] bg-indigo-400/15 rounded-full blur-[130px] pointer-events-none" />

    <div
      class="absolute inset-0 opacity-20 pointer-events-none"
      style="background-image: url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjAwIiBoZWlnaHQ9IjIwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZGVmcz48cGF0dGVybiBpZD0iZ3JpZCIgd2lkdGg9IjQwIiBoZWlnaHQ9IjQwIiBwYXR0ZXJuVW5pdHM9InVzZXJTcGFjZU9uVXNlIj48cGF0aCBkPSJNIDQwIDAgTCAwIDAgMCA0MCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSJ3aGl0ZSIgc3Ryb2tlLW9wYWNpdHk9IjAuMSIgc3Ryb2tlLXdpZHRoPSIxIi8+PC9wYXR0ZXJuPjwvZGVmcz48cmVjdCB3aWR0aD0iMTAwJSIgaGVpZ2h0PSIxMDAlIiBmaWxsPSJ1cmwoI2dyaWQpIi8+PC9zdmc+')"
    />

    <NuxtLink
      to="/"
      class="fixed top-4 left-4 sm:top-6 sm:left-6 z-10 inline-flex items-center gap-2 text-white/80 hover:text-white text-sm font-medium transition-colors bg-white/10 hover:bg-white/15 backdrop-blur-md border border-white/10 px-3.5 py-2 rounded-full"
    >
      <ArrowLeft class="w-4 h-4" /> Kembali ke Beranda
    </NuxtLink>

    <div class="relative w-full max-w-6xl">
      <div class="grid lg:grid-cols-2 gap-10 items-center">
        <!-- Left: branding -->
        <div class="text-white hidden lg:block">
          <div class="mb-10">
            <div class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-md border border-white/20 px-4 py-1.5 rounded-full mb-6">
              <Sparkles class="w-4 h-4 text-yellow-300" />
              <span class="text-sm font-medium">Platform #1 Persiapan SNBT 2025</span>
            </div>
            <img src="/logo-edvion.png" alt="Edvion" class="h-16 sm:h-20 w-auto mb-5 drop-shadow-lg" />
            <p class="text-xl text-white/90 leading-relaxed max-w-md">
              Sistem <span class="font-semibold text-white">End-to-End</span> untuk Drilling Tryout &amp; Rasionalisasi SNBT
            </p>
          </div>
          <div class="grid grid-cols-2 gap-3">
            <div
              v-for="f in brandFeatures" :key="f.label"
              class="group flex flex-col gap-3 p-4 bg-white/10 backdrop-blur-md border border-white/10 rounded-2xl hover:bg-white/[0.15] hover:border-white/20 transition-colors"
            >
              <div class="w-10 h-10 rounded-xl bg-gradient-to-br flex items-center justify-center shadow-lg shadow-black/10 group-hover:scale-105 transition-transform" :class="[f.from, f.to]">
                <component :is="f.icon" class="w-5 h-5 text-white" />
              </div>
              <div>
                <div class="font-semibold text-sm leading-tight">{{ f.label }}</div>
                <div class="text-xs text-white/60 mt-0.5">{{ f.sub }}</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Right: login form -->
        <Card class="p-8 sm:p-10 rounded-3xl shadow-2xl shadow-purple-950/40 border border-white/40 relative overflow-hidden">
          <div class="absolute inset-x-0 top-0 h-1.5 bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500" />

          <div class="text-center mb-7">
            <div class="w-14 h-14 rounded-2xl bg-gradient-to-br from-indigo-600 to-purple-600 flex items-center justify-center mx-auto mb-4 shadow-lg shadow-purple-500/30">
              <Hand class="w-7 h-7 text-white" />
            </div>
            <h2 class="text-2xl font-bold mb-1">Selamat Datang Kembali</h2>
            <p class="text-muted-foreground text-sm">Masuk dengan akun Anda untuk akses dashboard</p>
          </div>

          <form class="space-y-4" @submit.prevent="handleSubmit">
            <div>
              <Label for="email">Email</Label>
              <div class="relative mt-1.5">
                <Mail class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground pointer-events-none" />
                <Input id="email" v-model="email" type="email" placeholder="nama@email.com" required class="pl-10" />
              </div>
            </div>
            <div>
              <Label for="password">Password</Label>
              <div class="relative mt-1.5">
                <Lock class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground pointer-events-none" />
                <Input id="password" v-model="password" :type="showPassword ? 'text' : 'password'" placeholder="Masukkan password" required class="pl-10 pr-10" />
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

            <div v-if="error" class="p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-600">
              {{ error }}
            </div>

            <Button type="submit" size="lg" class="w-full bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-700 hover:to-purple-700 shadow-lg shadow-purple-500/30" :disabled="loading">
              {{ loading ? 'Memproses...' : 'Login' }}
            </Button>
          </form>

          <div class="mt-5 text-center text-sm text-muted-foreground">
            Belum punya akun? <NuxtLink :to="registerLink" class="text-indigo-600 font-semibold hover:underline">Daftar Sekarang</NuxtLink>
          </div>
        </Card>
      </div>

      <div class="lg:hidden text-center mt-8 text-white/80">
        <p class="text-sm">&copy; 2025 EdvionPTN. Platform Persiapan SNBT Terlengkap</p>
      </div>
    </div>
  </div>
</template>
