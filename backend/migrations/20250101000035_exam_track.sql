-- Explicit "exam track" (SNBT/UTBK vs TKA) on Package, TryoutSession, and SimulationTemplate.
-- Before this, "is this TKA content" was ONLY inferred indirectly: Package.elective_pick_count > 0
-- (see application::elective_service / application::school_tka_service doc comments), or a
-- SimulationTemplate's template_kind = 'per_subject'. That heuristic works fine for the "mapel
-- pilihan" gating feature it was built for, but gives the student-facing catalogue (DrillingZone)
-- no reliable per-session signal to group/filter by — a standalone TryoutSession carries neither
-- a package reference (Package<->TryoutSession is many-to-many via package_content_items) nor
-- any track info of its own. This column makes the track explicit and pushes it down onto
-- TryoutSession/SimulationTemplate directly, mirroring how is_elective was pushed down onto
-- TryoutSession alongside Package.elective_pick_count (20250101000033_package_electives.sql).
--
-- Default 'snbt' for every pre-existing row (the vast majority of current content is SNBT/UTBK
-- prep), then backfill 'tka' using the same heuristics this field is formalizing — anything that
-- already behaved like TKA content keeps behaving that way, now via an explicit flag instead of
-- an inference. The old heuristics are left in place (not removed) for now; nothing here depends
-- on them being changed.

ALTER TABLE packages ADD COLUMN exam_track TEXT NOT NULL DEFAULT 'snbt' CHECK (exam_track IN ('snbt', 'tka'));
ALTER TABLE tryout_sessions ADD COLUMN exam_track TEXT NOT NULL DEFAULT 'snbt' CHECK (exam_track IN ('snbt', 'tka'));
ALTER TABLE simulation_templates ADD COLUMN exam_track TEXT NOT NULL DEFAULT 'snbt' CHECK (exam_track IN ('snbt', 'tka'));

UPDATE packages SET exam_track = 'tka' WHERE elective_pick_count > 0;
UPDATE tryout_sessions SET exam_track = 'tka' WHERE is_elective = true;
UPDATE simulation_templates SET exam_track = 'tka' WHERE template_kind = 'per_subject' OR elective_package_id IS NOT NULL;
