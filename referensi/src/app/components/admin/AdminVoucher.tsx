import { useState, useMemo } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { toast } from 'sonner';
import {
  Tag, Plus, Search, Copy, Download, X, Save, ToggleLeft, ToggleRight,
  Gift, Users, Percent, Clock, CheckCircle2, XCircle, AlertCircle,
  BarChart3, Zap, Crown, School, RefreshCw, Eye, Trash2, ChevronDown,
  TrendingUp, Ticket, Link2, Share2, Filter
} from 'lucide-react';

// ─── Types ────────────────────────────────────────────────────────────────────
type VoucherType = 'access' | 'bulk-school' | 'promo' | 'referral';
type VoucherStatus = 'active' | 'redeemed' | 'expired' | 'inactive';
type Package = 'premium-1m' | 'premium-3m' | 'premium-6m' | 'elite-1m' | 'elite-3m' | 'free-trial';
type DiscountType = 'percent' | 'fixed' | 'full';

interface Voucher {
  id: string;
  code: string;
  type: VoucherType;
  pkg: Package;
  discountType: DiscountType;
  discountValue: number;      // % or Rp or 0 (full=free)
  maxUses: number;            // 1 for personal, N for bulk/promo
  usedCount: number;
  status: VoucherStatus;
  expiresAt: string;
  createdAt: string;
  createdBy: string;
  note?: string;
  // referral
  referrerName?: string;
  referrerCommission?: number; // Rp per redeem
  // bulk school
  schoolName?: string;
}

// ─── Seed data ────────────────────────────────────────────────────────────────
const PACKAGES: Record<Package, { label: string; price: number; duration: string }> = {
  'free-trial':  { label: 'Free Trial 7 Hari',   price: 0,       duration: '7 hari' },
  'premium-1m':  { label: 'Premium 1 Bulan',      price: 79000,   duration: '1 bulan' },
  'premium-3m':  { label: 'Premium 3 Bulan',      price: 210000,  duration: '3 bulan' },
  'premium-6m':  { label: 'Premium 6 Bulan',      price: 390000,  duration: '6 bulan' },
  'elite-1m':    { label: 'Elite 1 Bulan',        price: 199000,  duration: '1 bulan' },
  'elite-3m':    { label: 'Elite 3 Bulan',        price: 540000,  duration: '3 bulan' },
};

const TYPE_CONFIG: Record<VoucherType, { label: string; icon: React.ElementType; color: string; bg: string; desc: string }> = {
  'access':      { label: 'Akses Individual',   icon: Ticket,  color: 'text-indigo-700', bg: 'bg-indigo-100', desc: 'Kode unik untuk 1 siswa' },
  'bulk-school': { label: 'Bulk Sekolah',        icon: School,  color: 'text-blue-700',   bg: 'bg-blue-100',   desc: 'Batch kode untuk sekolah mitra' },
  'promo':       { label: 'Promo / Diskon',      icon: Percent, color: 'text-emerald-700',bg: 'bg-emerald-100',desc: 'Diskon % atau nominal untuk kampanye' },
  'referral':    { label: 'Referral',            icon: Share2,  color: 'text-purple-700', bg: 'bg-purple-100', desc: 'Link/kode afiliasi dengan komisi' },
};

const STATUS_CONFIG: Record<VoucherStatus, { label: string; dot: string; text: string }> = {
  active:   { label: 'Aktif',        dot: 'bg-emerald-500', text: 'text-emerald-700' },
  redeemed: { label: 'Terpakai',     dot: 'bg-slate-400',   text: 'text-slate-600' },
  expired:  { label: 'Kedaluwarsa',  dot: 'bg-red-400',     text: 'text-red-600' },
  inactive: { label: 'Nonaktif',     dot: 'bg-amber-400',   text: 'text-amber-700' },
};

