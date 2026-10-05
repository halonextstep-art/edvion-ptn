import { useState } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import {
  Plus, Edit2, Trash2, Eye, EyeOff, Package, DollarSign,
  Tag, CheckCircle2, X, GripVertical, Percent, Clock, ChevronRight
} from 'lucide-react';
import { toast } from 'sonner';

interface PackageItem {
  id: string;
  name: string;
  type: string;
  originalPrice: number;
  salePrice: number;
  discount: number;
  validity: string;
  features: string[];
  badge: string;
  emoji: string;
  gradient: string;
  accentColor: string;
  active: boolean;
  sortOrder: number;
  soldCount: number;
}

const INIT_PACKAGES: PackageItem[] = [
  {
    id: 'p1',
    name: '5 Kuota Tryout SNBT',
    type: 'TKA',
    originalPrice: 175000,
    salePrice: 105000,
    discount: 40,
    validity: '1 tahun',
    features: ['5x Tryout SNBT Full Simulasi', 'Kuota berlaku 1 tahun sejak pembelian', 'Pembahasan lengkap tiap soal', 'Analisis skor otomatis'],
    badge: '',
    emoji: '📘',
    gradient: 'from-sky-200 to-blue-100',
    accentColor: '#3b82f6',
    active: true,
    sortOrder: 1,
    soldCount: 342,
  },
  {
    id: 'p2',
    name: '12 Kuota Tryout SNBT & TPS',
    type: 'Full',
    originalPrice: 350000,
    salePrice: 210000,
    discount: 40,
    validity: '1 tahun',
    features: ['12x Tryout Full Simulasi', 'TPS + Literasi + Penalaran Mat.', 'Kuota berlaku 1 tahun sejak pembelian', 'Video pembahasan eksklusif'],
    badge: 'Terpopuler',
    emoji: '🎯',
    gradient: 'from-violet-200 to-purple-100',
    accentColor: '#7c3aed',
    active: true,
    sortOrder: 2,
    soldCount: 891,
  },
  {
    id: 'p3',
    name: '20 Kuota Tryout SNBT Full',
    type: 'Premium',
    originalPrice: 580000,
    salePrice: 348000,
    discount: 40,
    validity: '1 tahun',
    features: ['20x Tryout Full Simulasi SNBT', 'Semua subtes SNBT lengkap', 'Kuota berlaku 1 tahun sejak pembelian', 'Rasionalisasi PTN gratis'],
    badge: 'Best Value',
    emoji: '🏆',
    gradient: 'from-amber-200 to-orange-100',
    accentColor: '#f59e0b',
    active: true,
    sortOrder: 3,
    soldCount: 527,
  },
  {
    id: 'p4',
    name: 'Akses Drilling Intensif',
    type: 'Drilling',
    originalPrice: 195000,
    salePrice: 117000,
    discount: 40,
    validity: '1 tahun',
    features: ['Drilling tanpa batas 365 hari', '10.000+ soal per kategori', 'Adaptive difficulty AI', 'Progress tracking real-time'],
    badge: '',
    emoji: '⚡',
    gradient: 'from-emerald-200 to-teal-100',
    accentColor: '#10b981',
    active: true,
    sortOrder: 4,
    soldCount: 215,
  },
  {
    id: 'p5',
    name: 'Paket Lengkap Elite',
    type: 'Elite',
    originalPrice: 750000,
    salePrice: 450000,
    discount: 40,
    validity: '1 tahun',
    features: ['Tryout full simulasi tanpa batas', 'Drilling intensif tanpa batas', 'Rasionalisasi semua PTN', 'Mentoring 1-on-1 / minggu'],
    badge: 'All-In-One',
    emoji: '💎',
    gradient: 'from-rose-200 to-pink-100',
    accentColor: '#ec4899',
    active: false,
    sortOrder: 5,
    soldCount: 98,
  },
];

