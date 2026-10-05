-- ============================================================================
-- diagnose_real_duplicates.sql
-- Read-only. Query PART 2 sebelumnya salah filter (exam_track='tka' saja) —
-- padahal 3 paket "TKA SMP" yang ada semuanya ke-set exam_track='snbt' (bug
-- tersendiri, lihat catatan di bawah), jadi sesi anak-anaknya kemungkinan
-- BUKAN yang ke-tangkap PART 2 kemarin. Ini versi tanpa filter exam_track
-- sama sekali, supaya semua baris "Matematika"/"Bahasa Indonesia" ketahuan.
-- ============================================================================

-- PART 1: SEMUA sesi berjudul persis "Matematika" atau "Bahasa Indonesia"
-- (apapun exam_track-nya), plus kaitan ke package_content_items kalau ada.
SELECT
  ts.id AS session_id, ts.title, ts.exam_track, ts.is_premium, ts.is_draft,
  ts.question_count, ts.question_set_id, ts.created_at,
  pci.package_id, p.name AS package_name
FROM tryout_sessions ts
LEFT JOIN package_content_items pci ON pci.content_id = ts.id AND pci.content_type = 'tryout_session'
LEFT JOIN packages p ON p.id = pci.package_id
WHERE ts.title IN ('Matematika', 'Bahasa Indonesia')
ORDER BY ts.title, ts.created_at;

-- Baca hasilnya begini:
--  - Kalau ada LEBIH DARI 1 baris per title (mis. 2 baris "Matematika" dengan
--    session_id BEDA) → itu duplikat asli di database (bukan cuma bug render),
--    kemungkinan besar dari wizard yang dijalankan ulang (paket #3/#4).
--  - Kolom package_id/package_name NULL → sesi itu belum terdaftar di paket
--    manapun sama sekali (pasti selalu terkunci, apapun entitlement-nya).
--  - Bandingkan package_id di sini dengan package_id yang PUNYA entitlement
--    aktif ke sekolah Almasih (query PART A yang sebelumnya sudah saya kirim).


-- PART 2: entitlement aktif sekolah Almasih (ulang, biar satu paket lengkap)
SELECT s.id AS school_id, s.name AS school_name,
       e.package_id, p.name AS package_name, e.starts_at, e.expires_at
FROM schools s
LEFT JOIN school_package_entitlements e ON e.school_id = s.id
LEFT JOIN packages p ON p.id = e.package_id
WHERE s.name ILIKE '%Almasih%'
ORDER BY e.created_at DESC;