const SEED_VOUCHERS: Voucher[] = [
  { id: 'v1', code: 'GASPL-PREM-X7K2', type: 'access', pkg: 'premium-3m', discountType: 'full', discountValue: 0, maxUses: 1, usedCount: 0, status: 'active', expiresAt: '2025-03-31', createdAt: '2025-01-20', createdBy: 'Admin Pusat', note: 'Hadiah lomba esai' },
  { id: 'v2', code: 'SMAN1-BATCH-2025', type: 'bulk-school', pkg: 'premium-6m', discountType: 'full', discountValue: 0, maxUses: 120, usedCount: 87, status: 'active', expiresAt: '2025-07-31', createdAt: '2025-01-15', createdBy: 'Admin Pusat', schoolName: 'SMA Negeri 1 Jakarta' },
  { id: 'v3', code: 'PROMO-JAN25-50', type: 'promo', pkg: 'premium-1m', discountType: 'percent', discountValue: 50, maxUses: 500, usedCount: 312, status: 'active', expiresAt: '2025-01-31', createdAt: '2025-01-01', createdBy: 'Admin Pusat', note: 'Campaign Januari 2025' },
  { id: 'v4', code: 'REF-BUDI-2025', type: 'referral', pkg: 'premium-3m', discountType: 'percent', discountValue: 20, maxUses: 999, usedCount: 23, status: 'active', expiresAt: '2025-12-31', createdAt: '2025-01-10', createdBy: 'Admin Pusat', referrerName: 'Budi Santoso', referrerCommission: 25000 },
  { id: 'v5', code: 'GASPL-ELITE-9MNP', type: 'access', pkg: 'elite-1m', discountType: 'full', discountValue: 0, maxUses: 1, usedCount: 1, status: 'redeemed', expiresAt: '2025-02-28', createdAt: '2025-01-18', createdBy: 'Admin Pusat' },
  { id: 'v6', code: 'PROMO-DEC24-30', type: 'promo', pkg: 'premium-1m', discountType: 'percent', discountValue: 30, maxUses: 300, usedCount: 300, status: 'redeemed', expiresAt: '2024-12-31', createdAt: '2024-12-01', createdBy: 'Admin Pusat', note: 'Campaign Desember 2024' },
  { id: 'v7', code: 'REF-DEWI-2025', type: 'referral', pkg: 'premium-1m', discountType: 'percent', discountValue: 15, maxUses: 999, usedCount: 8, status: 'active', expiresAt: '2025-12-31', createdAt: '2025-01-12', createdBy: 'Admin Pusat', referrerName: 'Dewi Rahayu', referrerCommission: 15000 },
  { id: 'v8', code: 'MANBDK-BULK-0125', type: 'bulk-school', pkg: 'premium-3m', discountType: 'full', discountValue: 0, maxUses: 60, usedCount: 12, status: 'active', expiresAt: '2025-04-30', createdAt: '2025-01-22', createdBy: 'Admin Pusat', schoolName: 'MAN 2 Bandung' },
];

// ─── Helpers ──────────────────────────────────────────────────────────────────
function genCode(type: VoucherType): string {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
  const seg = () => Array.from({ length: 4 }, () => chars[Math.floor(Math.random() * chars.length)]).join('');
  const prefix: Record<VoucherType, string> = { access: 'GASPL', 'bulk-school': 'BULK', promo: 'PROMO', referral: 'REF' };
  return `${prefix[type]}-${seg()}-${seg()}`;
}

function fmtRp(n: number) { return 'Rp ' + n.toLocaleString('id'); }
function fmtDiscount(v: Voucher) {
  if (v.discountType === 'full') return 'Gratis';
  if (v.discountType === 'percent') return `Diskon ${v.discountValue}%`;
  return `Potongan ${fmtRp(v.discountValue)}`;
}

