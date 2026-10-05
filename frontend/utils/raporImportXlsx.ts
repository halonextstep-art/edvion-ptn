// Parser bersama untuk import nilai rapor via Excel — dipakai baik oleh
// RasionalisasiSnbpEditor.vue (import untuk SATU siswa, kolom NIS diabaikan) maupun
// SchoolRasionalisasi.vue (import MASSAL banyak siswa sekaligus, kolom NIS wajib dipakai
// untuk mencocokkan ke siswa yang tepat). Format tabel LEBAR: 1 baris = 1 siswa untuk 1
// semester, kolom = NIS, Nama, Semester, lalu satu kolom per mapel (lihat RAPOR_SUBJECTS)
// — dipilih ketimbang format panjang (1 baris = 1 nilai) karena mendekati bentuk rapor asli
// dan jauh lebih ringkas untuk mengisi banyak siswa sekaligus.
import * as XLSX from 'xlsx'
import { RAPOR_SUBJECTS, canonicalRaporSubject } from './raporSubjects'

export interface ParsedRaporEntry {
  subject: string
  score: number
}

export interface ParsedRaporRow {
  /** Nomor baris asli di file Excel (1 = header), untuk ditampilkan ke user kalau ada masalah. */
  rowNumber: number
  nis: string
  nama: string
  semester: number | null
  /** Nilai valid (0-100) yang siap diimpor dari baris ini. */
  entries: ParsedRaporEntry[]
  /** Masalah level-baris yang membuat SELURUH baris ini tidak bisa diproses (mis. semester
   * tidak valid) — beda dari cellWarnings yang cuma mengabaikan satu sel. */
  rowError: string | null
  /** Peringatan per-sel (nilai di luar 0-100, bukan angka, dst.) — sel itu diabaikan, baris
   * lain & sel lain tetap diproses. */
  cellWarnings: string[]
}

export interface RaporWorkbookParseResult {
  rows: ParsedRaporRow[]
  /** Mapel kanonik yang kolomnya benar-benar ditemukan di header file (bisa kurang dari
   * seluruh RAPOR_SUBJECTS — user boleh isi cuma sebagian mapel). */
  matchedSubjectColumns: string[]
  hasNisColumn: boolean
  totalValidEntries: number
}

function normalizeHeader(h: unknown): string {
  return String(h ?? '').trim().toLowerCase().replace(/\s+/g, ' ')
}

/** Baca file .xlsx/.xls format tabel lebar dan kembalikan hasil parse + validasi per-baris.
 * Melempar Error (pesan siap-tampil) hanya untuk masalah STRUKTURAL (file kosong, kolom
 * "semester" tidak ada padahal `requireSemester`, atau tidak ada satupun kolom mapel yang
 * dikenali) — masalah per-baris (NIS kosong, nilai di luar rentang, dst.) tidak melempar
 * error, tapi masuk ke `rowError`/`cellWarnings` supaya bisa ditampilkan sebagai preview
 * sebelum user konfirmasi import.
 *
 * `requireSemester: false` dipakai untuk snapshot "Detail Rapor" alumni (1 alumni, banyak
 * mapel sekaligus) — tidak ada konsep semester sama sekali, jadi kolom "semester" boleh
 * tidak ada dan tiap baris tetap diproses dengan `semester: null`. */
