//! Bounded repository root discovery.
//!
//! [`discover_repository_root`] walks upward from a start path looking for
//! marker files or directories (`.git`, `go.mod`, `package.json`, …) and
//! stops at the first match, a configured boundary, max depth, or the
//! filesystem root.
//!
//! This implements the Crucible FindRepositoryRoot contract for markers, max
//! depth, home/explicit ceilings, and symlink policy. PathConstraint /
//! RespectConstraints is **deferred** until a Rust constraint type exists;
//! pass an explicit [`RepositoryRootOptions::boundary`] instead.
//!
//! Traversal is lexical after the start directory is resolved. When
//! [`RepositoryRootOptions::follow_symlinks`] is `true`, the start and the
//! ceiling are canonicalized once and every hop uses that pair. When it is
//! `false`, only symlink components *between* the ceiling and the current
//! directory are skipped (ancestors of the ceiling, such as `/tmp` →
//! `/private/tmp`, do not hide markers). The filesystem root is a stop
//! condition and is not inspected for markers.
//!
//! The existing [`crate::pathfinder::find_repository_root`] helper stays as a
//! `.git`-only, unbounded `Option` walk for callers that want the historical
//! convenience API.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Git repository marker (`.git` directory or worktree file).
pub const GIT_MARKERS: &[&str] = &[".git"];
/// Go module marker.
pub const GO_MOD_MARKERS: &[&str] = &["go.mod"];
/// Node / npm package marker.
pub const NODE_MARKERS: &[&str] = &["package.json"];
/// Python project markers.
pub const PYTHON_MARKERS: &[&str] = &["pyproject.toml", "setup.py"];
/// Common monorepo markers.
pub const MONOREPO_MARKERS: &[&str] = &[".git", "pnpm-workspace.yaml", "lerna.json"];

/// Default upward walk depth (Crucible `FindOptions.MaxDepth`).
pub const DEFAULT_MAX_DEPTH: usize = 10;

/// Options for [`discover_repository_root`].
///
/// Marked `non_exhaustive` so PathConstraint / `respect_constraints` can be
/// added later without breaking struct literals in other crates. Construct
/// with [`Self::default`], [`Self::git`], or [`Self::with_markers`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RepositoryRootOptions {
    /// Marker file or directory names. Any match at a level wins.
    ///
    /// Order does not affect priority; the closest directory that contains
    /// any marker is returned when [`Self::stop_at_first`] is `true`.
    pub markers: Vec<String>,
    /// Maximum number of directories to inspect, including the start
    /// directory. Default: [`DEFAULT_MAX_DEPTH`]. `0` inspects nothing.
    pub max_depth: usize,
    /// Absolute ancestor directory; never walk above this ceiling.
    ///
    /// Explicit values must be absolute directories that contain the start
    /// path. Relative, file, and unrelated paths are rejected.
    ///
    /// `None` (default) uses the user home directory (`HOME` on Unix,
    /// `USERPROFILE` then `HOME` on Windows). If home is `/` or `/root`
    /// (or a volume root), or cannot be resolved, the current working
    /// directory is the ceiling. If home is not an ancestor of the start
    /// path, the walk stops with [`RepositoryNotFoundReason::BoundaryReached`]
    /// and does not search above the working directory.
    ///
    /// PathConstraint / RespectConstraints is not read from any application
    /// config; this field is the only caller-supplied ceiling.
    pub boundary: Option<PathBuf>,
    /// Follow directory symlinks and detect loops. Default: `false`.
    /// Parent (`..`) components in starts and boundaries require `true`.
    pub follow_symlinks: bool,
    /// Stop at the closest marker (`true`, default) or keep walking to the
    /// outermost marker within bounds (`false`).
    pub stop_at_first: bool,
}

impl Default for RepositoryRootOptions {
    fn default() -> Self {
        Self {
            markers: GIT_MARKERS.iter().map(|s| (*s).to_string()).collect(),
            max_depth: DEFAULT_MAX_DEPTH,
            boundary: None,
            follow_symlinks: false,
            stop_at_first: true,
        }
    }
}

impl RepositoryRootOptions {
    /// Git-marker discovery with Crucible defaults (max depth 10, home ceiling).
    pub fn git() -> Self {
        Self::default()
    }

    /// Discovery with an explicit marker list and otherwise-default options.
    pub fn with_markers<I, S>(markers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            markers: markers.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }
}

/// Why an upward search stopped without finding a marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RepositoryNotFoundReason {
    /// [`RepositoryRootOptions::max_depth`] directories were inspected.
    MaxDepthReached,
    /// Walk reached the configured (or default home) boundary.
    BoundaryReached,
    /// Walk reached the filesystem / volume root.
    FilesystemRootReached,
    /// `Path::parent` could not move further.
    TraversalTermination,
}

impl RepositoryNotFoundReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::MaxDepthReached => "max_depth_reached",
            Self::BoundaryReached => "boundary_reached",
            Self::FilesystemRootReached => "filesystem_root_reached",
            Self::TraversalTermination => "traversal_termination",
        }
    }
}

impl std::fmt::Display for RepositoryNotFoundReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Errors from [`discover_repository_root`].
///
/// [`Self::TraversalLoop`] is defensive: with `follow_symlinks` the walk
/// starts from a canonical path and steps lexical parents, so a loop is
/// not expected on a stable filesystem.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RepositoryRootError {
    /// Start path is empty, missing, or cannot be resolved.
    #[error("invalid start path {start_path}: {message}")]
    InvalidStartPath {
        /// Caller-supplied start path.
        start_path: PathBuf,
        /// Why the path was rejected.
        message: String,
    },
    /// Boundary path is missing or cannot be resolved.
    #[error("invalid boundary {boundary}: {message}")]
    InvalidBoundary {
        /// Caller-supplied or resolved boundary.
        boundary: PathBuf,
        /// Why the boundary was rejected.
        message: String,
    },
    /// Marker list was empty.
    #[error("markers list cannot be empty")]
    InvalidMarkers,
    /// A marker was not a single normal path component.
    #[error("invalid marker {marker}: {message}")]
    InvalidMarker {
        /// Rejected marker string.
        marker: String,
        /// Why the marker was rejected.
        message: String,
    },
    /// No marker found within depth / boundary / filesystem root.
    #[error(
        "no repository markers found within search boundaries (reason: {reason}, stopped at {stopped_at})"
    )]
    RepositoryNotFound {
        /// Caller-supplied start path.
        start_path: PathBuf,
        /// Markers that were searched.
        markers: Vec<String>,
        /// Directories inspected before stopping.
        searched_depth: usize,
        /// Directory where the walk stopped.
        stopped_at: PathBuf,
        /// Why the walk stopped.
        reason: RepositoryNotFoundReason,
    },
    /// A symlink cycle was detected while [`RepositoryRootOptions::follow_symlinks`]
    /// was enabled.
    #[error("symlink loop detected during repository root discovery at {current_path}")]
    TraversalLoop {
        /// Caller-supplied start path.
        start_path: PathBuf,
        /// Directory that repeated a previously seen real path.
        current_path: PathBuf,
        /// Directories inspected before the loop was detected.
        searched_depth: usize,
    },
}

