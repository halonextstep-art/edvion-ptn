-- Real roster metadata for a student's SNBP participation, entered by the School PIC —
-- powers the "Daftar Siswa" roster table (Konsultan/Status Penerimaan/Aktif). Pilihan 1/2
-- (target university+major) are deliberately NOT duplicated here — they're derived live
-- from the student's real `student_ptn_targets` (priority = utama/cadangan), the same
-- data the Rasionalisasi ranking already uses, so the roster never drifts out of sync
-- with the actual targets.

CREATE TABLE snbp_participation (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id  UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    school_id   UUID NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
    year        INT NOT NULL,
    konsultan   TEXT NOT NULL DEFAULT '',
    status      TEXT NOT NULL DEFAULT 'belum',
    aktif       BOOLEAN NOT NULL DEFAULT true,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_snbp_participation_school ON snbp_participation(school_id);
