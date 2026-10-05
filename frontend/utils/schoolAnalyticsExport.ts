// Ekspor "Analytics & Insights" Sekolah ke PDF — dibangun langsung dari data yang sama
// persis yang sudah dihitung & ditampilkan di SchoolAnalytics.vue (KPI, ringkasan per
// kelas, insight, penguasaan mata uji, daftar siswa), bukan data terpisah/dikarang.
// Gaya & helper (logo, warna brand, kartu KPI) sengaja disalin dari pola yang sama di
// utils/reportExport.ts / utils/simulationCertificate.ts / utils/attemptExport.ts supaya
// semua dokumen PDF yang diterbitkan platform ini terlihat konsisten satu sama lain.
import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import * as XLSX from 'xlsx'
import { EDVION_WORDMARK_ASPECT, EDVION_WORDMARK_PNG_BASE64 } from '~/utils/edvionLogo'

const NAVY: [number, number, number] = [13, 19, 33]
const GREEN: [number, number, number] = [0, 214, 163]
const SLATE = 100

export interface SchoolAnalyticsExportData {
  schoolName: string
  generatedAt: string
  kpis: { label: string; value: string }[]
  kelasSummaries: { rombel: string; total: number; aktif: number; avgScore: number | null }[]
  yearlyPerformance: { year: number; attempts: number; distinctStudents: number; avgScore: number }[]
  insights: string[]
  subjects: { subject: string; accuracy: number }[]
  students: { name: string; rombel: string; attempts: number; avgScore: number | null; status: string }[]
}

