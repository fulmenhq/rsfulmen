//! Workflow supply-chain pins used by the estate native matrix.
//!
//! How the checks fit together:
//!
//! 1. A strict style gate on the source text: no anchors, aliases, or merge
//!    keys; no quoted `uses` keys; no flow-style job or step mappings that
//!    contain `uses`. serde_yaml does not expose parser events, so the gate
//!    reads the text, skipping block-scalar content (`run: |` bodies) and
//!    comments.
//! 2. A small block-style line scanner that tracks the key path of each line
//!    by indentation. Under the style gate, every action reference is a block
//!    line `uses: <ref> # <comment>` (optionally `- uses:`) at
//!    `jobs.<id>.uses` or `jobs.<id>.steps[*].uses`.
//! 3. The parsed document gives the same positions in document order. Each
//!    parsed `uses` is bound to the scanned line at the same index, and the
//!    values must match. The counts must match (fail-closed backstop). The
//!    bound line's comment is then checked. Identical pins repeat across
//!    jobs, so binding is by position rather than by value lookup.

use serde_yaml::{Mapping, Value};

const WORKFLOW: &str = include_str!("../.github/workflows/ci.yml");
const TOOLCHAIN_FILE: &str = include_str!("../rust-toolchain.toml");

/// `dtolnay/rust-toolchain` commit on its `master` branch. The per-version
/// branches (`1.98.0`, `stable`, ...) are regenerated upstream, so a SHA taken
/// from one of them stops being reachable. Master history is not rewritten.
/// Check: `git merge-base --is-ancestor <sha> origin/master` in a clone.
const RUST_TOOLCHAIN_SHA: &str = "df2a88e9fe4098bad09cb42c3dec7187f15861fc";
const RUST_TOOLCHAIN_COMMENT: &str = "master 2026-11-14";
const RUST_TOOLCHAIN_ACTION: &str = "dtolnay/rust-toolchain";
const MSRV: &str = "1.88.0";

/// Local (`./`) and `docker://` refs allowed in the workflow. None today.
const ALLOWED_NON_ACTION_REFS: &[&str] = &[];

// ---------------------------------------------------------------------------
// Source scanning
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum Seg {
    Key(String),
    Item,
}

/// One block-style line outside block-scalar content.
#[derive(Debug)]
struct SourceLine {
    no: usize,
    /// Key path ending at this line's key (or at its enclosing node).
    path: Vec<Seg>,
    key: Option<String>,
    key_quoted: bool,
    /// Code after the key (or the whole code when there is no key).
    value: String,
    comment: Option<String>,
}

/// Splits a line at the first `#` that starts a comment (line start or
/// after whitespace, outside quotes).
fn split_comment(line: &str) -> (&str, Option<&str>) {
    let (mut single, mut double, mut prev_space) = (false, false, true);
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' if double => escaped = true,
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            '#' if !single && !double && prev_space => {
                return (line[..i].trim_end(), Some(line[i + 1..].trim()));
            }
            _ => {}
        }
        prev_space = c.is_whitespace();
    }
    (line.trim_end(), None)
}

/// Replaces quoted spans with empty quotes so their content is not checked.
fn strip_quoted(code: &str) -> String {
    let (mut out, mut quote) = (String::new(), None::<char>);
    let mut escaped = false;
    for c in code.chars() {
        match quote {
            Some(_) if escaped => escaped = false,
            Some('"') if c == '\\' => escaped = true,
            Some(q) if c == q => {
                quote = None;
                out.push(c);
            }
            Some(_) => {}
            None => {
                if c == '"' || c == '\'' {
                    quote = Some(c);
                }
                out.push(c);
            }
        }
    }
    out
}

/// Parses `key: value` or `key:`. Returns (key, quoted, value).
fn parse_key(rest: &str) -> Option<(String, bool, &str)> {
    let first = rest.chars().next()?;
    let (key, quoted, after) = if first == '"' || first == '\'' {
        let end = rest[1..].find(first)? + 1;
        (rest[1..end].to_string(), true, &rest[end + 1..])
    } else {
        if matches!(first, '{' | '[' | '&' | '*' | '!' | '|' | '>') {
            return None;
        }
        let idx = rest
            .find(": ")
            .or_else(|| rest.ends_with(':').then(|| rest.len() - 1))?;
        (rest[..idx].trim_end().to_string(), false, &rest[idx..])
    };
    let after = after.strip_prefix(':')?;
    if !(after.is_empty() || after.starts_with(' ')) {
        return None;
    }
    Some((key, quoted, after.trim()))
}

