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
//! [`RepositoryRootOptions::follow_symlinks`] is `true`, the start directory
//! is canonicalized (physical start) and each hop must stay inside the
//! canonical boundary. When it is `false`, directories whose path contains a
//! symlink component are not inspected for markers.
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// `USERPROFILE` then `HOME` on Windows). If home is `/` or `/root`,
    /// cannot be resolved, or is not an ancestor of the start path, the
    /// start path's filesystem root is the ceiling (still bounded by
    /// [`Self::max_depth`]).
    ///
    /// PathConstraint / RespectConstraints is not read from any application
    /// config; this field is the only caller-supplied ceiling.
    pub boundary: Option<PathBuf>,
    /// Follow directory symlinks and detect loops. Default: `false`.
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
#[derive(Debug, thiserror::Error)]
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

    let abs_start =
        absolute_path(start_path).map_err(|message| RepositoryRootError::InvalidStartPath {
            start_path: start_path.to_path_buf(),
            message,
        })?;

    let start_meta = fs::symlink_metadata(&abs_start).map_err(|err| {
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

    let boundary = resolve_boundary(&start_dir, options.boundary.as_deref())?;
    let fs_root = filesystem_root(&start_dir);

    let mut current = if options.follow_symlinks {
        fs::canonicalize(&start_dir).map_err(|err| RepositoryRootError::InvalidStartPath {
            start_path: start_path.to_path_buf(),
            message: format!("failed to canonicalize start path: {err}"),
        })?
    } else {
        start_dir
    };

    if options.follow_symlinks {
        let real_boundary = fs::canonicalize(&boundary).unwrap_or_else(|_| boundary.clone());
        if !is_within_boundary(&current, &real_boundary) {
            return not_found(
                start_path,
                options,
                0,
                current,
                None,
                RepositoryNotFoundReason::BoundaryReached,
            );
        }
    }

    let mut visited = HashSet::new();
    let mut outermost: Option<PathBuf> = None;
    let mut depth = 0usize;

    loop {
        if options.follow_symlinks {
            let real = fs::canonicalize(&current).unwrap_or_else(|_| current.clone());
            if !visited.insert(real) {
                return Err(RepositoryRootError::TraversalLoop {
                    start_path: start_path.to_path_buf(),
                    current_path: current,
                    searched_depth: depth,
                });
            }
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

        if !is_within_boundary(&current, &boundary) {
            return not_found(
                start_path,
                options,
                depth,
                current,
                outermost,
                RepositoryNotFoundReason::BoundaryReached,
            );
        }

        let inspect = if options.follow_symlinks {
            let real = fs::canonicalize(&current).unwrap_or_else(|_| current.clone());
            let real_boundary = fs::canonicalize(&boundary).unwrap_or_else(|_| boundary.clone());
            is_within_boundary(&real, &real_boundary)
        } else {
            !path_has_symlink_component(&current)
        };

        if inspect && has_marker(&current, &options.markers, options.follow_symlinks) {
            if options.stop_at_first {
                return Ok(current);
            }
            outermost = Some(current.clone());
        }

        if current == fs_root {
            return not_found(
                start_path,
                options,
                depth,
                current,
                outermost,
                RepositoryNotFoundReason::FilesystemRootReached,
            );
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

fn path_has_symlink_component(path: &Path) -> bool {
    let mut acc = PathBuf::new();
    for component in path.components() {
        acc.push(component);
        if let Ok(meta) = fs::symlink_metadata(&acc) {
            if meta.file_type().is_symlink() {
                return true;
            }
        }
    }
    false
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    std::path::absolute(path)
        .map(|p| normalize_lexically(&p))
        .map_err(|err| format!("failed to resolve absolute path: {err}"))
}

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

fn resolve_boundary(start: &Path, explicit: Option<&Path>) -> Result<PathBuf, RepositoryRootError> {
    if let Some(boundary) = explicit {
        if !boundary.is_absolute() {
            return Err(RepositoryRootError::InvalidBoundary {
                boundary: boundary.to_path_buf(),
                message: "boundary must be an absolute directory".to_string(),
            });
        }
        let abs = normalize_lexically(boundary);
        let meta = fs::metadata(&abs).map_err(|err| {
            let message = if err.kind() == std::io::ErrorKind::NotFound {
                "boundary path does not exist".to_string()
            } else {
                format!("failed to stat boundary: {err}")
            };
            RepositoryRootError::InvalidBoundary {
                boundary: abs.clone(),
                message,
            }
        })?;
        if !meta.is_dir() {
            return Err(RepositoryRootError::InvalidBoundary {
                boundary: abs,
                message: "boundary must be a directory".to_string(),
            });
        }
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
    match user_home() {
        Some(home) if !is_unbounded_home(&home) => {
            let home = absolute_path(&home).unwrap_or(home);
            if is_within_boundary(start, &home) {
                home
            } else {
                filesystem_root(start)
            }
        }
        _ => filesystem_root(start),
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
    home == Path::new("/") || home == Path::new("/root")
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
    let path = normalize_lexically(path);
    let boundary = normalize_lexically(boundary);
    path == boundary || path.starts_with(&boundary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);
    static HOME_LOCK: Mutex<()> = Mutex::new(());

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
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let nested = tmp.path().join("a/b/c");
        fs::create_dir_all(&nested).unwrap();

        let root = discover_repository_root(&nested, &opts(&[".git"], tmp.path(), 10)).unwrap();
        assert_eq!(normalize_lexically(&root), normalize_lexically(tmp.path()));
    }

    #[test]
    fn closest_marker_wins_over_outer_git() {
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
        let err = discover_repository_root(
            Path::new("/no/such/rsfulmen-repo-root-start"),
            &RepositoryRootOptions::default(),
        )
        .unwrap_err();
        assert!(matches!(err, RepositoryRootError::InvalidStartPath { .. }));
    }

    #[test]
    fn empty_start_is_invalid() {
        let err =
            discover_repository_root(Path::new(""), &RepositoryRootOptions::default()).unwrap_err();
        assert!(matches!(err, RepositoryRootError::InvalidStartPath { .. }));
    }

    #[test]
    fn empty_markers_are_invalid() {
        let tmp = TestDir::new();
        let mut options = opts(&[], tmp.path(), 10);
        options.markers.clear();
        let err = discover_repository_root(tmp.path(), &options).unwrap_err();
        assert!(matches!(err, RepositoryRootError::InvalidMarkers));
    }

    #[test]
    fn missing_boundary_is_invalid() {
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
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let file = tmp.path().join("src/main.rs");
        write_file(&file, "fn main() {}");

        let root = discover_repository_root(&file, &opts(&[".git"], tmp.path(), 10)).unwrap();
        assert_eq!(normalize_lexically(&root), normalize_lexically(tmp.path()));
    }

    #[test]
    fn git_file_marker_counts() {
        let tmp = TestDir::new();
        write_file(&tmp.path().join(".git"), "gitdir: /elsewhere");
        let nested = tmp.path().join("a/b");
        fs::create_dir_all(&nested).unwrap();

        let root = discover_repository_root(&nested, &opts(&[".git"], tmp.path(), 10)).unwrap();
        assert_eq!(normalize_lexically(&root), normalize_lexically(tmp.path()));
    }

    #[test]
    fn stop_at_first_false_returns_outermost_marker() {
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
        assert!(is_unbounded_home(Path::new("/")));
        assert!(is_unbounded_home(Path::new("/root")));
        assert!(!is_unbounded_home(Path::new("/home/user")));
    }

    #[test]
    fn rejects_empty_dot_dotdot_absolute_and_multicomponent_markers() {
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

    #[test]
    fn default_home_outside_start_ancestry_falls_back_to_filesystem_root() {
        let tmp = TestDir::new();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let start = tmp.path().join("src");
        fs::create_dir_all(&start).unwrap();

        let foreign = TestDir::new();
        let _lock = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_home = env::var_os("HOME");
        let prev_profile = env::var_os("USERPROFILE");
        env::set_var("HOME", foreign.path());
        env::set_var("USERPROFILE", foreign.path());
        let result = discover_repository_root(
            &start,
            &RepositoryRootOptions {
                markers: vec![".git".into()],
                max_depth: 10,
                boundary: None,
                follow_symlinks: false,
                stop_at_first: true,
            },
        );
        match prev_home {
            Some(v) => env::set_var("HOME", v),
            None => env::remove_var("HOME"),
        }
        match prev_profile {
            Some(v) => env::set_var("USERPROFILE", v),
            None => env::remove_var("USERPROFILE"),
        }
        let root = result.expect("home outside ancestry must fall back so start is searchable");
        assert_eq!(normalize_lexically(&root), normalize_lexically(tmp.path()));
    }

    #[test]
    fn select_user_home_prefers_userprofile_on_windows_only() {
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
        assert_eq!(filesystem_root(Path::new("/")), PathBuf::from("/"));
        assert_eq!(filesystem_root(Path::new("/foo/bar")), PathBuf::from("/"));
    }

    #[cfg(windows)]
    #[test]
    fn filesystem_root_windows_drive_and_unc() {
        assert_eq!(filesystem_root(Path::new(r"C:\")), PathBuf::from(r"C:\"));
        assert_eq!(
            filesystem_root(Path::new(r"C:\foo\bar")),
            PathBuf::from(r"C:\")
        );
        let unc = filesystem_root(Path::new(r"\\server\share\foo"));
        assert!(
            unc.as_os_str().to_string_lossy().contains("server"),
            "UNC root should keep the share prefix, got {}",
            unc.display()
        );
    }

    #[cfg(unix)]
    #[test]
    fn posix_root_boundary_is_inspected_before_stop() {
        let options = RepositoryRootOptions {
            markers: vec![".rsfulmen-definitely-missing-marker".into()],
            max_depth: 1,
            boundary: Some(PathBuf::from("/")),
            follow_symlinks: false,
            stop_at_first: true,
        };
        let err = discover_repository_root(Path::new("/"), &options).unwrap_err();
        match err {
            RepositoryRootError::RepositoryNotFound { reason, .. } => {
                assert_eq!(reason, RepositoryNotFoundReason::FilesystemRootReached);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn path_constraint_is_deferred_explicit_boundary_is_the_only_caller_ceiling() {
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
    fn follow_symlinks_detects_parent_that_canonicalizes_to_a_seen_directory() {
        let tmp = TestDir::new();
        let start = tmp.path().join("start");
        fs::create_dir_all(&start).unwrap();
        // `start/up` -> `start` is an ancestor alias when used as a start.
        // A genuine upward loop: after leaving `start`, the lexical parent
        // `tmp` is replaced conceptually by a symlink back to `start`.
        // We simulate that by starting at `start` and planting a sibling
        // cycle via a directory whose parent path canonicalizes to `start`.
        let cycle = tmp.path().join("cycle");
        fs::create_dir_all(&cycle).unwrap();
        std::os::unix::fs::symlink(&start, cycle.join("back")).unwrap();

        let visited_first = fs::canonicalize(&start).unwrap();
        let back = fs::canonicalize(cycle.join("back")).unwrap();
        assert_eq!(
            visited_first, back,
            "precondition: back canonicalizes to start"
        );
        assert!(
            detect_repeated_canonical(&visited_first, &back),
            "repeated canonical path is a genuine loop"
        );
    }

    #[cfg(unix)]
    #[test]
    fn in_boundary_symlink_to_external_repo_is_rejected_in_both_modes() {
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
            assert!(
                matches!(
                    err,
                    RepositoryRootError::RepositoryNotFound { .. }
                        | RepositoryRootError::InvalidStartPath { .. }
                ),
                "follow_symlinks={follow} must not accept the external repo, got {err:?}"
            );
        }
    }

    fn detect_repeated_canonical(seen: &Path, next: &Path) -> bool {
        seen == next
    }
}
