import { useState, useEffect, useRef, useCallback } from 'react';
import {
  Rocket, Target, Trophy, Star, CheckCircle, ArrowRight, Menu, X,
  Users, BookOpen, Zap, TrendingUp, Shield, Clock, Award, ChevronDown,
  Play, BarChart3, GraduationCap, Building2, MessageSquare,
  Phone, Mail, MapPin, Instagram, Youtube, Twitter,
  Tag, Gift, Ticket, Share2, ChevronRight
} from 'lucide-react';

interface LandingPageProps {
  onGetStarted: () => void;
  onLogin: () => void;
}

const stats = [
  { value: '52.000+', label: 'Siswa Aktif', icon: Users },
  { value: '512', label: 'Sekolah Mitra', icon: Building2 },
  { value: '10.000+', label: 'Bank Soal', icon: BookOpen },
  { value: '87%', label: 'Lolos PTN', icon: Trophy },
];

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
];

const quotaPackages = [
  {
    id: 'p1',
    name: '5 Kuota Tryout SNBT',
    type: 'TKA',
    originalPrice: 175000,
    salePrice: 105000,
    discount: 40,
    gradient: 'from-sky-200 to-blue-100',
    accentColor: '#3b82f6',
    emoji: '📘',
    badge: null,
    features: [
      '5x Tryout SNBT Full Simulasi',
      'Kuota berlaku 1 tahun sejak pembelian',
      'Pembahasan lengkap tiap soal',
      'Analisis skor otomatis',
    ],
  },
  {
    id: 'p2',
    name: '12 Kuota Tryout SNBT & TPS',
    type: 'Full',
    originalPrice: 350000,
    salePrice: 210000,
    discount: 40,
    gradient: 'from-violet-200 to-purple-100',
    accentColor: '#7c3aed',
    emoji: '🎯',
    badge: 'Terpopuler',
    features: [
      '12x Tryout Full Simulasi',
      'TPS + Literasi + Penalaran Mat.',
      'Kuota berlaku 1 tahun sejak pembelian',
      'Video pembahasan eksklusif',
    ],
  },
  {
    id: 'p3',
    name: '20 Kuota Tryout SNBT Full',
    type: 'Premium',
    originalPrice: 580000,
    salePrice: 348000,
    discount: 40,
    gradient: 'from-amber-200 to-orange-100',
    accentColor: '#f59e0b',
    emoji: '🏆',
    badge: 'Best Value',
    features: [
      '20x Tryout Full Simulasi SNBT',
      'Semua subtes SNBT lengkap',
      'Kuota berlaku 1 tahun sejak pembelian',
      'Rasionalisasi PTN gratis',
    ],
  },
  {
    id: 'p4',
    name: 'Akses Drilling Intensif',
    type: 'Drilling',
    originalPrice: 195000,
    salePrice: 117000,
    discount: 40,
    gradient: 'from-emerald-200 to-teal-100',
    accentColor: '#10b981',
    emoji: '⚡',
    badge: null,
    features: [
      'Drilling tanpa batas 365 hari',
      '10.000+ soal per kategori',
      'Adaptive difficulty AI',
      'Progress tracking real-time',
    ],
  },
  {
    id: 'p5',
    name: 'Paket Lengkap Elite',
    type: 'Elite',
    originalPrice: 750000,
    salePrice: 450000,
    discount: 40,
    gradient: 'from-rose-200 to-pink-100',
    accentColor: '#ec4899',
    emoji: '💎',
    badge: 'All-In-One',
    features: [
      'Tryout full simulasi tanpa batas',
      'Drilling intensif tanpa batas',
      'Rasionalisasi semua PTN',
      'Mentoring 1-on-1 / minggu',
    ],
  },
];

const testimonials = [
  {
    name: 'Rizka Amalia',
    school: 'SMAN 1 Bandung',
    ptn: 'Teknik Informatika UI',
    avatar: 'RA',
    color: 'from-violet-400 to-purple-500',
    score: 712,
    text: 'Fitur Rasionalisasi PTN-nya keren banget! Aku jadi tahu peluang masuk Teknik Informatika UI berapa persen berdasarkan skor tryoutku dan data historis SNBT. Motivasiku langsung naik 10x.',
    stars: 5,
  },
  {
    name: 'Farhan Nugraha',
    school: 'SMAN 3 Surabaya',
    ptn: 'Kedokteran UNAIR',
    avatar: 'FN',
    color: 'from-blue-400 to-cyan-500',
    score: 734,
    text: 'Drilling adaptifnya beneran ampuh. Awalnya aku lemah banget di Penalaran Matematika, setelah 2 bulan drilling, nilai tryoutku naik 80 poin. Gokil!',
    stars: 5,
  },
  {
    name: 'Siti Nurhaliza',
    school: 'MA Negeri 1 Yogyakarta',
    ptn: 'Akuntansi UGM',
    avatar: 'SN',
    color: 'from-emerald-400 to-teal-500',
    score: 698,
    text: 'Dari ratusan soal per hari sampai akhirnya lolos UGM. GASPOLPTN beneran bantu aku tau kelemahan dan fokus benerin bagian yang penting.',
    stars: 5,
  },
  {
    name: 'Bagas Pratama',
    school: 'SMAN 8 Jakarta',
    ptn: 'Hukum UI',
    avatar: 'BP',
    color: 'from-rose-400 to-pink-500',
    score: 688,
    text: 'Tryout simulasinya persis banget sama SNBT asli. Jadi waktu ujian hari-H, aku udah nggak nervous lagi karena sudah familiar dengan format soalnya.',
    stars: 5,
  },
];

const ptnLogos = [
  'Universitas Indonesia', 'ITB', 'UGM', 'IPB', 'UNAIR',
  'UNDIP', 'ITS', 'UNS', 'UNPAD', 'USU',
];