function formatDateTime(iso: string): string {
  return new Date(iso).toLocaleString('id-ID', { day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' })
}

function drawHeader(doc: jsPDF, data: SchoolAnalyticsExportData, marginX: number): number {
  const logoH = 7
  const logoW = logoH * EDVION_WORDMARK_ASPECT
  doc.addImage(`data:image/png;base64,${EDVION_WORDMARK_PNG_BASE64}`, 'PNG', marginX, 10, logoW, logoH)

  let y = 26
  doc.setFontSize(14)
  doc.setFont('helvetica', 'bold')
  doc.setTextColor(NAVY[0], NAVY[1], NAVY[2])
  doc.text('Analytics & Insights Sekolah', marginX, y)
  y += 6

  doc.setFontSize(8.5)
  doc.setFont('helvetica', 'normal')
  doc.setTextColor(SLATE)
  doc.text(`${data.schoolName} · Dibuat ${formatDateTime(data.generatedAt)}`, marginX, y)
  y += 4

  doc.setDrawColor(GREEN[0], GREEN[1], GREEN[2])
  doc.setLineWidth(0.6)
  doc.line(marginX, y, doc.internal.pageSize.getWidth() - marginX, y)
  doc.setTextColor(0)
  return y + 8
}

function drawKpiCards(doc: jsPDF, marginX: number, y: number, cards: { label: string; value: string; accent?: boolean }[]): number {
  const pageW = doc.internal.pageSize.getWidth()
  const contentW = pageW - marginX * 2
  const perRow = 3
  const gap = 4
  const w = (contentW - gap * (perRow - 1)) / perRow
  const h = 20
  const rowH = h + gap
  cards.forEach((c, i) => {
    const col = i % perRow
    const row = Math.floor(i / perRow)
    const x = marginX + col * (w + gap)
    const cy = y + row * rowH
    doc.setFillColor(c.accent ? 230 : 245, c.accent ? 253 : 246, c.accent ? 244 : 250)
    doc.setDrawColor(c.accent ? GREEN[0] : 225, c.accent ? GREEN[1] : 227, c.accent ? GREEN[2] : 232)
    doc.setLineWidth(0.3)
    doc.roundedRect(x, cy, w, h, 2, 2, 'FD')
    doc.setFont('helvetica', 'normal')
    doc.setFontSize(7)
    doc.setTextColor(SLATE)
    doc.text(c.label.toUpperCase(), x + w / 2, cy + 6, { align: 'center', maxWidth: w - 4 })
    doc.setFont('helvetica', 'bold')
    doc.setFontSize(14)
    doc.setTextColor(c.accent ? GREEN[0] : NAVY[0], c.accent ? GREEN[1] : NAVY[1], c.accent ? GREEN[2] : NAVY[2])
    doc.text(c.value, x + w / 2, cy + 15, { align: 'center' })
  })
  doc.setTextColor(0)
  const rows = Math.ceil(cards.length / perRow)
  return y + rows * rowH + 4
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

export function exportSchoolAnalyticsToPdf(data: SchoolAnalyticsExportData) {
  const doc = new jsPDF()
  const marginX = 14
  let y = drawHeader(doc, data, marginX)

  y = drawKpiCards(doc, marginX, y, data.kpis.map((k) => ({ ...k })))

  if (data.insights.length > 0) {
    y = sectionTitle(doc, 'Insight & Rekomendasi', marginX, y)
    autoTable(doc, {
      startY: y + 2,
      body: data.insights.map((text) => [text]),
      styles: { fontSize: 8.5, cellPadding: 2.5 },
      showHead: false,
      margin: { left: marginX, right: marginX },
    })
    y = (doc as any).lastAutoTable.finalY + 8
  }

  if (data.kelasSummaries.length > 0) {
    y = sectionTitle(doc, 'Ringkasan per Kelas', marginX, y)
    autoTable(doc, {
      startY: y + 2,
      head: [['Kelas', 'Total Siswa', 'Siswa Aktif', 'Skor Rata-rata']],
      body: data.kelasSummaries.map((k) => [k.rombel, String(k.total), String(k.aktif), k.avgScore != null ? String(k.avgScore) : '-']),
      styles: { fontSize: 8.5 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
    })
    y = (doc as any).lastAutoTable.finalY + 8
  }

  if (data.yearlyPerformance.length > 0) {
    if (y > 250) { doc.addPage(); y = 20 }
    y = sectionTitle(doc, 'Perbandingan Tahun', marginX, y)
    autoTable(doc, {
      startY: y + 2,
      head: [['Tahun', 'Jumlah Attempt', 'Siswa Aktif', 'Skor Rata-rata']],
      body: data.yearlyPerformance.map((yp) => [String(yp.year), String(yp.attempts), String(yp.distinctStudents), String(yp.avgScore)]),
      styles: { fontSize: 8.5 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
    })
    y = (doc as any).lastAutoTable.finalY + 8
  }

  if (data.subjects.length > 0) {
    if (y > 250) { doc.addPage(); y = 20 }
    y = sectionTitle(doc, 'Penguasaan per Mata Uji', marginX, y)
    autoTable(doc, {
      startY: y + 2,
      head: [['Mata Uji', 'Akurasi']],
      body: data.subjects.map((s) => [s.subject, `${s.accuracy}%`]),
      styles: { fontSize: 8.5 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
    })
    y = (doc as any).lastAutoTable.finalY + 8
  }

  if (data.students.length > 0) {
    doc.addPage()
    y = 20
    y = sectionTitle(doc, `Daftar Siswa (${data.students.length})`, marginX, y)
    autoTable(doc, {
      startY: y + 2,
      head: [['Nama', 'Kelas', 'Jumlah Attempt', 'Skor Rata-rata', 'Status']],
      body: data.students.map((s) => [s.name, s.rombel, String(s.attempts), s.avgScore != null ? String(s.avgScore) : '-', s.status]),
      styles: { fontSize: 8 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
      didParseCell: (d) => {
        if (d.section === 'body' && d.column.index === 4) {
          if (d.cell.raw === 'Belum Mulai') { d.cell.styles.textColor = [120, 120, 120] }
          else if (d.cell.raw === 'Tidak Aktif') { d.cell.styles.textColor = [180, 120, 20]; d.cell.styles.fontStyle = 'bold' }
          else if (d.cell.raw === 'Aktif') { d.cell.styles.textColor = [16, 150, 110]; d.cell.styles.fontStyle = 'bold' }
        }
      },
    })
  }

  footer(doc, marginX)

  const safeName = data.schoolName.replace(/[^a-z0-9]+/gi, '-').replace(/^-+|-+$/g, '').toLowerCase()
  doc.save(`analytics-insights-${safeName || 'sekolah'}.pdf`)
}

// ─── Excel ────────────────────────────────────────────────────────────────────────────
export function exportSchoolAnalyticsToExcel(data: SchoolAnalyticsExportData) {
  const wb = XLSX.utils.book_new()

  const summaryRows: Record<string, string | number>[] = [
    { Field: 'Sekolah', Value: data.schoolName },
    { Field: 'Dibuat', Value: formatDateTime(data.generatedAt) },
    ...data.kpis.map((k) => ({ Field: k.label, Value: k.value })),
  ]
  XLSX.utils.book_append_sheet(wb, XLSX.utils.json_to_sheet(summaryRows), 'Ringkasan')

  if (data.insights.length > 0) {
    XLSX.utils.book_append_sheet(wb, XLSX.utils.json_to_sheet(data.insights.map((text) => ({ Insight: text }))), 'Insight')
  }

  if (data.kelasSummaries.length > 0) {
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(data.kelasSummaries.map((k) => ({ Kelas: k.rombel, 'Total Siswa': k.total, 'Siswa Aktif': k.aktif, 'Skor Rata-rata': k.avgScore }))),
      'Per Kelas',
    )
  }

  if (data.yearlyPerformance.length > 0) {
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(data.yearlyPerformance.map((yp) => ({ Tahun: yp.year, 'Jumlah Attempt': yp.attempts, 'Siswa Aktif': yp.distinctStudents, 'Skor Rata-rata': yp.avgScore }))),
      'Perbandingan Tahun',
    )
  }

  if (data.subjects.length > 0) {
    XLSX.utils.book_append_sheet(wb, XLSX.utils.json_to_sheet(data.subjects.map((s) => ({ 'Mata Uji': s.subject, 'Akurasi (%)': s.accuracy }))), 'Mata Uji')
  }

  if (data.students.length > 0) {
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(data.students.map((s) => ({ Nama: s.name, Kelas: s.rombel, 'Jumlah Attempt': s.attempts, 'Skor Rata-rata': s.avgScore, Status: s.status }))),
      'Daftar Siswa',
    )
  }

  const safeName = data.schoolName.replace(/[^a-z0-9]+/gi, '-').replace(/^-+|-+$/g, '').toLowerCase()
  XLSX.writeFile(wb, `analytics-insights-${safeName || 'sekolah'}.xlsx`)
}
