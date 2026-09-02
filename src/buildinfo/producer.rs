//! Application `build.rs` producer for `FULMEN_HOST_*` compile stamps.
//!
//! Split into a testable [`probe`] (explicit manifest dir, target, and env) and
//! a thin [`emit_host_identity`] entry for build scripts. Git locator / index /
//! object / config overrides in the provided env are **not** forwarded to the
//! Git subprocess.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use super::go_form_os_arch;

/// Build-time env keys that may inject stamps (CI) and must rebuild on change.
pub const STAMP_ENV_KEYS: &[&str] = &[
    "FULMEN_HOST_VERSION",
    "FULMEN_HOST_COMMIT",
    "FULMEN_HOST_BUILD_DATE",
    "FULMEN_HOST_DIRTY",
    "FULMEN_HOST_RUNTIME",
    "FULMEN_HOST_PLATFORM",
];

/// Git overrides stripped from the probe subprocess.
pub const GIT_STRIP_VARS: &[&str] = &[
    "GIT_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_INDEX_FILE",
    "GIT_COMMON_DIR",
    "GIT_WORK_TREE",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

/// Whether `key` is a Git locator / index / object / config override.
#[must_use]
pub fn is_stripped_git_env(key: &str) -> bool {
    GIT_STRIP_VARS.contains(&key) || key.starts_with("GIT_CONFIG")
}

/// Cargo target triple pieces used to stamp Go-form `OS/ARCH`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetSpec {
    /// `CARGO_CFG_TARGET_OS` (e.g. `macos`, `linux`).
    pub os: String,
    /// `CARGO_CFG_TARGET_ARCH` (e.g. `aarch64`, `x86_64`).
    pub arch: String,
}

impl TargetSpec {
    /// Read target cfg from the current build-script environment.
    #[must_use]
    pub fn from_cargo_env() -> Self {
        Self {
            os: env::var("CARGO_CFG_TARGET_OS").unwrap_or_default(),
            arch: env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default(),
        }
    }

    /// Go-form platform (`darwin/arm64`, `linux/amd64`, …).
    #[must_use]
    pub fn go_form(&self) -> String {
        go_form_os_arch(&self.os, &self.arch)
    }
}

/// Inputs to [`probe`]. All paths and env are caller-supplied.
#[derive(Debug, Clone)]
pub struct ProbeRequest {
    /// Application `CARGO_MANIFEST_DIR`.
    pub manifest_dir: PathBuf,
    /// `CARGO_PKG_VERSION` when no `FULMEN_HOST_VERSION` override is set.
    pub pkg_version: String,
    /// Compile target used for the platform stamp.
    pub target: TargetSpec,
    /// Build-script environment (CI injection + inherited Git vars).
    pub env: HashMap<String, String>,
    /// Optional RFC3339 UTC timestamp (tests). `None` uses wall clock.
    pub now_rfc3339: Option<String>,
    /// Optional `rustc --version` first line (tests). `None` probes `RUSTC`.
    pub rustc_version: Option<String>,
}

/// Compile stamps plus Cargo invalidation directives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeResult {
    /// Values to emit as `cargo:rustc-env=FULMEN_HOST_*`.
    pub stamps: HostStamps,
    /// `cargo:rerun-if-changed` paths (HEAD, ref, packed-refs, worktree, …).
    pub rerun_if_changed: Vec<PathBuf>,
    /// `cargo:rerun-if-env-changed` keys.
    pub rerun_if_env_changed: Vec<String>,
}

/// Six host-identity stamps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostStamps {
    /// App version (`dev` if unknown).
    pub version: String,
    /// Full commit SHA, or `unknown`.
    pub commit: String,
    /// RFC3339 UTC, or `unknown`.
    pub build_date: String,
    /// Known dirty flag; `None` is unknown (empty stamp, never a false clean).
    pub dirty: Option<bool>,
    /// Runtime string (e.g. `rustc 1.98.0`). Empty omits at resolve time.
    pub runtime: String,
    /// Go-form platform. Empty omits at resolve time.
    pub platform: String,
}

