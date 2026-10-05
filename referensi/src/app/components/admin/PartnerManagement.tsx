import { useState, useMemo } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Badge } from '../ui/badge';
import {
  Search, Plus, School, Users, TrendingUp, DollarSign,
  CheckCircle2, Clock, XCircle, MapPin, Mail, Phone,
  Edit, Trash2, X, Save, Eye, AlertCircle, BookOpen, Award,
  ChevronRight, Upload, UserPlus, Download, RefreshCw,
  Send, Copy, Key, ShieldCheck, GraduationCap
} from 'lucide-react';
import { toast } from 'sonner';

type SchoolStatus = 'active' | 'inactive' | 'pending';
type PackageType = 'basic' | 'premium' | 'enterprise';

interface Partner {
  id: string;
  name: string;
  type: 'SMA' | 'SMK' | 'MA';
  city: string;
  province: string;
  email: string;
  phone: string;
  joinDate: string;
  packageType: PackageType;
  totalStudents: number;
  activeStudents: number;
  totalTryouts: number;
  averageScore: number;
  contactPerson: string;
  status: SchoolStatus;
  revenueShare: number;
  monthlyRevenue: number;
  studentsImported?: boolean;
}

interface ImportedStudent {
  nisn: string;
  name: string;
  class: string;
  email: string;
  tempPassword: string;
  status: 'ready' | 'error';
  error?: string;
}

const initialPartners: Partner[] = [
  { id: 'S001', name: 'SMA Negeri 1 Jakarta', type: 'SMA', city: 'Jakarta', province: 'DKI Jakarta', email: 'sman1jkt@sekolah.id', phone: '021-1234567', joinDate: '2024-03-15', packageType: 'enterprise', totalStudents: 312, activeStudents: 287, totalTryouts: 48, averageScore: 678, contactPerson: 'Dra. Suhartini M.Pd', status: 'active', revenueShare: 20, monthlyRevenue: 8250000, studentsImported: true },
  { id: 'S002', name: 'SMA Negeri 3 Surabaya', type: 'SMA', city: 'Surabaya', province: 'Jawa Timur', email: 'sman3sby@sekolah.id', phone: '031-8765432', joinDate: '2024-04-20', packageType: 'premium', totalStudents: 245, activeStudents: 198, totalTryouts: 36, averageScore: 652, contactPerson: 'Dr. Bambang Priyo', status: 'active', revenueShare: 15, monthlyRevenue: 5400000, studentsImported: true },
  { id: 'S003', name: 'MA Negeri 1 Yogyakarta', type: 'MA', city: 'Yogyakarta', province: 'DI Yogyakarta', email: 'man1yk@sekolah.id', phone: '0274-567890', joinDate: '2024-06-10', packageType: 'premium', totalStudents: 189, activeStudents: 156, totalTryouts: 28, averageScore: 644, contactPerson: 'H. Ahmad Fathoni, M.Si', status: 'active', revenueShare: 15, monthlyRevenue: 3800000, studentsImported: true },
  { id: 'S004', name: 'SMK Negeri 2 Bandung', type: 'SMK', city: 'Bandung', province: 'Jawa Barat', email: 'smkn2bdg@sekolah.id', phone: '022-4567890', joinDate: '2024-07-05', packageType: 'basic', totalStudents: 134, activeStudents: 89, totalTryouts: 15, averageScore: 598, contactPerson: 'Ir. Hendra Kusuma', status: 'inactive', revenueShare: 10, monthlyRevenue: 0, studentsImported: true },
  { id: 'S005', name: 'SMA Negeri 5 Semarang', type: 'SMA', city: 'Semarang', province: 'Jawa Tengah', email: 'sman5smg@sekolah.id', phone: '024-7654321', joinDate: '2025-01-02', packageType: 'basic', totalStudents: 0, activeStudents: 0, totalTryouts: 0, averageScore: 0, contactPerson: 'Supriyanto, S.Pd', status: 'pending', revenueShare: 10, monthlyRevenue: 0, studentsImported: false },
  { id: 'S006', name: 'SMA Muhammadiyah 1 Makassar', type: 'SMA', city: 'Makassar', province: 'Sulawesi Selatan', email: 'smam1mks@sekolah.id', phone: '0411-345678', joinDate: '2024-09-18', packageType: 'premium', totalStudents: 178, activeStudents: 143, totalTryouts: 22, averageScore: 631, contactPerson: 'Dr. Rahmat Hidayat', status: 'active', revenueShare: 15, monthlyRevenue: 4200000, studentsImported: true },
];

const packageConfig: Record<PackageType, { label: string; color: string; maxStudents: number; price: number }> = {
  basic: { label: 'Basic', color: 'bg-slate-100 text-slate-700 border-slate-300', maxStudents: 150, price: 750000 },
  premium: { label: 'Premium', color: 'bg-blue-100 text-blue-700 border-blue-300', maxStudents: 300, price: 1500000 },
  enterprise: { label: 'Enterprise', color: 'bg-purple-100 text-purple-700 border-purple-300', maxStudents: 9999, price: 3500000 },
};

const statusConfig: Record<SchoolStatus, { label: string; icon: typeof CheckCircle2; color: string }> = {
  active: { label: 'Aktif', icon: CheckCircle2, color: 'bg-green-100 text-green-700 border-green-300' },
  inactive: { label: 'Tidak Aktif', icon: XCircle, color: 'bg-red-100 text-red-700 border-red-300' },
  pending: { label: 'Menunggu Persetujuan', icon: Clock, color: 'bg-amber-100 text-amber-700 border-amber-300' },
};

