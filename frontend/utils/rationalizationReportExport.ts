// Export laporan rasionalisasi SNBP per-siswa ke PDF (jsPDF + jspdf-autotable) — dokumen
// FORMAL yang sekolah serahkan ke orang tua, jadi TIDAK memakai window.print() (yang akan
// mencetak seluruh UI aplikasi termasuk nav/sidebar). Teknik & konvensi visual dasar (font
// size, headStyles fillColor indigo brand [79, 70, 229], autoTable per section) sengaja
// disamakan dengan utils/reportExport.ts, ditambah lapisan visual baru (accent bar, pill
// tier berwarna, grafik radar tertanam) supaya dokumen terasa "keren" tapi tetap formal.
//
// Semua data yang dirender berasal dari input yang di-pass oleh caller (SchoolRasionalisasi.vue),
// yang sudah nyata (detailCard/detailPilihan/raporBySubject/alternatif dari mesin
// listPrograms+previewStudentChance yang sama dengan yang tampil di layar) — tidak ada nilai
// hardcode/fiktif di sini, termasuk `radar_chart_image` (di-generate dari Chart.js dengan data
// rapor asli siswa oleh caller, util ini hanya menempelkannya). Kalau `school_name` tidak
// tersedia, fallback ke judul generik (bukan blank/undefined).
import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'

const BRAND: [number, number, number] = [79, 70, 229]
const TIER_RGB: Record<string, [number, number, number]> = {
  aman: [5, 150, 105], moderat: [180, 83, 9], ketat: [220, 38, 38],
}
const TIER_BG_RGB: Record<string, [number, number, number]> = {
  aman: [209, 250, 229], moderat: [254, 243, 199], ketat: [254, 226, 226],
}
function tierRgb(tier: string): [number, number, number] { return TIER_RGB[tier] ?? [100, 116, 139] }
function tierBgRgb(tier: string): [number, number, number] { return TIER_BG_RGB[tier] ?? [241, 245, 249] }

export interface RationalizationAlternatifInput {
  nama_ptn: string
  nama_prodi: string
  chance_percent: number
  tier: string
  tier_label: string
}

export interface RationalizationPilihanInput {
  /** "Pilihan 1" / "Pilihan 2" */
  label: string
  nama_ptn: string
  nama_prodi: string
  chance_percent: number
  /** Kunci tier mentah ('aman'/'moderat'/'ketat') dipakai untuk pewarnaan, terpisah dari label. */
  tier: string
  tier_label: string
  nilai_contribution: number
  competition_adjustment: number
  prestasi_contribution: number
  pg_snbp: number
  pg_snbt: number
  daya_tampung_snbp: number
  peminat_snbp: number
  rumpun: string
  /** Alternatif program lain di rumpun yang sama (real, dari listPrograms+previewStudentChance). */
  alternatif_rumpun: RationalizationAlternatifInput[]
  /** Label minat jurusan efektif yang dipakai untuk pencarian alternatif (real: input eksplisit
   * siswa, atau fallback ke nama_prodi pilihan ini kalau siswa belum mengisi). */
  minat_label: string
  /** True kalau `minat_label` di atas adalah hasil fallback (bukan diisi eksplisit oleh siswa). */
  minat_is_fallback: boolean
  alternatif_minat: RationalizationAlternatifInput[]
  /** Data URL PNG grafik radar "Pencapaian vs Referensi PG" (di-generate via Chart.js oleh
   * caller dari data rapor asli siswa) — null kalau mata pelajaran syarat < 3 (tidak cukup
   * sumbu untuk radar, sama seperti syarat tampil di layar). */
  radar_chart_image: string | null
}

export interface RationalizationReportInput {
  school_name?: string | null
  student_name: string
  kelas?: string | null
  konsultan?: string | null
  /** Pilihan 1 & 2 (yang tersedia) — laporan menampilkan KEDUANYA sekaligus, lihat catatan di bawah. */
  pilihan: RationalizationPilihanInput[]
  rapor: { subject: string; avg: number }[]
}

function formatDateID(d: Date): string {
  return d.toLocaleString('id-ID', { day: 'numeric', month: 'long', year: 'numeric', hour: '2-digit', minute: '2-digit' })
}

function slug(s: string): string {
  return s.replace(/[^a-z0-9]+/gi, '-').replace(/^-+|-+$/g, '').toLowerCase() || 'data'
}

