//! Minimal `.docx` reader for the bulk question-import feature (see
//! `application::question_import` for the DSL parser this feeds). A `.docx` is a zip archive
//! of XML; this module only needs two things out of it:
//!
//! 1. Every paragraph's plain text, with all of its `<w:t>` runs concatenated in document
//!    order. Word routinely fragments a single visible line of text across several `<w:r>`
//!    runs (confirmed empirically against a real sample export — even short labels like
//!    "Tipe: " land split across 2-3 runs), so reading only the first run's text would silently
//!    truncate or garble almost every line.
//! 2. Any image(s) embedded in a paragraph (`<a:blip r:embed="rIdN">`), resolved to raw bytes
//!    — needed both for "Menjodohkan Gambar" pairs, where a pair's two images live side-by-side
//!    in one paragraph (confirmed against a real sample export: both `<a:blip>` tags sit inside
//!    the same `<w:p>...</w:p>`), and for a single free-standing image in a question's body,
//!    which `question_import` now turns into an inline Markdown stimulus reference instead of
//!    rejecting (see that module's doc comment, "Rich content" section).
//!
//! Deliberately hand-rolled instead of using a general XML crate (e.g. `quick-xml`): the exact
//! subset of OOXML shapes needed here is small and was verified byte-for-byte against a real
//! sample file, and a purpose-built scanner avoids betting on getting an unfamiliar crate's
//! version-specific API exactly right with no compiler available in this environment to check
//! against. The `zip` crate is still used for the archive itself — its API is small and has
//! been stable for years.

use std::collections::HashMap;
use std::io::Read;

use crate::error::{AppError, AppResult};

pub struct DocxParagraph {
    /// Concatenated text of every `<w:t>` run in this paragraph, in document order. A
    /// `<w:tab/>` becomes a literal space and a `<w:br/>` becomes `\n` so tab/line-broken
    /// content doesn't get glued together; both are rare in this format but cheap to handle.
    pub text: String,
    /// Raw bytes + MIME type of every image referenced from this paragraph, in document
    /// order.
    pub images: Vec<(Vec<u8>, String)>,
}

/// Reads a `.docx` file's `word/document.xml` into an ordered list of paragraphs.
pub fn read_docx_paragraphs(bytes: &[u8]) -> AppResult<Vec<DocxParagraph>> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| AppError::Validation(format!("file bukan .docx yang valid: {e}")))?;

    let document_xml = read_zip_entry_utf8(&mut archive, "word/document.xml")
        .ok_or_else(|| AppError::Validation("file .docx tidak memiliki word/document.xml — pastikan ini file Word (.docx) yang valid".to_string()))?;
    let rels_xml = read_zip_entry_utf8(&mut archive, "word/_rels/document.xml.rels").unwrap_or_default();

    // Map relationship id ("rId6") -> media target path ("media/image1.jpeg"), then eagerly
    // load every referenced media file's raw bytes once (there are only ever a handful of
    // images in a realistic question-bank import, so no need for lazy/streaming access).
    let rel_targets = parse_relationship_targets(&rels_xml);
    let mut media_cache: HashMap<String, Vec<u8>> = HashMap::new();
    for target in rel_targets.values() {
        let path = format!("word/{target}");
        if let Some(data) = read_zip_entry_bytes(&mut archive, &path) {
            media_cache.insert(target.clone(), data);
        }
    }

    Ok(split_paragraphs(&document_xml)
        .into_iter()
        .map(|p_xml| extract_paragraph(p_xml, &rel_targets, &media_cache))
        .collect())
}

fn read_zip_entry_utf8<R: Read + std::io::Seek>(archive: &mut zip::ZipArchive<R>, name: &str) -> Option<String> {
    let mut file = archive.by_name(name).ok()?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).ok()?;
    Some(buf)
}

fn read_zip_entry_bytes<R: Read + std::io::Seek>(archive: &mut zip::ZipArchive<R>, name: &str) -> Option<Vec<u8>> {
    let mut file = archive.by_name(name).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;
    Some(buf)
}

