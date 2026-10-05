-- GASPOLPTN initial schema
-- Enums are modeled as TEXT + CHECK constraints (matches domain enums encoded via
-- sqlx::Type(type_name = "text")), which keeps migrations simple to evolve.

CREATE EXTENSION IF NOT EXISTS "pgcrypto"; -- gen_random_uuid()

-- ─── Schools ────────────────────────────────────────────────────────────────────
CREATE TABLE schools (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    school_type     TEXT NOT NULL CHECK (school_type IN ('sma', 'smk', 'ma')),
    city            TEXT NOT NULL,
    province        TEXT NOT NULL,
    email           TEXT NOT NULL,
    phone           TEXT NOT NULL,
    join_date       DATE NOT NULL DEFAULT CURRENT_DATE,
    package_type    TEXT NOT NULL CHECK (package_type IN ('basic', 'premium', 'enterprise')),
    contact_person  TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ─── Users (admin / school / student / content) ────────────────────────────────
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    email           TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    role            TEXT NOT NULL CHECK (role IN ('admin', 'school', 'student', 'content')),
    school_id       UUID REFERENCES schools(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);

-- ─── Student profile (1:1 extension of users where role = 'student') ──────────
CREATE TABLE students (
    user_id         UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    school_id       UUID REFERENCES schools(id) ON DELETE SET NULL,
    grade           TEXT,
    class           TEXT,
    total_points    INT NOT NULL DEFAULT 0,
    streak          INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ─── Questions (Bank Soal) ──────────────────────────────────────────────────────
CREATE TABLE questions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code            TEXT NOT NULL UNIQUE,
    question_type   TEXT NOT NULL CHECK (question_type IN ('multiple_choice', 'complex_multiple', 'short_answer')),
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
    reviewed_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    reviewed_at     TIMESTAMPTZ,
    created_by      UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    usage_count     INT NOT NULL DEFAULT 0,
    average_score   DOUBLE PRECISION NOT NULL DEFAULT 0,
    time_limit      INT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_questions_subject ON questions(subject);
CREATE INDEX idx_questions_status ON questions(status);
CREATE INDEX idx_questions_difficulty ON questions(difficulty);
CREATE INDEX idx_questions_created_by ON questions(created_by);

-- ─── Tryout / Drilling session templates ───────────────────────────────────────
CREATE TABLE tryout_sessions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title               TEXT NOT NULL,
    session_type        TEXT NOT NULL CHECK (session_type IN ('tryout', 'drilling', 'mini')),
    duration_minutes    INT NOT NULL,
    question_count      INT NOT NULL,
    subject_filter      TEXT,
    topic_filter        TEXT,
    difficulty_filter   TEXT,
    is_premium          BOOLEAN NOT NULL DEFAULT false,
    created_by          UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ─── Attempts (a student's run through a session) ──────────────────────────────
CREATE TABLE attempts (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id          UUID NOT NULL REFERENCES tryout_sessions(id) ON DELETE CASCADE,
    student_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
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
CREATE INDEX idx_attempts_student ON attempts(student_id);
CREATE INDEX idx_attempts_session ON attempts(session_id);

-- Snapshot of which questions (and in what order) were assigned to an attempt.
CREATE TABLE attempt_questions (
    attempt_id      UUID NOT NULL REFERENCES attempts(id) ON DELETE CASCADE,
    question_id     UUID NOT NULL REFERENCES questions(id) ON DELETE RESTRICT,
    order_index     INT NOT NULL,
    PRIMARY KEY (attempt_id, question_id)
);
CREATE INDEX idx_attempt_questions_attempt ON attempt_questions(attempt_id, order_index);

-- Student's answers within an attempt (autosaved as they progress through the quiz).
CREATE TABLE attempt_answers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attempt_id      UUID NOT NULL REFERENCES attempts(id) ON DELETE CASCADE,
    question_id     UUID NOT NULL REFERENCES questions(id) ON DELETE RESTRICT,
    answer_text     TEXT,
    is_correct      BOOLEAN,
    flagged         BOOLEAN NOT NULL DEFAULT false,
    answered_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (attempt_id, question_id)
);
