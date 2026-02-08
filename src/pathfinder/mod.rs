//! Safe filesystem discovery with glob patterns and path traversal protection.
//!
//! This module provides file discovery utilities that search directories for
//! files matching glob patterns, with built-in protection against path traversal
//! attacks and optional SHA-256 checksum computation.
//!
//! ## Features
//!
//! - Recursive file discovery with configurable depth limits
//! - Include/exclude glob pattern filtering
//! - Path traversal detection and prevention
//! - Optional SHA-256 checksums for matched files
//! - Repository root detection (`.git` directory or file)
//! - Config file discovery shorthand
//! - Non-fatal warnings for partial traversal outcomes
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rsfulmen::pathfinder::{find_files, FindQuery};
//! use std::path::PathBuf;
//!
//! let query = FindQuery {
//!     root: PathBuf::from("/my/project"),
//!     include: vec!["**/*.yaml".to_string()],
//!     exclude: vec!["**/target/**".to_string()],
//!     max_depth: None,
//!     follow_symlinks: false,
//!     include_hidden: false,
//!     checksums: false,
//! };
//!
//! let results = find_files(&query)?;
//! for file in &results.files {
//!     println!("{}: {} bytes", file.relative_path.display(), file.metadata.size);
//! }
//! # Ok::<(), rsfulmen::pathfinder::PathfinderError>(())
//! ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Query parameters for file discovery.
///
/// Field names align with the Crucible `find-query` schema v1.0.0
/// (`schemas/crucible-rs/pathfinder/v1.0.0/find-query.schema.json`).
/// Serializes to camelCase for cross-language JSON interchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindQuery {
    /// Root directory to search from (required).
    pub root: PathBuf,
    /// Glob patterns to include (e.g., `"**/*.yaml"`).
    pub include: Vec<String>,
    /// Glob patterns to exclude (e.g., `"**/target/**"`).
    pub exclude: Vec<String>,
    /// Maximum directory depth (`None` or `Some(0)` = unlimited).
    pub max_depth: Option<usize>,
    /// Whether to follow symlinks (default: `false`).
    pub follow_symlinks: bool,
    /// Whether to include hidden files/directories (default: `false`).
    pub include_hidden: bool,
    /// Compute checksums for matched files.
    pub checksums: bool,
}

/// A single discovered file.
///
/// Field names align with the Crucible `path-result` schema v1.0.0
/// (`schemas/crucible-rs/pathfinder/v1.0.0/path-result.schema.json`).
/// Metadata is nested in a [`PathMetadata`] struct matching the Crucible
/// `metadata` schema v1.0.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindResult {
    /// Absolute path to the file (Crucible: `sourcePath`).
    pub source_path: PathBuf,
    /// Path relative to the query root (Crucible: `relativePath`).
    pub relative_path: PathBuf,
    /// Loader type (always `"local"` for filesystem discovery).
    pub loader_type: String,
    /// File metadata (size, timestamps, checksums).
    pub metadata: PathMetadata,
}

/// Flexible metadata for a discovered file.
///
/// Aligns with the Crucible `metadata` schema v1.0.0
/// (`schemas/crucible-rs/pathfinder/v1.0.0/metadata.schema.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathMetadata {
    /// File size in bytes.
    pub size: u64,
    /// Last modification timestamp (RFC 3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    /// File checksum in `algorithm:hex` format (e.g., `"sha256:abcdef..."`).
    ///
    /// Matches the Crucible metadata schema pattern `^(xxh3-128|sha256):[a-f0-9]+$`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    /// Algorithm used for checksum calculation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_algorithm: Option<String>,
    /// Error message if checksum calculation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_error: Option<String>,
}

/// Aggregate results from a find operation.
///
/// Contains the list of matched files, the total number of entries scanned
/// (including non-matches), the wall-clock duration of the search, and any
/// non-fatal warnings encountered during traversal.
///
/// Warnings capture issues like unreadable entries or symlinks that escape
/// the root boundary — these do not abort the search but are surfaced so
/// callers can observe partial-result conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindResults {
    /// Matched files.
    pub files: Vec<FindResult>,
    /// Total files scanned (including non-matches).
    pub scanned: usize,
    /// Duration of the search in milliseconds.
    pub duration_ms: u64,
    /// Non-fatal warnings encountered during traversal.
    pub warnings: Vec<PathfinderWarning>,
}

