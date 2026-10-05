import { useState } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import {
  Plus, Calendar, Users, Trophy, Clock, Edit, Trash2, X, Save,
  CheckCircle, AlertCircle, Archive, Play, Eye, Search, Filter,
  TrendingUp, DollarSign, Layers, MoreVertical
} from 'lucide-react';
import { toast } from 'sonner';

type EventStatus = 'draft' | 'upcoming' | 'ongoing' | 'completed' | 'archived';
type EventType = 'tryout' | 'rasionalisasi' | 'drilling' | 'mini-tryout';

interface Event {
  id: string;
  name: string;
  type: EventType;
  description: string;
  startDate: string;
  endDate: string;
  duration: number;
  participants: number;
  maxParticipants: number | null;
  price: number;
  status: EventStatus;
  packageId: string;
  packageName: string;
  prizes: string;
  targetClass: string;
}

const initialEvents: Event[] = [
  { id: 'E001', name: 'Tryout Nasional SNBT #45', type: 'tryout', description: 'Tryout simulasi SNBT full dengan soal terbaru dan ranking nasional.', startDate: '2025-01-25', endDate: '2025-01-27', duration: 195, participants: 4521, maxParticipants: 10000, price: 50000, status: 'ongoing', packageId: 'pkg-1', packageName: 'Tryout SNBT Full Simulasi #1', prizes: 'Voucher belanja Rp 1jt, Rp 500rb, Rp 250rb', targetClass: 'Kelas 12' },
  { id: 'E002', name: 'Rasionalisasi Gelombang 1 2025', type: 'rasionalisasi', description: 'Sesi rasionalisasi PTN berbasis AI dengan analisis mendalam peluang masuk PTN.', startDate: '2025-02-01', endDate: '2025-02-05', duration: 90, participants: 0, maxParticipants: 5000, price: 25000, status: 'upcoming', packageId: 'pkg-2', packageName: 'Mini Tryout Penalaran Umum', prizes: '', targetClass: 'Kelas 12' },
  { id: 'E003', name: 'Drilling Matematika Intensif', type: 'drilling', description: 'Program drilling penalaran matematika intensif 12 hari.', startDate: '2025-01-20', endDate: '2025-01-31', duration: 60, participants: 2134, maxParticipants: null, price: 0, status: 'ongoing', packageId: 'pkg-3', packageName: 'Drilling Penalaran Matematika Level Sulit', prizes: '', targetClass: 'Kelas 11-12' },
  { id: 'E004', name: 'Tryout PTN Premium #44', type: 'tryout', description: 'Tryout premium dengan pembahasan video dan analitik mendalam.', startDate: '2025-01-15', endDate: '2025-01-18', duration: 195, participants: 5632, maxParticipants: 6000, price: 75000, status: 'completed', packageId: 'pkg-1', packageName: 'Tryout SNBT Full Simulasi #1', prizes: 'Laptop, Smartphone, Tablet', targetClass: 'Kelas 12' },
  { id: 'E005', name: 'Mini Tryout Weekend #12', type: 'mini-tryout', description: 'Mini tryout 40 soal setiap akhir pekan untuk pemanasan.', startDate: '2025-02-08', endDate: '2025-02-09', duration: 45, participants: 0, maxParticipants: null, price: 0, status: 'draft', packageId: 'pkg-2', packageName: 'Mini Tryout Penalaran Umum', prizes: '', targetClass: 'Kelas 10-12' },
];

const typeConfig: Record<EventType, { label: string; color: string; bgColor: string; icon: typeof Trophy }> = {
  tryout: { label: 'Tryout', color: 'text-indigo-700', bgColor: 'bg-indigo-100', icon: Trophy },
  rasionalisasi: { label: 'Rasionalisasi', color: 'text-purple-700', bgColor: 'bg-purple-100', icon: TrendingUp },
  drilling: { label: 'Drilling', color: 'text-orange-700', bgColor: 'bg-orange-100', icon: Layers },
  'mini-tryout': { label: 'Mini Tryout', color: 'text-blue-700', bgColor: 'bg-blue-100', icon: Play },
};

