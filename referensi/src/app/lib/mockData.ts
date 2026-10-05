// Comprehensive Mock Data for GASPOLPTN System

export interface Question {
  id: string;
  code: string;
  type: 'multiple_choice' | 'complex_multiple' | 'short_answer';
  subject: string;
  topic: string;
  subtopic: string;
  difficulty: 'easy' | 'medium' | 'hard';
  bloomLevel: string;
  stimulus?: string;
  question: string;
  options?: string[];
  correctAnswer: string | string[];
  explanation: string;
  tags: string[];
  createdBy: string;
  createdAt: string;
  usageCount: number;
  averageScore: number;
  timeLimit?: number;
}

export interface Student {
  id: string;
  name: string;
  email: string;
  schoolId: string;
  schoolName: string;
  grade: string;
  class: string;
  registrationDate: string;
  lastActive: string;
  totalPoints: number;
  streak: number;
  badges: string[];
  stats: {
    totalTryouts: number;
    averageScore: number;
    totalQuestions: number;
    correctAnswers: number;
    studyTime: number;
  };
  targetUniversities: {
    name: string;
    major: string;
    probability: number;
  }[];
  strengths: string[];
  weaknesses: string[];
}

export interface School {
  id: string;
  name: string;
  type: 'SMA' | 'SMK' | 'MA';
  city: string;
  province: string;
  email: string;
  phone: string;
  joinDate: string;
  packageType: 'basic' | 'premium' | 'enterprise';
  totalStudents: number;
  activeStudents: number;
  totalTryouts: number;
  averageScore: number;
  contactPerson: string;
  status: 'active' | 'inactive';
}

export interface Event {
  id: string;
  title: string;
  type: 'tryout' | 'rasionalisasi' | 'drilling';
  description: string;
  startDate: string;
  endDate: string;
  duration: number;
  status: 'upcoming' | 'ongoing' | 'completed';
  participants: number;
  maxParticipants?: number;
  questionPackageId: string;
  createdBy: string;
  prizes?: string[];
}

