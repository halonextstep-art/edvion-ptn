import { useState } from 'react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Card } from './ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import {
  Rocket,
  School,
  Users,
  Eye,
  EyeOff,
  Sparkles,
  Trophy,
  Target,
  Brain,
  ArrowLeft,
  Layers
} from 'lucide-react';

type UserRole = 'admin' | 'school' | 'student' | 'content';

interface User {
  id: string;
  name: string;
  email: string;
  role: UserRole;
  schoolName?: string;
  students?: number;
}

interface LoginPageProps {
  onLogin: (user: User) => void;
  onBack?: () => void;
}

const demoAccounts = {
  admin: {
    email: 'admin@gaspolptn.com',
    password: 'admin123',
    user: { id: '1', name: 'Admin Pusat', email: 'admin@gaspolptn.com', role: 'admin' as const }
  },
  school: {
    email: 'sekolah@sman1.sch.id',
    password: 'sekolah123',
    user: { id: '2', name: 'SMA Negeri 1 Jakarta', email: 'sekolah@sman1.sch.id', role: 'school' as const, schoolName: 'SMA Negeri 1 Jakarta', students: 245 }
  },
  student: {
    email: 'siswa@student.com',
    password: 'siswa123',
    user: { id: '3', name: 'Ahmad Fauzi', email: 'siswa@student.com', role: 'student' as const, schoolName: 'SMA Negeri 1 Jakarta' }
  },
  content: {
    email: 'konten@edvion.id',
    password: 'konten123',
    user: { id: '4', name: 'Tim Konten Edvion', email: 'konten@edvion.id', role: 'content' as const }
  },
};

type TabKey = keyof typeof demoAccounts;