/// Discover a repository root by walking upward from `start_path`.
///
/// Returns the directory that contains a marker, not the marker path itself.
/// Unlike [`crate::pathfinder::find_repository_root`], this API is bounded
/// (default max depth 10, default home ceiling) and reports structured errors.
///
/// # Examples
///
/// ```
/// use rsfulmen::pathfinder::{discover_repository_root, RepositoryRootOptions};
/// use std::path::Path;
///
/// let opts = RepositoryRootOptions::with_markers([".git", "Cargo.toml"]);
/// let _ = discover_repository_root(Path::new("."), &opts);
/// ```
pub fn discover_repository_root(
    start_path: &Path,
    options: &RepositoryRootOptions,
) -> Result<PathBuf, RepositoryRootError> {
    if start_path.as_os_str().is_empty() {
        return Err(RepositoryRootError::InvalidStartPath {
            start_path: start_path.to_path_buf(),
            message: "start path cannot be empty".to_string(),
        });
    }
    if options.markers.is_empty() {
        return Err(RepositoryRootError::InvalidMarkers);
    }
    for marker in &options.markers {
        validate_marker(marker)?;
    }

    // Resolve the supplied spelling before deriving a parent or testing containment.
    // In no-follow mode, parent components are ambiguous across symlink hops.
    if !options.follow_symlinks && has_parent_component(start_path) {
        return Err(RepositoryRootError::InvalidStartPath {
            start_path: start_path.to_path_buf(),
            message: "parent components require follow_symlinks".to_string(),
        });
    }
    let normalized_start = normalize_path_spelling(start_path);
    let lexical_start = if normalized_start.as_os_str().is_empty() {
        // A start consisting only of '.' components still denotes the cwd.
        absolute_path(start_path).map_err(|message| RepositoryRootError::InvalidStartPath {
            start_path: start_path.to_path_buf(),
            message,
        })?
    } else {
        normalized_start
    };
    let start_meta = fs::symlink_metadata(&lexical_start).map_err(|err| {
        let message = if err.kind() == std::io::ErrorKind::NotFound {
            "start path does not exist".to_string()
        } else {
            format!("failed to stat start path: {err}")
        };
        RepositoryRootError::InvalidStartPath {
            start_path: start_path.to_path_buf(),
            message,
        }
    })?;

    let abs_start = if options.follow_symlinks {
        canonicalize_for_compare(&lexical_start).map_err(|err| {
            RepositoryRootError::InvalidStartPath {
                start_path: start_path.to_path_buf(),
                message: format!("failed to canonicalize start path: {err}"),
            }
        })?
    } else {
        absolute_path(&lexical_start).map_err(|message| RepositoryRootError::InvalidStartPath {
            start_path: start_path.to_path_buf(),
            message,
        })?
    };

    let start_is_file = if start_meta.file_type().is_symlink() && options.follow_symlinks {
        fs::metadata(&abs_start)
            .map(|m| m.is_file())
            .unwrap_or(false)
    } else {
        start_meta.is_file()
    };

    let start_dir = if start_is_file {
        abs_start
            .parent()
            .unwrap_or(abs_start.as_path())
            .to_path_buf()
    } else {
        abs_start
    };

    let lexical_boundary = resolve_boundary(
        &start_dir,
        options.boundary.as_deref(),
        options.follow_symlinks,
    )?;

    let (mut current, walk_boundary) = if options.follow_symlinks {
        let current = canonicalize_for_compare(&start_dir).map_err(|err| {
            RepositoryRootError::InvalidStartPath {
                start_path: start_path.to_path_buf(),
                message: format!("failed to canonicalize start path: {err}"),
            }
        })?;
        let walk_boundary = canonicalize_for_compare(&lexical_boundary)
            .unwrap_or_else(|_| strip_verbatim(&lexical_boundary));
        (current, walk_boundary)
    } else {
        (start_dir, lexical_boundary)
    };

    let fs_root = filesystem_root(&current);
    let mut visited = HashSet::new();
    let mut outermost: Option<PathBuf> = None;
    let mut depth = 0usize;

    loop {
        if options.follow_symlinks && !visited.insert(current.clone()) {
            return Err(RepositoryRootError::TraversalLoop {
                start_path: start_path.to_path_buf(),
                current_path: current,
                searched_depth: depth,
            });
        }

        if depth >= options.max_depth {
            return not_found(
                start_path,
                options,
                depth,
                current,
                outermost,
                RepositoryNotFoundReason::MaxDepthReached,
            );
        }

        if !is_within_boundary(&current, &walk_boundary) {
            return not_found(
                start_path,
                options,
                depth,
                current,
                outermost,
                RepositoryNotFoundReason::BoundaryReached,
            );
        }

        // Match gofulmen: the filesystem / volume root is a stop, not a
        // candidate repository (a container `/.git` must not win).
        if paths_equal(&current, &fs_root)
            || canonicalize_for_compare(&current)
                .is_ok_and(|resolved| paths_equal(&resolved, &filesystem_root(&resolved)))
        {
            return not_found(
                start_path,
                options,
                depth,
                current,
                outermost,
                RepositoryNotFoundReason::FilesystemRootReached,
            );
        }

        let inspect = if options.follow_symlinks {
            true
        } else {
            !path_has_symlink_between(&walk_boundary, &current)
        };

        if inspect && has_marker(&current, &options.markers, options.follow_symlinks) {
            if options.stop_at_first {
                return Ok(current);
            }
            outermost = Some(current.clone());
        }

        match current.parent() {
            Some(parent) if parent != current => {
                current = parent.to_path_buf();
                depth += 1;
            }
            _ => {
                return not_found(
                    start_path,
                    options,
                    depth,
                    current,
                    outermost,
                    RepositoryNotFoundReason::TraversalTermination,
                );
            }
        }
    }
}

fn not_found(
    start_path: &Path,
    options: &RepositoryRootOptions,
    searched_depth: usize,
    stopped_at: PathBuf,
    outermost: Option<PathBuf>,
    reason: RepositoryNotFoundReason,
) -> Result<PathBuf, RepositoryRootError> {
    if let Some(found) = outermost {
        return Ok(found);
    }
    Err(RepositoryRootError::RepositoryNotFound {
        start_path: start_path.to_path_buf(),
        markers: options.markers.clone(),
        searched_depth,
        stopped_at,
        reason,
    })
}

