-- ============================================================================
-- attach_matematika_to_set.sql
--
-- Pemulihan batch soal Matematika yang nyasar (sama seperti batch Bahasa
-- Indonesia yang sudah berhasil dipulihkan sebelumnya) — soal masuk Bank Soal
-- tapi tidak tertaut ke Set Soal manapun, sehingga tidak muncul di subtes.
--
-- Target Set Soal sudah diketahui: question_set_id milik sesi "Matematika"
-- (exam_track tka) = 4e8d2e58-e403-4b29-bbf1-2b6631132dfc — TIDAK perlu isi
-- placeholder apapun, langsung jalankan.
-- ============================================================================

-- ── PART 1 (baca-saja): cek soal Matematika draft/review yang belum
-- tertaut ke Set Soal manapun — pastikan jumlahnya sesuai ekspektasi (~30).
SELECT q.id, q.code, q.subject, q.topic, q.question_type, q.status, q.created_at, u.email AS dibuat_oleh
FROM questions q
JOIN users u ON u.id = q.created_by
WHERE q.status IN ('draft', 'review')
  AND q.subject = 'Matematika'
  AND NOT EXISTS (SELECT 1 FROM question_set_items qsi WHERE qsi.question_id = q.id)
ORDER BY q.created_at DESC;

-- ── PART 2 (UBAH DATA): tautkan semua soal di atas ke Set Soal Matematika
-- TKA SMP. Idempotent — aman dijalankan ulang (NOT EXISTS mencegah dobel).
BEGIN;

INSERT INTO question_set_items (id, set_id, question_id, sort_order)
SELECT
    gen_random_uuid(),
    '4e8d2e58-e403-4b29-bbf1-2b6631132dfc'::uuid,
    q.id,
    (SELECT COALESCE(MAX(sort_order), -1) FROM question_set_items WHERE set_id = '4e8d2e58-e403-4b29-bbf1-2b6631132dfc'::uuid)
        + ROW_NUMBER() OVER (ORDER BY q.created_at)
FROM questions q
WHERE q.status IN ('draft', 'review')
  AND q.subject = 'Matematika'
  AND NOT EXISTS (SELECT 1 FROM question_set_items qsi WHERE qsi.question_id = q.id)
RETURNING question_id;

-- Cek jumlah baris "INSERT n" di atas cocok dengan hasil PART 1 sebelum
-- COMMIT. Ganti ke ROLLBACK dulu kalau mau coba tanpa benar-benar menyimpan.
COMMIT;
