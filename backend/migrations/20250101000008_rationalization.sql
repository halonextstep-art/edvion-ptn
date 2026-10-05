-- Rasionalisasi SNBT/SNBP — PTN (university) admission-chance recommendation engine.
--
-- `ptn_programs` is a real, published admission-statistics catalog (daya tampung, peminat,
-- dan passing grade per program studi for both the SNBP and SNBT admission tracks) provided
-- by the user as "Hitungan Rasionalisasi.xlsx" and imported once via
-- `cargo run --bin import_ptn_catalog -- backend/data/ptn_programs.csv`. Admin can maintain
-- the figures afterward via ordinary CRUD (same pattern as `packages`/`vouchers`), so the
-- catalog can be corrected/updated for future admission cycles rather than being a frozen
-- snapshot.
CREATE TABLE ptn_programs (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kode                BIGINT NOT NULL UNIQUE,
    nama_ptn            TEXT NOT NULL,
    nama_prodi          TEXT NOT NULL,
    provinsi            TEXT NOT NULL,
    kota                TEXT NOT NULL,
    singkatan           TEXT NOT NULL,
    rumpun              TEXT NOT NULL,
    mapel_syarat        TEXT NOT NULL DEFAULT '',
    daya_tampung_snbp   INT NOT NULL DEFAULT 0 CHECK (daya_tampung_snbp >= 0),
    peminat_snbp        INT NOT NULL DEFAULT 0 CHECK (peminat_snbp >= 0),
    daya_tampung_snbt   INT NOT NULL DEFAULT 0 CHECK (daya_tampung_snbt >= 0),
    peminat_snbt        INT NOT NULL DEFAULT 0 CHECK (peminat_snbt >= 0),
    pg_snbt             DOUBLE PRECISION NOT NULL DEFAULT 0,
    pg_snbp             DOUBLE PRECISION NOT NULL DEFAULT 0,
    jenjang             TEXT NOT NULL DEFAULT 'S1',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_ptn_programs_ptn ON ptn_programs(nama_ptn);
CREATE INDEX idx_ptn_programs_prodi ON ptn_programs(nama_prodi);
CREATE INDEX idx_ptn_programs_rumpun ON ptn_programs(rumpun);

-- Student's own real academic record (rapor), entered by the student themselves — this is
-- the only source for the SNBP side of the engine; nothing here is ever auto-filled.
CREATE TABLE student_rapor_scores (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    semester    INT NOT NULL CHECK (semester BETWEEN 1 AND 6),
    subject     TEXT NOT NULL,
    score       DOUBLE PRECISION NOT NULL CHECK (score >= 0 AND score <= 100),
    is_minat    BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (student_id, semester, subject)
);
CREATE INDEX idx_student_rapor_scores_student ON student_rapor_scores(student_id);

-- Student's chosen target programs (their personal watchlist for the rationalization tool).
-- `priority` is a student-set label (utama/cadangan/aman a la the reference), distinct from
-- the *computed* chance tier which the engine derives live at read time.
CREATE TABLE student_ptn_targets (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ptn_program_id  UUID NOT NULL REFERENCES ptn_programs(id) ON DELETE CASCADE,
    track           TEXT NOT NULL DEFAULT 'snbt',
    priority        TEXT NOT NULL DEFAULT 'utama',
    sort_order      INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (student_id, ptn_program_id, track)
);
CREATE INDEX idx_student_ptn_targets_student ON student_ptn_targets(student_id);
