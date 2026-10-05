import { useState, useCallback } from 'react';
import { Card } from '../ui/card';
import { toast } from 'sonner';
import {
  Tag, X, CheckCircle2, XCircle, Gift, Ticket, Share2,
  Clock, Crown, Zap, ChevronRight, Loader2, Copy, Star
} from 'lucide-react';

interface VoucherActivationProps {
  onClose: () => void;
}

type CheckState = 'idle' | 'checking' | 'valid' | 'invalid' | 'activated';

interface VoucherResult {
  code: string;
  pkg: string;
  duration: string;
  discount: string;
  type: 'access' | 'promo' | 'referral';
  expires: string;
  activatesUntil: string;
}

// Simulated voucher database for demo
const MOCK_VOUCHERS: Record<string, VoucherResult> = {
  'GASPL-PREM-X7K2': { code: 'GASPL-PREM-X7K2', pkg: 'Premium 3 Bulan', duration: '3 bulan', discount: 'GRATIS', type: 'access', expires: '31 Mar 2025', activatesUntil: '30 Apr 2025' },
  'PROMO-JAN25-50':  { code: 'PROMO-JAN25-50', pkg: 'Premium 1 Bulan', duration: '1 bulan', discount: 'Diskon 50% → Rp 39.500', type: 'promo', expires: '31 Jan 2025', activatesUntil: '28 Feb 2025' },
  'SMAN1-BATCH-2025': { code: 'SMAN1-BATCH-2025', pkg: 'Premium 6 Bulan', duration: '6 bulan', discount: 'GRATIS (Paket Sekolah)', type: 'access', expires: '31 Jul 2025', activatesUntil: '31 Jul 2025' },
  'REF-BUDI-2025':   { code: 'REF-BUDI-2025', pkg: 'Premium 3 Bulan', duration: '3 bulan', discount: 'Diskon 20% → Rp 168.000', type: 'referral', expires: '31 Des 2025', activatesUntil: '30 Apr 2025' },
};

const PKG_ICONS: Record<string, React.ElementType> = {
  'access': Ticket, 'promo': Gift, 'referral': Share2,
};

const PKG_COLORS: Record<string, string> = {
  'access': 'from-indigo-600 to-purple-600',
  'promo':  'from-emerald-500 to-green-600',
  'referral': 'from-violet-500 to-fuchsia-600',
};

const TYPE_LABEL: Record<string, string> = {
  'access': 'Voucher Akses', 'promo': 'Kode Promo', 'referral': 'Kode Referral',
};

