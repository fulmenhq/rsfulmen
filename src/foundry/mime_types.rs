//! MIME Types Catalog
//!
//! Provides access to a curated set of common MIME types synced from Crucible.
//! Includes lookup by ID, MIME string, extension/filename, plus lightweight
//! content-based detection using magic numbers and heuristics.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

use super::{FoundryError, FoundryResult};

/// MIME type definition from the Crucible catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MimeType {
    /// Catalog identifier (e.g., `json`, `plain-text`).
    pub id: String,
    /// MIME string (e.g., `application/json`).
    pub mime: String,
    /// Human-friendly name.
    #[serde(default)]
    pub name: String,
    /// File extensions associated with this MIME type (without dots).
    #[serde(default)]
    pub extensions: Vec<String>,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

impl MimeType {
    /// Check if this MIME type matches an extension (with or without leading dot).
    ///
    /// Matching is case-insensitive.
    pub fn matches_extension(&self, ext: &str) -> bool {
        let normalized = normalize_extension(ext);
        self.extensions
            .iter()
            .any(|e| e.eq_ignore_ascii_case(&normalized))
    }

    /// Check if this MIME type matches a filename by extension.
    pub fn matches_filename(&self, filename: &str) -> bool {
        Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| self.matches_extension(ext))
    }

    /// Get the primary extension (the first extension, if any).
    pub fn primary_extension(&self) -> Option<&str> {
        self.extensions.first().map(|s| s.as_str())
    }

    /// True for `text/*` and structured text types.
    ///
    /// This treats common structured text formats like JSON/XML/YAML as text-based
    /// even when they live under `application/*`.
    pub fn is_text_based(&self) -> bool {
        let mime = self.mime.to_ascii_lowercase();
        if mime.starts_with("text/") {
            return true;
        }

        let subtype = mime.split_once('/').map(|(_, s)| s).unwrap_or("");

        subtype == "json"
            || subtype == "xml"
            || subtype == "yaml"
            || subtype == "x-ndjson"
            || subtype.ends_with("+json")
            || subtype.ends_with("+xml")
            || subtype.ends_with("+yaml")
    }
}

#[derive(Debug, Clone, Deserialize)]
struct MimeCatalog {
    #[allow(dead_code)]
    description: String,
    #[allow(dead_code)]
    version: String,
    types: Vec<MimeType>,
}

/// Embedded MIME types YAML from Crucible.
const MIME_TYPES_YAML: &str =
    include_str!("../../config/crucible-rs/library/foundry/mime-types.yaml");

/// Detection buffer size (bytes).
const DETECTION_BUFFER_SIZE: usize = 512;

/// MIME detection confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// Magic number match.
    High,
    /// Heuristic/structural match.
    Medium,
    /// Extension-based fallback.
    Low,
}

/// Detection result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionResult {
    /// Detected MIME type.
    pub mime_type: &'static MimeType,
    /// Confidence level.
    pub confidence: Confidence,
}

/// Indexes for fast lookup.
struct MimeIndexes {
    types: Vec<MimeType>,
    by_id: HashMap<String, usize>,
    by_mime: HashMap<String, usize>,
    by_extension: HashMap<String, usize>,
}

impl MimeIndexes {
    fn load() -> FoundryResult<Self> {
        let catalog: MimeCatalog = serde_yaml::from_str(MIME_TYPES_YAML)
            .map_err(|e| FoundryError::LoadError(format!("Failed to parse mime types: {e}")))?;

        let mut types = Vec::with_capacity(catalog.types.len());
        let mut by_id = HashMap::new();
        let mut by_mime = HashMap::new();
        let mut by_extension = HashMap::new();

        for mt in catalog.types {
            let index = types.len();

            if by_id.contains_key(&mt.id) {
                return Err(FoundryError::LoadError(format!(
                    "Duplicate mime type id: {}",
                    mt.id
                )));
            }

            by_id.insert(mt.id.clone(), index);
            by_mime.insert(mt.mime.to_lowercase(), index);

            for ext in &mt.extensions {
                let normalized = normalize_extension(ext);
                // First writer wins; catalog should avoid duplicates.
                by_extension.entry(normalized).or_insert(index);
            }

            types.push(mt);
        }

        Ok(Self {
            types,
            by_id,
            by_mime,
            by_extension,
        })
    }

    fn get_by_id(&self, id: &str) -> Option<&MimeType> {
        self.by_id.get(id).map(|&i| &self.types[i])
    }

