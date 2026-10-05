-- Real per-user notification feed. Starts with a single trigger (admin fulfills a manual
-- purchase request -> notify the buyer) but the shape is generic so more triggers can be
-- added later without a new table each time. `link_tab` is an optional tab-key string the
-- frontend already understands (each dashboard page has an `activeTab`/`activeView` ref and
-- an existing `@navigate="activeTab = $event"` convention) — clicking a notification can
-- jump straight to the relevant tab.
CREATE TABLE notifications (
    id          UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL,
    title       TEXT NOT NULL,
    message     TEXT NOT NULL,
    link_tab    TEXT,
    read_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_notifications_user_created ON notifications(user_id, created_at DESC);