const EMOJI_OPTIONS = ['📘', '🎯', '🏆', '⚡', '💎', '📚', '🚀', '🌟', '🔥', '🎓'];
const GRADIENT_OPTIONS = [
  { label: 'Biru', value: 'from-sky-200 to-blue-100' },
  { label: 'Ungu', value: 'from-violet-200 to-purple-100' },
  { label: 'Kuning', value: 'from-amber-200 to-orange-100' },
  { label: 'Hijau', value: 'from-emerald-200 to-teal-100' },
  { label: 'Merah Muda', value: 'from-rose-200 to-pink-100' },
  { label: 'Cyan', value: 'from-cyan-200 to-sky-100' },
  { label: 'Indigo', value: 'from-indigo-200 to-blue-100' },
];

function fmt(n: number) {
  return n.toLocaleString('id-ID');
}

const BLANK: PackageItem = {
  id: '',
  name: '',
  type: 'SNBT',
  originalPrice: 0,
  salePrice: 0,
  discount: 0,
  validity: '1 tahun',
  features: [''],
  badge: '',
  emoji: '📘',
  gradient: 'from-sky-200 to-blue-100',
  accentColor: '#3b82f6',
  active: true,
  sortOrder: 99,
  soldCount: 0,
};

export default function AdminPackageManagement() {
  const [packages, setPackages] = useState<PackageItem[]>(INIT_PACKAGES);
  const [showModal, setShowModal] = useState(false);
  const [editTarget, setEditTarget] = useState<PackageItem | null>(null);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [form, setForm] = useState<PackageItem>(BLANK);
  const [view, setView] = useState<'grid' | 'table'>('grid');

  const totalRevenue = packages.reduce((sum, p) => sum + p.salePrice * p.soldCount, 0);
  const activeCount = packages.filter(p => p.active).length;

  function openCreate() {
    setForm({ ...BLANK, id: `p${Date.now()}`, sortOrder: packages.length + 1 });
    setEditTarget(null);
    setShowModal(true);
  }

  function openEdit(pkg: PackageItem) {
    setForm({ ...pkg });
    setEditTarget(pkg);
    setShowModal(true);
  }

  function handleSave() {
    if (!form.name.trim()) { toast.error('Nama paket wajib diisi'); return; }
    if (form.originalPrice <= 0) { toast.error('Harga asli harus lebih dari 0'); return; }
    if (form.salePrice <= 0) { toast.error('Harga jual harus lebih dari 0'); return; }
    const finalForm = {
      ...form,
      discount: Math.round((1 - form.salePrice / form.originalPrice) * 100),
      features: form.features.filter(f => f.trim() !== ''),
    };
    if (editTarget) {
      setPackages(prev => prev.map(p => p.id === editTarget.id ? finalForm : p));
      toast.success('Paket berhasil diperbarui');
    } else {
      setPackages(prev => [...prev, finalForm]);
      toast.success('Paket baru berhasil ditambahkan');
    }
    setShowModal(false);
  }

  function toggleActive(id: string) {
    setPackages(prev => prev.map(p => p.id === id ? { ...p, active: !p.active } : p));
    const pkg = packages.find(p => p.id === id);
    toast.success(`Paket "${pkg?.name}" ${pkg?.active ? 'dinonaktifkan' : 'diaktifkan'}`);
  }

  function handleDelete(id: string) {
    setPackages(prev => prev.filter(p => p.id !== id));
    setDeleteId(null);
    toast.success('Paket berhasil dihapus');
  }

  function setFeature(idx: number, val: string) {
    setForm(f => {
      const features = [...f.features];
      features[idx] = val;
      return { ...f, features };
    });
  }

  function addFeature() {
    setForm(f => ({ ...f, features: [...f.features, ''] }));
  }

  function removeFeature(idx: number) {
    setForm(f => ({ ...f, features: f.features.filter((_, i) => i !== idx) }));
  }

  const sorted = [...packages].sort((a, b) => a.sortOrder - b.sortOrder);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-black text-slate-900">Manajemen Paket</h2>
          <p className="text-sm text-muted-foreground">Kelola paket tryout dan harga yang tampil di landing page</p>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex border rounded-lg overflow-hidden">
            <button
              onClick={() => setView('grid')}
              className={`px-3 py-1.5 text-xs font-semibold transition-colors ${view === 'grid' ? 'bg-indigo-600 text-white' : 'text-slate-600 hover:bg-slate-50'}`}
            >
              Grid
            </button>
            <button
              onClick={() => setView('table')}
              className={`px-3 py-1.5 text-xs font-semibold transition-colors ${view === 'table' ? 'bg-indigo-600 text-white' : 'text-slate-600 hover:bg-slate-50'}`}
            >
              Tabel
            </button>
          </div>
          <Button className="bg-gradient-to-r from-indigo-600 to-purple-600 gap-2" onClick={openCreate}>
            <Plus className="w-4 h-4" />
            Tambah Paket
          </Button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        {[
          { label: 'Total Paket', value: packages.length, icon: Package, color: 'bg-indigo-100 text-indigo-600' },
          { label: 'Paket Aktif', value: activeCount, icon: Eye, color: 'bg-emerald-100 text-emerald-600' },
          { label: 'Total Terjual', value: `${packages.reduce((s, p) => s + p.soldCount, 0).toLocaleString()}x`, icon: Tag, color: 'bg-amber-100 text-amber-600' },
          { label: 'Total Revenue', value: `Rp${(totalRevenue / 1000000).toFixed(1)}jt`, icon: DollarSign, color: 'bg-rose-100 text-rose-600' },
        ].map(s => (
          <Card key={s.label} className="p-4">
            <div className="flex items-center gap-3">
              <div className={`w-10 h-10 rounded-xl flex items-center justify-center ${s.color}`}>
                <s.icon className="w-5 h-5" />
              </div>
              <div>
                <p className="text-xs text-muted-foreground">{s.label}</p>
                <p className="text-xl font-black text-slate-900">{s.value}</p>
              </div>
            </div>
          </Card>
        ))}
      </div>

      {/* Preview Strip — same as landing page */}
      <Card className="p-5">
        <p className="text-sm font-bold text-slate-700 mb-3">Preview Landing Page</p>
        <div className="flex gap-3 overflow-x-auto pb-2" style={{ scrollbarWidth: 'none' }}>
          {sorted.filter(p => p.active).map(pkg => (
            <div key={pkg.id} className="flex-shrink-0 w-48 bg-white rounded-xl overflow-hidden shadow-md border relative">
              <div className="absolute top-2 left-2 z-10 w-9 h-9 rounded-full bg-orange-500 flex flex-col items-center justify-center">
                <span className="text-white font-black text-[10px] leading-none">{pkg.discount}%</span>
                <span className="text-white/80 text-[8px] leading-none">OFF</span>
              </div>
              {pkg.badge && (
                <div className="absolute top-2 right-2 z-10 text-white text-[9px] font-black px-1.5 py-0.5 rounded-full" style={{ backgroundColor: pkg.accentColor }}>
                  {pkg.badge}
                </div>
              )}
              <div className={`h-24 bg-gradient-to-br ${pkg.gradient} flex items-end justify-center pb-1`}>
                <span className="text-5xl">{pkg.emoji}</span>
              </div>
              <div className="p-3">
                <p className="font-bold text-slate-800 text-xs mb-1 leading-tight line-clamp-2">{pkg.name}</p>
                <p className="text-slate-400 text-[10px] line-through">Rp{fmt(pkg.originalPrice)}</p>
                <p className="font-black text-sm text-orange-500">Rp{fmt(pkg.salePrice)}</p>
              </div>
            </div>
          ))}
          <div className="flex-shrink-0 w-28 rounded-xl flex flex-col items-center justify-center p-3 gap-2 cursor-default" style={{ background: 'linear-gradient(135deg, #0d9488, #0891b2)' }}>
            <span className="text-3xl">📦</span>
            <p className="text-white font-bold text-center text-[10px] leading-tight">Lihat semua paket</p>
            <div className="w-7 h-7 rounded-full bg-white/20 flex items-center justify-center">
              <ChevronRight className="w-3.5 h-3.5 text-white" />
            </div>
          </div>
        </div>
      </Card>

      {/* Grid View */}
      {view === 'grid' && (
        <div className="grid sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-5">
          {sorted.map(pkg => (
            <Card key={pkg.id} className={`overflow-hidden flex flex-col ${!pkg.active ? 'opacity-60' : ''}`}>
              {/* Card illustration */}
              <div className={`relative h-32 bg-gradient-to-br ${pkg.gradient} flex items-end justify-center pb-1`}>
                <div className="absolute top-3 left-3 w-10 h-10 rounded-full bg-orange-500 flex flex-col items-center justify-center">
                  <span className="text-white font-black text-[10px] leading-none">{pkg.discount}%</span>
                  <span className="text-white/80 text-[8px] leading-none">OFF</span>
                </div>
                {pkg.badge && (
                  <div className="absolute top-3 right-3 text-white text-[9px] font-black px-1.5 py-0.5 rounded-full" style={{ backgroundColor: pkg.accentColor }}>
                    {pkg.badge}
                  </div>
                )}
                <span className="text-5xl">{pkg.emoji}</span>
              </div>

              <div className="p-4 flex flex-col flex-1">
                <div className="flex items-start justify-between gap-2 mb-2">
                  <h3 className="font-bold text-slate-900 text-sm leading-snug">{pkg.name}</h3>
                  <Badge variant="outline" className="text-[9px] shrink-0">{pkg.type}</Badge>
                </div>

                <div className="mb-3">
                  <p className="text-slate-400 text-xs line-through">Rp{fmt(pkg.originalPrice)}</p>
                  <p className="font-black text-lg text-orange-500 leading-none">Rp{fmt(pkg.salePrice)}</p>
                </div>

                <ul className="space-y-1 mb-3 flex-1">
                  {pkg.features.slice(0, 3).map(f => (
                    <li key={f} className="flex items-start gap-1.5 text-[11px] text-slate-600">
                      <CheckCircle2 className="w-3 h-3 text-emerald-500 mt-0.5 shrink-0" />
                      {f}
                    </li>
                  ))}
                  {pkg.features.length > 3 && (
                    <li className="text-[11px] text-muted-foreground pl-4">+{pkg.features.length - 3} lainnya</li>
                  )}
                </ul>

                <div className="flex items-center justify-between text-xs text-muted-foreground mb-3 pt-2 border-t">
                  <span className="flex items-center gap-1"><Clock className="w-3 h-3" />{pkg.validity}</span>
                  <span className="flex items-center gap-1"><Tag className="w-3 h-3" />{pkg.soldCount}x terjual</span>
                </div>

                {/* Actions */}
                <div className="flex gap-2">
                  <Button size="sm" variant="outline" className="flex-1 gap-1 text-xs h-8" onClick={() => openEdit(pkg)}>
                    <Edit2 className="w-3 h-3" />Edit
                  </Button>
                  <button
                    onClick={() => toggleActive(pkg.id)}
                    className={`w-8 h-8 rounded-lg flex items-center justify-center border transition-colors ${pkg.active ? 'bg-emerald-50 border-emerald-200 text-emerald-600 hover:bg-emerald-100' : 'bg-slate-50 border-slate-200 text-slate-400 hover:bg-slate-100'}`}
                    title={pkg.active ? 'Nonaktifkan' : 'Aktifkan'}
                  >
                    {pkg.active ? <Eye className="w-3.5 h-3.5" /> : <EyeOff className="w-3.5 h-3.5" />}
                  </button>
                  <button
                    onClick={() => setDeleteId(pkg.id)}
                    className="w-8 h-8 rounded-lg border border-red-200 bg-red-50 text-red-500 hover:bg-red-100 flex items-center justify-center transition-colors"
                    title="Hapus"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Table View */}
      {view === 'table' && (
        <Card className="overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b bg-slate-50">
                  <th className="text-left px-4 py-3 font-semibold text-slate-600 w-8"></th>
                  <th className="text-left px-4 py-3 font-semibold text-slate-600">Paket</th>
                  <th className="text-left px-4 py-3 font-semibold text-slate-600">Tipe</th>
                  <th className="text-right px-4 py-3 font-semibold text-slate-600">Harga Asli</th>
                  <th className="text-right px-4 py-3 font-semibold text-slate-600">Harga Jual</th>
                  <th className="text-center px-4 py-3 font-semibold text-slate-600">Diskon</th>
                  <th className="text-right px-4 py-3 font-semibold text-slate-600">Terjual</th>
                  <th className="text-center px-4 py-3 font-semibold text-slate-600">Status</th>
                  <th className="text-center px-4 py-3 font-semibold text-slate-600">Aksi</th>
                </tr>
              </thead>
              <tbody>
                {sorted.map(pkg => (
                  <tr key={pkg.id} className={`border-b hover:bg-slate-50 transition-colors ${!pkg.active ? 'opacity-60' : ''}`}>
                    <td className="px-4 py-3 text-slate-300 cursor-grab">
                      <GripVertical className="w-4 h-4" />
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        <span className="text-xl">{pkg.emoji}</span>
                        <div>
                          <p className="font-semibold text-slate-900 text-sm">{pkg.name}</p>
                          {pkg.badge && (
                            <span className="text-[10px] font-bold px-1.5 py-0.5 rounded-full text-white" style={{ backgroundColor: pkg.accentColor }}>{pkg.badge}</span>
                          )}
                        </div>
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <Badge variant="outline" className="text-xs">{pkg.type}</Badge>
                    </td>
                    <td className="px-4 py-3 text-right text-slate-400 text-xs line-through">Rp{fmt(pkg.originalPrice)}</td>
                    <td className="px-4 py-3 text-right font-bold text-orange-500">Rp{fmt(pkg.salePrice)}</td>
                    <td className="px-4 py-3 text-center">
                      <span className="bg-orange-100 text-orange-600 font-bold text-xs px-2 py-0.5 rounded-full">{pkg.discount}%</span>
                    </td>
                    <td className="px-4 py-3 text-right text-slate-700">{pkg.soldCount.toLocaleString()}</td>
                    <td className="px-4 py-3 text-center">
                      <button
                        onClick={() => toggleActive(pkg.id)}
                        className={`text-xs font-bold px-2.5 py-1 rounded-full transition-colors ${pkg.active ? 'bg-emerald-100 text-emerald-700 hover:bg-emerald-200' : 'bg-slate-100 text-slate-500 hover:bg-slate-200'}`}
                      >
                        {pkg.active ? 'Aktif' : 'Nonaktif'}
                      </button>
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center justify-center gap-1">
                        <button onClick={() => openEdit(pkg)} className="p-1.5 rounded hover:bg-slate-100 text-slate-600 transition-colors"><Edit2 className="w-3.5 h-3.5" /></button>
                        <button onClick={() => setDeleteId(pkg.id)} className="p-1.5 rounded hover:bg-red-50 text-red-500 transition-colors"><Trash2 className="w-3.5 h-3.5" /></button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      )}

      {/* ── Create / Edit Modal ── */}
      {showModal && (
        <div className="fixed inset-0 z-50 flex items-start justify-end">
          <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" onClick={() => setShowModal(false)} />
          <div className="relative w-full max-w-lg h-full bg-white shadow-2xl overflow-y-auto flex flex-col">
            {/* Header */}
            <div className="sticky top-0 z-10 bg-white border-b px-6 py-4 flex items-center justify-between">
              <h3 className="font-black text-slate-900">{editTarget ? 'Edit Paket' : 'Tambah Paket Baru'}</h3>
              <button onClick={() => setShowModal(false)} className="p-2 hover:bg-slate-100 rounded-lg"><X className="w-4 h-4" /></button>
            </div>

            <div className="flex-1 p-6 space-y-5">
              {/* Preview mini */}
              <div className="flex justify-center">
                <div className="w-40 bg-white rounded-xl overflow-hidden shadow-lg border relative">
                  <div className="absolute top-2 left-2 z-10 w-9 h-9 rounded-full bg-orange-500 flex flex-col items-center justify-center">
                    <span className="text-white font-black text-[10px] leading-none">
                      {form.originalPrice > 0 ? Math.round((1 - form.salePrice / form.originalPrice) * 100) : 0}%
                    </span>
                    <span className="text-white/80 text-[8px] leading-none">OFF</span>
                  </div>
                  <div className={`h-24 bg-gradient-to-br ${form.gradient} flex items-end justify-center pb-1`}>
                    <span className="text-5xl">{form.emoji}</span>
                  </div>
                  <div className="p-3">
                    <p className="font-bold text-slate-800 text-[11px] leading-tight mb-1 line-clamp-2">{form.name || 'Nama Paket'}</p>
                    <p className="text-slate-400 text-[10px] line-through">{form.originalPrice > 0 ? `Rp${fmt(form.originalPrice)}` : 'Harga asli'}</p>
                    <p className="font-black text-sm text-orange-500">{form.salePrice > 0 ? `Rp${fmt(form.salePrice)}` : 'Harga jual'}</p>
                  </div>
                </div>
              </div>

              {/* Name */}
              <div>
                <label className="block text-sm font-semibold text-slate-700 mb-1.5">Nama Paket *</label>
                <input
                  className="w-full border rounded-xl px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300"
                  placeholder="Contoh: 12 Kuota Tryout SNBT & TPS"
                  value={form.name}
                  onChange={e => setForm(f => ({ ...f, name: e.target.value }))}
                />
              </div>

              {/* Type & Badge */}
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-sm font-semibold text-slate-700 mb-1.5">Tipe</label>
                  <select
                    className="w-full border rounded-xl px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300"
                    value={form.type}
                    onChange={e => setForm(f => ({ ...f, type: e.target.value }))}
                  >
                    {['TKA', 'SNBT', 'Full', 'Premium', 'Drilling', 'Elite'].map(t => (
                      <option key={t} value={t}>{t}</option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-semibold text-slate-700 mb-1.5">Badge (opsional)</label>
                  <input
                    className="w-full border rounded-xl px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300"
                    placeholder="Terpopuler / Best Value"
                    value={form.badge}
                    onChange={e => setForm(f => ({ ...f, badge: e.target.value }))}
                  />
                </div>
              </div>

              {/* Prices */}
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-sm font-semibold text-slate-700 mb-1.5">Harga Asli (Rp) *</label>
                  <input
                    type="number"
                    className="w-full border rounded-xl px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300"
                    placeholder="175000"
                    value={form.originalPrice || ''}
                    onChange={e => setForm(f => ({ ...f, originalPrice: Number(e.target.value) }))}
                  />
                </div>
                <div>
                  <label className="block text-sm font-semibold text-slate-700 mb-1.5">Harga Jual (Rp) *</label>
                  <input
                    type="number"
                    className="w-full border rounded-xl px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300"
                    placeholder="105000"
                    value={form.salePrice || ''}
                    onChange={e => setForm(f => ({ ...f, salePrice: Number(e.target.value) }))}
                  />
                </div>
              </div>
              {form.originalPrice > 0 && form.salePrice > 0 && (
                <div className="flex items-center gap-2 -mt-3 text-sm">
                  <Percent className="w-4 h-4 text-orange-500" />
                  <span className="text-orange-600 font-bold">
                    Diskon {Math.round((1 - form.salePrice / form.originalPrice) * 100)}%
                  </span>
                  <span className="text-slate-500">— hemat Rp{fmt(form.originalPrice - form.salePrice)}</span>
                </div>
              )}

              {/* Validity */}
              <div>
                <label className="block text-sm font-semibold text-slate-700 mb-1.5">Masa Berlaku</label>
                <input
                  className="w-full border rounded-xl px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300"
                  placeholder="1 tahun"
                  value={form.validity}
                  onChange={e => setForm(f => ({ ...f, validity: e.target.value }))}
                />
              </div>

              {/* Emoji */}
              <div>
                <label className="block text-sm font-semibold text-slate-700 mb-1.5">Ikon</label>
                <div className="flex gap-2 flex-wrap">
                  {EMOJI_OPTIONS.map(em => (
                    <button
                      key={em}
                      onClick={() => setForm(f => ({ ...f, emoji: em }))}
                      className={`w-10 h-10 rounded-lg border-2 flex items-center justify-center text-xl transition-colors ${form.emoji === em ? 'border-indigo-500 bg-indigo-50' : 'border-slate-200 hover:border-slate-300'}`}
                    >
                      {em}
                    </button>
                  ))}
                </div>
              </div>

              {/* Gradient */}
              <div>
                <label className="block text-sm font-semibold text-slate-700 mb-1.5">Warna Kartu</label>
                <div className="flex gap-2 flex-wrap">
                  {GRADIENT_OPTIONS.map(g => (
                    <button
                      key={g.value}
                      onClick={() => setForm(f => ({ ...f, gradient: g.value }))}
                      className={`w-10 h-10 rounded-xl border-2 bg-gradient-to-br transition-all ${g.value} ${form.gradient === g.value ? 'border-indigo-500 scale-110' : 'border-transparent hover:border-slate-300'}`}
                      title={g.label}
                    />
                  ))}
                </div>
              </div>

              {/* Features */}
              <div>
                <label className="block text-sm font-semibold text-slate-700 mb-1.5">Fitur Paket</label>
                <div className="space-y-2">
                  {form.features.map((f, i) => (
                    <div key={i} className="flex gap-2 items-center">
                      <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                      <input
                        className="flex-1 border rounded-lg px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300"
                        placeholder={`Fitur ${i + 1}`}
                        value={f}
                        onChange={e => setFeature(i, e.target.value)}
                      />
                      <button onClick={() => removeFeature(i)} className="p-1 hover:text-red-500 text-slate-400 transition-colors">
                        <X className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  ))}
                  <button
                    onClick={addFeature}
                    className="flex items-center gap-1.5 text-sm text-indigo-600 hover:text-indigo-800 font-semibold"
                  >
                    <Plus className="w-3.5 h-3.5" /> Tambah fitur
                  </button>
                </div>
              </div>

              {/* Sort order */}
              <div>
                <label className="block text-sm font-semibold text-slate-700 mb-1.5">Urutan Tampil</label>
                <input
                  type="number"
                  className="w-full border rounded-xl px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300"
                  value={form.sortOrder}
                  onChange={e => setForm(f => ({ ...f, sortOrder: Number(e.target.value) }))}
                />
              </div>

              {/* Active toggle */}
              <div className="flex items-center gap-3 p-3 bg-slate-50 rounded-xl">
                <button
                  onClick={() => setForm(f => ({ ...f, active: !f.active }))}
                  className={`relative w-11 h-6 rounded-full transition-colors ${form.active ? 'bg-emerald-500' : 'bg-slate-300'}`}
                >
                  <div className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform ${form.active ? 'translate-x-5' : 'translate-x-0'}`} />
                </button>
                <span className="text-sm font-semibold text-slate-700">
                  {form.active ? 'Aktif — tampil di landing page' : 'Nonaktif — tersembunyi'}
                </span>
              </div>
            </div>

            {/* Footer */}
            <div className="sticky bottom-0 bg-white border-t px-6 py-4 flex gap-3">
              <Button variant="outline" className="flex-1" onClick={() => setShowModal(false)}>Batal</Button>
              <Button className="flex-1 bg-gradient-to-r from-indigo-600 to-purple-600" onClick={handleSave}>
                {editTarget ? 'Simpan Perubahan' : 'Tambah Paket'}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* ── Delete Confirm ── */}
      {deleteId && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" onClick={() => setDeleteId(null)} />
          <div className="relative bg-white rounded-2xl p-6 w-80 shadow-2xl">
            <div className="w-12 h-12 rounded-full bg-red-100 flex items-center justify-center mx-auto mb-4">
              <Trash2 className="w-6 h-6 text-red-600" />
            </div>
            <h3 className="font-black text-slate-900 text-center mb-2">Hapus Paket?</h3>
            <p className="text-sm text-muted-foreground text-center mb-6">
              Paket ini akan dihapus permanen dan tidak muncul lagi di landing page.
            </p>
            <div className="flex gap-3">
              <Button variant="outline" className="flex-1" onClick={() => setDeleteId(null)}>Batal</Button>
              <Button className="flex-1 bg-red-600 hover:bg-red-700 text-white" onClick={() => handleDelete(deleteId)}>
                Ya, Hapus
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
