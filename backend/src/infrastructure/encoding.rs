//! Minimal, dependency-free base64 encoder — used only to inline extracted `.docx` image
//! bytes as a `data:` URI for the bulk question-import *preview* response (see
//! `application::question_import`), so the frontend can show a thumbnail of a
//! "Menjodohkan Gambar" pair before anything is actually written to disk. The real,
//! persisted URL (via `infrastructure::storage`) is only generated at commit time.
//!
//! Hand-rolled instead of pulling in the `base64` crate: this is a well-specified, ~20-line
//! algorithm, and avoids one more dependency whose exact API surface (which has changed
//! between major versions — `base64::encode()` vs the newer `Engine` trait) can't be
//! double-checked against a real compiler in this environment.
const B64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        out.push(B64_CHARS[(b0 >> 2) as usize] as char);
        out.push(B64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64_CHARS[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(B64_CHARS[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Builds a `data:{mime};base64,{...}` URI — directly usable as an `<img src>` without ever
/// touching disk.
pub fn to_data_uri(bytes: &[u8], mime: &str) -> String {
    format!("data:{mime};base64,{}", base64_encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }
}
