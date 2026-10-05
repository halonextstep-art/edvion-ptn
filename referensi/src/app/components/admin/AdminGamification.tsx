import { useState } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { toast } from 'sonner';
import {
  Zap, Award, Trophy, Flame, Star, Crown, Target, TrendingUp,
  Plus, Edit3, Trash2, ToggleLeft, ToggleRight, X, Save,
  Users, BarChart3, Calendar, Clock, ChevronRight, Settings,
  Medal, Gift, Sparkles, Lock, Unlock, AlertCircle, CheckCircle2,
  ArrowUp, ArrowDown, Minus, Play, Pause, RotateCcw
} from 'lucide-react';

// ─── Types ────────────────────────────────────────────────────────────────────
type GamifView = 'poin' | 'badge' | 'leaderboard' | 'challenge';
type Rarity = 'common' | 'rare' | 'epic' | 'legendary';
type ChallengeStatus = 'upcoming' | 'active' | 'ended';
type ChallengeType = 'most_solved' | 'highest_score' | 'longest_streak' | 'speed';
type ConditionType = 'score_gte' | 'streak_gte' | 'questions_gte' | 'tryout_gte' | 'rank_lte';

interface PointRule {
  id: string;
  action: string;
  category: string;
  basePoints: number;
  multiplier: number;
  enabled: boolean;
  icon: string;
}

interface AppBadge {
  id: string;
  emoji: string;
  name: string;
  description: string;
  rarity: Rarity;
  conditionType: ConditionType;
  conditionValue: number;
  earnedBy: number;
  active: boolean;
}

interface LeaderboardEntry {
  rank: number;
  name: string;
  school: string;
  score: number;
  delta: number; // manual override delta
}

interface SubtesWeight {
  key: string;
  label: string;
  weight: number;
  color: string;
}

interface Challenge {
  id: string;
  name: string;
  type: ChallengeType;
  status: ChallengeStatus;
  startDate: string;
  endDate: string;
  targetValue: number;
  rewardPoints: number;
  rewardBadge?: string;
  participants: number;
  completions: number;
  description: string;
}

// ─── Seed Data ────────────────────────────────────────────────────────────────
const INITIAL_RULES: PointRule[] = [
  { id: 'r1', action: 'Jawab Benar — Pilihan Ganda', category: 'Soal', basePoints: 10, multiplier: 1, enabled: true, icon: '✅' },
  { id: 'r2', action: 'Jawab Benar — PG Kompleks', category: 'Soal', basePoints: 15, multiplier: 1, enabled: true, icon: '✅' },
  { id: 'r3', action: 'Jawab Benar — Benar/Salah', category: 'Soal', basePoints: 8, multiplier: 1, enabled: true, icon: '✅' },
  { id: 'r4', action: 'Jawab Benar — Isian Singkat', category: 'Soal', basePoints: 12, multiplier: 1, enabled: true, icon: '✅' },
  { id: 'r5', action: 'Jawab Benar — Uraian', category: 'Soal', basePoints: 20, multiplier: 1, enabled: true, icon: '✅' },
  { id: 'r6', action: 'Selesaikan Full Tryout', category: 'Sesi', basePoints: 100, multiplier: 1, enabled: true, icon: '🏁' },
  { id: 'r7', action: 'Selesaikan Mini Tryout', category: 'Sesi', basePoints: 50, multiplier: 1, enabled: true, icon: '🏁' },
  { id: 'r8', action: 'Selesaikan Drilling', category: 'Sesi', basePoints: 30, multiplier: 1, enabled: true, icon: '🏁' },
  { id: 'r9', action: 'Login Harian', category: 'Aktivitas', basePoints: 5, multiplier: 1, enabled: true, icon: '📅' },
  { id: 'r10', action: 'Streak +1 Hari', category: 'Streak', basePoints: 5, multiplier: 1, enabled: true, icon: '🔥' },
  { id: 'r11', action: 'Bonus Streak 7 Hari', category: 'Streak', basePoints: 100, multiplier: 1, enabled: true, icon: '🔥' },
  { id: 'r12', action: 'Bonus Streak 30 Hari', category: 'Streak', basePoints: 500, multiplier: 1, enabled: true, icon: '🔥' },
  { id: 'r13', action: 'Masuk Top 10 Nasional', category: 'Prestasi', basePoints: 200, multiplier: 1, enabled: true, icon: '👑' },
  { id: 'r14', action: 'Masuk Top 100 Nasional', category: 'Prestasi', basePoints: 100, multiplier: 1, enabled: true, icon: '🏅' },
  { id: 'r15', action: 'Skor Sempurna (100%)', category: 'Prestasi', basePoints: 300, multiplier: 1, enabled: true, icon: '⭐' },
];

const INITIAL_BADGES: AppBadge[] = [
  { id: 'b1', emoji: '🔥', name: '7 Day Streak', description: 'Latihan 7 hari berturut-turut tanpa jeda', rarity: 'common', conditionType: 'streak_gte', conditionValue: 7, earnedBy: 3421, active: true },
  { id: 'b2', emoji: '💪', name: '30 Day Warrior', description: 'Streak 30 hari — komitmen luar biasa!', rarity: 'rare', conditionType: 'streak_gte', conditionValue: 30, earnedBy: 892, active: true },
  { id: 'b3', emoji: '🎯', name: 'Sharpshooter', description: 'Akurasi 90%+ dalam satu sesi tryout', rarity: 'rare', conditionType: 'score_gte', conditionValue: 900, earnedBy: 234, active: true },
  { id: 'b4', emoji: '👑', name: 'Top 10 Nasional', description: 'Masuk 10 besar leaderboard nasional', rarity: 'epic', conditionType: 'rank_lte', conditionValue: 10, earnedBy: 10, active: true },
  { id: 'b5', emoji: '🏆', name: 'Marathon Runner', description: 'Selesaikan 20 tryout penuh', rarity: 'epic', conditionType: 'tryout_gte', conditionValue: 20, earnedBy: 156, active: true },
  { id: 'b6', emoji: '⭐', name: 'Perfect Score', description: 'Skor 100% di salah satu sesi', rarity: 'legendary', conditionType: 'score_gte', conditionValue: 1000, earnedBy: 12, active: true },
  { id: 'b7', emoji: '📚', name: 'Soal Addict', description: 'Menjawab 500 soal', rarity: 'common', conditionType: 'questions_gte', conditionValue: 500, earnedBy: 2150, active: true },
  { id: 'b8', emoji: '🚀', name: 'PTN Bound', description: 'Skor UTBK ≥ 700 dalam tryout simulasi', rarity: 'rare', conditionType: 'score_gte', conditionValue: 700, earnedBy: 678, active: true },
];

