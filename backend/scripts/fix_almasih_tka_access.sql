-- ============================================================================
-- fix_almasih_tka_access.sql
-- Otomatis, tidak perlu isi ID manual. Menjalankan file ini akan:
--   1. Menampilkan diagnosa (baca-saja) semua sesi "Matematika"/"Bahasa
--      Indonesia" apapun namanya persis, dan paket entitled sekolah Almasih.
--   2. MENGUBAH DATA: memastikan 2 sesi asli yang sudah punya soal sungguhan
--      (Matematika & Bahasa Indonesia, hasil pemulihan sebelumnya) terdaftar
--      di paket yang BENAR-BENAR di-assign aktif ke sekolah "SMP Almasih" —
--      apapun nama paketnya (#1/#3/#4), otomatis dicari dari entitlement-nya.
-- Aman dijalankan berkali-kali (ON CONFLICT DO NOTHING / idempotent).
-- ============================================================================

-- ── PART 1 (baca-saja): semua sesi "Matematika"/"Bahasa Indonesia" apapun
-- variasi judulnya, plus status paketnya ────────────────────────────────────
SELECT
  ts.id AS session_id, ts.title, ts.exam_track, ts.is_premium, ts.is_draft,
  ts.question_set_id, ts.created_at,
  pci.package_id, p.name AS package_name
FROM tryout_sessions ts
LEFT JOIN package_content_items pci ON pci.content_id = ts.id AND pci.content_type = 'tryout_session'
LEFT JOIN packages p ON p.id = pci.package_id
WHERE ts.title ILIKE '%Matematika%' OR ts.title ILIKE '%Bahasa Indonesia%'
ORDER BY ts.title, ts.created_at;

-- ── PART 2 (baca-saja): paket yang punya entitlement AKTIF ke sekolah Almasih
SELECT s.id AS school_id, s.name AS school_name, e.package_id, p.name AS package_name,
       e.starts_at, e.expires_at
FROM schools s
JOIN school_package_entitlements e ON e.school_id = s.id
JOIN packages p ON p.id = e.package_id
WHERE s.name ILIKE '%Almasih%'
  AND e.starts_at <= CURRENT_DATE
  AND (e.expires_at IS NULL OR e.expires_at >= CURRENT_DATE);

-- ── PART 3 (UBAH DATA): daftarkan 2 sesi asli (yang sudah punya soal
-- sungguhan dari pemulihan sebelumnya) ke paket yang entitled di PART 2 di
-- atas. Kalau PART 2 kosong (tidak ada entitlement aktif sama sekali),
-- INSERT ini tidak akan menghasilkan baris apapun — aman, tidak error.
INSERT INTO package_content_items (package_id, content_type, content_id)
SELECT e.package_id, 'tryout_session', ts.id
FROM schools s
JOIN school_package_entitlements e ON e.school_id = s.id
  AND e.starts_at <= CURRENT_DATE AND (e.expires_at IS NULL OR e.expires_at >= CURRENT_DATE)
CROSS JOIN tryout_sessions ts
WHERE s.name ILIKE '%Almasih%'
  AND ts.title IN ('Matematika', 'Bahasa Indonesia')
  AND ts.exam_track = 'tka'
  AND ts.is_draft = false
ON CONFLICT DO NOTHING
RETURNING package_id, content_id;

-- Kalau PART 3 mengembalikan baris (RETURNING) — berarti tadinya memang
-- belum terdaftar, dan sekarang sudah dibetulkan. Kalau tidak ada baris sama
-- sekali (0 rows), berarti sudah terdaftar sebelumnya (bukan ini masalahnya)
-- ATAU sekolahnya tidak ketemu / tidak ada entitlement aktif — kirim ke saya
-- hasil PART 1 & PART 2 kalau ini terjadi supaya saya cek lebih lanjut.
