//! Pattern Catalog
//!
//! Provides access to regex, glob, and literal patterns from the
//! Crucible foundry catalog for validation and matching.
//!
//! ## Example
//!
//! ```rust
//! use rsfulmen::foundry::patterns::{lookup_pattern, validate_email};
//!
//! let email = lookup_pattern("ansi-email").unwrap();
//! assert!(email.matches("user@example.com").unwrap());
//! assert!(validate_email("user@example.com"));
//! ```

use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{FoundryError, FoundryResult};

// ============================================================================
// Public Data Structures
// ============================================================================

/// Pattern kind enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatternKind {
    /// Regular expression pattern
    Regex,
    /// Glob pattern (file path matching)
    Glob,
    /// Exact literal match
    Literal,
}

/// Language-specific flags for regex compilation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternFlags {
    /// Enable Unicode character classes
    #[serde(default)]
    pub unicode: bool,
    /// Enable case-insensitive matching
    #[serde(default, rename = "ignoreCase")]
    pub ignore_case: bool,
    /// Enable multi-line mode
    #[serde(default)]
    pub multiline: bool,
    /// Allow dot to match newlines
    #[serde(default, rename = "dotAll")]
    pub dot_all: bool,
}

/// Pattern definition from the Crucible catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pattern {
    /// Pattern identifier (e.g., "ansi-email")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Pattern kind
    pub kind: PatternKind,
    /// Pattern string
    pub pattern: String,
    /// Language-specific flags
    #[serde(default)]
    pub flags: HashMap<String, PatternFlags>,
    /// Description of the pattern
    #[serde(default)]
    pub description: Option<String>,
    /// Example values that should match
    #[serde(default)]
    pub examples: Vec<String>,
}

impl Pattern {
    /// Get Rust-specific flags (or default).
    pub fn rust_flags(&self) -> PatternFlags {
        self.flags.get("rust").cloned().unwrap_or_default()
    }

    /// Compile the pattern into a reusable matcher.
    pub fn compile(&self) -> Result<CompiledPattern, PatternError> {
        let compiled = CompiledPattern::new(self.clone());
        compiled.ensure_compiled()?;
        Ok(compiled)
    }

    /// Check if the value matches the pattern.
    pub fn matches(&self, value: &str) -> Result<bool, PatternError> {
        if let Some(compiled) = get_compiled_pattern(&self.id) {
            return compiled.matches(value);
        }

        self.compile()?.matches(value)
    }

    /// Search for the pattern in the value (partial match).
    pub fn search(&self, value: &str) -> Result<bool, PatternError> {
        if let Some(compiled) = get_compiled_pattern(&self.id) {
            return compiled.search(value);
        }

        self.compile()?.search(value)
    }
}

/// Compiled pattern with cached matcher.
#[derive(Debug)]
pub struct CompiledPattern {
    pattern: Pattern,
    compiled: OnceCell<CompiledInner>,
}

#[derive(Debug)]
enum CompiledInner {
    Regex(regex::Regex),
    Glob(Vec<glob::Pattern>),
    Literal(String),
}

impl CompiledPattern {
    fn new(pattern: Pattern) -> Self {
        Self {
            pattern,
            compiled: OnceCell::new(),
        }
    }

    /// Return the source pattern definition.
    pub fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    fn ensure_compiled(&self) -> Result<&CompiledInner, PatternError> {
        self.compiled.get_or_try_init(|| self.build_inner())
    }

    fn build_inner(&self) -> Result<CompiledInner, PatternError> {
        match self.pattern.kind {
            PatternKind::Regex => self.compile_regex().map(CompiledInner::Regex),
            PatternKind::Glob => self.compile_glob().map(CompiledInner::Glob),
            PatternKind::Literal => Ok(CompiledInner::Literal(self.pattern.pattern.clone())),
        }
    }

    fn compile_regex(&self) -> Result<regex::Regex, PatternError> {
        let flags = self.pattern.rust_flags();
        let mut builder = regex::RegexBuilder::new(&self.pattern.pattern);

        builder.unicode(flags.unicode);
        builder.case_insensitive(flags.ignore_case);
        builder.multi_line(flags.multiline);
        builder.dot_matches_new_line(flags.dot_all);

        builder
            .build()
            .map_err(|source| PatternError::InvalidRegex {
                id: self.pattern.id.clone(),
                source,
            })
    }

