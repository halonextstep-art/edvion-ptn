//! Parses the bulk "Import Soal" `.docx` format (see the uploaded reference document this was
//! built against) into ready-to-create question payloads. Pure/side-effect-free: this module
//! never touches the database or the filesystem — it only turns paragraphs (from
//! `infrastructure::docx_reader`) into structured data, so the same parse can be run twice
//! (once for a read-only preview, once for the real commit) with identical results and no risk
//! of the preview call itself mutating anything.
//!
//! ## The DSL, block by block
//!
//! Each question is one block starting with a `"{no}. {question text}"` line (the leading
//! number is only ever used for user-facing "Soal #N" messages — it has no effect on import
//! order or on the created row, which always uses the document's own paragraph order). Every
//! other field is a `Label: value` line, except:
//! - Multiple choice options are `"A. text"` … `"E. text"` lines.
//! - "Benar/Salah Ganda" statements are `"Statement N: text"` followed by a lone `Benar`/`Salah`
//!   line.
//! - "Menjodohkan"/"Menjodohkan Gambar" pairs are `"Pair N:"` followed by either
//!   `"Pertanyaan: left"` + `"Jawaban: right"` (text pairs), or a paragraph containing the two
//!   side-by-side images (image pairs — see `infrastructure::docx_reader` doc comment, which
//!   confirms both images of one pair land in a single paragraph in a real export).
//! - "Urutkan" steps are `"Step N: text"` lines, already in their correct order.
//!
//! ## Rich content (images / tables / formulas) — Markdown+LaTeX alignment
//!
//! Question content everywhere else in the app (manual authoring, student-facing rendering —
//! see `frontend/composables/useQuestionMarkdown.ts`) is "Markdown + LaTeX" plain text. The
//! `.docx` import is aligned with that format as follows, with zero new required syntax for
//! authors who don't need it:
//! - **LaTeX typed directly as text** (e.g. `Luas lingkaran adalah $\pi r^2$`) already passes
//!   through untouched — `infrastructure::docx_reader` preserves `<w:t>` text verbatim, so
//!   nothing about import needed to change for hand-typed formulas.
//! - **A paragraph containing one or more images, encountered outside a `"Pair N:"` block**
//!   (i.e. not a "Menjodohkan Gambar" pair — see `in_pair` below), is treated as inline
//!   stimulus content: each image becomes a `![gambar](qimg://N)` Markdown reference appended
//!   to this question's `stimulus`, where `qimg://N` is a placeholder the caller
//!   (`application::question_import_service`) resolves to a real `data:` URI (preview) or a
//!   persisted `/uploads/...` URL (commit) — the exact same dual-resolution pattern already
//!   used for `MatchingImage` pairs, just applied to free-standing images instead of pairs.
//!   Previously any such paragraph was rejected outright ("Pasangan menjodohkan gambar
//!   memerlukan tepat 2 gambar") even for a plain diagram in a question's body — that was a
//!   parser bug, not an intentional restriction.
//! - **An explicit `"Stimulus:"` line** appends its text verbatim to the same accumulating
//!   stimulus buffer (in document order relative to any inline images), for authors who want a
//!   reading passage/context block above the question without relying on image paragraphs.
//! - **Tables** are NOT auto-converted from native Word tables (`<w:tbl>`) — `docx_reader` only
//!   ever reads `<w:p>` paragraph text, not table row/column structure, and reconstructing that
//!   structure is a larger, separate parser change nobody has asked for yet. Authors who need a
//!   table in an imported question should type it directly as a GFM Markdown table (e.g. via a
//!   `"Stimulus:"` line, pipes and all) — Word will preserve the literal `|` characters as
//!   plain text exactly like any other typed text, and the renderer already supports it.
//!
//! ## Two known, deliberate gaps versus the source format
//!
//! - **`Poin Soal:`** is parsed and validated (must be a positive integer) but never
//!   persisted. This schema has no per-question point-weight column, and every question is
//!   graded uniformly today (see `application::tryout_service`) — writing the number to a
//!   column that has zero effect on scoring would be more misleading than just not writing it.
//!   the model doesn't yet support point-weighted question scoring;
//! - **`Tingkat Kesulitan: Spesial`** is rejected with a clear message rather than silently
//!   remapped to `Sulit`. `domain::question::Difficulty` only has three tiers
//!   (Mudah/Sedang/Sulit); silently reassigning an author's explicit "Spesial" label to "Sulit"
//!   would misrepresent their intent. Adding a fourth tier is a larger, separate change (the
//!   enum is matched across the CHECK constraint, several UI badges, and filters) that nobody
//!   has asked for yet.
//! - **`Tipe esai: [kosongkan]`** (the spec's instruction to leave `Jawaban:` blank for essay
//!   questions) can't be honored: every question, of every type, needs a non-empty
//!   `correct_answer` today (`question_service::validate_answerable`) — essay's is the sample
//!   answer / grading rubric a student sees after submitting. An essay block with no
//!   `Jawaban:` line is reported as an issue asking the author to add one, rather than
//!   fabricating a placeholder rubric.

