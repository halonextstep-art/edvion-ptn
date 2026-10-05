-- Lanjutan diagnostik khusus untuk "TKA SMP #2" (package_id eafe5f0f-db91-4868-8f92-c8a0e77dabdc)
-- Read-only, aman dijalankan kapan saja.

-- A) Ada berapa paket lain dengan nama mengandung "TKA SMP"? (cek apakah ada paket "asli"
--    yang sudah benar terisi, dan #2 ini cuma duplikat sisa percobaan)
SELECT id, name, elective_pick_count, active, exam_track, created_at
FROM packages
WHERE name ILIKE '%TKA SMP%'
ORDER BY created_at;

-- B) Simulation_template yang ter-attach ke TKA SMP #2 — lihat apakah template ini sendiri
--    punya slot dengan sesi asli, atau ternyata juga kosong/tidak lengkap
SELECT
  st.id AS template_id, st.title AS template_title, st.is_active,
  sts.sequence_index, sts.session_id, ts.title AS session_title, ts.is_elective, ts.is_draft
FROM package_content_items pci
JOIN simulation_templates st ON st.id = pci.content_id AND pci.content_type = 'simulation_template'
LEFT JOIN simulation_template_slots sts ON sts.template_id = st.id
LEFT JOIN tryout_sessions ts ON ts.id = sts.session_id
WHERE pci.package_id = 'eafe5f0f-db91-4868-8f92-c8a0e77dabdc'
ORDER BY sts.sequence_index;

-- Cara baca:
--   - Kalau query A menunjukkan ada paket "TKA SMP" LAIN yang elective_pick_count>0, active=true,
--     DAN (dari query diagnostik sebelumnya) jumlah_item_tryout_session > 0 -> itu paket yang
--     sebenarnya dipakai siswa; "TKA SMP #2" adalah duplikat kosong yang aman dinonaktifkan.
--   - Kalau query B menunjukkan template-nya punya slot dengan session_id & session_title terisi
--     -> sesi-sesi itu SUDAH ADA di database, cuma belum pernah ditambahkan sebagai content item
--     langsung ke paket "TKA SMP #2" -> bisa diperbaiki dengan menambahkan sesi-sesi itu sebagai
--     content item tryout_session ke paket ini (aku bisa buatkan script fix-nya).
--   - Kalau query B kosong/semua NULL -> template-nya sendiri juga tidak punya slot terisi ->
--     paket ini memang harus dibuat ulang dari awal lewat wizard, bukan diperbaiki.
