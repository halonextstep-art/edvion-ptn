-- Adds an `abandoned` status to `ptn.attempts`, mirroring `ptn.simulation_runs.status`'s
-- existing 3-state model. Previously an Attempt could only ever be `in_progress` or
-- `submitted` — there was no way to clear a stuck in-progress attempt, which blocked the
-- student from ever starting a fresh one for the same session (see
-- `TryoutService::start_attempt_impl`'s resume-instead-of-duplicate guard: it hands back ANY
-- `in_progress` attempt for that session, forever, until it stops being `in_progress`).
-- `School::force_abandon_attempt` is the only place that sets this.
ALTER TABLE ptn.attempts DROP CONSTRAINT attempts_status_check;
ALTER TABLE ptn.attempts ADD CONSTRAINT attempts_status_check CHECK (status IN ('in_progress', 'submitted', 'abandoned'));
