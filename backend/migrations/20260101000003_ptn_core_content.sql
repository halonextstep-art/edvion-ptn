-- Question bank, tryout sessions, attempts — the core content/assessment tables.
-- All `created_by`/`reviewed_by`/`student_id` columns are bigint FKs to `core.akun(id)`
-- (not `core.akun.public_id` — that UUID is only the app-facing identity, internal FKs use
-- the real bigint PK, exactly like `tahfiz.setoran.akun_id` does).

CREATE TABLE ptn.questions (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id       BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    code            TEXT NOT NULL UNIQUE,
    question_type   TEXT NOT NULL CHECK (question_type IN ('multiple_choice', 'complex_multiple', 'short_answer', 'true_false', 'matching', 'essay', 'true_false_complex', 'matching_image', 'ordering')),
    subject         TEXT NOT NULL,
    topic           TEXT NOT NULL DEFAULT 'General',
    subtopic        TEXT NOT NULL DEFAULT 'General',
    difficulty      TEXT NOT NULL CHECK (difficulty IN ('easy', 'medium', 'hard')),
    bloom_level     TEXT NOT NULL DEFAULT 'C3 - Aplikasi',
    stimulus        TEXT,
    question_text   TEXT NOT NULL,
    options         JSONB,
    correct_answer  TEXT NOT NULL,
    explanation     TEXT NOT NULL DEFAULT '',
    tags            TEXT[] NOT NULL DEFAULT '{}',
    status          TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'review', 'approved', 'rejected', 'revision')),
    review_note     TEXT,
    reviewed_by     BIGINT REFERENCES core.akun(id) ON DELETE SET NULL,
    reviewed_at     TIMESTAMPTZ,
    created_by      BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE RESTRICT,
    usage_count     INT NOT NULL DEFAULT 0,
    average_score   DOUBLE PRECISION NOT NULL DEFAULT 0,
    time_limit      INT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE ONLY ptn.questions FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.questions ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.questions
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_questions_tenant_id ON ptn.questions(tenant_id);
CREATE INDEX idx_ptn_questions_subject ON ptn.questions(subject);
CREATE INDEX idx_ptn_questions_status ON ptn.questions(status);
CREATE INDEX idx_ptn_questions_difficulty ON ptn.questions(difficulty);
CREATE INDEX idx_ptn_questions_created_by ON ptn.questions(created_by);

CREATE TABLE ptn.question_sets (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id       BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    created_by      BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE RESTRICT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE ONLY ptn.question_sets FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.question_sets ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.question_sets
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_question_sets_tenant_id ON ptn.question_sets(tenant_id);

CREATE TABLE ptn.question_set_items (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    set_id          BIGINT NOT NULL REFERENCES ptn.question_sets(id) ON DELETE CASCADE,
    question_id     BIGINT NOT NULL REFERENCES ptn.questions(id) ON DELETE CASCADE,
    sort_order      INT NOT NULL DEFAULT 0,
    added_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (set_id, question_id)
);
CREATE INDEX idx_ptn_question_set_items_set ON ptn.question_set_items(set_id, sort_order);
-- No RLS/tenant_id here: scoping is transitively via `set_id -> question_sets.tenant_id`,
-- and this table is only ever queried joined against question_sets (same pattern as
-- `tahfiz` child tables that omit their own tenant_id when a parent already enforces it
-- — except `tahfiz` always adds its own redundant tenant_id; we skip it here only for
-- pure join tables with no independent existence, matching `attempt_questions` below).

CREATE TABLE ptn.question_irt_params (
    question_id     BIGINT PRIMARY KEY REFERENCES ptn.questions(id) ON DELETE CASCADE,
    difficulty_b    DOUBLE PRECISION NOT NULL,
    sample_size     BIGINT NOT NULL,
    calibrated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE ptn.tryout_sessions (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id           BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    title               TEXT NOT NULL,
    session_type        TEXT NOT NULL CHECK (session_type IN ('tryout', 'drilling', 'mini')),
    duration_minutes    INT NOT NULL,
    question_count      INT NOT NULL,
    subject_filter      TEXT,
    topic_filter        TEXT,
    difficulty_filter   TEXT,
    is_premium          BOOLEAN NOT NULL DEFAULT false,
    question_set_id     BIGINT REFERENCES ptn.question_sets(id) ON DELETE SET NULL,
    is_draft            BOOLEAN NOT NULL DEFAULT false,
    is_elective         BOOLEAN NOT NULL DEFAULT false,
    exam_track          TEXT NOT NULL DEFAULT 'snbt' CHECK (exam_track IN ('snbt', 'tka')),
    school_type_scope   TEXT CHECK (school_type_scope IN ('sma', 'smk', 'ma', 'smp')),
    created_by          BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE RESTRICT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE ONLY ptn.tryout_sessions FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.tryout_sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.tryout_sessions
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_tryout_sessions_tenant_id ON ptn.tryout_sessions(tenant_id);

CREATE TABLE ptn.attempts (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id           BIGINT DEFAULT (current_setting('app.tenant_id'::text))::bigint NOT NULL REFERENCES core.tenant(id),
    session_id          BIGINT NOT NULL REFERENCES ptn.tryout_sessions(id) ON DELETE CASCADE,
    student_id          BIGINT NOT NULL REFERENCES core.akun(id) ON DELETE CASCADE,
    status              TEXT NOT NULL DEFAULT 'in_progress' CHECK (status IN ('in_progress', 'submitted')),
    duration_minutes    INT NOT NULL,
    started_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    submitted_at        TIMESTAMPTZ,
    score               INT,
    accuracy            DOUBLE PRECISION,
    correct_count       INT,
    wrong_count         INT,
    unanswered_count    INT,
    time_used_seconds   INT
);
ALTER TABLE ONLY ptn.attempts FORCE ROW LEVEL SECURITY;
ALTER TABLE ptn.attempts ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ptn.attempts
    USING ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean)
    WITH CHECK ((tenant_id = (current_setting('app.tenant_id'::text))::bigint) OR (current_setting('app.is_platform_admin'::text))::boolean);
CREATE INDEX idx_ptn_attempts_tenant_id ON ptn.attempts(tenant_id);
CREATE INDEX idx_ptn_attempts_student ON ptn.attempts(student_id);
CREATE INDEX idx_ptn_attempts_session ON ptn.attempts(session_id);

CREATE TABLE ptn.attempt_questions (
    attempt_id      BIGINT NOT NULL REFERENCES ptn.attempts(id) ON DELETE CASCADE,
    question_id     BIGINT NOT NULL REFERENCES ptn.questions(id) ON DELETE RESTRICT,
    order_index     INT NOT NULL,
    PRIMARY KEY (attempt_id, question_id)
);
CREATE INDEX idx_ptn_attempt_questions_attempt ON ptn.attempt_questions(attempt_id, order_index);

CREATE TABLE ptn.attempt_answers (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    attempt_id      BIGINT NOT NULL REFERENCES ptn.attempts(id) ON DELETE CASCADE,
    question_id     BIGINT NOT NULL REFERENCES ptn.questions(id) ON DELETE RESTRICT,
    answer_text     TEXT,
    is_correct      BOOLEAN,
    flagged         BOOLEAN NOT NULL DEFAULT false,
    answered_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (attempt_id, question_id)
);