const INITIAL_WEIGHTS: SubtesWeight[] = [
  { key: 'PU', label: 'Penalaran Umum', weight: 25, color: 'bg-violet-500' },
  { key: 'PPU', label: 'Pemahaman Bacaan & Menulis', weight: 25, color: 'bg-blue-500' },
  { key: 'PBM', label: 'Pengetahuan & Pemahaman Umum', weight: 25, color: 'bg-emerald-500' },
  { key: 'PM', label: 'Penalaran Matematika', weight: 25, color: 'bg-orange-500' },
];

const INITIAL_LEADERBOARD: LeaderboardEntry[] = [
  { rank: 1, name: 'Budi Santoso', school: 'SMA Negeri 1 Surabaya', score: 842, delta: 0 },
  { rank: 2, name: 'Dewi Rahayu', school: 'SMA Negeri 3 Bandung', score: 831, delta: 0 },
  { rank: 3, name: 'Ahmad Fauzi', school: 'SMA Negeri 1 Jakarta', score: 828, delta: 0 },
  { rank: 4, name: 'Siti Nurhaliza', school: 'MAN 2 Yogyakarta', score: 819, delta: 0 },
  { rank: 5, name: 'Rizky Pratama', school: 'SMA Negeri 2 Medan', score: 811, delta: 0 },
  { rank: 6, name: 'Lina Marlina', school: 'SMA Negeri 5 Semarang', score: 805, delta: 0 },
  { rank: 7, name: 'Fajar Nugroho', school: 'SMA Negeri 1 Makassar', score: 798, delta: 0 },
  { rank: 8, name: 'Maya Sari', school: 'SMA Negeri 3 Jakarta', score: 792, delta: 0 },
];

const INITIAL_CHALLENGES: Challenge[] = [
  { id: 'c1', name: 'Tryout Marathon Januari', type: 'most_solved', status: 'active', startDate: '2025-01-01', endDate: '2025-01-31', targetValue: 10, rewardPoints: 500, rewardBadge: '🏅', participants: 4231, completions: 892, description: 'Selesaikan 10 tryout sepanjang bulan Januari dan raih hadiah eksklusif!' },
  { id: 'c2', name: 'Speed Challenge Minggu Ini', type: 'speed', status: 'active', startDate: '2025-01-20', endDate: '2025-01-26', targetValue: 30, rewardPoints: 200, rewardBadge: '⚡', participants: 1832, completions: 234, description: 'Selesaikan 30 soal dalam waktu paling singkat minggu ini.' },
  { id: 'c3', name: 'Streak King February', type: 'longest_streak', status: 'upcoming', startDate: '2025-02-01', endDate: '2025-02-28', targetValue: 28, rewardPoints: 1000, rewardBadge: '👑', participants: 0, completions: 0, description: 'Pertahankan streak 28 hari penuh sepanjang Februari.' },
  { id: 'c4', name: 'Skor Tertinggi Desember', type: 'highest_score', status: 'ended', startDate: '2024-12-01', endDate: '2024-12-31', targetValue: 800, rewardPoints: 300, rewardBadge: '🎯', participants: 5120, completions: 1240, description: 'Raih skor tryout tertinggi sepanjang Desember.' },
];

// ─── Helpers ──────────────────────────────────────────────────────────────────
const RARITY_CONFIG: Record<Rarity, { label: string; color: string; ring: string; text: string }> = {
  common:    { label: 'Common',    color: 'bg-slate-100',    ring: 'ring-slate-300',   text: 'text-slate-600' },
  rare:      { label: 'Rare',      color: 'bg-blue-100',     ring: 'ring-blue-400',    text: 'text-blue-700' },
  epic:      { label: 'Epic',      color: 'bg-purple-100',   ring: 'ring-purple-500',  text: 'text-purple-700' },
  legendary: { label: 'Legendary', color: 'bg-amber-100',    ring: 'ring-amber-400',   text: 'text-amber-700' },
};

const CONDITION_LABELS: Record<ConditionType, string> = {
  score_gte:     'Skor ≥',
  streak_gte:    'Streak ≥ hari',
  questions_gte: 'Soal dijawab ≥',
  tryout_gte:    'Tryout selesai ≥',
  rank_lte:      'Ranking ≤',
};

const CHALLENGE_TYPE_CONFIG: Record<ChallengeType, { label: string; icon: string; color: string }> = {
  most_solved:    { label: 'Terbanyak Soal',    icon: '📚', color: 'bg-blue-100 text-blue-700' },
  highest_score:  { label: 'Skor Tertinggi',    icon: '🏆', color: 'bg-amber-100 text-amber-700' },
  longest_streak: { label: 'Streak Terpanjang', icon: '🔥', color: 'bg-orange-100 text-orange-700' },
  speed:          { label: 'Kecepatan',         icon: '⚡', color: 'bg-purple-100 text-purple-700' },
};

const STATUS_CONFIG: Record<ChallengeStatus, { label: string; color: string; dot: string }> = {
  upcoming: { label: 'Akan Datang', color: 'bg-blue-100 text-blue-700',    dot: 'bg-blue-500' },
  active:   { label: 'Aktif',       color: 'bg-emerald-100 text-emerald-700', dot: 'bg-emerald-500' },
  ended:    { label: 'Berakhir',    color: 'bg-slate-100 text-slate-500',   dot: 'bg-slate-400' },
};

const CATEGORY_COLORS: Record<string, string> = {
  Soal:      'bg-blue-50 text-blue-700 border-blue-200',
  Sesi:      'bg-emerald-50 text-emerald-700 border-emerald-200',
  Aktivitas: 'bg-purple-50 text-purple-700 border-purple-200',
  Streak:    'bg-orange-50 text-orange-700 border-orange-200',
  Prestasi:  'bg-amber-50 text-amber-700 border-amber-200',
};

const BADGE_EMOJIS = ['🔥','💪','🎯','👑','🏆','⭐','📚','🚀','⚡','🧠','🎖️','💎','🌟','🦁','🦅','🎪','🏅','🌈','🔮','🎊'];

// ─── Sub-components ──────────────────────────────────────────────────────────

function SectionHeader({ title, subtitle, action }: { title: string; subtitle: string; action?: React.ReactNode }) {
  return (
    <div className="flex items-start justify-between mb-6">
      <div>
        <h2 className="text-xl font-bold text-slate-900">{title}</h2>
        <p className="text-sm text-muted-foreground mt-0.5">{subtitle}</p>
      </div>
      {action}
    </div>
  );
}

