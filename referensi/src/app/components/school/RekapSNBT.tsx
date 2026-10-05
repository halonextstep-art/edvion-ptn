import { useState, useMemo } from 'react';
import { Card } from '../ui/card';
import { Badge } from '../ui/badge';
import { toast } from 'sonner';
import {
  ClipboardList, Users, Medal, AlertCircle, Search, Plus, Download,
  Edit2, CheckCircle2, TrendingDown, MinusCircle, X, Save, TrendingUp,
  BookOpen, Target
} from 'lucide-react';

// ─── Types ────────────────────────────────────────────────────────────────────

interface SnbtRecord {
  id: string;
  studentName: string;
  class: string;
  year: number;
  terdaftarSNBT: boolean;
  pil1uni: string; pil1jur: string;
  pil2uni: string; pil2jur: string;
  estimasiSkor: number;
  skorAktual?: number;
  statusPenerimaan: 'belum-ujian' | 'sudah-ujian' | 'diterima' | 'tidak-diterima';
  tglUjian?: string;
  catatan?: string;
}

// ─── Constants ────────────────────────────────────────────────────────────────

const PTN_LIST = ['UI', 'ITB', 'UGM', 'UNAIR', 'IPB', 'UNPAD', 'UNDIP', 'ITS', 'UB', 'UNS', 'UNHAS', 'UPI'];

const JURUSAN_BY_PTN: Record<string, string[]> = {
  'UI': ['Teknik Informatika', 'Kedokteran', 'Psikologi', 'Hukum', 'Akuntansi', 'Teknik Sipil', 'Ilmu Komunikasi'],
  'ITB': ['Teknik Informatika', 'Teknik Sipil', 'Teknik Kimia', 'Matematika', 'Fisika', 'Teknik Mesin'],
  'UGM': ['Kedokteran', 'Psikologi', 'Teknik Informatika', 'Akuntansi', 'Ilmu Komunikasi', 'Farmasi', 'Hukum'],
  'UNPAD': ['Kedokteran', 'Farmasi', 'Psikologi', 'Hukum', 'Ilmu Komunikasi', 'Akuntansi'],
  'UNAIR': ['Kedokteran', 'Farmasi', 'Hukum', 'Akuntansi', 'Psikologi', 'Teknik Informatika'],
  'UNDIP': ['Teknik Informatika', 'Hukum', 'Akuntansi', 'Kedokteran', 'Teknik Sipil', 'Psikologi'],
  'ITS': ['Teknik Informatika', 'Teknik Elektro', 'Teknik Sipil', 'Statistika', 'Teknik Kimia'],
  'IPB': ['Kedokteran Hewan', 'Teknologi Pangan', 'Agribisnis', 'Biologi', 'Matematika'],
  'UB': ['Teknik Informatika', 'Hukum', 'Akuntansi', 'Kedokteran', 'Psikologi'],
  'UNS': ['Kedokteran', 'Hukum', 'Teknik Informatika', 'Psikologi', 'Akuntansi'],
  'UNHAS': ['Kedokteran', 'Teknik Informatika', 'Hukum', 'Psikologi', 'Farmasi'],
  'UPI': ['Pendidikan Matematika', 'Pendidikan B. Indonesia', 'Psikologi Pendidikan', 'Bimbingan Konseling'],
};

// PM UTBK per prodi (minimum passing score dari alumni)
const PM_UTBK: { uni: string; jurusan: string; minUTBK: number; kuota: number; peminat: number }[] = [
  { uni: 'UGM', jurusan: 'Psikologi', minUTBK: 670, kuota: 68, peminat: 4022 },
  { uni: 'UI', jurusan: 'Teknik Informatika', minUTBK: 720, kuota: 45, peminat: 3800 },
  { uni: 'ITB', jurusan: 'Teknik Informatika', minUTBK: 730, kuota: 40, peminat: 4200 },
  { uni: 'UGM', jurusan: 'Kedokteran', minUTBK: 750, kuota: 30, peminat: 5100 },
  { uni: 'UNPAD', jurusan: 'Kedokteran', minUTBK: 720, kuota: 35, peminat: 4800 },
  { uni: 'UI', jurusan: 'Akuntansi', minUTBK: 690, kuota: 50, peminat: 3200 },
  { uni: 'UNDIP', jurusan: 'Psikologi', minUTBK: 640, kuota: 55, peminat: 2400 },
  { uni: 'UNDIP', jurusan: 'Akuntansi', minUTBK: 635, kuota: 60, peminat: 2600 },
  { uni: 'ITS', jurusan: 'Teknik Elektro', minUTBK: 660, kuota: 48, peminat: 2100 },
  { uni: 'UNHAS', jurusan: 'Kedokteran', minUTBK: 680, kuota: 40, peminat: 3200 },
  { uni: 'UNS', jurusan: 'Kedokteran', minUTBK: 685, kuota: 38, peminat: 3500 },
];

