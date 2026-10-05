-- Lets a saved "Laporan" (Performance/Participation) record which exam_track it was scoped
-- to when generated — see application::report_service doc comment. NULL = "Semua Jalur"
-- (both SNBT and TKA attempts blended together, the pre-existing behavior) so every report
-- generated before this migration stays valid and unchanged in meaning.
ALTER TABLE school_reports ADD COLUMN exam_track_filter TEXT;