use crate::domain::question::{Difficulty, QuestionType};
use crate::infrastructure::docx_reader::DocxParagraph;

/// One image side of a "Menjodohkan Gambar" pair, still as raw bytes — turning these into a
/// stored `/uploads/...` URL is an I/O side effect intentionally left to the caller (see
/// module doc). `ParsedAnswer::ImagePairs` carries a `Vec` of these.
#[derive(Clone)]
pub struct ImagePair {
    pub left_bytes: Vec<u8>,
    pub left_mime: String,
    pub right_bytes: Vec<u8>,
    pub right_mime: String,
}

pub enum ParsedAnswer {
    /// Ready to drop straight into `CreateQuestionInput::{options, correct_answer}` as-is —
    /// every type except `MatchingImage`.
    Plain { options: Option<Vec<String>>, correct_answer: String },
    /// `MatchingImage` only: still needs each side's bytes resolved to a URL (persisted, or a
    /// throwaway `data:` URI for a preview) before `options`/`correct_answer` can be built.
    ImagePairs(Vec<ImagePair>),
}

pub struct ParsedImportQuestion {
    /// 1-based position among detected question blocks in the document (for "Soal #N" UI
    /// messages only — not stored, not used for ordering beyond document order).
    pub question_index: usize,
    /// The `[no_soal]` label as the author typed it, e.g. `"7"` — for display only.
    pub no_soal: String,
    pub question_type: QuestionType,
    pub question_text: String,
    pub difficulty: Difficulty,
    pub explanation: String,
    pub answer: ParsedAnswer,
    /// Markdown stimulus text accumulated from `"Stimulus:"` lines and/or inline
    /// `![gambar](qimg://N)` placeholders (see module doc's "Rich content" section) — `None`
    /// if the block had neither. Placeholder `N`s index into `stimulus_images`.
    pub stimulus: Option<String>,
    /// Raw bytes + MIME of every inline (non-pair) image referenced by a `qimg://N`
    /// placeholder in `stimulus`, in the order the placeholders were assigned.
    pub stimulus_images: Vec<(Vec<u8>, String)>,
}

pub struct ImportIssue {
    pub question_index: usize,
    pub no_soal: Option<String>,
    pub message: String,
}

pub struct ImportParseResult {
    /// Blocks that parsed and validated cleanly, ready to be created.
    pub questions: Vec<ParsedImportQuestion>,
    /// Blocks that failed to parse or validate, with a human-readable reason — these are
    /// excluded from `questions` entirely; nothing partial or guessed is ever included here.
    pub issues: Vec<ImportIssue>,
}

/// Entry point: turns the paragraphs of an already-read `.docx` into `ImportParseResult`. See
/// `infrastructure::docx_reader::read_docx_paragraphs` for turning raw file bytes into
/// paragraphs (kept separate so this module never has to know anything about zip/XML).
pub fn parse_paragraphs(paragraphs: &[DocxParagraph]) -> ImportParseResult {
    let blocks = split_into_blocks(paragraphs);
    let mut questions = Vec::new();
    let mut issues = Vec::new();

    if blocks.is_empty() {
        issues.push(ImportIssue {
            question_index: 0,
            no_soal: None,
            message:
                "Tidak ditemukan soal dalam format yang dikenali. Pastikan setiap soal diawali baris seperti \"1. Pertanyaan...\"."
                    .to_string(),
        });
        return ImportParseResult { questions, issues };
    }

    for block in &blocks {
        match parse_block(block) {
            Ok(q) => questions.push(q),
            Err(issue) => issues.push(issue),
        }
    }

    ImportParseResult { questions, issues }
}

struct RawBlock<'a> {
    question_index: usize,
    no_soal: String,
    question_text: String,
    /// Everything after the block's opening line, up to (not including) the next block.
    paragraphs: Vec<&'a DocxParagraph>,
}