export default function VoucherActivation({ onClose }: VoucherActivationProps) {
  const [code, setCode] = useState('');
  const [state, setState] = useState<CheckState>('idle');
  const [result, setResult] = useState<VoucherResult | null>(null);

  const handleInput = (val: string) => {
    setCode(val.toUpperCase().replace(/[^A-Z0-9-]/g, ''));
    if (state !== 'idle') { setState('idle'); setResult(null); }
  };

  const checkVoucher = useCallback(() => {
    const trimmed = code.trim();
    if (!trimmed) return;
    setState('checking');
    setTimeout(() => {
      const found = MOCK_VOUCHERS[trimmed];
      if (found) {
        setState('valid');
        setResult(found);
      } else {
        setState('invalid');
        setResult(null);
      }
    }, 900);
  }, [code]);

  const activateVoucher = () => {
    if (!result) return;
    setState('activated');
    toast.success(`Voucher ${result.code} berhasil diaktifkan!`, {
      description: `${result.pkg} aktif hingga ${result.activatesUntil}`,
      duration: 6000,
    });
  };

  const reset = () => {
    setCode('');
    setState('idle');
    setResult(null);
  };

  const PKG_ICON = result ? PKG_ICONS[result.type] : Gift;
  const PKG_GRAD = result ? PKG_COLORS[result.type] : 'from-indigo-600 to-purple-600';

  return (
    <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4">
      <Card className="w-full max-w-md shadow-2xl overflow-hidden">
        {/* Header */}
        <div className="relative bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 px-6 py-5 text-white">
          <div className="absolute inset-0 opacity-10" style={{ backgroundImage: 'radial-gradient(circle, white 1px, transparent 1px)', backgroundSize: '24px 24px' }} />
          <div className="relative flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-2xl bg-white/20 flex items-center justify-center">
                <Tag className="w-5 h-5 text-white" />
              </div>
              <div>
                <h2 className="font-black text-lg">Aktivasi Voucher</h2>
                <p className="text-white/70 text-xs">Masukkan kode untuk mengaktifkan akses</p>
              </div>
            </div>
            <button onClick={onClose} className="p-2 rounded-xl hover:bg-white/20 transition-colors">
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        <div className="p-6">
          {state !== 'activated' ? (
            <>
              {/* Code input */}
              <div className="mb-4">
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wider block mb-2">Kode Voucher</label>
                <div className="flex gap-2">
                  <input
                    value={code}
                    onChange={e => handleInput(e.target.value)}
                    onKeyDown={e => e.key === 'Enter' && state !== 'checking' && checkVoucher()}
                    placeholder="Contoh: GASPL-PREM-X7K2"
                    maxLength={24}
                    className="flex-1 px-4 py-3 border-2 rounded-xl text-sm font-mono tracking-wider focus:outline-none focus:border-indigo-400 transition-colors placeholder-slate-300"
                  />
                  <button
                    onClick={checkVoucher}
                    disabled={!code.trim() || state === 'checking'}
                    className="px-4 py-3 rounded-xl bg-indigo-600 hover:bg-indigo-700 text-white font-bold text-sm transition-all disabled:opacity-40 disabled:cursor-not-allowed shrink-0"
                  >
                    {state === 'checking' ? <Loader2 className="w-4 h-4 animate-spin" /> : 'Cek'}
                  </button>
                </div>
              </div>

              {/* Invalid */}
              {state === 'invalid' && (
                <div className="flex items-center gap-2.5 p-3.5 bg-red-50 border border-red-200 rounded-xl text-sm text-red-700 mb-4">
                  <XCircle className="w-5 h-5 shrink-0" />
                  <div>
                    <p className="font-semibold">Kode tidak ditemukan</p>
                    <p className="text-xs text-red-500 mt-0.5">Pastikan kode sudah benar dan belum kedaluwarsa</p>
                  </div>
                </div>
              )}

              {/* Valid */}
              {state === 'valid' && result && (
                <div className="mb-4 space-y-3">
                  <div className="p-4 bg-emerald-50 border-2 border-emerald-300 rounded-xl">
                    <div className="flex items-center gap-2 mb-3">
                      <CheckCircle2 className="w-5 h-5 text-emerald-600" />
                      <span className="font-bold text-emerald-800">Kode valid!</span>
                      <span className={`ml-auto text-[10px] font-bold px-2 py-0.5 rounded-full bg-gradient-to-r ${PKG_GRAD} text-white`}>{TYPE_LABEL[result.type]}</span>
                    </div>
                    <div className={`w-10 h-10 rounded-xl bg-gradient-to-br ${PKG_GRAD} flex items-center justify-center mb-3`}>
                      <PKG_ICON className="w-5 h-5 text-white" />
                    </div>
                    <h3 className="font-black text-slate-900 text-base mb-2">{result.pkg}</h3>
                    <div className="grid grid-cols-2 gap-2 text-xs">
                      <div className="bg-white rounded-lg p-2.5">
                        <span className="text-slate-500 block mb-0.5">Durasi Akses</span>
                        <strong className="text-slate-900">{result.duration}</strong>
                      </div>
                      <div className="bg-white rounded-lg p-2.5">
                        <span className="text-slate-500 block mb-0.5">Harga</span>
                        <strong className="text-emerald-600">{result.discount}</strong>
                      </div>
                      <div className="bg-white rounded-lg p-2.5">
                        <span className="text-slate-500 block mb-0.5">Akses Hingga</span>
                        <strong className="text-slate-900">{result.activatesUntil}</strong>
                      </div>
                      <div className="bg-white rounded-lg p-2.5">
                        <span className="text-slate-500 block mb-0.5">Exp. Voucher</span>
                        <strong className="text-slate-900">{result.expires}</strong>
                      </div>
                    </div>
                  </div>

                  <button
                    onClick={activateVoucher}
                    className={`w-full py-3.5 rounded-xl bg-gradient-to-r ${PKG_GRAD} text-white font-black text-sm transition-all hover:-translate-y-0.5 shadow-lg flex items-center justify-center gap-2`}
                  >
                    <Zap className="w-4 h-4" />
                    Aktifkan Sekarang
                    <ChevronRight className="w-4 h-4" />
                  </button>
                </div>
              )}

              {/* Type hints */}
              {state === 'idle' && (
                <div className="grid grid-cols-3 gap-2 mt-4 pt-4 border-t">
                  {[
                    { icon: Ticket, label: 'Akses Individual', color: 'bg-indigo-100 text-indigo-600' },
                    { icon: Gift,   label: 'Kode Promo',       color: 'bg-emerald-100 text-emerald-600' },
                    { icon: Share2, label: 'Kode Referral',    color: 'bg-purple-100 text-purple-600' },
                  ].map(({ icon: Icon, label, color }) => (
                    <div key={label} className="text-center p-2.5 bg-slate-50 rounded-xl">
                      <div className={`w-7 h-7 rounded-lg ${color} flex items-center justify-center mx-auto mb-1.5`}>
                        <Icon className="w-3.5 h-3.5" />
                      </div>
                      <p className="text-[10px] text-slate-500 font-semibold leading-tight">{label}</p>
                    </div>
                  ))}
                </div>
              )}
            </>
          ) : (
            /* Activated success screen */
            <div className="text-center py-4">
              <div className={`w-20 h-20 rounded-3xl bg-gradient-to-br ${PKG_GRAD} flex items-center justify-center mx-auto mb-5 shadow-xl`}>
                <Star className="w-10 h-10 text-white fill-white" />
              </div>
              <h3 className="text-2xl font-black text-slate-900 mb-1">Berhasil! 🎉</h3>
              <p className="text-muted-foreground text-sm mb-4">
                {result?.pkg} sudah aktif di akunmu
              </p>
              <div className="p-4 bg-gradient-to-br from-indigo-50 to-purple-50 border border-indigo-200 rounded-2xl mb-5 text-left">
                <div className="flex items-center gap-2 mb-3">
                  <Crown className="w-5 h-5 text-indigo-600" />
                  <span className="font-bold text-indigo-800">Akses Premium Aktif</span>
                </div>
                <div className="space-y-2 text-sm">
                  {['Tryout simulasi tanpa batas', 'Drilling adaptif per subtes', 'Rasionalisasi PTN real-time', 'Analytics performa mendalam', 'Akses bank soal 10.000+ soal'].map(f => (
                    <div key={f} className="flex items-center gap-2 text-slate-700">
                      <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0" />
                      {f}
                    </div>
                  ))}
                </div>
              </div>
              <div className="flex gap-2">
                <button onClick={reset} className="flex-1 py-2.5 rounded-xl border border-slate-200 text-sm font-semibold text-slate-600 hover:bg-slate-50 transition-colors">
                  Input Kode Lain
                </button>
                <button onClick={onClose} className={`flex-1 py-2.5 rounded-xl bg-gradient-to-r ${PKG_GRAD} text-white font-bold text-sm transition-all hover:-translate-y-0.5`}>
                  Mulai Belajar →
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Footer hint */}
        {state !== 'activated' && (
          <div className="px-6 pb-5">
            <p className="text-center text-xs text-muted-foreground">
              Belum punya voucher?{' '}
              <span className="text-indigo-600 font-semibold cursor-pointer hover:underline">Lihat cara mendapatkan voucher</span>
            </p>
          </div>
        )}
      </Card>
    </div>
  );
}
