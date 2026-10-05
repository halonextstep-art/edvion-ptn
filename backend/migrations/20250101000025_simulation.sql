-- Simulasi UTBK — chains several existing TryoutSessions into ONE continuous, server-enforced
-- exam run (fixed order, official per-slot timing, mandatory break between slots, no going
-- back), mirroring how real UTBK/SNBT's 7 subtests are administered in one sitting instead of
-- independent practice sessions. Reuses the EXISTING TryoutSession/Attempt machinery entirely
-- (TryoutService::start_attempt/submit_attempt do all real scoring/grading) — a "slot" just
-- tracks which attempt belongs to which position in a run, and the server (not the client
-- timer) is authoritative about deadlines/breaks/advancement. `combined_estimate_score` is
-- explicitly an ESTIMATE (equal-weighted average across slots, rescaled to the same UTBK
-- ballpark used elsewhere — see UTBK_SCALE_MAX/PLATFORM_SCALE_MAX in rationalization_service.rs),
-- never a real IRT-calibrated score.

CREATE TABLE simulation_templates (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title       TEXT NOT NULL,
    is_active   BOOLEAN NOT NULL DEFAULT true,
    created_by  UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Not DB-enforced to be exactly 7 rows (validated in SimulationService instead) so the schema
-- stays generic if the official subtest count/structure ever changes.
CREATE TABLE simulation_template_slots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id     UUID NOT NULL REFERENCES simulation_templates(id) ON DELETE CASCADE,
    sequence_index  INT NOT NULL,
    session_id      UUID NOT NULL REFERENCES tryout_sessions(id) ON DELETE RESTRICT,
    break_seconds   INT NOT NULL DEFAULT 300,
    UNIQUE (template_id, sequence_index)
);

CREATE TABLE simulation_runs (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id              UUID NOT NULL REFERENCES simulation_templates(id) ON DELETE RESTRICT,
    student_id               UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status                   TEXT NOT NULL DEFAULT 'in_progress' CHECK (status IN ('in_progress', 'completed', 'abandoned')),
    current_sequence_index   INT NOT NULL DEFAULT 1,
    combined_estimate_score  DOUBLE PRECISION,
    started_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at             TIMESTAMPTZ
);
CREATE INDEX idx_simulation_runs_student ON simulation_runs(student_id);

-- Copied from simulation_template_slots at run-creation time (not a live FK to the template's
-- slots) so editing/deleting a template later never mutates an already-started run's history.
CREATE TABLE simulation_run_slots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id          UUID NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    sequence_index  INT NOT NULL,
    session_id      UUID NOT NULL REFERENCES tryout_sessions(id) ON DELETE RESTRICT,
    break_seconds   INT NOT NULL,
    attempt_id      UUID REFERENCES attempts(id) ON DELETE SET NULL,
    status          TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'active', 'submitted')),
    deadline_at     TIMESTAMPTZ,
    break_ends_at   TIMESTAMPTZ,
    UNIQUE (run_id, sequence_index)
);
CREATE INDEX idx_simulation_run_slots_run ON simulation_run_slots(run_id);
