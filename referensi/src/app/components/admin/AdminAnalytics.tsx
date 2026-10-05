import { useState } from 'react';
import { Card } from '../ui/card';
import { Badge } from '../ui/badge';
import {
  LineChart, Line, BarChart, Bar, PieChart, Pie, Cell,
  XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer
} from 'recharts';
import { TrendingUp, Users, BookOpen, Award, School, ArrowUp, ArrowDown } from 'lucide-react';

const dauData = [
  { date: '1 Jan', users: 1820 }, { date: '3 Jan', users: 2150 }, { date: '5 Jan', users: 2480 },
  { date: '7 Jan', users: 3100 }, { date: '9 Jan', users: 2890 }, { date: '11 Jan', users: 3240 },
  { date: '13 Jan', users: 4100 }, { date: '15 Jan', users: 3850 }, { date: '17 Jan', users: 4320 },
  { date: '19 Jan', users: 5010 }, { date: '21 Jan', users: 5280 }, { date: '23 Jan', users: 4790 },
  { date: '25 Jan', users: 5640 }, { date: '27 Jan', users: 6120 }, { date: '29 Jan', users: 5980 },
  { date: '31 Jan', users: 6540 },
];

const revenueData = [
  { month: 'Agt', revenue: 28500000, target: 30000000 },
  { month: 'Sep', revenue: 34200000, target: 32000000 },
  { month: 'Okt', revenue: 41800000, target: 38000000 },
  { month: 'Nov', revenue: 55600000, target: 50000000 },
  { month: 'Des', revenue: 72400000, target: 65000000 },
  { month: 'Jan', revenue: 89300000, target: 80000000 },
];

const subtesData = [
  { name: 'Penalaran Umum', value: 38, fill: '#6366f1' },
  { name: 'PPU', value: 26, fill: '#8b5cf6' },
  { name: 'PBM', value: 22, fill: '#a78bfa' },
  { name: 'Pengetahuan Mat.', value: 14, fill: '#c4b5fd' },
];

const topSchools = [
  { rank: 1, name: 'SMAN 1 Jakarta', students: 342, avgScore: 724, trend: 'up', change: 12 },
  { rank: 2, name: 'SMAN 3 Bandung', students: 218, avgScore: 718, trend: 'up', change: 8 },
  { rank: 3, name: 'SMA Al-Azhar Surabaya', students: 194, avgScore: 711, trend: 'down', change: 3 },
  { rank: 4, name: 'SMAN 1 Yogyakarta', students: 276, avgScore: 708, trend: 'up', change: 15 },
  { rank: 5, name: 'SMAN 2 Semarang', students: 165, avgScore: 702, trend: 'up', change: 6 },
  { rank: 6, name: 'SMAN 5 Surabaya', students: 231, avgScore: 698, trend: 'down', change: 4 },
  { rank: 7, name: 'SMA Taruna Nusantara', students: 188, avgScore: 695, trend: 'up', change: 9 },
  { rank: 8, name: 'SMAN 1 Malang', students: 143, avgScore: 691, trend: 'up', change: 2 },
];

const growthStats = [
  { label: 'Pengguna Aktif (bulan ini)', val: '52.341', sub: '+18% vs bulan lalu', up: true, icon: Users },
  { label: 'Total Sesi Tryout', val: '284.920', sub: '+24% vs bulan lalu', up: true, icon: BookOpen },
  { label: 'Sekolah Mitra Aktif', val: '512', sub: '+31 sekolah baru', up: true, icon: School },
  { label: 'Rata-rata Skor Nasional', val: '672', sub: '+4.2 poin vs bulan lalu', up: true, icon: Award },
];

const timeFilters = ['7 hari', '30 hari', '3 bulan', '1 tahun'];

function formatRp(n: number) {
  if (n >= 1_000_000) return `Rp${(n / 1_000_000).toFixed(1)}jt`;
  return `Rp${(n / 1000).toFixed(0)}rb`;
}

