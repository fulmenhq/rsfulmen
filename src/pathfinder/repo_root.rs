//! Crucible-aligned repository root discovery.
//!
//! [`discover_repository_root`] walks upward from a start path looking for
//! marker files or directories (`.git`, `go.mod`, `package.json`, …) and
//! stops at the first match, a configured boundary, max depth, or the
//! filesystem root.
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
    /// Absolute ceiling; never walk above this directory.
    ///
    /// `None` (default) uses the user home directory (`HOME` /
    /// `USERPROFILE`). If home is `/` or `/root` (container/CI), the
    /// current working directory is used instead. If home cannot be
    /// resolved, the start directory itself is the ceiling.
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

    let abs_start =
        absolute_path(start_path).map_err(|message| RepositoryRootError::InvalidStartPath {
            start_path: start_path.to_path_buf(),
            message,
        })?;

    let start_meta = fs::metadata(&abs_start).map_err(|err| {
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

    let mut current = if start_meta.is_file() {
        abs_start
            .parent()
            .unwrap_or(abs_start.as_path())
            .to_path_buf()
    } else {
        abs_start
    };

    let boundary = resolve_boundary(&current, options.boundary.as_deref())?;
    let fs_root = filesystem_root(&current);

    let mut visited = HashSet::new();
    let mut outermost: Option<PathBuf> = None;
    let mut depth = 0usize;

    loop {
        if options.follow_symlinks {
            if let Ok(real) = fs::canonicalize(&current) {
                if !visited.insert(real) {
                    return Err(RepositoryRootError::TraversalLoop {
                        start_path: start_path.to_path_buf(),
                        current_path: current,
                        searched_depth: depth,
                    });
                }
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

        if has_marker(&current, &options.markers, options.follow_symlinks) {
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
    let raw = if let Some(boundary) = explicit {
        boundary.to_path_buf()
    } else {
        default_boundary(start)
    };

    let abs = absolute_path(&raw).map_err(|message| RepositoryRootError::InvalidBoundary {
        boundary: raw.clone(),
        message,
    })?;

    if !abs.exists() {
        return Err(RepositoryRootError::InvalidBoundary {
            boundary: abs,
            message: "boundary path does not exist".to_string(),
        });
    }
    Ok(abs)
}

fn default_boundary(start: &Path) -> PathBuf {
    match user_home() {
        Some(home) if !is_unbounded_home(&home) => home,
        _ => env::current_dir().unwrap_or_else(|_| start.to_path_buf()),
    }
}

fn user_home() -> Option<PathBuf> {
    env::var_os("HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or({
            #[cfg(windows)]
            {
                env::var_os("USERPROFILE")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            }
            #[cfg(not(windows))]
            {
                None
            }
        })
        .or_else(env::home_dir)
}

fn is_unbounded_home(home: &Path) -> bool {
    home == Path::new("/") || home == Path::new("/root")
}

fn filesystem_root(path: &Path) -> PathBuf {
    path.components()
        .next()
        .map(|c| PathBuf::from(c.as_os_str()))
        .unwrap_or_else(|| PathBuf::from(std::path::MAIN_SEPARATOR_STR))
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
    fn start_outside_boundary_is_not_found() {
        let boundary = TestDir::new();
        let start = TestDir::new();
        fs::create_dir_all(start.path().join("src")).unwrap();

        let err = discover_repository_root(
            &start.path().join("src"),
            &opts(&[".git"], boundary.path(), 10),
        )
        .unwrap_err();
        match err {
            RepositoryRootError::RepositoryNotFound {
                reason: RepositoryNotFoundReason::BoundaryReached,
                searched_depth: 0,
                ..
            } => {}
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

    #[cfg(unix)]
    #[test]
    fn follow_symlinks_reports_loop_when_start_is_self_symlink() {
        let tmp = TestDir::new();
        let repo = tmp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        std::os::unix::fs::symlink(&repo, repo.join("loop")).unwrap();

        let options = RepositoryRootOptions {
            follow_symlinks: true,
            ..opts(&[".git"], tmp.path(), 8)
        };
        let err = discover_repository_root(&repo.join("loop"), &options).unwrap_err();
        match err {
            RepositoryRootError::TraversalLoop { searched_depth, .. } => {
                assert!(searched_depth <= 1, "loop should be seen on the parent hop");
            }
            other => panic!("expected TraversalLoop, got {other:?}"),
        }
    }
}
