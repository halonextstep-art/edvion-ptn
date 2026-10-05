import { useState, useMemo } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Badge } from '../ui/badge';
import {
  Users, Plus, Search, Edit, Trash2, Eye, X, Save,
  ChevronDown, ChevronUp, Mail, Phone, BookOpen, Target,
  TrendingUp, AlertCircle, Download, UserCheck, UserX, GraduationCap,
  Upload, RefreshCw, Key, Copy, CheckCircle2, Send
} from 'lucide-react';
import { toast } from 'sonner';

const CSV_TEMPLATE = `NISN,Nama Lengkap,Kelas,Email
0045678909,Ahmad Fauzi,XII IPA 1,ahmad@siswa.sch.id
0045678910,Rina Marlena,XI IPA 2,rina@siswa.sch.id`;

interface BulkRow {
  nisn: string;
  name: string;
  class: string;
  email: string;
  tempPassword: string;
  status: 'ready' | 'error';
  error?: string;
}

function genPassword(nisn: string) {
  return `Gspl${nisn.slice(-4)}!`;
}

function parseCSV(raw: string): BulkRow[] {
  const lines = raw.trim().split('\n').filter(l => l.trim());
  const dataLines = lines[0].toLowerCase().includes('nisn') ? lines.slice(1) : lines;
  return dataLines.map((line, i) => {
    const parts = line.split(',').map(p => p.trim());
    const nisn = parts[0] || '';
    const name = parts[1] || '';
    const cls = parts[2] || '';
    const email = parts[3] || '';
    const errors: string[] = [];
    if (!nisn || nisn.length < 8) errors.push('NISN tidak valid');
    if (!name) errors.push('Nama kosong');
    return { nisn, name, class: cls, email, tempPassword: genPassword(nisn || String(i)), status: errors.length ? 'error' : 'ready', error: errors.join(', ') } as BulkRow;
  });
}

interface Student {
  id: string;
  name: string;
  nisn: string;
  class: string;
  grade: 10 | 11 | 12;
  email: string;
  phone: string;
  ptnTarget: string;
  majorTarget: string;
  avgScore: number;
  totalTryouts: number;
  lastActive: string;
  status: 'active' | 'inactive';
  joinDate: string;
}

const initialStudents: Student[] = [
  { id: 's1', name: 'Andi Pratama', nisn: '0045678901', class: 'XII IPA 1', grade: 12, email: 'andi@student.sch.id', phone: '081234567890', ptnTarget: 'UI', majorTarget: 'Teknik Informatika', avgScore: 721, totalTryouts: 14, lastActive: '2025-01-15', status: 'active', joinDate: '2024-08-01' },
  { id: 's2', name: 'Bunga Rahayu', nisn: '0045678902', class: 'XII IPA 1', grade: 12, email: 'bunga@student.sch.id', phone: '081234567891', ptnTarget: 'UGM', majorTarget: 'Kedokteran', avgScore: 745, totalTryouts: 18, lastActive: '2025-01-14', status: 'active', joinDate: '2024-08-01' },
  { id: 's3', name: 'Cahyo Nugroho', nisn: '0045678903', class: 'XII IPA 2', grade: 12, email: 'cahyo@student.sch.id', phone: '081234567892', ptnTarget: 'ITB', majorTarget: 'Teknik Sipil', avgScore: 698, totalTryouts: 10, lastActive: '2025-01-10', status: 'active', joinDate: '2024-08-01' },
  { id: 's4', name: 'Dewi Kusuma', nisn: '0045678904', class: 'XI IPA 1', grade: 11, email: 'dewi@student.sch.id', phone: '081234567893', ptnTarget: 'UNPAD', majorTarget: 'Farmasi', avgScore: 664, totalTryouts: 7, lastActive: '2025-01-12', status: 'active', joinDate: '2024-08-01' },
  { id: 's5', name: 'Edo Santoso', nisn: '0045678905', class: 'XI IPA 2', grade: 11, email: 'edo@student.sch.id', phone: '081234567894', ptnTarget: 'UNAIR', majorTarget: 'Hukum', avgScore: 0, totalTryouts: 0, lastActive: '-', status: 'inactive', joinDate: '2024-09-01' },
  { id: 's6', name: 'Fani Lestari', nisn: '0045678906', class: 'XII IPS 1', grade: 12, email: 'fani@student.sch.id', phone: '081234567895', ptnTarget: 'UI', majorTarget: 'Akuntansi', avgScore: 688, totalTryouts: 11, lastActive: '2025-01-13', status: 'active', joinDate: '2024-08-01' },
  { id: 's7', name: 'Gilang Putra', nisn: '0045678907', class: 'X IPA 1', grade: 10, email: 'gilang@student.sch.id', phone: '081234567896', ptnTarget: 'ITB', majorTarget: 'Teknik Mesin', avgScore: 643, totalTryouts: 4, lastActive: '2025-01-08', status: 'active', joinDate: '2024-08-01' },
  { id: 's8', name: 'Hana Wijaya', nisn: '0045678908', class: 'XII IPA 2', grade: 12, email: 'hana@student.sch.id', phone: '081234567897', ptnTarget: 'UGM', majorTarget: 'Psikologi', avgScore: 710, totalTryouts: 15, lastActive: '2025-01-15', status: 'active', joinDate: '2024-08-01' },
];

