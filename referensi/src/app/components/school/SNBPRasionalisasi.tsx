import { useState, useMemo } from 'react';
import { Card } from '../ui/card';
import { Badge } from '../ui/badge';
import { toast } from 'sonner';
import {
  Target, Users, GraduationCap, TrendingUp, Plus, Search,
  ChevronRight, ChevronLeft, X, Edit2, Trash2, Eye, CheckCircle2,
  AlertCircle, Download, RefreshCw, Star, Building2, Save, Medal,
  Award, Printer, BookOpen, MapPin, BarChart2, ClipboardList,
  TrendingDown, MinusCircle
} from 'lucide-react';
import {
  RadarChart, Radar, PolarGrid, PolarAngleAxis, PolarRadiusAxis,
  ResponsiveContainer, PieChart as RechartsPieChart, Pie, Cell, Tooltip,
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Legend
} from 'recharts';

// ─── Types ────────────────────────────────────────────────────────────────────

type SectionKey = 'dashboard' | 'siswa' | 'rasionalisasi' | 'alumni' | 'eligible' | 'posmin';

interface Achievement {
  id: string;
  nama: string;
  tingkat: 'sekolah' | 'kab-kota' | 'provinsi' | 'nasional' | 'internasional';
  tahun: number;
  juara: string;
}

type SubjectMap = Record<string, { value: number; active: boolean }>;
type SemesterScores = Record<string, SubjectMap>;

interface RasIndex {
  nilai: number; snbp: number; snbt: number; prestasi: number;
  universitas: number; jurusan: number; akreditasi: number; total: number;
}

interface SnbpStudent {
  id: string; name: string; class: string; year: number; konsultan: string;
  pil1uni: string; pil1jur: string; pil2uni: string; pil2jur: string;
  statusPenerimaan: 'belum' | 'diterima-snbp' | 'diterima-snbt' | 'tidak';
  aktif: boolean; rankSekolah?: number; peluang1?: number; peluang2?: number;
  scores?: SemesterScores; prestasi?: Achievement[];
  indexes1?: RasIndex; indexes2?: RasIndex; hasRasionalisasi: boolean;
}

interface AlumniRecord {
  id: string; name: string; uni: string; jurusan: string; year: number;
  pmRapor: number; pmUTBK: number;
}

interface EligibleYear { id: string; year: number; total: number; }

interface PosMin {
  id: string; uni: string; jurusan: string; jenjang: string; provinsi: string;
  rumpun: string; prasyarat: string; minRapor: number; minUTBK: number;
  kuotaSNBP: number; peminatSNBP: number;
}

interface SnbtRecord {
  id: string;
  studentName: string;
  class: string;
  year: number;
  terdaftarSNBT: boolean;
  pil1uni: string; pil1jur: string;
  pil2uni: string; pil2jur: string;
  estimasiSkor: number;
  skorAktual?: number;
  statusPenerimaan: 'belum-ujian' | 'sudah-ujian' | 'diterima' | 'tidak-diterima';
  tglUjian?: string;
  catatan?: string;
}

// ─── Constants ────────────────────────────────────────────────────────────────

const SUBJECTS = [
  'B. Indonesia', 'Matematika', 'B. Inggris', 'Fisika', 'Kimia', 'Biologi',
  'Sejarah', 'Ekonomi', 'Sosiologi', 'Geografi', 'PKN', 'Seni Budaya',
  'PJOK', 'Prakarya', 'Mat. Tingkat Lanjut',
];
const SEMESTERS = ['S1', 'S2', 'S3', 'S4', 'S5'];
const PTN_LIST = ['UI', 'ITB', 'UGM', 'UNAIR', 'IPB', 'UNPAD', 'UNDIP', 'ITS', 'UB', 'UNS', 'UNHAS', 'UPI'];
const JURUSAN_BY_PTN: Record<string, string[]> = {
  'UI': ['Teknik Informatika', 'Kedokteran', 'Psikologi', 'Hukum', 'Akuntansi', 'Teknik Sipil', 'Ilmu Komunikasi'],
  'ITB': ['Teknik Informatika', 'Teknik Sipil', 'Teknik Kimia', 'Matematika', 'Fisika', 'Teknik Mesin'],
  'UGM': ['Kedokteran', 'Psikologi', 'Teknik Informatika', 'Akuntansi', 'Ilmu Komunikasi', 'Farmasi', 'Hukum'],
  'UNPAD': ['Kedokteran', 'Farmasi', 'Psikologi', 'Hukum', 'Ilmu Komunikasi', 'Akuntansi'],
  'UNAIR': ['Kedokteran', 'Farmasi', 'Hukum', 'Akuntansi', 'Psikologi', 'Teknik Informatika'],
  'UNDIP': ['Teknik Informatika', 'Hukum', 'Akuntansi', 'Kedokteran', 'Teknik Sipil', 'Psikologi'],
  'ITS': ['Teknik Informatika', 'Teknik Elektro', 'Teknik Sipil', 'Statistika', 'Teknik Kimia'],
  'IPB': ['Kedokteran Hewan', 'Teknologi Pangan', 'Agribisnis', 'Biologi', 'Matematika'],
  'UB': ['Teknik Informatika', 'Hukum', 'Akuntansi', 'Kedokteran', 'Psikologi'],
  'UNS': ['Kedokteran', 'Hukum', 'Teknik Informatika', 'Psikologi', 'Akuntansi'],
  'UNHAS': ['Kedokteran', 'Teknik Informatika', 'Hukum', 'Psikologi', 'Farmasi'],
  'UPI': ['Pendidikan Matematika', 'Pendidikan B. Indonesia', 'Psikologi Pendidikan', 'Bimbingan Konseling'],
};
const UNI_INDEX: Record<string, number> = {
  'UI': 8, 'ITB': 8, 'UGM': 8, 'UNAIR': 6, 'IPB': 5.5, 'UNDIP': 5.5,
  'ITS': 6, 'UNPAD': 5, 'UB': 4, 'UNS': 4, 'UNHAS': 4, 'UPI': 3,
};
const PRESTASI_BONUS: Record<string, number> = {
  'sekolah': 0.5, 'kab-kota': 1, 'provinsi': 2, 'nasional': 4, 'internasional': 8,
};
const PIE_COLORS = ['#6366f1', '#8b5cf6', '#06b6d4', '#10b981', '#f59e0b', '#ef4444', '#ec4899'];

// ─── Seed Data ────────────────────────────────────────────────────────────────

function mkScores(base: number, ipa = true): SemesterScores {
  const s: SemesterScores = {};
  for (const sem of SEMESTERS) {
    s[sem] = {};
    for (const subj of SUBJECTS) {
      const isIPS = ['Ekonomi', 'Sosiologi', 'Geografi'].includes(subj);
      const isIPA = ['Fisika', 'Kimia', 'Biologi'].includes(subj);
      const isMTL = subj === 'Mat. Tingkat Lanjut';
      const off = (ipa && isIPS) || (!ipa && (isIPA || isMTL));
      const v = off ? 0 : Math.min(100, base + Math.floor(Math.random() * 6 - 1));
      s[sem][subj] = { value: v, active: !off };
    }
  }
  return s;
}

const INIT_STUDENTS: SnbpStudent[] = [
  {
    id: 'sn1', name: 'Tijanil Ulfa', class: 'XII IPA 1', year: 2026, konsultan: 'Bu Sari',
    pil1uni: 'UGM', pil1jur: 'Psikologi', pil2uni: 'UNDIP', pil2jur: 'Psikologi',
    statusPenerimaan: 'diterima-snbp', aktif: true, rankSekolah: 2, peluang1: 58, peluang2: 71,
    hasRasionalisasi: true, scores: mkScores(91),
    indexes1: { nilai: 49.91, snbp: 1.62, snbt: 0, prestasi: 0, universitas: 8, jurusan: 0, akreditasi: 1, total: 60.53 },
    indexes2: { nilai: 52.3, snbp: 1.62, snbt: 0, prestasi: 0, universitas: 5.5, jurusan: 0, akreditasi: 1, total: 60.42 },
    prestasi: [],
  },
  {
    id: 'sn2', name: 'Ahmad Fauzi', class: 'XII IPA 1', year: 2026, konsultan: 'Pak Budi',
    pil1uni: 'UI', pil1jur: 'Teknik Informatika', pil2uni: 'ITB', pil2jur: 'Teknik Informatika',
    statusPenerimaan: 'belum', aktif: true, rankSekolah: 1, peluang1: 74, peluang2: 68,
    hasRasionalisasi: true, scores: mkScores(93),
    indexes1: { nilai: 54.2, snbp: 2.1, snbt: 0, prestasi: 6, universitas: 8, jurusan: 0, akreditasi: 1, total: 71.3 },
    indexes2: { nilai: 53.8, snbp: 2.1, snbt: 0, prestasi: 6, universitas: 8, jurusan: 0, akreditasi: 1, total: 70.9 },
    prestasi: [
      { id: 'p1', nama: 'Olimpiade Matematika', tingkat: 'provinsi', tahun: 2024, juara: 'Juara 2' },
      { id: 'p2', nama: 'Hackathon Pelajar Nasional', tingkat: 'nasional', tahun: 2025, juara: 'Juara 3' },
    ],
  },
  {
    id: 'sn3', name: 'Sari Dewi Anggraeni', class: 'XII IPA 2', year: 2026, konsultan: 'Bu Sari',
    pil1uni: 'UGM', pil1jur: 'Kedokteran', pil2uni: 'UNPAD', pil2jur: 'Kedokteran',
    statusPenerimaan: 'belum', aktif: true, rankSekolah: 4, peluang1: 38, peluang2: 45,
    hasRasionalisasi: true, scores: mkScores(87),
    indexes1: { nilai: 40.5, snbp: 1.8, snbt: 0, prestasi: 0, universitas: 8, jurusan: 0, akreditasi: 1, total: 51.3 },
    indexes2: { nilai: 44.2, snbp: 1.8, snbt: 0, prestasi: 0, universitas: 5, jurusan: 0, akreditasi: 1, total: 52.0 },
    prestasi: [],
  },
  {
    id: 'sn4', name: 'Budi Santoso', class: 'XII IPS 1', year: 2026, konsultan: 'Pak Budi',
    pil1uni: 'UI', pil1jur: 'Akuntansi', pil2uni: 'UNDIP', pil2jur: 'Akuntansi',
    statusPenerimaan: 'belum', aktif: true, rankSekolah: 3, peluang1: 61, peluang2: 68,
    hasRasionalisasi: true, scores: mkScores(89, false),
    indexes1: { nilai: 48.5, snbp: 1.5, snbt: 0, prestasi: 0, universitas: 8, jurusan: 0, akreditasi: 1, total: 59.0 },
    indexes2: { nilai: 52.1, snbp: 1.5, snbt: 0, prestasi: 0, universitas: 5.5, jurusan: 0, akreditasi: 1, total: 60.1 },
    prestasi: [],
  },
  {
    id: 'sn5', name: 'Rina Marlena', class: 'XI IPA 1', year: 2027, konsultan: 'Bu Sari',
    pil1uni: 'ITS', pil1jur: 'Teknik Elektro', pil2uni: 'UNDIP', pil2jur: 'Teknik Sipil',
    statusPenerimaan: 'belum', aktif: true, hasRasionalisasi: false, prestasi: [],
  },
  {
    id: 'sn6', name: 'Gilang Pratama', class: 'XII IPA 2', year: 2026, konsultan: 'Pak Budi',
    pil1uni: 'UGM', pil1jur: 'Kedokteran', pil2uni: 'UNHAS', pil2jur: 'Kedokteran',
    statusPenerimaan: 'belum', aktif: true, rankSekolah: 5, peluang1: 28, peluang2: 35,
    hasRasionalisasi: true, scores: mkScores(85),
    indexes1: { nilai: 36.2, snbp: 1.2, snbt: 0, prestasi: 0, universitas: 8, jurusan: 0, akreditasi: 1, total: 46.4 },
    indexes2: { nilai: 38.5, snbp: 1.2, snbt: 0, prestasi: 0, universitas: 4, jurusan: 0, akreditasi: 1, total: 44.7 },
    prestasi: [],
  },
];

const INIT_ALUMNI: AlumniRecord[] = [
  { id: 'a1', name: 'Dewi Kusuma Wardhani', uni: 'UGM', jurusan: 'Psikologi', year: 2024, pmRapor: 91.2, pmUTBK: 674 },
  { id: 'a2', name: 'Rizky Aditya Nugraha', uni: 'UI', jurusan: 'Teknik Informatika', year: 2023, pmRapor: 93.5, pmUTBK: 720 },
  { id: 'a3', name: 'Putri Handayani Saputri', uni: 'UNPAD', jurusan: 'Kedokteran', year: 2024, pmRapor: 94.1, pmUTBK: 710 },
  { id: 'a4', name: 'Eko Prasetyo Wibowo', uni: 'UNDIP', jurusan: 'Teknik Informatika', year: 2023, pmRapor: 87.4, pmUTBK: 645 },
  { id: 'a5', name: 'Fajar Nugraha Putra', uni: 'ITS', jurusan: 'Teknik Elektro', year: 2024, pmRapor: 88.7, pmUTBK: 658 },
  { id: 'a6', name: 'Lina Susanti Rahayu', uni: 'UGM', jurusan: 'Akuntansi', year: 2023, pmRapor: 90.3, pmUTBK: 680 },
];

const INIT_ELIGIBLE: EligibleYear[] = [
  { id: 'e1', year: 2021, total: 120 },
  { id: 'e2', year: 2022, total: 135 },
  { id: 'e3', year: 2023, total: 128 },
  { id: 'e4', year: 2024, total: 145 },
  { id: 'e5', year: 2025, total: 35 },
];

