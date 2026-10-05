import { useState } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import {
  Play, Zap, Trophy, Clock, Target, BookOpen, Brain, Flame, CheckCircle2,
  ChevronRight, Star, TrendingUp, Lock, Sparkles
} from 'lucide-react';


type DrillingMode = 'tryout' | 'daily' | 'topic' | 'speed' | null;

interface SessionConfig {
  title: string;
  type: 'tryout' | 'drilling' | 'mini';
  duration: number;
}

const availableTryouts = [
  { id: 'T1', name: 'Tryout Nasional SNBT #45', date: '25-27 Jan', participants: 4521, duration: 195, isPremium: false },
  { id: 'T2', name: 'Mini Tryout Weekend #12', date: '8-9 Feb', participants: 1240, duration: 45, isPremium: false },
  { id: 'T3', name: 'Tryout PTN Premium #46', date: '15-17 Feb', participants: 892, duration: 195, isPremium: true },
];

const topicPackages = [
  { id: 'TP1', name: 'Penalaran Umum — Level Sedang', questions: 40, duration: 45, subject: 'PU', difficulty: 'Sedang', done: false },
  { id: 'TP2', name: 'Penalaran Matematika — Aljabar', questions: 30, duration: 35, subject: 'PM', difficulty: 'Sulit', done: false },
  { id: 'TP3', name: 'Pemahaman Bacaan — Ide Pokok', questions: 25, duration: 30, subject: 'PPU', difficulty: 'Mudah', done: true },
  { id: 'TP4', name: 'Literasi Bahasa Indonesia — Ejaan', questions: 35, duration: 40, subject: 'PBM', difficulty: 'Sedang', done: false },
  { id: 'TP5', name: 'Penalaran Matematika — Statistika', questions: 30, duration: 35, subject: 'PM', difficulty: 'Sedang', done: false },
];

const subjects = [
  { id: 'PU', name: 'Penalaran Umum', fullName: 'Penalaran Umum', progress: 78, total: 245, icon: '🧠', color: 'from-violet-500 to-purple-600', weakTopics: ['Silogisme', 'Analogi Verbal'] },
  { id: 'PPU', name: 'Pem. Bacaan & Peng. Umum', fullName: 'Pemahaman Bacaan & Pengetahuan Umum', progress: 72, total: 189, icon: '📖', color: 'from-blue-500 to-cyan-600', weakTopics: ['Inferensi', 'Makna Implisit'] },
  { id: 'PBM', name: 'Literasi Bahasa Indonesia', fullName: 'Penalaran Bahasa Melayu-Indonesia', progress: 82, total: 156, icon: '🇮🇩', color: 'from-emerald-500 to-teal-600', weakTopics: ['Koherensi Paragraf'] },
  { id: 'PM', name: 'Penalaran Matematika', fullName: 'Penalaran Matematika', progress: 63, total: 312, icon: '🔢', color: 'from-orange-500 to-amber-600', weakTopics: ['Trigonometri', 'Peluang', 'Integral'] },
];

const recentSessions = [
  { id: 1, type: 'Tryout Nasional #45', score: 685, accuracy: 78, date: '2 jam lalu', duration: '120 min', correct: 78, wrong: 22, unanswered: 0 },
  { id: 2, type: 'Drilling PU', score: 42, accuracy: 82, date: 'Kemarin', duration: '45 min', correct: 33, wrong: 7, unanswered: 5 },
  { id: 3, type: 'Literasi Bahasa Indonesia', score: 38, accuracy: 84, date: '2 hari lalu', duration: '30 min', correct: 28, wrong: 5, unanswered: 2 },
];

interface DrillingZoneProps {
  onStartSession?: (s: { title: string; type: 'tryout' | 'drilling' | 'mini'; durationMinutes: number }) => void;
}