const classOptions = ['X IPA 1', 'X IPA 2', 'X IPS 1', 'X IPS 2', 'XI IPA 1', 'XI IPA 2', 'XI IPS 1', 'XI IPS 2', 'XII IPA 1', 'XII IPA 2', 'XII IPS 1', 'XII IPS 2'];
const ptnOptions = ['UI', 'ITB', 'UGM', 'IPB', 'UNAIR', 'UNPAD', 'UNDIP', 'ITS', 'UB', 'UNS', 'UNHAS'];

const emptyForm = { name: '', nisn: '', class: 'XII IPA 1', grade: 12 as const, email: '', phone: '', ptnTarget: '', majorTarget: '', status: 'active' as const };

function gradeBadge(avg: number) {
  if (avg === 0) return { label: 'Belum Tes', color: 'bg-slate-100 text-slate-500' };
  if (avg >= 700) return { label: 'Unggul', color: 'bg-emerald-100 text-emerald-700' };
  if (avg >= 650) return { label: 'Baik', color: 'bg-blue-100 text-blue-700' };
  if (avg >= 600) return { label: 'Cukup', color: 'bg-amber-100 text-amber-700' };
  return { label: 'Perlu Bimbingan', color: 'bg-red-100 text-red-700' };
}

export default function StudentManagement() {
  const [students, setStudents] = useState<Student[]>(initialStudents);
  const [search, setSearch] = useState('');
  const [filterGrade, setFilterGrade] = useState('all');
  const [filterStatus, setFilterStatus] = useState('all');
  const [sortBy, setSortBy] = useState<'name' | 'score' | 'tryouts'>('name');
  const [sortDir, setSortDir] = useState<'asc' | 'desc'>('asc');
  const [modalOpen, setModalOpen] = useState(false);
  const [editStudent, setEditStudent] = useState<Student | null>(null);
  const [form, setForm] = useState(emptyForm);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [detailStudent, setDetailStudent] = useState<Student | null>(null);
  const [bulkOpen, setBulkOpen] = useState(false);
  const [bulkCSV, setBulkCSV] = useState('');
  const [bulkRows, setBulkRows] = useState<BulkRow[]>([]);
  const [bulkStep, setBulkStep] = useState<1 | 2>(1);

  const filtered = useMemo(() => {
    let list = students.filter(s => {
      const q = search.toLowerCase();
      const match = s.name.toLowerCase().includes(q) || s.nisn.includes(q) || s.class.toLowerCase().includes(q) || s.ptnTarget.toLowerCase().includes(q);
      const matchGrade = filterGrade === 'all' || String(s.grade) === filterGrade;
      const matchStatus = filterStatus === 'all' || s.status === filterStatus;
      return match && matchGrade && matchStatus;
    });
    list.sort((a, b) => {
      let diff = 0;
      if (sortBy === 'name') diff = a.name.localeCompare(b.name);
      else if (sortBy === 'score') diff = a.avgScore - b.avgScore;
      else diff = a.totalTryouts - b.totalTryouts;
      return sortDir === 'asc' ? diff : -diff;
    });
    return list;
  }, [students, search, filterGrade, filterStatus, sortBy, sortDir]);

  const stats = useMemo(() => {
    const active = students.filter(s => s.status === 'active' && s.avgScore > 0);
    const avgScore = active.length ? Math.round(active.reduce((sum, s) => sum + s.avgScore, 0) / active.length) : 0;
    return { total: students.length, active: students.filter(s => s.status === 'active').length, grade12: students.filter(s => s.grade === 12).length, avgScore };
  }, [students]);

  const openCreate = () => { setEditStudent(null); setForm(emptyForm); setModalOpen(true); };

  const openEdit = (s: Student) => {
    setEditStudent(s);
    setForm({ name: s.name, nisn: s.nisn, class: s.class, grade: s.grade, email: s.email, phone: s.phone, ptnTarget: s.ptnTarget, majorTarget: s.majorTarget, status: s.status });
    setModalOpen(true);
    setDetailStudent(null);
  };

  const handleSave = () => {
    if (!form.name.trim()) { toast.error('Nama siswa wajib diisi'); return; }
    if (!form.nisn.trim()) { toast.error('NISN wajib diisi'); return; }
    if (editStudent) {
      setStudents(prev => prev.map(s => s.id === editStudent.id ? { ...s, ...form } : s));
      toast.success('Data siswa diperbarui');
    } else {
      const now = new Date().toISOString().split('T')[0];
      setStudents(prev => [...prev, { ...form, id: `s${Date.now()}`, avgScore: 0, totalTryouts: 0, lastActive: '-', joinDate: now }]);
      toast.success('Siswa berhasil ditambahkan');
    }
    setModalOpen(false);
  };

  const handleDelete = (id: string) => {
    setStudents(prev => prev.filter(s => s.id !== id));
    setDeleteId(null);
    setDetailStudent(null);
    toast.success('Data siswa dihapus');
  };

  const toggleStatus = (id: string) => {
    setStudents(prev => prev.map(s => {
      if (s.id !== id) return s;
      const ns = s.status === 'active' ? 'inactive' : 'active';
      toast.success(`Siswa ${ns === 'active' ? 'diaktifkan' : 'dinonaktifkan'}`);
      return { ...s, status: ns };
    }));
  };

  const handleSort = (col: typeof sortBy) => {
    if (sortBy === col) setSortDir(d => d === 'asc' ? 'desc' : 'asc');
    else { setSortBy(col); setSortDir('asc'); }
  };

  const setF = <K extends keyof typeof form>(k: K, v: (typeof form)[K]) => setForm(f => ({ ...f, [k]: v }));

  const handleBulkParse = () => { setBulkRows(parseCSV(bulkCSV)); };

  const handleBulkConfirm = () => {
    const ready = bulkRows.filter(r => r.status === 'ready');
    const now = new Date().toISOString().split('T')[0];
    const newStudents: Student[] = ready.map(r => ({
      id: `s${Date.now()}_${r.nisn}`,
      name: r.name, nisn: r.nisn, class: r.class, grade: (Number(r.class[0]) || 12) as 10 | 11 | 12,
      email: r.email, phone: '', ptnTarget: '', majorTarget: '',
      avgScore: 0, totalTryouts: 0, lastActive: '-', status: 'active', joinDate: now,
    }));
    setStudents(prev => [...prev, ...newStudents]);
    toast.success(`${ready.length} siswa berhasil diimport dan akun siap digunakan`);
    setBulkOpen(false);
    setBulkCSV('');
    setBulkRows([]);
    setBulkStep(1);
  };

  const downloadTemplate = () => {
    const blob = new Blob([CSV_TEMPLATE], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a'); a.href = url; a.download = 'template_import_siswa.csv'; a.click();
    URL.revokeObjectURL(url);
    toast.success('Template diunduh');
  };

  const copyCredentials = () => {
    const text = bulkRows.filter(r => r.status === 'ready').map(r => `${r.name} | ${r.nisn} | ${r.tempPassword}`).join('\n');
    navigator.clipboard.writeText(text);
    toast.success('Kredensial disalin ke clipboard');
  };

  const SortIcon = ({ col }: { col: typeof sortBy }) =>
    sortBy === col ? (sortDir === 'asc' ? <ChevronUp className="w-3 h-3 inline ml-1" /> : <ChevronDown className="w-3 h-3 inline ml-1" />) : null;

  return (
    <div className="space-y-6">
      {/* Stats */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
        {[
          { label: 'Total Siswa', val: stats.total, icon: Users, color: 'text-indigo-600', bg: 'bg-indigo-50 border-indigo-200' },
          { label: 'Aktif', val: stats.active, icon: UserCheck, color: 'text-emerald-600', bg: 'bg-emerald-50 border-emerald-200' },
          { label: 'Kelas 12', val: stats.grade12, icon: GraduationCap, color: 'text-purple-600', bg: 'bg-purple-50 border-purple-200' },
          { label: 'Rata-rata Skor', val: stats.avgScore || '-', icon: TrendingUp, color: 'text-blue-600', bg: 'bg-blue-50 border-blue-200' },
        ].map(({ label, val, icon: Icon, color, bg }) => (
          <Card key={label} className={`p-4 border ${bg}`}>
            <div className="flex items-start justify-between">
              <div><p className="text-xs text-muted-foreground mb-1">{label}</p><p className={`text-2xl font-black ${color}`}>{val}</p></div>
              <div className={`w-9 h-9 rounded-xl ${bg} border flex items-center justify-center`}><Icon className={`w-4 h-4 ${color}`} /></div>
            </div>
          </Card>
        ))}
      </div>

      {/* Toolbar */}
      <Card className="p-4">
        <div className="flex flex-col sm:flex-row gap-3">
          <div className="flex-1 relative">
            <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
            <Input className="pl-9" placeholder="Cari nama, NISN, kelas, atau PTN target..." value={search} onChange={e => setSearch(e.target.value)} />
          </div>
          <div className="flex gap-2 flex-wrap">
            <select className="border rounded-xl px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterGrade} onChange={e => setFilterGrade(e.target.value)}>
              <option value="all">Semua Kelas</option>
              <option value="10">Kelas X</option>
              <option value="11">Kelas XI</option>
              <option value="12">Kelas XII</option>
            </select>
            <select className="border rounded-xl px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterStatus} onChange={e => setFilterStatus(e.target.value)}>
              <option value="all">Semua Status</option>
              <option value="active">Aktif</option>
              <option value="inactive">Nonaktif</option>
            </select>
            <Button variant="outline" className="gap-2 shrink-0"><Download className="w-4 h-4" /> Export</Button>
            <Button variant="outline" className="gap-2 shrink-0 border-indigo-300 text-indigo-700 hover:bg-indigo-50" onClick={() => { setBulkOpen(true); setBulkStep(1); setBulkCSV(''); setBulkRows([]); }}>
              <Upload className="w-4 h-4" /> Import CSV
            </Button>
            <Button onClick={openCreate} className="bg-indigo-600 hover:bg-indigo-700 gap-2 shrink-0"><Plus className="w-4 h-4" /> Tambah Siswa</Button>
          </div>
        </div>
      </Card>

      {/* Table */}
      <Card className="overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-slate-50">
                <th className="text-left px-4 py-3 font-semibold text-slate-600">
                  <button onClick={() => handleSort('name')} className="flex items-center gap-1 hover:text-indigo-600 transition-colors">Siswa <SortIcon col="name" /></button>
                </th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Kelas</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Target PTN</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">
                  <button onClick={() => handleSort('score')} className="flex items-center gap-1 hover:text-indigo-600 transition-colors">Rata-rata <SortIcon col="score" /></button>
                </th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">
                  <button onClick={() => handleSort('tryouts')} className="flex items-center gap-1 hover:text-indigo-600 transition-colors">Tryout <SortIcon col="tryouts" /></button>
                </th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Status</th>
                <th className="text-left px-4 py-3 font-semibold text-slate-600">Aksi</th>
              </tr>
            </thead>
            <tbody>
              {filtered.length === 0 ? (
                <tr><td colSpan={7} className="px-4 py-12 text-center text-muted-foreground">Tidak ada siswa yang sesuai filter.</td></tr>
              ) : filtered.map(s => {
                const gb = gradeBadge(s.avgScore);
                return (
                  <tr key={s.id} className="border-b hover:bg-slate-50 transition-colors">
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-3">
                        <div className="w-9 h-9 rounded-full bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white text-xs font-bold shrink-0">
                          {s.name.split(' ').slice(0, 2).map(w => w[0]).join('')}
                        </div>
                        <div><p className="font-semibold text-slate-900">{s.name}</p><p className="text-xs text-muted-foreground">{s.nisn}</p></div>
                      </div>
                    </td>
                    <td className="px-4 py-3 text-slate-700">{s.class}</td>
                    <td className="px-4 py-3">
                      <span className="font-semibold text-indigo-600">{s.ptnTarget || '—'}</span>
                      {s.majorTarget && <p className="text-xs text-muted-foreground">{s.majorTarget}</p>}
                    </td>
                    <td className="px-4 py-3">
                      <span className="font-bold text-slate-900">{s.avgScore > 0 ? s.avgScore : '—'}</span>
                      <Badge className={`ml-2 text-xs ${gb.color}`}>{gb.label}</Badge>
                    </td>
                    <td className="px-4 py-3">
                      <span className="font-semibold text-slate-700">{s.totalTryouts}</span>
                      <span className="text-xs text-muted-foreground ml-1">sesi</span>
                    </td>
                    <td className="px-4 py-3">
                      <Badge className={s.status === 'active' ? 'bg-emerald-100 text-emerald-700' : 'bg-slate-100 text-slate-500'}>
                        {s.status === 'active' ? 'Aktif' : 'Nonaktif'}
                      </Badge>
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-1">
                        <button onClick={() => setDetailStudent(s)} className="p-1.5 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-indigo-600 transition-colors" title="Detail"><Eye className="w-4 h-4" /></button>
                        <button onClick={() => openEdit(s)} className="p-1.5 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-slate-700 transition-colors" title="Edit"><Edit className="w-4 h-4" /></button>
                        <button onClick={() => toggleStatus(s.id)} className={`p-1.5 rounded-lg hover:bg-slate-100 transition-colors ${s.status === 'active' ? 'text-slate-400 hover:text-amber-600' : 'text-slate-400 hover:text-emerald-600'}`} title={s.status === 'active' ? 'Nonaktifkan' : 'Aktifkan'}>
                          {s.status === 'active' ? <UserX className="w-4 h-4" /> : <UserCheck className="w-4 h-4" />}
                        </button>
                        <button onClick={() => setDeleteId(s.id)} className="p-1.5 rounded-lg hover:bg-red-50 text-slate-400 hover:text-red-500 transition-colors" title="Hapus"><Trash2 className="w-4 h-4" /></button>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
        <div className="px-4 py-3 bg-slate-50 border-t text-xs text-muted-foreground">Menampilkan {filtered.length} dari {students.length} siswa</div>
      </Card>

      {/* Detail Drawer */}
      {detailStudent && (
        <div className="fixed inset-0 bg-black/40 flex justify-end z-50" onClick={() => setDetailStudent(null)}>
          <div className="bg-white w-full max-w-md h-full overflow-y-auto shadow-2xl" onClick={e => e.stopPropagation()}>
            <div className="sticky top-0 bg-white border-b p-5 flex items-center justify-between z-10">
              <h3 className="font-bold text-slate-900">Detail Siswa</h3>
              <button onClick={() => setDetailStudent(null)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-5 space-y-6">
              <div className="flex items-center gap-4">
                <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white text-2xl font-black">
                  {detailStudent.name.split(' ').slice(0, 2).map(w => w[0]).join('')}
                </div>
                <div>
                  <h4 className="text-xl font-black text-slate-900">{detailStudent.name}</h4>
                  <p className="text-sm text-muted-foreground">NISN: {detailStudent.nisn}</p>
                  <Badge className={detailStudent.status === 'active' ? 'bg-emerald-100 text-emerald-700 mt-1' : 'bg-slate-100 text-slate-500 mt-1'}>
                    {detailStudent.status === 'active' ? 'Aktif' : 'Nonaktif'}
                  </Badge>
                </div>
              </div>
              <div className="grid grid-cols-3 gap-3">
                {[
                  { label: 'Rata-rata', val: detailStudent.avgScore > 0 ? detailStudent.avgScore : '-', color: 'text-indigo-600' },
                  { label: 'Total Sesi', val: detailStudent.totalTryouts, color: 'text-purple-600' },
                  { label: 'Terakhir Aktif', val: detailStudent.lastActive, color: 'text-slate-700', small: true },
                ].map(({ label, val, color, small }) => (
                  <div key={label} className="bg-slate-50 rounded-xl p-3 text-center">
                    <p className={`${small ? 'text-sm' : 'text-xl'} font-black ${color}`}>{val}</p>
                    <p className="text-xs text-muted-foreground mt-0.5">{label}</p>
                  </div>
                ))}
              </div>
              <div className="space-y-3">
                <h5 className="font-semibold text-slate-700 text-sm">Informasi Kontak</h5>
                {[
                  { icon: Mail, label: 'Email', val: detailStudent.email || '-' },
                  { icon: Phone, label: 'Telepon', val: detailStudent.phone || '-' },
                  { icon: BookOpen, label: 'Kelas', val: `${detailStudent.class} (Kelas ${detailStudent.grade})` },
                ].map(({ icon: Icon, label, val }) => (
                  <div key={label} className="flex items-center gap-3 p-3 bg-slate-50 rounded-xl">
                    <Icon className="w-4 h-4 text-slate-400 shrink-0" />
                    <div><p className="text-xs text-muted-foreground">{label}</p><p className="text-sm font-semibold text-slate-900">{val}</p></div>
                  </div>
                ))}
              </div>
              {(detailStudent.ptnTarget || detailStudent.majorTarget) && (
                <div className="space-y-2">
                  <h5 className="font-semibold text-slate-700 text-sm">Target PTN</h5>
                  <div className="p-4 bg-indigo-50 border border-indigo-200 rounded-xl flex items-start gap-3">
                    <Target className="w-5 h-5 text-indigo-600 shrink-0 mt-0.5" />
                    <div>
                      <p className="font-bold text-indigo-900">{detailStudent.ptnTarget}</p>
                      {detailStudent.majorTarget && <p className="text-sm text-indigo-700">{detailStudent.majorTarget}</p>}
                    </div>
                  </div>
                </div>
              )}
              {detailStudent.avgScore > 0 && (
                <div className="space-y-2">
                  <h5 className="font-semibold text-slate-700 text-sm">Performa</h5>
                  <div className="p-4 bg-slate-50 rounded-xl space-y-2">
                    <div className="flex items-center justify-between">
                      <span className="text-sm text-muted-foreground">Skor rata-rata</span>
                      <Badge className={gradeBadge(detailStudent.avgScore).color}>{gradeBadge(detailStudent.avgScore).label}</Badge>
                    </div>
                    <div className="flex items-center gap-3">
                      <div className="flex-1 h-3 bg-slate-200 rounded-full overflow-hidden">
                        <div className="h-full bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full" style={{ width: `${Math.round((detailStudent.avgScore / 1000) * 100)}%` }} />
                      </div>
                      <span className="font-black text-slate-900 text-lg">{detailStudent.avgScore}</span>
                    </div>
                  </div>
                </div>
              )}
              <div className="flex gap-3 pt-2">
                <Button variant="outline" className="flex-1" onClick={() => toggleStatus(detailStudent.id)}>
                  {detailStudent.status === 'active' ? <><UserX className="w-4 h-4 mr-1" />Nonaktifkan</> : <><UserCheck className="w-4 h-4 mr-1" />Aktifkan</>}
                </Button>
                <Button className="flex-1 bg-indigo-600 hover:bg-indigo-700" onClick={() => openEdit(detailStudent)}>
                  <Edit className="w-4 h-4 mr-1" />Edit Data
                </Button>
              </div>
              <Button variant="destructive" className="w-full" onClick={() => setDeleteId(detailStudent.id)}>
                <Trash2 className="w-4 h-4 mr-1" />Hapus Siswa
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Create/Edit Modal */}
      {modalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4 overflow-y-auto">
          <div className="bg-white rounded-2xl w-full max-w-lg my-4 shadow-2xl">
            <div className="flex items-center justify-between p-6 border-b">
              <h3 className="text-lg font-bold">{editStudent ? 'Edit Data Siswa' : 'Tambah Siswa Baru'}</h3>
              <button onClick={() => setModalOpen(false)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-6 space-y-4 max-h-[65vh] overflow-y-auto">
              <div className="grid grid-cols-2 gap-4">
                <div className="col-span-2">
                  <Label>Nama Lengkap *</Label>
                  <Input className="mt-1.5" value={form.name} onChange={e => setF('name', e.target.value)} placeholder="Nama lengkap siswa" />
                </div>
                <div>
                  <Label>NISN *</Label>
                  <Input className="mt-1.5" value={form.nisn} onChange={e => setF('nisn', e.target.value)} placeholder="10 digit NISN" maxLength={10} />
                </div>
                <div>
                  <Label>Kelas</Label>
                  <select className="w-full mt-1.5 border rounded-xl px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.class} onChange={e => { setF('class', e.target.value); setF('grade', Number(e.target.value[0]) as 10 | 11 | 12); }}>
                    {classOptions.map(c => <option key={c} value={c}>{c}</option>)}
                  </select>
                </div>
                <div>
                  <Label>Email</Label>
                  <Input className="mt-1.5" type="email" value={form.email} onChange={e => setF('email', e.target.value)} placeholder="email@student.sch.id" />
                </div>
                <div>
                  <Label>Telepon</Label>
                  <Input className="mt-1.5" value={form.phone} onChange={e => setF('phone', e.target.value)} placeholder="08xxxxxxxxxx" />
                </div>
                <div>
                  <Label>Target PTN</Label>
                  <select className="w-full mt-1.5 border rounded-xl px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.ptnTarget} onChange={e => setF('ptnTarget', e.target.value)}>
                    <option value="">Belum diset</option>
                    {ptnOptions.map(p => <option key={p} value={p}>{p}</option>)}
                  </select>
                </div>
                <div>
                  <Label>Program Studi</Label>
                  <Input className="mt-1.5" value={form.majorTarget} onChange={e => setF('majorTarget', e.target.value)} placeholder="Teknik Informatika" />
                </div>
                <div className="col-span-2">
                  <Label>Status</Label>
                  <div className="flex gap-2 mt-1.5">
                    {(['active', 'inactive'] as const).map(st => (
                      <button key={st} onClick={() => setF('status', st)} className={`flex-1 py-2 rounded-xl text-sm font-semibold border transition-colors ${form.status === st ? (st === 'active' ? 'bg-emerald-100 text-emerald-700 border-emerald-300' : 'bg-slate-100 text-slate-600 border-slate-300') : 'border-slate-200 text-slate-400 hover:bg-slate-50'}`}>
                        {st === 'active' ? 'Aktif' : 'Nonaktif'}
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            </div>
            <div className="flex gap-3 p-6 border-t">
              <Button variant="outline" className="flex-1" onClick={() => setModalOpen(false)}>Batal</Button>
              <Button className="flex-1 bg-gradient-to-r from-indigo-600 to-purple-600 gap-2" onClick={handleSave}>
                <Save className="w-4 h-4" /> {editStudent ? 'Simpan Perubahan' : 'Tambah Siswa'}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* ─── Bulk Import Modal ─── */}
      {bulkOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4 overflow-y-auto">
          <div className="bg-white rounded-2xl w-full max-w-2xl my-4 shadow-2xl">
            <div className="flex items-center justify-between p-6 border-b">
              <div>
                <h3 className="text-lg font-bold">Import Siswa via CSV</h3>
                <p className="text-sm text-muted-foreground">Upload data massal — akun langsung dibuat otomatis</p>
              </div>
              <button onClick={() => setBulkOpen(false)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
            </div>

            {/* Step tabs */}
            <div className="flex border-b">
              {['Input Data', 'Preview & Konfirmasi'].map((label, i) => {
                const s = (i + 1) as 1 | 2;
                return (
                  <div key={label} className={`flex-1 py-3 text-center text-sm font-semibold border-b-2 transition-colors ${bulkStep === s ? 'border-indigo-600 text-indigo-600' : 'border-transparent text-slate-400'}`}>
                    <span className={`inline-flex items-center justify-center w-5 h-5 rounded-full text-xs mr-2 ${bulkStep > i ? 'bg-green-500 text-white' : bulkStep === s ? 'bg-indigo-600 text-white' : 'bg-slate-200 text-slate-400'}`}>
                      {bulkStep > i ? <CheckCircle2 className="w-3 h-3" /> : s}
                    </span>
                    {label}
                  </div>
                );
              })}
            </div>

            <div className="p-6 space-y-5 max-h-[60vh] overflow-y-auto">
              {bulkStep === 1 && (
                <>
                  <div className="flex items-center justify-between">
                    <div className="p-3 bg-slate-50 border rounded-xl flex-1 mr-3">
                      <p className="text-xs font-mono text-slate-500 mb-1">Format CSV:</p>
                      <p className="text-xs font-mono text-slate-800">NISN, Nama Lengkap, Kelas, Email</p>
                    </div>
                    <Button variant="outline" size="sm" className="gap-2 shrink-0" onClick={downloadTemplate}>
                      <Download className="w-4 h-4" /> Template
                    </Button>
                  </div>

                  <div>
                    <Label>Paste data CSV</Label>
                    <textarea
                      className="w-full mt-1.5 px-3 py-3 border rounded-xl text-sm font-mono resize-none h-40 focus:outline-none focus:ring-2 focus:ring-indigo-300"
                      value={bulkCSV}
                      onChange={e => { setBulkCSV(e.target.value); setBulkRows([]); }}
                      placeholder={"NISN,Nama Lengkap,Kelas,Email\n0045678909,Ahmad Fauzi,XII IPA 1,ahmad@siswa.sch.id\n0045678910,Rina Marlena,XI IPA 2,rina@siswa.sch.id"}
                    />
                  </div>

                  <Button className="w-full bg-indigo-600 hover:bg-indigo-700 gap-2" disabled={!bulkCSV.trim()} onClick={() => { handleBulkParse(); setBulkStep(2); }}>
                    <RefreshCw className="w-4 h-4" /> Validasi & Lanjut
                  </Button>

                  <div className="p-4 bg-blue-50 border border-blue-200 rounded-xl">
                    <p className="text-xs font-bold text-blue-700 mb-1.5 flex items-center gap-1.5"><Key className="w-3.5 h-3.5" />Kredensial Otomatis</p>
                    <ul className="text-xs text-blue-800 space-y-1">
                      <li>• <strong>Username:</strong> NISN siswa</li>
                      <li>• <strong>Password sementara:</strong> <code className="bg-blue-100 px-1 rounded">Gspl[4 digit terakhir NISN]!</code></li>
                      <li>• Siswa wajib ganti password saat login pertama</li>
                    </ul>
                  </div>
                </>
              )}

              {bulkStep === 2 && (
                <>
                  <div className="flex items-center justify-between">
                    <p className="text-sm font-semibold text-slate-700">
                      <span className="text-green-600 font-bold">{bulkRows.filter(r => r.status === 'ready').length} siap</span>
                      {bulkRows.filter(r => r.status === 'error').length > 0 && (
                        <>, <span className="text-red-500 font-bold">{bulkRows.filter(r => r.status === 'error').length} error</span></>
                      )}
                      {' '}<span className="text-muted-foreground font-normal">dari {bulkRows.length} baris</span>
                    </p>
                    <Button variant="outline" size="sm" className="gap-1.5" onClick={copyCredentials}>
                      <Copy className="w-3.5 h-3.5" /> Salin Kredensial
                    </Button>
                  </div>

                  <div className="border rounded-xl overflow-hidden">
                    <table className="w-full text-xs">
                      <thead className="sticky top-0 bg-slate-50">
                        <tr className="border-b">
                          <th className="text-left px-3 py-2.5 font-semibold text-slate-600">Nama</th>
                          <th className="text-left px-3 py-2.5 font-semibold text-slate-600">NISN</th>
                          <th className="text-left px-3 py-2.5 font-semibold text-slate-600">Password</th>
                          <th className="text-left px-3 py-2.5 font-semibold text-slate-600">Kelas</th>
                          <th className="text-left px-3 py-2.5 font-semibold text-slate-600">Status</th>
                        </tr>
                      </thead>
                      <tbody>
                        {bulkRows.map((r, i) => (
                          <tr key={i} className={`border-b ${r.status === 'error' ? 'bg-red-50' : 'hover:bg-slate-50'}`}>
                            <td className="px-3 py-2 font-semibold">{r.name || '—'}</td>
                            <td className="px-3 py-2 font-mono text-indigo-600">{r.nisn || '—'}</td>
                            <td className="px-3 py-2 font-mono text-purple-600">{r.status === 'ready' ? r.tempPassword : '—'}</td>
                            <td className="px-3 py-2">{r.class || '—'}</td>
                            <td className="px-3 py-2">
                              {r.status === 'ready'
                                ? <span className="text-green-600 font-semibold flex items-center gap-1"><CheckCircle2 className="w-3 h-3" />Siap</span>
                                : <span className="text-red-500 font-semibold">{r.error}</span>
                              }
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>

                  {bulkRows.filter(r => r.status === 'ready').length > 0 && (
                    <div className="p-4 bg-green-50 border border-green-200 rounded-xl flex items-start gap-2">
                      <Send className="w-4 h-4 text-green-600 shrink-0 mt-0.5" />
                      <p className="text-xs text-green-800">
                        <span className="font-bold">{bulkRows.filter(r => r.status === 'ready').length} akun</span> akan langsung aktif. Siswa bisa login menggunakan NISN dan password sementara yang tertera di atas.
                      </p>
                    </div>
                  )}
                </>
              )}
            </div>

            <div className="flex gap-3 p-6 border-t bg-slate-50 rounded-b-2xl">
              <Button variant="outline" onClick={() => bulkStep === 1 ? setBulkOpen(false) : setBulkStep(1)}>
                {bulkStep === 1 ? 'Batal' : '← Kembali'}
              </Button>
              {bulkStep === 2 && (
                <Button
                  className="flex-1 bg-gradient-to-r from-indigo-600 to-purple-600 gap-2"
                  disabled={bulkRows.filter(r => r.status === 'ready').length === 0}
                  onClick={handleBulkConfirm}
                >
                  <UserCheck className="w-4 h-4" /> Konfirmasi Import {bulkRows.filter(r => r.status === 'ready').length} Siswa
                </Button>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Delete confirm */}
      {deleteId && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-2xl p-6 w-full max-w-sm shadow-2xl">
            <div className="w-12 h-12 rounded-full bg-red-100 flex items-center justify-center mx-auto mb-4"><AlertCircle className="w-6 h-6 text-red-600" /></div>
            <h3 className="font-bold text-slate-900 text-center mb-2">Hapus Data Siswa?</h3>
            <p className="text-sm text-muted-foreground text-center mb-6">Data siswa ini akan dihapus permanen termasuk riwayat tryout.</p>
            <div className="flex gap-3">
              <Button variant="outline" className="flex-1" onClick={() => setDeleteId(null)}>Batal</Button>
              <Button className="flex-1 bg-red-600 hover:bg-red-700" onClick={() => handleDelete(deleteId)}>Hapus</Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
