-- Fixes a real bug surfaced by end-to-end testing: Admin/Content accounts are
-- platform-scoped (`tenant_id = 0`, the reserved platform tenant — see
-- `TenantCtx::PLATFORM_TENANT_ID`), so every question/package/tryout session/etc. they
-- create lands under `tenant_id = 0`. But the original `tenant_isolation` policy
-- (`tenant_id = current_setting('app.tenant_id') OR is_platform_admin`) only lets a
-- School/Student actor see rows under their OWN tenant — never tenant 0's — so no
-- student anywhere could ever see the question bank, tryout catalog, or packages an
-- admin built for them. That's backwards: this platform's whole model is a central
-- admin/content team publishing a shared catalog that EVERY school's students consume.
--
-- Fix: catalog tables (built by Admin/Content, read by everyone) get `OR tenant_id = 0`
-- added to their policy, so platform-wide content is visible to every tenant. Tables
-- that hold one school/student's own private data (attempts, rapor, audit log, payments,
-- etc.) are NOT touched here — those must stay strictly tenant-scoped.

DO $$
DECLARE
    t text;
BEGIN
    FOREACH t IN ARRAY ARRAY[
        'questions', 'question_sets',
        'tryout_sessions', 'packages', 'vouchers',
        'simulation_templates',
        'point_rules', 'badges', 'challenges',
        'events'
    ]
    LOOP
        EXECUTE format('DROP POLICY IF EXISTS tenant_isolation ON ptn.%I', t);
        EXECUTE format(
            $pol$CREATE POLICY tenant_isolation ON ptn.%I
                USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (tenant_id = 0) OR (current_setting('app.is_platform_admin'::text))::boolean)
                WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (tenant_id = 0) OR (current_setting('app.is_platform_admin'::text))::boolean)$pol$,
            t
        );
    END LOOP;
END $$;

-- `ptn.admission_deadlines` was missed in the original migration — no `tenant_id`/RLS at
-- all. It's genuinely platform-wide (SNBP/SNBT/UTBK calendar, same shared-catalog shape
-- as the tables above), so give it the identical treatment rather than leaving it as the
-- one inconsistent table.
ALTER TABLE ptn.admission_deadlines ADD COLUMN IF NOT EXISTS tenant_id BIGINT DEFAULT (current_setting('app.tenant_id'::text, true))::bigint NOT NULL;
ALTER TABLE ONLY ptn.admission_deadlines FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.admission_deadlines ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON ptn.admission_deadlines;
CREATE POLICY tenant_isolation ON ptn.admission_deadlines
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (tenant_id = 0) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (tenant_id = 0) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX IF NOT EXISTS idx_ptn_admission_deadlines_tenant_id ON ptn.admission_deadlines(tenant_id);