    fn compile_glob(&self) -> Result<Vec<glob::Pattern>, PatternError> {
        let expanded = expand_glob_braces(&self.pattern.pattern);
        let mut compiled = Vec::with_capacity(expanded.len());

        for pattern in expanded {
            let glob = glob::Pattern::new(&pattern).map_err(|err| PatternError::InvalidGlob {
                id: self.pattern.id.clone(),
                message: err.to_string(),
            })?;
            compiled.push(glob);
        }

        Ok(compiled)
    }

    /// Full match (anchored).
    pub fn matches(&self, value: &str) -> Result<bool, PatternError> {
        match self.ensure_compiled()? {
            CompiledInner::Regex(regex) => {
                if let Some(mat) = regex.find(value) {
                    Ok(mat.start() == 0 && mat.end() == value.len())
                } else {
                    Ok(false)
                }
            }
            CompiledInner::Glob(patterns) => Ok(patterns.iter().any(|p| p.matches(value))),
            CompiledInner::Literal(literal) => Ok(literal == value),
        }
    }

    /// Partial match (search).
    pub fn search(&self, value: &str) -> Result<bool, PatternError> {
        match self.ensure_compiled()? {
            CompiledInner::Regex(regex) => Ok(regex.is_match(value)),
            CompiledInner::Glob(patterns) => Ok(patterns.iter().any(|p| p.matches(value))),
            CompiledInner::Literal(literal) => Ok(value.contains(literal)),
        }
    }

    /// Find all matches within a value.
    pub fn find_all<'a>(&self, value: &'a str) -> Result<Vec<&'a str>, PatternError> {
        match self.ensure_compiled()? {
            CompiledInner::Regex(regex) => Ok(regex.find_iter(value).map(|m| m.as_str()).collect()),
            CompiledInner::Glob(patterns) => {
                if patterns.iter().any(|p| p.matches(value)) {
                    Ok(vec![value])
                } else {
                    Ok(Vec::new())
                }
            }
            CompiledInner::Literal(literal) => {
                let mut matches = Vec::new();
                let mut start = 0;

                while let Some(index) = value[start..].find(literal) {
                    let match_start = start + index;
                    let match_end = match_start + literal.len();
                    matches.push(&value[match_start..match_end]);
                    start = match_end;
                }

                Ok(matches)
            }
        }
    }
}

fn expand_glob_braces(pattern: &str) -> Vec<String> {
    let start = match pattern.find('{') {
        Some(index) => index,
        None => return vec![pattern.to_string()],
    };

    let end = match pattern[start..].find('}') {
        Some(index) => start + index,
        None => return vec![pattern.to_string()],
    };

    let prefix = &pattern[..start];
    let suffix = &pattern[end + 1..];
    let inner = &pattern[start + 1..end];

    let mut expanded = Vec::new();
    for option in inner.split(',') {
        let mut candidate = String::new();
        candidate.push_str(prefix);
        candidate.push_str(option);
        candidate.push_str(suffix);
        expanded.extend(expand_glob_braces(&candidate));
    }

    expanded
}

/// Errors that can occur during pattern compilation or matching.
#[derive(thiserror::Error, Debug)]
pub enum PatternError {
    /// Regex pattern failed to compile.
    #[error("invalid regex for pattern '{id}': {source}")]
    InvalidRegex {
        /// Pattern identifier.
        id: String,
        /// Underlying regex compilation error.
        source: regex::Error,
    },

    /// Glob pattern failed to compile.
    #[error("invalid glob for pattern '{id}': {message}")]
    InvalidGlob {
        /// Pattern identifier.
        id: String,
        /// Underlying glob compilation error message.
        message: String,
    },
}

// ============================================================================
// Catalog Loading and Indexing
// ============================================================================

/// Embedded patterns YAML from Crucible.
const PATTERNS_YAML: &str = include_str!("../../config/crucible-rs/library/foundry/patterns.yaml");

