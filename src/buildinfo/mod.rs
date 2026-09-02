//! Host binary identity (`version` / `version --extended`).
//!
//! This module resolves facts about the **running product binary** (version,
//! commit, build date, dirty, runtime, platform). It does **not** treat
//! rsfulmen's git or the synced Crucible commit as the host commit.
//!
//! | Identity | Owner |
//! |----------|--------|
//! | Host binary | App injects stamps; this module resolves and formats |
//! | Crate / Crucible pins | [`crate::VERSION`] / [`crate::CRUCIBLE_VERSION`] via [`Pins`] |
//!
//! Compile-time `FULMEN_HOST_*` `rustc-env` values are stamped by the **application**
//! `build.rs`, not by this crate. Use [`host_identity`](crate::host_identity) from
//! the binary crate so `option_env!` is evaluated at the app's compile time.
//! [`resolve_from_stamps`] is **compile-stamp-only**: missing stamps use documented
//! defaults and never read process `FULMEN_HOST_*`. Runtime environment resolution
//! is only available via [`resolve_from_process_env`].
//!
//! Host identity is diagnostics only. Do not use it for authentication,
//! authorization, or integrity.
//!
//! # Application `build.rs`
//!
//! Enable the `host-identity-producer` feature in `[build-dependencies]` and call
//! [`producer::emit_host_identity`]. That probe reads Git from
//! `CARGO_MANIFEST_DIR` and strips Git locator/index/object/config overrides
//! (`GIT_DIR`, `GIT_WORK_TREE`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, …). Then:
//!
//! ```rust
//! let info = rsfulmen::host_identity!();
//! println!("{}", info.format_basic("mycli"));
//! println!("{}", info.format_extended("mycli", Some(&rsfulmen::buildinfo::Pins::from_crate())));
//! ```

use std::env;

#[cfg(feature = "host-identity-producer")]
#[cfg_attr(docsrs, doc(cfg(feature = "host-identity-producer")))]
pub mod producer;

const ENV_VERSION: &str = "FULMEN_HOST_VERSION";
const ENV_COMMIT: &str = "FULMEN_HOST_COMMIT";
const ENV_BUILD_DATE: &str = "FULMEN_HOST_BUILD_DATE";
const ENV_DIRTY: &str = "FULMEN_HOST_DIRTY";
const ENV_RUNTIME: &str = "FULMEN_HOST_RUNTIME";
const ENV_PLATFORM: &str = "FULMEN_HOST_PLATFORM";

const DEFAULT_VERSION: &str = "dev";
const DEFAULT_UNKNOWN: &str = "unknown";

/// SDK / SSOT versions the binary was built against. Extended output only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pins {
    /// rsfulmen crate version linked into the binary.
    pub rsfulmen: String,
    /// Crucible SSOT version embedded in that rsfulmen build.
    pub crucible: String,
}

impl Pins {
    /// Pins for the rsfulmen crate compiling this call (not the host git).
    pub fn from_crate() -> Self {
        Self {
            rsfulmen: crate::VERSION.to_string(),
            crucible: crate::CRUCIBLE_VERSION.to_string(),
        }
    }
}

/// Host binary identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildInfo {
    /// App/binary version. Placeholder: `dev`.
    pub version: String,
    /// Host source commit. Placeholder: `unknown`. Prefer full SHA internally.
    pub commit: String,
    /// RFC3339 UTC build timestamp. Placeholder: `unknown`.
    pub build_date: String,
    /// `Some(true/false)` when known; `None` when unknown (never a false clean).
    pub dirty: Option<bool>,
    /// Language runtime, e.g. `rustc 1.98.0`. Empty means omit.
    pub runtime: String,
    /// Platform, Go-form `OS/ARCH` when derived (e.g. `darwin/arm64`). Empty means omit.
    pub platform: String,
}

/// Resolve host identity from compile-time stamps only.
///
/// Stamp arguments are typically `option_env!("FULMEN_HOST_*")` from the
/// **calling** crate via [`crate::host_identity`]. Empty or missing stamps use
/// documented defaults (`dev` / `unknown` / omitted dirty / empty runtime and
/// platform). This function **never** reads process `FULMEN_HOST_*` and does
/// **not** derive platform from the running host.
pub fn resolve_from_stamps(
    version: Option<&str>,
    commit: Option<&str>,
    build_date: Option<&str>,
    dirty: Option<&str>,
    runtime: Option<&str>,
    platform: Option<&str>,
) -> BuildInfo {
    BuildInfo {
        version: stamp_or_default(version, DEFAULT_VERSION),
        commit: stamp_or_default(commit, DEFAULT_UNKNOWN),
        build_date: stamp_or_default(build_date, DEFAULT_UNKNOWN),
        dirty: parse_dirty(dirty),
        runtime: stamp_or_default(runtime, ""),
        platform: stamp_or_default(platform, ""),
    }
}