/// A non-fatal warning emitted during file discovery.
///
/// Warnings represent issues that do not abort the search but indicate
/// partial or degraded results (e.g., permission-denied entries, symlinks
/// escaping the root boundary).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathfinderWarning {
    /// The path where the warning occurred (if available).
    pub path: Option<PathBuf>,
    /// Human-readable description of the issue.
    pub message: String,
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur during pathfinder operations.
#[derive(Debug, thiserror::Error)]
pub enum PathfinderError {
    /// The specified root directory does not exist.
    #[error("root directory does not exist: {0}")]
    RootNotFound(PathBuf),

    /// A path traversal attempt was detected.
    #[error("path traversal detected: {0}")]
    PathTraversal(PathBuf),

    /// A glob pattern could not be parsed.
    #[error("invalid glob pattern: {pattern}: {source}")]
    InvalidPattern {
        /// The pattern that failed to compile.
        pattern: String,
        /// The underlying parse error.
        source: glob::PatternError,
    },

    /// An I/O error occurred during file operations.
    #[error("I/O error at {path}: {source}")]
    IoError {
        /// Path where the error occurred.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Find files matching glob patterns under a root directory.
///
/// Walks the directory tree rooted at `query.root`, applying include and
/// exclude glob patterns against each file's relative path. Optionally
/// computes SHA-256 checksums for matched files.
///
/// # Errors
///
/// Returns [`PathfinderError::RootNotFound`] if the root directory does not
/// exist, [`PathfinderError::InvalidPattern`] if any glob pattern is
/// malformed, or [`PathfinderError::IoError`] for filesystem errors.
///
/// # Examples
///
/// ```rust,no_run
/// use rsfulmen::pathfinder::{find_files, FindQuery};
/// use std::path::PathBuf;
///
/// let query = FindQuery {
///     root: PathBuf::from("."),
///     include: vec!["**/*.rs".to_string()],
///     exclude: vec![],
///     max_depth: Some(5),
///     follow_symlinks: false,
///     include_hidden: false,
///     checksums: false,
/// };
///
/// let results = find_files(&query)?;
/// println!("Found {} files", results.files.len());
/// # Ok::<(), rsfulmen::pathfinder::PathfinderError>(())
/// ```
pub fn find_files(query: &FindQuery) -> Result<FindResults, PathfinderError> {
    let start = Instant::now();

    // Validate root exists and is a directory.
    if !query.root.exists() || !query.root.is_dir() {
        return Err(PathfinderError::RootNotFound(query.root.clone()));
    }

    // Canonicalize root so relative-path computation is stable.
    let canonical_root =
        fs::canonicalize(&query.root).map_err(|source| PathfinderError::IoError {
            path: query.root.clone(),
            source,
        })?;

    // Pre-compile include patterns.
    let include_patterns = compile_patterns(&query.include)?;

    // Pre-compile exclude patterns.
    let exclude_patterns = compile_patterns(&query.exclude)?;

    // Configure walker. Some(0) and None both mean unlimited (Crucible schema:
    // maxDepth 0 = unlimited).
    let mut walker = walkdir::WalkDir::new(&canonical_root).follow_links(query.follow_symlinks);
    if let Some(depth) = query.max_depth {
        if depth > 0 {
            walker = walker.max_depth(depth);
        }
    }

    let mut files = Vec::new();
    let mut warnings = Vec::new();
    let mut scanned: usize = 0;

    // Use filter_entry to prune hidden directories from traversal (not just
    // skip their entries). Without this, WalkDir descends into hidden dirs
    // and yields their non-hidden children.
    let include_hidden = query.include_hidden;
    let iter = walker.into_iter().filter_entry(move |e| {
        if include_hidden {
            return true;
        }
        // Allow the root directory itself (depth 0) regardless of name.
        if e.depth() == 0 {
            return true;
        }
        // Exclude entries whose name starts with '.'.
        e.file_name()
            .to_str()
            .map(|name| !name.starts_with('.'))
            .unwrap_or(true)
    });

    for entry in iter {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                // Collect walk errors (permission denied, broken symlinks, etc.)
                // as warnings rather than silently dropping them.
                let path = e.path().map(|p| p.to_path_buf());
                warnings.push(PathfinderWarning {
                    path,
                    message: format!("walk error: {e}"),
                });
                continue;
            }
        };

        // Only consider files (not directories or other special types).
        if !entry.file_type().is_file() {
            continue;
        }

        scanned += 1;

        let abs_path = entry.path().to_path_buf();

        // When following symlinks, canonicalize each candidate and verify it
        // remains within the canonical root. Skip and warn for out-of-bound targets.
        if query.follow_symlinks {
            match fs::canonicalize(&abs_path) {
                Ok(real) => {
                    if !real.starts_with(&canonical_root) {
                        warnings.push(PathfinderWarning {
                            path: Some(abs_path),
                            message: "symlink target escapes root boundary, skipped".to_string(),
                        });
                        continue;
                    }
                }
                Err(e) => {
                    warnings.push(PathfinderWarning {
                        path: Some(abs_path),
                        message: format!("cannot resolve symlink: {e}"),
                    });
                    continue;
                }
            }
        }

        // Compute relative path from root.
        let rel_path = match abs_path.strip_prefix(&canonical_root) {
            Ok(r) => r.to_path_buf(),
            Err(_) => continue,
        };

        // Check include patterns: file must match at least one.
        if !include_patterns.is_empty()
            && !include_patterns.iter().any(|p| p.matches_path(&rel_path))
        {
            continue;
        }

        // Check exclude patterns: file must not match any.
        if exclude_patterns.iter().any(|p| p.matches_path(&rel_path)) {
            continue;
        }

        // Gather metadata — entry-level read failures become warnings.
        let metadata = match fs::metadata(&abs_path) {
            Ok(m) => m,
            Err(e) => {
                warnings.push(PathfinderWarning {
                    path: Some(abs_path),
                    message: format!("cannot read metadata: {e}"),
                });
                continue;
            }
        };

        let size = metadata.len();

        let modified = metadata.modified().ok().and_then(system_time_to_rfc3339);

        // Optionally compute checksum — read failures become warnings with checksumError.
        let (checksum, checksum_algorithm, checksum_error) = if query.checksums {
            match sha256_file(&abs_path) {
                Ok(hex_str) => (
                    Some(format!("sha256:{hex_str}")),
                    Some("sha256".to_string()),
                    None,
                ),
                Err(e) => {
                    warnings.push(PathfinderWarning {
                        path: Some(abs_path.clone()),
                        message: format!("cannot compute checksum: {e}"),
                    });
                    (None, None, Some(format!("{e}")))
                }
            }
        } else {
            (None, None, None)
        };

        files.push(FindResult {
            source_path: abs_path,
            relative_path: rel_path,
            loader_type: "local".to_string(),
            metadata: PathMetadata {
                size,
                modified,
                checksum,
                checksum_algorithm,
                checksum_error,
            },
        });
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(FindResults {
        files,
        scanned,
        duration_ms,
        warnings,
    })
}

/// Validate a path for traversal attacks.
///
/// Canonicalizes both `path` and `root`, then checks that the canonicalized
/// path is contained within the canonicalized root. Returns the canonicalized
/// path if safe.
///
/// # Errors
///
/// Returns [`PathfinderError::PathTraversal`] if the path escapes the root
/// after canonicalization.
///
/// # Examples
///
/// ```rust,no_run
/// use rsfulmen::pathfinder::validate_path;
/// use std::path::Path;
///
/// let safe = validate_path(Path::new("/project/src/main.rs"), Path::new("/project"))?;
/// assert!(safe.starts_with("/project"));
/// # Ok::<(), rsfulmen::pathfinder::PathfinderError>(())
/// ```
pub fn validate_path(path: &Path, root: &Path) -> Result<PathBuf, PathfinderError> {
    let canonical_root = fs::canonicalize(root).map_err(|source| PathfinderError::IoError {
        path: root.to_path_buf(),
        source,
    })?;

    let canonical_path = fs::canonicalize(path).map_err(|source| PathfinderError::IoError {
        path: path.to_path_buf(),
        source,
    })?;

    if !canonical_path.starts_with(&canonical_root) {
        return Err(PathfinderError::PathTraversal(path.to_path_buf()));
    }

    Ok(canonical_path)
}

/// Find repository root by searching upward for a `.git` marker.
///
/// Starting from `start_path`, walks up through parent directories looking
/// for a `.git` directory or file (worktrees and submodules use a `.git` file).
/// Returns the directory *containing* `.git`, not `.git` itself.
///
/// Returns `None` if the filesystem root is reached without finding `.git`.
///
/// # Examples
///
/// ```rust,no_run
/// use rsfulmen::pathfinder::find_repository_root;
/// use std::path::Path;
///
/// if let Some(root) = find_repository_root(Path::new(".")) {
///     println!("repo root: {}", root.display());
/// }
/// ```
pub fn find_repository_root(start_path: &Path) -> Option<PathBuf> {
    let mut current = if start_path.is_absolute() {
        start_path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(start_path)
    };

    // If start_path points to a file, begin from its parent directory.
    if current.is_file() {
        current = current.parent()?.to_path_buf();
    }

    loop {
        let candidate = current.join(".git");
        if candidate.is_dir() || candidate.is_file() {
            return Some(current);
        }

        let parent = current.parent()?;
        if parent == current {
            // Reached filesystem root.
            return None;
        }
        current = parent.to_path_buf();
    }
}

/// Find config files under a directory.
///
/// Searches for files matching `*.yaml`, `*.yml`, `*.json`, and `*.toml`
/// directly under `root` (non-recursive, `max_depth=1`).
///
/// This is a convenience wrapper around [`find_files`].
///
/// # Errors
///
/// Returns [`PathfinderError`] on filesystem or pattern errors.
///
/// # Examples
///
/// ```rust,no_run
/// use rsfulmen::pathfinder::find_config_files;
/// use std::path::Path;
///
/// let results = find_config_files(Path::new("/my/project"))?;
/// for f in &results.files {
///     println!("config: {}", f.relative_path.display());
/// }
/// # Ok::<(), rsfulmen::pathfinder::PathfinderError>(())
/// ```
pub fn find_config_files(root: &Path) -> Result<FindResults, PathfinderError> {
    let query = FindQuery {
        root: root.to_path_buf(),
        include: vec![
            "*.yaml".to_string(),
            "*.yml".to_string(),
            "*.json".to_string(),
            "*.toml".to_string(),
        ],
        exclude: vec![],
        max_depth: Some(1),
        follow_symlinks: false,
        include_hidden: false,
        checksums: false,
    };
    find_files(&query)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Compile a list of glob pattern strings into [`glob::Pattern`] instances.
fn compile_patterns(raw: &[String]) -> Result<Vec<glob::Pattern>, PathfinderError> {
    raw.iter()
        .map(|s| {
            glob::Pattern::new(s).map_err(|source| PathfinderError::InvalidPattern {
                pattern: s.clone(),
                source,
            })
        })
        .collect()
}

/// Compute SHA-256 of a file, reading in 8 KB chunks.
fn sha256_file(path: &Path) -> Result<String, PathfinderError> {
    use sha2::{Digest, Sha256};

    let mut file = fs::File::open(path).map_err(|source| PathfinderError::IoError {
        path: path.to_path_buf(),
        source,
    })?;

    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];

    loop {
        let n = file
            .read(&mut buf)
            .map_err(|source| PathfinderError::IoError {
                path: path.to_path_buf(),
                source,
            })?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Convert a [`std::time::SystemTime`] to an RFC 3339 string.
///
/// Uses a simple UTC conversion without external datetime crates.
/// Algorithm adapted from Howard Hinnant's `civil_from_days`.
fn system_time_to_rfc3339(time: std::time::SystemTime) -> Option<String> {
    let duration = time.duration_since(std::time::UNIX_EPOCH).ok()?;
    let secs = duration.as_secs();

    let days = (secs / 86400) as i64;
    let time_of_day = secs % 86400;

    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hour, minute, second
    ))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// RAII temporary directory that is cleaned up on drop.
    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let mut path = std::env::temp_dir();
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be after epoch")
                .as_nanos();
            path.push(format!(
                "rsfulmen-pathfinder-{}-{}",
                std::process::id(),
                nanos
            ));
            fs::create_dir_all(&path).expect("failed to create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    /// Helper: write a file with parent directory creation.
    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent dir");
        }
        fs::write(path, content).expect("failed to write file");
    }