impl HostStamps {
    /// `FULMEN_HOST_DIRTY` value: `true` / `false` / empty (unknown).
    #[must_use]
    pub fn dirty_stamp(&self) -> &'static str {
        match self.dirty {
            Some(true) => "true",
            Some(false) => "false",
            None => "",
        }
    }
}

/// Probe Git and env from [`ProbeRequest`] without printing Cargo directives.
#[must_use]
pub fn probe(req: &ProbeRequest) -> ProbeResult {
    let layout = discover_git_layout(&req.manifest_dir);
    let mut rerun_if_changed = invalidation_paths(&req.manifest_dir, layout.as_ref());

    let version = nonempty_env(&req.env, "FULMEN_HOST_VERSION")
        .map(str::to_string)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            let v = req.pkg_version.trim();
            if v.is_empty() {
                "dev".to_string()
            } else {
                v.to_string()
            }
        });

    let commit = nonempty_env(&req.env, "FULMEN_HOST_COMMIT")
        .map(str::to_string)
        .unwrap_or_else(|| {
            git_head(&req.manifest_dir, &req.env).unwrap_or_else(|| "unknown".into())
        });

    let build_date = nonempty_env(&req.env, "FULMEN_HOST_BUILD_DATE")
        .map(str::to_string)
        .unwrap_or_else(|| req.now_rfc3339.clone().unwrap_or_else(rfc3339_now_utc));

    let dirty = if let Some(raw) = req.env.get("FULMEN_HOST_DIRTY") {
        parse_dirty_stamp(raw)
    } else {
        git_dirty(&req.manifest_dir, &req.env)
    };

    let runtime = nonempty_env(&req.env, "FULMEN_HOST_RUNTIME")
        .map(str::to_string)
        .unwrap_or_else(|| {
            req.rustc_version
                .clone()
                .unwrap_or_else(|| rustc_version_line(&req.env))
        });

    let platform = nonempty_env(&req.env, "FULMEN_HOST_PLATFORM")
        .map(str::to_string)
        .unwrap_or_else(|| req.target.go_form());

    if let Some(top) = git_toplevel(&req.manifest_dir, &req.env) {
        if !rerun_if_changed.iter().any(|p| p == &top) {
            rerun_if_changed.push(top);
        }
    } else if !rerun_if_changed.iter().any(|p| p == &req.manifest_dir) {
        rerun_if_changed.push(req.manifest_dir.clone());
    }

    let rerun_if_env_changed = STAMP_ENV_KEYS.iter().map(|s| (*s).to_string()).collect();

    ProbeResult {
        stamps: HostStamps {
            version,
            commit,
            build_date,
            dirty,
            runtime,
            platform,
        },
        rerun_if_changed,
        rerun_if_env_changed,
    }
}

/// Print `cargo:rustc-env` and invalidation lines for a [`ProbeResult`].
pub fn emit(result: &ProbeResult) {
    let s = &result.stamps;
    println!("cargo:rustc-env=FULMEN_HOST_VERSION={}", s.version);
    println!("cargo:rustc-env=FULMEN_HOST_COMMIT={}", s.commit);
    println!("cargo:rustc-env=FULMEN_HOST_BUILD_DATE={}", s.build_date);
    println!("cargo:rustc-env=FULMEN_HOST_DIRTY={}", s.dirty_stamp());
    println!("cargo:rustc-env=FULMEN_HOST_RUNTIME={}", s.runtime);
    println!("cargo:rustc-env=FULMEN_HOST_PLATFORM={}", s.platform);
    for path in &result.rerun_if_changed {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    for key in &result.rerun_if_env_changed {
        println!("cargo:rerun-if-env-changed={key}");
    }
}

/// Thin `build.rs` entry: probe the build-script environment and emit stamps.
pub fn emit_host_identity() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "dev".into());
    let req = ProbeRequest {
        manifest_dir: PathBuf::from(manifest_dir),
        pkg_version,
        target: TargetSpec::from_cargo_env(),
        env: env::vars().collect(),
        now_rfc3339: None,
        rustc_version: None,
    };
    emit(&probe(&req));
}

#[derive(Debug, Clone)]
struct GitLayout {
    git_dir: PathBuf,
    common_dir: PathBuf,
}