fn split_into_blocks(paragraphs: &[DocxParagraph]) -> Vec<RawBlock<'_>> {
    let mut blocks: Vec<RawBlock> = Vec::new();
    let mut index = 0usize;
    for p in paragraphs {
        let text = p.text.trim();
        if text.is_empty() && p.images.is_empty() {
            continue; // pure spacing paragraph
        }
        if let Some((no_soal, question_text)) = parse_block_start(text) {
            index += 1;
            blocks.push(RawBlock { question_index: index, no_soal, question_text, paragraphs: Vec::new() });
            continue;
        }
        if let Some(last) = blocks.last_mut() {
            last.paragraphs.push(p);
        }
        // Otherwise: still inside the format-legend preamble, before the first real question —
        // deliberately ignored rather than reported as an issue.
    }
    blocks
}

/// Matches a block-opening line: leading digits, then `.`, then non-empty remaining text —
/// e.g. `"7. Hubungkan nama negara dan ibukota?"` -> `("7", "Hubungkan nama negara dan ibukota?")`.
fn parse_block_start(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    let digits_end = trimmed.find(|c: char| !c.is_ascii_digit())?;
    if digits_end == 0 {
        return None;
    }
    let (num, rest) = trimmed.split_at(digits_end);
    let rest = rest.strip_prefix('.')?;
    let text = rest.trim();
    if text.is_empty() {
        return None;
    }
    Some((num.to_string(), text.to_string()))
}

/// Case-insensitive `"Label:"` prefix match, returning the trimmed remainder.
///
/// Uses `str::get` (not `split_at`/direct byte-range indexing) to probe the prefix: `label` is
/// always ASCII, but `line` is arbitrary author-typed prose (very often containing multi-byte
/// UTF-8 — curly “quotes”, en/em dashes, accented letters — routinely inserted by Word's
/// autocorrect). If `label.len()` doesn't land on a char boundary in `trimmed` — which happens
/// whenever a non-matching line has such a character straddling that exact byte offset —
/// `split_at`/`[..N]` would panic. `get(..N)` returns `None` instead, so a non-matching line is
/// just correctly reported as "no match" like any other.
fn strip_label<'a>(line: &'a str, label: &str) -> Option<&'a str> {
    let trimmed = line.trim_start();
    let head = trimmed.get(..label.len())?;
    if head.eq_ignore_ascii_case(label) {
        Some(trimmed[label.len()..].trim())
    } else {
        None
    }
}

/// Matches `"{prefix} <digits>:"` (case-insensitive on the prefix), e.g. `"Pair 1:"`,
/// `"Statement 2: text"`, `"Step 3: Hipotesis"` — returns whatever trails the colon, trimmed
/// (may be empty, as for a bare `"Pair 1:"`). See `strip_label` doc comment for why the prefix
/// probe uses `get(..N)` rather than direct byte-range slicing.
fn strip_numbered_label<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let trimmed = line.trim_start();
    let head = trimmed.get(..prefix.len())?;
    if !head.eq_ignore_ascii_case(prefix) {
        return None;
    }
    let after_prefix = trimmed[prefix.len()..].trim_start();
    let digits_end = after_prefix.find(|c: char| !c.is_ascii_digit())?;
    if digits_end == 0 {
        return None;
    }
    let rest = after_prefix[digits_end..].strip_prefix(':')?;
    Some(rest.trim())
}

/// Matches an option line `"A. text"` … `"E. text"`.
fn parse_option_line(line: &str) -> Option<(char, String)> {
    let trimmed = line.trim();
    let mut chars = trimmed.chars();
    let letter = chars.next()?;
    if !('A'..='E').contains(&letter) {
        return None;
    }
    let rest = chars.as_str().strip_prefix('.')?;
    let text = rest.trim();
    if text.is_empty() {
        return None;
    }
    Some((letter, text.to_string()))
}

fn parse_question_type_label(s: &str) -> Option<QuestionType> {
    match s.trim().to_lowercase().as_str() {
        "pilihan ganda" => Some(QuestionType::MultipleChoice),
        "pilihan ganda kompleks" | "pilihan ganda komplex" => Some(QuestionType::ComplexMultiple),
        "benar salah" => Some(QuestionType::TrueFalse),
        "benar salah ganda" | "pilihan benar salah ganda" => Some(QuestionType::TrueFalseComplex),
        "isian singkat" | "isian_singkat" => Some(QuestionType::ShortAnswer),
        "esai" => Some(QuestionType::Essay),
        "menjodohkan" => Some(QuestionType::Matching),
        "menjodohkan gambar" => Some(QuestionType::MatchingImage),
        "urutkan" => Some(QuestionType::Ordering),
        _ => None,
    }
}

