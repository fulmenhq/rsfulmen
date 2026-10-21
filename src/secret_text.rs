//! Hand-written secret scan for text that must be safe to log.
//!
//! This module does not use the `regex` crate, so signal panic text can use
//! it when the `logging` feature is off. The logging middleware keeps its own
//! patterns and is checked against this scan.

/// Replacement written over a recognized secret.
pub const REDACTION_REPLACEMENT: &str = "[REDACTED]";

/// Field names redacted in `key=value` / `key: value` text and in log fields.
///
/// Matching is case-insensitive. `x-api-key` is included so a hyphenated
/// header name is covered on the same list as `api_key`.
pub const SECRET_KEYS: &[&str] = &[
    "password",
    "passwd",
    "pwd",
    "token",
    "apikey",
    "api_key",
    "x-api-key",
    "authorization",
    "secret",
    "private_key",
    "privatekey",
    "credential",
    "pass",
    "cardnumber",
    "card_number",
    "cvv",
    "ssn",
    "accesstoken",
    "access_token",
    "refreshtoken",
    "refresh_token",
];

/// Redact secret assignments and bare token shapes.
///
/// Assignments run first. Bare tokens are `Bearer`, `Basic`, `ghp_`/`gho_`/
/// `ghu_`/`ghs_`/`ghr_`, `github_pat_`, and `AKIA` plus 16 uppercase letters
/// or digits. This does not invent a pattern for `sk-` prefixes.
pub fn redact_secret_text(raw: &str) -> String {
    redact_bare_tokens(&redact_assignments(raw))
}

