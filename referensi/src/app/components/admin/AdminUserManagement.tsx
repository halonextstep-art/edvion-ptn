import { useState, useMemo } from 'react';
import {
  Search, Plus, Edit3, Trash2, X, Save, Eye, EyeOff,
  UserCheck, UserX, Filter, Download, Upload,
  GraduationCap, School, Layers, ShieldCheck,
  ChevronDown, RefreshCw, MoreHorizontal, Copy
} from 'lucide-react';
import { toast } from 'sonner';

type UserRole = 'student' | 'school' | 'content' | 'admin';
type UserStatus = 'active' | 'inactive' | 'pending';

interface AppUser {
  id: string;
  name: string;
  email: string;
  role: UserRole;
  status: UserStatus;
  schoolName?: string;     // for student, school
  grade?: string;          // for student (Kelas 10/11/12)
  nisn?: string;           // for student
  phone?: string;
  createdAt: string;
  lastLogin?: string;
  notes?: string;
}

const ROLE_CONFIG: Record<UserRole, { label: string; icon: typeof GraduationCap; color: string; bg: string }> = {
  student:  { label: 'Siswa',      icon: GraduationCap, color: 'text-blue-600',   bg: 'bg-blue-50 border-blue-200' },
  school:   { label: 'Sekolah',    icon: School,        color: 'text-emerald-600', bg: 'bg-emerald-50 border-emerald-200' },
  content:  { label: 'Tim Konten', icon: Layers,        color: 'text-violet-600',  bg: 'bg-violet-50 border-violet-200' },
  admin:    { label: 'Admin',      icon: ShieldCheck,   color: 'text-rose-600',    bg: 'bg-rose-50 border-rose-200' },
};

const STATUS_CONFIG: Record<UserStatus, { label: string; dot: string; text: string }> = {
  active:   { label: 'Aktif',    dot: 'bg-emerald-400', text: 'text-emerald-700 bg-emerald-50 border-emerald-200' },
  inactive: { label: 'Nonaktif', dot: 'bg-slate-400',   text: 'text-slate-600 bg-slate-100 border-slate-200' },
  pending:  { label: 'Pending',  dot: 'bg-amber-400',   text: 'text-amber-700 bg-amber-50 border-amber-200' },
};

const MOCK_USERS: AppUser[] = [
  { id: 'u001', name: 'Budi Santoso', email: 'budi@gaspolptn.id', role: 'admin', status: 'active', createdAt: '2024-01-01', lastLogin: '2025-01-23' },
  { id: 'u002', name: 'SMA Negeri 1 Bandung', email: 'admin@sman1bdg.sch.id', role: 'school', status: 'active', schoolName: 'SMA Negeri 1 Bandung', phone: '022-4201234', createdAt: '2024-03-10', lastLogin: '2025-01-22' },
  { id: 'u003', name: 'SMA BPK Penabur', email: 'admin@bpkpenabur.sch.id', role: 'school', status: 'active', schoolName: 'SMA BPK Penabur', phone: '021-5678901', createdAt: '2024-04-15', lastLogin: '2025-01-20' },
  { id: 'u004', name: 'SMA Muhammadiyah 2', email: 'admin@muhammadiyah2.sch.id', role: 'school', status: 'pending', schoolName: 'SMA Muhammadiyah 2', phone: '021-3456789', createdAt: '2025-01-20' },
  { id: 'u005', name: 'Sari Dewi', email: 'sari@gaspolptn.id', role: 'content', status: 'active', createdAt: '2024-06-01', lastLogin: '2025-01-23' },
  { id: 'u006', name: 'Ahmad Fauzi', email: 'ahmad@gaspolptn.id', role: 'content', status: 'active', createdAt: '2024-06-01', lastLogin: '2025-01-21' },
  { id: 'u007', name: 'Rina Marlina', email: 'rina@gaspolptn.id', role: 'content', status: 'inactive', createdAt: '2024-08-15', lastLogin: '2024-12-30' },
  { id: 'u008', name: 'Bunga Rahayu', email: 'bunga@sman1bdg.sch.id', role: 'student', status: 'active', schoolName: 'SMA Negeri 1 Bandung', grade: 'Kelas 12', nisn: '0056781234', createdAt: '2024-07-01', lastLogin: '2025-01-23' },
  { id: 'u009', name: 'Andi Pratama', email: 'andi@sman1bdg.sch.id', role: 'student', status: 'active', schoolName: 'SMA Negeri 1 Bandung', grade: 'Kelas 12', nisn: '0056785678', createdAt: '2024-07-01', lastLogin: '2025-01-22' },
  { id: 'u010', name: 'Hana Wijaya', email: 'hana@sman1bdg.sch.id', role: 'student', status: 'active', schoolName: 'SMA Negeri 1 Bandung', grade: 'Kelas 11', nisn: '0056789012', createdAt: '2024-07-01', lastLogin: '2025-01-20' },
  { id: 'u011', name: 'Rizki Maulana', email: 'rizki@bpkpenabur.sch.id', role: 'student', status: 'inactive', schoolName: 'SMA BPK Penabur', grade: 'Kelas 12', nisn: '0067891234', createdAt: '2024-08-01', lastLogin: '2025-01-10' },
  { id: 'u012', name: 'Dewi Lestari', email: 'dewi@bpkpenabur.sch.id', role: 'student', status: 'active', schoolName: 'SMA BPK Penabur', grade: 'Kelas 11', nisn: '0067895678', createdAt: '2024-08-01', lastLogin: '2025-01-21' },
];

