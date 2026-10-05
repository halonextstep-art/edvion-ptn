-- Badge & Package icon system — replaces the raw emoji picker with a curated
-- lucide-vue-next preset icon ("Pilih Ikon") or an admin-uploaded custom image
-- ("Upload Gambar"). Purely additive: the old `emoji` column on both tables is left
-- untouched (harmless legacy data — the app simply stops reading it for display), and
-- every existing row is backfilled below so nothing renders blank after this migration.
--
-- icon_type: 'preset' (icon_name holds a lucide icon name, e.g. 'trophy') or 'custom'
-- (icon_url holds an uploaded file URL under /uploads/icons/, see
-- infrastructure::storage::ICON_POLICY).

ALTER TABLE badges
    ADD COLUMN icon_type TEXT NOT NULL DEFAULT 'preset' CHECK (icon_type IN ('preset', 'custom')),
    ADD COLUMN icon_name TEXT,
    ADD COLUMN icon_url  TEXT;

-- Backfill every existing badge row from its current `emoji` value. Covers every emoji
-- literal found in this codebase: frontend/components/admin/GamificationManager.vue's
-- BADGE_EMOJIS array, and the badge_seeds in backend/src/bin/seed.rs. Anything not on
-- either list (shouldn't exist today, but just in case) falls back to 'award'.
UPDATE badges SET icon_name = CASE emoji
    WHEN '🔥' THEN 'flame'
    WHEN '💪' THEN 'dumbbell'
    WHEN '🎯' THEN 'target'
    WHEN '👑' THEN 'crown'
    WHEN '🏆' THEN 'trophy'
    WHEN '⭐' THEN 'star'
    WHEN '📚' THEN 'book-open'
    WHEN '🚀' THEN 'rocket'
    WHEN '⚡' THEN 'zap'
    WHEN '🧠' THEN 'brain'
    WHEN '🎖️' THEN 'medal'
    WHEN '💎' THEN 'gem'
    ELSE 'award'
END;

ALTER TABLE packages
    ADD COLUMN icon_type TEXT NOT NULL DEFAULT 'preset' CHECK (icon_type IN ('preset', 'custom')),
    ADD COLUMN icon_name TEXT,
    ADD COLUMN icon_url  TEXT;

-- Backfill every existing package row from its current `emoji` value. Covers every emoji
-- literal found in frontend/components/admin/PackageManager.vue's EMOJI_OPTIONS array and
-- the package_seeds in backend/src/bin/seed.rs. Fallback 'package' for anything unmapped.
UPDATE packages SET icon_name = CASE emoji
    WHEN '📘' THEN 'book'
    WHEN '🎯' THEN 'target'
    WHEN '🏆' THEN 'trophy'
    WHEN '⚡' THEN 'zap'
    WHEN '💎' THEN 'gem'
    WHEN '📚' THEN 'book-open'
    WHEN '🚀' THEN 'rocket'
    WHEN '🌟' THEN 'sparkles'
    WHEN '🔥' THEN 'flame'
    WHEN '🎓' THEN 'graduation-cap'
    ELSE 'package'
END;
