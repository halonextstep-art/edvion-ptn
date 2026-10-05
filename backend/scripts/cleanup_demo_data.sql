-- ============================================================================
-- cleanup_demo_data.sql
--
-- Menghapus SEMUA data demo yang dibuat oleh `cargo run --bin seed`
-- (backend/src/bin/seed.rs) dari database, supaya aplikasi siap dipakai oleh
-- user sungguhan. Cocokkan baris ini terhadap nilai PERSIS yang ditulis di
-- seed.rs (nama, email, judul) — bukan "hapus semua", supaya data real yang
-- mungkin sudah dibuat lewat aplikasi (bukan oleh seed) tidak ikut kehapus.
--
-- YANG DIHAPUS:
--   - 12 akun siswa demo, 4 akun PIC sekolah demo, 2 akun tim konten demo
--   - 4 sekolah demo (+ data alumni/eligible/SNBP roster yang menempel)
--   - Seluruh attempt/jawaban/rapor/prestasi/target PTN milik siswa demo di atas
--   - 29 soal demo (24 approved + 5 contoh antrean review)
--   - 7 sesi tryout/drilling demo, 6 event demo, 5 paket demo, voucher demo
--
-- YANG SENGAJA TIDAK DIHAPUS (baca sebelum menjalankan):
--   - Akun admin demo (admin@edvionptn.id) — TIDAK dihapus. Banyak data lain
--     (kategori/mata uji, point rules, badge, challenge) mencatat akun ini
--     sebagai pembuat (created_by, FK RESTRICT) — menghapusnya akan gagal atau
--     berisiko mengunci Anda dari akses Admin sebelum akun pengganti dibuat.
--     ACTION WAJIB: login sekali dengan akun ini, lalu segera ganti password
--     dan nama tampilan lewat halaman profil (password "admin123" ini sudah
--     tidak rahasia lagi setelah ada di riwayat percakapan/kode ini).
--   - Kategori/mata uji (SNBT, TKA-IPA, TKA-IPS, AKM) — ini struktur rujukan
--     resmi, bukan data contoh, tetap dipakai apa adanya.
--   - Point rules, badge, challenge gamifikasi — ini konfigurasi fitur nyata
--     (bukan data dummy), silakan diedit/dihapus manual lewat Admin > Gamifikasi
--     kalau memang tidak dipakai.
--
-- CARA PAKAI:
--   1. BACKUP dulu: pg_dump "$DATABASE_URL" > backup-sebelum-cleanup.sql
--   2. Jalankan: psql "$DATABASE_URL" -f backend/scripts/cleanup_demo_data.sql
--   3. Skrip ini jalan di dalam SATU transaksi (BEGIN...COMMIT) — kalau salah
--      satu langkah gagal (misalnya karena data demo ini ternyata sudah
--      dipakai oleh sesuatu yang nyata, lihat catatan RESTRICT di bawah),
--      seluruh transaksi otomatis batal, tidak ada perubahan setengah jalan.
--   4. Mau coba dulu tanpa benar-benar menghapus? Ganti baris `COMMIT;` di
--      paling bawah jadi `ROLLBACK;`, jalankan, lihat pesan "DELETE n" di
--      setiap langkah, baru ganti balik ke COMMIT kalau sudah yakin.
--
-- AMAN DIJALANKAN BERULANG (idempotent) — kalau datanya sudah tidak ada,
-- setiap DELETE cukup menghapus 0 baris, tidak error.
-- ============================================================================

BEGIN;

-- ── 1. Sekolah demo — cascade otomatis: alumni benchmark, data eligible SNBP,
--       dan roster partisipasi SNBP milik sekolah ini. Akun siswa/PIC yang
--       masih mereferensikan sekolah ini (dihapus di langkah berikutnya) hanya
--       di-SET NULL, bukan diblokir. ──────────────────────────────────────
DELETE FROM schools WHERE name IN (
  'SMA Negeri 1 Jakarta',
  'SMA Negeri 3 Surabaya',
  'MA Negeri 1 Yogyakarta',
  'SMK Negeri 2 Bandung'
);

-- ── 2. Akun siswa demo — cascade otomatis: seluruh attempt (+ jawaban),
--       rapor, prestasi, target PTN, tracking SNBT milik mereka. ──────────
DELETE FROM users WHERE email IN (
  'siswa@student.com',
  'bunga@sman1.sch.id',
  'andi@sman1.sch.id',
  'hana@sman1.sch.id',
  'rizki@sman3sby.sch.id',
  'dewi@sman3sby.sch.id',
  'fajar@sman3sby.sch.id',
  'siti@man1yk.sch.id',
  'bagas@man1yk.sch.id',
  'putri@man1yk.sch.id',
  'farhan@smkn2bdg.sch.id',
  'yusuf@smkn2bdg.sch.id'
);

