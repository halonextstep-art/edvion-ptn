-- Remodel: a student sits ONE real UTBK/SNBT exam that produces ONE score, applied
-- across whichever SNBT program choices (pilihan 1/2) they registered — it is not a
-- separate exam per choice. The original `student_snbt_tracking` table (keyed by
-- `ptn_target_id`) incorrectly modeled tracking as per-target, forcing the frontend to
-- guess a single "representative" row per student for display. This migration re-keys
-- the table to `student_id` (one row per student), backfilling from the most-advanced
-- existing per-target row (Diterima > Sudah Ujian/Tidak Diterima > Terdaftar, tie-broken
-- by score then recency) so no real data entered so far is lost.

ALTER TABLE student_snbt_tracking ADD COLUMN student_id UUID REFERENCES users(id) ON DELETE CASCADE;

UPDATE student_snbt_tracking t
SET student_id = spt.student_id
FROM student_ptn_targets spt
WHERE t.ptn_target_id = spt.id;

-- Any row that couldn't resolve a student (orphaned target) can't be kept.
DELETE FROM student_snbt_tracking WHERE student_id IS NULL;

-- Collapse multiple per-target rows for the same student into the single most-advanced one.
WITH ranked AS (
  SELECT id,
         ROW_NUMBER() OVER (
           PARTITION BY student_id
           ORDER BY
             CASE status
               WHEN 'diterima' THEN 4
               WHEN 'tidak-diterima' THEN 3
               WHEN 'sudah-ujian' THEN 2
               ELSE 1
             END DESC,
             actual_score DESC NULLS LAST,
             updated_at DESC
         ) AS rn
  FROM student_snbt_tracking
)
DELETE FROM student_snbt_tracking WHERE id IN (SELECT id FROM ranked WHERE rn > 1);

ALTER TABLE student_snbt_tracking ALTER COLUMN student_id SET NOT NULL;
ALTER TABLE student_snbt_tracking ADD CONSTRAINT student_snbt_tracking_student_id_key UNIQUE (student_id);
ALTER TABLE student_snbt_tracking DROP COLUMN ptn_target_id;
