-- `ptn` schema — business data for this app, isolated from other modules sharing the same
-- database. Follows the same convention already proven by `tahfiz.*`: bigint PK + a
-- `tenant_id` column defaulted from the RLS session GUC + FORCE RLS + a `tenant_isolation`
-- policy. See migrations in this file and the ones that follow for the full table set.

CREATE SCHEMA IF NOT EXISTS ptn;

-- The app connects as `lms_app` (non-superuser, RLS-subject) — same role every other
-- module's tables grant to. Default privileges so tables created by LATER migrations in
-- this same schema (everything below, plus any future ones) are covered automatically.
GRANT USAGE ON SCHEMA ptn TO lms_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA ptn GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO lms_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA ptn GRANT USAGE, SELECT ON SEQUENCES TO lms_app;

CREATE FUNCTION ptn.trigger_set_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$;

-- `core.tenant` only has `name`/`slug` — PTN's "sekolah" concept carries extra business
-- columns (package tier, revenue share, onboarding status, contact info) that have no home
-- in the shared `core.tenant` row. This is the 1:1 extension, exactly like PTN's own
-- `ptn.student_profile` extends `core.akun` below.
CREATE TABLE ptn.school_profile (
    tenant_id           bigint PRIMARY KEY REFERENCES core.tenant(id) ON DELETE CASCADE,
    school_type         TEXT NOT NULL CHECK (school_type IN ('sma', 'smk', 'ma', 'smp')),
    city                TEXT NOT NULL,
    province            TEXT NOT NULL,
    email               TEXT NOT NULL,
    phone               TEXT NOT NULL,
    join_date           DATE NOT NULL DEFAULT CURRENT_DATE,
    package_type        TEXT NOT NULL CHECK (package_type IN ('basic', 'premium', 'enterprise')),
    contact_person      TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'pending')),
    revenue_share       INT NOT NULL DEFAULT 10 CHECK (revenue_share >= 0 AND revenue_share <= 50),
    monthly_revenue     BIGINT NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- No RLS on `school_profile`: it is keyed 1:1 by `tenant_id` (the PK itself), so a school
-- can only ever reference its own row by construction, and platform admin/content accounts
-- (tenant_id = 0) need to list every school's profile (admin "Manajemen Sekolah") — the same
-- shape of exception `core.tenant` itself has no RLS for.

-- 1:1 extension of `core.akun` where role = 'student' — nisn/grade/nis/rombel_code/etc. are
-- PTN-specific import fields (see old migration 20250101000038_student_import_fields.sql),
-- not shared with academic.siswa (different school-management module, different fields).
CREATE TABLE ptn.student_profile (
    akun_id         bigint PRIMARY KEY REFERENCES core.akun(id) ON DELETE CASCADE,
    tenant_id       bigint DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    grade           TEXT,
    class           TEXT,
    total_points    INT NOT NULL DEFAULT 0,
    streak          INT NOT NULL DEFAULT 0,
    nisn            TEXT,
    nis             TEXT,
    gender          TEXT,
    birth_date      DATE,
    enrolled_at     DATE,
    rombel_code     TEXT,
    username        TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE ONLY ptn.student_profile FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.student_profile ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.student_profile
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_student_profile_tenant_id ON ptn.student_profile(tenant_id);
