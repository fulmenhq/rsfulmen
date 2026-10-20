//! `release-publish.sh` must bind the attested, signed tag to what it
//! publishes: tag signature and tagger, tag target == HEAD, attestation
//! fields == live tag, and the remote tag == local tag. Every case runs the
//! real script in a throwaway repository with a throwaway GPG key in a
//! temporary GNUPGHOME, a local bare "origin", and fake `gh` / `minisign`
//! shims. Nothing real is signed, pushed or published.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static FIXTURE_SEQ: AtomicUsize = AtomicUsize::new(0);

const TAGGER_NAME: &str = "rsfulmen-test-tagger";
const TAGGER_EMAIL: &str = "tagger@example.test";
const VERSION: &str = "9.9.9";
const TAG: &str = "v9.9.9";

// Repository locators and config sources that must not leak in from a hook
// or the developer's environment.
const STRIP: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_CEILING_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_NAMESPACE",
    "GIT_PREFIX",
    "GIT_QUARANTINE_PATH",
    "GNUPGHOME",
    "RSFULMEN_RELEASE_TAG",
    "RSFULMEN_ALLOW_UNTAGGED",
    "RSFULMEN_GPG_HOMEDIR",
    "RSFULMEN_TAGGER_NAME",
    "RSFULMEN_TAGGER_EMAIL",
    "RSFULMEN_MINISIGN_PUB",
    "RSFULMEN_RELEASE_REMOTE",
    "RSFULMEN_RELEASE_DIST",
    "RSFULMEN_PGP_KEY_ID",
    "RSFULMEN_MINISIGN_KEY",
];

