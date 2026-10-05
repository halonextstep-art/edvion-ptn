-- Subject taxonomy catalogue (Kategori & Mata Uji) — replaces the previously hardcoded
-- 8-item SUBJECTS array scattered across several frontend files with an admin-editable
-- database catalogue, grouped by exam/assessment category: SNBT (UTBK admission test),
-- TKA-IPA / TKA-IPS (Tes Kemampuan Akademik — the school-leaving exam that replaced UN
-- starting 2025/2026, elective subjects split by track), and AKM (Asesmen Kompetensi
-- Minimum — the national school-quality literacy/numeracy assessment). These are four
-- genuinely distinct exams/assessments with different purposes, not four variants of one
-- exam, and their subject requirements have already changed once in the last year (SNBT
-- dropped its old TKA-Saintek/Soshum subtests; TKA itself is brand new) — hence storing this
-- as editable data instead of hardcoding it again, so a future policy change doesn't require
-- a code deploy.
--
-- This migration is purely additive: it does NOT touch the existing `questions.subject`
-- free-text column, any existing question rows, or the separate 13-subject SNBP rapor list
-- (student_rapor_scores.subject, used by the rationalization engine's root-keyword subject
-- matcher — a structurally unrelated system this deliberately does not touch). The seed step
-- (see seed.rs) folds the 8 pre-existing legacy subject names into real categories here so no
-- existing question becomes orphaned/invisible in the new catalogue-driven dropdowns.

CREATE TABLE subject_categories (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    sort_order  INT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE subjects (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id UUID NOT NULL REFERENCES subject_categories(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    code        TEXT NOT NULL DEFAULT '',
    sort_order  INT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (category_id, name)
);
CREATE INDEX idx_subjects_category ON subjects(category_id);
CREATE INDEX idx_subjects_sort ON subjects(sort_order);
