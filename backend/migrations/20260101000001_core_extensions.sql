-- Additive extensions to the SHARED `core` schema (also used by academic/tahfiz/cms/konten).
-- Every statement here must be safe to run against a `core` that already has other
-- tenants/roles/data from those other modules — nothing here may assume a fresh DB.

-- `core.tenant` only has a bigint `id` internally; PTN's wire format (and the rest of this
-- app's domain model) represents every entity id as a UUID string, so a tenant needs a
-- public, UUID identity too — exactly like `core.akun.public_id` already does for users.
-- Additive/backward compatible: other modules keep using `core.tenant.id` internally and
-- are unaffected.
ALTER TABLE core.tenant ADD COLUMN IF NOT EXISTS public_id uuid DEFAULT gen_random_uuid() NOT NULL;
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'tenant_public_id_key'
    ) THEN
        ALTER TABLE core.tenant ADD CONSTRAINT tenant_public_id_key UNIQUE (public_id);
    END IF;
END $$;

-- `core.akun` only has a boolean `is_active` — PTN needs a real 3-state lifecycle
-- (active/inactive/pending) because `pending` actually gates login (see
-- `AuthService::login`), not just a display label; and `avatar_url`/`minat_jurusan` are
-- PTN-only profile fields with no equivalent anywhere else in `core.akun`. Additive, other
-- modules simply never read/write these three columns.
ALTER TABLE core.akun ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'pending'));
ALTER TABLE core.akun ADD COLUMN IF NOT EXISTS avatar_url TEXT;
ALTER TABLE core.akun ADD COLUMN IF NOT EXISTS minat_jurusan TEXT;

-- PTN has two account roles that don't exist in any other module: `school` (sekolah
-- mitra/PIC) and `content` (tim penulis soal pusat). `admin` and `student` reuse the
-- existing ADMIN/SISWA rows core.role already has (seeded by other modules).
INSERT INTO core.role (kode, name, description)
VALUES
    ('SEKOLAH', 'Sekolah', 'PTN partner school account'),
    ('KONTEN', 'Konten', 'PTN central question-writer account')
ON CONFLICT (kode) DO NOTHING;

-- `core.akun`/`core.tenant` are RLS-protected (`FORCE ROW LEVEL SECURITY`), and PTN login
-- is email+password only (no "pick your school" step) — so the very first step of login
-- (find which account/tenant this email belongs to) cannot have `app.tenant_id` set yet,
-- since that's exactly what we're trying to discover. A narrow BYPASSRLS role, usable only
-- for that one lookup query (never for writes), solves this without widening RLS policy
-- itself. See AuthService::login / AUTH_LOOKUP_DATABASE_URL.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ptn_auth_lookup') THEN
        CREATE ROLE ptn_auth_lookup LOGIN PASSWORD 'ptn_auth_lookup_dev_only' BYPASSRLS;
    END IF;
END $$;

GRANT USAGE ON SCHEMA core TO ptn_auth_lookup;
GRANT SELECT ON core.akun, core.role, core.tenant TO ptn_auth_lookup;

-- `lms_app` (the app's normal connection role) needs ordinary CRUD on these three —
-- RLS (`tenant_isolation` on `core.akun`) still applies and does the real scoping; this
-- is just the baseline table-level grant without which even a correctly-scoped query
-- would be rejected outright. Not present already in this dev DB, unlike `core.platform_admin`/
-- `core.audit_log`/`core.menu*` which already had it.
GRANT SELECT, INSERT, UPDATE, DELETE ON core.akun, core.role, core.tenant TO lms_app;
GRANT USAGE, SELECT ON core.akun_id_seq, core.tenant_id_seq, core.role_id_seq TO lms_app;

-- `lms_app` (the app's main connection role) needs `CREATE` on `public` because sqlx's
-- own migration-tracking table (`public._sqlx_migrations`) lives there, not in `ptn` —
-- that's sqlx's hardcoded convention, not something this app controls.
GRANT USAGE, CREATE ON SCHEMA public TO lms_app;
