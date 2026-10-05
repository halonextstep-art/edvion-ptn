import { useState, useMemo } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { toast } from 'sonner';
import {
  Search, Eye, CheckCircle2, XCircle, RotateCcw, Clock,
  ChevronRight, X, Save, AlertCircle, Filter, BookOpen,
  User, Calendar, Tag, BarChart3, FileText, CheckCheck,
  MessageSquare, Inbox, History, TrendingUp, Zap
} from 'lucide-react';

// ─── Types ────────────────────────────────────────────────────────────────────
type ReviewStatus = 'pending' | 'approved' | 'rejected' | 'revision';
type QuestionType = 'pg' | 'pgk' | 'bs' | 'menjodohkan' | 'isian' | 'uraian';
type TestCategory = 'SNBT' | 'TKA-IPA' | 'TKA-IPS' | 'AKM';
type Difficulty = 'Mudah' | 'Sedang' | 'Sulit';
type PanelView = 'queue' | 'history';

interface ReviewQuestion {
  id: string;
  code: string;
  category: TestCategory;
  subject: string;
  questionType: QuestionType;
  difficulty: Difficulty;
  stimulus?: string;
  text: string;
  options?: { key: string; text: string }[];
  correctAnswer: string;
  explanation: string;
  tags: string[];
  submittedBy: string;
  submittedAt: string;
  status: ReviewStatus;
  reviewNote?: string;
  reviewedAt?: string;
  reviewedBy?: string;
  revisionCount: number;
}

