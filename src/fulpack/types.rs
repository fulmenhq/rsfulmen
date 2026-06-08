//! Shared types for the `fulpack` archive module.
//!
//! Field names and enums mirror the Crucible fulpack v1.0.0 schemas under
//! `schemas/crucible-rs/library/fulpack/v1.0.0/` so payloads round-trip across
//! the Fulmen language libraries.

use serde::{Deserialize, Serialize};

/// Supported archive formats (taxonomy: archive-formats v1.0.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveFormat {
    /// POSIX tar, uncompressed.
    Tar,
    /// POSIX tar with gzip compression.
    #[serde(rename = "tar.gz")]
    TarGz,
    /// ZIP archive with deflate compression.
    Zip,
    /// GZIP-compressed single file.
    Gzip,
}

/// Archive entry kind (taxonomy: entry-types v1.0.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    /// Regular file with data.
    File,
    /// Directory entry.
    Directory,
    /// Symbolic link (security-validated on extract/verify, not on scan).
    Symlink,
}

/// Compression algorithm reported in [`ArchiveInfo`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Compression {
    /// gzip (tar.gz, gzip).
    Gzip,
    /// deflate (zip).
    Deflate,
    /// no compression (tar).
    None,
}

/// Checksum algorithm names from the schema enum. `xxh3-128` and `sha256` are
/// supported by [`crate::fulhash`]; the rest are reserved for cross-language parity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChecksumAlgorithm {
    /// xxh3-128 (fast, non-cryptographic).
    #[serde(rename = "xxh3-128")]
    Xxh3_128,
    /// SHA-256 (default).
    Sha256,
    /// SHA-512 (reserved).
    Sha512,
    /// SHA-1 (reserved).
    Sha1,
    /// MD5 (reserved).
    Md5,
}

/// Overwrite policy for extraction (reserved for the extract path, PR-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverwriteMode {
    /// Fail if the destination exists.
    Error,
    /// Keep the existing file.
    Skip,
    /// Replace the existing file.
    Overwrite,
}

/// A single archive entry (schema: `archive-entry.schema.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveEntry {
    /// Normalized entry path.
    pub path: String,
    /// Entry kind.
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    /// Uncompressed size in bytes.
    pub size: u64,
    /// Compressed size, when the format reports it.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub compressed_size: Option<u64>,
    /// Modification time (RFC 3339), when available.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub modified: Option<String>,
    /// SHA-256 checksum hex, when present in a manifest.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub checksum: Option<String>,
    /// Unix mode as octal string (e.g. `0644`), when available.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub mode: Option<String>,
    /// Raw, unresolved symlink target (symlink entries only).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub symlink_target: Option<String>,
}

/// Archive metadata without extraction (schema: `archive-info.schema.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveInfo {
    /// Detected format.
    pub format: ArchiveFormat,
    /// Compression algorithm.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub compression: Option<Compression>,
    /// Total entries.
    pub entry_count: u64,
    /// Sum of uncompressed entry sizes.
    pub total_size: u64,
    /// Archive file size on disk.
    pub compressed_size: u64,
    /// `total_size / compressed_size` (1.0 for uncompressed tar).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub compression_ratio: Option<f64>,
    /// Whether per-entry checksums are present.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub has_checksums: Option<bool>,
    /// Checksum algorithm, when `has_checksums`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// Archive creation time (RFC 3339), when available.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub created: Option<String>,
    /// Archive-level checksums by algorithm name, populated by [`crate::fulpack::create`].
    ///
    /// Note: the v1.0.0 `archive-info` schema does not yet declare this field (it
    /// is `additionalProperties: false`), but the fulpack standard's prose and the
    /// other language libraries return checksums here. The schema fix is escalated
    /// upstream to Crucible. Absent (skipped) for `info`/`scan`, so their output
    /// stays schema-valid — only `create` populates it.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub checksums: Option<std::collections::BTreeMap<String, String>>,
}

fn default_true() -> bool {
    true
}

fn default_scan_max_entries() -> u64 {
    100_000
}

/// Options for [`crate::fulpack::scan`] (schema: `scan-options.schema.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanOptions {
    /// Populate size/mtime/mode metadata per entry.
    #[serde(default = "default_true")]
    pub include_metadata: bool,
    /// Restrict to these entry kinds (all kinds when `None`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub entry_types: Option<Vec<EntryType>>,
    /// Maximum path depth to include (unlimited when `None`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_depth: Option<u32>,
    /// Safety cap on the number of entries scanned.
    #[serde(default = "default_scan_max_entries")]
    pub max_entries: u64,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            include_metadata: true,
            entry_types: None,
            max_depth: None,
            max_entries: default_scan_max_entries(),
        }
    }
}

// ---------------------------------------------------------------------------
// Types below are part of the stable public surface but are populated/consumed
// by the write paths (extract/create/verify) landing in later PRs.
// ---------------------------------------------------------------------------

/// Options for archive creation (schema: `create-options.schema.json`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateOptions {
    /// Compression level 1-9 (ignored for uncompressed tar).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub compression_level: Option<u32>,
    /// Glob include patterns.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub include_patterns: Option<Vec<String>>,
    /// Glob exclude patterns.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub exclude_patterns: Option<Vec<String>>,
    /// Per-entry checksum algorithm.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub checksum_algorithm: Option<ChecksumAlgorithm>,
    /// Preserve Unix permissions.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub preserve_permissions: Option<bool>,
    /// Follow symlinks while archiving.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub follow_symlinks: Option<bool>,
}

/// Options for archive extraction (schema: `extract-options.schema.json`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractOptions {
    /// Overwrite policy for existing files.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub overwrite: Option<OverwriteMode>,
    /// Verify entry checksums when present.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub verify_checksums: Option<bool>,
    /// Restore Unix permissions.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub preserve_permissions: Option<bool>,
    /// Glob include patterns.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub include_patterns: Option<Vec<String>>,
    /// Decompression-bomb guard: max total extracted bytes.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_size: Option<u64>,
    /// Decompression-bomb guard: max entries.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_entries: Option<u64>,
}

/// Result of an extraction (schema: `extract-result.schema.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractResult {
    /// Files written.
    pub extracted_count: u64,
    /// Entries skipped.
    pub skipped_count: u64,
    /// Entries that errored.
    pub error_count: u64,
    /// Error messages.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub errors: Option<Vec<String>>,
    /// Warning messages.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub warnings: Option<Vec<String>>,
    /// Number of checksum verifications performed.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub checksums_verified: Option<u64>,
    /// Total bytes written.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub total_bytes: Option<u64>,
}

/// Result of archive verification (schema: `validation-result.schema.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the archive passed all checks.
    pub valid: bool,
    /// Validation errors.
    pub errors: Vec<String>,
    /// Validation warnings.
    pub warnings: Vec<String>,
    /// Entries validated.
    pub entry_count: u64,
    /// Checksum verifications performed.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub checksums_verified: Option<u64>,
    /// Checks performed.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub checks_performed: Option<Vec<String>>,
}

/// Full archive manifest / table of contents (schema: `archive-manifest.schema.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveManifest {
    /// Archive format.
    pub format: ArchiveFormat,
    /// Manifest schema version (semver).
    pub version: String,
    /// Generation time (RFC 3339).
    pub generated: String,
    /// Total entries.
    pub entry_count: u64,
    /// All entries.
    pub entries: Vec<ArchiveEntry>,
    /// Sum of uncompressed sizes.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub total_size: Option<u64>,
    /// Archive file size.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub compressed_size: Option<u64>,
}
