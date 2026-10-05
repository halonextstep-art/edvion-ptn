// Sertifikat + Laporan Hasil untuk Simulasi TO (UTBK/TKA) — dibuat begitu siswa menyelesaikan
// sebuah run, dan bisa diunduh ulang kapan saja dari menu "Sertifikat". Dua halaman dalam satu
// PDF: halaman 1 sertifikat bergaya diploma (landscape) dengan scorecard visual per mata uji,
// halaman 2+ rincian skor per subtes + grafik perbandingan + analisis kekuatan/kelemahan
// (portrait) — pola tabel & analisis mengikuti utils/attemptExport.ts supaya konsisten dengan
// laporan TO tunggal yang sudah ada. Semua angka diturunkan langsung dari data run/slot asli —
// tidak ada skor atau kategori yang dikarang; lihat komentar tkaScaleFor/tkaScaledScore/
// tkaCategoryLabel di pages/simulasi/[runId].vue untuk rasionale skala TKA yang direplikasi
// persis di sini supaya kedua tempat konsisten.
import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import type { SchoolType, SimulationRunItem, TemplateStatsItem } from '~/types'
import { EDVION_FULL_LOCKUP_ASPECT, EDVION_FULL_LOCKUP_PNG_BASE64, EDVION_WORDMARK_ASPECT, EDVION_WORDMARK_PNG_BASE64 } from '~/utils/edvionLogo'

// Warna brand resmi Edvion — diambil langsung dari file logo (bukan indigo generik seperti
// sebelumnya), supaya sertifikat & laporan konsisten dengan identitas visual asli.
const NAVY = { r: 13, g: 19, b: 33 } // teks utama & elemen gelap logo
const GREEN = { r: 0, g: 214, b: 163 } // aksen mark logo — dipakai utk kategori "Istimewa"
const SLATE = { r: 100, g: 110, b: 130 } // teks sekunder netral

function logoDataUrl(base64: string): string {
  return `data:image/png;base64,${base64}`
}

/** Menggambar wordmark logo Edvion terpusat secara horizontal pada `cy` (mm, dari atas),
 *  dengan tinggi `heightMm`. Mengembalikan tinggi yang dipakai supaya caller bisa lanjut
 *  menata elemen berikutnya secara presisi. */
function drawCenteredLogo(doc: jsPDF, cx: number, topY: number, heightMm: number, variant: 'wordmark' | 'full' = 'wordmark'): number {
  const aspect = variant === 'wordmark' ? EDVION_WORDMARK_ASPECT : EDVION_FULL_LOCKUP_ASPECT
  const base64 = variant === 'wordmark' ? EDVION_WORDMARK_PNG_BASE64 : EDVION_FULL_LOCKUP_PNG_BASE64
  const w = heightMm * aspect
  doc.addImage(logoDataUrl(base64), 'PNG', cx - w / 2, topY, w, heightMm)
  return heightMm
}

function formatDateLong(iso: string): string {
  return new Date(iso).toLocaleDateString('id-ID', { day: 'numeric', month: 'long', year: 'numeric' })
}

// Replikasi persis dari pages/simulasi/[runId].vue — lihat komentar di sana untuk rasionale
// (rescaling 0-1000 platform -> skala jenjang TKA — 0-100 SD/SMP ambang Istimewa 95, 200-800
// SMA/SMK/MA ambang Istimewa 725, per Perka BSKAP No. 047/2025 & No. 045/2025; kategori lain
// tidak dikarang karena tidak ada data standard-setting resmi). Jenjang diambil dari
// `run.school_type_scope` (denormalized dari template — lihat backend
// `domain::simulation::SimulationRun::school_type_scope` doc comment).
function tkaScaleFor(scope: SchoolType | null | undefined): { base: number; span: number; istimewaThreshold: number; label: string } {
  if (scope === 'smp') return { base: 0, span: 100, istimewaThreshold: 95, label: '0–100' }
  return { base: 200, span: 600, istimewaThreshold: 725, label: '200–800' }
}
function tkaScaledScore(rawScore: number | null, scope: SchoolType | null | undefined): number | null {
  if (rawScore == null) return null
  const scale = tkaScaleFor(scope)
  return Math.round(scale.base + (rawScore / 1000) * scale.span)
}
function tkaCategoryLabel(rawScore: number | null, scope: SchoolType | null | undefined): string | null {
  const scaled = tkaScaledScore(rawScore, scope)
  if (scaled == null) return null
  return scaled >= tkaScaleFor(scope).istimewaThreshold ? 'Istimewa' : 'Di bawah Istimewa'
}

