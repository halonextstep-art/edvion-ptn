import { useState } from 'react';
import { Trophy, Medal, Flame, TrendingUp, Star, Users, Crown, Zap, Award, BarChart3 } from 'lucide-react';

type LeaderboardTab = 'nasional' | 'sekolah' | 'subtes';
type TimeRange = 'harian' | 'mingguan' | 'alltime';
type SubtesFilter = 'semua' | 'PU' | 'PPU' | 'PBM' | 'PM';

interface LeaderboardEntry {
  rank: number;
  name: string;
  school: string;
  city: string;
  score: number;
  streak: number;
  badges: string[];
  avatar: string;
  avatarGrad: string;
  change: number; // rank change from last period
  isMe?: boolean;
}

const myRankNasional = 312;
const myRankSekolah = 14;

function generateNasional(): LeaderboardEntry[] {
  const names = [
    ['Galih Satria Nugraha', 'SMAN 8 Jakarta', 'Jakarta'],
    ['Putri Ramadhani', 'SMA Negeri 3 Surabaya', 'Surabaya'],
    ['Reza Firmansyah', 'SMA Negeri 1 Bandung', 'Bandung'],
    ['Nabila Khairunnisa', 'SMA Negeri 5 Yogyakarta', 'Yogyakarta'],
    ['Alif Maulana', 'SMA Negeri 1 Semarang', 'Semarang'],
    ['Sinta Permatasari', 'MA Negeri 1 Malang', 'Malang'],
    ['Dimas Pratama', 'SMA Negeri 2 Medan', 'Medan'],
    ['Kirana Dewi', 'SMA Negeri 1 Makassar', 'Makassar'],
    ['Fadhil Akbar', 'SMA Negeri 3 Denpasar', 'Denpasar'],
    ['Zahra Aulia', 'SMA Negeri 1 Palembang', 'Palembang'],
  ];
  const grads = [
    'from-violet-400 to-purple-500', 'from-rose-400 to-pink-500', 'from-blue-400 to-cyan-500',
    'from-amber-400 to-orange-500', 'from-emerald-400 to-teal-500', 'from-fuchsia-400 to-violet-500',
    'from-cyan-400 to-sky-500', 'from-orange-400 to-red-500', 'from-indigo-400 to-blue-500',
    'from-teal-400 to-emerald-500',
  ];
  const scores = [948, 932, 918, 905, 891, 879, 865, 854, 841, 827];
  return names.map(([name, school, city], i) => ({
    rank: i + 1,
    name, school, city,
    score: scores[i],
    streak: Math.floor(Math.random() * 60) + 10,
    badges: i < 3 ? ['🏆', '⭐', '🔥'] : i < 6 ? ['⭐', '🔥'] : ['🔥'],
    avatar: name.split(' ').map(n => n[0]).join('').slice(0, 2),
    avatarGrad: grads[i],
    change: Math.floor(Math.random() * 10) - 5,
  }));
}

function generateSekolah(): LeaderboardEntry[] {
  const names = [
    ['Ahmad Fauzi Rahman', 'IPA 1'], ['Budi Santoso', 'IPA 2'], ['Citra Lestari', 'IPA 1'],
    ['Deni Kurniawan', 'IPA 3'], ['Eka Putri', 'IPS 1'], ['Fitra Ramadhan', 'IPA 2'],
    ['Gina Amalia', 'IPS 2'], ['Hendra Wijaya', 'IPA 1'], ['Ika Sari', 'IPA 3'],
    ['Joko Prabowo', 'IPS 1'],
  ];
  const grads = [
    'from-violet-400 to-purple-500', 'from-blue-400 to-cyan-500', 'from-emerald-400 to-teal-500',
    'from-amber-400 to-orange-500', 'from-rose-400 to-pink-500', 'from-indigo-400 to-blue-500',
    'from-cyan-400 to-sky-500', 'from-orange-400 to-red-500', 'from-fuchsia-400 to-violet-500',
    'from-teal-400 to-emerald-500',
  ];
  const scores = [875, 862, 848, 835, 821, 808, 795, 783, 769, 754];
  return names.map(([name, cls], i) => ({
    rank: i + 1,
    name,
    school: `Kelas ${cls}`,
    city: 'SMA Negeri 1 Jakarta',
    score: scores[i],
    streak: Math.floor(Math.random() * 30) + 5,
    badges: i < 3 ? ['🏅', '⭐'] : ['🔥'],
    avatar: name.split(' ').map(n => n[0]).join('').slice(0, 2),
    avatarGrad: grads[i],
    change: Math.floor(Math.random() * 6) - 3,
    isMe: i === 13, // not in top 10 — shown in "my rank" footer
  }));
}

