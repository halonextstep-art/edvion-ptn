import { useState } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend
} from 'recharts';
import {
  FileText, Download, Filter, Calendar, Users, TrendingUp,
  BookOpen, CheckCircle2, Clock, ArrowUp, ArrowDown, Printer
} from 'lucide-react';
import { toast } from 'sonner';

const reportTypes = [
  { id: 'performance', label: 'Performa Siswa', desc: 'Skor rata-rata, distribusi nilai, perkembangan per subtes' },
  { id: 'attendance', label: 'Kehadiran Tryout', desc: 'Partisipasi dan keaktifan siswa dalam tryout' },
  { id: 'ranking', label: 'Ranking Internal', desc: 'Peringkat siswa dalam sekolah per subtes' },
  { id: 'ptn', label: 'Analisis Target PTN', desc: 'Peta target PTN dan peluang siswa kelas 12' },
];

const periods = [
  'Januari 2025', 'Desember 2024', 'November 2024', 'Oktober 2024',
  'Semester Ganjil 2024/2025', 'Semester Genap 2023/2024',
];

const previewPerformance = [
  { kelas: 'XII IPA 1', avgPU: 68.4, avgPPU: 71.2, avgPBM: 65.8, avgPM: 72.1, total: 69.4 },
  { kelas: 'XII IPA 2', avgPU: 65.1, avgPPU: 68.4, avgPBM: 63.2, avgPM: 67.9, total: 66.2 },
  { kelas: 'XII IPS 1', avgPU: 62.8, avgPPU: 70.1, avgPBM: 67.4, avgPM: 58.3, total: 64.7 },
  { kelas: 'XI IPA 1', avgPU: 59.3, avgPPU: 62.7, avgPBM: 60.1, avgPM: 64.5, total: 61.7 },
  { kelas: 'XI IPA 2', avgPU: 57.8, avgPPU: 60.4, avgPBM: 58.9, avgPM: 61.2, total: 59.6 },
];

const trendData = [
  { month: 'Sep', PU: 62, PPU: 65, PBM: 60, PM: 64 },
  { month: 'Okt', PU: 64, PPU: 67, PBM: 63, PM: 66 },
  { month: 'Nov', PU: 65, PPU: 69, PBM: 64, PM: 68 },
  { month: 'Des', PU: 67, PPU: 70, PBM: 65, PM: 70 },
  { month: 'Jan', PU: 68, PPU: 71, PBM: 66, PM: 72 },
];

const topStudentsPreview = [
  { rank: 1, name: 'Bunga Rahayu', class: 'XII IPA 1', avgScore: 745, tryouts: 18, trend: 'up' },
  { rank: 2, name: 'Andi Pratama', class: 'XII IPA 1', avgScore: 721, tryouts: 14, trend: 'up' },
  { rank: 3, name: 'Hana Wijaya', class: 'XII IPA 2', avgScore: 710, tryouts: 15, trend: 'up' },
  { rank: 4, name: 'Cahyo Nugroho', class: 'XII IPA 2', avgScore: 698, tryouts: 10, trend: 'down' },
  { rank: 5, name: 'Fani Lestari', class: 'XII IPS 1', avgScore: 688, tryouts: 11, trend: 'up' },
];

const savedReports = [
  { id: 'r1', name: 'Laporan Performa - Desember 2024', type: 'Performa Siswa', createdAt: '2025-01-03', status: 'done' },
  { id: 'r2', name: 'Laporan Kehadiran - November 2024', type: 'Kehadiran Tryout', createdAt: '2024-12-05', status: 'done' },
  { id: 'r3', name: 'Ranking Internal - Semester Ganjil', type: 'Ranking Internal', createdAt: '2024-12-28', status: 'done' },
];