export default function LoginPage({ onLogin, onBack }: LoginPageProps) {
  const [activeTab, setActiveTab] = useState<TabKey>('student');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState('');

  const handleLogin = (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    const account = demoAccounts[activeTab];
    if (email === account.email && password === account.password) {
      onLogin(account.user);
    } else {
      setError('Email atau password salah!');
    }
  };

  const handleDemoLogin = (role: TabKey) => {
    onLogin(demoAccounts[role].user);
  };

  const placeholders: Record<TabKey, string> = {
    student: 'siswa@student.com',
    school: 'sekolah@sman1.sch.id',
    admin: 'admin@gaspolptn.com',
    content: 'konten@edvion.id',
  };

  const roleInfo: Record<TabKey, { icon: string; title: string; desc: string }> = {
    student: { icon: '👨‍🎓', title: 'Portal Siswa', desc: 'Akses drilling tryout, rasionalisasi PTN, tracking progress, dan leaderboard nasional' },
    school: { icon: '🏫', title: 'Portal Sekolah', desc: 'Monitoring siswa, analytics performa kolektif, dan laporan lengkap' },
    admin: { icon: '👨‍💼', title: 'Admin Pusat', desc: 'Kelola bank soal, event, mitra sekolah, dan analytics nasional' },
    content: { icon: '✍️', title: 'Tim Konten Internal', desc: 'Input soal, buat paket tryout, dan kelola bank soal GASPOLPTN' },
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 flex items-center justify-center p-4">
      <div className="absolute inset-0 opacity-30" style={{ backgroundImage: "url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjAwIiBoZWlnaHQ9IjIwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZGVmcz48cGF0dGVybiBpZD0iZ3JpZCIgd2lkdGg9IjQwIiBoZWlnaHQ9IjQwIiBwYXR0ZXJuVW5pdHM9InVzZXJTcGFjZU9uVXNlIj48cGF0aCBkPSJNIDQwIDAgTCAwIDAgMCA0MCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSJ3aGl0ZSIgc3Ryb2tlLW9wYWNpdHk9IjAuMSIgc3Ryb2tlLXdpZHRoPSIxIi8+PC9wYXR0ZXJuPjwvZGVmcz48cmVjdCB3aWR0aD0iMTAwJSIgaGVpZ2h0PSIxMDAlIiBmaWxsPSJ1cmwoI2dyaWQpIi8+PC9zdmc+')" }} />

      <div className="relative w-full max-w-6xl">
        {/* Back button */}
        {onBack && (
          <button onClick={onBack} className="absolute -top-12 left-0 flex items-center gap-2 text-white/80 hover:text-white text-sm font-medium transition-colors">
            <ArrowLeft className="w-4 h-4" /> Kembali ke Beranda
          </button>
        )}

        <div className="grid lg:grid-cols-2 gap-8 items-center">
          {/* Left Side - Branding */}
          <div className="text-white hidden lg:block">
            <div className="mb-8">
              <div className="inline-flex items-center gap-2 bg-white/10 backdrop-blur-sm px-4 py-2 rounded-full mb-6">
                <Sparkles className="w-4 h-4 text-yellow-300" />
                <span className="text-sm">Platform #1 Persiapan SNBT 2025</span>
              </div>
              <h1 className="text-6xl font-black mb-4">
                <span className="block">GASPOL</span>
                <span className="block bg-gradient-to-r from-yellow-300 to-orange-400 bg-clip-text text-transparent">PTN</span>
              </h1>
              <p className="text-xl text-white/90 mb-8">Sistem End-to-End untuk Drilling Tryout & Rasionalisasi SNBT</p>
            </div>
            <div className="space-y-4">
              {[
                { icon: Brain, label: 'AI-Powered Analysis', sub: 'Adaptive learning system', from: 'from-purple-400', to: 'to-pink-400' },
                { icon: Target, label: 'Rasionalisasi PTN', sub: 'Prediksi peluang real-time', from: 'from-blue-400', to: 'to-cyan-400' },
                { icon: Trophy, label: '10K+ Soal Berkualitas', sub: 'Bank soal terlengkap', from: 'from-orange-400', to: 'to-red-400' },
                { icon: Layers, label: 'Tim Konten Internal', sub: 'Manajemen paket soal', from: 'from-emerald-400', to: 'to-teal-400' },
              ].map(({ icon: Icon, label, sub, from, to }) => (
                <div key={label} className="flex items-center gap-3 p-4 bg-white/10 backdrop-blur-sm rounded-lg">
                  <div className={`w-10 h-10 rounded-lg bg-gradient-to-br ${from} ${to} flex items-center justify-center`}>
                    <Icon className="w-5 h-5 text-white" />
                  </div>
                  <div>
                    <div className="font-semibold">{label}</div>
                    <div className="text-sm text-white/70">{sub}</div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Right Side - Login Form */}
          <Card className="p-8 shadow-2xl">
            <div className="text-center mb-6">
              <h2 className="text-2xl font-bold mb-1">Selamat Datang! 👋</h2>
              <p className="text-muted-foreground text-sm">Login untuk akses dashboard</p>
            </div>

            <Tabs value={activeTab} onValueChange={v => { setActiveTab(v as TabKey); setError(''); setEmail(''); setPassword(''); }} className="space-y-5">
              <TabsList className="grid w-full grid-cols-4">
                <TabsTrigger value="student" className="gap-1.5 text-xs">
                  <Rocket className="w-3.5 h-3.5" />
                  <span className="hidden sm:inline">Siswa</span>
                </TabsTrigger>
                <TabsTrigger value="school" className="gap-1.5 text-xs">
                  <School className="w-3.5 h-3.5" />
                  <span className="hidden sm:inline">Sekolah</span>
                </TabsTrigger>
                <TabsTrigger value="admin" className="gap-1.5 text-xs">
                  <Users className="w-3.5 h-3.5" />
                  <span className="hidden sm:inline">Admin</span>
                </TabsTrigger>
                <TabsTrigger value="content" className="gap-1.5 text-xs">
                  <Layers className="w-3.5 h-3.5" />
                  <span className="hidden sm:inline">Konten</span>
                </TabsTrigger>
              </TabsList>

              <form onSubmit={handleLogin} className="space-y-4">
                <div>
                  <Label htmlFor="email">Email</Label>
                  <Input
                    id="email"
                    type="email"
                    placeholder={placeholders[activeTab]}
                    value={email}
                    onChange={e => setEmail(e.target.value)}
                    required
                    className="mt-1.5"
                  />
                </div>
                <div>
                  <Label htmlFor="password">Password</Label>
                  <div className="relative mt-1.5">
                    <Input
                      id="password"
                      type={showPassword ? 'text' : 'password'}
                      placeholder="Masukkan password"
                      value={password}
                      onChange={e => setPassword(e.target.value)}
                      required
                    />
                    <button type="button" onClick={() => setShowPassword(!showPassword)} className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground">
                      {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </button>
                  </div>
                </div>

                {error && <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-600">{error}</div>}

                <Button type="submit" size="lg" className="w-full bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-700 hover:to-purple-700">
                  Login
                </Button>
              </form>

              {/* Demo credentials */}
              <div className="p-4 bg-slate-50 rounded-lg border border-slate-200">
                <div className="text-sm mb-2"><strong>🔐 Demo Credentials:</strong></div>
                <div className="space-y-0.5 text-xs text-muted-foreground">
                  <div>Email: <span className="font-mono text-slate-700">{demoAccounts[activeTab].email}</span></div>
                  <div>Password: <span className="font-mono text-slate-700">{demoAccounts[activeTab].password}</span></div>
                </div>
                <Button type="button" variant="outline" size="sm" className="w-full mt-3" onClick={() => handleDemoLogin(activeTab)}>
                  Quick Demo Login
                </Button>
              </div>

              {/* Role description */}
              <div className="p-4 bg-gradient-to-br from-purple-50 to-pink-50 rounded-lg border border-purple-200">
                <strong className="text-sm">{roleInfo[activeTab].icon} {roleInfo[activeTab].title}</strong>
                <p className="text-xs text-muted-foreground mt-1">{roleInfo[activeTab].desc}</p>
              </div>
            </Tabs>

            <div className="mt-5 text-center text-sm text-muted-foreground">
              Belum punya akun? <a href="#" className="text-indigo-600 hover:underline">Daftar Sekarang</a>
            </div>
          </Card>
        </div>

        <div className="lg:hidden text-center mt-8 text-white">
          <p className="text-sm">© 2025 GASPOLPTN by Edvion. Platform Persiapan SNBT Terlengkap</p>
        </div>
      </div>
    </div>
  );
}