const subtesScores: Record<string, number[]> = {
  PU: [945, 928, 912, 899, 884, 870, 856, 843, 829, 815],
  PPU: [938, 920, 906, 892, 878, 864, 849, 835, 820, 806],
  PBM: [952, 936, 920, 905, 890, 876, 862, 848, 834, 819],
  PM: [941, 924, 908, 894, 880, 866, 852, 838, 824, 810],
};

function Medal1() { return <Crown className="w-5 h-5 text-amber-400" />; }
function Medal2() { return <Medal className="w-5 h-5 text-slate-400" />; }
function Medal3() { return <Award className="w-5 h-5 text-amber-700" />; }

function RankIcon({ rank }: { rank: number }) {
  if (rank === 1) return <Medal1 />;
  if (rank === 2) return <Medal2 />;
  if (rank === 3) return <Medal3 />;
  return <span className="text-sm font-black text-slate-500 w-5 text-center">#{rank}</span>;
}

export default function Leaderboard({ myName = 'Ahmad Fauzi', onGoToDrilling }: { myName?: string; onGoToDrilling?: () => void }) {
  const [tab, setTab] = useState<LeaderboardTab>('nasional');
  const [timeRange, setTimeRange] = useState<TimeRange>('mingguan');
  const [subtesFilter, setSubtesFilter] = useState<SubtesFilter>('semua');

  const nasionalData = generateNasional();
  const sekolahData = generateSekolah();

  const topThree = tab === 'nasional' ? nasionalData.slice(0, 3) : sekolahData.slice(0, 3);
  const rest = tab === 'nasional' ? nasionalData.slice(3) : sekolahData.slice(3);
  const myRank = tab === 'nasional' ? myRankNasional : myRankSekolah;
  const myScore = tab === 'nasional' ? 723 : 821;

  return (
    <div className="space-y-5">
      {/* Header */}
      <div className="relative bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 rounded-3xl p-6 text-white overflow-hidden">
        <div className="absolute inset-0 opacity-10" style={{ backgroundImage: 'linear-gradient(rgba(255,255,255,.4) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,.4) 1px, transparent 1px)', backgroundSize: '40px 40px' }} />
        <div className="relative flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <Trophy className="w-6 h-6 text-amber-300" />
              <span className="font-bold text-lg">Leaderboard GASPOLPTN</span>
            </div>
            <p className="text-purple-200 text-sm">Kompetisi nasional real-time. Pertahankan posisimu!</p>
          </div>
          <div className="flex gap-3">
            {[
              { label: 'Rank Kamu', val: `#${myRank}`, sub: 'Nasional' },
              { label: 'Skor', val: myScore.toString(), sub: 'UTBK Prediksi' },
              { label: 'Streak', val: '24 🔥', sub: 'Hari berturut' },
            ].map(({ label, val, sub }) => (
              <div key={label} className="bg-white/15 backdrop-blur-sm px-4 py-2.5 rounded-xl text-center">
                <p className="text-lg font-black">{val}</p>
                <p className="text-xs text-purple-200">{sub}</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Tabs + Filters */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
        <div className="flex bg-white border rounded-xl p-1 shadow-sm">
          {([['nasional', 'Nasional', Users], ['sekolah', 'Sekolahku', Trophy], ['subtes', 'Per Subtes', BarChart3]] as [LeaderboardTab, string, typeof Users][]).map(([id, label, Icon]) => (
            <button
              key={id}
              onClick={() => setTab(id)}
              className={`flex items-center gap-1.5 px-4 py-2 rounded-lg text-sm font-semibold transition-all ${tab === id ? 'bg-indigo-600 text-white shadow-md' : 'text-slate-500 hover:text-slate-800'}`}
            >
              <Icon className="w-4 h-4" />
              {label}
            </button>
          ))}
        </div>
        <div className="flex items-center gap-2">
          {tab === 'subtes' && (
            <div className="flex bg-white border rounded-xl p-1 shadow-sm">
              {(['semua', 'PU', 'PPU', 'PBM', 'PM'] as SubtesFilter[]).map(s => (
                <button key={s} onClick={() => setSubtesFilter(s)} className={`px-3 py-1.5 rounded-lg text-xs font-bold transition-all ${subtesFilter === s ? 'bg-indigo-600 text-white' : 'text-slate-500 hover:text-slate-800'}`}>
                  {s}
                </button>
              ))}
            </div>
          )}
          <div className="flex bg-white border rounded-xl p-1 shadow-sm">
            {([['harian', 'Hari Ini'], ['mingguan', 'Minggu Ini'], ['alltime', 'All Time']] as [TimeRange, string][]).map(([id, label]) => (
              <button key={id} onClick={() => setTimeRange(id)} className={`px-3 py-1.5 rounded-lg text-xs font-bold transition-all ${timeRange === id ? 'bg-purple-600 text-white' : 'text-slate-500 hover:text-slate-800'}`}>
                {label}
              </button>
            ))}
          </div>
        </div>
      </div>

      {tab !== 'subtes' ? (
        <>
          {/* Podium — top 3 */}
          <div className="grid grid-cols-3 gap-4">
            {/* 2nd */}
            <div className="flex flex-col items-center pt-8">
              <div className={`w-16 h-16 rounded-full bg-gradient-to-br ${topThree[1].avatarGrad} flex items-center justify-center text-white font-black text-lg border-4 border-slate-300 shadow-lg mb-3`}>
                {topThree[1].avatar}
              </div>
              <Medal2 />
              <p className="text-sm font-bold text-slate-800 text-center mt-1.5 line-clamp-1">{topThree[1].name}</p>
              <p className="text-xs text-slate-500 text-center line-clamp-1">{topThree[1].school}</p>
              <div className="mt-2 px-4 py-1.5 bg-slate-100 rounded-full text-sm font-black text-slate-700">{topThree[1].score}</div>
            </div>

            {/* 1st */}
            <div className="flex flex-col items-center">
              <div className="relative">
                <div className="absolute -top-4 left-1/2 -translate-x-1/2">
                  <Crown className="w-8 h-8 text-amber-400 drop-shadow-lg" />
                </div>
                <div className={`w-20 h-20 rounded-full bg-gradient-to-br ${topThree[0].avatarGrad} flex items-center justify-center text-white font-black text-xl border-4 border-amber-400 shadow-xl`}>
                  {topThree[0].avatar}
                </div>
              </div>
              <div className="mt-3 flex flex-col items-center">
                <Medal1 />
                <p className="text-base font-bold text-slate-900 text-center mt-1">{topThree[0].name}</p>
                <p className="text-xs text-slate-500 text-center">{topThree[0].school}</p>
                <div className="mt-2 px-5 py-1.5 bg-gradient-to-r from-amber-400 to-orange-500 rounded-full text-sm font-black text-white shadow-md">{topThree[0].score}</div>
              </div>
            </div>

            {/* 3rd */}
            <div className="flex flex-col items-center pt-10">
              <div className={`w-14 h-14 rounded-full bg-gradient-to-br ${topThree[2].avatarGrad} flex items-center justify-center text-white font-black border-4 border-amber-700/50 shadow-lg mb-3`}>
                {topThree[2].avatar}
              </div>
              <Medal3 />
              <p className="text-sm font-bold text-slate-800 text-center mt-1.5 line-clamp-1">{topThree[2].name}</p>
              <p className="text-xs text-slate-500 text-center line-clamp-1">{topThree[2].school}</p>
              <div className="mt-2 px-4 py-1.5 bg-amber-100 rounded-full text-sm font-black text-amber-700">{topThree[2].score}</div>
            </div>
          </div>

          {/* Rest of list */}
          <div className="bg-white rounded-2xl border shadow-sm overflow-hidden">
            <div className="grid grid-cols-[40px_1fr_auto_auto] sm:grid-cols-[40px_1fr_auto_auto_auto] gap-3 px-5 py-2.5 bg-slate-50 border-b text-xs font-bold text-slate-400 uppercase tracking-wider">
              <span>#</span>
              <span>Siswa</span>
              <span className="hidden sm:block text-right">Streak</span>
              <span className="text-right">Skor</span>
              <span className="text-right">Δ</span>
            </div>
            {rest.map((entry) => (
              <div key={entry.rank} className={`grid grid-cols-[40px_1fr_auto_auto] sm:grid-cols-[40px_1fr_auto_auto_auto] gap-3 px-5 py-3.5 border-b last:border-0 hover:bg-indigo-50/50 transition-colors items-center ${entry.isMe ? 'bg-indigo-50 border-l-4 border-l-indigo-500' : ''}`}>
                <div className="flex items-center justify-center">
                  <RankIcon rank={entry.rank} />
                </div>
                <div className="flex items-center gap-3 min-w-0">
                  <div className={`w-9 h-9 rounded-xl bg-gradient-to-br ${entry.avatarGrad} flex items-center justify-center text-white text-xs font-black shrink-0`}>
                    {entry.avatar}
                  </div>
                  <div className="min-w-0">
                    <p className="text-sm font-bold text-slate-800 truncate">{entry.name}</p>
                    <p className="text-xs text-slate-400 truncate">{entry.school} · {entry.city}</p>
                  </div>
                </div>
                <div className="hidden sm:flex items-center gap-1 text-sm font-bold text-orange-500 justify-end">
                  <Flame className="w-3.5 h-3.5" />{entry.streak}d
                </div>
                <div className="text-right">
                  <span className="text-base font-black text-slate-900">{entry.score}</span>
                </div>
                <div className="text-right">
                  <span className={`text-xs font-bold ${entry.change > 0 ? 'text-emerald-500' : entry.change < 0 ? 'text-red-500' : 'text-slate-400'}`}>
                    {entry.change > 0 ? `↑${entry.change}` : entry.change < 0 ? `↓${Math.abs(entry.change)}` : '–'}
                  </span>
                </div>
              </div>
            ))}
          </div>

          {/* My rank footer */}
          <div className="bg-gradient-to-r from-indigo-600 to-purple-600 rounded-2xl p-4 flex items-center gap-4 shadow-lg">
            <div className="w-10 h-10 rounded-xl bg-white/20 flex items-center justify-center text-white font-black text-sm">AF</div>
            <div className="flex-1">
              <p className="font-bold text-white text-sm">{myName} (Kamu)</p>
              <p className="text-xs text-indigo-200">SMA Negeri 1 Jakarta</p>
            </div>
            <div className="flex items-center gap-6">
              <div className="text-center">
                <p className="text-xl font-black text-white">#{myRank}</p>
                <p className="text-xs text-indigo-200">Rank</p>
              </div>
              <div className="text-center">
                <p className="text-xl font-black text-white">{myScore}</p>
                <p className="text-xs text-indigo-200">Skor</p>
              </div>
            </div>
            <button onClick={onGoToDrilling} className="px-4 py-2 bg-white/20 hover:bg-white/30 rounded-xl text-sm font-bold text-white transition-colors flex items-center gap-1.5">
              <TrendingUp className="w-4 h-4" /> Naikkan Rank
            </button>
          </div>
        </>
      ) : (
        // Per-subtes view
        <div className="space-y-5">
          {(subtesFilter === 'semua' ? ['PU', 'PPU', 'PBM', 'PM'] : [subtesFilter]).map(subtes => {
            const subtesNames: Record<string, string> = {
              PU: 'Penalaran Umum',
              PPU: 'Pemahaman & Pengetahuan Umum',
              PBM: 'Penalaran Bahasa Melayu-Indonesia',
              PM: 'Penalaran Matematika',
            };
            const subtesColors: Record<string, string> = {
              PU: 'from-violet-500 to-purple-500',
              PPU: 'from-blue-500 to-cyan-500',
              PBM: 'from-emerald-500 to-teal-500',
              PM: 'from-orange-500 to-amber-500',
            };
            const scores = subtesScores[subtes];
            const dummyNames = ['Galih S.N.', 'Putri R.', 'Reza F.', 'Nabila K.', 'Alif M.', 'Sinta P.', 'Dimas P.', 'Kirana D.', 'Fadhil A.', 'Zahra A.'];
            return (
              <div key={subtes} className="bg-white rounded-2xl border shadow-sm overflow-hidden">
                <div className={`px-5 py-3.5 bg-gradient-to-r ${subtesColors[subtes]} flex items-center justify-between`}>
                  <div>
                    <p className="text-xs font-bold text-white/70 uppercase tracking-wider">{subtes}</p>
                    <p className="font-black text-white">{subtesNames[subtes]}</p>
                  </div>
                  <Star className="w-5 h-5 text-white/70" />
                </div>
                <div className="divide-y">
                  {dummyNames.slice(0, 5).map((name, i) => (
                    <div key={i} className="flex items-center gap-3 px-5 py-3">
                      <span className="text-xs font-black text-slate-400 w-5">#{i + 1}</span>
                      <div className={`w-8 h-8 rounded-lg bg-gradient-to-br ${Object.values(subtesColors)[i % 4]} flex items-center justify-center text-white text-xs font-black`}>
                        {name.slice(0, 2)}
                      </div>
                      <span className="flex-1 text-sm font-semibold text-slate-800">{name}</span>
                      <span className="font-black text-slate-900">{scores[i]}</span>
                    </div>
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