// ─── Point Rules Section ─────────────────────────────────────────────────────
function PointRules() {
  const [rules, setRules] = useState<PointRule[]>(INITIAL_RULES);
  const [editingRule, setEditingRule] = useState<PointRule | null>(null);
  const [form, setForm] = useState({ basePoints: 0, multiplier: 1, enabled: true });

  const totalPointsPerSession = rules.filter(r => r.enabled && r.category === 'Sesi').reduce((s, r) => s + r.basePoints * r.multiplier, 0);
  const enabledCount = rules.filter(r => r.enabled).length;

  const openEdit = (rule: PointRule) => {
    setEditingRule(rule);
    setForm({ basePoints: rule.basePoints, multiplier: rule.multiplier, enabled: rule.enabled });
  };

  const saveEdit = () => {
    if (!editingRule) return;
    setRules(prev => prev.map(r => r.id === editingRule.id ? { ...r, ...form } : r));
    toast.success(`Aturan "${editingRule.action}" berhasil diperbarui`);
    setEditingRule(null);
  };

  const toggleRule = (id: string) => {
    setRules(prev => prev.map(r => r.id === id ? { ...r, enabled: !r.enabled } : r));
  };

  const categories = [...new Set(rules.map(r => r.category))];

  return (
    <div>
      <SectionHeader
        title="Konfigurasi Poin"
        subtitle="Atur berapa poin yang diperoleh siswa untuk setiap aksi di dalam platform"
        action={
          <div className="flex items-center gap-3">
            <Card className="px-4 py-2 text-center border-2 border-emerald-200 bg-emerald-50">
              <div className="text-lg font-black text-emerald-700">{enabledCount}<span className="text-sm font-normal">/{rules.length}</span></div>
              <div className="text-xs text-emerald-600">Aturan Aktif</div>
            </Card>
            <Card className="px-4 py-2 text-center border-2 border-blue-200 bg-blue-50">
              <div className="text-lg font-black text-blue-700">+{totalPointsPerSession}</div>
              <div className="text-xs text-blue-600">Maks/Sesi</div>
            </Card>
          </div>
        }
      />

      <div className="space-y-6">
        {categories.map(cat => (
          <div key={cat}>
            <div className={`inline-flex items-center gap-1.5 text-xs font-bold px-3 py-1 rounded-full border mb-3 ${CATEGORY_COLORS[cat]}`}>
              {cat}
            </div>
            <Card className="overflow-hidden">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b bg-slate-50 text-xs text-muted-foreground">
                    <th className="text-left px-4 py-3 font-semibold">Aksi</th>
                    <th className="text-center px-4 py-3 font-semibold">Poin Dasar</th>
                    <th className="text-center px-4 py-3 font-semibold hidden md:table-cell">Multiplier</th>
                    <th className="text-center px-4 py-3 font-semibold hidden md:table-cell">Total/Aksi</th>
                    <th className="text-center px-4 py-3 font-semibold">Status</th>
                    <th className="px-4 py-3"></th>
                  </tr>
                </thead>
                <tbody className="divide-y">
                  {rules.filter(r => r.category === cat).map(rule => (
                    <tr key={rule.id} className={`transition-colors ${rule.enabled ? 'hover:bg-slate-50' : 'opacity-40 bg-slate-50'}`}>
                      <td className="px-4 py-3">
                        <div className="flex items-center gap-2">
                          <span className="text-lg">{rule.icon}</span>
                          <span className={`font-medium ${rule.enabled ? 'text-slate-900' : 'text-slate-400'}`}>{rule.action}</span>
                        </div>
                      </td>
                      <td className="px-4 py-3 text-center">
                        <span className="font-black text-base text-indigo-600">+{rule.basePoints}</span>
                      </td>
                      <td className="px-4 py-3 text-center hidden md:table-cell">
                        <span className="text-slate-500">×{rule.multiplier}</span>
                      </td>
                      <td className="px-4 py-3 text-center hidden md:table-cell">
                        <span className="font-bold text-emerald-600">+{rule.basePoints * rule.multiplier}</span>
                      </td>
                      <td className="px-4 py-3 text-center">
                        <button onClick={() => toggleRule(rule.id)} className="transition-colors">
                          {rule.enabled
                            ? <ToggleRight className="w-7 h-7 text-emerald-500 mx-auto" />
                            : <ToggleLeft className="w-7 h-7 text-slate-300 mx-auto" />}
                        </button>
                      </td>
                      <td className="px-4 py-3 text-right">
                        <Button variant="ghost" size="sm" onClick={() => openEdit(rule)} className="h-8">
                          <Edit3 className="w-3.5 h-3.5" />
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </Card>
          </div>
        ))}
      </div>

      {/* Edit Modal */}
      {editingRule && (
        <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4">
          <Card className="w-full max-w-md p-6 shadow-2xl">
            <div className="flex items-center justify-between mb-5">
              <div>
                <h3 className="font-bold text-slate-900">Edit Aturan Poin</h3>
                <p className="text-sm text-muted-foreground mt-0.5">{editingRule.icon} {editingRule.action}</p>
              </div>
              <button onClick={() => setEditingRule(null)} className="p-1.5 rounded-lg hover:bg-slate-100"><X className="w-4 h-4" /></button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Poin Dasar</label>
                <input type="number" min={0} value={form.basePoints} onChange={e => setForm(f => ({ ...f, basePoints: +e.target.value }))} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Multiplier</label>
                <input type="number" min={0.1} step={0.1} value={form.multiplier} onChange={e => setForm(f => ({ ...f, multiplier: +e.target.value }))} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                <p className="text-xs text-muted-foreground mt-1">Total per aksi: <strong className="text-emerald-600">+{Math.round(form.basePoints * form.multiplier)}</strong></p>
              </div>
              <div className="flex items-center gap-3 p-3 bg-slate-50 rounded-lg">
                <button onClick={() => setForm(f => ({ ...f, enabled: !f.enabled }))}>
                  {form.enabled ? <ToggleRight className="w-7 h-7 text-emerald-500" /> : <ToggleLeft className="w-7 h-7 text-slate-300" />}
                </button>
                <div>
                  <p className="text-sm font-medium">{form.enabled ? 'Aktif' : 'Nonaktif'}</p>
                  <p className="text-xs text-muted-foreground">Aturan {form.enabled ? 'akan memberikan poin' : 'tidak akan memberikan poin'}</p>
                </div>
              </div>
            </div>
            <div className="flex gap-2 mt-6">
              <Button variant="outline" className="flex-1" onClick={() => setEditingRule(null)}>Batal</Button>
              <Button className="flex-1 bg-indigo-600 hover:bg-indigo-700" onClick={saveEdit}><Save className="w-4 h-4 mr-1.5" />Simpan</Button>
            </div>
          </Card>
        </div>
      )}
    </div>
  );
}

// ─── Badge Management Section ─────────────────────────────────────────────────
function BadgeManagement() {
  const [badges, setBadges] = useState<AppBadge[]>(INITIAL_BADGES);
  const [modal, setModal] = useState<'create' | 'edit' | null>(null);
  const [editTarget, setEditTarget] = useState<AppBadge | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [form, setForm] = useState<Omit<AppBadge, 'id' | 'earnedBy'>>({
    emoji: '🏆', name: '', description: '', rarity: 'common',
    conditionType: 'score_gte', conditionValue: 0, active: true,
  });

  const openCreate = () => {
    setForm({ emoji: '🏆', name: '', description: '', rarity: 'common', conditionType: 'score_gte', conditionValue: 0, active: true });
    setModal('create');
  };

  const openEdit = (b: AppBadge) => {
    setEditTarget(b);
    setForm({ emoji: b.emoji, name: b.name, description: b.description, rarity: b.rarity, conditionType: b.conditionType, conditionValue: b.conditionValue, active: b.active });
    setModal('edit');
  };

  const saveBadge = () => {
    if (!form.name.trim()) { toast.error('Nama badge wajib diisi'); return; }
    if (modal === 'create') {
      const newBadge: AppBadge = { ...form, id: `b${Date.now()}`, earnedBy: 0 };
      setBadges(prev => [newBadge, ...prev]);
      toast.success(`Badge "${form.name}" berhasil dibuat`);
    } else if (editTarget) {
      setBadges(prev => prev.map(b => b.id === editTarget.id ? { ...b, ...form } : b));
      toast.success(`Badge "${form.name}" berhasil diperbarui`);
    }
    setModal(null);
  };

  const toggleBadge = (id: string) => {
    setBadges(prev => prev.map(b => b.id === id ? { ...b, active: !b.active } : b));
  };

  const deleteBadge = (id: string) => {
    const b = badges.find(b => b.id === id);
    setBadges(prev => prev.filter(b => b.id !== id));
    toast.success(`Badge "${b?.name}" dihapus`);
    setDeletingId(null);
  };

  const stats = {
    total: badges.length,
    active: badges.filter(b => b.active).length,
    totalEarned: badges.reduce((s, b) => s + b.earnedBy, 0),
    legendary: badges.filter(b => b.rarity === 'legendary').length,
  };

  return (
    <div>
      <SectionHeader
        title="Manajemen Badge & Achievement"
        subtitle="Buat dan kelola badge yang dapat diraih siswa berdasarkan pencapaian tertentu"
        action={<Button onClick={openCreate} className="bg-indigo-600 hover:bg-indigo-700 gap-2"><Plus className="w-4 h-4" />Buat Badge</Button>}
      />

      {/* Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        {[
          { label: 'Total Badge', val: stats.total, icon: Award, color: 'text-indigo-600', bg: 'bg-indigo-50' },
          { label: 'Badge Aktif', val: stats.active, icon: CheckCircle2, color: 'text-emerald-600', bg: 'bg-emerald-50' },
          { label: 'Total Diraih', val: stats.totalEarned.toLocaleString(), icon: Users, color: 'text-blue-600', bg: 'bg-blue-50' },
          { label: 'Legendary', val: stats.legendary, icon: Crown, color: 'text-amber-600', bg: 'bg-amber-50' },
        ].map(({ label, val, icon: Icon, color, bg }) => (
          <Card key={label} className="p-4 flex items-center gap-3">
            <div className={`w-10 h-10 rounded-xl ${bg} flex items-center justify-center shrink-0`}>
              <Icon className={`w-5 h-5 ${color}`} />
            </div>
            <div>
              <div className={`text-xl font-black ${color}`}>{val}</div>
              <div className="text-xs text-muted-foreground">{label}</div>
            </div>
          </Card>
        ))}
      </div>

      {/* Badge Grid */}
      <div className="grid sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {badges.map(b => {
          const rc = RARITY_CONFIG[b.rarity];
          return (
            <Card key={b.id} className={`p-4 relative transition-all hover:shadow-md ${!b.active ? 'opacity-50' : ''} ring-2 ${rc.ring}`}>
              <div className="flex items-start justify-between mb-3">
                <div className={`w-12 h-12 rounded-2xl ${rc.color} flex items-center justify-center text-2xl`}>{b.emoji}</div>
                <div className="flex items-center gap-1">
                  <button onClick={() => toggleBadge(b.id)} className="p-1 rounded hover:bg-slate-100 transition-colors">
                    {b.active ? <ToggleRight className="w-5 h-5 text-emerald-500" /> : <ToggleLeft className="w-5 h-5 text-slate-300" />}
                  </button>
                  <button onClick={() => openEdit(b)} className="p-1 rounded hover:bg-slate-100 transition-colors">
                    <Edit3 className="w-4 h-4 text-slate-400" />
                  </button>
                  <button onClick={() => setDeletingId(b.id)} className="p-1 rounded hover:bg-red-50 transition-colors">
                    <Trash2 className="w-4 h-4 text-slate-400 hover:text-red-500" />
                  </button>
                </div>
              </div>
              <h4 className="font-bold text-slate-900 text-sm mb-0.5">{b.name}</h4>
              <p className="text-xs text-muted-foreground mb-3 leading-relaxed">{b.description}</p>
              <div className="flex items-center justify-between">
                <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${rc.color} ${rc.text}`}>{rc.label}</span>
                <span className="text-xs text-muted-foreground flex items-center gap-1">
                  <Users className="w-3 h-3" />{b.earnedBy.toLocaleString()}
                </span>
              </div>
              <div className="mt-2 pt-2 border-t text-xs text-slate-500">
                {CONDITION_LABELS[b.conditionType]} <strong>{b.conditionValue}</strong>
              </div>
            </Card>
          );
        })}
      </div>

      {/* Create/Edit Modal */}
      {modal && (
        <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4">
          <Card className="w-full max-w-lg p-6 shadow-2xl max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-5">
              <h3 className="font-bold text-slate-900">{modal === 'create' ? 'Buat Badge Baru' : 'Edit Badge'}</h3>
              <button onClick={() => setModal(null)} className="p-1.5 rounded-lg hover:bg-slate-100"><X className="w-4 h-4" /></button>
            </div>
            <div className="space-y-4">
              {/* Emoji picker */}
              <div>
                <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-2">Icon Badge</label>
                <div className="flex flex-wrap gap-2 p-3 bg-slate-50 rounded-xl">
                  {BADGE_EMOJIS.map(e => (
                    <button key={e} onClick={() => setForm(f => ({ ...f, emoji: e }))}
                      className={`w-9 h-9 rounded-lg text-xl flex items-center justify-center transition-colors ${form.emoji === e ? 'bg-indigo-100 ring-2 ring-indigo-400' : 'hover:bg-slate-200'}`}>
                      {e}
                    </button>
                  ))}
                </div>
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Badge</label>
                <input value={form.name} onChange={e => setForm(f => ({ ...f, name: e.target.value }))} placeholder="contoh: 7 Day Streak" className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Deskripsi</label>
                <textarea rows={2} value={form.description} onChange={e => setForm(f => ({ ...f, description: e.target.value }))} placeholder="Jelaskan cara mendapatkan badge ini..." className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 resize-none" />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Rarity</label>
                  <select value={form.rarity} onChange={e => setForm(f => ({ ...f, rarity: e.target.value as Rarity }))} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400">
                    <option value="common">Common</option>
                    <option value="rare">Rare</option>
                    <option value="epic">Epic</option>
                    <option value="legendary">Legendary</option>
                  </select>
                </div>
                <div>
                  <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Tipe Kondisi</label>
                  <select value={form.conditionType} onChange={e => setForm(f => ({ ...f, conditionType: e.target.value as ConditionType }))} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400">
                    <option value="score_gte">Skor ≥</option>
                    <option value="streak_gte">Streak ≥ hari</option>
                    <option value="questions_gte">Soal dijawab ≥</option>
                    <option value="tryout_gte">Tryout selesai ≥</option>
                    <option value="rank_lte">Ranking ≤</option>
                  </select>
                </div>
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Nilai Kondisi</label>
                <input type="number" min={0} value={form.conditionValue} onChange={e => setForm(f => ({ ...f, conditionValue: +e.target.value }))} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                <p className="text-xs text-muted-foreground mt-1">Kondisi: {CONDITION_LABELS[form.conditionType]} <strong>{form.conditionValue}</strong></p>
              </div>
              {/* Preview */}
              <div className={`p-4 rounded-xl ring-2 ${RARITY_CONFIG[form.rarity].ring} ${RARITY_CONFIG[form.rarity].color}`}>
                <div className="flex items-center gap-3">
                  <span className="text-3xl">{form.emoji}</span>
                  <div>
                    <div className="font-bold text-slate-900">{form.name || '(nama badge)'}</div>
                    <div className="text-xs text-muted-foreground">{form.description || '(deskripsi)'}</div>
                    <span className={`text-[10px] font-bold ${RARITY_CONFIG[form.rarity].text}`}>{RARITY_CONFIG[form.rarity].label}</span>
                  </div>
                </div>
              </div>
            </div>
            <div className="flex gap-2 mt-6">
              <Button variant="outline" className="flex-1" onClick={() => setModal(null)}>Batal</Button>
              <Button className="flex-1 bg-indigo-600 hover:bg-indigo-700" onClick={saveBadge}><Save className="w-4 h-4 mr-1.5" />Simpan Badge</Button>
            </div>
          </Card>
        </div>
      )}

      {/* Delete Confirm */}
      {deletingId && (
        <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4">
          <Card className="w-full max-w-sm p-6 shadow-2xl">
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 rounded-full bg-red-100 flex items-center justify-center"><Trash2 className="w-5 h-5 text-red-600" /></div>
              <div><h3 className="font-bold">Hapus Badge?</h3><p className="text-sm text-muted-foreground">Tindakan ini tidak dapat dibatalkan</p></div>
            </div>
            <div className="flex gap-2">
              <Button variant="outline" className="flex-1" onClick={() => setDeletingId(null)}>Batal</Button>
              <Button className="flex-1 bg-red-600 hover:bg-red-700" onClick={() => deleteBadge(deletingId)}>Hapus</Button>
            </div>
          </Card>
        </div>
      )}
    </div>
  );
}

// ─── Leaderboard Control Section ──────────────────────────────────────────────
function LeaderboardControl() {
  const [weights, setWeights] = useState<SubtesWeight[]>(INITIAL_WEIGHTS);
  const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>(INITIAL_LEADERBOARD);
  const [resetPeriod, setResetPeriod] = useState<'daily' | 'weekly' | 'monthly' | 'never'>('weekly');
  const [resetTime, setResetTime] = useState('00:00');
  const [overrideId, setOverrideId] = useState<number | null>(null);
  const [overrideDelta, setOverrideDelta] = useState(0);
  const [saved, setSaved] = useState(false);

  const totalWeight = weights.reduce((s, w) => s + w.weight, 0);

  const adjustWeight = (key: string, delta: number) => {
    setWeights(prev => prev.map(w => w.key === key ? { ...w, weight: Math.max(0, Math.min(100, w.weight + delta)) } : w));
  };

  const applyOverride = (rank: number) => {
    setLeaderboard(prev => prev.map(e => e.rank === rank ? { ...e, delta: overrideDelta } : e));
    toast.success(`Override skor +${overrideDelta > 0 ? '+' : ''}${overrideDelta} diterapkan ke ${leaderboard.find(e => e.rank === rank)?.name}`);
    setOverrideId(null);
    setOverrideDelta(0);
  };

  const saveSettings = () => {
    setSaved(true);
    toast.success('Pengaturan leaderboard berhasil disimpan');
    setTimeout(() => setSaved(false), 2000);
  };

  const PERIOD_LABELS = { daily: 'Setiap Hari', weekly: 'Setiap Minggu', monthly: 'Setiap Bulan', never: 'Tidak Reset' };

  return (
    <div>
      <SectionHeader
        title="Kontrol Leaderboard"
        subtitle="Atur periode reset, bobot subtes, dan lakukan manual override untuk ranking siswa"
        action={
          <Button onClick={saveSettings} className={`gap-2 ${saved ? 'bg-emerald-600 hover:bg-emerald-600' : 'bg-indigo-600 hover:bg-indigo-700'}`}>
            {saved ? <CheckCircle2 className="w-4 h-4" /> : <Save className="w-4 h-4" />}
            {saved ? 'Tersimpan!' : 'Simpan Pengaturan'}
          </Button>
        }
      />

      <div className="grid lg:grid-cols-2 gap-6">
        {/* Left column */}
        <div className="space-y-5">
          {/* Reset settings */}
          <Card className="p-5">
            <div className="flex items-center gap-2 mb-4">
              <RotateCcw className="w-4 h-4 text-indigo-500" />
              <h3 className="font-bold text-slate-900">Periode Reset</h3>
            </div>
            <div className="grid grid-cols-2 gap-2 mb-4">
              {(['daily', 'weekly', 'monthly', 'never'] as const).map(p => (
                <button key={p} onClick={() => setResetPeriod(p)}
                  className={`px-3 py-2.5 rounded-xl text-sm font-semibold border-2 transition-colors ${resetPeriod === p ? 'border-indigo-500 bg-indigo-50 text-indigo-700' : 'border-slate-200 text-slate-500 hover:border-slate-300'}`}>
                  {PERIOD_LABELS[p]}
                </button>
              ))}
            </div>
            {resetPeriod !== 'never' && (
              <div>
                <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Waktu Reset</label>
                <input type="time" value={resetTime} onChange={e => setResetTime(e.target.value)} className="border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
              </div>
            )}
            <div className="mt-3 p-3 bg-amber-50 border border-amber-200 rounded-lg flex items-start gap-2">
              <AlertCircle className="w-4 h-4 text-amber-500 mt-0.5 shrink-0" />
              <p className="text-xs text-amber-700">Reset leaderboard akan menghapus semua ranking periode lama dan memulai periode baru. Histori tetap tersimpan di Analytics.</p>
            </div>
          </Card>

          {/* Score weights */}
          <Card className="p-5">
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-2">
                <BarChart3 className="w-4 h-4 text-indigo-500" />
                <h3 className="font-bold text-slate-900">Bobot Skor per Subtes</h3>
              </div>
              <span className={`text-sm font-bold ${totalWeight === 100 ? 'text-emerald-600' : 'text-red-500'}`}>Total: {totalWeight}%</span>
            </div>
            {totalWeight !== 100 && (
              <div className="mb-3 p-2.5 bg-red-50 border border-red-200 rounded-lg flex items-center gap-2">
                <AlertCircle className="w-4 h-4 text-red-500 shrink-0" />
                <p className="text-xs text-red-600">Total bobot harus 100%. Saat ini {totalWeight}%.</p>
              </div>
            )}
            <div className="space-y-4">
              {weights.map(w => (
                <div key={w.key}>
                  <div className="flex items-center justify-between mb-1.5">
                    <span className="text-sm font-medium text-slate-700">{w.key} — {w.label}</span>
                    <div className="flex items-center gap-2">
                      <button onClick={() => adjustWeight(w.key, -5)} className="w-7 h-7 rounded-lg border hover:bg-slate-100 flex items-center justify-center text-slate-500"><Minus className="w-3 h-3" /></button>
                      <span className="w-12 text-center text-sm font-black text-slate-900">{w.weight}%</span>
                      <button onClick={() => adjustWeight(w.key, 5)} className="w-7 h-7 rounded-lg border hover:bg-slate-100 flex items-center justify-center text-slate-500"><Plus className="w-3 h-3" /></button>
                    </div>
                  </div>
                  <div className="h-2 bg-slate-100 rounded-full overflow-hidden">
                    <div className={`h-full ${w.color} rounded-full transition-all`} style={{ width: `${w.weight}%` }} />
                  </div>
                </div>
              ))}
            </div>
          </Card>
        </div>

        {/* Right column — Leaderboard preview + override */}
        <Card className="p-5">
          <div className="flex items-center gap-2 mb-4">
            <Crown className="w-4 h-4 text-amber-500" />
            <h3 className="font-bold text-slate-900">Preview Leaderboard Nasional</h3>
          </div>
          <div className="space-y-2">
            {leaderboard.map(entry => (
              <div key={entry.rank} className={`flex items-center gap-3 p-3 rounded-xl transition-colors ${entry.rank <= 3 ? 'bg-amber-50 border border-amber-200' : 'bg-slate-50'}`}>
                <div className={`w-8 h-8 rounded-lg flex items-center justify-center text-sm font-black shrink-0 ${entry.rank === 1 ? 'bg-amber-400 text-white' : entry.rank === 2 ? 'bg-slate-300 text-slate-700' : entry.rank === 3 ? 'bg-orange-300 text-white' : 'bg-white border text-slate-500'}`}>
                  {entry.rank <= 3 ? ['🥇','🥈','🥉'][entry.rank - 1] : `#${entry.rank}`}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="font-semibold text-sm text-slate-900 truncate">{entry.name}</p>
                  <p className="text-xs text-muted-foreground truncate">{entry.school}</p>
                </div>
                <div className="text-right">
                  <p className="font-black text-sm text-slate-900">{entry.score + entry.delta}</p>
                  {entry.delta !== 0 && <p className={`text-xs font-bold ${entry.delta > 0 ? 'text-emerald-500' : 'text-red-500'}`}>{entry.delta > 0 ? `+${entry.delta}` : entry.delta}</p>}
                </div>
                {overrideId === entry.rank ? (
                  <div className="flex items-center gap-1.5 shrink-0">
                    <input type="number" value={overrideDelta} onChange={e => setOverrideDelta(+e.target.value)} placeholder="±0" className="w-16 border rounded-lg px-2 py-1 text-xs focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                    <button onClick={() => applyOverride(entry.rank)} className="px-2 py-1 text-xs bg-indigo-600 text-white rounded-lg hover:bg-indigo-700">OK</button>
                    <button onClick={() => setOverrideId(null)} className="p-1 rounded hover:bg-slate-200"><X className="w-3 h-3" /></button>
                  </div>
                ) : (
                  <button onClick={() => { setOverrideId(entry.rank); setOverrideDelta(0); }} className="shrink-0 px-2 py-1 text-xs border rounded-lg text-slate-500 hover:bg-slate-100 transition-colors">Override</button>
                )}
              </div>
            ))}
          </div>
          <p className="text-xs text-muted-foreground mt-3 text-center">Override hanya mempengaruhi tampilan. Skor aktual siswa tidak berubah.</p>
        </Card>
      </div>
    </div>
  );
}

// ─── Challenge & Event Section ────────────────────────────────────────────────
function ChallengeEvent() {
  const [challenges, setChallenges] = useState<Challenge[]>(INITIAL_CHALLENGES);
  const [modal, setModal] = useState<'create' | 'edit' | null>(null);
  const [editTarget, setEditTarget] = useState<Challenge | null>(null);
  const [filterStatus, setFilterStatus] = useState<ChallengeStatus | 'all'>('all');
  const [form, setForm] = useState<Omit<Challenge, 'id' | 'participants' | 'completions'>>({
    name: '', type: 'most_solved', status: 'upcoming',
    startDate: '', endDate: '', targetValue: 0,
    rewardPoints: 100, rewardBadge: '🏅', description: '',
  });

  const openCreate = () => {
    const today = new Date().toISOString().split('T')[0];
    setForm({ name: '', type: 'most_solved', status: 'upcoming', startDate: today, endDate: today, targetValue: 10, rewardPoints: 100, rewardBadge: '🏅', description: '' });
    setModal('create');
  };

  const openEdit = (c: Challenge) => {
    setEditTarget(c);
    setForm({ name: c.name, type: c.type, status: c.status, startDate: c.startDate, endDate: c.endDate, targetValue: c.targetValue, rewardPoints: c.rewardPoints, rewardBadge: c.rewardBadge, description: c.description });
    setModal('edit');
  };

  const saveChallenge = () => {
    if (!form.name.trim()) { toast.error('Nama challenge wajib diisi'); return; }
    if (modal === 'create') {
      const newC: Challenge = { ...form, id: `c${Date.now()}`, participants: 0, completions: 0 };
      setChallenges(prev => [newC, ...prev]);
      toast.success(`Challenge "${form.name}" berhasil dibuat`);
    } else if (editTarget) {
      setChallenges(prev => prev.map(c => c.id === editTarget.id ? { ...c, ...form } : c));
      toast.success(`Challenge "${form.name}" berhasil diperbarui`);
    }
    setModal(null);
  };

  const toggleStatus = (id: string) => {
    setChallenges(prev => prev.map(c => {
      if (c.id !== id) return c;
      const next: ChallengeStatus = c.status === 'active' ? 'ended' : c.status === 'upcoming' ? 'active' : 'ended';
      toast.success(`Challenge "${c.name}" → ${STATUS_CONFIG[next].label}`);
      return { ...c, status: next };
    }));
  };

  const deleteChallenge = (id: string) => {
    const c = challenges.find(c => c.id === id);
    setChallenges(prev => prev.filter(c => c.id !== id));
    toast.success(`Challenge "${c?.name}" dihapus`);
  };

  const filtered = filterStatus === 'all' ? challenges : challenges.filter(c => c.status === filterStatus);
  const stats = {
    active: challenges.filter(c => c.status === 'active').length,
    upcoming: challenges.filter(c => c.status === 'upcoming').length,
    totalParticipants: challenges.reduce((s, c) => s + c.participants, 0),
    totalCompletions: challenges.reduce((s, c) => s + c.completions, 0),
  };

  return (
    <div>
      <SectionHeader
        title="Challenge & Event"
        subtitle="Kelola kompetisi berbatas waktu untuk meningkatkan engagement dan motivasi siswa"
        action={<Button onClick={openCreate} className="bg-indigo-600 hover:bg-indigo-700 gap-2"><Plus className="w-4 h-4" />Buat Challenge</Button>}
      />

      {/* Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        {[
          { label: 'Challenge Aktif', val: stats.active, icon: Play, color: 'text-emerald-600', bg: 'bg-emerald-50' },
          { label: 'Akan Datang', val: stats.upcoming, icon: Clock, color: 'text-blue-600', bg: 'bg-blue-50' },
          { label: 'Total Peserta', val: stats.totalParticipants.toLocaleString(), icon: Users, color: 'text-indigo-600', bg: 'bg-indigo-50' },
          { label: 'Total Selesai', val: stats.totalCompletions.toLocaleString(), icon: CheckCircle2, color: 'text-amber-600', bg: 'bg-amber-50' },
        ].map(({ label, val, icon: Icon, color, bg }) => (
          <Card key={label} className="p-4 flex items-center gap-3">
            <div className={`w-10 h-10 rounded-xl ${bg} flex items-center justify-center shrink-0`}>
              <Icon className={`w-5 h-5 ${color}`} />
            </div>
            <div>
              <div className={`text-xl font-black ${color}`}>{val}</div>
              <div className="text-xs text-muted-foreground">{label}</div>
            </div>
          </Card>
        ))}
      </div>

      {/* Filter */}
      <div className="flex gap-2 mb-5">
        {(['all', 'active', 'upcoming', 'ended'] as const).map(s => (
          <button key={s} onClick={() => setFilterStatus(s)}
            className={`px-4 py-1.5 rounded-full text-sm font-semibold border-2 transition-colors ${filterStatus === s ? 'border-indigo-500 bg-indigo-50 text-indigo-700' : 'border-transparent bg-slate-100 text-slate-500 hover:bg-slate-200'}`}>
            {s === 'all' ? 'Semua' : STATUS_CONFIG[s].label}
          </button>
        ))}
      </div>

      {/* Challenge cards */}
      <div className="grid lg:grid-cols-2 gap-5">
        {filtered.map(c => {
          const sc = STATUS_CONFIG[c.status];
          const tc = CHALLENGE_TYPE_CONFIG[c.type];
          const completion = c.participants > 0 ? Math.round((c.completions / c.participants) * 100) : 0;
          return (
            <Card key={c.id} className={`p-5 hover:shadow-md transition-shadow ${c.status === 'ended' ? 'opacity-70' : ''}`}>
              <div className="flex items-start justify-between mb-3">
                <div className="flex-1 min-w-0 pr-3">
                  <div className="flex items-center gap-2 mb-1 flex-wrap">
                    <span className={`inline-flex items-center gap-1 text-[10px] font-bold px-2 py-0.5 rounded-full ${sc.color}`}>
                      <span className={`w-1.5 h-1.5 rounded-full ${sc.dot}`} />{sc.label}
                    </span>
                    <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${tc.color}`}>{tc.icon} {tc.label}</span>
                  </div>
                  <h4 className="font-bold text-slate-900">{c.name}</h4>
                  <p className="text-xs text-muted-foreground mt-0.5">{c.description}</p>
                </div>
                <div className="flex items-center gap-1 shrink-0">
                  <button onClick={() => openEdit(c)} className="p-1.5 rounded-lg hover:bg-slate-100"><Edit3 className="w-4 h-4 text-slate-400" /></button>
                  <button onClick={() => deleteChallenge(c.id)} className="p-1.5 rounded-lg hover:bg-red-50"><Trash2 className="w-4 h-4 text-slate-400 hover:text-red-500" /></button>
                </div>
              </div>

              <div className="grid grid-cols-3 gap-3 mb-3 text-center">
                <div className="bg-slate-50 rounded-xl p-2.5">
                  <p className="text-lg font-black text-slate-900">{c.participants.toLocaleString()}</p>
                  <p className="text-[10px] text-muted-foreground">Peserta</p>
                </div>
                <div className="bg-emerald-50 rounded-xl p-2.5">
                  <p className="text-lg font-black text-emerald-700">{c.completions.toLocaleString()}</p>
                  <p className="text-[10px] text-muted-foreground">Selesai</p>
                </div>
                <div className="bg-indigo-50 rounded-xl p-2.5">
                  <p className="text-lg font-black text-indigo-700">{c.rewardPoints}</p>
                  <p className="text-[10px] text-muted-foreground">Poin Reward</p>
                </div>
              </div>

              {c.participants > 0 && (
                <div className="mb-3">
                  <div className="flex justify-between text-xs text-muted-foreground mb-1">
                    <span>Completion Rate</span><span className="font-bold text-slate-700">{completion}%</span>
                  </div>
                  <div className="h-2 bg-slate-100 rounded-full overflow-hidden">
                    <div className="h-full bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full" style={{ width: `${completion}%` }} />
                  </div>
                </div>
              )}

              <div className="flex items-center justify-between pt-3 border-t">
                <div className="flex items-center gap-3 text-xs text-muted-foreground">
                  <span className="flex items-center gap-1"><Calendar className="w-3.5 h-3.5" />{c.startDate}</span>
                  <ChevronRight className="w-3 h-3" />
                  <span>{c.endDate}</span>
                </div>
                {c.rewardBadge && <span className="text-xl">{c.rewardBadge}</span>}
                {c.status !== 'ended' && (
                  <button onClick={() => toggleStatus(c.id)}
                    className={`text-xs font-semibold px-3 py-1.5 rounded-lg transition-colors ${c.status === 'active' ? 'bg-red-50 text-red-600 hover:bg-red-100' : 'bg-emerald-50 text-emerald-700 hover:bg-emerald-100'}`}>
                    {c.status === 'active' ? <span className="flex items-center gap-1"><Pause className="w-3 h-3" />Akhiri</span> : <span className="flex items-center gap-1"><Play className="w-3 h-3" />Aktifkan</span>}
                  </button>
                )}
              </div>
            </Card>
          );
        })}
      </div>

      {/* Create/Edit Modal */}
      {modal && (
        <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4">
          <Card className="w-full max-w-lg p-6 shadow-2xl max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-5">
              <h3 className="font-bold text-slate-900">{modal === 'create' ? 'Buat Challenge Baru' : 'Edit Challenge'}</h3>
              <button onClick={() => setModal(null)} className="p-1.5 rounded-lg hover:bg-slate-100"><X className="w-4 h-4" /></button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Challenge</label>
                <input value={form.name} onChange={e => setForm(f => ({ ...f, name: e.target.value }))} placeholder="contoh: Tryout Marathon Januari" className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Deskripsi</label>
                <textarea rows={2} value={form.description} onChange={e => setForm(f => ({ ...f, description: e.target.value }))} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 resize-none" />
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-2">Tipe Challenge</label>
                <div className="grid grid-cols-2 gap-2">
                  {(Object.keys(CHALLENGE_TYPE_CONFIG) as ChallengeType[]).map(t => {
                    const tc = CHALLENGE_TYPE_CONFIG[t];
                    return (
                      <button key={t} onClick={() => setForm(f => ({ ...f, type: t }))}
                        className={`flex items-center gap-2 px-3 py-2.5 rounded-xl border-2 text-sm font-semibold transition-colors ${form.type === t ? 'border-indigo-500 bg-indigo-50 text-indigo-700' : 'border-slate-200 text-slate-500 hover:border-slate-300'}`}>
                        <span>{tc.icon}</span>{tc.label}
                      </button>
                    );
                  })}
                </div>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Mulai</label>
                  <input type="date" value={form.startDate} onChange={e => setForm(f => ({ ...f, startDate: e.target.value }))} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                </div>
                <div>
                  <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Selesai</label>
                  <input type="date" value={form.endDate} onChange={e => setForm(f => ({ ...f, endDate: e.target.value }))} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                </div>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Target</label>
                  <input type="number" min={1} value={form.targetValue} onChange={e => setForm(f => ({ ...f, targetValue: +e.target.value }))} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                </div>
                <div>
                  <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-1.5">Reward Poin</label>
                  <input type="number" min={0} value={form.rewardPoints} onChange={e => setForm(f => ({ ...f, rewardPoints: +e.target.value }))} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                </div>
              </div>
              <div>
                <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider block mb-2">Reward Badge (opsional)</label>
                <div className="flex flex-wrap gap-2">
                  {BADGE_EMOJIS.slice(0, 12).map(e => (
                    <button key={e} onClick={() => setForm(f => ({ ...f, rewardBadge: e }))}
                      className={`w-9 h-9 rounded-lg text-xl flex items-center justify-center transition-colors ${form.rewardBadge === e ? 'bg-indigo-100 ring-2 ring-indigo-400' : 'bg-slate-50 hover:bg-slate-200'}`}>
                      {e}
                    </button>
                  ))}
                  <button onClick={() => setForm(f => ({ ...f, rewardBadge: undefined }))}
                    className={`w-9 h-9 rounded-lg text-xs flex items-center justify-center border-2 border-dashed transition-colors ${!form.rewardBadge ? 'border-indigo-400 bg-indigo-50' : 'border-slate-200 hover:border-slate-300 text-slate-400'}`}>
                    ∅
                  </button>
                </div>
              </div>
            </div>
            <div className="flex gap-2 mt-6">
              <Button variant="outline" className="flex-1" onClick={() => setModal(null)}>Batal</Button>
              <Button className="flex-1 bg-indigo-600 hover:bg-indigo-700" onClick={saveChallenge}><Save className="w-4 h-4 mr-1.5" />Simpan</Button>
            </div>
          </Card>
        </div>
      )}
    </div>
  );
}

// ─── Main Component ───────────────────────────────────────────────────────────
const VIEWS: { id: GamifView; label: string; icon: React.ElementType; desc: string }[] = [
  { id: 'poin',        label: 'Konfigurasi Poin',   icon: Zap,      desc: 'Aturan perolehan poin per aksi' },
  { id: 'badge',       label: 'Badge & Achievement', icon: Award,    desc: 'Buat & kelola badge siswa' },
  { id: 'leaderboard', label: 'Kontrol Leaderboard', icon: Trophy,   desc: 'Bobot, reset & override' },
  { id: 'challenge',   label: 'Challenge & Event',   icon: Sparkles, desc: 'Kompetisi berbatas waktu' },
];

export default function AdminGamification() {
  const [view, setView] = useState<GamifView>('poin');

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-gradient-to-r from-indigo-600 via-purple-600 to-pink-600 rounded-2xl p-6 text-white relative overflow-hidden">
        <div className="absolute inset-0 opacity-10" style={{ backgroundImage: 'radial-gradient(circle at 20% 50%, white 1px, transparent 1px), radial-gradient(circle at 80% 20%, white 1px, transparent 1px)', backgroundSize: '40px 40px' }} />
        <div className="relative flex items-center justify-between">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <Sparkles className="w-5 h-5 text-yellow-300" />
              <span className="text-sm font-semibold text-indigo-200">Gamification Engine</span>
            </div>
            <h1 className="text-2xl font-black mb-1">Manajemen Gamifikasi</h1>
            <p className="text-indigo-200 text-sm">Konfigurasi poin, badge, leaderboard, dan challenge untuk meningkatkan engagement siswa</p>
          </div>
          <div className="hidden md:flex items-center gap-3">
            <div className="text-center bg-white/15 rounded-xl px-4 py-2.5">
              <p className="text-2xl font-black">52K+</p>
              <p className="text-xs text-indigo-200">Poin Tersebar</p>
            </div>
            <div className="text-center bg-white/15 rounded-xl px-4 py-2.5">
              <p className="text-2xl font-black">8</p>
              <p className="text-xs text-indigo-200">Badge Aktif</p>
            </div>
            <div className="text-center bg-white/15 rounded-xl px-4 py-2.5">
              <p className="text-2xl font-black">2</p>
              <p className="text-xs text-indigo-200">Challenge Live</p>
            </div>
          </div>
        </div>
      </div>

      {/* Sub-nav */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
        {VIEWS.map(({ id, label, icon: Icon, desc }) => (
          <button key={id} onClick={() => setView(id)}
            className={`p-4 rounded-xl border-2 text-left transition-all hover:shadow-sm ${view === id ? 'border-indigo-500 bg-indigo-50' : 'border-slate-200 bg-white hover:border-slate-300'}`}>
            <div className={`w-9 h-9 rounded-lg flex items-center justify-center mb-2.5 ${view === id ? 'bg-indigo-600' : 'bg-slate-100'}`}>
              <Icon className={`w-4.5 h-4.5 ${view === id ? 'text-white' : 'text-slate-500'}`} />
            </div>
            <p className={`text-sm font-bold mb-0.5 ${view === id ? 'text-indigo-700' : 'text-slate-900'}`}>{label}</p>
            <p className="text-xs text-muted-foreground">{desc}</p>
          </button>
        ))}
      </div>

      {/* Content */}
      <div>
        {view === 'poin'        && <PointRules />}
        {view === 'badge'       && <BadgeManagement />}
        {view === 'leaderboard' && <LeaderboardControl />}
        {view === 'challenge'   && <ChallengeEvent />}
      </div>
    </div>
  );
}