const emptyPartner: Omit<Partner, 'id' | 'totalStudents' | 'activeStudents' | 'totalTryouts' | 'averageScore' | 'monthlyRevenue' | 'studentsImported'> = {
  name: '', type: 'SMA', city: '', province: '', email: '', phone: '',
  joinDate: new Date().toISOString().split('T')[0],
  packageType: 'basic', contactPerson: '', status: 'pending', revenueShare: 10,
};

const CSV_TEMPLATE = `NISN,Nama Lengkap,Kelas,Email
0045678901,Budi Santoso,XII IPA 1,budi@siswa.sch.id
0045678902,Siti Aminah,XII IPA 2,siti@siswa.sch.id`;

function genPassword(nisn: string) {
  return `Gspl${nisn.slice(-4)}!`;
}

function parseCSV(raw: string): ImportedStudent[] {
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
    return {
      nisn, name, class: cls, email,
      tempPassword: genPassword(nisn || String(i)),
      status: errors.length ? 'error' : 'ready',
      error: errors.join(', '),
    } as ImportedStudent;
  });
}

// ─── Onboarding Wizard ───────────────────────────────────────────────────────
type WizardStep = 1 | 2 | 3;

interface WizardState {
  partner: Partner;
  step: WizardStep;
  csvText: string;
  students: ImportedStudent[];
  done: boolean;
}

