-- Singleton platform-wide settings row (always id = 1) — starts with just the B2C
-- (schoolless) self-registration on/off toggle, but the table is shaped to grow more
-- platform-wide flags later without a new table per flag.
CREATE TABLE platform_settings (
    id                          SMALLINT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    b2c_registration_enabled   BOOLEAN NOT NULL DEFAULT true,
    updated_by                  UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO platform_settings (id, b2c_registration_enabled) VALUES (1, true);