#[derive(Debug, Deserialize)]
struct PatternCatalog {
    #[allow(dead_code)]
    description: String,
    #[allow(dead_code)]
    version: String,
    patterns: Vec<Pattern>,
}

/// Pre-computed indexes for fast lookups.
struct PatternIndexes {
    patterns: Vec<Pattern>,
    by_id: HashMap<String, usize>,
    compiled: Vec<CompiledPattern>,
}

impl PatternIndexes {
    fn load() -> FoundryResult<Self> {
        let catalog: PatternCatalog = serde_yaml::from_str(PATTERNS_YAML)
            .map_err(|e| FoundryError::LoadError(format!("Failed to parse patterns: {}", e)))?;

        let mut patterns = Vec::with_capacity(catalog.patterns.len());
        let mut by_id = HashMap::new();

        for pattern in catalog.patterns {
            let index = patterns.len();
            if by_id.contains_key(&pattern.id) {
                return Err(FoundryError::LoadError(format!(
                    "Duplicate pattern id: {}",
                    pattern.id
                )));
            }
            by_id.insert(pattern.id.clone(), index);
            patterns.push(pattern);
        }

        let compiled = patterns.iter().cloned().map(CompiledPattern::new).collect();

        Ok(Self {
            patterns,
            by_id,
            compiled,
        })
    }

    fn get_by_id(&self, id: &str) -> Option<&Pattern> {
        self.by_id.get(id).map(|&i| &self.patterns[i])
    }

    fn get_compiled_by_id(&self, id: &str) -> Option<&CompiledPattern> {
        self.by_id.get(id).map(|&i| &self.compiled[i])
    }
}

/// Lazily initialized pattern catalog indexes.
static CATALOGS: Lazy<PatternIndexes> =
    Lazy::new(|| PatternIndexes::load().expect("Failed to load patterns catalog"));

// ============================================================================
// Public API Functions
// ============================================================================

/// Get a pattern by ID.
pub fn lookup_pattern(id: &str) -> Option<&'static Pattern> {
    CATALOGS.get_by_id(id)
}

/// List all patterns in the catalog.
pub fn list_patterns() -> &'static [Pattern] {
    &CATALOGS.patterns
}

/// List patterns by kind.
pub fn list_patterns_by_kind(kind: PatternKind) -> Vec<&'static Pattern> {
    CATALOGS
        .patterns
        .iter()
        .filter(|pattern| pattern.kind == kind)
        .collect()
}

/// Get the total pattern count.
pub fn pattern_count() -> usize {
    CATALOGS.patterns.len()
}

/// Get a compiled pattern by ID (lazy compilation).
pub fn get_compiled_pattern(id: &str) -> Option<&'static CompiledPattern> {
    CATALOGS.get_compiled_by_id(id)
}

fn matches_pattern(id: &str, value: &str) -> bool {
    get_compiled_pattern(id)
        .and_then(|compiled| compiled.matches(value).ok())
        .unwrap_or(false)
}

/// Validate an email address using the catalog pattern.
pub fn validate_email(value: &str) -> bool {
    matches_pattern("ansi-email", value)
}

/// Validate a slug using the catalog pattern.
pub fn validate_slug(value: &str) -> bool {
    matches_pattern("slug", value)
}

/// Validate a semantic version string.
pub fn validate_semver(value: &str) -> bool {
    matches_pattern("semantic-version", value)
}

/// Validate a UUID v4 string.
pub fn validate_uuid_v4(value: &str) -> bool {
    matches_pattern("uuid-v4", value)
}

/// Validate a ULID string.
pub fn validate_ulid(value: &str) -> bool {
    matches_pattern("ulid", value)
}

/// Validate an ISO 8601 date (YYYY-MM-DD).
pub fn validate_iso_date(value: &str) -> bool {
    matches_pattern("strict-iso-date", value)
}

/// Validate an ISO 8601 UTC timestamp.
pub fn validate_iso_timestamp_utc(value: &str) -> bool {
    matches_pattern("iso-timestamp-z", value)
}

