import { useState } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer
} from 'recharts';
import {
  DollarSign, TrendingUp, ArrowUp, ArrowDown, Download,
  Calendar, CheckCircle2, Clock, AlertCircle, Filter
} from 'lucide-react';
import { toast } from 'sonner';

const revenueBreakdown = [
  { month: 'Sep', b2c: 18500000, b2b: 15700000 },
  { month: 'Okt', b2c: 22300000, b2b: 19500000 },
  { month: 'Nov', b2c: 31400000, b2b: 24200000 },
  { month: 'Des', b2c: 44100000, b2b: 28300000 },
  { month: 'Jan', b2c: 58700000, b2b: 30600000 },
];

type TxStatus = 'success' | 'pending' | 'failed';

interface Transaction {
  id: string;
  date: string;
  description: string;
  type: 'B2C' | 'B2B' | 'Event';
  amount: number;
  status: TxStatus;
  ref: string;
}

const transactions: Transaction[] = [
  { id: 'tx1', date: '2025-01-15', description: 'Premium Langganan - Andi P.', type: 'B2C', amount: 79000, status: 'success', ref: 'INV-2025011501' },
  { id: 'tx2', date: '2025-01-15', description: 'Elite Langganan - Bunga R.', type: 'B2C', amount: 199000, status: 'success', ref: 'INV-2025011502' },
  { id: 'tx3', date: '2025-01-14', description: 'Lisensi Sekolah - SMAN 1 Jakarta', type: 'B2B', amount: 4500000, status: 'success', ref: 'INV-2025011401' },
  { id: 'tx4', date: '2025-01-14', description: 'Tryout Nasional UTBK 2025 - Pendaftaran', type: 'Event', amount: 25000, status: 'pending', ref: 'INV-2025011402' },
  { id: 'tx5', date: '2025-01-13', description: 'Premium Langganan - Cahyo N.', type: 'B2C', amount: 79000, status: 'success', ref: 'INV-2025011301' },
  { id: 'tx6', date: '2025-01-13', description: 'Lisensi Sekolah - SMAN 3 Bandung', type: 'B2B', amount: 3800000, status: 'success', ref: 'INV-2025011302' },
  { id: 'tx7', date: '2025-01-12', description: 'Elite Langganan - Dewi K.', type: 'B2C', amount: 199000, status: 'failed', ref: 'INV-2025011201' },
  { id: 'tx8', date: '2025-01-12', description: 'Tryout Mini Series - Pendaftaran', type: 'Event', amount: 15000, status: 'success', ref: 'INV-2025011202' },
];

interface Payout {
  id: string;
  school: string;
  period: string;
  amount: number;
  status: 'paid' | 'pending' | 'processing';
}

const payouts: Payout[] = [
  { id: 'po1', school: 'SMAN 1 Jakarta', period: 'Desember 2024', amount: 1125000, status: 'paid' },
  { id: 'po2', school: 'SMAN 3 Bandung', period: 'Desember 2024', amount: 950000, status: 'paid' },
  { id: 'po3', school: 'SMA Al-Azhar Surabaya', period: 'Desember 2024', amount: 875000, status: 'processing' },
  { id: 'po4', school: 'SMAN 1 Yogyakarta', period: 'Januari 2025', amount: 1340000, status: 'pending' },
  { id: 'po5', school: 'SMAN 2 Semarang', period: 'Januari 2025', amount: 820000, status: 'pending' },
];

const statusConfig: Record<TxStatus, { label: string; color: string; icon: React.ElementType }> = {
  success: { label: 'Berhasil', color: 'bg-emerald-100 text-emerald-700', icon: CheckCircle2 },
  pending: { label: 'Pending', color: 'bg-amber-100 text-amber-700', icon: Clock },
  failed: { label: 'Gagal', color: 'bg-red-100 text-red-700', icon: AlertCircle },
};

const typeColor: Record<string, string> = {
  B2C: 'bg-indigo-100 text-indigo-700',
  B2B: 'bg-purple-100 text-purple-700',
  Event: 'bg-orange-100 text-orange-700',
};