// Mock Questions Database
export const mockQuestions: Question[] = [
  {
    id: 'Q001',
    code: 'MAT-ALG-001',
    type: 'multiple_choice',
    subject: 'Matematika',
    topic: 'Aljabar',
    subtopic: 'Persamaan Kuadrat',
    difficulty: 'medium',
    bloomLevel: 'C3 - Aplikasi',
    stimulus: 'Sebuah perusahaan startup teknologi memiliki pendapatan bulanan yang mengikuti pola kuadrat.',
    question: 'Jika pendapatan pada bulan ke-$t$ dinyatakan dengan $P(t) = -2t^2 + 12t + 10$ (dalam juta rupiah), pada bulan ke berapa perusahaan mencapai pendapatan maksimum?',
    options: ['Bulan ke-2', 'Bulan ke-3', 'Bulan ke-4', 'Bulan ke-5', 'Bulan ke-6'],
    correctAnswer: 'Bulan ke-3',
    explanation: 'Pendapatan maksimum tercapai pada $t = -\\frac{b}{2a} = -\\frac{12}{2(-2)} = 3$',
    tags: ['fungsi kuadrat', 'optimasi', 'titik maksimum'],
    createdBy: 'Admin Pusat',
    createdAt: '2025-01-15',
    usageCount: 1250,
    averageScore: 68,
    timeLimit: 120
  },
  {
    id: 'Q002',
    code: 'FIS-MKN-002',
    type: 'multiple_choice',
    subject: 'Fisika',
    topic: 'Mekanika',
    subtopic: 'Gerak Parabola',
    difficulty: 'hard',
    bloomLevel: 'C4 - Analisis',
    stimulus: 'Seorang atlet melempar bola basket dengan kecepatan awal 10 m/s membentuk sudut 45° terhadap horizontal. Gravitasi bumi 10 m/s².',
    question: 'Berapakah jarak mendatar maksimum yang dapat dicapai bola tersebut?',
    options: ['5 meter', '8 meter', '10 meter', '12 meter', '15 meter'],
    correctAnswer: '10 meter',
    explanation: 'Jarak mendatar maksimum: $R = \\frac{v_0^2 \\sin(2\\theta)}{g} = \\frac{100 \\times 1}{10} = 10$ meter',
    tags: ['gerak proyektil', 'kinematika', 'trigonometri'],
    createdBy: 'Admin Pusat',
    createdAt: '2025-01-14',
    usageCount: 980,
    averageScore: 52,
    timeLimit: 150
  },
  {
    id: 'Q003',
    code: 'BIO-SEL-003',
    type: 'complex_multiple',
    subject: 'Biologi',
    topic: 'Biologi Sel',
    subtopic: 'Respirasi Seluler',
    difficulty: 'medium',
    bloomLevel: 'C3 - Aplikasi',
    question: 'Pernyataan yang benar mengenai respirasi aerob adalah...\n1. Menghasilkan 38 ATP per molekul glukosa\n2. Terjadi di mitokondria\n3. Memerlukan oksigen sebagai akseptor elektron terakhir\n4. Menghasilkan CO₂ dan H₂O sebagai produk akhir',
    options: ['1, 2, dan 3', '1 dan 3', '2 dan 4', '1, 2, 3, dan 4', '2, 3, dan 4'],
    correctAnswer: '2, 3, dan 4',
    explanation: 'Respirasi aerob modern menghasilkan 30-32 ATP (bukan 38), terjadi di mitokondria, memerlukan O₂, dan menghasilkan CO₂ + H₂O',
    tags: ['metabolisme', 'energi', 'organel'],
    createdBy: 'Admin Pusat',
    createdAt: '2025-01-13',
    usageCount: 1450,
    averageScore: 61
  },
  {
    id: 'Q004',
    code: 'KIM-STO-004',
    type: 'short_answer',
    subject: 'Kimia',
    topic: 'Stoikiometri',
    subtopic: 'Konsep Mol',
    difficulty: 'easy',
    bloomLevel: 'C2 - Pemahaman',
    question: 'Berapa gram NaCl (Mr = 58,5) yang diperlukan untuk membuat 500 mL larutan NaCl 0,2 M? (Jawab dalam gram, tanpa satuan)',
    correctAnswer: '5.85',
    explanation: 'mol = M × V = 0,2 × 0,5 = 0,1 mol\nmassa = mol × Mr = 0,1 × 58,5 = 5,85 gram',
    tags: ['molaritas', 'perhitungan', 'larutan'],
    createdBy: 'Admin Pusat',
    createdAt: '2025-01-12',
    usageCount: 2100,
    averageScore: 78
  },
  {
    id: 'Q005',
    code: 'MAT-TRG-005',
    type: 'multiple_choice',
    subject: 'Matematika',
    topic: 'Trigonometri',
    subtopic: 'Identitas Trigonometri',
    difficulty: 'hard',
    bloomLevel: 'C4 - Analisis',
    question: 'Jika $\\sin x + \\cos x = \\frac{1}{2}$, maka nilai dari $\\sin x \\cos x$ adalah...',
    options: ['-3/8', '-1/4', '1/8', '1/4', '3/8'],
    correctAnswer: '-3/8',
    explanation: 'Kuadratkan kedua ruas: $(\\sin x + \\cos x)^2 = \\frac{1}{4}$\n$\\sin^2 x + 2\\sin x \\cos x + \\cos^2 x = \\frac{1}{4}$\n$1 + 2\\sin x \\cos x = \\frac{1}{4}$\n$\\sin x \\cos x = -\\frac{3}{8}$',
    tags: ['identitas', 'manipulasi aljabar'],
    createdBy: 'Admin Pusat',
    createdAt: '2025-01-11',
    usageCount: 756,
    averageScore: 45
  }
];

