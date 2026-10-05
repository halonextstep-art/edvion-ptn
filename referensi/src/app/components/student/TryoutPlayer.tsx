import { useState, useEffect, useCallback, useRef } from 'react';
import {
  Clock, Flag, ChevronLeft, ChevronRight, CheckCircle, XCircle,
  AlertCircle, BarChart3, BookOpen, Send, RotateCcw, Home,
  Eye, ZoomIn, Award, TrendingUp, Target, Flame
} from 'lucide-react';

interface Question {
  id: string;
  subject: string;
  topic: string;
  type: 'multiple_choice' | 'complex_multiple' | 'short_answer';
  stimulus?: string;
  question: string;
  options?: string[];
  correctAnswer: string | string[];
  explanation: string;
  difficulty: 'easy' | 'medium' | 'hard';
}

export interface SessionResult {
  sessionTitle: string;
  score: number;
  accuracy: number;
  correct: number;
  wrong: number;
  total: number;
  timeUsed: number;
}

interface TryoutPlayerProps {
  sessionTitle: string;
  sessionType: 'tryout' | 'drilling' | 'mini';
  questions: Question[];
  durationMinutes: number;
  onFinish: (result?: SessionResult) => void;
}

type Phase = 'intro' | 'quiz' | 'results' | 'review';