// ─── Mock data ────────────────────────────────────────────────────────────────
const SEED_QUESTIONS: ReviewQuestion[] = [
  {
    id: 'rq1', code: 'SNBT-PU-091', category: 'SNBT', subject: 'Penalaran Umum', questionType: 'pg', difficulty: 'Sedang',
    stimulus: 'Sebuah perusahaan memiliki 5 divisi: A, B, C, D, dan E. Diketahui bahwa divisi A menghasilkan lebih banyak produk dari B. Divisi C menghasilkan lebih sedikit dari A namun lebih banyak dari D. Divisi E menghasilkan paling sedikit dari semua divisi.',
    text: 'Urutan divisi dari yang menghasilkan produk paling banyak ke paling sedikit adalah...',
    options: [
      { key: 'A', text: 'A > B > C > D > E' },
      { key: 'B', text: 'A > C > B > D > E' },
      { key: 'C', text: 'B > A > C > D > E' },
      { key: 'D', text: 'A > B > D > C > E' },
      { key: 'E', text: 'C > A > B > D > E' },
    ],
    correctAnswer: 'A',
    explanation: 'Dari keterangan: A > B, A > C > D, E paling kecil. Belum ada info pasti posisi B vs C, tapi opsi A (A>B>C>D>E) konsisten dengan semua kondisi yang diberikan.',
    tags: ['logika', 'urutan', 'penalaran'],
    submittedBy: 'Reza Firmansyah', submittedAt: '2025-01-23 10:30', status: 'pending', revisionCount: 0,
  },
  {
    id: 'rq2', code: 'SNBT-PM-054', category: 'SNBT', subject: 'Penalaran Matematika', questionType: 'pg', difficulty: 'Sulit',
    text: 'Jika f(x) = 2x³ - 3x² + x - 5, maka f\'\'(2) adalah...',
    options: [
      { key: 'A', text: '18' },
      { key: 'B', text: '21' },
      { key: 'C', text: '24' },
      { key: 'D', text: '27' },
      { key: 'E', text: '30' },
    ],
    correctAnswer: 'B',
    explanation: 'f\'(x) = 6x² - 6x + 1, f\'\'(x) = 12x - 6. Substitusi x=2: f\'\'(2) = 24 - 6 = 18. *Catatan: jawaban B=21 perlu diverifikasi kembali.',
    tags: ['kalkulus', 'turunan', 'fungsi'],
    submittedBy: 'Sari Dewi', submittedAt: '2025-01-23 09:15', status: 'pending', revisionCount: 1,
  },
  {
    id: 'rq3', code: 'SNBT-PPU-022', category: 'SNBT', subject: 'Pemahaman Bacaan', questionType: 'pgk', difficulty: 'Mudah',
    stimulus: 'Bacalah teks berikut: "Pemanasan global merupakan fenomena meningkatnya suhu rata-rata atmosfer, laut, dan daratan bumi. Fenomena ini dipicu oleh emisi gas rumah kaca yang berlebihan, terutama CO₂ dari pembakaran bahan bakar fosil. Dampaknya meliputi pencairan es di kutub, kenaikan permukaan laut, dan perubahan pola cuaca ekstrem."',
    text: 'Berdasarkan teks tersebut, pernyataan mana SAJA yang benar?',
    options: [
      { key: 'A', text: 'Pemanasan global hanya mempengaruhi suhu atmosfer' },
      { key: 'B', text: 'CO₂ adalah salah satu penyebab utama pemanasan global' },
      { key: 'C', text: 'Kenaikan permukaan laut adalah dampak dari pemanasan global' },
      { key: 'D', text: 'Pembakaran bahan bakar fosil tidak berkaitan dengan pemanasan global' },
      { key: 'E', text: 'Pencairan es di kutub merupakan salah satu dampaknya' },
    ],
    correctAnswer: 'B,C,E',
    explanation: 'B, C, dan E benar sesuai teks. A salah karena pemanasan global mempengaruhi atmosfer, laut, dan daratan. D salah karena bertentangan langsung dengan teks.',
    tags: ['pemahaman bacaan', 'lingkungan', 'teks eksposisi'],
    submittedBy: 'Reza Firmansyah', submittedAt: '2025-01-22 16:45', status: 'pending', revisionCount: 0,
  },
  {
    id: 'rq4', code: 'TKA-IPA-033', category: 'TKA-IPA', subject: 'Fisika', questionType: 'pg', difficulty: 'Sulit',
    text: 'Sebuah benda bermassa 2 kg bergerak dengan kecepatan 10 m/s. Jika gaya gesekan sebesar 4 N bekerja berlawanan arah gerak, berapa jarak yang ditempuh benda hingga berhenti?',
    options: [
      { key: 'A', text: '20 m' },
      { key: 'B', text: '25 m' },
      { key: 'C', text: '30 m' },
      { key: 'D', text: '35 m' },
      { key: 'E', text: '40 m' },
    ],
    correctAnswer: 'B',
    explanation: 'Gunakan teorema kerja-energi: W = ΔEk. -f·d = 0 - ½mv². -4d = -½(2)(10²) = -100. d = 100/4 = 25 m.',
    tags: ['mekanika', 'kerja-energi', 'gerak'],
    submittedBy: 'Dian Pratiwi', submittedAt: '2025-01-22 14:20', status: 'pending', revisionCount: 0,
  },
  {
    id: 'rq5', code: 'AKM-LIT-011', category: 'AKM', subject: 'Literasi', questionType: 'isian', difficulty: 'Sedang',
    stimulus: 'Infografis menunjukkan data penjualan online 5 platform e-commerce di Indonesia Q4 2024: Tokopedia 32%, Shopee 41%, Lazada 14%, Bukalapak 8%, lainnya 5%.',
    text: 'Berdasarkan infografis tersebut, platform e-commerce mana yang memiliki pangsa pasar terbesar? Tuliskan nama platform dan persentasenya.',
    correctAnswer: 'Shopee, 41%',
    explanation: 'Berdasarkan data infografis, Shopee memiliki pangsa pasar terbesar yaitu 41%, diikuti Tokopedia 32%.',
    tags: ['literasi data', 'infografis', 'interpretasi'],
    submittedBy: 'Sari Dewi', submittedAt: '2025-01-21 11:00', status: 'pending', revisionCount: 0,
  },
  // History items
  {
    id: 'rq6', code: 'SNBT-PU-088', category: 'SNBT', subject: 'Penalaran Umum', questionType: 'pg', difficulty: 'Mudah',
    text: 'Semua burung bisa terbang. Penguin adalah burung. Kesimpulan yang tepat adalah...',
    options: [
      { key: 'A', text: 'Penguin bisa terbang' },
      { key: 'B', text: 'Penguin tidak bisa terbang' },
      { key: 'C', text: 'Semua burung adalah penguin' },
      { key: 'D', text: 'Beberapa burung tidak bisa terbang' },
      { key: 'E', text: 'Tidak dapat disimpulkan' },
    ],
    correctAnswer: 'A',
    explanation: 'Secara logika silogisme, jika semua burung bisa terbang dan penguin adalah burung, maka penguin bisa terbang. Meskipun faktanya salah, kesimpulan logisnya adalah A.',
    tags: ['silogisme', 'logika deduktif'],
    submittedBy: 'Reza Firmansyah', submittedAt: '2025-01-20 09:00', status: 'approved',
    reviewedAt: '2025-01-20 14:00', reviewedBy: 'Admin Pusat', revisionCount: 0,
  },
  {
    id: 'rq7', code: 'SNBT-PM-049', category: 'SNBT', subject: 'Penalaran Matematika', questionType: 'pg', difficulty: 'Sulit',
    text: 'Nilai dari ∫₀² (3x² - 2x + 1) dx adalah...',
    options: [
      { key: 'A', text: '6' }, { key: 'B', text: '7' }, { key: 'C', text: '8' },
      { key: 'D', text: '9' }, { key: 'E', text: '10' },
    ],
    correctAnswer: 'A',
    explanation: '∫(3x²-2x+1)dx = x³-x²+x. Evaluasi dari 0 ke 2: (8-4+2)-(0) = 6.',
    tags: ['integral', 'kalkulus'],
    submittedBy: 'Dian Pratiwi', submittedAt: '2025-01-19 15:30', status: 'rejected',
    reviewNote: 'Kunci jawaban salah. Hasil integral dari 0 ke 2 adalah 6, bukan yang tercantum. Mohon diperiksa kembali setiap opsi dan kunci jawabannya.',
    reviewedAt: '2025-01-19 17:00', reviewedBy: 'Admin Pusat', revisionCount: 0,
  },
  {
    id: 'rq8', code: 'TKA-IPS-018', category: 'TKA-IPS', subject: 'Ekonomi', questionType: 'pg', difficulty: 'Sedang',
    text: 'Kebijakan moneter yang dilakukan bank sentral untuk mengurangi inflasi adalah...',
    options: [
      { key: 'A', text: 'Menurunkan suku bunga acuan' },
      { key: 'B', text: 'Menaikkan suku bunga acuan' },
      { key: 'C', text: 'Mencetak uang baru' },
      { key: 'D', text: 'Mengurangi pajak' },
      { key: 'E', text: 'Meningkatkan pengeluaran pemerintah' },
    ],
    correctAnswer: 'B',
    explanation: 'Untuk mengurangi inflasi, bank sentral menaikkan suku bunga sehingga kredit lebih mahal, konsumsi turun, dan tekanan inflasi berkurang.',
    tags: ['ekonomi', 'kebijakan moneter', 'inflasi'],
    submittedBy: 'Sari Dewi', submittedAt: '2025-01-18 10:00', status: 'revision',
    reviewNote: 'Stimulus soal perlu ditambahkan agar konteks lebih jelas. Saat ini soal terlalu langsung tanpa konteks situasi ekonomi.',
    reviewedAt: '2025-01-18 13:00', reviewedBy: 'Admin Pusat', revisionCount: 1,
  },
];