const payoutStatusConfig: Record<string, { label: string; color: string }> = {
  paid: { label: 'Dibayar', color: 'bg-emerald-100 text-emerald-700' },
  processing: { label: 'Diproses', color: 'bg-blue-100 text-blue-700' },
  pending: { label: 'Pending', color: 'bg-amber-100 text-amber-700' },
};

function formatRp(n: number) {
  return `Rp${n.toLocaleString('id-ID')}`;
}

export default function AdminFinance() {
  const [txFilter, setTxFilter] = useState<'all' | 'B2C' | 'B2B' | 'Event'>('all');
  const [payoutList, setPayoutList] = useState(payouts);

  const filteredTx = txFilter === 'all' ? transactions : transactions.filter(t => t.type === txFilter);

  const kpis = {
    totalRevenue: 89300000,
    b2c: 58700000,
    b2b: 30600000,
    pending: transactions.filter(t => t.status === 'pending').reduce((s, t) => s + t.amount, 0),
  };

  const processPayout = (id: string) => {
    setPayoutList(prev => prev.map(p => p.id === id ? { ...p, status: 'processing' as const } : p));
    toast.success('Payout sedang diproses');
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between flex-wrap gap-3">
        <div>
          <h2 className="text-xl font-black text-slate-900">Manajemen Keuangan</h2>
          <p className="text-sm text-muted-foreground">Laporan pendapatan, transaksi, dan revenue sharing mitra</p>
        </div>
        <Button variant="outline" className="gap-2" onClick={() => toast.success('Export laporan keuangan berhasil')}>
          <Download className="w-4 h-4" /> Export Laporan
        </Button>
      </div>

      {/* KPI */}
      <div className="grid grid-cols-2 xl:grid-cols-4 gap-4">
        {[
          { label: 'Total Revenue Jan 2025', val: formatRp(kpis.totalRevenue), sub: '+24% vs Desember', up: true, color: 'text-indigo-600', bg: 'bg-indigo-50 border-indigo-200' },
          { label: 'Revenue B2C', val: formatRp(kpis.b2c), sub: '66% dari total', up: true, color: 'text-purple-600', bg: 'bg-purple-50 border-purple-200' },
          { label: 'Revenue B2B', val: formatRp(kpis.b2b), sub: '34% dari total', up: true, color: 'text-blue-600', bg: 'bg-blue-50 border-blue-200' },
          { label: 'Transaksi Pending', val: formatRp(kpis.pending), sub: `${transactions.filter(t => t.status === 'pending').length} transaksi`, up: false, color: 'text-amber-600', bg: 'bg-amber-50 border-amber-200' },
        ].map(({ label, val, sub, up, color, bg }) => (
          <Card key={label} className={`p-5 border ${bg}`}>
            <p className="text-xs text-muted-foreground mb-2">{label}</p>
            <p className={`text-xl font-black ${color}`}>{val}</p>
            <span className={`flex items-center gap-0.5 text-xs font-semibold mt-1 ${up ? 'text-emerald-600' : 'text-amber-600'}`}>
              {up ? <ArrowUp className="w-3 h-3" /> : <AlertCircle className="w-3 h-3" />} {sub}
            </span>
          </Card>
        ))}
      </div>

      {/* Revenue Breakdown Chart */}
      <Card className="p-6">
        <div className="flex items-center justify-between mb-5">
          <div>
            <h3 className="font-bold text-slate-900">Revenue B2C vs B2B</h3>
            <p className="text-sm text-muted-foreground">5 bulan terakhir</p>
          </div>
        </div>
        <ResponsiveContainer width="100%" height={220}>
          <BarChart data={revenueBreakdown} barGap={6}>
            <CartesianGrid strokeDasharray="3 3" stroke="#f1f5f9" />
            <XAxis dataKey="month" tick={{ fontSize: 11, fill: '#94a3b8' }} />
            <YAxis tick={{ fontSize: 11, fill: '#94a3b8' }} tickFormatter={v => `${(v / 1_000_000).toFixed(0)}jt`} />
            <Tooltip formatter={(v: number) => [formatRp(v), '']} />
            <Bar dataKey="b2c" name="B2C (Siswa)" fill="#6366f1" radius={[4, 4, 0, 0]} />
            <Bar dataKey="b2b" name="B2B (Sekolah)" fill="#a78bfa" radius={[4, 4, 0, 0]} />
          </BarChart>
        </ResponsiveContainer>
      </Card>

      {/* Transactions */}
      <Card className="overflow-hidden">
        <div className="p-5 border-b flex items-center justify-between flex-wrap gap-3">
          <div>
            <h3 className="font-bold text-slate-900">Riwayat Transaksi</h3>
            <p className="text-sm text-muted-foreground">{transactions.length} transaksi terbaru</p>
          </div>
          <div className="flex gap-2">
            {(['all', 'B2C', 'B2B', 'Event'] as const).map(f => (
              <button key={f} onClick={() => setTxFilter(f)} className={`px-3 py-1.5 rounded-xl text-sm font-semibold transition-colors ${txFilter === f ? 'bg-indigo-600 text-white' : 'bg-slate-100 text-slate-600 hover:bg-slate-200'}`}>
                {f === 'all' ? 'Semua' : f}
              </button>
            ))}
          </div>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-slate-50">
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Tanggal</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Keterangan</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Tipe</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Jumlah</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Status</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Ref</th>
              </tr>
            </thead>
            <tbody>
              {filteredTx.map(tx => {
                const sc = statusConfig[tx.status];
                const StatusIcon = sc.icon;
                return (
                  <tr key={tx.id} className="border-b hover:bg-slate-50 transition-colors">
                    <td className="px-4 py-3 text-slate-600">{tx.date}</td>
                    <td className="px-4 py-3 font-semibold text-slate-900">{tx.description}</td>
                    <td className="px-4 py-3"><Badge className={typeColor[tx.type]}>{tx.type}</Badge></td>
                    <td className="px-4 py-3 font-bold text-slate-900">{formatRp(tx.amount)}</td>
                    <td className="px-4 py-3">
                      <Badge className={`gap-1 ${sc.color}`}>
                        <StatusIcon className="w-3 h-3" />{sc.label}
                      </Badge>
                    </td>
                    <td className="px-4 py-3 text-xs text-muted-foreground font-mono">{tx.ref}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </Card>

      {/* Revenue Sharing / Payout */}
      <Card className="overflow-hidden">
        <div className="p-5 border-b">
          <h3 className="font-bold text-slate-900">Revenue Sharing Mitra</h3>
          <p className="text-sm text-muted-foreground">Payout bagi hasil sekolah mitra (25% dari revenue yang dibawa)</p>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-slate-50">
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Sekolah</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Periode</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Jumlah Payout</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Status</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Aksi</th>
              </tr>
            </thead>
            <tbody>
              {payoutList.map(p => {
                const pc = payoutStatusConfig[p.status];
                return (
                  <tr key={p.id} className="border-b hover:bg-slate-50 transition-colors">
                    <td className="px-4 py-3 font-semibold text-slate-900">{p.school}</td>
                    <td className="px-4 py-3 text-slate-600">{p.period}</td>
                    <td className="px-4 py-3 font-bold text-slate-900">{formatRp(p.amount)}</td>
                    <td className="px-4 py-3"><Badge className={pc.color}>{pc.label}</Badge></td>
                    <td className="px-4 py-3">
                      {p.status === 'pending' && (
                        <Button size="sm" className="bg-indigo-600 hover:bg-indigo-700 text-xs h-7 px-3" onClick={() => processPayout(p.id)}>
                          Proses Payout
                        </Button>
                      )}
                      {p.status === 'processing' && <span className="text-xs text-blue-600 font-semibold">Sedang diproses...</span>}
                      {p.status === 'paid' && <span className="text-xs text-emerald-600 font-semibold flex items-center gap-1"><CheckCircle2 className="w-3 h-3" />Selesai</span>}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </Card>
    </div>
  );
}
