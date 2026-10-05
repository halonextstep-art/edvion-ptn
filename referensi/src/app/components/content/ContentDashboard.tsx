import { useState, useMemo } from 'react';
import {
  BookOpen, Package, BarChart3, LogOut, Plus, Search,
  Edit3, Trash2, Eye, Clock, CheckCircle, AlertCircle, Tag, X, Save,
  Layers, Target, Award, TrendingUp,
  ChevronRight, Copy, Filter, ArrowRight, FileText,
  CheckCircle2, XCircle, RefreshCw, Star, Hash,
  BarChart2, Users, Flame, Send, Link2,
  GripVertical, PenLine, EyeOff, PlusCircle, RotateCcw
} from 'lucide-react';
import { toast } from 'sonner';
import NotificationBell from '../shared/NotificationBell';
import {
  BarChart, Bar, LineChart, Line, PieChart, Pie, Cell,
  XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer
} from 'recharts';

// ─── Types ────────────────────────────────────────────────────────────────────
type PackageType = 'full-tryout' | 'mini-tryout' | 'drilling' | 'chapter-test';
type PackageStatus = 'draft' | 'review' | 'published' | 'archived';
type QuestionStatus = 'draft' | 'review' | 'approved' | 'published' | 'rejected';
type QuestionType = 'pg' | 'pgk' | 'bs' | 'menjodohkan' | 'isian' | 'uraian';
type TestCategory = 'SNBT' | 'TKA-IPA' | 'TKA-IPS' | 'AKM';
type Difficulty = 'Mudah' | 'Sedang' | 'Sulit';
type ActiveView = 'questions' | 'packages' | 'analytics';

interface PGOption { key: string; text: string }
interface BSStatement { id: string; text: string; answer: boolean }
interface MatchPair { id: string; left: string; right: string }

interface Question {
  id: string;
  code: string;
  category: TestCategory;
  subject: string;
  topic: string;
  questionType: QuestionType;
  stimulus: string;
  text: string;
  // PG / PGK options
  options: PGOption[];
  correctKeys: string[];       // PG: 1 key; PGK: multiple keys
  // Benar/Salah
  bsStatements: BSStatement[];
  // Menjodohkan
  matchPairs: MatchPair[];
  // Isian / Uraian
  keyAnswer: string;
  rubric: string;
  explanation: string;
  difficulty: Difficulty;
  status: QuestionStatus;
  tags: string[];
  createdBy: string;
  createdAt: string;
  updatedAt: string;
  reviewNote?: string;
  usedInPackages: string[];
  attemptCount: number;
  correctRate: number;
}

interface TryoutPackage {
  id: string; title: string; subtitle: string;
  type: PackageType; category: TestCategory;
  subjectFilter: string[];
  questionIds: string[];
  duration: number; difficulty: Difficulty | 'Campuran';
  status: PackageStatus; targetClass: string;
  tags: string[]; isPremium: boolean;
  createdBy: string; createdAt: string; updatedAt: string;
  publishedAt?: string; totalAttempts: number; avgScore: number;
}

// ─── Config ───────────────────────────────────────────────────────────────────
const CATEGORY_SUBJECTS: Record<TestCategory, string[]> = {
  'SNBT':    ['PU', 'PPU', 'PBM', 'PM'],
  'TKA-IPA': ['Matematika', 'Fisika', 'Kimia', 'Biologi'],
  'TKA-IPS': ['Matematika', 'Ekonomi', 'Geografi', 'Sosiologi', 'Sejarah'],
  'AKM':     ['Literasi', 'Numerasi'],
};

const ALL_QUESTION_TYPES: QuestionType[] = ['pg', 'pgk', 'bs', 'menjodohkan', 'isian', 'uraian'];

const QTYPE_LABELS: Record<QuestionType, string> = {
  pg: 'Pilihan Ganda', pgk: 'PG Kompleks', bs: 'Benar/Salah',
  menjodohkan: 'Menjodohkan', isian: 'Isian Singkat', uraian: 'Uraian/Essay',
};

const QTYPE_DESC: Record<QuestionType, string> = {
  pg:          '1 jawaban benar dari 5 opsi',
  pgk:         'Lebih dari 1 opsi bisa benar',
  bs:          'Tiap pernyataan dijudge Benar/Salah',
  menjodohkan: 'Pasangkan kolom kiri dan kanan',
  isian:       'Jawaban singkat/kata kunci',
  uraian:      'Jawaban panjang + rubrik penilaian',
};

const CATEGORY_COLORS: Record<TestCategory, string> = {
  'SNBT':    'text-violet-400 bg-violet-900/30 border-violet-700/40',
  'TKA-IPA': 'text-blue-400 bg-blue-900/30 border-blue-700/40',
  'TKA-IPS': 'text-emerald-400 bg-emerald-900/30 border-emerald-700/40',
  'AKM':     'text-orange-400 bg-orange-900/30 border-orange-700/40',
};

const qStatusConfig: Record<QuestionStatus, { label: string; color: string; dot: string }> = {
  draft:     { label: 'Draft',     color: 'text-slate-400 bg-slate-800 border-slate-700',            dot: 'bg-slate-500' },
  review:    { label: 'Review',    color: 'text-amber-300 bg-amber-900/40 border-amber-700/50',      dot: 'bg-amber-400' },
  approved:  { label: 'Approved',  color: 'text-blue-300 bg-blue-900/40 border-blue-700/50',         dot: 'bg-blue-400' },
  published: { label: 'Published', color: 'text-emerald-300 bg-emerald-900/40 border-emerald-700/50',dot: 'bg-emerald-400' },
  rejected:  { label: 'Ditolak',   color: 'text-red-400 bg-red-900/40 border-red-700/50',            dot: 'bg-red-500' },
};

const difficultyColors: Record<Difficulty, string> = {
  Mudah: 'text-green-400', Sedang: 'text-amber-400', Sulit: 'text-red-400',
};

const WORKFLOW: { from: QuestionStatus; to: QuestionStatus; label: string; btn: string }[] = [
  { from: 'draft',     to: 'review',    label: 'Kirim Review', btn: 'bg-amber-600/20 border-amber-600/30 text-amber-300 hover:bg-amber-600/40' },
  { from: 'review',    to: 'approved',  label: 'Approve',      btn: 'bg-blue-600/20 border-blue-600/30 text-blue-300 hover:bg-blue-600/40' },
  { from: 'review',    to: 'rejected',  label: 'Tolak',        btn: 'bg-red-600/20 border-red-600/30 text-red-400 hover:bg-red-600/40' },
  { from: 'approved',  to: 'published', label: 'Publish',      btn: 'bg-emerald-600/20 border-emerald-600/30 text-emerald-300 hover:bg-emerald-600/40' },
  { from: 'rejected',  to: 'draft',     label: 'Revisi',       btn: 'bg-slate-700 border-slate-600 text-slate-300 hover:bg-slate-600' },
  { from: 'published', to: 'archived',  label: 'Arsipkan',     btn: 'bg-red-900/20 border-red-700/30 text-red-400 hover:bg-red-900/40' },
];

// ─── Helpers ──────────────────────────────────────────────────────────────────
const makeId = () => `q${Date.now()}${Math.random().toString(36).slice(2, 6)}`;
const today = () => new Date().toISOString().split('T')[0];

function makeBlankQuestion(category: TestCategory = 'SNBT', author: string = ''): Omit<Question, 'id' | 'code' | 'createdAt' | 'updatedAt' | 'usedInPackages' | 'attemptCount' | 'correctRate'> {
  return {
    category, subject: CATEGORY_SUBJECTS[category][0],
    topic: '', questionType: 'pg',
    stimulus: '', text: '',
    options: [{ key: 'A', text: '' }, { key: 'B', text: '' }, { key: 'C', text: '' }, { key: 'D', text: '' }, { key: 'E', text: '' }],
    correctKeys: ['A'],
    bsStatements: [{ id: '1', text: '', answer: true }, { id: '2', text: '', answer: false }],
    matchPairs: [{ id: '1', left: '', right: '' }, { id: '2', left: '', right: '' }],
    keyAnswer: '', rubric: '', explanation: '',
    difficulty: 'Sedang', status: 'draft', tags: [], createdBy: author,
  };
}

