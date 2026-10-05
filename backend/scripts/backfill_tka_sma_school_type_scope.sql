-- Backfill school_type_scope='sma' untuk paket "TKA SMA" (id bd0eb837-3d87-4097-a96d-40f8ddaf81fe)
-- — ditemukan lewat diagnose_tka_scope_and_track_mismatches.sql (Query 1): 21 dari 21 sesi
-- paket ini punya school_type_scope NULL. Sama seperti backfill_tka_smp_school_type_scope.sql
-- sebelumnya, tapi untuk jenjang SMA kali ini.
--
-- CATATAN PENTING beda dari kasus SMP: karena tka_scale_for() default (untuk scope NULL/non-SMP)
-- SUDAH sama persis dengan skala SMA (200-800 / Istimewa>=725), backfill ini TIDAK mengubah
-- angka apa pun yang sudah tampil ke siswa/sekolah — murni kerapian data, supaya "jenjang belum
-- diisi" tidak lagi ambigu dengan "memang SMA" di laporan/diagnostik ke depannya. Jalankan SETELAH
-- fix_tka_sma_exam_track.sql (skrip ini tidak bergantung padanya, tapi keduanya sama-sama
-- mengoreksi data paket "TKA SMA").

BEGIN;

-- 1) Sesi yang terhubung LANGSUNG sebagai content item TryoutSession milik paket ini.
UPDATE tryout_sessions
SET school_type_scope = 'sma'
WHERE school_type_scope IS NULL
  AND id IN (
    SELECT pci.content_id
    FROM package_content_items pci
    WHERE pci.package_id = 'bd0eb837-3d87-4097-a96d-40f8ddaf81fe' -- TKA SMA
      AND pci.content_type = 'tryout_session'
  );

-- 2) Sesi yang terhubung lewat slot Simulation Template yang di-attach ke paket ini.
UPDATE tryout_sessions
SET school_type_scope = 'sma'
WHERE school_type_scope IS NULL
  AND id IN (
    SELECT sts.session_id
    FROM simulation_template_slots sts
    JOIN package_content_items pci
      ON pci.content_type = 'simulation_template' AND pci.content_id = sts.template_id
    WHERE pci.package_id = 'bd0eb837-3d87-4097-a96d-40f8ddaf81fe' -- TKA SMA
      AND sts.session_id IS NOT NULL
  );

-- 3) Simulation Template itu sendiri, supaya konsisten dengan sesi-sesinya (badge jenjang di admin).
UPDATE simulation_templates
SET school_type_scope = 'sma'
WHERE school_type_scope IS NULL
  AND id IN (
    SELECT pci.content_id
    FROM package_content_items pci
    WHERE pci.package_id = 'bd0eb837-3d87-4097-a96d-40f8ddaf81fe' -- TKA SMA
      AND pci.content_type = 'simulation_template'
  );

-- Verifikasi hasil sebelum commit — semua baris di bawah harus school_type_scope = 'sma'.
SELECT id, title, school_type_scope
FROM tryout_sessions
WHERE id IN (
  SELECT pci.content_id FROM package_content_items pci
  WHERE pci.package_id = 'bd0eb837-3d87-4097-a96d-40f8ddaf81fe' AND pci.content_type = 'tryout_session'
  UNION
  SELECT sts.session_id FROM simulation_template_slots sts
  JOIN package_content_items pci ON pci.content_type = 'simulation_template' AND pci.content_id = sts.template_id
  WHERE pci.package_id = 'bd0eb837-3d87-4097-a96d-40f8ddaf81fe' AND sts.session_id IS NOT NULL
);

SELECT id, title, school_type_scope
FROM simulation_templates
WHERE id IN (
  SELECT pci.content_id FROM package_content_items pci
  WHERE pci.package_id = 'bd0eb837-3d87-4097-a96d-40f8ddaf81fe' AND pci.content_type = 'simulation_template'
);

COMMIT; -- ganti jadi ROLLBACK kalau salah satu hasil verifikasi di atas bukan 'sma'