-- ── 3. Soal demo — harus setelah langkah 2 (attempt milik siswa demo yang
--       menjawab soal-soal ini sudah hilang, jadi tidak lagi diblokir FK
--       RESTRICT dari attempt_questions/attempt_answers). ─────────────────
DELETE FROM questions WHERE question_text IN (
  'Jika pendapatan pada bulan ke-t dinyatakan dengan P(t) = -2t^2 + 12t + 10 (juta rupiah), pada bulan ke berapa pendapatan maksimum tercapai?',
  'Sebuah tabung memiliki jari-jari 7 cm dan tinggi 10 cm. Berapa volume tabung tersebut? (gunakan pi = 22/7)',
  'Nilai dari sin(75 derajat) adalah...',
  'Berapakah jarak mendatar maksimum yang dicapai bola tersebut?',
  'Dua resistor 6 ohm dan 3 ohm disusun paralel, kemudian diseri dengan resistor 4 ohm. Berapa hambatan total rangkaian?',
  'Suatu gas menyerap kalor 500 J dan melakukan usaha 200 J terhadap lingkungan. Berapa perubahan energi dalam gas tersebut?',
  'Berapa gram NaCl (Mr = 58,5) yang diperlukan untuk membuat 500 mL larutan NaCl 0,2 M? (jawab angka saja)',
  'Berapa pH larutan HCl 0,001 M?',
  'Reaksi pembentukan air melepaskan 286 kJ/mol. Jika 2 mol air terbentuk, berapa total kalor yang dilepaskan?',
  'Berapa persentase keturunan F2 yang berbunga merah jika merah dominan terhadap putih?',
  'Dalam rantai makanan padi -> tikus -> ular -> elang, tikus berperan sebagai...',
  'Organel sel yang berfungsi sebagai penghasil energi (ATP) utama pada sel eukariotik adalah...',
  'Ide pokok paragraf tersebut adalah...',
  'Penulisan kata baku yang tepat di bawah ini adalah...',
  'Struktur teks argumentasi yang tepat secara berurutan adalah...',
  'What is the main topic of the passage?',
  'She ___ to the library every Saturday.',
  'The word ''abundant'' is closest in meaning to...',
  'Semua siswa rajin lulus ujian. Budi lulus ujian. Kesimpulan yang tepat adalah...',
  'DOKTER : RUMAH SAKIT = GURU : ...',
  'Jika hari hujan maka jalan basah. Jalan tidak basah. Kesimpulan yang sah adalah...',
  'Rata-rata nilai 8 siswa adalah 75. Jika ditambah 2 siswa baru, rata-rata menjadi 73. Berapa jumlah nilai 2 siswa baru?',
  'Sebuah dadu dilempar sekali. Berapa peluang muncul mata dadu genap atau lebih dari 4?',
  'Suku ke-10 dari deret aritmatika 3, 7, 11, 15, ... adalah...',
  'Jelaskan pengertian rantai makanan dalam suatu ekosistem.',
  'Sebutkan bunyi hukum pemantulan cahaya.',
  'Jelaskan perbedaan sel volta dan sel elektrolisis.',
  'Berapa banyak cara menyusun 3 orang dari 5 orang dalam satu baris?',
  'Deskripsikan pola pada deret gambar berikutnya (draft, belum diajukan).'
);

-- ── 4. Akun tim konten demo — harus setelah langkah 3 (soal yang mereka
--       tulis, kolom created_by, sudah dihapus di atas). ──────────────────
DELETE FROM users WHERE email IN (
  'konten@edvion.id',
  'rina.konten@edvion.id'
);

-- ── 5. Akun PIC sekolah demo — aman sekarang karena data alumni benchmark
--       yang mereka input (created_by) sudah hilang lewat cascade sekolah
--       di langkah 1. ──────────────────────────────────────────────────
DELETE FROM users WHERE email IN (
  'sekolah@sman1.sch.id',
  'pic@sman3sby.sch.id',
  'pic@man1yk.sch.id',
  'pic@smkn2bdg.sch.id'
);

-- ── 6. Event demo — harus SEBELUM sesi (event.session_id ON DELETE RESTRICT).
DELETE FROM events WHERE name IN (
  'Tryout Nasional SNBT #45',
  'Drilling Matematika Intensif',
  'Mini Tryout Weekend #12',
  'Tryout PTN Premium #44',
  'Drilling Biologi Genetika & Ekologi',
  'Mini Tryout Bahasa Edisi Lama'
);

-- ── 7. Voucher demo — harus SEBELUM paket (voucher.package_id ON DELETE
--       RESTRICT). Satu "note" mewakili banyak kode voucher sekaligus
--       (quantity > 1 saat di-generate), DELETE ini menghapus semuanya. ──
DELETE FROM vouchers WHERE note IN (
  'Campaign promo bulanan',
  'Program afiliasi',
  'Batch voucher sekolah mitra'
);

-- ── 8. Sesi tryout/drilling demo — harus setelah event (langkah 6). ──────
-- CATATAN: kalau langkah ini GAGAL dengan error FK dari simulation_template_
-- slots/simulation_run_slots, artinya sesi ini sudah dipakai di dalam sebuah
-- Simulasi UTBK/TKA nyata yang dibuat setelah seeding — cek dulu secara
-- manual lewat Admin > Simulasi sebelum memutuskan menghapusnya.
DELETE FROM tryout_sessions WHERE title IN (
  'Tryout Nasional SNBT #45',
  'Latihan Harian Adaptif',
  'Mini Tryout SNBT',
  'Drilling Matematika - Level Sedang',
  'Drilling Biologi - Genetika & Ekologi',
  'Drilling Bahasa Indonesia - Pemahaman Teks',
  'Drilling Bahasa Inggris - Reading & Grammar'
);

-- ── 9. Paket demo — harus setelah voucher (langkah 7). ───────────────────
-- CATATAN: kalau langkah ini GAGAL dengan error FK dari
-- school_package_entitlements, artinya paket ini sudah di-assign nyata ke
-- sebuah sekolah mitra (Admin > School Entitlements) — cek dulu sebelum
-- memutuskan menghapusnya.
DELETE FROM packages WHERE name IN (
  '5 Kuota Tryout SNBT',
  '12 Kuota Tryout SNBT & TPS',
  '20 Kuota Tryout SNBT Full',
  'Akses Drilling Intensif',
  'Paket Lengkap Elite'
);

-- Ganti ke ROLLBACK dulu kalau ingin dry-run (lihat instruksi di atas).
COMMIT;