const POSMIN_DB: PosMin[] = [
  { id: 'pm1', uni: 'UGM', jurusan: 'Psikologi', jenjang: 'S1', provinsi: 'D.I. Yogyakarta', rumpun: 'Soshum', prasyarat: 'B.Indonesia, B.Inggris', minRapor: 90.0, minUTBK: 670, kuotaSNBP: 68, peminatSNBP: 1614 },
  { id: 'pm2', uni: 'UI', jurusan: 'Teknik Informatika', jenjang: 'S1', provinsi: 'DKI Jakarta', rumpun: 'Saintek', prasyarat: 'Matematika, Fisika', minRapor: 92.5, minUTBK: 720, kuotaSNBP: 45, peminatSNBP: 980 },
  { id: 'pm3', uni: 'ITB', jurusan: 'Teknik Informatika', jenjang: 'S1', provinsi: 'Jawa Barat', rumpun: 'Saintek', prasyarat: 'Matematika, Fisika', minRapor: 93.0, minUTBK: 730, kuotaSNBP: 40, peminatSNBP: 1200 },
  { id: 'pm4', uni: 'UGM', jurusan: 'Kedokteran', jenjang: 'S1', provinsi: 'D.I. Yogyakarta', rumpun: 'Saintek', prasyarat: 'Biologi, Kimia', minRapor: 95.0, minUTBK: 750, kuotaSNBP: 30, peminatSNBP: 2100 },
  { id: 'pm5', uni: 'UNPAD', jurusan: 'Kedokteran', jenjang: 'S1', provinsi: 'Jawa Barat', rumpun: 'Saintek', prasyarat: 'Biologi, Kimia', minRapor: 94.0, minUTBK: 720, kuotaSNBP: 35, peminatSNBP: 1800 },
  { id: 'pm6', uni: 'UI', jurusan: 'Akuntansi', jenjang: 'S1', provinsi: 'DKI Jakarta', rumpun: 'Soshum', prasyarat: 'Ekonomi, Matematika', minRapor: 91.0, minUTBK: 690, kuotaSNBP: 50, peminatSNBP: 1100 },
  { id: 'pm7', uni: 'UNDIP', jurusan: 'Psikologi', jenjang: 'S1', provinsi: 'Jawa Tengah', rumpun: 'Soshum', prasyarat: 'B.Indonesia, B.Inggris', minRapor: 87.0, minUTBK: 640, kuotaSNBP: 55, peminatSNBP: 890 },
  { id: 'pm8', uni: 'UNDIP', jurusan: 'Akuntansi', jenjang: 'S1', provinsi: 'Jawa Tengah', rumpun: 'Soshum', prasyarat: 'Ekonomi, Matematika', minRapor: 86.5, minUTBK: 635, kuotaSNBP: 60, peminatSNBP: 950 },
  { id: 'pm9', uni: 'ITS', jurusan: 'Teknik Elektro', jenjang: 'S1', provinsi: 'Jawa Timur', rumpun: 'Saintek', prasyarat: 'Matematika, Fisika', minRapor: 89.0, minUTBK: 660, kuotaSNBP: 48, peminatSNBP: 750 },
  { id: 'pm10', uni: 'UNHAS', jurusan: 'Kedokteran', jenjang: 'S1', provinsi: 'Sulawesi Selatan', rumpun: 'Saintek', prasyarat: 'Biologi, Kimia', minRapor: 90.0, minUTBK: 680, kuotaSNBP: 40, peminatSNBP: 1200 },
  { id: 'pm11', uni: 'UNS', jurusan: 'Kedokteran', jenjang: 'S1', provinsi: 'Jawa Tengah', rumpun: 'Saintek', prasyarat: 'Biologi, Kimia', minRapor: 91.0, minUTBK: 685, kuotaSNBP: 38, peminatSNBP: 1350 },
  { id: 'pm12', uni: 'IPB', jurusan: 'Teknologi Pangan', jenjang: 'S1', provinsi: 'Jawa Barat', rumpun: 'Saintek', prasyarat: 'Kimia, Biologi', minRapor: 85.0, minUTBK: 620, kuotaSNBP: 70, peminatSNBP: 650 },
];

const INIT_SNBT_RECORDS: SnbtRecord[] = [
  { id:'sb1', studentName:'Ahmad Fauzi', class:'XII IPA 1', year:2026, terdaftarSNBT:true, pil1uni:'UI', pil1jur:'Teknik Informatika', pil2uni:'ITB', pil2jur:'Teknik Informatika', estimasiSkor:721, skorAktual:724, statusPenerimaan:'tidak-diterima', tglUjian:'2026-05-13', catatan:'Skor SNBT sudah masuk, menunggu pengumuman' },
  { id:'sb2', studentName:'Sari Dewi Anggraeni', class:'XII IPA 2', year:2026, terdaftarSNBT:true, pil1uni:'UGM', pil1jur:'Kedokteran', pil2uni:'UNPAD', pil2jur:'Kedokteran', estimasiSkor:695, skorAktual:701, statusPenerimaan:'sudah-ujian', tglUjian:'2026-05-13' },
  { id:'sb3', studentName:'Budi Santoso', class:'XII IPS 1', year:2026, terdaftarSNBT:true, pil1uni:'UI', pil1jur:'Akuntansi', pil2uni:'UNDIP', pil2jur:'Akuntansi', estimasiSkor:672, skorAktual:681, statusPenerimaan:'diterima', tglUjian:'2026-05-13' },
  { id:'sb4', studentName:'Gilang Pratama', class:'XII IPA 2', year:2026, terdaftarSNBT:true, pil1uni:'UGM', pil1jur:'Kedokteran', pil2uni:'UNHAS', pil2jur:'Kedokteran', estimasiSkor:645, statusPenerimaan:'belum-ujian' },
  { id:'sb5', studentName:'Tijanil Ulfa', class:'XII IPA 1', year:2026, terdaftarSNBT:false, pil1uni:'UGM', pil1jur:'Psikologi', pil2uni:'UNDIP', pil2jur:'Psikologi', estimasiSkor:682, statusPenerimaan:'belum-ujian', catatan:'Sudah diterima SNBP, tidak perlu ikut SNBT' },
  { id:'sb6', studentName:'Rina Marlena', class:'XI IPA 1', year:2027, terdaftarSNBT:false, pil1uni:'ITS', pil1jur:'Teknik Elektro', pil2uni:'UNDIP', pil2jur:'Teknik Sipil', estimasiSkor:658, statusPenerimaan:'belum-ujian' },
];

// ─── Helpers ──────────────────────────────────────────────────────────────────

function avgRapor(scores: SemesterScores): number {
  let total = 0, count = 0;
  for (const sem of Object.values(scores)) {
    for (const subj of Object.values(sem)) {
      if (subj.active && subj.value > 0) { total += subj.value; count++; }
    }
  }
  return count > 0 ? Math.round((total / count) * 100) / 100 : 0;
}

function peluangBadge(pct?: number): { label: string; textColor: string; bgColor: string } {
  if (!pct) return { label: '—', textColor: 'text-slate-400', bgColor: 'bg-slate-100' };
  if (pct >= 70) return { label: 'TINGGI', textColor: 'text-emerald-700', bgColor: 'bg-emerald-100' };
  if (pct >= 50) return { label: 'SEDANG', textColor: 'text-amber-700', bgColor: 'bg-amber-100' };
  return { label: 'RENDAH', textColor: 'text-red-700', bgColor: 'bg-red-100' };
}

function statusInfo(s: SnbpStudent['statusPenerimaan']): { label: string; cls: string } {
  if (s === 'diterima-snbp') return { label: 'Diterima SNBP', cls: 'bg-emerald-100 text-emerald-700' };
  if (s === 'diterima-snbt') return { label: 'Diterima SNBT', cls: 'bg-blue-100 text-blue-700' };
  if (s === 'tidak') return { label: 'Tidak Diterima', cls: 'bg-red-100 text-red-700' };
  return { label: 'Belum Diterima', cls: 'bg-slate-100 text-slate-500' };
}

function calcIndexes(scores: SemesterScores, prestasi: Achievement[], uni: string, jur: string): RasIndex {
  const avg = avgRapor(scores);
  const posmin = POSMIN_DB.find(p => p.uni === uni && p.jurusan === jur);
  const minR = posmin?.minRapor ?? 88;
  const indexNilai = Math.min((avg / minR) * 50, 50);
  const indexSNBP = avg > 0 ? 1.5 : 0;
  const indexPrestasi = Math.min(prestasi.reduce((sum, p) => sum + (PRESTASI_BONUS[p.tingkat] ?? 0), 0), 10);
  const indexUniversitas = UNI_INDEX[uni] ?? 3;
  const indexAkreditasi = 1;
  const total = indexNilai + indexSNBP + indexPrestasi + indexUniversitas + indexAkreditasi;
  return {
    nilai: Math.round(indexNilai * 100) / 100,
    snbp: indexSNBP,
    snbt: 0,
    prestasi: Math.round(indexPrestasi * 100) / 100,
    universitas: indexUniversitas,
    jurusan: 0,
    akreditasi: indexAkreditasi,
    total: Math.round(total * 100) / 100,
  };
}

function initSemScores(): SemesterScores {
  const s: SemesterScores = {};
  for (const sem of SEMESTERS) {
    s[sem] = {};
    for (const subj of SUBJECTS) {
      s[sem][subj] = { value: 0, active: true };
    }
  }
  return s;
}

// ─── Section: Dashboard ───────────────────────────────────────────────────────