// ─── Stats cards ──────────────────────────────────────────────────────────────
function StatsRow({ vouchers }: { vouchers: Voucher[] }) {
  const active   = vouchers.filter(v => v.status === 'active').length;
  const redeemed = vouchers.reduce((s, v) => s + v.usedCount, 0);
  const totalMax = vouchers.reduce((s, v) => s + v.maxUses, 0);
  const rate     = totalMax > 0 ? Math.round((redeemed / totalMax) * 100) : 0;
  const estRev   = vouchers.filter(v => v.type === 'promo' || v.type === 'access')
    .reduce((s, v) => {
      const base = PACKAGES[v.pkg].price;
      const disc = v.discountType === 'full' ? base : v.discountType === 'percent' ? base * v.discountValue / 100 : v.discountValue;
      return s + (base - disc) * v.usedCount;
    }, 0);

  return (
    <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
      {[
        { label: 'Voucher Aktif',     val: active,               icon: Ticket,    color: 'text-indigo-600', bg: 'bg-indigo-50' },
        { label: 'Total Digunakan',   val: redeemed,             icon: CheckCircle2, color: 'text-emerald-600', bg: 'bg-emerald-50' },
        { label: 'Redemption Rate',   val: `${rate}%`,           icon: TrendingUp, color: 'text-blue-600', bg: 'bg-blue-50' },
        { label: 'Est. Revenue',      val: fmtRp(estRev),        icon: BarChart3,  color: 'text-amber-600', bg: 'bg-amber-50' },
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
  );
}

// ─── Create Voucher Modal ─────────────────────────────────────────────────────
interface CreateForm {
  type: VoucherType;
  pkg: Package;
  discountType: DiscountType;
  discountValue: number;
  maxUses: number;
  expiresAt: string;
  note: string;
  quantity: number;       // for bulk-school: how many codes to generate
  schoolName: string;
  referrerName: string;
  referrerCommission: number;
}

function CreateModal({ onClose, onCreate }: {
  onClose: () => void;
  onCreate: (v: Voucher | Voucher[]) => void;
}) {
  const defaultExpiry = new Date(Date.now() + 90 * 86400000).toISOString().split('T')[0];
  const [form, setForm] = useState<CreateForm>({
    type: 'access', pkg: 'premium-3m', discountType: 'full', discountValue: 0,
    maxUses: 1, expiresAt: defaultExpiry, note: '', quantity: 1,
    schoolName: '', referrerName: '', referrerCommission: 25000,
  });
  const [step, setStep] = useState<1 | 2>(1);

  const set = (k: keyof CreateForm, v: any) => setForm(f => ({ ...f, [k]: v }));

  const pkg = PACKAGES[form.pkg];
  const discountedPrice = form.discountType === 'full' ? 0
    : form.discountType === 'percent' ? Math.round(pkg.price * (1 - form.discountValue / 100))
    : Math.max(0, pkg.price - form.discountValue);

  const handleCreate = () => {
    const base: Omit<Voucher, 'id' | 'code'> = {
      type: form.type, pkg: form.pkg, discountType: form.discountType,
      discountValue: form.discountValue, maxUses: form.type === 'promo' ? form.maxUses : form.type === 'bulk-school' ? form.maxUses : 1,
      usedCount: 0, status: 'active', expiresAt: form.expiresAt,
      createdAt: new Date().toISOString().split('T')[0], createdBy: 'Admin Pusat',
      note: form.note || undefined, schoolName: form.schoolName || undefined,
      referrerName: form.referrerName || undefined,
      referrerCommission: form.type === 'referral' ? form.referrerCommission : undefined,
    };

    const qty = form.type === 'bulk-school' ? form.quantity : form.type === 'access' ? form.quantity : 1;
    if (qty > 1) {
      const batch: Voucher[] = Array.from({ length: qty }, (_, i) => ({
        ...base, id: `v${Date.now()}${i}`, code: genCode(form.type),
        maxUses: form.type === 'bulk-school' ? form.maxUses : 1,
      }));
      onCreate(batch);
      toast.success(`${qty} voucher berhasil digenerate`);
    } else {
      const single: Voucher = { ...base, id: `v${Date.now()}`, code: genCode(form.type) };
      onCreate(single);
      toast.success(`Voucher ${single.code} berhasil dibuat`);
    }
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4">
      <Card className="w-full max-w-xl shadow-2xl flex flex-col max-h-[90vh]">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b shrink-0">
          <div>
            <h3 className="font-bold text-slate-900">Buat Voucher Baru</h3>
            <p className="text-xs text-muted-foreground mt-0.5">Langkah {step} dari 2</p>
          </div>
          <button onClick={onClose} className="p-1.5 rounded-lg hover:bg-slate-100"><X className="w-4 h-4" /></button>
        </div>

        <div className="flex-1 overflow-y-auto px-6 py-5 space-y-5">
          {step === 1 && (
            <>
              {/* Type selector */}
              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-2">Tipe Voucher</label>
                <div className="grid grid-cols-2 gap-2">
                  {(Object.keys(TYPE_CONFIG) as VoucherType[]).map(t => {
                    const tc = TYPE_CONFIG[t];
                    const Icon = tc.icon;
                    return (
                      <button key={t} onClick={() => set('type', t)}
                        className={`flex items-start gap-3 p-3 rounded-xl border-2 text-left transition-all ${form.type === t ? 'border-indigo-500 bg-indigo-50' : 'border-slate-200 hover:border-slate-300'}`}>
                        <div className={`w-8 h-8 rounded-lg ${form.type === t ? 'bg-indigo-600' : 'bg-slate-100'} flex items-center justify-center shrink-0`}>
                          <Icon className={`w-4 h-4 ${form.type === t ? 'text-white' : 'text-slate-500'}`} />
                        </div>
                        <div>
                          <p className={`text-sm font-bold ${form.type === t ? 'text-indigo-700' : 'text-slate-700'}`}>{tc.label}</p>
                          <p className="text-xs text-muted-foreground">{tc.desc}</p>
                        </div>
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Package */}
              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Paket yang Diaktifkan</label>
                <div className="grid grid-cols-2 gap-2">
                  {(Object.keys(PACKAGES) as Package[]).map(p => (
                    <button key={p} onClick={() => set('pkg', p)}
                      className={`px-3 py-2.5 rounded-xl border-2 text-sm font-semibold text-left transition-colors ${form.pkg === p ? 'border-indigo-500 bg-indigo-50 text-indigo-700' : 'border-slate-200 text-slate-500 hover:border-slate-300'}`}>
                      <div>{PACKAGES[p].label}</div>
                      <div className="text-xs font-normal opacity-70">{PACKAGES[p].price === 0 ? 'Gratis' : fmtRp(PACKAGES[p].price)}</div>
                    </button>
                  ))}
                </div>
              </div>

              {/* Quantity — access and bulk */}
              {(form.type === 'access' || form.type === 'bulk-school') && (
                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">
                      {form.type === 'bulk-school' ? 'Jumlah Kode' : 'Jumlah Voucher'}
                    </label>
                    <input type="number" min={1} max={500} value={form.quantity}
                      onChange={e => set('quantity', +e.target.value)}
                      className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                  </div>
                  {form.type === 'bulk-school' && (
                    <div>
                      <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Sekolah</label>
                      <input value={form.schoolName} onChange={e => set('schoolName', e.target.value)} placeholder="SMA Negeri 1 ..." className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                    </div>
                  )}
                </div>
              )}

              {/* Promo max uses */}
              {form.type === 'promo' && (
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Batas Penggunaan</label>
                  <input type="number" min={1} value={form.maxUses} onChange={e => set('maxUses', +e.target.value)} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                </div>
              )}

              {/* Referral fields */}
              {form.type === 'referral' && (
                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Nama Referrer</label>
                    <input value={form.referrerName} onChange={e => set('referrerName', e.target.value)} placeholder="Nama influencer/afiliasi" className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                  </div>
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Komisi per Redeem</label>
                    <input type="number" min={0} value={form.referrerCommission} onChange={e => set('referrerCommission', +e.target.value)} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                  </div>
                </div>
              )}
            </>
          )}

          {step === 2 && (
            <>
              {/* Discount */}
              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-2">Jenis Diskon</label>
                <div className="grid grid-cols-3 gap-2 mb-3">
                  {([['full', 'Gratis (100%)'], ['percent', 'Diskon %'], ['fixed', 'Potongan Rp']] as [DiscountType, string][]).map(([dt, label]) => (
                    <button key={dt} onClick={() => { set('discountType', dt); set('discountValue', 0); }}
                      className={`py-2.5 rounded-xl border-2 text-sm font-semibold transition-colors ${form.discountType === dt ? 'border-indigo-500 bg-indigo-50 text-indigo-700' : 'border-slate-200 text-slate-500 hover:border-slate-300'}`}>
                      {label}
                    </button>
                  ))}
                </div>
                {form.discountType !== 'full' && (
                  <input type="number" min={0} max={form.discountType === 'percent' ? 100 : undefined}
                    value={form.discountValue} onChange={e => set('discountValue', +e.target.value)}
                    placeholder={form.discountType === 'percent' ? 'Persentase (0-100)' : 'Nominal potongan (Rp)'}
                    className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
                )}
              </div>

              {/* Expiry */}
              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Tanggal Kedaluwarsa</label>
                <input type="date" value={form.expiresAt} onChange={e => set('expiresAt', e.target.value)} className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
              </div>

              {/* Note */}
              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-1.5">Catatan Internal (opsional)</label>
                <input value={form.note} onChange={e => set('note', e.target.value)} placeholder="contoh: Hadiah lomba, Campaign IG, ..." className="w-full border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400" />
              </div>

              {/* Preview */}
              <div className="p-4 bg-gradient-to-br from-indigo-50 to-purple-50 border-2 border-indigo-200 rounded-xl">
                <p className="text-xs font-bold text-indigo-500 uppercase tracking-wider mb-3">Preview Voucher</p>
                <div className="flex items-center gap-3 mb-3">
                  <div className={`w-10 h-10 rounded-xl ${TYPE_CONFIG[form.type].bg} flex items-center justify-center`}>
                    {(() => { const Icon = TYPE_CONFIG[form.type].icon; return <Icon className={`w-5 h-5 ${TYPE_CONFIG[form.type].color}`} />; })()}
                  </div>
                  <div>
                    <p className="font-bold text-slate-900">{TYPE_CONFIG[form.type].label}</p>
                    <p className="text-sm text-muted-foreground">{PACKAGES[form.pkg].label}</p>
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div className="bg-white rounded-lg p-2"><span className="text-muted-foreground">Harga Normal:</span> <strong>{fmtRp(pkg.price)}</strong></div>
                  <div className="bg-white rounded-lg p-2"><span className="text-muted-foreground">Harga Voucher:</span> <strong className="text-emerald-600">{discountedPrice === 0 ? 'GRATIS' : fmtRp(discountedPrice)}</strong></div>
                  <div className="bg-white rounded-lg p-2"><span className="text-muted-foreground">Berlaku hingga:</span> <strong>{form.expiresAt}</strong></div>
                  <div className="bg-white rounded-lg p-2"><span className="text-muted-foreground">Durasi Akses:</span> <strong>{PACKAGES[form.pkg].duration}</strong></div>
                </div>
              </div>
            </>
          )}
        </div>

        <div className="px-6 py-4 border-t flex gap-2 shrink-0">
          {step === 1 ? (
            <>
              <Button variant="outline" className="flex-1" onClick={onClose}>Batal</Button>
              <Button className="flex-1 bg-indigo-600 hover:bg-indigo-700" onClick={() => setStep(2)}>Lanjut →</Button>
            </>
          ) : (
            <>
              <Button variant="outline" className="flex-1" onClick={() => setStep(1)}>← Kembali</Button>
              <Button className="flex-1 bg-indigo-600 hover:bg-indigo-700" onClick={handleCreate}><Save className="w-4 h-4 mr-1.5" />Generate Voucher</Button>
            </>
          )}
        </div>
      </Card>
    </div>
  );
}

// ─── Referral Leaderboard ─────────────────────────────────────────────────────
function ReferralTab({ vouchers }: { vouchers: Voucher[] }) {
  const refs = vouchers.filter(v => v.type === 'referral');
  const totalCommission = refs.reduce((s, v) => s + (v.referrerCommission || 0) * v.usedCount, 0);

  return (
    <div className="space-y-5">
      <div className="grid grid-cols-3 gap-4">
        {[
          { label: 'Total Referrer', val: refs.length, color: 'text-purple-600' },
          { label: 'Total Redeem', val: refs.reduce((s, v) => s + v.usedCount, 0), color: 'text-indigo-600' },
          { label: 'Total Komisi', val: fmtRp(totalCommission), color: 'text-emerald-600' },
        ].map(({ label, val, color }) => (
          <Card key={label} className="p-4 text-center">
            <div className={`text-2xl font-black ${color}`}>{val}</div>
            <div className="text-xs text-muted-foreground mt-0.5">{label}</div>
          </Card>
        ))}
      </div>

      <Card className="overflow-hidden">
        <div className="px-5 py-3 border-b bg-slate-50 flex items-center justify-between">
          <h3 className="font-bold text-slate-900 text-sm">Performa Referrer</h3>
          <span className="text-xs text-muted-foreground">Diurutkan: total redeem</span>
        </div>
        <div className="divide-y">
          {refs.sort((a, b) => b.usedCount - a.usedCount).map((v, idx) => {
            const commission = (v.referrerCommission || 0) * v.usedCount;
            return (
              <div key={v.id} className="flex items-center gap-4 px-5 py-4">
                <div className={`w-8 h-8 rounded-xl flex items-center justify-center font-black text-sm ${idx === 0 ? 'bg-amber-400 text-white' : idx === 1 ? 'bg-slate-300 text-slate-700' : 'bg-slate-100 text-slate-500'}`}>
                  {idx + 1}
                </div>
                <div className="flex-1">
                  <p className="font-semibold text-sm">{v.referrerName}</p>
                  <p className="text-xs text-muted-foreground font-mono">{v.code}</p>
                </div>
                <div className="text-center">
                  <p className="font-black text-indigo-600">{v.usedCount}</p>
                  <p className="text-xs text-muted-foreground">redeem</p>
                </div>
                <div className="text-center">
                  <p className="font-black text-emerald-600">{fmtRp(commission)}</p>
                  <p className="text-xs text-muted-foreground">komisi</p>
                </div>
                <div className="text-center">
                  <span className={`text-xs font-semibold px-2 py-1 rounded-full flex items-center gap-1 ${STATUS_CONFIG[v.status].text} bg-slate-100`}>
                    <span className={`w-1.5 h-1.5 rounded-full ${STATUS_CONFIG[v.status].dot}`} />
                    {STATUS_CONFIG[v.status].label}
                  </span>
                </div>
                <Button variant="ghost" size="sm" onClick={() => {
                  navigator.clipboard?.writeText(v.code);
                  toast.success('Kode referral disalin');
                }}>
                  <Copy className="w-4 h-4" />
                </Button>
              </div>
            );
          })}
        </div>
      </Card>
    </div>
  );
}

// ─── Main ─────────────────────────────────────────────────────────────────────
type AdminTab = 'semua' | 'access' | 'bulk-school' | 'promo' | 'referral';

export default function AdminVoucher() {
  const [vouchers, setVouchers] = useState<Voucher[]>(SEED_VOUCHERS);
  const [tab, setTab] = useState<AdminTab>('semua');
  const [showCreate, setShowCreate] = useState(false);
  const [search, setSearch] = useState('');
  const [filterStatus, setFilterStatus] = useState<VoucherStatus | 'all'>('all');
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const filtered = useMemo(() => {
    let list = tab === 'semua' ? vouchers : vouchers.filter(v => v.type === tab);
    if (filterStatus !== 'all') list = list.filter(v => v.status === filterStatus);
    if (search) list = list.filter(v =>
      v.code.toLowerCase().includes(search.toLowerCase()) ||
      v.note?.toLowerCase().includes(search.toLowerCase()) ||
      v.schoolName?.toLowerCase().includes(search.toLowerCase()) ||
      v.referrerName?.toLowerCase().includes(search.toLowerCase())
    );
    return list;
  }, [vouchers, tab, filterStatus, search]);

  const handleCreate = (v: Voucher | Voucher[]) => {
    setVouchers(prev => Array.isArray(v) ? [...v, ...prev] : [v, ...prev]);
  };

  const toggleStatus = (id: string) => {
    setVouchers(prev => prev.map(v => {
      if (v.id !== id) return v;
      const next: VoucherStatus = v.status === 'active' ? 'inactive' : v.status === 'inactive' ? 'active' : v.status;
      toast.success(`Voucher ${v.code} → ${STATUS_CONFIG[next].label}`);
      return { ...v, status: next };
    }));
  };

  const deleteVoucher = (id: string) => {
    const v = vouchers.find(v => v.id === id);
    if (v && v.usedCount > 0) { toast.error('Voucher yang sudah dipakai tidak bisa dihapus'); return; }
    setVouchers(prev => prev.filter(v => v.id !== id));
    toast.success('Voucher dihapus');
  };

  const copyCode = (code: string) => {
    navigator.clipboard?.writeText(code);
    toast.success(`Kode ${code} disalin`);
  };

  const exportBulk = (schoolName: string) => {
    toast.info(`Export CSV voucher "${schoolName}" sedang disiapkan...`);
  };

  const TABS: { id: AdminTab; label: string }[] = [
    { id: 'semua', label: 'Semua' },
    { id: 'access', label: 'Akses Individual' },
    { id: 'bulk-school', label: 'Bulk Sekolah' },
    { id: 'promo', label: 'Promo' },
    { id: 'referral', label: 'Referral' },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-gradient-to-r from-violet-600 via-purple-600 to-indigo-600 rounded-2xl p-6 text-white relative overflow-hidden">
        <div className="absolute inset-0 opacity-10" style={{ backgroundImage: 'repeating-linear-gradient(135deg, white 0, white 1px, transparent 0, transparent 50%)', backgroundSize: '20px 20px' }} />
        <div className="relative flex items-start justify-between gap-4">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <Tag className="w-5 h-5 text-purple-200" />
              <span className="text-sm font-semibold text-purple-200">Sistem Voucher</span>
            </div>
            <h1 className="text-2xl font-black mb-1">Manajemen Voucher</h1>
            <p className="text-purple-200 text-sm">Kelola voucher akses, bulk sekolah, promo, dan program referral afiliasi</p>
          </div>
          <Button onClick={() => setShowCreate(true)} className="bg-white text-indigo-700 hover:bg-purple-50 gap-2 shrink-0">
            <Plus className="w-4 h-4" />Buat Voucher
          </Button>
        </div>
      </div>

      <StatsRow vouchers={vouchers} />

      {/* Tabs */}
      <div className="flex items-center gap-1 p-1 bg-slate-100 rounded-xl w-fit flex-wrap">
        {TABS.map(t => (
          <button key={t.id} onClick={() => setTab(t.id)}
            className={`px-4 py-2 rounded-lg text-sm font-semibold transition-all ${tab === t.id ? 'bg-white shadow-sm text-slate-900' : 'text-slate-500 hover:text-slate-700'}`}>
            {t.label}
          </button>
        ))}
      </div>

      {/* Referral tab gets its own view */}
      {tab === 'referral' ? (
        <ReferralTab vouchers={vouchers} />
      ) : (
        <>
          {/* Filters */}
          <div className="flex gap-2 flex-wrap">
            <div className="relative flex-1 min-w-48">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
              <input value={search} onChange={e => setSearch(e.target.value)} placeholder="Cari kode, sekolah, catatan..." className="w-full pl-9 pr-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 bg-white" />
            </div>
            <select value={filterStatus} onChange={e => setFilterStatus(e.target.value as VoucherStatus | 'all')} className="px-3 py-2 border rounded-lg text-sm bg-white focus:outline-none">
              <option value="all">Semua Status</option>
              {(Object.keys(STATUS_CONFIG) as VoucherStatus[]).map(s => <option key={s} value={s}>{STATUS_CONFIG[s].label}</option>)}
            </select>
          </div>

          {/* Voucher list */}
          <div className="space-y-2.5">
            {filtered.length === 0 && (
              <Card className="p-12 text-center">
                <Tag className="w-10 h-10 text-slate-300 mx-auto mb-3" />
                <p className="font-semibold text-slate-500">Tidak ada voucher ditemukan</p>
              </Card>
            )}
            {filtered.map(v => {
              const tc = TYPE_CONFIG[v.type];
              const sc = STATUS_CONFIG[v.status];
              const Icon = tc.icon;
              const usagePercent = v.maxUses > 1 ? Math.round((v.usedCount / v.maxUses) * 100) : 0;
              const isExpanded = expandedId === v.id;

              return (
                <Card key={v.id} className="overflow-hidden hover:shadow-sm transition-shadow">
                  <div className="flex items-center gap-4 p-4 cursor-pointer" onClick={() => setExpandedId(isExpanded ? null : v.id)}>
                    {/* Icon */}
                    <div className={`w-10 h-10 rounded-xl ${tc.bg} flex items-center justify-center shrink-0`}>
                      <Icon className={`w-5 h-5 ${tc.color}`} />
                    </div>

                    {/* Code + meta */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1 flex-wrap">
                        <span className="font-mono font-black text-slate-900 tracking-wider text-sm">{v.code}</span>
                        <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${tc.bg} ${tc.color}`}>{tc.label}</span>
                        <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full flex items-center gap-1 bg-slate-100 ${sc.text}`}>
                          <span className={`w-1.5 h-1.5 rounded-full ${sc.dot}`} />{sc.label}
                        </span>
                      </div>
                      <div className="flex items-center gap-3 text-xs text-muted-foreground flex-wrap">
                        <span>{PACKAGES[v.pkg].label}</span>
                        <span>·</span>
                        <span>{fmtDiscount(v)}</span>
                        {v.schoolName && <><span>·</span><span>{v.schoolName}</span></>}
                        {v.referrerName && <><span>·</span><span>{v.referrerName}</span></>}
                        <span>·</span>
                        <span className="flex items-center gap-1"><Clock className="w-3 h-3" />Exp: {v.expiresAt}</span>
                      </div>
                    </div>

                    {/* Usage */}
                    <div className="shrink-0 text-right hidden sm:block">
                      <p className="font-black text-sm text-slate-900">{v.usedCount}/{v.maxUses}</p>
                      <p className="text-xs text-muted-foreground">digunakan</p>
                      {v.maxUses > 1 && (
                        <div className="w-20 h-1.5 bg-slate-100 rounded-full mt-1 ml-auto overflow-hidden">
                          <div className="h-full bg-indigo-500 rounded-full" style={{ width: `${usagePercent}%` }} />
                        </div>
                      )}
                    </div>

                    <ChevronDown className={`w-4 h-4 text-slate-400 shrink-0 transition-transform ${isExpanded ? 'rotate-180' : ''}`} />
                  </div>

                  {/* Expanded row */}
                  {isExpanded && (
                    <div className="border-t bg-slate-50 px-4 py-3 flex items-center justify-between gap-3 flex-wrap">
                      <div className="flex items-center gap-2 flex-wrap text-xs text-muted-foreground">
                        {v.note && <span className="italic">"{v.note}"</span>}
                        <span>Dibuat: {v.createdAt}</span>
                        {v.referrerCommission && <span className="text-purple-600 font-semibold">Komisi: {fmtRp(v.referrerCommission)}/redeem</span>}
                      </div>
                      <div className="flex items-center gap-2">
                        <Button variant="outline" size="sm" onClick={() => copyCode(v.code)} className="gap-1.5">
                          <Copy className="w-3.5 h-3.5" />Salin Kode
                        </Button>
                        {v.type === 'bulk-school' && (
                          <Button variant="outline" size="sm" onClick={() => exportBulk(v.schoolName || '')} className="gap-1.5">
                            <Download className="w-3.5 h-3.5" />Export CSV
                          </Button>
                        )}
                        {(v.status === 'active' || v.status === 'inactive') && (
                          <button onClick={() => toggleStatus(v.id)} className="transition-colors">
                            {v.status === 'active'
                              ? <ToggleRight className="w-7 h-7 text-emerald-500" />
                              : <ToggleLeft className="w-7 h-7 text-slate-300" />}
                          </button>
                        )}
                        {v.usedCount === 0 && (
                          <Button variant="ghost" size="sm" onClick={() => deleteVoucher(v.id)} className="text-red-400 hover:text-red-600 hover:bg-red-50">
                            <Trash2 className="w-4 h-4" />
                          </Button>
                        )}
                      </div>
                    </div>
                  )}
                </Card>
              );
            })}
          </div>
        </>
      )}

      {showCreate && <CreateModal onClose={() => setShowCreate(false)} onCreate={handleCreate} />}
    </div>
  );
}
