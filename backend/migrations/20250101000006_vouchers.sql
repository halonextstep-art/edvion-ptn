-- Voucher catalogue (Manajemen Voucher) — access/bulk-school/promo/referral codes,
-- mirroring the reference `AdminVoucher`. `package_id` references our real `packages`
-- table (the reference hardcodes a separate plan list; we tie it to the actual catalogue
-- instead). `used_count` starts at 0 and stays there: no checkout/redemption flow exists
-- in this codebase yet, so there is no real event that would ever increment it — it is a
-- genuine column (not a fabricated response value) that will start reporting real numbers
-- the moment a redemption flow is built, same "honest zero" pattern as other modules.

CREATE TABLE vouchers (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code                TEXT NOT NULL UNIQUE,
    voucher_type        TEXT NOT NULL CHECK (voucher_type IN ('access', 'bulk_school', 'promo', 'referral')),
    package_id          UUID NOT NULL REFERENCES packages(id) ON DELETE RESTRICT,
    discount_type       TEXT NOT NULL CHECK (discount_type IN ('percent', 'fixed', 'full')),
    discount_value      INT NOT NULL DEFAULT 0,
    max_uses            INT NOT NULL DEFAULT 1,
    used_count          INT NOT NULL DEFAULT 0,
    active              BOOLEAN NOT NULL DEFAULT true,
    expires_at          DATE NOT NULL,
    note                TEXT NOT NULL DEFAULT '',
    school_name         TEXT,
    referrer_name       TEXT,
    referrer_commission INT,
    created_by          UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_vouchers_type ON vouchers(voucher_type);
CREATE INDEX idx_vouchers_package ON vouchers(package_id);