export default function DrillingZone({ onStartSession }: DrillingZoneProps) {
  const [showAllHistory, setShowAllHistory] = useState(false);

  const startSession = (config: SessionConfig) => {
    if (onStartSession) {
      onStartSession({ title: config.title, type: config.type, durationMinutes: config.duration });
    }
  };

  return (
    <div className="space-y-6">
      {/* Hero */}
      <Card className="p-6 bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 text-white relative overflow-hidden">
        <div className="absolute inset-0 opacity-10" style={{ backgroundImage: 'linear-gradient(rgba(255,255,255,.4) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,.4) 1px, transparent 1px)', backgroundSize: '40px 40px' }} />
        <div className="relative flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
          <div>
            <div className="flex items-center gap-2 mb-1"><Zap className="w-5 h-5 text-yellow-300" /><span className="font-bold">Drilling Zone</span></div>
            <h2 className="text-2xl font-black mb-1">Latihan Intensif & Tryout</h2>
            <p className="text-purple-100 text-sm">Sistem adaptif yang menyesuaikan dengan level kemampuanmu secara real-time.</p>
          </div>
          <div className="flex gap-3">
            {[{ val: '685', sub: 'Skor Terakhir' }, { val: '8', sub: 'Tryout Selesai' }, { val: '24🔥', sub: 'Day Streak' }].map(({ val, sub }) => (
              <div key={sub} className="bg-white/15 backdrop-blur-sm px-4 py-2.5 rounded-xl text-center">
                <div className="text-xl font-black">{val}</div>
                <div className="text-xs text-purple-200">{sub}</div>
              </div>
            ))}
          </div>
        </div>
      </Card>

      {/* Quick start: Tryout Tersedia */}
      <div>
        <h3 className="text-lg font-bold mb-3 flex items-center gap-2"><Trophy className="w-5 h-5 text-indigo-600" /> Tryout Tersedia</h3>
        <div className="space-y-3">
          {availableTryouts.map(tryout => (
            <Card key={tryout.id} className="p-4 hover:shadow-md transition-shadow">
              <div className="flex items-center gap-4">
                <div className={`w-11 h-11 rounded-xl flex items-center justify-center shrink-0 ${tryout.isPremium ? 'bg-amber-100' : 'bg-indigo-100'}`}>
                  {tryout.isPremium ? <Star className="w-5 h-5 text-amber-600" /> : <Trophy className="w-5 h-5 text-indigo-600" />}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-0.5">
                    <h4 className="font-bold text-sm text-slate-900 truncate">{tryout.name}</h4>
                    {tryout.isPremium && <Badge className="bg-amber-100 text-amber-700 text-[10px]">Premium</Badge>}
                  </div>
                  <div className="flex items-center gap-3 text-xs text-muted-foreground">
                    <span className="flex items-center gap-1"><Clock className="w-3 h-3" />{tryout.duration} menit</span>
                    <span className="flex items-center gap-1"><Target className="w-3 h-3" />{tryout.date}</span>
                    <span className="flex items-center gap-1"><Brain className="w-3 h-3" />{tryout.participants.toLocaleString()} peserta</span>
                  </div>
                </div>
                {tryout.isPremium ? (
                  <div className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-slate-100 text-slate-400 text-sm font-semibold">
                    <Lock className="w-3.5 h-3.5" /> Premium
                  </div>
                ) : (
                  <Button
                    onClick={() => startSession({ title: tryout.name, type: 'tryout', duration: tryout.duration })}
                    className="bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 gap-1.5 text-sm"
                  >
                    <Play className="w-4 h-4" /> Mulai
                  </Button>
                )}
              </div>
            </Card>
          ))}
        </div>
      </div>

      {/* Drilling modes */}
      <div>
        <h3 className="text-lg font-bold mb-3 flex items-center gap-2"><Zap className="w-5 h-5 text-orange-500" /> Mode Latihan</h3>
        <div className="grid md:grid-cols-3 gap-4">
          {[
            {
              icon: Zap, title: 'Latihan Harian', desc: 'Soal adaptif 15 menit berdasarkan kelemahanmu hari ini.',
              color: 'from-orange-600 to-red-600', bg: 'from-orange-50 to-red-50', border: 'border-orange-200',
              config: { title: 'Latihan Harian Adaptif', type: 'drilling' as const, duration: 15 }
            },
            {
              icon: Flame, title: 'Speed Challenge', desc: '20 soal dalam 10 menit. Asah kecepatan & akurasi.',
              color: 'from-red-600 to-pink-600', bg: 'from-red-50 to-pink-50', border: 'border-red-200',
              config: { title: 'Speed Challenge', type: 'drilling' as const, duration: 10 }
            },
            {
              icon: Sparkles, title: 'Mini Tryout', desc: '40 soal semua subtes dalam 45 menit.',
              color: 'from-violet-600 to-purple-600', bg: 'from-violet-50 to-purple-50', border: 'border-violet-200',
              config: { title: 'Mini Tryout SNBT', type: 'mini' as const, duration: 45 }
            },
          ].map(({ icon: Icon, title, desc, color, bg, border, config }) => (
            <Card key={title} className={`p-5 border-2 ${border} bg-gradient-to-br ${bg} hover:shadow-lg transition-all cursor-pointer group`}>
              <div className={`w-11 h-11 rounded-xl bg-gradient-to-br ${color} flex items-center justify-center mb-3 shadow-lg group-hover:scale-110 transition-transform`}>
                <Icon className="w-5 h-5 text-white" />
              </div>
              <h4 className="font-bold text-slate-900 mb-1">{title}</h4>
              <p className="text-xs text-muted-foreground mb-4">{desc}</p>
              <Button onClick={() => startSession(config)} className={`w-full bg-gradient-to-r ${color} hover:opacity-90 gap-2 text-sm`}>
                <Play className="w-4 h-4" /> Mulai
              </Button>
            </Card>
          ))}
        </div>
      </div>

      {/* Per-topic drilling */}
      <div>
        <h3 className="text-lg font-bold mb-3 flex items-center gap-2"><Target className="w-5 h-5 text-blue-600" /> Latihan Per Topik</h3>
        <div className="space-y-2.5">
          {topicPackages.map(pkg => (
            <Card key={pkg.id} className="p-4 hover:shadow-sm transition-shadow">
              <div className="flex items-center gap-4">
                <div className={`w-10 h-10 rounded-lg flex items-center justify-center text-xs font-black shrink-0 ${
                  pkg.subject === 'PU' ? 'bg-violet-100 text-violet-700' :
                  pkg.subject === 'PM' ? 'bg-orange-100 text-orange-700' :
                  pkg.subject === 'PPU' ? 'bg-blue-100 text-blue-700' :
                  'bg-emerald-100 text-emerald-700'
                }`}>{pkg.subject}</div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-0.5">
                    <p className="font-semibold text-sm text-slate-900 truncate">{pkg.name}</p>
                    {pkg.done && <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0" />}
                  </div>
                  <div className="flex items-center gap-3 text-xs text-muted-foreground">
                    <span>{pkg.questions} soal</span>
                    <span>{pkg.duration} menit</span>
                    <span className={`font-medium ${pkg.difficulty === 'Sulit' ? 'text-red-500' : pkg.difficulty === 'Sedang' ? 'text-amber-600' : 'text-green-600'}`}>{pkg.difficulty}</span>
                  </div>
                </div>
                <Button
                  variant={pkg.done ? 'outline' : 'default'}
                  size="sm"
                  onClick={() => startSession({ title: pkg.name, type: 'drilling', duration: pkg.duration })}
                  className={pkg.done ? 'gap-1.5' : 'bg-indigo-600 hover:bg-indigo-700 gap-1.5'}
                >
                  <Play className="w-3.5 h-3.5" /> {pkg.done ? 'Ulangi' : 'Mulai'}
                </Button>
              </div>
            </Card>
          ))}
        </div>
      </div>

      {/* Subject progress */}
      <Card className="p-6">
        <div className="flex items-center justify-between mb-5">
          <div>
            <h3 className="font-bold text-slate-900 mb-0.5">Progress Per Subtes</h3>
            <p className="text-xs text-muted-foreground">Klik subtes untuk latihan fokus</p>
          </div>
          <Button variant="outline" size="sm" className="gap-1.5 text-xs" onClick={() => startSession({ title: 'Latihan Harian Adaptif', type: 'drilling', duration: 20 })}>
            <TrendingUp className="w-3.5 h-3.5" /> Latihan Fokus Lemah
          </Button>
        </div>
        <div className="grid sm:grid-cols-2 gap-4">
          {subjects.map(sub => (
            <div
              key={sub.id}
              className="p-4 border-2 border-transparent hover:border-indigo-200 rounded-xl transition-all cursor-pointer group"
              onClick={() => startSession({ title: `Latihan ${sub.fullName}`, type: 'drilling', duration: 30 })}
            >
              <div className="flex items-center gap-3 mb-3">
                <div className="text-2xl">{sub.icon}</div>
                <div className="flex-1">
                  <p className="font-bold text-sm text-slate-900">{sub.name}</p>
                  <p className="text-xs text-muted-foreground">{sub.total} soal tersedia</p>
                </div>
                <ChevronRight className="w-4 h-4 text-slate-300 group-hover:text-indigo-500 transition-colors" />
              </div>
              <div className="mb-2">
                <div className="flex justify-between text-xs mb-1.5">
                  <span className="text-muted-foreground">Penguasaan</span>
                  <span className={`font-bold ${sub.progress >= 75 ? 'text-green-600' : sub.progress >= 60 ? 'text-amber-600' : 'text-red-500'}`}>{sub.progress}%</span>
                </div>
                <div className="h-2 bg-slate-100 rounded-full overflow-hidden">
                  <div
                    className={`h-full rounded-full bg-gradient-to-r ${sub.color}`}
                    style={{ width: `${sub.progress}%` }}
                  />
                </div>
              </div>
              {sub.weakTopics.length > 0 && (
                <div className="flex items-center gap-1 flex-wrap">
                  <span className="text-[10px] text-red-500 font-medium">Lemah:</span>
                  {sub.weakTopics.slice(0, 2).map(t => (
                    <span key={t} className="text-[10px] px-1.5 py-0.5 bg-red-50 text-red-500 rounded">{t}</span>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      </Card>

      {/* Recent sessions */}
      <Card className="p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-bold text-slate-900">Riwayat Latihan</h3>
          <Button variant="outline" size="sm" className="text-xs" onClick={() => setShowAllHistory(v => !v)}>
            {showAllHistory ? 'Sembunyikan' : 'Lihat Semua'}
          </Button>
        </div>
        <div className="space-y-3">
          {(showAllHistory ? recentSessions : recentSessions.slice(0, 3)).map(s => (
            <div key={s.id} className="flex items-center gap-4 p-3.5 bg-slate-50 rounded-xl hover:bg-slate-100 transition-colors">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-500 flex items-center justify-center">
                <Trophy className="w-5 h-5 text-white" />
              </div>
              <div className="flex-1 min-w-0">
                <p className="font-semibold text-sm text-slate-900 truncate">{s.type}</p>
                <div className="flex items-center gap-3 text-xs text-muted-foreground mt-0.5">
                  <span className="flex items-center gap-1"><Clock className="w-3 h-3" />{s.duration}</span>
                  <span>{s.date}</span>
                  <span className="text-emerald-600 font-medium">{s.correct} benar</span>
                  <span className="text-red-500 font-medium">{s.wrong} salah</span>
                </div>
              </div>
              <div className="text-right shrink-0">
                <p className="text-2xl font-black text-slate-900">{s.score}</p>
                <p className="text-xs text-muted-foreground">{s.accuracy}% akurasi</p>
              </div>
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
}
