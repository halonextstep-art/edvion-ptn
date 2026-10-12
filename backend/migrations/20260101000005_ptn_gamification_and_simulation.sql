-- Gamification (points/badges/challenges) + full exam simulation (templates/runs/slots).

CREATE TABLE ptn.point_rules (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id       BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    action          TEXT NOT NULL,
    category        TEXT NOT NULL,
    base_points     INT NOT NULL DEFAULT 0,
    multiplier      DOUBLE PRECISION NOT NULL DEFAULT 1,
    enabled         BOOLEAN NOT NULL DEFAULT true,
    icon            TEXT NOT NULL DEFAULT '✅',
    sort_order      INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE ONLY ptn.point_rules FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.point_rules ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.point_rules
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_point_rules_tenant_id ON ptn.point_rules(tenant_id);

CREATE TABLE ptn.badges (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id       BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    emoji           TEXT NOT NULL DEFAULT '🏆',
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    rarity          TEXT NOT NULL CHECK (rarity IN ('common', 'rare', 'epic', 'legendary')),
    condition_type  TEXT NOT NULL CHECK (condition_type IN ('score_gte', 'streak_gte', 'questions_gte', 'tryout_gte', 'rank_lte')),
    condition_value INT NOT NULL DEFAULT 0,
    active          BOOLEAN NOT NULL DEFAULT true,
    icon_type       TEXT NOT NULL DEFAULT 'preset' CHECK (icon_type IN ('preset', 'custom')),
    icon_name       TEXT,
    icon_url        TEXT,
    created_by      BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE RESTRICT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE ONLY ptn.badges FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.badges ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.badges
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_badges_tenant_id ON ptn.badges(tenant_id);

CREATE TABLE ptn.challenges (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id       BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    name            TEXT NOT NULL,
    challenge_type  TEXT NOT NULL CHECK (challenge_type IN ('most_solved', 'highest_score', 'longest_streak', 'speed')),
    status          TEXT NOT NULL DEFAULT 'upcoming' CHECK (status IN ('upcoming', 'active', 'ended')),
    start_date      DATE NOT NULL,
    end_date        DATE NOT NULL,
    target_value    INT NOT NULL DEFAULT 0,
    reward_points   INT NOT NULL DEFAULT 0,
    reward_badge    TEXT,
    description     TEXT NOT NULL DEFAULT '',
    created_by      BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE RESTRICT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE ONLY ptn.challenges FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.challenges ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.challenges
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_challenges_tenant_id ON ptn.challenges(tenant_id);

CREATE TABLE ptn.simulation_templates (
    id                      BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id               BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    title                   TEXT NOT NULL,
    is_active               BOOLEAN NOT NULL DEFAULT true,
    is_premium              BOOLEAN NOT NULL DEFAULT true,
    is_draft                BOOLEAN NOT NULL DEFAULT false,
    elective_package_id     BIGINT REFERENCES ptn.packages(id) ON DELETE SET NULL,
    template_kind           TEXT NOT NULL DEFAULT 'utbk_combined' CHECK (template_kind IN ('utbk_combined', 'per_subject')),
    exam_track              TEXT NOT NULL DEFAULT 'snbt' CHECK (exam_track IN ('snbt', 'tka')),
    school_type_scope       TEXT CHECK (school_type_scope IN ('sma', 'smk', 'ma', 'smp')),
    lockdown_override       BOOLEAN,
    created_by              BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE RESTRICT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE ONLY ptn.simulation_templates FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.simulation_templates ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.simulation_templates
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_simulation_templates_tenant_id ON ptn.simulation_templates(tenant_id);

CREATE TABLE ptn.simulation_template_slots (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    template_id     BIGINT NOT NULL REFERENCES ptn.simulation_templates(id) ON DELETE CASCADE,
    sequence_index  INT NOT NULL,
    session_id      BIGINT REFERENCES ptn.tryout_sessions(id) ON DELETE RESTRICT,
    break_seconds   INT NOT NULL DEFAULT 300,
    is_elective     BOOLEAN NOT NULL DEFAULT false,
    UNIQUE (template_id, sequence_index),
    CONSTRAINT chk_simulation_template_slot_session CHECK (is_elective OR (session_id IS NOT NULL))
);

CREATE TABLE ptn.simulation_runs (
    id                          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id                   BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    template_id                 BIGINT NOT NULL REFERENCES ptn.simulation_templates(id) ON DELETE RESTRICT,
    student_id                  BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE CASCADE,
    status                      TEXT NOT NULL DEFAULT 'in_progress' CHECK (status IN ('in_progress', 'completed', 'abandoned')),
    current_sequence_index      INT NOT NULL DEFAULT 1,
    combined_estimate_score     DOUBLE PRECISION,
    started_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at                TIMESTAMPTZ,
    lockdown_enabled            BOOLEAN NOT NULL DEFAULT false
);
ALTER TABLE ONLY ptn.simulation_runs FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.simulation_runs ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.simulation_runs
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_simulation_runs_tenant_id ON ptn.simulation_runs(tenant_id);
CREATE INDEX idx_ptn_simulation_runs_student ON ptn.simulation_runs(student_id);

CREATE TABLE ptn.simulation_run_slots (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    run_id          BIGINT NOT NULL REFERENCES ptn.simulation_runs(id) ON DELETE CASCADE,
    sequence_index  INT NOT NULL,
    session_id      BIGINT NOT NULL REFERENCES ptn.tryout_sessions(id) ON DELETE RESTRICT,
    break_seconds   INT NOT NULL,
    attempt_id      BIGINT REFERENCES ptn.attempts(id) ON DELETE SET NULL,
    status          TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'active', 'submitted')),
    deadline_at     TIMESTAMPTZ,
    break_ends_at   TIMESTAMPTZ,
    UNIQUE (run_id, sequence_index)
);
CREATE INDEX idx_ptn_simulation_run_slots_run ON ptn.simulation_run_slots(run_id);

CREATE TABLE ptn.simulation_lockdown_violations (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    run_id          BIGINT NOT NULL REFERENCES ptn.simulation_runs(id) ON DELETE CASCADE,
    student_id      BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE CASCADE,
    event_type      TEXT NOT NULL,
    detail          TEXT,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_ptn_lockdown_violations_run ON ptn.simulation_lockdown_violations(run_id, occurred_at DESC);
