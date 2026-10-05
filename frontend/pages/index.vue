<script setup lang="ts">
import {
  Rocket, Target, Trophy, CheckCircle, ArrowRight, Menu, X,
  Users, BookOpen, Zap, TrendingUp, Shield, Award, ChevronDown,
  Play, BarChart3, GraduationCap, Building2, MessageSquare,
  Phone, Mail, MapPin, Instagram, Youtube, Twitter,
  Tag, Gift, Ticket, Share2, ChevronRight, Package, PartyPopper,
} from 'lucide-vue-next'

import type { PackageItem, PublicStats, PublicVoucherCheck } from '~/types'

definePageMeta({ layout: false })

const { publicService } = useApi()
const { resolve: resolveUploadUrl } = useUploadUrl()

function onGetStarted() {
  navigateTo('/login')
}
function onLogin() {
  navigateTo('/login')
}
// "Beli Paket" carries package context through login so the checkout flow can pick up
// exactly this package right after auth — every other CTA on this page stays generic.
function onBuyPackage(pkg: PackageItem) {
  navigateTo(`/login?intent=purchase&packageId=${pkg.id}`)
}

// ─── Data nyata dari backend (no-auth) — menggantikan angka marketing hardcode ────────
// (lihat backend AnalyticsService::public_summary / PackageService::list_public /
// VoucherService::check_code, di-ekspos lewat /api/public/*). Tidak ada fallback angka
// fiktif kalau fetch gagal — kalau publicStats masih null, kartu statistik tidak
// menampilkan apa pun daripada berbohong dengan angka lama.
const publicStats = ref<PublicStats | null>(null)
const realPackages = ref<PackageItem[]>([])
const statsLoading = ref(true)

// Format ringkas ala "52,3rb+" dari angka asli — dipakai supaya tampilan tetap ringkas
// seperti sebelumnya, tapi angkanya benar-benar dihitung dari database, bukan ditulis manual.
function fmtStat(n: number | undefined) {
  if (n == null) return '—'
  if (n >= 1000) return `${(n / 1000).toFixed(n >= 10000 ? 0 : 1).replace('.0', '')}rb+`
  return `${n}`
}

onMounted(async () => {
  try {
    const [s, p] = await Promise.all([publicService.stats(), publicService.packages()])
    publicStats.value = s
    realPackages.value = p
  } catch {
    // Halaman marketing publik — kalau backend belum bisa diakses, biarkan kartu
    // statistik/paket kosong (honest-empty) daripada menampilkan angka palsu.
  } finally {
    statsLoading.value = false
  }
})

const stats = computed(() => [
  { value: fmtStat(publicStats.value?.total_students), label: 'Siswa Aktif', icon: Users },
  { value: fmtStat(publicStats.value?.total_schools), label: 'Sekolah Mitra', icon: Building2 },
  { value: fmtStat(publicStats.value?.total_questions), label: 'Bank Soal', icon: BookOpen },
  { value: fmtStat(publicStats.value?.tryouts_completed), label: 'Tryout Diselesaikan', icon: Trophy },
])

const features = [
  {
    icon: BarChart3,
    title: 'Rasionalisasi PTN',
    desc: 'Analisis peluang masuk PTN impianmu berdasarkan data historis SNBT 5 tahun terakhir, passing grade, dan daya tampung.',
    color: 'from-violet-500 to-purple-600',
    badge: 'Data-driven',
  },
  {
    icon: Target,
    title: 'Drilling Adaptif',
    desc: 'Sistem belajar yang menyesuaikan tingkat kesulitan soal secara real-time berdasarkan kelemahanmu. Efisien dan efektif.',
    color: 'from-blue-500 to-cyan-600',
    badge: 'Smart',
  },
  {
    icon: Zap,
    title: 'Tryout Simulasi SNBT',
    desc: 'Latihan tryout persis seperti SNBT asli — timer, antarmuka, tipe soal, hingga skor UTBK yang akurat.',
    color: 'from-orange-500 to-amber-600',
    badge: 'Terbaru',
  },
  {
    icon: BarChart3,
    title: 'Analytics Real-Time',
    desc: 'Dashboard progress lengkap: perkembangan nilai, perbandingan nasional, analisis per submateri, dan rekomendasi belajar.',
    color: 'from-emerald-500 to-teal-600',
    badge: 'Live',
  },
  {
    icon: Trophy,
    title: 'Gamifikasi & Reward',
    desc: 'Kumpulkan poin, naiki leaderboard, dan raih badge eksklusif. Belajar jadi seru dan kamu tetap termotivasi setiap hari.',
    color: 'from-rose-500 to-pink-600',
    badge: 'Fun',
  },
  {
    icon: Shield,
    title: 'Bank Soal 10.000+',
    desc: 'Soal-soal berkualitas tinggi dikurasi oleh tim akademik kami, mencakup semua subtes SNBT dengan pembahasan lengkap.',
    color: 'from-indigo-500 to-violet-600',
    badge: 'Premium',
  },
]

// Paket ditampilkan langsung dari realPackages (data admin sungguhan lewat
// /api/public/packages) — tidak ada lagi katalog hardcode terpisah yang bisa
// tidak sinkron dengan harga/diskon yang sebenarnya berlaku.

const ptnLogos = ['Universitas Indonesia', 'ITB', 'UGM', 'IPB', 'UNAIR', 'UNDIP', 'ITS', 'UNS', 'UNPAD', 'USU']
const ptnLogosLoop = [...ptnLogos, ...ptnLogos]