// ─── Mock Data ────────────────────────────────────────────────────────────────
const MOCK_QUESTIONS: Question[] = [
  { id: 'q001', code: 'SNBT-PU-001', category: 'SNBT', subject: 'PU', topic: 'Analogi', questionType: 'pg', stimulus: '', text: 'Buku : Perpustakaan = Lukisan : ...', options: [{ key: 'A', text: 'Kanvas' }, { key: 'B', text: 'Museum' }, { key: 'C', text: 'Seniman' }, { key: 'D', text: 'Pameran' }, { key: 'E', text: 'Galeri' }], correctKeys: ['B'], bsStatements: [], matchPairs: [], keyAnswer: '', rubric: '', explanation: 'Buku disimpan di perpustakaan, lukisan disimpan di museum.', difficulty: 'Mudah', status: 'published', tags: ['analogi'], createdBy: 'Budi S.', createdAt: '2025-01-05', updatedAt: '2025-01-10', usedInPackages: ['pkg-1', 'pkg-2'], attemptCount: 8420, correctRate: 78 },
  { id: 'q002', code: 'SNBT-PM-001', category: 'SNBT', subject: 'PM', topic: 'Aljabar', questionType: 'pg', stimulus: '', text: 'Jika 2x + 5 = 17, maka nilai x adalah...', options: [{ key: 'A', text: '5' }, { key: 'B', text: '6' }, { key: 'C', text: '7' }, { key: 'D', text: '8' }, { key: 'E', text: '9' }], correctKeys: ['B'], bsStatements: [], matchPairs: [], keyAnswer: '', rubric: '', explanation: '2x = 17 − 5 = 12, jadi x = 6.', difficulty: 'Mudah', status: 'published', tags: ['aljabar'], createdBy: 'Tim Konten', createdAt: '2025-01-07', updatedAt: '2025-01-07', usedInPackages: ['pkg-1'], attemptCount: 9210, correctRate: 71 },
  { id: 'q003', code: 'TKA-IPA-BIO-001', category: 'TKA-IPA', subject: 'Biologi', topic: 'Sel', questionType: 'pgk', stimulus: '', text: 'Manakah pernyataan yang BENAR tentang mitokondria?', options: [{ key: 'A', text: 'Tempat respirasi aerob' }, { key: 'B', text: 'Mengandung DNA sendiri' }, { key: 'C', text: 'Ditemukan pada sel prokariotik' }, { key: 'D', text: 'Menghasilkan ATP' }, { key: 'E', text: 'Tidak punya membran' }], correctKeys: ['A', 'B', 'D'], bsStatements: [], matchPairs: [], keyAnswer: '', rubric: '', explanation: 'Mitokondria: tempat respirasi aerob, memiliki DNA sendiri, menghasilkan ATP.', difficulty: 'Sedang', status: 'approved', tags: ['sel', 'mitokondria'], createdBy: 'Sari D.', createdAt: '2025-01-10', updatedAt: '2025-01-20', usedInPackages: [], attemptCount: 0, correctRate: 0 },
  { id: 'q004', code: 'AKM-LIT-001', category: 'AKM', subject: 'Literasi', topic: 'Pemahaman Teks', questionType: 'bs', stimulus: 'Perubahan iklim adalah perubahan jangka panjang dalam pola cuaca dan suhu rata-rata global. Sejak abad ke-19, aktivitas manusia menjadi pendorong utama perubahan iklim, terutama karena pembakaran bahan bakar fosil.', text: 'Berdasarkan teks, tentukan Benar atau Salah tiap pernyataan berikut:', options: [], correctKeys: [], bsStatements: [{ id: '1', text: 'Perubahan iklim hanya terjadi sejak abad ke-19', answer: false }, { id: '2', text: 'Aktivitas manusia berkontribusi pada perubahan iklim', answer: true }, { id: '3', text: 'Pembakaran bahan bakar fosil merupakan faktor utama', answer: true }], matchPairs: [], keyAnswer: '', rubric: '', explanation: '', difficulty: 'Sedang', status: 'review', tags: ['literasi', 'lingkungan'], createdBy: 'Rina M.', createdAt: '2025-01-15', updatedAt: '2025-01-20', usedInPackages: [], attemptCount: 0, correctRate: 0, reviewNote: 'Cek akurasi pernyataan ke-1' },
  { id: 'q005', code: 'AKM-NUM-001', category: 'AKM', subject: 'Numerasi', topic: 'Statistika', questionType: 'isian', stimulus: '', text: 'Rata-rata nilai ulangan 5 siswa adalah 78. Jika nilai siswa keenam ditambahkan, rata-rata menjadi 80. Berapakah nilai siswa keenam?', options: [], correctKeys: [], bsStatements: [], matchPairs: [], keyAnswer: '90', rubric: '', explanation: 'Total 5 siswa = 5×78 = 390. Total 6 siswa = 6×80 = 480. Nilai ke-6 = 480 − 390 = 90.', difficulty: 'Sedang', status: 'draft', tags: ['numerasi', 'statistika'], createdBy: 'Tim Konten', createdAt: '2025-01-22', updatedAt: '2025-01-22', usedInPackages: [], attemptCount: 0, correctRate: 0 },
  { id: 'q006', code: 'TKA-IPS-EKO-001', category: 'TKA-IPS', subject: 'Ekonomi', topic: 'Permintaan', questionType: 'menjodohkan', stimulus: '', text: 'Pasangkan jenis elastisitas dengan definisinya:', options: [], correctKeys: [], bsStatements: [], matchPairs: [{ id: '1', left: 'Elastis', right: 'Ed > 1' }, { id: '2', left: 'Inelastis', right: 'Ed < 1' }, { id: '3', left: 'Elastis sempurna', right: 'Ed = ∞' }], keyAnswer: '', rubric: '', explanation: '', difficulty: 'Mudah', status: 'published', tags: ['elastisitas'], createdBy: 'Ahmad F.', createdAt: '2025-01-08', updatedAt: '2025-01-08', usedInPackages: ['pkg-1'], attemptCount: 3200, correctRate: 85 },
];

const MOCK_PACKAGES: TryoutPackage[] = [
  { id: 'pkg-1', title: 'Tryout SNBT Full Simulasi #1', subtitle: 'Simulasi lengkap SNBT 2025', type: 'full-tryout', category: 'SNBT', subjectFilter: ['PU', 'PPU', 'PBM', 'PM'], questionIds: ['q001', 'q002', 'q006'], duration: 195, difficulty: 'Campuran', status: 'published', targetClass: 'Kelas 12', tags: ['SNBT', 'Full'], isPremium: true, createdBy: 'Tim Konten', createdAt: '2025-01-10', updatedAt: '2025-01-15', publishedAt: '2025-01-15', totalAttempts: 12450, avgScore: 68.4 },
  { id: 'pkg-2', title: 'Mini Tryout AKM Literasi', subtitle: 'Fokus AKM Literasi untuk SMA/SMK', type: 'mini-tryout', category: 'AKM', subjectFilter: ['Literasi'], questionIds: ['q004'], duration: 60, difficulty: 'Sedang', status: 'review', targetClass: 'Kelas 11-12', tags: ['AKM'], isPremium: false, createdBy: 'Rina M.', createdAt: '2025-01-16', updatedAt: '2025-01-20', totalAttempts: 0, avgScore: 0 },
  { id: 'pkg-3', title: 'TKA IPA — Biologi Sel', subtitle: 'Chapter test biologi tingkat lanjut', type: 'chapter-test', category: 'TKA-IPA', subjectFilter: ['Biologi'], questionIds: ['q003'], duration: 45, difficulty: 'Sedang', status: 'draft', targetClass: 'Kelas 12', tags: ['TKA', 'Biologi'], isPremium: true, createdBy: 'Sari D.', createdAt: '2025-01-18', updatedAt: '2025-01-20', totalAttempts: 0, avgScore: 0 },
];

// ─── Analytics mock ───────────────────────────────────────────────────────────
const attemptTrend = [{ w: 'W1', v: 4200 }, { w: 'W2', v: 6800 }, { w: 'W3', v: 9100 }, { w: 'W4', v: 12450 }];
const categoryDistrib = [
  { name: 'SNBT', count: 2, fill: '#8b5cf6' }, { name: 'TKA-IPA', count: 1, fill: '#3b82f6' },
  { name: 'TKA-IPS', count: 1, fill: '#10b981' }, { name: 'AKM', count: 2, fill: '#f97316' },
];
const typeDistrib = [
  { name: 'PG', count: 3, fill: '#60a5fa' }, { name: 'PGK', count: 1, fill: '#a78bfa' },
  { name: 'B/S', count: 1, fill: '#34d399' }, { name: 'Jodoh', count: 1, fill: '#fb923c' },
  { name: 'Isian', count: 1, fill: '#f472b6' },
];

// ─── Side-panel Question Form ─────────────────────────────────────────────────
interface QuestionFormState extends Omit<Question, 'id' | 'code' | 'createdAt' | 'updatedAt' | 'usedInPackages' | 'attemptCount' | 'correctRate'> {}