export default function AdminAnalytics() {
  const [timeFilter, setTimeFilter] = useState('30 hari');

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between flex-wrap gap-3">
        <div>
          <h2 className="text-xl font-black text-slate-900">Analytics Platform</h2>
          <p className="text-sm text-muted-foreground">Pantau pertumbuhan dan performa platform secara keseluruhan</p>
        </div>
        <div className="flex gap-2">
          {timeFilters.map(t => (
            <button
              key={t}
              onClick={() => setTimeFilter(t)}
              className={`px-4 py-1.5 rounded-xl text-sm font-semibold transition-colors ${timeFilter === t ? 'bg-indigo-600 text-white' : 'bg-white border text-slate-600 hover:bg-slate-50'}`}
            >
              {t}
            </button>
          ))}
        </div>
      </div>

      {/* KPI Strip */}
      <div className="grid grid-cols-2 xl:grid-cols-4 gap-4">
        {growthStats.map(({ label, val, sub, up, icon: Icon }) => (
          <Card key={label} className="p-5">
            <div className="flex items-start justify-between mb-3">
              <div className="w-10 h-10 rounded-xl bg-indigo-50 flex items-center justify-center">
                <Icon className="w-5 h-5 text-indigo-600" />
              </div>
              <span className={`flex items-center gap-0.5 text-xs font-bold ${up ? 'text-emerald-600' : 'text-red-500'}`}>
                {up ? <ArrowUp className="w-3 h-3" /> : <ArrowDown className="w-3 h-3" />}
                {sub.split(' ')[0]}
              </span>
            </div>
            <p className="text-2xl font-black text-slate-900">{val}</p>
            <p className="text-xs text-muted-foreground mt-0.5">{label}</p>
          </Card>
        ))}
      </div>

      {/* DAU Chart */}
      <Card className="p-6">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h3 className="font-bold text-slate-900">Pengguna Aktif Harian (DAU)</h3>
            <p className="text-sm text-muted-foreground">Jumlah pengguna unik yang login & mengerjakan soal</p>
          </div>
          <Badge className="bg-emerald-100 text-emerald-700">Tumbuh +31%</Badge>
        </div>
        <ResponsiveContainer width="100%" height={240}>
          <LineChart data={dauData}>
            <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" />
            <XAxis dataKey="date" tick={{ fontSize: 11, fill: '#94a3b8' }} />
            <YAxis tick={{ fontSize: 11, fill: '#94a3b8' }} />
            <Tooltip formatter={(v: number) => [v.toLocaleString('id-ID'), 'Pengguna Aktif']} />
            <Line type="monotone" dataKey="users" stroke="#6366f1" strokeWidth={2.5} dot={false} activeDot={{ r: 5 }} />
          </LineChart>
        </ResponsiveContainer>
      </Card>

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        {/* Revenue Chart */}
        <Card className="p-6 xl:col-span-2">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h3 className="font-bold text-slate-900">Revenue Bulanan</h3>
              <p className="text-sm text-muted-foreground">Pendapatan vs target (6 bulan terakhir)</p>
            </div>
          </div>
          <ResponsiveContainer width="100%" height={220}>
            <BarChart data={revenueData} barGap={4}>
              <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" />
              <XAxis dataKey="month" tick={{ fontSize: 11, fill: '#94a3b8' }} />
              <YAxis tick={{ fontSize: 11, fill: '#94a3b8' }} tickFormatter={formatRp} />
              <Tooltip formatter={(v: number) => [`Rp${v.toLocaleString('id-ID')}`, '']} />
              <Legend />
              <Bar dataKey="revenue" name="Aktual" fill="#6366f1" radius={[4, 4, 0, 0]} />
              <Bar dataKey="target" name="Target" fill="#e2e8f0" radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </Card>

        {/* Subtes distribution */}
        <Card className="p-6">
          <h3 className="font-bold text-slate-900 mb-1">Distribusi Subtes</h3>
          <p className="text-sm text-muted-foreground mb-4">Soal yang paling banyak dikerjakan</p>
          <ResponsiveContainer width="100%" height={180}>
            <PieChart>
              <Pie data={subtesData} cx="50%" cy="50%" innerRadius={50} outerRadius={80} dataKey="value" paddingAngle={3}>
                {subtesData.map((entry, i) => <Cell key={i} fill={entry.fill} />)}
              </Pie>
              <Tooltip formatter={(v: number) => [`${v}%`, '']} />
            </PieChart>
          </ResponsiveContainer>
          <div className="space-y-2 mt-2">
            {subtesData.map(d => (
              <div key={d.name} className="flex items-center justify-between text-sm">
                <div className="flex items-center gap-2">
                  <span className="w-3 h-3 rounded-full shrink-0" style={{ backgroundColor: d.fill }} />
                  <span className="text-slate-700">{d.name}</span>
                </div>
                <span className="font-bold text-slate-900">{d.value}%</span>
              </div>
            ))}
          </div>
        </Card>
      </div>

      {/* Top Schools */}
      <Card className="overflow-hidden">
        <div className="p-5 border-b flex items-center justify-between">
          <div>
            <h3 className="font-bold text-slate-900">Top Sekolah Mitra</h3>
            <p className="text-sm text-muted-foreground">Berdasarkan rata-rata skor siswa</p>
          </div>
          <Badge className="bg-indigo-100 text-indigo-700">Bulan Ini</Badge>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-slate-50">
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Rank</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Sekolah</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Siswa Aktif</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Rata-rata Skor</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Tren</th>
              </tr>
            </thead>
            <tbody>
              {topSchools.map(s => (
                <tr key={s.rank} className="border-b hover:bg-slate-50 transition-colors">
                  <td className="px-4 py-3">
                    <span className={`w-7 h-7 rounded-full flex items-center justify-center text-xs font-black ${s.rank <= 3 ? 'bg-gradient-to-br from-indigo-500 to-purple-600 text-white' : 'bg-slate-100 text-slate-700'}`}>
                      {s.rank}
                    </span>
                  </td>
                  <td className="px-4 py-3 font-semibold text-slate-900">{s.name}</td>
                  <td className="px-4 py-3 text-slate-700">{s.students.toLocaleString('id-ID')}</td>
                  <td className="px-4 py-3">
                    <span className="font-bold text-indigo-600">{s.avgScore}</span>
                  </td>
                  <td className="px-4 py-3">
                    <span className={`flex items-center gap-1 text-xs font-bold ${s.trend === 'up' ? 'text-emerald-600' : 'text-red-500'}`}>
                      {s.trend === 'up' ? <ArrowUp className="w-3 h-3" /> : <ArrowDown className="w-3 h-3" />}
                      {s.change} poin
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>
    </div>
  );
}
