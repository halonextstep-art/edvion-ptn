-- ============================================================================
-- cleanup_duplicate_sessions.sql
-- Membersihkan sesi tryout duplikat (judul sama persis, ID beda) yang bikin
-- kartu "Matematika"/"Bahasa Indonesia" tampil 2x di layar siswa — sisa dari
-- wizard yang sempat dijalankan ulang beberapa kali.
--
-- Aman by design: HANYA menghapus baris yang 100% tidak dipakai sama sekali
-- (tidak ada soal di Set Soal-nya, tidak terdaftar di paket manapun, tidak
-- ada attempt siswa yang pernah mengerjakannya). Baris yang PUNYA salah satu
-- dari itu tidak akan disentuh, apapun judulnya.
-- ============================================================================

-- PART 1 (baca-saja): lihat dulu semua duplikat & status pakainya sebelum hapus.
SELECT
  ts.id, ts.title, ts.exam_track, ts.created_at,
  (SELECT count(*) FROM question_set_items qsi WHERE qsi.set_id = ts.question_set_id) AS jumlah_soal,
  (SELECT count(*) FROM package_content_items pci WHERE pci.content_id = ts.id) AS jumlah_paket,
  (SELECT count(*) FROM attempts a WHERE a.session_id = ts.id) AS jumlah_attempt
FROM tryout_sessions ts
WHERE ts.title IN (
  SELECT title FROM tryout_sessions
  WHERE exam_track = 'tka'
  GROUP BY title
  HAVING count(*) > 1
)
ORDER BY ts.title, ts.created_at;

-- Baris dengan jumlah_soal = 0 DAN jumlah_paket = 0 DAN jumlah_attempt = 0
-- itu yang akan dihapus PART 2 di bawah. Kalau ada baris duplikat yang
-- ternyata SEMUANYA punya soal/paket/attempt (tidak ada yang benar-benar
-- kosong), PART 2 tidak akan menghapus apapun — aman, tidak ada yang salah
-- kena hapus.

-- PART 2 (HAPUS): jalankan setelah cek PART 1.
DELETE FROM tryout_sessions ts
WHERE ts.title IN (
  SELECT title FROM tryout_sessions
  WHERE exam_track = 'tka'
  GROUP BY title
  HAVING count(*) > 1
)
AND NOT EXISTS (SELECT 1 FROM question_set_items qsi WHERE qsi.set_id = ts.question_set_id)
AND NOT EXISTS (SELECT 1 FROM package_content_items pci WHERE pci.content_id = ts.id)
AND NOT EXISTS (SELECT 1 FROM attempts a WHERE a.session_id = ts.id)
RETURNING id, title;
