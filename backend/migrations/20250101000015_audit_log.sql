-- Accountability trail for "edit-on-behalf" actions: whenever a School PIC (or Admin)
-- creates/edits/deletes a STUDENT's own academic data (rapor, prestasi, target PTN,
-- SNBP/SNBT tracking) on their behalf, one row is recorded here. Students editing their
-- own data are NOT logged — this exists specifically so a student (or another PIC) can
-- later see who touched their record and what changed, since school-side editing bypasses
-- the student's own confirmation.
CREATE TABLE audit_log (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    actor_name  TEXT NOT NULL,
    actor_role  TEXT NOT NULL,
    student_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action      TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    summary     TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_audit_log_student ON audit_log(student_id, created_at DESC);