fn is_block_scalar_indicator(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some('|' | '>'))
        && chars.all(|c| c.is_ascii_digit() || c == '+' || c == '-')
}

/// Block-style scan of the workflow, skipping blank lines, comment-only
/// lines, and block-scalar content.
fn scan_source(text: &str) -> Vec<SourceLine> {
    let mut out = Vec::new();
    let mut stack: Vec<(usize, Seg)> = Vec::new();
    let mut scalar_col: Option<usize> = None;
    for (idx, raw) in text.lines().enumerate() {
        let indent = raw.len() - raw.trim_start_matches(' ').len();
        if let Some(col) = scalar_col {
            if raw.trim().is_empty() || indent > col {
                continue;
            }
            scalar_col = None;
        }
        let (code, comment) = split_comment(raw);
        if code.trim().is_empty() {
            continue;
        }
        let (mut col, mut rest) = (indent, &code[indent..]);
        while rest == "-" || rest.starts_with("- ") {
            while stack
                .last()
                .is_some_and(|(c, s)| *c > col || (*c == col && *s == Seg::Item))
            {
                stack.pop();
            }
            stack.push((col, Seg::Item));
            let after = rest.get(1..).unwrap_or("");
            let trimmed = after.trim_start();
            col += 1 + after.len() - trimmed.len();
            rest = trimmed;
        }
        let mut line = SourceLine {
            no: idx + 1,
            path: Vec::new(),
            key: None,
            key_quoted: false,
            value: rest.to_string(),
            comment: comment.map(str::to_string),
        };
        if let Some((key, quoted, value)) = parse_key(rest) {
            while stack.last().is_some_and(|(c, _)| *c >= col) {
                stack.pop();
            }
            stack.push((col, Seg::Key(key.clone())));
            if is_block_scalar_indicator(value) {
                scalar_col = Some(col);
            }
            line.key = Some(key);
            line.key_quoted = quoted;
            line.value = value.to_string();
        }
        line.path = stack.iter().map(|(_, s)| s.clone()).collect();
        if !rest.is_empty() {
            out.push(line);
        }
    }
    out
}

fn key_names(path: &[Seg]) -> Vec<Option<&str>> {
    path.iter()
        .map(|s| match s {
            Seg::Key(k) => Some(k.as_str()),
            Seg::Item => None,
        })
        .collect()
}

/// `jobs.<id>.uses` or `jobs.<id>.steps[*].uses`.
fn is_action_position(path: &[Seg]) -> bool {
    matches!(
        key_names(path).as_slice(),
        [Some("jobs"), Some(_), Some("uses")]
            | [Some("jobs"), Some(_), Some("steps"), None, Some("uses")]
    )
}

/// Strict style gate. Returns the first violation.
fn check_style(text: &str) -> Result<(), String> {
    for line in scan_source(text) {
        let n = line.no;
        let mut code = String::new();
        if let Some(key) = &line.key {
            if line.key_quoted && key == "uses" {
                return Err(format!(
                    "style: line {n}: quoted `uses` key; write `uses:` unquoted"
                ));
            }
            if key == "<<" {
                return Err(format!(
                    "style: line {n}: merge keys (`<<`) are not allowed"
                ));
            }
            if !line.key_quoted {
                code.push_str(key);
                code.push(' ');
            }
        }
        code.push_str(&strip_quoted(&line.value));
        if code.contains("<<:") {
            return Err(format!(
                "style: line {n}: merge keys (`<<`) are not allowed"
            ));
        }
        for token in code.split(|c: char| c.is_whitespace() || "[]{},".contains(c)) {
            let mut chars = token.chars();
            let (Some(sigil @ ('&' | '*')), Some(next)) = (chars.next(), chars.next()) else {
                continue;
            };
            if next.is_ascii_alphanumeric() || next == '_' || next == '-' {
                let what = if sigil == '&' { "anchors" } else { "aliases" };
                return Err(format!(
                    "style: line {n}: YAML {what} are not allowed: {token}"
                ));
            }
        }
        let flow = strip_quoted(&line.value);
        if (flow.starts_with('{') || flow.starts_with('[')) && flow.contains("uses") {
            let names = key_names(&line.path);
            let owner = names.iter().rev().flatten().next().copied();
            let job_level =
                line.key.is_some() && matches!(names.as_slice(), [Some("jobs"), Some(_)]);
            if job_level || matches!(owner, Some("jobs" | "steps")) {
                return Err(format!(
                    "style: line {n}: flow-style job or step with `uses`; use block style"
                ));
            }
        }
    }
    Ok(())
}