export async function parseRaporImportWorkbook(
  file: File,
  opts: { requireSemester?: boolean } = {},
): Promise<RaporWorkbookParseResult> {
  const requireSemester = opts.requireSemester ?? true
  const buf = await file.arrayBuffer()
  const wb = XLSX.read(buf, { type: 'array' })
  const sheet = wb.Sheets[wb.SheetNames[0]]
  if (!sheet) throw new Error('File tidak punya sheet apa pun.')

  const raw: unknown[][] = XLSX.utils.sheet_to_json(sheet, { header: 1, blankrows: false, defval: '' })
  if (raw.length === 0) throw new Error('File kosong.')

  const headerRow = raw[0].map(normalizeHeader)
  const nisIdx = headerRow.indexOf('nis')
  const namaIdx = headerRow.indexOf('nama')
  const semesterIdx = headerRow.indexOf('semester')
  if (requireSemester && semesterIdx === -1) {
    throw new Error('Kolom "semester" tidak ditemukan di header file. Cek nama kolomnya sesuai template.')
  }

  // Cocokkan tiap kolom header ke nama mapel kanonik (case-insensitive, trim) — kolom yang
  // headernya tidak dikenali (typo, atau bukan kolom mapel) diabaikan begitu saja, bukan
  // dianggap error, supaya user bebas menambah kolom catatan sendiri di file kalau perlu.
  const subjectColumns: { subject: string; colIdx: number }[] = []
  headerRow.forEach((h, idx) => {
    if (idx === nisIdx || idx === namaIdx || idx === semesterIdx) return
    const canonical = canonicalRaporSubject(h)
    if (canonical) subjectColumns.push({ subject: canonical, colIdx: idx })
  })
  if (subjectColumns.length === 0) {
    throw new Error(`Tidak ada kolom mapel yang dikenali di header file. Nama kolom mapel harus persis sama dengan salah satu dari ${RAPOR_SUBJECTS.length} mapel yang didukung (unduh template untuk contoh lengkap).`)
  }

  let totalValidEntries = 0
  const rows: ParsedRaporRow[] = raw.slice(1).map((row, i): ParsedRaporRow => {
    const rowNumber = i + 2
    const nis = nisIdx === -1 ? '' : String(row[nisIdx] ?? '').trim()
    const nama = namaIdx === -1 ? '' : String(row[namaIdx] ?? '').trim()

    let semesterNum: number | null = null
    if (requireSemester) {
      const semesterRaw = row[semesterIdx]
      const parsed = Number(String(semesterRaw ?? '').trim())
      const semesterValid = Number.isInteger(parsed) && parsed >= 1 && parsed <= 6
      if (!semesterValid) {
        return {
          rowNumber, nis, nama, semester: null, entries: [],
          rowError: `Semester tidak valid ("${semesterRaw}") — harus angka 1-6`,
          cellWarnings: [],
        }
      }
      semesterNum = parsed
    }

    const entries: ParsedRaporEntry[] = []
    const cellWarnings: string[] = []
    for (const { subject, colIdx } of subjectColumns) {
      const cellRaw = row[colIdx]
      const cellStr = String(cellRaw ?? '').trim()
      if (cellStr === '') continue // mapel ini memang tidak diisi untuk siswa/semester ini — bukan error
      const score = Number(cellStr)
      if (Number.isNaN(score) || score < 0 || score > 100) {
        cellWarnings.push(`${subject}: nilai "${cellStr}" tidak valid (diabaikan, harus 0-100)`)
        continue
      }
      entries.push({ subject, score })
    }
    totalValidEntries += entries.length

    return { rowNumber, nis, nama, semester: semesterNum, entries, rowError: null, cellWarnings }
  })

  return { rows, matchedSubjectColumns: subjectColumns.map((c) => c.subject), hasNisColumn: nisIdx !== -1, totalValidEntries }
}

/** Trigger unduh file .xlsx contoh format import. `includeNis: true` (Portal Sekolah, import
 * massal) menambah kolom NIS+Nama di depan; `includeNis: false` (form nilai rapor 1 siswa)
 * langsung mulai dari kolom Semester karena konteksnya sudah pasti satu siswa itu saja.
 * `includeSemester: false` (Detail Rapor alumni) menghilangkan kolom Semester sama sekali —
 * templatenya jadi cuma satu baris berisi nilai per mapel untuk satu alumni. */
export function downloadRaporImportTemplate(
  filename = 'template_import_nilai_rapor.xlsx',
  opts: { includeNis?: boolean; includeSemester?: boolean } = {},
) {
  const includeNis = opts.includeNis ?? false
  const includeSemester = opts.includeSemester ?? true
  const header = [...(includeNis ? ['nis', 'nama'] : []), ...(includeSemester ? ['semester'] : []), ...RAPOR_SUBJECTS]
  const exampleRow: (string | number)[] = [...(includeNis ? ['1095', 'Contoh Nama Siswa'] : []), ...(includeSemester ? [1] : [])]
  for (const subj of RAPOR_SUBJECTS) {
    if (subj === 'Matematika') exampleRow.push(88)
    else if (subj === 'Bahasa Indonesia') exampleRow.push(90)
    else if (subj === 'Bahasa Inggris') exampleRow.push(85)
    else exampleRow.push('')
  }

  const ws = XLSX.utils.aoa_to_sheet([header, exampleRow])
  const wb = XLSX.utils.book_new()
  XLSX.utils.book_append_sheet(wb, ws, 'Nilai Rapor')
  XLSX.writeFile(wb, filename)
}