// Alumni benchmark (dari Kaka Kelas)
const ALUMNI_BENCHMARK = [
  { name: 'Dewi Kusuma Wardhani', uni: 'UGM', jurusan: 'Psikologi', year: 2024, pmUTBK: 674 },
  { name: 'Rizky Aditya Nugraha', uni: 'UI', jurusan: 'Teknik Informatika', year: 2023, pmUTBK: 720 },
  { name: 'Putri Handayani Saputri', uni: 'UNPAD', jurusan: 'Kedokteran', year: 2024, pmUTBK: 710 },
  { name: 'Eko Prasetyo Wibowo', uni: 'UNDIP', jurusan: 'Teknik Informatika', year: 2023, pmUTBK: 645 },
  { name: 'Fajar Nugraha Putra', uni: 'ITS', jurusan: 'Teknik Elektro', year: 2024, pmUTBK: 658 },
  { name: 'Lina Susanti Rahayu', uni: 'UGM', jurusan: 'Akuntansi', year: 2023, pmUTBK: 680 },
];

// ─── Seed Data ────────────────────────────────────────────────────────────────

const INIT_RECORDS: SnbtRecord[] = [
  {
    id: 'sb1', studentName: 'Ahmad Fauzi', class: 'XII IPA 1', year: 2026,
    terdaftarSNBT: true, pil1uni: 'UI', pil1jur: 'Teknik Informatika',
    pil2uni: 'ITB', pil2jur: 'Teknik Informatika',
    estimasiSkor: 721, skorAktual: 724,
    statusPenerimaan: 'tidak-diterima', tglUjian: '2026-05-13',
    catatan: 'Skor sudah masuk, menunggu pengumuman SNBT',
  },
  {
    id: 'sb2', studentName: 'Sari Dewi Anggraeni', class: 'XII IPA 2', year: 2026,
    terdaftarSNBT: true, pil1uni: 'UGM', pil1jur: 'Kedokteran',
    pil2uni: 'UNPAD', pil2jur: 'Kedokteran',
    estimasiSkor: 695, skorAktual: 701,
    statusPenerimaan: 'sudah-ujian', tglUjian: '2026-05-13',
  },
  {
    id: 'sb3', studentName: 'Budi Santoso', class: 'XII IPS 1', year: 2026,
    terdaftarSNBT: true, pil1uni: 'UI', pil1jur: 'Akuntansi',
    pil2uni: 'UNDIP', pil2jur: 'Akuntansi',
    estimasiSkor: 672, skorAktual: 681,
    statusPenerimaan: 'diterima', tglUjian: '2026-05-13',
  },
  {
    id: 'sb4', studentName: 'Gilang Pratama', class: 'XII IPA 2', year: 2026,
    terdaftarSNBT: true, pil1uni: 'UGM', pil1jur: 'Kedokteran',
    pil2uni: 'UNHAS', pil2jur: 'Kedokteran',
    estimasiSkor: 645, statusPenerimaan: 'belum-ujian',
  },
  {
    id: 'sb5', studentName: 'Tijanil Ulfa', class: 'XII IPA 1', year: 2026,
    terdaftarSNBT: false, pil1uni: 'UGM', pil1jur: 'Psikologi',
    pil2uni: 'UNDIP', pil2jur: 'Psikologi',
    estimasiSkor: 682, statusPenerimaan: 'belum-ujian',
    catatan: 'Sudah diterima SNBP — tidak mendaftar SNBT',
  },
  {
    id: 'sb6', studentName: 'Rina Marlena', class: 'XI IPA 1', year: 2027,
    terdaftarSNBT: false, pil1uni: 'ITS', pil1jur: 'Teknik Elektro',
    pil2uni: 'UNDIP', pil2jur: 'Teknik Sipil',
    estimasiSkor: 658, statusPenerimaan: 'belum-ujian',
  },
];

