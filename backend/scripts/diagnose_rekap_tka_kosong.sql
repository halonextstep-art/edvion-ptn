-- Diagnostik: kenapa "Rekap TKA" di Portal Sekolah kosong ("Belum ada paket TKA yang
-- dikonfigurasi di sistem"). Read-only — tidak mengubah data apapun, aman dijalankan kapan saja.
--
-- CATATAN: query di bawah ini masih berguna untuk melihat data mentahnya, tapi penjelasan
-- syarat "paket TKA" di bawah sudah DIPERBARUI mengikuti perbaikan school_tka_service.rs —
-- deteksi sekarang pakai exam_track, bukan elective_pick_count lagi:
--
-- Rekap TKA (backend: SchoolTkaService::tka_roster) menganggap sebuah Package sebagai "paket
-- TKA" HANYA kalau kedua syarat ini terpenuhi:
--   1) packages.exam_track = 'tka'
--   2) Package itu punya minimal 1 sesi non-draft yang bisa diresolve — baik lewat
--      package_content_items berjenis 'tryout_session' langsung, ATAUPUN lewat slot sebuah
--      'simulation_template' yang ter-attach ke paket itu (keduanya digabung/union sekarang)
-- Kalau salah satu syarat gagal, paket itu di-skip diam-diam dari Rekap TKA.
--
-- Query-query lawas di bawah (yang mengecek elective_pick_count > 0 dan jumlah_item_tryout_session)
-- tetap valid untuk diagnostik data, tapi bukan lagi syarat penentu "paket TKA" itu sendiri.

-- 1) Semua paket dengan elective_pick_count > 0 (kandidat "paket TKA")
SELECT id, name, elective_pick_count, active, exam_track, created_at
FROM packages
WHERE elective_pick_count > 0
ORDER BY created_at DESC;

-- 2) Untuk tiap paket di atas: jumlah content item per jenis, dan berapa sesi yang MASIH draft
--    (kalau semua draft, paket itu tetap tidak akan muncul walau elective_pick_count > 0)
SELECT
  p.id AS package_id,
  p.name AS package_name,
  p.elective_pick_count,
  p.active AS package_aktif,
  COUNT(*) FILTER (WHERE pci.content_type = 'tryout_session') AS jumlah_item_tryout_session,
  COUNT(*) FILTER (WHERE pci.content_type = 'simulation_template') AS jumlah_item_simulation_template,
  COUNT(*) FILTER (WHERE pci.content_type = 'tryout_session' AND ts.is_draft = false) AS sesi_sudah_publish,
  COUNT(*) FILTER (WHERE pci.content_type = 'tryout_session' AND ts.is_draft = true) AS sesi_masih_draft
FROM packages p
LEFT JOIN package_content_items pci ON pci.package_id = p.id
LEFT JOIN tryout_sessions ts ON pci.content_type = 'tryout_session' AND ts.id = pci.content_id
WHERE p.elective_pick_count > 0
GROUP BY p.id, p.name, p.elective_pick_count, p.active
ORDER BY p.created_at DESC;

-- Cara baca hasil query #2:
--   - Kalau tabel ini KOSONG sama sekali -> memang belum ada paket dengan elective_pick_count > 0
--     sama sekali. Solusinya: buat/edit paket TKA di Admin > Kelola Paket, isi "Jumlah Mapel
--     Pilihan" > 0 di Step 1 wizard.
--   - Kalau ada baris tapi "jumlah_item_tryout_session" = 0 -> paket ini dibuat TIDAK lewat
--     wizard "Buat Paket + Subtes" standar (sesi individualnya tidak pernah ditambahkan sebagai
--     content item ke paket). Perlu ditambahkan manual, atau dibuat ulang lewat wizard.
--   - Kalau "sesi_sudah_publish" = 0 tapi "sesi_masih_draft" > 0 -> paket ini SUDAH benar
--     strukturnya, tapi belum "dipublish"/diaktifkan. Cek toggle "Aktif" paket ini di
--     Admin > Kelola Paket — begitu diaktifkan, publish cascade akan set is_draft = false pada
--     sesi-sesinya dan Rekap TKA akan langsung terisi.
