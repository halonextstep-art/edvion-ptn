-- Tambah 'smp' sebagai jenjang sekolah yang valid — sebelumnya schools.school_type cuma
-- menerima sma/smk/ma (lihat 20250101000001_init.sql), padahal katalog paket sekarang sudah
-- ada jalur "TKA SMP" (lihat 20250101000020_subject_taxonomy.sql & materializeSubtests() di
-- PackageWizard.vue) — sekolah mitra tingkat SMP perlu bisa didaftarkan dengan jenjang yang
-- benar, bukan dipaksa pilih sma/smk/ma yang salah.
--
-- Constraint lama dibuat inline tanpa nama eksplisit, jadi Postgres menamainya otomatis
-- mengikuti pola default <table>_<column>_check.
ALTER TABLE schools DROP CONSTRAINT schools_school_type_check;
ALTER TABLE schools ADD CONSTRAINT schools_school_type_check
    CHECK (school_type IN ('sma', 'smk', 'ma', 'smp'));
