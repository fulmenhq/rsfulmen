//! Forging-path acceptance gate: stamp time vs run time, real `cargo build`.
//! Fixtures live under OS temp (`rsfulmen-hostid-` prefix) and RAII-clean.

#![cfg(feature = "host-identity-producer")]

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use tempfile::TempDir;

const POISON: &[(&str, &str)] = &[
    ("FULMEN_HOST_VERSION", "poison-ver"),
    ("FULMEN_HOST_COMMIT", "poison-commit"),
    ("FULMEN_HOST_BUILD_DATE", "poison-date"),
    ("FULMEN_HOST_DIRTY", "true"),
    ("FULMEN_HOST_RUNTIME", "poison-runtime"),
    ("FULMEN_HOST_PLATFORM", "linux/forge"),
];

fn rsfulmen_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn shared_target_dir() -> PathBuf {
    static DIR: OnceLock<TempDir> = OnceLock::new();
    DIR.get_or_init(|| {
        tempfile::Builder::new()
            .prefix("rsfulmen-hostid-target-")
            .tempdir()
            .expect("shared CARGO_TARGET_DIR")
    })
    .path()
    .to_path_buf()
}

fn build_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn temp(prefix: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir()
        .expect("tempdir")
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "rsfulmen-hostid")
        .env("GIT_AUTHOR_EMAIL", "hostid@example.test")
        .env("GIT_COMMITTER_NAME", "rsfulmen-hostid")
        .env("GIT_COMMITTER_EMAIL", "hostid@example.test")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("spawn git");
    assert!(status.success(), "git {args:?} failed in {}", dir.display());
}

fn git_stdout(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("git stdout");
    assert!(out.status.success(), "git {args:?} failed");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn init_git_repo(dir: &Path, readme: &str) -> String {
    git(dir, &["init"]);
    git(dir, &["config", "user.email", "hostid@example.test"]);
    git(dir, &["config", "user.name", "rsfulmen-hostid"]);
    fs::write(dir.join("README"), readme).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-m", "init"]);
    git_stdout(dir, &["rev-parse", "HEAD"])
}

#[derive(Clone, Copy)]
enum StampMode {
    Producer,
    AbsentPlatform,
}

fn write_consumer(dir: &Path, pkg: &str, mode: StampMode) {
    let root = rsfulmen_root();
    let root_str = root.display().to_string().replace('\\', "/");
    fs::write(
        dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{pkg}"
version = "0.0.0"
edition = "2021"
rust-version = "1.88"
build = "build.rs"

[dependencies]
rsfulmen = {{ path = "{root_str}", default-features = false, features = ["host-identity-producer"] }}

[build-dependencies]
rsfulmen = {{ path = "{root_str}", default-features = false, features = ["host-identity-producer"] }}
"#
        ),
    )
    .unwrap();
    fs::write(
        dir.join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"1.98.0\"\n",
    )
    .unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("src/main.rs"),
        r#"fn main() {
    let info = rsfulmen::host_identity!();
    print!("{}", info.to_json("fixture", None));
}
"#,
    )
    .unwrap();
    let build_rs = match mode {
        StampMode::Producer => {
            r#"fn main() {
    rsfulmen::buildinfo::producer::emit_host_identity();
}
"#
        }
        StampMode::AbsentPlatform => {
            r#"fn main() {
    println!("cargo:rustc-env=FULMEN_HOST_VERSION=0.0.0");
    println!("cargo:rustc-env=FULMEN_HOST_COMMIT=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    println!("cargo:rustc-env=FULMEN_HOST_BUILD_DATE=2026-01-01T00:00:00Z");
    println!("cargo:rustc-env=FULMEN_HOST_DIRTY=false");
    println!("cargo:rustc-env=FULMEN_HOST_RUNTIME=rustc 1.98.0");
}
"#
        }
    };
    fs::write(dir.join("build.rs"), build_rs).unwrap();
}

fn cargo_build(dir: &Path, extra_env: &HashMap<String, String>) -> PathBuf {
    let _g = build_lock();
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    cmd.current_dir(dir);
    cmd.env("CARGO_TARGET_DIR", shared_target_dir());
    cmd.env("CARGO_TERM_COLOR", "never");
    for (k, _) in env::vars() {
        if rsfulmen::buildinfo::producer::is_stripped_git_env(&k) || k.starts_with("FULMEN_HOST_") {
            cmd.env_remove(k);
        }
    }
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("spawn cargo build");
    assert!(
        out.status.success(),
        "cargo build failed in {}\nstdout:\n{}\nstderr:\n{}",
        dir.display(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let name = {
        let toml = fs::read_to_string(dir.join("Cargo.toml")).expect("fixture Cargo.toml");
        toml.lines()
            .find_map(|l| l.trim().strip_prefix("name = \""))
            .and_then(|s| s.strip_suffix('"'))
            .expect("package name")
            .to_string()
    };
    let bin_name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name
    };
    let bin = shared_target_dir().join("debug").join(bin_name);
    assert!(bin.is_file(), "missing binary {}", bin.display());
    bin
}

fn run_json(bin: &Path, extra_env: &[(&str, &str)]) -> String {
    let mut cmd = Command::new(bin);
    for (k, _) in env::vars() {
        if k.starts_with("FULMEN_HOST_") {
            cmd.env_remove(k);
        }
    }
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("run fixture");
    assert!(
        out.status.success(),
        "fixture failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 json")
}

fn json_str(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":");
    let rest = json.split_once(&needle)?.1;
    if let Some(stripped) = rest.strip_prefix('"') {
        let inner = stripped.split_once('"')?.0;
        return Some(inner.to_string());
    }
    None
}