// Mock Students Database
export const mockStudents: Student[] = [
  {
    id: 'S001',
    name: 'Ahmad Fauzi Rahman',
    email: 'ahmad.fauzi@student.com',
    schoolId: 'SCH001',
    schoolName: 'SMA Negeri 1 Jakarta',
    grade: '12',
    class: 'IPA 1',
    registrationDate: '2024-07-15',
    lastActive: '2025-01-10 14:30',
    totalPoints: 15750,
    streak: 28,
    badges: ['Fast Learner', 'Math Wizard', 'Perfect Score', 'Marathon Runner', '100 Days'],
    stats: {
      totalTryouts: 45,
      averageScore: 567,
      totalQuestions: 2340,
      correctAnswers: 1872,
      studyTime: 12450
    },
    targetUniversities: [
      { name: 'Universitas Indonesia', major: 'Teknik Informatika', probability: 85 },
      { name: 'Institut Teknologi Bandung', major: 'Teknik Elektro', probability: 72 },
      { name: 'Universitas Gadjah Mada', major: 'Ilmu Komputer', probability: 88 }
    ],
    strengths: ['Matematika', 'Fisika', 'Penalaran Matematika'],
    weaknesses: ['Bahasa Inggris', 'Pengetahuan Umum']
  },
  {
    id: 'S002',
    name: 'Siti Nurhaliza',
    email: 'siti.nur@student.com',
    schoolId: 'SCH001',
    schoolName: 'SMA Negeri 1 Jakarta',
    grade: '12',
    class: 'IPA 2',
    registrationDate: '2024-07-16',
    lastActive: '2025-01-10 16:45',
    totalPoints: 18920,
    streak: 35,
    badges: ['Bio Master', 'Perfect Attendance', 'Top 10', 'Consistent', 'Rising Star'],
    stats: {
      totalTryouts: 52,
      averageScore: 612,
      totalQuestions: 2860,
      correctAnswers: 2374,
      studyTime: 15680
    },
    targetUniversities: [
      { name: 'Universitas Indonesia', major: 'Kedokteran', probability: 78 },
      { name: 'Universitas Airlangga', major: 'Kedokteran', probability: 82 },
      { name: 'Universitas Gadjah Mada', major: 'Farmasi', probability: 90 }
    ],
    strengths: ['Biologi', 'Kimia', 'Literasi'],
    weaknesses: ['Matematika', 'Fisika']
  },
  {
    id: 'S003',
    name: 'Budi Santoso',
    email: 'budi.santoso@student.com',
    schoolId: 'SCH001',
    schoolName: 'SMA Negeri 1 Jakarta',
    grade: '12',
    class: 'IPA 1',
    registrationDate: '2024-07-20',
    lastActive: '2025-01-09 20:15',
    totalPoints: 12340,
    streak: 15,
    badges: ['Newcomer', 'First Try', 'Persistent'],
    stats: {
      totalTryouts: 28,
      averageScore: 485,
      totalQuestions: 1540,
      correctAnswers: 1078,
      studyTime: 8920
    },
    targetUniversities: [
      { name: 'Institut Teknologi Sepuluh Nopember', major: 'Teknik Sipil', probability: 75 },
      { name: 'Universitas Diponegoro', major: 'Teknik Mesin', probability: 82 },
      { name: 'Universitas Brawijaya', major: 'Teknik Industri', probability: 88 }
    ],
    strengths: ['Fisika', 'Penalaran Umum'],
    weaknesses: ['Biologi', 'Bahasa Indonesia', 'Kimia']
  },
  {
    id: 'S004',
    name: 'Dewi Lestari',
    email: 'dewi.lestari@student.com',
    schoolId: 'SCH001',
    schoolName: 'SMA Negeri 1 Jakarta',
    grade: '12',
    class: 'IPS 1',
    registrationDate: '2024-07-18',
    lastActive: '2025-01-10 18:20',
    totalPoints: 16450,
    streak: 22,
    badges: ['Social Science Expert', 'Economics Master', 'Debate Champion'],
    stats: {
      totalTryouts: 38,
      averageScore: 578,
      totalQuestions: 2090,
      correctAnswers: 1776,
      studyTime: 11230
    },
    targetUniversities: [
      { name: 'Universitas Indonesia', major: 'Ilmu Ekonomi', probability: 83 },
      { name: 'Universitas Gadjah Mada', major: 'Manajemen', probability: 87 },
      { name: 'Universitas Padjadjaran', major: 'Akuntansi', probability: 91 }
    ],
    strengths: ['Ekonomi', 'Sosiologi', 'Geografi'],
    weaknesses: ['Matematika IPS', 'Sejarah']
  },
  {
    id: 'S005',
    name: 'Rizky Pratama',
    email: 'rizky.pratama@student.com',
    schoolId: 'SCH001',
    schoolName: 'SMA Negeri 1 Jakarta',
    grade: '11',
    class: 'IPA 3',
    registrationDate: '2024-08-01',
    lastActive: '2025-01-10 12:00',
    totalPoints: 8750,
    streak: 10,
    badges: ['Early Bird', 'Quick Learner'],
    stats: {
      totalTryouts: 15,
      averageScore: 512,
      totalQuestions: 825,
      correctAnswers: 618,
      studyTime: 4560
    },
    targetUniversities: [
      { name: 'Institut Teknologi Bandung', major: 'Astronomi', probability: 65 },
      { name: 'Universitas Indonesia', major: 'Fisika', probability: 70 },
      { name: 'Institut Pertanian Bogor', major: 'Meteorologi', probability: 78 }
    ],
    strengths: ['Fisika', 'Matematika'],
    weaknesses: ['Kimia', 'Biologi', 'Bahasa Inggris']
  }
];

