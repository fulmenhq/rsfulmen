//! Signed-tag scripts require RSFULMEN_TAGGER_NAME / RSFULMEN_TAGGER_EMAIL.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn script(name: &str) -> PathBuf {
    repo_root().join("scripts").join(name)
}

fn run_without_tagger(script_name: &str) -> std::process::Output {
    Command::new(script(script_name))
        .current_dir(repo_root())
        .env_remove("RSFULMEN_TAGGER_NAME")
        .env_remove("RSFULMEN_TAGGER_EMAIL")
        .output()
        .unwrap_or_else(|e| panic!("failed to run {script_name}: {e}"))
}

#[test]
fn release_tag_requires_tagger_env() {
    let output = run_without_tagger("release-tag.sh");
    assert!(
        !output.status.success(),
        "release-tag.sh must fail without tagger env"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("RSFULMEN_TAGGER_NAME is required"),
        "stderr should name the missing env, got: {stderr}"
    );
}

#[test]
fn release_verify_tag_requires_tagger_env() {
    let output = run_without_tagger("release-verify-tag.sh");
    assert!(
        !output.status.success(),
        "release-verify-tag.sh must fail without tagger env"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("RSFULMEN_TAGGER_NAME is required"),
        "stderr should name the missing env, got: {stderr}"
    );
}

#[test]
fn release_tag_requires_tagger_email() {
    let output = Command::new(script("release-tag.sh"))
        .current_dir(repo_root())
        .env("RSFULMEN_TAGGER_NAME", "example-tagger")
        .env_remove("RSFULMEN_TAGGER_EMAIL")
        .output()
        .expect("run release-tag.sh");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("RSFULMEN_TAGGER_EMAIL is required"),
        "stderr should name the missing env, got: {stderr}"
    );
}
