-- Another real bug surfaced by end-to-end testing, same root cause as
-- 20260101000008_ptn_catalog_rls.sql but in the SHARED `core.akun` table: a question's
-- `created_by`/`reviewed_by` (an Admin/Content account, `tenant_id = 0`) needs to be
-- resolved to a `public_id`/`full_name` when a Student or School actor reads that
-- question — but `core.akun`'s original policy only allows seeing accounts in your own
-- tenant or being `is_platform_admin` yourself. A Student could never see who wrote a
-- question, which account granted a school's entitlement, etc.
--
-- Fix: add the same `OR tenant_id = 0` carve-out used for `ptn.*` catalog tables. This is
-- additive and narrow — it only ever exposes platform/admin-tier accounts (tenant 0) to
-- every tenant; it does NOT let one school see another school's own students/staff
-- (those keep `tenant_id <> 0`, still fully isolated by the unchanged first clause).
-- Touches a table shared with academic/tahfiz/cms/konten, but is safe for all of them:
-- none of those modules have any accounts under tenant 0 today (that's PTN's reserved
-- platform tenant, see `TenantCtx::PLATFORM_TENANT_ID`), so this is a no-op for them
-- unless/until they adopt the same platform-admin convention.

DROP POLICY IF EXISTS tenant_isolation ON core.akun;
CREATE POLICY tenant_isolation ON core.akun
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (tenant_id = 0) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (tenant_id = 0) OR (current_setting('app.is_platform_admin'::text))::boolean);
