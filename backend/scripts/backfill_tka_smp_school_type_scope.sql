-- Backfill school_type_scope='smp' untuk paket "TKA SMP #2" (satu-satunya paket TKA SMP yang
-- terbukti punya konten nyata — lihat fix_tka_smp_exam_track.sql). Diperlukan supaya fitur skala
-- nilai jenjang-aware (0-100 utk SD/SMP vs 200-800 utk SMA/SMK/MA, per Perka BSKAP No. 045 & 047
-- Tahun 2025) menampilkan skala yang BENAR untuk siswa SMP — sebelum ini, school_type_scope
-- sesi-sesi TKA SMP #2 belum pernah diisi (fitur school_type_scope predates fitur ini, dan tidak
-- ada backfill otomatis saat itu), jadi kalau dibiarkan kosong sistem akan salah jatuh ke skala
-- SMA/SMK/MA (200-800) untuk siswa SMP.
--
-- Jalankan SETELAH fix_tka_smp_exam_track.sql (skrip ini tidak bergantung padanya, tapi
-- keduanya sama-sama mengoreksi data paket "TKA SMP #2").

BEGIN;

-- 1) Sesi yang terhubung LANGSUNG sebagai content item TryoutSession milik paket ini.
UPDATE tryout_sessions
SET school_type_scope = 'smp'
WHERE school_type_scope IS NULL
  AND id IN (
    SELECT pci.content_id
    FROM package_content_items pci
    WHERE pci.package_id = 'eafe5f0f-db91-4868-8f92-c8a0e77dabdc' -- TKA SMP #2
      AND pci.content_type = 'tryout_session'
  );

-- 2) Sesi yang terhubung lewat slot Simulation Template yang di-attach ke paket ini (jalur
--    "Susun Simulasi TKA" wizard — lihat application::school_tka_service doc comment, sesi tidak
--    pernah diattach lagi secara terpisah lewat jalur ini).
UPDATE tryout_sessions
SET school_type_scope = 'smp'
WHERE school_type_scope IS NULL
  AND id IN (
    SELECT sts.session_id
    FROM simulation_template_slots sts
    JOIN package_content_items pci
      ON pci.content_type = 'simulation_template' AND pci.content_id = sts.template_id
    WHERE pci.package_id = 'eafe5f0f-db91-4868-8f92-c8a0e77dabdc' -- TKA SMP #2
      AND sts.session_id IS NOT NULL
  );

-- 3) Simulation Template itu sendiri juga dikoreksi supaya konsisten dengan sesi-sesinya (badge
--    jenjang di admin, dsb).
UPDATE simulation_templates
SET school_type_scope = 'smp'
WHERE school_type_scope IS NULL
  AND id IN (
    SELECT pci.content_id
    FROM package_content_items pci
    WHERE pci.package_id = 'eafe5f0f-db91-4868-8f92-c8a0e77dabdc' -- TKA SMP #2
      AND pci.content_type = 'simulation_template'
  );

-- Verifikasi hasil sebelum commit — semua baris di bawah harus school_type_scope = 'smp'.
SELECT id, title, school_type_scope
FROM tryout_sessions
WHERE id IN (
  SELECT pci.content_id FROM package_content_items pci
  WHERE pci.package_id = 'eafe5f0f-db91-4868-8f92-c8a0e77dabdc' AND pci.content_type = 'tryout_session'
  UNION
  SELECT sts.session_id FROM simulation_template_slots sts
  JOIN package_content_items pci ON pci.content_type = 'simulation_template' AND pci.content_id = sts.template_id
  WHERE pci.package_id = 'eafe5f0f-db91-4868-8f92-c8a0e77dabdc' AND sts.session_id IS NOT NULL
);

SELECT id, title, school_type_scope
FROM simulation_templates
WHERE id IN (
  SELECT pci.content_id FROM package_content_items pci
  WHERE pci.package_id = 'eafe5f0f-db91-4868-8f92-c8a0e77dabdc' AND pci.content_type = 'simulation_template'
);

COMMIT; -- ganti jadi ROLLBACK kalau salah satu hasil verifikasi di atas bukan 'smp'


-- ─────────────────────────────────────────────────────────────────────────────────────
-- CATATAN untuk paket TKA lain (mis. "TKA SMA"): sengaja TIDAK disentuh skrip ini. Paket
-- ber-jenjang SMA/SMK/MA (atau school_type_scope kosong sama sekali) sudah otomatis memakai
-- skala default 200-800/Istimewa≥725 di kode baru — tidak perlu backfill apa pun. Jalankan
-- query di bawah HANYA untuk mengecek apakah ada paket TKA lain yang ternyata perlu jenjang SMP
-- juga (nama mengandung "SMP" tapi belum ke-cover di atas):

SELECT p.id, p.name, p.exam_track,
       COUNT(*) FILTER (WHERE ts.school_type_scope IS NULL) AS sesi_tanpa_jenjang,
       COUNT(*) FILTER (WHERE ts.school_type_scope = 'smp') AS sesi_smp,
       COUNT(*) FILTER (WHERE ts.school_type_scope IS NOT NULL AND ts.school_type_scope != 'smp') AS sesi_jenjang_lain
FROM packages p
JOIN package_content_items pci ON pci.package_id = p.id AND pci.content_type = 'tryout_session'
JOIN tryout_sessions ts ON ts.id = pci.content_id
WHERE p.exam_track = 'tka' AND p.name ILIKE '%smp%'
GROUP BY p.id, p.name, p.exam_track;
