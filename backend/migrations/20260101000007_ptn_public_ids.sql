-- Every `ptn.*` business table has a bigint internal PK (for FK joins/RLS), but every
-- domain struct (`Question`, `Package`, `TryoutSession`, ...) still exposes `id: Uuid` to
-- the rest of the app (API wire format unchanged from before this migration). This adds a
-- `public_id uuid DEFAULT gen_random_uuid() UNIQUE NOT NULL` column to every `ptn.*` table
-- whose Rust domain struct has `id: Uuid` — exactly the same pattern `core.akun`/
-- `core.tenant` already use for their own public identity.
--
-- NOT added to pure join/child tables with no independent identity of their own (composite
-- PK, no separate `id` column in Rust either): `ptn.attempt_questions`,
-- `ptn.question_set_items`. `ptn.question_irt_params` also has no `public_id` — its PK
-- *is* `question_id` (1:1 with `ptn.questions`, see domain::irt — no separate id at all).
-- `ptn.platform_settings` is a pinned singleton (`id` always 1), no `public_id` needed.

DO $$
DECLARE
    t text;
BEGIN
    FOREACH t IN ARRAY ARRAY[
        'questions', 'question_sets',
        'tryout_sessions', 'attempts', 'attempt_answers',
        'events', 'packages', 'package_content_items', 'package_elective_choices',
        'vouchers', 'voucher_redemptions', 'payment_transactions', 'school_package_entitlements',
        'point_rules', 'badges', 'challenges',
        'simulation_templates', 'simulation_template_slots', 'simulation_runs', 'simulation_run_slots',
        'simulation_lockdown_violations',
        'institutions', 'ptn_programs', 'student_rapor_scores', 'student_ptn_targets',
        'student_achievements', 'student_snbt_tracking', 'snbp_participation',
        'school_alumni_benchmarks', 'alumni_rapor_scores', 'school_snbp_eligibility',
        'school_reports', 'audit_log', 'notifications', 'subject_categories', 'subjects',
        'admission_deadlines'
    ]
    LOOP
        EXECUTE format('ALTER TABLE ptn.%I ADD COLUMN IF NOT EXISTS public_id uuid DEFAULT gen_random_uuid() UNIQUE NOT NULL', t);
    END LOOP;
END $$;
