import { useState } from 'react';
import { Button } from '../ui/button';
import { Card } from '../ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { toast } from 'sonner';
import type { SessionResult } from './TryoutPlayer';
import {
  LogOut, LayoutDashboard, Target, Trophy, TrendingUp, Zap, BookOpen, Crown, Play, X, ChevronLeft, Tag
} from 'lucide-react';
import StudentOverview from './StudentOverview';
import DrillingZone from './DrillingZone';
import RasionalisasiSNBT from './RasionalisasiSNBT';
import StudentProgress from './StudentProgress';
import Leaderboard from './Leaderboard';
import TryoutPlayer from './TryoutPlayer';
import VoucherActivation from './VoucherActivation';
import NotificationBell from '../shared/NotificationBell';

interface StudentDashboardProps {
  onLogout: () => void;
  userData?: { name?: string; email?: string; schoolName?: string };
}

export type TryoutSession = {
  title: string;
  type: 'tryout' | 'drilling' | 'mini';
  durationMinutes: number;
};

const TRYOUT_SESSIONS: TryoutSession[] = [
  { title: 'Tryout Nasional #46 — SNBT Full Simulasi', type: 'tryout', durationMinutes: 30 },
  { title: 'Mini Tryout Penalaran Umum', type: 'mini', durationMinutes: 15 },
  { title: 'Drilling Matematika — Level Sedang', type: 'drilling', durationMinutes: 20 },
  { title: 'Drilling Penalaran Umum — Level Mudah', type: 'drilling', durationMinutes: 15 },
];

