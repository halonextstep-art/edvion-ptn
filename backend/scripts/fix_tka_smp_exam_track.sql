-- Koreksi data untuk "TKA SMP #2" (satu-satunya paket TKA SMP yang terbukti punya konten
-- nyata — 2 sesi wajib "Matematika" & "Bahasa Indonesia", sudah terpublish, terhubung lewat
-- Simulation Template "TKA 1 SMP"). Dua baris di bawah AMAN dijalankan: exam_track dikoreksi
-- dari 'snbt' (mislabel dari sebelum fitur jalur ujian eksplisit ada) jadi 'tka', dan
-- elective_pick_count dinormalkan ke 0 karena template-nya memang tidak punya slot mapel
-- pilihan sama sekali (TKA SMP = semua wajib, sesuai aturan TKA yang sesungguhnya).

BEGIN;

UPDATE packages
SET exam_track = 'tka'
WHERE id = 'eafe5f0f-db91-4868-8f92-c8a0e77dabdc'; -- TKA SMP #2

UPDATE packages
SET elective_pick_count = 0
WHERE id = 'eafe5f0f-db91-4868-8f92-c8a0e77dabdc'; -- TKA SMP #2

-- Verifikasi hasil sebelum commit
SELECT id, name, elective_pick_count, active, exam_track
FROM packages
WHERE id = 'eafe5f0f-db91-4868-8f92-c8a0e77dabdc';

COMMIT; -- ganti jadi ROLLBACK kalau hasil verifikasi di atas tidak sesuai harapan


-- ─────────────────────────────────────────────────────────────────────────────────────
-- TKA SMP #1, #3, #4 SENGAJA TIDAK disentuh oleh script ini — semuanya elective_pick_count=0
-- dan namanya mengikuti pola paket kosong duplikat yang pernah ditemukan sebelumnya di
-- proyek ini (lihat task historis "Script cleanup paket kosong duplikat"). Sebelum
-- memutuskan apa-apa untuk ketiganya, jalankan dulu query di bawah untuk lihat apakah
-- masing-masing punya konten nyata atau memang kosong:

SELECT
  p.id, p.name, p.active,
  COUNT(pci.id) AS jumlah_content_item,
  COUNT(*) FILTER (WHERE pci.content_type = 'tryout_session') AS jumlah_tryout_session,
  COUNT(*) FILTER (WHERE pci.content_type = 'simulation_template') AS jumlah_simulation_template
FROM packages p
LEFT JOIN package_content_items pci ON pci.package_id = p.id
WHERE p.name IN ('TKA SMP #1', 'TKA SMP #3', 'TKA SMP #4')
GROUP BY p.id, p.name, p.active;

-- Kalau hasilnya menunjukkan jumlah_content_item = 0 untuk semua tiga -> benar paket kosong
-- sisa percobaan, aman dinonaktifkan (UPDATE packages SET active = false WHERE id = '...')
-- supaya tidak membingungkan di daftar paket admin. Kalau ternyata salah satu punya konten
-- nyata, kabari dulu sebelum diapa-apakan — jangan diasumsikan aman dihapus/dinonaktifkan.