// Mock Schools Database
export const mockSchools: School[] = [
  {
    id: 'SCH001',
    name: 'SMA Negeri 1 Jakarta',
    type: 'SMA',
    city: 'Jakarta Pusat',
    province: 'DKI Jakarta',
    email: 'info@sman1jakarta.sch.id',
    phone: '021-3456789',
    joinDate: '2024-01-15',
    packageType: 'enterprise',
    totalStudents: 540,
    activeStudents: 487,
    totalTryouts: 2340,
    averageScore: 565,
    contactPerson: 'Drs. Bambang Suryadi, M.Pd',
    status: 'active'
  },
  {
    id: 'SCH002',
    name: 'SMA Negeri 3 Bandung',
    type: 'SMA',
    city: 'Bandung',
    province: 'Jawa Barat',
    email: 'contact@sman3bandung.sch.id',
    phone: '022-7654321',
    joinDate: '2024-02-01',
    packageType: 'premium',
    totalStudents: 420,
    activeStudents: 398,
    totalTryouts: 1890,
    averageScore: 578,
    contactPerson: 'Dr. Siti Aminah, S.Pd., M.Ed',
    status: 'active'
  },
  {
    id: 'SCH003',
    name: 'SMA Negeri 5 Surabaya',
    type: 'SMA',
    city: 'Surabaya',
    province: 'Jawa Timur',
    email: 'admin@sman5surabaya.sch.id',
    phone: '031-8901234',
    joinDate: '2024-03-10',
    packageType: 'premium',
    totalStudents: 380,
    activeStudents: 356,
    totalTryouts: 1680,
    averageScore: 552,
    contactPerson: 'Dra. Retno Wijayanti, M.Pd',
    status: 'active'
  },
  {
    id: 'SCH004',
    name: 'SMA Negeri 2 Yogyakarta',
    type: 'SMA',
    city: 'Yogyakarta',
    province: 'DI Yogyakarta',
    email: 'sekolah@sman2yogya.sch.id',
    phone: '0274-567890',
    joinDate: '2024-01-20',
    packageType: 'enterprise',
    totalStudents: 460,
    activeStudents: 445,
    totalTryouts: 2150,
    averageScore: 592,
    contactPerson: 'Prof. Dr. Hadi Susanto, M.Sc',
    status: 'active'
  },
  {
    id: 'SCH005',
    name: 'SMA Negeri 1 Medan',
    type: 'SMA',
    city: 'Medan',
    province: 'Sumatera Utara',
    email: 'info@sman1medan.sch.id',
    phone: '061-4567890',
    joinDate: '2024-04-05',
    packageType: 'basic',
    totalStudents: 320,
    activeStudents: 285,
    totalTryouts: 1240,
    averageScore: 538,
    contactPerson: 'Drs. Ahmad Dahlan, M.M',
    status: 'active'
  }
];

