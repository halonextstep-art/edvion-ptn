-- Real student-facing voucher redemption flow. Previously `vouchers.used_count` was a
-- genuine-but-always-zero column (no redemption flow existed to increment it — see
-- 20250101000006_vouchers.sql). This table is that flow: one row per (voucher, student)
-- redemption, enforced unique so the same student can't redeem the same code twice.
-- `used_count` on the voucher row is incremented in the same transaction as the insert
-- (see PostgresVoucherRepository::redeem), so it now reports a real, live figure.
CREATE TABLE voucher_redemptions (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    voucher_id   UUID NOT NULL REFERENCES vouchers(id) ON DELETE CASCADE,
    student_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    redeemed_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (voucher_id, student_id)
);
CREATE INDEX idx_voucher_redemptions_student ON voucher_redemptions(student_id);
