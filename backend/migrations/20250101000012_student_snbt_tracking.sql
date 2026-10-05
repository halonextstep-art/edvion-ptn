-- Real post-exam tracking for SNBT targets, entered by the School PIC — powers "Rekap
-- SNBT": actual exam score, exam date, admission status, and free-text notes per target.
-- One-to-one with student_ptn_targets (a student can have multiple SNBT choices, each
-- tracked separately).

CREATE TABLE student_snbt_tracking (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ptn_target_id UUID NOT NULL UNIQUE REFERENCES student_ptn_targets(id) ON DELETE CASCADE,
    actual_score  DOUBLE PRECISION,
    exam_date     DATE,
    status        TEXT NOT NULL DEFAULT 'terdaftar',
    notes         TEXT NOT NULL DEFAULT '',
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