fn which(tool: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {tool}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

struct Fixture {
    base: PathBuf,
    repo: PathBuf,
    gnupg: PathBuf,
    bin: PathBuf,
    gh_log: PathBuf,
}

impl Fixture {
    /// None when gpg is unavailable (the case is then skipped with a note).
    fn new(label: &str) -> Option<Self> {
        if !which("gpg") {
            eprintln!("skipping {label}: gpg not found");
            return None;
        }
        // Short path: the gpg-agent socket lives under GNUPGHOME. The counter
        // keeps parallel fixtures apart; create_dir fails on any collision
        // rather than letting one fixture's Drop remove another's tree.
        let seq = FIXTURE_SEQ.fetch_add(1, Ordering::Relaxed);
        let base = PathBuf::from(format!("/tmp/rsf-rp-{}-{seq}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir(&base).expect("create unique fixture dir");
        let repo = base.join("repo");
        let gnupg = base.join("g");
        let bin = base.join("bin");
        for d in [&repo, &gnupg, &bin] {
            fs::create_dir_all(d).expect("mkdir");
        }
        fs::set_permissions(&gnupg, fs::Permissions::from_mode(0o700)).expect("chmod gnupg");
        let gh_log = base.join("gh.log");
        let fx = Self {
            base,
            repo,
            gnupg,
            bin,
            gh_log,
        };
        fx.write_shims();
        fx.gen_key();
        fx.init_repo();
        Some(fx)
    }

    fn write_exec(&self, name: &str, body: &str) {
        let path = self.bin.join(name);
        fs::write(&path, body).expect("write shim");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod shim");
    }

    fn write_shims(&self) {
        // `gh release view` reports "no such release"; everything is logged.
        self.write_exec(
            "gh",
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\n[ \"$1 $2\" = 'release view' ] && exit 1\nexit 0\n",
                self.gh_log.display()
            ),
        );
        // minisign is not under test here; the binding checks are.
        self.write_exec("minisign", "#!/bin/sh\nexit 0\n");
    }

    fn cmd(&self, program: &str) -> Command {
        let mut cmd = Command::new(program);
        for key in STRIP {
            cmd.env_remove(key);
        }
        let path = format!(
            "{}:{}",
            self.bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        cmd.current_dir(&self.repo)
            .env("PATH", path)
            .env("GNUPGHOME", &self.gnupg)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", TAGGER_NAME)
            .env("GIT_AUTHOR_EMAIL", TAGGER_EMAIL)
            .env("GIT_COMMITTER_NAME", TAGGER_NAME)
            .env("GIT_COMMITTER_EMAIL", TAGGER_EMAIL);
        cmd
    }

    fn run(&self, program: &str, args: &[&str]) -> String {
        let out = self.cmd(program).args(args).output().expect("spawn");
        assert!(
            out.status.success(),
            "{program} {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn git(&self, args: &[&str]) -> String {
        self.run("git", args)
    }

    fn gen_key(&self) {
        self.run(
            "gpg",
            &[
                "--batch",
                "--passphrase",
                "",
                "--quick-gen-key",
                &format!("{TAGGER_NAME} <{TAGGER_EMAIL}>"),
                "ed25519",
                "sign",
                "1d",
            ],
        );
    }

    fn init_repo(&self) {
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let scripts = self.repo.join("scripts");
        fs::create_dir_all(scripts.join("lib")).expect("mkdir scripts");
        for rel in [
            "scripts/release-publish.sh",
            "scripts/release-guard-tag-version.sh",
            "scripts/lib/release-tagger.sh",
        ] {
            fs::copy(src.join(rel), self.repo.join(rel)).expect("copy script");
        }
        fs::write(self.repo.join("VERSION"), format!("{VERSION}\n")).expect("VERSION");
        fs::write(self.repo.join("CHANGELOG.md"), "# Changelog\n").expect("CHANGELOG");
        fs::write(self.repo.join("pub.key"), "test minisign pubkey\n").expect("pub");
        let origin = self.base.join("origin.git");
        self.run("git", &["init", "-q", "--bare", origin.to_str().unwrap()]);
        self.git(&["init", "-q", "-b", "main"]);
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", "first"]);
        self.git(&["commit", "-q", "--allow-empty", "-m", "second"]);
        self.git(&["remote", "add", "origin", origin.to_str().unwrap()]);
        self.git(&["push", "-q", "origin", "main"]);
    }

    fn signed_tag(&self, target: &str) {
        self.git(&["tag", "-s", "-a", TAG, "-m", "Release", target]);
    }

    fn write_attestation(&self, tag: &str, object: &str, target: &str) {
        let dist = self.repo.join("dist/release");
        fs::create_dir_all(&dist).expect("dist");
        let payload = dist.join(format!("{TAG}.tag.txt"));
        // Same layout release-tag.sh writes (tab-indented lines).
        fs::write(
            &payload,
            format!("\ttag: {tag}\n\ttag_object: {object}\n\ttag_target: {target}\n"),
        )
        .expect("payload");
        fs::write(dist.join(format!("{TAG}.tag.txt.minisig")), "sig\n").expect("sig");
    }

    fn attest_live_tag(&self) {
        let object = self.git(&["rev-parse", &format!("refs/tags/{TAG}")]);
        let target = self.git(&["rev-parse", &format!("refs/tags/{TAG}^{{commit}}")]);
        self.write_attestation(TAG, &object, &target);
    }

    fn push_tag(&self) {
        self.git(&["push", "-q", "origin", &format!("refs/tags/{TAG}")]);
    }

    fn publish(&self) -> Output {
        self.publish_with(&[])
    }

    fn publish_with(&self, extra: &[(&str, &str)]) -> Output {
        let script = self.repo.join("scripts/release-publish.sh");
        let mut cmd = self.cmd(script.to_str().unwrap());
        cmd.env("RSFULMEN_TAGGER_NAME", TAGGER_NAME)
            .env("RSFULMEN_TAGGER_EMAIL", TAGGER_EMAIL)
            .env("RSFULMEN_MINISIGN_PUB", self.repo.join("pub.key"));
        for (k, v) in extra {
            cmd.env(k, v);
        }
        cmd.output().expect("run release-publish.sh")
    }

    fn gh_created(&self) -> bool {
        fs::read_to_string(&self.gh_log)
            .map(|log| log.lines().any(|l| l.starts_with("release create")))
            .unwrap_or(false)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = Command::new("gpgconf")
            .args(["--kill", "gpg-agent"])
            .env("GNUPGHOME", &self.gnupg)
            .output();
        let _ = fs::remove_dir_all(&self.base);
    }
}

fn assert_refused(fx: &Fixture, out: &Output, needle: &str) {
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "publish must fail; stderr: {stderr}");
    assert!(stderr.contains(needle), "expected {needle:?} in: {stderr}");
    assert!(!fx.gh_created(), "gh release create must not run");
}

#[test]
fn publishes_when_tag_attestation_and_remote_agree() {
    let Some(fx) = Fixture::new("happy path") else {
        return;
    };
    fx.signed_tag("HEAD");
    fx.attest_live_tag();
    fx.push_tag();
    for extra in [&[][..], &[("RSFULMEN_RELEASE_TAG", TAG)][..]] {
        let _ = fs::remove_file(&fx.gh_log);
        let out = fx.publish_with(extra);
        assert!(
            out.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(fx.gh_created(), "gh release create should run");
    }
}

#[test]
fn rejects_attestation_with_mismatched_target() {
    let Some(fx) = Fixture::new("payload target") else {
        return;
    };
    fx.signed_tag("HEAD");
    let object = fx.git(&["rev-parse", &format!("refs/tags/{TAG}")]);
    let other = fx.git(&["rev-parse", "HEAD~1"]);
    fx.write_attestation(TAG, &object, &other);
    fx.push_tag();
    assert_refused(&fx, &fx.publish(), "attestation tag_target mismatch");
}

#[test]
fn rejects_attestation_for_a_different_tag_object() {
    let Some(fx) = Fixture::new("payload object") else {
        return;
    };
    fx.signed_tag("HEAD");
    let target = fx.git(&["rev-parse", "HEAD"]);
    fx.write_attestation(TAG, &"0".repeat(40), &target);
    fx.push_tag();
    assert_refused(&fx, &fx.publish(), "attestation tag_object mismatch");
}

#[test]
fn rejects_unsigned_annotated_tag() {
    let Some(fx) = Fixture::new("unsigned tag") else {
        return;
    };
    fx.git(&["tag", "-a", TAG, "-m", "Release", "HEAD"]);
    fx.attest_live_tag();
    fx.push_tag();
    assert_refused(&fx, &fx.publish(), "has no valid signature");
}

#[test]
fn rejects_tag_that_does_not_target_head() {
    let Some(fx) = Fixture::new("wrong-target tag") else {
        return;
    };
    fx.signed_tag("HEAD~1");
    fx.attest_live_tag();
    fx.push_tag();
    // Without the override the existing guard already refuses (no exact tag
    // at HEAD); the override path is the one that needs the target check.
    assert_refused(&fx, &fx.publish(), "no exact tag found for HEAD");
    assert_refused(
        &fx,
        &fx.publish_with(&[("RSFULMEN_RELEASE_TAG", TAG)]),
        "target vs HEAD mismatch",
    );
}

#[test]
fn rejects_wrong_tagger_identity() {
    let Some(fx) = Fixture::new("wrong tagger") else {
        return;
    };
    fx.signed_tag("HEAD");
    fx.attest_live_tag();
    fx.push_tag();
    let script = fx.repo.join("scripts/release-publish.sh");
    let out = fx
        .cmd(script.to_str().unwrap())
        .env("RSFULMEN_TAGGER_NAME", "someone-else")
        .env("RSFULMEN_TAGGER_EMAIL", TAGGER_EMAIL)
        .env("RSFULMEN_MINISIGN_PUB", fx.repo.join("pub.key"))
        .output()
        .expect("run");
    assert_refused(&fx, &out, "tagger does not match");
}

#[test]
fn rejects_when_remote_tag_differs_or_is_missing() {
    let Some(fx) = Fixture::new("remote tag") else {
        return;
    };
    fx.signed_tag("HEAD");
    fx.attest_live_tag();
    // Not pushed: remote has no such tag.
    assert_refused(&fx, &fx.publish(), "tag v9.9.9 object mismatch");

    // Remote holds a different tag object (same name, older commit).
    fx.git(&["tag", "-d", TAG]);
    fx.signed_tag("HEAD~1");
    fx.push_tag();
    fx.git(&["tag", "-d", TAG]);
    fx.signed_tag("HEAD");
    fx.attest_live_tag();
    assert_refused(&fx, &fx.publish(), "tag v9.9.9 object mismatch");
}

#[test]
fn rejects_lightweight_tag() {
    let Some(fx) = Fixture::new("lightweight tag") else {
        return;
    };
    fx.git(&["tag", TAG, "HEAD"]);
    let head = fx.git(&["rev-parse", "HEAD"]);
    fx.write_attestation(TAG, &head, &head);
    assert_refused(&fx, &fx.publish(), "is not an annotated tag");
}
