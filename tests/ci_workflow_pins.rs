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
        "estate shape is a single include matrix without a trust flag"
    );

    let include = parse_job_matrix_include(workflow, "platform_smoke");
    let expected: Vec<(String, String)> = [
        ("ubuntu-latest", "x86_64-unknown-linux-gnu"),
        ("ubuntu-latest-arm64-s", "aarch64-unknown-linux-gnu"),
        ("macos-15", "aarch64-apple-darwin"),
        ("windows-latest", "x86_64-pc-windows-msvc"),
        ("windows-latest-arm64-s", "aarch64-pc-windows-msvc"),
    ]
    .into_iter()
    .map(|(runner, target)| (runner.to_string(), target.to_string()))
    .collect();
    assert_eq!(
        include, expected,
        "platform_smoke.strategy.matrix.include must be exactly the five estate pairs"
    );

    let needs = parse_job_needs(workflow, "platforms");
    assert!(
        needs.iter().any(|n| n == "platform_smoke"),
        "platforms.needs must contain platform_smoke, got {needs:?}"
    );

    for label in ["ubuntu-latest-arm64-s", "windows-latest-arm64-s"] {
        assert!(
            actionlint_lists_label(actionlint, label),
            "actionlint.yaml must list {label} so lint recognizes the estate runners"
        );
    }
}

fn parse_job_matrix_include(workflow: &str, job: &str) -> Vec<(String, String)> {
    let body = job_body(workflow, job);
    let include = mapping_block(&body, "include").unwrap_or_else(|| {
        panic!("{job}.strategy.matrix.include is missing");
    });
    let mut pairs = Vec::new();
    let mut runner = None;
    for raw in include.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(value) = line.strip_prefix("- runner:") {
            assert!(
                runner.is_none(),
                "{job} include entry is missing target before the next runner"
            );
            runner = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("target:") {
            let runner = runner
                .take()
                .unwrap_or_else(|| panic!("{job} include target has no runner"));
            pairs.push((runner, value.trim().to_string()));
        }
    }
    assert!(
        runner.is_none(),
        "{job} include entry is missing target at end of list"
    );
    pairs
}

fn parse_job_needs(workflow: &str, job: &str) -> Vec<String> {
    let body = job_body(workflow, job);
    for raw in body.lines() {
        let indent = raw.len() - raw.trim_start().len();
        let line = raw.trim();
        if indent != 4 || !line.starts_with("needs:") {
            continue;
        }
        let value = line.trim_start_matches("needs:").trim();
        if value.is_empty() {
            return mapping_block(&body, "needs")
                .unwrap_or_default()
                .lines()
                .filter_map(|item| item.trim().strip_prefix("- "))
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect();
        }
        return vec![value.to_string()];
    }
    Vec::new()
}

fn job_body(workflow: &str, job: &str) -> String {
    let mut lines = workflow.lines().peekable();
    let header = format!("{job}:");
    while let Some(line) = lines.next() {
        if line == format!("  {header}") || line.trim_start() == header && line.starts_with("  ") {
            let mut body = Vec::new();
            for next in lines.by_ref() {
                let is_next_job = next.starts_with("  ")
                    && !next.starts_with("   ")
                    && next.trim_end().ends_with(':');
                if is_next_job {
                    break;
                }
                body.push(next);
            }
            return body.join("\n");
        }
    }
    panic!("job {job} not found");
}

fn mapping_block(body: &str, key: &str) -> Option<String> {
    let header = format!("{key}:");
    let mut lines = body.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim() != header {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let mut block = Vec::new();
        for next in lines.by_ref() {
            if next.trim().is_empty() {
                continue;
            }
            let next_indent = next.len() - next.trim_start().len();
            if next_indent <= indent {
                break;
            }
            block.push(next);
        }
        return Some(block.join("\n"));
    }
    None
}

fn actionlint_lists_label(actionlint: &str, label: &str) -> bool {
    actionlint
        .lines()
        .map(str::trim)
        .any(|line| line == format!("- {label}"))
}