/// Action references found by the scanner, in document order.
fn scanned_action_lines(text: &str) -> Vec<SourceLine> {
    scan_source(text)
        .into_iter()
        .filter(|l| l.key.as_deref() == Some("uses") && is_action_position(&l.path))
        .collect()
}

fn unquote(value: &str) -> &str {
    for q in ['"', '\''] {
        if let Some(inner) = value.strip_prefix(q).and_then(|v| v.strip_suffix(q)) {
            return inner;
        }
    }
    value
}

// ---------------------------------------------------------------------------
// Parsed document
// ---------------------------------------------------------------------------

fn parse_workflow(text: &str) -> Result<Value, String> {
    serde_yaml::from_str(text).map_err(|e| format!("workflow is not valid YAML: {e}"))
}

fn jobs(doc: &Value) -> Vec<&Mapping> {
    doc.get("jobs")
        .and_then(Value::as_mapping)
        .map(|jobs| jobs.values().filter_map(Value::as_mapping).collect())
        .unwrap_or_default()
}

/// Parsed values at `jobs.<id>.uses` and `jobs.<id>.steps[*].uses`, in
/// document order.
fn parsed_action_uses(doc: &Value) -> Vec<&Value> {
    let mut out = Vec::new();
    for job in jobs(doc) {
        for (key, value) in job {
            match key.as_str() {
                Some("uses") => out.push(value),
                Some("steps") => {
                    for step in value.as_sequence().into_iter().flatten() {
                        if let Some(uses) = step.get("uses") {
                            out.push(uses);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// Splits `owner/repo[/path]@ref`. Returns lowercased `owner/repo` and ref.
fn split_action(spec: &str) -> Option<(String, &str)> {
    let (path, git_ref) = spec.rsplit_once('@')?;
    let segments: Vec<&str> = path.split('/').collect();
    let valid = |s: &&str| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    if segments.len() < 2 || !segments.iter().all(valid) || segments[..2].contains(&"..") {
        return None;
    }
    Some((
        format!("{}/{}", segments[0], segments[1]).to_ascii_lowercase(),
        git_ref,
    ))
}

fn is_full_sha(git_ref: &str) -> bool {
    git_ref.len() == 40 && git_ref.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_release_tag(comment: &str) -> bool {
    let Some(version) = comment.strip_prefix('v') else {
        return false;
    };
    !version.is_empty()
        && version
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

// ---------------------------------------------------------------------------
// Checks
// ---------------------------------------------------------------------------

/// Style gate, parsed ref form, recorded rust-toolchain SHA, and the comment
/// on the source line bound to each parsed action reference. Returns the
/// number of action references.
fn check_action_pins(text: &str) -> Result<usize, String> {
    check_style(text)?;
    let doc = parse_workflow(text)?;
    let parsed = parsed_action_uses(&doc);
    let lines = scanned_action_lines(text);
    if parsed.len() != lines.len() {
        return Err(format!(
            "parser found {} action refs but the line scan matched {}",
            parsed.len(),
            lines.len()
        ));
    }
    for (value, line) in parsed.into_iter().zip(&lines) {
        let n = line.no;
        let spec = value
            .as_str()
            .ok_or_else(|| format!("line {n}: uses must be a string, got {value:?}"))?;
        if unquote(&line.value) != spec {
            return Err(format!(
                "line {n}: source `{}` does not match parsed `{spec}`",
                line.value
            ));
        }
        if spec.starts_with("./") || spec.starts_with("docker://") {
            if ALLOWED_NON_ACTION_REFS.contains(&spec) {
                continue;
            }
            return Err(format!(
                "line {n}: uses: {spec} is a local or docker ref and is not allowed"
            ));
        }
        let (action, git_ref) = split_action(spec).ok_or_else(|| {
            format!("line {n}: uses: {spec} must be owner/repo[/path]@<40-hex SHA>")
        })?;
        if !is_full_sha(git_ref) {
            return Err(format!(
                "line {n}: uses: {spec} must be pinned to a 40-hex SHA"
            ));
        }
        let comment = line.comment.as_deref().unwrap_or("");
        if action == RUST_TOOLCHAIN_ACTION {
            if git_ref != RUST_TOOLCHAIN_SHA {
                return Err(format!(
                    "line {n}: uses: {spec} must use the recorded rust-toolchain commit {RUST_TOOLCHAIN_SHA}"
                ));
            }
            if comment != RUST_TOOLCHAIN_COMMENT {
                return Err(format!(
                    "line {n}: rust-toolchain pin comment must be `# {RUST_TOOLCHAIN_COMMENT}`, got `# {comment}`"
                ));
            }
        } else if !is_release_tag(comment) {
            return Err(format!(
                "line {n}: pinned action needs a release tag comment (vX.Y.Z), got `# {comment}`"
            ));
        }
    }
    Ok(lines.len())
}

/// For each rust-toolchain step, its own `with.toolchain` must be a string
/// equal to `channel` or the MSRV. Returns the toolchains in document order.
fn check_toolchain_steps(text: &str, channel: &str) -> Result<Vec<String>, String> {
    let doc = parse_workflow(text)?;
    let mut seen = Vec::new();
    for job in jobs(&doc) {
        let steps = job.get("steps").and_then(Value::as_sequence);
        for step in steps.into_iter().flatten() {
            let spec = step.get("uses").and_then(Value::as_str).unwrap_or("");
            let is_toolchain = split_action(spec).is_some_and(|(a, _)| a == RUST_TOOLCHAIN_ACTION);
            if !is_toolchain {
                continue;
            }
            let toolchain = step
                .get("with")
                .and_then(|with| with.get("toolchain"))
                .ok_or_else(|| format!("step `{spec}` must set with.toolchain"))?;
            let toolchain = toolchain
                .as_str()
                .ok_or_else(|| format!("step `{spec}` with.toolchain must be a string"))?;
            if toolchain != channel && toolchain != MSRV {
                return Err(format!(
                    "step `{spec}` with.toolchain must be {channel} or {MSRV}, got {toolchain}"
                ));
            }
            seen.push(toolchain.to_string());
        }
    }
    Ok(seen)
}

fn toolchain_channel() -> &'static str {
    TOOLCHAIN_FILE
        .lines()
        .find_map(|l| l.trim().strip_prefix("channel ="))
        .map(|v| v.trim().trim_matches('"'))
        .expect("rust-toolchain.toml sets channel")
}

#[test]
fn ci_third_party_actions_are_sha_pinned() {
    let uses = check_action_pins(WORKFLOW).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        uses >= 8,
        "expected every job to declare a uses: pin, found {uses}"
    );
}

#[test]
fn rust_toolchain_steps_set_an_exact_toolchain() {
    let channel = toolchain_channel();
    let seen = check_toolchain_steps(WORKFLOW, channel).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        seen.iter().any(|t| t == channel) && seen.iter().any(|t| t == MSRV),
        "expected rust-toolchain steps for {channel} and {MSRV}, found {seen:?}"
    );
}

mod controls {
    use super::*;

    const OLD_SHA: &str = "16c20dfc2b8bc5cd72c3bf5072a9453664776e70";
    const OTHER_SHA: &str = "516c5a67e611cdaf2ef0bb5d78620e3c7d245fe6";
    const CACHE_SHA: &str = "0057852bfaa89a56745cba8c7296529d2fc39830";

    fn workflow(steps: &str) -> String {
        format!("name: t\njobs:\n  a:\n    runs-on: ubuntu-latest\n    steps:\n{steps}")
    }

    fn good_step() -> String {
        format!(
            "      - uses: dtolnay/rust-toolchain@{RUST_TOOLCHAIN_SHA} # {RUST_TOOLCHAIN_COMMENT}\n        with:\n          toolchain: 1.98.0\n"
        )
    }

    fn expect_err(result: Result<impl std::fmt::Debug, String>, needle: &str) {
        let err = result.expect_err("control must fail");
        assert!(err.contains(needle), "unexpected error: {err}");
    }

    fn expect_pins(text: &str, count: usize) {
        assert_eq!(check_action_pins(text), Ok(count), "workflow:\n{text}");
    }

    #[test]
    fn valid_control_passes() {
        let text = workflow(&good_step());
        expect_pins(&text, 1);
        assert_eq!(
            check_toolchain_steps(&text, "1.98.0"),
            Ok(vec!["1.98.0".to_string()])
        );
    }

    #[test]
    fn quoted_uses_key_fails_style() {
        let text = workflow(&format!(
            "      - \"uses\": dtolnay/rust-toolchain@{OLD_SHA} # {RUST_TOOLCHAIN_COMMENT}\n        with:\n          toolchain: 1.98.0\n"
        ));
        expect_err(check_action_pins(&text), "quoted `uses` key");
    }

    #[test]
    fn quoted_uses_value_with_wrong_comment_fails() {
        let text = workflow(&format!(
            "      - uses: \"dtolnay/rust-toolchain@{RUST_TOOLCHAIN_SHA}\" # 1.98.0\n        with:\n          toolchain: 1.98.0\n"
        ));
        expect_err(check_action_pins(&text), "pin comment must be");
    }

    #[test]
    fn flow_style_steps_fail_style() {
        for step in [
            "      - {uses: actions/checkout@v4}\n".to_string(),
            format!("      - {{uses: actions/checkout@{CACHE_SHA}}} # v4.3.0\n"),
        ] {
            let text = workflow(&format!("{}{step}", good_step()));
            expect_err(check_action_pins(&text), "flow-style");
        }
        let text = format!(
            "jobs:\n  a:\n    runs-on: x\n    steps: [{{uses: actions/checkout@{CACHE_SHA}}}]\n"
        );
        expect_err(check_action_pins(&text), "flow-style");
    }

    #[test]
    fn anchors_aliases_and_merge_keys_fail_style() {
        let anchor = workflow(&format!(
            "      - &tc\n        uses: dtolnay/rust-toolchain@{RUST_TOOLCHAIN_SHA} # {RUST_TOOLCHAIN_COMMENT}\n        with:\n          toolchain: 1.98.0\n      - *tc\n"
        ));
        expect_err(check_action_pins(&anchor), "anchors are not allowed");
        let alias = format!(
            "x: &pin dtolnay/rust-toolchain@{OTHER_SHA}\njobs:\n  a:\n    runs-on: x\n    steps:\n      - uses: *pin\n"
        );
        expect_err(check_action_pins(&alias), "anchors are not allowed");
        let alias_only = workflow(&format!("{}      - uses: *pin\n", good_step()));
        expect_err(check_action_pins(&alias_only), "aliases are not allowed");
        let merge = workflow(&format!(
            "{}      - <<: {{uses: actions/checkout@{CACHE_SHA}}}\n",
            good_step()
        ));
        expect_err(check_action_pins(&merge), "merge keys");
    }

    #[test]
    fn mixed_case_owner_with_other_sha_fails() {
        let text = workflow(&format!(
            "      - uses: Dtolnay/rust-toolchain@{OTHER_SHA} # {RUST_TOOLCHAIN_COMMENT}\n        with:\n          toolchain: 1.98.0\n"
        ));
        expect_err(check_action_pins(&text), "recorded rust-toolchain commit");
    }

    #[test]
    fn stable_toolchain_with_env_decoy_fails() {
        let text = workflow(&format!(
            "      - uses: dtolnay/rust-toolchain@{RUST_TOOLCHAIN_SHA} # {RUST_TOOLCHAIN_COMMENT}\n        with: {{toolchain: stable}}\n        env: {{toolchain: 1.98.0}}\n"
        ));
        expect_pins(&text, 1);
        expect_err(check_toolchain_steps(&text, "1.98.0"), "got stable");
    }

    #[test]
    fn missing_with_toolchain_fails() {
        let text = workflow(&format!(
            "      - uses: dtolnay/rust-toolchain@{RUST_TOOLCHAIN_SHA} # {RUST_TOOLCHAIN_COMMENT}\n        env:\n          toolchain: 1.98.0\n"
        ));
        expect_err(
            check_toolchain_steps(&text, "1.98.0"),
            "must set with.toolchain",
        );
    }

    #[test]
    fn local_and_docker_refs_fail() {
        for spec in ["./.github/actions/setup", "docker://alpine:3"] {
            let text = workflow(&format!("      - uses: {spec} # v1\n"));
            expect_err(check_action_pins(&text), "not allowed");
        }
    }

    #[test]
    fn uses_keys_outside_action_positions_pass() {
        let text = workflow(&format!(
            "{}      - name: report\n        env: {{uses: report}}\n        run: |\n          echo start\n          uses: report\n          - uses: other\n        working-directory: .\n      - name: block env\n        env:\n          uses: report\n        run: echo ok\n",
            good_step()
        ));
        expect_pins(&text, 1);
    }

    #[test]
    fn action_subpath_and_reusable_workflow_pass() {
        let text = format!(
            "jobs:\n  a:\n    runs-on: x\n    steps:\n      - uses: actions/cache/save@{CACHE_SHA} # v4.3.0\n        with:\n          path: |\n            target\n          key: k\n  b:\n    uses: octo-org/shared/.github/workflows/ci.yml@{CACHE_SHA} # v1.2.0\n    with:\n      uses: input\n"
        );
        expect_pins(&text, 2);
    }

    #[test]
    fn reusable_workflow_wrong_comment_fails() {
        let text = format!(
            "jobs:\n  b:\n    uses: octo-org/shared/.github/workflows/ci.yml@{CACHE_SHA} # main\n"
        );
        expect_err(check_action_pins(&text), "release tag comment");
    }
}

#[test]
fn estate_matrix_uses_standard_hosted_labels() {
    let workflow = WORKFLOW;
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