fn json_has_key(json: &str, key: &str) -> bool {
    json.contains(&format!("\"{key}\":"))
}

fn json_bool(json: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\":");
    let rest = json.split_once(&needle)?.1;
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

#[test]
fn gate1_runtime_poison_stamp_present_byte_identical() {
    let dir = temp("rsfulmen-hostid-g1-");
    write_consumer(dir.path(), "hostid-g1", StampMode::Producer);
    let sha = init_git_repo(dir.path(), "own-g1\n");
    let bin = cargo_build(dir.path(), &HashMap::new());
    let clean = run_json(&bin, &[]);
    let poisoned = run_json(&bin, POISON);
    assert_eq!(
        clean, poisoned,
        "runtime poison must not change stamped output"
    );
    assert_eq!(json_str(&clean, "commit").as_deref(), Some(sha.as_str()));
    assert_eq!(json_str(&clean, "commit").unwrap().len(), 40);
    assert!(!clean.contains("poison"));
    assert_ne!(json_str(&clean, "platform").as_deref(), Some("linux/forge"));
}

#[test]
fn gate2_runtime_poison_stamp_absent_uses_default() {
    let dir = temp("rsfulmen-hostid-g2-");
    write_consumer(dir.path(), "hostid-g2", StampMode::AbsentPlatform);
    let bin = cargo_build(dir.path(), &HashMap::new());
    let json = run_json(&bin, &[("FULMEN_HOST_PLATFORM", "linux/forge")]);
    assert_eq!(
        json_str(&json, "commit").as_deref(),
        Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );
    assert!(
        !json_has_key(&json, "platform"),
        "unstamped platform must be omitted, got {json}"
    );
    assert!(
        !json.contains("linux/forge"),
        "absent stamp must not take process env: {json}"
    );
}

fn git_redirect_cases(foreign: &Path) -> Vec<(&'static str, String)> {
    let git_dir = foreign.join(".git");
    let objects = git_dir.join("objects");
    let index = git_dir.join("index");
    vec![
        ("GIT_DIR", git_dir.to_string_lossy().into()),
        ("GIT_OBJECT_DIRECTORY", objects.to_string_lossy().into()),
        ("GIT_INDEX_FILE", index.to_string_lossy().into()),
        ("GIT_COMMON_DIR", git_dir.to_string_lossy().into()),
        (
            "GIT_CONFIG",
            git_dir.join("config").to_string_lossy().into(),
        ),
        (
            "GIT_CONFIG_GLOBAL",
            git_dir.join("config").to_string_lossy().into(),
        ),
        ("GIT_WORK_TREE", foreign.to_string_lossy().into()),
        (
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
            objects.to_string_lossy().into(),
        ),
    ]
}

#[test]
fn gate3_build_time_git_redirects_never_stamp_foreign() {
    let foreign = temp("rsfulmen-hostid-g3-foreign-");
    let foreign_sha = init_git_repo(foreign.path(), "foreign\n");
    for (var, value) in git_redirect_cases(foreign.path()) {
        let own = temp("rsfulmen-hostid-g3-own-");
        let pkg = format!("hostid-g3-{}", var.to_lowercase().replace('_', "-"));
        write_consumer(own.path(), &pkg, StampMode::Producer);
        let own_sha = init_git_repo(own.path(), &format!("own-{var}\n"));
        assert_ne!(own_sha, foreign_sha);
        let mut extra = HashMap::new();
        extra.insert(var.to_string(), value);
        let bin = cargo_build(own.path(), &extra);
        let json = run_json(&bin, &[]);
        let commit = json_str(&json, "commit").expect("commit field");
        assert_ne!(
            commit, foreign_sha,
            "{var} must not stamp foreign commit {foreign_sha}, got {commit}"
        );
        assert!(
            commit == own_sha || commit == "unknown",
            "{var}: commit must be own HEAD or unknown, got {commit}"
        );
    }
}

#[test]
fn gate4_dirty_and_foreign_index() {
    let foreign = temp("rsfulmen-hostid-g4-foreign-");
    init_git_repo(foreign.path(), "foreign-clean\n");

    let own = temp("rsfulmen-hostid-g4-own-");
    write_consumer(own.path(), "hostid-g4-idx", StampMode::Producer);
    init_git_repo(own.path(), "own-clean\n");
    fs::write(own.path().join("README"), "tracked edit\n").unwrap();
    fs::write(own.path().join("untracked.txt"), "untracked add\n").unwrap();

    let mut extra = HashMap::new();
    extra.insert(
        "GIT_INDEX_FILE".into(),
        foreign
            .path()
            .join(".git/index")
            .to_string_lossy()
            .into_owned(),
    );
    let bin = cargo_build(own.path(), &extra);
    let json = run_json(&bin, &[]);
    assert_eq!(
        json_bool(&json, "dirty"),
        Some(true),
        "dirty must stay true under foreign GIT_INDEX_FILE: {json}"
    );
}

#[test]
fn gate4_no_git_omits_dirty() {
    let dir = temp("rsfulmen-hostid-g4-nogit-");
    write_consumer(dir.path(), "hostid-g4-nogit", StampMode::Producer);
    let bin = cargo_build(dir.path(), &HashMap::new());
    let json = run_json(&bin, &[]);
    assert_eq!(json_str(&json, "commit").as_deref(), Some("unknown"));
    assert!(
        !json_has_key(&json, "dirty"),
        "no-git dirty must be omitted, got {json}"
    );
}