const howItWorks = [
  { step: '01', title: 'Daftar Gratis', desc: 'Buat akun dalam 30 detik. Tidak perlu kartu kredit.', icon: Users },
  { step: '02', title: 'Ikuti Tes Diagnostik', desc: 'Sistem kami analisis tingkat pemahamanmu di semua subtes SNBT secara terstruktur.', icon: Target },
  { step: '03', title: 'Drill & Tryout', desc: 'Kerjakan soal adaptif harian dan simulasi tryout lengkap.', icon: Target },
  { step: '04', title: 'Analisis & Raih PTN', desc: 'Pantau progress, lihat rasionalisasi berbasis data, dan lolos PTN impianmu!', icon: Trophy },
]

const purchaseSteps = [
  { step: '1', icon: Ticket, title: 'Dapat Voucher', desc: 'Beli di minimarket, website, atau dapat dari referral teman', color: 'from-violet-600 to-purple-600' },
  { step: '2', icon: Users, title: 'Buat Akun', desc: 'Daftar gratis dalam 30 detik, tidak perlu kartu kredit', color: 'from-blue-600 to-cyan-600' },
  { step: '3', icon: Tag, title: 'Input Kode', desc: 'Masukkan kode voucher di dashboard siswa setelah login', color: 'from-emerald-600 to-green-600' },
  { step: '4', icon: Zap, title: 'Akses Aktif!', desc: 'Langsung mulai drilling, tryout, dan rasionalisasi PTN', color: 'from-amber-500 to-orange-500' },
]

const voucherTypes = [
  { icon: Ticket, label: 'Voucher Akses', desc: 'Dari admin / event' },
  { icon: Gift, label: 'Kode Promo', desc: 'Diskon & cashback' },
  { icon: Share2, label: 'Kode Referral', desc: 'Dari teman / afiliasi' },
]

// Statistik B2B nyata dari publicStats — dulu 4 angka ini (+34%, 92%, 4.2x, 512+)
// ditulis manual di kode; sekarang hanya menampilkan agregat yang benar-benar
// terhitung dari database (jumlah sekolah, siswa, soal, rata-rata skor tryout).
const schoolStats = computed(() => [
  { val: fmtStat(publicStats.value?.total_schools), label: 'Sekolah aktif bermitra', color: 'text-cyan-400' },
  { val: fmtStat(publicStats.value?.total_students), label: 'Siswa aktif belajar', color: 'text-emerald-400' },
  { val: fmtStat(publicStats.value?.total_questions), label: 'Soal tersedia', color: 'text-violet-400' },
  { val: publicStats.value ? publicStats.value.average_score.toFixed(1) : '—', label: 'Rata-rata skor tryout', color: 'text-amber-400' },
])

const schoolBenefits = [
  'Dashboard analitik real-time per kelas & siswa',
  'Tryout internal terjadwal untuk seluruh siswa',
  'Laporan bulanan ke wali kelas & orang tua',
  'Revenue sharing program untuk sekolah mitra',
  'Dukungan teknis & pelatihan tim pengajar',
]

const faqs = [
  { q: 'Apa bedanya EdvionPTN dengan aplikasi belajar lainnya?', a: 'EdvionPTN fokus khusus untuk persiapan SNBT dan rasionalisasi PTN. Kami punya analisis peluang berbasis data historis SNBT 5 tahun, drilling adaptif berbasis kelemahan masing-masing siswa, dan bank soal terkurasi 10.000+ soal yang terus diperbarui sesuai pola soal SNBT terbaru.' },
  { q: 'Apakah tryout simulasinya benar-benar seperti SNBT asli?', a: 'Ya! Tryout simulasi kami menggunakan format, tipe soal, durasi, dan sistem penilaian yang identik dengan SNBT. Termasuk soal dengan stimulus panjang, persamaan LaTeX, dan tipe majemuk. Skornya juga dikonversi ke skala UTBK.' },
  { q: 'Bagaimana cara kerja fitur Rasionalisasi PTN?', a: 'Sistem kami menganalisis skor tryoutmu, tren nilai passing grade 5 tahun terakhir, tingkat persaingan per prodi, dan pola penerimaan SNBT. Hasilnya menunjukkan persentase peluang masuk untuk setiap PTN dan prodi yang kamu targetkan — berbasis data nyata, bukan spekulasi.' },
  { q: 'Apakah ada garansi lolos PTN?', a: 'Paket Elite menawarkan garansi refund jika kamu tidak lolos PTN (dengan syarat telah menyelesaikan minimal 80% program yang ditetapkan). Detail syarat dan ketentuan bisa dibaca di halaman Paket Elite.' },
  { q: 'Bisa diakses dari HP?', a: 'Tentu! EdvionPTN fully responsive dan tersedia di semua perangkat — smartphone, tablet, dan desktop. Kamu bisa drilling soal kapanpun, dimanapun.' },
]

const footerColumns = [
  { title: 'Platform', links: ['Fitur', 'Paket & Harga', 'Demo Gratis', 'Blog & Tips SNBT', 'Jadwal Tryout'] },
  { title: 'Sekolah', links: ['Program Mitra', 'Dashboard Sekolah', 'Revenue Sharing', 'Kontak B2B', 'Studi Kasus'] },
  { title: 'Dukungan', links: ['Pusat Bantuan', 'Kontak Kami', 'Syarat & Ketentuan', 'Kebijakan Privasi', 'Status Sistem'] },
]
const socialIcons = [Instagram, Youtube, Twitter]
const navItems = ['Fitur', 'Paket', 'FAQ', 'Untuk Sekolah']

