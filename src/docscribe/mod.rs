//! Docscribe
//!
//! Provides APIs for accessing embedded Crucible documentation assets and
//! extracting YAML frontmatter.

use std::collections::BTreeMap;

use serde::Deserialize;

/// Parsed frontmatter from a markdown document.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Frontmatter {
    /// Document title.
    pub title: Option<String>,
    /// Short description.
    pub description: Option<String>,
    /// Author.
    pub author: Option<String>,
    /// Document date.
    pub date: Option<String>,
    /// Last updated date.
    pub last_updated: Option<String>,
    /// Lifecycle status.
    pub status: Option<String>,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Any additional fields.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yaml::Value>,
}

/// A parsed documentation asset.
#[derive(Debug, Clone, PartialEq)]
pub struct Doc {
    /// Path relative to `docs/crucible-rs/`.
    pub path: String,
    /// Parsed frontmatter (empty if none).
    pub frontmatter: Frontmatter,
    /// Markdown content without frontmatter.
    pub content: String,
}

/// Errors that can occur when reading or parsing docs.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum DocError {
    /// The requested doc does not exist in embedded assets.
    #[error("doc not found: {0}")]
    NotFound(String),

    /// The doc is not valid UTF-8.
    #[error("doc is not valid UTF-8: {0}")]
    InvalidUtf8(String),

    /// Frontmatter YAML could not be parsed.
    #[error("invalid frontmatter in {path}: {message}")]
    InvalidFrontmatter {
        /// Document path.
        path: String,
        /// Error details.
        message: String,
    },
}

/// Read a documentation asset as raw UTF-8.
pub fn read_doc(path: &str) -> Result<&'static str, DocError> {
    let bytes =
        crate::crucible::open_docs(path).ok_or_else(|| DocError::NotFound(path.to_string()))?;

    std::str::from_utf8(bytes).map_err(|_| DocError::InvalidUtf8(path.to_string()))
}

/// Parse a markdown document into `(frontmatter, content)`.
///
/// If no frontmatter is present, returns default frontmatter and the original
/// markdown as content.
pub fn parse_frontmatter(markdown: &str) -> Result<(Frontmatter, &str), DocError> {
    let Some((yaml, body)) = split_frontmatter(markdown) else {
        return Ok((Frontmatter::default(), markdown));
    };

    let frontmatter: Frontmatter =
        serde_yaml::from_str(yaml).map_err(|e| DocError::InvalidFrontmatter {
            path: "<in-memory>".to_string(),
            message: e.to_string(),
        })?;

    Ok((frontmatter, body))
}

/// Read and parse a documentation asset.
pub fn read_parsed_doc(path: &str) -> Result<Doc, DocError> {
    let raw = read_doc(path)?;

    let (frontmatter, content) = parse_frontmatter(raw).map_err(|e| match e {
        DocError::InvalidFrontmatter { message, .. } => DocError::InvalidFrontmatter {
            path: path.to_string(),
            message,
        },
        other => other,
    })?;

    Ok(Doc {
        path: path.to_string(),
        frontmatter,
        content: content.to_string(),
    })
}

/// Split YAML frontmatter from markdown.
///
/// Expects a leading `---` line and a closing `---` line.
fn split_frontmatter(markdown: &str) -> Option<(&str, &str)> {
    if !markdown.starts_with("---") {
        return None;
    }

    let mut lines = markdown.lines();
    let first = lines.next()?;
    if first.trim_end() != "---" {
        return None;
    }

    // Find byte offsets for YAML block.
    let mut offset = first.len();
    // account for newline after first line
    offset += newline_len_after(markdown, offset);

    let yaml_start = offset;

    let mut yaml_end = None;
    for line in lines {
        let line_len = line.len();
        if line.trim_end() == "---" {
            yaml_end = Some(offset);
            offset += line_len;
            offset += newline_len_after(markdown, offset);
            break;
        }

        offset += line_len;
        offset += newline_len_after(markdown, offset);
    }

    let yaml_end = yaml_end?;

    let yaml = &markdown[yaml_start..yaml_end];
    let mut body = &markdown[offset..];

    // Common convention: a single blank line after frontmatter.
    if let Some(stripped) = body.strip_prefix("\r\n") {
        body = stripped;
    } else if let Some(stripped) = body.strip_prefix('\n') {
        body = stripped;
    }

    Some((yaml, body))
}

fn newline_len_after(s: &str, idx: usize) -> usize {
    // idx is positioned at end of a line; detect following newline sequence.
    let bytes = s.as_bytes();
    if idx >= bytes.len() {
        return 0;
    }

    if bytes[idx] == b'\n' {
        return 1;
    }

    // `lines()` splits on either \n or \r\n, but when reconstructing offsets
    // we must handle the raw string.
    if bytes[idx] == b'\r' {
        if idx + 1 < bytes.len() && bytes[idx + 1] == b'\n' {
            return 2;
        }
        return 1;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_present() {
        let doc = read_parsed_doc("architecture/fulmen-helper-library-standard.md").unwrap();
        assert_eq!(
            doc.frontmatter.title.as_deref(),
            Some("Fulmen Helper Library Standard")
        );
        assert!(doc.content.starts_with("# Fulmen Helper Library Standard"));
    }

    #[test]
    fn test_parse_frontmatter_absent() {
        let md = "# Title\n\nBody\n";
        let (fm, body) = parse_frontmatter(md).unwrap();
        assert_eq!(fm, Frontmatter::default());
        assert_eq!(body, md);
    }

    #[test]
    fn test_read_doc_not_found() {
        let err = read_doc("nope.md").unwrap_err();
        assert!(matches!(err, DocError::NotFound(_)));
    }

    #[test]
    fn test_split_frontmatter_round_trip() {
        let md = "---\ntitle: Hello\n---\n# Hi\n";
        let (fm, body) = parse_frontmatter(md).unwrap();
        assert_eq!(fm.title.as_deref(), Some("Hello"));
        assert_eq!(body, "# Hi\n");
    }
}