// ─── Helpers ──────────────────────────────────────────────────────────────────
const QTYPE_LABELS: Record<QuestionType, string> = {
  pg: 'Pilihan Ganda', pgk: 'PG Kompleks', bs: 'Benar/Salah',
  menjodohkan: 'Menjodohkan', isian: 'Isian Singkat', uraian: 'Uraian',
};

const DIFF_CONFIG: Record<Difficulty, { color: string }> = {
  Mudah:  { color: 'bg-emerald-100 text-emerald-700' },
  Sedang: { color: 'bg-amber-100 text-amber-700' },
  Sulit:  { color: 'bg-red-100 text-red-700' },
};

const STATUS_CONFIG: Record<ReviewStatus, { label: string; color: string; dot: string; icon: React.ElementType }> = {
  pending:  { label: 'Menunggu Review', color: 'bg-amber-100 text-amber-700',    dot: 'bg-amber-500',  icon: Clock },
  approved: { label: 'Disetujui',       color: 'bg-emerald-100 text-emerald-700', dot: 'bg-emerald-500', icon: CheckCircle2 },
  rejected: { label: 'Ditolak',         color: 'bg-red-100 text-red-700',         dot: 'bg-red-500',    icon: XCircle },
  revision: { label: 'Perlu Revisi',    color: 'bg-blue-100 text-blue-700',       dot: 'bg-blue-500',   icon: RotateCcw },
};

