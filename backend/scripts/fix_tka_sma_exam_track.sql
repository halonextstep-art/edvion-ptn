-- Koreksi data untuk paket "TKA SMA" (id bd0eb837-3d87-4097-a96d-40f8ddaf81fe) — ditemukan
-- lewat diagnose_tka_scope_and_track_mismatches.sql (Query 3): 3 sesi wajibnya masih ke-tag
-- exam_track='snbt' padahal paket induknya 'tka' dan judul sesinya sendiri jelas "TKA SMA - ...
-- (Wajib)". Ini data LAMA dari sebelum jalur ujian eksplisit ada (persis kelas bug yang sama
-- dengan "TKA SMP #2", lihat fix_tka_smp_exam_track.sql) — guardrail baru di
-- PackageService::add_content_item mencegah kasus BARU seperti ini terulang lewat aplikasi,
-- tapi baris lama ini perlu dikoreksi manual.
--
-- Dampak sebelum dikoreksi: ketiga sesi ini berpotensi salah tampil di tab "SNBT/UTBK" alih-alih
-- "TKA" di katalog siswa (DrillingZone.vue), karena filter tab memakai exam_track milik sesi
-- itu sendiri, bukan cuma exam_track paket induknya.

BEGIN;

UPDATE tryout_sessions
SET exam_track = 'tka'
WHERE id IN (
  'c66ce729-ad79-4a32-934b-33ae01967dac', -- TKA SMA - Bahasa Indonesia (Wajib)
  'c57f4376-5a0f-4e1a-bc92-a37fcd4b024d', -- TKA SMA - Matematika (Wajib)
  '6bd7d003-8154-4903-adfc-618abfe12e61'  -- TKA SMA - Bahasa Inggris (Wajib)
);

-- Verifikasi hasil sebelum commit — ketiga baris di bawah harus exam_track = 'tka'.
SELECT id, title, exam_track
FROM tryout_sessions
WHERE id IN (
  'c66ce729-ad79-4a32-934b-33ae01967dac',
  'c57f4376-5a0f-4e1a-bc92-a37fcd4b024d',
  '6bd7d003-8154-4903-adfc-618abfe12e61'
);

COMMIT; -- ganti jadi ROLLBACK kalau salah satu hasil verifikasi di atas bukan 'tka'

-- Setelah COMMIT, jalankan ulang Query 3 di diagnose_tka_scope_and_track_mismatches.sql untuk
-- pastikan tidak ada baris tersisa untuk paket "TKA SMA" ini.
