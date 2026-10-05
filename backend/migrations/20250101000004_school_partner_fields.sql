-- School/Partner enrichment for the onboarding + revenue-share workflow.
--
-- revenue_share: percentage (0-50) of the school's contracted plan value that goes back
-- to the school — an admin-set business term, not a computed metric.
-- monthly_revenue: the school's contracted plan value in Rupiah, admin-recorded (there is
-- no billing/invoicing system yet, so this is manually maintained rather than fabricated).
-- 'pending' status lets a newly-added partner sit unapproved until the onboarding wizard
-- (CSV student import) is completed — mirrors the same pattern as user accounts.
ALTER TABLE schools ADD COLUMN revenue_share INT NOT NULL DEFAULT 10 CHECK (revenue_share >= 0 AND revenue_share <= 50);
ALTER TABLE schools ADD COLUMN monthly_revenue BIGINT NOT NULL DEFAULT 0;

ALTER TABLE schools DROP CONSTRAINT schools_status_check;
ALTER TABLE schools ADD CONSTRAINT schools_status_check CHECK (status IN ('active', 'inactive', 'pending'));
