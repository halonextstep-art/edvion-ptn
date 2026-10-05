// Ekspor "Journey to PTN" (Progress siswa) ke PDF — dibangun dari data yang sama persis
// yang sudah dihitung & ditampilkan di StudentProgress.vue (KPI, penguasaan mata uji,
// insight, rekap jawaban). Gaya & helper disalin dari pola yang sama di
// utils/schoolAnalyticsExport.ts / utils/adminAnalyticsExport.ts supaya semua dokumen PDF
// yang diterbitkan platform ini konsisten satu sama lain.
import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import * as XLSX from 'xlsx'
import { EDVION_WORDMARK_ASPECT, EDVION_WORDMARK_PNG_BASE64 } from '~/utils/edvionLogo'

const NAVY: [number, number, number] = [13, 19, 33]
const GREEN: [number, number, number] = [0, 214, 163]
const SLATE = 100

export interface StudentProgressExportData {
  studentName: string
  generatedAt: string
  kpis: { label: string; value: string }[]
  subjects: { subject: string; accuracy: number }[]
  insights: string[]
  totals: { correct: number; wrong: number; unanswered: number }
}

function formatDateTime(iso: string): string {
  return new Date(iso).toLocaleString('id-ID', { day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' })
}

function drawHeader(doc: jsPDF, data: StudentProgressExportData, marginX: number): number {
  const logoH = 7
  const logoW = logoH * EDVION_WORDMARK_ASPECT
  doc.addImage(`data:image/png;base64,${EDVION_WORDMARK_PNG_BASE64}`, 'PNG', marginX, 10, logoW, logoH)

  let y = 26
  doc.setFontSize(14)
  doc.setFont('helvetica', 'bold')
  doc.setTextColor(NAVY[0], NAVY[1], NAVY[2])
  doc.text('Journey to PTN — Progress Belajar', marginX, y)
  y += 6

  doc.setFontSize(8.5)
  doc.setFont('helvetica', 'normal')
  doc.setTextColor(SLATE)
  doc.text(`${data.studentName} · Dibuat ${formatDateTime(data.generatedAt)}`, marginX, y)
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
    doc.setFillColor(245, 246, 250)
    doc.setDrawColor(225, 227, 232)
    doc.setLineWidth(0.3)
    doc.roundedRect(x, cy, w, h, 2, 2, 'FD')
    doc.setFont('helvetica', 'normal')
    doc.setFontSize(7)
    doc.setTextColor(SLATE)
    doc.text(c.label.toUpperCase(), x + w / 2, cy + 6, { align: 'center', maxWidth: w - 4 })
    doc.setFont('helvetica', 'bold')
    doc.setFontSize(14)
    doc.setTextColor(NAVY[0], NAVY[1], NAVY[2])
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
    doc.text('Dibuat otomatis oleh Edvion dari data latihan asli — bukan data simulasi/dikarang.', marginX, pageH - 9)
    doc.text(`Halaman ${p} dari ${pageCount}`, pageW - marginX, pageH - 9, { align: 'right' })
  }
}

export function exportStudentProgressToPdf(data: StudentProgressExportData) {
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

  if (data.subjects.length > 0) {
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

  const totalAnswered = data.totals.correct + data.totals.wrong + data.totals.unanswered
  if (totalAnswered > 0) {
    y = sectionTitle(doc, 'Rekap Jawaban (akumulasi semua sesi)', marginX, y)
    autoTable(doc, {
      startY: y + 2,
      head: [['Benar', 'Salah', 'Tidak Dijawab']],
      body: [[String(data.totals.correct), String(data.totals.wrong), String(data.totals.unanswered)]],
      styles: { fontSize: 9 },
      headStyles: { fillColor: NAVY },
      margin: { left: marginX, right: marginX },
    })
  }

  footer(doc, marginX)

  const safeName = data.studentName.replace(/[^a-z0-9]+/gi, '-').replace(/^-+|-+$/g, '').toLowerCase()
  doc.save(`progress-${safeName || 'siswa'}.pdf`)
}

// ─── Excel ────────────────────────────────────────────────────────────────────────────
export function exportStudentProgressToExcel(data: StudentProgressExportData) {
  const wb = XLSX.utils.book_new()

  const summaryRows: Record<string, string | number>[] = [
    { Field: 'Siswa', Value: data.studentName },
    { Field: 'Dibuat', Value: formatDateTime(data.generatedAt) },
    ...data.kpis.map((k) => ({ Field: k.label, Value: k.value })),
  ]
  XLSX.utils.book_append_sheet(wb, XLSX.utils.json_to_sheet(summaryRows), 'Ringkasan')

  if (data.insights.length > 0) {
    XLSX.utils.book_append_sheet(wb, XLSX.utils.json_to_sheet(data.insights.map((text) => ({ Insight: text }))), 'Insight')
  }

  if (data.subjects.length > 0) {
    XLSX.utils.book_append_sheet(wb, XLSX.utils.json_to_sheet(data.subjects.map((s) => ({ 'Mata Uji': s.subject, 'Akurasi (%)': s.accuracy }))), 'Mata Uji')
  }

  const totalAnswered = data.totals.correct + data.totals.wrong + data.totals.unanswered
  if (totalAnswered > 0) {
    XLSX.utils.book_append_sheet(
      wb,
      XLSX.utils.json_to_sheet([{ Benar: data.totals.correct, Salah: data.totals.wrong, 'Tidak Dijawab': data.totals.unanswered }]),
      'Rekap Jawaban',
    )
  }

  const safeName = data.studentName.replace(/[^a-z0-9]+/gi, '-').replace(/^-+|-+$/g, '').toLowerCase()
  XLSX.writeFile(wb, `progress-${safeName || 'siswa'}.xlsx`)
}