/// `word/_rels/document.xml.rels` is a flat, non-nested list of
/// `<Relationship Id=".." Target=".."/>` elements — a plain attribute scan is enough.
fn parse_relationship_targets(rels_xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut rest = rels_xml;
    while let Some(rel_idx) = rest.find("<Relationship ") {
        let tag_start = &rest[rel_idx..];
        let Some(gt) = tag_start.find('>') else { break };
        let tag = &tag_start[..gt];
        if let (Some(id), Some(target)) = (extract_attr(tag, "Id"), extract_attr(tag, "Target")) {
            map.insert(id, target);
        }
        rest = &tag_start[gt + 1..];
    }
    map
}

fn guess_image_mime(target: &str) -> &'static str {
    let lower = target.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".bmp") {
        "image/bmp"
    } else {
        "application/octet-stream"
    }
}

/// Splits `word/document.xml`'s raw text into one string slice per `<w:p>...</w:p>` element
/// (`w:p` never nests in OOXML, so a "find the next matching close tag" scan — no depth
/// counting needed — is correct). A bare `<w:p/>` (empty paragraph) yields an empty slice.
fn split_paragraphs(xml: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut search_from = 0usize;
    let bytes = xml.as_bytes();
    while let Some(rel_idx) = xml[search_from..].find("<w:p") {
        let start = search_from + rel_idx;
        // Distinguish a real `<w:p>`/`<w:p ...>` from `<w:pPr>`, `<w:pStyle>`, etc. — the
        // character right after "<w:p" must be whitespace, '>', or '/' for this to be the
        // paragraph element itself.
        let next = bytes.get(start + 4).copied();
        if !matches!(next, Some(b' ') | Some(b'\t') | Some(b'\r') | Some(b'\n') | Some(b'>') | Some(b'/')) {
            search_from = start + 4;
            continue;
        }
        let Some(gt_rel) = xml[start..].find('>') else { break };
        let open_tag_end = start + gt_rel; // index of '>'
        let opening = &xml[start..=open_tag_end];
        if opening.trim_end().ends_with("/>") {
            // Self-closing, e.g. <w:p/> or <w:p w:rsidR="..."/> — no body.
            result.push("");
            search_from = open_tag_end + 1;
            continue;
        }
        if let Some(close_rel) = xml[open_tag_end..].find("</w:p>") {
            let body_start = open_tag_end + 1;
            let body_end = open_tag_end + close_rel;
            result.push(&xml[body_start..body_end]);
            search_from = body_end + "</w:p>".len();
        } else {
            break; // malformed / truncated — stop rather than risk an infinite loop
        }
    }
    result
}

/// Scans one paragraph's inner XML (everything between `<w:p ...>` and `</w:p>`) for `<w:t>`
/// text, `<w:tab/>`/`<w:br/>` whitespace, and `<a:blip r:embed="...">` image references —
/// in document order, regardless of how deeply the tag is nested inside `<w:r>`/`<w:drawing>`/
/// `<pic:pic>` wrappers (those wrapper tags are simply skipped over one character at a time).
fn extract_paragraph(
    p_xml: &str,
    rel_targets: &HashMap<String, String>,
    media_cache: &HashMap<String, Vec<u8>>,
) -> DocxParagraph {
    let mut text = String::new();
    let mut images = Vec::new();
    let mut rest = p_xml;

    loop {
        let Some(lt) = rest.find('<') else { break };
        if lt > 0 {
            rest = &rest[lt..];
        }
        if rest.starts_with("<w:t") && matches!(rest.as_bytes().get(4), Some(b' ') | Some(b'>') | Some(b'/')) {
            let Some(gt) = rest.find('>') else { break };
            let opening = &rest[..gt];
            if opening.trim_end().ends_with('/') {
                rest = &rest[gt + 1..]; // <w:t/> — empty text
                continue;
            }
            let after_open = &rest[gt + 1..];
            if let Some(close) = after_open.find("</w:t>") {
                text.push_str(&xml_unescape(&after_open[..close]));
                rest = &after_open[close + "</w:t>".len()..];
            } else {
                rest = after_open;
            }
        } else if rest.starts_with("<w:tab") && matches!(rest.as_bytes().get(6), Some(b' ') | Some(b'>') | Some(b'/')) {
            text.push(' ');
            rest = advance_past_tag(rest);
        } else if rest.starts_with("<w:br") && matches!(rest.as_bytes().get(5), Some(b' ') | Some(b'>') | Some(b'/')) {
            text.push('\n');
            rest = advance_past_tag(rest);
        } else if rest.starts_with("<a:blip") && matches!(rest.as_bytes().get(7), Some(b' ') | Some(b'>') | Some(b'/')) {
            if let Some(gt) = rest.find('>') {
                let opening = &rest[..gt];
                if let Some(rid) = extract_attr(opening, "r:embed") {
                    if let Some(target) = rel_targets.get(&rid) {
                        if let Some(data) = media_cache.get(target) {
                            images.push((data.clone(), guess_image_mime(target).to_string()));
                        }
                    }
                }
                rest = &rest[gt + 1..];
            } else {
                break;
            }
        } else {
            rest = &rest[1..];
        }
    }

    DocxParagraph { text, images }
}