const sampleQuestions: Question[] = [
  {
    id: 'Q001', subject: 'Penalaran Matematika', topic: 'Aljabar', type: 'multiple_choice', difficulty: 'medium',
    stimulus: 'Sebuah startup teknologi mencatat pendapatan bulanan (dalam juta rupiah) mengikuti pola tertentu.',
    question: 'Jika pendapatan pada bulan ke-t dinyatakan dengan P(t) = -2t² + 12t + 10, pada bulan ke berapa perusahaan mencapai pendapatan maksimum?',
    options: ['Bulan ke-2', 'Bulan ke-3', 'Bulan ke-4', 'Bulan ke-5', 'Bulan ke-6'],
    correctAnswer: 'Bulan ke-3',
    explanation: 'Pendapatan maksimum dicapai pada puncak parabola: t = -b/2a = -12/(2×-2) = 3. Jadi pendapatan maksimum pada bulan ke-3.'
  },
  {
    id: 'Q002', subject: 'Penalaran Umum', topic: 'Logika', type: 'multiple_choice', difficulty: 'easy',
    stimulus: 'Di sebuah kompetisi sains, ada 5 peserta: Andi, Beni, Cici, Dedi, dan Elly. Diketahui: Andi lebih tinggi dari Beni. Cici lebih tinggi dari Andi. Dedi lebih pendek dari Beni. Elly lebih tinggi dari Cici.',
    question: 'Urutan peserta dari yang tertinggi ke terendah adalah...',
    options: [
      'Elly > Cici > Andi > Beni > Dedi',
      'Cici > Elly > Andi > Beni > Dedi',
      'Elly > Cici > Beni > Andi > Dedi',
      'Elly > Andi > Cici > Beni > Dedi',
      'Cici > Elly > Beni > Andi > Dedi'
    ],
    correctAnswer: 'Elly > Cici > Andi > Beni > Dedi',
    explanation: 'Dari keterangan: Elly > Cici > Andi > Beni > Dedi. Urutkan dari yang terbesar: Elly, Cici, Andi, Beni, Dedi.'
  },
  {
    id: 'Q003', subject: 'Penalaran Matematika', topic: 'Trigonometri', type: 'multiple_choice', difficulty: 'hard',
    question: 'Jika sin x + cos x = 1/2, maka nilai sin x · cos x adalah...',
    options: ['-3/8', '-1/4', '1/8', '1/4', '3/8'],
    correctAnswer: '-3/8',
    explanation: 'Kuadratkan: (sin x + cos x)² = 1/4 → sin²x + 2 sin x cos x + cos²x = 1/4 → 1 + 2 sin x cos x = 1/4 → sin x cos x = -3/8.'
  },
  {
    id: 'Q004', subject: 'Pemahaman Bacaan', topic: 'Ide Pokok', type: 'multiple_choice', difficulty: 'easy',
    stimulus: 'Perubahan iklim merupakan ancaman nyata bagi ekosistem laut. Pemanasan global menyebabkan suhu permukaan laut meningkat, memicu pemutihan karang secara massal. Terumbu karang yang memutih kehilangan alga simbiotik, sumber utama nutrisinya. Tanpa intervensi manusia, lebih dari 70% terumbu karang dunia diprediksi mengalami kerusakan permanen pada akhir abad ini.',
    question: 'Ide pokok paragraf di atas adalah...',
    options: [
      'Terumbu karang kehilangan alga simbiotik akibat pemutihan',
      'Perubahan iklim mengancam ekosistem laut terutama terumbu karang',
      'Suhu permukaan laut meningkat karena pemanasan global',
      'Lebih dari 70% terumbu karang akan rusak permanen',
      'Intervensi manusia diperlukan untuk menyelamatkan terumbu karang'
    ],
    correctAnswer: 'Perubahan iklim mengancam ekosistem laut terutama terumbu karang',
    explanation: 'Ide pokok terletak pada kalimat pertama yang menjadi topik utama paragraf, yaitu perubahan iklim sebagai ancaman bagi ekosistem laut.'
  },
  {
    id: 'Q005', subject: 'Penalaran Umum', topic: 'Pola Bilangan', type: 'multiple_choice', difficulty: 'medium',
    question: 'Bilangan berikutnya dalam pola: 2, 6, 12, 20, 30, ... adalah...',
    options: ['38', '40', '42', '44', '46'],
    correctAnswer: '42',
    explanation: 'Selisih antar suku: 4, 6, 8, 10, 12 (bertambah 2 setiap suku). Maka suku berikutnya: 30 + 12 = 42.'
  },
  {
    id: 'Q006', subject: 'Pemahaman Bacaan', topic: 'Kalimat Pendukung', type: 'multiple_choice', difficulty: 'medium',
    stimulus: 'Digitalisasi UMKM di Indonesia mengalami percepatan signifikan pasca pandemi. Platform e-commerce melaporkan penambahan lebih dari 2 juta penjual baru dalam setahun. Pemerintah pun meluncurkan berbagai program literasi digital untuk mendorong UMKM masuk ekosistem digital. Namun, tantangan infrastruktur internet di daerah 3T masih menjadi hambatan utama.',
    question: 'Pernyataan yang TIDAK mendukung gagasan utama paragraf adalah...',
    options: [
      'E-commerce menambah 2 juta penjual baru dalam setahun',
      'Pemerintah meluncurkan program literasi digital untuk UMKM',
      'Pandemi mempercepat digitalisasi UMKM di Indonesia',
      'Infrastruktur di daerah 3T masih menjadi hambatan',
      'UMKM digital mampu bersaing di pasar internasional'
    ],
    correctAnswer: 'UMKM digital mampu bersaing di pasar internasional',
    explanation: 'Paragraf tidak menyebutkan daya saing internasional UMKM. Semua pilihan lain secara eksplisit disebutkan dalam paragraf.'
  },
  {
    id: 'Q007', subject: 'Penalaran Matematika', topic: 'Statistika', type: 'multiple_choice', difficulty: 'medium',
    question: 'Rata-rata nilai ulangan 8 siswa adalah 75. Jika ditambah nilai 2 siswa baru, rata-rata menjadi 73. Berapakah jumlah nilai 2 siswa baru tersebut?',
    options: ['120', '125', '130', '135', '140'],
    correctAnswer: '130',
    explanation: 'Total nilai 8 siswa = 8 × 75 = 600. Total nilai 10 siswa = 10 × 73 = 730. Jumlah nilai 2 siswa baru = 730 − 600 = 130. Jawaban: 130.'
  },
  {
    id: 'Q008', subject: 'Penalaran Umum', topic: 'Analogi', type: 'multiple_choice', difficulty: 'easy',
    question: 'DOKTER : RUMAH SAKIT = GURU : ...',
    options: ['Siswa', 'Buku', 'Sekolah', 'Pelajaran', 'Perpustakaan'],
    correctAnswer: 'Sekolah',
    explanation: 'Dokter bekerja di Rumah Sakit, demikian pula Guru bekerja di Sekolah. Ini adalah analogi hubungan profesi dengan tempat kerja.'
  },
  {
    id: 'Q009', subject: 'Penalaran Matematika', topic: 'Peluang', type: 'multiple_choice', difficulty: 'hard',
    question: 'Dalam sebuah tas terdapat 4 bola merah dan 6 bola biru. Jika diambil 2 bola secara acak tanpa pengembalian, peluang keduanya berwarna berbeda adalah...',
    options: ['8/15', '4/15', '6/15', '12/15', '2/5'],
    correctAnswer: '8/15',
    explanation: 'P(beda warna) = P(merah lalu biru) + P(biru lalu merah) = (4/10)(6/9) + (6/10)(4/9) = 24/90 + 24/90 = 48/90 = 8/15.'
  },
  {
    id: 'Q010', subject: 'Pemahaman Bacaan', topic: 'Makna Kata', type: 'multiple_choice', difficulty: 'easy',
    stimulus: 'Perkembangan kecerdasan buatan telah merevolusi berbagai sektor industri. Inovasi ini bukan sekadar disruptif, melainkan transformatif dalam mengubah cara manusia bekerja dan berinteraksi.',
    question: 'Makna kata "disruptif" dalam konteks bacaan di atas adalah...',
    options: [
      'Merusak dan menghancurkan sistem yang ada',
      'Mengganggu atau mengubah secara mendasar tatanan yang ada',
      'Menciptakan konflik di antara pelaku industri',
      'Memperlambat proses transformasi digital',
      'Memperburuk kondisi tenaga kerja'
    ],
    correctAnswer: 'Mengganggu atau mengubah secara mendasar tatanan yang ada',
    explanation: '"Disruptif" dalam konteks teknologi berarti mengubah atau mengganggu tatanan yang sudah ada secara mendasar, bukan sekadar merusak.'
  },
];