/** Pill kecil berwarna sesuai tier (mengikuti konvensi emerald/amber/red yang sama dengan
 * layar) — dipakai untuk badge "Peluang Kompetisi" & sel tabel alternatif/resume. */
function drawTierPill(doc: jsPDF, x: number, y: number, tier: string, text: string): number {
  doc.setFontSize(9)
  doc.setFont('helvetica', 'bold')
  const w = doc.getTextWidth(text) + 6
  const h = 5.5
  doc.setFillColor(...tierBgRgb(tier))
  doc.roundedRect(x, y - h + 1.3, w, h, 1.3, 1.3, 'F')
  doc.setTextColor(...tierRgb(tier))
  doc.text(text, x + 3, y)
  doc.setTextColor(0)
  return w
}

/** Judul section dengan aksen garis kecil berwarna indigo di kiri teks — dipakai konsisten
 * untuk semua judul section baru supaya dokumen terasa satu sistem visual. */
function sectionTitle(doc: jsPDF, text: string, x: number, y: number) {
  doc.setFillColor(...BRAND)
  doc.rect(x, y - 3.3, 1.3, 4.3, 'F')
  doc.setFontSize(10.5)
  doc.setFont('helvetica', 'bold')
  doc.setTextColor(30, 30, 40)
  doc.text(text, x + 3, y)
  doc.setTextColor(0)
}

function alternatifTable(
  doc: jsPDF,
  startY: number,
  marginX: number,
  rows: RationalizationAlternatifInput[],
  emptyLabel: string,
): number {
  autoTable(doc, {
    startY,
    head: [['Universitas / Program Studi', 'Peluang', 'Tier']],
    body: rows.length
      ? rows.map((a) => [`${a.nama_ptn}\n${a.nama_prodi}`, `${a.chance_percent}%`, a.tier_label])
      : [[emptyLabel, '', '']],
    styles: { fontSize: 8.5, cellPadding: 2 },
    headStyles: { fillColor: BRAND },
    columnStyles: { 1: { cellWidth: 22, halign: 'center' }, 2: { cellWidth: 26, halign: 'center' } },
    margin: { left: marginX, right: marginX },
    didParseCell: (data) => {
      if (data.section === 'body' && data.column.index === 2 && rows[data.row.index]) {
        const t = rows[data.row.index].tier
        data.cell.styles.fillColor = tierBgRgb(t)
        data.cell.styles.textColor = tierRgb(t)
        data.cell.styles.fontStyle = 'bold'
      }
    },
  })
  return (doc as any).lastAutoTable.finalY
}

