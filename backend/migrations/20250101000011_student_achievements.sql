-- Real per-student achievement (lomba/kompetisi) entries, self-reported by the student.
-- Feeds `domain::rationalization::index_prestasi` / `compute_chance_snbp` so the SNBP
-- estimate's "Index Prestasi" component is computed from genuine student input instead of
-- being fabricated or omitted.

CREATE TABLE student_achievements (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    nama       TEXT NOT NULL,
    tingkat    TEXT NOT NULL,
    tahun      INT NOT NULL,
    juara      TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_student_achievements_student ON student_achievements(student_id);