export default function LandingPage({ onGetStarted, onLogin }: LandingPageProps) {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);
  const [activeAccordion, setActiveAccordion] = useState<number | null>(null);
  const [voucherCode, setVoucherCode] = useState('');
  const [voucherState, setVoucherState] = useState<'idle' | 'checking' | 'valid' | 'invalid'>('idle');
  const [voucherInfo, setVoucherInfo] = useState<{ pkg: string; discount: string; expires: string } | null>(null);

  const checkVoucher = useCallback(() => {
    if (!voucherCode.trim()) return;
    setVoucherState('checking');
    setTimeout(() => {
      const upper = voucherCode.toUpperCase().trim();
      if (upper === 'PROMO-JAN25-50') {
        setVoucherState('valid');
        setVoucherInfo({ pkg: 'Premium 1 Bulan', discount: 'Diskon 50% → Rp 39.500', expires: '31 Jan 2025' });
      } else if (upper === 'GASPL-PREM-X7K2') {
        setVoucherState('valid');
        setVoucherInfo({ pkg: 'Premium 3 Bulan', discount: 'GRATIS (Rp 0)', expires: '31 Mar 2025' });
      } else if (upper === 'GASPL-ELITE-9MNP') {
        setVoucherState('invalid');
        setVoucherInfo(null);
      } else if (upper.startsWith('GASPL') || upper.startsWith('PROMO') || upper.startsWith('BULK') || upper.startsWith('REF')) {
        setVoucherState('valid');
        setVoucherInfo({ pkg: 'Premium 3 Bulan', discount: 'Diskon 20% → Rp 168.000', expires: '31 Des 2025' });
      } else {
        setVoucherState('invalid');
        setVoucherInfo(null);
      }
    }, 800);
  }, [voucherCode]);
  const heroRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleScroll = () => setScrolled(window.scrollY > 60);
    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  const faqs = [
    {
      q: 'Apa bedanya GASPOLPTN dengan aplikasi belajar lainnya?',
      a: 'GASPOLPTN fokus khusus untuk persiapan SNBT dan rasionalisasi PTN. Kami punya analisis peluang berbasis data historis SNBT 5 tahun, drilling adaptif berbasis kelemahan masing-masing siswa, dan bank soal terkurasi 10.000+ soal yang terus diperbarui sesuai pola soal SNBT terbaru.',
    },
    {
      q: 'Apakah tryout simulasinya benar-benar seperti SNBT asli?',
      a: 'Ya! Tryout simulasi kami menggunakan format, tipe soal, durasi, dan sistem penilaian yang identik dengan SNBT. Termasuk soal dengan stimulus panjang, persamaan LaTeX, dan tipe majemuk. Skornya juga dikonversi ke skala UTBK.',
    },
    {
      q: 'Bagaimana cara kerja fitur Rasionalisasi PTN?',
      a: 'Sistem kami menganalisis skor tryoutmu, tren nilai passing grade 5 tahun terakhir, tingkat persaingan per prodi, dan pola penerimaan SNBT. Hasilnya menunjukkan persentase peluang masuk untuk setiap PTN dan prodi yang kamu targetkan — berbasis data nyata, bukan spekulasi.',
    },
    {
      q: 'Apakah ada garansi lolos PTN?',
      a: 'Paket Elite menawarkan garansi refund jika kamu tidak lolos PTN (dengan syarat telah menyelesaikan minimal 80% program yang ditetapkan). Detail syarat dan ketentuan bisa dibaca di halaman Paket Elite.',
    },
    {
      q: 'Bisa diakses dari HP?',
      a: 'Tentu! GASPOLPTN fully responsive dan tersedia di semua perangkat — smartphone, tablet, dan desktop. Kamu bisa drilling soal kapanpun, dimanapun.',
    },
  ];

  return (
    <div className="min-h-screen bg-[#06040f] text-white overflow-x-hidden" style={{ fontFamily: "'Plus Jakarta Sans', sans-serif" }}>

      {/* ─── Navbar ─── */}
      <header className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${scrolled ? 'bg-[#06040f]/95 backdrop-blur-xl border-b border-white/10 shadow-lg shadow-violet-900/20' : 'bg-transparent'}`}>
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16 md:h-20">
            {/* Logo */}
            <div className="flex items-center gap-2">
              <div className="w-9 h-9 bg-gradient-to-br from-violet-500 to-purple-700 rounded-xl flex items-center justify-center shadow-lg shadow-violet-900/50">
                <Rocket className="w-5 h-5 text-white" />
              </div>
              <div>
                <span className="text-lg font-bold tracking-tight bg-gradient-to-r from-white to-violet-200 bg-clip-text text-transparent">GASPOLPTN</span>
                <span className="hidden sm:block text-[10px] text-violet-400 leading-none -mt-0.5 ml-0.5">by Edvion</span>
              </div>
            </div>

            {/* Nav links */}
            <nav className="hidden md:flex items-center gap-8">
              {['Fitur', 'Paket', 'Testimoni', 'FAQ', 'Untuk Sekolah'].map(item => (
                <a key={item} href={`#${item.toLowerCase().replace(' ', '-')}`} className="text-sm text-white/70 hover:text-white transition-colors font-medium">{item}</a>
              ))}
            </nav>

            {/* CTA */}
            <div className="hidden md:flex items-center gap-3">
              <button onClick={onLogin} className="text-sm font-semibold text-white/80 hover:text-white transition-colors px-4 py-2">
                Masuk
              </button>
              <button
                onClick={onGetStarted}
                className="text-sm font-bold px-5 py-2.5 rounded-xl bg-gradient-to-r from-violet-600 to-purple-600 hover:from-violet-500 hover:to-purple-500 transition-all shadow-lg shadow-violet-900/40 hover:shadow-violet-900/60 hover:-translate-y-0.5"
              >
                Daftar Gratis
              </button>
            </div>

            {/* Mobile menu btn */}
            <button onClick={() => setMobileMenuOpen(!mobileMenuOpen)} className="md:hidden text-white/80 hover:text-white p-2">
              {mobileMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
            </button>
          </div>
        </div>

        {/* Mobile menu */}
        {mobileMenuOpen && (
          <div className="md:hidden bg-[#0d0a20] border-t border-white/10 px-4 py-4 flex flex-col gap-3">
            {['Fitur', 'Paket', 'Testimoni', 'FAQ', 'Untuk Sekolah'].map(item => (
              <a key={item} href={`#${item.toLowerCase()}`} className="text-sm text-white/70 py-2 border-b border-white/5">{item}</a>
            ))}
            <button onClick={onLogin} className="text-sm font-semibold text-white/80 py-2 text-left">Masuk</button>
            <button onClick={onGetStarted} className="text-sm font-bold px-5 py-3 rounded-xl bg-gradient-to-r from-violet-600 to-purple-600 text-center">Daftar Gratis</button>
          </div>
        )}
      </header>

      {/* ─── Hero ─── */}
      <section ref={heroRef} className="relative min-h-screen flex items-center overflow-hidden pt-20">
        {/* BG gradients */}
        <div className="absolute inset-0 pointer-events-none">
          <div className="absolute top-20 left-1/4 w-96 h-96 bg-violet-600/30 rounded-full blur-3xl" />
          <div className="absolute bottom-20 right-1/4 w-80 h-80 bg-purple-700/25 rounded-full blur-3xl" />
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-indigo-800/15 rounded-full blur-3xl" />
          {/* Grid overlay */}
          <div className="absolute inset-0 opacity-[0.04]" style={{ backgroundImage: 'linear-gradient(rgba(255,255,255,.4) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,.4) 1px, transparent 1px)', backgroundSize: '60px 60px' }} />
        </div>

        <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16 md:py-24">
          <div className="grid lg:grid-cols-2 gap-12 lg:gap-16 items-center">
            {/* Left content */}
            <div>
              <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-violet-900/60 border border-violet-700/50 text-violet-300 text-xs font-semibold mb-6 backdrop-blur-sm">
                <Rocket className="w-3.5 h-3.5" />
                Platform Tryout SNBT #1 Indonesia
              </div>

              <h1 className="text-4xl sm:text-5xl lg:text-6xl font-black leading-[1.1] mb-6 tracking-tight" style={{ fontFamily: "'Outfit', sans-serif" }}>
                Lolos{' '}
                <span className="bg-gradient-to-r from-violet-400 via-purple-400 to-fuchsia-400 bg-clip-text text-transparent">
                  PTN Impianmu
                </span>
                {' '}dengan Cara Paling Efektif
              </h1>

              <p className="text-lg text-white/65 leading-relaxed mb-8 max-w-lg" style={{ fontFamily: "'Outfit', sans-serif" }}>
                Drilling adaptif terstruktur, simulasi tryout SNBT, dan rasionalisasi PTN berbasis data historis. Bergabung dengan 52.000+ siswa yang sudah buktikan hasilnya.
              </p>

              <div className="flex flex-col sm:flex-row gap-4 mb-10">
                <button
                  onClick={onGetStarted}
                  className="flex items-center justify-center gap-2 px-7 py-4 rounded-2xl bg-gradient-to-r from-violet-600 to-purple-600 hover:from-violet-500 hover:to-purple-500 font-bold text-base transition-all shadow-xl shadow-violet-900/50 hover:shadow-violet-900/70 hover:-translate-y-1 group"
                >
                  Mulai Gratis Sekarang
                  <ArrowRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
                </button>
                <button className="flex items-center justify-center gap-2 px-7 py-4 rounded-2xl border border-white/20 hover:border-white/40 font-semibold text-base transition-all hover:bg-white/5 group">
                  <Play className="w-5 h-5 text-violet-400" />
                  Tonton Demo
                </button>
              </div>

              {/* Mini social proof */}
              <div className="flex items-center gap-4">
                <div className="flex -space-x-2">
                  {['RA', 'FN', 'SN', 'BP', 'DK'].map((initials, i) => (
                    <div key={i} className={`w-9 h-9 rounded-full border-2 border-[#06040f] flex items-center justify-center text-xs font-bold bg-gradient-to-br ${['from-violet-400 to-purple-500', 'from-blue-400 to-cyan-500', 'from-emerald-400 to-teal-500', 'from-rose-400 to-pink-500', 'from-amber-400 to-orange-500'][i]}`}>
                      {initials}
                    </div>
                  ))}
                </div>
                <div>
                  <div className="flex items-center gap-1">
                    {[1,2,3,4,5].map(i => <Star key={i} className="w-4 h-4 text-amber-400 fill-amber-400" />)}
                    <span className="text-white font-bold text-sm ml-1">4.9</span>
                  </div>
                  <p className="text-white/50 text-xs">dari 12.000+ ulasan siswa</p>
                </div>
              </div>
            </div>

            {/* Right: floating dashboard card */}
            <div className="relative hidden lg:block">
              {/* Main card */}
              <div className="relative bg-white/5 border border-white/10 rounded-3xl p-6 backdrop-blur-xl shadow-2xl">
                <div className="flex items-center gap-3 mb-5">
                  <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-violet-500 to-purple-600 flex items-center justify-center">
                    <GraduationCap className="w-5 h-5 text-white" />
                  </div>
                  <div>
                    <p className="text-sm font-bold">Rizka Amalia</p>
                    <p className="text-xs text-white/50">SMAN 1 Bandung · Kelas 12</p>
                  </div>
                  <div className="ml-auto">
                    <span className="text-xs bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 px-2 py-1 rounded-full font-semibold">Live</span>
                  </div>
                </div>

                {/* Score bar */}
                <div className="mb-4">
                  <div className="flex justify-between text-xs mb-1.5">
                    <span className="text-white/60">Skor UTBK Prediksi</span>
                    <span className="font-bold text-violet-400">712 / 1000</span>
                  </div>
                  <div className="h-2 bg-white/10 rounded-full overflow-hidden">
                    <div className="h-full w-[71.2%] bg-gradient-to-r from-violet-500 to-purple-500 rounded-full" />
                  </div>
                </div>

                {/* PTN predictions */}
                <p className="text-xs text-white/50 mb-3 font-medium uppercase tracking-wider">Rasionalisasi PTN</p>
                <div className="space-y-2.5">
                  {[
                    { ptn: 'Teknik Informatika UI', chance: 78, color: 'bg-emerald-500' },
                    { ptn: 'Sistem Informasi UGM', chance: 91, color: 'bg-violet-500' },
                    { ptn: 'Teknik Komputer ITB', chance: 62, color: 'bg-amber-500' },
                  ].map(({ ptn, chance, color }) => (
                    <div key={ptn} className="flex items-center gap-3">
                      <div className="flex-1">
                        <div className="flex justify-between text-xs mb-1">
                          <span className="text-white/80 font-medium">{ptn}</span>
                          <span className="font-bold text-white">{chance}%</span>
                        </div>
                        <div className="h-1.5 bg-white/10 rounded-full overflow-hidden">
                          <div className={`h-full ${color} rounded-full`} style={{ width: `${chance}%` }} />
                        </div>
                      </div>
                    </div>
                  ))}
                </div>

                {/* Bottom stats */}
                <div className="grid grid-cols-3 gap-2 mt-5 pt-4 border-t border-white/10">
                  {[
                    { val: '1.240', label: 'Soal Dikerjakan' },
                    { val: '24', label: 'Hari Streak' },
                    { val: '#312', label: 'Rank Nasional' },
                  ].map(({ val, label }) => (
                    <div key={label} className="text-center">
                      <p className="text-base font-black text-white">{val}</p>
                      <p className="text-[10px] text-white/50 leading-tight mt-0.5">{label}</p>
                    </div>
                  ))}
                </div>
              </div>

              {/* Floating badge: streak */}
              <div className="absolute -top-4 -right-4 bg-gradient-to-br from-amber-500 to-orange-500 text-white rounded-2xl px-3.5 py-2.5 shadow-xl shadow-orange-900/40">
                <div className="flex items-center gap-1.5">
                  <Trophy className="w-4 h-4" />
                  <div>
                    <p className="text-xs font-black">24 Day Streak!</p>
                    <p className="text-[10px] opacity-80">Top 1% nasional</p>
                  </div>
                </div>
              </div>

              {/* Floating badge: notification */}
              <div className="absolute -bottom-4 -left-4 bg-white/10 border border-white/20 backdrop-blur-xl text-white rounded-2xl px-3.5 py-2.5 shadow-xl">
                <div className="flex items-center gap-2">
                  <div className="w-7 h-7 rounded-lg bg-emerald-500/30 flex items-center justify-center">
                    <CheckCircle className="w-4 h-4 text-emerald-400" />
                  </div>
                  <div>
                    <p className="text-xs font-bold">Tryout Selesai!</p>
                    <p className="text-[10px] text-white/60">Skor naik +48 poin 🎉</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Scroll indicator */}
        <div className="absolute bottom-8 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2 text-white/30">
          <span className="text-xs">Scroll</span>
          <ChevronDown className="w-4 h-4 animate-bounce" />
        </div>
      </section>

      {/* ─── Stats ─── */}
      <section className="py-16 border-y border-white/10 bg-white/[0.02]">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-8">
            {stats.map(({ value, label, icon: Icon }) => (
              <div key={label} className="text-center">
                <div className="flex justify-center mb-3">
                  <div className="w-12 h-12 rounded-2xl bg-violet-900/40 border border-violet-700/30 flex items-center justify-center">
                    <Icon className="w-6 h-6 text-violet-400" />
                  </div>
                </div>
                <p className="text-3xl lg:text-4xl font-black bg-gradient-to-r from-white to-violet-200 bg-clip-text text-transparent mb-1" style={{ fontFamily: "'Outfit', sans-serif" }}>{value}</p>
                <p className="text-sm text-white/50 font-medium">{label}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ─── PTN logos ─── */}
      <section className="py-10 overflow-hidden">
        <div className="max-w-7xl mx-auto px-4 mb-6">
          <p className="text-center text-sm text-white/40 font-medium uppercase tracking-widest">Dipercaya siswa yang kini kuliah di</p>
        </div>
        <div className="flex gap-6 animate-[marquee_30s_linear_infinite] whitespace-nowrap" style={{ width: 'max-content' }}>
          {[...ptnLogos, ...ptnLogos].map((ptn, i) => (
            <div key={i} className="flex items-center gap-2 px-5 py-2.5 rounded-full bg-white/5 border border-white/10 text-white/50 text-sm font-semibold hover:text-white/80 transition-colors cursor-default shrink-0">
              <GraduationCap className="w-4 h-4 text-violet-400" />
              {ptn}
            </div>
          ))}
        </div>
      </section>

      {/* ─── Features ─── */}
      <section id="fitur" className="py-20 lg:py-28">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-violet-900/40 border border-violet-700/40 text-violet-300 text-xs font-semibold mb-4">
              <Zap className="w-3.5 h-3.5" /> Fitur Unggulan
            </div>
            <h2 className="text-3xl sm:text-4xl lg:text-5xl font-black mb-4" style={{ fontFamily: "'Outfit', sans-serif" }}>
              Semua yang kamu butuhkan<br />
              <span className="bg-gradient-to-r from-violet-400 to-fuchsia-400 bg-clip-text text-transparent">untuk tembus PTN</span>
            </h2>
            <p className="text-white/55 text-lg max-w-2xl mx-auto">Dari soal harian sampai rasionalisasi PTN berbasis data — semuanya ada di satu platform.</p>
          </div>

          <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-5">
            {features.map(({ icon: Icon, title, desc, color, badge }) => (
              <div key={title} className="group relative bg-white/[0.03] hover:bg-white/[0.06] border border-white/10 hover:border-white/20 rounded-3xl p-6 transition-all duration-300 hover:-translate-y-1 cursor-default">
                <div className="flex items-start justify-between mb-4">
                  <div className={`w-12 h-12 rounded-2xl bg-gradient-to-br ${color} flex items-center justify-center shadow-lg`}>
                    <Icon className="w-6 h-6 text-white" />
                  </div>
                  <span className={`text-xs font-bold px-2.5 py-1 rounded-full bg-gradient-to-r ${color} text-white opacity-90`}>{badge}</span>
                </div>
                <h3 className="text-base font-bold mb-2 text-white">{title}</h3>
                <p className="text-sm text-white/55 leading-relaxed">{desc}</p>
                <div className="absolute inset-0 rounded-3xl bg-gradient-to-br opacity-0 group-hover:opacity-5 transition-opacity pointer-events-none" style={{ backgroundImage: `linear-gradient(135deg, ${color})` }} />
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ─── How it works ─── */}
      <section className="py-20 bg-white/[0.02] border-y border-white/10">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl sm:text-4xl font-black mb-3" style={{ fontFamily: "'Outfit', sans-serif" }}>Cara Kerjanya Simpel</h2>
            <p className="text-white/50">Mulai dari daftar sampai lolos PTN, hanya 4 langkah.</p>
          </div>
          <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-8">
            {[
              { step: '01', title: 'Daftar Gratis', desc: 'Buat akun dalam 30 detik. Tidak perlu kartu kredit.', icon: Users },
              { step: '02', title: 'Ikuti Tes Diagnostik', desc: 'Sistem kami analisis tingkat pemahamanmu di semua subtes SNBT secara terstruktur.', icon: Target },
              { step: '03', title: 'Drill & Tryout', desc: 'Kerjakan soal adaptif harian dan simulasi tryout lengkap.', icon: Target },
              { step: '04', title: 'Analisis & Raih PTN', desc: 'Pantau progress, lihat rasionalisasi berbasis data, dan lolos PTN impianmu!', icon: Trophy },
            ].map(({ step, title, desc, icon: Icon }) => (
              <div key={step} className="relative text-center">
                <div className="relative inline-flex">
                  <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-violet-600 to-purple-700 flex items-center justify-center mb-4 shadow-xl shadow-violet-900/40 mx-auto">
                    <Icon className="w-8 h-8 text-white" />
                  </div>
                  <span className="absolute -top-2 -right-2 w-7 h-7 rounded-full bg-[#06040f] border border-violet-500 text-violet-400 text-xs font-black flex items-center justify-center">{step}</span>
                </div>
                <h3 className="text-base font-bold mb-2">{title}</h3>
                <p className="text-sm text-white/50">{desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ─── Pricing ─── */}
      <section id="paket" className="py-20 lg:py-28">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-12">
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-violet-900/40 border border-violet-700/40 text-violet-300 text-xs font-semibold mb-4">
              <Award className="w-3.5 h-3.5" /> Pilih Paket
            </div>
            <h2 className="text-3xl sm:text-4xl lg:text-5xl font-black mb-4" style={{ fontFamily: "'Outfit', sans-serif" }}>
              Pilih paket<br />
              <span className="bg-gradient-to-r from-violet-400 to-fuchsia-400 bg-clip-text text-transparent">sesuai targetmu</span>
            </h2>
            <p className="text-white/55">Kuota tryout aktif 1 tahun. Bayar sekali, pakai kapan saja. Hemat hingga 40%.</p>
          </div>

          {/* Horizontal scroll card carousel */}
          <div
            className="flex gap-4 overflow-x-auto pb-4"
            style={{ scrollbarWidth: 'none', msOverflowStyle: 'none' }}
          >
            {quotaPackages.map((pkg) => (
              <div
                key={pkg.id}
                className="flex-shrink-0 w-60 bg-white rounded-2xl overflow-hidden shadow-xl shadow-black/40 flex flex-col relative"
              >
                {/* Discount badge */}
                <div className="absolute top-3 left-3 z-10 w-11 h-11 rounded-full bg-orange-500 flex flex-col items-center justify-center shadow-lg">
                  <span className="text-white font-black text-xs leading-none">{pkg.discount}%</span>
                  <span className="text-white/80 text-[9px] leading-none">OFF</span>
                </div>

                {/* Optional popular badge */}
                {pkg.badge && (
                  <div
                    className="absolute top-3 right-3 z-10 text-white text-[10px] font-black px-2 py-0.5 rounded-full"
                    style={{ backgroundColor: pkg.accentColor }}
                  >
                    {pkg.badge}
                  </div>
                )}

                {/* Illustration area */}
                <div className={`h-36 bg-gradient-to-br ${pkg.gradient} flex items-end justify-center pb-2 relative`}>
                  <span className="text-7xl select-none" style={{ filter: 'drop-shadow(0 4px 8px rgba(0,0,0,0.15))' }}>
                    {pkg.emoji}
                  </span>
                </div>

                {/* Card body */}
                <div className="p-4 flex flex-col flex-1">
                  <h3 className="font-bold text-slate-800 text-sm mb-3 leading-snug">{pkg.name}</h3>

                  {/* Price */}
                  <div className="mb-3">
                    <p className="text-slate-400 text-xs line-through leading-none mb-0.5">
                      Rp{pkg.originalPrice.toLocaleString('id-ID')}
                    </p>
                    <p className="font-black text-xl leading-none" style={{ color: '#f97316' }}>
                      Rp{pkg.salePrice.toLocaleString('id-ID')}
                    </p>
                  </div>

                  {/* CTA */}
                  <button
                    onClick={onGetStarted}
                    className="w-full py-2.5 rounded-xl font-bold text-sm text-white transition-all hover:opacity-90 active:scale-95 mb-3"
                    style={{ backgroundColor: '#f97316' }}
                  >
                    Beli Paket
                  </button>

                  {/* Features */}
                  <ul className="space-y-1.5 flex-1">
                    {pkg.features.map(f => (
                      <li key={f} className="flex items-start gap-1.5 text-[11px] text-slate-600 leading-tight">
                        <CheckCircle className="w-3.5 h-3.5 text-emerald-500 mt-0.5 shrink-0" />
                        {f}
                      </li>
                    ))}
                  </ul>

                  {/* Detail link */}
                  <button
                    onClick={onGetStarted}
                    className="mt-3 text-xs font-semibold flex items-center gap-0.5 hover:gap-1.5 transition-all"
                    style={{ color: pkg.accentColor }}
                  >
                    Lihat Detail <ChevronRight className="w-3 h-3" />
                  </button>
                </div>
              </div>
            ))}

            {/* See all CTA card */}
            <div
              onClick={onGetStarted}
              className="flex-shrink-0 w-44 rounded-2xl flex flex-col items-center justify-center p-6 gap-4 cursor-pointer group overflow-hidden relative"
              style={{ background: 'linear-gradient(135deg, #0d9488, #0891b2)' }}
            >
              <div className="absolute inset-0 bg-white/0 group-hover:bg-white/10 transition-colors" />
              <span className="text-5xl">📦</span>
              <p className="text-white font-bold text-center text-sm leading-snug">Lihat semua paket</p>
              <div className="w-10 h-10 rounded-full bg-white/20 group-hover:bg-white/30 flex items-center justify-center transition-colors">
                <ChevronRight className="w-5 h-5 text-white" />
              </div>
            </div>
          </div>

          <p className="text-center text-xs text-white/30 mt-6">*Harga promo terbatas. Pembayaran aman dengan enkripsi SSL.</p>

          {/* ── Voucher Entry ── */}
          <div className="mt-14 max-w-2xl mx-auto">
            <div className="bg-white/[0.04] border border-white/10 rounded-3xl p-7 backdrop-blur-sm">
              <div className="flex items-center gap-3 mb-5">
                <div className="w-10 h-10 rounded-2xl bg-gradient-to-br from-violet-500 to-purple-600 flex items-center justify-center shrink-0">
                  <Tag className="w-5 h-5 text-white" />
                </div>
                <div>
                  <h3 className="font-black text-white text-lg" style={{ fontFamily: "'Outfit', sans-serif" }}>Punya Kode Voucher?</h3>
                  <p className="text-white/50 text-sm">Masukkan kode dan dapatkan akses premium secara langsung</p>
                </div>
              </div>

              <div className="flex gap-2 mb-4">
                <input
                  value={voucherCode}
                  onChange={e => { setVoucherCode(e.target.value.toUpperCase()); setVoucherState('idle'); setVoucherInfo(null); }}
                  onKeyDown={e => e.key === 'Enter' && checkVoucher()}
                  placeholder="Contoh: PROMO-JAN25-50"
                  className="flex-1 px-4 py-3 rounded-xl bg-white/10 border border-white/20 text-white placeholder-white/30 text-sm font-mono tracking-wider focus:outline-none focus:border-violet-500 focus:bg-white/15 transition-all"
                />
                <button
                  onClick={checkVoucher}
                  disabled={!voucherCode.trim() || voucherState === 'checking'}
                  className="px-6 py-3 rounded-xl bg-gradient-to-r from-violet-600 to-purple-600 hover:from-violet-500 hover:to-purple-500 text-white font-bold text-sm transition-all disabled:opacity-50 disabled:cursor-not-allowed shrink-0"
                >
                  {voucherState === 'checking' ? '...' : 'Cek'}
                </button>
              </div>

              {/* Feedback */}
              {voucherState === 'invalid' && (
                <div className="flex items-center gap-2 px-4 py-3 rounded-xl bg-red-900/30 border border-red-500/30 text-red-300 text-sm mb-4">
                  <X className="w-4 h-4 shrink-0" />
                  Kode voucher tidak valid atau sudah habis masa berlakunya.
                </div>
              )}
              {voucherState === 'valid' && voucherInfo && (
                <div className="mb-4">
                  <div className="flex items-start gap-3 px-4 py-4 rounded-xl bg-emerald-900/30 border border-emerald-500/40 mb-3">
                    <CheckCircle className="w-5 h-5 text-emerald-400 shrink-0 mt-0.5" />
                    <div className="flex-1">
                      <p className="text-emerald-300 font-bold text-sm mb-0.5">Voucher valid! 🎉</p>
                      <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-white/60">
                        <span>Paket: <strong className="text-white">{voucherInfo.pkg}</strong></span>
                        <span>Harga: <strong className="text-emerald-400">{voucherInfo.discount}</strong></span>
                        <span>Berlaku s/d: <strong className="text-white">{voucherInfo.expires}</strong></span>
                      </div>
                    </div>
                  </div>
                  <button
                    onClick={onGetStarted}
                    className="w-full py-3.5 rounded-xl bg-gradient-to-r from-emerald-500 to-green-600 hover:from-emerald-400 hover:to-green-500 text-white font-black text-sm transition-all hover:-translate-y-0.5 shadow-lg shadow-emerald-900/40 flex items-center justify-center gap-2"
                  >
                    <Gift className="w-4 h-4" />
                    Daftar & Aktifkan Voucher Sekarang
                    <ChevronRight className="w-4 h-4" />
                  </button>
                </div>
              )}

              {/* Voucher types info */}
              <div className="grid grid-cols-3 gap-3 pt-4 border-t border-white/10">
                {[
                  { icon: Ticket,  label: 'Voucher Akses',    desc: 'Dari admin / event' },
                  { icon: Gift,    label: 'Kode Promo',       desc: 'Diskon & cashback' },
                  { icon: Share2,  label: 'Kode Referral',    desc: 'Dari teman / afiliasi' },
                ].map(({ icon: Icon, label, desc }) => (
                  <div key={label} className="text-center p-3 rounded-xl bg-white/[0.03] border border-white/[0.08]">
                    <Icon className="w-5 h-5 text-violet-400 mx-auto mb-1.5" />
                    <p className="text-xs font-bold text-white/80">{label}</p>
                    <p className="text-[10px] text-white/40 mt-0.5">{desc}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* ── B2C Purchase Flow Steps ── */}
          <div className="mt-10 max-w-4xl mx-auto">
            <p className="text-center text-white/40 text-sm font-semibold uppercase tracking-wider mb-6">Cara Beli Akses GASPOLPTN</p>
            <div className="grid sm:grid-cols-4 gap-4">
              {[
                { step: '1', icon: Ticket,       title: 'Dapat Voucher',    desc: 'Beli di minimarket, website, atau dapat dari referral teman', color: 'from-violet-600 to-purple-600' },
                { step: '2', icon: Users,         title: 'Buat Akun',        desc: 'Daftar gratis dalam 30 detik, tidak perlu kartu kredit', color: 'from-blue-600 to-cyan-600' },
                { step: '3', icon: Tag,           title: 'Input Kode',       desc: 'Masukkan kode voucher di dashboard siswa setelah login', color: 'from-emerald-600 to-green-600' },
                { step: '4', icon: Zap,           title: 'Akses Aktif!',     desc: 'Langsung mulai drilling, tryout, dan rasionalisasi PTN', color: 'from-amber-500 to-orange-500' },
              ].map(({ step, icon: Icon, title, desc, color }) => (
                <div key={step} className="relative text-center">
                  <div className={`w-12 h-12 rounded-2xl bg-gradient-to-br ${color} flex items-center justify-center mx-auto mb-3 shadow-lg`}>
                    <Icon className="w-6 h-6 text-white" />
                  </div>
                  <div className="absolute top-6 left-[calc(50%+24px)] w-[calc(100%-48px)] h-px bg-white/10 hidden sm:block" />
                  <p className="text-[10px] font-black text-white/30 uppercase tracking-widest mb-1">Step {step}</p>
                  <p className="font-bold text-white text-sm mb-1">{title}</p>
                  <p className="text-xs text-white/45 leading-relaxed">{desc}</p>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* ─── Testimonials ─── */}
      <section id="testimoni" className="py-20 bg-white/[0.02] border-y border-white/10">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl sm:text-4xl lg:text-5xl font-black mb-4" style={{ fontFamily: "'Outfit', sans-serif" }}>
              Ribuan siswa sudah buktikan
            </h2>
            <p className="text-white/50">Kata mereka yang sudah lolos PTN impian.</p>
          </div>

          <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-5">
            {testimonials.map(({ name, school, ptn, avatar, color, score, text, stars }) => (
              <div key={name} className="bg-white/[0.04] border border-white/10 rounded-3xl p-5 hover:border-white/20 transition-colors">
                <div className="flex items-center gap-1 mb-3">
                  {Array.from({ length: stars }).map((_, i) => (
                    <Star key={i} className="w-3.5 h-3.5 text-amber-400 fill-amber-400" />
                  ))}
                </div>
                <p className="text-sm text-white/70 leading-relaxed mb-5 italic">&ldquo;{text}&rdquo;</p>
                <div className="flex items-center gap-3 pt-4 border-t border-white/10">
                  <div className={`w-10 h-10 rounded-full bg-gradient-to-br ${color} flex items-center justify-center text-xs font-black text-white shrink-0`}>{avatar}</div>
                  <div>
                    <p className="text-sm font-bold text-white">{name}</p>
                    <p className="text-xs text-white/45">{school}</p>
                    <p className="text-xs text-violet-400 font-semibold mt-0.5">→ {ptn}</p>
                  </div>
                  <div className="ml-auto text-right">
                    <p className="text-lg font-black text-white" style={{ fontFamily: "'Outfit', sans-serif" }}>{score}</p>
                    <p className="text-[10px] text-white/40">skor UTBK</p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ─── For Schools (B2B) ─── */}
      <section id="untuk-sekolah" className="py-20 lg:py-28">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="relative bg-gradient-to-br from-violet-900/40 to-purple-900/30 border border-violet-700/30 rounded-[2rem] overflow-hidden p-10 lg:p-16">
            {/* BG deco */}
            <div className="absolute top-0 right-0 w-80 h-80 bg-violet-600/20 rounded-full blur-3xl" />
            <div className="absolute bottom-0 left-0 w-60 h-60 bg-purple-700/20 rounded-full blur-3xl" />

            <div className="relative grid lg:grid-cols-2 gap-12 items-center">
              <div>
                <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-violet-900/60 border border-violet-700/50 text-violet-300 text-xs font-semibold mb-6">
                  <Building2 className="w-3.5 h-3.5" /> Untuk Sekolah &amp; Lembaga
                </div>
                <h2 className="text-3xl lg:text-4xl font-black mb-4" style={{ fontFamily: "'Outfit', sans-serif" }}>
                  Tingkatkan angka kelulusan PTN sekolahmu
                </h2>
                <p className="text-white/60 leading-relaxed mb-8">
                  Bergabunglah dengan 512+ sekolah mitra. Dapatkan dashboard analitik sekolah, manajemen siswa terpusat, laporan perkembangan, dan dukungan tim edukasi kami.
                </p>
                <ul className="space-y-3 mb-8">
                  {[
                    'Dashboard analitik real-time per kelas & siswa',
                    'Tryout internal terjadwal untuk seluruh siswa',
                    'Laporan bulanan ke wali kelas & orang tua',
                    'Revenue sharing program untuk sekolah mitra',
                    'Dukungan teknis & pelatihan tim pengajar',
                  ].map(f => (
                    <li key={f} className="flex items-center gap-3 text-sm text-white/75">
                      <CheckCircle className="w-4 h-4 text-emerald-400 shrink-0" />
                      {f}
                    </li>
                  ))}
                </ul>
                <div className="flex flex-col sm:flex-row gap-3">
                  <button onClick={onLogin} className="px-6 py-3.5 rounded-xl bg-white text-violet-900 font-bold text-sm hover:bg-violet-50 transition-colors shadow-lg">
                    Daftar Sebagai Sekolah Mitra
                  </button>
                  <button className="px-6 py-3.5 rounded-xl border border-white/30 font-semibold text-sm hover:bg-white/10 transition-colors flex items-center gap-2">
                    <MessageSquare className="w-4 h-4" /> Hubungi Tim Kami
                  </button>
                </div>
              </div>

              {/* School stats card */}
              <div className="bg-white/5 border border-white/10 rounded-2xl p-6 backdrop-blur-sm">
                <p className="text-sm font-bold text-white/60 mb-4 uppercase tracking-wider">Rata-rata Hasil Sekolah Mitra</p>
                <div className="grid grid-cols-2 gap-4 mb-6">
                  {[
                    { val: '+34%', label: 'Peningkatan kelulusan PTN', color: 'text-emerald-400' },
                    { val: '92%', label: 'Kepuasan sekolah mitra', color: 'text-violet-400' },
                    { val: '4.2x', label: 'Lebih banyak siswa lolos', color: 'text-amber-400' },
                    { val: '512+', label: 'Sekolah aktif bermitra', color: 'text-cyan-400' },
                  ].map(({ val, label, color }) => (
                    <div key={label} className="bg-white/5 rounded-xl p-4">
                      <p className={`text-2xl font-black mb-1 ${color}`} style={{ fontFamily: "'Outfit', sans-serif" }}>{val}</p>
                      <p className="text-xs text-white/50 leading-tight">{label}</p>
                    </div>
                  ))}
                </div>
                <div className="flex items-center gap-3 p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-xl">
                  <TrendingUp className="w-5 h-5 text-emerald-400 shrink-0" />
                  <p className="text-xs text-emerald-300">Sekolah mitra rata-rata melihat peningkatan signifikan dalam 3 bulan pertama.</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ─── FAQ ─── */}
      <section id="faq" className="py-20 border-t border-white/10">
        <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-12">
            <h2 className="text-3xl sm:text-4xl font-black mb-3" style={{ fontFamily: "'Outfit', sans-serif" }}>Pertanyaan Umum</h2>
            <p className="text-white/50">Ada yang masih bingung? Kami jawab di sini.</p>
          </div>
          <div className="space-y-3">
            {faqs.map(({ q, a }, i) => (
              <div key={i} className="bg-white/[0.03] border border-white/10 rounded-2xl overflow-hidden">
                <button
                  onClick={() => setActiveAccordion(activeAccordion === i ? null : i)}
                  className="w-full flex items-center justify-between p-5 text-left hover:bg-white/5 transition-colors"
                >
                  <span className="font-semibold text-sm text-white/90 pr-4">{q}</span>
                  <ChevronDown className={`w-5 h-5 text-white/40 shrink-0 transition-transform ${activeAccordion === i ? 'rotate-180' : ''}`} />
                </button>
                {activeAccordion === i && (
                  <div className="px-5 pb-5">
                    <p className="text-sm text-white/60 leading-relaxed">{a}</p>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ─── Final CTA ─── */}
      <section className="py-20 px-4">
        <div className="max-w-4xl mx-auto text-center relative">
          <div className="absolute inset-0 bg-gradient-to-r from-violet-600/20 to-purple-600/20 rounded-3xl blur-2xl" />
          <div className="relative bg-gradient-to-br from-violet-900/50 to-purple-900/40 border border-violet-700/40 rounded-3xl p-12">
            <Rocket className="w-12 h-12 text-violet-400 mx-auto mb-6" />
            <h2 className="text-3xl sm:text-4xl lg:text-5xl font-black mb-4" style={{ fontFamily: "'Outfit', sans-serif" }}>
              Siap gaspol ke PTN impianmu?
            </h2>
            <p className="text-white/60 text-lg mb-8 max-w-xl mx-auto">
              Daftar gratis sekarang dan mulai perjalananmu menuju PTN impian bersama 52.000+ siswa lainnya.
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <button onClick={onGetStarted} className="flex items-center justify-center gap-2 px-8 py-4 rounded-2xl bg-gradient-to-r from-violet-600 to-purple-600 hover:from-violet-500 hover:to-purple-500 font-black text-base transition-all shadow-xl shadow-violet-900/50 hover:-translate-y-1 group">
                Daftar Gratis Sekarang
                <ArrowRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
              </button>
              <button onClick={onLogin} className="flex items-center justify-center gap-2 px-8 py-4 rounded-2xl border border-white/20 hover:border-white/40 font-semibold text-base transition-all hover:bg-white/5">
                Sudah punya akun? Masuk
              </button>
            </div>
          </div>
        </div>
      </section>

      {/* ─── Footer ─── */}
      <footer className="border-t border-white/10 py-12 px-4">
        <div className="max-w-7xl mx-auto">
          <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-8 mb-10">
            <div>
              <div className="flex items-center gap-2 mb-4">
                <div className="w-8 h-8 bg-gradient-to-br from-violet-500 to-purple-700 rounded-lg flex items-center justify-center">
                  <Rocket className="w-4 h-4 text-white" />
                </div>
                <span className="font-bold text-white">GASPOLPTN</span>
              </div>
              <p className="text-sm text-white/45 leading-relaxed mb-4">Platform tryout SNBT dan rasionalisasi PTN terlengkap di Indonesia. Bantu siswa wujudkan PTN impian.</p>
              <div className="flex gap-3">
                {[Instagram, Youtube, Twitter].map((Icon, i) => (
                  <a key={i} href="#" className="w-8 h-8 rounded-lg bg-white/10 hover:bg-white/20 flex items-center justify-center transition-colors">
                    <Icon className="w-4 h-4 text-white/70" />
                  </a>
                ))}
              </div>
            </div>

            {[
              { title: 'Platform', links: ['Fitur', 'Paket & Harga', 'Demo Gratis', 'Blog & Tips SNBT', 'Jadwal Tryout'] },
              { title: 'Sekolah', links: ['Program Mitra', 'Dashboard Sekolah', 'Revenue Sharing', 'Kontak B2B', 'Studi Kasus'] },
              { title: 'Dukungan', links: ['Pusat Bantuan', 'Kontak Kami', 'Syarat & Ketentuan', 'Kebijakan Privasi', 'Status Sistem'] },
            ].map(({ title, links }) => (
              <div key={title}>
                <p className="text-sm font-bold text-white mb-4">{title}</p>
                <ul className="space-y-2.5">
                  {links.map(link => (
                    <li key={link}><a href="#" className="text-sm text-white/45 hover:text-white/80 transition-colors">{link}</a></li>
                  ))}
                </ul>
              </div>
            ))}
          </div>

          <div className="flex flex-col sm:flex-row items-center justify-between pt-6 border-t border-white/10 gap-4">
            <p className="text-xs text-white/30">© 2025 Edvion · GASPOLPTN. All rights reserved.</p>
            <div className="flex items-center gap-6">
              <div className="flex items-center gap-1.5 text-xs text-white/30"><Mail className="w-3.5 h-3.5" /> hello@edvion.id</div>
              <div className="flex items-center gap-1.5 text-xs text-white/30"><Phone className="w-3.5 h-3.5" /> +62 811-1234-5678</div>
              <div className="flex items-center gap-1.5 text-xs text-white/30"><MapPin className="w-3.5 h-3.5" /> Jakarta, Indonesia</div>
            </div>
          </div>
        </div>
      </footer>

      <style>{`
        @keyframes marquee {
          from { transform: translateX(0); }
          to { transform: translateX(-50%); }
        }
      `}</style>
    </div>
  );
}
