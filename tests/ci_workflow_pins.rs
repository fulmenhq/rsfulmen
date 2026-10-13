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
fn custom_s_runners_require_same_repository_pull_request() {
    let text = include_str!("../.github/workflows/ci.yml");
    assert!(
        text.contains("trusted_runner: true"),
        "custom -s runners must be marked trusted_runner"
    );
    assert!(
        text.contains("ubuntu-latest-arm64-s") && text.contains("windows-latest-arm64-s"),
        "estate 5-way labels must stay present"
    );
    assert!(
        text.contains("platform_smoke_trusted:"),
        "custom -s runners must live in a separate job so the gate can skip them"
    );
    assert!(
        text.contains("pull_request.head.repo.full_name") && text.contains("github.repository"),
        "fork pull_request events must not run on custom runners"
    );
    // GitHub rejects `matrix` in a job-level `if` at compile time
    // ("Unrecognized named-value: 'matrix'") and fails the whole workflow
    // with zero jobs. The trusted-runner gate must not use that context.
    for (idx, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.starts_with("if:") {
            assert!(
                !line.contains("matrix."),
                "line {}: job-level if cannot reference matrix: {line}",
                idx + 1
            );
        }
    }
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