const CAT_COLORS: Record<TestCategory, string> = {
  'SNBT':    'bg-violet-100 text-violet-700',
  'TKA-IPA': 'bg-blue-100 text-blue-700',
  'TKA-IPS': 'bg-teal-100 text-teal-700',
  'AKM':     'bg-orange-100 text-orange-700',
};

// ─── Detail Panel ─────────────────────────────────────────────────────────────
function QuestionDetailPanel({ question, onClose, onApprove, onReject, onRevision }: {
  question: ReviewQuestion;
  onClose: () => void;
  onApprove: (id: string) => void;
  onReject: (id: string, note: string) => void;
  onRevision: (id: string, note: string) => void;
}) {
  const [action, setAction] = useState<'reject' | 'revision' | null>(null);
  const [note, setNote] = useState('');

  const isPending = question.status === 'pending';

  const handleAction = () => {
    if (!note.trim()) { toast.error('Catatan wajib diisi'); return; }
    if (action === 'reject') onReject(question.id, note);
    if (action === 'revision') onRevision(question.id, note);
    setAction(null);
    setNote('');
  };

  return (
    <div className="fixed inset-0 z-50 flex">
      <div className="flex-1 bg-black/40" onClick={onClose} />
      <div className="w-full max-w-2xl bg-white shadow-2xl flex flex-col overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b shrink-0">
          <div>
            <div className="flex items-center gap-2 mb-0.5">
              <span className="font-mono text-sm font-bold text-indigo-600">{question.code}</span>
              <span className={`text-xs px-2 py-0.5 rounded-full font-semibold ${CAT_COLORS[question.category]}`}>{question.category}</span>
              <span className={`text-xs px-2 py-0.5 rounded-full font-semibold ${DIFF_CONFIG[question.difficulty].color}`}>{question.difficulty}</span>
            </div>
            <p className="text-sm text-muted-foreground">{question.subject} · {QTYPE_LABELS[question.questionType]}</p>
          </div>
          <button onClick={onClose} className="p-2 rounded-lg hover:bg-slate-100"><X className="w-5 h-5" /></button>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto p-6 space-y-5">
          {/* Meta */}
          <div className="flex items-center gap-4 text-xs text-muted-foreground">
            <span className="flex items-center gap-1"><User className="w-3.5 h-3.5" />{question.submittedBy}</span>
            <span className="flex items-center gap-1"><Calendar className="w-3.5 h-3.5" />{question.submittedAt}</span>
            {question.revisionCount > 0 && <span className="flex items-center gap-1 text-blue-600"><RotateCcw className="w-3.5 h-3.5" />Revisi ke-{question.revisionCount}</span>}
          </div>

          {/* Stimulus */}
          {question.stimulus && (
            <div className="p-4 bg-blue-50 border border-blue-200 rounded-xl">
              <p className="text-xs font-bold text-blue-600 uppercase tracking-wider mb-2">Stimulus / Konteks</p>
              <p className="text-sm text-slate-700 leading-relaxed">{question.stimulus}</p>
            </div>
          )}

          {/* Question */}
          <div className="p-4 bg-slate-50 rounded-xl border">
            <p className="text-xs font-bold text-slate-500 uppercase tracking-wider mb-2">Pertanyaan</p>
            <p className="text-sm text-slate-900 leading-relaxed font-medium">{question.text}</p>
          </div>

          {/* Options */}
          {question.options && (
            <div className="space-y-2">
              <p className="text-xs font-bold text-slate-500 uppercase tracking-wider">Pilihan Jawaban</p>
              {question.options.map(opt => (
                <div key={opt.key}
                  className={`flex items-start gap-3 p-3 rounded-xl border-2 ${opt.key === question.correctAnswer || question.correctAnswer.split(',').includes(opt.key) ? 'border-emerald-400 bg-emerald-50' : 'border-transparent bg-slate-50'}`}>
                  <span className={`w-7 h-7 rounded-lg flex items-center justify-center text-xs font-black shrink-0 ${opt.key === question.correctAnswer || question.correctAnswer.split(',').includes(opt.key) ? 'bg-emerald-500 text-white' : 'bg-slate-200 text-slate-600'}`}>{opt.key}</span>
                  <span className="text-sm mt-0.5">{opt.text}</span>
                  {(opt.key === question.correctAnswer || question.correctAnswer.split(',').includes(opt.key)) && (
                    <CheckCircle2 className="w-4 h-4 text-emerald-500 ml-auto shrink-0 mt-0.5" />
                  )}
                </div>
              ))}
            </div>
          )}

          {/* Correct answer for non-option types */}
          {!question.options && (
            <div className="p-3 bg-emerald-50 border border-emerald-200 rounded-xl">
              <p className="text-xs font-bold text-emerald-600 uppercase tracking-wider mb-1">Kunci Jawaban</p>
              <p className="text-sm font-mono text-slate-800">{question.correctAnswer}</p>
            </div>
          )}

          {/* Explanation */}
          <div className="p-4 bg-amber-50 border border-amber-200 rounded-xl">
            <p className="text-xs font-bold text-amber-700 uppercase tracking-wider mb-2">Pembahasan</p>
            <p className="text-sm text-slate-700 leading-relaxed">{question.explanation}</p>
          </div>

          {/* Tags */}
          {question.tags.length > 0 && (
            <div className="flex items-center gap-2 flex-wrap">
              <Tag className="w-3.5 h-3.5 text-slate-400" />
              {question.tags.map(t => <span key={t} className="text-xs bg-slate-100 text-slate-600 px-2 py-0.5 rounded-full">{t}</span>)}
            </div>
          )}

          {/* Previous review note */}
          {question.reviewNote && (
            <div className="p-4 bg-slate-50 border border-slate-200 rounded-xl">
              <p className="text-xs font-bold text-slate-500 uppercase tracking-wider mb-2 flex items-center gap-1"><MessageSquare className="w-3.5 h-3.5" />Catatan Review Sebelumnya</p>
              <p className="text-sm text-slate-700 italic">"{question.reviewNote}"</p>
              {question.reviewedBy && <p className="text-xs text-muted-foreground mt-1">— {question.reviewedBy}, {question.reviewedAt}</p>}
            </div>
          )}

          {/* Reject/Revision form */}
          {action && (
            <div className={`p-4 rounded-xl border-2 ${action === 'reject' ? 'border-red-300 bg-red-50' : 'border-blue-300 bg-blue-50'}`}>
              <p className={`text-xs font-bold uppercase tracking-wider mb-2 ${action === 'reject' ? 'text-red-600' : 'text-blue-600'}`}>
                {action === 'reject' ? '🚫 Alasan Penolakan' : '🔄 Catatan Revisi'}
              </p>
              <textarea
                rows={3}
                value={note}
                onChange={e => setNote(e.target.value)}
                placeholder={action === 'reject' ? 'Jelaskan mengapa soal ini ditolak...' : 'Jelaskan apa yang perlu direvisi...'}
                className="w-full px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 resize-none bg-white"
              />
              <div className="flex gap-2 mt-2">
                <Button variant="outline" size="sm" onClick={() => { setAction(null); setNote(''); }}>Batal</Button>
                <Button size="sm" className={action === 'reject' ? 'bg-red-600 hover:bg-red-700' : 'bg-blue-600 hover:bg-blue-700'} onClick={handleAction}>
                  <Save className="w-3.5 h-3.5 mr-1" />Kirim
                </Button>
              </div>
            </div>
          )}
        </div>

        {/* Footer actions */}
        {isPending && !action && (
          <div className="px-6 py-4 border-t bg-slate-50 shrink-0 flex gap-2">
            <Button variant="outline" className="flex-1 gap-2 text-blue-700 border-blue-300 hover:bg-blue-50" onClick={() => setAction('revision')}>
              <RotateCcw className="w-4 h-4" />Minta Revisi
            </Button>
            <Button variant="outline" className="flex-1 gap-2 text-red-600 border-red-300 hover:bg-red-50" onClick={() => setAction('reject')}>
              <XCircle className="w-4 h-4" />Tolak
            </Button>
            <Button className="flex-1 gap-2 bg-emerald-600 hover:bg-emerald-700" onClick={() => onApprove(question.id)}>
              <CheckCircle2 className="w-4 h-4" />Setujui
            </Button>
          </div>
        )}
        {!isPending && (
          <div className="px-6 py-4 border-t bg-slate-50 shrink-0">
            <div className={`flex items-center gap-2 text-sm font-semibold ${STATUS_CONFIG[question.status].color} px-3 py-2 rounded-lg`}>
              {(() => { const Icon = STATUS_CONFIG[question.status].icon; return <Icon className="w-4 h-4" />; })()}
              {STATUS_CONFIG[question.status].label}
              {question.reviewedAt && <span className="font-normal text-xs ml-auto opacity-70">{question.reviewedAt}</span>}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// ─── Main Component ───────────────────────────────────────────────────────────
export default function QuestionBank() {
  const [questions, setQuestions] = useState<ReviewQuestion[]>(SEED_QUESTIONS);
  const [panelView, setPanelView] = useState<PanelView>('queue');
  const [selected, setSelected] = useState<ReviewQuestion | null>(null);
  const [search, setSearch] = useState('');
  const [filterCat, setFilterCat] = useState<TestCategory | 'all'>('all');
  const [filterSubmitter, setFilterSubmitter] = useState<string>('all');

  // Stats
  const pending  = questions.filter(q => q.status === 'pending').length;
  const approved = questions.filter(q => q.status === 'approved').length;
  const rejected = questions.filter(q => q.status === 'rejected').length;
  const revision = questions.filter(q => q.status === 'revision').length;

  const submitters = useMemo(() => [...new Set(questions.map(q => q.submittedBy))], [questions]);

  const displayList = useMemo(() => {
    const base = panelView === 'queue'
      ? questions.filter(q => q.status === 'pending')
      : questions.filter(q => q.status !== 'pending');
    return base.filter(q => {
      const ms = !search || q.code.toLowerCase().includes(search.toLowerCase()) || q.text.toLowerCase().includes(search.toLowerCase()) || q.subject.toLowerCase().includes(search.toLowerCase());
      const mc = filterCat === 'all' || q.category === filterCat;
      const msub = filterSubmitter === 'all' || q.submittedBy === filterSubmitter;
      return ms && mc && msub;
    });
  }, [questions, panelView, search, filterCat, filterSubmitter]);

  const handleApprove = (id: string) => {
    const q = questions.find(q => q.id === id);
    setQuestions(prev => prev.map(q => q.id === id ? { ...q, status: 'approved', reviewedAt: new Date().toLocaleString('id'), reviewedBy: 'Admin Pusat' } : q));
    toast.success(`Soal ${q?.code} disetujui dan siap dipublish`);
    setSelected(null);
  };

  const handleReject = (id: string, note: string) => {
    const q = questions.find(q => q.id === id);
    setQuestions(prev => prev.map(q => q.id === id ? { ...q, status: 'rejected', reviewNote: note, reviewedAt: new Date().toLocaleString('id'), reviewedBy: 'Admin Pusat' } : q));
    toast.error(`Soal ${q?.code} ditolak. Notifikasi dikirim ke Tim Konten.`);
    setSelected(null);
  };

  const handleRevision = (id: string, note: string) => {
    const q = questions.find(q => q.id === id);
    setQuestions(prev => prev.map(q => q.id === id ? { ...q, status: 'revision', reviewNote: note, reviewedAt: new Date().toLocaleString('id'), reviewedBy: 'Admin Pusat' } : q));
    toast.info(`Soal ${q?.code} dikembalikan untuk revisi. Notifikasi dikirim ke ${q?.submittedBy}.`);
    setSelected(null);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-gradient-to-r from-slate-800 to-slate-900 rounded-2xl p-6 text-white relative overflow-hidden">
        <div className="absolute inset-0 opacity-5" style={{ backgroundImage: 'repeating-linear-gradient(45deg, white 0, white 1px, transparent 0, transparent 50%)', backgroundSize: '12px 12px' }} />
        <div className="relative flex items-start justify-between">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <FileText className="w-5 h-5 text-slate-300" />
              <span className="text-sm font-semibold text-slate-300">Governance Soal</span>
            </div>
            <h1 className="text-2xl font-black mb-1">Review & Approval Soal</h1>
            <p className="text-slate-400 text-sm">Tinjau dan setujui soal yang disubmit Tim Konten sebelum dipublikasikan</p>
          </div>
          <div className="hidden md:flex items-center gap-3">
            <div className="text-center bg-white/10 rounded-xl px-4 py-2.5">
              <p className="text-2xl font-black text-amber-400">{pending}</p>
              <p className="text-xs text-slate-400">Menunggu</p>
            </div>
            <div className="text-center bg-white/10 rounded-xl px-4 py-2.5">
              <p className="text-2xl font-black text-emerald-400">{approved}</p>
              <p className="text-xs text-slate-400">Disetujui</p>
            </div>
            <div className="text-center bg-white/10 rounded-xl px-4 py-2.5">
              <p className="text-2xl font-black text-blue-400">{revision}</p>
              <p className="text-xs text-slate-400">Revisi</p>
            </div>
            <div className="text-center bg-white/10 rounded-xl px-4 py-2.5">
              <p className="text-2xl font-black text-red-400">{rejected}</p>
              <p className="text-xs text-slate-400">Ditolak</p>
            </div>
          </div>
        </div>
      </div>

      {/* Urgent pending alert */}
      {pending > 0 && (
        <div className="flex items-center gap-3 p-4 bg-amber-50 border-2 border-amber-300 rounded-xl">
          <div className="w-9 h-9 rounded-xl bg-amber-400 flex items-center justify-center shrink-0">
            <Inbox className="w-5 h-5 text-white" />
          </div>
          <div className="flex-1">
            <p className="font-bold text-amber-900">{pending} soal menunggu review</p>
            <p className="text-sm text-amber-700">Soal paling lama: <strong>SNBT-PU-091</strong> — sudah 2 hari belum ditinjau</p>
          </div>
          <Button size="sm" className="bg-amber-500 hover:bg-amber-600 shrink-0" onClick={() => setPanelView('queue')}>
            Tinjau Sekarang <ChevronRight className="w-4 h-4 ml-1" />
          </Button>
        </div>
      )}

      {/* View toggle + filters */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center gap-3">
        <div className="flex items-center gap-1 p-1 bg-slate-100 rounded-xl shrink-0">
          <button onClick={() => setPanelView('queue')}
            className={`flex items-center gap-1.5 px-4 py-2 rounded-lg text-sm font-semibold transition-all ${panelView === 'queue' ? 'bg-white shadow-sm text-slate-900' : 'text-slate-500 hover:text-slate-700'}`}>
            <Inbox className="w-4 h-4" />Antrian Review
            {pending > 0 && <span className="ml-1 text-xs bg-amber-500 text-white px-1.5 py-0.5 rounded-full font-black">{pending}</span>}
          </button>
          <button onClick={() => setPanelView('history')}
            className={`flex items-center gap-1.5 px-4 py-2 rounded-lg text-sm font-semibold transition-all ${panelView === 'history' ? 'bg-white shadow-sm text-slate-900' : 'text-slate-500 hover:text-slate-700'}`}>
            <History className="w-4 h-4" />Riwayat
          </button>
        </div>

        <div className="flex gap-2 flex-1 flex-wrap">
          <div className="relative flex-1 min-w-48">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <input value={search} onChange={e => setSearch(e.target.value)} placeholder="Cari kode, soal, mata uji..." className="w-full pl-9 pr-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-400 bg-white" />
          </div>
          <select value={filterCat} onChange={e => setFilterCat(e.target.value as TestCategory | 'all')} className="px-3 py-2 border rounded-lg text-sm focus:outline-none bg-white">
            <option value="all">Semua Kategori</option>
            {(['SNBT', 'TKA-IPA', 'TKA-IPS', 'AKM'] as TestCategory[]).map(c => <option key={c} value={c}>{c}</option>)}
          </select>
          <select value={filterSubmitter} onChange={e => setFilterSubmitter(e.target.value)} className="px-3 py-2 border rounded-lg text-sm focus:outline-none bg-white">
            <option value="all">Semua Pembuat</option>
            {submitters.map(s => <option key={s} value={s}>{s}</option>)}
          </select>
        </div>
      </div>

      {/* Question list */}
      {displayList.length === 0 ? (
        <Card className="p-16 text-center">
          <CheckCheck className="w-12 h-12 text-emerald-400 mx-auto mb-3" />
          <h3 className="font-bold text-slate-700 mb-1">{panelView === 'queue' ? 'Antrian kosong!' : 'Tidak ada riwayat'}</h3>
          <p className="text-sm text-muted-foreground">{panelView === 'queue' ? 'Semua soal sudah ditinjau. Bagus!' : 'Belum ada soal yang pernah ditinjau.'}</p>
        </Card>
      ) : (
        <div className="space-y-2.5">
          {displayList.map(q => {
            const sc = STATUS_CONFIG[q.status];
            const StatusIcon = sc.icon;
            return (
              <Card key={q.id}
                onClick={() => setSelected(q)}
                className="p-4 cursor-pointer hover:shadow-md hover:border-indigo-300 transition-all group">
                <div className="flex items-start gap-4">
                  {/* Left: code + meta */}
                  <div className="shrink-0">
                    <p className="font-mono text-sm font-bold text-indigo-600">{q.code}</p>
                    <p className="text-xs text-muted-foreground mt-0.5">{q.submittedAt.split(' ')[0]}</p>
                  </div>

                  {/* Middle: content */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap mb-1.5">
                      <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${CAT_COLORS[q.category]}`}>{q.category}</span>
                      <span className="text-[10px] font-bold px-2 py-0.5 rounded-full bg-slate-100 text-slate-600">{q.subject}</span>
                      <span className="text-[10px] font-bold px-2 py-0.5 rounded-full bg-slate-100 text-slate-600">{QTYPE_LABELS[q.questionType]}</span>
                      <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${DIFF_CONFIG[q.difficulty].color}`}>{q.difficulty}</span>
                      {q.revisionCount > 0 && <span className="text-[10px] font-bold px-2 py-0.5 rounded-full bg-blue-100 text-blue-700 flex items-center gap-0.5"><RotateCcw className="w-2.5 h-2.5" />Revisi ke-{q.revisionCount}</span>}
                    </div>
                    <p className="text-sm text-slate-800 line-clamp-2">{q.text}</p>
                    <p className="text-xs text-muted-foreground mt-1 flex items-center gap-1"><User className="w-3 h-3" />{q.submittedBy}</p>
                  </div>

                  {/* Right: status + arrow */}
                  <div className="shrink-0 flex items-center gap-2">
                    <span className={`text-xs font-semibold px-2.5 py-1 rounded-full flex items-center gap-1 ${sc.color}`}>
                      <span className={`w-1.5 h-1.5 rounded-full ${sc.dot}`} />
                      {sc.label}
                    </span>
                    <ChevronRight className="w-4 h-4 text-slate-400 group-hover:text-indigo-500 transition-colors" />
                  </div>
                </div>

                {/* Review note (history) */}
                {q.reviewNote && panelView === 'history' && (
                  <div className="mt-3 pt-3 border-t flex items-start gap-2 text-xs text-slate-500">
                    <MessageSquare className="w-3.5 h-3.5 shrink-0 mt-0.5" />
                    <span className="italic line-clamp-1">"{q.reviewNote}"</span>
                  </div>
                )}
              </Card>
            );
          })}
        </div>
      )}

      {/* Detail panel */}
      {selected && (
        <QuestionDetailPanel
          question={selected}
          onClose={() => setSelected(null)}
          onApprove={handleApprove}
          onReject={handleReject}
          onRevision={handleRevision}
        />
      )}
    </div>
  );
}
