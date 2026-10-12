//! `release-publish.sh` must fail before `gh` when the guard or assets are absent.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn version_tag() -> String {
    let version = fs::read_to_string(repo_root().join("VERSION")).expect("read VERSION");
    format!("v{}", version.trim())
}

struct FakeGh {
    bin_dir: PathBuf,
    log: PathBuf,
}

impl FakeGh {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!(
            "rsfulmen-release-publish-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let bin_dir = base.join("bin");
        fs::create_dir_all(&bin_dir).expect("create fake gh dir");
        let log = base.join("gh.log");
        let script = bin_dir.join("gh");
        let body = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexit 0\n",
            log.display()
        );
        fs::write(&script, body).expect("write fake gh");
        let mut perms = fs::metadata(&script).expect("stat gh").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).expect("chmod gh");
        Self { bin_dir, log }
    }

    fn path_env(&self) -> String {
        let original = std::env::var("PATH").unwrap_or_default();
        format!("{}:{}", self.bin_dir.display(), original)
    }

    fn was_called(&self) -> bool {
        self.log.exists()
    }
}

impl Drop for FakeGh {
    fn drop(&mut self) {
        if let Some(parent) = self.bin_dir.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }
}

fn run_publish(
    tag: &str,
    dist: &Path,
    gh: &FakeGh,
    extra: &[(&str, &str)],
) -> std::process::Output {
    let mut cmd = Command::new(repo_root().join("scripts/release-publish.sh"));
    cmd.current_dir(repo_root())
        .env("RSFULMEN_RELEASE_TAG", tag)
        .env("RSFULMEN_RELEASE_DIST", dist)
        .env("PATH", gh.path_env())
        .env_remove("RSFULMEN_ALLOW_UNTAGGED");
    for (key, value) in extra {
        cmd.env(key, value);
    }
    cmd.output().expect("run release-publish.sh")
}

#[test]
fn release_publish_rejects_untagged_override() {
    let gh = FakeGh::new();
    let dist = gh.bin_dir.parent().expect("temp parent").join("dist");
    fs::create_dir_all(&dist).expect("dist");
    let output = run_publish(
        &version_tag(),
        &dist,
        &gh,
        &[("RSFULMEN_ALLOW_UNTAGGED", "1")],
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("RSFULMEN_ALLOW_UNTAGGED=1 is not valid for release-publish"),
        "stderr: {stderr}"
    );
    assert!(
        !gh.was_called(),
        "gh must not run when untagged override is set"
    );
}

#[test]
fn release_publish_fails_when_tag_mismatches_version() {
    let gh = FakeGh::new();
    let dist = gh.bin_dir.parent().expect("temp parent").join("dist");
    fs::create_dir_all(&dist).expect("dist");
    let output = run_publish("v0.0.0", &dist, &gh, &[]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("release tag/version mismatch"),
        "stderr: {stderr}"
    );
    assert!(!gh.was_called(), "gh must not run when the guard fails");
}

#[test]
fn release_publish_fails_fast_when_assets_missing() {
    let gh = FakeGh::new();
    let dist = gh.bin_dir.parent().expect("temp parent").join("empty-dist");
    fs::create_dir_all(&dist).expect("dist");
    let tag = version_tag();
    let output = run_publish(&tag, &dist, &gh, &[]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let payload = dist.join(format!("{tag}.tag.txt"));
    let sig = dist.join(format!("{tag}.tag.txt.minisig"));
    assert!(
        stderr.contains(&format!(
            "required release asset missing or empty: {}",
            payload.display()
        )),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains(&format!(
            "required release asset missing or empty: {}",
            sig.display()
        )),
        "stderr: {stderr}"
    );
    assert!(
        !gh.was_called(),
        "gh must not run when attestation assets are missing"
    );
}
