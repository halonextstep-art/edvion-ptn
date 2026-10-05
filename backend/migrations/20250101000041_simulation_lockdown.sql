-- "Mode Terkunci" for Simulasi TO — a proctoring-lite focus mode that hides in-app navigation,
-- forces fullscreen, disables copy/right-click/common shortcuts, and detects tab-switch /
-- fullscreen-exit while a student is running a SimulationRun. Deliberately scoped to Simulasi
-- TO only (SimulationTemplate/SimulationRun) — Drilling and Latihan/Mini sessions are
-- untouched, per product decision.
--
-- Two levels of control, per explicit product decision ("bisa keduanya"):
--   1. A global platform-wide default (`platform_settings.simulation_lockdown_default`).
--   2. An optional per-template override (`simulation_templates.lockdown_override`) — NULL
--      means "inherit the global default", `true`/`false` force it on/off for that template
--      regardless of the global default.
--
-- The *effective* decision is resolved once, at `start_run` time, and snapshotted onto
-- `simulation_runs.lockdown_enabled` — so an admin flipping either setting mid-exam can never
-- change the rules for a run already in progress (avoids a half-locked, inconsistent UX).
ALTER TABLE platform_settings ADD COLUMN simulation_lockdown_default BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE simulation_templates ADD COLUMN lockdown_override BOOLEAN;

ALTER TABLE simulation_runs ADD COLUMN lockdown_enabled BOOLEAN NOT NULL DEFAULT false;

-- One row per detected breach (tab switch, fullscreen exit, devtools attempt, etc.) — the
-- student is warned in the UI immediately, and this row lets Admin/Sekolah review afterward.
-- Mirrors the shape of `audit_log` (20250101000015_audit_log.sql).
CREATE TABLE simulation_lockdown_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id UUID NOT NULL REFERENCES simulation_runs(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    detail TEXT,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_lockdown_violations_run ON simulation_lockdown_violations(run_id, occurred_at DESC);