fn validate_marker(marker: &str) -> Result<(), RepositoryRootError> {
    if marker.is_empty() {
        return Err(RepositoryRootError::InvalidMarker {
            marker: marker.to_string(),
            message: "marker cannot be empty".to_string(),
        });
    }
    if marker.contains('/') || marker.contains('\\') {
        return Err(RepositoryRootError::InvalidMarker {
            marker: marker.to_string(),
            message: "marker must be a single path component".to_string(),
        });
    }
    let path = Path::new(marker);
    if path.is_absolute() {
        return Err(RepositoryRootError::InvalidMarker {
            marker: marker.to_string(),
            message: "marker cannot be an absolute path".to_string(),
        });
    }
    let mut components = path.components();
    match components.next() {
        Some(std::path::Component::Normal(_)) if components.next().is_none() => Ok(()),
        _ => Err(RepositoryRootError::InvalidMarker {
            marker: marker.to_string(),
            message:
                "marker must be one non-empty normal path component (not '.', '..', or a prefix)"
                    .to_string(),
        }),
    }
}

fn has_marker(dir: &Path, markers: &[String], follow_symlinks: bool) -> bool {
    markers.iter().any(|marker| {
        let candidate = dir.join(marker);
        let meta = if follow_symlinks {
            fs::metadata(&candidate)
        } else {
            fs::symlink_metadata(&candidate)
        };
        meta.is_ok()
    })
}

/// True when a path component *after* `boundary` is a symlink.
///
/// Ancestors of the ceiling (macOS `/tmp` → `/private/tmp`, Fedora
/// `/home` → `/var/home`) are ignored so default discovery still works.
fn path_has_symlink_between(boundary: &Path, current: &Path) -> bool {
    let Ok(boundary) = canonicalize_for_compare(boundary) else {
        return true;
    };
    // Use the outermost resolved ceiling occurrence: an in-bound symlink back
    // to the ceiling must itself be screened, not become the screening anchor.
    // Aliases at or above that occurrence remain allowed.
    let Some(ceiling) = current
        .ancestors()
        .filter(|ancestor| {
            canonicalize_for_compare(ancestor).is_ok_and(|path| paths_equal(&path, &boundary))
        })
        .last()
    else {
        return true;
    };
    for descendant in current.ancestors().take_while(|path| *path != ceiling) {
        match fs::symlink_metadata(descendant) {
            Ok(meta) if !meta.file_type().is_symlink() => {}
            _ => return true,
        }
    }
    false
}

fn canonicalize_for_compare(path: &Path) -> std::io::Result<PathBuf> {
    fs::canonicalize(path).map(|p| strip_verbatim(&p))
}

fn strip_verbatim(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let raw = path.to_string_lossy();
        if let Some(rest) = raw.strip_prefix(r"\\?\") {
            if let Some(unc) = rest.strip_prefix(r"UNC\") {
                return PathBuf::from(format!(r"\\{unc}"));
            }
            return PathBuf::from(rest);
        }
        path.to_path_buf()
    }
    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

// Rebuild paths before metadata or containment checks: on Unix, a trailing
// separator makes symlink_metadata follow the final directory symlink. Preserve
// ParentDir components so the existing no-follow rejection / follow resolution
// remains authoritative; this is spelling normalization, not parent resolution.
fn normalize_path_spelling(path: &Path) -> PathBuf {
    path.components()
        .filter(|component| *component != std::path::Component::CurDir)
        .collect()
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    std::path::absolute(path)
        .map(|path| normalize_path_spelling(&path))
        .map_err(|err| format!("failed to resolve absolute path: {err}"))
}

fn has_parent_component(path: &Path) -> bool {
    path.components()
        .any(|c| c == std::path::Component::ParentDir)
}

#[cfg(test)]
fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn resolve_boundary(
    start: &Path,
    explicit: Option<&Path>,
    follow_symlinks: bool,
) -> Result<PathBuf, RepositoryRootError> {
    if let Some(boundary) = explicit {
        let normalized_boundary = normalize_path_spelling(boundary);
        let boundary = normalized_boundary.as_path();
        if !boundary.is_absolute() {
            return Err(RepositoryRootError::InvalidBoundary {
                boundary: boundary.to_path_buf(),
                message: "boundary must be an absolute directory".to_string(),
            });
        }
        if !follow_symlinks && has_parent_component(boundary) {
            return Err(RepositoryRootError::InvalidBoundary {
                boundary: boundary.to_path_buf(),
                message: "parent components require follow_symlinks".to_string(),
            });
        }
        let meta = fs::metadata(boundary).map_err(|err| {
            let message = if err.kind() == std::io::ErrorKind::NotFound {
                "boundary path does not exist".to_string()
            } else {
                format!("failed to stat boundary: {err}")
            };
            RepositoryRootError::InvalidBoundary {
                boundary: boundary.to_path_buf(),
                message,
            }
        })?;
        if !meta.is_dir() {
            return Err(RepositoryRootError::InvalidBoundary {
                boundary: boundary.to_path_buf(),
                message: "boundary must be a directory".to_string(),
            });
        }
        let resolved = canonicalize_for_compare(boundary).map_err(|err| {
            RepositoryRootError::InvalidBoundary {
                boundary: boundary.to_path_buf(),
                message: format!("failed to canonicalize boundary: {err}"),
            }
        })?;
        let abs = if follow_symlinks {
            resolved
        } else {
            boundary.to_path_buf()
        };
        if !is_within_boundary(start, &abs) {
            return Err(RepositoryRootError::InvalidBoundary {
                boundary: abs,
                message: "boundary is not an ancestor of the start path".to_string(),
            });
        }
        return Ok(abs);
    }

    Ok(default_boundary(start))
}

fn default_boundary(start: &Path) -> PathBuf {
    match user_home().map(|home| normalize_path_spelling(&home)) {
        Some(home) if !is_unbounded_home(&home) => home,
        _ => normalize_path_spelling(&env::current_dir().unwrap_or_else(|_| start.to_path_buf())),
    }
}

fn nonempty_os(value: Option<std::ffi::OsString>) -> Option<PathBuf> {
    value.filter(|v| !v.is_empty()).map(PathBuf::from)
}

fn select_user_home(
    home: Option<std::ffi::OsString>,
    userprofile: Option<std::ffi::OsString>,
) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        nonempty_os(userprofile).or_else(|| nonempty_os(home))
    }
    #[cfg(not(windows))]
    {
        let _ = userprofile;
        nonempty_os(home)
    }
}

fn user_home() -> Option<PathBuf> {
    select_user_home(env::var_os("HOME"), env::var_os("USERPROFILE")).or_else(env::home_dir)
}

