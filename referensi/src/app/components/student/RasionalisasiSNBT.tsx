import { useState } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Badge } from '../ui/badge';
import {
  Target, TrendingUp, AlertCircle, CheckCircle2, School, Users,
  Award, ArrowRight, Plus, Edit, Trash2, X, Save, Search,
  BarChart3, Info, BookOpen, ChevronDown, ChevronUp, Star
} from 'lucide-react';
import { toast } from 'sonner';

interface PTNTarget {
  id: string;
  university: string;
  abbr: string;
  major: string;
  passingGrade: number;
  applicants: number;
  capacity: number;
  priority: 'utama' | 'cadangan' | 'aman';
  note: string;
}

const popularUniversities = [
  { name: 'Universitas Indonesia', abbr: 'UI' },
  { name: 'Institut Teknologi Bandung', abbr: 'ITB' },
  { name: 'Universitas Gadjah Mada', abbr: 'UGM' },
  { name: 'Institut Pertanian Bogor', abbr: 'IPB' },
  { name: 'Universitas Airlangga', abbr: 'UNAIR' },
  { name: 'Universitas Padjadjaran', abbr: 'UNPAD' },
  { name: 'Universitas Diponegoro', abbr: 'UNDIP' },
  { name: 'Institut Teknologi Sepuluh Nopember', abbr: 'ITS' },
  { name: 'Universitas Brawijaya', abbr: 'UB' },
  { name: 'Universitas Sebelas Maret', abbr: 'UNS' },
  { name: 'Universitas Hasanuddin', abbr: 'UNHAS' },
  { name: 'Universitas Negeri Yogyakarta', abbr: 'UNY' },
];

const initialTargets: PTNTarget[] = [
  { id: 't1', university: 'Universitas Indonesia', abbr: 'UI', major: 'Teknik Informatika', passingGrade: 680, applicants: 2450, capacity: 60, priority: 'utama', note: 'Target utama — perlu skor minimal 680' },
  { id: 't2', university: 'Universitas Gadjah Mada', abbr: 'UGM', major: 'Ilmu Komputer', passingGrade: 675, applicants: 1980, capacity: 70, priority: 'cadangan', note: 'Pilihan kedua jika UI tidak tembus' },
  { id: 't3', university: 'Universitas Padjadjaran', abbr: 'UNPAD', major: 'Informatika', passingGrade: 665, applicants: 1560, capacity: 65, priority: 'aman', note: 'Pilihan aman dengan passing grade lebih rendah' },
];

const priorityConfig = {
  utama: { label: 'Target Utama', color: 'bg-indigo-100 text-indigo-700 border-indigo-300', border: 'border-l-indigo-500', star: 3 },
  cadangan: { label: 'Cadangan', color: 'bg-amber-100 text-amber-700 border-amber-300', border: 'border-l-amber-500', star: 2 },
  aman: { label: 'Pilihan Aman', color: 'bg-emerald-100 text-emerald-700 border-emerald-300', border: 'border-l-emerald-500', star: 1 },
};

const emptyTarget: Omit<PTNTarget, 'id'> = {
  university: '', abbr: '', major: '', passingGrade: 650, applicants: 1000, capacity: 50, priority: 'cadangan', note: '',
};

const MY_SCORE = 685;

function calcChance(myScore: number, passingGrade: number, applicants: number, capacity: number): number {
  const scoreDiff = myScore - passingGrade;
  const ratio = capacity / applicants;
  let base = (ratio * 100) + (scoreDiff * 0.8);
  return Math.min(98, Math.max(5, Math.round(base)));
}

function chanceBadge(chance: number) {
  if (chance >= 85) return { label: 'Sangat Aman', color: 'bg-emerald-100 text-emerald-700' };
  if (chance >= 70) return { label: 'Peluang Tinggi', color: 'bg-blue-100 text-blue-700' };
  if (chance >= 50) return { label: 'Peluang Sedang', color: 'bg-amber-100 text-amber-700' };
  return { label: 'Perlu Peningkatan', color: 'bg-red-100 text-red-700' };
}

