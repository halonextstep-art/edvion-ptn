// Parser bersama untuk format import siswa mitra sekolah (.xlsx), dipakai baik oleh
// SchoolOnboardingWizard.vue (Admin -> onboarding/import per sekolah) maupun
// StudentManager.vue (Portal Sekolah -> self-service oleh PIC sekolah), supaya kedua alur
// menerima file yang sama persis (kolom: nis, nisn, nama, jenis kelamin, tanggal lahir,
// tanggal masuk, kode rombel, nama rombel, username, email, telepon, password) tanpa
// duplikasi logika parsing/validasi yang gampang divergen kalau ditulis dua kali.
import * as XLSX from 'xlsx'

export interface ImportedStudentRow {
  nis: string
  nisn: string
  name: string
  gender: string
  birthDate: string
  enrolledAt: string
  rombelCode: string
  grade: string // "nama rombel"
  username: string
  email: string
  phone: string
  password: string
  status: 'ready' | 'error'
  error: string
}

// Header kanonik yang dicari di baris pertama file — dicocokkan case-insensitive & trim,
// supaya "NISN", " nisn ", "Nisn" dst. semua kena, tidak harus persis huruf kecil.
// Urutan kolom di file bebas (dicari lewat nama header, bukan posisi tetap).
export const STUDENT_IMPORT_HEADERS = ['nis', 'nisn', 'nama', 'jenis kelamin', 'tanggal lahir', 'tanggal masuk', 'kode rombel', 'nama rombel', 'username', 'email', 'telepon', 'password'] as const

function normalizeHeader(h: unknown): string {
  return String(h ?? '').trim().toLowerCase().replace(/\s+/g, ' ')
}

function excelDateToIso(v: unknown): string {
  if (!v) return ''
  if (v instanceof Date) {
    if (Number.isNaN(v.getTime())) return ''
    return v.toISOString().slice(0, 10)
  }
  // Fallback: file punya tanggal sebagai teks (bukan cell bertipe Date beneran) — dibiarkan
  // apa adanya, backend akan menolak dengan pesan jelas kalau format-nya memang tidak valid.
  return String(v).trim()
}

/** Baca file .xlsx/.xls dan kembalikan setiap baris data siswa + hasil validasinya. Lempar
 * Error (dengan pesan siap-tampil) kalau file kosong atau kolom wajibnya tidak ketemu sama
 * sekali di header — bukan per-baris, karena itu artinya file salah format, bukan salah isi. */
export async function parseStudentImportWorkbook(file: File): Promise<ImportedStudentRow[]> {
  const buf = await file.arrayBuffer()
  const wb = XLSX.read(buf, { type: 'array', cellDates: true })
  const sheet = wb.Sheets[wb.SheetNames[0]]
  if (!sheet) throw new Error('File tidak punya sheet apa pun.')

  const raw: unknown[][] = XLSX.utils.sheet_to_json(sheet, { header: 1, blankrows: false, defval: '' })
  if (raw.length === 0) throw new Error('File kosong.')

  const headerRow = raw[0].map(normalizeHeader)
  const colIndex: Record<string, number> = {}
  for (const key of STUDENT_IMPORT_HEADERS) colIndex[key] = headerRow.indexOf(key)

  const missingRequired = (['nisn', 'nama', 'email', 'password'] as const).filter((k) => colIndex[k] === -1)
  if (missingRequired.length > 0) {
    throw new Error(`Kolom wajib tidak ditemukan di header file: ${missingRequired.join(', ')}. Cek nama kolomnya sesuai template.`)
  }

  const cell = (row: unknown[], key: (typeof STUDENT_IMPORT_HEADERS)[number]): string => {
    const idx = colIndex[key]
    if (idx === -1) return ''
    const v = row[idx]
    return v == null ? '' : String(v).trim()
  }

  return raw.slice(1).map((row): ImportedStudentRow => {
    const nis = cell(row, 'nis')
    const nisn = cell(row, 'nisn')
    const name = cell(row, 'nama')
    const gender = cell(row, 'jenis kelamin')
    const rombelCode = cell(row, 'kode rombel')
    const grade = cell(row, 'nama rombel')
    const username = cell(row, 'username')
    const email = cell(row, 'email')
    const phone = cell(row, 'telepon')
    const password = cell(row, 'password')
    const birthDateIdx = colIndex['tanggal lahir']
    const enrolledAtIdx = colIndex['tanggal masuk']
    const birthDate = birthDateIdx === -1 ? '' : excelDateToIso(row[birthDateIdx])
    const enrolledAt = enrolledAtIdx === -1 ? '' : excelDateToIso(row[enrolledAtIdx])

    const errors: string[] = []
    if (!nisn || nisn.length < 8) errors.push('NISN tidak valid')
    if (!name || name.trim().length < 2) errors.push('Nama minimal 2 karakter')
    if (!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) errors.push('Email tidak valid')
    if (!password || password.length < 6) errors.push('Password minimal 6 karakter')

    return {
      nis, nisn, name, gender, birthDate, enrolledAt, rombelCode, grade, username, email, phone, password,
      status: errors.length ? 'error' : 'ready',
      error: errors.join(', '),
    }
  })
}

/** Trigger unduh file .xlsx contoh format import — dipakai dari kedua layar (Admin & Portal
 * Sekolah) supaya template yang dibagikan selalu identik dengan yang benar-benar diparsing. */
export function downloadStudentImportTemplate(filename = 'template_import_siswa.xlsx') {
  const wb = XLSX.utils.book_new()
  const ws = XLSX.utils.json_to_sheet([
    { nis: 1095, nisn: '0125733860', nama: 'Contoh Nama Siswa', 'jenis kelamin': 'L', 'tanggal lahir': '', 'tanggal masuk': '', 'kode rombel': 'Kelas 9', 'nama rombel': 'Kelas 9', username: 'Contoh Nama Siswa', email: 'contoh@siswa.sch.id', telepon: '', password: 'password123' },
  ], { header: [...STUDENT_IMPORT_HEADERS] })
  XLSX.utils.book_append_sheet(wb, ws, 'Siswa')
  XLSX.writeFile(wb, filename)
}

/** Payload siap-kirim ke `userService.create()` dari satu baris hasil parse — dipakai kedua
 * layar supaya pemetaan field (termasuk yang gampang typo seperti birthDate -> birth_date)
 * hanya ditulis sekali. */
export function importedRowToCreatePayload(r: ImportedStudentRow, schoolId?: string | null) {
  return {
    name: r.name,
    email: r.email,
    password: r.password,
    role: 'student' as const,
    school_id: schoolId ?? undefined,
    status: 'active' as const,
    nisn: r.nisn,
    grade: r.grade || undefined,
    phone: r.phone || undefined,
    nis: r.nis || undefined,
    gender: r.gender || undefined,
    birth_date: r.birthDate || undefined,
    enrolled_at: r.enrolledAt || undefined,
    rombel_code: r.rombelCode || undefined,
    username: r.username || undefined,
  }
}