fn discover_git_layout(start: &Path) -> Option<GitLayout> {
    let mut dir = start.to_path_buf();
    loop {
        let dotgit = dir.join(".git");
        if dotgit.is_dir() {
            return Some(GitLayout {
                git_dir: dotgit.clone(),
                common_dir: dotgit,
            });
        }
        if dotgit.is_file() {
            let text = fs::read_to_string(&dotgit).ok()?;
            let git_dir = parse_gitdir_file(&text, &dir)?;
            let common_file = git_dir.join("commondir");
            let common_dir = if common_file.is_file() {
                let rel = fs::read_to_string(&common_file).ok()?;
                let trimmed = rel.trim();
                let joined = git_dir.join(trimmed);
                fs::canonicalize(&joined).unwrap_or(joined)
            } else {
                git_dir.clone()
            };
            let git_dir = fs::canonicalize(&git_dir).unwrap_or(git_dir);
            return Some(GitLayout {
                git_dir,
                common_dir,
            });
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn parse_gitdir_file(text: &str, worktree: &Path) -> Option<PathBuf> {
    for line in text.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("gitdir:") else {
            continue;
        };
        let path = Path::new(rest.trim());
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            worktree.join(path)
        };
        return Some(fs::canonicalize(&resolved).unwrap_or(resolved));
    }
    None
}

fn invalidation_paths(manifest_dir: &Path, layout: Option<&GitLayout>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let Some(layout) = layout else {
        paths.push(manifest_dir.to_path_buf());
        return paths;
    };
    let head = layout.git_dir.join("HEAD");
    paths.push(head.clone());
    if let Ok(contents) = fs::read_to_string(&head) {
        let contents = contents.trim();
        if let Some(rel) = contents.strip_prefix("ref: ") {
            let rel = rel.trim();
            paths.push(layout.git_dir.join(rel));
            paths.push(layout.common_dir.join(rel));
        }
    }
    paths.push(layout.common_dir.join("packed-refs"));
    paths.push(layout.git_dir.clone());
    paths.push(layout.common_dir.clone());
    paths
}

fn nonempty_env<'a>(env_map: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    env_map.get(key).map(|s| s.trim()).filter(|s| !s.is_empty())
}

fn parse_dirty_stamp(raw: &str) -> Option<bool> {
    match raw.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn git_head(manifest_dir: &Path, env_map: &HashMap<String, String>) -> Option<String> {
    let out = git_capture(manifest_dir, env_map, &["rev-parse", "HEAD"])?;
    let sha = out.trim();
    if sha.is_empty() || sha == "HEAD" {
        None
    } else {
        Some(sha.to_string())
    }
}

fn git_dirty(manifest_dir: &Path, env_map: &HashMap<String, String>) -> Option<bool> {
    let out = git_capture(
        manifest_dir,
        env_map,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )?;
    Some(!out.trim().is_empty())
}

fn git_toplevel(manifest_dir: &Path, env_map: &HashMap<String, String>) -> Option<PathBuf> {
    let out = git_capture(manifest_dir, env_map, &["rev-parse", "--show-toplevel"])?;
    let p = PathBuf::from(out.trim());
    if p.as_os_str().is_empty() {
        None
    } else {
        Some(fs::canonicalize(&p).unwrap_or(p))
    }
}

fn git_capture(
    manifest_dir: &Path,
    env_map: &HashMap<String, String>,
    args: &[&str],
) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());
    cmd.current_dir(manifest_dir);
    cmd.env_clear();
    for (k, v) in env_map {
        if is_stripped_git_env(k) {
            continue;
        }
        cmd.env(k, v);
    }
    if !env_map.contains_key("PATH") {
        if let Ok(path) = env::var("PATH") {
            cmd.env("PATH", path);
        }
    }
    cmd.args(args);
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

fn rustc_version_line(env_map: &HashMap<String, String>) -> String {
    let rustc = env_map
        .get("RUSTC")
        .cloned()
        .or_else(|| env::var("RUSTC").ok())
        .unwrap_or_else(|| "rustc".into());
    let mut cmd = Command::new(rustc);
    cmd.arg("--version");
    cmd.stdin(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd.env_clear();
    for (k, v) in env_map {
        if is_stripped_git_env(k) {
            continue;
        }
        cmd.env(k, v);
    }
    if !env_map.contains_key("PATH") {
        if let Ok(path) = env::var("PATH") {
            cmd.env("PATH", path);
        }
    }
    cmd.output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(|l| l.trim().to_string()))
        .unwrap_or_default()
}

