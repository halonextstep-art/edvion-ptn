-- User status/profile enrichment.
--
-- status: lifecycle for any account (active/inactive/pending). `pending` is meaningful for
-- school accounts awaiting the onboarding wizard, and for any account an admin wants to
-- park before activating. Login is gated on this (see AuthService::login) so it's a real
-- business rule, not a cosmetic label.
-- last_login: set by AuthService on every successful login — never backfilled or faked.
-- phone: contact number, mainly used for school/admin accounts.
-- students.nisn: student national ID number, shown alongside the existing `grade` column.
ALTER TABLE users ADD COLUMN status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'pending'));
ALTER TABLE users ADD COLUMN last_login TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN phone TEXT;

ALTER TABLE students ADD COLUMN nisn TEXT;

CREATE INDEX idx_users_status ON users(status);