// Mock Events Database
export const mockEvents: Event[] = [
  {
    id: 'EVT001',
    title: 'Tryout SNBT Nasional #15',
    type: 'tryout',
    description: 'Tryout komprehensif dengan sistem penilaian IRT. Mengukur kemampuan di semua subtes SNBT 2025.',
    startDate: '2025-01-15 08:00',
    endDate: '2025-01-15 13:00',
    duration: 195,
    status: 'completed',
    participants: 12450,
    maxParticipants: 15000,
    questionPackageId: 'PKG001',
    createdBy: 'Admin Pusat',
    prizes: ['Beasiswa Rp 10 juta', 'Laptop', 'Tablet']
  },
  {
    id: 'EVT002',
    title: 'Tryout SNBT Nasional #16',
    type: 'tryout',
    description: 'Tryout dengan soal prediksi terbaru berdasarkan kisi-kisi SNBT 2025. Dilengkapi pembahasan video.',
    startDate: '2025-01-22 08:00',
    endDate: '2025-01-22 13:00',
    duration: 195,
    status: 'ongoing',
    participants: 8750,
    maxParticipants: 15000,
    questionPackageId: 'PKG002',
    createdBy: 'Admin Pusat',
    prizes: ['Beasiswa Rp 10 juta', 'Smartphone', 'Voucher Belajar']
  },
  {
    id: 'EVT003',
    title: 'Rasionalisasi PTN - Gelombang 3',
    type: 'rasionalisasi',
    description: 'Simulasi pengisian pilihan PTN dengan AI predictor. Dapatkan rekomendasi PTN terbaik berdasarkan skor Anda.',
    startDate: '2025-01-20 00:00',
    endDate: '2025-01-27 23:59',
    duration: 0,
    status: 'ongoing',
    participants: 15680,
    questionPackageId: 'PKG003',
    createdBy: 'Admin Pusat'
  },
  {
    id: 'EVT004',
    title: 'Marathon Drilling - Matematika',
    type: 'drilling',
    description: '500 soal matematika dari mudah hingga sulit. Uji konsistensi dan kecepatan Anda!',
    startDate: '2025-01-18 00:00',
    endDate: '2025-01-25 23:59',
    duration: 0,
    status: 'ongoing',
    participants: 5240,
    questionPackageId: 'PKG004',
    createdBy: 'Admin Pusat',
    prizes: ['Trophy Digital', 'Badge Eksklusif']
  },
  {
    id: 'EVT005',
    title: 'Tryout SNBT Nasional #17',
    type: 'tryout',
    description: 'Tryout terakhir sebelum SNBT! Soal berkualitas dengan analisis mendalam.',
    startDate: '2025-01-29 08:00',
    endDate: '2025-01-29 13:00',
    duration: 195,
    status: 'upcoming',
    participants: 0,
    maxParticipants: 15000,
    questionPackageId: 'PKG005',
    createdBy: 'Admin Pusat',
    prizes: ['Beasiswa Rp 15 juta', 'iPad', 'Buku Premium']
  }
];

// Analytics Data
export const mockAnalytics = {
  national: {
    totalStudents: 52340,
    activeStudents: 48120,
    totalSchools: 512,
    totalQuestions: 10240,
    totalTryouts: 156,
    averageScore: 558,
    completionRate: 87.5,
    growthRate: 23.5
  },
  performance: {
    bySubject: [
      { subject: 'Matematika', avgScore: 62, totalAttempts: 125000 },
      { subject: 'Fisika', avgScore: 58, totalAttempts: 98000 },
      { subject: 'Kimia', avgScore: 64, totalAttempts: 102000 },
      { subject: 'Biologi', avgScore: 68, totalAttempts: 115000 },
      { subject: 'Bahasa Indonesia', avgScore: 72, totalAttempts: 130000 },
      { subject: 'Bahasa Inggris', avgScore: 66, totalAttempts: 118000 }
    ],
    byDifficulty: [
      { level: 'Easy', avgScore: 78, totalAttempts: 180000 },
      { level: 'Medium', avgScore: 62, totalAttempts: 250000 },
      { level: 'Hard', avgScore: 45, totalAttempts: 145000 }
    ]
  },
  revenue: {
    monthly: [
      { month: 'Jul', amount: 245000000 },
      { month: 'Aug', amount: 278000000 },
      { month: 'Sep', amount: 312000000 },
      { month: 'Oct', amount: 356000000 },
      { month: 'Nov', amount: 398000000 },
      { month: 'Dec', amount: 445000000 },
      { month: 'Jan', amount: 512000000 }
    ],
    total: 2546000000,
    growth: 18.5
  }
};