    fn get_by_mime(&self, mime: &str) -> Option<&MimeType> {
        self.by_mime
            .get(&mime.to_lowercase())
            .map(|&i| &self.types[i])
    }

    fn get_by_extension(&self, ext: &str) -> Option<&MimeType> {
        self.by_extension
            .get(&normalize_extension(ext))
            .map(|&i| &self.types[i])
    }
}

static CATALOGS: Lazy<MimeIndexes> =
    Lazy::new(|| MimeIndexes::load().expect("Failed to load mime types catalog"));

fn normalize_extension(ext: &str) -> String {
    ext.trim_start_matches('.').trim().to_ascii_lowercase()
}

/// Get a MIME type by ID.
pub fn lookup_mime_type(id: &str) -> Option<&'static MimeType> {
    CATALOGS.get_by_id(id)
}

/// Get a MIME type by MIME string (case-insensitive).
pub fn lookup_by_mime(mime: &str) -> Option<&'static MimeType> {
    CATALOGS.get_by_mime(mime)
}

/// Get a MIME type by extension (with or without dot, case-insensitive).
pub fn lookup_by_extension(ext: &str) -> Option<&'static MimeType> {
    CATALOGS.get_by_extension(ext)
}

/// Get a MIME type by filename.
pub fn lookup_by_filename(filename: &str) -> Option<&'static MimeType> {
    Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .and_then(lookup_by_extension)
}

/// List all MIME types.
pub fn list_mime_types() -> &'static [MimeType] {
    &CATALOGS.types
}

/// Get MIME type count.
pub fn mime_type_count() -> usize {
    CATALOGS.types.len()
}

/// Check if a MIME string is in the catalog (case-insensitive).
pub fn is_supported_mime(mime: &str) -> bool {
    lookup_by_mime(mime).is_some()
}

/// Detect MIME type from content bytes.
pub fn detect_mime_type(content: &[u8]) -> Option<DetectionResult> {
    if content.is_empty() {
        return None;
    }

    let trimmed = trim_bom_and_whitespace(content);
    if trimmed.is_empty() {
        return None;
    }

    // 1) XML (high confidence): XML declaration only (v0.1.0 parity).
    if trimmed.starts_with(b"<?xml") {
        return Some(DetectionResult {
            mime_type: lookup_mime_type("xml")?,
            confidence: Confidence::High,
        });
    }

    // 2) NDJSON (medium confidence): must run before JSON.
    if looks_like_ndjson(trimmed) {
        return Some(DetectionResult {
            mime_type: lookup_mime_type("ndjson")?,
            confidence: Confidence::Medium,
        });
    }

    // 3) JSON (high confidence): starts with { or [ and has some JSON structure.
    if looks_like_json(trimmed) {
        return Some(DetectionResult {
            mime_type: lookup_mime_type("json")?,
            confidence: Confidence::High,
        });
    }

    // 4) YAML (medium confidence): exact markers, then heuristic.
    if looks_like_yaml(trimmed) {
        return Some(DetectionResult {
            mime_type: lookup_mime_type("yaml")?,
            confidence: Confidence::Medium,
        });
    }

    // 5) CSV (medium confidence): delimiter consistency across lines.
    if looks_like_csv(trimmed) {
        return Some(DetectionResult {
            mime_type: lookup_mime_type("csv")?,
            confidence: Confidence::Medium,
        });
    }

    // 6) Plain text fallback (medium confidence).
    if is_text_content(&content[..content.len().min(DETECTION_BUFFER_SIZE)]) {
        return Some(DetectionResult {
            mime_type: lookup_mime_type("plain-text")?,
            confidence: Confidence::Medium,
        });
    }

    None
}

/// Detect MIME type with a filename hint.
///
/// If content detection fails, this falls back to extension-based detection with `Low` confidence.
pub fn detect_mime_type_with_hint(
    content: &[u8],
    filename: Option<&str>,
) -> Option<DetectionResult> {
    let hint = filename.and_then(lookup_by_filename);

    if let Some(detected) = detect_mime_type(content) {
        // If content is only classified as plain text, allow a more specific
        // extension hint to win (common for stdin/streams).
        if detected.mime_type.id == "plain-text" {
            if let Some(hint_mt) = hint {
                if hint_mt.id != "plain-text" {
                    return Some(DetectionResult {
                        mime_type: hint_mt,
                        confidence: Confidence::Low,
                    });
                }
            }
        }

        return Some(detected);
    }

    let mime_type = hint?;
    Some(DetectionResult {
        mime_type,
        confidence: Confidence::Low,
    })
}