function DashboardSNBP({ students, eligible }: { students: SnbpStudent[]; eligible: EligibleYear[] }) {
  const [selectedYear, setSelectedYear] = useState(2026);
  const yearSt = students.filter(s => s.year === selectedYear && s.aktif);
  const eligibleCount = eligible.find(e => e.year === selectedYear)?.total ?? 0;

  const uniDistrib = useMemo(() => {
    const map: Record<string, number> = {};
    for (const s of students) {
      if (s.pil1uni) map[s.pil1uni] = (map[s.pil1uni] ?? 0) + 1;
    }
    return Object.entries(map).map(([name, value]) => ({ name, value }));
  }, [students]);

  const progressBars = [
    { label: 'Siswa Aktif', val: students.filter(s => s.aktif && s.year === selectedYear).length, max: Math.max(eligibleCount, 1) },
    { label: 'Data Rapor Lengkap', val: yearSt.filter(s => s.scores && Object.keys(s.scores).length > 0).length, max: Math.max(eligibleCount, 1) },
    { label: 'Terasionalisasi', val: yearSt.filter(s => s.hasRasionalisasi).length, max: Math.max(eligibleCount, 1) },
    { label: 'Terkonfirmasi Lolos', val: yearSt.filter(s => s.statusPenerimaan !== 'belum').length, max: Math.max(eligibleCount, 1) },
  ];

  const statsCards = [
    { label: 'Siswa Aktif Platform', val: students.filter(s => s.aktif).length, icon: Users, txt: 'text-indigo-600', bg: 'bg-indigo-50', border: 'border-indigo-200', ib: 'bg-indigo-100' },
    { label: `Eligible SNBP ${selectedYear}`, val: eligibleCount, icon: Award, txt: 'text-purple-600', bg: 'bg-purple-50', border: 'border-purple-200', ib: 'bg-purple-100' },
    { label: 'Terasionalisasi', val: yearSt.filter(s => s.hasRasionalisasi).length, icon: Target, txt: 'text-blue-600', bg: 'bg-blue-50', border: 'border-blue-200', ib: 'bg-blue-100' },
    { label: 'Terkonfirmasi Lolos', val: yearSt.filter(s => s.statusPenerimaan !== 'belum').length, icon: Medal, txt: 'text-emerald-600', bg: 'bg-emerald-50', border: 'border-emerald-200', ib: 'bg-emerald-100' },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3 flex-wrap">
        <span className="text-sm font-semibold text-slate-500">Tahun SNBP:</span>
        {[2024, 2025, 2026, 2027].map(y => (
          <button key={y} onClick={() => setSelectedYear(y)}
            className={`px-4 py-1.5 rounded-full text-sm font-bold border transition-all ${selectedYear === y ? 'bg-indigo-600 text-white border-indigo-600 shadow-sm' : 'bg-white text-slate-600 border-slate-200 hover:border-indigo-300'}`}>
            {y}
          </button>
        ))}
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
        {statsCards.map(({ label, val, icon: Icon, txt, bg, border, ib }) => (
          <Card key={label} className={`p-4 ${bg} ${border} border`}>
            <div className="flex items-start justify-between">
              <div>
                <p className="text-xs text-slate-500 mb-1 leading-tight">{label}</p>
                <p className={`text-3xl font-black ${txt}`}>{val}</p>
              </div>
              <div className={`w-10 h-10 rounded-xl ${ib} flex items-center justify-center`}>
                <Icon className={`w-5 h-5 ${txt}`} />
              </div>
            </div>
          </Card>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card className="p-5">
          <h3 className="font-bold text-slate-800 mb-4">Sebaran Universitas Tujuan</h3>
          {uniDistrib.length > 0 ? (
            <ResponsiveContainer width="100%" height={220}>
              <RechartsPieChart>
                <Pie data={uniDistrib} cx="50%" cy="50%" innerRadius={55} outerRadius={85} dataKey="value" label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`} labelLine={false}>
                  {uniDistrib.map((_, i) => <Cell key={i} fill={PIE_COLORS[i % PIE_COLORS.length]} />)}
                </Pie>
                <Tooltip />
              </RechartsPieChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-[220px] flex items-center justify-center text-slate-400 text-sm">Belum ada data siswa</div>
          )}
        </Card>

        <Card className="p-5">
          <h3 className="font-bold text-slate-800 mb-5">Perkembangan Proses SNBP {selectedYear}</h3>
          <div className="space-y-5">
            {progressBars.map(({ label, val, max }) => (
              <div key={label}>
                <div className="flex items-center justify-between text-sm mb-1.5">
                  <span className="text-slate-600 font-medium">{label}</span>
                  <span className="font-black text-slate-900">{val}</span>
                </div>
                <div className="h-2.5 bg-slate-100 rounded-full overflow-hidden">
                  <div className="h-full bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full transition-all" style={{ width: `${Math.min(100, (val / max) * 100)}%` }} />
                </div>
              </div>
            ))}
          </div>
        </Card>
      </div>

      {students.some(s => s.hasRasionalisasi && s.rankSekolah) && (
        <Card className="p-5">
          <h3 className="font-bold text-slate-800 mb-4">Top Peluang {selectedYear}</h3>
          <div className="space-y-2">
            {[...students]
              .filter(s => s.hasRasionalisasi && s.year === selectedYear && s.peluang1 !== undefined)
              .sort((a, b) => (b.peluang1 ?? 0) - (a.peluang1 ?? 0))
              .slice(0, 5)
              .map((s, i) => {
                const pb = peluangBadge(s.peluang1);
                return (
                  <div key={s.id} className="flex items-center gap-3 p-3 rounded-xl hover:bg-slate-50 transition-colors">
                    <div className={`w-7 h-7 rounded-full flex items-center justify-center text-xs font-black ${i === 0 ? 'bg-yellow-400 text-white' : i === 1 ? 'bg-slate-300 text-slate-700' : i === 2 ? 'bg-amber-600 text-white' : 'bg-slate-100 text-slate-500'}`}>
                      {i + 1}
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="font-semibold text-slate-900 text-sm truncate">{s.name}</p>
                      <p className="text-xs text-slate-400">{s.pil1uni} — {s.pil1jur}</p>
                    </div>
                    <span className={`text-sm font-black px-2.5 py-1 rounded-full ${pb.bgColor} ${pb.textColor}`}>{s.peluang1}%</span>
                    <Badge className={pb.bgColor + ' ' + pb.textColor + ' text-xs'}>{pb.label}</Badge>
                  </div>
                );
              })}
          </div>
        </Card>
      )}
    </div>
  );
}

// ─── Section: Daftar Siswa SNBP ───────────────────────────────────────────────

function DaftarSiswaSNBP({ students, onUpdate }: { students: SnbpStudent[]; onUpdate: (s: SnbpStudent[]) => void }) {
  const [search, setSearch] = useState('');
  const [filterYear, setFilterYear] = useState('all');
  const [detailSt, setDetailSt] = useState<SnbpStudent | null>(null);
  const [editOpen, setEditOpen] = useState(false);
  const [editSt, setEditSt] = useState<SnbpStudent | null>(null);
  const [form, setForm] = useState({ name: '', class: '', year: 2026, konsultan: '', pil1uni: '', pil1jur: '', pil2uni: '', pil2jur: '', statusPenerimaan: 'belum' as SnbpStudent['statusPenerimaan'], aktif: true });

  const filtered = useMemo(() => students.filter(s => {
    const q = search.toLowerCase();
    const match = s.name.toLowerCase().includes(q) || s.pil1uni.toLowerCase().includes(q) || s.pil1jur.toLowerCase().includes(q);
    const matchYear = filterYear === 'all' || String(s.year) === filterYear;
    return match && matchYear;
  }), [students, search, filterYear]);

  const openEdit = (s: SnbpStudent) => {
    setEditSt(s);
    setForm({ name: s.name, class: s.class, year: s.year, konsultan: s.konsultan, pil1uni: s.pil1uni, pil1jur: s.pil1jur, pil2uni: s.pil2uni, pil2jur: s.pil2jur, statusPenerimaan: s.statusPenerimaan, aktif: s.aktif });
    setEditOpen(true);
  };

  const handleSave = () => {
    if (!form.name.trim()) { toast.error('Nama siswa wajib diisi'); return; }
    onUpdate(students.map(s => s.id === editSt?.id ? { ...s, ...form } : s));
    toast.success('Data siswa SNBP diperbarui');
    setEditOpen(false);
  };

  const toggleAktif = (id: string) => {
    onUpdate(students.map(s => {
      if (s.id !== id) return s;
      toast.success(s.aktif ? 'Siswa dinonaktifkan dari SNBP' : 'Siswa diaktifkan di SNBP');
      return { ...s, aktif: !s.aktif };
    }));
  };

  const deleteSt = (id: string) => {
    onUpdate(students.filter(s => s.id !== id));
    setDetailSt(null);
    toast.success('Data siswa dihapus');
  };

  const setF = <K extends keyof typeof form>(k: K, v: (typeof form)[K]) => setForm(f => ({ ...f, [k]: v }));

  return (
    <div className="space-y-4">
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="flex-1 relative">
          <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
          <input className="w-full pl-9 pr-4 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" placeholder="Cari nama, universitas, atau jurusan..." value={search} onChange={e => setSearch(e.target.value)} />
        </div>
        <select className="border rounded-xl px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterYear} onChange={e => setFilterYear(e.target.value)}>
          <option value="all">Semua Tahun</option>
          {[2024, 2025, 2026, 2027].map(y => <option key={y} value={String(y)}>{y}</option>)}
        </select>
        <button onClick={() => toast.info('Export CSV disiapkan...')} className="flex items-center gap-2 px-4 py-2.5 border rounded-xl text-sm font-semibold hover:bg-slate-50 transition-colors">
          <Download className="w-4 h-4" /> Export
        </button>
      </div>

      <Card className="overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-slate-50">
                {['No', 'Nama', 'Kelas', 'Konsultan', 'Pilihan 1', 'Pilihan 2', 'Status Penerimaan', 'Aktif', 'Aksi'].map(h => (
                  <th key={h} className="text-left px-4 py-3 font-semibold text-slate-600 whitespace-nowrap">{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {filtered.length === 0 ? (
                <tr><td colSpan={9} className="px-4 py-12 text-center text-slate-400">Tidak ada data siswa.</td></tr>
              ) : filtered.map((s, i) => {
                const si = statusInfo(s.statusPenerimaan);
                return (
                  <tr key={s.id} className="border-b hover:bg-slate-50 transition-colors">
                    <td className="px-4 py-3 text-slate-500 font-mono text-xs">{i + 1}</td>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2.5">
                        <div className="w-8 h-8 rounded-full bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center text-white text-xs font-bold shrink-0">
                          {s.name.split(' ').slice(0, 2).map(w => w[0]).join('')}
                        </div>
                        <div>
                          <p className="font-semibold text-slate-900">{s.name}</p>
                          <p className="text-xs text-slate-400">Tahun {s.year}</p>
                        </div>
                      </div>
                    </td>
                    <td className="px-4 py-3 text-slate-600">{s.class}</td>
                    <td className="px-4 py-3 text-slate-600">{s.konsultan || '—'}</td>
                    <td className="px-4 py-3">
                      <p className="font-semibold text-indigo-700">{s.pil1uni || '—'}</p>
                      <p className="text-xs text-slate-400">{s.pil1jur || '—'}</p>
                    </td>
                    <td className="px-4 py-3">
                      <p className="font-semibold text-purple-700">{s.pil2uni || '—'}</p>
                      <p className="text-xs text-slate-400">{s.pil2jur || '—'}</p>
                    </td>
                    <td className="px-4 py-3">
                      <Badge className={`${si.cls} text-xs`}>{si.label}</Badge>
                    </td>
                    <td className="px-4 py-3">
                      <button onClick={() => toggleAktif(s.id)} className={`w-10 h-5 rounded-full transition-colors relative ${s.aktif ? 'bg-emerald-500' : 'bg-slate-200'}`}>
                        <div className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-all ${s.aktif ? 'left-5' : 'left-0.5'}`} />
                      </button>
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-1">
                        <button onClick={() => setDetailSt(s)} className="p-1.5 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-indigo-600 transition-colors"><Eye className="w-4 h-4" /></button>
                        <button onClick={() => openEdit(s)} className="p-1.5 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-slate-700 transition-colors"><Edit2 className="w-4 h-4" /></button>
                        <button onClick={() => deleteSt(s.id)} className="p-1.5 rounded-lg hover:bg-red-50 text-slate-400 hover:text-red-500 transition-colors"><Trash2 className="w-4 h-4" /></button>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
        <div className="px-4 py-3 bg-slate-50 border-t text-xs text-slate-400">Menampilkan {filtered.length} dari {students.length} siswa</div>
      </Card>

      {/* Detail Drawer */}
      {detailSt && (
        <div className="fixed inset-0 bg-black/40 flex justify-end z-50" onClick={() => setDetailSt(null)}>
          <div className="bg-white w-full max-w-sm h-full overflow-y-auto shadow-2xl" onClick={e => e.stopPropagation()}>
            <div className="sticky top-0 bg-white border-b p-5 flex items-center justify-between">
              <h3 className="font-bold text-slate-900">Detail Siswa SNBP</h3>
              <button onClick={() => setDetailSt(null)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-5 space-y-5">
              <div className="flex items-center gap-4">
                <div className="w-14 h-14 rounded-2xl bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center text-white text-xl font-black">
                  {detailSt.name.split(' ').slice(0, 2).map(w => w[0]).join('')}
                </div>
                <div>
                  <h4 className="text-lg font-black text-slate-900">{detailSt.name}</h4>
                  <p className="text-sm text-slate-400">{detailSt.class} · Tahun {detailSt.year}</p>
                  <Badge className={`mt-1 text-xs ${statusInfo(detailSt.statusPenerimaan).cls}`}>{statusInfo(detailSt.statusPenerimaan).label}</Badge>
                </div>
              </div>
              {[
                { label: 'Konsultan', val: detailSt.konsultan || '—' },
                { label: 'Pilihan 1', val: `${detailSt.pil1uni} — ${detailSt.pil1jur}` || '—' },
                { label: 'Pilihan 2', val: `${detailSt.pil2uni} — ${detailSt.pil2jur}` || '—' },
                { label: 'Peluang Pil. 1', val: detailSt.peluang1 ? `${detailSt.peluang1}%` : 'Belum dihitung' },
                { label: 'Peluang Pil. 2', val: detailSt.peluang2 ? `${detailSt.peluang2}%` : 'Belum dihitung' },
                { label: 'Rasionalisasi', val: detailSt.hasRasionalisasi ? 'Sudah selesai' : 'Belum dilakukan' },
                { label: 'Prestasi', val: `${detailSt.prestasi?.length ?? 0} entri` },
              ].map(({ label, val }) => (
                <div key={label} className="flex items-center justify-between py-2 border-b border-slate-100">
                  <span className="text-sm text-slate-500">{label}</span>
                  <span className="text-sm font-semibold text-slate-900 text-right max-w-[60%]">{val}</span>
                </div>
              ))}
              <div className="flex gap-2 pt-2">
                <button onClick={() => { setDetailSt(null); openEdit(detailSt); }} className="flex-1 py-2.5 rounded-xl border text-sm font-semibold hover:bg-slate-50 transition-colors">Edit</button>
                <button onClick={() => deleteSt(detailSt.id)} className="flex-1 py-2.5 rounded-xl bg-red-600 hover:bg-red-700 text-white text-sm font-semibold transition-colors">Hapus</button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Edit Modal */}
      {editOpen && editSt && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-2xl w-full max-w-lg shadow-2xl overflow-hidden">
            <div className="flex items-center justify-between p-6 border-b">
              <h3 className="font-bold text-slate-900">Edit Data Siswa SNBP</h3>
              <button onClick={() => setEditOpen(false)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-6 space-y-4 max-h-[60vh] overflow-y-auto">
              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Nama Lengkap</label>
                <input className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.name} onChange={e => setF('name', e.target.value)} />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Kelas</label>
                  <input className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.class} onChange={e => setF('class', e.target.value)} />
                </div>
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Tahun Lulus</label>
                  <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.year} onChange={e => setF('year', Number(e.target.value))}>
                    {[2025, 2026, 2027, 2028].map(y => <option key={y} value={y}>{y}</option>)}
                  </select>
                </div>
              </div>
              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Konsultan</label>
                <input className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.konsultan} onChange={e => setF('konsultan', e.target.value)} placeholder="Nama guru BK / konsultan" />
              </div>
              {(['1', '2'] as const).map(n => (
                <div key={n} className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Universitas Pilihan {n}</label>
                    <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={n === '1' ? form.pil1uni : form.pil2uni} onChange={e => { if (n === '1') { setF('pil1uni', e.target.value); setF('pil1jur', ''); } else { setF('pil2uni', e.target.value); setF('pil2jur', ''); } }}>
                      <option value="">Pilih PTN</option>
                      {PTN_LIST.map(p => <option key={p} value={p}>{p}</option>)}
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Jurusan Pilihan {n}</label>
                    <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={n === '1' ? form.pil1jur : form.pil2jur} onChange={e => n === '1' ? setF('pil1jur', e.target.value) : setF('pil2jur', e.target.value)}>
                      <option value="">Pilih Jurusan</option>
                      {(JURUSAN_BY_PTN[n === '1' ? form.pil1uni : form.pil2uni] ?? []).map(j => <option key={j} value={j}>{j}</option>)}
                    </select>
                  </div>
                </div>
              ))}
              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Status Penerimaan</label>
                <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.statusPenerimaan} onChange={e => setF('statusPenerimaan', e.target.value as SnbpStudent['statusPenerimaan'])}>
                  <option value="belum">Belum Diterima</option>
                  <option value="diterima-snbp">Diterima SNBP</option>
                  <option value="diterima-snbt">Diterima SNBT</option>
                  <option value="tidak">Tidak Diterima</option>
                </select>
              </div>
            </div>
            <div className="flex gap-3 p-6 border-t bg-slate-50">
              <button onClick={() => setEditOpen(false)} className="flex-1 py-2.5 rounded-xl border text-sm font-semibold hover:bg-white transition-colors">Batal</button>
              <button onClick={handleSave} className="flex-1 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-bold transition-colors flex items-center justify-center gap-2">
                <Save className="w-4 h-4" /> Simpan
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ─── Section: Rasionalisasi ───────────────────────────────────────────────────

const RADAR_SUBJECTS = ['B. Indonesia', 'Matematika', 'B. Inggris', 'Fisika', 'Biologi', 'Kimia'];

function RasionalisasiSection({ students, onUpdate }: { students: SnbpStudent[]; onUpdate: (s: SnbpStudent[]) => void }) {
  const [filterUni, setFilterUni] = useState('all');
  const [filterYear, setFilterYear] = useState('2026');
  const [detailSt, setDetailSt] = useState<SnbpStudent | null>(null);
  const [detailTab, setDetailTab] = useState<'pil1' | 'pil2'>('pil1');

  // wizard state
  const [showWizard, setShowWizard] = useState(false);
  const [step, setStep] = useState<1 | 2 | 3 | 4>(1);
  const [activeSem, setActiveSem] = useState('S1');
  const [wForm, setWForm] = useState({
    studentName: '', class: '', year: 2026, konsultan: '',
    pil1uni: '', pil1jur: '', pil2uni: '', pil2jur: '',
  });
  const [wPrestasi, setWPrestasi] = useState<Achievement[]>([]);
  const [wScores, setWScores] = useState<SemesterScores>(initSemScores());
  const [showAddPrestasi, setShowAddPrestasi] = useState(false);
  const [presForm, setPresForm] = useState({ nama: '', tingkat: 'provinsi' as Achievement['tingkat'], tahun: 2024, juara: '' });

  const rasStudents = useMemo(() => {
    let list = students.filter(s => s.hasRasionalisasi && (filterYear === 'all' || String(s.year) === filterYear));
    if (filterUni !== 'all') list = list.filter(s => s.pil1uni === filterUni || s.pil2uni === filterUni);
    return [...list].sort((a, b) => (a.rankSekolah ?? 99) - (b.rankSekolah ?? 99));
  }, [students, filterUni, filterYear]);

  const generateRank = () => {
    const ranked = [...students].filter(s => s.hasRasionalisasi);
    ranked.sort((a, b) => (b.peluang1 ?? 0) - (a.peluang1 ?? 0));
    const updated = students.map(s => {
      const ri = ranked.findIndex(r => r.id === s.id);
      return ri >= 0 ? { ...s, rankSekolah: ri + 1 } : s;
    });
    onUpdate(updated);
    toast.success('Ranking sekolah berhasil digenerate!');
  };

  const openWizard = () => {
    setStep(1);
    setWForm({ studentName: '', class: '', year: 2026, konsultan: '', pil1uni: '', pil1jur: '', pil2uni: '', pil2jur: '' });
    setWPrestasi([]);
    setWScores(initSemScores());
    setActiveSem('S1');
    setShowWizard(true);
  };

  const addPrestasi = () => {
    if (!presForm.nama) { toast.error('Nama prestasi wajib diisi'); return; }
    setWPrestasi(prev => [...prev, { ...presForm, id: `p${Date.now()}` }]);
    setPresForm({ nama: '', tingkat: 'provinsi', tahun: 2024, juara: '' });
    setShowAddPrestasi(false);
  };

  const finishWizard = () => {
    if (!wForm.studentName || !wForm.pil1uni || !wForm.pil1jur) { toast.error('Lengkapi data siswa dan pilihan universitas'); return; }
    const idx1 = calcIndexes(wScores, wPrestasi, wForm.pil1uni, wForm.pil1jur);
    const idx2 = calcIndexes(wScores, wPrestasi, wForm.pil2uni, wForm.pil2jur);
    const peluang1 = Math.round(Math.min(idx1.total, 100));
    const peluang2 = Math.round(Math.min(idx2.total, 100));
    const newSt: SnbpStudent = {
      id: `sn${Date.now()}`,
      name: wForm.studentName, class: wForm.class, year: wForm.year, konsultan: wForm.konsultan,
      pil1uni: wForm.pil1uni, pil1jur: wForm.pil1jur, pil2uni: wForm.pil2uni, pil2jur: wForm.pil2jur,
      statusPenerimaan: 'belum', aktif: true,
      hasRasionalisasi: true, scores: wScores, prestasi: wPrestasi,
      indexes1: idx1, indexes2: idx2, peluang1, peluang2,
    };
    onUpdate([...students, newSt]);
    setShowWizard(false);
    toast.success(`Rasionalisasi ${wForm.studentName} selesai! Peluang Pil.1: ${peluang1}%`);
  };

  const computedStep4 = useMemo(() => {
    const idx1 = calcIndexes(wScores, wPrestasi, wForm.pil1uni, wForm.pil1jur);
    const idx2 = calcIndexes(wScores, wPrestasi, wForm.pil2uni, wForm.pil2jur);
    const avg = avgRapor(wScores);
    const pm1 = POSMIN_DB.find(p => p.uni === wForm.pil1uni && p.jurusan === wForm.pil1jur);
    const pm2 = POSMIN_DB.find(p => p.uni === wForm.pil2uni && p.jurusan === wForm.pil2jur);
    const altMinat = POSMIN_DB.filter(p => p.jurusan === wForm.pil1jur && p.uni !== wForm.pil1uni && p.uni !== wForm.pil2uni).slice(0, 2);
    const altRumpun = POSMIN_DB.filter(p => pm1 && p.rumpun === pm1.rumpun && p.uni !== wForm.pil1uni && p.jurusan !== wForm.pil1jur && p.jurusan !== wForm.pil2jur).slice(0, 2);
    return { idx1, idx2, avg, pm1, pm2, altMinat, altRumpun };
  }, [wScores, wPrestasi, wForm]);

  const detailIdxs = detailTab === 'pil1' ? detailSt?.indexes1 : detailSt?.indexes2;
  const detailUni = detailSt ? (detailTab === 'pil1' ? detailSt.pil1uni : detailSt.pil2uni) : '';
  const detailJur = detailSt ? (detailTab === 'pil1' ? detailSt.pil1jur : detailSt.pil2jur) : '';
  const detailPM = POSMIN_DB.find(p => p.uni === detailUni && p.jurusan === detailJur);
  const detailPeluang = detailTab === 'pil1' ? detailSt?.peluang1 : detailSt?.peluang2;
  const pb = peluangBadge(detailPeluang);

  const radarData = useMemo(() => {
    if (!detailSt?.scores) return [];
    return RADAR_SUBJECTS.map(subj => {
      let total = 0, count = 0;
      for (const sem of Object.values(detailSt.scores!)) {
        const s = sem[subj];
        if (s?.active && s.value > 0) { total += s.value; count++; }
      }
      const pencapaian = count > 0 ? Math.round(total / count) : 0;
      const saran = detailPM?.minRapor ?? 88;
      return { subject: subj.replace('B. ', 'B.'), pencapaian, saran };
    });
  }, [detailSt, detailPM]);

  return (
    <div className="space-y-4">
      {/* Toolbar */}
      <div className="flex flex-col sm:flex-row gap-3">
        <select className="border rounded-xl px-3 py-2.5 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterYear} onChange={e => setFilterYear(e.target.value)}>
          {['all', '2025', '2026', '2027'].map(y => <option key={y} value={y}>{y === 'all' ? 'Semua Tahun' : y}</option>)}
        </select>
        <select className="border rounded-xl px-3 py-2.5 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterUni} onChange={e => setFilterUni(e.target.value)}>
          <option value="all">Semua Universitas</option>
          {PTN_LIST.map(u => <option key={u} value={u}>{u}</option>)}
        </select>
        <div className="flex-1" />
        <button onClick={generateRank} className="flex items-center gap-2 px-4 py-2.5 border rounded-xl text-sm font-semibold hover:bg-slate-50 transition-colors">
          <RefreshCw className="w-4 h-4" /> Generate Rank
        </button>
        <button onClick={openWizard} className="flex items-center gap-2 px-4 py-2.5 bg-indigo-600 hover:bg-indigo-700 text-white rounded-xl text-sm font-bold transition-colors">
          <Plus className="w-4 h-4" /> Tambah Data
        </button>
      </div>

      {/* Student cards */}
      <div className="space-y-3">
        {rasStudents.length === 0 ? (
          <Card className="p-12 text-center">
            <Target className="w-12 h-12 text-slate-300 mx-auto mb-3" />
            <p className="text-slate-500 font-semibold">Belum ada data rasionalisasi</p>
            <p className="text-slate-400 text-sm mt-1">Klik "Tambah Data" untuk memulai rasionalisasi siswa</p>
          </Card>
        ) : rasStudents.map(s => {
          const pb1 = peluangBadge(s.peluang1);
          const pb2 = peluangBadge(s.peluang2);
          return (
            <Card key={s.id} className="p-4 hover:shadow-md transition-shadow">
              <div className="flex items-start gap-4">
                <div className="flex flex-col items-center gap-1 shrink-0">
                  <div className={`w-10 h-10 rounded-xl flex items-center justify-center text-sm font-black ${s.rankSekolah === 1 ? 'bg-yellow-100 text-yellow-600' : s.rankSekolah === 2 ? 'bg-slate-200 text-slate-600' : s.rankSekolah === 3 ? 'bg-amber-100 text-amber-700' : 'bg-slate-100 text-slate-500'}`}>
                    #{s.rankSekolah ?? '—'}
                  </div>
                  <span className="text-[10px] text-slate-400 font-semibold">RANK</span>
                </div>

                <div className="flex-1 min-w-0">
                  <div className="flex items-start justify-between flex-wrap gap-2">
                    <div>
                      <h4 className="font-bold text-slate-900">{s.name}</h4>
                      <p className="text-xs text-slate-400">{s.class} · Tahun {s.year} · {s.konsultan}</p>
                    </div>
                    <Badge className={`${statusInfo(s.statusPenerimaan).cls} text-xs shrink-0`}>{statusInfo(s.statusPenerimaan).label}</Badge>
                  </div>

                  <div className="mt-3 grid grid-cols-1 sm:grid-cols-2 gap-3">
                    {[
                      { label: 'Pilihan 1', uni: s.pil1uni, jur: s.pil1jur, peluang: s.peluang1, pb: pb1 },
                      { label: 'Pilihan 2', uni: s.pil2uni, jur: s.pil2jur, peluang: s.peluang2, pb: pb2 },
                    ].map(({ label, uni, jur, peluang, pb }) => (
                      <div key={label} className="flex items-center gap-3 p-3 bg-slate-50 rounded-xl">
                        <div className="flex-1 min-w-0">
                          <p className="text-[10px] text-slate-400 font-bold uppercase tracking-wide">{label}</p>
                          <p className="font-bold text-slate-900 text-sm">{uni}</p>
                          <p className="text-xs text-slate-500 truncate">{jur}</p>
                        </div>
                        <div className="text-right shrink-0">
                          <p className={`text-xl font-black ${pb.textColor}`}>{peluang ?? '—'}%</p>
                          <span className={`text-[10px] font-bold px-1.5 py-0.5 rounded ${pb.bgColor} ${pb.textColor}`}>{pb.label}</span>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                <button onClick={() => { setDetailSt(s); setDetailTab('pil1'); }} className="shrink-0 flex items-center gap-1.5 px-3 py-2 border rounded-xl text-sm font-semibold hover:bg-indigo-50 hover:border-indigo-300 hover:text-indigo-700 transition-all">
                  Detail <ChevronRight className="w-4 h-4" />
                </button>
              </div>
            </Card>
          );
        })}
      </div>

      {/* ─── Detail Panel ─── */}
      {detailSt && (
        <div className="fixed inset-0 bg-black/40 flex justify-end z-50" onClick={() => setDetailSt(null)}>
          <div className="bg-white w-full max-w-lg h-full overflow-y-auto shadow-2xl flex flex-col" onClick={e => e.stopPropagation()}>
            <div className="sticky top-0 bg-white border-b z-10">
              <div className="p-5 flex items-center justify-between">
                <div>
                  <h3 className="font-bold text-slate-900">{detailSt.name}</h3>
                  <p className="text-xs text-slate-400">{detailSt.class} · SMA Nusantara Sejahtera</p>
                </div>
                <button onClick={() => setDetailSt(null)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
              </div>
              <div className="flex border-t">
                {(['pil1', 'pil2'] as const).map(t => (
                  <button key={t} onClick={() => setDetailTab(t)} className={`flex-1 py-2.5 text-sm font-semibold border-b-2 transition-colors ${detailTab === t ? 'border-indigo-600 text-indigo-600' : 'border-transparent text-slate-400 hover:text-slate-600'}`}>
                    {t === 'pil1' ? `Pilihan 1 — ${detailSt.pil1uni}` : `Pilihan 2 — ${detailSt.pil2uni}`}
                  </button>
                ))}
              </div>
            </div>

            <div className="flex-1 p-5 space-y-5">
              {/* Peluang badge */}
              <div className={`p-4 rounded-2xl ${pb.bgColor} flex items-center justify-between`}>
                <div>
                  <p className="text-xs font-bold text-slate-500 uppercase tracking-wide mb-0.5">Peluang Kompetisi</p>
                  <p className={`text-4xl font-black ${pb.textColor}`}>{detailPeluang ?? '—'}%</p>
                  <p className={`text-sm font-bold ${pb.textColor} mt-0.5`}>{pb.label}</p>
                </div>
                <div className={`w-16 h-16 rounded-2xl flex items-center justify-center ${pb.bgColor} border-2 ${detailPeluang && detailPeluang >= 70 ? 'border-emerald-400' : detailPeluang && detailPeluang >= 50 ? 'border-amber-400' : 'border-red-400'}`}>
                  <Target className={`w-8 h-8 ${pb.textColor}`} />
                </div>
              </div>

              {/* Skor Bobot */}
              <div>
                <h4 className="font-bold text-slate-800 mb-3">Skor Bobot Rasionalisasi</h4>
                <div className="space-y-2">
                  {detailIdxs && [
                    { label: 'Index Nilai', val: detailIdxs.nilai, max: 50, color: 'bg-indigo-500' },
                    { label: 'Index SNBP', val: detailIdxs.snbp, max: 10, color: 'bg-purple-500' },
                    { label: 'Index SNBT', val: detailIdxs.snbt, max: 10, color: 'bg-blue-500' },
                    { label: 'Index Prestasi', val: detailIdxs.prestasi, max: 10, color: 'bg-amber-500' },
                    { label: 'Index Universitas', val: detailIdxs.universitas, max: 10, color: 'bg-cyan-500' },
                    { label: 'Index Jurusan', val: detailIdxs.jurusan, max: 5, color: 'bg-rose-500' },
                    { label: 'Index Akreditasi', val: detailIdxs.akreditasi, max: 5, color: 'bg-emerald-500' },
                  ].map(({ label, val, max, color }) => (
                    <div key={label} className="flex items-center gap-3">
                      <span className="text-xs text-slate-500 w-32 shrink-0">{label}</span>
                      <div className="flex-1 h-2 bg-slate-100 rounded-full overflow-hidden">
                        <div className={`h-full ${color} rounded-full`} style={{ width: `${Math.min(100, (val / max) * 100)}%` }} />
                      </div>
                      <span className="text-xs font-black text-slate-800 w-10 text-right">{val}%</span>
                    </div>
                  ))}
                  <div className="flex items-center justify-between pt-2 border-t mt-3">
                    <span className="text-sm font-bold text-slate-700">Total Index</span>
                    <span className={`text-lg font-black ${pb.textColor}`}>{detailIdxs?.total.toFixed(2)}%</span>
                  </div>
                </div>
              </div>

              {/* Data Detail Jurusan */}
              {detailPM && (
                <div>
                  <h4 className="font-bold text-slate-800 mb-3">Data Detail — {detailUni} {detailJur}</h4>
                  <div className="grid grid-cols-2 gap-2">
                    {[
                      { label: 'Min. Nilai Rapor', val: detailPM.minRapor },
                      { label: 'Min. Nilai UTBK', val: detailPM.minUTBK },
                      { label: 'Kuota SNBP', val: `${detailPM.kuotaSNBP} kursi` },
                      { label: 'Peminat SNBP', val: `${detailPM.peminatSNBP.toLocaleString()} orang` },
                      { label: 'Persaingan', val: `1:${Math.round(detailPM.peminatSNBP / detailPM.kuotaSNBP)}` },
                      { label: 'Rumpun', val: detailPM.rumpun },
                    ].map(({ label, val }) => (
                      <div key={label} className="p-2.5 bg-slate-50 rounded-xl">
                        <p className="text-xs text-slate-400 mb-0.5">{label}</p>
                        <p className="text-sm font-bold text-slate-800">{val}</p>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Radar Chart */}
              {detailSt.scores && (
                <div>
                  <h4 className="font-bold text-slate-800 mb-3">Pencapaian vs Minimum Prodi</h4>
                  <ResponsiveContainer width="100%" height={220}>
                    <RadarChart data={radarData}>
                      <PolarGrid stroke="#e2e8f0" />
                      <PolarAngleAxis dataKey="subject" tick={{ fontSize: 11, fill: '#64748b' }} />
                      <PolarRadiusAxis angle={90} domain={[70, 100]} tick={{ fontSize: 9 }} />
                      <Radar name="Pencapaian" dataKey="pencapaian" stroke="#6366f1" fill="#6366f1" fillOpacity={0.35} />
                      <Radar name="Minimum Prodi" dataKey="saran" stroke="#f59e0b" fill="#f59e0b" fillOpacity={0.1} />
                      <Legend wrapperStyle={{ fontSize: 11 }} />
                      <Tooltip />
                    </RadarChart>
                  </ResponsiveContainer>
                </div>
              )}

              {/* Rekap per mapel */}
              {detailSt.scores && (
                <div>
                  <h4 className="font-bold text-slate-800 mb-3">Rekap Perbandingan Nilai</h4>
                  <div className="border rounded-xl overflow-hidden">
                    <table className="w-full text-xs">
                      <thead>
                        <tr className="bg-slate-50 border-b">
                          <th className="text-left px-3 py-2.5 font-semibold text-slate-600">Mata Pelajaran</th>
                          <th className="text-center px-3 py-2.5 font-semibold text-slate-600">Pencapaian</th>
                          <th className="text-center px-3 py-2.5 font-semibold text-slate-600">Min. Prodi</th>
                          <th className="text-center px-3 py-2.5 font-semibold text-slate-600">Status</th>
                        </tr>
                      </thead>
                      <tbody>
                        {SUBJECTS.filter(subj => {
                          const s1 = detailSt.scores?.['S1']?.[subj];
                          return s1?.active && s1.value > 0;
                        }).slice(0, 8).map(subj => {
                          let total = 0, count = 0;
                          for (const sem of Object.values(detailSt.scores!)) {
                            const s = sem[subj];
                            if (s?.active && s.value > 0) { total += s.value; count++; }
                          }
                          const avg = count > 0 ? Math.round(total / count) : 0;
                          const min = detailPM?.minRapor ? Math.round(detailPM.minRapor * 0.95) : 85;
                          const ok = avg >= min;
                          return (
                            <tr key={subj} className="border-b hover:bg-slate-50">
                              <td className="px-3 py-2 font-medium text-slate-700">{subj}</td>
                              <td className="px-3 py-2 text-center font-bold text-slate-900">{avg}</td>
                              <td className="px-3 py-2 text-center text-slate-500">{min}</td>
                              <td className="px-3 py-2 text-center">
                                {ok ? <CheckCircle2 className="w-4 h-4 text-emerald-500 mx-auto" /> : <AlertCircle className="w-4 h-4 text-amber-500 mx-auto" />}
                              </td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}

              <button onClick={() => toast.info('Menyiapkan laporan PDF...')} className="w-full py-3 rounded-xl border-2 border-dashed border-indigo-300 text-indigo-600 hover:bg-indigo-50 text-sm font-bold transition-colors flex items-center justify-center gap-2">
                <Printer className="w-4 h-4" /> Cetak Laporan Rasionalisasi
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ─── Wizard ─── */}
      {showWizard && (
        <div className="fixed inset-0 bg-black/50 flex items-start justify-center z-50 p-4 overflow-y-auto">
          <div className="bg-white rounded-2xl w-full max-w-2xl my-4 shadow-2xl overflow-hidden">
            {/* Header */}
            <div className="bg-gradient-to-r from-indigo-600 to-purple-600 px-6 py-5 text-white">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-black">Tambah Data Rasionalisasi</h3>
                <button onClick={() => setShowWizard(false)} className="p-2 rounded-xl hover:bg-white/20 transition-colors"><X className="w-5 h-5" /></button>
              </div>
              {/* Stepper */}
              <div className="flex items-center gap-0">
                {[1, 2, 3, 4].map((s, i) => (
                  <div key={s} className="flex items-center flex-1">
                    <div className="flex flex-col items-center flex-1">
                      <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-black border-2 transition-all ${step > s ? 'bg-white text-indigo-600 border-white' : step === s ? 'bg-white/20 text-white border-white' : 'bg-transparent text-white/50 border-white/30'}`}>
                        {step > s ? <CheckCircle2 className="w-4 h-4" /> : s}
                      </div>
                      <span className="text-[10px] text-white/70 mt-1 text-center leading-tight">{['Data Siswa', 'Prestasi', 'Nilai Rapor', 'Resume'][i]}</span>
                    </div>
                    {i < 3 && <div className={`h-0.5 flex-1 mb-4 transition-all ${step > s + 1 ? 'bg-white' : 'bg-white/20'}`} />}
                  </div>
                ))}
              </div>
            </div>

            {/* Step content */}
            <div className="p-6 max-h-[55vh] overflow-y-auto">
              {step === 1 && (
                <div className="space-y-4">
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Nama Siswa *</label>
                    <input className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={wForm.studentName} onChange={e => setWForm(f => ({ ...f, studentName: e.target.value }))} placeholder="Nama lengkap siswa" />
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Kelas</label>
                      <input className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={wForm.class} onChange={e => setWForm(f => ({ ...f, class: e.target.value }))} placeholder="XII IPA 1" />
                    </div>
                    <div>
                      <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Tahun Lulus</label>
                      <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={wForm.year} onChange={e => setWForm(f => ({ ...f, year: Number(e.target.value) }))}>
                        {[2025, 2026, 2027].map(y => <option key={y} value={y}>{y}</option>)}
                      </select>
                    </div>
                  </div>
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Konsultan / Guru BK</label>
                    <input className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={wForm.konsultan} onChange={e => setWForm(f => ({ ...f, konsultan: e.target.value }))} placeholder="Nama konsultan" />
                  </div>
                  {(['1', '2'] as const).map(n => (
                    <div key={n} className="grid grid-cols-2 gap-3 p-4 bg-slate-50 rounded-xl">
                      <p className="col-span-2 text-xs font-bold text-slate-700 uppercase tracking-wide">Pilihan Universitas {n} {n === '1' && '*'}</p>
                      <div>
                        <label className="text-xs text-slate-500 block mb-1">Universitas</label>
                        <select className="w-full px-3 py-2 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300 bg-white" value={n === '1' ? wForm.pil1uni : wForm.pil2uni} onChange={e => { if (n === '1') setWForm(f => ({ ...f, pil1uni: e.target.value, pil1jur: '' })); else setWForm(f => ({ ...f, pil2uni: e.target.value, pil2jur: '' })); }}>
                          <option value="">Pilih PTN</option>
                          {PTN_LIST.map(p => <option key={p} value={p}>{p}</option>)}
                        </select>
                      </div>
                      <div>
                        <label className="text-xs text-slate-500 block mb-1">Jurusan / Prodi</label>
                        <select className="w-full px-3 py-2 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300 bg-white" value={n === '1' ? wForm.pil1jur : wForm.pil2jur} onChange={e => n === '1' ? setWForm(f => ({ ...f, pil1jur: e.target.value })) : setWForm(f => ({ ...f, pil2jur: e.target.value }))}>
                          <option value="">Pilih Prodi</option>
                          {(JURUSAN_BY_PTN[n === '1' ? wForm.pil1uni : wForm.pil2uni] ?? []).map(j => <option key={j} value={j}>{j}</option>)}
                        </select>
                      </div>
                    </div>
                  ))}
                </div>
              )}

              {step === 2 && (
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <p className="text-sm font-semibold text-slate-700">{wPrestasi.length} prestasi ditambahkan</p>
                    <button onClick={() => setShowAddPrestasi(true)} className="flex items-center gap-1.5 px-3 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-xl text-sm font-bold transition-colors">
                      <Plus className="w-4 h-4" /> Tambah Prestasi
                    </button>
                  </div>
                  {wPrestasi.length === 0 ? (
                    <div className="py-12 text-center border-2 border-dashed border-slate-200 rounded-2xl">
                      <Star className="w-10 h-10 text-slate-300 mx-auto mb-2" />
                      <p className="text-slate-400 text-sm font-semibold">Belum ada prestasi</p>
                      <p className="text-slate-300 text-xs mt-1">Prestasi akan meningkatkan Index Pendukung</p>
                    </div>
                  ) : (
                    <div className="space-y-2">
                      {wPrestasi.map(p => (
                        <div key={p.id} className="flex items-center gap-3 p-3 bg-slate-50 rounded-xl">
                          <Medal className="w-5 h-5 text-amber-500 shrink-0" />
                          <div className="flex-1 min-w-0">
                            <p className="font-semibold text-slate-900 text-sm">{p.nama}</p>
                            <p className="text-xs text-slate-400">{p.juara} · Tk. {p.tingkat} · {p.tahun}</p>
                          </div>
                          <button onClick={() => setWPrestasi(prev => prev.filter(x => x.id !== p.id))} className="p-1.5 rounded-lg hover:bg-red-50 text-slate-400 hover:text-red-500 transition-colors"><X className="w-4 h-4" /></button>
                        </div>
                      ))}
                    </div>
                  )}
                  {showAddPrestasi && (
                    <div className="p-4 bg-indigo-50 border border-indigo-200 rounded-2xl space-y-3">
                      <p className="font-bold text-indigo-800 text-sm">Tambah Prestasi</p>
                      <input className="w-full px-3 py-2 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300 bg-white" placeholder="Nama prestasi / lomba" value={presForm.nama} onChange={e => setPresForm(f => ({ ...f, nama: e.target.value }))} />
                      <div className="grid grid-cols-3 gap-2">
                        <select className="px-3 py-2 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300 bg-white" value={presForm.tingkat} onChange={e => setPresForm(f => ({ ...f, tingkat: e.target.value as Achievement['tingkat'] }))}>
                          {['sekolah', 'kab-kota', 'provinsi', 'nasional', 'internasional'].map(t => <option key={t} value={t}>{t}</option>)}
                        </select>
                        <input className="px-3 py-2 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300 bg-white" placeholder="Juara / Finalis" value={presForm.juara} onChange={e => setPresForm(f => ({ ...f, juara: e.target.value }))} />
                        <input className="px-3 py-2 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300 bg-white" type="number" value={presForm.tahun} onChange={e => setPresForm(f => ({ ...f, tahun: Number(e.target.value) }))} />
                      </div>
                      <div className="flex gap-2">
                        <button onClick={() => setShowAddPrestasi(false)} className="flex-1 py-2 rounded-xl border text-sm font-semibold">Batal</button>
                        <button onClick={addPrestasi} className="flex-1 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-bold">Tambah</button>
                      </div>
                    </div>
                  )}
                </div>
              )}

              {step === 3 && (
                <div className="space-y-4">
                  <div className="flex gap-2 flex-wrap">
                    {SEMESTERS.map(sem => (
                      <button key={sem} onClick={() => setActiveSem(sem)} className={`px-3 py-1.5 rounded-full text-sm font-bold border transition-all ${activeSem === sem ? 'bg-indigo-600 text-white border-indigo-600' : 'bg-white text-slate-600 border-slate-200 hover:border-indigo-300'}`}>{sem}</button>
                    ))}
                  </div>
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                    {SUBJECTS.map(subj => {
                      const entry = wScores[activeSem]?.[subj] ?? { value: 0, active: true };
                      return (
                        <div key={subj} className={`flex items-center gap-2 p-2.5 rounded-xl border transition-colors ${entry.active ? 'bg-white' : 'bg-slate-50 opacity-60'}`}>
                          <span className="text-xs font-medium text-slate-700 flex-1 truncate">{subj}</span>
                          <input
                            type="number" min={0} max={100} disabled={!entry.active}
                            className="w-16 px-2 py-1 border rounded-lg text-sm text-center font-bold focus:outline-none focus:ring-1 focus:ring-indigo-300 disabled:bg-slate-100 disabled:text-slate-400"
                            value={entry.value || ''}
                            onChange={e => setWScores(prev => ({ ...prev, [activeSem]: { ...prev[activeSem], [subj]: { ...entry, value: Number(e.target.value) } } }))}
                          />
                          <button
                            onClick={() => setWScores(prev => ({ ...prev, [activeSem]: { ...prev[activeSem], [subj]: { ...entry, active: !entry.active } } }))}
                            className={`w-12 h-6 rounded-full transition-colors relative shrink-0 ${entry.active ? 'bg-emerald-500' : 'bg-slate-200'}`}
                          >
                            <div className={`absolute top-1 w-4 h-4 rounded-full bg-white shadow transition-all ${entry.active ? 'left-7' : 'left-1'}`} />
                          </button>
                        </div>
                      );
                    })}
                  </div>
                  <p className="text-xs text-slate-400 text-center">Toggle Off untuk mata pelajaran yang tidak relevan / tidak diambil</p>
                </div>
              )}

              {step === 4 && (
                <div className="space-y-5">
                  <div className="p-4 bg-indigo-50 border border-indigo-200 rounded-xl flex items-center gap-3">
                    <CheckCircle2 className="w-5 h-5 text-indigo-600 shrink-0" />
                    <div>
                      <p className="font-bold text-indigo-800 text-sm">Rasionalisasi siap!</p>
                      <p className="text-xs text-indigo-600">Rata-rata rapor: <strong>{computedStep4.avg.toFixed(2)}</strong> · Prestasi: <strong>{wPrestasi.length} entri</strong></p>
                    </div>
                  </div>
                  <div>
                    <h4 className="font-bold text-slate-800 mb-3">Resume Peluang</h4>
                    <div className="border rounded-xl overflow-hidden">
                      <table className="w-full text-xs">
                        <thead>
                          <tr className="bg-slate-50 border-b">
                            <th className="text-left px-3 py-2.5 font-semibold text-slate-600">Jurusan</th>
                            <th className="text-center px-2 py-2.5 font-semibold text-slate-600">Min Rapor</th>
                            <th className="text-center px-2 py-2.5 font-semibold text-slate-600">Rapor Saya</th>
                            <th className="text-center px-2 py-2.5 font-semibold text-slate-600">Index Nilai</th>
                            <th className="text-center px-2 py-2.5 font-semibold text-slate-600">Total</th>
                          </tr>
                        </thead>
                        <tbody>
                          {[
                            { label: `Pil.1 — ${wForm.pil1uni}`, jur: wForm.pil1jur, pm: computedStep4.pm1, idx: computedStep4.idx1 },
                            { label: `Pil.2 — ${wForm.pil2uni}`, jur: wForm.pil2jur, pm: computedStep4.pm2, idx: computedStep4.idx2 },
                          ].map(({ label, jur, pm, idx }) => {
                            const pct = Math.min(Math.round(idx.total), 100);
                            const pb = peluangBadge(pct);
                            return (
                              <tr key={label} className="border-b bg-white hover:bg-slate-50">
                                <td className="px-3 py-3">
                                  <p className="font-bold text-slate-800">{label}</p>
                                  <p className="text-slate-400">{jur}</p>
                                </td>
                                <td className="px-2 py-3 text-center text-slate-600">{pm?.minRapor ?? '—'}</td>
                                <td className="px-2 py-3 text-center font-bold text-slate-900">{computedStep4.avg.toFixed(1)}</td>
                                <td className="px-2 py-3 text-center text-indigo-600 font-semibold">{idx.nilai.toFixed(1)}%</td>
                                <td className="px-2 py-3 text-center">
                                  <span className={`font-black text-sm px-2 py-0.5 rounded-full ${pb.bgColor} ${pb.textColor}`}>{pct}%</span>
                                </td>
                              </tr>
                            );
                          })}
                          {computedStep4.altMinat.length > 0 && (
                            <>
                              <tr><td colSpan={5} className="px-3 py-2 bg-amber-50 text-xs font-bold text-amber-700 uppercase tracking-wide">Alternatif Sesuai Minat Jurusan</td></tr>
                              {computedStep4.altMinat.map(pm => {
                                const idx = calcIndexes(wScores, wPrestasi, pm.uni, pm.jurusan);
                                const pct = Math.min(Math.round(idx.total), 100);
                                const pb = peluangBadge(pct);
                                return (
                                  <tr key={pm.id} className="border-b bg-amber-50/30">
                                    <td className="px-3 py-3">
                                      <p className="font-semibold text-slate-700">{pm.uni}</p>
                                      <p className="text-slate-400">{pm.jurusan}</p>
                                    </td>
                                    <td className="px-2 py-3 text-center text-slate-600">{pm.minRapor}</td>
                                    <td className="px-2 py-3 text-center font-bold text-slate-900">{computedStep4.avg.toFixed(1)}</td>
                                    <td className="px-2 py-3 text-center text-indigo-600 font-semibold">{idx.nilai.toFixed(1)}%</td>
                                    <td className="px-2 py-3 text-center">
                                      <span className={`font-black text-sm px-2 py-0.5 rounded-full ${pb.bgColor} ${pb.textColor}`}>{pct}%</span>
                                    </td>
                                  </tr>
                                );
                              })}
                            </>
                          )}
                          {computedStep4.altRumpun.length > 0 && (
                            <>
                              <tr><td colSpan={5} className="px-3 py-2 bg-purple-50 text-xs font-bold text-purple-700 uppercase tracking-wide">Alternatif Sesuai Rumpun</td></tr>
                              {computedStep4.altRumpun.map(pm => {
                                const idx = calcIndexes(wScores, wPrestasi, pm.uni, pm.jurusan);
                                const pct = Math.min(Math.round(idx.total), 100);
                                const pb = peluangBadge(pct);
                                return (
                                  <tr key={pm.id} className="border-b bg-purple-50/20">
                                    <td className="px-3 py-3">
                                      <p className="font-semibold text-slate-700">{pm.uni}</p>
                                      <p className="text-slate-400">{pm.jurusan}</p>
                                    </td>
                                    <td className="px-2 py-3 text-center text-slate-600">{pm.minRapor}</td>
                                    <td className="px-2 py-3 text-center font-bold text-slate-900">{computedStep4.avg.toFixed(1)}</td>
                                    <td className="px-2 py-3 text-center text-indigo-600 font-semibold">{idx.nilai.toFixed(1)}%</td>
                                    <td className="px-2 py-3 text-center">
                                      <span className={`font-black text-sm px-2 py-0.5 rounded-full ${pb.bgColor} ${pb.textColor}`}>{pct}%</span>
                                    </td>
                                  </tr>
                                );
                              })}
                            </>
                          )}
                        </tbody>
                      </table>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* Wizard footer */}
            <div className="flex gap-3 p-6 border-t bg-slate-50">
              {step > 1 && (
                <button onClick={() => setStep(s => (s - 1) as 1|2|3|4)} className="flex items-center gap-2 px-4 py-2.5 border rounded-xl text-sm font-semibold hover:bg-white transition-colors">
                  <ChevronLeft className="w-4 h-4" /> Kembali
                </button>
              )}
              <div className="flex-1" />
              {step < 4 ? (
                <button onClick={() => { if (step === 1 && !wForm.studentName) { toast.error('Nama siswa wajib diisi'); return; } setStep(s => (s + 1) as 1|2|3|4); }} className="flex items-center gap-2 px-5 py-2.5 bg-indigo-600 hover:bg-indigo-700 text-white rounded-xl text-sm font-bold transition-colors">
                  Selanjutnya <ChevronRight className="w-4 h-4" />
                </button>
              ) : (
                <button onClick={finishWizard} className="flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-700 hover:to-purple-700 text-white rounded-xl text-sm font-black transition-all shadow-lg">
                  <CheckCircle2 className="w-4 h-4" /> Selesai & Simpan
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ─── Section: Jurusan Kaka Kelas ──────────────────────────────────────────────

function JurusanKakaKelas() {
  const [alumni, setAlumni] = useState<AlumniRecord[]>(INIT_ALUMNI);
  const [filterUni, setFilterUni] = useState('all');
  const [activeTab, setActiveTab] = useState<'rapor' | 'utbk'>('rapor');
  const [addOpen, setAddOpen] = useState(false);
  const [form, setForm] = useState({ name: '', uni: '', jurusan: '', year: 2024, pmRapor: 0, pmUTBK: 0 });

  const filtered = useMemo(() => alumni.filter(a => filterUni === 'all' || a.uni === filterUni), [alumni, filterUni]);
  const unis = [...new Set(alumni.map(a => a.uni))];

  const handleAdd = () => {
    if (!form.name || !form.uni || !form.jurusan) { toast.error('Lengkapi semua data'); return; }
    setAlumni(prev => [...prev, { ...form, id: `a${Date.now()}` }]);
    setAddOpen(false);
    setForm({ name: '', uni: '', jurusan: '', year: 2024, pmRapor: 0, pmUTBK: 0 });
    toast.success('Data alumni ditambahkan');
  };

  return (
    <div className="space-y-4">
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="flex gap-2">
          {(['rapor', 'utbk'] as const).map(t => (
            <button key={t} onClick={() => setActiveTab(t)} className={`px-4 py-2 rounded-xl text-sm font-bold border transition-all ${activeTab === t ? 'bg-indigo-600 text-white border-indigo-600' : 'bg-white text-slate-600 border-slate-200 hover:border-indigo-300'}`}>
              PM {t === 'rapor' ? 'Rapor Alumni' : 'UTBK Alumni'}
            </button>
          ))}
        </div>
        <select className="border rounded-xl px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterUni} onChange={e => setFilterUni(e.target.value)}>
          <option value="all">Semua Universitas</option>
          {unis.map(u => <option key={u} value={u}>{u}</option>)}
        </select>
        <div className="flex-1" />
        <button onClick={() => setAddOpen(true)} className="flex items-center gap-2 px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-xl text-sm font-bold transition-colors">
          <Plus className="w-4 h-4" /> Tambah Alumni
        </button>
      </div>

      <Card className="overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-slate-50">
                {['No', 'Nama Alumni', 'Universitas', 'Jurusan', 'Tahun', activeTab === 'rapor' ? 'PM Rapor' : 'PM UTBK', 'Aksi'].map(h => (
                  <th key={h} className="text-left px-4 py-3 font-semibold text-slate-600">{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {filtered.map((a, i) => (
                <tr key={a.id} className="border-b hover:bg-slate-50 transition-colors">
                  <td className="px-4 py-3 text-slate-400 text-xs font-mono">{i + 1}</td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2.5">
                      <div className="w-8 h-8 rounded-full bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center text-white text-xs font-bold shrink-0">
                        {a.name[0]}
                      </div>
                      <span className="font-semibold text-slate-900">{a.name}</span>
                    </div>
                  </td>
                  <td className="px-4 py-3 font-semibold text-indigo-700">{a.uni}</td>
                  <td className="px-4 py-3 text-slate-600">{a.jurusan}</td>
                  <td className="px-4 py-3 text-slate-500">{a.year}</td>
                  <td className="px-4 py-3">
                    <span className="font-black text-slate-900">{activeTab === 'rapor' ? a.pmRapor.toFixed(1) : a.pmUTBK}</span>
                  </td>
                  <td className="px-4 py-3">
                    <button onClick={() => { setAlumni(prev => prev.filter(x => x.id !== a.id)); toast.success('Data alumni dihapus'); }} className="p-1.5 rounded-lg hover:bg-red-50 text-slate-400 hover:text-red-500 transition-colors"><Trash2 className="w-4 h-4" /></button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>

      {addOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-2xl w-full max-w-md shadow-2xl overflow-hidden">
            <div className="flex items-center justify-between p-6 border-b">
              <h3 className="font-bold text-slate-900">Tambah Data Alumni</h3>
              <button onClick={() => setAddOpen(false)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-6 space-y-4">
              {[
                { label: 'Nama Alumni', key: 'name', placeholder: 'Nama lengkap', type: 'text' },
                { label: 'Jurusan', key: 'jurusan', placeholder: 'Teknik Informatika', type: 'text' },
              ].map(({ label, key, placeholder }) => (
                <div key={key}>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">{label}</label>
                  <input className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" placeholder={placeholder} value={(form as Record<string, unknown>)[key] as string} onChange={e => setForm(f => ({ ...f, [key]: e.target.value }))} />
                </div>
              ))}
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Universitas</label>
                  <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.uni} onChange={e => setForm(f => ({ ...f, uni: e.target.value }))}>
                    <option value="">Pilih PTN</option>
                    {PTN_LIST.map(p => <option key={p} value={p}>{p}</option>)}
                  </select>
                </div>
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Tahun Masuk</label>
                  <input type="number" className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.year} onChange={e => setForm(f => ({ ...f, year: Number(e.target.value) }))} />
                </div>
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">PM Rapor</label>
                  <input type="number" step="0.1" className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.pmRapor || ''} onChange={e => setForm(f => ({ ...f, pmRapor: Number(e.target.value) }))} placeholder="91.5" />
                </div>
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">PM UTBK</label>
                  <input type="number" className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={form.pmUTBK || ''} onChange={e => setForm(f => ({ ...f, pmUTBK: Number(e.target.value) }))} placeholder="680" />
                </div>
              </div>
            </div>
            <div className="flex gap-3 p-6 border-t bg-slate-50">
              <button onClick={() => setAddOpen(false)} className="flex-1 py-2.5 rounded-xl border text-sm font-semibold">Batal</button>
              <button onClick={handleAdd} className="flex-1 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-bold">Simpan</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ─── Section: Eligible ────────────────────────────────────────────────────────

function EligibleSection() {
  const [eligible, setEligible] = useState<EligibleYear[]>(INIT_ELIGIBLE);
  const [editId, setEditId] = useState<string | null>(null);
  const [editVal, setEditVal] = useState('');
  const [addOpen, setAddOpen] = useState(false);
  const [addForm, setAddForm] = useState({ year: new Date().getFullYear() + 1, total: 0 });

  const saveEdit = (id: string) => {
    const n = Number(editVal);
    if (!n || n < 0) { toast.error('Jumlah tidak valid'); return; }
    setEligible(prev => prev.map(e => e.id === id ? { ...e, total: n } : e));
    setEditId(null);
    toast.success('Data eligible diperbarui');
  };

  const handleAdd = () => {
    if (eligible.some(e => e.year === addForm.year)) { toast.error('Tahun sudah ada'); return; }
    setEligible(prev => [...prev, { id: `e${Date.now()}`, year: addForm.year, total: addForm.total }]);
    setAddOpen(false);
    toast.success('Data eligible ditambahkan');
  };

  return (
    <div className="max-w-lg space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="font-bold text-slate-800">Manajemen Eligible SNBP</h3>
          <p className="text-sm text-slate-400 mt-0.5">Total siswa eligible per tahun ajaran</p>
        </div>
        <button onClick={() => setAddOpen(true)} className="flex items-center gap-2 px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-xl text-sm font-bold transition-colors">
          <Plus className="w-4 h-4" /> Tambah Tahun
        </button>
      </div>

      <Card className="overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b bg-slate-50">
              <th className="text-left px-6 py-3 font-semibold text-slate-600">No</th>
              <th className="text-left px-6 py-3 font-semibold text-slate-600">Tahun</th>
              <th className="text-left px-6 py-3 font-semibold text-slate-600">Total Eligible</th>
              <th className="text-left px-6 py-3 font-semibold text-slate-600">Aksi</th>
            </tr>
          </thead>
          <tbody>
            {[...eligible].sort((a, b) => b.year - a.year).map((e, i) => (
              <tr key={e.id} className="border-b hover:bg-slate-50 transition-colors">
                <td className="px-6 py-4 text-slate-400">{i + 1}</td>
                <td className="px-6 py-4 font-bold text-slate-900">{e.year}</td>
                <td className="px-6 py-4">
                  {editId === e.id ? (
                    <div className="flex items-center gap-2">
                      <input type="number" className="w-24 px-3 py-1.5 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={editVal} onChange={el => setEditVal(el.target.value)} autoFocus onKeyDown={ev => ev.key === 'Enter' && saveEdit(e.id)} />
                      <button onClick={() => saveEdit(e.id)} className="px-3 py-1.5 bg-indigo-600 text-white rounded-lg text-xs font-bold">OK</button>
                      <button onClick={() => setEditId(null)} className="px-3 py-1.5 border rounded-lg text-xs">✕</button>
                    </div>
                  ) : (
                    <span className="text-2xl font-black text-indigo-600">{e.total}</span>
                  )}
                </td>
                <td className="px-6 py-4">
                  <div className="flex items-center gap-1">
                    <button onClick={() => { setEditId(e.id); setEditVal(String(e.total)); }} className="p-1.5 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-slate-700 transition-colors"><Edit2 className="w-4 h-4" /></button>
                    <button onClick={() => { setEligible(prev => prev.filter(x => x.id !== e.id)); toast.success('Data dihapus'); }} className="p-1.5 rounded-lg hover:bg-red-50 text-slate-400 hover:text-red-500 transition-colors"><Trash2 className="w-4 h-4" /></button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </Card>

      {addOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-2xl w-full max-w-sm shadow-2xl p-6">
            <div className="flex items-center justify-between mb-5">
              <h3 className="font-bold text-slate-900">Tambah Tahun Eligible</h3>
              <button onClick={() => setAddOpen(false)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Tahun</label>
                <input type="number" className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={addForm.year} onChange={e => setAddForm(f => ({ ...f, year: Number(e.target.value) }))} />
              </div>
              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Total Eligible</label>
                <input type="number" className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={addForm.total || ''} onChange={e => setAddForm(f => ({ ...f, total: Number(e.target.value) }))} placeholder="150" />
              </div>
            </div>
            <div className="flex gap-3 mt-6">
              <button onClick={() => setAddOpen(false)} className="flex-1 py-2.5 rounded-xl border text-sm font-semibold">Batal</button>
              <button onClick={handleAdd} className="flex-1 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-bold">Simpan</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ─── Section: Posibilitas Minimum ─────────────────────────────────────────────

function PosibilitasMinimum() {
  const [search, setSearch] = useState('');
  const [filterRumpun, setFilterRumpun] = useState('all');
  const [expanded, setExpanded] = useState<string | null>(null);

  const filtered = useMemo(() => POSMIN_DB.filter(p => {
    const q = search.toLowerCase();
    const match = p.uni.toLowerCase().includes(q) || p.jurusan.toLowerCase().includes(q) || p.provinsi.toLowerCase().includes(q);
    const matchRumpun = filterRumpun === 'all' || p.rumpun === filterRumpun;
    return match && matchRumpun;
  }), [search, filterRumpun]);

  const rumpuns = [...new Set(POSMIN_DB.map(p => p.rumpun))];

  return (
    <div className="space-y-4">
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="flex-1 relative">
          <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
          <input className="w-full pl-9 pr-4 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" placeholder="Cari universitas, jurusan, atau provinsi..." value={search} onChange={e => setSearch(e.target.value)} />
        </div>
        <select className="border rounded-xl px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterRumpun} onChange={e => setFilterRumpun(e.target.value)}>
          <option value="all">Semua Rumpun</option>
          {rumpuns.map(r => <option key={r} value={r}>{r}</option>)}
        </select>
      </div>

      <Card className="overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-slate-50">
                {['No', 'Universitas', 'Jurusan', 'Jenjang', 'Rumpun', 'Min Rapor', 'Min UTBK', 'Kuota/Peminat'].map(h => (
                  <th key={h} className="text-left px-4 py-3 font-semibold text-slate-600 whitespace-nowrap">{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {filtered.map((p, i) => (
                <>
                  <tr key={p.id} className={`border-b hover:bg-slate-50 transition-colors cursor-pointer ${expanded === p.id ? 'bg-indigo-50/50' : ''}`} onClick={() => setExpanded(expanded === p.id ? null : p.id)}>
                    <td className="px-4 py-3 text-slate-400 text-xs font-mono">{i + 1}</td>
                    <td className="px-4 py-3 font-bold text-indigo-700">{p.uni}</td>
                    <td className="px-4 py-3 font-semibold text-slate-800">{p.jurusan}</td>
                    <td className="px-4 py-3"><Badge className="bg-blue-100 text-blue-700 text-xs">{p.jenjang}</Badge></td>
                    <td className="px-4 py-3"><Badge className="bg-purple-100 text-purple-700 text-xs">{p.rumpun}</Badge></td>
                    <td className="px-4 py-3 font-black text-slate-900">{p.minRapor}</td>
                    <td className="px-4 py-3 font-black text-slate-900">{p.minUTBK}</td>
                    <td className="px-4 py-3">
                      <span className="font-semibold text-slate-700">{p.kuotaSNBP}</span>
                      <span className="text-slate-400 mx-1">/</span>
                      <span className="font-semibold text-slate-500">{p.peminatSNBP.toLocaleString()}</span>
                      <span className="ml-2 text-xs text-red-500 font-semibold">1:{Math.round(p.peminatSNBP / p.kuotaSNBP)}</span>
                    </td>
                  </tr>
                  {expanded === p.id && (
                    <tr key={`${p.id}-detail`} className="bg-indigo-50/30 border-b">
                      <td colSpan={8} className="px-6 py-3">
                        <div className="flex items-center gap-6 text-sm">
                          <div className="flex items-center gap-2">
                            <MapPin className="w-4 h-4 text-slate-400" />
                            <span className="text-slate-600">{p.provinsi}</span>
                          </div>
                          <div className="flex items-center gap-2">
                            <BookOpen className="w-4 h-4 text-slate-400" />
                            <span className="text-slate-600">Prasyarat: <strong>{p.prasyarat}</strong></span>
                          </div>
                        </div>
                      </td>
                    </tr>
                  )}
                </>
              ))}
            </tbody>
          </table>
        </div>
        <div className="px-4 py-3 bg-slate-50 border-t text-xs text-slate-400">Menampilkan {filtered.length} dari {POSMIN_DB.length} prodi</div>
      </Card>
    </div>
  );
}

// ─── Section: Rekap SNBT ─────────────────────────────────────────────────────

function snbtStatusInfo(s: SnbtRecord['statusPenerimaan']): { label: string; cls: string } {
  if (s === 'diterima') return { label: 'Diterima SNBT', cls: 'bg-emerald-100 text-emerald-700' };
  if (s === 'tidak-diterima') return { label: 'Tidak Diterima', cls: 'bg-red-100 text-red-700' };
  if (s === 'sudah-ujian') return { label: 'Sudah Ujian', cls: 'bg-blue-100 text-blue-700' };
  return { label: 'Belum Ujian', cls: 'bg-slate-100 text-slate-500' };
}

function RekapSNBT({ students }: { students: SnbpStudent[] }) {
  const [records, setRecords] = useState<SnbtRecord[]>(INIT_SNBT_RECORDS);
  const [search, setSearch] = useState('');
  const [filterStatus, setFilterStatus] = useState('all');
  const [filterYear, setFilterYear] = useState('2026');
  const [editId, setEditId] = useState<string | null>(null);
  const [editForm, setEditForm] = useState<Partial<SnbtRecord>>({});
  const [addOpen, setAddOpen] = useState(false);
  const [addForm, setAddForm] = useState<Partial<SnbtRecord>>({
    studentName: '', class: '', year: 2026, terdaftarSNBT: true,
    pil1uni: '', pil1jur: '', pil2uni: '', pil2jur: '',
    estimasiSkor: 0, statusPenerimaan: 'belum-ujian',
  });

  const filtered = useMemo(() => records.filter(r => {
    const q = search.toLowerCase();
    const match = r.studentName.toLowerCase().includes(q) || r.pil1uni.toLowerCase().includes(q) || r.pil1jur.toLowerCase().includes(q);
    const matchStatus = filterStatus === 'all' || r.statusPenerimaan === filterStatus;
    const matchYear = filterYear === 'all' || String(r.year) === filterYear;
    return match && matchStatus && matchYear;
  }), [records, search, filterStatus, filterYear]);

  const stats = useMemo(() => ({
    terdaftar: records.filter(r => r.terdaftarSNBT).length,
    sudahUjian: records.filter(r => r.skorAktual !== undefined).length,
    diterima: records.filter(r => r.statusPenerimaan === 'diterima').length,
    perluUpdate: records.filter(r => r.terdaftarSNBT && r.skorAktual === undefined && r.tglUjian).length,
  }), [records]);

  const openEdit = (r: SnbtRecord) => {
    setEditId(r.id);
    setEditForm({ ...r });
  };

  const saveEdit = () => {
    if (!editId) return;
    setRecords(prev => prev.map(r => r.id === editId ? { ...r, ...editForm } : r));
    setEditId(null);
    toast.success('Data SNBT diperbarui');
  };

  const handleAdd = () => {
    if (!addForm.studentName) { toast.error('Nama siswa wajib diisi'); return; }
    setRecords(prev => [...prev, { id: `sb${Date.now()}`, ...addForm } as SnbtRecord]);
    setAddOpen(false);
    setAddForm({ studentName: '', class: '', year: 2026, terdaftarSNBT: true, pil1uni: '', pil1jur: '', pil2uni: '', pil2jur: '', estimasiSkor: 0, statusPenerimaan: 'belum-ujian' });
    toast.success('Siswa ditambahkan ke monitoring SNBT');
  };

  const toggleTerdaftar = (id: string) => {
    setRecords(prev => prev.map(r => {
      if (r.id !== id) return r;
      toast.success(r.terdaftarSNBT ? 'Siswa dinonaktifkan dari SNBT' : 'Siswa diaktifkan untuk SNBT');
      return { ...r, terdaftarSNBT: !r.terdaftarSNBT };
    }));
  };

  const setEF = <K extends keyof SnbtRecord>(k: K, v: SnbtRecord[K]) =>
    setEditForm(f => ({ ...f, [k]: v }));
  const setAF = <K extends keyof SnbtRecord>(k: K, v: SnbtRecord[K]) =>
    setAddForm(f => ({ ...f, [k]: v }));

  const scoreGap = (r: SnbtRecord) => {
    const pm = POSMIN_DB.find(p => p.uni === r.pil1uni && p.jurusan === r.pil1jur);
    if (!pm || r.skorAktual === undefined) return null;
    return r.skorAktual - pm.minUTBK;
  };

  return (
    <div className="space-y-5">
      {/* Stats */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
        {[
          { label: 'Terdaftar SNBT', val: stats.terdaftar, icon: ClipboardList, txt: 'text-indigo-600', bg: 'bg-indigo-50', border: 'border-indigo-200', ib: 'bg-indigo-100' },
          { label: 'Sudah Ujian', val: stats.sudahUjian, icon: CheckCircle2, txt: 'text-blue-600', bg: 'bg-blue-50', border: 'border-blue-200', ib: 'bg-blue-100' },
          { label: 'Diterima via SNBT', val: stats.diterima, icon: Medal, txt: 'text-emerald-600', bg: 'bg-emerald-50', border: 'border-emerald-200', ib: 'bg-emerald-100' },
          { label: 'Perlu Update Skor', val: stats.perluUpdate, icon: AlertCircle, txt: 'text-amber-600', bg: 'bg-amber-50', border: 'border-amber-200', ib: 'bg-amber-100' },
        ].map(({ label, val, icon: Icon, txt, bg, border, ib }) => (
          <Card key={label} className={`p-4 ${bg} ${border} border`}>
            <div className="flex items-start justify-between">
              <div>
                <p className="text-xs text-slate-500 mb-1 leading-tight">{label}</p>
                <p className={`text-3xl font-black ${txt}`}>{val}</p>
              </div>
              <div className={`w-10 h-10 rounded-xl ${ib} flex items-center justify-center`}>
                <Icon className={`w-5 h-5 ${txt}`} />
              </div>
            </div>
          </Card>
        ))}
      </div>

      {/* Alert: skor perlu diisi */}
      {stats.perluUpdate > 0 && (
        <div className="flex items-center gap-3 p-4 bg-amber-50 border border-amber-300 rounded-xl">
          <AlertCircle className="w-5 h-5 text-amber-600 shrink-0" />
          <div className="flex-1">
            <p className="text-sm font-bold text-amber-800">{stats.perluUpdate} siswa sudah ujian tapi skor aktual belum diinput</p>
            <p className="text-xs text-amber-600 mt-0.5">Klik tombol edit pada baris siswa untuk memasukkan skor SNBT aktual</p>
          </div>
        </div>
      )}

      {/* Toolbar */}
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="flex-1 relative">
          <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
          <input className="w-full pl-9 pr-4 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" placeholder="Cari nama, universitas, atau prodi..." value={search} onChange={e => setSearch(e.target.value)} />
        </div>
        <select className="border rounded-xl px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterYear} onChange={e => setFilterYear(e.target.value)}>
          {['all', '2025', '2026', '2027'].map(y => <option key={y} value={y}>{y === 'all' ? 'Semua Tahun' : y}</option>)}
        </select>
        <select className="border rounded-xl px-3 py-2 text-sm bg-white focus:outline-none focus:ring-2 focus:ring-indigo-300" value={filterStatus} onChange={e => setFilterStatus(e.target.value)}>
          <option value="all">Semua Status</option>
          <option value="belum-ujian">Belum Ujian</option>
          <option value="sudah-ujian">Sudah Ujian</option>
          <option value="diterima">Diterima</option>
          <option value="tidak-diterima">Tidak Diterima</option>
        </select>
        <button onClick={() => toast.info('Export CSV disiapkan...')} className="flex items-center gap-2 px-4 py-2.5 border rounded-xl text-sm font-semibold hover:bg-slate-50 transition-colors shrink-0">
          <Download className="w-4 h-4" /> Export
        </button>
        <button onClick={() => setAddOpen(true)} className="flex items-center gap-2 px-4 py-2.5 bg-indigo-600 hover:bg-indigo-700 text-white rounded-xl text-sm font-bold transition-colors shrink-0">
          <Plus className="w-4 h-4" /> Tambah Siswa
        </button>
      </div>

      {/* Table */}
      <Card className="overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-slate-50">
                {['Siswa', 'Terdaftar', 'Pilihan 1 SNBT', 'Pilihan 2 SNBT', 'Est. Skor', 'Skor Aktual', 'vs PM UTBK', 'Status', 'Aksi'].map(h => (
                  <th key={h} className="text-left px-4 py-3 font-semibold text-slate-600 whitespace-nowrap text-xs">{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {filtered.length === 0 ? (
                <tr><td colSpan={9} className="px-4 py-12 text-center text-slate-400">Tidak ada data siswa SNBT.</td></tr>
              ) : filtered.map(r => {
                const gap = scoreGap(r);
                const si = snbtStatusInfo(r.statusPenerimaan);
                const pm = POSMIN_DB.find(p => p.uni === r.pil1uni && p.jurusan === r.pil1jur);
                return (
                  <tr key={r.id} className="border-b hover:bg-slate-50 transition-colors">
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2.5">
                        <div className="w-8 h-8 rounded-full bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white text-xs font-bold shrink-0">
                          {r.studentName.split(' ').slice(0, 2).map(w => w[0]).join('')}
                        </div>
                        <div>
                          <p className="font-semibold text-slate-900 whitespace-nowrap">{r.studentName}</p>
                          <p className="text-xs text-slate-400">{r.class} · {r.year}</p>
                        </div>
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <button onClick={() => toggleTerdaftar(r.id)} className={`w-10 h-5 rounded-full transition-colors relative ${r.terdaftarSNBT ? 'bg-emerald-500' : 'bg-slate-200'}`}>
                        <div className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-all ${r.terdaftarSNBT ? 'left-5' : 'left-0.5'}`} />
                      </button>
                    </td>
                    <td className="px-4 py-3">
                      <p className="font-semibold text-indigo-700 text-xs">{r.pil1uni || '—'}</p>
                      <p className="text-xs text-slate-400">{r.pil1jur || '—'}</p>
                    </td>
                    <td className="px-4 py-3">
                      <p className="font-semibold text-purple-700 text-xs">{r.pil2uni || '—'}</p>
                      <p className="text-xs text-slate-400">{r.pil2jur || '—'}</p>
                    </td>
                    <td className="px-4 py-3">
                      <span className="font-bold text-slate-700">{r.estimasiSkor || '—'}</span>
                      <p className="text-[10px] text-slate-400">dari tryout</p>
                    </td>
                    <td className="px-4 py-3">
                      {r.skorAktual !== undefined ? (
                        <span className="font-black text-slate-900 text-base">{r.skorAktual}</span>
                      ) : (
                        <span className="text-slate-300 text-xs">Belum diisi</span>
                      )}
                    </td>
                    <td className="px-4 py-3">
                      {gap !== null ? (
                        <div className="flex items-center gap-1.5">
                          {gap >= 0
                            ? <CheckCircle2 className="w-4 h-4 text-emerald-500 shrink-0" />
                            : <TrendingDown className="w-4 h-4 text-red-400 shrink-0" />}
                          <span className={`text-sm font-black ${gap >= 0 ? 'text-emerald-600' : 'text-red-500'}`}>
                            {gap >= 0 ? '+' : ''}{gap}
                          </span>
                          {pm && <span className="text-[10px] text-slate-400">/ {pm.minUTBK}</span>}
                        </div>
                      ) : (
                        <div className="flex items-center gap-1.5">
                          <MinusCircle className="w-4 h-4 text-slate-300" />
                          {pm && <span className="text-xs text-slate-400">min {pm.minUTBK}</span>}
                        </div>
                      )}
                    </td>
                    <td className="px-4 py-3">
                      <Badge className={`${si.cls} text-xs whitespace-nowrap`}>{si.label}</Badge>
                    </td>
                    <td className="px-4 py-3">
                      <button onClick={() => openEdit(r)} className="p-1.5 rounded-lg hover:bg-slate-100 text-slate-400 hover:text-indigo-600 transition-colors"><Edit2 className="w-4 h-4" /></button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
        <div className="px-4 py-3 bg-slate-50 border-t text-xs text-slate-400">Menampilkan {filtered.length} dari {records.length} siswa</div>
      </Card>

      {/* PM UTBK info panel */}
      <Card className="p-5 bg-gradient-to-br from-slate-50 to-indigo-50 border-indigo-100">
        <div className="flex items-center gap-2 mb-3">
          <TrendingUp className="w-4 h-4 text-indigo-600" />
          <h4 className="font-bold text-slate-800 text-sm">Referensi PM UTBK dari Kaka Kelas</h4>
        </div>
        <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
          {INIT_ALUMNI.map(a => (
            <div key={a.id} className="flex items-center justify-between p-2.5 bg-white rounded-xl border text-xs">
              <div>
                <p className="font-bold text-slate-700">{a.uni} — {a.jurusan}</p>
                <p className="text-slate-400">{a.year}</p>
              </div>
              <span className="font-black text-indigo-600">{a.pmUTBK}</span>
            </div>
          ))}
        </div>
      </Card>

      {/* Edit Modal */}
      {editId && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-2xl w-full max-w-md shadow-2xl overflow-hidden">
            <div className="bg-gradient-to-r from-indigo-600 to-purple-600 px-6 py-5 text-white flex items-center justify-between">
              <div>
                <h3 className="font-black text-lg">Edit Data SNBT</h3>
                <p className="text-white/70 text-sm">{editForm.studentName}</p>
              </div>
              <button onClick={() => setEditId(null)} className="p-2 rounded-xl hover:bg-white/20 transition-colors"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-6 space-y-4">
              {/* Pilihan PTN SNBT */}
              {(['1', '2'] as const).map(n => (
                <div key={n} className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Universitas Pil. {n}</label>
                    <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={n === '1' ? (editForm.pil1uni ?? '') : (editForm.pil2uni ?? '')} onChange={e => { if (n === '1') setEF('pil1uni', e.target.value); else setEF('pil2uni', e.target.value); }}>
                      <option value="">Pilih PTN</option>
                      {PTN_LIST.map(p => <option key={p} value={p}>{p}</option>)}
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Prodi Pil. {n}</label>
                    <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={n === '1' ? (editForm.pil1jur ?? '') : (editForm.pil2jur ?? '')} onChange={e => n === '1' ? setEF('pil1jur', e.target.value) : setEF('pil2jur', e.target.value)}>
                      <option value="">Pilih Prodi</option>
                      {(JURUSAN_BY_PTN[n === '1' ? (editForm.pil1uni ?? '') : (editForm.pil2uni ?? '')] ?? []).map(j => <option key={j} value={j}>{j}</option>)}
                    </select>
                  </div>
                </div>
              ))}

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Skor Aktual SNBT</label>
                  <input type="number" min={0} max={1000} className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300 font-bold" value={editForm.skorAktual ?? ''} onChange={e => setEF('skorAktual', Number(e.target.value) || undefined as unknown as number)} placeholder="cth: 698" />
                </div>
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Tanggal Ujian</label>
                  <input type="date" className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={editForm.tglUjian ?? ''} onChange={e => setEF('tglUjian', e.target.value)} />
                </div>
              </div>

              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Status Penerimaan</label>
                <div className="grid grid-cols-2 gap-2">
                  {(['belum-ujian', 'sudah-ujian', 'diterima', 'tidak-diterima'] as const).map(st => (
                    <button key={st} onClick={() => setEF('statusPenerimaan', st)} className={`py-2 px-3 rounded-xl text-xs font-bold border transition-colors text-left ${editForm.statusPenerimaan === st ? snbtStatusInfo(st).cls + ' border-current' : 'border-slate-200 text-slate-400 hover:bg-slate-50'}`}>
                      {snbtStatusInfo(st).label}
                    </button>
                  ))}
                </div>
              </div>

              <div>
                <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Catatan</label>
                <textarea rows={2} className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300 resize-none" value={editForm.catatan ?? ''} onChange={e => setEF('catatan', e.target.value)} placeholder="Catatan tambahan..." />
              </div>

              {editForm.skorAktual && editForm.pil1uni && editForm.pil1jur && (() => {
                const pm = POSMIN_DB.find(p => p.uni === editForm.pil1uni && p.jurusan === editForm.pil1jur);
                if (!pm) return null;
                const gap = editForm.skorAktual - pm.minUTBK;
                return (
                  <div className={`flex items-center gap-3 p-3 rounded-xl ${gap >= 0 ? 'bg-emerald-50 border border-emerald-200' : 'bg-red-50 border border-red-200'}`}>
                    {gap >= 0 ? <CheckCircle2 className="w-5 h-5 text-emerald-600 shrink-0" /> : <TrendingDown className="w-5 h-5 text-red-500 shrink-0" />}
                    <div className="text-sm">
                      <p className={`font-bold ${gap >= 0 ? 'text-emerald-800' : 'text-red-700'}`}>
                        {gap >= 0 ? `Skor di atas minimum (+${gap})` : `Skor di bawah minimum (${gap})`}
                      </p>
                      <p className={`text-xs ${gap >= 0 ? 'text-emerald-600' : 'text-red-500'}`}>PM UTBK {editForm.pil1uni} {editForm.pil1jur}: {pm.minUTBK}</p>
                    </div>
                  </div>
                );
              })()}
            </div>
            <div className="flex gap-3 p-6 border-t bg-slate-50">
              <button onClick={() => setEditId(null)} className="flex-1 py-2.5 rounded-xl border text-sm font-semibold hover:bg-white transition-colors">Batal</button>
              <button onClick={saveEdit} className="flex-1 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-black transition-colors flex items-center justify-center gap-2">
                <Save className="w-4 h-4" /> Simpan
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Add Modal */}
      {addOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4 overflow-y-auto">
          <div className="bg-white rounded-2xl w-full max-w-md shadow-2xl my-4 overflow-hidden">
            <div className="flex items-center justify-between p-6 border-b">
              <h3 className="font-bold text-slate-900">Tambah Siswa SNBT</h3>
              <button onClick={() => setAddOpen(false)} className="p-2 rounded-xl hover:bg-slate-100"><X className="w-5 h-5" /></button>
            </div>
            <div className="p-6 space-y-4 max-h-[60vh] overflow-y-auto">
              <div className="grid grid-cols-2 gap-3">
                <div className="col-span-2">
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Nama Siswa *</label>
                  <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={addForm.studentName} onChange={e => {
                    const st = students.find(s => s.name === e.target.value);
                    setAddForm(f => ({ ...f, studentName: e.target.value, class: st?.class ?? f.class, year: st?.year ?? f.year, pil1uni: st?.pil1uni ?? '', pil1jur: st?.pil1jur ?? '', pil2uni: st?.pil2uni ?? '', pil2jur: st?.pil2jur ?? '' }));
                  }}>
                    <option value="">Pilih dari daftar siswa</option>
                    {students.map(s => <option key={s.id} value={s.name}>{s.name} — {s.class}</option>)}
                  </select>
                </div>
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Estimasi Skor</label>
                  <input type="number" className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={addForm.estimasiSkor || ''} onChange={e => setAF('estimasiSkor', Number(e.target.value))} placeholder="dari avg tryout" />
                </div>
                <div>
                  <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Tahun</label>
                  <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={addForm.year} onChange={e => setAF('year', Number(e.target.value))}>
                    {[2025, 2026, 2027].map(y => <option key={y} value={y}>{y}</option>)}
                  </select>
                </div>
              </div>
              {(['1', '2'] as const).map(n => (
                <div key={n} className="grid grid-cols-2 gap-3">
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Universitas Pil. {n}</label>
                    <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={n === '1' ? (addForm.pil1uni ?? '') : (addForm.pil2uni ?? '')} onChange={e => { if (n === '1') setAF('pil1uni', e.target.value); else setAF('pil2uni', e.target.value); }}>
                      <option value="">Pilih PTN</option>
                      {PTN_LIST.map(p => <option key={p} value={p}>{p}</option>)}
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-bold text-slate-500 uppercase tracking-wide block mb-1.5">Prodi Pil. {n}</label>
                    <select className="w-full px-3 py-2.5 border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-indigo-300" value={n === '1' ? (addForm.pil1jur ?? '') : (addForm.pil2jur ?? '')} onChange={e => n === '1' ? setAF('pil1jur', e.target.value) : setAF('pil2jur', e.target.value)}>
                      <option value="">Pilih Prodi</option>
                      {(JURUSAN_BY_PTN[n === '1' ? (addForm.pil1uni ?? '') : (addForm.pil2uni ?? '')] ?? []).map(j => <option key={j} value={j}>{j}</option>)}
                    </select>
                  </div>
                </div>
              ))}
            </div>
            <div className="flex gap-3 p-6 border-t bg-slate-50">
              <button onClick={() => setAddOpen(false)} className="flex-1 py-2.5 rounded-xl border text-sm font-semibold">Batal</button>
              <button onClick={handleAdd} className="flex-1 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-bold">Tambah</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ─── Main Export ──────────────────────────────────────────────────────────────

const SECTIONS: { key: SectionKey; label: string; icon: React.ElementType }[] = [
  { key: 'dashboard', label: 'Dashboard', icon: BarChart2 },
  { key: 'siswa', label: 'Daftar Siswa', icon: Users },
  { key: 'rasionalisasi', label: 'Rasionalisasi SNBP', icon: Target },
  { key: 'alumni', label: 'Kaka Kelas', icon: GraduationCap },
  { key: 'eligible', label: 'Eligible', icon: Award },
  { key: 'posmin', label: 'Pos. Minimum', icon: Building2 },
];

export default function SNBPRasionalisasi() {
  const [section, setSection] = useState<SectionKey>('dashboard');
  const [students, setStudents] = useState<SnbpStudent[]>(INIT_STUDENTS);
  const [eligible] = useState<EligibleYear[]>(INIT_ELIGIBLE);

  const pendingCount = students.filter(s => !s.hasRasionalisasi && s.aktif).length;

  return (
    <div className="space-y-6">
      {/* Module header */}
      <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-blue-700 via-indigo-700 to-purple-700 p-6 text-white">
        <div className="absolute inset-0 opacity-10" style={{ backgroundImage: 'radial-gradient(circle at 20% 50%, white 1px, transparent 1px)', backgroundSize: '28px 28px' }} />
        <div className="relative flex items-start justify-between flex-wrap gap-4">
          <div>
            <div className="flex items-center gap-2 mb-1">
              <Target className="w-5 h-5 text-white/70" />
              <span className="text-white/70 text-sm font-semibold uppercase tracking-wide">Sistem Rasionalisasi</span>
            </div>
            <h2 className="text-2xl font-black">SNBP Rasionalisasi</h2>
            <p className="text-white/70 text-sm mt-1">Bantu siswa menemukan prodi impian dengan peluang terbaik</p>
          </div>
          <div className="flex items-center gap-4">
            {[
              { label: 'Total Siswa', val: students.length },
              { label: 'Terasionalisasi', val: students.filter(s => s.hasRasionalisasi).length },
              { label: 'Diterima', val: students.filter(s => s.statusPenerimaan !== 'belum').length },
            ].map(({ label, val }) => (
              <div key={label} className="text-center hidden sm:block">
                <p className="text-2xl font-black">{val}</p>
                <p className="text-white/60 text-xs">{label}</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      {pendingCount > 0 && (
        <div className="flex items-center gap-3 p-4 bg-amber-50 border border-amber-200 rounded-xl">
          <AlertCircle className="w-5 h-5 text-amber-600 shrink-0" />
          <p className="text-sm font-semibold text-amber-800">
            <strong>{pendingCount} siswa</strong> belum memiliki data rasionalisasi. Klik tab "Rasionalisasi" untuk mulai.
          </p>
        </div>
      )}

      {/* Sub-navigation */}
      <div className="flex gap-1.5 flex-wrap">
        {SECTIONS.map(({ key, label, icon: Icon }) => (
          <button key={key} onClick={() => setSection(key)}
            className={`flex items-center gap-2 px-4 py-2.5 rounded-xl text-sm font-semibold transition-all border ${section === key ? 'bg-indigo-600 text-white border-indigo-600 shadow-sm' : 'bg-white text-slate-600 border-slate-200 hover:border-indigo-300 hover:text-indigo-700'}`}>
            <Icon className="w-4 h-4" />
            {label}
          </button>
        ))}
      </div>

      {/* Section content */}
      {section === 'dashboard' && <DashboardSNBP students={students} eligible={eligible} />}
      {section === 'siswa' && <DaftarSiswaSNBP students={students} onUpdate={setStudents} />}
      {section === 'rasionalisasi' && <RasionalisasiSection students={students} onUpdate={setStudents} />}
      {section === 'alumni' && <JurusanKakaKelas />}
      {section === 'eligible' && <EligibleSection />}
      {section === 'posmin' && <PosibilitasMinimum />}
    </div>
  );
}