function OnboardingWizard({ partner, onClose, onFinish }: {
  partner: Partner;
  onClose: () => void;
  onFinish: (partner: Partner, students: ImportedStudent[]) => void;
}) {
  const [step, setStep] = useState<WizardStep>(1);
  const [csvText, setCsvText] = useState('');
  const [students, setStudents] = useState<ImportedStudent[]>([]);
  const [done, setDone] = useState(false);
  const [schoolEmail, setSchoolEmail] = useState(partner.email);

  const quota = packageConfig[partner.packageType].maxStudents;
  const readyCount = students.filter(s => s.status === 'ready').length;
  const errorCount = students.filter(s => s.status === 'error').length;

  const handleParse = () => {
    const parsed = parseCSV(csvText);
    setStudents(parsed);
  };

  const handleConfirm = () => {
    const ready = students.filter(s => s.status === 'ready');
    onFinish({ ...partner, status: 'active', totalStudents: ready.length, activeStudents: ready.length }, ready);
    setDone(true);
  };

  const downloadTemplate = () => {
    const blob = new Blob([CSV_TEMPLATE], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = 'template_import_siswa.csv'; a.click();
    URL.revokeObjectURL(url);
    toast.success('Template CSV diunduh');
  };

  const copyCredentials = () => {
    const text = students.filter(s => s.status === 'ready')
      .map(s => `${s.name} | Username: ${s.nisn} | Password: ${s.tempPassword}`)
      .join('\n');
    navigator.clipboard.writeText(text);
    toast.success('Daftar kredensial disalin ke clipboard');
  };

  const stepLabels = ['Konfirmasi Mitra', 'Import Siswa', 'Aktivasi Akun'];

  if (done) {
    return (
      <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
        <div className="bg-white rounded-2xl w-full max-w-md shadow-2xl text-center p-10">
          <div className="w-20 h-20 rounded-full bg-green-100 flex items-center justify-center mx-auto mb-5">
            <ShieldCheck className="w-10 h-10 text-green-600" />
          </div>
          <h3 className="text-2xl font-black text-slate-900 mb-2">Onboarding Selesai!</h3>
          <p className="text-sm text-muted-foreground mb-2">
            <span className="font-bold text-slate-900">{partner.name}</span> kini aktif sebagai mitra.
          </p>
          <p className="text-sm text-muted-foreground mb-6">
            <span className="font-bold text-green-600">{readyCount} akun siswa</span> berhasil dibuat dan siap digunakan.
          </p>
          <div className="p-4 bg-blue-50 border border-blue-200 rounded-xl text-left mb-6">
            <p className="text-xs font-bold text-blue-700 mb-1.5 flex items-center gap-1"><Send className="w-3.5 h-3.5" />Langkah Selanjutnya</p>
            <ul className="text-xs text-blue-800 space-y-1.5">
              <li>• Kirim daftar kredensial ke PIC sekolah: <span className="font-bold">{schoolEmail}</span></li>
              <li>• Siswa login dengan NISN (username) dan password sementara</li>
              <li>• Siswa wajib ganti password saat login pertama kali</li>
            </ul>
          </div>
          <div className="flex gap-3">
            <Button variant="outline" className="flex-1 gap-2" onClick={copyCredentials}>
              <Copy className="w-4 h-4" /> Salin Kredensial
            </Button>
            <Button className="flex-1 bg-green-600 hover:bg-green-700" onClick={onClose}>
              Selesai
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4 overflow-y-auto">
      <div className="bg-white rounded-2xl w-full max-w-2xl my-4 shadow-2xl">
        {/* Wizard header */}
        <div className="p-6 border-b">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-black text-slate-900">Onboarding Mitra Sekolah</h3>
            <button onClick={onClose} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
          </div>
          {/* Step indicator */}
          <div className="flex items-center gap-0">
            {stepLabels.map((label, i) => {
              const s = (i + 1) as WizardStep;
              const isActive = step === s;
              const isDone = step > s;
              return (
                <div key={label} className="flex items-center flex-1">
                  <div className="flex flex-col items-center flex-1">
                    <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-black transition-colors ${isDone ? 'bg-green-500 text-white' : isActive ? 'bg-indigo-600 text-white' : 'bg-slate-100 text-slate-400'}`}>
                      {isDone ? <CheckCircle2 className="w-4 h-4" /> : s}
                    </div>
                    <p className={`text-[10px] font-semibold mt-1 text-center leading-tight ${isActive ? 'text-indigo-600' : isDone ? 'text-green-600' : 'text-slate-400'}`}>{label}</p>
                  </div>
                  {i < stepLabels.length - 1 && (
                    <div className={`h-0.5 w-full mx-1 rounded-full ${isDone ? 'bg-green-400' : 'bg-slate-200'}`} style={{ marginBottom: '16px' }} />
                  )}
                </div>
              );
            })}
          </div>
        </div>

        {/* Step 1: Confirm partner details */}
        {step === 1 && (
          <div className="p-6 space-y-5">
            <div className="flex items-start gap-4 p-4 bg-blue-50 border border-blue-200 rounded-xl">
              <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center text-white shrink-0">
                <School className="w-6 h-6" />
              </div>
              <div>
                <h4 className="font-black text-slate-900">{partner.name}</h4>
                <p className="text-sm text-muted-foreground">{partner.type} · {partner.city}, {partner.province}</p>
                <div className="flex items-center gap-2 mt-1.5">
                  <Badge className={packageConfig[partner.packageType].color}>{packageConfig[partner.packageType].label}</Badge>
                  <span className="text-xs text-muted-foreground">Maks. {quota === 9999 ? 'Unlimited' : quota} siswa</span>
                </div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              {[
                { label: 'PIC / Kontak', val: partner.contactPerson, icon: Users },
                { label: 'Email Sekolah', val: partner.email, icon: Mail },
                { label: 'Telepon', val: partner.phone, icon: Phone },
                { label: 'Revenue Share', val: `${partner.revenueShare}%`, icon: DollarSign },
              ].map(({ label, val, icon: Icon }) => (
                <div key={label} className="flex items-center gap-3 p-3 bg-slate-50 rounded-xl">
                  <Icon className="w-4 h-4 text-slate-400 shrink-0" />
                  <div>
                    <p className="text-xs text-muted-foreground">{label}</p>
                    <p className="text-sm font-semibold text-slate-900">{val || '—'}</p>
                  </div>
                </div>
              ))}
            </div>

            <div>
              <Label>Email untuk terima kredensial siswa *</Label>
              <Input className="mt-1.5" value={schoolEmail} onChange={e => setSchoolEmail(e.target.value)} placeholder="email PIC sekolah" />
              <p className="text-xs text-muted-foreground mt-1">Daftar akun siswa akan dikirim ke email ini setelah onboarding selesai.</p>
            </div>

            <div className="p-4 bg-amber-50 border border-amber-200 rounded-xl flex items-start gap-2">
              <AlertCircle className="w-4 h-4 text-amber-600 shrink-0 mt-0.5" />
              <p className="text-xs text-amber-800">Dengan menyetujui, status mitra akan berubah dari <strong>Pending</strong> ke <strong>Aktif</strong> setelah import siswa selesai.</p>
            </div>
          </div>
        )}

        {/* Step 2: Import students */}
        {step === 2 && (
          <div className="p-6 space-y-5">
            <div className="flex items-center justify-between">
              <div>
                <h4 className="font-bold text-slate-900">Import Data Siswa</h4>
                <p className="text-sm text-muted-foreground">Paste data CSV atau ketik manual. Kuota: <span className="font-bold text-indigo-600">{quota === 9999 ? 'Unlimited' : quota} siswa</span></p>
              </div>
              <Button variant="outline" size="sm" className="gap-2" onClick={downloadTemplate}>
                <Download className="w-4 h-4" /> Template CSV
              </Button>
            </div>

            <div className="p-3 bg-slate-50 border rounded-xl">
              <p className="text-xs font-mono text-slate-500 mb-1">Format CSV yang diterima:</p>
              <p className="text-xs font-mono text-slate-700">NISN, Nama Lengkap, Kelas, Email</p>
              <p className="text-xs font-mono text-slate-500">0045678901, Budi Santoso, XII IPA 1, budi@siswa.sch.id</p>
            </div>

            <div>
              <Label>Paste data CSV di sini</Label>
              <textarea
                className="w-full mt-1.5 px-3 py-3 border rounded-xl text-sm font-mono resize-none h-36 focus:outline-none focus:ring-2 focus:ring-indigo-300"
                value={csvText}
                onChange={e => { setCsvText(e.target.value); setStudents([]); }}
                placeholder={"NISN,Nama Lengkap,Kelas,Email\n0045678901,Budi Santoso,XII IPA 1,budi@siswa.sch.id\n0045678902,Siti Aminah,XII IPA 2,siti@siswa.sch.id"}
              />
            </div>

            <Button onClick={handleParse} className="w-full bg-indigo-600 hover:bg-indigo-700 gap-2" disabled={!csvText.trim()}>
              <RefreshCw className="w-4 h-4" /> Proses & Validasi Data
            </Button>

            {/* Preview table */}
            {students.length > 0 && (
              <div>
                <div className="flex items-center justify-between mb-2">
                  <p className="text-sm font-semibold text-slate-700">
                    Hasil validasi: <span className="text-green-600">{readyCount} siap</span>
                    {errorCount > 0 && <>, <span className="text-red-500">{errorCount} error</span></>}
                  </p>
                </div>
                <div className="border rounded-xl overflow-hidden max-h-52 overflow-y-auto">
                  <table className="w-full text-xs">
                    <thead className="sticky top-0 bg-slate-50">
                      <tr className="border-b">
                        <th className="text-left px-3 py-2 font-semibold text-slate-600">NISN</th>
                        <th className="text-left px-3 py-2 font-semibold text-slate-600">Nama</th>
                        <th className="text-left px-3 py-2 font-semibold text-slate-600">Kelas</th>
                        <th className="text-left px-3 py-2 font-semibold text-slate-600">Status</th>
                      </tr>
                    </thead>
                    <tbody>
                      {students.map((s, i) => (
                        <tr key={i} className={`border-b ${s.status === 'error' ? 'bg-red-50' : 'hover:bg-slate-50'}`}>
                          <td className="px-3 py-2 font-mono">{s.nisn || '—'}</td>
                          <td className="px-3 py-2 font-semibold">{s.name || '—'}</td>
                          <td className="px-3 py-2 text-slate-600">{s.class || '—'}</td>
                          <td className="px-3 py-2">
                            {s.status === 'ready'
                              ? <span className="text-green-600 font-semibold flex items-center gap-1"><CheckCircle2 className="w-3 h-3" />Siap</span>
                              : <span className="text-red-500 font-semibold flex items-center gap-1"><AlertCircle className="w-3 h-3" />{s.error}</span>
                            }
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                {readyCount > quota && (
                  <p className="text-xs text-red-600 mt-2 flex items-center gap-1"><AlertCircle className="w-3 h-3" />Melebihi kuota paket ({quota} siswa). Upgrade paket atau kurangi jumlah siswa.</p>
                )}
              </div>
            )}
          </div>
        )}

        {/* Step 3: Activate accounts */}
        {step === 3 && (
          <div className="p-6 space-y-5">
            <div>
              <h4 className="font-bold text-slate-900 mb-1">Preview Akun yang Akan Dibuat</h4>
              <p className="text-sm text-muted-foreground">
                <span className="font-bold text-green-600">{readyCount} akun siswa</span> akan diaktifkan. Siswa login dengan NISN + password sementara.
              </p>
            </div>

            <div className="p-4 bg-indigo-50 border border-indigo-200 rounded-xl">
              <p className="text-xs font-bold text-indigo-700 mb-2 flex items-center gap-1.5"><Key className="w-3.5 h-3.5" />Aturan Akun Siswa</p>
              <ul className="text-xs text-indigo-800 space-y-1">
                <li>• <strong>Username:</strong> NISN siswa (10 digit)</li>
                <li>• <strong>Password sementara:</strong> <code className="bg-indigo-100 px-1 rounded">Gspl[4 digit terakhir NISN]!</code></li>
                <li>• Siswa wajib ganti password saat login pertama</li>
                <li>• Admin sekolah dapat reset password dari dashboard sekolah</li>
              </ul>
            </div>

            {/* Sample preview */}
            <div className="border rounded-xl overflow-hidden max-h-56 overflow-y-auto">
              <table className="w-full text-xs">
                <thead className="sticky top-0 bg-slate-50">
                  <tr className="border-b">
                    <th className="text-left px-3 py-2 font-semibold text-slate-600">Nama</th>
                    <th className="text-left px-3 py-2 font-semibold text-slate-600">Username (NISN)</th>
                    <th className="text-left px-3 py-2 font-semibold text-slate-600">Password Sementara</th>
                    <th className="text-left px-3 py-2 font-semibold text-slate-600">Kelas</th>
                  </tr>
                </thead>
                <tbody>
                  {students.filter(s => s.status === 'ready').map((s, i) => (
                    <tr key={i} className="border-b hover:bg-slate-50">
                      <td className="px-3 py-2 font-semibold">{s.name}</td>
                      <td className="px-3 py-2 font-mono text-indigo-600">{s.nisn}</td>
                      <td className="px-3 py-2 font-mono text-purple-600">{s.tempPassword}</td>
                      <td className="px-3 py-2 text-slate-600">{s.class}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div className="p-3 bg-emerald-50 border border-emerald-200 rounded-xl text-center">
                <p className="text-2xl font-black text-emerald-600">{readyCount}</p>
                <p className="text-xs text-emerald-700 font-semibold">Akun siap dibuat</p>
              </div>
              <div className="p-3 bg-slate-50 border rounded-xl text-center">
                <p className="text-2xl font-black text-slate-700">{packageConfig[partner.packageType].maxStudents === 9999 ? '∞' : packageConfig[partner.packageType].maxStudents - readyCount}</p>
                <p className="text-xs text-slate-500 font-semibold">Sisa kuota</p>
              </div>
            </div>

            <div className="p-4 bg-green-50 border border-green-200 rounded-xl flex items-start gap-2">
              <ShieldCheck className="w-4 h-4 text-green-600 shrink-0 mt-0.5" />
              <div className="text-xs text-green-800">
                <p className="font-bold mb-0.5">Setelah konfirmasi:</p>
                <p>• Status mitra berubah ke <strong>Aktif</strong></p>
                <p>• {readyCount} akun siswa langsung aktif dan bisa login</p>
                <p>• Daftar kredensial dikirim ke <strong>{schoolEmail}</strong></p>
              </div>
            </div>
          </div>
        )}

        {/* Footer nav */}
        <div className="flex items-center justify-between p-6 border-t bg-slate-50 rounded-b-2xl">
          <Button variant="outline" onClick={() => step === 1 ? onClose() : setStep(s => (s - 1) as WizardStep)}>
            {step === 1 ? 'Batal' : '← Kembali'}
          </Button>
          <div className="flex items-center gap-1">
            {[1, 2, 3].map(s => <div key={s} className={`w-2 h-2 rounded-full ${step === s ? 'bg-indigo-600' : 'bg-slate-300'}`} />)}
          </div>
          {step < 3 ? (
            <Button
              className="bg-gradient-to-r from-indigo-600 to-purple-600 gap-2"
              onClick={() => setStep(s => (s + 1) as WizardStep)}
              disabled={step === 2 && (students.length === 0 || readyCount === 0 || readyCount > packageConfig[partner.packageType].maxStudents)}
            >
              Lanjut <ChevronRight className="w-4 h-4" />
            </Button>
          ) : (
            <Button className="bg-green-600 hover:bg-green-700 gap-2" onClick={handleConfirm}>
              <ShieldCheck className="w-4 h-4" /> Konfirmasi & Aktifkan
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}

// ─── Main Component ───────────────────────────────────────────────────────────
export default function PartnerManagement() {
  const [partners, setPartners] = useState<Partner[]>(initialPartners);
  const [search, setSearch] = useState('');
  const [filterStatus, setFilterStatus] = useState<SchoolStatus | 'all'>('all');
  const [filterPkg, setFilterPkg] = useState<PackageType | 'all'>('all');
  const [modalOpen, setModalOpen] = useState(false);
  const [editTarget, setEditTarget] = useState<Partner | null>(null);
  const [form, setForm] = useState(emptyPartner);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [detailPartner, setDetailPartner] = useState<Partner | null>(null);
  const [wizardPartner, setWizardPartner] = useState<Partner | null>(null);
  // For re-import (add more students to active school)
  const [importTarget, setImportTarget] = useState<Partner | null>(null);

  const filtered = partners.filter(p => {
    const ms = !search || p.name.toLowerCase().includes(search.toLowerCase()) || p.city.toLowerCase().includes(search.toLowerCase());
    const mSt = filterStatus === 'all' || p.status === filterStatus;
    const mPk = filterPkg === 'all' || p.packageType === filterPkg;
    return ms && mSt && mPk;
  });

  const stats = useMemo(() => ({
    total: partners.length,
    active: partners.filter(p => p.status === 'active').length,
    pending: partners.filter(p => p.status === 'pending').length,
    totalStudents: partners.reduce((s, p) => s + p.totalStudents, 0),
    totalRevenue: partners.reduce((s, p) => s + p.monthlyRevenue, 0),
  }), [partners]);

  const openCreate = () => { setEditTarget(null); setForm(emptyPartner); setModalOpen(true); };
  const openEdit = (p: Partner) => {
    setEditTarget(p);
    setForm({ name: p.name, type: p.type, city: p.city, province: p.province, email: p.email, phone: p.phone, joinDate: p.joinDate, packageType: p.packageType, contactPerson: p.contactPerson, status: p.status, revenueShare: p.revenueShare });
    setModalOpen(true);
  };

  const handleSave = () => {
    if (!form.name.trim()) { toast.error('Nama sekolah wajib diisi'); return; }
    if (!form.email.trim()) { toast.error('Email wajib diisi'); return; }
    if (editTarget) {
      setPartners(prev => prev.map(p => p.id === editTarget.id ? { ...p, ...form } : p));
      toast.success(`Data "${form.name}" berhasil diperbarui`);
    } else {
      const newP: Partner = { ...form, id: `S${Date.now()}`, totalStudents: 0, activeStudents: 0, totalTryouts: 0, averageScore: 0, monthlyRevenue: 0, studentsImported: false };
      setPartners(prev => [newP, ...prev]);
      toast.success(`Mitra "${form.name}" ditambahkan — lanjutkan dengan onboarding siswa`);
    }
    setModalOpen(false);
  };

  const handleApproveWithWizard = (partner: Partner) => setWizardPartner(partner);

  const handleWizardFinish = (updatedPartner: Partner, students: ImportedStudent[]) => {
    setPartners(prev => prev.map(p =>
      p.id === updatedPartner.id
        ? { ...updatedPartner, studentsImported: true }
        : p
    ));
    toast.success(`${students.length} akun siswa berhasil dibuat untuk ${updatedPartner.name}`);
  };

  const handleWizardClose = () => setWizardPartner(null);

  const handleDeactivate = (id: string) => {
    setPartners(prev => prev.map(p => p.id === id ? { ...p, status: 'inactive' } : p));
    toast.success('Mitra dinonaktifkan');
  };

  const handleReactivate = (id: string) => {
    setPartners(prev => prev.map(p => p.id === id ? { ...p, status: 'active' } : p));
    toast.success('Mitra diaktifkan kembali');
  };

  const handleDelete = (id: string) => {
    const p = partners.find(p => p.id === id);
    setPartners(prev => prev.filter(p => p.id !== id));
    setDeleteId(null);
    setDetailPartner(null);
    toast.success(`"${p?.name}" dihapus dari mitra`);
  };

  const handleImportMore = (p: Partner) => setImportTarget(p);

  const handleImportMoreFinish = (updated: Partner, students: ImportedStudent[]) => {
    setPartners(prev => prev.map(p =>
      p.id === updated.id
        ? { ...p, totalStudents: p.totalStudents + students.length, activeStudents: p.activeStudents + students.length }
        : p
    ));
    toast.success(`${students.length} siswa tambahan berhasil diimport`);
  };

  const setF = <K extends keyof typeof form>(k: K, v: (typeof form)[K]) => setForm(f => ({ ...f, [k]: v }));

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card className="p-6">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div>
            <h2 className="text-2xl font-bold mb-1">Manajemen Mitra Sekolah</h2>
            <p className="text-sm text-muted-foreground">Kelola sekolah partner B2B, onboarding siswa, dan monitoring performa</p>
          </div>
          <Button onClick={openCreate} className="gap-2 bg-gradient-to-r from-blue-600 to-cyan-600 hover:from-blue-700 hover:to-cyan-700">
            <Plus className="w-4 h-4" /> Tambah Mitra
          </Button>
        </div>
      </Card>

      {/* Pending banner */}
      {stats.pending > 0 && (
        <div className="bg-amber-50 border border-amber-200 rounded-xl p-4 flex items-center gap-3">
          <AlertCircle className="w-5 h-5 text-amber-600 shrink-0" />
          <p className="text-sm text-amber-800">
            <span className="font-bold">{stats.pending} sekolah</span> menunggu persetujuan. Klik <strong>"Setujui & Onboarding"</strong> untuk memulai wizard import siswa.
          </p>
          <button onClick={() => setFilterStatus('pending')} className="ml-auto text-xs font-bold text-amber-700 underline hover:text-amber-900 shrink-0">
            Lihat Sekarang
          </button>
        </div>
      )}

      {/* Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-5 gap-4">
        {[
          { label: 'Total Mitra', val: stats.total, color: 'border-l-green-500', icolor: 'text-green-600', ibg: 'bg-green-100', icon: School },
          { label: 'Aktif', val: stats.active, color: 'border-l-blue-500', icolor: 'text-blue-600', ibg: 'bg-blue-100', icon: CheckCircle2 },
          { label: 'Menunggu', val: stats.pending, color: 'border-l-amber-500', icolor: 'text-amber-600', ibg: 'bg-amber-100', icon: Clock },
          { label: 'Total Siswa', val: stats.totalStudents.toLocaleString('id-ID'), color: 'border-l-purple-500', icolor: 'text-purple-600', ibg: 'bg-purple-100', icon: Users },
          { label: 'Rev. B2B/bulan', val: `Rp ${(stats.totalRevenue / 1e6).toFixed(1)}jt`, color: 'border-l-rose-500', icolor: 'text-rose-600', ibg: 'bg-rose-100', icon: DollarSign },
        ].map(({ label, val, color, icolor, ibg, icon: Icon }) => (
          <Card key={label} className={`p-4 border-l-4 ${color}`}>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-xs text-muted-foreground mb-1">{label}</p>
                <h3 className="text-xl font-bold">{val}</h3>
              </div>
              <div className={`w-9 h-9 rounded-lg ${ibg} flex items-center justify-center`}>
                <Icon className={`w-4 h-4 ${icolor}`} />
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
            <Input placeholder="Cari sekolah atau kota..." className="pl-10" value={search} onChange={e => setSearch(e.target.value)} />
          </div>
          <select className="px-3 py-2 border rounded-xl text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterStatus} onChange={e => setFilterStatus(e.target.value as SchoolStatus | 'all')}>
            <option value="all">Semua Status</option>
            <option value="active">Aktif</option>
            <option value="inactive">Tidak Aktif</option>
            <option value="pending">Menunggu</option>
          </select>
          <select className="px-3 py-2 border rounded-xl text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterPkg} onChange={e => setFilterPkg(e.target.value as PackageType | 'all')}>
            <option value="all">Semua Paket</option>
            <option value="basic">Basic</option>
            <option value="premium">Premium</option>
            <option value="enterprise">Enterprise</option>
          </select>
        </div>
      </Card>

      {/* Partners grid */}
      <div className="grid lg:grid-cols-2 gap-5">
        {filtered.length === 0 && (
          <div className="lg:col-span-2">
            <Card className="p-10 text-center">
              <School className="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" />
              <p className="text-muted-foreground">Tidak ada mitra ditemukan</p>
            </Card>
          </div>
        )}
        {filtered.map(partner => {
          const sc = statusConfig[partner.status];
          const pc = packageConfig[partner.packageType];
          const StatusIcon = sc.icon;
          return (
            <Card key={partner.id} className={`p-5 hover:shadow-lg transition-shadow ${partner.status === 'pending' ? 'border-amber-300 border-2' : ''}`}>
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-start gap-3">
                  <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center text-white shrink-0">
                    <School className="w-6 h-6" />
                  </div>
                  <div>
                    <h3 className="font-bold text-slate-900 leading-tight">{partner.name}</h3>
                    <div className="flex items-center gap-1.5 text-xs text-muted-foreground mt-0.5">
                      <MapPin className="w-3 h-3" />{partner.city}, {partner.province}
                    </div>
                  </div>
                </div>
                <div className="flex flex-col items-end gap-1.5">
                  <Badge variant="outline" className={`text-xs gap-1 ${sc.color}`}>
                    <StatusIcon className="w-3 h-3" />{sc.label}
                  </Badge>
                  <Badge variant="outline" className={`text-xs ${pc.color}`}>{pc.label}</Badge>
                </div>
              </div>

              {/* Stats */}
              <div className="grid grid-cols-4 gap-2 mb-4">
                {[
                  { label: 'Total Siswa', val: partner.totalStudents },
                  { label: 'Aktif', val: partner.activeStudents, colored: true },
                  { label: 'Tryout', val: partner.totalTryouts },
                  { label: 'Avg Score', val: partner.averageScore || '—' },
                ].map(({ label, val, colored }) => (
                  <div key={label} className="p-2.5 bg-slate-50 rounded-lg text-center">
                    <p className={`font-bold text-sm ${colored ? 'text-green-600' : 'text-slate-900'}`}>{val}</p>
                    <p className="text-[10px] text-muted-foreground mt-0.5 leading-tight">{label}</p>
                  </div>
                ))}
              </div>

              {/* Onboarding needed indicator */}
              {partner.status === 'active' && !partner.studentsImported && (
                <div className="flex items-center gap-2 p-2.5 bg-indigo-50 border border-indigo-200 rounded-lg mb-3">
                  <GraduationCap className="w-4 h-4 text-indigo-600 shrink-0" />
                  <p className="text-xs text-indigo-800 font-medium flex-1">Belum ada siswa diimport</p>
                  <button onClick={() => handleImportMore(partner)} className="text-xs text-indigo-600 font-bold underline">Import Sekarang</button>
                </div>
              )}

              {/* Revenue share */}
              {partner.status === 'active' && partner.monthlyRevenue > 0 && (
                <div className="flex items-center justify-between p-3 bg-gradient-to-r from-green-50 to-emerald-50 border border-green-200 rounded-lg mb-4">
                  <div className="flex items-center gap-2 text-xs text-green-700">
                    <DollarSign className="w-3.5 h-3.5" />
                    <span>Revenue share ({partner.revenueShare}%)</span>
                  </div>
                  <span className="text-sm font-black text-green-700">
                    Rp {Math.round(partner.monthlyRevenue * partner.revenueShare / 100).toLocaleString('id-ID')}/bln
                  </span>
                </div>
              )}

              {/* Contact */}
              <div className="space-y-1.5 mb-4">
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  <Mail className="w-3.5 h-3.5" />{partner.email}
                </div>
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  <Phone className="w-3.5 h-3.5" />{partner.phone}
                </div>
              </div>

              {/* Actions */}
              <div className="flex items-center justify-between pt-3 border-t gap-2">
                <p className="text-xs text-muted-foreground shrink-0">Sejak {partner.joinDate}</p>
                <div className="flex gap-1.5 flex-wrap justify-end">
                  {partner.status === 'pending' && (
                    <>
                      <Button size="sm" className="gap-1.5 bg-gradient-to-r from-indigo-600 to-purple-600 text-xs" onClick={() => handleApproveWithWizard(partner)}>
                        <UserPlus className="w-3.5 h-3.5" /> Setujui & Onboarding
                      </Button>
                      <Button size="sm" variant="outline" className="gap-1.5 text-red-600 border-red-200 text-xs" onClick={() => setDeleteId(partner.id)}>
                        <XCircle className="w-3.5 h-3.5" /> Tolak
                      </Button>
                    </>
                  )}
                  {partner.status === 'active' && (
                    <>
                      <Button size="sm" variant="outline" className="gap-1.5 text-xs" onClick={() => handleImportMore(partner)}>
                        <Upload className="w-3.5 h-3.5" /> Import Siswa
                      </Button>
                      <Button size="sm" variant="outline" className="text-xs" onClick={() => handleDeactivate(partner.id)}>
                        Nonaktifkan
                      </Button>
                    </>
                  )}
                  {partner.status === 'inactive' && (
                    <Button size="sm" variant="outline" className="gap-1.5 text-green-700 border-green-300 text-xs" onClick={() => handleReactivate(partner.id)}>
                      <CheckCircle2 className="w-3.5 h-3.5" /> Aktifkan
                    </Button>
                  )}
                  <Button size="sm" variant="outline" className="text-xs" onClick={() => setDetailPartner(partner)}>
                    <Eye className="w-3.5 h-3.5" />
                  </Button>
                  <Button size="sm" variant="outline" className="text-xs" onClick={() => openEdit(partner)}>
                    <Edit className="w-3.5 h-3.5" />
                  </Button>
                  <Button size="sm" variant="outline" className="text-xs text-red-600 border-red-200 hover:bg-red-50" onClick={() => setDeleteId(partner.id)}>
                    <Trash2 className="w-3.5 h-3.5" />
                  </Button>
                </div>
              </div>
            </Card>
          );
        })}
      </div>

      <div className="text-center text-sm text-muted-foreground">
        Menampilkan {filtered.length} dari {partners.length} mitra
      </div>

      {/* Onboarding wizard (pending → approve + import students) */}
      {wizardPartner && (
        <OnboardingWizard
          partner={wizardPartner}
          onClose={handleWizardClose}
          onFinish={(p, students) => {
            handleWizardFinish(p, students);
          }}
        />
      )}

      {/* Re-import wizard (active school, add more students) */}
      {importTarget && (
        <OnboardingWizard
          partner={importTarget}
          onClose={() => setImportTarget(null)}
          onFinish={(p, students) => {
            handleImportMoreFinish(p, students);
            setImportTarget(null);
          }}
        />
      )}

      {/* Create/Edit Modal */}
      {modalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4 overflow-y-auto">
          <div className="bg-white rounded-2xl w-full max-w-xl my-4 shadow-2xl">
            <div className="flex items-center justify-between p-6 border-b">
              <h3 className="text-lg font-bold">{editTarget ? 'Edit Mitra' : 'Tambah Mitra Baru'}</h3>
              <button onClick={() => setModalOpen(false)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-6 space-y-4 overflow-y-auto max-h-[65vh]">
              <div>
                <Label>Nama Sekolah *</Label>
                <Input className="mt-1.5" value={form.name} onChange={e => setF('name', e.target.value)} placeholder="SMA Negeri X Kota Y" />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>Jenis Sekolah</Label>
                  <select className="w-full mt-1.5 px-3 py-2 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.type} onChange={e => setF('type', e.target.value as 'SMA' | 'SMK' | 'MA')}>
                    <option value="SMA">SMA</option>
                    <option value="SMK">SMK</option>
                    <option value="MA">MA</option>
                  </select>
                </div>
                <div>
                  <Label>Paket</Label>
                  <select className="w-full mt-1.5 px-3 py-2 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.packageType} onChange={e => setF('packageType', e.target.value as PackageType)}>
                    <option value="basic">Basic (≤150 siswa)</option>
                    <option value="premium">Premium (≤300 siswa)</option>
                    <option value="enterprise">Enterprise (unlimited)</option>
                  </select>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>Kota</Label>
                  <Input className="mt-1.5" value={form.city} onChange={e => setF('city', e.target.value)} placeholder="Jakarta" />
                </div>
                <div>
                  <Label>Provinsi</Label>
                  <Input className="mt-1.5" value={form.province} onChange={e => setF('province', e.target.value)} placeholder="DKI Jakarta" />
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>Email *</Label>
                  <Input type="email" className="mt-1.5" value={form.email} onChange={e => setF('email', e.target.value)} placeholder="sekolah@example.id" />
                </div>
                <div>
                  <Label>No. Telepon</Label>
                  <Input className="mt-1.5" value={form.phone} onChange={e => setF('phone', e.target.value)} placeholder="021-XXXXXXX" />
                </div>
              </div>
              <div>
                <Label>Nama PIC / Kontak Person</Label>
                <Input className="mt-1.5" value={form.contactPerson} onChange={e => setF('contactPerson', e.target.value)} placeholder="Nama kepala sekolah / koordinator" />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>Revenue Share (%)</Label>
                  <Input type="number" className="mt-1.5" value={form.revenueShare} onChange={e => setF('revenueShare', Number(e.target.value))} min={0} max={50} />
                </div>
                <div>
                  <Label>Status Awal</Label>
                  <select className="w-full mt-1.5 px-3 py-2 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.status} onChange={e => setF('status', e.target.value as SchoolStatus)}>
                    <option value="pending">Pending (onboarding wizard)</option>
                    <option value="active">Langsung Aktifkan</option>
                    <option value="inactive">Tidak Aktif</option>
                  </select>
                </div>
              </div>
              {!editTarget && (
                <div className="p-3 bg-indigo-50 border border-indigo-200 rounded-xl text-xs text-indigo-800 flex items-start gap-2">
                  <GraduationCap className="w-4 h-4 shrink-0 mt-0.5 text-indigo-600" />
                  Setelah mitra ditambahkan, gunakan tombol <strong>"Setujui & Onboarding"</strong> untuk import siswa dan generate akun secara massal.
                </div>
              )}
            </div>
            <div className="flex gap-3 p-6 border-t">
              <Button variant="outline" className="flex-1" onClick={() => setModalOpen(false)}>Batal</Button>
              <Button className="flex-1 bg-gradient-to-r from-blue-600 to-cyan-600 gap-2" onClick={handleSave}>
                <Save className="w-4 h-4" /> {editTarget ? 'Simpan' : 'Tambah Mitra'}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Detail Panel */}
      {detailPartner && (
        <div className="fixed inset-0 bg-black/40 flex items-center justify-end z-50 p-4">
          <div className="bg-white h-full w-full max-w-sm rounded-2xl shadow-2xl overflow-y-auto">
            <div className="flex items-center justify-between p-5 border-b">
              <h3 className="font-bold text-slate-900">Detail Mitra</h3>
              <button onClick={() => setDetailPartner(null)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-4 h-4" /></button>
            </div>
            <div className="p-5">
              <div className="flex items-center gap-3 mb-5">
                <div className="w-14 h-14 rounded-xl bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center text-white">
                  <School className="w-7 h-7" />
                </div>
                <div>
                  <h4 className="font-bold text-slate-900 text-sm">{detailPartner.name}</h4>
                  <p className="text-xs text-muted-foreground">{detailPartner.type} · {detailPartner.city}</p>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-3 mb-5">
                {[
                  { icon: Users, label: 'Total Siswa', val: detailPartner.totalStudents },
                  { icon: CheckCircle2, label: 'Siswa Aktif', val: detailPartner.activeStudents },
                  { icon: BookOpen, label: 'Total Tryout', val: detailPartner.totalTryouts },
                  { icon: Award, label: 'Avg Score', val: detailPartner.averageScore || '—' },
                ].map(({ icon: Icon, label, val }) => (
                  <div key={label} className="bg-slate-50 rounded-xl p-3">
                    <Icon className="w-4 h-4 text-blue-500 mb-1.5" />
                    <p className="text-lg font-black text-slate-900">{val}</p>
                    <p className="text-[10px] text-muted-foreground">{label}</p>
                  </div>
                ))}
              </div>
              <div className="space-y-3 mb-5">
                {[
                  ['Status', statusConfig[detailPartner.status].label],
                  ['Paket', packageConfig[detailPartner.packageType].label],
                  ['Revenue Share', `${detailPartner.revenueShare}%`],
                  ['Rev. Sekolah/bln', detailPartner.monthlyRevenue > 0 ? `Rp ${Math.round(detailPartner.monthlyRevenue * detailPartner.revenueShare / 100).toLocaleString('id-ID')}` : '—'],
                  ['PIC', detailPartner.contactPerson],
                  ['Email', detailPartner.email],
                  ['Telepon', detailPartner.phone],
                  ['Bergabung', detailPartner.joinDate],
                ].map(([label, val]) => (
                  <div key={label} className="flex justify-between py-2 border-b last:border-0">
                    <span className="text-xs text-muted-foreground">{label}</span>
                    <span className="text-xs font-semibold text-slate-800 text-right max-w-[55%]">{val}</span>
                  </div>
                ))}
              </div>
              <div className="flex flex-col gap-2">
                <div className="flex gap-2">
                  <Button variant="outline" size="sm" className="flex-1 gap-1.5" onClick={() => { openEdit(detailPartner); setDetailPartner(null); }}>
                    <Edit className="w-3.5 h-3.5" /> Edit
                  </Button>
                  {detailPartner.status === 'active' && (
                    <Button size="sm" className="flex-1 gap-1.5 bg-indigo-600 hover:bg-indigo-700" onClick={() => { handleImportMore(detailPartner); setDetailPartner(null); }}>
                      <Upload className="w-3.5 h-3.5" /> Import Siswa
                    </Button>
                  )}
                </div>
                {detailPartner.status === 'pending' && (
                  <Button size="sm" className="w-full bg-gradient-to-r from-indigo-600 to-purple-600 gap-1.5" onClick={() => { handleApproveWithWizard(detailPartner); setDetailPartner(null); }}>
                    <UserPlus className="w-3.5 h-3.5" /> Setujui & Onboarding Siswa
                  </Button>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Delete confirm */}
      {deleteId && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-2xl p-6 w-full max-w-sm shadow-2xl">
            <div className="w-12 h-12 rounded-full bg-red-100 flex items-center justify-center mx-auto mb-4">
              <AlertCircle className="w-6 h-6 text-red-600" />
            </div>
            <h3 className="font-bold text-slate-900 text-center mb-2">Hapus Mitra?</h3>
            <p className="text-sm text-muted-foreground text-center mb-6">Semua data mitra dan siswa terkait akan dihapus permanen.</p>
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
