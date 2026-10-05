-- Per-package content scoping — replaces the previous all-or-nothing access model where owning
-- ANY package (via voucher redemption or school entitlement) unlocked EVERY is_premium
-- TryoutSession platform-wide, with SimulationTemplate (Simulasi UTBK / "TO") having NO gating
-- at all. From here on, a Package explicitly lists which TryoutSessions and/or
-- SimulationTemplates it grants access to — see AccessService rework in the application layer.
--
-- A single polymorphic content_type column keeps this to ONE join table instead of two
-- near-identical ones. content_id intentionally has no FK (Postgres can't FK one column to two
-- different tables) — the application layer (PackageService) validates the referenced row
-- exists in the right table before inserting.
CREATE TABLE package_content_items (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id   UUID NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    content_type TEXT NOT NULL CHECK (content_type IN ('tryout_session', 'simulation_template')),
    content_id   UUID NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (package_id, content_type, content_id)
);
CREATE INDEX idx_package_content_items_package ON package_content_items(package_id);
CREATE INDEX idx_package_content_items_content ON package_content_items(content_type, content_id);

-- Simulasi UTBK (TO) had ZERO access gating until now (simulation_service.rs::start_run never
-- consulted AccessService — any logged-in student could start any active template for free).
-- Defaulting true here treats a template as premium content just like a TryoutSession, unlockable
-- only via a Package that includes it (see backfill below so nobody who already paid loses
-- access the moment this ships).
ALTER TABLE simulation_templates ADD COLUMN is_premium BOOLEAN NOT NULL DEFAULT true;

-- Backfill so no existing buyer/school loses access the moment this ships: every Package that
-- already exists today is retroactively assigned every premium TryoutSession and every
-- SimulationTemplate that already exists today. This is a one-time snapshot taken at migration
-- time, not an ongoing rule — content created after this migration must be explicitly assigned
-- to a Package by an admin (see PackageService::add_content_item).
INSERT INTO package_content_items (package_id, content_type, content_id)
SELECT p.id, 'tryout_session', ts.id
FROM packages p
CROSS JOIN tryout_sessions ts
WHERE ts.is_premium = true
ON CONFLICT DO NOTHING;

INSERT INTO package_content_items (package_id, content_type, content_id)
SELECT p.id, 'simulation_template', st.id
FROM packages p
CROSS JOIN simulation_templates st
ON CONFLICT DO NOTHING;