const mobileMenuOpen = ref(false)
const scrolled = ref(false)
const activeAccordion = ref<number | null>(null)
const voucherCode = ref('')
const voucherState = ref<'idle' | 'checking' | 'valid' | 'invalid'>('idle')
const voucherInfo = ref<{ pkg: string; discount: string; expires: string } | null>(null)
const voucherReason = ref('')

// Cek voucher nyata lewat /api/public/vouchers/check — tidak lagi mensimulasikan
// kode ajaib (PROMO-JAN25-50 dkk) dengan setTimeout, langsung tanya backend.
async function checkVoucher() {
  const code = voucherCode.value.trim()
  if (!code) return
  voucherState.value = 'checking'
  try {
    const result: PublicVoucherCheck = await publicService.checkVoucher(code)
    if (result.valid) {
      voucherState.value = 'valid'
      const discount = result.discount_type === 'full'
        ? 'GRATIS (Rp 0)'
        : result.discount_type === 'percent'
          ? `Diskon ${result.discount_value}%`
          : `Diskon Rp${(result.discount_value ?? 0).toLocaleString('id-ID')}`
      voucherInfo.value = {
        pkg: result.package_name ?? '-',
        discount,
        expires: result.expires_at ? new Date(result.expires_at).toLocaleDateString('id-ID', { day: 'numeric', month: 'long', year: 'numeric' }) : '-',
      }
    } else {
      voucherState.value = 'invalid'
      voucherInfo.value = null
      voucherReason.value = result.reason ?? 'Kode voucher tidak valid atau sudah habis masa berlakunya.'
    }
  } catch {
    voucherState.value = 'invalid'
    voucherInfo.value = null
    voucherReason.value = 'Gagal memeriksa voucher. Coba lagi beberapa saat.'
  }
}

function onVoucherInput(e: Event) {
  voucherCode.value = (e.target as HTMLInputElement).value.toUpperCase()
  voucherState.value = 'idle'
  voucherInfo.value = null
}

function formatRp(n: number) {
  return `Rp${n.toLocaleString('id-ID')}`
}

function handleScroll() {
  scrolled.value = window.scrollY > 60
}
onMounted(() => window.addEventListener('scroll', handleScroll))
onUnmounted(() => window.removeEventListener('scroll', handleScroll))
</script>