fn rfc3339_now_utc() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    unix_secs_to_rfc3339(secs)
}

fn unix_secs_to_rfc3339(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let hour = rem / 3_600;
    let min = (rem % 3_600) / 60;
    let sec = rem % 60;
    let (year, month, day) = civil_from_unix_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Civil date from days since 1970-01-01 (Howard Hinnant).
fn civil_from_unix_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::{Command, Stdio};
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn base_env() -> HashMap<String, String> {
        let mut m = HashMap::new();
        if let Ok(v) = env::var("PATH") {
            m.insert("PATH".into(), v);
        }
        if let Ok(v) = env::var("HOME") {
            m.insert("HOME".into(), v);
        }
        m
    }

    fn temp_prefix(name: &str) -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix(name)
            .tempdir()
            .expect("tempdir")
    }

    fn git_in(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .current_dir(dir)
            .args(args)
            .env("GIT_AUTHOR_NAME", "rsfulmen-hostid")
            .env("GIT_AUTHOR_EMAIL", "hostid@example.test")
            .env("GIT_COMMITTER_NAME", "rsfulmen-hostid")
            .env("GIT_COMMITTER_EMAIL", "hostid@example.test")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("spawn git");
        assert!(status.success(), "git {args:?} failed in {}", dir.display());
    }

    fn init_repo(dir: &Path, marker: &str) -> String {
        git_in(dir, &["init"]);
        git_in(dir, &["config", "user.email", "hostid@example.test"]);
        git_in(dir, &["config", "user.name", "rsfulmen-hostid"]);
        fs::write(dir.join("README"), marker).unwrap();
        git_in(dir, &["add", "README"]);
        git_in(dir, &["commit", "-m", "init"]);
        git_head(dir, &base_env()).expect("own HEAD")
    }

    fn probe_at(dir: &Path, env_map: HashMap<String, String>) -> ProbeResult {
        probe(&ProbeRequest {
            manifest_dir: dir.to_path_buf(),
            pkg_version: "0.0.0".into(),
            target: TargetSpec {
                os: "linux".into(),
                arch: "x86_64".into(),
            },
            env: env_map,
            now_rfc3339: Some("2026-09-02T00:00:00Z".into()),
            rustc_version: Some("rustc 1.98.0".into()),
        })
    }

    #[test]
    fn unix_epoch_formats() {
        assert_eq!(unix_secs_to_rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(unix_secs_to_rfc3339(1_000_000_000), "2001-09-09T01:46:40Z");
    }

    #[test]
    fn strip_set_includes_secrev_additions() {
        for k in GIT_STRIP_VARS {
            assert!(is_stripped_git_env(k), "{k}");
        }
        assert!(is_stripped_git_env("GIT_CONFIG"));
        assert!(is_stripped_git_env("GIT_CONFIG_GLOBAL"));
        assert!(is_stripped_git_env("GIT_CONFIG_SYSTEM"));
        assert!(is_stripped_git_env("GIT_CONFIG_NOSYSTEM"));
        assert!(is_stripped_git_env("GIT_CONFIG_PARAMETERS"));
        assert!(is_stripped_git_env("GIT_CONFIG_COUNT"));
        assert!(!is_stripped_git_env("GIT_EXEC_PATH"));
        assert!(!is_stripped_git_env("PATH"));
    }

    #[test]
    fn platform_from_target_not_host() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_prefix("rsfulmen-hostid-plat-");
        let mut env_map = base_env();
        env_map.insert("FULMEN_HOST_COMMIT".into(), "abc".into());
        let result = probe_at(dir.path(), env_map);
        assert_eq!(result.stamps.platform, "linux/amd64");
    }

    #[test]
    fn foreign_git_dir_does_not_stamp_foreign_commit() {
        let own = temp_prefix("rsfulmen-hostid-own-");
        let foreign = temp_prefix("rsfulmen-hostid-foreign-");
        let own_sha = init_repo(own.path(), "own-repo\n");
        let foreign_sha = init_repo(foreign.path(), "foreign-repo\n");
        assert_ne!(own_sha, foreign_sha);

        let mut env_map = base_env();
        env_map.insert(
            "GIT_DIR".into(),
            foreign.path().join(".git").to_string_lossy().into(),
        );
        let result = probe_at(own.path(), env_map);
        assert_ne!(result.stamps.commit, foreign_sha);
        assert_eq!(result.stamps.commit, own_sha);
    }

    #[test]
    fn dirty_tracked_and_untracked_is_true() {
        let own = temp_prefix("rsfulmen-hostid-dirty-");
        init_repo(own.path(), "own-dirty\n");
        fs::write(own.path().join("README"), "edited\n").unwrap();
        fs::write(own.path().join("untracked.txt"), "new\n").unwrap();
        let result = probe_at(own.path(), base_env());
        assert_eq!(result.stamps.dirty, Some(true));
    }

    #[test]
    fn foreign_index_does_not_mask_dirty() {
        let own = temp_prefix("rsfulmen-hostid-idx-own-");
        let foreign = temp_prefix("rsfulmen-hostid-idx-foreign-");
        init_repo(own.path(), "own-idx\n");
        init_repo(foreign.path(), "foreign-idx\n");
        fs::write(own.path().join("README"), "edited\n").unwrap();
        fs::write(own.path().join("untracked.txt"), "new\n").unwrap();
        let mut env_map = base_env();
        env_map.insert(
            "GIT_INDEX_FILE".into(),
            foreign.path().join(".git/index").to_string_lossy().into(),
        );
        let result = probe_at(own.path(), env_map);
        assert_eq!(result.stamps.dirty, Some(true));
    }

    #[test]
    fn no_git_commit_unknown_dirty_omitted() {
        let dir = temp_prefix("rsfulmen-hostid-nogit-");
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();
        let result = probe_at(dir.path(), base_env());
        assert_eq!(result.stamps.commit, "unknown");
        assert_eq!(result.stamps.dirty, None);
    }

    #[test]
    fn malformed_dirty_override_is_unknown() {
        let dir = temp_prefix("rsfulmen-hostid-mal-");
        let mut env_map = base_env();
        env_map.insert("FULMEN_HOST_DIRTY".into(), "yes".into());
        let result = probe_at(dir.path(), env_map);
        assert_eq!(result.stamps.dirty, None);
    }

    #[test]
    fn detached_packed_and_worktree_resolve_own_commit() {
        let own = temp_prefix("rsfulmen-hostid-geo-");
        let sha = init_repo(own.path(), "own-geo\n");

        git_in(own.path(), &["checkout", "--detach", "HEAD"]);
        let detached = probe_at(own.path(), base_env());
        assert_eq!(detached.stamps.commit, sha);
        assert!(
            detached
                .rerun_if_changed
                .iter()
                .any(|p| p.ends_with("HEAD")),
            "detached HEAD must invalidate git HEAD: {:?}",
            detached.rerun_if_changed
        );

        git_in(own.path(), &["checkout", "-B", "hostid-rebranch"]);
        git_in(own.path(), &["pack-refs", "--all"]);
        let packed = probe_at(own.path(), base_env());
        assert_eq!(packed.stamps.commit, sha);
        assert!(
            packed
                .rerun_if_changed
                .iter()
                .any(|p| p.ends_with("packed-refs")),
            "packed-refs must be in rerun list: {:?}",
            packed.rerun_if_changed
        );

        let wt = temp_prefix("rsfulmen-hostid-wt-");
        let wt_path = wt.path().join("tree");
        git_in(
            own.path(),
            &[
                "worktree",
                "add",
                "--detach",
                wt_path.to_str().unwrap(),
                "HEAD",
            ],
        );
        let linked = probe_at(&wt_path, base_env());
        assert_eq!(linked.stamps.commit, sha);
        assert!(
            linked.rerun_if_changed.iter().any(|p| {
                p.components().any(|c| c.as_os_str() == "worktrees")
                    || p.join("HEAD").exists()
                    || p.ends_with("HEAD")
            }),
            "linked worktree must invalidate worktree git dir: {:?}",
            linked.rerun_if_changed
        );
    }
}
