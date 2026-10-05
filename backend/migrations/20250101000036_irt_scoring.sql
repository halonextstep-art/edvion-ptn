-- "Estimasi IRT" (Item Response Theory) sebagai skor KEDUA yang berdampingan dengan "Skor
-- Instan" (skor persentase sederhana yang sudah ada sejak awal, cuma sekarang diberi label
-- eksplisit) — lihat domain::irt doc comment untuk penjelasan lengkap model & keterbatasannya
-- (estimasi platform sendiri dari data historis platform ini, BUKAN replikasi skor UTBK resmi
-- SNPMB yang metode & datanya tidak dipublikasikan ke publik).
--
-- question_irt_params menyimpan hasil kalibrasi kesulitan tiap butir soal (parameter b, model
-- Rasch/1PL) dari data historis attempt_answers yang sudah pernah dikerjakan siswa di platform
-- ini. Tidak ada infrastruktur scheduler di backend ini, jadi kalibrasi TIDAK berjalan otomatis
-- — admin men-trigger kalibrasi ulang secara manual (lihat application::irt_service,
-- POST /api/settings/irt/recalibrate), sama seperti pola "manual trigger" lain yang sudah ada
-- di aplikasi ini (mis. bulk-approve soal per paket).
CREATE TABLE question_irt_params (
    question_id     UUID PRIMARY KEY REFERENCES questions(id) ON DELETE CASCADE,
    difficulty_b    DOUBLE PRECISION NOT NULL,
    sample_size     BIGINT NOT NULL,
    calibrated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Admin-configurable: skor mana yang ditampilkan ke siswa & laporan sekolah untuk attempt
-- UTBK/SNBT. 'instant' = hanya Skor Instan (default — perilaku persis seperti sebelum fitur
-- ini ada, sampai admin sadar sengaja mengaktifkan Estimasi IRT). 'irt' = hanya Estimasi IRT.
-- 'both' = tampilkan keduanya berdampingan. Tidak memengaruhi skema penilaian TKA (200-800 /
-- Istimewa 725), yang selalu dipakai apa adanya terlepas dari setting ini.
ALTER TABLE platform_settings ADD COLUMN score_display_mode TEXT NOT NULL DEFAULT 'instant'
    CHECK (score_display_mode IN ('instant', 'irt', 'both'));
