-- "Detail Rapor" opsional untuk satu entri "Kaka Kelas" (alumni benchmark): snapshot
-- nilai per mata pelajaran alumni tsb, dipakai untuk radar chart pencapaian siswa aktif
-- vs alumni nyata (fitur ini sengaja TIDAK dibangun sebelumnya karena datanya belum ada —
-- lihat komentar lama di SchoolRasionalisasi.vue). Beda dari `student_rapor_scores`:
-- TIDAK per-semester (alumni sudah lulus, ini snapshot nilai akhir/rata-rata rapor mereka,
-- bukan riwayat multi-semester), jadi cukup satu baris per (alumni, mapel).
CREATE TABLE alumni_rapor_scores (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alumni_id   UUID NOT NULL REFERENCES school_alumni_benchmarks(id) ON DELETE CASCADE,
    subject     TEXT NOT NULL,
    score       DOUBLE PRECISION NOT NULL CHECK (score >= 0 AND score <= 100),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (alumni_id, subject)
);
CREATE INDEX idx_alumni_rapor_scores_alumni ON alumni_rapor_scores(alumni_id);
