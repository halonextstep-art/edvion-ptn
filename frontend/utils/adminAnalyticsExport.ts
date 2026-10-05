// Ekspor "Analytics Platform" Admin ke PDF — dibangun dari data yang sama persis yang
// sudah dihitung & ditampilkan di AnalyticsPanel.vue (KPI, insight, akurasi mata uji,
// peringkat sekolah). Gaya & helper (logo, warna brand, kartu KPI) disalin dari pola yang
// sama di utils/schoolAnalyticsExport.ts / utils/reportExport.ts supaya semua dokumen PDF
// yang diterbitkan platform ini konsisten satu sama lain.
import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import * as XLSX from 'xlsx'
import { EDVION_WORDMARK_ASPECT, EDVION_WORDMARK_PNG_BASE64 } from '~/utils/edvionLogo'

const NAVY: [number, number, number] = [13, 19, 33]
const GREEN: [number, number, number] = [0, 214, 163]
const SLATE = 100

export interface AdminAnalyticsExportData {
  generatedAt: string
  kpis: { label: string; value: string }[]
  insights: string[]
  yearlyPerformance: { year: number; attempts: number; distinctStudents: number; avgScore: number }[]
  subjects: { subject: string; accuracy: number }[]
  topSchools: { name: string; students: number; avgScore: number; delta: number | null }[]
}

function formatDateTime(iso: string): string {
  return new Date(iso).toLocaleString('id-ID', { day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' })
}

function drawHeader(doc: jsPDF, data: AdminAnalyticsExportData, marginX: number): number {
  const logoH = 7
  const logoW = logoH * EDVION_WORDMARK_ASPECT
  doc.addImage(`data:image/png;base64,${EDVION_WORDMARK_PNG_BASE64}`, 'PNG', marginX, 10, logoW, logoH)

  let y = 26
  doc.setFontSize(14)
  doc.setFont('helvetica', 'bold')
  doc.setTextColor(NAVY[0], NAVY[1], NAVY[2])
  doc.text('Analytics Platform', marginX, y)
  y += 6

  doc.setFontSize(8.5)
  doc.setFont('helvetica', 'normal')
  doc.setTextColor(SLATE)
  doc.text(`Ringkasan seluruh platform · Dibuat ${formatDateTime(data.generatedAt)}`, marginX, y)
  y += 4

  doc.setDrawColor(GREEN[0], GREEN[1], GREEN[2])
  doc.setLineWidth(0.6)
  doc.line(marginX, y, doc.internal.pageSize.getWidth() - marginX, y)
  doc.setTextColor(0)
  return y + 8
}

function drawKpiCards(doc: jsPDF, marginX: number, y: number, cards: { label: string; value: string }[]): number {
  const pageW = doc.internal.pageSize.getWidth()
  const contentW = pageW - marginX * 2
  const gap = 4
  const w = (contentW - gap * (cards.length - 1)) / cards.length
  const h = 20
  cards.forEach((c, i) => {
    const x = marginX + i * (w + gap)
    doc.setFillColor(245, 246, 250)
    doc.setDrawColor(225, 227, 232)
    doc.setLineWidth(0.3)
    doc.roundedRect(x, y, w, h, 2, 2, 'FD')
    doc.setFont('helvetica', 'normal')
    doc.setFontSize(7)
    doc.setTextColor(SLATE)
    doc.text(c.label.toUpperCase(), x + w / 2, y + 6, { align: 'center', maxWidth: w - 4 })
    doc.setFont('helvetica', 'bold')
    doc.setFontSize(14)
    doc.setTextColor(NAVY[0], NAVY[1], NAVY[2])
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
    doc.text('Dibuat otomatis oleh Edvion dari data platform asli — bukan data simulasi/dikarang.', marginX, pageH - 9)
    doc.text(`Halaman ${p} dari ${pageCount}`, pageW - marginX, pageH - 9, { align: 'right' })
  }
}

export function exportAdminAnalyticsToPdf(data: AdminAnalyticsExportData) {
  const doc = new jsPDF()
  const marginX = 14
  let y = drawHeader(doc, data, marginX)

  y = drawKpiCards(doc, marginX, y, data.kpis)

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

  if (data.yearlyPerformance.length > 0) {
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
    y = sectionTitle(doc, 'Akurasi per Mata Uji', marginX, y)
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

  if (data.topSchools.length > 0) {
    if (y > 240) { doc.addPage(); y = 20 }
    y = sectionTitle(doc, `Peringkat Sekolah Mitra (${data.topSchools.length})`, marginX, y)
    autoTable(doc, {
      startY: y + 2,
      head: [['Sekolah', 'Siswa Aktif', 'Rata-rata Skor', 'vs Rata-rata Platform']],
      body: data.topSchools.map((s) => [s.name, String(s.students), String(s.avgScore), s.delta != null ? `${s.delta >= 0 ? '+' : ''}${s.delta}` : '-']),
      styles: { fontSize: 8 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
      didParseCell: (d) => {
        if (d.section === 'body' && d.column.index === 3) {
          const raw = String(d.cell.raw)
          if (raw.startsWith('+')) { d.cell.styles.textColor = [16, 150, 110]; d.cell.styles.fontStyle = 'bold' }
          else if (raw.startsWith('-')) { d.cell.styles.textColor = [190, 50, 50]; d.cell.styles.fontStyle = 'bold' }
        }
      },
    })
  }

  footer(doc, marginX)
  doc.save(`analytics-platform-${new Date().toISOString().slice(0, 10)}.pdf`)
}

// ─── Excel ────────────────────────────────────────────────────────────────────────────
export function exportAdminAnalyticsToExcel(data: AdminAnalyticsExportData) {
  const wb = XLSX.utils.book_new()

  const summaryRows: Record<string, string | number>[] = [
    { Field: 'Dibuat', Value: formatDateTime(data.generatedAt) },
    ...data.kpis.map((k) => ({ Field: k.label, Value: k.value })),
  ]
  XLSX.utils.book_append_sheet(wb, XLSX.utils.json_to_sheet(summaryRows), 'Ringkasan')

  if (data.insights.length > 0) {
    XLSX.utils.book_append_sheet(wb, XLSX.utils.json_to_sheet(data.insights.map((text) => ({ Insight: text }))), 'Insight')
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

  if (data.topSchools.length > 0) {
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet(data.topSchools.map((s) => ({ Sekolah: s.name, 'Siswa Aktif': s.students, 'Rata-rata Skor': s.avgScore, 'vs Rata-rata Platform': s.delta }))),
      'Sekolah',
    )
  }

  XLSX.writeFile(wb, `analytics-platform-${new Date().toISOString().slice(0, 10)}.xlsx`)
}
