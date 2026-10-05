-- ============================================================================
-- diagnose_duplicate_sessions.sql
-- Read-only. Untuk cek APAKAH ada sesi tryout duplikat (judul sama, ID beda)
-- dan MANA persisnya yang terdaftar sebagai isi paket "TKA SMP #1".
-- ============================================================================

-- ── PART 1: Semua paket bernama mirip "TKA SMP" (jaga-jaga kalau ada paket
-- duplikat juga, bukan cuma sesinya) ────────────────────────────────────────
SELECT id AS package_id, name, exam_track, created_at
FROM packages
WHERE name ILIKE '%TKA SMP%'
ORDER BY created_at;

-- ── PART 2: Semua sesi tryout "Matematika" / "Bahasa Indonesia" jalur TKA,
-- plus status draft & premium-nya ───────────────────────────────────────────
SELECT id AS session_id, title, exam_track, is_premium, is_draft, question_count, question_set_id, created_at
FROM tryout_sessions
WHERE exam_track = 'tka' AND (title ILIKE '%Matematika%' OR title ILIKE '%Bahasa Indonesia%')
ORDER BY title, created_at;

-- ── PART 3: Dari session_id di PART 2, MANA yang benar-benar terdaftar
-- sebagai isi package_id dari PART 1 ────────────────────────────────────────
-- Ganti '<PACKAGE_ID>' dengan package_id "TKA SMP #1" dari PART 1.
SELECT pci.content_id AS session_id, ts.title, ts.created_at
FROM package_content_items pci
JOIN tryout_sessions ts ON ts.id = pci.content_id
WHERE pci.package_id = '<PACKAGE_ID>'::uuid
  AND pci.content_type = 'tryout_session';

-- Bandingkan hasil PART 3 dengan PART 2:
--   - Kalau session_id di PART 3 TIDAK ADA di daftar PART 2 sama sekali →
--     paket ini menunjuk ke sesi yang sudah tidak muncul di katalog (mis.
--     is_draft = true, atau malah sudah terhapus) — itu penyebabnya.
--   - Kalau session_id di PART 3 ADA tapi is_draft = true di PART 2 →
--     sesi yang terdaftar di paket masih berstatus draft, jadi tidak pernah
--     tampil sebagai "aktif" ke siswa meski secara akses seharusnya terbuka.
--   - Kalau PART 2 menunjukkan 2 baris Matematika tapi PART 3 cuma
--     menyebut SALAH SATU id-nya → itu konfirmasi resmi duplikat: yang satu
--     terdaftar (harusnya unlocked), satunya lagi orphan (akan selalu
--     terkunci karena tidak ada paket manapun yang mengaitkannya).
