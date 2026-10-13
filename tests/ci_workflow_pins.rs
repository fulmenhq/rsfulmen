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
        text.contains("pull_request.head.repo.full_name") && text.contains("github.repository"),
        "fork pull_request events must not run on custom runners"
    );
}