export default function RasionalisasiSNBT() {
  const [score, setScore] = useState(MY_SCORE.toString());
  const [targets, setTargets] = useState<PTNTarget[]>(initialTargets);
  const [modalOpen, setModalOpen] = useState(false);
  const [editTarget, setEditTarget] = useState<PTNTarget | null>(null);
  const [form, setForm] = useState(emptyTarget);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [uniSearch, setUniSearch] = useState('');

  const myScore = parseInt(score) || 0;

  const openCreate = () => {
    setEditTarget(null);
    setForm(emptyTarget);
    setUniSearch('');
    setModalOpen(true);
  };

  const openEdit = (t: PTNTarget) => {
    setEditTarget(t);
    setForm({ university: t.university, abbr: t.abbr, major: t.major, passingGrade: t.passingGrade, applicants: t.applicants, capacity: t.capacity, priority: t.priority, note: t.note });
    setModalOpen(true);
  };

  const handleSave = () => {
    if (!form.university.trim()) { toast.error('Nama universitas wajib diisi'); return; }
    if (!form.major.trim()) { toast.error('Program studi wajib diisi'); return; }
    if (editTarget) {
      setTargets(prev => prev.map(t => t.id === editTarget.id ? { ...t, ...form } : t));
      toast.success('Target PTN diperbarui');
    } else {
      setTargets(prev => [...prev, { ...form, id: `t${Date.now()}` }]);
      toast.success('Target PTN ditambahkan');
    }
    setModalOpen(false);
  };

  const handleDelete = (id: string) => {
    setTargets(prev => prev.filter(t => t.id !== id));
    setDeleteId(null);
    toast.success('Target PTN dihapus');
  };

  const setF = <K extends keyof typeof form>(k: K, v: (typeof form)[K]) =>
    setForm(f => ({ ...f, [k]: v }));

  const selectUni = (u: typeof popularUniversities[0]) => {
    setF('university', u.name);
    setF('abbr', u.abbr);
    setUniSearch('');
  };

  const filteredUnis = popularUniversities.filter(u =>
    u.name.toLowerCase().includes(uniSearch.toLowerCase()) || u.abbr.toLowerCase().includes(uniSearch.toLowerCase())
  );

  const sorted = [...targets].sort((a, b) => {
    const order = { utama: 0, cadangan: 1, aman: 2 };
    return order[a.priority] - order[b.priority];
  });

  const bestChance = targets.length > 0
    ? targets.reduce((best, t) => {
        const c = calcChance(myScore, t.passingGrade, t.applicants, t.capacity);
        const bc = calcChance(myScore, best.passingGrade, best.applicants, best.capacity);
        return c > bc ? t : best;
      }, targets[0])
    : null;

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card className="p-6 bg-gradient-to-br from-blue-600 via-indigo-600 to-purple-600 text-white relative overflow-hidden">
        <div className="absolute inset-0 opacity-10" style={{ backgroundImage: 'linear-gradient(rgba(255,255,255,.4) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,.4) 1px, transparent 1px)', backgroundSize: '40px 40px' }} />
        <div className="relative flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
          <div>
            <div className="flex items-center gap-2 mb-1"><Target className="w-5 h-5 text-cyan-300" /><span className="font-bold">Rasionalisasi SNBT</span></div>
            <h2 className="text-2xl font-black mb-1">Peluang Masuk PTN</h2>
            <p className="text-blue-100 text-sm">Analisis berbasis data historis SNBT, passing grade, dan daya tampung per prodi.</p>
          </div>
          <div className="flex items-center gap-3">
            <div className="bg-white/15 backdrop-blur-sm px-4 py-2.5 rounded-xl text-center">
              <p className="text-2xl font-black">{myScore}</p>
              <p className="text-xs text-blue-200">Skor Kamu</p>
            </div>
            <div className="bg-white/15 backdrop-blur-sm px-4 py-2.5 rounded-xl text-center">
              <p className="text-2xl font-black">{targets.length}</p>
              <p className="text-xs text-blue-200">PTN Target</p>
            </div>
            {bestChance && (
              <div className="bg-white/15 backdrop-blur-sm px-4 py-2.5 rounded-xl text-center">
                <p className="text-2xl font-black">{calcChance(myScore, bestChance.passingGrade, bestChance.applicants, bestChance.capacity)}%</p>
                <p className="text-xs text-blue-200">Peluang Terbaik</p>
              </div>
            )}
          </div>
        </div>
      </Card>

      {/* Score input + add target */}
      <div className="flex flex-col sm:flex-row gap-4">
        <Card className="p-5 flex-1">
          <p className="text-sm font-semibold text-slate-700 mb-2">Update Skor Tryoutmu</p>
          <div className="flex gap-3">
            <div className="flex-1">
              <Input
                type="number"
                placeholder="Masukkan skor (contoh: 685)"
                value={score}
                onChange={e => setScore(e.target.value)}
                min={0} max={1000}
              />
              <p className="text-xs text-muted-foreground mt-1">Skor dari tryout terakhir kamu (skala 0–1000)</p>
            </div>
            <Button onClick={() => toast.success('Skor diperbarui! Peluang dihitung ulang.')} className="bg-indigo-600 hover:bg-indigo-700 shrink-0">
              Update
            </Button>
          </div>
        </Card>
        <div className="flex items-center sm:items-stretch">
          <Button onClick={openCreate} className="w-full sm:w-auto bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 gap-2 shadow-lg px-6">
            <Plus className="w-4 h-4" /> Tambah Target PTN
          </Button>
        </div>
      </div>

      {/* Summary strip */}
      {targets.length > 0 && (
        <div className="grid grid-cols-3 gap-4">
          {[
            { label: 'Target Utama', val: targets.filter(t => t.priority === 'utama').length, color: 'text-indigo-600', bg: 'bg-indigo-50 border-indigo-200' },
            { label: 'Cadangan', val: targets.filter(t => t.priority === 'cadangan').length, color: 'text-amber-600', bg: 'bg-amber-50 border-amber-200' },
            { label: 'Pilihan Aman', val: targets.filter(t => t.priority === 'aman').length, color: 'text-emerald-600', bg: 'bg-emerald-50 border-emerald-200' },
          ].map(({ label, val, color, bg }) => (
            <Card key={label} className={`p-4 text-center border ${bg}`}>
              <p className={`text-2xl font-black ${color}`}>{val}</p>
              <p className="text-xs text-muted-foreground mt-0.5">{label}</p>
            </Card>
          ))}
        </div>
      )}

      {/* Target list */}
      {targets.length === 0 ? (
        <Card className="p-10 text-center">
          <Target className="w-12 h-12 mx-auto mb-3 text-slate-300" />
          <h3 className="font-bold text-slate-600 mb-1">Belum ada target PTN</h3>
          <p className="text-sm text-muted-foreground mb-4">Tambahkan PTN dan prodi yang ingin kamu masuki untuk melihat analisis peluang.</p>
          <Button onClick={openCreate} className="bg-indigo-600 hover:bg-indigo-700 gap-2">
            <Plus className="w-4 h-4" /> Tambah PTN Pertama
          </Button>
        </Card>
      ) : (
        <div className="space-y-4">
          <h3 className="font-bold text-slate-800 flex items-center gap-2"><BarChart3 className="w-4 h-4 text-indigo-600" /> Analisis Peluang per PTN</h3>
          {sorted.map(target => {
            const chance = calcChance(myScore, target.passingGrade, target.applicants, target.capacity);
            const badge = chanceBadge(chance);
            const pc = priorityConfig[target.priority];
            const isExpanded = expandedId === target.id;
            const scoreDiff = myScore - target.passingGrade;
            const ratio = ((target.capacity / target.applicants) * 100).toFixed(1);
            return (
              <Card key={target.id} className={`overflow-hidden border-l-4 ${pc.border} hover:shadow-md transition-shadow`}>
                <div className="p-5">
                  <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
                    {/* Left: PTN info */}
                    <div className="flex items-start gap-4 flex-1">
                      <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white text-sm font-black shrink-0">
                        {target.abbr}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 flex-wrap mb-1">
                          <h4 className="font-bold text-slate-900">{target.university}</h4>
                          <Badge variant="outline" className={pc.color}>{pc.label}</Badge>
                        </div>
                        <p className="text-sm text-indigo-600 font-semibold mb-1">{target.major}</p>
                        <div className="flex items-center gap-4 text-xs text-muted-foreground">
                          <span className="flex items-center gap-1"><Users className="w-3 h-3" />{target.applicants.toLocaleString('id-ID')} peminat</span>
                          <span className="flex items-center gap-1"><BookOpen className="w-3 h-3" />{target.capacity} kursi</span>
                          <span className="flex items-center gap-1"><Target className="w-3 h-3" />PG: {target.passingGrade}</span>
                        </div>
                      </div>
                    </div>

                    {/* Right: chance meter */}
                    <div className="flex items-center gap-6 lg:shrink-0">
                      <div className="text-center">
                        <div className={`text-4xl font-black mb-0.5 ${chance >= 70 ? 'text-emerald-600' : chance >= 50 ? 'text-amber-600' : 'text-red-500'}`}>{chance}%</div>
                        <Badge className={badge.color}>{badge.label}</Badge>
                        <div className="mt-2 w-32 h-2 bg-slate-100 rounded-full overflow-hidden">
                          <div
                            className={`h-full rounded-full ${chance >= 70 ? 'bg-gradient-to-r from-emerald-500 to-green-400' : chance >= 50 ? 'bg-gradient-to-r from-amber-500 to-yellow-400' : 'bg-gradient-to-r from-red-500 to-rose-400'}`}
                            style={{ width: `${chance}%` }}
                          />
                        </div>
                      </div>
                      <div className="flex flex-col gap-1.5">
                        <button onClick={() => setExpandedId(isExpanded ? null : target.id)} className="p-2 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-slate-700 transition-colors">
                          {isExpanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                        </button>
                        <button onClick={() => openEdit(target)} className="p-2 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-slate-700 transition-colors">
                          <Edit className="w-4 h-4" />
                        </button>
                        <button onClick={() => setDeleteId(target.id)} className="p-2 rounded-lg hover:bg-red-50 text-slate-400 hover:text-red-500 transition-colors">
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  </div>

                  {/* Expanded detail */}
                  {isExpanded && (
                    <div className="mt-4 pt-4 border-t space-y-4">
                      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                        {[
                          { label: 'Passing Grade', val: target.passingGrade.toString(), sub: 'referensi tahun lalu' },
                          { label: 'Skor Kamu', val: myScore.toString(), sub: `selisih ${scoreDiff >= 0 ? '+' : ''}${scoreDiff}`, color: scoreDiff >= 0 ? 'text-emerald-600' : 'text-red-500' },
                          { label: 'Rasio Kursi', val: `${ratio}%`, sub: `${target.capacity} kursi / ${target.applicants.toLocaleString()} peminat` },
                          { label: 'Kompetisi', val: `${Math.round(target.applicants / target.capacity)}:1`, sub: 'peminat per kursi' },
                        ].map(({ label, val, sub, color }) => (
                          <div key={label} className="bg-slate-50 rounded-xl p-3">
                            <p className="text-xs text-muted-foreground mb-1">{label}</p>
                            <p className={`text-lg font-black ${color || 'text-slate-900'}`}>{val}</p>
                            <p className="text-[10px] text-muted-foreground mt-0.5">{sub}</p>
                          </div>
                        ))}
                      </div>

                      {target.note && (
                        <div className="flex items-start gap-2 p-3 bg-blue-50 border border-blue-200 rounded-xl">
                          <Info className="w-4 h-4 text-blue-500 shrink-0 mt-0.5" />
                          <p className="text-sm text-blue-800">{target.note}</p>
                        </div>
                      )}

                      {/* Score gap insight */}
                      <div className={`flex items-start gap-2 p-3 rounded-xl border ${scoreDiff >= 0 ? 'bg-emerald-50 border-emerald-200' : 'bg-red-50 border-red-200'}`}>
                        {scoreDiff >= 0
                          ? <CheckCircle2 className="w-4 h-4 text-emerald-600 shrink-0 mt-0.5" />
                          : <AlertCircle className="w-4 h-4 text-red-500 shrink-0 mt-0.5" />
                        }
                        <p className={`text-sm ${scoreDiff >= 0 ? 'text-emerald-800' : 'text-red-700'}`}>
                          {scoreDiff >= 0
                            ? `Skor kamu sudah ${scoreDiff} poin di atas passing grade. Pertahankan dan tingkatkan konsistensi latihan.`
                            : `Skor kamu masih ${Math.abs(scoreDiff)} poin di bawah passing grade. Fokus drilling untuk meningkatkan skor.`
                          }
                        </p>
                      </div>
                    </div>
                  )}
                </div>
              </Card>
            );
          })}
        </div>
      )}

      {/* Strategy tips */}
      {targets.length > 0 && (
        <Card className="p-6 bg-gradient-to-br from-indigo-50 to-purple-50 border-indigo-200">
          <div className="flex items-start gap-3 mb-4">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-600 to-purple-600 flex items-center justify-center shrink-0">
              <Award className="w-5 h-5 text-white" />
            </div>
            <div>
              <h3 className="font-bold text-slate-900">Tips Strategi Pilihan PTN</h3>
              <p className="text-xs text-muted-foreground">Berdasarkan data historis SNBT dan konfigurasi targetmu</p>
            </div>
          </div>
          <ul className="space-y-2.5">
            {[
              'Gunakan pola 1-1-1: 1 target ambisius, 1 realistis, 1 aman untuk meminimalkan risiko tidak lolos.',
              'Perhatikan daya tampung dan tren peminat — prodi dengan kursi banyak dan peminat stabil lebih terprediksi.',
              'Skor SNBT tidak berubah setelah ujian. Optimalkan semua subtes sebelum hari-H.',
              'Data passing grade bersifat referensi dari tahun sebelumnya — bisa naik atau turun tergantung kompetisi tahun ini.',
            ].map((tip, i) => (
              <li key={i} className="flex items-start gap-2.5 text-sm text-slate-700">
                <span className="w-5 h-5 rounded-full bg-indigo-100 text-indigo-700 text-xs font-black flex items-center justify-center shrink-0 mt-0.5">{i + 1}</span>
                {tip}
              </li>
            ))}
          </ul>
        </Card>
      )}

      {/* ─── Create/Edit Modal ─── */}
      {modalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4 overflow-y-auto">
          <div className="bg-white rounded-2xl w-full max-w-lg my-4 shadow-2xl">
            <div className="flex items-center justify-between p-6 border-b">
              <h3 className="text-lg font-bold">{editTarget ? 'Edit Target PTN' : 'Tambah Target PTN'}</h3>
              <button onClick={() => setModalOpen(false)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-6 space-y-5 overflow-y-auto max-h-[65vh]">
              {/* University search */}
              <div>
                <Label>Universitas *</Label>
                <div className="relative mt-1.5">
                  <Input
                    value={form.university || uniSearch}
                    onChange={e => { setUniSearch(e.target.value); setF('university', e.target.value); setF('abbr', ''); }}
                    placeholder="Cari atau ketik nama universitas..."
                  />
                  {uniSearch && filteredUnis.length > 0 && (
                    <div className="absolute z-10 w-full mt-1 bg-white border rounded-xl shadow-lg max-h-40 overflow-y-auto">
                      {filteredUnis.map(u => (
                        <button key={u.abbr} onClick={() => selectUni(u)} className="w-full text-left px-4 py-2.5 text-sm hover:bg-indigo-50 transition-colors flex items-center gap-2">
                          <span className="font-bold text-indigo-600 w-12 shrink-0">{u.abbr}</span>
                          <span>{u.name}</span>
                        </button>
                      ))}
                    </div>
                  )}
                </div>
                {form.abbr && <p className="text-xs text-indigo-600 mt-1 font-semibold">{form.abbr} — {form.university}</p>}
              </div>

              <div>
                <Label>Program Studi *</Label>
                <Input className="mt-1.5" value={form.major} onChange={e => setF('major', e.target.value)} placeholder="Teknik Informatika" />
              </div>

              <div className="grid grid-cols-3 gap-3">
                <div>
                  <Label>Passing Grade</Label>
                  <Input type="number" className="mt-1.5" value={form.passingGrade} onChange={e => setF('passingGrade', Number(e.target.value))} />
                </div>
                <div>
                  <Label>Peminat</Label>
                  <Input type="number" className="mt-1.5" value={form.applicants} onChange={e => setF('applicants', Number(e.target.value))} />
                </div>
                <div>
                  <Label>Daya Tampung</Label>
                  <Input type="number" className="mt-1.5" value={form.capacity} onChange={e => setF('capacity', Number(e.target.value))} />
                </div>
              </div>

              <div>
                <Label>Prioritas</Label>
                <div className="grid grid-cols-3 gap-2 mt-1.5">
                  {(['utama', 'cadangan', 'aman'] as const).map(p => (
                    <button
                      key={p}
                      onClick={() => setF('priority', p)}
                      className={`py-2.5 rounded-xl text-sm font-bold transition-colors border ${form.priority === p ? `${priorityConfig[p].color}` : 'border-slate-200 text-slate-500 hover:bg-slate-50'}`}
                    >
                      {priorityConfig[p].label}
                    </button>
                  ))}
                </div>
              </div>

              <div>
                <Label>Catatan Pribadi</Label>
                <textarea
                  className="w-full mt-1.5 px-3 py-2.5 border rounded-xl text-sm resize-none h-20 focus:outline-none focus:ring-2 focus:ring-indigo-300"
                  value={form.note}
                  onChange={e => setF('note', e.target.value)}
                  placeholder="Alasan memilih, info tambahan, dll."
                />
              </div>

              {/* Preview */}
              {myScore > 0 && form.passingGrade > 0 && (
                <div className="p-4 bg-indigo-50 border border-indigo-200 rounded-xl">
                  <p className="text-xs font-bold text-indigo-700 mb-1">Preview Peluang</p>
                  <div className="flex items-center gap-3">
                    <p className="text-3xl font-black text-indigo-600">
                      {calcChance(myScore, form.passingGrade, form.applicants, form.capacity)}%
                    </p>
                    <div className="flex-1 h-2 bg-indigo-100 rounded-full overflow-hidden">
                      <div className="h-full bg-indigo-500 rounded-full transition-all" style={{ width: `${calcChance(myScore, form.passingGrade, form.applicants, form.capacity)}%` }} />
                    </div>
                  </div>
                </div>
              )}
            </div>
            <div className="flex gap-3 p-6 border-t">
              <Button variant="outline" className="flex-1" onClick={() => setModalOpen(false)}>Batal</Button>
              <Button className="flex-1 bg-gradient-to-r from-indigo-600 to-purple-600 gap-2" onClick={handleSave}>
                <Save className="w-4 h-4" /> {editTarget ? 'Simpan' : 'Tambah Target'}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Delete confirm */}
      {deleteId && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-2xl p-6 w-full max-w-sm shadow-2xl">
            <h3 className="font-bold text-slate-900 mb-2">Hapus Target PTN?</h3>
            <p className="text-sm text-muted-foreground mb-6">Target PTN ini akan dihapus dari daftar rasionalisasimu.</p>
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
