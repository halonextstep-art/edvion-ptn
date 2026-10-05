-- Payment gateway scaffolding — provider-agnostic transaction ledger. This system has no
-- payment gateway credentials configured yet (no Midtrans/Xendit account/API key exists),
-- so this migration and the code built on top of it are intentionally scaffolding only:
-- a real schema and a real "pending" transaction can be created, but no transaction can
-- ever be marked "paid" until a real `PaymentProvider` is wired in and its webhook
-- verifies a real payment. See `application::payment_service` for the honest
-- `NotConfiguredProvider` stub.

CREATE TABLE payment_transactions (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id       UUID NOT NULL REFERENCES packages(id) ON DELETE RESTRICT,
    buyer_user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount           INTEGER NOT NULL,
    currency         TEXT NOT NULL DEFAULT 'IDR',
    -- NULL until a real provider (e.g. "midtrans", "xendit") is configured and actually
    -- used to create this checkout — never fabricated.
    provider         TEXT,
    -- The provider's own order/transaction reference, filled in once a real checkout is
    -- created with a real provider. NULL for every transaction created today.
    provider_ref     TEXT,
    status           TEXT NOT NULL DEFAULT 'pending',
    failure_reason   TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    paid_at          TIMESTAMPTZ
);

CREATE INDEX idx_payment_transactions_buyer ON payment_transactions(buyer_user_id);
CREATE INDEX idx_payment_transactions_status ON payment_transactions(status);
