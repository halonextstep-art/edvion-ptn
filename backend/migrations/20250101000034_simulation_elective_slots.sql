-- "Simulasi TKA" support — lets a SimulationTemplate slot be PERSONALIZED per student instead of
-- pointing at one fixed TryoutSession for everyone. Real TKA SMA/SMK administers 2 mapel pilihan
-- (student-chosen, out of up to N options) in one continuous session on day 2, alongside 3 fixed
-- mapel wajib in one continuous session on day 1 — see domain::simulation doc comment for the
-- full rationale. A slot with is_elective = true has session_id = NULL in the TEMPLATE (there is
-- no single fixed session — it depends on who's taking it); SimulationService::start_run resolves
-- it to a real session_id per-student (from package_elective_choices, via elective_package_id
-- below) when copying template slots into simulation_run_slots, which keeps session_id NOT NULL
-- there exactly as before (a run's slots are always a fully-resolved, immutable snapshot).

ALTER TABLE simulation_template_slots ALTER COLUMN session_id DROP NOT NULL;
ALTER TABLE simulation_template_slots ADD COLUMN is_elective BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE simulation_template_slots ADD CONSTRAINT chk_simulation_template_slot_session
    CHECK (is_elective OR session_id IS NOT NULL);

-- Which Package's "mapel pilihan" choices (see 20250101000033_package_electives.sql) resolve
-- this template's is_elective slots. NULL for templates with no elective slot at all (e.g. every
-- existing Simulasi UTBK template, and a TKA "Hari 1 (Wajib)" template). Required (validated in
-- SimulationService, not DB-enforced) whenever the template has at least one is_elective slot.
ALTER TABLE simulation_templates ADD COLUMN elective_package_id UUID REFERENCES packages(id) ON DELETE SET NULL;

-- 'utbk_combined' (default, existing behavior): finish_run computes combined_estimate_score as
-- an equal-weighted average across slots, rescaled to the UTBK 0-825 ballpark — correct for
-- Simulasi UTBK, where SNBT really does have one combined/weighted score for ranking.
-- 'per_subject': finish_run leaves combined_estimate_score NULL — real TKA reports a score PER
-- MATA UJI and never combines them into one composite for the student (see domain::simulation
-- doc comment). Frontend must read each slot's own attempt score instead of the run's combined
-- field for a 'per_subject' template.
ALTER TABLE simulation_templates ADD COLUMN template_kind TEXT NOT NULL DEFAULT 'utbk_combined'
    CHECK (template_kind IN ('utbk_combined', 'per_subject'));