function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

function difficultyColor(d: string) {
  if (d === 'easy') return 'text-emerald-600 bg-emerald-50';
  if (d === 'medium') return 'text-amber-600 bg-amber-50';
  return 'text-red-600 bg-red-50';
}

function difficultyLabel(d: string) {
  if (d === 'easy') return 'Mudah';
  if (d === 'medium') return 'Sedang';
  return 'Sulit';
}

export default function TryoutPlayer({ sessionTitle, sessionType, questions: propQuestions, durationMinutes, onFinish }: TryoutPlayerProps) {
  const questions = propQuestions.length > 0 ? propQuestions : sampleQuestions;
  const totalSeconds = durationMinutes * 60;

  const [phase, setPhase] = useState<Phase>('intro');
  const [current, setCurrent] = useState(0);
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [flagged, setFlagged] = useState<Set<string>>(new Set());
  const [timeLeft, setTimeLeft] = useState(totalSeconds);
  const [startTime, setStartTime] = useState<number>(0);
  const [reviewIndex, setReviewIndex] = useState(0);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const handleSubmit = useCallback(() => {
    if (timerRef.current) clearInterval(timerRef.current);
    setPhase('results');
  }, []);

  useEffect(() => {
    if (phase === 'quiz') {
      timerRef.current = setInterval(() => {
        setTimeLeft(prev => {
          if (prev <= 1) { handleSubmit(); return 0; }
          return prev - 1;
        });
      }, 1000);
    }
    return () => { if (timerRef.current) clearInterval(timerRef.current); };
  }, [phase, handleSubmit]);

  const startQuiz = () => {
    setStartTime(Date.now());
    setPhase('quiz');
  };

  const selectAnswer = (qId: string, answer: string) => {
    setAnswers(prev => ({ ...prev, [qId]: answer }));
  };

  const toggleFlag = (qId: string) => {
    setFlagged(prev => {
      const next = new Set(prev);
      next.has(qId) ? next.delete(qId) : next.add(qId);
      return next;
    });
  };

  const getResult = (q: Question) => {
    const ans = answers[q.id];
    if (!ans) return 'unanswered';
    const correct = Array.isArray(q.correctAnswer) ? q.correctAnswer[0] : q.correctAnswer;
    return ans === correct ? 'correct' : 'wrong';
  };

  const results = questions.map(q => ({ q, result: getResult(q) }));
  const correctCount = results.filter(r => r.result === 'correct').length;
  const wrongCount = results.filter(r => r.result === 'wrong').length;
  const unansweredCount = results.filter(r => r.result === 'unanswered').length;
  const score = Math.round((correctCount / questions.length) * 1000);
  const accuracy = questions.length > 0 ? Math.round((correctCount / (correctCount + wrongCount || 1)) * 100) : 0;
  const timeUsed = totalSeconds - timeLeft;

  const subjectBreakdown = questions.reduce<Record<string, { correct: number; total: number }>>((acc, q) => {
    if (!acc[q.subject]) acc[q.subject] = { correct: 0, total: 0 };
    acc[q.subject].total++;
    if (getResult(q) === 'correct') acc[q.subject].correct++;
    return acc;
  }, {});

  const q = questions[current];
  const answered = Object.keys(answers).length;

  // ─── INTRO ───
  if (phase === 'intro') {
    return (
      <div className="min-h-screen bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 flex items-center justify-center p-4">
        <div className="bg-white rounded-3xl shadow-2xl max-w-lg w-full p-10 text-center">
          <div className="w-20 h-20 rounded-2xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center mx-auto mb-6 shadow-xl">
            <Target className="w-10 h-10 text-white" />
          </div>
          <div className="inline-block px-3 py-1 rounded-full bg-indigo-100 text-indigo-700 text-xs font-bold mb-3 uppercase tracking-wider">
            {sessionType === 'tryout' ? 'Tryout Simulasi' : sessionType === 'mini' ? 'Mini Tryout' : 'Drilling'}
          </div>
          <h1 className="text-2xl font-black text-slate-900 mb-2">{sessionTitle}</h1>
          <p className="text-slate-500 mb-8 text-sm">Pastikan kamu siap sebelum memulai. Waktu berjalan setelah tombol Mulai ditekan.</p>

          <div className="grid grid-cols-3 gap-4 mb-8">
            {[
              { icon: BookOpen, label: 'Soal', val: questions.length.toString() },
              { icon: Clock, label: 'Waktu', val: `${durationMinutes}m` },
              { icon: Target, label: 'Subtes', val: [...new Set(questions.map(q => q.subject))].length.toString() },
            ].map(({ icon: Icon, label, val }) => (
              <div key={label} className="bg-slate-50 rounded-xl p-4">
                <Icon className="w-5 h-5 text-indigo-500 mx-auto mb-2" />
                <p className="text-2xl font-black text-slate-900">{val}</p>
                <p className="text-xs text-slate-500 mt-0.5">{label}</p>
              </div>
            ))}
          </div>

          <ul className="text-left space-y-2 mb-8 bg-amber-50 border border-amber-200 rounded-xl p-4">
            {[
              'Waktu berjalan terus, tidak bisa di-pause',
              'Navigasi bebas antar soal kapan saja',
              'Tandai soal yang ragu dengan 🚩 Flag',
              'Pastikan semua soal terjawab sebelum submit',
            ].map(tip => (
              <li key={tip} className="flex items-start gap-2 text-sm text-amber-800">
                <AlertCircle className="w-4 h-4 mt-0.5 shrink-0 text-amber-500" />
                {tip}
              </li>
            ))}
          </ul>

          <button
            onClick={startQuiz}
            className="w-full py-4 rounded-2xl bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white font-black text-lg shadow-lg hover:shadow-indigo-300 transition-all hover:-translate-y-0.5"
          >
            Mulai Sekarang →
          </button>
          <button onClick={onFinish} className="mt-3 text-sm text-slate-400 hover:text-slate-600 transition-colors">
            Kembali
          </button>
        </div>
      </div>
    );
  }

  // ─── QUIZ ───
  if (phase === 'quiz') {
    const isLast = current === questions.length - 1;
    const timePercent = (timeLeft / totalSeconds) * 100;
    const timerDanger = timeLeft < 300;

    return (
      <div className="min-h-screen bg-slate-50 flex flex-col">
        {/* Top bar */}
        <div className="bg-white border-b sticky top-0 z-50 shadow-sm">
          <div className="max-w-6xl mx-auto px-4 py-3 flex items-center gap-4">
            <div className="flex-1">
              <p className="text-xs text-slate-500 font-medium mb-1">{sessionTitle}</p>
              <div className="flex items-center gap-2">
                <div className="flex-1 h-1.5 bg-slate-100 rounded-full overflow-hidden">
                  <div className="h-full bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full transition-all" style={{ width: `${(answered / questions.length) * 100}%` }} />
                </div>
                <span className="text-xs text-slate-500 font-mono shrink-0">{answered}/{questions.length}</span>
              </div>
            </div>

            <div className={`flex items-center gap-2 px-4 py-2 rounded-xl font-mono font-bold text-lg transition-colors ${timerDanger ? 'bg-red-50 text-red-600 border border-red-200 animate-pulse' : 'bg-indigo-50 text-indigo-700'}`}>
              <Clock className="w-5 h-5" />
              {formatTime(timeLeft)}
            </div>

            <button
              onClick={handleSubmit}
              className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white font-bold text-sm transition-all"
            >
              <Send className="w-4 h-4" /> Submit
            </button>
          </div>
        </div>

        <div className="flex-1 max-w-6xl mx-auto w-full px-4 py-6 grid lg:grid-cols-[260px_1fr] gap-6">
          {/* Navigation sidebar */}
          <div className="hidden lg:block">
            <div className="bg-white rounded-2xl border p-4 sticky top-[76px]">
              <p className="text-xs font-bold text-slate-500 mb-3 uppercase tracking-wider">Navigasi Soal</p>
              <div className="grid grid-cols-5 gap-1.5 mb-4">
                {questions.map((qq, i) => {
                  const res = answers[qq.id] ? 'answered' : 'unanswered';
                  const isFlagged = flagged.has(qq.id);
                  const isCurrent = i === current;
                  return (
                    <button
                      key={qq.id}
                      onClick={() => setCurrent(i)}
                      className={`w-9 h-9 rounded-lg text-xs font-bold transition-all relative ${
                        isCurrent ? 'bg-indigo-600 text-white shadow-md shadow-indigo-300' :
                        res === 'answered' ? 'bg-emerald-100 text-emerald-700 border border-emerald-300' :
                        'bg-slate-100 text-slate-500 hover:bg-slate-200'
                      }`}
                    >
                      {i + 1}
                      {isFlagged && <span className="absolute -top-1 -right-1 text-[8px]">🚩</span>}
                    </button>
                  );
                })}
              </div>
              <div className="space-y-2 text-xs">
                {[
                  { color: 'bg-indigo-600', label: `Soal ini (${current + 1})` },
                  { color: 'bg-emerald-100 border border-emerald-300', label: `Dijawab (${answered})` },
                  { color: 'bg-slate-100', label: `Belum dijawab (${questions.length - answered})` },
                ].map(({ color, label }) => (
                  <div key={label} className="flex items-center gap-2">
                    <div className={`w-4 h-4 rounded ${color}`} />
                    <span className="text-slate-500">{label}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Question panel */}
          <div className="bg-white rounded-2xl border shadow-sm overflow-hidden">
            {/* Question header */}
            <div className="px-6 py-4 border-b bg-gradient-to-r from-slate-50 to-indigo-50 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <span className="text-xs font-bold text-indigo-600 bg-indigo-100 px-2.5 py-1 rounded-lg">Soal {current + 1} / {questions.length}</span>
                <span className="text-xs text-slate-500">{q.subject}</span>
                <span className={`text-xs font-semibold px-2 py-0.5 rounded ${difficultyColor(q.difficulty)}`}>{difficultyLabel(q.difficulty)}</span>
              </div>
              <button
                onClick={() => toggleFlag(q.id)}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-colors ${flagged.has(q.id) ? 'bg-amber-100 text-amber-700 border border-amber-300' : 'bg-slate-100 text-slate-500 hover:bg-amber-50 hover:text-amber-600'}`}
              >
                <Flag className="w-3.5 h-3.5" />
                {flagged.has(q.id) ? 'Ditandai' : 'Tandai'}
              </button>
            </div>

            <div className="p-6">
              {/* Stimulus */}
              {q.stimulus && (
                <div className="mb-5 p-4 bg-blue-50 border border-blue-200 rounded-xl">
                  <p className="text-xs font-bold text-blue-600 mb-2 uppercase tracking-wider">Bacaan / Stimulus</p>
                  <p className="text-sm text-slate-700 leading-relaxed">{q.stimulus}</p>
                </div>
              )}

              {/* Question text */}
              <p className="text-base font-semibold text-slate-900 leading-relaxed mb-6">{q.question}</p>

              {/* Options */}
              {q.options && q.type !== 'short_answer' && (
                <div className="space-y-3">
                  {q.options.map((opt, idx) => {
                    const labels = ['A', 'B', 'C', 'D', 'E'];
                    const selected = answers[q.id] === opt;
                    return (
                      <button
                        key={opt}
                        onClick={() => selectAnswer(q.id, opt)}
                        className={`w-full text-left flex items-start gap-3 p-4 rounded-xl border-2 transition-all hover:-translate-y-0.5 ${
                          selected
                            ? 'border-indigo-500 bg-indigo-50 shadow-md shadow-indigo-100'
                            : 'border-slate-200 hover:border-indigo-300 hover:bg-indigo-50/50'
                        }`}
                      >
                        <span className={`w-7 h-7 rounded-lg flex items-center justify-center text-xs font-black shrink-0 mt-0.5 transition-colors ${
                          selected ? 'bg-indigo-600 text-white' : 'bg-slate-100 text-slate-600'
                        }`}>
                          {labels[idx]}
                        </span>
                        <span className={`text-sm leading-relaxed ${selected ? 'text-indigo-900 font-medium' : 'text-slate-700'}`}>{opt}</span>
                      </button>
                    );
                  })}
                </div>
              )}

              {q.type === 'short_answer' && (
                <input
                  value={answers[q.id] || ''}
                  onChange={e => selectAnswer(q.id, e.target.value)}
                  placeholder="Ketik jawaban kamu di sini..."
                  className="w-full px-4 py-3 border-2 border-slate-200 rounded-xl focus:border-indigo-500 focus:outline-none text-slate-900 font-medium"
                />
              )}
            </div>

            {/* Navigation footer */}
            <div className="px-6 py-4 border-t bg-slate-50 flex items-center justify-between">
              <button
                onClick={() => setCurrent(Math.max(0, current - 1))}
                disabled={current === 0}
                className="flex items-center gap-2 px-4 py-2 rounded-xl border border-slate-200 text-sm font-semibold text-slate-600 hover:bg-white disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
              >
                <ChevronLeft className="w-4 h-4" /> Sebelumnya
              </button>

              {/* Mobile nav dots */}
              <div className="flex lg:hidden items-center gap-1">
                {questions.slice(Math.max(0, current - 2), Math.min(questions.length, current + 3)).map((_, i) => {
                  const idx = Math.max(0, current - 2) + i;
                  return (
                    <button key={idx} onClick={() => setCurrent(idx)} className={`w-2 h-2 rounded-full transition-colors ${idx === current ? 'bg-indigo-600 w-4' : 'bg-slate-300'}`} />
                  );
                })}
              </div>

              {isLast ? (
                <button
                  onClick={handleSubmit}
                  className="flex items-center gap-2 px-5 py-2 rounded-xl bg-gradient-to-r from-indigo-600 to-purple-600 text-white font-bold text-sm hover:from-indigo-500 hover:to-purple-500 transition-all shadow-md"
                >
                  <Send className="w-4 h-4" /> Submit Semua
                </button>
              ) : (
                <button
                  onClick={() => setCurrent(Math.min(questions.length - 1, current + 1))}
                  className="flex items-center gap-2 px-4 py-2 rounded-xl bg-indigo-600 text-white text-sm font-semibold hover:bg-indigo-700 transition-colors"
                >
                  Selanjutnya <ChevronRight className="w-4 h-4" />
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    );
  }

  // ─── RESULTS ───
  if (phase === 'results') {
    const grade = score >= 700 ? 'A' : score >= 600 ? 'B' : score >= 500 ? 'C' : 'D';
    const gradeColor = grade === 'A' ? 'text-emerald-600' : grade === 'B' ? 'text-blue-600' : grade === 'C' ? 'text-amber-600' : 'text-red-600';

    return (
      <div className="min-h-screen bg-gradient-to-br from-indigo-50 via-purple-50 to-pink-50 py-10 px-4">
        <div className="max-w-3xl mx-auto">
          {/* Score card */}
          <div className="bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 rounded-3xl p-8 text-white mb-6 shadow-2xl relative overflow-hidden">
            <div className="absolute inset-0 opacity-10" style={{ backgroundImage: 'linear-gradient(rgba(255,255,255,.3) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,.3) 1px, transparent 1px)', backgroundSize: '40px 40px' }} />
            <div className="relative">
              <div className="flex items-center justify-between mb-6">
                <div>
                  <p className="text-indigo-200 text-sm font-medium">Hasil {sessionTitle}</p>
                  <h2 className="text-2xl font-black mt-0.5">Tryout Selesai! 🎉</h2>
                </div>
                <div className="text-center">
                  <p className={`text-6xl font-black ${gradeColor} bg-white/20 w-20 h-20 rounded-2xl flex items-center justify-center`}>{grade}</p>
                </div>
              </div>
              <div className="grid grid-cols-4 gap-4">
                {[
                  { label: 'Skor UTBK', val: score.toString(), icon: Award },
                  { label: 'Akurasi', val: `${accuracy}%`, icon: Target },
                  { label: 'Benar', val: correctCount.toString(), icon: CheckCircle },
                  { label: 'Waktu', val: formatTime(timeUsed), icon: Clock },
                ].map(({ label, val, icon: Icon }) => (
                  <div key={label} className="bg-white/15 backdrop-blur-sm rounded-xl p-3 text-center">
                    <Icon className="w-4 h-4 mx-auto mb-1.5 text-indigo-200" />
                    <p className="text-xl font-black">{val}</p>
                    <p className="text-xs text-indigo-200 mt-0.5">{label}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Breakdown */}
          <div className="bg-white rounded-2xl border p-6 mb-5 shadow-sm">
            <h3 className="font-bold text-slate-800 mb-4 flex items-center gap-2"><BarChart3 className="w-5 h-5 text-indigo-500" /> Nilai per Subtes</h3>
            <div className="space-y-3">
              {Object.entries(subjectBreakdown).map(([subject, { correct, total }]) => {
                const pct = Math.round((correct / total) * 100);
                return (
                  <div key={subject}>
                    <div className="flex justify-between text-sm mb-1.5">
                      <span className="font-medium text-slate-700">{subject}</span>
                      <span className={`font-bold ${pct >= 75 ? 'text-emerald-600' : pct >= 55 ? 'text-amber-600' : 'text-red-500'}`}>{correct}/{total} ({pct}%)</span>
                    </div>
                    <div className="h-2.5 bg-slate-100 rounded-full overflow-hidden">
                      <div
                        className={`h-full rounded-full transition-all ${pct >= 75 ? 'bg-gradient-to-r from-emerald-500 to-green-400' : pct >= 55 ? 'bg-gradient-to-r from-amber-500 to-yellow-400' : 'bg-gradient-to-r from-red-500 to-rose-400'}`}
                        style={{ width: `${pct}%` }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Summary counts */}
          <div className="grid grid-cols-3 gap-4 mb-6">
            {[
              { icon: CheckCircle, label: 'Benar', val: correctCount, color: 'text-emerald-600 bg-emerald-50 border-emerald-200' },
              { icon: XCircle, label: 'Salah', val: wrongCount, color: 'text-red-500 bg-red-50 border-red-200' },
              { icon: AlertCircle, label: 'Tidak Dijawab', val: unansweredCount, color: 'text-slate-500 bg-slate-50 border-slate-200' },
            ].map(({ icon: Icon, label, val, color }) => (
              <div key={label} className={`border rounded-2xl p-4 text-center ${color}`}>
                <Icon className="w-6 h-6 mx-auto mb-2" />
                <p className="text-3xl font-black">{val}</p>
                <p className="text-sm font-medium mt-0.5">{label}</p>
              </div>
            ))}
          </div>

          {/* Action buttons */}
          <div className="flex gap-3">
            <button
              onClick={() => { setReviewIndex(0); setPhase('review'); }}
              className="flex-1 flex items-center justify-center gap-2 py-3.5 rounded-xl border-2 border-indigo-300 text-indigo-700 font-bold hover:bg-indigo-50 transition-colors"
            >
              <Eye className="w-5 h-5" /> Review Pembahasan
            </button>
            <button
              onClick={() => onFinish({ sessionTitle, score, accuracy, correct: correctCount, wrong: wrongCount, total: questions.length, timeUsed })}
              className="flex-1 flex items-center justify-center gap-2 py-3.5 rounded-xl bg-gradient-to-r from-indigo-600 to-purple-600 text-white font-bold hover:from-indigo-500 hover:to-purple-500 transition-all shadow-lg"
            >
              <Home className="w-5 h-5" /> Kembali ke Beranda
            </button>
          </div>
        </div>
      </div>
    );
  }

  // ─── REVIEW ───
  if (phase === 'review') {
    const rq = questions[reviewIndex];
    const rResult = getResult(rq);
    const userAnswer = answers[rq.id];
    const correctAns = Array.isArray(rq.correctAnswer) ? rq.correctAnswer[0] : rq.correctAnswer;

    return (
      <div className="min-h-screen bg-slate-50 py-6 px-4">
        <div className="max-w-2xl mx-auto">
          {/* Header */}
          <div className="flex items-center justify-between mb-5">
            <button onClick={() => setPhase('results')} className="flex items-center gap-2 text-slate-600 hover:text-slate-900 font-medium text-sm">
              <ChevronLeft className="w-4 h-4" /> Kembali ke Hasil
            </button>
            <span className="text-sm text-slate-500 font-medium">Soal {reviewIndex + 1} dari {questions.length}</span>
          </div>

          {/* Question navigation strip */}
          <div className="flex gap-1.5 mb-5 overflow-x-auto pb-1">
            {questions.map((qq, i) => {
              const r = getResult(qq);
              return (
                <button
                  key={qq.id}
                  onClick={() => setReviewIndex(i)}
                  className={`w-8 h-8 rounded-lg text-xs font-bold shrink-0 transition-colors ${
                    i === reviewIndex ? 'bg-indigo-600 text-white' :
                    r === 'correct' ? 'bg-emerald-100 text-emerald-700' :
                    r === 'wrong' ? 'bg-red-100 text-red-600' :
                    'bg-slate-200 text-slate-500'
                  }`}
                >
                  {i + 1}
                </button>
              );
            })}
          </div>

          {/* Question card */}
          <div className="bg-white rounded-2xl border shadow-sm overflow-hidden mb-4">
            <div className={`px-6 py-4 border-b flex items-center gap-3 ${rResult === 'correct' ? 'bg-emerald-50' : rResult === 'wrong' ? 'bg-red-50' : 'bg-slate-50'}`}>
              {rResult === 'correct' && <CheckCircle className="w-5 h-5 text-emerald-600" />}
              {rResult === 'wrong' && <XCircle className="w-5 h-5 text-red-500" />}
              {rResult === 'unanswered' && <AlertCircle className="w-5 h-5 text-slate-400" />}
              <span className={`text-sm font-bold ${rResult === 'correct' ? 'text-emerald-700' : rResult === 'wrong' ? 'text-red-600' : 'text-slate-500'}`}>
                {rResult === 'correct' ? 'Jawaban Benar!' : rResult === 'wrong' ? 'Jawaban Salah' : 'Tidak Dijawab'}
              </span>
              <span className="ml-auto text-xs text-slate-500">{rq.subject} · {difficultyLabel(rq.difficulty)}</span>
            </div>

            <div className="p-6">
              {rq.stimulus && (
                <div className="mb-4 p-4 bg-blue-50 border border-blue-200 rounded-xl">
                  <p className="text-xs font-bold text-blue-600 mb-1.5 uppercase tracking-wider">Stimulus</p>
                  <p className="text-sm text-slate-700 leading-relaxed">{rq.stimulus}</p>
                </div>
              )}

              <p className="text-base font-semibold text-slate-900 mb-5 leading-relaxed">{rq.question}</p>

              {rq.options && rq.options.map((opt, idx) => {
                const labels = ['A', 'B', 'C', 'D', 'E'];
                const isCorrect = opt === correctAns;
                const isUser = opt === userAnswer;
                return (
                  <div
                    key={opt}
                    className={`flex items-start gap-3 p-3.5 rounded-xl mb-2 border-2 ${
                      isCorrect ? 'border-emerald-400 bg-emerald-50' :
                      isUser && !isCorrect ? 'border-red-400 bg-red-50' :
                      'border-transparent bg-slate-50'
                    }`}
                  >
                    <span className={`w-7 h-7 rounded-lg flex items-center justify-center text-xs font-black shrink-0 ${
                      isCorrect ? 'bg-emerald-500 text-white' :
                      isUser && !isCorrect ? 'bg-red-500 text-white' :
                      'bg-slate-200 text-slate-600'
                    }`}>{labels[idx]}</span>
                    <span className={`text-sm mt-0.5 ${isCorrect ? 'text-emerald-800 font-medium' : isUser && !isCorrect ? 'text-red-700' : 'text-slate-600'}`}>{opt}</span>
                    {isCorrect && <CheckCircle className="w-4 h-4 text-emerald-500 shrink-0 ml-auto mt-0.5" />}
                    {isUser && !isCorrect && <XCircle className="w-4 h-4 text-red-500 shrink-0 ml-auto mt-0.5" />}
                  </div>
                );
              })}
            </div>

            {/* Explanation */}
            <div className="mx-6 mb-6 p-4 bg-amber-50 border border-amber-200 rounded-xl">
              <p className="text-xs font-bold text-amber-700 mb-2 uppercase tracking-wider flex items-center gap-1.5">
                <ZoomIn className="w-3.5 h-3.5" /> Pembahasan
              </p>
              <p className="text-sm text-slate-700 leading-relaxed">{rq.explanation}</p>
            </div>
          </div>

          {/* Nav */}
          <div className="flex gap-3">
            <button onClick={() => setReviewIndex(Math.max(0, reviewIndex - 1))} disabled={reviewIndex === 0} className="flex-1 flex items-center justify-center gap-2 py-3 rounded-xl border border-slate-200 text-slate-600 font-semibold disabled:opacity-30 hover:bg-slate-50 transition-colors">
              <ChevronLeft className="w-4 h-4" /> Sebelumnya
            </button>
            {reviewIndex < questions.length - 1 ? (
              <button onClick={() => setReviewIndex(reviewIndex + 1)} className="flex-1 flex items-center justify-center gap-2 py-3 rounded-xl bg-indigo-600 text-white font-bold hover:bg-indigo-700 transition-colors">
                Selanjutnya <ChevronRight className="w-4 h-4" />
              </button>
            ) : (
              <button onClick={() => onFinish({ sessionTitle, score, accuracy, correct: correctCount, wrong: wrongCount, total: questions.length, timeUsed })} className="flex-1 flex items-center justify-center gap-2 py-3 rounded-xl bg-gradient-to-r from-indigo-600 to-purple-600 text-white font-bold transition-all">
                <Home className="w-4 h-4" /> Selesai
              </button>
            )}
          </div>
        </div>
      </div>
    );
  }

  return null;
}