function safeSlug(s: string): string {
  return s.replace(/[^a-z0-9]+/gi, '-').replace(/^-+|-+$/g, '').toLowerCase()
}

/** `stats` boleh `null` (mis. gagal dimuat) — bagian pembanding cukup dilewati, bukan diisi 0. */
export function exportSimulationCertificate(studentName: string, schoolName: string | null, run: SimulationRunItem, stats: TemplateStatsItem | null) {
  if (run.status !== 'completed') return // tidak ada sertifikat untuk run yang belum selesai
  const isPerSubject = run.template_kind === 'per_subject'
  const scale = tkaScaleFor(run.school_type_scope)

  // ── Halaman 1 — Sertifikat (landscape, bergaya diploma) ──────────────────────────────
  const doc = new jsPDF({ orientation: 'landscape', unit: 'mm', format: 'a4' })
  const pageW = doc.internal.pageSize.getWidth()
  const pageH = doc.internal.pageSize.getHeight()
  const cx = pageW / 2

  // Bingkai ganda + strip aksen hijau tipis di tepi atas — sentuhan identitas brand tanpa
  // mengganggu keterbacaan.
  doc.setFillColor(GREEN.r, GREEN.g, GREEN.b)
  doc.rect(0, 0, pageW, 2.2, 'F')
  doc.setDrawColor(NAVY.r, NAVY.g, NAVY.b)
  doc.setLineWidth(1)
  doc.roundedRect(10, 10, pageW - 20, pageH - 20, 4, 4, 'S')
  doc.setDrawColor(GREEN.r, GREEN.g, GREEN.b)
  doc.setLineWidth(0.4)
  doc.roundedRect(14, 14, pageW - 28, pageH - 28, 3, 3, 'S')

  drawCenteredLogo(doc, cx, 20, 11, 'wordmark')
  doc.setFont('helvetica', 'normal')
  doc.setFontSize(8)
  doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
  doc.text('Digital Vision for Modern Schools · Platform Persiapan SNBT & TKA', cx, 35, { align: 'center' })

  doc.setFont('times', 'bold')
  doc.setFontSize(24)
  doc.setTextColor(NAVY.r, NAVY.g, NAVY.b)
  doc.text('SERTIFIKAT PENYELESAIAN SIMULASI', cx, 48, { align: 'center' })
  doc.setFont('times', 'italic')
  doc.setFontSize(11.5)
  doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
  doc.text(isPerSubject ? 'Tes Kemampuan Akademik (TKA)' : 'Simulasi Try Out UTBK', cx, 55, { align: 'center' })

  doc.setFont('helvetica', 'normal')
  doc.setFontSize(10.5)
  doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
  doc.text('Diberikan kepada:', cx, 66, { align: 'center' })

  doc.setFont('times', 'bolditalic')
  doc.setFontSize(24)
  doc.setTextColor(NAVY.r, NAVY.g, NAVY.b)
  doc.text(studentName, cx, 78, { align: 'center' })
  if (schoolName) {
    doc.setFont('helvetica', 'normal')
    doc.setFontSize(9.5)
    doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
    doc.text(schoolName, cx, 84, { align: 'center' })
  }

  doc.setFont('helvetica', 'normal')
  doc.setFontSize(10)
  doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
  doc.text('atas keberhasilan menyelesaikan', cx, 93, { align: 'center' })
  doc.setFont('helvetica', 'bold')
  doc.setFontSize(12)
  doc.setTextColor(NAVY.r, NAVY.g, NAVY.b)
  doc.text(run.template_title, cx, 100, { align: 'center' })

  // ── Hasil ──────────────────────────────────────────────────────────────────────────
  // UTBK: satu skor gabungan (angka nyata, bukan yang dikarang).
  // TKA: TIDAK PERNAH digabung jadi satu angka (lihat backend application::simulation_service
  // doc comment) — sebelumnya halaman ini hanya menampilkan "X dari Y Mata Uji Meraih
  // Istimewa", yang jadi tidak informatif (bahkan membingungkan) saat X = 0, karena pembaca
  // tidak tahu skor sebenarnya, cuma tahu "gagal Istimewa". Sekarang setiap mata uji
  // ditampilkan sebagai kartu skor sendiri-sendiri (skor asli + kategori berwarna), supaya
  // hasil selalu jelas terlepas dari berapa banyak yang meraih Istimewa.
  const cardsTop = 107
  if (isPerSubject) {
    const scored = run.slots.filter((s) => s.score != null)
    const istimewaCount = scored.filter((s) => tkaCategoryLabel(s.score, run.school_type_scope) === 'Istimewa').length

    doc.setFont('helvetica', 'bold')
    doc.setFontSize(8.5)
    doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
    doc.text(`HASIL PER MATA UJI  ·  SKALA ${scale.label}  ·  ISTIMEWA >= ${scale.istimewaThreshold}`, cx, cardsTop, { align: 'center' })

    const n = Math.max(run.slots.length, 1)
    const perRow = n <= 5 ? n : Math.ceil(n / 2)
    const gap = 4
    const areaW = pageW - 44
    const cardW = Math.min(46, (areaW - gap * (perRow - 1)) / perRow)
    const rowW = cardW * perRow + gap * (perRow - 1)
    const startX = cx - rowW / 2
    const cardH = 26
    const rows = Math.ceil(n / perRow)

    run.slots.forEach((s, i) => {
      const row = Math.floor(i / perRow)
      const col = i % perRow
      const x = startX + col * (cardW + gap)
      const y = cardsTop + 4 + row * (cardH + 3)
      const label = tkaCategoryLabel(s.score, run.school_type_scope)
      const isIstimewa = label === 'Istimewa'
      const scored_ = s.score != null

      if (isIstimewa) {
        doc.setFillColor(230, 253, 244)
        doc.setDrawColor(GREEN.r, GREEN.g, GREEN.b)
      } else if (scored_) {
        doc.setFillColor(245, 246, 250)
        doc.setDrawColor(210, 214, 224)
      } else {
        doc.setFillColor(250, 250, 251)
        doc.setDrawColor(225, 227, 232)
      }
      doc.setLineWidth(0.4)
      doc.roundedRect(x, y, cardW, cardH, 2, 2, 'FD')

      doc.setFont('helvetica', 'bold')
      doc.setFontSize(6.6)
      doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
      const titleLines = doc.splitTextToSize(s.session_title, cardW - 4)
      doc.text(titleLines.slice(0, 2), x + cardW / 2, y + 5, { align: 'center' })

      doc.setFont('helvetica', 'bold')
      doc.setFontSize(15)
      doc.setTextColor(isIstimewa ? GREEN.r : NAVY.r, isIstimewa ? GREEN.g : NAVY.g, isIstimewa ? GREEN.b : NAVY.b)
      doc.text(scored_ ? String(tkaScaledScore(s.score, run.school_type_scope)) : '—', x + cardW / 2, y + 17, { align: 'center' })

      doc.setFont('helvetica', 'bold')
      doc.setFontSize(6.2)
      doc.setTextColor(isIstimewa ? GREEN.r : 150, isIstimewa ? GREEN.g : 150, isIstimewa ? GREEN.b : 150)
      doc.text(scored_ ? (label ?? '-') : 'Belum dinilai', x + cardW / 2, y + 22.5, { align: 'center' })
    })

    const cardsBottom = cardsTop + 4 + rows * (cardH + 3)
    doc.setFont('helvetica', 'normal')
    doc.setFontSize(8.5)
    doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
    doc.text(`${istimewaCount} dari ${scored.length || run.slots.length} mata uji yang sudah dinilai meraih kategori Istimewa.`, cx, cardsBottom + 2, { align: 'center' })
  } else if (run.combined_estimate_score != null) {
    doc.setFillColor(245, 246, 255)
    doc.setDrawColor(GREEN.r, GREEN.g, GREEN.b)
    doc.setLineWidth(0.4)
    doc.roundedRect(cx - 55, cardsTop, 110, 24, 3, 3, 'FD')
    doc.setFont('helvetica', 'bold')
    doc.setFontSize(9)
    doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
    doc.text('ESTIMASI SKOR UTBK GABUNGAN', cx, cardsTop + 7, { align: 'center' })
    doc.setFontSize(24)
    doc.setTextColor(NAVY.r, NAVY.g, NAVY.b)
    doc.text(String(Math.round(run.combined_estimate_score)), cx, cardsTop + 18, { align: 'center' })

    // Gauge visual: posisi skor pada rentang 0–1000, dengan garis marker.
    const gaugeY = cardsTop + 30
    const gaugeW = 140
    const gaugeX = cx - gaugeW / 2
    doc.setFillColor(235, 236, 242)
    doc.roundedRect(gaugeX, gaugeY, gaugeW, 2.6, 1.3, 1.3, 'F')
    const frac = Math.max(0, Math.min(1, run.combined_estimate_score / 1000))
    doc.setFillColor(GREEN.r, GREEN.g, GREEN.b)
    doc.roundedRect(gaugeX, gaugeY, gaugeW * frac, 2.6, 1.3, 1.3, 'F')
    doc.setFont('helvetica', 'normal')
    doc.setFontSize(6.5)
    doc.setTextColor(180)
    doc.text('0', gaugeX, gaugeY + 6.5)
    doc.text('1000', gaugeX + gaugeW, gaugeY + 6.5, { align: 'right' })
  } else {
    doc.setFillColor(245, 246, 250)
    doc.roundedRect(cx - 55, cardsTop, 110, 20, 3, 3, 'F')
    doc.setFont('helvetica', 'normal')
    doc.setFontSize(10.5)
    doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
    doc.text('Skor belum dapat dihitung untuk simulasi ini', cx, cardsTop + 12, { align: 'center' })
  }

  doc.setFont('helvetica', 'normal')
  doc.setFontSize(9)
  doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
  doc.text(`Diselesaikan pada ${run.completed_at ? formatDateLong(run.completed_at) : '-'}`, cx, pageH - 27, { align: 'center' })

  doc.setFontSize(7)
  doc.setTextColor(160)
  doc.text(
    'Sertifikat ini diterbitkan otomatis oleh sistem Edvion sebagai bukti penyelesaian simulasi internal.',
    cx, pageH - 20, { align: 'center' },
  )
  doc.text('Bukan dokumen resmi SNPMB/Kemendikdasmen — skor bersifat estimasi dari data pengerjaan asli platform ini.', cx, pageH - 16.5, { align: 'center' })
  doc.setFont('courier', 'normal')
  doc.setFontSize(6.5)
  doc.text(`ID Verifikasi: ${run.id}`, cx, pageH - 12, { align: 'center' })

  // ── Halaman 2 — Laporan Hasil & Analisis Kemampuan (portrait) ────────────────────────
  doc.addPage('a4', 'portrait')
  const marginX = 14
  const contentW = doc.internal.pageSize.getWidth() - marginX * 2
  let y = 16

  drawCenteredLogo(doc, marginX + 16, y, 6.5, 'wordmark')
  doc.setFont('helvetica', 'bold')
  doc.setFontSize(14)
  doc.setTextColor(NAVY.r, NAVY.g, NAVY.b)
  doc.text('Laporan Hasil & Analisis Kemampuan', marginX + 36, y + 5)
  y += 15

  doc.setDrawColor(230)
  doc.setLineWidth(0.3)
  doc.line(marginX, y, marginX + contentW, y)
  y += 6

  doc.setFont('helvetica', 'normal')
  doc.setFontSize(10)
  doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
  doc.text(studentName + (schoolName ? ` · ${schoolName}` : ''), marginX, y)
  y += 5
  doc.text(run.template_title, marginX, y)
  y += 5
  doc.text(`Diselesaikan: ${run.completed_at ? formatDateLong(run.completed_at) : '-'}`, marginX, y)
  y += 9

  // Ringkasan skor + pembanding jujur (kalau ada data)
  doc.setFont('helvetica', 'bold')
  doc.setFontSize(11)
  doc.setTextColor(NAVY.r, NAVY.g, NAVY.b)
  if (!isPerSubject && run.combined_estimate_score != null) {
    doc.text(`Estimasi Skor UTBK Gabungan: ${Math.round(run.combined_estimate_score)}`, marginX, y)
    y += 6
  } else if (isPerSubject) {
    const scored = run.slots.filter((s) => s.score != null)
    const istimewaCount = scored.filter((s) => tkaCategoryLabel(s.score, run.school_type_scope) === 'Istimewa').length
    doc.text(`Ringkasan: ${istimewaCount} dari ${scored.length} mata uji meraih kategori Istimewa (skala ${scale.label}, ambang >= ${scale.istimewaThreshold}).`, marginX, y, { maxWidth: contentW })
    y += 6
  }
  doc.setFont('helvetica', 'normal')
  doc.setFontSize(9)
  doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
  if (stats && stats.participant_count > 0) {
    if (!isPerSubject && stats.avg_combined_score != null) {
      doc.text(
        `Rata-rata skor gabungan peserta lain yang menyelesaikan simulasi ini: ${Math.round(stats.avg_combined_score)} (dari ${stats.participant_count} peserta).`,
        marginX, y, { maxWidth: contentW },
      )
      y += 6
    } else {
      doc.text(`Simulasi ini sudah diselesaikan oleh ${stats.participant_count} peserta lain di platform ini.`, marginX, y)
      y += 6
    }
  }
  y += 3

  // Rincian per subtes — kategori Istimewa disorot warna hijau langsung di tabel.
  doc.setFont('helvetica', 'bold')
  doc.setFontSize(11)
  doc.setTextColor(NAVY.r, NAVY.g, NAVY.b)
  doc.text('Rincian per Subtes', marginX, y)
  const categoryColIndex = isPerSubject ? 2 : -1
  autoTable(doc, {
    startY: y + 3,
    head: isPerSubject
      ? [[`Mata Uji`, `Skor (${scale.label})`, 'Kategori', 'Benar', 'Salah', 'Kosong', 'Akurasi']]
      : [['Subtes', 'Skor', 'Benar', 'Salah', 'Kosong', 'Akurasi']],
    body: run.slots.map((s) => {
      const acc = s.accuracy != null ? `${Math.round(s.accuracy)}%` : '-'
      const base = [
        s.session_title,
        ...(isPerSubject ? [s.score != null ? String(tkaScaledScore(s.score, run.school_type_scope)) : '-', tkaCategoryLabel(s.score, run.school_type_scope) ?? '-'] : [s.score != null ? String(s.score) : '-']),
        s.correct_count != null ? String(s.correct_count) : '-',
        s.wrong_count != null ? String(s.wrong_count) : '-',
        s.unanswered_count != null ? String(s.unanswered_count) : '-',
        acc,
      ]
      return base
    }),
    styles: { fontSize: 9 },
    headStyles: { fillColor: [NAVY.r, NAVY.g, NAVY.b] },
    margin: { left: marginX, right: marginX },
    didParseCell: (data) => {
      if (data.section === 'body' && categoryColIndex >= 0 && data.column.index === categoryColIndex) {
        if (data.cell.raw === 'Istimewa') {
          data.cell.styles.textColor = [16, 150, 110]
          data.cell.styles.fontStyle = 'bold'
        }
      }
    },
  })
  y = (doc as any).lastAutoTable.finalY + 9

  // Grafik perbandingan skor — visualisasi bar horizontal sederhana per mata uji/subtes,
  // supaya perbedaan hasil antar subtes langsung terlihat tanpa harus membaca angka satu-satu.
  const scoredSlots = run.slots.filter((s) => s.score != null)
  if (scoredSlots.length > 0) {
    if (y > 245) {
      doc.addPage('a4', 'portrait')
      y = 18
    }
    doc.setFont('helvetica', 'bold')
    doc.setFontSize(11)
    doc.setTextColor(NAVY.r, NAVY.g, NAVY.b)
    doc.text('Perbandingan Skor per ' + (isPerSubject ? 'Mata Uji' : 'Subtes'), marginX, y)
    y += 7

    const maxVal = isPerSubject ? scale.base + scale.span : 1000
    const labelW = 52
    const valueW = 16
    const barAreaW = contentW - labelW - valueW - 4
    const barH = 4.2
    const rowGap = 2.4

    scoredSlots.forEach((s) => {
      if (y > 280) {
        doc.addPage('a4', 'portrait')
        y = 18
      }
      const display = isPerSubject ? tkaScaledScore(s.score, run.school_type_scope)! : (s.score as number)
      const frac = Math.max(0, Math.min(1, (display - (isPerSubject ? scale.base : 0)) / (maxVal - (isPerSubject ? scale.base : 0))))
      const isIstimewa = isPerSubject && tkaCategoryLabel(s.score, run.school_type_scope) === 'Istimewa'

      doc.setFont('helvetica', 'normal')
      doc.setFontSize(7.8)
      doc.setTextColor(70)
      const lbl = doc.splitTextToSize(s.session_title, labelW - 2)[0]
      doc.text(lbl, marginX, y + barH - 0.6)

      doc.setFillColor(238, 239, 244)
      doc.roundedRect(marginX + labelW, y, barAreaW, barH, 1, 1, 'F')
      doc.setFillColor(isIstimewa ? GREEN.r : NAVY.r, isIstimewa ? GREEN.g : NAVY.g, isIstimewa ? GREEN.b : NAVY.b)
      doc.roundedRect(marginX + labelW, y, Math.max(barAreaW * frac, 2), barH, 1, 1, 'F')

      doc.setFont('helvetica', 'bold')
      doc.setFontSize(7.8)
      doc.setTextColor(30)
      doc.text(String(display), marginX + labelW + barAreaW + 3, y + barH - 0.6)

      y += barH + rowGap
    })
    y += 4
  }

  // Analisis kekuatan/kelemahan — murni derivasi dari accuracy asli tiap subtes, sama persis
  // polanya dengan utils/attemptExport.ts. Tidak ditampilkan sama sekali kalau tidak ada satu
  // pun subtes dengan akurasi diketahui (honest-zero, bukan dikira-kira).
  const withAccuracy = run.slots
    .filter((s) => s.accuracy != null)
    .map((s) => ({ label: s.session_title, acc: Math.round(s.accuracy as number) }))
  if (withAccuracy.length > 0) {
    if (y > 260) {
      doc.addPage('a4', 'portrait')
      y = 18
    }
    const sorted = [...withAccuracy].sort((a, b) => a.acc - b.acc)
    const weak = sorted.filter((s) => s.acc < 60)
    const strong = sorted.filter((s) => s.acc >= 80)

    doc.setFont('helvetica', 'bold')
    doc.setFontSize(11)
    doc.setTextColor(NAVY.r, NAVY.g, NAVY.b)
    doc.text('Analisis Kekuatan & Kelemahan', marginX, y)
    y += 6
    doc.setFont('helvetica', 'normal')
    doc.setFontSize(9)

    if (weak.length > 0) {
      doc.setTextColor(190, 50, 50)
      const line = `Perlu perhatian (akurasi < 60%): ${weak.map((s) => `${s.label} (${s.acc}%)`).join(', ')}`
      doc.text(line, marginX, y, { maxWidth: contentW })
      y += Math.ceil(doc.getTextDimensions(line, { maxWidth: contentW }).h) + 4
    } else {
      doc.setTextColor(SLATE.r, SLATE.g, SLATE.b)
      doc.text('Tidak ada subtes dengan akurasi di bawah 60% — tidak ada area yang butuh perhatian khusus.', marginX, y, { maxWidth: contentW })
      y += 6
    }
    if (strong.length > 0) {
      doc.setTextColor(16, 150, 110)
      const line = `Subtes terkuat (akurasi >= 80%): ${strong.map((s) => `${s.label} (${s.acc}%)`).join(', ')}`
      doc.text(line, marginX, y, { maxWidth: contentW })
      y += Math.ceil(doc.getTextDimensions(line, { maxWidth: contentW }).h) + 4
    }
  }

  const pageCount = doc.getNumberOfPages()
  for (let p = 2; p <= pageCount; p++) {
    doc.setPage(p)
    const ph = doc.internal.pageSize.getHeight()
    doc.setDrawColor(235)
    doc.setLineWidth(0.2)
    doc.line(marginX, ph - 15, marginX + contentW, ph - 15)
    doc.setFont('helvetica', 'normal')
    doc.setFontSize(7)
    doc.setTextColor(160)
    doc.text('Dibuat otomatis oleh Edvion dari data pengerjaan asli — bukan estimasi dikarang.', marginX, ph - 10)
    doc.text(`Halaman ${p - 1} dari ${pageCount - 1}`, marginX + contentW, ph - 10, { align: 'right' })
  }

  doc.save(`sertifikat-${safeSlug(studentName)}-${safeSlug(run.template_title)}.pdf`)
}
