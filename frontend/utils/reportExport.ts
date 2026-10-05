// Export laporan ke PDF (jsPDF + jspdf-autotable) dan Excel (SheetJS/xlsx) — sungguhan,
// dibangun langsung dari `ReportItem.payload` yang sama persis dengan yang ditampilkan
// di dialog detail (bukan data terpisah/fiktif). Satu file util dipakai dari
// SchoolReports.vue supaya tombol "Unduh PDF"/"Unduh Excel" konsisten untuk keempat jenis
// laporan (performance/participation/ptn-target/tka).
import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import * as XLSX from 'xlsx'
import type { ReportItem } from '~/types'
import { EDVION_WORDMARK_ASPECT, EDVION_WORDMARK_PNG_BASE64 } from '~/utils/edvionLogo'

// Warna brand resmi Edvion (sama dengan utils/simulationCertificate.ts) — dipakai konsisten
// di seluruh dokumen PDF yang diterbitkan platform ini.
const NAVY: [number, number, number] = [13, 19, 33]
const GREEN: [number, number, number] = [0, 214, 163]
const SLATE = 100

function typeLabel(t: ReportItem['report_type']): string {
  if (t === 'performance') return 'Performa Siswa'
  if (t === 'participation') return 'Partisipasi Tryout'
  if (t === 'tka') return 'Hasil TKA'
  if (t === 'akreditasi') return 'Laporan Akreditasi (EDS)'
  return 'Analisis Target PTN'
}

function examTrackLabel(v: ReportItem['exam_track_filter']): string {
  if (v === 'snbt') return 'SNBT / UTBK'
  if (v === 'tka') return 'TKA'
  return 'Semua Jalur'
}

function fileBaseName(report: ReportItem): string {
  const safe = report.title.replace(/[^a-z0-9]+/gi, '-').replace(/^-+|-+$/g, '').toLowerCase()
  return safe || `laporan-${report.id}`
}

