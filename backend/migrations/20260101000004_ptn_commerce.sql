-- Events, packages, vouchers, payments, school entitlements — the commerce/catalogue side.

CREATE TABLE ptn.events (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id           BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    name                TEXT NOT NULL,
    event_type          TEXT NOT NULL CHECK (event_type IN ('tryout', 'drilling', 'mini')),
    description         TEXT NOT NULL DEFAULT '',
    session_id          BIGINT NOT NULL REFERENCES ptn.tryout_sessions(id) ON DELETE RESTRICT,
    start_date          DATE NOT NULL,
    end_date            DATE,
    max_participants    INT,
    price               INT NOT NULL DEFAULT 0,
    status              TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'upcoming', 'ongoing', 'completed', 'archived')),
    prizes              TEXT NOT NULL DEFAULT '',
    target_class        TEXT NOT NULL DEFAULT 'Kelas 12',
    created_by          BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE RESTRICT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE ONLY ptn.events FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.events ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.events
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_events_tenant_id ON ptn.events(tenant_id);
CREATE INDEX idx_ptn_events_session ON ptn.events(session_id);
CREATE INDEX idx_ptn_events_status ON ptn.events(status);

CREATE TABLE ptn.packages (
    id                      BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id               BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    name                    TEXT NOT NULL,
    package_type            TEXT NOT NULL,
    original_price          INT NOT NULL CHECK (original_price >= 0),
    sale_price              INT NOT NULL CHECK (sale_price >= 0),
    validity                TEXT NOT NULL DEFAULT '1 tahun',
    features                TEXT[] NOT NULL DEFAULT '{}',
    badge                   TEXT NOT NULL DEFAULT '',
    emoji                   TEXT NOT NULL DEFAULT '📘',
    gradient                TEXT NOT NULL DEFAULT 'from-sky-200 to-blue-100',
    accent_color            TEXT NOT NULL DEFAULT '#3b82f6',
    active                  BOOLEAN NOT NULL DEFAULT true,
    sort_order              INT NOT NULL DEFAULT 0,
    icon_type               TEXT NOT NULL DEFAULT 'preset' CHECK (icon_type IN ('preset', 'custom')),
    icon_name               TEXT,
    icon_url                TEXT,
    banner_url              TEXT,
    elective_pick_count     INT NOT NULL DEFAULT 0,
    exam_track              TEXT NOT NULL DEFAULT 'snbt' CHECK (exam_track IN ('snbt', 'tka')),
    created_by              BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE RESTRICT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE ONLY ptn.packages FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.packages ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.packages
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_packages_tenant_id ON ptn.packages(tenant_id);
CREATE INDEX idx_ptn_packages_sort ON ptn.packages(sort_order);

CREATE TABLE ptn.package_content_items (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    package_id      BIGINT NOT NULL REFERENCES ptn.packages(id) ON DELETE CASCADE,
    content_type    TEXT NOT NULL CHECK (content_type IN ('tryout_session', 'simulation_template')),
    content_id      BIGINT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (package_id, content_type, content_id)
);
CREATE INDEX idx_ptn_package_content_items_package ON ptn.package_content_items(package_id);
CREATE INDEX idx_ptn_package_content_items_content ON ptn.package_content_items(content_type, content_id);

CREATE TABLE ptn.package_elective_choices (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    student_id      BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE CASCADE,
    package_id      BIGINT NOT NULL REFERENCES ptn.packages(id) ON DELETE CASCADE,
    session_id      BIGINT NOT NULL REFERENCES ptn.tryout_sessions(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (student_id, package_id, session_id)
);
CREATE INDEX idx_ptn_package_elective_choices_student_package ON ptn.package_elective_choices(student_id, package_id);

CREATE TABLE ptn.vouchers (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id           BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    code                TEXT NOT NULL UNIQUE,
    voucher_type        TEXT NOT NULL CHECK (voucher_type IN ('access', 'bulk_school', 'promo', 'referral')),
    package_id          BIGINT NOT NULL REFERENCES ptn.packages(id) ON DELETE RESTRICT,
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
    created_by          BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE RESTRICT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE ONLY ptn.vouchers FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.vouchers ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.vouchers
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_vouchers_tenant_id ON ptn.vouchers(tenant_id);
CREATE INDEX idx_ptn_vouchers_type ON ptn.vouchers(voucher_type);
CREATE INDEX idx_ptn_vouchers_package ON ptn.vouchers(package_id);

CREATE TABLE ptn.voucher_redemptions (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    voucher_id      BIGINT NOT NULL REFERENCES ptn.vouchers(id) ON DELETE CASCADE,
    student_id      BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE CASCADE,
    redeemed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (voucher_id, student_id)
);
CREATE INDEX idx_ptn_voucher_redemptions_student ON ptn.voucher_redemptions(student_id);

CREATE TABLE ptn.payment_transactions (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id       BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    package_id      BIGINT NOT NULL REFERENCES ptn.packages(id) ON DELETE RESTRICT,
    buyer_user_id   BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE CASCADE,
    amount          INT NOT NULL,
    currency        TEXT NOT NULL DEFAULT 'IDR',
    provider        TEXT,
    provider_ref    TEXT,
    status          TEXT NOT NULL DEFAULT 'pending',
    failure_reason  TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    paid_at         TIMESTAMPTZ
);
ALTER TABLE ONLY ptn.payment_transactions FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.payment_transactions ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.payment_transactions
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_payment_transactions_tenant_id ON ptn.payment_transactions(tenant_id);
CREATE INDEX idx_ptn_payment_transactions_buyer ON ptn.payment_transactions(buyer_user_id);
CREATE INDEX idx_ptn_payment_transactions_status ON ptn.payment_transactions(status);

CREATE TABLE ptn.school_package_entitlements (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    school_id       BIGINT NOT NULL REFERENCES core.tenant(id) ON DELETE CASCADE,
    package_id      BIGINT NOT NULL REFERENCES ptn.packages(id) ON DELETE RESTRICT,
    starts_at       DATE NOT NULL DEFAULT CURRENT_DATE,
    expires_at      DATE,
    note            TEXT,
    granted_by      BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE RESTRICT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_ptn_school_entitlements_school ON ptn.school_package_entitlements(school_id);
