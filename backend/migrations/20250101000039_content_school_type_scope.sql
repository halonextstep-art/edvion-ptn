-- Jenjang scoping untuk katalog konten siswa — sebelumnya TryoutSession/SimulationTemplate
-- hanya punya `exam_track` (snbt/tka), tidak ada cara membedakan "konten ini untuk siswa SMP"
-- vs "konten ini untuk siswa SMA/SMK/MA". Akibatnya siswa SMP bisa melihat konten SNBT/UTBK
-- (mis. "Tryout PTN 1") yang sebenarnya ditujukan untuk siswa SMA, tercampur dalam satu
-- katalog tanpa filter apapun.
--
-- `school_type_scope` NULL (default, untuk semua baris yang sudah ada) berarti "tampil ke
-- semua jenjang" — 100% backward compatible, tidak ada konten existing yang tiba-tiba
-- hilang dari katalog siapapun. Admin isi eksplisit lewat PackageWizard kalau suatu subtes/
-- simulasi memang khusus jenjang tertentu (mis. paket "TKA SMP").
--
-- Nama kolom sengaja `school_type_scope`, bukan `school_type` — untuk membedakan dari
-- `schools.school_type` (jenjang milik entitas Sekolah itu sendiri) dengan "jenjang yang
-- DITARGET oleh konten ini", supaya tidak tertukar saat dibaca ulang nanti.
ALTER TABLE tryout_sessions ADD COLUMN school_type_scope TEXT CHECK (school_type_scope IN ('sma', 'smk', 'ma', 'smp'));
ALTER TABLE simulation_templates ADD COLUMN school_type_scope TEXT CHECK (school_type_scope IN ('sma', 'smk', 'ma', 'smp'));
