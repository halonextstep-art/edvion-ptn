-- Package catalogue (Manajemen Paket) — admin-managed pricing/marketing content for the
-- landing page, mirroring the reference `AdminPackageManagement`. `discount` is never
-- stored (derived live from original_price/sale_price, see domain::package::compute_discount)
-- so it can never drift out of sync with the two prices. There is no `sold_count` column:
-- no purchase/checkout system exists in this codebase yet, so a real sales figure cannot be
-- computed — the API always reports 0 rather than storing/faking a number (same "honest
-- zero" pattern as School.monthly_revenue).

CREATE TABLE packages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    package_type    TEXT NOT NULL,
    original_price  INT NOT NULL CHECK (original_price >= 0),
    sale_price      INT NOT NULL CHECK (sale_price >= 0),
    validity        TEXT NOT NULL DEFAULT '1 tahun',
    features        TEXT[] NOT NULL DEFAULT '{}',
    badge           TEXT NOT NULL DEFAULT '',
    emoji           TEXT NOT NULL DEFAULT '📘',
    gradient        TEXT NOT NULL DEFAULT 'from-sky-200 to-blue-100',
    accent_color    TEXT NOT NULL DEFAULT '#3b82f6',
    active          BOOLEAN NOT NULL DEFAULT true,
    sort_order      INT NOT NULL DEFAULT 0,
    created_by      UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_packages_sort ON packages(sort_order);