/// Detect MIME type from a file path (reads first 512 bytes).
pub fn detect_mime_type_from_path(path: &Path) -> Result<Option<DetectionResult>, io::Error> {
    let mut file = File::open(path)?;
    let mut buf = [0u8; DETECTION_BUFFER_SIZE];
    let n = file.read(&mut buf)?;
    Ok(detect_mime_type(&buf[..n]))
}

fn trim_bom_and_whitespace(data: &[u8]) -> &[u8] {
    let mut start = 0usize;

    // UTF-8 BOM
    if data.len() >= 3 && data[0..3] == [0xEF, 0xBB, 0xBF] {
        start += 3;
    }

    // UTF-16 BOM (LE/BE)
    if data.len() >= start + 2 {
        let next = &data[start..start + 2];
        if next == [0xFF, 0xFE] || next == [0xFE, 0xFF] {
            start += 2;
        }
    }

    while start < data.len() {
        match data[start] {
            b' ' | b'\t' | b'\n' | b'\r' => start += 1,
            _ => break,
        }
    }

    &data[start..]
}

fn looks_like_json(trimmed: &[u8]) -> bool {
    if trimmed.is_empty() {
        return false;
    }

    let first = trimmed[0];
    if first != b'{' && first != b'[' {
        return false;
    }

    // Lightweight structure check (matches gofulmen/pyfulmen style):
    // look for a small set of structural characters in the prefix.
    let sample_len = trimmed.len().min(50);
    trimmed[..sample_len]
        .iter()
        .any(|&b| b == b'{' || b == b'[' || b == b'"' || b == b':')
}

fn looks_like_ndjson(trimmed: &[u8]) -> bool {
    let text = match std::str::from_utf8(&trimmed[..trimmed.len().min(DETECTION_BUFFER_SIZE)]) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();

    if lines.len() < 2 {
        return false;
    }

    // Validate first 2-3 lines as JSON.
    let to_check = lines.len().min(3);
    let mut valid = 0usize;

    for line in lines.iter().take(to_check) {
        if serde_json::from_str::<serde_json::Value>(line).is_ok() {
            valid += 1;
        } else {
            return false;
        }
    }

    valid >= 2
}

fn looks_like_yaml(trimmed: &[u8]) -> bool {
    if trimmed.starts_with(b"---") || trimmed.starts_with(b"%YAML") {
        return true;
    }

    // Heuristic inspired by tsfulmen: count YAML indicators, reject JSON-ish indicators.
    let text = match std::str::from_utf8(&trimmed[..trimmed.len().min(DETECTION_BUFFER_SIZE)]) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let mut yaml_indicators = 0usize;
    let mut non_yaml_indicators = 0usize;

    for line in text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .take(10)
    {
        if line.starts_with('#') {
            continue;
        }

        // YAML list item
        if line.starts_with("- ") {
            yaml_indicators += 1;
            continue;
        }

        // YAML key-value (rough)
        if let Some((key, value)) = line.split_once(':') {
            if !key.trim().is_empty() && !value.trim().is_empty() {
                yaml_indicators += 1;
                continue;
            }
        }

        if line.starts_with('{') || line.starts_with('[') || line.ends_with(',') {
            non_yaml_indicators += 1;
        }
    }

    yaml_indicators >= 2 && non_yaml_indicators == 0
}

fn looks_like_csv(trimmed: &[u8]) -> bool {
    let text = match std::str::from_utf8(&trimmed[..trimmed.len().min(DETECTION_BUFFER_SIZE)]) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .take(10)
        .collect();

    if lines.len() < 2 {
        return false;
    }

    let delimiters = [',', ';', '\t'];

    for delim in delimiters {
        let counts: Vec<usize> = lines
            .iter()
            .map(|line| line.chars().filter(|&c| c == delim).count())
            .collect();

        let first = counts[0];
        if first > 0 && counts.iter().all(|&c| c == first) {
            return true;
        }
    }

    false
}

