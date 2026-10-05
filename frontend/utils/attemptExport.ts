// Export "Riwayat Pengerjaan TO" per-attempt detail ke PDF — dipakai dari School portal's
// Rekap SNBT drill-down (SchoolRekapSnbt.vue) supaya PIC sekolah bisa mencetak laporan per TO
// untuk satu siswa: skor, rincian per mata uji, dan ringkasan kekuatan/kekurangan yang
// diturunkan langsung dari data breakdown asli (tidak ada angka dikarang) — pola dan gaya
// sama persis dengan utils/reportExport.ts supaya konsisten dengan laporan lain di aplikasi.
import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import type { Attempt, ScoreDisplayMode, SubjectBreakdown } from '~/types'
import { EDVION_WORDMARK_ASPECT, EDVION_WORDMARK_PNG_BASE64 } from '~/utils/edvionLogo'

// Warna brand resmi Edvion — sama dengan utils/simulationCertificate.ts & reportExport.ts.
const NAVY: [number, number, number] = [13, 19, 33]
const GREEN: [number, number, number] = [0, 214, 163]

const SESSION_TYPE_LABEL: Record<string, string> = { tryout: 'Tryout', drilling: 'Drilling', mini: 'Mini Tryout' }

function formatDateTime(iso: string): string {
  return new Date(iso).toLocaleString('id-ID', { day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' })
}

/** Kategorisasi murni dari rasio benar/total — tidak ada ambang batas tersembunyi selain yang
 *  ditulis di sini, dan tidak pernah menampilkan kategori untuk mata uji yang total soalnya 0. */
function subjectAccuracyPercent(b: SubjectBreakdown): number | null {
  if (b.total <= 0) return null
  return Math.round((b.correct / b.total) * 100)
}

export function exportAttemptResultToPdf(
  studentName: string,
  attempt: Attempt,
  subjectBreakdown: SubjectBreakdown[],
  /** "Estimasi IRT" — see backend domain::irt doc comment. Optional so existing callers keep
   *  working unchanged; only printed when the admin's score_display_mode says to. */
  irt?: { irtScore: number | null; scoreDisplayMode: ScoreDisplayMode },
) {
  const doc = new jsPDF()
  const marginX = 14
  const logoH = 7
  doc.addImage(`data:image/png;base64,${EDVION_WORDMARK_PNG_BASE64}`, 'PNG', marginX, 10, logoH * EDVION_WORDMARK_ASPECT, logoH)
  let y = 26

  doc.setFontSize(15)
  doc.setFont('helvetica', 'bold')
  doc.setTextColor(NAVY[0], NAVY[1], NAVY[2])
  doc.text('Laporan Hasil Tryout', marginX, y)
  doc.setTextColor(0)
  y += 4
  doc.setDrawColor(GREEN[0], GREEN[1], GREEN[2])
  doc.setLineWidth(0.5)
  doc.line(marginX, y, doc.internal.pageSize.getWidth() - marginX, y)
  y += 6

  doc.setFontSize(10)
  doc.setFont('helvetica', 'normal')
  doc.setTextColor(90)
  doc.text(`${studentName}`, marginX, y)
  y += 5
  doc.text(`${attempt.session_title} · ${SESSION_TYPE_LABEL[attempt.session_type] || attempt.session_type}`, marginX, y)
  y += 5
  doc.text(`Dikerjakan: ${attempt.submitted_at ? formatDateTime(attempt.submitted_at) : '-'}`, marginX, y)
  doc.setTextColor(0)
  y += 9

  // Ringkasan skor
  doc.setFontSize(11)
  doc.setFont('helvetica', 'bold')
  doc.text(`Skor: ${attempt.score ?? '-'}`, marginX, y)
  doc.text(`Akurasi: ${attempt.accuracy != null ? Math.round(attempt.accuracy) + '%' : '-'}`, marginX + 60, y)
  doc.text(`Benar/Salah/Kosong: ${attempt.correct_count ?? 0}/${attempt.wrong_count ?? 0}/${attempt.unanswered_count ?? 0}`, marginX + 120, y)
  y += 7

  // Estimasi IRT — hanya dicetak kalau admin mengaktifkan mode 'irt'/'both'. Lihat backend
  // domain::irt doc comment: estimasi kalibrasi platform sendiri dari data historis platform
  // ini, BUKAN replikasi skor UTBK resmi SNPMB.
  if (irt && irt.scoreDisplayMode !== 'instant') {
    doc.setFont('helvetica', 'normal')
    doc.setFontSize(9)
    doc.setTextColor(NAVY[0], NAVY[1], NAVY[2])
    doc.text(`Estimasi IRT: ${irt.irtScore != null ? Math.round(irt.irtScore) : 'Belum tersedia'}`, marginX, y)
    doc.setTextColor(0)
    y += 6
  }
  y += 2

  // Rincian per mata uji
  doc.setFontSize(11)
  doc.setFont('helvetica', 'bold')
  doc.text('Rincian per Mata Uji', marginX, y)
  autoTable(doc, {
    startY: y + 3,
    head: [['Mata Uji', 'Benar', 'Total', 'Akurasi']],
    body: subjectBreakdown.length
      ? subjectBreakdown.map((b) => [b.subject, String(b.correct), String(b.total), subjectAccuracyPercent(b) != null ? `${subjectAccuracyPercent(b)}%` : '-'])
      : [['Tidak ada rincian mata uji untuk attempt ini.', '', '', '']],
    styles: { fontSize: 9 },
    headStyles: { fillColor: NAVY },
    margin: { left: marginX, right: marginX },
  })
  y = (doc as any).lastAutoTable.finalY + 9

  // Analisis kekuatan/kekurangan — murni derivasi dari subjectBreakdown, diurutkan akurasi
  // naik supaya mata uji terlemah tampil duluan; tidak menampilkan analisis sama sekali kalau
  // tidak ada data mata uji apa pun (honest-zero, bukan dikira-kira).
  const withAccuracy = subjectBreakdown
    .map((b) => ({ ...b, acc: subjectAccuracyPercent(b) }))
    .filter((b) => b.acc != null) as (SubjectBreakdown & { acc: number })[]
  if (withAccuracy.length > 0) {
    const sorted = [...withAccuracy].sort((a, b) => a.acc - b.acc)
    const weak = sorted.filter((b) => b.acc < 60)
    const strong = sorted.filter((b) => b.acc >= 80)

    doc.setFontSize(11)
    doc.setFont('helvetica', 'bold')
    doc.text('Analisis Kekuatan & Kekurangan', marginX, y)
    y += 6
    doc.setFontSize(9)
    doc.setFont('helvetica', 'normal')

    if (weak.length > 0) {
      doc.setTextColor(180, 40, 40)
      doc.text(`Perlu perhatian (akurasi < 60%): ${weak.map((b) => `${b.subject} (${b.acc}%)`).join(', ')}`, marginX, y, { maxWidth: 182 })
      y += Math.ceil(doc.getTextDimensions(weak.map((b) => `${b.subject} (${b.acc}%)`).join(', '), { maxWidth: 182 }).h) + 4
      doc.setTextColor(0)
    } else {
      doc.text('Tidak ada mata uji dengan akurasi di bawah 60% — tidak ada area yang butuh perhatian khusus.', marginX, y, { maxWidth: 182 })
      y += 6
    }
    if (strong.length > 0) {
      doc.setTextColor(20, 130, 80)
      doc.text(`Mata uji terkuat (akurasi >= 80%): ${strong.map((b) => `${b.subject} (${b.acc}%)`).join(', ')}`, marginX, y, { maxWidth: 182 })
      doc.setTextColor(0)
    }
  }

  doc.setFontSize(7)
  doc.setTextColor(150)
  doc.text('Dibuat otomatis oleh Edvion dari data pengerjaan asli — bukan estimasi.', marginX, 290)

  const safeName = studentName.replace(/[^a-z0-9]+/gi, '-').replace(/^-+|-+$/g, '').toLowerCase()
  const safeTitle = attempt.session_title.replace(/[^a-z0-9]+/gi, '-').replace(/^-+|-+$/g, '').toLowerCase()
  doc.save(`hasil-to-${safeName}-${safeTitle}.pdf`)
}