fn redact_assignments(raw: &str) -> String {
    let chars: Vec<char> = raw.chars().collect();
    let lower: Vec<char> = raw.to_ascii_lowercase().chars().collect();
    let mut keys: Vec<Vec<char>> = SECRET_KEYS.iter().map(|k| k.chars().collect()).collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));

    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let mut val_start = None;
        for key in &keys {
            let end = i + key.len();
            if end < lower.len()
                && lower[i..end] == key[..]
                && (i == 0 || !lower[i - 1].is_ascii_alphanumeric())
            {
                if let Some(start) = assignment_value_start(&chars, end) {
                    let through_eol = key_is_authorization(key);
                    val_start = Some((start, through_eol));
                    break;
                }
            }
        }
        if let Some((start, through_eol)) = val_start {
            for c in &chars[i..start] {
                out.push(*c);
            }
            out.push_str(REDACTION_REPLACEMENT);
            i = if through_eol {
                // An unquoted authorization value runs to the end of the
                // line. Trailing prose on that line is removed with it.
                authorization_value_end(&chars, start)
            } else {
                assignment_value_end(&chars, start)
            };
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn assignment_value_start(chars: &[char], end: usize) -> Option<usize> {
    let mut k = end;
    if k < chars.len() && matches!(chars[k], '"' | '\'') {
        k += 1;
    }
    while k < chars.len() && chars[k].is_whitespace() {
        k += 1;
    }
    if k >= chars.len() || !matches!(chars[k], '=' | ':') {
        return None;
    }
    k += 1;
    while k < chars.len() && chars[k].is_whitespace() {
        k += 1;
    }
    Some(k)
}

fn key_is_authorization(key: &[char]) -> bool {
    key.iter().copied().eq("authorization".chars())
}

/// End of an authorization header value.
///
/// A quoted value stops at its closer, and a backslash escapes the next
/// character. An unquoted value runs through the end of the line, so
/// parameters and any trailing prose on that line are removed together.
fn authorization_value_end(chars: &[char], start: usize) -> usize {
    if start < chars.len() && matches!(chars[start], '"' | '\'') {
        return quoted_value_end(chars, start);
    }
    let mut j = start;
    while j < chars.len() && chars[j] != '\n' && chars[j] != '\r' {
        j += 1;
    }
    j
}

fn assignment_value_end(chars: &[char], start: usize) -> usize {
    if start < chars.len() && matches!(chars[start], '"' | '\'') {
        return quoted_value_end(chars, start);
    }
    let word_end = unquoted_word_end(chars, start);
    if is_auth_scheme(&chars[start..word_end]) {
        let mut next = word_end;
        while next < chars.len() && chars[next].is_whitespace() {
            next += 1;
        }
        if next > word_end && next < chars.len() {
            return unquoted_word_end(chars, next);
        }
    }
    word_end
}

fn quoted_value_end(chars: &[char], start: usize) -> usize {
    let quote = chars[start];
    let mut j = start + 1;
    while j < chars.len() {
        if chars[j] == '\\' && j + 1 < chars.len() {
            j += 2;
            continue;
        }
        if chars[j] == quote {
            return j + 1;
        }
        j += 1;
    }
    j
}

fn unquoted_word_end(chars: &[char], start: usize) -> usize {
    let mut j = start;
    while j < chars.len() && !chars[j].is_whitespace() {
        j += 1;
    }
    j
}

fn is_auth_scheme(word: &[char]) -> bool {
    const SCHEMES: [&str; 4] = ["bearer", "basic", "digest", "token"];
    SCHEMES.iter().any(|scheme| {
        word.len() == scheme.len()
            && word
                .iter()
                .zip(scheme.chars())
                .all(|(got, expected)| got.eq_ignore_ascii_case(&expected))
    })
}

fn redact_bare_tokens(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        if is_word_boundary(bytes, i) {
            if let Some(end) = match_bearer(bytes, i)
                .or_else(|| match_basic(bytes, i))
                .or_else(|| match_literal_prefix(bytes, i, b"github_pat_"))
                .or_else(|| match_gh_token(bytes, i))
                .or_else(|| match_akia(bytes, i))
            {
                out.push_str(REDACTION_REPLACEMENT);
                i = end;
                continue;
            }
        }
        let ch = raw[i..].chars().next().expect("i is a char boundary");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_word_boundary(bytes: &[u8], i: usize) -> bool {
    i == 0 || !is_word_byte(bytes[i - 1])
}

fn match_bearer(bytes: &[u8], i: usize) -> Option<usize> {
    if !starts_with_ignore_ascii_case(bytes, i, "bearer") {
        return None;
    }
    let mut j = i + "bearer".len();
    if j >= bytes.len() || !bytes[j].is_ascii_whitespace() {
        return None;
    }
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    let token = j;
    while j < bytes.len() && !bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    if j == token {
        None
    } else {
        Some(j)
    }
}

fn match_basic(bytes: &[u8], i: usize) -> Option<usize> {
    if !starts_with_ignore_ascii_case(bytes, i, "basic") {
        return None;
    }
    let mut j = i + "basic".len();
    if j >= bytes.len() || !bytes[j].is_ascii_whitespace() {
        return None;
    }
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    let token_start = j;
    while j < bytes.len()
        && (bytes[j].is_ascii_alphanumeric() || matches!(bytes[j], b'+' | b'/' | b'='))
    {
        j += 1;
    }
    let token = &bytes[token_start..j];
    let mixed = token.iter().any(|b| b.is_ascii_lowercase())
        && token.iter().any(|b| b.is_ascii_uppercase());
    let has_digit = token.iter().any(|b| b.is_ascii_digit());
    if token.len() >= 4 && (mixed || has_digit) {
        Some(j)
    } else {
        None
    }
}

fn match_literal_prefix(bytes: &[u8], i: usize, prefix: &[u8]) -> Option<usize> {
    if i + prefix.len() > bytes.len() || &bytes[i..i + prefix.len()] != prefix {
        return None;
    }
    let mut j = i + prefix.len();
    if j >= bytes.len() || !is_token_body(bytes[j]) {
        return None;
    }
    while j < bytes.len() && is_token_body(bytes[j]) {
        j += 1;
    }
    Some(j)
}

fn match_gh_token(bytes: &[u8], i: usize) -> Option<usize> {
    if i + 4 > bytes.len() {
        return None;
    }
    if bytes[i] != b'g' || bytes[i + 1] != b'h' || bytes[i + 3] != b'_' {
        return None;
    }
    if !matches!(bytes[i + 2], b'p' | b'o' | b'u' | b's' | b'r') {
        return None;
    }
    let mut j = i + 4;
    if j >= bytes.len() || !is_token_body(bytes[j]) {
        return None;
    }
    while j < bytes.len() && is_token_body(bytes[j]) {
        j += 1;
    }
    Some(j)
}

fn match_akia(bytes: &[u8], i: usize) -> Option<usize> {
    const LEN: usize = 4 + 16;
    if i + LEN > bytes.len() || &bytes[i..i + 4] != b"AKIA" {
        return None;
    }
    if !bytes[i + 4..i + LEN]
        .iter()
        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    {
        return None;
    }
    if i + LEN < bytes.len() && is_word_byte(bytes[i + LEN]) {
        return None;
    }
    Some(i + LEN)
}

fn is_token_body(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn starts_with_ignore_ascii_case(bytes: &[u8], i: usize, word: &str) -> bool {
    let word = word.as_bytes();
    i + word.len() <= bytes.len() && bytes[i..i + word.len()].eq_ignore_ascii_case(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_redacted(raw: &str, secret: &str) {
        let out = redact_secret_text(raw);
        assert!(!out.contains(secret), "{raw:?} -> {out}");
        assert!(out.contains(REDACTION_REPLACEMENT), "{out}");
    }

    #[test]
    fn assignments_cover_the_shared_key_names() {
        for key in ["passwd", "pwd", "private_key", "cvv", "ssn", "x-api-key"] {
            assert_redacted(&format!("panic {key}=hunter2"), "hunter2");
            assert_redacted(&format!(r#"panic {{"{key}":"hunter2"}}"#), "hunter2");
        }
    }

    #[test]
    fn bare_tokens_redact_without_an_assignment() {
        assert_redacted(
            "saw ghp_abcdefghijklmnopqrstuvwxyz0123456789 in the panic",
            "ghp_abcdefghijklmnopqrstuvwxyz0123456789",
        );
        assert_redacted("saw gho_abc123 in the panic", "gho_abc123");
        assert_redacted(
            "saw github_pat_11AAAAAAA_secret in the panic",
            "github_pat_11AAAAAAA_secret",
        );
        assert_redacted(
            "saw AKIAIOSFODNN7EXAMPLE in the panic",
            "AKIAIOSFODNN7EXAMPLE",
        );
        assert_redacted("saw Bearer abcDEF123xyz in the panic", "abcDEF123xyz");
        assert_redacted("saw Basic dXNlcjpwYXNz in the panic", "dXNlcjpwYXNz");
    }

    #[test]
    fn authorization_headers_redact_through_end_of_line() {
        let cases = [
            (
                "Authorization: Digest username=\"u\", response=\"deadbeef\"",
                "deadbeef",
            ),
            (
                "Proxy-Authorization: Negotiate YII6secretblob",
                "YII6secretblob",
            ),
            (
                "Authorization: AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE, Signature=sigdeadbeef",
                "sigdeadbeef",
            ),
            ("Authorization: Bearer abcDEF123xyz", "abcDEF123xyz"),
            (
                "handler got Authorization: Bearer sk-live-12345 from client",
                "sk-live-12345",
            ),
        ];
        for (raw, secret) in cases {
            let out = redact_secret_text(raw);
            assert!(!out.contains(secret), "{raw:?} -> {out}");
            assert!(!out.contains("from client"), "trailing prose stayed: {out}");
        }
        let quoted =
            redact_secret_text(r#"{"authorization":"Digest response=\"deadbeef\""} tail stays"#);
        assert!(!quoted.contains("deadbeef"), "{quoted}");
        assert!(quoted.contains("tail stays"), "{quoted}");
        let escaped = redact_secret_text(r#"panic: {"password":"abc\"VISIBLE_SUFFIX"}"#);
        assert!(
            !escaped.contains("VISIBLE_SUFFIX"),
            "escaped suffix leaked: {escaped}"
        );
        let next = redact_secret_text("Authorization: Bearer secret\nnext line stays");
        assert!(next.contains("next line stays"), "{next}");
        assert!(!next.contains("secret"), "{next}");
    }

    #[test]
    fn bare_token_scan_leaves_lookalikes() {
        let samples = [
            "myghp_abcdefghijklmnopqrstuvwxyz0123456789 stays",
            "basic word stays",
            "AKIAIOSFODNN7EXAMPLEX stays",
            "prefix AKIAIOSFODNN7EXAMPL stays short",
            "sk-live-12345 stays",
        ];
        for raw in samples {
            let out = redact_secret_text(raw);
            assert_eq!(out, raw, "{raw:?} -> {out}");
        }
    }
}
