-- ============================================================================
-- attach_unlinked_questions_to_set.sql
--
-- Skrip pemulihan untuk kasus: soal berhasil dibuat/diimpor (ada di Bank Soal),
-- tapi TIDAK terhubung ke Set Soal manapun — sehingga tidak muncul di subtes
-- paket yang dituju. Ini bisa terjadi kalau dropdown "Set Soal" di dialog
-- Import Soal / form Tulis Soal kosong ("Tidak ada") saat soal dibuat.
--
-- (Sejak perbaikan di QuestionImportDialog.vue — dropdown ini sekarang jadi
-- banner target terkunci saat dibuka dari dalam subtes suatu paket, supaya
-- tidak ke-kosongkan tanpa sadar — skrip ini untuk memulihkan soal yang
-- SUDAH terlanjur nyasar SEBELUM perbaikan itu ada.)
--
-- CARA PAKAI — 3 LANGKAH, jalankan satu-satu, baca hasilnya sebelum lanjut:
--
--   psql "$DATABASE_URL" -f backend/scripts/attach_unlinked_questions_to_set.sql
--
-- atau tempel tiap PART secara terpisah ke psql interaktif kalau mau lebih
-- hati-hati (disarankan, karena PART 2 dan PART 3 butuh nilai yang Anda isi
-- sendiri berdasarkan hasil PART 1).
-- ============================================================================


-- ── PART 1: Temukan Set Soal tujuan ─────────────────────────────────────────
-- Ganti '%Bahasa Indonesia%' dengan potongan nama subtes/paket yang dituju.
-- Nama Set Soal mengikuti pola "<Nama Paket> — <Judul Subtes>" (lihat
-- PackageWizard.vue materializeSubtests()). Catat `id` dari baris yang benar.

SELECT id, name, created_at
FROM question_sets
WHERE name ILIKE '%Bahasa Indonesia%'
ORDER BY created_at DESC;


-- ── PART 2: Lihat soal yang BELUM terhubung ke Set Soal manapun ────────────
-- Soal draft/review yang tidak punya baris di question_set_items sama sekali.
-- Cek kolom `dibuat_oleh` dan `created_at` untuk pastikan ini memang soal
-- hasil import/tulis yang "nyasar" tadi, bukan draft lain yang sengaja
-- dibiarkan lepas dari Set Soal manapun.

SELECT q.id, q.code, q.subject, q.topic, q.question_type, q.status, q.created_at,
       u.email AS dibuat_oleh
FROM questions q
JOIN users u ON u.id = q.created_by
WHERE q.status IN ('draft', 'review')
  AND NOT EXISTS (SELECT 1 FROM question_set_items qsi WHERE qsi.question_id = q.id)
ORDER BY q.created_at DESC
LIMIT 50;


-- ── PART 3: Tautkan ke Set Soal tujuan ──────────────────────────────────────
-- WAJIB ganti KEDUA placeholder di bawah dengan nilai literalnya SEBELUM
-- menjalankan bagian ini (edit langsung teks skrip ini, lalu jalankan lewat
-- psql seperti biasa — bukan lewat argumen -v):
--
--   <TARGET_SET_ID>     -> id dari PART 1, contoh: 8f14e2a1-... (tanpa tanda kutip tambahan)
--   <CUTOFF_TIMESTAMP>  -> timestamp SEBELUM soal pertama yang mau ditautkan
--                          dibuat (dari kolom created_at di PART 2), supaya
--                          tidak ikut menyapu draft lama yang tidak terkait.
--                          Format: 2026-08-19 10:00:00+00
--
-- Skrip ini idempotent — aman dijalankan ulang: setelah COMMIT pertama,
-- soal yang sama tidak akan ke-insert dua kali (sudah lolos NOT EXISTS di
-- atas), jalan kedua kalinya cukup meng-insert 0 baris, tidak error.
-- Disarankan `pg_dump "$DATABASE_URL" > backup-sebelum-attach.sql` dulu,
-- sama seperti catatan di cleanup_demo_data.sql.

BEGIN;

INSERT INTO question_set_items (id, set_id, question_id, sort_order)
SELECT
    gen_random_uuid(),
    '<TARGET_SET_ID>'::uuid,
    q.id,
    (SELECT COALESCE(MAX(sort_order), -1) FROM question_set_items WHERE set_id = '<TARGET_SET_ID>'::uuid)
        + ROW_NUMBER() OVER (ORDER BY q.created_at)
FROM questions q
WHERE q.status IN ('draft', 'review')
  AND q.created_at >= '<CUTOFF_TIMESTAMP>'::timestamptz
  AND NOT EXISTS (SELECT 1 FROM question_set_items qsi WHERE qsi.question_id = q.id)
RETURNING question_id;

-- Cek jumlah baris "INSERT n" di atas cocok dengan jumlah soal yang Anda
-- harapkan (dari PART 2) sebelum COMMIT. Ganti ke ROLLBACK dulu kalau mau
-- coba tanpa benar-benar menyimpan.
COMMIT;
