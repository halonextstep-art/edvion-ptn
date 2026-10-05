-- Gamification (Manajemen Gamifikasi) — point rules, badges, and challenges,
-- mirroring the reference `AdminGamification`. Unlike the reference (which fabricates
-- `earnedBy`/leaderboard/`participants`/`completions` as static seed numbers), none of
-- those figures are stored here: they are always computed live from the real `attempts`
-- table at read time (same "derive, never store" pattern as Event.participants), so a
-- badge's earned count or a challenge's participant count can never drift from what
-- students actually did.

CREATE TABLE point_rules (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action          TEXT NOT NULL,
    category        TEXT NOT NULL,
    base_points     INT NOT NULL DEFAULT 0,
    multiplier      DOUBLE PRECISION NOT NULL DEFAULT 1,
    enabled         BOOLEAN NOT NULL DEFAULT true,
    icon            TEXT NOT NULL DEFAULT '✅',
    sort_order      INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE badges (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    emoji           TEXT NOT NULL DEFAULT '🏆',
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    rarity          TEXT NOT NULL CHECK (rarity IN ('common', 'rare', 'epic', 'legendary')),
    condition_type  TEXT NOT NULL CHECK (condition_type IN ('score_gte', 'streak_gte', 'questions_gte', 'tryout_gte', 'rank_lte')),
    condition_value INT NOT NULL DEFAULT 0,
    active          BOOLEAN NOT NULL DEFAULT true,
    created_by      UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE challenges (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    challenge_type  TEXT NOT NULL CHECK (challenge_type IN ('most_solved', 'highest_score', 'longest_streak', 'speed')),
    status          TEXT NOT NULL DEFAULT 'upcoming' CHECK (status IN ('upcoming', 'active', 'ended')),
    start_date      DATE NOT NULL,
    end_date        DATE NOT NULL,
    target_value    INT NOT NULL DEFAULT 0,
    reward_points   INT NOT NULL DEFAULT 0,
    reward_badge    TEXT,
    description     TEXT NOT NULL DEFAULT '',
    created_by      UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