/// Documented defaults with no compile stamps and no process environment.
///
/// Prefer [`crate::host_identity`] in application crates. For an explicit
/// runtime-environment read, use [`resolve_from_process_env`].
pub fn resolve() -> BuildInfo {
    resolve_from_stamps(None, None, None, None, None, None)
}

/// Resolve from process `FULMEN_HOST_*` only.
///
/// This is the intentionally runtime-oriented API. Launchers can influence the
/// result. Prefer [`crate::host_identity`] for compiled binary identity.
pub fn resolve_from_process_env() -> BuildInfo {
    BuildInfo {
        version: env_or_default(ENV_VERSION, DEFAULT_VERSION),
        commit: env_or_default(ENV_COMMIT, DEFAULT_UNKNOWN),
        build_date: env_or_default(ENV_BUILD_DATE, DEFAULT_UNKNOWN),
        dirty: parse_dirty(env_get(ENV_DIRTY).as_deref()),
        runtime: env_or_default(ENV_RUNTIME, ""),
        platform: env_or_default(ENV_PLATFORM, ""),
    }
}

/// Resolve from explicit override strings (unit-test and injection helper).
///
/// Empty strings use documented defaults. Process environment is not consulted.
pub fn resolve_with_overrides(
    version: impl AsRef<str>,
    commit: impl AsRef<str>,
    build_date: impl AsRef<str>,
    dirty: impl AsRef<str>,
) -> BuildInfo {
    resolve_from_stamps(
        Some(version.as_ref()),
        Some(commit.as_ref()),
        Some(build_date.as_ref()),
        Some(dirty.as_ref()),
        None,
        None,
    )
}

impl BuildInfo {
    /// Short line: `name version` (root `--version` / bare `version`).
    pub fn format_basic(&self, binary_name: &str) -> String {
        format!("{} {}", binary_name, self.version)
    }

    /// Full host block plus optional pins. Host `Commit:` is never a pin commit.
    pub fn format_extended(&self, binary_name: &str, pins: Option<&Pins>) -> String {
        let mut out = String::new();
        out.push_str(&self.format_basic(binary_name));
        out.push('\n');
        out.push_str("Commit:     ");
        out.push_str(&short_commit(&self.commit));
        out.push('\n');
        out.push_str("BuildDate:  ");
        out.push_str(&self.build_date);
        out.push('\n');
        if let Some(d) = self.dirty {
            out.push_str("Dirty:      ");
            out.push_str(if d { "true" } else { "false" });
            out.push('\n');
        }
        if !self.runtime.is_empty() {
            out.push_str("Runtime:    ");
            out.push_str(&self.runtime);
            out.push('\n');
        }
        if !self.platform.is_empty() {
            out.push_str("Platform:   ");
            out.push_str(&self.platform);
            out.push('\n');
        }
        if let Some(pins) = pins {
            out.push('\n');
            out.push_str("Pins:\n");
            out.push_str("  rsfulmen    ");
            out.push_str(&pins.rsfulmen);
            out.push('\n');
            out.push_str("  Crucible    ");
            out.push_str(&pins.crucible);
            out.push('\n');
        }
        out
    }

    /// Structured identity. `dirty` omitted when unknown. `pins` only when provided.
    pub fn to_json(&self, binary_name: &str, pins: Option<&Pins>) -> String {
        let mut out = String::from("{");
        out.push_str("\"name\":");
        out.push_str(&json_escape(binary_name));
        out.push_str(",\"version\":");
        out.push_str(&json_escape(&self.version));
        out.push_str(",\"commit\":");
        out.push_str(&json_escape(&self.commit));
        out.push_str(",\"buildDate\":");
        out.push_str(&json_escape(&self.build_date));
        if let Some(d) = self.dirty {
            out.push_str(",\"dirty\":");
            out.push_str(if d { "true" } else { "false" });
        }
        if !self.runtime.is_empty() {
            out.push_str(",\"runtime\":");
            out.push_str(&json_escape(&self.runtime));
        }
        if !self.platform.is_empty() {
            out.push_str(",\"platform\":");
            out.push_str(&json_escape(&self.platform));
        }
        if let Some(pins) = pins {
            out.push_str(",\"pins\":{");
            out.push_str("\"rsfulmen\":");
            out.push_str(&json_escape(&pins.rsfulmen));
            out.push_str(",\"crucible\":");
            out.push_str(&json_escape(&pins.crucible));
            out.push('}');
        }
        out.push('}');
        out
    }
}