// ─── Helpers ──────────────────────────────────────────────────────────────────

function statusInfo(s: SnbtRecord['statusPenerimaan']): { label: string; cls: string } {
  if (s === 'diterima') return { label: 'Diterima SNBT', cls: 'bg-emerald-100 text-emerald-700' };
  if (s === 'tidak-diterima') return { label: 'Tidak Diterima', cls: 'bg-red-100 text-red-700' };
  if (s === 'sudah-ujian') return { label: 'Sudah Ujian', cls: 'bg-blue-100 text-blue-700' };
  return { label: 'Belum Ujian', cls: 'bg-slate-100 text-slate-500' };
}

function getPM(uni: string, jur: string) {
  return PM_UTBK.find(p => p.uni === uni && p.jurusan === jur);
}

// ─── Main Component ───────────────────────────────────────────────────────────

export default function RekapSNBT() {
  const [records, setRecords] = useState<SnbtRecord[]>(INIT_RECORDS);
  const [search, setSearch] = useState('');
  const [filterStatus, setFilterStatus] = useState('all');
  const [filterYear, setFilterYear] = useState('2026');
  const [editId, setEditId] = useState<string | null>(null);
  const [editForm, setEditForm] = useState<Partial<SnbtRecord>>({});
  const [addOpen, setAddOpen] = useState(false);
  const [addForm, setAddForm] = useState<Partial<SnbtRecord>>({
    studentName: '', class: '', year: 2026, terdaftarSNBT: true,
    pil1uni: '', pil1jur: '', pil2uni: '', pil2jur: '',
    estimasiSkor: 0, statusPenerimaan: 'belum-ujian',
  });

  const filtered = useMemo(() => records.filter(r => {
    const q = search.toLowerCase();
    const match = r.studentName.toLowerCase().includes(q)
      || r.pil1uni.toLowerCase().includes(q)
      || r.pil1jur.toLowerCase().includes(q);
    const matchStatus = filterStatus === 'all' || r.statusPenerimaan === filterStatus;
    const matchYear = filterYear === 'all' || String(r.year) === filterYear;
    return match && matchStatus && matchYear;
  }), [records, search, filterStatus, filterYear]);

  const stats = useMemo(() => ({
    terdaftar: records.filter(r => r.terdaftarSNBT).length,
    sudahUjian: records.filter(r => r.skorAktual !== undefined).length,
    diterima: records.filter(r => r.statusPenerimaan === 'diterima').length,
    perluUpdate: records.filter(r => r.terdaftarSNBT && r.skorAktual === undefined && r.tglUjian).length,
  }), [records]);

  const openEdit = (r: SnbtRecord) => { setEditId(r.id); setEditForm({ ...r }); };

  const saveEdit = () => {
    if (!editId) return;
    setRecords(prev => prev.map(r => r.id === editId ? { ...r, ...editForm } : r));
    setEditId(null);
    toast.success('Data SNBT diperbarui');
  };

  const handleAdd = () => {
    if (!addForm.studentName) { toast.error('Nama siswa wajib diisi'); return; }
    setRecords(prev => [...prev, { id: `sb${Date.now()}`, ...addForm } as SnbtRecord]);
    setAddOpen(false);
    setAddForm({ studentName: '', class: '', year: 2026, terdaftarSNBT: true, pil1uni: '', pil1jur: '', pil2uni: '', pil2jur: '', estimasiSkor: 0, statusPenerimaan: 'belum-ujian' });
    toast.success('Siswa ditambahkan ke monitoring SNBT');
  };

  const toggleTerdaftar = (id: string) => {
    setRecords(prev => prev.map(r => {
      if (r.id !== id) return r;
      toast.success(r.terdaftarSNBT ? 'Dinonaktifkan dari SNBT' : 'Diaktifkan untuk SNBT');
      return { ...r, terdaftarSNBT: !r.terdaftarSNBT };
    }));
  };

  const setEF = <K extends keyof SnbtRecord>(k: K, v: SnbtRecord[K]) =>
    setEditForm(f => ({ ...f, [k]: v }));
  const setAF = <K extends keyof SnbtRecord>(k: K, v: SnbtRecord[K]) =>
    setAddForm(f => ({ ...f, [k]: v }));

  const gapScore = (r: SnbtRecord) => {
    const pm = getPM(r.pil1uni, r.pil1jur);
    if (!pm || r.skorAktual === undefined) return null;
    return r.skorAktual - pm.minUTBK;
  };

  const editPM = editForm.pil1uni && editForm.pil1jur ? getPM(editForm.pil1uni, editForm.pil1jur) : null;
  const editGap = editPM && editForm.skorAktual ? editForm.skorAktual - editPM.minUTBK : null;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-cyan-700 via-blue-700 to-indigo-700 p-6 text-white">
        <div className="absolute inset-0 opacity-10" style={{ backgroundImage: 'radial-gradient(circle at 80% 50%, white 1px, transparent 1px)', backgroundSize: '28px 28px' }} />
        <div className="relative flex items-start justify-between flex-wrap gap-4">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <ClipboardList className="w-5 h-5 text-white/70" />
              <span className="text-white/70 text-sm font-semibold uppercase tracking-wide">Monitoring Jalur Tulis</span>
            </div>
            <h2 className="text-2xl font-black">Rekap SNBT</h2>
            <p className="text-white/70 text-sm mt-1">
              Pantau pendaftaran, skor aktual, dan status penerimaan siswa via SNBT
            </p>
          </div>
          <div className="flex items-center gap-5 hidden sm:flex">
            {[
              { label: 'Terdaftar', val: stats.terdaftar },
              { label: 'Sudah Ujian', val: stats.sudahUjian },
              { label: 'Diterima', val: stats.diterima },
            ].map(({ label, val }) => (
              <div key={label} className="text-center">
                <p className="text-2xl font-black">{val}</p>
                <p className="text-white/60 text-xs">{label}</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Stats cards */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
        {[
          { label: 'Terdaftar SNBT', val: stats.terdaftar, icon: ClipboardList, txt: 'text-indigo-600', bg: 'bg-indigo-50', border: 'border-indigo-200', ib: 'bg-indigo-100' },
          { label: 'Sudah Ujian', val: stats.sudahUjian, icon: BookOpen, txt: 'text-blue-600', bg: 'bg-blue-50', border: 'border-blue-200', ib: 'bg-blue-100' },
          { label: 'Diterima via SNBT', val: stats.diterima, icon: Medal, txt: 'text-emerald-600', bg: 'bg-emerald-50', border: 'border-emerald-200', ib: 'bg-emerald-100' },
          { label: 'Skor Perlu Diisi', val: stats.perluUpdate, icon: AlertCircle, txt: 'text-amber-600', bg: 'bg-amber-50', border: 'border-amber-200', ib: 'bg-amber-100' },
        ].map(({ label, val, icon: Icon, txt, bg, border, ib }) => (
          <Card key={label} className={`p-4 ${bg} ${border} border`}>
            <div className="flex items-start justify-between">
              <div>
                <p className="text-xs text-slate-500 mb-1 leading-tight">{label}</p>
                <p className={`text-3xl font-black ${txt}`}>{val}</p>
              </div>
              <div className={`w-10 h-10 rounded-xl ${ib} flex items-center justify-center shrink-0`}>
                <Icon className={`w-5 h-5 ${txt}`} />
              </div>
            </div>
          </Card>
        ))}
      </div>

      {/* Alert: ada skor belum diisi */}
      {stats.perluUpdate > 0 && (
        <div className="flex items-start gap-3 p-4 bg-amber-50 border border-amber-300 rounded-xl">
          <AlertCircle className="w-5 h-5 text-amber-600 shrink-0 mt-0.5" />
          <div>
            <p className="text-sm font-bold text-amber-800">
              {stats.perluUpdate} siswa sudah memiliki tanggal ujian tapi skor aktual belum diinput
            </p>
            <p className="text-xs text-amber-600 mt-0.5">
              Klik ikon edit pada baris siswa untuk memasukkan skor SNBT aktual mereka
            </p>
          </div>
        </div>
      )}

      {/* Toolbar */}
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="flex-1 relative">
          <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
          <input
            className="w-full pl-9 pr-4 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300"
            placeholder="Cari nama, universitas, atau prodi..."
            value={search} onChange={e => setSearch(e.target.value)}
          />
        </div>
        <select className="border rounded-xl px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterYear} onChange={e => setFilterYear(e.target.value)}>
          {['all', '2025', '2026', '2027'].map(y => <option key={y} value={y}>{y === 'all' ? 'Semua Tahun' : y}</option>)}
        </select>
        <select className="border rounded-xl px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterStatus} onChange={e => setFilterStatus(e.target.value)}>
          <option value="all">Semua Status</option>
          <option value="belum-ujian">Belum Ujian</option>
          <option value="sudah-ujian">Sudah Ujian</option>
          <option value="diterima">Diterima</option>
          <option value="tidak-diterima">Tidak Diterima</option>
        </select>
        <button onClick={() => toast.info('Export CSV disiapkan...')} className="flex items-center gap-2 px-4 py-2.5 border rounded-xl text-sm font-semibold hover:bg-slate-50 transition-colors shrink-0">
          <Download className="w-4 h-4" /> Export
        </button>
        <button onClick={() => setAddOpen(true)} className="flex items-center gap-2 px-4 py-2.5 bg-cyan-600 hover:bg-cyan-700 text-white rounded-xl text-sm font-bold transition-colors shrink-0">
          <Plus className="w-4 h-4" /> Tambah Siswa
        </button>
      </div>

      {/* Main table */}
      <Card className="overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-slate-50">
                {['Siswa', 'Terdaftar', 'Pilihan 1 via SNBT', 'Pilihan 2 via SNBT', 'Est. Skor', 'Skor Aktual', 'vs PM UTBK', 'Tgl Ujian', 'Status', 'Aksi'].map(h => (
                  <th key={h} className="text-left px-4 py-3 font-semibold text-slate-600 whitespace-nowrap text-xs">{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {filtered.length === 0 ? (
                <tr>
                  <td colSpan={10} className="px-4 py-12 text-center">
                    <ClipboardList className="w-10 h-10 text-slate-200 mx-auto mb-2" />
                    <p className="text-slate-400 text-sm font-semibold">Tidak ada data siswa SNBT</p>
                  </td>
                </tr>
              ) : filtered.map(r => {
                const gap = gapScore(r);
                const si = statusInfo(r.statusPenerimaan);
                const pm = getPM(r.pil1uni, r.pil1jur);
                return (
                  <tr key={r.id} className="border-b hover:bg-slate-50 transition-colors">
                    {/* Siswa */}
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2.5">
                        <div className="w-8 h-8 rounded-full bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center text-white text-xs font-bold shrink-0">
                          {r.studentName.split(' ').slice(0, 2).map(w => w[0]).join('')}
                        </div>
                        <div>
                          <p className="font-semibold text-slate-900 whitespace-nowrap">{r.studentName}</p>
                          <p className="text-xs text-slate-400">{r.class} · {r.year}</p>
                        </div>
                      </div>
                    </td>
                    {/* Terdaftar toggle */}
                    <td className="px-4 py-3">
                      <button onClick={() => toggleTerdaftar(r.id)} className={`w-10 h-5 rounded-full transition-colors relative ${r.terdaftarSNBT ? 'bg-emerald-500' : 'bg-slate-200'}`}>
                        <div className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-all ${r.terdaftarSNBT ? 'left-5' : 'left-0.5'}`} />
                      </button>
                    </td>
                    {/* Pil 1 */}
                    <td className="px-4 py-3">
                      <p className="font-semibold text-indigo-700 text-xs">{r.pil1uni || '—'}</p>
                      <p className="text-xs text-slate-400">{r.pil1jur || '—'}</p>
                    </td>
                    {/* Pil 2 */}
                    <td className="px-4 py-3">
                      <p className="font-semibold text-purple-700 text-xs">{r.pil2uni || '—'}</p>
                      <p className="text-xs text-slate-400">{r.pil2jur || '—'}</p>
                    </td>
                    {/* Estimasi skor */}
                    <td className="px-4 py-3">
                      <span className="font-bold text-slate-600">{r.estimasiSkor || '—'}</span>
                      <p className="text-[10px] text-slate-400">dari tryout</p>
                    </td>
                    {/* Skor aktual */}
                    <td className="px-4 py-3">
                      {r.skorAktual !== undefined
                        ? <span className="font-black text-slate-900 text-base">{r.skorAktual}</span>
                        : <span className="text-slate-300 text-xs italic">Belum diisi</span>}
                    </td>
                    {/* vs PM UTBK */}
                    <td className="px-4 py-3">
                      {gap !== null ? (
                        <div className="flex items-center gap-1.5">
                          {gap >= 0
                            ? <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0" />
                            : <TrendingDown className="w-4 h-4 text-red-400 shrink-0" />}
                          <span className={`font-black text-sm ${gap >= 0 ? 'text-emerald-600' : 'text-red-500'}`}>
                            {gap >= 0 ? `+${gap}` : gap}
                          </span>
                          {pm && <span className="text-[10px] text-slate-400">/ {pm.minUTBK}</span>}
                        </div>
                      ) : (
                        <div className="flex items-center gap-1.5">
                          <MinusCircle className="w-4 h-4 text-slate-300" />
                          {pm && <span className="text-xs text-slate-400">min {pm.minUTBK}</span>}
                        </div>
                      )}
                    </td>
                    {/* Tanggal ujian */}
                    <td className="px-4 py-3 text-xs text-slate-500 whitespace-nowrap">
                      {r.tglUjian ?? '—'}
                    </td>
                    {/* Status */}
                    <td className="px-4 py-3">
                      <Badge className={`${si.cls} text-xs whitespace-nowrap`}>{si.label}</Badge>
                    </td>
                    {/* Aksi */}
                    <td className="px-4 py-3">
                      <button onClick={() => openEdit(r)} className="p-1.5 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-cyan-600 transition-colors">
                        <Edit2 className="w-4 h-4" />
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
        <div className="px-4 py-3 bg-slate-50 border-t text-xs text-slate-400">
          Menampilkan {filtered.length} dari {records.length} siswa
        </div>
      </Card>

      {/* PM UTBK Reference panel */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Alumni benchmark */}
        <Card className="p-5">
          <div className="flex items-center gap-2 mb-4">
            <TrendingUp className="w-4 h-4 text-indigo-600" />
            <h4 className="font-bold text-slate-800 text-sm">PM UTBK Kaka Kelas (Alumni)</h4>
          </div>
          <div className="space-y-2">
            {ALUMNI_BENCHMARK.map(a => (
              <div key={a.name} className="flex items-center justify-between p-2.5 bg-slate-50 rounded-xl">
                <div>
                  <p className="text-xs font-bold text-slate-700">{a.uni} — {a.jurusan}</p>
                  <p className="text-[10px] text-slate-400">{a.name} · {a.year}</p>
                </div>
                <span className="font-black text-indigo-600 text-base">{a.pmUTBK}</span>
              </div>
            ))}
          </div>
        </Card>

        {/* Persaingan SNBT per prodi */}
        <Card className="p-5">
          <div className="flex items-center gap-2 mb-4">
            <Target className="w-4 h-4 text-cyan-600" />
            <h4 className="font-bold text-slate-800 text-sm">Tingkat Persaingan per Prodi</h4>
          </div>
          <div className="space-y-2">
            {PM_UTBK.filter(p =>
              records.some(r => r.pil1uni === p.uni && r.pil1jur === p.jurusan)
            ).map(p => {
              const ratio = Math.round(p.peminat / p.kuota);
              const isHard = ratio > 80;
              return (
                <div key={`${p.uni}-${p.jurusan}`} className="flex items-center gap-3 p-2.5 bg-slate-50 rounded-xl">
                  <div className="flex-1 min-w-0">
                    <p className="text-xs font-bold text-slate-700 truncate">{p.uni} — {p.jurusan}</p>
                    <p className="text-[10px] text-slate-400">Kuota {p.kuota} · Peminat {p.peminat.toLocaleString()}</p>
                  </div>
                  <div className="text-right shrink-0">
                    <p className={`text-xs font-black ${isHard ? 'text-red-500' : 'text-amber-600'}`}>1:{ratio}</p>
                    <p className="text-[10px] text-slate-400">persaingan</p>
                  </div>
                </div>
              );
            })}
          </div>
        </Card>
      </div>

      {/* ─── Edit Modal ─── */}
      {editId && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4 overflow-y-auto">
          <div className="bg-white rounded-2xl w-full max-w-md shadow-2xl my-4 overflow-hidden">
            <div className="bg-gradient-to-r from-cyan-600 to-blue-600 px-6 py-5 text-white flex items-center justify-between">
              <div>
                <h3 className="font-black text-lg">Edit Data SNBT</h3>
                <p className="text-white/70 text-sm">{editForm.studentName}</p>
              </div>
              <button onClick={() => setEditId(null)} className="p-2 rounded-xl hover:bg-white/20 transition-colors"><X className="w-5 h-5" /></button>
            </div>

            <div className="p-6 space-y-4 max-h-[65vh] overflow-y-auto">
              {/* Pilihan PTN SNBT */}
              {(['1', '2'] as const).map(n => (
                <div key={n} className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Universitas Pil. {n}</label>
                    <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-300" value={n === '1' ? (editForm.pil1uni ?? '') : (editForm.pil2uni ?? '')} onChange={e => { if (n === '1') { setEF('pil1uni', e.target.value); setEF('pil1jur', '' as string); } else { setEF('pil2uni', e.target.value); setEF('pil2jur', '' as string); } }}>
                      <option value="">Pilih PTN</option>
                      {PTN_LIST.map(p => <option key={p} value={p}>{p}</option>)}
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Prodi Pil. {n}</label>
                    <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-300" value={n === '1' ? (editForm.pil1jur ?? '') : (editForm.pil2jur ?? '')} onChange={e => n === '1' ? setEF('pil1jur', e.target.value) : setEF('pil2jur', e.target.value)}>
                      <option value="">Pilih Prodi</option>
                      {(JURUSAN_BY_PTN[n === '1' ? (editForm.pil1uni ?? '') : (editForm.pil2uni ?? '')] ?? []).map(j => <option key={j} value={j}>{j}</option>)}
                    </select>
                  </div>
                </div>
              ))}

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Skor Aktual SNBT</label>
                  <input
                    type="number" min={0} max={1000}
                    className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-300 font-bold"
                    value={editForm.skorAktual ?? ''}
                    onChange={e => setEF('skorAktual', Number(e.target.value) || undefined as unknown as number)}
                    placeholder="cth: 698"
                  />
                </div>
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Tanggal Ujian</label>
                  <input type="date" className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-300" value={editForm.tglUjian ?? ''} onChange={e => setEF('tglUjian', e.target.value)} />
                </div>
              </div>

              {/* Real-time gap indicator */}
              {editGap !== null && editPM && (
                <div className={`flex items-center gap-3 p-3 rounded-xl ${editGap >= 0 ? 'bg-emerald-50 border border-emerald-200' : 'bg-red-50 border border-red-200'}`}>
                  {editGap >= 0
                    ? <CheckCircle2 className="w-5 h-5 text-emerald-600 shrink-0" />
                    : <TrendingDown className="w-5 h-5 text-red-500 shrink-0" />}
                  <div>
                    <p className={`text-sm font-bold ${editGap >= 0 ? 'text-emerald-800' : 'text-red-700'}`}>
                      {editGap >= 0 ? `Di atas minimum (+${editGap})` : `Di bawah minimum (${editGap})`}
                    </p>
                    <p className={`text-xs ${editGap >= 0 ? 'text-emerald-600' : 'text-red-500'}`}>
                      PM UTBK {editForm.pil1uni} {editForm.pil1jur}: {editPM.minUTBK}
                    </p>
                  </div>
                </div>
              )}

              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Status Penerimaan</label>
                <div className="grid grid-cols-2 gap-2">
                  {(['belum-ujian', 'sudah-ujian', 'diterima', 'tidak-diterima'] as const).map(st => {
                    const si = statusInfo(st);
                    return (
                      <button key={st} onClick={() => setEF('statusPenerimaan', st)}
                        className={`py-2 px-3 rounded-xl text-xs font-bold border transition-colors text-left ${editForm.statusPenerimaan === st ? `${si.cls} border-current` : 'border-slate-200 text-slate-400 hover:bg-slate-50'}`}>
                        {si.label}
                      </button>
                    );
                  })}
                </div>
              </div>

              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Catatan</label>
                <textarea rows={2} className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-300 resize-none" value={editForm.catatan ?? ''} onChange={e => setEF('catatan', e.target.value)} placeholder="Catatan tambahan untuk siswa ini..." />
              </div>
            </div>

            <div className="flex gap-3 p-6 border-t bg-slate-50">
              <button onClick={() => setEditId(null)} className="flex-1 py-2.5 rounded-xl border text-sm font-semibold hover:bg-white transition-colors">Batal</button>
              <button onClick={saveEdit} className="flex-1 py-2.5 rounded-xl bg-cyan-600 hover:bg-cyan-700 text-white text-sm font-black transition-colors flex items-center justify-center gap-2">
                <Save className="w-4 h-4" /> Simpan
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ─── Add Modal ─── */}
      {addOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4 overflow-y-auto">
          <div className="bg-white rounded-2xl w-full max-w-md shadow-2xl my-4 overflow-hidden">
            <div className="flex items-center justify-between p-6 border-b">
              <h3 className="font-bold text-slate-900">Tambah Siswa ke Rekap SNBT</h3>
              <button onClick={() => setAddOpen(false)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-6 space-y-4 max-h-[60vh] overflow-y-auto">
              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Nama Siswa *</label>
                <input className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-300" value={addForm.studentName ?? ''} onChange={e => setAF('studentName', e.target.value)} placeholder="Nama lengkap siswa" />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Kelas</label>
                  <input className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-300" value={addForm.class ?? ''} onChange={e => setAF('class', e.target.value)} placeholder="XII IPA 1" />
                </div>
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Tahun</label>
                  <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-300" value={addForm.year} onChange={e => setAF('year', Number(e.target.value))}>
                    {[2025, 2026, 2027].map(y => <option key={y} value={y}>{y}</option>)}
                  </select>
                </div>
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Est. Skor Tryout</label>
                  <input type="number" className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-300" value={addForm.estimasiSkor || ''} onChange={e => setAF('estimasiSkor', Number(e.target.value))} placeholder="680" />
                </div>
              </div>

              {(['1', '2'] as const).map(n => (
                <div key={n} className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Universitas Pil. {n}</label>
                    <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-300" value={n === '1' ? (addForm.pil1uni ?? '') : (addForm.pil2uni ?? '')} onChange={e => { if (n === '1') { setAF('pil1uni', e.target.value); setAF('pil1jur', '' as string); } else { setAF('pil2uni', e.target.value); setAF('pil2jur', '' as string); } }}>
                      <option value="">Pilih PTN</option>
                      {PTN_LIST.map(p => <option key={p} value={p}>{p}</option>)}
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Prodi Pil. {n}</label>
                    <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-cyan-300" value={n === '1' ? (addForm.pil1jur ?? '') : (addForm.pil2jur ?? '')} onChange={e => n === '1' ? setAF('pil1jur', e.target.value) : setAF('pil2jur', e.target.value)}>
                      <option value="">Pilih Prodi</option>
                      {(JURUSAN_BY_PTN[n === '1' ? (addForm.pil1uni ?? '') : (addForm.pil2uni ?? '')] ?? []).map(j => <option key={j} value={j}>{j}</option>)}
                    </select>
                  </div>
                </div>
              ))}
            </div>
            <div className="flex gap-3 p-6 border-t bg-slate-50">
              <button onClick={() => setAddOpen(false)} className="flex-1 py-2.5 rounded-xl border text-sm font-semibold">Batal</button>
              <button onClick={handleAdd} className="flex-1 py-2.5 rounded-xl bg-cyan-600 hover:bg-cyan-700 text-white text-sm font-bold transition-colors">Tambah</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