export default function SchoolReports() {
  const [selectedType, setSelectedType] = useState('performance');
  const [selectedPeriod, setSelectedPeriod] = useState('Januari 2025');
  const [filterClass, setFilterClass] = useState('all');
  const [generating, setGenerating] = useState(false);
  const [reports, setReports] = useState(savedReports);

  const generate = () => {
    setGenerating(true);
    setTimeout(() => {
      setGenerating(false);
      const label = reportTypes.find(r => r.id === selectedType)?.label ?? 'Laporan';
      const newReport = {
        id: `r${Date.now()}`,
        name: `${label} - ${selectedPeriod}`,
        type: label,
        createdAt: new Date().toISOString().split('T')[0],
        status: 'done',
      };
      setReports(prev => [newReport, ...prev]);
      toast.success('Laporan berhasil dibuat! Klik Download untuk mengunduh.');
    }, 1800);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-xl font-black text-slate-900">Laporan & Export</h2>
        <p className="text-sm text-muted-foreground">Generate laporan performa siswa dan ekspor dalam format PDF atau Excel</p>
      </div>

      {/* Generator */}
      <Card className="p-6">
        <h3 className="font-bold text-slate-900 mb-4">Generate Laporan Baru</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* Type */}
          <div>
            <p className="text-sm font-semibold text-slate-700 mb-3">Jenis Laporan</p>
            <div className="space-y-2">
              {reportTypes.map(rt => (
                <button
                  key={rt.id}
                  onClick={() => setSelectedType(rt.id)}
                  className={`w-full text-left p-3 rounded-xl border transition-all ${selectedType === rt.id ? 'border-indigo-500 bg-indigo-50' : 'border-slate-200 hover:bg-slate-50'}`}
                >
                  <p className={`text-sm font-semibold ${selectedType === rt.id ? 'text-indigo-700' : 'text-slate-900'}`}>{rt.label}</p>
                  <p className="text-xs text-muted-foreground mt-0.5 leading-tight">{rt.desc}</p>
                </button>
              ))}
            </div>
          </div>

          {/* Period + Class */}
          <div className="space-y-4">
            <div>
              <p className="text-sm font-semibold text-slate-700 mb-2 flex items-center gap-2"><Calendar className="w-4 h-4" />Periode</p>
              <div className="space-y-1.5">
                {periods.map(p => (
                  <label key={p} className="flex items-center gap-2 cursor-pointer">
                    <input type="radio" name="period" value={p} checked={selectedPeriod === p} onChange={() => setSelectedPeriod(p)} className="accent-indigo-600" />
                    <span className="text-sm text-slate-700">{p}</span>
                  </label>
                ))}
              </div>
            </div>
            <div>
              <p className="text-sm font-semibold text-slate-700 mb-2 flex items-center gap-2"><Filter className="w-4 h-4" />Filter Kelas</p>
              <select className="w-full border rounded-xl px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterClass} onChange={e => setFilterClass(e.target.value)}>
                <option value="all">Semua Kelas</option>
                <option value="12">Kelas XII</option>
                <option value="11">Kelas XI</option>
                <option value="10">Kelas X</option>
              </select>
            </div>
          </div>

          {/* Summary + actions */}
          <div className="flex flex-col">
            <div className="bg-slate-50 border rounded-xl p-4 flex-1 mb-4">
              <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-3">Ringkasan Laporan</p>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between"><span className="text-muted-foreground">Jenis:</span><span className="font-semibold">{reportTypes.find(r => r.id === selectedType)?.label}</span></div>
                <div className="flex justify-between"><span className="text-muted-foreground">Periode:</span><span className="font-semibold">{selectedPeriod}</span></div>
                <div className="flex justify-between"><span className="text-muted-foreground">Cakupan:</span><span className="font-semibold">{filterClass === 'all' ? 'Semua Kelas' : `Kelas ${filterClass}`}</span></div>
                <div className="flex justify-between"><span className="text-muted-foreground">Siswa:</span><span className="font-semibold">8 siswa</span></div>
              </div>
            </div>
            <div className="space-y-2">
              <Button className="w-full bg-gradient-to-r from-indigo-600 to-purple-600 gap-2" onClick={generate} disabled={generating}>
                {generating ? <><Clock className="w-4 h-4 animate-spin" />Generating...</> : <><FileText className="w-4 h-4" />Generate Laporan</>}
              </Button>
              <div className="flex gap-2">
                <Button variant="outline" className="flex-1 gap-1.5 text-sm" onClick={() => toast.success('Laporan diekspor sebagai PDF')}>
                  <Download className="w-4 h-4" />PDF
                </Button>
                <Button variant="outline" className="flex-1 gap-1.5 text-sm" onClick={() => toast.success('Laporan diekspor sebagai Excel')}>
                  <Download className="w-4 h-4" />Excel
                </Button>
                <Button variant="outline" className="flex-1 gap-1.5 text-sm" onClick={() => toast.success('Membuka dialog print...')}>
                  <Printer className="w-4 h-4" />Print
                </Button>
              </div>
            </div>
          </div>
        </div>
      </Card>

      {/* Preview */}
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
        {/* Table preview */}
        <Card className="overflow-hidden">
          <div className="p-4 border-b flex items-center justify-between">
            <h3 className="font-bold text-slate-900">Preview: Rata-rata per Kelas</h3>
            <Badge className="bg-indigo-100 text-indigo-700">{selectedPeriod}</Badge>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b bg-slate-50">
                  <th className="text-left px-4 py-2.5 font-semibold text-slate-600">Kelas</th>
                  <th className="text-left px-4 py-2.5 font-semibold text-slate-600">PU</th>
                  <th className="text-left px-4 py-2.5 font-semibold text-slate-600">PPU</th>
                  <th className="text-left px-4 py-2.5 font-semibold text-slate-600">PBM</th>
                  <th className="text-left px-4 py-2.5 font-semibold text-slate-600">PM</th>
                  <th className="text-left px-4 py-2.5 font-semibold text-slate-600">Rata</th>
                </tr>
              </thead>
              <tbody>
                {previewPerformance.map(row => (
                  <tr key={row.kelas} className="border-b hover:bg-slate-50 transition-colors">
                    <td className="px-4 py-2.5 font-semibold text-slate-900">{row.kelas}</td>
                    <td className="px-4 py-2.5">{row.avgPU}</td>
                    <td className="px-4 py-2.5">{row.avgPPU}</td>
                    <td className="px-4 py-2.5">{row.avgPBM}</td>
                    <td className="px-4 py-2.5">{row.avgPM}</td>
                    <td className="px-4 py-2.5">
                      <span className={`font-bold ${row.total >= 68 ? 'text-emerald-600' : row.total >= 63 ? 'text-blue-600' : 'text-amber-600'}`}>{row.total}</span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>

        {/* Trend chart */}
        <Card className="p-5">
          <h3 className="font-bold text-slate-900 mb-1">Tren Skor per Subtes</h3>
          <p className="text-sm text-muted-foreground mb-4">5 bulan terakhir (skala 0-100)</p>
          <ResponsiveContainer width="100%" height={200}>
            <BarChart data={trendData} barGap={2}>
              <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" />
              <XAxis dataKey="month" tick={{ fontSize: 11, fill: '#94a3b8' }} />
              <YAxis domain={[50, 80]} tick={{ fontSize: 11, fill: '#94a3b8' }} />
              <Tooltip />
              <Legend iconSize={10} />
              <Bar key="bar-PU" dataKey="PU" name="PU" fill="#6366f1" radius={[3, 3, 0, 0]} />
              <Bar key="bar-PPU" dataKey="PPU" name="PPU" fill="#8b5cf6" radius={[3, 3, 0, 0]} />
              <Bar key="bar-PBM" dataKey="PBM" name="PBM" fill="#a78bfa" radius={[3, 3, 0, 0]} />
              <Bar key="bar-PM" dataKey="PM" name="PM" fill="#c4b5fd" radius={[3, 3, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </Card>
      </div>

      {/* Top students */}
      <Card className="overflow-hidden">
        <div className="p-4 border-b">
          <h3 className="font-bold text-slate-900">Preview: Top 5 Siswa</h3>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-slate-50">
                <th className="text-left px-4 py-2.5 font-semibold text-slate-600">Rank</th>
                <th className="text-left px-4 py-2.5 font-semibold text-slate-600">Siswa</th>
                <th className="text-left px-4 py-2.5 font-semibold text-slate-600">Kelas</th>
                <th className="text-left px-4 py-2.5 font-semibold text-slate-600">Avg Skor</th>
                <th className="text-left px-4 py-2.5 font-semibold text-slate-600">Tryout</th>
                <th className="text-left px-4 py-2.5 font-semibold text-slate-600">Tren</th>
              </tr>
            </thead>
            <tbody>
              {topStudentsPreview.map(s => (
                <tr key={s.rank} className="border-b hover:bg-slate-50">
                  <td className="px-4 py-2.5">
                    <span className={`w-7 h-7 rounded-full flex items-center justify-center text-xs font-black ${s.rank <= 3 ? 'bg-gradient-to-br from-indigo-500 to-purple-600 text-white' : 'bg-slate-100 text-slate-700'}`}>{s.rank}</span>
                  </td>
                  <td className="px-4 py-2.5 font-semibold text-slate-900">{s.name}</td>
                  <td className="px-4 py-2.5 text-slate-600">{s.class}</td>
                  <td className="px-4 py-2.5 font-bold text-indigo-600">{s.avgScore}</td>
                  <td className="px-4 py-2.5 text-slate-700">{s.tryouts} sesi</td>
                  <td className="px-4 py-2.5">
                    <span className={`flex items-center gap-1 text-xs font-bold ${s.trend === 'up' ? 'text-emerald-600' : 'text-red-500'}`}>
                      {s.trend === 'up' ? <ArrowUp className="w-3 h-3" /> : <ArrowDown className="w-3 h-3" />}
                      {s.trend === 'up' ? 'Naik' : 'Turun'}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>

      {/* Saved Reports */}
      <Card className="overflow-hidden">
        <div className="p-5 border-b">
          <h3 className="font-bold text-slate-900">Laporan Tersimpan</h3>
          <p className="text-sm text-muted-foreground">Laporan yang sudah pernah dibuat</p>
        </div>
        <div className="divide-y">
          {reports.map(r => (
            <div key={r.id} className="flex items-center justify-between px-5 py-4 hover:bg-slate-50 transition-colors">
              <div className="flex items-start gap-3">
                <div className="w-10 h-10 rounded-xl bg-indigo-50 flex items-center justify-center shrink-0">
                  <FileText className="w-5 h-5 text-indigo-600" />
                </div>
                <div>
                  <p className="font-semibold text-slate-900">{r.name}</p>
                  <div className="flex items-center gap-2 mt-0.5">
                    <Badge className="text-xs bg-slate-100 text-slate-600">{r.type}</Badge>
                    <span className="text-xs text-muted-foreground flex items-center gap-1"><Calendar className="w-3 h-3" />{r.createdAt}</span>
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Badge className="bg-emerald-100 text-emerald-700 gap-1"><CheckCircle2 className="w-3 h-3" />Siap</Badge>
                <Button size="sm" variant="outline" className="gap-1.5 text-xs h-8" onClick={() => toast.success(`Mengunduh ${r.name}`)}>
                  <Download className="w-3.5 h-3.5" />Download
                </Button>
              </div>
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
}
