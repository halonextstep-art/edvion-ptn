-- Set Soal: an admin/content-curated, ordered, fixed list of specific questions — an
-- alternative to TryoutSession's existing random-filter draw (random_approved), for when a
-- content team wants a guaranteed exact composition instead of trusting a random pool match.
-- A session with `question_set_id` set uses this fixed list (in order) at attempt-start
-- instead of drawing randomly; `question_set_id` is nullable so every existing session keeps
-- its current random-filter behavior completely unchanged (this feature is opt-in, additive).
CREATE TABLE question_sets (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_by  UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE question_set_items (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    set_id       UUID NOT NULL REFERENCES question_sets(id) ON DELETE CASCADE,
    question_id  UUID NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    sort_order   INT NOT NULL DEFAULT 0,
    added_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (set_id, question_id)
);
CREATE INDEX idx_question_set_items_set ON question_set_items(set_id, sort_order);

-- Nullable, ON DELETE SET NULL so deleting a Set Soal never deletes/breaks a session — it
-- just falls back to the session's original filter fields (which remain intact/untouched).
ALTER TABLE tryout_sessions ADD COLUMN question_set_id UUID REFERENCES question_sets(id) ON DELETE SET NULL;
