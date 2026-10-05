-- Field tambahan untuk profil siswa, mengikuti format import Excel mitra sekolah (kolom:
-- nis, nisn, nama, jenis kelamin, tanggal lahir, tanggal masuk, kode rombel, nama rombel,
-- username, email, telepon, password). Beberapa kolom di file itu sudah punya rumah:
--   - "nisn"    -> students.nisn (sudah ada, lihat 20250101000003_user_profile_fields.sql)
--   - "nama"    -> users.name (sudah ada)
--   - "email"   -> users.email (sudah ada)
--   - "telepon" -> users.phone (sudah ada)
--   - "password"-> users.password_hash (sudah ada — dipakai langsung dari file, bukan
--                  di-generate otomatis lagi, lihat SchoolOnboardingWizard.vue)
--   - "nama rombel" -> students.grade (sudah ada, sebelumnya diisi manual field "Kelas")
-- Kolom baru di bawah ini menampung sisanya, yang sebelumnya tidak punya tempat sama sekali:

ALTER TABLE students ADD COLUMN nis TEXT;
ALTER TABLE students ADD COLUMN gender TEXT;
ALTER TABLE students ADD COLUMN birth_date DATE;
ALTER TABLE students ADD COLUMN enrolled_at DATE;
-- "kode rombel" disimpan terpisah dari "nama rombel" (-> grade) karena di beberapa sekolah
-- keduanya beda nilai (mis. kode "9A" vs nama tampilan "Kelas 9 A"), meski di file contoh
-- yang pertama dipakai kebetulan sama persis dengan nama rombel.
ALTER TABLE students ADD COLUMN rombel_code TEXT;
-- "username" di file import ini beda konsep dari NAMA (kadang sama persis, kadang tidak) —
-- login tetap pakai email seperti sekarang, kolom ini murni menyimpan nilai apa adanya dari
-- file untuk referensi/ekspor, tidak dipakai untuk autentikasi.
ALTER TABLE students ADD COLUMN username TEXT;