<template>
  <div class="min-h-screen bg-[#06040f] text-white overflow-x-hidden" style="font-family: 'Plus Jakarta Sans', sans-serif">
    <!-- ─── Navbar ─── -->
    <header
      class="fixed top-0 left-0 right-0 z-50 transition-all duration-300"
      :class="scrolled ? 'bg-[#06040f]/95 backdrop-blur-xl border-b border-white/10 shadow-lg shadow-violet-900/20' : 'bg-transparent'"
    >
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex items-center justify-between h-16 md:h-20">
          <div class="flex items-center gap-2">
            <div class="w-9 h-9 rounded-xl overflow-hidden shrink-0 shadow-lg shadow-violet-900/50">
              <img src="/logo-produk.png" alt="EdvionPTN" class="w-full h-full object-cover" />
            </div>
            <div>
              <span class="text-lg font-bold tracking-tight bg-gradient-to-r from-white to-violet-200 bg-clip-text text-transparent">EdvionPTN</span>
              <span class="hidden sm:block text-[10px] text-violet-400 leading-none -mt-0.5 ml-0.5">by Edvion</span>
            </div>
          </div>

          <nav class="hidden md:flex items-center gap-8">
            <a v-for="item in navItems" :key="item" :href="`#${item.toLowerCase().replace(' ', '-')}`" class="text-sm text-white/70 hover:text-white transition-colors font-medium">{{ item }}</a>
          </nav>

          <div class="hidden md:flex items-center gap-3">
            <button class="text-sm font-semibold text-white/80 hover:text-white transition-colors px-4 py-2" @click="onLogin">Masuk</button>
            <button
              class="text-sm font-bold px-5 py-2.5 rounded-xl bg-gradient-to-r from-violet-600 to-purple-600 hover:from-violet-500 hover:to-purple-500 transition-all shadow-lg shadow-violet-900/40 hover:shadow-violet-900/60 hover:-translate-y-0.5"
              @click="onGetStarted"
            >
              Daftar Gratis
            </button>
          </div>

          <button class="md:hidden text-white/80 hover:text-white p-2" @click="mobileMenuOpen = !mobileMenuOpen">
            <X v-if="mobileMenuOpen" class="w-6 h-6" />
            <Menu v-else class="w-6 h-6" />
          </button>
        </div>
      </div>

      <div v-if="mobileMenuOpen" class="md:hidden bg-[#0d0a20] border-t border-white/10 px-4 py-4 flex flex-col gap-3">
        <a v-for="item in navItems" :key="item" :href="`#${item.toLowerCase()}`" class="text-sm text-white/70 py-2 border-b border-white/5">{{ item }}</a>
        <button class="text-sm font-semibold text-white/80 py-2 text-left" @click="onLogin">Masuk</button>
        <button class="text-sm font-bold px-5 py-3 rounded-xl bg-gradient-to-r from-violet-600 to-purple-600 text-center" @click="onGetStarted">Daftar Gratis</button>
      </div>
    </header>

    <!-- ─── Hero ─── -->
    <section class="relative min-h-screen flex items-center overflow-hidden pt-20">
      <div class="absolute inset-0 pointer-events-none">
        <div class="absolute top-20 left-1/4 w-96 h-96 bg-violet-600/30 rounded-full blur-3xl" />
        <div class="absolute bottom-20 right-1/4 w-80 h-80 bg-purple-700/25 rounded-full blur-3xl" />
        <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-indigo-800/15 rounded-full blur-3xl" />
        <div class="absolute inset-0 opacity-[0.04]" style="background-image: linear-gradient(rgba(255,255,255,.4) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,.4) 1px, transparent 1px); background-size: 60px 60px" />
      </div>

      <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16 md:py-24">
        <div class="grid lg:grid-cols-2 gap-12 lg:gap-16 items-center">
          <div>
            <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-violet-900/60 border border-violet-700/50 text-violet-300 text-xs font-semibold mb-6 backdrop-blur-sm">
              <Rocket class="w-3.5 h-3.5" />
              Platform Tryout SNBT #1 Indonesia
            </div>

            <h1 class="text-4xl sm:text-5xl lg:text-6xl font-black leading-[1.1] mb-6 tracking-tight" style="font-family: 'Outfit', sans-serif">
              Lolos
              <span class="bg-gradient-to-r from-violet-400 via-purple-400 to-fuchsia-400 bg-clip-text text-transparent">PTN Impianmu</span>
              dengan Cara Paling Efektif
            </h1>

            <p class="text-lg text-white/65 leading-relaxed mb-8 max-w-lg" style="font-family: 'Outfit', sans-serif">
              Drilling adaptif terstruktur, simulasi tryout SNBT, dan rasionalisasi PTN berbasis data historis.
              <template v-if="publicStats && publicStats.total_students > 0"> Bergabung dengan {{ fmtStat(publicStats.total_students) }} siswa yang sudah buktikan hasilnya.</template>
            </p>

            <div class="flex flex-col sm:flex-row gap-4 mb-10">
              <button
                class="flex items-center justify-center gap-2 px-7 py-4 rounded-2xl bg-gradient-to-r from-violet-600 to-purple-600 hover:from-violet-500 hover:to-purple-500 font-bold text-base transition-all shadow-xl shadow-violet-900/50 hover:shadow-violet-900/70 hover:-translate-y-1 group"
                @click="onGetStarted"
              >
                Mulai Gratis Sekarang
                <ArrowRight class="w-5 h-5 group-hover:translate-x-1 transition-transform" />
              </button>
              <button class="flex items-center justify-center gap-2 px-7 py-4 rounded-2xl border border-white/20 hover:border-white/40 font-semibold text-base transition-all hover:bg-white/5 group">
                <Play class="w-5 h-5 text-violet-400" />
                Tonton Demo
              </button>
            </div>

          </div>

          <!-- Kartu statistik platform real-time — dulu berupa mockup satu siswa fiktif
               ("Rizka Amalia", skor & peluang PTN karangan). Diganti dengan angka agregat
               sungguhan dari /api/public/stats supaya tidak ada klaim yang tidak bisa
               dipertanggungjawabkan. -->
          <div class="relative hidden lg:block">
            <div class="relative bg-white/5 border border-white/10 rounded-3xl p-6 backdrop-blur-xl shadow-2xl">
              <div class="flex items-center gap-3 mb-5">
                <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-violet-500 to-purple-600 flex items-center justify-center">
                  <BarChart3 class="w-5 h-5 text-white" />
                </div>
                <div>
                  <p class="text-sm font-bold">Statistik Platform</p>
                  <p class="text-xs text-white/50">Update real-time</p>
                </div>
                <div class="ml-auto">
                  <span class="text-xs bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 px-2 py-1 rounded-full font-semibold">Live</span>
                </div>
              </div>

              <div class="grid grid-cols-2 gap-3">
                <div v-for="s in stats" :key="s.label" class="p-3 rounded-2xl bg-white/5 border border-white/10">
                  <component :is="s.icon" class="w-4 h-4 text-violet-400 mb-1.5" />
                  <p class="text-lg font-black text-white">{{ s.value }}</p>
                  <p class="text-[10px] text-white/50 leading-tight">{{ s.label }}</p>
                </div>
              </div>

              <div v-if="publicStats" class="mt-4 pt-4 border-t border-white/10 flex items-center justify-between text-xs">
                <span class="text-white/50">Rata-rata skor tryout SNBT</span>
                <span class="font-bold text-violet-300">{{ publicStats.average_score.toFixed(1) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="absolute bottom-8 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2 text-white/30">
        <span class="text-xs">Scroll</span>
        <ChevronDown class="w-4 h-4 animate-bounce" />
      </div>
    </section>

    <!-- ─── Stats ─── -->
    <section class="py-16 border-y border-white/10 bg-white/[0.02]">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-8">
          <div v-for="s in stats" :key="s.label" class="text-center">
            <div class="flex justify-center mb-3">
              <div class="w-12 h-12 rounded-2xl bg-violet-900/40 border border-violet-700/30 flex items-center justify-center">
                <component :is="s.icon" class="w-6 h-6 text-violet-400" />
              </div>
            </div>
            <p class="text-3xl lg:text-4xl font-black bg-gradient-to-r from-white to-violet-200 bg-clip-text text-transparent mb-1" style="font-family: 'Outfit', sans-serif">{{ s.value }}</p>
            <p class="text-sm text-white/50 font-medium">{{ s.label }}</p>
          </div>
        </div>
      </div>
    </section>

    <!-- ─── PTN logos ─── -->
    <section class="py-10 overflow-hidden">
      <div class="max-w-7xl mx-auto px-4 mb-6">
        <p class="text-center text-sm text-white/40 font-medium uppercase tracking-widest">Dipercaya siswa yang kini kuliah di</p>
      </div>
      <div class="marquee flex gap-6 whitespace-nowrap" style="width: max-content">
        <div v-for="(ptn, i) in ptnLogosLoop" :key="i" class="flex items-center gap-2 px-5 py-2.5 rounded-full bg-white/5 border border-white/10 text-white/50 text-sm font-semibold hover:text-white/80 transition-colors cursor-default shrink-0">
          <GraduationCap class="w-4 h-4 text-violet-400" />
          {{ ptn }}
        </div>
      </div>
    </section>

    <!-- ─── Features ─── -->
    <section id="fitur" class="py-20 lg:py-28">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="text-center mb-16">
          <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-violet-900/40 border border-violet-700/40 text-violet-300 text-xs font-semibold mb-4">
            <Zap class="w-3.5 h-3.5" /> Fitur Unggulan
          </div>
          <h2 class="text-3xl sm:text-4xl lg:text-5xl font-black mb-4" style="font-family: 'Outfit', sans-serif">
            Semua yang kamu butuhkan<br />
            <span class="bg-gradient-to-r from-violet-400 to-fuchsia-400 bg-clip-text text-transparent">untuk tembus PTN</span>
          </h2>
          <p class="text-white/55 text-lg max-w-2xl mx-auto">Dari soal harian sampai rasionalisasi PTN berbasis data — semuanya ada di satu platform.</p>
        </div>

        <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-5">
          <div v-for="f in features" :key="f.title" class="group relative bg-white/[0.03] hover:bg-white/[0.06] border border-white/10 hover:border-white/20 rounded-3xl p-6 transition-all duration-300 hover:-translate-y-1 cursor-default">
            <div class="flex items-start justify-between mb-4">
              <div class="w-12 h-12 rounded-2xl bg-gradient-to-br flex items-center justify-center shadow-lg" :class="f.color">
                <component :is="f.icon" class="w-6 h-6 text-white" />
              </div>
              <span class="text-xs font-bold px-2.5 py-1 rounded-full bg-gradient-to-r text-white opacity-90" :class="f.color">{{ f.badge }}</span>
            </div>
            <h3 class="text-base font-bold mb-2 text-white">{{ f.title }}</h3>
            <p class="text-sm text-white/55 leading-relaxed">{{ f.desc }}</p>
          </div>
        </div>
      </div>
    </section>

    <!-- ─── How it works ─── -->
    <section class="py-20 bg-white/[0.02] border-y border-white/10">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="text-center mb-16">
          <h2 class="text-3xl sm:text-4xl font-black mb-3" style="font-family: 'Outfit', sans-serif">Cara Kerjanya Simpel</h2>
          <p class="text-white/50">Mulai dari daftar sampai lolos PTN, hanya 4 langkah.</p>
        </div>
        <div class="grid sm:grid-cols-2 lg:grid-cols-4 gap-8">
          <div v-for="h in howItWorks" :key="h.step" class="relative text-center">
            <div class="relative inline-flex">
              <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-violet-600 to-purple-700 flex items-center justify-center mb-4 shadow-xl shadow-violet-900/40 mx-auto">
                <component :is="h.icon" class="w-8 h-8 text-white" />
              </div>
              <span class="absolute -top-2 -right-2 w-7 h-7 rounded-full bg-[#06040f] border border-violet-500 text-violet-400 text-xs font-black flex items-center justify-center">{{ h.step }}</span>
            </div>
            <h3 class="text-base font-bold mb-2">{{ h.title }}</h3>
            <p class="text-sm text-white/50">{{ h.desc }}</p>
          </div>
        </div>
      </div>
    </section>

    <!-- ─── Pricing ─── -->
    <section id="paket" class="py-20 lg:py-28">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="text-center mb-12">
          <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-violet-900/40 border border-violet-700/40 text-violet-300 text-xs font-semibold mb-4">
            <Award class="w-3.5 h-3.5" /> Pilih Paket
          </div>
          <h2 class="text-3xl sm:text-4xl lg:text-5xl font-black mb-4" style="font-family: 'Outfit', sans-serif">
            Pilih paket<br />
            <span class="bg-gradient-to-r from-violet-400 to-fuchsia-400 bg-clip-text text-transparent">sesuai targetmu</span>
          </h2>
          <p class="text-white/55">Kuota tryout aktif 1 tahun. Bayar sekali, pakai kapan saja. Hemat hingga 40%.</p>
        </div>

        <div v-if="statsLoading" class="text-center text-white/40 text-sm py-10">Memuat paket...</div>
        <div v-else-if="realPackages.length === 0" class="text-center text-white/40 text-sm py-10">Belum ada paket aktif saat ini.</div>
        <div v-else class="no-scrollbar flex gap-4 overflow-x-auto pb-4">
          <div v-for="pkg in realPackages" :key="pkg.id" class="flex-shrink-0 w-60 bg-white rounded-2xl overflow-hidden shadow-xl shadow-black/40 flex flex-col relative">
            <div v-if="pkg.discount > 0" class="absolute top-3 left-3 z-10 w-11 h-11 rounded-full bg-orange-500 flex flex-col items-center justify-center shadow-lg">
              <span class="text-white font-black text-xs leading-none">{{ pkg.discount }}%</span>
              <span class="text-white/80 text-[9px] leading-none">OFF</span>
            </div>

            <div v-if="pkg.badge" class="absolute top-3 right-3 z-10 text-white text-[10px] font-black px-2 py-0.5 rounded-full" :style="{ backgroundColor: pkg.accent_color }">
              {{ pkg.badge }}
            </div>

            <div v-if="pkg.banner_url" class="h-36 relative">
              <img :src="resolveUploadUrl(pkg.banner_url) ?? undefined" :alt="pkg.name" class="w-full h-full object-cover" />
            </div>
            <div v-else class="h-36 bg-gradient-to-br flex items-end justify-center pb-4 relative" :class="pkg.gradient">
              <IconDisplay :icon-type="pkg.icon_type" :icon-name="pkg.icon_name" :icon-url="pkg.icon_url" size="w-16 h-16 text-slate-700" />
            </div>

            <div class="p-4 flex flex-col flex-1">
              <h3 class="font-bold text-slate-800 text-sm mb-3 leading-snug">{{ pkg.name }}</h3>

              <div class="mb-3">
                <p v-if="pkg.discount > 0" class="text-slate-400 text-xs line-through leading-none mb-0.5">{{ formatRp(pkg.original_price) }}</p>
                <p class="font-black text-xl leading-none" style="color: #f97316">{{ formatRp(pkg.sale_price) }}</p>
              </div>

              <button
                class="w-full py-2.5 rounded-xl font-bold text-sm text-white transition-all hover:opacity-90 active:scale-95 mb-3"
                style="background-color: #f97316"
                @click="onBuyPackage(pkg)"
              >
                Beli Paket
              </button>

              <ul class="space-y-1.5 flex-1">
                <li v-for="f in pkg.features" :key="f" class="flex items-start gap-1.5 text-[11px] text-slate-600 leading-tight">
                  <CheckCircle class="w-3.5 h-3.5 text-emerald-500 mt-0.5 shrink-0" />
                  {{ f }}
                </li>
              </ul>

              <div v-if="pkg.content_titles.length" class="pt-2 mt-1 border-t border-dashed">
                <p class="text-[10px] font-bold text-slate-400 uppercase tracking-wide mb-1">Termasuk</p>
                <ul class="space-y-1">
                  <li v-for="c in pkg.content_titles" :key="c" class="flex items-start gap-1.5 text-[11px] text-slate-500 leading-tight">
                    <Package class="w-3 h-3 mt-0.5 shrink-0" :style="{ color: pkg.accent_color }" />
                    {{ c }}
                  </li>
                </ul>
              </div>

              <button class="mt-3 text-xs font-semibold flex items-center gap-0.5 hover:gap-1.5 transition-all" :style="{ color: pkg.accent_color }" @click="onGetStarted">
                Lihat Detail <ChevronRight class="w-3 h-3" />
              </button>
            </div>
          </div>

          <div
            class="flex-shrink-0 w-44 rounded-2xl flex flex-col items-center justify-center p-6 gap-4 cursor-pointer group overflow-hidden relative"
            style="background: linear-gradient(135deg, #0d9488, #0891b2)"
            @click="onGetStarted"
          >
            <div class="absolute inset-0 bg-white/0 group-hover:bg-white/10 transition-colors" />
            <Package class="w-12 h-12 text-white" />
            <p class="text-white font-bold text-center text-sm leading-snug">Lihat semua paket</p>
            <div class="w-10 h-10 rounded-full bg-white/20 group-hover:bg-white/30 flex items-center justify-center transition-colors">
              <ChevronRight class="w-5 h-5 text-white" />
            </div>
          </div>
        </div>

        <p class="text-center text-xs text-white/30 mt-6">*Harga promo terbatas. Pembayaran aman dengan enkripsi SSL.</p>

        <!-- Voucher entry -->
        <div class="mt-14 max-w-2xl mx-auto">
          <div class="bg-white/[0.04] border border-white/10 rounded-3xl p-7 backdrop-blur-sm">
            <div class="flex items-center gap-3 mb-5">
              <div class="w-10 h-10 rounded-2xl bg-gradient-to-br from-violet-500 to-purple-600 flex items-center justify-center shrink-0">
                <Tag class="w-5 h-5 text-white" />
              </div>
              <div>
                <h3 class="font-black text-white text-lg" style="font-family: 'Outfit', sans-serif">Punya Kode Voucher?</h3>
                <p class="text-white/50 text-sm">Masukkan kode dan dapatkan akses premium secara langsung</p>
              </div>
            </div>

            <div class="flex gap-2 mb-4">
              <input
                :value="voucherCode"
                placeholder="Contoh: PROMO-JAN25-50"
                class="flex-1 px-4 py-3 rounded-xl bg-white/10 border border-white/20 text-white placeholder-white/30 text-sm font-mono tracking-wider focus:outline-none focus:border-violet-500 focus:bg-white/15 transition-all"
                @input="onVoucherInput"
                @keydown.enter="checkVoucher"
              />
              <button
                :disabled="!voucherCode.trim() || voucherState === 'checking'"
                class="px-6 py-3 rounded-xl bg-gradient-to-r from-violet-600 to-purple-600 hover:from-violet-500 hover:to-purple-500 text-white font-bold text-sm transition-all disabled:opacity-50 disabled:cursor-not-allowed shrink-0"
                @click="checkVoucher"
              >
                {{ voucherState === 'checking' ? '...' : 'Cek' }}
              </button>
            </div>

            <div v-if="voucherState === 'invalid'" class="flex items-center gap-2 px-4 py-3 rounded-xl bg-red-900/30 border border-red-500/30 text-red-300 text-sm mb-4">
              <X class="w-4 h-4 shrink-0" />
              {{ voucherReason || 'Kode voucher tidak valid atau sudah habis masa berlakunya.' }}
            </div>
            <div v-else-if="voucherState === 'valid' && voucherInfo" class="mb-4">
              <div class="flex items-start gap-3 px-4 py-4 rounded-xl bg-emerald-900/30 border border-emerald-500/40 mb-3">
                <CheckCircle class="w-5 h-5 text-emerald-400 shrink-0 mt-0.5" />
                <div class="flex-1">
                  <p class="text-emerald-300 font-bold text-sm mb-0.5 flex items-center gap-1.5">Voucher valid! <PartyPopper class="w-4 h-4" /></p>
                  <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-white/60">
                    <span>Paket: <strong class="text-white">{{ voucherInfo.pkg }}</strong></span>
                    <span>Harga: <strong class="text-emerald-400">{{ voucherInfo.discount }}</strong></span>
                    <span>Berlaku s/d: <strong class="text-white">{{ voucherInfo.expires }}</strong></span>
                  </div>
                </div>
              </div>
              <button
                class="w-full py-3.5 rounded-xl bg-gradient-to-r from-emerald-500 to-green-600 hover:from-emerald-400 hover:to-green-500 text-white font-black text-sm transition-all hover:-translate-y-0.5 shadow-lg shadow-emerald-900/40 flex items-center justify-center gap-2"
                @click="onGetStarted"
              >
                <Gift class="w-4 h-4" />
                Daftar &amp; Aktifkan Voucher Sekarang
                <ChevronRight class="w-4 h-4" />
              </button>
            </div>

            <div class="grid grid-cols-3 gap-3 pt-4 border-t border-white/10">
              <div v-for="v in voucherTypes" :key="v.label" class="text-center p-3 rounded-xl bg-white/[0.03] border border-white/[0.08]">
                <component :is="v.icon" class="w-5 h-5 text-violet-400 mx-auto mb-1.5" />
                <p class="text-xs font-bold text-white/80">{{ v.label }}</p>
                <p class="text-[10px] text-white/40 mt-0.5">{{ v.desc }}</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Purchase flow -->
        <div class="mt-10 max-w-4xl mx-auto">
          <p class="text-center text-white/40 text-sm font-semibold uppercase tracking-wider mb-6">Cara Beli Akses EdvionPTN</p>
          <div class="grid sm:grid-cols-4 gap-4">
            <div v-for="s in purchaseSteps" :key="s.step" class="relative text-center">
              <div class="w-12 h-12 rounded-2xl bg-gradient-to-br flex items-center justify-center mx-auto mb-3 shadow-lg" :class="s.color">
                <component :is="s.icon" class="w-6 h-6 text-white" />
              </div>
              <div class="absolute top-6 left-[calc(50%+24px)] w-[calc(100%-48px)] h-px bg-white/10 hidden sm:block" />
              <p class="text-[10px] font-black text-white/30 uppercase tracking-widest mb-1">Step {{ s.step }}</p>
              <p class="font-bold text-white text-sm mb-1">{{ s.title }}</p>
              <p class="text-xs text-white/45 leading-relaxed">{{ s.desc }}</p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- ─── For Schools (B2B) ─── -->
    <section id="untuk-sekolah" class="py-20 lg:py-28">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="relative bg-gradient-to-br from-violet-900/40 to-purple-900/30 border border-violet-700/30 rounded-[2rem] overflow-hidden p-10 lg:p-16">
          <div class="absolute top-0 right-0 w-80 h-80 bg-violet-600/20 rounded-full blur-3xl" />
          <div class="absolute bottom-0 left-0 w-60 h-60 bg-purple-700/20 rounded-full blur-3xl" />

          <div class="relative grid lg:grid-cols-2 gap-12 items-center">
            <div>
              <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-violet-900/60 border border-violet-700/50 text-violet-300 text-xs font-semibold mb-6">
                <Building2 class="w-3.5 h-3.5" /> Untuk Sekolah &amp; Lembaga
              </div>
              <h2 class="text-3xl lg:text-4xl font-black mb-4" style="font-family: 'Outfit', sans-serif">Tingkatkan angka kelulusan PTN sekolahmu</h2>
              <p class="text-white/60 leading-relaxed mb-8">
                <template v-if="publicStats && publicStats.total_schools > 0">Bergabunglah dengan {{ fmtStat(publicStats.total_schools) }} sekolah mitra. </template>Dapatkan dashboard analitik sekolah, manajemen siswa terpusat, laporan perkembangan, dan dukungan tim edukasi kami.
              </p>
              <ul class="space-y-3 mb-8">
                <li v-for="f in schoolBenefits" :key="f" class="flex items-center gap-3 text-sm text-white/75">
                  <CheckCircle class="w-4 h-4 text-emerald-400 shrink-0" />
                  {{ f }}
                </li>
              </ul>
              <div class="flex flex-col sm:flex-row gap-3">
                <button class="px-6 py-3.5 rounded-xl bg-white text-violet-900 font-bold text-sm hover:bg-violet-50 transition-colors shadow-lg" @click="onLogin">
                  Daftar Sebagai Sekolah Mitra
                </button>
                <button class="px-6 py-3.5 rounded-xl border border-white/30 font-semibold text-sm hover:bg-white/10 transition-colors flex items-center gap-2">
                  <MessageSquare class="w-4 h-4" /> Hubungi Tim Kami
                </button>
              </div>
            </div>

            <div class="bg-white/5 border border-white/10 rounded-2xl p-6 backdrop-blur-sm">
              <p class="text-sm font-bold text-white/60 mb-4 uppercase tracking-wider">Rata-rata Hasil Sekolah Mitra</p>
              <div class="grid grid-cols-2 gap-4 mb-6">
                <div v-for="s in schoolStats" :key="s.label" class="bg-white/5 rounded-xl p-4">
                  <p class="text-2xl font-black mb-1" :class="s.color" style="font-family: 'Outfit', sans-serif">{{ s.val }}</p>
                  <p class="text-xs text-white/50 leading-tight">{{ s.label }}</p>
                </div>
              </div>
              <div class="flex items-center gap-3 p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-xl">
                <TrendingUp class="w-5 h-5 text-emerald-400 shrink-0" />
                <p class="text-xs text-emerald-300">Sekolah mitra rata-rata melihat peningkatan signifikan dalam 3 bulan pertama.</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- ─── FAQ ─── -->
    <section id="faq" class="py-20 border-t border-white/10">
      <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="text-center mb-12">
          <h2 class="text-3xl sm:text-4xl font-black mb-3" style="font-family: 'Outfit', sans-serif">Pertanyaan Umum</h2>
          <p class="text-white/50">Ada yang masih bingung? Kami jawab di sini.</p>
        </div>
        <div class="space-y-3">
          <div v-for="(f, i) in faqs" :key="i" class="bg-white/[0.03] border border-white/10 rounded-2xl overflow-hidden">
            <button class="w-full flex items-center justify-between p-5 text-left hover:bg-white/5 transition-colors" @click="activeAccordion = activeAccordion === i ? null : i">
              <span class="font-semibold text-sm text-white/90 pr-4">{{ f.q }}</span>
              <ChevronDown class="w-5 h-5 text-white/40 shrink-0 transition-transform" :class="{ 'rotate-180': activeAccordion === i }" />
            </button>
            <div v-if="activeAccordion === i" class="px-5 pb-5">
              <p class="text-sm text-white/60 leading-relaxed">{{ f.a }}</p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- ─── Final CTA ─── -->
    <section class="py-20 px-4">
      <div class="max-w-4xl mx-auto text-center relative">
        <div class="absolute inset-0 bg-gradient-to-r from-violet-600/20 to-purple-600/20 rounded-3xl blur-2xl" />
        <div class="relative bg-gradient-to-br from-violet-900/50 to-purple-900/40 border border-violet-700/40 rounded-3xl p-12">
          <Rocket class="w-12 h-12 text-violet-400 mx-auto mb-6" />
          <h2 class="text-3xl sm:text-4xl lg:text-5xl font-black mb-4" style="font-family: 'Outfit', sans-serif">Siap gaspol ke PTN impianmu?</h2>
          <p class="text-white/60 text-lg mb-8 max-w-xl mx-auto">Daftar gratis sekarang dan mulai perjalananmu menuju PTN impian<template v-if="publicStats && publicStats.total_students > 0"> bersama {{ fmtStat(publicStats.total_students) }} siswa lainnya</template>.</p>
          <div class="flex flex-col sm:flex-row gap-4 justify-center">
            <button class="flex items-center justify-center gap-2 px-8 py-4 rounded-2xl bg-gradient-to-r from-violet-600 to-purple-600 hover:from-violet-500 hover:to-purple-500 font-black text-base transition-all shadow-xl shadow-violet-900/50 hover:-translate-y-1 group" @click="onGetStarted">
              Daftar Gratis Sekarang
              <ArrowRight class="w-5 h-5 group-hover:translate-x-1 transition-transform" />
            </button>
            <button class="flex items-center justify-center gap-2 px-8 py-4 rounded-2xl border border-white/20 hover:border-white/40 font-semibold text-base transition-all hover:bg-white/5" @click="onLogin">
              Sudah punya akun? Masuk
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- ─── Footer ─── -->
    <footer class="border-t border-white/10 py-12 px-4">
      <div class="max-w-7xl mx-auto">
        <div class="grid sm:grid-cols-2 lg:grid-cols-4 gap-8 mb-10">
          <div>
            <div class="flex items-center gap-2 mb-4">
              <div class="w-8 h-8 rounded-lg overflow-hidden shrink-0">
                <img src="/logo-produk.png" alt="EdvionPTN" class="w-full h-full object-cover" />
              </div>
              <span class="font-bold text-white">EdvionPTN</span>
            </div>
            <p class="text-sm text-white/45 leading-relaxed mb-4">Platform tryout SNBT dan rasionalisasi PTN terlengkap di Indonesia. Bantu siswa wujudkan PTN impian.</p>
            <div class="flex gap-3">
              <a v-for="(Icon, i) in socialIcons" :key="i" href="#" class="w-8 h-8 rounded-lg bg-white/10 hover:bg-white/20 flex items-center justify-center transition-colors">
                <component :is="Icon" class="w-4 h-4 text-white/70" />
              </a>
            </div>
          </div>

          <div v-for="col in footerColumns" :key="col.title">
            <p class="text-sm font-bold text-white mb-4">{{ col.title }}</p>
            <ul class="space-y-2.5">
              <li v-for="link in col.links" :key="link"><a href="#" class="text-sm text-white/45 hover:text-white/80 transition-colors">{{ link }}</a></li>
            </ul>
          </div>
        </div>

        <div class="flex flex-col sm:flex-row items-center justify-between pt-6 border-t border-white/10 gap-4">
          <p class="text-xs text-white/30">&copy; 2025 EdvionPTN. All rights reserved.</p>
          <div class="flex items-center gap-6">
            <div class="flex items-center gap-1.5 text-xs text-white/30"><Mail class="w-3.5 h-3.5" /> hello@edvion.id</div>
            <div class="flex items-center gap-1.5 text-xs text-white/30"><Phone class="w-3.5 h-3.5" /> +62 811-1234-5678</div>
            <div class="flex items-center gap-1.5 text-xs text-white/30"><MapPin class="w-3.5 h-3.5" /> Jakarta, Indonesia</div>
          </div>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.marquee {
  animation: marquee 30s linear infinite;
}
@keyframes marquee {
  from { transform: translateX(0); }
  to { transform: translateX(-50%); }
}
.no-scrollbar {
  scrollbar-width: none;
  -ms-overflow-style: none;
}
.no-scrollbar::-webkit-scrollbar {
  display: none;
}
</style>
