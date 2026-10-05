-- Widen the questions.question_type CHECK constraint to allow the 3 new authoring types:
-- true_false (Benar/Salah), matching (Menjodohkan), essay (Uraian). No new columns needed —
-- these types reuse the existing `options`/`correct_answer` text fields with a type-specific
-- encoding (see domain/question.rs QuestionType doc comments for the exact convention each
-- type uses). `essay` is Drilling-only/self-check by product decision (never auto-graded,
-- never selected into Tryout/Mini attempts — enforced in random_approved()).
ALTER TABLE questions DROP CONSTRAINT questions_question_type_check;
ALTER TABLE questions ADD CONSTRAINT questions_question_type_check
    CHECK (question_type IN ('multiple_choice', 'complex_multiple', 'short_answer', 'true_false', 'matching', 'essay'));