fn is_text_content(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    let printable = data
        .iter()
        .filter(|&&b| (32..=126).contains(&b) || b == b'\n' || b == b'\r' || b == b'\t' || b >= 128)
        .count();

    (printable as f64 / data.len() as f64) > 0.8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_loads_all_mime_types() {
        assert_eq!(mime_type_count(), 7);
        assert_eq!(list_mime_types().len(), 7);
    }

    #[test]
    fn test_lookup_by_id() {
        let json = lookup_mime_type("json").expect("json should exist");
        assert_eq!(json.mime, "application/json");
    }

    #[test]
    fn test_lookup_by_mime() {
        let json = lookup_by_mime("application/json").expect("application/json should exist");
        assert_eq!(json.id, "json");

        let json2 = lookup_by_mime("APPLICATION/JSON").expect("case-insensitive");
        assert_eq!(json2.id, "json");
    }

    #[test]
    fn test_lookup_by_extension() {
        assert_eq!(lookup_by_extension("json").unwrap().id, "json");
        assert_eq!(lookup_by_extension(".json").unwrap().id, "json");
        assert_eq!(lookup_by_extension("JSON").unwrap().id, "json");
    }

    #[test]
    fn test_lookup_by_filename() {
        assert_eq!(lookup_by_filename("config.yaml").unwrap().id, "yaml");
        assert_eq!(lookup_by_filename("config.YML").unwrap().id, "yaml");
    }

    #[test]
    fn test_is_supported_mime() {
        assert!(is_supported_mime("application/json"));
        assert!(is_supported_mime("APPLICATION/JSON"));
        assert!(!is_supported_mime("application/unknown"));
    }

    #[test]
    fn test_detect_json_object() {
        let content = br#"{"key": "value"}"#;
        let result = detect_mime_type(content).unwrap();
        assert_eq!(result.mime_type.id, "json");
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_detect_json_array() {
        let content = b"[1, 2, 3]";
        let result = detect_mime_type(content).unwrap();
        assert_eq!(result.mime_type.id, "json");
    }

    #[test]
    fn test_detect_xml_declaration_only() {
        let content = b"<?xml version=\"1.0\"?><root/>";
        let result = detect_mime_type(content).unwrap();
        assert_eq!(result.mime_type.id, "xml");
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_detect_yaml() {
        let content = b"---\nkey: value\nlist:\n  - item1\n  - item2";
        let result = detect_mime_type(content).unwrap();
        assert_eq!(result.mime_type.id, "yaml");
        assert_eq!(result.confidence, Confidence::Medium);
    }

    #[test]
    fn test_detect_csv() {
        let content = b"name,age,city\nAlice,30,NYC\nBob,25,LA";
        let result = detect_mime_type(content).unwrap();
        assert_eq!(result.mime_type.id, "csv");
        assert_eq!(result.confidence, Confidence::Medium);
    }

    #[test]
    fn test_detect_ndjson_runs_before_json() {
        let content = b"{\"a\":1}\n{\"b\":2}\n";
        let result = detect_mime_type(content).unwrap();
        assert_eq!(result.mime_type.id, "ndjson");
        assert_eq!(result.confidence, Confidence::Medium);
    }

    #[test]
    fn test_detect_plain_text_fallback() {
        let content = b"some text content";
        let result = detect_mime_type(content).unwrap();
        assert_eq!(result.mime_type.id, "plain-text");
    }

    #[test]
    fn test_extension_with_hint() {
        // Ambiguous content but filename provides hint
        let content = b"some text content";
        let result = detect_mime_type_with_hint(content, Some("data.json")).unwrap();
        assert_eq!(result.mime_type.id, "json");
        // Extension hint beats plain-text fallback.
        assert_eq!(result.confidence, Confidence::Low);

        let content = b"\x00\x01\x02\xff\xfe";
        let result = detect_mime_type_with_hint(content, Some("data.json")).unwrap();
        assert_eq!(result.mime_type.id, "json");
        assert_eq!(result.confidence, Confidence::Low);
    }

    #[test]
    fn test_trim_bom_and_whitespace() {
        let content = b"\xef\xbb\xbf  {\"k\":1}";
        let detected = detect_mime_type(content).unwrap();
        assert_eq!(detected.mime_type.id, "json");
    }

    #[test]
    fn test_mime_type_methods() {
        let mt = lookup_mime_type("json").unwrap();
        assert!(mt.matches_extension("json"));
        assert!(mt.matches_extension(".JSON"));
        assert!(mt.matches_filename("foo.json"));
        assert_eq!(mt.primary_extension(), Some("json"));
        assert!(mt.is_text_based());
    }
}
