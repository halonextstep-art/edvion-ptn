-- ─── Events (Manajemen Event) ───────────────────────────────────────────────────
-- An event schedules an existing tryout_session ("paket soal") over a date range with
-- capacity/pricing/lifecycle metadata. Duration is intentionally NOT duplicated here —
-- it's read from the referenced session. Participant counts are also never stored;
-- they're derived live from `attempts` at query time (see PostgresEventRepository).
CREATE TABLE events (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                TEXT NOT NULL,
    event_type          TEXT NOT NULL CHECK (event_type IN ('tryout', 'drilling', 'mini')),
    description         TEXT NOT NULL DEFAULT '',
    session_id          UUID NOT NULL REFERENCES tryout_sessions(id) ON DELETE RESTRICT,
    start_date          DATE NOT NULL,
    end_date            DATE,
    max_participants    INT,
    price               INT NOT NULL DEFAULT 0,
    status              TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'upcoming', 'ongoing', 'completed', 'archived')),
    prizes              TEXT NOT NULL DEFAULT '',
    target_class        TEXT NOT NULL DEFAULT 'Kelas 12',
    created_by          UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_events_session ON events(session_id);
CREATE INDEX idx_events_status ON events(status);