fn env_get(key: &str) -> Option<String> {
    env::var(key).ok()
}

fn stamp_or_default(stamp: Option<&str>, default: &str) -> String {
    if let Some(s) = stamp {
        let t = s.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    default.to_string()
}

fn env_or_default(env_key: &str, default: &str) -> String {
    if let Some(v) = env_get(env_key) {
        let t = v.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    default.to_string()
}

fn parse_dirty(raw: Option<&str>) -> Option<bool> {
    let s = raw?.trim();
    if s.is_empty() {
        return None;
    }
    match s {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Map Cargo / `std::env::consts` OS+ARCH to Go-form `OS/ARCH`.
#[cfg(feature = "host-identity-producer")]
pub(crate) fn go_form_os_arch(os: &str, arch: &str) -> String {
    let os = match os {
        "macos" => "darwin",
        other => other,
    };
    let arch = match arch {
        "aarch64" => "arm64",
        "x86_64" => "amd64",
        other => other,
    };
    format!("{os}/{arch}")
}

fn short_commit(commit: &str) -> String {
    if commit == DEFAULT_UNKNOWN {
        return commit.to_string();
    }
    if commit.len() >= 7 && commit.chars().take(7).all(|c| c.is_ascii_hexdigit()) {
        commit.chars().take(7).collect()
    } else {
        commit.to_string()
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Capture `FULMEN_HOST_*` rustc-env stamps from the **calling crate**.
#[macro_export]
macro_rules! host_identity {
    () => {
        $crate::buildinfo::resolve_from_stamps(
            ::core::option_env!("FULMEN_HOST_VERSION"),
            ::core::option_env!("FULMEN_HOST_COMMIT"),
            ::core::option_env!("FULMEN_HOST_BUILD_DATE"),
            ::core::option_env!("FULMEN_HOST_DIRTY"),
            ::core::option_env!("FULMEN_HOST_RUNTIME"),
            ::core::option_env!("FULMEN_HOST_PLATFORM"),
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_clean_env<F: FnOnce()>(f: F) {
        let _g = ENV_LOCK.lock().expect("env lock");
        for k in [
            ENV_VERSION,
            ENV_COMMIT,
            ENV_BUILD_DATE,
            ENV_DIRTY,
            ENV_RUNTIME,
            ENV_PLATFORM,
        ] {
            // SAFETY: ENV_LOCK serializes env mutation in these tests.
            unsafe { env::remove_var(k) };
        }
        f();
        for k in [
            ENV_VERSION,
            ENV_COMMIT,
            ENV_BUILD_DATE,
            ENV_DIRTY,
            ENV_RUNTIME,
            ENV_PLATFORM,
        ] {
            unsafe { env::remove_var(k) };
        }
    }

    #[test]
    fn t1_all_overrides_set() {
        with_clean_env(|| {
            let info =
                resolve_with_overrides("1.2.3", "abcdef1234567890", "2026-08-20T12:00:00Z", "true");
            assert_eq!(info.version, "1.2.3");
            assert_eq!(info.commit, "abcdef1234567890");
            assert_eq!(info.build_date, "2026-08-20T12:00:00Z");
            assert_eq!(info.dirty, Some(true));
        });
    }

    #[test]
    fn t2_missing_env_defaults() {
        with_clean_env(|| {
            let info = resolve();
            assert_eq!(info.version, "dev");
            assert_eq!(info.commit, "unknown");
            assert_eq!(info.build_date, "unknown");
            assert_eq!(info.dirty, None);
            assert!(info.platform.is_empty());
        });
    }

    #[test]
    fn t3_dirty_true() {
        with_clean_env(|| {
            let info = resolve_with_overrides("1.0.0", "abc", "2026-01-01T00:00:00Z", "true");
            assert_eq!(info.dirty, Some(true));
        });
    }

    #[test]
    fn t4_dirty_false() {
        with_clean_env(|| {
            let info = resolve_with_overrides("1.0.0", "abc", "2026-01-01T00:00:00Z", "false");
            assert_eq!(info.dirty, Some(false));
        });
    }

    #[test]
    fn t5_invalid_dirty_is_unknown() {
        with_clean_env(|| {
            let info = resolve_with_overrides("1.0.0", "abc", "2026-01-01T00:00:00Z", "yes");
            assert_eq!(info.dirty, None);
        });
    }

    #[test]
    fn invalid_stamp_does_not_become_runtime_false_clean() {
        with_clean_env(|| {
            // SAFETY: ENV_LOCK serializes env mutation in these tests.
            unsafe {
                env::set_var(ENV_DIRTY, "false");
            }
            let info = resolve_from_stamps(
                Some("1.0.0"),
                Some("abc"),
                Some("2026-01-01T00:00:00Z"),
                Some("yes"),
                None,
                None,
            );
            assert_eq!(info.dirty, None);
            let text = info.format_extended("tool", None);
            assert!(
                !text.contains("Dirty:"),
                "unknown dirty must be omitted from extended text"
            );
            let json = info.to_json("tool", None);
            assert!(
                !json.contains("dirty"),
                "unknown dirty must be omitted from JSON"
            );
        });
    }

    #[test]
    fn t6_format_basic() {
        with_clean_env(|| {
            let info = resolve_with_overrides("0.4.16", "deadbeef", "unknown", "");
            assert_eq!(info.format_basic("tool-name"), "tool-name 0.4.16");
            assert!(!info.format_basic("tool-name").contains("Commit"));
        });
    }

    #[test]
    fn t7_format_extended_includes_host_fields() {
        with_clean_env(|| {
            let mut info =
                resolve_with_overrides("0.4.16", "a1b2c3d4e5f6", "2026-08-07T12:00:00Z", "true");
            info.runtime = "rustc 1.98.0".into();
            info.platform = "darwin/arm64".into();
            let text = info.format_extended("tool-name", None);
            assert!(text.starts_with("tool-name 0.4.16\n"));
            assert!(text.contains("Commit:     a1b2c3d"));
            assert!(text.contains("BuildDate:  2026-08-07T12:00:00Z"));
            assert!(text.contains("Dirty:      true"));
            assert!(text.contains("Runtime:    rustc 1.98.0"));
            assert!(text.contains("Platform:   darwin/arm64"));
            assert!(!text.contains("Pins:"));
        });
    }

    #[test]
    fn t8_format_extended_pins_are_a_second_block() {
        with_clean_env(|| {
            let info =
                resolve_with_overrides("0.4.16", "aaaaaaaa", "2026-08-07T12:00:00Z", "false");
            let pins = Pins {
                rsfulmen: "0.1.5".into(),
                crucible: "v0.4.19".into(),
            };
            let text = info.format_extended("tool-name", Some(&pins));
            assert!(text.contains("Pins:\n  rsfulmen    0.1.5\n  Crucible    v0.4.19\n"));
            let commit_line = text
                .lines()
                .find(|l| l.starts_with("Commit:"))
                .expect("commit line");
            assert!(commit_line.contains("aaaaaaa"));
            assert!(!commit_line.contains("v0.4.19"));
        });
    }

    #[test]
    fn t9_empty_string_overrides_fall_through() {
        with_clean_env(|| {
            let info = resolve_with_overrides("  ", "  ", "", "");
            assert_eq!(info.version, "dev");
            assert_eq!(info.commit, "unknown");
            assert_eq!(info.build_date, "unknown");
            assert_eq!(info.dirty, None);
        });
    }

    #[test]
    fn t10_commit_shortening_in_text_keeps_full_in_json() {
        with_clean_env(|| {
            let full = "a1b2c3d4e5f6789012345678901234567890abcd";
            let info = resolve_with_overrides("1.0.0", full, "unknown", "");
            let text = info.format_extended("tool", None);
            assert!(text.contains("Commit:     a1b2c3d\n"));
            let json = info.to_json("tool", None);
            assert!(json.contains(&format!("\"commit\":\"{full}\"")));
        });
    }

    #[test]
    fn t11_pins_do_not_become_host_commit() {
        with_clean_env(|| {
            let info = resolve_with_overrides("1.0.0", "hostc0ffee", "unknown", "");
            let pins = Pins::from_crate();
            assert_eq!(pins.rsfulmen, crate::VERSION);
            assert_eq!(pins.crucible, crate::CRUCIBLE_VERSION);
            let text = info.format_extended("tool", Some(&pins));
            let commit_line = text.lines().find(|l| l.starts_with("Commit:")).unwrap();
            assert!(commit_line.contains("hostc0f"));
            assert!(
                !commit_line.contains(&pins.crucible),
                "Crucible SSOT must not appear as host Commit"
            );
            assert!(text.contains("Pins:"));
            assert!(text.contains("Crucible"));
        });
    }

    #[test]
    fn t12_no_default_features_still_has_resolve() {
        with_clean_env(|| {
            let _ = resolve();
        });
    }

    #[test]
    fn json_omits_unknown_dirty() {
        with_clean_env(|| {
            let info = resolve_with_overrides("1.0.0", "abc", "unknown", "");
            let json = info.to_json("tool", None);
            assert!(!json.contains("dirty"));
            assert!(json.contains("\"name\":\"tool\""));
            assert!(json.contains("\"buildDate\":\"unknown\""));
        });
    }

    #[test]
    fn runtime_env_fills_only_via_explicit_process_api() {
        with_clean_env(|| {
            unsafe {
                env::set_var(ENV_VERSION, "9.9.9");
                env::set_var(ENV_COMMIT, "eeeeeee");
                env::set_var(ENV_DIRTY, "false");
                env::set_var(ENV_PLATFORM, "linux/forge");
            }
            let info = resolve_from_process_env();
            assert_eq!(info.version, "9.9.9");
            assert_eq!(info.commit, "eeeeeee");
            assert_eq!(info.dirty, Some(false));
            assert_eq!(info.platform, "linux/forge");
        });
    }

    #[test]
    fn stamps_ignore_poisoned_process_env() {
        with_clean_env(|| {
            unsafe {
                env::set_var(ENV_VERSION, "poison-ver");
                env::set_var(ENV_COMMIT, "poison-commit");
                env::set_var(ENV_BUILD_DATE, "poison-date");
                env::set_var(ENV_DIRTY, "true");
                env::set_var(ENV_RUNTIME, "poison-runtime");
                env::set_var(ENV_PLATFORM, "linux/forge");
            }
            let info = resolve_from_stamps(
                Some("1.0.0"),
                Some("abcdef1234567890"),
                Some("2026-01-01T00:00:00Z"),
                Some("false"),
                Some("rustc 1.98.0"),
                Some("darwin/arm64"),
            );
            assert_eq!(info.version, "1.0.0");
            assert_eq!(info.commit, "abcdef1234567890");
            assert_eq!(info.build_date, "2026-01-01T00:00:00Z");
            assert_eq!(info.dirty, Some(false));
            assert_eq!(info.runtime, "rustc 1.98.0");
            assert_eq!(info.platform, "darwin/arm64");
        });
    }

    #[test]
    fn absent_stamps_do_not_take_process_env() {
        with_clean_env(|| {
            unsafe {
                env::set_var(ENV_VERSION, "poison-ver");
                env::set_var(ENV_COMMIT, "poison-commit");
                env::set_var(ENV_PLATFORM, "linux/forge");
                env::set_var(ENV_DIRTY, "false");
            }
            let info = resolve_from_stamps(None, None, None, None, None, None);
            assert_eq!(info.version, "dev");
            assert_eq!(info.commit, "unknown");
            assert_eq!(info.build_date, "unknown");
            assert_eq!(info.dirty, None);
            assert!(info.platform.is_empty());
            let json = info.to_json("tool", None);
            assert!(!json.contains("poison"));
            assert!(!json.contains("linux/forge"));
            assert!(!json.contains("\"dirty\""));
        });
    }

    #[test]
    fn host_identity_macro_ignores_runtime_poison() {
        with_clean_env(|| {
            unsafe {
                env::set_var(ENV_VERSION, "poison-ver");
                env::set_var(ENV_COMMIT, "poison-commit");
                env::set_var(ENV_PLATFORM, "linux/forge");
            }
            let poisoned = crate::host_identity!();
            let clean = resolve_from_stamps(None, None, None, None, None, None);
            assert_eq!(poisoned.to_json("tool", None), clean.to_json("tool", None));
            assert!(!poisoned.to_json("tool", None).contains("poison"));
            assert!(!poisoned.to_json("tool", None).contains("linux/forge"));
        });
    }
}
