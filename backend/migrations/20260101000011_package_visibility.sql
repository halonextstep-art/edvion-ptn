-- Package visibility: 'public' (default, shown in every student's/B2C's "Beli Paket"
-- catalog — the pre-existing behavior) vs 'restricted' (only shown to students whose school
-- has an active ptn.school_package_entitlements row for this package; never shown to B2C
-- students). See `application::package_service::PackageService::list_public` doc comment.
ALTER TABLE ptn.packages
    ADD COLUMN visibility TEXT NOT NULL DEFAULT 'public' CHECK (visibility IN ('public', 'restricted'));