function QuestionForm({
  form, setForm, editId, onSave, onSaveAndNew, onCancel
}: {
  form: QuestionFormState;
  setForm: React.Dispatch<React.SetStateAction<QuestionFormState>>;
  editId: string | null;
  onSave: () => void;
  onSaveAndNew: () => void;
  onCancel: () => void;
}) {
  const [tagInput, setTagInput] = useState('');
  const allowedTypes = ALL_QUESTION_TYPES;

  const setField = <K extends keyof QuestionFormState>(k: K, v: QuestionFormState[K]) =>
    setForm(f => ({ ...f, [k]: v }));

  const addTag = () => {
    const t = tagInput.trim();
    if (t && !form.tags.includes(t)) setField('tags', [...form.tags, t]);
    setTagInput('');
  };

  const togglePGK = (key: string) => {
    const has = form.correctKeys.includes(key);
    setField('correctKeys', has ? form.correctKeys.filter(k => k !== key) : [...form.correctKeys, key]);
  };

  const addBSStatement = () =>
    setField('bsStatements', [...form.bsStatements, { id: makeId(), text: '', answer: true }]);

  const addMatchPair = () =>
    setField('matchPairs', [...form.matchPairs, { id: makeId(), left: '', right: '' }]);

  const inp = "w-full px-3 py-2 bg-white/5 border border-white/10 rounded-xl text-sm text-white placeholder-white/25 focus:outline-none focus:border-violet-500/60 transition-colors";

  return (
    <div className="flex flex-col h-full">
      {/* Form header */}
      <div className="px-5 py-4 border-b border-white/10 shrink-0">
        <div className="flex items-center justify-between">
          <h3 className="font-black text-white text-base">{editId ? 'Edit Soal' : 'Buat Soal Baru'}</h3>
          <button onClick={onCancel} className="p-1.5 rounded-lg hover:bg-white/10 text-white/40 hover:text-white transition-colors"><X className="w-4 h-4" /></button>
        </div>
      </div>

      {/* Scrollable form body */}
      <div className="flex-1 overflow-y-auto px-5 py-4 space-y-4">

        {/* Category + Subject */}
        <div>
          <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Kategori Ujian</label>
          <div className="grid grid-cols-2 gap-1.5 mb-2">
            {(['SNBT', 'TKA-IPA', 'TKA-IPS', 'AKM'] as TestCategory[]).map(cat => (
              <button key={cat} onClick={() => {
                const firstSubj = CATEGORY_SUBJECTS[cat][0];
                const firstType: QuestionType = 'pg';
                setForm(f => ({ ...f, category: cat, subject: firstSubj, questionType: firstType }));
              }} className={`py-2 rounded-xl text-xs font-bold border transition-colors ${form.category === cat ? 'bg-violet-600/40 border-violet-500/60 text-violet-200' : 'bg-white/5 border-white/10 text-white/50 hover:text-white'}`}>
                {cat}
              </button>
            ))}
          </div>
          <div className="flex flex-wrap gap-1.5">
            {CATEGORY_SUBJECTS[form.category].map(s => (
              <button key={s} onClick={() => setField('subject', s)} className={`px-3 py-1 rounded-lg text-xs font-semibold border transition-colors ${form.subject === s ? 'bg-indigo-600/40 border-indigo-500/60 text-indigo-200' : 'bg-white/5 border-white/10 text-white/50 hover:text-white'}`}>{s}</button>
            ))}
          </div>
        </div>

        {/* Question Type */}
        <div>
          <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Tipe Soal</label>
          <div className="space-y-1">
            {allowedTypes.map(qt => (
              <button key={qt} onClick={() => setField('questionType', qt)} className={`w-full flex items-center gap-3 px-3 py-2 rounded-xl border text-left transition-colors ${form.questionType === qt ? 'bg-violet-600/30 border-violet-500/50' : 'bg-white/5 border-white/10 hover:bg-white/[0.07]'}`}>
                <div className={`w-4 h-4 rounded-full border-2 shrink-0 flex items-center justify-center ${form.questionType === qt ? 'border-violet-400' : 'border-white/20'}`}>
                  {form.questionType === qt && <div className="w-2 h-2 rounded-full bg-violet-400" />}
                </div>
                <div>
                  <p className={`text-xs font-semibold ${form.questionType === qt ? 'text-violet-200' : 'text-white/70'}`}>{QTYPE_LABELS[qt]}</p>
                  <p className="text-[10px] text-white/30">{QTYPE_DESC[qt]}</p>
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* Topic + Difficulty */}
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Topik</label>
            <input value={form.topic} onChange={e => setField('topic', e.target.value)} placeholder="mis. Analogi, Aljabar" className={inp} />
          </div>
          <div>
            <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Kesulitan</label>
            <div className="flex gap-1">
              {(['Mudah', 'Sedang', 'Sulit'] as Difficulty[]).map(d => (
                <button key={d} onClick={() => setField('difficulty', d)} className={`flex-1 py-2 rounded-lg text-[10px] font-bold border transition-colors ${form.difficulty === d ? 'bg-violet-600/40 border-violet-500/60 text-violet-200' : 'bg-white/5 border-white/10 text-white/40 hover:text-white'}`}>{d}</button>
              ))}
            </div>
          </div>
        </div>

        {/* Stimulus */}
        <div>
          <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Stimulus / Teks Bacaan <span className="text-white/20">(opsional)</span></label>
          <textarea value={form.stimulus} onChange={e => setField('stimulus', e.target.value)} placeholder="Teks wacana, grafik deskripsi, atau konteks soal..." className={`${inp} resize-none h-16`} />
        </div>

        {/* Question text */}
        <div>
          <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Teks Soal *</label>
          <textarea value={form.text} onChange={e => setField('text', e.target.value)} placeholder="Tulis pertanyaan di sini..." className={`${inp} resize-none h-20`} />
        </div>

        {/* ── PG: 5 options, 1 correct ── */}
        {form.questionType === 'pg' && (
          <div>
            <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-2">Pilihan Jawaban — klik huruf untuk tandai benar *</label>
            <div className="space-y-2">
              {form.options.map((opt, i) => (
                <div key={opt.key} className="flex items-center gap-2">
                  <button onClick={() => setField('correctKeys', [opt.key])} className={`w-7 h-7 rounded-full flex items-center justify-center text-xs font-black shrink-0 border-2 transition-colors ${form.correctKeys[0] === opt.key ? 'bg-emerald-600 border-emerald-500 text-white' : 'bg-white/5 border-white/20 text-white/40 hover:border-white/50'}`}>{opt.key}</button>
                  <input value={opt.text} onChange={e => { const opts = [...form.options]; opts[i] = { ...opts[i], text: e.target.value }; setField('options', opts); }} placeholder={`Pilihan ${opt.key}`} className={`${inp} flex-1`} />
                </div>
              ))}
            </div>
          </div>
        )}

        {/* ── PGK: multiple correct ── */}
        {form.questionType === 'pgk' && (
          <div>
            <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-2">Pilihan Jawaban — centang semua yang benar *</label>
            <div className="space-y-2">
              {form.options.map((opt, i) => {
                const checked = form.correctKeys.includes(opt.key);
                return (
                  <div key={opt.key} className="flex items-center gap-2">
                    <button onClick={() => togglePGK(opt.key)} className={`w-5 h-5 rounded flex items-center justify-center shrink-0 border-2 transition-colors ${checked ? 'bg-emerald-600 border-emerald-500' : 'bg-white/5 border-white/20 hover:border-white/40'}`}>
                      {checked && <CheckCircle2 className="w-3 h-3 text-white" />}
                    </button>
                    <span className={`text-xs font-black w-4 ${checked ? 'text-emerald-400' : 'text-white/30'}`}>{opt.key}</span>
                    <input value={opt.text} onChange={e => { const opts = [...form.options]; opts[i] = { ...opts[i], text: e.target.value }; setField('options', opts); }} placeholder={`Pilihan ${opt.key}`} className={`${inp} flex-1`} />
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* ── Benar/Salah ── */}
        {form.questionType === 'bs' && (
          <div>
            <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-2">Pernyataan — tandai Benar (B) atau Salah (S)</label>
            <div className="space-y-2">
              {form.bsStatements.map((stmt, i) => (
                <div key={stmt.id} className="flex items-center gap-2">
                  <button onClick={() => { const s = [...form.bsStatements]; s[i] = { ...s[i], answer: !s[i].answer }; setField('bsStatements', s); }} className={`px-2.5 py-1.5 rounded-lg text-xs font-black border transition-colors shrink-0 ${stmt.answer ? 'bg-emerald-600/30 border-emerald-500/50 text-emerald-300' : 'bg-red-600/30 border-red-500/50 text-red-300'}`}>
                    {stmt.answer ? 'B' : 'S'}
                  </button>
                  <input value={stmt.text} onChange={e => { const s = [...form.bsStatements]; s[i] = { ...s[i], text: e.target.value }; setField('bsStatements', s); }} placeholder={`Pernyataan ${i + 1}`} className={`${inp} flex-1`} />
                  {form.bsStatements.length > 2 && (
                    <button onClick={() => setField('bsStatements', form.bsStatements.filter(s => s.id !== stmt.id))} className="p-1 text-white/25 hover:text-red-400 transition-colors"><X className="w-3.5 h-3.5" /></button>
                  )}
                </div>
              ))}
            </div>
            <button onClick={addBSStatement} className="mt-2 flex items-center gap-1.5 text-xs text-violet-400 hover:text-violet-300 transition-colors">
              <PlusCircle className="w-3.5 h-3.5" /> Tambah pernyataan
            </button>
          </div>
        )}

        {/* ── Menjodohkan ── */}
        {form.questionType === 'menjodohkan' && (
          <div>
            <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-2">Pasangan — Kolom Kiri ↔ Kolom Kanan</label>
            <div className="space-y-2">
              {form.matchPairs.map((pair, i) => (
                <div key={pair.id} className="flex items-center gap-2">
                  <input value={pair.left} onChange={e => { const p = [...form.matchPairs]; p[i] = { ...p[i], left: e.target.value }; setField('matchPairs', p); }} placeholder="Kiri" className={`${inp} flex-1`} />
                  <span className="text-white/30 text-xs">↔</span>
                  <input value={pair.right} onChange={e => { const p = [...form.matchPairs]; p[i] = { ...p[i], right: e.target.value }; setField('matchPairs', p); }} placeholder="Kanan" className={`${inp} flex-1`} />
                  {form.matchPairs.length > 2 && (
                    <button onClick={() => setField('matchPairs', form.matchPairs.filter(p => p.id !== pair.id))} className="p-1 text-white/25 hover:text-red-400 transition-colors"><X className="w-3.5 h-3.5" /></button>
                  )}
                </div>
              ))}
            </div>
            <button onClick={addMatchPair} className="mt-2 flex items-center gap-1.5 text-xs text-violet-400 hover:text-violet-300 transition-colors">
              <PlusCircle className="w-3.5 h-3.5" /> Tambah pasangan
            </button>
          </div>
        )}

        {/* ── Isian Singkat ── */}
        {form.questionType === 'isian' && (
          <div>
            <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Kunci Jawaban *</label>
            <input value={form.keyAnswer} onChange={e => setField('keyAnswer', e.target.value)} placeholder="Jawaban yang diterima (pisahkan dengan koma jika ada variasi)" className={inp} />
          </div>
        )}

        {/* ── Uraian / Essay ── */}
        {form.questionType === 'uraian' && (
          <div className="space-y-3">
            <div>
              <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Kunci Jawaban / Kata Kunci</label>
              <textarea value={form.keyAnswer} onChange={e => setField('keyAnswer', e.target.value)} placeholder="Poin-poin utama yang harus ada dalam jawaban..." className={`${inp} resize-none h-16`} />
            </div>
            <div>
              <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Rubrik Penilaian</label>
              <textarea value={form.rubric} onChange={e => setField('rubric', e.target.value)} placeholder="Skor 4: ..., Skor 3: ..., Skor 2: ..., Skor 1: ..." className={`${inp} resize-none h-16`} />
            </div>
          </div>
        )}

        {/* Explanation — semua tipe */}
        <div>
          <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Pembahasan / Penjelasan</label>
          <textarea value={form.explanation} onChange={e => setField('explanation', e.target.value)} placeholder="Jelaskan mengapa jawaban tersebut benar..." className={`${inp} resize-none h-16`} />
        </div>

        {/* Status + Tags */}
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Status</label>
            <select value={form.status} onChange={e => setField('status', e.target.value as QuestionStatus)} className={`${inp} pr-8`}>
              {(['draft', 'review', 'approved'] as QuestionStatus[]).map(s => (
                <option key={s} value={s} className="bg-[#0f0d1a]">{qStatusConfig[s].label}</option>
              ))}
            </select>
          </div>
          <div>
            <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Tags</label>
            <div className="flex gap-1.5">
              <input value={tagInput} onChange={e => setTagInput(e.target.value)} onKeyDown={e => e.key === 'Enter' && (e.preventDefault(), addTag())} placeholder="Tag + Enter" className={`${inp} flex-1`} />
              <button onClick={addTag} className="px-3 py-2 rounded-xl bg-violet-600/20 text-violet-300 border border-violet-600/30 text-sm hover:bg-violet-600/30">+</button>
            </div>
          </div>
        </div>
        {form.tags.length > 0 && (
          <div className="flex flex-wrap gap-1.5">
            {form.tags.map(t => (
              <span key={t} className="flex items-center gap-1 text-[10px] px-2 py-0.5 rounded-full bg-white/10 text-white/60">
                #{t}<button onClick={() => setField('tags', form.tags.filter(x => x !== t))} className="text-white/30 hover:text-red-400"><X className="w-3 h-3" /></button>
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Form footer */}
      <div className="px-5 py-4 border-t border-white/10 shrink-0 space-y-2">
        {!editId && (
          <button onClick={onSaveAndNew} className="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl border border-violet-500/40 bg-violet-600/10 hover:bg-violet-600/25 text-violet-300 font-bold text-sm transition-colors">
            <RotateCcw className="w-4 h-4" /> Simpan & Buat Lagi
          </button>
        )}
        <button onClick={onSave} className="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-gradient-to-r from-violet-600 to-purple-600 hover:from-violet-500 hover:to-purple-500 font-bold text-sm transition-all">
          <Save className="w-4 h-4" /> {editId ? 'Simpan Perubahan' : 'Simpan & Tutup'}
        </button>
      </div>
    </div>
  );
}

// ─── Main Component ───────────────────────────────────────────────────────────
export default function ContentDashboard({ onLogout, userData }: { onLogout: () => void; userData: { name: string; email: string } }) {
  const [activeView, setActiveView] = useState<ActiveView>('questions');

  // Questions
  const [questions, setQuestions] = useState<Question[]>(MOCK_QUESTIONS);
  const [qSearch, setQSearch] = useState('');
  const [qFilterCat, setQFilterCat] = useState<TestCategory | 'all'>('all');
  const [qFilterStatus, setQFilterStatus] = useState<QuestionStatus | 'all'>('all');
  const [qFilterDiff, setQFilterDiff] = useState<Difficulty | 'all'>('all');
  const [qFilterType, setQFilterType] = useState<QuestionType | 'all'>('all');

  // Side panel form
  const [panelOpen, setPanelOpen] = useState(false);
  const [editId, setEditId] = useState<string | null>(null);
  const [qForm, setQForm] = useState<QuestionFormState>(() => makeBlankQuestion('SNBT', userData.name));
  const [qDeleteId, setQDeleteId] = useState<string | null>(null);
  const [qDetailId, setQDetailId] = useState<string | null>(null);

  // Packages
  const [packages, setPackages] = useState<TryoutPackage[]>(MOCK_PACKAGES);
  const [pSearch, setPSearch] = useState('');
  const [pEditorOpen, setPEditorOpen] = useState(false);
  const [editPkgId, setEditPkgId] = useState<string | null>(null);
  const [pForm, setPForm] = useState<Omit<TryoutPackage, 'id' | 'createdBy' | 'createdAt' | 'updatedAt' | 'totalAttempts' | 'avgScore'>>({ title: '', subtitle: '', type: 'mini-tryout', category: 'SNBT', subjectFilter: [], questionIds: [], duration: 45, difficulty: 'Sedang', status: 'draft', targetClass: 'Kelas 12', tags: [], isPremium: false });
  const [pTagInput, setPTagInput] = useState('');
  const [pDeleteId, setPDeleteId] = useState<string | null>(null);
  const [builderOpen, setBuilderOpen] = useState(false);
  const [builderPkg, setBuilderPkg] = useState<TryoutPackage | null>(null);
  const [builderSearch, setBuilderSearch] = useState('');

  // ─── Question helpers ────────────────────────────────────────────────────
  const genCode = (q: QuestionFormState) => {
    const count = questions.filter(x => x.category === q.category && x.subject === q.subject).length + 1;
    return `${q.category}-${q.subject}-${String(count).padStart(3, '0')}`;
  };

  const openCreate = () => { setEditId(null); setQForm(makeBlankQuestion('SNBT', userData.name)); setPanelOpen(true); setQDetailId(null); };
  const openEdit = (q: Question) => {
    setEditId(q.id);
    setQForm({ category: q.category, subject: q.subject, topic: q.topic, questionType: q.questionType, stimulus: q.stimulus, text: q.text, options: q.options.map(o => ({ ...o })), correctKeys: [...q.correctKeys], bsStatements: q.bsStatements.map(s => ({ ...s })), matchPairs: q.matchPairs.map(p => ({ ...p })), keyAnswer: q.keyAnswer, rubric: q.rubric, explanation: q.explanation, difficulty: q.difficulty, status: q.status, tags: [...q.tags], createdBy: q.createdBy, reviewNote: q.reviewNote });
    setPanelOpen(true);
    setQDetailId(null);
  };

  const doSave = (andNew: boolean) => {
    if (!qForm.text.trim()) { toast.error('Teks soal wajib diisi'); return; }
    const now = today();
    if (editId) {
      setQuestions(prev => prev.map(q => q.id === editId ? { ...q, ...qForm, updatedAt: now } : q));
      toast.success('Soal diperbarui');
      setPanelOpen(false);
    } else {
      const code = genCode(qForm);
      const newQ: Question = { ...qForm, id: makeId(), code, createdAt: now, updatedAt: now, usedInPackages: [], attemptCount: 0, correctRate: 0 };
      setQuestions(prev => [newQ, ...prev]);
      toast.success('Soal tersimpan');
      if (andNew) { setQForm(makeBlankQuestion(qForm.category, userData.name)); }
      else { setPanelOpen(false); }
    }
  };

  const handleWorkflow = (qId: string, toStatus: QuestionStatus) => {
    setQuestions(prev => prev.map(q => q.id === qId ? { ...q, status: toStatus, updatedAt: today() } : q));
    const labels: Record<QuestionStatus, string> = { draft: 'dikembalikan ke draft', review: 'dikirim ke review', approved: 'diapprove', published: 'dipublish', rejected: 'ditolak' };
    toast.success(`Soal ${labels[toStatus]}`);
  };

  const handleDeleteQ = (id: string) => { setQuestions(prev => prev.filter(q => q.id !== id)); setQDeleteId(null); setQDetailId(null); toast.success('Soal dihapus'); };

  const duplicateQ = (q: Question) => {
    const now = today();
    setQuestions(prev => [{ ...q, id: makeId(), code: genCode(q), status: 'draft', createdAt: now, updatedAt: now, usedInPackages: [], attemptCount: 0, correctRate: 0 }, ...prev]);
    toast.success('Soal diduplikasi sebagai draft baru');
  };

  const filteredQ = useMemo(() => questions.filter(q => {
    const ms = !qSearch || q.text.toLowerCase().includes(qSearch.toLowerCase()) || q.code.toLowerCase().includes(qSearch.toLowerCase()) || q.topic.toLowerCase().includes(qSearch.toLowerCase());
    return ms && (qFilterCat === 'all' || q.category === qFilterCat) && (qFilterStatus === 'all' || q.status === qFilterStatus) && (qFilterDiff === 'all' || q.difficulty === qFilterDiff) && (qFilterType === 'all' || q.questionType === qFilterType);
  }), [questions, qSearch, qFilterCat, qFilterStatus, qFilterDiff, qFilterType]);

  const qStats = useMemo(() => ({
    total: questions.length, draft: questions.filter(q => q.status === 'draft').length,
    review: questions.filter(q => q.status === 'review').length, approved: questions.filter(q => q.status === 'approved').length,
    published: questions.filter(q => q.status === 'published').length, rejected: questions.filter(q => q.status === 'rejected').length,
  }), [questions]);

  // ─── Package helpers ─────────────────────────────────────────────────────
  const filteredP = packages.filter(p => (!pSearch || p.title.toLowerCase().includes(pSearch.toLowerCase())));
  const openCreateP = () => { setEditPkgId(null); setPForm({ title: '', subtitle: '', type: 'mini-tryout', category: 'SNBT', subjectFilter: [], questionIds: [], duration: 45, difficulty: 'Sedang', status: 'draft', targetClass: 'Kelas 12', tags: [], isPremium: false }); setPEditorOpen(true); };
  const openEditP = (p: TryoutPackage) => { setEditPkgId(p.id); setPForm({ title: p.title, subtitle: p.subtitle, type: p.type, category: p.category, subjectFilter: [...p.subjectFilter], questionIds: [...p.questionIds], duration: p.duration, difficulty: p.difficulty, status: p.status, targetClass: p.targetClass, tags: [...p.tags], isPremium: p.isPremium }); setPEditorOpen(true); };

  const handleSaveP = () => {
    if (!pForm.title.trim()) { toast.error('Judul paket wajib diisi'); return; }
    const now = today();
    if (editPkgId) {
      setPackages(prev => prev.map(p => p.id === editPkgId ? { ...p, ...pForm, updatedAt: now } : p));
      toast.success('Paket diperbarui');
    } else {
      setPackages(prev => [{ ...pForm, id: `pkg-${Date.now()}`, createdBy: userData.name, createdAt: now, updatedAt: now, totalAttempts: 0, avgScore: 0 }, ...prev]);
      toast.success('Paket dibuat');
    }
    setPEditorOpen(false);
  };

  const handleDeleteP = (id: string) => { setPackages(prev => prev.filter(p => p.id !== id)); setPDeleteId(null); toast.success('Paket dihapus'); };

  const addPTag = () => { const t = pTagInput.trim(); if (t && !pForm.tags.includes(t)) setPForm(f => ({ ...f, tags: [...f.tags, t] })); setPTagInput(''); };

  const openBuilder = (p: TryoutPackage) => { setBuilderPkg({ ...p, questionIds: [...p.questionIds] }); setBuilderSearch(''); setBuilderOpen(true); };
  const toggleQInPkg = (qId: string) => setBuilderPkg(prev => !prev ? prev : { ...prev, questionIds: prev.questionIds.includes(qId) ? prev.questionIds.filter(id => id !== qId) : [...prev.questionIds, qId] });
  const saveBuilder = () => {
    if (!builderPkg) return;
    setPackages(prev => prev.map(p => p.id === builderPkg.id ? { ...builderPkg, updatedAt: today() } : p));
    setBuilderOpen(false);
    toast.success(`${builderPkg.questionIds.length} soal disimpan`);
  };

  const builderQList = useMemo(() => {
    if (!builderPkg) return [];
    return questions.filter(q => {
      const ms = !builderSearch || q.text.toLowerCase().includes(builderSearch.toLowerCase()) || q.code.toLowerCase().includes(builderSearch.toLowerCase());
      const cat = q.category === builderPkg.category;
      const ready = q.status === 'approved' || q.status === 'published';
      return ms && cat && ready;
    });
  }, [questions, builderPkg, builderSearch]);

  const detailQ = qDetailId ? questions.find(q => q.id === qDetailId) : null;

  const typeColors: Record<string, string> = { 'full-tryout': 'bg-violet-900/50 text-violet-300 border-violet-700/50', 'mini-tryout': 'bg-blue-900/50 text-blue-300 border-blue-700/50', 'drilling': 'bg-orange-900/50 text-orange-300 border-orange-700/50', 'chapter-test': 'bg-emerald-900/50 text-emerald-300 border-emerald-700/50' };
  const typeLabels: Record<string, string> = { 'full-tryout': 'Full Tryout', 'mini-tryout': 'Mini Tryout', 'drilling': 'Drilling', 'chapter-test': 'Chapter Test' };
  const pkgStatus: Record<string, { label: string; color: string }> = { draft: { label: 'Draft', color: 'bg-gray-800 text-gray-400 border-gray-700' }, review: { label: 'Review', color: 'bg-amber-900/50 text-amber-300 border-amber-700/50' }, published: { label: 'Published', color: 'bg-emerald-900/50 text-emerald-300 border-emerald-700/50' }, archived: { label: 'Archived', color: 'bg-red-900/50 text-red-400 border-red-700/50' } };

  // ─── Render ───────────────────────────────────────────────────────────────
  return (
    <div className="min-h-screen bg-[#09080f] text-white flex" style={{ fontFamily: "'Plus Jakarta Sans', sans-serif" }}>

      {/* Sidebar */}
      <aside className="w-56 bg-[#0f0d1a] border-r border-white/10 flex flex-col shrink-0">
        <div className="p-4 border-b border-white/10">
          <div className="flex items-center gap-2.5">
            <div className="w-7 h-7 bg-gradient-to-br from-violet-500 to-purple-700 rounded-lg flex items-center justify-center">
              <Layers className="w-3.5 h-3.5 text-white" />
            </div>
            <div>
              <p className="text-sm font-bold text-white">Tim Konten</p>
              <p className="text-[10px] text-violet-400">GASPOLPTN</p>
            </div>
          </div>
        </div>

        <nav className="flex-1 p-2.5 space-y-0.5">
          {([
            { id: 'questions' as ActiveView, label: 'Bank Soal', icon: BookOpen, badge: qStats.review > 0 ? qStats.review : null },
            { id: 'packages' as ActiveView, label: 'Paket Tryout', icon: Package, badge: null },
            { id: 'analytics' as ActiveView, label: 'Analitik', icon: BarChart3, badge: null },
          ]).map(({ id, label, icon: Icon, badge }) => (
            <button key={id} onClick={() => setActiveView(id)} className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-medium transition-colors ${activeView === id ? 'bg-violet-600/30 text-violet-300 border border-violet-600/30' : 'text-white/50 hover:text-white hover:bg-white/5'}`}>
              <Icon className="w-4 h-4" /><span className="flex-1 text-left">{label}</span>
              {badge !== null && <span className="text-[10px] font-black px-1.5 py-0.5 rounded-full bg-amber-500 text-white">{badge}</span>}
            </button>
          ))}
        </nav>

        {/* Alur soal mini */}
        <div className="mx-2.5 mb-2.5 p-3 bg-white/[0.03] border border-white/10 rounded-xl">
          <p className="text-[9px] font-bold text-white/25 uppercase tracking-widest mb-2">Alur Soal</p>
          <div className="space-y-1">
            {(['draft', 'review', 'approved', 'published'] as QuestionStatus[]).map((s, i) => (
              <div key={s} className="flex items-center gap-1.5">
                <span className={`w-1.5 h-1.5 rounded-full ${qStatusConfig[s].dot} shrink-0`} />
                <span className="text-[9px] text-white/35">{qStatusConfig[s].label}</span>
                {i < 3 && <ArrowRight className="w-2 h-2 text-white/15 ml-auto" />}
              </div>
            ))}
          </div>
        </div>

        <div className="p-2.5 border-t border-white/10">
          <div className="flex items-center gap-2 px-2 py-1.5 mb-0.5">
            <div className="w-7 h-7 rounded-full bg-gradient-to-br from-violet-500 to-purple-600 flex items-center justify-center text-xs font-bold shrink-0">{userData.name.charAt(0)}</div>
            <div className="flex-1 min-w-0">
              <p className="text-xs font-semibold text-white truncate">{userData.name}</p>
              <p className="text-[10px] text-white/35 truncate">{userData.email}</p>
            </div>
            <NotificationBell role="content" dark={true} />
          </div>
          <button onClick={onLogout} className="w-full flex items-center gap-2 px-2 py-2 rounded-xl text-sm text-white/40 hover:text-red-400 hover:bg-red-900/20 transition-colors">
            <LogOut className="w-4 h-4" /> Keluar
          </button>
        </div>
      </aside>

      {/* ═══════════ BANK SOAL (questions view) ═══════════════════════════════ */}
      {activeView === 'questions' && (
        <div className="flex-1 flex overflow-hidden">
          {/* List pane */}
          <div className={`flex flex-col overflow-hidden transition-all ${panelOpen ? 'flex-1' : 'flex-1'}`}>
            {/* Toolbar */}
            <div className="p-5 border-b border-white/10 space-y-4 shrink-0">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <h1 className="text-xl font-black text-white" style={{ fontFamily: "'Outfit', sans-serif" }}>Bank Soal</h1>
                  <p className="text-xs text-white/40 mt-0.5">SNBT · TKA-IPA · TKA-IPS · AKM</p>
                </div>
                <button onClick={openCreate} className="flex items-center gap-2 px-4 py-2.5 rounded-xl bg-gradient-to-r from-violet-600 to-purple-600 hover:from-violet-500 hover:to-purple-500 font-bold text-sm transition-all shadow-lg hover:-translate-y-0.5 shrink-0">
                  <Plus className="w-4 h-4" /> Buat Soal
                </button>
              </div>

              {/* Status strip */}
              <div className="flex gap-2 overflow-x-auto pb-0.5">
                {(['all', 'draft', 'review', 'approved', 'published', 'rejected'] as const).map(s => {
                  const count = s === 'all' ? questions.length : qStats[s as keyof typeof qStats];
                  const active = qFilterStatus === s;
                  return (
                    <button key={s} onClick={() => setQFilterStatus(s as QuestionStatus | 'all')} className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-bold border transition-colors whitespace-nowrap ${active ? 'border-violet-500/60 bg-violet-600/20 text-violet-200' : 'border-white/10 bg-white/5 text-white/50 hover:text-white'}`}>
                      {s !== 'all' && <span className={`w-1.5 h-1.5 rounded-full ${qStatusConfig[s as QuestionStatus].dot}`} />}
                      {s === 'all' ? `Semua (${count})` : `${qStatusConfig[s as QuestionStatus].label} (${count})`}
                    </button>
                  );
                })}
              </div>

              {/* Search + filters */}
              <div className="flex gap-2 flex-wrap">
                <div className="relative flex-1 min-w-36">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-white/30" />
                  <input value={qSearch} onChange={e => setQSearch(e.target.value)} placeholder="Cari soal..." className="w-full pl-8 pr-3 py-2 bg-white/5 border border-white/10 rounded-xl text-sm text-white placeholder-white/25 focus:outline-none focus:border-violet-500/50" />
                </div>
                <select value={qFilterCat} onChange={e => setQFilterCat(e.target.value as TestCategory | 'all')} className="px-3 py-2 bg-white/5 border border-white/10 rounded-xl text-xs text-white/70 focus:outline-none">
                  <option value="all" className="bg-[#0f0d1a]">Semua Kategori</option>
                  {(['SNBT', 'TKA-IPA', 'TKA-IPS', 'AKM'] as TestCategory[]).map(c => <option key={c} value={c} className="bg-[#0f0d1a]">{c}</option>)}
                </select>
                <select value={qFilterType} onChange={e => setQFilterType(e.target.value as QuestionType | 'all')} className="px-3 py-2 bg-white/5 border border-white/10 rounded-xl text-xs text-white/70 focus:outline-none">
                  <option value="all" className="bg-[#0f0d1a]">Semua Tipe</option>
                  {(['pg', 'pgk', 'bs', 'menjodohkan', 'isian', 'uraian'] as QuestionType[]).map(t => <option key={t} value={t} className="bg-[#0f0d1a]">{QTYPE_LABELS[t]}</option>)}
                </select>
                <select value={qFilterDiff} onChange={e => setQFilterDiff(e.target.value as Difficulty | 'all')} className="px-3 py-2 bg-white/5 border border-white/10 rounded-xl text-xs text-white/70 focus:outline-none">
                  <option value="all" className="bg-[#0f0d1a]">Semua Kesulitan</option>
                  {(['Mudah', 'Sedang', 'Sulit'] as Difficulty[]).map(d => <option key={d} value={d} className="bg-[#0f0d1a]">{d}</option>)}
                </select>
              </div>
            </div>

            {/* Question rows */}
            <div className="flex-1 overflow-y-auto p-5 space-y-2">
              {filteredQ.length === 0 && (
                <div className="text-center py-16">
                  <BookOpen className="w-10 h-10 mx-auto mb-3 text-white/10" />
                  <p className="text-sm text-white/30">Tidak ada soal ditemukan</p>
                  <button onClick={openCreate} className="mt-4 text-xs text-violet-400 hover:text-violet-300 flex items-center gap-1.5 mx-auto"><Plus className="w-3.5 h-3.5" />Buat soal pertama</button>
                </div>
              )}
              {filteredQ.map(q => {
                const sc = qStatusConfig[q.status];
                const transitions = WORKFLOW.filter(w => w.from === q.status);
                const isSelected = qDetailId === q.id;
                return (
                  <div key={q.id} className={`bg-white/[0.03] border rounded-2xl p-4 transition-all cursor-pointer ${isSelected ? 'border-violet-500/40 bg-violet-900/10' : 'border-white/10 hover:border-white/20'}`} onClick={() => setQDetailId(isSelected ? null : q.id)}>
                    <div className="flex items-start gap-3">
                      {/* Left meta */}
                      <div className="shrink-0 w-[4.5rem] text-center">
                        <p className="text-[10px] font-black text-violet-400 font-mono leading-tight">{q.code}</p>
                        <span className={`mt-1 inline-block text-[9px] px-1.5 py-0.5 rounded border font-bold ${CATEGORY_COLORS[q.category]}`}>{q.category}</span>
                        <p className="text-[9px] text-white/30 mt-0.5">{q.subject}</p>
                      </div>
                      {/* Content */}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 flex-wrap mb-1">
                          <span className={`text-[10px] px-2 py-0.5 rounded-full border font-semibold flex items-center gap-1 ${sc.color}`}><span className={`w-1.5 h-1.5 rounded-full ${sc.dot}`} />{sc.label}</span>
                          <span className="text-[10px] px-2 py-0.5 rounded-full bg-white/5 border border-white/10 text-white/40 font-medium">{QTYPE_LABELS[q.questionType]}</span>
                          <span className={`text-xs font-bold ${difficultyColors[q.difficulty]}`}>{q.difficulty}</span>
                          {q.topic && <span className="text-xs text-white/30">{q.topic}</span>}
                        </div>
                        <p className="text-sm text-white/80 leading-snug line-clamp-2">{q.text}</p>
                        {q.reviewNote && (q.status === 'review' || q.status === 'rejected') && (
                          <p className={`text-xs mt-1 flex items-center gap-1 ${q.status === 'rejected' ? 'text-red-400' : 'text-amber-400'}`}>
                            <AlertCircle className="w-3 h-3" />{q.reviewNote}
                          </p>
                        )}
                        {q.attemptCount > 0 && (
                          <div className="flex items-center gap-3 mt-1.5 text-[10px] text-white/30">
                            <span>{q.attemptCount.toLocaleString('id-ID')} percobaan</span>
                            <span className={q.correctRate >= 70 ? 'text-emerald-400' : q.correctRate >= 50 ? 'text-amber-400' : 'text-red-400'}>{q.correctRate}% benar</span>
                          </div>
                        )}
                      </div>
                      {/* Actions */}
                      <div className="flex items-center gap-1 shrink-0" onClick={e => e.stopPropagation()}>
                        {transitions.slice(0, 2).map(t => (
                          <button key={t.to} onClick={() => handleWorkflow(q.id, t.to)} className={`px-2.5 py-1 rounded-lg text-[10px] font-bold border transition-colors ${t.btn}`}>{t.label}</button>
                        ))}
                        <button onClick={() => openEdit(q)} className="p-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-white/40 hover:text-white transition-colors"><Edit3 className="w-3.5 h-3.5" /></button>
                        <button onClick={() => duplicateQ(q)} className="p-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-white/40 hover:text-white transition-colors"><Copy className="w-3.5 h-3.5" /></button>
                        <button onClick={() => setQDeleteId(q.id)} className="p-1.5 rounded-lg bg-white/5 hover:bg-red-900/30 text-white/40 hover:text-red-400 transition-colors"><Trash2 className="w-3.5 h-3.5" /></button>
                      </div>
                    </div>

                    {/* Expanded detail */}
                    {isSelected && (
                      <div className="mt-3 pt-3 border-t border-white/10 grid grid-cols-2 gap-3 text-xs" onClick={e => e.stopPropagation()}>
                        {q.questionType === 'pg' || q.questionType === 'pgk' ? (
                          <div className="col-span-2 space-y-1">
                            {q.options.map(opt => (
                              <div key={opt.key} className={`flex items-center gap-2 px-3 py-1.5 rounded-lg border ${q.correctKeys.includes(opt.key) ? 'bg-emerald-900/30 border-emerald-600/40 text-emerald-300' : 'bg-white/[0.02] border-white/10 text-white/50'}`}>
                                <span className="font-black w-4">{opt.key}</span><span>{opt.text}</span>
                                {q.correctKeys.includes(opt.key) && <CheckCircle2 className="w-3 h-3 ml-auto" />}
                              </div>
                            ))}
                          </div>
                        ) : q.questionType === 'bs' ? (
                          <div className="col-span-2 space-y-1">
                            {q.bsStatements.map((s, i) => (
                              <div key={s.id} className={`flex items-center gap-2 px-3 py-1.5 rounded-lg border ${s.answer ? 'bg-emerald-900/20 border-emerald-600/30 text-emerald-300' : 'bg-red-900/20 border-red-600/30 text-red-300'}`}>
                                <span className="font-black shrink-0">{i + 1}.</span><span className="flex-1">{s.text}</span>
                                <span className="font-black text-[10px]">{s.answer ? 'BENAR' : 'SALAH'}</span>
                              </div>
                            ))}
                          </div>
                        ) : q.questionType === 'menjodohkan' ? (
                          <div className="col-span-2 space-y-1">
                            {q.matchPairs.map(p => (
                              <div key={p.id} className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/[0.03] border border-white/10 text-white/60">
                                <span className="flex-1">{p.left}</span><span className="text-white/25">↔</span><span className="flex-1 text-right">{p.right}</span>
                              </div>
                            ))}
                          </div>
                        ) : (
                          <div className="col-span-2 px-3 py-2 rounded-lg bg-white/[0.03] border border-white/10 text-white/60">{q.keyAnswer || '—'}</div>
                        )}
                        {q.explanation && <div className="col-span-2 text-white/40 text-[11px]"><span className="font-bold text-white/30">Pembahasan:</span> {q.explanation}</div>}
                      </div>
                    )}
                  </div>
                );
              })}
              <p className="text-center text-[10px] text-white/20 pt-2">Menampilkan {filteredQ.length} dari {questions.length} soal</p>
            </div>
          </div>

          {/* ── Persistent Side Panel ─────────────────────────────────────── */}
          {panelOpen && (
            <div className="w-96 border-l border-white/10 bg-[#0f0d1a] flex flex-col shrink-0 overflow-hidden">
              <QuestionForm
                form={qForm} setForm={setQForm} editId={editId}
                onSave={() => doSave(false)}
                onSaveAndNew={() => doSave(true)}
                onCancel={() => setPanelOpen(false)}
              />
            </div>
          )}

          {/* FAB when panel closed */}
          {!panelOpen && (
            <button onClick={openCreate} title="Buat soal baru" className="fixed bottom-8 right-8 w-14 h-14 rounded-2xl bg-gradient-to-br from-violet-600 to-purple-700 hover:from-violet-500 hover:to-purple-600 shadow-2xl flex items-center justify-center transition-all hover:-translate-y-1 hover:shadow-violet-900/50">
              <Plus className="w-6 h-6 text-white" />
            </button>
          )}
        </div>
      )}

      {/* ═══════════ PAKET TRYOUT ════════════════════════════════════════════ */}
      {activeView === 'packages' && (
        <div className="flex-1 overflow-auto p-6 lg:p-8">
          <div className="flex items-center justify-between gap-4 mb-6">
            <div>
              <h1 className="text-2xl font-black text-white" style={{ fontFamily: "'Outfit', sans-serif" }}>Paket Tryout</h1>
              <p className="text-sm text-white/50">Kelola paket dan susun soal dari bank soal</p>
            </div>
            <button onClick={openCreateP} className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gradient-to-r from-violet-600 to-purple-600 hover:from-violet-500 hover:to-purple-500 font-bold text-sm transition-all shadow-lg hover:-translate-y-0.5">
              <Plus className="w-4 h-4" /> Buat Paket
            </button>
          </div>

          <div className="flex gap-3 mb-5">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-white/30" />
              <input value={pSearch} onChange={e => setPSearch(e.target.value)} placeholder="Cari paket..." className="w-full pl-9 pr-4 py-2.5 bg-white/5 border border-white/10 rounded-xl text-sm text-white placeholder-white/30 focus:outline-none focus:border-violet-500/50" />
            </div>
          </div>

          <div className="space-y-3">
            {filteredP.map(pkg => {
              const sc = pkgStatus[pkg.status];
              const assignedQ = questions.filter(q => pkg.questionIds.includes(q.id));
              const readyQ = assignedQ.filter(q => q.status === 'approved' || q.status === 'published');
              return (
                <div key={pkg.id} className="bg-white/[0.03] border border-white/10 hover:border-white/20 rounded-2xl p-5 transition-all">
                  <div className="flex flex-col lg:flex-row lg:items-center gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 flex-wrap mb-1">
                        <h3 className="text-sm font-bold text-white">{pkg.title}</h3>
                        {pkg.isPremium && <span className="text-[9px] px-2 py-0.5 rounded-full bg-amber-900/40 text-amber-300 border border-amber-700/40 font-bold">PREMIUM</span>}
                      </div>
                      <p className="text-xs text-white/50 mb-2">{pkg.subtitle}</p>
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className={`text-xs px-2.5 py-0.5 rounded-full border font-medium ${typeColors[pkg.type]}`}>{typeLabels[pkg.type]}</span>
                        <span className={`text-xs px-2.5 py-0.5 rounded-full border font-medium ${sc.color}`}>{sc.label}</span>
                        <span className={`text-xs px-2 py-0.5 rounded border ${CATEGORY_COLORS[pkg.category]}`}>{pkg.category}</span>
                      </div>
                    </div>
                    <div className="flex items-center gap-5 shrink-0">
                      {[{ val: pkg.questionIds.length, lbl: 'Soal' }, { val: `${pkg.duration}m`, lbl: 'Durasi' }, { val: pkg.totalAttempts > 0 ? pkg.totalAttempts.toLocaleString('id-ID') : '—', lbl: 'Percobaan' }].map(({ val, lbl }) => (
                        <div key={lbl} className="text-center"><p className="text-sm font-black text-white">{val}</p><p className="text-[10px] text-white/40">{lbl}</p></div>
                      ))}
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <button onClick={() => openBuilder(pkg)} className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-violet-900/40 hover:bg-violet-600/40 text-violet-300 text-xs font-semibold border border-violet-700/40 transition-colors"><GripVertical className="w-3.5 h-3.5" />Susun Soal</button>
                      <button onClick={() => openEditP(pkg)} className="p-2 rounded-lg bg-white/5 hover:bg-white/10 text-white/60 hover:text-white transition-colors"><Edit3 className="w-4 h-4" /></button>
                      <button onClick={() => setPDeleteId(pkg.id)} className="p-2 rounded-lg bg-white/5 hover:bg-red-900/30 text-white/60 hover:text-red-400 transition-colors"><Trash2 className="w-4 h-4" /></button>
                    </div>
                  </div>
                  {pkg.questionIds.length > 0 && (
                    <div className="mt-3 pt-3 border-t border-white/5">
                      <div className="flex justify-between text-[10px] text-white/30 mb-1">
                        <span>{readyQ.length}/{pkg.questionIds.length} soal siap</span>
                        <span>Update: {pkg.updatedAt}</span>
                      </div>
                      <div className="h-1 bg-white/10 rounded-full overflow-hidden">
                        <div className="h-full bg-gradient-to-r from-violet-500 to-purple-500 rounded-full" style={{ width: `${(readyQ.length / pkg.questionIds.length) * 100}%` }} />
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* ═══════════ ANALITIK ════════════════════════════════════════════════ */}
      {activeView === 'analytics' && (
        <div className="flex-1 overflow-auto p-6 lg:p-8 space-y-6">
          <div>
            <h1 className="text-2xl font-black text-white" style={{ fontFamily: "'Outfit', sans-serif" }}>Analitik Konten</h1>
            <p className="text-sm text-white/40">Performa soal, distribusi, dan statistik paket</p>
          </div>
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
            {[
              { label: 'Total Soal', val: questions.length, sub: `${qStats.published} published`, icon: BookOpen, c: 'text-violet-400' },
              { label: 'Total Paket', val: packages.length, sub: `${packages.filter(p => p.status === 'published').length} live`, icon: Package, c: 'text-blue-400' },
              { label: 'Total Percobaan', val: packages.reduce((s, p) => s + p.totalAttempts, 0).toLocaleString('id-ID'), sub: 'semua paket', icon: Users, c: 'text-emerald-400' },
              { label: 'Tipe Soal', val: Object.values(QTYPE_LABELS).length, sub: 'format didukung', icon: FileText, c: 'text-amber-400' },
            ].map(({ label, val, sub, icon: Icon, c }) => (
              <div key={label} className="bg-white/[0.03] border border-white/10 rounded-2xl p-5">
                <Icon className={`w-5 h-5 ${c} mb-3`} />
                <p className={`text-2xl font-black ${c}`} style={{ fontFamily: "'Outfit', sans-serif" }}>{val}</p>
                <p className="text-xs text-white/40 mt-0.5">{label}</p>
                <p className="text-[10px] text-white/25">{sub}</p>
              </div>
            ))}
          </div>
          <div className="grid lg:grid-cols-2 gap-5">
            <div className="bg-white/[0.03] border border-white/10 rounded-2xl p-6">
              <h3 className="text-sm font-bold text-white mb-1 flex items-center gap-2"><TrendingUp className="w-4 h-4 text-violet-400" />Tren Percobaan (Mingguan)</h3>
              <ResponsiveContainer width="100%" height={180}>
                <LineChart data={attemptTrend}><CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.05)" /><XAxis dataKey="w" tick={{ fill: 'rgba(255,255,255,0.35)', fontSize: 10 }} /><YAxis tick={{ fill: 'rgba(255,255,255,0.35)', fontSize: 10 }} /><Tooltip contentStyle={{ background: '#0f0d1a', border: '1px solid rgba(255,255,255,0.1)', borderRadius: 12 }} /><Line key="line-attempts" type="monotone" dataKey="v" stroke="#8b5cf6" strokeWidth={2.5} dot={{ fill: '#8b5cf6', r: 4 }} /></LineChart>
              </ResponsiveContainer>
            </div>
            <div className="bg-white/[0.03] border border-white/10 rounded-2xl p-6">
              <h3 className="text-sm font-bold text-white mb-1 flex items-center gap-2"><BarChart2 className="w-4 h-4 text-blue-400" />Distribusi per Kategori</h3>
              <ResponsiveContainer width="100%" height={180}>
                <BarChart data={categoryDistrib}><CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.05)" /><XAxis dataKey="name" tick={{ fill: 'rgba(255,255,255,0.5)', fontSize: 11 }} /><YAxis tick={{ fill: 'rgba(255,255,255,0.35)', fontSize: 10 }} /><Tooltip contentStyle={{ background: '#0f0d1a', border: '1px solid rgba(255,255,255,0.1)', borderRadius: 12 }} /><Bar key="bar-category" dataKey="count" radius={[6, 6, 0, 0]}>{categoryDistrib.map((e) => <Cell key={`cat-${e.name}`} fill={e.fill} />)}</Bar></BarChart>
              </ResponsiveContainer>
            </div>
            <div className="bg-white/[0.03] border border-white/10 rounded-2xl p-6">
              <h3 className="text-sm font-bold text-white mb-1 flex items-center gap-2"><Filter className="w-4 h-4 text-emerald-400" />Distribusi Tipe Soal</h3>
              <ResponsiveContainer width="100%" height={180}>
                <BarChart data={typeDistrib}><CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.05)" /><XAxis dataKey="name" tick={{ fill: 'rgba(255,255,255,0.5)', fontSize: 10 }} /><YAxis tick={{ fill: 'rgba(255,255,255,0.35)', fontSize: 10 }} /><Tooltip contentStyle={{ background: '#0f0d1a', border: '1px solid rgba(255,255,255,0.1)', borderRadius: 12 }} /><Bar key="bar-type" dataKey="count" radius={[6, 6, 0, 0]}>{typeDistrib.map((e) => <Cell key={`type-${e.name}`} fill={e.fill} />)}</Bar></BarChart>
              </ResponsiveContainer>
            </div>
            <div className="bg-white/[0.03] border border-white/10 rounded-2xl p-6">
              <h3 className="text-sm font-bold text-white mb-4 flex items-center gap-2"><Flame className="w-4 h-4 text-orange-400" />Soal Paling Sulit</h3>
              <div className="space-y-3">
                {[...questions].filter(q => q.attemptCount > 0).sort((a, b) => a.correctRate - b.correctRate).slice(0, 5).map((q, i) => (
                  <div key={q.id} className="flex items-center gap-3">
                    <span className="text-xs font-black text-white/20 w-4">#{i + 1}</span>
                    <div className="flex-1 min-w-0">
                      <p className="text-xs font-semibold text-white/70 truncate">{q.text.slice(0, 45)}...</p>
                      <div className="flex items-center gap-2 mt-1">
                        <span className={`text-[9px] px-1.5 py-0.5 rounded border font-bold ${CATEGORY_COLORS[q.category]}`}>{q.subject}</span>
                        <div className="flex-1 h-1.5 bg-white/10 rounded-full overflow-hidden">
                          <div className="h-full bg-red-500 rounded-full" style={{ width: `${q.correctRate}%` }} />
                        </div>
                        <span className="text-xs font-bold text-red-400 shrink-0">{q.correctRate}%</span>
                      </div>
                    </div>
                  </div>
                ))}
                {questions.filter(q => q.attemptCount > 0).length === 0 && <p className="text-xs text-white/25 text-center py-4">Belum ada data percobaan</p>}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ═══ PACKAGE EDITOR MODAL ════════════════════════════════════════════ */}
      {pEditorOpen && (
        <div className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 p-4 overflow-y-auto">
          <div className="bg-[#0f0d1a] border border-white/15 rounded-3xl w-full max-w-xl my-4 shadow-2xl">
            <div className="flex items-center justify-between p-6 border-b border-white/10">
              <h2 className="text-lg font-black text-white">{editPkgId ? 'Edit Paket' : 'Buat Paket Baru'}</h2>
              <button onClick={() => setPEditorOpen(false)} className="p-2 rounded-xl hover:bg-white/10 text-white/50 hover:text-white transition-colors"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-6 space-y-4 overflow-y-auto max-h-[70vh]">
              <div><label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Judul *</label><input value={pForm.title} onChange={e => setPForm(f => ({ ...f, title: e.target.value }))} placeholder="Tryout SNBT Full Simulasi #3" className="w-full px-4 py-3 bg-white/5 border border-white/10 rounded-xl text-sm text-white placeholder-white/30 focus:outline-none focus:border-violet-500/60" /></div>
              <div><label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Deskripsi</label><input value={pForm.subtitle} onChange={e => setPForm(f => ({ ...f, subtitle: e.target.value }))} placeholder="Deskripsi singkat isi paket" className="w-full px-4 py-3 bg-white/5 border border-white/10 rounded-xl text-sm text-white placeholder-white/30 focus:outline-none focus:border-violet-500/60" /></div>
              <div>
                <label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Kategori Ujian</label>
                <div className="grid grid-cols-4 gap-1.5">
                  {(['SNBT', 'TKA-IPA', 'TKA-IPS', 'AKM'] as TestCategory[]).map(cat => (
                    <button key={cat} onClick={() => setPForm(f => ({ ...f, category: cat, subjectFilter: [] }))} className={`py-2 rounded-xl text-xs font-bold border transition-colors ${pForm.category === cat ? 'bg-violet-600/40 border-violet-500/60 text-violet-200' : 'bg-white/5 border-white/10 text-white/50 hover:text-white'}`}>{cat}</button>
                  ))}
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div><label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Tipe</label><select value={pForm.type} onChange={e => setPForm(f => ({ ...f, type: e.target.value as PackageType }))} className="w-full px-3 py-2.5 bg-white/5 border border-white/10 rounded-xl text-sm text-white focus:outline-none">{(['full-tryout', 'mini-tryout', 'drilling', 'chapter-test'] as PackageType[]).map(t => <option key={t} value={t} className="bg-[#0f0d1a]">{typeLabels[t]}</option>)}</select></div>
                <div><label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Durasi (menit)</label><input type="number" value={pForm.duration} onChange={e => setPForm(f => ({ ...f, duration: Number(e.target.value) }))} className="w-full px-3 py-2.5 bg-white/5 border border-white/10 rounded-xl text-sm text-white focus:outline-none" /></div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div><label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Target Kelas</label><select value={pForm.targetClass} onChange={e => setPForm(f => ({ ...f, targetClass: e.target.value }))} className="w-full px-3 py-2.5 bg-white/5 border border-white/10 rounded-xl text-sm text-white focus:outline-none">{['Kelas 10', 'Kelas 11', 'Kelas 12', 'Kelas 11-12', 'Kelas 10-12'].map(c => <option key={c} value={c} className="bg-[#0f0d1a]">{c}</option>)}</select></div>
                <div><label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Status</label><select value={pForm.status} onChange={e => setPForm(f => ({ ...f, status: e.target.value as PackageStatus }))} className="w-full px-3 py-2.5 bg-white/5 border border-white/10 rounded-xl text-sm text-white focus:outline-none">{(['draft', 'review', 'published'] as PackageStatus[]).map(s => <option key={s} value={s} className="bg-[#0f0d1a]">{pkgStatus[s].label}</option>)}</select></div>
              </div>
              <div><label className="text-[10px] font-bold text-white/40 uppercase tracking-wider block mb-1.5">Tags</label><div className="flex gap-2 mb-2"><input value={pTagInput} onChange={e => setPTagInput(e.target.value)} onKeyDown={e => e.key === 'Enter' && (e.preventDefault(), addPTag())} placeholder="Tag + Enter" className="flex-1 px-3 py-2 bg-white/5 border border-white/10 rounded-xl text-sm text-white placeholder-white/25 focus:outline-none" /><button onClick={addPTag} className="px-4 py-2 rounded-xl bg-violet-600/20 text-violet-300 border border-violet-600/30 text-sm hover:bg-violet-600/30">+</button></div><div className="flex flex-wrap gap-1.5">{pForm.tags.map(t => (<span key={t} className="flex items-center gap-1 text-xs px-2.5 py-1 rounded-full bg-white/10 text-white/60">#{t}<button onClick={() => setPForm(f => ({ ...f, tags: f.tags.filter(x => x !== t) }))} className="text-white/40 hover:text-red-400"><X className="w-3 h-3" /></button></span>))}</div></div>
              <label className="flex items-center gap-3 cursor-pointer"><div onClick={() => setPForm(f => ({ ...f, isPremium: !f.isPremium }))} className={`w-11 h-6 rounded-full transition-colors relative ${pForm.isPremium ? 'bg-amber-500' : 'bg-white/20'}`}><div className={`absolute top-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform ${pForm.isPremium ? 'translate-x-5' : 'translate-x-0.5'}`} /></div><span className="text-sm text-white/70">Konten Premium</span></label>
            </div>
            <div className="flex gap-3 p-6 border-t border-white/10">
              <button onClick={() => setPEditorOpen(false)} className="flex-1 py-3 rounded-xl border border-white/15 text-sm font-semibold text-white/60 hover:text-white hover:bg-white/5 transition-colors">Batal</button>
              <button onClick={handleSaveP} className="flex-1 flex items-center justify-center gap-2 py-3 rounded-xl bg-gradient-to-r from-violet-600 to-purple-600 font-bold text-sm transition-all hover:from-violet-500 hover:to-purple-500"><Save className="w-4 h-4" />{editPkgId ? 'Simpan' : 'Buat Paket'}</button>
            </div>
          </div>
        </div>
      )}

      {/* ═══ PACKAGE BUILDER ════════════════════════════════════════════════ */}
      {builderOpen && builderPkg && (
        <div className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="bg-[#0f0d1a] border border-white/15 rounded-3xl w-full max-w-3xl h-[85vh] flex flex-col shadow-2xl">
            <div className="flex items-center justify-between p-5 border-b border-white/10 shrink-0">
              <div><h2 className="text-lg font-black text-white">Susun Soal Paket</h2><p className="text-sm text-white/40">{builderPkg.title} · {builderPkg.category}</p></div>
              <div className="flex items-center gap-3">
                <span className="text-sm font-bold text-violet-400">{builderPkg.questionIds.length} dipilih</span>
                <button onClick={() => setBuilderOpen(false)} className="p-2 rounded-xl hover:bg-white/10 text-white/50 hover:text-white transition-colors"><X className="w-5 h-5" /></button>
              </div>
            </div>
            <div className="flex flex-1 overflow-hidden">
              <div className="flex-1 border-r border-white/10 flex flex-col">
                <div className="p-4 border-b border-white/10 shrink-0">
                  <p className="text-[10px] font-bold text-white/30 mb-2">SOAL TERSEDIA (approved/published, kategori {builderPkg.category})</p>
                  <div className="relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-white/30" />
                    <input value={builderSearch} onChange={e => setBuilderSearch(e.target.value)} placeholder="Cari soal..." className="w-full pl-8 pr-3 py-2 bg-white/5 border border-white/10 rounded-xl text-xs text-white placeholder-white/30 focus:outline-none" />
                  </div>
                </div>
                <div className="flex-1 overflow-y-auto p-3 space-y-1.5">
                  {builderQList.length === 0 && <p className="text-center text-xs text-white/25 py-8">Tidak ada soal tersedia untuk kategori ini</p>}
                  {builderQList.map(q => {
                    const sel = builderPkg.questionIds.includes(q.id);
                    return (
                      <div key={q.id} onClick={() => toggleQInPkg(q.id)} className={`flex items-start gap-3 p-3 rounded-xl cursor-pointer border transition-all ${sel ? 'bg-violet-900/30 border-violet-600/40' : 'bg-white/[0.02] border-white/10 hover:bg-white/5'}`}>
                        <div className={`w-5 h-5 rounded border-2 flex items-center justify-center shrink-0 mt-0.5 ${sel ? 'bg-violet-600 border-violet-500' : 'border-white/20'}`}>{sel && <CheckCircle2 className="w-3 h-3 text-white" />}</div>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-1.5 mb-0.5">
                            <span className="text-[9px] font-bold text-violet-400 font-mono">{q.code}</span>
                            <span className="text-[9px] px-1.5 py-0.5 rounded bg-white/5 text-white/40">{QTYPE_LABELS[q.questionType]}</span>
                            <span className={`text-[9px] font-bold ${difficultyColors[q.difficulty]}`}>{q.difficulty}</span>
                          </div>
                          <p className="text-xs text-white/60 line-clamp-2">{q.text}</p>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
              <div className="w-56 flex flex-col">
                <div className="p-4 border-b border-white/10 shrink-0"><p className="text-[10px] font-bold text-white/30">TERPILIH ({builderPkg.questionIds.length})</p></div>
                <div className="flex-1 overflow-y-auto p-3 space-y-1.5">
                  {builderPkg.questionIds.length === 0 && <p className="text-center text-xs text-white/20 py-8">Belum ada soal</p>}
                  {builderPkg.questionIds.map((qId, i) => {
                    const q = questions.find(x => x.id === qId);
                    if (!q) return null;
                    return (
                      <div key={qId} className="flex items-center gap-2 p-2 rounded-lg bg-white/[0.03] border border-white/10">
                        <span className="text-[9px] text-white/20 w-4 shrink-0">{i + 1}</span>
                        <div className="flex-1 min-w-0"><p className="text-[9px] font-bold text-violet-400 font-mono">{q.code}</p><p className="text-[10px] text-white/50 truncate">{q.text.slice(0, 28)}...</p></div>
                        <button onClick={() => toggleQInPkg(qId)} className="p-1 text-white/20 hover:text-red-400 transition-colors"><X className="w-3 h-3" /></button>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
            <div className="flex gap-3 p-5 border-t border-white/10 shrink-0">
              <button onClick={() => setBuilderOpen(false)} className="flex-1 py-2.5 rounded-xl border border-white/15 text-sm text-white/60 hover:bg-white/5 transition-colors">Batal</button>
              <button onClick={saveBuilder} className="flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl bg-gradient-to-r from-violet-600 to-purple-600 font-bold text-sm hover:from-violet-500 hover:to-purple-500 transition-all"><Save className="w-4 h-4" />Simpan ({builderPkg.questionIds.length} soal)</button>
            </div>
          </div>
        </div>
      )}

      {/* ─── Delete confirms ─────────────────────────────────────────────────── */}
      {(qDeleteId || pDeleteId) && (
        <div className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-[60] p-4">
          <div className="bg-[#0f0d1a] border border-white/15 rounded-2xl p-6 w-full max-w-sm">
            <h3 className="font-black text-white mb-2">Hapus {qDeleteId ? 'Soal' : 'Paket'}?</h3>
            <p className="text-sm text-white/50 mb-6">Tindakan ini tidak bisa dibatalkan.</p>
            <div className="flex gap-3">
              <button onClick={() => { setQDeleteId(null); setPDeleteId(null); }} className="flex-1 py-2.5 rounded-xl border border-white/15 text-sm text-white/60 hover:text-white transition-colors">Batal</button>
              <button onClick={() => qDeleteId ? handleDeleteQ(qDeleteId) : handleDeleteP(pDeleteId!)} className="flex-1 py-2.5 rounded-xl bg-red-600 hover:bg-red-500 text-white text-sm font-bold transition-colors">Hapus</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