/// "Spesial" is deliberately NOT mapped here — see module doc.
fn parse_difficulty_label(s: &str) -> Option<Difficulty> {
    match s.trim().to_lowercase().as_str() {
        "mudah" => Some(Difficulty::Easy),
        "sedang" => Some(Difficulty::Medium),
        "sulit" => Some(Difficulty::Hard),
        _ => None,
    }
}

fn parse_block(block: &RawBlock) -> Result<ParsedImportQuestion, ImportIssue> {
    let issue = |message: String| ImportIssue {
        question_index: block.question_index,
        no_soal: Some(block.no_soal.clone()),
        message,
    };

    let mut question_type: Option<QuestionType> = None;
    let mut difficulty: Option<Difficulty> = None;
    let mut explanation = String::new();
    let mut top_jawaban: Option<String> = None;
    let mut mc_options: Vec<(char, String)> = Vec::new();
    let mut tf_statements: Vec<(String, String)> = Vec::new();
    let mut pending_statement: Option<String> = None;
    let mut match_pairs: Vec<(String, String)> = Vec::new();
    let mut image_pairs: Vec<ImagePair> = Vec::new();
    let mut ordering_steps: Vec<String> = Vec::new();
    let mut pending_pertanyaan: Option<String> = None;
    let mut in_pair = false;
    let mut stimulus_text = String::new();
    let mut stimulus_images: Vec<(Vec<u8>, String)> = Vec::new();

    for p in &block.paragraphs {
        let line = p.text.trim();

        if let Some(v) = strip_label(line, "Tipe:") {
            question_type = Some(parse_question_type_label(v).ok_or_else(|| issue(format!("Tipe soal '{v}' tidak dikenali.")))?);
            continue;
        }
        if let Some(v) = strip_label(line, "Tingkat Kesulitan:") {
            difficulty = Some(
                parse_difficulty_label(v)
                    .ok_or_else(|| issue(format!("Tingkat Kesulitan '{v}' belum didukung sistem saat ini — gunakan Mudah/Sedang/Sulit.")))?,
            );
            continue;
        }
        if let Some(v) = strip_label(line, "Poin Soal:") {
            if v.trim().parse::<u32>().is_err() {
                return Err(issue(format!("Poin Soal '{v}' harus berupa angka.")));
            }
            continue; // validated, intentionally not persisted — see module doc
        }
        if let Some(v) = strip_label(line, "Pembahasan:") {
            explanation = v.to_string();
            continue;
        }
        if let Some(v) = strip_label(line, "Stimulus:") {
            if !stimulus_text.is_empty() {
                stimulus_text.push('\n');
            }
            stimulus_text.push_str(v);
            continue;
        }
        if strip_numbered_label(line, "Pair").is_some() {
            in_pair = true;
            pending_pertanyaan = None;
            continue;
        }
        if let Some(v) = strip_label(line, "Pertanyaan:") {
            pending_pertanyaan = Some(v.to_string());
            continue;
        }
        if let Some(v) = strip_numbered_label(line, "Statement") {
            pending_statement = Some(v.to_string());
            continue;
        }
        if let Some(v) = strip_numbered_label(line, "Step") {
            ordering_steps.push(v.to_string());
            continue;
        }
        if let Some(v) = strip_label(line, "Jawaban:") {
            if in_pair {
                let left = pending_pertanyaan
                    .take()
                    .ok_or_else(|| issue("Field 'Jawaban:' pada pasangan menjodohkan muncul tanpa 'Pertanyaan:' sebelumnya.".to_string()))?;
                match_pairs.push((left, v.to_string()));
            } else {
                top_jawaban = Some(v.to_string());
            }
            continue;
        }
        if let Some((letter, text)) = parse_option_line(line) {
            mc_options.push((letter, text));
            continue;
        }
        if let Some(stmt) = pending_statement.take() {
            if line.eq_ignore_ascii_case("benar") {
                tf_statements.push((stmt, "Benar".to_string()));
            } else if line.eq_ignore_ascii_case("salah") {
                tf_statements.push((stmt, "Salah".to_string()));
            } else {
                return Err(issue(format!(
                    "Setelah 'Statement ...:' harus diikuti baris 'Benar' atau 'Salah', ditemukan: '{line}'."
                )));
            }
            continue;
        }
        if !p.images.is_empty() {
            if in_pair {
                // Inside a "Pair N:" block — a "Menjodohkan Gambar" pair, which needs exactly
                // the two side-by-side images docx_reader confirmed land in one paragraph.
                if p.images.len() < 2 {
                    return Err(issue("Pasangan menjodohkan gambar memerlukan tepat 2 gambar (kiri dan kanan) per 'Pair'.".to_string()));
                }
                let (left_bytes, left_mime) = p.images[0].clone();
                let (right_bytes, right_mime) = p.images[1].clone();
                image_pairs.push(ImagePair { left_bytes, left_mime, right_bytes, right_mime });
            } else {
                // Not part of a matching pair — a plain diagram/graphic embedded in the
                // question body (e.g. a chart the question refers to). Each image becomes an
                // inline Markdown reference in the accumulating stimulus (see module doc's
                // "Rich content" section) instead of being rejected outright.
                for (bytes, mime) in &p.images {
                    let placeholder_idx = stimulus_images.len();
                    stimulus_images.push((bytes.clone(), mime.clone()));
                    if !stimulus_text.is_empty() {
                        stimulus_text.push('\n');
                    }
                    stimulus_text.push_str(&format!("![gambar](qimg://{placeholder_idx})"));
                }
            }
            continue;
        }
        // Anything else (stray blank lines, decorative text) is silently ignored rather than
        // treated as a hard parse error — keeps the parser forgiving of minor formatting slack.
    }

    let question_type = question_type.ok_or_else(|| issue("Field 'Tipe:' tidak ditemukan.".to_string()))?;
    let difficulty = difficulty.ok_or_else(|| issue("Field 'Tingkat Kesulitan:' tidak ditemukan.".to_string()))?;

    let answer = match question_type {
        QuestionType::MultipleChoice => {
            if mc_options.len() < 2 {
                return Err(issue("Soal pilihan ganda memerlukan minimal 2 opsi (baris 'A. ...', 'B. ...', dst).".to_string()));
            }
            let letter_raw = top_jawaban.as_deref().ok_or_else(|| issue("Field 'Jawaban:' tidak ditemukan.".to_string()))?;
            let letter = letter_raw.trim().chars().next().unwrap_or(' ').to_ascii_uppercase();
            let text = mc_options
                .iter()
                .find(|(c, _)| *c == letter)
                .map(|(_, t)| t.clone())
                .ok_or_else(|| issue(format!("Jawaban '{letter_raw}' tidak cocok dengan opsi manapun (A-E).")))?;
            ParsedAnswer::Plain { options: Some(mc_options.iter().map(|(_, t)| t.clone()).collect()), correct_answer: text }
        }
        QuestionType::ComplexMultiple => {
            if mc_options.len() < 2 {
                return Err(issue("Soal pilihan ganda kompleks memerlukan minimal 2 opsi.".to_string()));
            }
            let letters_raw = top_jawaban.as_deref().ok_or_else(|| issue("Field 'Jawaban:' tidak ditemukan.".to_string()))?;
            let mut answers = Vec::new();
            for part in letters_raw.split(',') {
                let letter = part.trim().chars().next().unwrap_or(' ').to_ascii_uppercase();
                let text = mc_options
                    .iter()
                    .find(|(c, _)| *c == letter)
                    .map(|(_, t)| t.clone())
                    .ok_or_else(|| issue(format!("Jawaban '{letters_raw}' memuat huruf yang tidak cocok dengan opsi manapun (A-E).")))?;
                answers.push(text);
            }
            ParsedAnswer::Plain {
                options: Some(mc_options.iter().map(|(_, t)| t.clone()).collect()),
                correct_answer: answers.join(", "),
            }
        }
        QuestionType::TrueFalse => {
            let raw = top_jawaban.as_deref().ok_or_else(|| issue("Field 'Jawaban:' tidak ditemukan.".to_string()))?;
            let value = if raw.trim().eq_ignore_ascii_case("benar") {
                "Benar"
            } else if raw.trim().eq_ignore_ascii_case("salah") {
                "Salah"
            } else {
                return Err(issue(format!("Jawaban '{raw}' untuk tipe benar salah harus 'benar' atau 'salah'.")));
            };
            ParsedAnswer::Plain { options: Some(vec!["Benar".to_string(), "Salah".to_string()]), correct_answer: value.to_string() }
        }
        QuestionType::ShortAnswer => {
            let raw = top_jawaban
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| issue("Field 'Jawaban:' tidak ditemukan atau kosong.".to_string()))?;
            ParsedAnswer::Plain { options: None, correct_answer: raw.to_string() }
        }
        QuestionType::Essay => {
            let raw = top_jawaban.as_deref().map(str::trim).filter(|s| !s.is_empty()).ok_or_else(|| {
                issue(
                    "Tipe esai memerlukan contoh jawaban/rubrik — tambahkan baris 'Jawaban: ...' di bawah 'Tipe: esai' (sistem mewajibkan field ini agar soal bisa disimpan, meski format contoh menyebut field ini boleh dikosongkan)."
                        .to_string(),
                )
            })?;
            ParsedAnswer::Plain { options: None, correct_answer: raw.to_string() }
        }
        QuestionType::TrueFalseComplex => {
            if tf_statements.len() < 2 {
                return Err(issue("Soal benar/salah ganda memerlukan minimal 2 pernyataan ('Statement N: ...' diikuti 'Benar'/'Salah').".to_string()));
            }
            let options: Vec<String> = tf_statements.iter().map(|(t, _)| t.clone()).collect();
            let correct_answer = tf_statements.iter().map(|(_, a)| a.clone()).collect::<Vec<_>>().join(", ");
            ParsedAnswer::Plain { options: Some(options), correct_answer }
        }
        QuestionType::Matching => {
            if match_pairs.len() < 2 {
                return Err(issue("Soal menjodohkan memerlukan minimal 2 pasangan ('Pair N:' + 'Pertanyaan:' + 'Jawaban:').".to_string()));
            }
            let options: Vec<String> = match_pairs.iter().map(|(l, r)| format!("{}::{}", l.trim(), r.trim())).collect();
            let correct_answer = options.join(", ");
            ParsedAnswer::Plain { options: Some(options), correct_answer }
        }
        QuestionType::MatchingImage => {
            if image_pairs.len() < 2 {
                return Err(issue("Soal menjodohkan gambar memerlukan minimal 2 pasangan gambar ('Pair N:' diikuti 2 gambar).".to_string()));
            }
            ParsedAnswer::ImagePairs(image_pairs)
        }
        QuestionType::Ordering => {
            if ordering_steps.len() < 2 {
                return Err(issue("Soal urutkan memerlukan minimal 2 langkah ('Step N: ...').".to_string()));
            }
            let correct_answer = ordering_steps.join(", ");
            ParsedAnswer::Plain { options: Some(ordering_steps.clone()), correct_answer }
        }
    };

    Ok(ParsedImportQuestion {
        question_index: block.question_index,
        no_soal: block.no_soal.clone(),
        question_type,
        question_text: block.question_text.clone(),
        difficulty,
        explanation,
        answer,
        stimulus: if stimulus_text.trim().is_empty() { None } else { Some(stimulus_text.trim().to_string()) },
        stimulus_images,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_para(s: &str) -> DocxParagraph {
        DocxParagraph { text: s.to_string(), images: Vec::new() }
    }

    #[test]
    fn parses_multiple_choice() {
        let paras = vec![
            text_para("1. Apa ibu kota Indonesia ?"),
            text_para("Tipe: pilihan ganda"),
            text_para("A. Jakarta"),
            text_para("B. Bandung"),
            text_para("C. Surabaya"),
            text_para("D. Medan"),
            text_para("Jawaban: A"),
            text_para("Poin Soal: 1"),
            text_para("Tingkat Kesulitan: Mudah"),
            text_para("Pembahasan: Ini Pembahasan"),
        ];
        let result = parse_paragraphs(&paras);
        assert!(result.issues.is_empty(), "unexpected issues: {:?}", result.issues.iter().map(|i| &i.message).collect::<Vec<_>>());
        assert_eq!(result.questions.len(), 1);
        let q = &result.questions[0];
        assert_eq!(q.question_type, QuestionType::MultipleChoice);
        assert_eq!(q.explanation, "Ini Pembahasan");
        match &q.answer {
            ParsedAnswer::Plain { options, correct_answer } => {
                assert_eq!(
                    options.as_ref().unwrap(),
                    &vec!["Jakarta".to_string(), "Bandung".to_string(), "Surabaya".to_string(), "Medan".to_string()]
                );
                assert_eq!(correct_answer, "Jakarta");
            }
            _ => panic!("expected Plain"),
        }
    }

    #[test]
    fn parses_complex_multiple() {
        let paras = vec![
            text_para("2. Pilih yang provinsi?"),
            text_para("Tipe: pilihan ganda kompleks"),
            text_para("A. Jakarta"),
            text_para("B. Bandung"),
            text_para("C. Jawa Barat"),
            text_para("D. Jawa Tengah"),
            text_para("Jawaban: A, C"),
            text_para("Tingkat Kesulitan: Mudah"),
        ];
        let result = parse_paragraphs(&paras);
        assert!(result.issues.is_empty());
        match &result.questions[0].answer {
            ParsedAnswer::Plain { correct_answer, .. } => assert_eq!(correct_answer, "Jakarta, Jawa Barat"),
            _ => panic!(),
        }
    }

    #[test]
    fn parses_true_false() {
        let paras = vec![
            text_para("3. Jakarta adalah ibu kota indonesia ?"),
            text_para("Tipe: benar salah"),
            text_para("Jawaban: benar"),
            text_para("Tingkat Kesulitan: Mudah"),
        ];
        let result = parse_paragraphs(&paras);
        assert!(result.issues.is_empty());
        match &result.questions[0].answer {
            ParsedAnswer::Plain { correct_answer, options } => {
                assert_eq!(correct_answer, "Benar");
                assert_eq!(options.as_ref().unwrap(), &vec!["Benar".to_string(), "Salah".to_string()]);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parses_true_false_complex() {
        let paras = vec![
            text_para("4. Jakarta adalah ibu kota indonesia ?"),
            text_para("Tipe: benar salah ganda"),
            text_para("Statement 1: ibu kota indonesia jakarta"),
            text_para("Benar"),
            text_para("Statement 2: ibu kota indonesia semarang"),
            text_para("Salah"),
            text_para("Tingkat Kesulitan: Mudah"),
        ];
        let result = parse_paragraphs(&paras);
        assert!(result.issues.is_empty(), "{:?}", result.issues.iter().map(|i| &i.message).collect::<Vec<_>>());
        match &result.questions[0].answer {
            ParsedAnswer::Plain { options, correct_answer } => {
                assert_eq!(options.as_ref().unwrap().len(), 2);
                assert_eq!(correct_answer, "Benar, Salah");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parses_short_answer_and_essay() {
        let sa = vec![
            text_para("5. ... adalah ibu kota indonesia ?"),
            text_para("Tipe: isian singkat"),
            text_para("Jawaban: jakarta"),
            text_para("Tingkat Kesulitan: Mudah"),
        ];
        let r1 = parse_paragraphs(&sa);
        assert!(r1.issues.is_empty());

        // Essay with NO Jawaban line (matching the spec's literal example) must be reported as
        // an issue, not silently created with a fabricated rubric.
        let essay_missing = vec![
            text_para("6. jelaskan tentang ibu kota jakarta ?"),
            text_para("Tipe: esai"),
            text_para("Tingkat Kesulitan: Mudah"),
        ];
        let r2 = parse_paragraphs(&essay_missing);
        assert!(r2.questions.is_empty());
        assert_eq!(r2.issues.len(), 1);
    }

    #[test]
    fn parses_matching_pairs() {
        let paras = vec![
            text_para("7. Hubungkan planet dan jaraknya?"),
            text_para("Tipe: menjodohkan"),
            text_para("Tingkat Kesulitan: Mudah"),
            text_para("Pair 1:"),
            text_para("Pertanyaan: Merkurius"),
            text_para("Jawaban: Planet Terdekat"),
            text_para("Pair 2:"),
            text_para("Pertanyaan: Neptunus"),
            text_para("Jawaban: Planet Terjauh"),
        ];
        let result = parse_paragraphs(&paras);
        assert!(result.issues.is_empty(), "{:?}", result.issues.iter().map(|i| &i.message).collect::<Vec<_>>());
        match &result.questions[0].answer {
            ParsedAnswer::Plain { options, correct_answer } => {
                assert_eq!(options.as_ref().unwrap(), &vec!["Merkurius::Planet Terdekat".to_string(), "Neptunus::Planet Terjauh".to_string()]);
                assert_eq!(correct_answer, "Merkurius::Planet Terdekat, Neptunus::Planet Terjauh");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parses_matching_image_pairs() {
        let img_para = DocxParagraph {
            text: " ".to_string(),
            images: vec![(vec![1, 2, 3], "image/jpeg".to_string()), (vec![4, 5, 6], "image/jpeg".to_string())],
        };
        let paras = vec![
            text_para("8. Cocokkan gambar"),
            text_para("Tipe: menjodohkan gambar"),
            text_para("Tingkat Kesulitan: Mudah"),
            text_para("Pair 1:"),
            img_para.clone(),
            text_para("Pair 2:"),
            img_para,
        ];
        let result = parse_paragraphs(&paras);
        assert!(result.issues.is_empty(), "{:?}", result.issues.iter().map(|i| &i.message).collect::<Vec<_>>());
        match &result.questions[0].answer {
            ParsedAnswer::ImagePairs(pairs) => assert_eq!(pairs.len(), 2),
            _ => panic!("expected ImagePairs"),
        }
    }

    #[test]
    fn inline_image_outside_a_pair_becomes_stimulus_markdown() {
        // Regression test for the parser bug described in the module doc's "Rich content"
        // section: a single diagram embedded in a question's body (NOT inside a "Pair N:"
        // block) used to be hard-rejected ("...memerlukan tepat 2 gambar") — it must now
        // become an inline `![gambar](qimg://0)` reference in `stimulus` instead.
        let diagram_para = DocxParagraph { text: String::new(), images: vec![(vec![9, 9, 9], "image/png".to_string())] };
        let paras = vec![
            text_para("10. Perhatikan diagram berikut, berapa nilai x?"),
            text_para("Tipe: isian singkat"),
            diagram_para,
            text_para("Jawaban: 5"),
            text_para("Tingkat Kesulitan: Sedang"),
        ];
        let result = parse_paragraphs(&paras);
        assert!(result.issues.is_empty(), "{:?}", result.issues.iter().map(|i| &i.message).collect::<Vec<_>>());
        let q = &result.questions[0];
        assert_eq!(q.stimulus.as_deref(), Some("![gambar](qimg://0)"));
        assert_eq!(q.stimulus_images.len(), 1);
    }

    #[test]
    fn explicit_stimulus_label_is_captured() {
        let paras = vec![
            text_para("11. Berdasarkan bacaan di atas, apa kesimpulannya?"),
            text_para("Tipe: isian singkat"),
            text_para("Stimulus: Suatu populasi bakteri berkembang biak dua kali lipat setiap jam."),
            text_para("Jawaban: eksponensial"),
            text_para("Tingkat Kesulitan: Sedang"),
        ];
        let result = parse_paragraphs(&paras);
        assert!(result.issues.is_empty(), "{:?}", result.issues.iter().map(|i| &i.message).collect::<Vec<_>>());
        assert_eq!(
            result.questions[0].stimulus.as_deref(),
            Some("Suatu populasi bakteri berkembang biak dua kali lipat setiap jam.")
        );
    }

    #[test]
    fn parses_ordering() {
        let paras = vec![
            text_para("9. Urutkan langkah metode ilmiah"),
            text_para("Tipe: Urutkan"),
            text_para("Tingkat Kesulitan: Mudah"),
            text_para("Step 1: Observasi"),
            text_para("Step 2: Rumusan Masalah"),
            text_para("Step 3: Hipotesis"),
        ];
        let result = parse_paragraphs(&paras);
        assert!(result.issues.is_empty(), "{:?}", result.issues.iter().map(|i| &i.message).collect::<Vec<_>>());
        match &result.questions[0].answer {
            ParsedAnswer::Plain { options, correct_answer } => {
                assert_eq!(options.as_ref().unwrap(), &vec!["Observasi".to_string(), "Rumusan Masalah".to_string(), "Hipotesis".to_string()]);
                assert_eq!(correct_answer, "Observasi, Rumusan Masalah, Hipotesis");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn rejects_unknown_difficulty() {
        let paras = vec![
            text_para("1. Contoh"),
            text_para("Tipe: isian singkat"),
            text_para("Jawaban: x"),
            text_para("Tingkat Kesulitan: Spesial"),
        ];
        let result = parse_paragraphs(&paras);
        assert!(result.questions.is_empty());
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].message.contains("Spesial") || result.issues[0].message.contains("belum didukung"));
    }

    #[test]
    fn preamble_without_numbered_lines_is_ignored() {
        let paras = vec![text_para("General Format:"), text_para("Tipe: [pilihan]"), text_para("pilihan ganda")];
        let result = parse_paragraphs(&paras);
        assert!(result.questions.is_empty());
        assert_eq!(result.issues.len(), 1); // "no questions found at all"
    }
}