function formatDateTime(iso: string): string {
  return new Date(iso).toLocaleString('id-ID', { day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' })
}

/** Header standar tiap laporan: logo Edvion + judul + meta line + garis pemisah. Mengembalikan
 *  posisi Y setelah header, siap dipakai untuk konten berikutnya. */
function drawHeader(doc: jsPDF, report: ReportItem, marginX: number): number {
  const logoH = 7
  const logoW = logoH * EDVION_WORDMARK_ASPECT
  doc.addImage(`data:image/png;base64,${EDVION_WORDMARK_PNG_BASE64}`, 'PNG', marginX, 10, logoW, logoH)

  let y = 26
  doc.setFontSize(14)
  doc.setFont('helvetica', 'bold')
  doc.setTextColor(NAVY[0], NAVY[1], NAVY[2])
  doc.text(report.title, marginX, y)
  y += 6

  doc.setFontSize(8.5)
  doc.setFont('helvetica', 'normal')
  doc.setTextColor(SLATE)
  const metaLine = `${typeLabel(report.report_type)} · Periode ${report.period_label} · Cakupan ${report.class_filter || 'Semua Kelas'} · Jalur ${examTrackLabel(report.exam_track_filter)} · Dibuat ${formatDateTime(report.created_at)}`
  doc.text(metaLine, marginX, y)
  y += 4

  doc.setDrawColor(GREEN[0], GREEN[1], GREEN[2])
  doc.setLineWidth(0.6)
  doc.line(marginX, y, doc.internal.pageSize.getWidth() - marginX, y)
  doc.setTextColor(0)
  return y + 8
}

/** Baris kartu KPI ringkas — dipakai sebagai pengganti teks "Label: nilai" polos, supaya angka
 *  kunci laporan langsung menonjol secara visual di bagian atas dokumen. */
function drawKpiCards(doc: jsPDF, marginX: number, y: number, cards: { label: string; value: string; accent?: boolean }[]): number {
  const pageW = doc.internal.pageSize.getWidth()
  const contentW = pageW - marginX * 2
  const gap = 4
  const w = (contentW - gap * (cards.length - 1)) / cards.length
  const h = 20
  cards.forEach((c, i) => {
    const x = marginX + i * (w + gap)
    doc.setFillColor(c.accent ? 230 : 245, c.accent ? 253 : 246, c.accent ? 244 : 250)
    doc.setDrawColor(c.accent ? GREEN[0] : 225, c.accent ? GREEN[1] : 227, c.accent ? GREEN[2] : 232)
    doc.setLineWidth(0.3)
    doc.roundedRect(x, y, w, h, 2, 2, 'FD')
    doc.setFont('helvetica', 'normal')
    doc.setFontSize(7)
    doc.setTextColor(SLATE)
    doc.text(c.label.toUpperCase(), x + w / 2, y + 6, { align: 'center', maxWidth: w - 4 })
    doc.setFont('helvetica', 'bold')
    doc.setFontSize(14)
    doc.setTextColor(c.accent ? GREEN[0] : NAVY[0], c.accent ? GREEN[1] : NAVY[1], c.accent ? GREEN[2] : NAVY[2])
    doc.text(c.value, x + w / 2, y + 15, { align: 'center' })
  })
  doc.setTextColor(0)
  return y + h + 8
}

function sectionTitle(doc: jsPDF, text: string, marginX: number, y: number): number {
  doc.setFont('helvetica', 'bold')
  doc.setFontSize(10.5)
  doc.setTextColor(NAVY[0], NAVY[1], NAVY[2])
  doc.text(text, marginX, y)
  doc.setTextColor(0)
  return y + 4
}

function footer(doc: jsPDF, marginX: number) {
  const pageCount = doc.getNumberOfPages()
  for (let p = 1; p <= pageCount; p++) {
    doc.setPage(p)
    const pageW = doc.internal.pageSize.getWidth()
    const pageH = doc.internal.pageSize.getHeight()
    doc.setDrawColor(235)
    doc.setLineWidth(0.2)
    doc.line(marginX, pageH - 14, pageW - marginX, pageH - 14)
    doc.setFont('helvetica', 'normal')
    doc.setFontSize(7)
    doc.setTextColor(160)
    doc.text('Dibuat otomatis oleh Edvion dari data sekolah asli — bukan data simulasi/dikarang.', marginX, pageH - 9)
    doc.text(`Halaman ${p} dari ${pageCount}`, pageW - marginX, pageH - 9, { align: 'right' })
  }
}

// ─── PDF ──────────────────────────────────────────────────────────────────────────────
export function exportReportToPdf(report: ReportItem) {
  const doc = new jsPDF()
  const marginX = 14
  let y = drawHeader(doc, report, marginX)

  const payload = report.payload

  if (payload.kind === 'performance') {
    y = drawKpiCards(doc, marginX, y, [
      { label: 'Total Siswa', value: String(payload.total_students) },
      { label: 'Rata-rata Skor', value: payload.avg_score != null ? String(Math.round(payload.avg_score)) : '-', accent: true },
    ])

    y = sectionTitle(doc, 'Rata-rata per Kelas', marginX, y)
    autoTable(doc, {
      startY: y + 2,
      head: [['Kelas', 'Jumlah Siswa', 'Rata-rata']],
      body: payload.per_class.length
        ? payload.per_class.map((c) => [c.grade, String(c.student_count), c.avg_score != null ? String(Math.round(c.avg_score)) : '-'])
        : [['Belum ada data.', '', '']],
      styles: { fontSize: 9 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
    })
    y = (doc as any).lastAutoTable.finalY + 8

    y = sectionTitle(doc, 'Tren Skor Bulanan', marginX, y)
    autoTable(doc, {
      startY: y + 2,
      head: [['Bulan', 'Rata-rata Skor', 'Jumlah Attempt']],
      body: payload.monthly_trend.length
        ? payload.monthly_trend.map((m) => [m.month_label, m.avg_score != null ? String(Math.round(m.avg_score)) : '-', String(m.attempt_count)])
        : [['Belum cukup data.', '', '']],
      styles: { fontSize: 9 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
    })
    y = (doc as any).lastAutoTable.finalY + 8

    y = sectionTitle(doc, 'Ranking Internal (Top 10)', marginX, y)
    autoTable(doc, {
      startY: y + 2,
      head: [['Siswa', 'Kelas', 'Attempt', 'Terbaik', 'Rata-rata']],
      body: payload.ranking.length
        ? payload.ranking.map((s) => [s.student_name, s.grade || '-', String(s.attempt_count), s.best_score != null ? String(s.best_score) : '-', s.avg_score != null ? String(Math.round(s.avg_score)) : '-'])
        : [['Belum ada siswa dengan attempt selesai.', '', '', '', '']],
      styles: { fontSize: 9 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
    })
  } else if (payload.kind === 'participation') {
    y = drawKpiCards(doc, marginX, y, [
      { label: 'Total Siswa', value: String(payload.total_students) },
      { label: 'Aktif', value: String(payload.active_students), accent: true },
      { label: 'Partisipasi', value: `${Math.round(payload.participation_rate)}%` },
      { label: 'Total Attempt', value: String(payload.total_attempts) },
    ])

    y = sectionTitle(doc, 'Detail per Siswa', marginX, y)
    autoTable(doc, {
      startY: y + 2,
      head: [['Siswa', 'Kelas', 'Jumlah Attempt', 'Terakhir Aktif']],
      body: payload.rows.length
        ? payload.rows.map((r) => [r.student_name, r.grade || '-', String(r.attempt_count), r.last_attempt_at ? formatDateTime(r.last_attempt_at) : '-'])
        : [['Belum ada siswa.', '', '', '']],
      styles: { fontSize: 9 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
    })
  } else if (payload.kind === 'tka') {
    const totalIstimewa = payload.students.reduce((sum, s) => sum + s.istimewa_count, 0)
    y = drawKpiCards(doc, marginX, y, [
      { label: 'Total Siswa', value: String(payload.total_students) },
      { label: 'Paket TKA', value: String(payload.packages.length) },
      { label: 'Mata Uji Istimewa', value: String(totalIstimewa), accent: true },
    ])

    y = sectionTitle(doc, 'Ringkasan per Paket', marginX, y)
    autoTable(doc, {
      startY: y + 2,
      head: [['Paket', 'Skala', 'Siswa', 'Siap Pilih', 'Rata² Wajib', 'Rata² Pilihan', 'Rata² Istimewa']],
      body: payload.packages.length
        ? payload.packages.map((p) => [
            p.package_name,
            `${p.score_scale_label} (>=${p.istimewa_threshold})`,
            String(p.student_count),
            `${p.elective_ready_count}/${p.student_count}`,
            p.avg_wajib_score != null ? String(Math.round(p.avg_wajib_score)) : '-',
            p.avg_pilihan_score != null ? String(Math.round(p.avg_pilihan_score)) : '-',
            p.avg_istimewa_count != null ? p.avg_istimewa_count.toFixed(1) : '-',
          ])
        : [['Belum ada paket TKA dengan konten nyata.', '', '', '', '', '', '']],
      styles: { fontSize: 9 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
    })
    y = (doc as any).lastAutoTable.finalY + 5
    doc.setFontSize(7.5)
    doc.setFont('helvetica', 'normal')
    doc.setTextColor(140)
    doc.text('Skala nilai mengikuti jenjang tiap paket (bukan skor resmi TKA Kemendikdasmen) — perkiraan platform dari hasil pengerjaan siswa.', marginX, y)
    doc.setTextColor(0)
    y += 7

    y = sectionTitle(doc, 'Detail per Siswa', marginX, y)
    const istimewaColIndex = 6
    autoTable(doc, {
      startY: y + 2,
      head: [['Siswa', 'Kelas', 'Paket', 'Pilihan', 'Rata² Wajib', 'Rata² Pilihan', 'Istimewa']],
      body: payload.students.length
        ? payload.students.map((s) => [
            s.student_name,
            s.grade || '-',
            s.package_name,
            `${s.elective_chosen_count}/${s.elective_pick_count}`,
            s.avg_wajib_score != null ? String(Math.round(s.avg_wajib_score)) : '-',
            s.avg_pilihan_score != null ? String(Math.round(s.avg_pilihan_score)) : '-',
            `${s.istimewa_count}/${s.subject_count}`,
          ])
        : [['Belum ada siswa.', '', '', '', '', '', '']],
      styles: { fontSize: 9 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
      didParseCell: (data) => {
        if (data.section === 'body' && data.column.index === istimewaColIndex) {
          const raw = String(data.cell.raw ?? '')
          const [got] = raw.split('/')
          if (Number(got) > 0) {
            data.cell.styles.textColor = [16, 150, 110]
            data.cell.styles.fontStyle = 'bold'
          }
        }
      },
    })
  } else if (payload.kind === 'akreditasi') {
    // Laporan Akreditasi (EDS) sengaja tidak didukung sebagai PDF — tabelnya (per mata
    // pelajaran + per kelas + per siswa) lebih cocok dibuka/diolah sebagai Excel. Lempar
    // pesan ramah yang ditangkap oleh try/catch di SchoolReports.vue (muncul sebagai toast).
    throw new Error('Laporan Akreditasi (EDS) belum tersedia dalam format PDF — silakan gunakan "Unduh Excel".')
  } else {
    y = drawKpiCards(doc, marginX, y, [
      { label: 'Siswa Punya Target', value: String(payload.total_siswa_dengan_target) },
      { label: 'Rata² Peluang SNBP', value: payload.avg_chance_snbp != null ? `${Math.round(payload.avg_chance_snbp)}%` : '-', accent: true },
      { label: 'SNBT Terdaftar', value: String(payload.snbt_terdaftar) },
      { label: 'SNBT Diterima', value: String(payload.snbt_diterima), accent: true },
    ])

    y = sectionTitle(doc, 'Sebaran Universitas Tujuan (SNBP)', marginX, y)
    autoTable(doc, {
      startY: y + 2,
      head: [['Universitas', 'Jumlah Siswa']],
      body: payload.distribusi_universitas.length
        ? payload.distribusi_universitas.map((d) => [d.nama_ptn, String(d.count)])
        : [['Belum ada siswa yang menambahkan target SNBP.', '']],
      styles: { fontSize: 9 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
    })
  }

  footer(doc, marginX)
  doc.save(`${fileBaseName(report)}.pdf`)
}

// ─── Excel ────────────────────────────────────────────────────────────────────────────
export function exportReportToExcel(report: ReportItem) {
  const wb = XLSX.utils.book_new()
  const payload = report.payload

  const summaryRows: Record<string, string | number>[] = [
    { Field: 'Judul', Value: report.title },
    { Field: 'Jenis', Value: typeLabel(report.report_type) },
    { Field: 'Periode', Value: report.period_label },
    { Field: 'Cakupan', Value: report.class_filter || 'Semua Kelas' },
    { Field: 'Jalur Ujian', Value: examTrackLabel(report.exam_track_filter) },
    { Field: 'Dibuat', Value: formatDateTime(report.created_at) },
  ]
  XLSX.utils.book_append_sheet(wb, XLSX.utils.json_to_sheet(summaryRows), 'Ringkasan')

  if (payload.kind === 'performance') {
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(payload.per_class.map((c) => ({ Kelas: c.grade, 'Jumlah Siswa': c.student_count, 'Rata-rata': c.avg_score != null ? Math.round(c.avg_score) : null }))),
      'Per Kelas',
    )
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(payload.monthly_trend.map((m) => ({ Bulan: m.month_label, 'Rata-rata Skor': m.avg_score != null ? Math.round(m.avg_score) : null, 'Jumlah Attempt': m.attempt_count }))),
      'Tren Bulanan',
    )
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(payload.ranking.map((s) => ({ Siswa: s.student_name, Kelas: s.grade || '-', Attempt: s.attempt_count, Terbaik: s.best_score ?? null, 'Rata-rata': s.avg_score != null ? Math.round(s.avg_score) : null }))),
      'Ranking',
    )
  } else if (payload.kind === 'participation') {
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(payload.rows.map((r) => ({ Siswa: r.student_name, Kelas: r.grade || '-', 'Jumlah Attempt': r.attempt_count, 'Terakhir Aktif': r.last_attempt_at ? formatDateTime(r.last_attempt_at) : '-' }))),
      'Detail Siswa',
    )
  } else if (payload.kind === 'tka') {
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(
        payload.packages.map((p) => ({
          Paket: p.package_name,
          'Skala Nilai': p.score_scale_label,
          'Ambang Istimewa': p.istimewa_threshold,
          Siswa: p.student_count,
          'Siap Pilih': `${p.elective_ready_count}/${p.student_count}`,
          'Rata-rata Wajib': p.avg_wajib_score != null ? Math.round(p.avg_wajib_score) : null,
          'Rata-rata Pilihan': p.avg_pilihan_score != null ? Math.round(p.avg_pilihan_score) : null,
          'Skor Mentah Wajib (0-1000)': p.avg_raw_wajib_score != null ? Math.round(p.avg_raw_wajib_score) : null,
          'Skor Mentah Pilihan (0-1000)': p.avg_raw_pilihan_score != null ? Math.round(p.avg_raw_pilihan_score) : null,
          'Rata-rata Mata Uji Istimewa': p.avg_istimewa_count != null ? Number(p.avg_istimewa_count.toFixed(1)) : null,
        })),
      ),
      'Ringkasan Paket TKA',
    )
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(
        payload.students.map((s) => ({
          Siswa: s.student_name,
          Kelas: s.grade || '-',
          Paket: s.package_name,
          'Skala Nilai': s.score_scale_label,
          'Mata Uji Pilihan': `${s.elective_chosen_count}/${s.elective_pick_count}`,
          'Rata-rata Wajib': s.avg_wajib_score != null ? Math.round(s.avg_wajib_score) : null,
          'Rata-rata Pilihan': s.avg_pilihan_score != null ? Math.round(s.avg_pilihan_score) : null,
          'Skor Mentah Wajib (0-1000)': s.avg_raw_wajib_score != null ? Math.round(s.avg_raw_wajib_score) : null,
          'Skor Mentah Pilihan (0-1000)': s.avg_raw_pilihan_score != null ? Math.round(s.avg_raw_pilihan_score) : null,
          Istimewa: `${s.istimewa_count}/${s.subject_count}`,
        })),
      ),
      'Detail Siswa TKA',
    )
  } else if (payload.kind === 'akreditasi') {
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet([
        { Field: 'Ambang Nilai Kelulusan (KKM)', Value: payload.threshold_used },
        { Field: 'Total Siswa', Value: payload.total_students },
        { Field: 'Rata-rata Skor', Value: payload.avg_score_overall != null ? Math.round(payload.avg_score_overall) : null },
        { Field: 'Siswa di Atas KKM', Value: payload.students_above_threshold },
        { Field: 'Siswa Punya Target PTN', Value: payload.students_with_ptn_target },
      ]),
      'Ringkasan Akreditasi',
    )
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(
        payload.per_subject.map((s) => ({
          'Mata Pelajaran': s.subject,
          'Jumlah Soal Dikerjakan': s.total,
          'Jumlah Benar': s.correct,
          'Akurasi (%)': s.total > 0 ? Math.round((s.correct / s.total) * 100) : 0,
        })),
      ),
      'Per Mata Pelajaran',
    )
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(
        payload.per_class.map((c) => ({
          Kelas: c.grade,
          'Jumlah Siswa': c.student_count,
          'Rata-rata Skor': c.avg_score != null ? Math.round(c.avg_score) : null,
          'Di Atas KKM': `${c.above_threshold_count}/${c.student_count}`,
        })),
      ),
      'Per Kelas',
    )
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(
        payload.students.map((s) => ({
          Siswa: s.student_name,
          Kelas: s.grade || '-',
          Attempt: s.attempt_count,
          'Rata-rata Skor': s.avg_score != null ? Math.round(s.avg_score) : null,
          Status: s.avg_score == null ? '-' : s.above_threshold ? 'Di Atas KKM' : 'Di Bawah KKM',
          'Punya Target PTN': s.has_ptn_target ? 'Ya' : '-',
        })),
      ),
      'Detail Siswa',
    )
  } else {
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(payload.distribusi_universitas.map((d) => ({ Universitas: d.nama_ptn, 'Jumlah Siswa': d.count }))),
      'Sebaran Universitas',
    )
  }

  XLSX.writeFile(wb, `${fileBaseName(report)}.xlsx`)
}
