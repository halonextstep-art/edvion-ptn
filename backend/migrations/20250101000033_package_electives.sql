-- "Mapel Pilihan" — lets a Package (e.g. "TKA SMA") mark some of its TryoutSessions as
-- elective (is_elective = true) instead of mandatory, and require students to pick exactly
-- `elective_pick_count` of them before those specific sessions become startable — mirroring
-- real TKA rules (3 mapel wajib selalu tersedia + siswa memilih tepat 2 dari mapel pilihan).
-- Default false/0 for every pre-existing row: no package is retroactively restricted by this
-- migration, matching the "honest, backward-compatible" pattern used by every prior migration
-- in this project (see 20250101000032's is_draft doc comment).
ALTER TABLE tryout_sessions ADD COLUMN is_elective BOOLEAN NOT NULL DEFAULT false;

-- 0 = no elective restriction at all (every session in the package, wajib or not, behaves
-- exactly as before this feature existed — this is the default for every package that predates
-- and every package that simply never opts into the elective flow). A positive number is the
-- exact count of `is_elective` sessions IN THIS PACKAGE a student must choose (see
-- package_elective_choices below) before starting any of them.
ALTER TABLE packages ADD COLUMN elective_pick_count INT NOT NULL DEFAULT 0;

-- One row = "this student has currently picked this elective session within this package".
-- Per product decision, choices are NOT locked once made — a student may freely add/remove
-- picks any time (see ElectiveService::set_choices, which does a full replace rather than an
-- append-only ledger), so this table always reflects the student's CURRENT selection, not a
-- history of past ones.
CREATE TABLE package_elective_choices (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    package_id UUID NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    session_id UUID NOT NULL REFERENCES tryout_sessions(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (student_id, package_id, session_id)
);
CREATE INDEX idx_package_elective_choices_student_package ON package_elective_choices(student_id, package_id);
