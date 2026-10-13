//! Workflow supply-chain pins used by the estate native matrix.

#[test]
fn ci_third_party_actions_are_sha_pinned() {
    let text = include_str!("../.github/workflows/ci.yml");
    let mut uses = 0usize;
    for (idx, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if !line.starts_with("uses:") {
            continue;
        }
        uses += 1;
        let spec = line.trim_start_matches("uses:").trim();
        let (action, rest) = spec
            .split_once('#')
            .map(|(a, c)| (a.trim(), Some(c.trim())))
            .unwrap_or((spec, None));
        let sha = action.rsplit_once('@').map(|(_, sha)| sha).unwrap_or("");
        assert!(
            sha.len() == 40 && sha.chars().all(|c| c.is_ascii_hexdigit()),
            "line {}: action must be pinned to a 40-hex SHA: {line}",
            idx + 1
        );
        assert!(
            rest.is_some_and(|c| !c.is_empty()),
            "line {}: pinned action needs a version comment: {line}",
            idx + 1
        );
    }
    assert!(
        uses >= 8,
        "expected every job to declare a uses: pin, found {uses}"
    );
}

#[test]
fn rust_toolchain_sha_pins_set_toolchain() {
    let text = include_str!("../.github/workflows/ci.yml");
    let lines: Vec<&str> = text.lines().collect();
    let mut pins = 0usize;
    for (i, line) in lines.iter().enumerate() {
        if !line.contains("dtolnay/rust-toolchain@") {
            continue;
        }
        pins += 1;
        let window = lines[i..i.saturating_add(8).min(lines.len())].join("\n");
        assert!(
            window.contains("toolchain:"),
            "line {}: SHA-pinned rust-toolchain must set toolchain:",
            i + 1
        );
    }
    assert!(
        pins >= 2,
        "expected rust-toolchain pins for 1.98 and 1.88, found {pins}"
    );
}

#[test]
fn estate_matrix_uses_standard_hosted_labels() {
    let workflow = include_str!("../.github/workflows/ci.yml");
    let actionlint = include_str!("../.github/actionlint.yaml");
    assert!(
        !workflow.contains("platform_smoke_trusted:"),
        "estate labels are GitHub-hosted larger runners; do not split a same-repo job"
    );
    assert!(
        !workflow.contains("trusted_runner:"),
        "decernor/storageprims shape is a single include matrix without a trust flag"
    );
    for label in ["ubuntu-latest-arm64-s", "windows-latest-arm64-s"] {
        assert!(
            workflow.contains(label),
            "workflow must use the standard estate label {label}"
        );
        assert!(
            actionlint.contains(label),
            "actionlint.yaml must list {label} so lint recognizes the estate runners"
        );
    }
    assert!(
        workflow.contains("ubuntu-latest")
            && workflow.contains("macos-15")
            && workflow.contains("windows-latest"),
        "5-way matrix must keep the GitHub-hosted amd64/mac labels"
    );
}