// Leaderboard Data
export const mockLeaderboard = [
  { rank: 1, name: 'Siti Nurhaliza', school: 'SMA Negeri 1 Jakarta', score: 612, badge: '🥇' },
  { rank: 2, name: 'Muhammad Rizki', school: 'SMA Negeri 2 Yogyakarta', score: 608, badge: '🥈' },
  { rank: 3, name: 'Anisa Rahma', school: 'SMA Negeri 3 Bandung', score: 605, badge: '🥉' },
  { rank: 4, name: 'Farhan Aziz', school: 'SMA Negeri 1 Surabaya', score: 598, badge: '⭐' },
  { rank: 5, name: 'Zahra Amelia', school: 'SMA Negeri 5 Semarang', score: 594, badge: '⭐' },
  { rank: 6, name: 'Ahmad Fauzi Rahman', school: 'SMA Negeri 1 Jakarta', score: 567, badge: '⭐' },
  { rank: 7, name: 'Dewi Lestari', school: 'SMA Negeri 1 Jakarta', score: 578, badge: '⭐' },
  { rank: 8, name: 'Rafi Akbar', school: 'SMA Negeri 8 Jakarta', score: 565, badge: '⭐' },
  { rank: 9, name: 'Nina Safitri', school: 'SMA Negeri 1 Malang', score: 562, badge: '⭐' },
  { rank: 10, name: 'Hendra Wijaya', school: 'SMA Negeri 2 Surabaya', score: 558, badge: '⭐' }
];

// Achievement Badges
export const achievementBadges = [
  { id: 'fast_learner', name: 'Fast Learner', icon: '⚡', description: 'Selesaikan 10 tryout dalam sebulan', rarity: 'common' },
  { id: 'math_wizard', name: 'Math Wizard', icon: '🧙‍♂️', description: 'Skor matematika 90+ sebanyak 5 kali', rarity: 'rare' },
  { id: 'perfect_score', name: 'Perfect Score', icon: '💯', description: 'Raih skor sempurna dalam satu subtes', rarity: 'epic' },
  { id: 'marathon_runner', name: 'Marathon Runner', icon: '🏃', description: 'Selesaikan 500 soal dalam seminggu', rarity: 'rare' },
  { id: '100_days', name: '100 Days Streak', icon: '🔥', description: 'Belajar konsisten 100 hari berturut-turut', rarity: 'legendary' },
  { id: 'bio_master', name: 'Biology Master', icon: '🧬', description: 'Jawab benar 100 soal biologi berturut-turut', rarity: 'epic' },
  { id: 'top_10', name: 'Top 10 National', icon: '👑', description: 'Masuk 10 besar leaderboard nasional', rarity: 'legendary' },
  { id: 'consistent', name: 'Consistent Learner', icon: '📚', description: 'Belajar setiap hari selama 30 hari', rarity: 'uncommon' },
  { id: 'rising_star', name: 'Rising Star', icon: '🌟', description: 'Tingkatkan skor 100 poin dalam sebulan', rarity: 'rare' }
];

// University Data
export const universities = [
  { name: 'Universitas Indonesia', city: 'Depok', passing: 620, capacity: 8500 },
  { name: 'Institut Teknologi Bandung', city: 'Bandung', passing: 615, capacity: 6200 },
  { name: 'Universitas Gadjah Mada', city: 'Yogyakarta', passing: 618, capacity: 9100 },
  { name: 'Institut Pertanian Bogor', city: 'Bogor', passing: 598, capacity: 5800 },
  { name: 'Universitas Airlangga', city: 'Surabaya', passing: 605, capacity: 7200 },
  { name: 'Institut Teknologi Sepuluh Nopember', city: 'Surabaya', passing: 595, capacity: 5400 },
  { name: 'Universitas Diponegoro', city: 'Semarang', passing: 585, capacity: 6800 },
  { name: 'Universitas Brawijaya', city: 'Malang', passing: 582, capacity: 6500 },
  { name: 'Universitas Padjadjaran', city: 'Bandung', passing: 592, capacity: 6900 },
  { name: 'Universitas Hasanuddin', city: 'Makassar', passing: 578, capacity: 6200 }
];