export default function StudentDashboard({ onLogout, userData }: StudentDashboardProps) {
  const [activeTab, setActiveTab] = useState('overview');
  const [activeSession, setActiveSession] = useState<TryoutSession | null>(null);
  const [showVoucher, setShowVoucher] = useState(false);

  const name = userData?.name || 'Student';
  const initials = name.split(' ').map((n: string) => n[0]).join('').slice(0, 2).toUpperCase();

  const startSession = (session: TryoutSession) => {
    setActiveSession(session);
  };

  const endSession = (result?: SessionResult) => {
    setActiveSession(null);
    if (result) {
      const grade = result.score >= 700 ? 'A' : result.score >= 600 ? 'B' : result.score >= 500 ? 'C' : 'D';
      toast.success(`Sesi selesai! Skor: ${result.score} (${grade}) · Akurasi: ${result.accuracy}%`, {
        description: `${result.correct} benar · ${result.wrong} salah dari ${result.total} soal`,
        duration: 6000,
      });
      setActiveTab('progress');
    }
  };

  // ── Full-screen TryoutPlayer overlay ──────────────────────────────────────
  if (activeSession) {
    return (
      <div className="min-h-screen bg-slate-50">
        {/* Minimal header */}
        <div className="bg-white border-b sticky top-0 z-50 px-4 py-2.5 flex items-center justify-between shadow-sm">
          <div className="flex items-center gap-2.5">
            <div className="w-7 h-7 rounded-lg bg-gradient-to-br from-purple-600 to-pink-600 flex items-center justify-center">
              <Trophy className="w-4 h-4 text-white" />
            </div>
            <span className="text-sm font-bold text-slate-800">GASPOLPTN</span>
            <span className="text-slate-300 mx-1">›</span>
            <span className="text-sm text-slate-500 truncate max-w-xs">{activeSession.title}</span>
          </div>
          <button
            onClick={endSession}
            className="flex items-center gap-1.5 text-sm text-slate-500 hover:text-red-500 transition-colors px-3 py-1.5 rounded-lg hover:bg-red-50"
          >
            <X className="w-4 h-4" /> Keluar Sesi
          </button>
        </div>
        <TryoutPlayer
          sessionTitle={activeSession.title}
          sessionType={activeSession.type}
          questions={[]}
          durationMinutes={activeSession.durationMinutes}
          onFinish={(result) => endSession(result)}
        />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-purple-50">
      {/* Top Navigation */}
      <div className="bg-white/80 backdrop-blur-lg border-b sticky top-0 z-50 shadow-sm">
        <div className="max-w-[1800px] mx-auto px-4 sm:px-6 py-3.5">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-purple-600 to-pink-600 flex items-center justify-center">
                <Trophy className="w-5 h-5 text-white" />
              </div>
              <div>
                <h1 className="text-base font-bold leading-tight">GASPOLPTN</h1>
                <p className="text-xs text-muted-foreground leading-tight">Portal Siswa</p>
              </div>
            </div>

            <div className="flex items-center gap-2 sm:gap-3">
              {/* Voucher button */}
              <button
                onClick={() => setShowVoucher(true)}
                className="hidden sm:flex items-center gap-1.5 px-3 py-1.5 rounded-xl border-2 border-dashed border-indigo-300 bg-indigo-50 hover:bg-indigo-100 text-indigo-600 text-xs font-bold transition-colors"
              >
                <Tag className="w-3.5 h-3.5" />
                Voucher
              </button>

              {/* Points */}
              <Card className="hidden sm:flex items-center gap-2 px-3 py-1.5 border-2 border-purple-200 bg-gradient-to-r from-purple-50 to-pink-50 cursor-default">
                <div className="w-6 h-6 rounded-lg bg-gradient-to-br from-yellow-400 to-orange-500 flex items-center justify-center">
                  <Zap className="w-3.5 h-3.5 text-white" />
                </div>
                <div>
                  <div className="text-[10px] text-muted-foreground leading-tight">Poin Reward</div>
                  <div className="text-xs font-black text-slate-900">2.450 pts</div>
                </div>
              </Card>

              <NotificationBell role="student" />

              <div className="w-px h-7 bg-slate-200" />

              <div className="flex items-center gap-2">
                <div className="w-8 h-8 rounded-full bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center">
                  <span className="text-white text-xs font-bold">{initials}</span>
                </div>
                <div className="hidden sm:block">
                  <div className="text-sm font-semibold leading-tight">{name}</div>
                  <div className="text-xs text-muted-foreground leading-tight">{userData?.schoolName || 'Sekolah'}</div>
                </div>
              </div>

              <Button variant="ghost" size="icon" className="w-9 h-9" onClick={onLogout}>
                <LogOut className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      {/* Quick launch bar */}
      <div className="bg-white border-b">
        <div className="max-w-[1800px] mx-auto px-4 sm:px-6 py-2 flex items-center gap-2 overflow-x-auto">
          <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider shrink-0">Mulai Sekarang:</span>
          {TRYOUT_SESSIONS.map((s, i) => (
            <button
              key={i}
              onClick={() => startSession(s)}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-semibold border transition-colors whitespace-nowrap shrink-0 ${
                s.type === 'tryout'
                  ? 'bg-purple-50 border-purple-200 text-purple-700 hover:bg-purple-100'
                  : s.type === 'drilling'
                  ? 'bg-orange-50 border-orange-200 text-orange-700 hover:bg-orange-100'
                  : 'bg-blue-50 border-blue-200 text-blue-700 hover:bg-blue-100'
              }`}
            >
              <Play className="w-3 h-3" />
              {s.title.split('—')[0].trim()}
            </button>
          ))}
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-[1800px] mx-auto px-4 sm:px-6 py-6">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
          <TabsList className="inline-flex h-auto p-1 bg-white/80 backdrop-blur-sm border shadow-sm gap-0.5 flex-wrap">
            <TabsTrigger value="overview" className="gap-1.5 text-sm px-3 py-2">
              <LayoutDashboard className="w-4 h-4" />
              <span className="hidden sm:inline">Dashboard</span>
            </TabsTrigger>
            <TabsTrigger value="drilling" className="gap-1.5 text-sm px-3 py-2">
              <Zap className="w-4 h-4" />
              <span className="hidden sm:inline">Drilling</span>
            </TabsTrigger>
            <TabsTrigger value="rasionalisasi" className="gap-1.5 text-sm px-3 py-2">
              <Target className="w-4 h-4" />
              <span className="hidden sm:inline">Rasionalisasi PTN</span>
            </TabsTrigger>
            <TabsTrigger value="progress" className="gap-1.5 text-sm px-3 py-2">
              <TrendingUp className="w-4 h-4" />
              <span className="hidden sm:inline">Progress</span>
            </TabsTrigger>
            <TabsTrigger value="leaderboard" className="gap-1.5 text-sm px-3 py-2">
              <Crown className="w-4 h-4" />
              <span className="hidden sm:inline">Leaderboard</span>
            </TabsTrigger>
          </TabsList>

          <TabsContent value="overview">
            <StudentOverview userData={userData} onStartSession={startSession} onNavigate={setActiveTab} />
          </TabsContent>

          <TabsContent value="drilling">
            <DrillingZone onStartSession={startSession} />
          </TabsContent>

          <TabsContent value="rasionalisasi">
            <RasionalisasiSNBT />
          </TabsContent>

          <TabsContent value="progress">
            <StudentProgress />
          </TabsContent>

          <TabsContent value="leaderboard">
            <Leaderboard myName={name} onGoToDrilling={() => setActiveTab('drilling')} />
          </TabsContent>
        </Tabs>
      </div>

      {showVoucher && <VoucherActivation onClose={() => setShowVoucher(false)} />}
    </div>
  );
}

