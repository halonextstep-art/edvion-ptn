// Daftar mapel Kurikulum Merdeka (SMA/MA/SMK) untuk form Nilai Rapor SNBP — SATU-SATUNYA
// sumber daftar ini di frontend. Diekspor dari sini (bukan didefinisikan ulang di tiap
// komponen) supaya grid input manual (RasionalisasiSnbpEditor.vue), matcher root-keyword
// (SchoolRasionalisasi.vue), dan parser import Excel (raporImportXlsx.ts) selalu memakai
// daftar yang identik persis — kalau daftar ini berubah, ketiganya otomatis ikut berubah
// tanpa risiko salah satu ketinggalan.
export const RAPOR_SUBJECTS = [
  'Pendidikan Agama dan Budi Pekerti', 'Pendidikan Pancasila dan Kewarganegaraan',
  'Bahasa Indonesia', 'Matematika', 'Sejarah', 'Bahasa Inggris', 'Seni Budaya',
  'Pendidikan Jasmani, Olahraga, dan Kesehatan', 'Prakarya dan Kewirausahaan',
  'Ilmu Pengetahuan Alam', 'Ilmu Pengetahuan Sosial', 'Fisika', 'Kimia', 'Biologi',
  'Sosiologi', 'Geografi', 'Ekonomi', 'Matematika Tingkat Lanjut', 'Antropologi',
  'Bahasa Jerman', 'Bahasa Prancis', 'Bahasa Jepang', 'Informatika', 'Bahasa Bali',
  'Bahasa Mandarin', 'Bahasa Jawa', 'Bahasa Sunda', 'Bahasa Arab', 'Bahasa Korea',
  'Bahasa Asing Lainnya', 'Fiqih', 'Muatan Lokal Bahasa Daerah', 'Sejarah Peminatan',
  'Bahasa Indonesia Tingkat Lanjut', 'Bahasa Inggris Tingkat Lanjut', 'Conversation',
  "Al-Qur'an Hadist", 'Akidah Akhlak', 'Sejarah Kebudayaan Islam', 'Ilmu Tafsir',
  'Ilmu Hadist', 'Riset', 'Robotik', 'Ushul Fiqih', 'Bahasa Arab Tingkat Lanjut',
  'Sejarah Tingkat Lanjut',
] as const

/** Katalog PTN pakai nama mapel bebas ("Matematika Tingkat Lanjut") yang kadang beda
 * penulisan dari nama mapel di form nilai rapor (mis. "Pendidikan Pancasila dan
 * Kewarganegaraan" vs katalog yang cuma tulis "Pendidikan Pancasila") — jadi dicocokkan
 * lewat kata dasar mapel (root keyword), bukan substring satu arah yang gagal mengenali
 * variasi seperti itu. Dipakai RasionalisasiSnbpEditor.vue dan SchoolRasionalisasi.vue. */
export const RAPOR_SUBJECT_ROOTS = [
  'matematika', 'fisika', 'kimia', 'biologi', 'ekonomi', 'sosiologi', 'geografi', 'sejarah',
  'pancasila', 'ppkn', 'pkn', 'indonesia', 'inggris', 'seni', 'budaya', 'agama', 'jasmani',
  'penjas', 'pjok', 'prakarya', 'antropologi', 'jerman', 'prancis', 'jepang', 'informatika',
  'bali', 'mandarin', 'jawa', 'sunda', 'arab', 'korea', 'asing', 'fiqih', 'tafsir', 'hadist',
  'ushul', 'muatan lokal', 'robotik', 'riset', 'conversation',
] as const

export function raporSubjectRoot(subject: string): string | null {
  const lower = subject.toLowerCase()
  return RAPOR_SUBJECT_ROOTS.find((root) => lower.includes(root)) ?? null
}

/** Cari nama mapel kanonik (persis seperti di `RAPOR_SUBJECTS`) dari teks bebas — dipakai
 * parser import Excel supaya header kolom "matematika" atau " Matematika " tetap cocok ke
 * "Matematika", termasuk kalau penggunanya salah kapitalisasi/trailing space. Cocok exact
 * (case-insensitive, trim) dulu; kalau tidak ketemu, kembalikan `null` (bukan ditebak lewat
 * root-keyword — untuk import, mapel yang tidak persis dikenal harus ditandai error, bukan
 * diam-diam dipetakan ke mapel lain yang mirip). */
export function canonicalRaporSubject(raw: string): string | null {
  const needle = raw.trim().toLowerCase()
  return RAPOR_SUBJECTS.find((s) => s.toLowerCase() === needle) ?? null
}
