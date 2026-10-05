-- School-owned real-data inputs for the Rasionalisasi SNBP sub-tabs "Kaka Kelas"
-- (alumni benchmark) and "Eligible" (per-year SNBP-eligible student count). Both are
-- entered by the School PIC themselves (real, not fabricated) — the reference
-- prototype's mock arrays are replaced with genuine per-school records.

CREATE TABLE school_alumni_benchmarks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    school_id       UUID NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
    alumni_name     TEXT NOT NULL,
    graduation_year INT NOT NULL,
    track           TEXT NOT NULL DEFAULT 'snbp',
    nama_ptn        TEXT NOT NULL,
    nama_prodi      TEXT NOT NULL,
    benchmark_score DOUBLE PRECISION NOT NULL,
    created_by      UUID NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_school_alumni_school ON school_alumni_benchmarks(school_id);

CREATE TABLE school_snbp_eligibility (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    school_id      UUID NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
    year           INT NOT NULL,
    eligible_count INT NOT NULL DEFAULT 0 CHECK (eligible_count >= 0),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (school_id, year)
);