/// Skips past a single tag (`<tag ...>` or `<tag ... />`), returning the remainder of the
/// string right after its closing `>`.
fn advance_past_tag(s: &str) -> &str {
    match s.find('>') {
        Some(gt) => &s[gt + 1..],
        None => "",
    }
}

/// Extracts `name="value"` from a raw tag string (the slice between `<` and the matching `>`),
/// XML-unescaping the value. Namespace-prefixed attribute names (e.g. `r:embed`) are matched
/// literally since that's exactly how they appear in the source.
fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    let needle = format!("{attr_name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(xml_unescape(&tag[start..end]))
}

/// Decodes the five predefined XML entities plus numeric character references
/// (`&#NN;` / `&#xHH;`). Unrecognized entities are left as-is (literal `&...;`) rather than
/// dropped, so malformed input degrades gracefully instead of silently losing content.
fn xml_unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp_idx) = rest.find('&') {
        out.push_str(&rest[..amp_idx]);
        let after_amp = &rest[amp_idx + 1..];
        let Some(semi_idx) = after_amp.find(';') else {
            out.push('&');
            rest = after_amp;
            break;
        };
        // A ';' more than ~12 chars after the '&' almost certainly isn't a real entity
        // (guards against runaway scanning through unrelated text containing a bare '&').
        if semi_idx > 12 {
            out.push('&');
            rest = after_amp;
            continue;
        }
        let entity = &after_amp[..semi_idx];
        let replacement = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                u32::from_str_radix(&entity[2..], 16).ok().and_then(char::from_u32)
            }
            _ if entity.starts_with('#') => entity[1..].parse::<u32>().ok().and_then(char::from_u32),
            _ => None,
        };
        match replacement {
            Some(c) => {
                out.push(c);
                rest = &after_amp[semi_idx + 1..];
            }
            None => {
                out.push('&');
                rest = after_amp;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unescapes_predefined_entities() {
        assert_eq!(xml_unescape("A &amp; B &lt;tag&gt; &quot;q&quot; &apos;a&apos;"), "A & B <tag> \"q\" 'a'");
    }

    #[test]
    fn unescapes_numeric_refs() {
        assert_eq!(xml_unescape("&#65;&#x42;"), "AB");
    }

    #[test]
    fn leaves_unknown_entities_literal() {
        assert_eq!(xml_unescape("A &foo; B"), "A &foo; B");
    }

    #[test]
    fn splits_simple_paragraphs() {
        let xml = r#"<w:body><w:p w:rsidR="1"><w:r><w:t>Hello</w:t></w:r></w:p><w:p/><w:p><w:r><w:t>World</w:t></w:r></w:p></w:body>"#;
        let paras = split_paragraphs(xml);
        assert_eq!(paras.len(), 3);
        assert!(paras[0].contains("Hello"));
        assert_eq!(paras[1], "");
        assert!(paras[2].contains("World"));
    }

    #[test]
    fn concatenates_fragmented_runs() {
        let p = r#"<w:r><w:t>Tipe</w:t></w:r><w:r><w:t xml:space="preserve">: </w:t></w:r><w:r><w:t>pilihan ganda</w:t></w:r>"#;
        let result = extract_paragraph(p, &HashMap::new(), &HashMap::new());
        assert_eq!(result.text, "Tipe: pilihan ganda");
    }

    #[test]
    fn tab_and_break_become_whitespace() {
        let p = r#"<w:r><w:t>A</w:t><w:tab/><w:t>B</w:t><w:br/><w:t>C</w:t></w:r>"#;
        let result = extract_paragraph(p, &HashMap::new(), &HashMap::new());
        assert_eq!(result.text, "A B\nC");
    }
}
