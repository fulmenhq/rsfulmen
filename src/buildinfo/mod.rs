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
//! [`resolve`] reads process environment as a fallback and is
//! informational / caller-influenceable.
//!
//! Host identity is diagnostics only. Do not use it for authentication,
//! authorization, or integrity.
//!
//! # Application `build.rs` (copy-paste)
//!
//! The application crate (not rsfulmen) should stamp:
//!
//! ```text
//! FULMEN_HOST_VERSION
//! FULMEN_HOST_COMMIT
//! FULMEN_HOST_BUILD_DATE   // RFC3339 UTC
//! FULMEN_HOST_DIRTY        // "true" | "false" | empty = unknown
//! ```
//!
//! See the Host Binary Identity Standard
//! (`docs/crucible-rs/standards/repository-structure/host-binary-identity.md`).
//! A self-contained `build.rs` recipe lives in that standard (Phase A). After
//! stamping, the binary crate calls:
//!
//! ```rust
//! let info = rsfulmen::host_identity!();
//! println!("{}", info.format_basic("mycli"));
//! println!("{}", info.format_extended("mycli", Some(&rsfulmen::buildinfo::Pins::from_crate())));
//! ```

use std::env;

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

/// Resolve host identity from compile-time stamps, then process env, then defaults.
///
/// Stamp arguments are typically `option_env!("FULMEN_HOST_*")` from the
/// **calling** crate via [`crate::host_identity`]. Empty strings are placeholders.
pub fn resolve_from_stamps(
    version: Option<&str>,
    commit: Option<&str>,
    build_date: Option<&str>,
    dirty: Option<&str>,
    runtime: Option<&str>,
    platform: Option<&str>,
) -> BuildInfo {
    let runtime = pick_field(runtime, ENV_RUNTIME, "");
    let platform = pick_field(platform, ENV_PLATFORM, "");
    BuildInfo {
        version: pick_field(version, ENV_VERSION, DEFAULT_VERSION),
        commit: pick_field(commit, ENV_COMMIT, DEFAULT_UNKNOWN),
        build_date: pick_field(build_date, ENV_BUILD_DATE, DEFAULT_UNKNOWN),
        dirty: parse_dirty(dirty).or_else(|| parse_dirty(env_get(ENV_DIRTY).as_deref())),
        runtime,
        platform: if platform.is_empty() {
            go_platform()
        } else {
            platform
        },
    }
}

/// Resolve from process `FULMEN_HOST_*` only (runtime fallback).
///
/// Prefer [`crate::host_identity`] in application crates so compile-time
/// `rustc-env` stamps from the app `build.rs` are visible.
pub fn resolve() -> BuildInfo {
    resolve_from_stamps(None, None, None, None, None, None)
}

/// Resolve from explicit override strings (unit-test and injection helper).
///
/// Empty strings fall through to process env, then defaults.
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

fn pick_field(stamp: Option<&str>, env_key: &str, default: &str) -> String {
    if let Some(s) = stamp {
        let t = s.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
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

fn go_platform() -> String {
    let os = match env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let arch = match env::consts::ARCH {
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
            assert!(!info.platform.is_empty());
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
    fn runtime_env_fills_when_stamps_empty() {
        with_clean_env(|| {
            unsafe {
                env::set_var(ENV_VERSION, "9.9.9");
                env::set_var(ENV_COMMIT, "eeeeeee");
                env::set_var(ENV_DIRTY, "false");
            }
            let info = resolve();
            assert_eq!(info.version, "9.9.9");
            assert_eq!(info.commit, "eeeeeee");
            assert_eq!(info.dirty, Some(false));
        });
    }
}