export function exportRationalizationReportToPdf(input: RationalizationReportInput) {
  const doc = new jsPDF()
  const marginX = 14
  const pageRight = 196
  const pageBottom = 283
  let y = 20

  // ── Accent bar atas — sentuhan visual pertama yang terlihat sebelum konten apa pun. ──
  doc.setFillColor(...BRAND)
  doc.rect(0, 0, 210, 2.6, 'F')

  function ensureSpace(need: number) {
    if (y + need > pageBottom) { doc.addPage(); y = 20 }
  }

  // ── Letterhead (nama sekolah dari akun PIC yang login) ──
  doc.setFontSize(14)
  doc.setFont('helvetica', 'bold')
  doc.text(input.school_name || 'Laporan Sekolah', marginX, y)
  y += 6

  doc.setFontSize(11)
  doc.setFont('helvetica', 'normal')
  doc.setTextColor(90)
  doc.text('Laporan Rasionalisasi SNBP', marginX, y)
  y += 5
  doc.setFontSize(9)
  doc.text(`Dibuat ${formatDateID(new Date())}`, marginX, y)
  doc.setTextColor(0)
  y += 3
  doc.setDrawColor(...BRAND)
  doc.setLineWidth(0.6)
  doc.line(marginX, y, pageRight, y)
  y += 8

  // ── Identitas siswa ──
  sectionTitle(doc, 'Data Siswa', marginX, y)
  y += 2
  autoTable(doc, {
    startY: y + 2,
    theme: 'plain',
    styles: { fontSize: 9, cellPadding: 1 },
    body: [
      ['Nama', input.student_name],
      ['Kelas', input.kelas || '-'],
      ['Konsultan', input.konsultan || '-'],
    ],
    columnStyles: { 0: { fontStyle: 'bold', cellWidth: 32 } },
    margin: { left: marginX, right: marginX },
  })
  y = (doc as any).lastAutoTable.finalY + 8

  // ── Resume Peluang: Pilihan 1 vs Pilihan 2 (kalau keduanya ada) ──
  if (input.pilihan.length === 2) {
    ensureSpace(30)
    sectionTitle(doc, 'Resume Peluang: Pilihan 1 vs Pilihan 2', marginX, y)
    y += 2
    const [p1, p2] = input.pilihan
    autoTable(doc, {
      startY: y + 2,
      head: [['', p1.label, p2.label]],
      body: [
        ['Universitas', p1.nama_ptn, p2.nama_ptn],
        ['Program Studi', p1.nama_prodi, p2.nama_prodi],
        ['Peluang Kompetisi', `${p1.chance_percent}% (${p1.tier_label})`, `${p2.chance_percent}% (${p2.tier_label})`],
      ],
      styles: { fontSize: 9, cellPadding: 2 },
      headStyles: { fillColor: BRAND },
      columnStyles: { 0: { fontStyle: 'bold', cellWidth: 34 } },
      margin: { left: marginX, right: marginX },
      didParseCell: (data) => {
        if (data.section === 'body' && data.row.index === 2 && (data.column.index === 1 || data.column.index === 2)) {
          const t = data.column.index === 1 ? p1.tier : p2.tier
          data.cell.styles.fillColor = tierBgRgb(t)
          data.cell.styles.textColor = tierRgb(t)
          data.cell.styles.fontStyle = 'bold'
        }
      },
    })
    y = (doc as any).lastAutoTable.finalY + 8
  }

  // ── Pilihan 1 & 2 (breakdown peluang lengkap + grafik + alternatif) ──
  for (const p of input.pilihan) {
    ensureSpace(40)

    doc.setFontSize(11)
    doc.setFont('helvetica', 'bold')
    doc.setTextColor(...BRAND)
    doc.text(`${p.label} — ${p.nama_ptn}`, marginX, y)
    doc.setTextColor(0)
    y += 5

    doc.setFontSize(9)
    doc.setFont('helvetica', 'normal')
    doc.text(p.nama_prodi, marginX, y)
    y += 6

    doc.setFont('helvetica', 'bold')
    doc.text('Peluang Kompetisi:', marginX, y)
    const pillW = drawTierPill(doc, marginX + 32, y, p.tier, `${p.chance_percent}% · ${p.tier_label}`)
    void pillW
    y += 4

    autoTable(doc, {
      startY: y + 2,
      head: [['Komponen Skor Bobot Rasionalisasi', 'Nilai']],
      body: [
        ['Nilai Rapor vs Passing Grade', `${p.nilai_contribution > 0 ? '+' : ''}${p.nilai_contribution}`],
        ['Rasio Persaingan Program', `${p.competition_adjustment > 0 ? '+' : ''}${Math.round(p.competition_adjustment * 10) / 10}`],
        ['Index Prestasi', `+${Math.round(p.prestasi_contribution * 10) / 10}`],
        ['Total Index', `${p.chance_percent}%`],
      ],
      styles: { fontSize: 9 },
      headStyles: { fillColor: BRAND },
      margin: { left: marginX, right: marginX },
    })
    y = (doc as any).lastAutoTable.finalY + 6

    autoTable(doc, {
      startY: y,
      head: [['Min. Nilai Rapor', 'Min UTBK (ref. SNBT)', 'Kuota SNBP', 'Peminat SNBP', 'Persaingan', 'Rumpun']],
      body: [[
        String(p.pg_snbp),
        String(p.pg_snbt),
        `${p.daya_tampung_snbp} kursi`,
        `${p.peminat_snbp} orang`,
        `1:${p.daya_tampung_snbp > 0 ? Math.round(p.peminat_snbp / p.daya_tampung_snbp) : '-'}`,
        p.rumpun,
      ]],
      styles: { fontSize: 8 },
      headStyles: { fillColor: BRAND },
      margin: { left: marginX, right: marginX },
    })
    y = (doc as any).lastAutoTable.finalY + 8

    // ── Grafik radar "Pencapaian vs Referensi PG" — real, dari nilai rapor siswa ──
    if (p.radar_chart_image) {
      const imgSize = 78
      ensureSpace(imgSize + 12)
      doc.setFontSize(9.5)
      doc.setFont('helvetica', 'bold')
      doc.setTextColor(30, 30, 40)
      doc.text('Grafik Pencapaian vs Referensi PG', marginX, y)
      doc.setTextColor(0)
      y += 3
      const imgX = marginX + (pageRight - marginX - imgSize) / 2
      try {
        doc.addImage(p.radar_chart_image, 'PNG', imgX, y, imgSize, imgSize)
      } catch {
        // gagal render gambar (mis. data URL tidak valid) — lewati saja, sisa laporan tetap utuh
      }
      y += imgSize + 8
    }

    // ── Alternatif Sesuai Rumpun Program Studi ──
    ensureSpace(28)
    sectionTitle(doc, `Alternatif Sesuai Rumpun (${p.rumpun})`, marginX, y)
    y += 2
    y = alternatifTable(doc, y + 2, marginX, p.alternatif_rumpun, 'Tidak ada alternatif lain di rumpun yang sama.') + 4

    // ── Alternatif Sesuai Minat Jurusan ──
    ensureSpace(30)
    sectionTitle(doc, 'Alternatif Sesuai Minat Jurusan', marginX, y)
    y += 4
    doc.setFontSize(8)
    doc.setFont('helvetica', p.minat_is_fallback ? 'italic' : 'normal')
    doc.setTextColor(100)
    const minatNote = p.minat_is_fallback
      ? `Berdasarkan program yang dipilih (${p.minat_label}) — siswa belum mengisi "Jurusan yang Diminati" secara eksplisit.`
      : `Minat: ${p.minat_label}`
    doc.text(minatNote, marginX, y)
    doc.setTextColor(0)
    doc.setFont('helvetica', 'normal')
    y += 3
    y = alternatifTable(doc, y + 2, marginX, p.alternatif_minat, 'Tidak ada alternatif lain untuk minat jurusan ini.') + 10
  }

  // ── Rekap Nilai Rapor Siswa ──
  ensureSpace(30)
  sectionTitle(doc, 'Rekap Nilai Rapor Siswa', marginX, y)
  autoTable(doc, {
    startY: y + 4,
    head: [['Mata Pelajaran', 'Rata-rata']],
    body: input.rapor.length ? input.rapor.map((r) => [r.subject, String(r.avg)]) : [['Belum ada nilai rapor.', '']],
    styles: { fontSize: 9 },
    headStyles: { fillColor: BRAND },
    margin: { left: marginX, right: marginX },
  })
  y = (doc as any).lastAutoTable.finalY + 16

  // ── Penutup formal: tanggal + kolom tanda tangan ──
  ensureSpace(45)
  doc.setFontSize(9)
  doc.setFont('helvetica', 'normal')
  doc.setTextColor(90)
  doc.text(`Dokumen ini dibuat otomatis oleh sistem pada ${formatDateID(new Date())}.`, marginX, y)
  doc.setTextColor(0)
  y += 20

  const leftX = marginX
  const rightX = marginX + 100
  doc.setFont('helvetica', 'normal')
  doc.text('Mengetahui,', leftX, y)
  doc.text('Orang Tua/Wali,', rightX, y)
  y += 20
  doc.text('(_________________________)', leftX, y)
  doc.text('(_________________________)', rightX, y)
  y += 5
  doc.setFont('helvetica', 'bold')
  doc.text('Konselor/PIC Sekolah', leftX, y)
  doc.text('Nama & Tanda Tangan', rightX, y)

  const ptnSlug = slug(input.pilihan[0]?.nama_ptn || 'ptn')
  doc.save(`laporan-rasionalisasi-${slug(input.student_name)}-${ptnSlug}.pdf`)
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// Laporan ranking rasionalisasi SNBP — versi RINGKASAN/TABEL untuk SELURUH siswa yang
// sedang tampil di tab "Rasionalisasi" (school/SchoolRasionalisasi.vue, filteredRankingCards),
// bukan satu siswa. Dipakai pihak sekolah/rekap internal (bukan diserahkan ke orang tua satu
// per satu — untuk itu pakai exportRationalizationReportToPdf di atas). Baris tabel PERSIS
// urutan & isi filteredRankingCards saat tombol diklik, termasuk filter tahun/universitas
// yang sedang aktif — supaya dokumen ini selalu mencerminkan apa yang terlihat di layar, bukan
// selalu daftar penuh tanpa filter.
export interface RankingListPilihanInput {
  nama_ptn: string
  chance_percent: number
}

export interface RankingListRowInput {
  student_name: string
  kelas: string | null
  pilihan_1: RankingListPilihanInput | null
  pilihan_2: RankingListPilihanInput | null
  best_chance_percent: number
  /** Kunci tier mentah ('aman'/'moderat'/'ketat') untuk pewarnaan sel "Peluang Terbaik". */
  tier: string
  /** Label tier yang sudah diterjemahkan (mis. "Aman"/"Moderat"/"Ketat") — diterjemahkan oleh
   * caller karena TIER_LABEL adalah konvensi tampilan komponen, bukan util generik ini. */
  tier_label: string
}

export interface RankingListReportInput {
  school_name?: string | null
  /** Filter tahun yang sedang aktif di layar ('' = semua tahun). */
  year_filter: number | ''
  /** Filter universitas yang sedang aktif di layar ('' = semua universitas). */
  univ_filter: string
  /** Baris SUDAH terfilter & dalam urutan ranking asli (jangan diurutkan ulang di sini). */
  rows: RankingListRowInput[]
}

function pilihanCellLabel(p: RankingListPilihanInput | null): string {
  return p ? `${p.nama_ptn} (${p.chance_percent}%)` : 'Belum ada'
}

function dateStamp(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

export function exportRankingListToPdf(input: RankingListReportInput) {
  const doc = new jsPDF()
  const marginX = 14
  const pageRight = 196
  let y = 20

  // ── Accent bar atas — konsisten dengan laporan per-siswa ──
  doc.setFillColor(...BRAND)
  doc.rect(0, 0, 210, 2.6, 'F')

  // ── Letterhead ──
  doc.setFontSize(14)
  doc.setFont('helvetica', 'bold')
  doc.text(input.school_name || 'Laporan Sekolah', marginX, y)
  y += 6

  doc.setFontSize(11)
  doc.setFont('helvetica', 'normal')
  doc.setTextColor(90)
  doc.text('Laporan Ranking Rasionalisasi SNBP', marginX, y)
  y += 5
  doc.setFontSize(9)
  doc.text(`Dibuat ${formatDateID(new Date())}`, marginX, y)
  y += 5

  // ── Konteks filter aktif — supaya dokumen ini jelas mewakili tampilan yang mana ──
  doc.setFont('helvetica', 'bold')
  doc.text(
    `Tahun: ${input.year_filter === '' ? 'Semua Tahun' : String(input.year_filter)}   ·   Universitas: ${input.univ_filter || 'Semua Universitas'}`,
    marginX, y,
  )
  y += 5
  doc.setFont('helvetica', 'normal')
  doc.text(`Total Siswa: ${input.rows.length}`, marginX, y)
  doc.setTextColor(0)
  y += 3
  doc.setDrawColor(...BRAND)
  doc.setLineWidth(0.6)
  doc.line(marginX, y, pageRight, y)
  y += 6

  // ── Tabel ranking ──
  autoTable(doc, {
    startY: y,
    head: [['Rank', 'Nama', 'Kelas', 'Pilihan 1', 'Pilihan 2', 'Peluang Terbaik']],
    body: input.rows.length
      ? input.rows.map((r, i) => [
          String(i + 1),
          r.student_name,
          r.kelas || '-',
          pilihanCellLabel(r.pilihan_1),
          pilihanCellLabel(r.pilihan_2),
          `${r.best_chance_percent}% (${r.tier_label})`,
        ])
      : [['Belum ada siswa dengan target SNBP.', '', '', '', '', '']],
    styles: { fontSize: 8, cellPadding: 2 },
    headStyles: { fillColor: BRAND },
    columnStyles: { 0: { cellWidth: 12 }, 2: { cellWidth: 18 } },
    margin: { left: marginX, right: marginX },
    didParseCell: (data) => {
      if (data.section === 'body' && data.column.index === 5 && input.rows[data.row.index]) {
        const t = input.rows[data.row.index].tier
        data.cell.styles.fillColor = tierBgRgb(t)
        data.cell.styles.textColor = tierRgb(t)
        data.cell.styles.fontStyle = 'bold'
      }
    },
  })

  const filterParts = [
    input.year_filter !== '' ? String(input.year_filter) : '',
    input.univ_filter ? slug(input.univ_filter) : '',
  ].filter(Boolean)
  const filterSlug = filterParts.length ? `-${filterParts.join('-')}` : ''
  doc.save(`laporan-ranking-rasionalisasi${filterSlug}-${dateStamp(new Date())}.pdf`)
}
