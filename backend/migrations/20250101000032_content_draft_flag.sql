-- Supports the new "Buat Paket + Subtes + Soal" wizard (dipakai Admin & akun Konten): sesi
-- Tryout/Drilling dan Simulasi UTBK yang masih dalam proses penyusunan (belum di-review/publish
-- oleh Admin) tidak boleh langsung muncul di katalog publik siswa — sebelum kolom ini, sebuah
-- TryoutSession/SimulationTemplate langsung terlihat platform-wide begitu dibuat, tidak peduli
-- soalnya sudah lengkap atau belum.
--
-- is_draft = true berarti "masih disusun, sembunyikan dari katalog siswa/sekolah". Default
-- false untuk semua baris yang sudah ada supaya tidak ada regresi pada konten yang sudah live.
-- PackageService::set_active(true) men-cascade set_draft(false) ke seluruh content item paket
-- itu — jadi "Publish Paket" adalah satu-satunya jalan resmi menerbitkan draft ke publik.
ALTER TABLE tryout_sessions ADD COLUMN is_draft BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE simulation_templates ADD COLUMN is_draft BOOLEAN NOT NULL DEFAULT false;
