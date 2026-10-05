-- Diagnostik read-only (TIDAK mengubah data apa pun) — cek dua kelas masalah yang sama
-- persis dengan yang sudah terbukti terjadi di produksi ("TKA SMP #2"): lihat
-- fix_tka_smp_exam_track.sql dan backfill_tka_smp_school_type_scope.sql.
--
-- Bedanya dari query diagnostik di backfill_tka_smp_school_type_scope.sql (yang cuma menyaring
-- nama paket ILIKE '%smp%'): script ini memeriksa SEMUA paket TKA apa pun namanya, dan juga
-- memeriksa Simulation Template (bukan cuma tryout_sessions langsung), plus satu kelas masalah
-- baru: sesi/template yang exam_track-nya tidak cocok dengan paket induknya.
--
-- Cara pakai: jalankan seluruh file ini (read-only, aman berkali-kali). Kirim hasil ketiga
-- query di bawah balik ke Claude untuk ditentukan apakah perlu backfill lanjutan.

-- ═══════════════════════════════════════════════════════════════════════════════════════
-- QUERY 1: Paket TKA yang jenjangnya (school_type_scope) AMBIGU — semua sesi & template yang
-- terhubung punya school_type_scope = NULL. Paket seperti ini diam-diam jatuh ke skala
-- SMA/SMK/MA (200-800/Istimewa>=725) di kode saat ini (lihat tka_scale_for() di
-- backend/src/application/school_tka_service.rs) — kalau paket ini sebenarnya untuk SMP,
-- siswanya akan melihat skala dan ambang Istimewa yang SALAH.
-- ═══════════════════════════════════════════════════════════════════════════════════════
WITH tka_sessions AS (
  -- Sesi yang terhubung LANGSUNG sebagai content item TryoutSession
  SELECT pci.package_id, ts.id AS session_id, ts.title, ts.school_type_scope
  FROM package_content_items pci
  JOIN tryout_sessions ts ON ts.id = pci.content_id
  WHERE pci.content_type = 'tryout_session'
  UNION
  -- Sesi yang terhubung lewat slot Simulation Template yang di-attach ke paket
  SELECT pci.package_id, sts.session_id, ts.title, ts.school_type_scope
  FROM package_content_items pci
  JOIN simulation_template_slots sts ON sts.template_id = pci.content_id
  JOIN tryout_sessions ts ON ts.id = sts.session_id
  WHERE pci.content_type = 'simulation_template' AND sts.session_id IS NOT NULL
)
SELECT
  p.id AS package_id,
  p.name AS package_name,
  p.exam_track,
  COUNT(DISTINCT ts.session_id) AS total_sesi,
  COUNT(DISTINCT ts.session_id) FILTER (WHERE ts.school_type_scope IS NULL) AS sesi_tanpa_jenjang,
  STRING_AGG(DISTINCT ts.school_type_scope::text, ', ') FILTER (WHERE ts.school_type_scope IS NOT NULL) AS jenjang_terdeteksi
FROM packages p
JOIN tka_sessions ts ON ts.package_id = p.id
WHERE p.exam_track = 'tka'
GROUP BY p.id, p.name, p.exam_track
HAVING COUNT(DISTINCT ts.session_id) FILTER (WHERE ts.school_type_scope IS NOT NULL) = 0
ORDER BY p.name;

-- ═══════════════════════════════════════════════════════════════════════════════════════
-- QUERY 2: Paket TKA yang sesinya jenjangnya BERCAMPUR (sebagian sesi punya jenjang berbeda
-- satu sama lain dalam paket yang sama) — kemungkinan kesalahan input admin, karena satu
-- paket TKA seharusnya cuma untuk satu jenjang.
-- ═══════════════════════════════════════════════════════════════════════════════════════
WITH tka_sessions AS (
  SELECT pci.package_id, ts.id AS session_id, ts.school_type_scope
  FROM package_content_items pci
  JOIN tryout_sessions ts ON ts.id = pci.content_id
  WHERE pci.content_type = 'tryout_session'
  UNION
  SELECT pci.package_id, sts.session_id, ts.school_type_scope
  FROM package_content_items pci
  JOIN simulation_template_slots sts ON sts.template_id = pci.content_id
  JOIN tryout_sessions ts ON ts.id = sts.session_id
  WHERE pci.content_type = 'simulation_template' AND sts.session_id IS NOT NULL
)
SELECT
  p.id AS package_id,
  p.name AS package_name,
  COUNT(DISTINCT ts.school_type_scope) FILTER (WHERE ts.school_type_scope IS NOT NULL) AS jumlah_jenjang_berbeda,
  STRING_AGG(DISTINCT COALESCE(ts.school_type_scope::text, '(kosong)'), ', ') AS daftar_jenjang
FROM packages p
JOIN tka_sessions ts ON ts.package_id = p.id
WHERE p.exam_track = 'tka'
GROUP BY p.id, p.name
HAVING COUNT(DISTINCT ts.school_type_scope) FILTER (WHERE ts.school_type_scope IS NOT NULL) > 1
ORDER BY p.name;

-- ═══════════════════════════════════════════════════════════════════════════════════════
-- QUERY 3: Konten (TryoutSession / SimulationTemplate) yang exam_track-nya TIDAK COCOK
-- dengan paket induknya — persis kelas bug "TKA SMP #2" (paket TKA tapi sesinya ke-tag
-- 'snbt', atau sebaliknya). Setelah perbaikan guardrail di add_content_item, kasus BARU
-- tidak akan bisa lagi terjadi lewat aplikasi — tapi baris LAMA yang sudah kepasang sebelum
-- guardrail ini perlu dicek manual di sini.
-- ═══════════════════════════════════════════════════════════════════════════════════════
SELECT
  p.id AS package_id, p.name AS package_name, p.exam_track AS jalur_paket,
  pci.content_type, pci.content_id,
  COALESCE(ts.title, st.title) AS judul_konten,
  COALESCE(ts.exam_track::text, st.exam_track::text) AS jalur_konten
FROM package_content_items pci
JOIN packages p ON p.id = pci.package_id
LEFT JOIN tryout_sessions ts ON pci.content_type = 'tryout_session' AND ts.id = pci.content_id
LEFT JOIN simulation_templates st ON pci.content_type = 'simulation_template' AND st.id = pci.content_id
WHERE COALESCE(ts.exam_track, st.exam_track) IS DISTINCT FROM p.exam_track
ORDER BY p.name;