fn is_unbounded_home(home: &Path) -> bool {
    let Ok(home) = canonicalize_for_compare(home) else {
        return true;
    };
    home == Path::new("/")
        || home == Path::new("/root")
        || paths_equal(&home, &filesystem_root(&home))
}

fn filesystem_root(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut comps = path.components();
    match comps.next() {
        Some(prefix @ Component::Prefix(_)) => {
            let mut root = PathBuf::new();
            root.push(prefix);
            if let Some(Component::RootDir) = comps.next() {
                root.push(Component::RootDir);
            }
            root
        }
        Some(Component::RootDir) => PathBuf::from(std::path::MAIN_SEPARATOR_STR),
        Some(other) => PathBuf::from(other.as_os_str()),
        None => PathBuf::from(std::path::MAIN_SEPARATOR_STR),
    }
}

fn is_within_boundary(path: &Path, boundary: &Path) -> bool {
    let path_n = strip_verbatim(path);
    let boundary_n = strip_verbatim(boundary);
    if path_starts_with(&path_n, &boundary_n) {
        return true;
    }
    // `absolute(".")` uses getcwd (physical). HOME / an explicit ceiling may
    // still be the symlink form. Containment uses one representation.
    match (
        canonicalize_for_compare(path),
        canonicalize_for_compare(boundary),
    ) {
        (Ok(path), Ok(boundary)) => path_starts_with(&path, &boundary),
        _ => false,
    }
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    path_starts_with(left, right) && path_starts_with(right, left)
}

fn path_starts_with(path: &Path, prefix: &Path) -> bool {
    let mut path_comps = path.components();
    for expected in prefix.components() {
        match path_comps.next() {
            Some(got) if components_match(got, expected) => {}
            _ => return false,
        }
    }
    true
}