const statusConfig: Record<EventStatus, { label: string; color: string }> = {
  draft: { label: 'Draft', color: 'border-slate-400 text-slate-600 bg-slate-50' },
  upcoming: { label: 'Dijadwalkan', color: 'border-orange-400 text-orange-700 bg-orange-50' },
  ongoing: { label: 'Berlangsung', color: 'border-green-500 text-green-700 bg-green-50' },
  completed: { label: 'Selesai', color: 'border-blue-400 text-blue-700 bg-blue-50' },
  archived: { label: 'Diarsipkan', color: 'border-slate-300 text-slate-400 bg-slate-50' },
};

const packageOptions = [
  { id: 'pkg-1', name: 'Tryout SNBT Full Simulasi #1' },
  { id: 'pkg-2', name: 'Mini Tryout Penalaran Umum' },
  { id: 'pkg-3', name: 'Drilling Penalaran Matematika Level Sulit' },
  { id: 'pkg-4', name: 'Chapter Test: Literasi Bahasa Indonesia' },
  { id: 'pkg-5', name: 'Tryout SNBT Full Simulasi #2' },
];

const emptyEvent: Omit<Event, 'id' | 'participants'> = {
  name: '', type: 'tryout', description: '', startDate: '', endDate: '',
  duration: 195, maxParticipants: null, price: 0, status: 'draft',
  packageId: 'pkg-1', packageName: 'Tryout SNBT Full Simulasi #1',
  prizes: '', targetClass: 'Kelas 12',
};