/// Validate a domain name string.
pub fn validate_domain(value: &str) -> bool {
    matches_pattern("domain-name", value)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_loads_all_patterns() {
        assert_eq!(pattern_count(), 21);
        assert_eq!(pattern_count(), list_patterns().len());
    }

    #[test]
    fn test_lookup_pattern_by_id() {
        let email = lookup_pattern("ansi-email").expect("ansi-email should exist");
        assert_eq!(email.kind, PatternKind::Regex);
        assert!(email.pattern.contains('@'));
    }

    #[test]
    fn test_lookup_pattern_not_found() {
        assert!(lookup_pattern("missing").is_none());
    }

    #[test]
    fn test_list_patterns_by_kind() {
        let regexes = list_patterns_by_kind(PatternKind::Regex);
        let globs = list_patterns_by_kind(PatternKind::Glob);
        let literals = list_patterns_by_kind(PatternKind::Literal);

        assert_eq!(regexes.len(), 19);
        assert_eq!(globs.len(), 2);
        assert!(literals.is_empty());
    }

    #[test]
    fn test_compile_all_regex_patterns() {
        for pattern in list_patterns_by_kind(PatternKind::Regex) {
            let compiled = pattern.compile().expect("regex should compile");
            assert!(matches!(
                compiled.ensure_compiled().expect("compiled"),
                CompiledInner::Regex(_)
            ));
        }
    }

    #[test]
    fn test_example_matches() {
        let skip_ids = ["jwt", "uuid-v4"];

        for pattern in list_patterns() {
            if skip_ids.contains(&pattern.id.as_str()) {
                continue;
            }

            for example in &pattern.examples {
                if example.contains("...") {
                    continue;
                }

                assert!(
                    pattern.matches(example).unwrap_or(false),
                    "Example should match pattern {}",
                    pattern.id
                );
            }
        }
    }

    #[test]
    fn test_glob_patterns() {
        let json = lookup_pattern("glob-any-json").expect("glob-any-json should exist");
        let yaml = lookup_pattern("glob-any-yaml").expect("glob-any-yaml should exist");

        assert!(json.matches("data/config.json").unwrap());
        assert!(yaml.matches("data/config.yaml").unwrap());
        assert!(yaml.matches("data/config.yml").unwrap());
        assert!(!json.matches("data/config.yaml").unwrap());
    }

    #[test]
    fn test_validator_helpers() {
        assert!(validate_email("user@example.com"));
        assert!(validate_slug("fulmen-hq"));
        assert!(validate_semver("1.2.3"));
        assert!(validate_uuid_v4("123e4567-e89b-42d3-a456-426614174000"));
        assert!(validate_ulid("01HCP5MZF1H84D0CW7V5T9FQ0P"));
        assert!(validate_iso_date("2025-10-09"));
        assert!(validate_iso_timestamp_utc("2025-10-09T14:15:16Z"));
        assert!(validate_domain("example.com"));
    }

    #[test]
    fn test_validator_helpers_empty_string() {
        assert!(!validate_email(""));
        assert!(!validate_slug(""));
        assert!(!validate_semver(""));
        assert!(!validate_uuid_v4(""));
        assert!(!validate_ulid(""));
        assert!(!validate_iso_date(""));
        assert!(!validate_iso_timestamp_utc(""));
        assert!(!validate_domain(""));
    }

    #[test]
    fn test_regex_search_vs_matches() {
        let pattern = Pattern {
            id: "test".to_string(),
            name: "Test".to_string(),
            kind: PatternKind::Regex,
            pattern: "foo".to_string(),
            flags: HashMap::new(),
            description: None,
            examples: Vec::new(),
        };

        let compiled = pattern.compile().expect("compile");
        assert!(!compiled.matches("barfoo").unwrap());
        assert!(compiled.search("barfoo").unwrap());
    }

    #[test]
    fn test_literal_find_all() {
        let pattern = Pattern {
            id: "literal".to_string(),
            name: "Literal".to_string(),
            kind: PatternKind::Literal,
            pattern: "test".to_string(),
            flags: HashMap::new(),
            description: None,
            examples: Vec::new(),
        };

        let compiled = pattern.compile().expect("compile");
        let matches = compiled.find_all("test test").expect("find");
        assert_eq!(matches, vec!["test", "test"]);
    }
}