const makeId = () => `u${Date.now()}`;
const today = () => new Date().toISOString().split('T')[0];

const emptyForm = (): Omit<AppUser, 'id' | 'createdAt'> => ({
  name: '', email: '', role: 'student', status: 'active',
  schoolName: '', grade: 'Kelas 12', nisn: '', phone: '', notes: '',
});

function genPassword(name: string) {
  const part = name.replace(/\s+/g, '').slice(0, 4).toLowerCase();
  return `Gspl${part}${Math.floor(1000 + Math.random() * 9000)}!`;
}

export default function AdminUserManagement() {
  const [users, setUsers] = useState<AppUser[]>(MOCK_USERS);
  const [search, setSearch] = useState('');
  const [filterRole, setFilterRole] = useState<UserRole | 'all'>('all');
  const [filterStatus, setFilterStatus] = useState<UserStatus | 'all'>('all');
  const [editorOpen, setEditorOpen] = useState(false);
  const [editId, setEditId] = useState<string | null>(null);
  const [form, setForm] = useState(emptyForm());
  const [showPass, setShowPass] = useState(false);
  const [tempPass, setTempPass] = useState('');
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [detailUser, setDetailUser] = useState<AppUser | null>(null);

  const filtered = useMemo(() => users.filter(u => {
    const ms = !search || u.name.toLowerCase().includes(search.toLowerCase()) || u.email.toLowerCase().includes(search.toLowerCase()) || (u.schoolName || '').toLowerCase().includes(search.toLowerCase());
    return ms && (filterRole === 'all' || u.role === filterRole) && (filterStatus === 'all' || u.status === filterStatus);
  }), [users, search, filterRole, filterStatus]);

  const stats = useMemo(() => ({
    total: users.length,
    student: users.filter(u => u.role === 'student').length,
    school: users.filter(u => u.role === 'school').length,
    content: users.filter(u => u.role === 'content').length,
    admin: users.filter(u => u.role === 'admin').length,
    active: users.filter(u => u.status === 'active').length,
    pending: users.filter(u => u.status === 'pending').length,
  }), [users]);

  const openCreate = () => {
    setEditId(null);
    const f = emptyForm();
    setForm(f);
    const pass = genPassword('user');
    setTempPass(pass);
    setEditorOpen(true);
  };

  const openEdit = (u: AppUser) => {
    setEditId(u.id);
    setForm({ name: u.name, email: u.email, role: u.role, status: u.status, schoolName: u.schoolName || '', grade: u.grade || 'Kelas 12', nisn: u.nisn || '', phone: u.phone || '', notes: u.notes || '' });
    setTempPass('');
    setEditorOpen(true);
  };

  const handleSave = () => {
    if (!form.name.trim() || !form.email.trim()) { toast.error('Nama dan email wajib diisi'); return; }
    if (editId) {
      setUsers(prev => prev.map(u => u.id === editId ? { ...u, ...form } : u));
      toast.success('User diperbarui');
    } else {
      const newUser: AppUser = { ...form, id: makeId(), createdAt: today() };
      setUsers(prev => [newUser, ...prev]);
      toast.success(`User baru dibuat — password sementara: ${tempPass}`);
    }
    setEditorOpen(false);
  };

  const toggleStatus = (id: string) => {
    setUsers(prev => prev.map(u => {
      if (u.id !== id) return u;
      const next = u.status === 'active' ? 'inactive' : 'active';
      toast.success(`User ${next === 'active' ? 'diaktifkan' : 'dinonaktifkan'}`);
      return { ...u, status: next };
    }));
  };

  const approveUser = (id: string) => {
    setUsers(prev => prev.map(u => u.id === id ? { ...u, status: 'active' } : u));
    toast.success('User disetujui dan diaktifkan');
  };

  const handleDelete = (id: string) => {
    setUsers(prev => prev.filter(u => u.id !== id));
    setDeleteId(null);
    setDetailUser(null);
    toast.success('User dihapus');
  };

  const resetPassword = (u: AppUser) => {
    const pass = genPassword(u.name);
    navigator.clipboard?.writeText(pass).catch(() => {});
    toast.success(`Password baru: ${pass} (disalin ke clipboard)`);
  };

  const inp = 'w-full px-3 py-2.5 border border-slate-200 rounded-xl text-sm text-slate-900 focus:outline-none focus:border-indigo-400 transition-colors';

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold text-slate-900">Manajemen User</h2>
          <p className="text-sm text-slate-500 mt-0.5">Kelola semua akun — siswa, sekolah, tim konten, dan admin</p>
        </div>
        <button onClick={openCreate} className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white font-bold text-sm transition-all shadow-lg hover:-translate-y-0.5 shrink-0">
          <Plus className="w-4 h-4" /> Tambah User
        </button>
      </div>

      {/* Role stats */}
      <div className="grid grid-cols-2 lg:grid-cols-5 gap-3">
        {([
          { label: 'Total User', val: stats.total, icon: null, extra: `${stats.active} aktif`, color: 'text-slate-800' },
          { label: 'Siswa', val: stats.student, icon: GraduationCap, extra: '', color: 'text-blue-600' },
          { label: 'Sekolah', val: stats.school, icon: School, extra: `${stats.pending} pending`, color: 'text-emerald-600' },
          { label: 'Tim Konten', val: stats.content, icon: Layers, extra: '', color: 'text-violet-600' },
          { label: 'Admin', val: stats.admin, icon: ShieldCheck, extra: '', color: 'text-rose-600' },
        ] as const).map(({ label, val, icon: Icon, extra, color }) => (
          <div key={label} className="bg-white rounded-2xl border border-slate-200 p-4 hover:shadow-sm transition-shadow">
            {Icon && <Icon className={`w-5 h-5 ${color} mb-2`} />}
            <p className={`text-2xl font-black ${color}`}>{val}</p>
            <p className="text-xs text-slate-500 mt-0.5">{label}</p>
            {extra && <p className="text-[10px] text-slate-400 mt-0.5">{extra}</p>}
          </div>
        ))}
      </div>

      {/* Filters */}
      <div className="bg-white rounded-2xl border border-slate-200 p-4">
        <div className="flex flex-col sm:flex-row gap-3">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
            <input value={search} onChange={e => setSearch(e.target.value)} placeholder="Cari nama, email, atau sekolah..." className="w-full pl-9 pr-4 py-2.5 border border-slate-200 rounded-xl text-sm focus:outline-none focus:border-indigo-400" />
          </div>
          <div className="flex gap-2 flex-wrap">
            {(['all', 'student', 'school', 'content', 'admin'] as const).map(r => (
              <button key={r} onClick={() => setFilterRole(r)} className={`px-3 py-2 rounded-xl text-xs font-semibold border transition-colors ${filterRole === r ? 'bg-indigo-600 text-white border-indigo-600' : 'bg-white text-slate-600 border-slate-200 hover:border-indigo-300'}`}>
                {r === 'all' ? 'Semua Role' : ROLE_CONFIG[r].label}
              </button>
            ))}
            <select value={filterStatus} onChange={e => setFilterStatus(e.target.value as UserStatus | 'all')} className="px-3 py-2 border border-slate-200 rounded-xl text-xs text-slate-600 focus:outline-none focus:border-indigo-400">
              <option value="all">Semua Status</option>
              <option value="active">Aktif</option>
              <option value="inactive">Nonaktif</option>
              <option value="pending">Pending</option>
            </select>
          </div>
        </div>
      </div>

      {/* Pending approvals banner */}
      {stats.pending > 0 && (
        <div className="bg-amber-50 border border-amber-200 rounded-2xl p-4 flex items-center justify-between gap-4">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-xl bg-amber-100 flex items-center justify-center"><Filter className="w-4 h-4 text-amber-600" /></div>
            <div>
              <p className="text-sm font-bold text-amber-800">{stats.pending} akun menunggu persetujuan</p>
              <p className="text-xs text-amber-600">Sekolah baru mendaftar dan perlu diverifikasi</p>
            </div>
          </div>
          <button onClick={() => setFilterStatus('pending')} className="px-4 py-2 rounded-xl bg-amber-600 hover:bg-amber-500 text-white text-xs font-bold transition-colors">Lihat Semua</button>
        </div>
      )}

      {/* User table */}
      <div className="bg-white rounded-2xl border border-slate-200 overflow-hidden">
        <div className="px-5 py-3.5 border-b border-slate-100 flex items-center justify-between">
          <p className="text-sm font-semibold text-slate-700">Menampilkan {filtered.length} dari {users.length} user</p>
        </div>

        <div className="divide-y divide-slate-50">
          {filtered.length === 0 && (
            <div className="py-16 text-center">
              <GraduationCap className="w-10 h-10 text-slate-200 mx-auto mb-3" />
              <p className="text-sm text-slate-400">Tidak ada user ditemukan</p>
            </div>
          )}
          {filtered.map(u => {
            const rc = ROLE_CONFIG[u.role];
            const sc = STATUS_CONFIG[u.status];
            const RoleIcon = rc.icon;
            return (
              <div key={u.id} className="flex items-center gap-4 px-5 py-3.5 hover:bg-slate-50 transition-colors">
                {/* Avatar */}
                <div className={`w-9 h-9 rounded-xl flex items-center justify-center border shrink-0 ${rc.bg}`}>
                  <RoleIcon className={`w-4 h-4 ${rc.color}`} />
                </div>

                {/* Info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <p className="text-sm font-semibold text-slate-900">{u.name}</p>
                    <span className={`text-[10px] px-2 py-0.5 rounded-full border font-semibold ${rc.bg} ${rc.color}`}>{rc.label}</span>
                    <span className={`text-[10px] px-2 py-0.5 rounded-full border flex items-center gap-1 font-semibold ${sc.text}`}>
                      <span className={`w-1.5 h-1.5 rounded-full ${sc.dot}`} />{sc.label}
                    </span>
                  </div>
                  <div className="flex items-center gap-3 mt-0.5 flex-wrap">
                    <p className="text-xs text-slate-500">{u.email}</p>
                    {u.schoolName && <p className="text-xs text-slate-400">{u.schoolName}</p>}
                    {u.grade && <p className="text-xs text-slate-400">{u.grade}</p>}
                    {u.nisn && <p className="text-xs text-slate-400 font-mono">NISN: {u.nisn}</p>}
                  </div>
                </div>

                {/* Meta */}
                <div className="hidden lg:block text-right shrink-0">
                  <p className="text-xs text-slate-400">Dibuat: {u.createdAt}</p>
                  {u.lastLogin && <p className="text-xs text-slate-400">Login: {u.lastLogin}</p>}
                </div>

                {/* Actions */}
                <div className="flex items-center gap-1.5 shrink-0">
                  {u.status === 'pending' && (
                    <button onClick={() => approveUser(u.id)} className="px-3 py-1.5 rounded-lg bg-emerald-50 hover:bg-emerald-100 text-emerald-700 text-xs font-bold border border-emerald-200 transition-colors flex items-center gap-1">
                      <UserCheck className="w-3.5 h-3.5" /> Setujui
                    </button>
                  )}
                  <button onClick={() => toggleStatus(u.id)} className={`p-1.5 rounded-lg border transition-colors ${u.status === 'active' ? 'text-slate-500 hover:text-orange-500 hover:bg-orange-50 border-transparent hover:border-orange-200' : 'text-slate-400 hover:text-emerald-600 hover:bg-emerald-50 border-transparent hover:border-emerald-200'}`} title={u.status === 'active' ? 'Nonaktifkan' : 'Aktifkan'}>
                    {u.status === 'active' ? <UserX className="w-4 h-4" /> : <UserCheck className="w-4 h-4" />}
                  </button>
                  <button onClick={() => setDetailUser(u)} className="p-1.5 rounded-lg border border-transparent hover:bg-blue-50 hover:border-blue-200 text-slate-400 hover:text-blue-600 transition-colors"><Eye className="w-4 h-4" /></button>
                  <button onClick={() => openEdit(u)} className="p-1.5 rounded-lg border border-transparent hover:bg-slate-100 text-slate-400 hover:text-slate-700 transition-colors"><Edit3 className="w-4 h-4" /></button>
                  <button onClick={() => resetPassword(u)} className="p-1.5 rounded-lg border border-transparent hover:bg-amber-50 hover:border-amber-200 text-slate-400 hover:text-amber-600 transition-colors" title="Reset password"><RefreshCw className="w-4 h-4" /></button>
                  <button onClick={() => setDeleteId(u.id)} className="p-1.5 rounded-lg border border-transparent hover:bg-red-50 hover:border-red-200 text-slate-400 hover:text-red-500 transition-colors"><Trash2 className="w-4 h-4" /></button>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* ═══ CREATE/EDIT MODAL ════════════════════════════════════════════════ */}
      {editorOpen && (
        <div className="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center z-50 p-4 overflow-y-auto">
          <div className="bg-white rounded-3xl w-full max-w-lg my-4 shadow-2xl border border-slate-100">
            <div className="flex items-center justify-between p-6 border-b border-slate-100">
              <h3 className="text-lg font-black text-slate-900">{editId ? 'Edit User' : 'Tambah User Baru'}</h3>
              <button onClick={() => setEditorOpen(false)} className="p-2 rounded-xl hover:bg-slate-100 text-slate-400 hover:text-slate-700 transition-colors"><X className="w-5 h-5" /></button>
            </div>

            <div className="p-6 space-y-4 overflow-y-auto max-h-[70vh]">
              {/* Role selector */}
              <div>
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-2">Role *</label>
                <div className="grid grid-cols-2 gap-2">
                  {(['student', 'school', 'content', 'admin'] as UserRole[]).map(r => {
                    const rc = ROLE_CONFIG[r];
                    const Icon = rc.icon;
                    return (
                      <button key={r} onClick={() => setForm(f => ({ ...f, role: r }))} className={`flex items-center gap-2.5 p-3 rounded-xl border-2 text-left transition-all ${form.role === r ? 'border-indigo-500 bg-indigo-50' : 'border-slate-200 hover:border-indigo-200'}`}>
                        <Icon className={`w-4 h-4 ${form.role === r ? 'text-indigo-600' : rc.color}`} />
                        <span className={`text-sm font-semibold ${form.role === r ? 'text-indigo-700' : 'text-slate-600'}`}>{rc.label}</span>
                      </button>
                    );
                  })}
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="col-span-2">
                  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Lengkap *</label>
                  <input value={form.name} onChange={e => { setForm(f => ({ ...f, name: e.target.value })); if (!editId) setTempPass(genPassword(e.target.value)); }} placeholder="Nama lengkap" className={inp} />
                </div>
                <div className="col-span-2">
                  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Email *</label>
                  <input type="email" value={form.email} onChange={e => setForm(f => ({ ...f, email: e.target.value }))} placeholder="email@domain.com" className={inp} />
                </div>
              </div>

              {/* Conditional fields */}
              {(form.role === 'student' || form.role === 'school') && (
                <div>
                  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Sekolah</label>
                  <input value={form.schoolName} onChange={e => setForm(f => ({ ...f, schoolName: e.target.value }))} placeholder="SMA Negeri ..." className={inp} />
                </div>
              )}

              {form.role === 'student' && (
                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1.5">NISN</label>
                    <input value={form.nisn} onChange={e => setForm(f => ({ ...f, nisn: e.target.value }))} placeholder="0012345678" className={`${inp} font-mono`} />
                  </div>
                  <div>
                    <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Kelas</label>
                    <select value={form.grade} onChange={e => setForm(f => ({ ...f, grade: e.target.value }))} className={inp}>
                      <option>Kelas 10</option><option>Kelas 11</option><option>Kelas 12</option>
                    </select>
                  </div>
                </div>
              )}

              {(form.role === 'school') && (
                <div>
                  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1.5">No. Telepon</label>
                  <input value={form.phone} onChange={e => setForm(f => ({ ...f, phone: e.target.value }))} placeholder="021-..." className={inp} />
                </div>
              )}

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Status</label>
                  <select value={form.status} onChange={e => setForm(f => ({ ...f, status: e.target.value as UserStatus }))} className={inp}>
                    <option value="active">Aktif</option>
                    <option value="inactive">Nonaktif</option>
                    <option value="pending">Pending</option>
                  </select>
                </div>
              </div>

              {/* Password (only for new user) */}
              {!editId && (
                <div className="p-4 bg-amber-50 border border-amber-200 rounded-xl">
                  <p className="text-xs font-bold text-amber-700 mb-2">Password Sementara</p>
                  <div className="flex items-center gap-2">
                    <code className="flex-1 text-sm font-mono bg-white px-3 py-2 rounded-lg border border-amber-200 text-slate-800">
                      {showPass ? tempPass : '••••••••••'}
                    </code>
                    <button onClick={() => setShowPass(s => !s)} className="p-2 rounded-lg hover:bg-amber-100 text-amber-600 transition-colors">{showPass ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}</button>
                    <button onClick={() => { navigator.clipboard?.writeText(tempPass).catch(() => {}); toast.success('Password disalin'); }} className="p-2 rounded-lg hover:bg-amber-100 text-amber-600 transition-colors"><Copy className="w-4 h-4" /></button>
                  </div>
                  <p className="text-[10px] text-amber-600 mt-1.5">User wajib ganti password saat login pertama kali.</p>
                </div>
              )}

              <div>
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Catatan Internal</label>
                <textarea value={form.notes} onChange={e => setForm(f => ({ ...f, notes: e.target.value }))} placeholder="Catatan opsional..." className={`${inp} resize-none h-16`} />
              </div>
            </div>

            <div className="flex gap-3 p-6 border-t border-slate-100">
              <button onClick={() => setEditorOpen(false)} className="flex-1 py-3 rounded-xl border border-slate-200 text-sm font-semibold text-slate-600 hover:bg-slate-50 transition-colors">Batal</button>
              <button onClick={handleSave} className="flex-1 flex items-center justify-center gap-2 py-3 rounded-xl bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white font-bold text-sm transition-all">
                <Save className="w-4 h-4" /> {editId ? 'Simpan' : 'Buat User'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ═══ DETAIL DRAWER ═══════════════════════════════════════════════════ */}
      {detailUser && (
        <div className="fixed inset-0 bg-black/30 flex justify-end z-50" onClick={() => setDetailUser(null)}>
          <div className="bg-white w-full max-w-sm h-full overflow-y-auto shadow-2xl border-l border-slate-200" onClick={e => e.stopPropagation()}>
            <div className="sticky top-0 bg-white border-b border-slate-100 p-5 flex items-center justify-between">
              <h3 className="font-black text-slate-900">Detail User</h3>
              <button onClick={() => setDetailUser(null)} className="p-1.5 rounded-xl hover:bg-slate-100 text-slate-400"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-5 space-y-4">
              {(() => {
                const rc = ROLE_CONFIG[detailUser.role];
                const sc = STATUS_CONFIG[detailUser.status];
                const Icon = rc.icon;
                return (
                  <>
                    <div className={`p-4 rounded-2xl border ${rc.bg} flex items-center gap-3`}>
                      <div className={`w-12 h-12 rounded-xl flex items-center justify-center bg-white border ${rc.bg}`}>
                        <Icon className={`w-6 h-6 ${rc.color}`} />
                      </div>
                      <div>
                        <p className="font-bold text-slate-900">{detailUser.name}</p>
                        <p className={`text-xs font-semibold ${rc.color}`}>{rc.label}</p>
                      </div>
                    </div>

                    <div className="space-y-2.5">
                      {[
                        { label: 'Email', val: detailUser.email },
                        { label: 'Status', val: <span className={`text-xs px-2 py-0.5 rounded-full border font-semibold flex items-center gap-1 w-fit ${sc.text}`}><span className={`w-1.5 h-1.5 rounded-full ${sc.dot}`} />{sc.label}</span> },
                        detailUser.schoolName && { label: 'Sekolah', val: detailUser.schoolName },
                        detailUser.grade && { label: 'Kelas', val: detailUser.grade },
                        detailUser.nisn && { label: 'NISN', val: <code className="font-mono text-sm">{detailUser.nisn}</code> },
                        detailUser.phone && { label: 'Telepon', val: detailUser.phone },
                        { label: 'Dibuat', val: detailUser.createdAt },
                        detailUser.lastLogin && { label: 'Login Terakhir', val: detailUser.lastLogin },
                        detailUser.notes && { label: 'Catatan', val: detailUser.notes },
                      ].filter(Boolean).map(row => row && (
                        <div key={row.label} className="flex items-start justify-between gap-3 py-2 border-b border-slate-50">
                          <span className="text-xs font-bold text-slate-400 shrink-0">{row.label}</span>
                          <span className="text-sm text-slate-700 text-right">{row.val}</span>
                        </div>
                      ))}
                    </div>

                    <div className="space-y-2 pt-2">
                      <button onClick={() => { openEdit(detailUser); setDetailUser(null); }} className="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-indigo-50 hover:bg-indigo-100 text-indigo-700 text-sm font-semibold border border-indigo-200 transition-colors"><Edit3 className="w-4 h-4" />Edit User</button>
                      <button onClick={() => resetPassword(detailUser)} className="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-amber-50 hover:bg-amber-100 text-amber-700 text-sm font-semibold border border-amber-200 transition-colors"><RefreshCw className="w-4 h-4" />Reset Password</button>
                      <button onClick={() => { toggleStatus(detailUser.id); setDetailUser(u => u ? { ...u, status: u.status === 'active' ? 'inactive' : 'active' } : u); }} className="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-slate-50 hover:bg-slate-100 text-slate-700 text-sm font-semibold border border-slate-200 transition-colors">
                        {detailUser.status === 'active' ? <><UserX className="w-4 h-4" />Nonaktifkan</> : <><UserCheck className="w-4 h-4" />Aktifkan</>}
                      </button>
                      <button onClick={() => setDeleteId(detailUser.id)} className="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-red-50 hover:bg-red-100 text-red-600 text-sm font-semibold border border-red-200 transition-colors"><Trash2 className="w-4 h-4" />Hapus User</button>
                    </div>
                  </>
                );
              })()}
            </div>
          </div>
        </div>
      )}

      {/* ═══ DELETE CONFIRM ════════════════════════════════════════════════════ */}
      {deleteId && (
        <div className="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center z-[60] p-4">
          <div className="bg-white rounded-2xl p-6 w-full max-w-sm shadow-2xl border border-slate-100">
            <h3 className="font-black text-slate-900 mb-2">Hapus User?</h3>
            <p className="text-sm text-slate-500 mb-6">Akun ini akan dihapus permanen dan tidak bisa dipulihkan.</p>
            <div className="flex gap-3">
              <button onClick={() => setDeleteId(null)} className="flex-1 py-2.5 rounded-xl border border-slate-200 text-sm font-semibold text-slate-600 hover:bg-slate-50 transition-colors">Batal</button>
              <button onClick={() => handleDelete(deleteId)} className="flex-1 py-2.5 rounded-xl bg-red-600 hover:bg-red-500 text-white text-sm font-bold transition-colors">Hapus</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
