-- Rasionalisasi SNBT/SNBP engine, student achievements/tracking, school benchmarks/reports,
-- audit log, notifications, taxonomy, institutions, admission deadlines, platform settings.

-- `institutions`/`ptn_programs` are a shared reference catalogue (official admission
-- statistics), not tenant-owned data — no tenant_id/RLS, same as `core.role`/`core.menu`.
CREATE TABLE ptn.institutions (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nama_ptn        TEXT NOT NULL UNIQUE,
    singkatan       TEXT NOT NULL,
    website         TEXT,
    alamat          TEXT,
    kota            TEXT,
    provinsi        TEXT,
    tahun_berdiri   INT,
    status          TEXT,
    akreditasi      TEXT,
    logo_url        TEXT,
    sumber          TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_ptn_institutions_nama_ptn ON ptn.institutions(nama_ptn);

CREATE TABLE ptn.ptn_programs (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
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
    institution_id      BIGINT REFERENCES ptn.institutions(id),
    has_official_stats  BOOLEAN NOT NULL DEFAULT true,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_ptn_ptn_programs_ptn ON ptn.ptn_programs(nama_ptn);
CREATE INDEX idx_ptn_ptn_programs_prodi ON ptn.ptn_programs(nama_prodi);
CREATE INDEX idx_ptn_ptn_programs_rumpun ON ptn.ptn_programs(rumpun);
CREATE INDEX idx_ptn_ptn_programs_institution_id ON ptn.ptn_programs(institution_id);

CREATE TABLE ptn.student_rapor_scores (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    student_id  BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE CASCADE,
    semester    INT NOT NULL CHECK (semester BETWEEN 1 AND 6),
    subject     TEXT NOT NULL,
    score       DOUBLE PRECISION NOT NULL CHECK (score >= 0 AND score <= 100),
    is_minat    BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (student_id, semester, subject)
);
CREATE INDEX idx_ptn_student_rapor_scores_student ON ptn.student_rapor_scores(student_id);

CREATE TABLE ptn.student_ptn_targets (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    student_id      BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE CASCADE,
    ptn_program_id  BIGINT NOT NULL REFERENCES ptn.ptn_programs(id) ON DELETE CASCADE,
    track           TEXT NOT NULL DEFAULT 'snbt',
    priority        TEXT NOT NULL DEFAULT 'utama',
    sort_order      INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (student_id, ptn_program_id, track)
);
CREATE INDEX idx_ptn_student_ptn_targets_student ON ptn.student_ptn_targets(student_id);

CREATE TABLE ptn.student_achievements (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    student_id      BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE CASCADE,
    nama            TEXT NOT NULL,
    tingkat         TEXT NOT NULL,
    tahun           INT NOT NULL,
    juara           TEXT NOT NULL DEFAULT '',
    certificate_url TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_ptn_student_achievements_student ON ptn.student_achievements(student_id);

CREATE TABLE ptn.student_snbt_tracking (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    student_id      BIGINT NOT NULL UNIQUE REFERENCES core.akun(id) ON DELETE CASCADE,
    actual_score    DOUBLE PRECISION,
    exam_date       DATE,
    status          TEXT NOT NULL DEFAULT 'terdaftar',
    notes           TEXT NOT NULL DEFAULT '',
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE ptn.snbp_participation (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    student_id  BIGINT NOT NULL UNIQUE REFERENCES core.akun(id) ON DELETE CASCADE,
    school_id   BIGINT NOT NULL REFERENCES core.tenant(id) ON DELETE CASCADE,
    year        INT NOT NULL,
    konsultan   TEXT NOT NULL DEFAULT '',
    status      TEXT NOT NULL DEFAULT 'belum',
    aktif       BOOLEAN NOT NULL DEFAULT true,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_ptn_snbp_participation_school ON ptn.snbp_participation(school_id);

CREATE TABLE ptn.school_alumni_benchmarks (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    school_id           BIGINT NOT NULL REFERENCES core.tenant(id) ON DELETE CASCADE,
    alumni_name         TEXT NOT NULL,
    graduation_year     INT NOT NULL,
    track               TEXT NOT NULL DEFAULT 'snbp',
    nama_ptn            TEXT NOT NULL,
    nama_prodi          TEXT NOT NULL,
    benchmark_score     DOUBLE PRECISION NOT NULL,
    created_by          BIGINT REFERENCES core.akun(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_ptn_school_alumni_school ON ptn.school_alumni_benchmarks(school_id);

CREATE TABLE ptn.alumni_rapor_scores (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    alumni_id   BIGINT NOT NULL REFERENCES ptn.school_alumni_benchmarks(id) ON DELETE CASCADE,
    subject     TEXT NOT NULL,
    score       DOUBLE PRECISION NOT NULL CHECK (score >= 0 AND score <= 100),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (alumni_id, subject)
);
CREATE INDEX idx_ptn_alumni_rapor_scores_alumni ON ptn.alumni_rapor_scores(alumni_id);

CREATE TABLE ptn.school_snbp_eligibility (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    school_id       BIGINT NOT NULL REFERENCES core.tenant(id) ON DELETE CASCADE,
    year            INT NOT NULL,
    eligible_count  INT NOT NULL DEFAULT 0 CHECK (eligible_count >= 0),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (school_id, year)
);

CREATE TABLE ptn.school_reports (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    school_id           BIGINT NOT NULL REFERENCES core.tenant(id) ON DELETE CASCADE,
    report_type         TEXT NOT NULL,
    title               TEXT NOT NULL,
    period_label        TEXT NOT NULL,
    class_filter        TEXT,
    exam_track_filter   TEXT,
    payload             JSONB NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_ptn_school_reports_school ON ptn.school_reports(school_id, created_at DESC);

CREATE TABLE ptn.audit_log (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id       BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    actor_id        BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE CASCADE,
    actor_name      TEXT NOT NULL,
    actor_role      TEXT NOT NULL,
    student_id      BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE CASCADE,
    action          TEXT NOT NULL,
    entity_type     TEXT NOT NULL,
    summary         TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE ONLY ptn.audit_log FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.audit_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.audit_log
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_audit_log_tenant_id ON ptn.audit_log(tenant_id);
CREATE INDEX idx_ptn_audit_log_student ON ptn.audit_log(student_id, created_at DESC);

CREATE TABLE ptn.notifications (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE CASCADE,
    kind            TEXT NOT NULL,
    title           TEXT NOT NULL,
    message         TEXT NOT NULL,
    link_tab        TEXT,
    read_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_ptn_notifications_user_created ON ptn.notifications(user_id, created_at DESC);

CREATE TABLE ptn.subject_categories (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    description     TEXT NOT NULL DEFAULT '',
    sort_order      INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE ptn.subjects (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    category_id     BIGINT NOT NULL REFERENCES ptn.subject_categories(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    code            TEXT NOT NULL DEFAULT '',
    sort_order      INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (category_id, name)
);
CREATE INDEX idx_ptn_subjects_category ON ptn.subjects(category_id);
CREATE INDEX idx_ptn_subjects_sort ON ptn.subjects(sort_order);

CREATE TABLE ptn.admission_deadlines (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    track           TEXT NOT NULL CHECK (track IN ('snbp', 'snbt', 'utbk')),
    year            INT NOT NULL,
    label           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    deadline_date   DATE NOT NULL,
    created_by      BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE RESTRICT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_ptn_admission_deadlines_date ON ptn.admission_deadlines(deadline_date);
CREATE INDEX idx_ptn_admission_deadlines_track_year ON ptn.admission_deadlines(track, year);

-- Singleton (same pattern as old `public.platform_settings`, `id` pinned to 1) — this is
-- genuinely global PTN-admin configuration, not per-school, so it carries `tenant_id = 0`
-- (the platform tenant) rather than a real per-tenant row.
CREATE TABLE ptn.platform_settings (
    id                              SMALLINT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    tenant_id                       BIGINT NOT NULL DEFAULT 0,
    b2c_registration_enabled        BOOLEAN NOT NULL DEFAULT true,
    score_display_mode              TEXT NOT NULL DEFAULT 'instant' CHECK (score_display_mode IN ('instant', 'irt', 'both')),
    simulation_lockdown_default     BOOLEAN NOT NULL DEFAULT false,
    updated_by                      BIGINT REFERENCES core.akun(id) ON DELETE SET NULL,
    updated_at                      TIMESTAMPTZ NOT NULL DEFAULT now()
);
INSERT INTO ptn.platform_settings (id) VALUES (1) ON CONFLICT (id) DO NOTHING;

-- `ALTER DEFAULT PRIVILEGES` (set in 20260101000002) only covers tables created AFTER it
-- ran — grant explicitly here too so every table from migrations 3-6 (created earlier in
-- this same migration run, before any default-privilege rule existed for them) is covered.
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA ptn TO lms_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA ptn TO lms_app;