export default function EventManagement() {
  const [events, setEvents] = useState<Event[]>(initialEvents);
  const [search, setSearch] = useState('');
  const [filterStatus, setFilterStatus] = useState<EventStatus | 'all'>('all');
  const [filterType, setFilterType] = useState<EventType | 'all'>('all');
  const [modalOpen, setModalOpen] = useState(false);
  const [editTarget, setEditTarget] = useState<Event | null>(null);
  const [form, setForm] = useState(emptyEvent);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [detailEvent, setDetailEvent] = useState<Event | null>(null);

  const filtered = events.filter(e => {
    const ms = search === '' || e.name.toLowerCase().includes(search.toLowerCase());
    const mSt = filterStatus === 'all' || e.status === filterStatus;
    const mTy = filterType === 'all' || e.type === filterType;
    return ms && mSt && mTy;
  });

  const openCreate = () => {
    setEditTarget(null);
    setForm(emptyEvent);
    setModalOpen(true);
  };

  const openEdit = (ev: Event) => {
    setEditTarget(ev);
    setForm({ name: ev.name, type: ev.type, description: ev.description, startDate: ev.startDate, endDate: ev.endDate, duration: ev.duration, maxParticipants: ev.maxParticipants, price: ev.price, status: ev.status, packageId: ev.packageId, packageName: ev.packageName, prizes: ev.prizes, targetClass: ev.targetClass });
    setModalOpen(true);
  };

  const handleSave = () => {
    if (!form.name.trim()) { toast.error('Nama event wajib diisi'); return; }
    if (!form.startDate) { toast.error('Tanggal mulai wajib diisi'); return; }
    if (editTarget) {
      setEvents(prev => prev.map(e => e.id === editTarget.id ? { ...e, ...form } : e));
      toast.success(`Event "${form.name}" berhasil diperbarui`);
    } else {
      setEvents(prev => [{ ...form, id: `E${Date.now()}`, participants: 0 }, ...prev]);
      toast.success(`Event "${form.name}" berhasil dibuat`);
    }
    setModalOpen(false);
  };

  const handleDelete = (id: string) => {
    const ev = events.find(e => e.id === id);
    setEvents(prev => prev.filter(e => e.id !== id));
    setDeleteId(null);
    toast.success(`Event "${ev?.name}" dihapus`);
  };

  const handleStatusChange = (id: string, status: EventStatus) => {
    setEvents(prev => prev.map(e => e.id === id ? { ...e, status } : e));
    toast.success('Status event diperbarui');
  };

  const setFormField = <K extends keyof typeof form>(k: K, v: (typeof form)[K]) =>
    setForm(f => ({ ...f, [k]: v }));

  const stats = {
    ongoing: events.filter(e => e.status === 'ongoing').length,
    upcoming: events.filter(e => e.status === 'upcoming').length,
    totalParticipants: events.reduce((s, e) => s + e.participants, 0),
    revenue: events.reduce((s, e) => s + e.price * e.participants, 0),
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card className="p-6">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div>
            <h2 className="text-2xl font-bold mb-1">Manajemen Event</h2>
            <p className="text-sm text-muted-foreground">Kelola tryout, rasionalisasi, dan drilling events</p>
          </div>
          <Button onClick={openCreate} className="gap-2 bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700">
            <Plus className="w-4 h-4" /> Buat Event Baru
          </Button>
        </div>
      </Card>

      {/* Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        {[
          { label: 'Sedang Berlangsung', val: stats.ongoing, icon: Play, color: 'border-l-green-500', iconBg: 'bg-green-100', iconColor: 'text-green-600' },
          { label: 'Terjadwal', val: stats.upcoming, icon: Clock, color: 'border-l-orange-500', iconBg: 'bg-orange-100', iconColor: 'text-orange-600' },
          { label: 'Total Peserta', val: stats.totalParticipants.toLocaleString('id-ID'), icon: Users, color: 'border-l-blue-500', iconBg: 'bg-blue-100', iconColor: 'text-blue-600' },
          { label: 'Revenue Event', val: `Rp ${(stats.revenue / 1e9).toFixed(1)}M`, icon: DollarSign, color: 'border-l-purple-500', iconBg: 'bg-purple-100', iconColor: 'text-purple-600' },
        ].map(({ label, val, icon: Icon, color, iconBg, iconColor }) => (
          <Card key={label} className={`p-4 border-l-4 ${color}`}>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground mb-1">{label}</p>
                <h3 className="text-2xl font-bold">{val}</h3>
              </div>
              <div className={`w-10 h-10 rounded-lg ${iconBg} flex items-center justify-center`}>
                <Icon className={`w-5 h-5 ${iconColor}`} />
              </div>
            </div>
          </Card>
        ))}
      </div>

      {/* Filters */}
      <Card className="p-4">
        <div className="flex flex-col sm:flex-row gap-3">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input placeholder="Cari event..." className="pl-10" value={search} onChange={e => setSearch(e.target.value)} />
          </div>
          <select className="px-3 py-2 border rounded-md text-sm bg-white" value={filterStatus} onChange={e => setFilterStatus(e.target.value as EventStatus | 'all')}>
            <option value="all">Semua Status</option>
            <option value="draft">Draft</option>
            <option value="upcoming">Dijadwalkan</option>
            <option value="ongoing">Berlangsung</option>
            <option value="completed">Selesai</option>
            <option value="archived">Diarsipkan</option>
          </select>
          <select className="px-3 py-2 border rounded-md text-sm bg-white" value={filterType} onChange={e => setFilterType(e.target.value as EventType | 'all')}>
            <option value="all">Semua Tipe</option>
            <option value="tryout">Tryout</option>
            <option value="rasionalisasi">Rasionalisasi</option>
            <option value="drilling">Drilling</option>
            <option value="mini-tryout">Mini Tryout</option>
          </select>
        </div>
      </Card>

      {/* Events list */}
      <div className="space-y-4">
        {filtered.length === 0 && (
          <Card className="p-10 text-center">
            <Calendar className="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" />
            <p className="text-muted-foreground">Tidak ada event yang cocok dengan filter</p>
          </Card>
        )}
        {filtered.map((ev) => {
          const tc = typeConfig[ev.type];
          const sc = statusConfig[ev.status];
          const TypeIcon = tc.icon;
          const pct = ev.maxParticipants ? (ev.participants / ev.maxParticipants) * 100 : null;
          return (
            <Card key={ev.id} className="p-5 hover:shadow-md transition-shadow">
              <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
                <div className="flex-1">
                  <div className="flex items-start gap-3 mb-3">
                    <div className={`w-11 h-11 rounded-xl ${tc.bgColor} flex items-center justify-center shrink-0`}>
                      <TypeIcon className={`w-5 h-5 ${tc.color}`} />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 flex-wrap mb-1">
                        <h3 className="font-bold text-slate-900">{ev.name}</h3>
                        <Badge variant="outline" className={sc.color}>{sc.label}</Badge>
                        <Badge variant="outline" className="capitalize">{tc.label}</Badge>
                      </div>
                      <p className="text-xs text-muted-foreground line-clamp-1">{ev.description}</p>
                      <div className="flex flex-wrap items-center gap-3 mt-1.5 text-xs text-muted-foreground">
                        <span className="flex items-center gap-1"><Calendar className="w-3.5 h-3.5" />{ev.startDate} – {ev.endDate}</span>
                        <span className="flex items-center gap-1"><Clock className="w-3.5 h-3.5" />{ev.duration} menit</span>
                        <span className="flex items-center gap-1"><Users className="w-3.5 h-3.5" />{ev.participants.toLocaleString('id-ID')} peserta</span>
                        <span className="text-indigo-600 font-medium">{ev.price === 0 ? 'Gratis' : `Rp ${(ev.price / 1000).toFixed(0)}K`}</span>
                      </div>
                    </div>
                  </div>

                  {pct !== null && (
                    <div className="mt-2">
                      <div className="flex justify-between text-xs text-muted-foreground mb-1">
                        <span>Kapasitas ({Math.round(pct)}%)</span>
                        <span>{ev.participants.toLocaleString()}/{ev.maxParticipants!.toLocaleString()}</span>
                      </div>
                      <div className="h-1.5 bg-slate-100 rounded-full overflow-hidden">
                        <div className="h-full bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full" style={{ width: `${pct}%` }} />
                      </div>
                    </div>
                  )}
                </div>

                {/* Actions */}
                <div className="flex items-center gap-2 flex-wrap">
                  {ev.status === 'draft' && (
                    <Button size="sm" variant="outline" className="gap-1.5 text-green-700 border-green-300 hover:bg-green-50" onClick={() => handleStatusChange(ev.id, 'upcoming')}>
                      <CheckCircle className="w-3.5 h-3.5" /> Publish
                    </Button>
                  )}
                  {ev.status === 'upcoming' && (
                    <Button size="sm" variant="outline" className="gap-1.5 text-blue-700 border-blue-300 hover:bg-blue-50" onClick={() => handleStatusChange(ev.id, 'ongoing')}>
                      <Play className="w-3.5 h-3.5" /> Mulai
                    </Button>
                  )}
                  {ev.status === 'ongoing' && (
                    <Button size="sm" variant="outline" className="gap-1.5 text-orange-700 border-orange-300 hover:bg-orange-50" onClick={() => handleStatusChange(ev.id, 'completed')}>
                      <CheckCircle className="w-3.5 h-3.5" /> Selesaikan
                    </Button>
                  )}
                  {(ev.status === 'completed' || ev.status === 'upcoming') && (
                    <Button size="sm" variant="outline" className="gap-1.5" onClick={() => handleStatusChange(ev.id, 'archived')}>
                      <Archive className="w-3.5 h-3.5" /> Arsipkan
                    </Button>
                  )}
                  <Button size="sm" variant="outline" className="gap-1.5" onClick={() => setDetailEvent(ev)}>
                    <Eye className="w-3.5 h-3.5" /> Detail
                  </Button>
                  <Button size="sm" variant="outline" className="gap-1.5" onClick={() => openEdit(ev)}>
                    <Edit className="w-3.5 h-3.5" /> Edit
                  </Button>
                  <Button size="sm" variant="outline" className="gap-1.5 text-red-600 border-red-200 hover:bg-red-50" onClick={() => setDeleteId(ev.id)}>
                    <Trash2 className="w-3.5 h-3.5" />
                  </Button>
                </div>
              </div>
            </Card>
          );
        })}
      </div>

      {/* ─── Create/Edit Modal ─── */}
      {modalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4 overflow-y-auto">
          <div className="bg-white rounded-2xl w-full max-w-2xl my-4 shadow-2xl">
            <div className="flex items-center justify-between p-6 border-b">
              <h3 className="text-lg font-bold">{editTarget ? 'Edit Event' : 'Buat Event Baru'}</h3>
              <button onClick={() => setModalOpen(false)} className="p-2 rounded-xl hover:bg-slate-100 transition-colors"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-6 space-y-5 overflow-y-auto max-h-[70vh]">
              <div>
                <Label>Nama Event *</Label>
                <Input className="mt-1.5" value={form.name} onChange={e => setFormField('name', e.target.value)} placeholder="Tryout Nasional SNBT #46" />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>Tipe Event</Label>
                  <select className="w-full mt-1.5 px-3 py-2 border rounded-md text-sm" value={form.type} onChange={e => setFormField('type', e.target.value as EventType)}>
                    <option value="tryout">Tryout Full</option>
                    <option value="mini-tryout">Mini Tryout</option>
                    <option value="rasionalisasi">Rasionalisasi PTN</option>
                    <option value="drilling">Drilling</option>
                  </select>
                </div>
                <div>
                  <Label>Target Kelas</Label>
                  <select className="w-full mt-1.5 px-3 py-2 border rounded-md text-sm" value={form.targetClass} onChange={e => setFormField('targetClass', e.target.value)}>
                    {['Kelas 10', 'Kelas 11', 'Kelas 12', 'Kelas 11-12', 'Kelas 10-12'].map(c => <option key={c}>{c}</option>)}
                  </select>
                </div>
              </div>

              <div>
                <Label>Deskripsi</Label>
                <textarea
                  className="w-full mt-1.5 px-3 py-2 border rounded-md text-sm resize-none h-20 focus:outline-none focus:ring-2 focus:ring-indigo-300"
                  value={form.description}
                  onChange={e => setFormField('description', e.target.value)}
                  placeholder="Deskripsi singkat event..."
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>Tanggal Mulai *</Label>
                  <Input type="date" className="mt-1.5" value={form.startDate} onChange={e => setFormField('startDate', e.target.value)} />
                </div>
                <div>
                  <Label>Tanggal Selesai</Label>
                  <Input type="date" className="mt-1.5" value={form.endDate} onChange={e => setFormField('endDate', e.target.value)} />
                </div>
              </div>

              <div className="grid grid-cols-3 gap-4">
                <div>
                  <Label>Durasi (menit)</Label>
                  <Input type="number" className="mt-1.5" value={form.duration} onChange={e => setFormField('duration', Number(e.target.value))} />
                </div>
                <div>
                  <Label>Maks. Peserta</Label>
                  <Input type="number" className="mt-1.5" value={form.maxParticipants ?? ''} placeholder="∞" onChange={e => setFormField('maxParticipants', e.target.value ? Number(e.target.value) : null)} />
                </div>
                <div>
                  <Label>Harga (Rp)</Label>
                  <Input type="number" className="mt-1.5" value={form.price} onChange={e => setFormField('price', Number(e.target.value))} />
                </div>
              </div>

              <div>
                <Label>Paket Soal</Label>
                <select
                  className="w-full mt-1.5 px-3 py-2 border rounded-md text-sm"
                  value={form.packageId}
                  onChange={e => {
                    const pkg = packageOptions.find(p => p.id === e.target.value);
                    setFormField('packageId', e.target.value);
                    if (pkg) setFormField('packageName', pkg.name);
                  }}
                >
                  {packageOptions.map(p => <option key={p.id} value={p.id}>{p.name}</option>)}
                </select>
              </div>

              <div>
                <Label>Hadiah (opsional)</Label>
                <Input className="mt-1.5" value={form.prizes} onChange={e => setFormField('prizes', e.target.value)} placeholder="Laptop, Voucher Rp 1jt, ..." />
              </div>

              <div>
                <Label>Status</Label>
                <select className="w-full mt-1.5 px-3 py-2 border rounded-md text-sm" value={form.status} onChange={e => setFormField('status', e.target.value as EventStatus)}>
                  <option value="draft">Draft</option>
                  <option value="upcoming">Jadwalkan (Upcoming)</option>
                  <option value="ongoing">Mulai Sekarang</option>
                  <option value="completed">Tandai Selesai</option>
                  <option value="archived">Arsipkan</option>
                </select>
              </div>
            </div>
            <div className="flex gap-3 p-6 border-t">
              <Button variant="outline" className="flex-1" onClick={() => setModalOpen(false)}>Batal</Button>
              <Button className="flex-1 bg-gradient-to-r from-purple-600 to-pink-600 gap-2" onClick={handleSave}>
                <Save className="w-4 h-4" /> {editTarget ? 'Simpan' : 'Buat Event'}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* ─── Detail drawer ─── */}
      {detailEvent && (
        <div className="fixed inset-0 bg-black/40 flex items-center justify-end z-50 p-4">
          <div className="bg-white h-full w-full max-w-md rounded-2xl shadow-2xl overflow-y-auto">
            <div className="flex items-center justify-between p-5 border-b">
              <h3 className="font-bold text-slate-900 text-base">Detail Event</h3>
              <button onClick={() => setDetailEvent(null)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-4 h-4" /></button>
            </div>
            <div className="p-5 space-y-5">
              <div>
                <div className="flex items-center gap-2 flex-wrap mb-2">
                  <Badge variant="outline" className={statusConfig[detailEvent.status].color}>{statusConfig[detailEvent.status].label}</Badge>
                  <Badge variant="outline">{typeConfig[detailEvent.type].label}</Badge>
                </div>
                <h2 className="text-xl font-bold text-slate-900">{detailEvent.name}</h2>
                <p className="text-sm text-muted-foreground mt-1">{detailEvent.description}</p>
              </div>
              {[
                ['Tanggal', `${detailEvent.startDate} – ${detailEvent.endDate}`],
                ['Durasi', `${detailEvent.duration} menit`],
                ['Target', detailEvent.targetClass],
                ['Paket Soal', detailEvent.packageName],
                ['Harga', detailEvent.price === 0 ? 'Gratis' : `Rp ${detailEvent.price.toLocaleString('id-ID')}`],
                ['Peserta', `${detailEvent.participants.toLocaleString()} / ${detailEvent.maxParticipants?.toLocaleString() ?? '∞'}`],
                ['Hadiah', detailEvent.prizes || '—'],
              ].map(([label, val]) => (
                <div key={label} className="flex justify-between py-2.5 border-b last:border-0">
                  <span className="text-sm text-muted-foreground">{label}</span>
                  <span className="text-sm font-semibold text-slate-800 text-right max-w-[60%]">{val}</span>
                </div>
              ))}
              {detailEvent.maxParticipants && (
                <div>
                  <div className="flex justify-between text-xs mb-1.5 text-muted-foreground">
                    <span>Kapasitas terisi</span>
                    <span>{Math.round((detailEvent.participants / detailEvent.maxParticipants) * 100)}%</span>
                  </div>
                  <div className="h-2 bg-slate-100 rounded-full overflow-hidden">
                    <div className="h-full bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full" style={{ width: `${(detailEvent.participants / detailEvent.maxParticipants) * 100}%` }} />
                  </div>
                </div>
              )}
              <div className="flex gap-2 pt-2">
                <Button variant="outline" className="flex-1 gap-2" onClick={() => { openEdit(detailEvent); setDetailEvent(null); }}>
                  <Edit className="w-4 h-4" /> Edit
                </Button>
                <Button variant="outline" className="flex-1 gap-2 text-red-600 border-red-200 hover:bg-red-50" onClick={() => { setDeleteId(detailEvent.id); setDetailEvent(null); }}>
                  <Trash2 className="w-4 h-4" /> Hapus
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ─── Delete confirm ─── */}
      {deleteId && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-2xl p-6 w-full max-w-sm shadow-2xl">
            <h3 className="font-bold text-slate-900 mb-2">Hapus Event?</h3>
            <p className="text-sm text-muted-foreground mb-6">Tindakan ini tidak bisa dibatalkan. Semua data event akan hilang permanen.</p>
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
