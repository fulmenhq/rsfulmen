//! Crucible Shim
//!
//! Provides read-only, deterministic access to embedded Crucible assets (docs,
//! schemas, config) and exposes SSOT sync metadata.
//!
//! This module is the foundation for docscribe, schema validation, and
//! ecosystem version reporting.

use once_cell::sync::Lazy;
use serde::Deserialize;

/// Asset categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetCategory {
    /// Documentation assets from `docs/crucible-rs`.
    Docs,
    /// Schema assets from `schemas/crucible-rs`.
    Schemas,
    /// Config assets from `config/crucible-rs`.
    Config,
}

/// An embedded Crucible asset entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrucibleAsset {
    /// Asset category.
    pub category: AssetCategory,
    /// Path relative to the category root.
    pub path: &'static str,
    /// Byte length.
    pub bytes_len: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct SourceMetadataFile {
    generated_at: String,
    sources: Vec<SourceMetadataSource>,
}

#[derive(Debug, Clone, Deserialize)]
struct SourceMetadataSource {
    version: String,
    commit: String,
    method: String,

    // Present in the on-disk schema; kept for future parity.
    #[allow(dead_code)]
    #[serde(default)]
    forced_remote: bool,
}

/// Snapshot metadata for the embedded Crucible sync.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrucibleMetadata {
    /// CalVer-ish Crucible version without leading `v` (e.g. `0.3.1`).
    pub version: String,
    /// Commit SHA.
    pub commit: String,
    /// True when sync was performed from a dirty working tree.
    pub dirty: bool,
    /// Sync timestamp (RFC3339) as recorded by goneat.
    pub synced_at: String,
    /// Sync method (e.g. `git_ref`).
    pub sync_method: String,
}

const METADATA_YAML: &str = include_str!("../../.crucible/metadata/metadata.yaml");

static METADATA: Lazy<CrucibleMetadata> = Lazy::new(|| {
    let parsed: SourceMetadataFile = serde_yaml::from_str(METADATA_YAML)
        .expect("failed to parse .crucible/metadata/metadata.yaml");

    let source = parsed
        .sources
        .into_iter()
        .next()
        .expect("metadata.yaml should include at least one source");

    CrucibleMetadata {
        version: source.version,
        commit: source.commit,
        dirty: false,
        synced_at: parsed.generated_at,
        sync_method: source.method,
    }
});

/// Return sync metadata for the embedded Crucible snapshot.
pub fn metadata() -> &'static CrucibleMetadata {
    &METADATA
}

/// Return the embedded Crucible version string (with `v` prefix).
///
/// This mirrors `rsfulmen::CRUCIBLE_VERSION`.
pub fn version() -> &'static str {
    crate::CRUCIBLE_VERSION
}

// Generated index and open_* helpers.
#[allow(missing_docs)]
mod generated {
    use super::{AssetCategory, CrucibleAsset};

    include!(concat!(env!("OUT_DIR"), "/crucible_asset_index.rs"));
}

/// List embedded docs assets.
pub fn list_docs() -> &'static [CrucibleAsset] {
    generated::DOCS
}

/// List embedded schema assets.
pub fn list_schemas() -> &'static [CrucibleAsset] {
    generated::SCHEMAS
}

/// List embedded config assets.
pub fn list_config() -> &'static [CrucibleAsset] {
    generated::CONFIG
}

/// Open an embedded doc asset by path.
///
/// Paths are relative to `docs/crucible-rs/`.
pub fn open_docs(path: &str) -> Option<&'static [u8]> {
    generated::open_doc(path)
}

/// Open an embedded schema asset by path.
///
/// Paths are relative to `schemas/crucible-rs/`.
pub fn open_schemas(path: &str) -> Option<&'static [u8]> {
    generated::open_schema(path)
}

/// Open an embedded config asset by path.
///
/// Paths are relative to `config/crucible-rs/`.
pub fn open_config_bytes(path: &str) -> Option<&'static [u8]> {
    generated::open_config(path)
}

/// Open an embedded doc as UTF-8.
pub fn open_doc_str(path: &str) -> Option<&'static str> {
    open_docs(path).and_then(|b| std::str::from_utf8(b).ok())
}

/// Open an asset by category.
pub fn open(category: AssetCategory, path: &str) -> Option<&'static [u8]> {
    match category {
        AssetCategory::Docs => open_docs(path),
        AssetCategory::Schemas => open_schemas(path),
        AssetCategory::Config => open_config_bytes(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_parses_and_matches_version() {
        let m = metadata();
        assert!(!m.version.is_empty());
        assert!(!m.commit.is_empty());

        let expected = crate::CRUCIBLE_VERSION.trim_start_matches('v');
        assert_eq!(m.version, expected);
    }

    #[test]
    fn test_asset_lists_non_empty() {
        assert!(!list_docs().is_empty());
        assert!(!list_schemas().is_empty());
        assert!(!list_config().is_empty());
    }

    #[test]
    fn test_open_known_doc() {
        let content = open_doc_str("architecture/fulmen-technical-manifesto.md").unwrap();
        assert!(content.contains("Fulmen"));
    }

    #[test]
    fn test_open_unknown_returns_none() {
        assert!(open_docs("nope.md").is_none());
        assert!(open_schemas("nope.json").is_none());
        assert!(open_config_bytes("nope.yaml").is_none());
    }
}
