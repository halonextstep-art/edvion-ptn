-- ─── Kalender Deadline SNBP/SNBT/UTBK ──────────────────────────────────────────
-- Tanggal-tanggal resmi jalur seleksi (mis. "Pengumuman Kuota SNBP", "Penutupan
-- Pendaftaran SNBT", "Ujian UTBK Gelombang 1"), diinput manual oleh Admin (satu-satunya
-- sumber kebenaran resmi, dari SNPMB) dan dibaca oleh Admin/Sekolah/Siswa untuk
-- widget countdown "Deadline Mendatang". Tidak ada infrastruktur cron/notifikasi
-- terjadwal di codebase ini — countdown dihitung live dari deadline_date saat halaman
-- dibuka, bukan push notification.
CREATE TABLE admission_deadlines (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    track         TEXT NOT NULL CHECK (track IN ('snbp', 'snbt', 'utbk')),
    year          INT NOT NULL,
    label         TEXT NOT NULL,
    description   TEXT NOT NULL DEFAULT '',
    deadline_date DATE NOT NULL,
    created_by    UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_admission_deadlines_date ON admission_deadlines(deadline_date);
CREATE INDEX idx_admission_deadlines_track_year ON admission_deadlines(track, year);
