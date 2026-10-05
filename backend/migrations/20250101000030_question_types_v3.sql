-- Widen questions.question_type CHECK constraint to allow 3 more authoring types:
-- true_false_complex (Benar/Salah Ganda), matching_image (Menjodohkan Gambar), ordering
-- (Urutkan). No new columns needed — these reuse the existing `options`/`correct_answer` text
-- fields with a type-specific encoding (see domain/question.rs QuestionType doc comments for
-- the exact convention each type uses).
ALTER TABLE questions DROP CONSTRAINT questions_question_type_check;
ALTER TABLE questions ADD CONSTRAINT questions_question_type_check
    CHECK (question_type IN (
        'multiple_choice', 'complex_multiple', 'short_answer', 'true_false', 'matching', 'essay',
        'true_false_complex', 'matching_image', 'ordering'
    ));
