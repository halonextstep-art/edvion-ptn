-- School Package Entitlements — lets a platform Admin attach an existing pricing Package to
-- a School so ALL of that school's students automatically get premium (`is_premium`) tryout
-- access, without granting individual vouchers one-by-one (impractical for schools with
-- 150-9999 students, see PACKAGE_QUOTA in frontend/types/index.ts). Checked live by
-- school_id at access-check time (see AccessService::has_premium_access), not baked into
-- each student row, so a student changing schools or an entitlement expiring is reflected
-- immediately with no batch job needed. This is a NEW feature — previously `packages` had
-- zero functional effect on content access anywhere in this codebase.
CREATE TABLE school_package_entitlements (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    school_id     UUID NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
    package_id    UUID NOT NULL REFERENCES packages(id) ON DELETE RESTRICT,
    starts_at     DATE NOT NULL DEFAULT CURRENT_DATE,
    expires_at    DATE,
    note          TEXT,
    granted_by    UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_school_entitlements_school ON school_package_entitlements(school_id);