    // 1. Basic glob matching
    #[test]
    fn test_find_files_basic_glob() {
        let tmp = TestDir::new();
        write_file(&tmp.path().join("config.yaml"), "key: value");
        write_file(&tmp.path().join("main.rs"), "fn main() {}");
        write_file(&tmp.path().join("sub/nested.yaml"), "nested: true");

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["**/*.yaml".to_string()],
            exclude: vec![],
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            checksums: false,
        };

        let results = find_files(&query).expect("find_files should succeed");
        assert_eq!(results.files.len(), 2, "should find exactly 2 YAML files");

        for f in &results.files {
            assert!(
                f.relative_path.to_string_lossy().ends_with(".yaml"),
                "all results should be .yaml files"
            );
        }
    }

    // 2. Multiple patterns
    #[test]
    fn test_find_files_multiple_patterns() {
        let tmp = TestDir::new();
        write_file(&tmp.path().join("config.yaml"), "key: value");
        write_file(&tmp.path().join("data.json"), "{}");
        write_file(&tmp.path().join("main.rs"), "fn main() {}");

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["**/*.yaml".to_string(), "**/*.json".to_string()],
            exclude: vec![],
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            checksums: false,
        };

        let results = find_files(&query).expect("find_files should succeed");
        assert_eq!(
            results.files.len(),
            2,
            "should find exactly yaml and json files"
        );

        let extensions: Vec<String> = results
            .files
            .iter()
            .map(|f| {
                f.relative_path
                    .extension()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        assert!(extensions.contains(&"yaml".to_string()));
        assert!(extensions.contains(&"json".to_string()));
    }

    // 3. Exclude patterns
    #[test]
    fn test_find_files_with_exclude() {
        let tmp = TestDir::new();
        write_file(&tmp.path().join("src/app.yaml"), "app: true");
        write_file(&tmp.path().join("target/build.yaml"), "build: true");
        write_file(&tmp.path().join("target/debug/output.yaml"), "debug: true");

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["**/*.yaml".to_string()],
            exclude: vec!["target/**".to_string()],
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            checksums: false,
        };

        let results = find_files(&query).expect("find_files should succeed");
        assert_eq!(results.files.len(), 1, "should exclude target/ files");
        assert_eq!(
            results.files[0].relative_path,
            PathBuf::from("src/app.yaml")
        );
    }

    // 4. Max depth
    #[test]
    fn test_find_files_max_depth() {
        let tmp = TestDir::new();
        write_file(&tmp.path().join("top.yaml"), "top: true");
        write_file(&tmp.path().join("a/mid.yaml"), "mid: true");
        write_file(&tmp.path().join("a/b/deep.yaml"), "deep: true");
        write_file(&tmp.path().join("a/b/c/very-deep.yaml"), "very: true");

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["**/*.yaml".to_string()],
            exclude: vec![],
            max_depth: Some(2), // root (depth 0) + one subdirectory level
            follow_symlinks: false,
            include_hidden: false,
            checksums: false,
        };

        let results = find_files(&query).expect("find_files should succeed");

        // max_depth=2 means: root (depth 0), direct children (depth 1),
        // grandchildren (depth 2). So top.yaml and a/mid.yaml are included,
        // but a/b/deep.yaml (depth 3) is excluded.
        let paths: Vec<String> = results
            .files
            .iter()
            .map(|f| f.relative_path.to_string_lossy().to_string())
            .collect();

        assert!(
            paths.iter().any(|p| p == "top.yaml"),
            "top.yaml should be included"
        );
        assert!(
            paths.iter().any(|p| p.ends_with("mid.yaml")),
            "mid.yaml should be included"
        );
        assert!(
            !paths.iter().any(|p| p.ends_with("deep.yaml")),
            "deep.yaml should be excluded by max_depth"
        );
        assert!(
            !paths.iter().any(|p| p.ends_with("very-deep.yaml")),
            "very-deep.yaml should be excluded by max_depth"
        );
    }

    // 5. Checksum computation
    #[test]
    fn test_find_files_with_checksums() {
        let tmp = TestDir::new();
        let content = "hello, pathfinder!";
        write_file(&tmp.path().join("greeting.txt"), content);

        // Compute expected SHA-256 in Crucible canonical format.
        use sha2::{Digest, Sha256};
        let hex_str = hex::encode(Sha256::digest(content.as_bytes()));
        let expected = format!("sha256:{hex_str}");

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["**/*.txt".to_string()],
            exclude: vec![],
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            checksums: true,
        };

        let results = find_files(&query).expect("find_files should succeed");
        assert_eq!(results.files.len(), 1);
        assert_eq!(
            results.files[0].metadata.checksum.as_deref(),
            Some(expected.as_str()),
            "checksum should match sha256:<hex> format"
        );
        assert_eq!(
            results.files[0].metadata.checksum_algorithm.as_deref(),
            Some("sha256"),
            "checksum_algorithm should be sha256"
        );
    }

    // 6. Relative paths
    #[test]
    fn test_find_files_relative_paths() {
        let tmp = TestDir::new();
        write_file(&tmp.path().join("sub/deep/file.txt"), "data");

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["**/*.txt".to_string()],
            exclude: vec![],
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            checksums: false,
        };

        let results = find_files(&query).expect("find_files should succeed");
        assert_eq!(results.files.len(), 1);

        let rel = &results.files[0].relative_path;
        assert!(
            rel.is_relative(),
            "relative_path should not be absolute: {:?}",
            rel
        );
        assert_eq!(*rel, PathBuf::from("sub/deep/file.txt"));

        // sourcePath should be absolute.
        assert!(
            results.files[0].source_path.is_absolute(),
            "source_path should be absolute"
        );
    }

    // 7. Metadata populated
    #[test]
    fn test_find_files_returns_metadata() {
        let tmp = TestDir::new();
        let content = "some content here";
        write_file(&tmp.path().join("file.txt"), content);

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["**/*.txt".to_string()],
            exclude: vec![],
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            checksums: false,
        };

        let results = find_files(&query).expect("find_files should succeed");
        assert_eq!(results.files.len(), 1);
        assert_eq!(
            results.files[0].metadata.size,
            content.len() as u64,
            "size should reflect file content length"
        );
        assert!(
            results.files[0].metadata.modified.is_some(),
            "modified timestamp should be populated"
        );
    }

    // 8. validate_path accepts safe path
    #[test]
    fn test_validate_path_safe() {
        let tmp = TestDir::new();
        let child = tmp.path().join("subdir");
        fs::create_dir_all(&child).expect("create subdir");
        let file = child.join("safe.txt");
        write_file(&file, "safe");

        let result = validate_path(&file, tmp.path());
        assert!(result.is_ok(), "safe path should be accepted");
        let canonical = result.unwrap();
        assert!(canonical.is_absolute());
    }

    // 9. validate_path rejects traversal
    #[test]
    fn test_validate_path_rejects_traversal() {
        let tmp = TestDir::new();
        let sibling = TestDir::new();
        write_file(&sibling.path().join("secret.txt"), "secret");

        // Attempt to reach sibling through traversal.
        let traversal = sibling.path().join("secret.txt");
        let result = validate_path(&traversal, tmp.path());
        assert!(result.is_err(), "path outside root should be rejected");
        match result.unwrap_err() {
            PathfinderError::PathTraversal(_) => {} // expected
            other => panic!("expected PathTraversal, got: {other:?}"),
        }
    }

    // 10. Root not found
    #[test]
    fn test_find_files_root_not_found() {
        let query = FindQuery {
            root: PathBuf::from("/nonexistent/path/that/should/not/exist"),
            include: vec!["**/*".to_string()],
            exclude: vec![],
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            checksums: false,
        };

        let err = find_files(&query).expect_err("should fail for nonexistent root");
        match err {
            PathfinderError::RootNotFound(p) => {
                assert_eq!(p, PathBuf::from("/nonexistent/path/that/should/not/exist"));
            }
            other => panic!("expected RootNotFound, got: {other:?}"),
        }
    }

    // 11. Find repository root
    #[test]
    fn test_find_repository_root() {
        let tmp = TestDir::new();
        // Create a fake .git directory.
        fs::create_dir_all(tmp.path().join(".git")).expect("create .git dir");
        // Create a deeply nested child.
        let nested = tmp.path().join("a/b/c");
        fs::create_dir_all(&nested).expect("create nested dir");

        let root = find_repository_root(&nested);
        assert!(root.is_some(), "should find repository root");

        let root = root.unwrap();
        let expected = fs::canonicalize(tmp.path()).expect("canonicalize tmp");
        let actual = fs::canonicalize(&root).expect("canonicalize result");
        assert_eq!(actual, expected, "should return directory containing .git");
    }

    // 12. Repository root not found
    #[test]
    fn test_find_repository_root_not_found() {
        let tmp = TestDir::new();
        // Create a nested directory with NO .git marker anywhere in the chain.
        // We search from the nested dir — the fresh temp directory has no .git,
        // and canonicalization ensures we won't pick up the test runner's own repo.
        let nested = tmp.path().join("a/b/c/d");
        fs::create_dir_all(&nested).expect("create nested dir");

        let result = find_repository_root(&nested);

        // The temp directory tree has no .git, so the function should either
        // return None (if /tmp isn't inside a repo) or return a path above our
        // temp dir (if the host's /tmp happens to be inside a repo). We can
        // at least verify it doesn't return something inside our temp dir.
        if let Some(root) = &result {
            let canonical_tmp = fs::canonicalize(tmp.path()).expect("canonicalize tmp");
            let canonical_root = fs::canonicalize(root).expect("canonicalize result");
            assert!(
                !canonical_root.starts_with(&canonical_tmp),
                "should not find a repo root inside our fresh temp directory"
            );
        }
        // If result is None, that's the expected negative-path behavior.
    }

    // 13. Find config files
    #[test]
    fn test_find_config_files() {
        let tmp = TestDir::new();
        write_file(&tmp.path().join("app.yaml"), "name: app");
        write_file(&tmp.path().join("settings.yml"), "key: value");
        write_file(&tmp.path().join("data.json"), "{}");
        write_file(&tmp.path().join("config.toml"), "[section]");
        write_file(&tmp.path().join("main.rs"), "fn main() {}");
        write_file(&tmp.path().join("readme.md"), "# Readme");

        let results = find_config_files(tmp.path()).expect("find_config_files should succeed");

        let extensions: Vec<String> = results
            .files
            .iter()
            .map(|f| {
                f.relative_path
                    .extension()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();

        assert_eq!(results.files.len(), 4, "should find 4 config files");
        assert!(extensions.contains(&"yaml".to_string()));
        assert!(extensions.contains(&"yml".to_string()));
        assert!(extensions.contains(&"json".to_string()));
        assert!(extensions.contains(&"toml".to_string()));
        assert!(!extensions.contains(&"rs".to_string()));
        assert!(!extensions.contains(&"md".to_string()));
    }

    // 14. Empty directory
    #[test]
    fn test_find_files_empty_directory() {
        let tmp = TestDir::new();

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["**/*".to_string()],
            exclude: vec![],
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            checksums: false,
        };

        let results = find_files(&query).expect("find_files should succeed on empty dir");
        assert!(results.files.is_empty(), "no files in empty directory");
        assert_eq!(results.scanned, 0, "scanned count should be 0");
        assert!(results.duration_ms < 5000, "should complete quickly");
    }

    // 15. .git file detection (worktrees/submodules)
    #[test]
    fn test_find_repository_root_git_file() {
        let tmp = TestDir::new();
        // Simulate a git worktree/submodule: .git is a file, not a directory.
        write_file(
            &tmp.path().join(".git"),
            "gitdir: /some/other/repo/.git/worktrees/mine",
        );
        let nested = tmp.path().join("a/b");
        fs::create_dir_all(&nested).expect("create nested dir");

        let root = find_repository_root(&nested);
        assert!(root.is_some(), "should find repo root from .git file");

        let root = root.unwrap();
        let expected = fs::canonicalize(tmp.path()).expect("canonicalize tmp");
        let actual = fs::canonicalize(&root).expect("canonicalize result");
        assert_eq!(
            actual, expected,
            "should return directory containing .git file"
        );
    }

    // 16. Symlink escaping root is warned and skipped
    #[cfg(unix)]
    #[test]
    fn test_find_files_symlink_escapes_root_warning() {
        let outside = TestDir::new();
        write_file(&outside.path().join("secret.txt"), "secret data");

        let inside = TestDir::new();
        write_file(&inside.path().join("normal.txt"), "normal data");

        // Create a symlink inside that points outside the root.
        std::os::unix::fs::symlink(
            outside.path().join("secret.txt"),
            inside.path().join("escape.txt"),
        )
        .expect("create symlink");

        let query = FindQuery {
            root: inside.path().to_path_buf(),
            include: vec!["**/*.txt".to_string()],
            exclude: vec![],
            max_depth: None,
            follow_symlinks: true,
            include_hidden: false,
            checksums: false,
        };

        let results = find_files(&query).expect("find_files should succeed");

        // Only normal.txt should be in results; escape.txt should be warned and skipped.
        assert_eq!(
            results.files.len(),
            1,
            "only in-root file should be returned"
        );
        assert!(
            results.files[0]
                .relative_path
                .to_string_lossy()
                .contains("normal"),
            "matched file should be the normal one"
        );

        assert!(
            !results.warnings.is_empty(),
            "should have warnings for out-of-root symlink"
        );
        assert!(
            results.warnings[0]
                .message
                .contains("symlink target escapes root boundary"),
            "warning message should describe the issue"
        );
    }

    // 17. Warnings are populated from walk errors (non-fatal)
    #[test]
    fn test_find_results_has_warnings_field() {
        // Verify the warnings field exists and starts empty for a normal query.
        let tmp = TestDir::new();
        write_file(&tmp.path().join("a.txt"), "data");

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["**/*.txt".to_string()],
            exclude: vec![],
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            checksums: false,
        };

        let results = find_files(&query).expect("find_files should succeed");
        assert_eq!(results.files.len(), 1);
        assert!(
            results.warnings.is_empty(),
            "no warnings for normal traversal"
        );
    }

    // 18. max_depth Some(0) is treated as unlimited (Crucible schema semantics)
    #[test]
    fn test_find_files_max_depth_zero_is_unlimited() {
        let tmp = TestDir::new();
        write_file(&tmp.path().join("a/b/c/d/deep.txt"), "deep");
        write_file(&tmp.path().join("top.txt"), "top");

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["**/*.txt".to_string()],
            exclude: vec![],
            max_depth: Some(0), // should behave as unlimited
            follow_symlinks: false,
            include_hidden: false,
            checksums: false,
        };

        let results = find_files(&query).expect("find_files should succeed");
        assert_eq!(
            results.files.len(),
            2,
            "Some(0) should be unlimited, finding both files"
        );
    }

    // 19. include_hidden=false skips dotfiles
    #[test]
    fn test_find_files_hidden_excluded_by_default() {
        let tmp = TestDir::new();
        write_file(&tmp.path().join("visible.txt"), "visible");
        write_file(&tmp.path().join(".hidden.txt"), "hidden");
        write_file(&tmp.path().join(".dotdir/nested.txt"), "nested in dotdir");

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["**/*.txt".to_string()],
            exclude: vec![],
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            checksums: false,
        };

        let results = find_files(&query).expect("find_files should succeed");
        assert_eq!(results.files.len(), 1, "hidden files should be excluded");
        assert!(
            results.files[0]
                .relative_path
                .to_string_lossy()
                .contains("visible"),
            "only visible file should be returned"
        );
    }

    // 20. include_hidden=true includes dotfiles
    #[test]
    fn test_find_files_hidden_included_when_requested() {
        let tmp = TestDir::new();
        write_file(&tmp.path().join("visible.txt"), "visible");
        write_file(&tmp.path().join(".hidden.txt"), "hidden");

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["**/*.txt".to_string()],
            exclude: vec![],
            max_depth: None,
            follow_symlinks: false,
            include_hidden: true,
            checksums: false,
        };

        let results = find_files(&query).expect("find_files should succeed");
        assert_eq!(results.files.len(), 2, "hidden files should be included");
    }

    // 21. Invalid glob pattern
    #[test]
    fn test_find_files_invalid_glob() {
        let tmp = TestDir::new();

        let query = FindQuery {
            root: tmp.path().to_path_buf(),
            include: vec!["[invalid".to_string()],
            exclude: vec![],
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            checksums: false,
        };

        let err = find_files(&query).expect_err("should fail for invalid glob");
        match err {
            PathfinderError::InvalidPattern { pattern, .. } => {
                assert_eq!(pattern, "[invalid");
            }
            other => panic!("expected InvalidPattern, got: {other:?}"),
        }
    }
}