fn components_match(left: std::path::Component<'_>, right: std::path::Component<'_>) -> bool {
    #[cfg(windows)]
    {
        left.as_os_str().eq_ignore_ascii_case(right.as_os_str())
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let mut path = env::temp_dir();
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos();
            let seq = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
            path.push(format!(
                "rsfulmen-repo-root-{}-{}-{}",
                std::process::id(),
                nanos,
                seq
            ));
            fs::create_dir_all(&path).expect("temp dir");
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

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent");
        }
        fs::write(path, content).expect("write");
    }

    fn opts(markers: &[&str], boundary: &Path, max_depth: usize) -> RepositoryRootOptions {
        RepositoryRootOptions {
            markers: markers.iter().map(|s| (*s).to_string()).collect(),
            max_depth,
            boundary: Some(boundary.to_path_buf()),
            follow_symlinks: false,
            stop_at_first: true,
        }
    }

    #[test]
    fn discovers_git_dir_n_levels_up() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let nested = tmp.path().join("a/b/c");
        fs::create_dir_all(&nested).unwrap();

        let root = discover_repository_root(&nested, &opts(&[".git"], tmp.path(), 10)).unwrap();
        assert_eq!(normalize_lexically(&root), normalize_lexically(tmp.path()));
    }

    #[test]
    fn closest_marker_wins_over_outer_git() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        write_file(&tmp.path().join("proj/go.mod"), "module example");
        let start = tmp.path().join("proj/src");
        fs::create_dir_all(&start).unwrap();

        let root =
            discover_repository_root(&start, &opts(&[".git", "go.mod"], tmp.path(), 10)).unwrap();
        assert_eq!(
            normalize_lexically(&root),
            normalize_lexically(&tmp.path().join("proj"))
        );
    }

    #[test]
    fn max_depth_stops_before_marker() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let nested = tmp.path().join("a/b/c");
        fs::create_dir_all(&nested).unwrap();

        let err = discover_repository_root(&nested, &opts(&[".git"], tmp.path(), 2)).unwrap_err();
        match err {
            RepositoryRootError::RepositoryNotFound {
                reason,
                searched_depth,
                ..
            } => {
                assert_eq!(reason, RepositoryNotFoundReason::MaxDepthReached);
                assert_eq!(searched_depth, 2);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn max_depth_zero_does_not_inspect_start() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();

        let err =
            discover_repository_root(tmp.path(), &opts(&[".git"], tmp.path(), 0)).unwrap_err();
        match err {
            RepositoryRootError::RepositoryNotFound {
                reason: RepositoryNotFoundReason::MaxDepthReached,
                searched_depth: 0,
                ..
            } => {}
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn explicit_boundary_blocks_marker_above() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let inner = tmp.path().join("inner");
        fs::create_dir_all(inner.join("src")).unwrap();

        let err =
            discover_repository_root(&inner.join("src"), &opts(&[".git"], &inner, 10)).unwrap_err();
        match err {
            RepositoryRootError::RepositoryNotFound { reason, .. } => {
                assert_eq!(reason, RepositoryNotFoundReason::BoundaryReached);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn start_outside_boundary_is_invalid() {
        let _lock = crate::test_env_lock();
        let boundary = TestDir::new();
        let start = TestDir::new();
        fs::create_dir_all(start.path().join("src")).unwrap();

        let err = discover_repository_root(
            &start.path().join("src"),
            &opts(&[".git"], boundary.path(), 10),
        )
        .unwrap_err();
        match err {
            RepositoryRootError::InvalidBoundary { message, .. } => {
                assert!(message.contains("ancestor"), "{message}");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn missing_start_is_invalid() {
        let _lock = crate::test_env_lock();
        let err = discover_repository_root(
            Path::new("/no/such/rsfulmen-repo-root-start"),
            &RepositoryRootOptions::default(),
        )
        .unwrap_err();
        assert!(matches!(err, RepositoryRootError::InvalidStartPath { .. }));
    }

    #[test]
    fn empty_start_is_invalid() {
        let _lock = crate::test_env_lock();
        let err =
            discover_repository_root(Path::new(""), &RepositoryRootOptions::default()).unwrap_err();
        assert!(matches!(err, RepositoryRootError::InvalidStartPath { .. }));
    }

    #[test]
    fn empty_markers_are_invalid() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        let mut options = opts(&[], tmp.path(), 10);
        options.markers.clear();
        let err = discover_repository_root(tmp.path(), &options).unwrap_err();
        assert!(matches!(err, RepositoryRootError::InvalidMarkers));
    }

    #[test]
    fn missing_boundary_is_invalid() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        let options = RepositoryRootOptions {
            boundary: Some(tmp.path().join("missing-boundary")),
            ..opts(&[".git"], tmp.path(), 10)
        };
        let err = discover_repository_root(tmp.path(), &options).unwrap_err();
        assert!(matches!(err, RepositoryRootError::InvalidBoundary { .. }));
    }

    #[test]
    fn file_start_uses_parent() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let file = tmp.path().join("src/main.rs");
        write_file(&file, "fn main() {}");

        let root = discover_repository_root(&file, &opts(&[".git"], tmp.path(), 10)).unwrap();
        assert_eq!(normalize_lexically(&root), normalize_lexically(tmp.path()));
    }

    #[test]
    fn git_file_marker_counts() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        write_file(&tmp.path().join(".git"), "gitdir: /elsewhere");
        let nested = tmp.path().join("a/b");
        fs::create_dir_all(&nested).unwrap();

        let root = discover_repository_root(&nested, &opts(&[".git"], tmp.path(), 10)).unwrap();
        assert_eq!(normalize_lexically(&root), normalize_lexically(tmp.path()));
    }

    #[test]
    fn stop_at_first_false_returns_outermost_marker() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        write_file(&tmp.path().join("proj/go.mod"), "module example");
        let start = tmp.path().join("proj/src");
        fs::create_dir_all(&start).unwrap();

        let mut options = opts(&[".git", "go.mod"], tmp.path(), 10);
        options.stop_at_first = false;
        let root = discover_repository_root(&start, &options).unwrap();
        assert_eq!(
            normalize_lexically(&root),
            normalize_lexically(tmp.path()),
            "outermost marker is the root .git"
        );
    }

    #[test]
    fn node_and_python_markers() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        write_file(&tmp.path().join("package.json"), "{}");
        write_file(
            &tmp.path().join("py/pyproject.toml"),
            "[project]\nname='x'\n",
        );
        fs::create_dir_all(tmp.path().join("py/src")).unwrap();

        let node = discover_repository_root(
            &tmp.path().join("py/src"),
            &opts(&["package.json"], tmp.path(), 10),
        )
        .unwrap();
        assert_eq!(normalize_lexically(&node), normalize_lexically(tmp.path()));

        let py = discover_repository_root(
            &tmp.path().join("py/src"),
            &opts(&["pyproject.toml", "setup.py"], tmp.path(), 10),
        )
        .unwrap();
        assert_eq!(
            normalize_lexically(&py),
            normalize_lexically(&tmp.path().join("py"))
        );
    }

    #[test]
    fn unbounded_home_sentinel() {
        let _lock = crate::test_env_lock();
        assert!(is_unbounded_home(Path::new("/")));
        assert!(is_unbounded_home(Path::new("/root")));
        let tmp = TestDir::new();
        assert!(!is_unbounded_home(tmp.path()));
        assert!(is_unbounded_home(&tmp.path().join("missing")));
    }

    #[test]
    fn rejects_empty_dot_dotdot_absolute_and_multicomponent_markers() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let cases = ["", ".", "..", "/.git", "foo/bar", "foo\\bar"];
        for marker in cases {
            let err =
                discover_repository_root(tmp.path(), &opts(&[marker], tmp.path(), 4)).unwrap_err();
            assert!(
                matches!(err, RepositoryRootError::InvalidMarker { .. }),
                "marker {marker:?} should be rejected, got {err:?}"
            );
        }
    }

    #[test]
    fn relative_and_file_boundaries_are_invalid() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let file = tmp.path().join("not-a-dir");
        write_file(&file, "x");

        let relative = RepositoryRootOptions {
            boundary: Some(PathBuf::from("relative-ceiling")),
            ..opts(&[".git"], tmp.path(), 4)
        };
        assert!(matches!(
            discover_repository_root(tmp.path(), &relative).unwrap_err(),
            RepositoryRootError::InvalidBoundary { .. }
        ));

        let as_file = RepositoryRootOptions {
            boundary: Some(file),
            ..opts(&[".git"], tmp.path(), 4)
        };
        assert!(matches!(
            discover_repository_root(tmp.path(), &as_file).unwrap_err(),
            RepositoryRootError::InvalidBoundary { .. }
        ));
    }

    struct ProcessState {
        home: Option<std::ffi::OsString>,
        profile: Option<std::ffi::OsString>,
        cwd: PathBuf,
    }

    impl Drop for ProcessState {
        fn drop(&mut self) {
            env::set_current_dir(&self.cwd).expect("restore cwd before temp cleanup");
            for (key, value) in [("HOME", &self.home), ("USERPROFILE", &self.profile)] {
                match value {
                    Some(value) => env::set_var(key, value),
                    None => env::remove_var(key),
                }
            }
        }
    }

    // Each test holds the crate-wide lock through restoration and temp cleanup.
    fn with_home<T>(home: &Path, f: impl FnOnce() -> T) -> T {
        let _state = ProcessState {
            home: env::var_os("HOME"),
            profile: env::var_os("USERPROFILE"),
            cwd: env::current_dir().unwrap(),
        };
        env::set_var("HOME", home);
        env::set_var("USERPROFILE", home);
        f()
    }

    fn default_opts() -> RepositoryRootOptions {
        RepositoryRootOptions {
            markers: vec![".git".into()],
            max_depth: 10,
            boundary: None,
            follow_symlinks: false,
            stop_at_first: true,
        }
    }

    #[test]
    fn default_home_outside_start_ancestry_is_boundary_reached() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let start = tmp.path().join("src");
        fs::create_dir_all(&start).unwrap();
        let foreign = TestDir::new();

        let err = with_home(foreign.path(), || {
            discover_repository_root(&start, &default_opts())
        })
        .unwrap_err();
        match err {
            RepositoryRootError::RepositoryNotFound { reason, .. } => {
                assert_eq!(reason, RepositoryNotFoundReason::BoundaryReached);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn unbounded_home_falls_back_to_cwd_not_filesystem_root() {
        let _lock = crate::test_env_lock();
        let outer = TestDir::new();
        fs::create_dir_all(outer.path().join(".git")).unwrap();
        let work = outer.path().join("work/deep");
        fs::create_dir_all(&work).unwrap();
        let err = with_home(Path::new("/root"), || {
            env::set_current_dir(&work).unwrap();
            let result = discover_repository_root(Path::new("."), &default_opts());
            result
        })
        .unwrap_err();
        match err {
            RepositoryRootError::RepositoryNotFound { reason, .. } => {
                assert_eq!(reason, RepositoryNotFoundReason::BoundaryReached);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn select_user_home_prefers_userprofile_on_windows_only() {
        let _lock = crate::test_env_lock();
        use std::ffi::OsString;
        let home = Some(OsString::from("/from-home"));
        let profile = Some(OsString::from("/from-userprofile"));
        let selected = select_user_home(home, profile).unwrap();
        #[cfg(windows)]
        assert_eq!(selected, PathBuf::from("/from-userprofile"));
        #[cfg(not(windows))]
        assert_eq!(selected, PathBuf::from("/from-home"));
    }

    #[test]
    fn filesystem_root_posix_and_windows_shapes() {
        let _lock = crate::test_env_lock();
        assert_eq!(filesystem_root(Path::new("/")), PathBuf::from("/"));
        assert_eq!(filesystem_root(Path::new("/foo/bar")), PathBuf::from("/"));
    }

    #[cfg(windows)]
    #[test]
    fn filesystem_root_windows_drive_and_unc() {
        let _lock = crate::test_env_lock();
        assert_eq!(filesystem_root(Path::new(r"C:\")), PathBuf::from(r"C:\"));
        assert_eq!(
            filesystem_root(Path::new(r"C:\foo\bar")),
            PathBuf::from(r"C:\")
        );
        let unc = filesystem_root(Path::new(r"\\server\share\foo"));
        assert_eq!(unc, PathBuf::from(r"\\server\share\"));
    }

    #[test]
    fn path_starts_with_does_not_match_sibling_prefix() {
        let _lock = crate::test_env_lock();
        assert!(path_starts_with(
            Path::new("/users/x/proj"),
            Path::new("/users/x")
        ));
        assert!(!path_starts_with(
            Path::new("/users2/x"),
            Path::new("/users/x")
        ));
    }

    #[cfg(windows)]
    #[test]
    fn windows_boundary_compare_is_case_insensitive() {
        let _lock = crate::test_env_lock();
        assert!(path_starts_with(
            Path::new(r"C:\Users\x\proj"),
            Path::new(r"c:\users\x")
        ));
        assert!(!path_starts_with(
            Path::new(r"C:\Users2\x"),
            Path::new(r"C:\Users\x")
        ));
    }

    #[cfg(unix)]
    #[test]
    fn posix_root_is_a_stop_and_is_not_inspected() {
        let _lock = crate::test_env_lock();
        // `/tmp` exists on every Unix runner. If the root-stop ran *after*
        // has_marker, this would return Ok("/") instead of FilesystemRootReached.
        let options = RepositoryRootOptions {
            markers: vec!["tmp".into()],
            max_depth: 1,
            boundary: Some(PathBuf::from("/")),
            follow_symlinks: false,
            stop_at_first: true,
        };
        match discover_repository_root(Path::new("/"), &options) {
            Err(RepositoryRootError::RepositoryNotFound { reason, .. }) => {
                assert_eq!(reason, RepositoryNotFoundReason::FilesystemRootReached);
            }
            Ok(path) => panic!("filesystem root must not be a candidate, got {path:?}"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn path_constraint_is_deferred_explicit_boundary_is_the_only_caller_ceiling() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let defaults = RepositoryRootOptions::with_markers([".git"]);
        assert!(
            defaults.boundary.is_none(),
            "no implicit PathConstraint field; callers pass boundary"
        );
        let root = discover_repository_root(tmp.path(), &opts(&[".git"], tmp.path(), 4)).unwrap();
        assert_eq!(normalize_lexically(&root), normalize_lexically(tmp.path()));
    }

    #[cfg(unix)]
    #[test]
    fn follow_symlinks_ancestor_alias_is_not_a_loop() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        let repo = tmp.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        std::os::unix::fs::symlink(&repo, repo.join("loop")).unwrap();

        let options = RepositoryRootOptions {
            follow_symlinks: true,
            ..opts(&[".git"], tmp.path(), 8)
        };
        let root = discover_repository_root(&repo.join("loop"), &options)
            .expect("canonical start is the repo, not a cycle");
        assert_eq!(
            fs::canonicalize(&root).unwrap(),
            fs::canonicalize(&repo).unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_ancestor_above_boundary_still_finds_markers() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        let real = tmp.path().join("real");
        fs::create_dir_all(real.join("repo/.git")).unwrap();
        let alias = tmp.path().join("alias");
        std::os::unix::fs::symlink(&real, &alias).unwrap();
        let start = alias.join("repo/sub");
        fs::create_dir_all(&start).unwrap();

        let root = discover_repository_root(&start, &opts(&[".git"], &alias, 8))
            .expect("symlink above/at the ceiling must not hide in-tree markers");
        assert_eq!(
            normalize_lexically(&root),
            normalize_lexically(&alias.join("repo"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn relative_start_under_symlinked_home_finds_repo() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        let real = tmp.path().join("real");
        fs::create_dir_all(real.join("repo/.git")).unwrap();
        fs::create_dir_all(real.join("repo/sub")).unwrap();
        let alias = tmp.path().join("alias");
        std::os::unix::fs::symlink(&real, &alias).unwrap();
        let work = alias.join("repo/sub");
        let root = with_home(&alias, || {
            env::set_current_dir(&work).unwrap();
            let result = discover_repository_root(Path::new("."), &default_opts());
            result
        })
        .expect("getcwd-resolved '.' must stay under a symlinked HOME");
        assert_eq!(
            fs::canonicalize(&root).unwrap(),
            fs::canonicalize(real.join("repo")).unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn follow_symlinks_accepts_mismatched_symlink_start_and_boundary() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        let real = tmp.path().join("real");
        fs::create_dir_all(real.join(".git")).unwrap();
        fs::create_dir_all(real.join("nested")).unwrap();
        let alias = tmp.path().join("alias");
        std::os::unix::fs::symlink(&real, &alias).unwrap();

        for (start, boundary) in [
            (alias.join("nested"), real.clone()),
            (real.join("nested"), alias.clone()),
        ] {
            let options = RepositoryRootOptions {
                follow_symlinks: true,
                ..opts(&[".git"], &boundary, 8)
            };
            let root = discover_repository_root(&start, &options).unwrap_or_else(|err| {
                panic!("follow_symlinks start={start:?} boundary={boundary:?}: {err}")
            });
            assert_eq!(
                fs::canonicalize(&root).unwrap(),
                fs::canonicalize(&real).unwrap(),
                "start={start:?} boundary={boundary:?}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn follow_symlinks_accepts_symlinked_explicit_boundary() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        let real = tmp.path().join("real");
        fs::create_dir_all(real.join(".git")).unwrap();
        fs::create_dir_all(real.join("nested")).unwrap();
        let alias = tmp.path().join("alias");
        std::os::unix::fs::symlink(&real, &alias).unwrap();

        let options = RepositoryRootOptions {
            follow_symlinks: true,
            ..opts(&[".git"], &alias, 8)
        };
        let root = discover_repository_root(&alias.join("nested"), &options)
            .expect("canonical start must stay inside the canonical boundary");
        assert_eq!(
            fs::canonicalize(&root).unwrap(),
            fs::canonicalize(&real).unwrap()
        );
    }

    #[test]
    fn max_depth_equal_to_distance_finds_marker() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let nested = tmp.path().join("a/b/c");
        fs::create_dir_all(&nested).unwrap();
        let root = discover_repository_root(&nested, &opts(&[".git"], tmp.path(), 4)).unwrap();
        assert_eq!(normalize_lexically(&root), normalize_lexically(tmp.path()));
    }

    #[cfg(unix)]
    #[test]
    fn in_boundary_symlink_to_external_repo_is_rejected_in_both_modes() {
        let _lock = crate::test_env_lock();
        let outside = TestDir::new();
        fs::create_dir_all(outside.path().join(".git")).unwrap();
        let inside = TestDir::new();
        fs::create_dir_all(inside.path().join("work")).unwrap();
        std::os::unix::fs::symlink(outside.path(), inside.path().join("escape")).unwrap();

        for follow in [false, true] {
            let options = RepositoryRootOptions {
                follow_symlinks: follow,
                ..opts(&[".git"], inside.path(), 8)
            };
            let err =
                discover_repository_root(&inside.path().join("escape"), &options).unwrap_err();
            if follow {
                assert!(matches!(err, RepositoryRootError::InvalidBoundary { .. }));
            } else {
                assert!(matches!(
                    err,
                    RepositoryRootError::RepositoryNotFound {
                        reason: RepositoryNotFoundReason::BoundaryReached,
                        ..
                    }
                ));
            }
        }
    }
    #[cfg(unix)]
    fn check_alias_back_to_ceiling(home_ceiling: bool, ceiling_marker: bool) {
        let tmp = TestDir::new();
        let ceiling = tmp.path().join("ceiling");
        fs::create_dir_all(ceiling.join("repo/.git")).unwrap();
        if ceiling_marker {
            fs::create_dir(ceiling.join(".git")).unwrap();
        }
        std::os::unix::fs::symlink(&ceiling, ceiling.join("back")).unwrap();

        // Cover the hop itself, a marked descendant, and repeated back aliases.
        for suffix in ["back", "back/repo", "back/back", "back/back/repo"] {
            let start = ceiling.join(suffix);
            let result = if home_ceiling {
                with_home(&ceiling, || {
                    discover_repository_root(&start, &default_opts())
                })
            } else {
                discover_repository_root(&start, &opts(&[".git"], &ceiling, 10))
            };
            if ceiling_marker {
                assert_eq!(
                    result.unwrap(),
                    ceiling,
                    "only the actual ceiling may supply a marker: start={start:?}"
                );
            } else {
                assert!(
                    matches!(result, Err(RepositoryRootError::RepositoryNotFound { .. })),
                    "markers behind the back alias must be hidden: start={start:?}, {result:?}"
                );
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn regression_m1_back_to_explicit_ceiling_without_marker() {
        let _lock = crate::test_env_lock();
        check_alias_back_to_ceiling(false, false);
    }

    #[cfg(unix)]
    #[test]
    fn regression_m1_back_to_explicit_ceiling_with_marker() {
        let _lock = crate::test_env_lock();
        check_alias_back_to_ceiling(false, true);
    }

    #[cfg(unix)]
    #[test]
    fn regression_m1_back_to_home_ceiling_without_marker() {
        let _lock = crate::test_env_lock();
        check_alias_back_to_ceiling(true, false);
    }

    #[cfg(unix)]
    #[test]
    fn regression_m1_back_to_home_ceiling_with_marker() {
        let _lock = crate::test_env_lock();
        check_alias_back_to_ceiling(true, true);
    }

    #[cfg(unix)]
    fn check_trailing_separator_symlinks(home_ceiling: bool, target: &str) {
        let tmp = TestDir::new();
        let ceiling = tmp.path().join("ceiling");
        fs::create_dir_all(ceiling.join("nested")).unwrap();
        let marked_target = match target {
            "external" => tmp.path().join("outside"),
            "ceiling" => ceiling.clone(),
            "internal" => ceiling.join("repo"),
            _ => unreachable!(),
        };
        fs::create_dir_all(marked_target.join(".git")).unwrap();
        for placement in ["link", "nested/link"] {
            std::os::unix::fs::symlink(&marked_target, ceiling.join(placement)).unwrap();
        }

        let mut failures = Vec::new();
        for placement in ["link", "nested/link"] {
            for suffix in ["/", "//", "/./"] {
                // Append raw spelling: PathBuf::join must not normalize the suffix.
                let mut raw_start = ceiling.join(placement).into_os_string();
                raw_start.push(suffix);
                let start = PathBuf::from(raw_start);
                for boundary_suffix in ["", "/", "//", "/./"] {
                    let mut raw_boundary = ceiling.clone().into_os_string();
                    raw_boundary.push(boundary_suffix);
                    let boundary = PathBuf::from(raw_boundary);
                    let result = if home_ceiling {
                        with_home(&boundary, || {
                            discover_repository_root(&start, &default_opts())
                        })
                    } else {
                        discover_repository_root(&start, &opts(&[".git"], &boundary, 10))
                    };
                    let valid = if target == "ceiling" {
                        // A marked ceiling remains discoverable at its actual spelling,
                        // after both direct and nested back-links have been skipped.
                        matches!(&result, Ok(root) if root.as_os_str() == ceiling.as_os_str())
                    } else {
                        matches!(
                            &result,
                            Err(RepositoryRootError::RepositoryNotFound {
                                reason: RepositoryNotFoundReason::BoundaryReached,
                                ..
                            })
                        )
                    };
                    if !valid {
                        failures.push(format!(
                            "start={start:?}, boundary={boundary:?}, result={result:?}"
                        ));
                    }
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[cfg(unix)]
    #[test]
    fn regression_m1_trailing_separator_external_explicit_ceiling() {
        let _lock = crate::test_env_lock();
        check_trailing_separator_symlinks(false, "external");
    }

    #[cfg(unix)]
    #[test]
    fn regression_m1_trailing_separator_external_home_ceiling() {
        let _lock = crate::test_env_lock();
        check_trailing_separator_symlinks(true, "external");
    }

    #[cfg(unix)]
    #[test]
    fn regression_m1_trailing_separator_back_to_explicit_ceiling() {
        let _lock = crate::test_env_lock();
        check_trailing_separator_symlinks(false, "ceiling");
    }

    #[cfg(unix)]
    #[test]
    fn regression_m1_trailing_separator_back_to_home_ceiling() {
        let _lock = crate::test_env_lock();
        check_trailing_separator_symlinks(true, "ceiling");
    }

    #[cfg(unix)]
    #[test]
    fn regression_m1_trailing_separator_internal_explicit_ceiling() {
        let _lock = crate::test_env_lock();
        check_trailing_separator_symlinks(false, "internal");
    }

    #[cfg(unix)]
    #[test]
    fn regression_m1_trailing_separator_internal_home_ceiling() {
        let _lock = crate::test_env_lock();
        check_trailing_separator_symlinks(true, "internal");
    }

    #[cfg(unix)]
    fn check_boundary_alias(mode: u8) {
        let tmp = TestDir::new();
        let real = tmp.path().join("real");
        let alias = tmp.path().join("alias");
        fs::create_dir_all(real.join("target/.git")).unwrap();
        std::os::unix::fs::symlink(&real, &alias).unwrap();
        std::os::unix::fs::symlink(real.join("target"), real.join("link")).unwrap();
        let result = match mode {
            0 => discover_repository_root(&real.join("link"), &opts(&[".git"], &alias, 8)),
            1 => discover_repository_root(&alias.join("link"), &opts(&[".git"], &real, 8)),
            _ => with_home(&alias, || {
                discover_repository_root(&real.join("link"), &default_opts())
            }),
        };
        assert!(
            matches!(result, Err(RepositoryRootError::RepositoryNotFound { .. })),
            "mode={mode}: {result:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn regression_h1_aliased_boundary_physical_start() {
        let _lock = crate::test_env_lock();
        check_boundary_alias(0);
    }

    #[cfg(unix)]
    #[test]
    fn regression_h1_physical_boundary_aliased_start() {
        let _lock = crate::test_env_lock();
        check_boundary_alias(1);
    }

    #[cfg(unix)]
    #[test]
    fn regression_h1_aliased_home_physical_start() {
        let _lock = crate::test_env_lock();
        check_boundary_alias(2);
    }

    #[cfg(unix)]
    #[test]
    fn regression_h1_side_entrance_is_not_a_repository() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        let real = tmp.path().join("real");
        fs::create_dir_all(real.join("repo/.git")).unwrap();
        let entrance = tmp.path().join("entrance");
        std::os::unix::fs::symlink(real.join("repo"), &entrance).unwrap();
        assert!(matches!(
            discover_repository_root(&entrance, &opts(&[".git"], &real, 8)),
            Err(RepositoryRootError::RepositoryNotFound { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn regression_m2_symlink_before_parent_start() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        let real = tmp.path().join("real");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(real.join(".git")).unwrap();
        fs::create_dir_all(outside.join("sub")).unwrap();
        std::os::unix::fs::symlink(outside.join("sub"), real.join("escape")).unwrap();
        for follow in [true, false] {
            let mut options = opts(&[".git"], &real, 8);
            options.follow_symlinks = follow;
            let result = discover_repository_root(&real.join("escape/.."), &options);
            if follow {
                assert!(matches!(
                    result,
                    Err(RepositoryRootError::InvalidBoundary { .. })
                ));
            } else {
                assert!(matches!(
                    result,
                    Err(RepositoryRootError::InvalidStartPath { .. })
                ));
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn regression_m2_symlink_before_parent_boundary() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        let real = tmp.path().join("real");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(real.join(".git")).unwrap();
        fs::create_dir_all(outside.join("sub")).unwrap();
        std::os::unix::fs::symlink(outside.join("sub"), real.join("escape")).unwrap();
        for follow in [true, false] {
            let mut options = opts(&[".git"], &real.join("escape/.."), 8);
            options.follow_symlinks = follow;
            assert!(matches!(
                discover_repository_root(&real, &options),
                Err(RepositoryRootError::InvalidBoundary { .. })
            ));
        }
    }

    #[test]
    fn regression_m2_missing_component_start() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        for follow in [true, false] {
            let mut options = opts(&[".git"], tmp.path(), 8);
            options.follow_symlinks = follow;
            assert!(matches!(
                discover_repository_root(&tmp.path().join("missing/.."), &options),
                Err(RepositoryRootError::InvalidStartPath { .. })
            ));
        }
    }

    #[test]
    fn regression_m2_missing_component_boundary() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        for follow in [true, false] {
            let mut options = opts(&[".git"], &tmp.path().join("missing/.."), 8);
            options.follow_symlinks = follow;
            assert!(matches!(
                discover_repository_root(tmp.path(), &options),
                Err(RepositoryRootError::InvalidBoundary { .. })
            ));
        }
    }

    #[test]
    fn regression_m2_non_directory_component() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        write_file(&tmp.path().join("file"), "x");
        for follow in [true, false] {
            let mut options = opts(&[".git"], tmp.path(), 8);
            options.follow_symlinks = follow;
            assert!(matches!(
                discover_repository_root(&tmp.path().join("file/.."), &options),
                Err(RepositoryRootError::InvalidStartPath { .. })
            ));
            options.boundary = Some(tmp.path().join("file/.."));
            assert!(matches!(
                discover_repository_root(tmp.path(), &options),
                Err(RepositoryRootError::InvalidBoundary { .. })
            ));
        }
    }

    #[cfg(unix)]
    #[test]
    fn regression_m1_home_resolving_to_root_uses_cwd() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let work = tmp.path().join("work/deep");
        fs::create_dir_all(&work).unwrap();
        let home = tmp.path().join("home");
        std::os::unix::fs::symlink("/", &home).unwrap();
        with_home(&home, || {
            env::set_current_dir(&work).unwrap();
            for follow in [false, true] {
                let mut options = default_opts();
                options.follow_symlinks = follow;
                assert!(matches!(
                    discover_repository_root(Path::new("."), &options),
                    Err(RepositoryRootError::RepositoryNotFound {
                        reason: RepositoryNotFoundReason::BoundaryReached,
                        ..
                    })
                ));
            }
        });
    }

    #[cfg(unix)]
    #[test]
    fn regression_m1_explicit_root_alias_is_not_inspected() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        let alias = tmp.path().join("root");
        std::os::unix::fs::symlink("/", &alias).unwrap();
        // Uses existing /tmp as the marker; never creates anything at /.
        for follow in [false, true] {
            let mut options = opts(&["tmp"], &alias, 8);
            options.follow_symlinks = follow;
            assert!(matches!(
                discover_repository_root(&alias, &options),
                Err(RepositoryRootError::RepositoryNotFound {
                    reason: RepositoryNotFoundReason::FilesystemRootReached,
                    ..
                })
            ));
        }
    }

    #[test]
    fn process_state_is_restored_after_panic() {
        let _lock = crate::test_env_lock();
        let tmp = TestDir::new();
        let home = env::var_os("HOME");
        let profile = env::var_os("USERPROFILE");
        let cwd = env::current_dir().unwrap();
        let result = std::panic::catch_unwind(|| {
            with_home(tmp.path(), || {
                env::set_current_dir(tmp.path()).unwrap();
                panic!("exercise process-state restoration");
            });
        });
        assert!(result.is_err());
        assert_eq!(env::var_os("HOME"), home);
        assert_eq!(env::var_os("USERPROFILE"), profile);
        assert_eq!(env::current_dir().unwrap(), cwd);
    }
}
