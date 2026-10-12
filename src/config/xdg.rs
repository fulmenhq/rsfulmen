//! Platform config/data/cache directory resolution without the `dirs` crate.
//!
//! Implements the [Fulmen Config Path Standard](https://github.com/fulmenhq/crucible)
//! using only `std`. XDG environment variables win when they are set to an
//! absolute path; otherwise each platform uses the documented fallback.

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::XdgBaseDirs;

/// Resolve XDG-style base directories from the process environment.
pub(crate) fn from_process_env() -> XdgBaseDirs {
    from_env(|key| env::var_os(key), home_dir())
}

/// Resolve base directories from an env lookup and an optional home.
///
/// `get_env` is injected so tests can cover overrides without mutating the
/// process environment (which would race other tests).
pub(crate) fn from_env<F>(get_env: F, home: Option<PathBuf>) -> XdgBaseDirs
where
    F: Fn(&str) -> Option<OsString>,
{
    XdgBaseDirs {
        config_home: first_dir(
            xdg_override(get_env("XDG_CONFIG_HOME")),
            platform_config_home(&get_env, home.as_deref()),
        ),
        data_home: first_dir(
            xdg_override(get_env("XDG_DATA_HOME")),
            platform_data_home(&get_env, home.as_deref()),
        ),
        cache_home: first_dir(
            xdg_override(get_env("XDG_CACHE_HOME")),
            platform_cache_home(&get_env, home.as_deref()),
        ),
    }
}

fn first_dir(preferred: Option<PathBuf>, fallback: Option<PathBuf>) -> PathBuf {
    preferred.or(fallback).unwrap_or_else(|| PathBuf::from("."))
}

/// XDG Base Directory spec: empty or relative values are ignored.
fn xdg_override(value: Option<OsString>) -> Option<PathBuf> {
    let path = PathBuf::from(value.filter(|v| !v.is_empty())?);
    path.is_absolute().then_some(path)
}

fn platform_config_home<F>(get_env: &F, home: Option<&Path>) -> Option<PathBuf>
where
    F: Fn(&str) -> Option<OsString>,
{
    #[cfg(windows)]
    {
        let _ = home;
        windows_roaming(get_env, home)
    }
    #[cfg(target_os = "macos")]
    {
        let _ = get_env;
        home.map(|h| h.join("Library").join("Application Support"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = get_env;
        home.map(|h| h.join(".config"))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = get_env;
        home.map(|h| h.join(".config"))
    }
}

fn platform_data_home<F>(get_env: &F, home: Option<&Path>) -> Option<PathBuf>
where
    F: Fn(&str) -> Option<OsString>,
{
    #[cfg(windows)]
    {
        windows_roaming(get_env, home)
    }
    #[cfg(target_os = "macos")]
    {
        let _ = get_env;
        home.map(|h| h.join("Library").join("Application Support"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = get_env;
        home.map(|h| h.join(".local").join("share"))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = get_env;
        home.map(|h| h.join(".local").join("share"))
    }
}

fn platform_cache_home<F>(get_env: &F, home: Option<&Path>) -> Option<PathBuf>
where
    F: Fn(&str) -> Option<OsString>,
{
    #[cfg(windows)]
    {
        windows_local(get_env, home)
    }
    #[cfg(target_os = "macos")]
    {
        let _ = get_env;
        home.map(|h| h.join("Library").join("Caches"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = get_env;
        home.map(|h| h.join(".cache"))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = get_env;
        home.map(|h| h.join(".cache"))
    }
}

#[cfg(windows)]
fn windows_roaming<F>(get_env: &F, home: Option<&Path>) -> Option<PathBuf>
where
    F: Fn(&str) -> Option<OsString>,
{
    env_path(get_env("APPDATA")).or_else(|| home.map(|h| h.join("AppData").join("Roaming")))
}

#[cfg(windows)]
fn windows_local<F>(get_env: &F, home: Option<&Path>) -> Option<PathBuf>
where
    F: Fn(&str) -> Option<OsString>,
{
    env_path(get_env("LOCALAPPDATA")).or_else(|| home.map(|h| h.join("AppData").join("Local")))
}

#[cfg(windows)]
fn env_path(value: Option<OsString>) -> Option<PathBuf> {
    Some(PathBuf::from(value.filter(|v| !v.is_empty())?))
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or({
            #[cfg(windows)]
            {
                env::var_os("USERPROFILE")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            }
            #[cfg(not(windows))]
            {
                None
            }
        })
        .or_else(env::home_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::Path;

    fn lookup<'a>(map: &'a HashMap<&'a str, &'a str>) -> impl Fn(&str) -> Option<OsString> + 'a {
        move |key| map.get(key).map(|v| OsString::from(*v))
    }

    #[test]
    fn xdg_absolute_overrides_win() {
        let env = HashMap::from([
            ("XDG_CONFIG_HOME", "/tmp/cfg"),
            ("XDG_DATA_HOME", "/tmp/data"),
            ("XDG_CACHE_HOME", "/tmp/cache"),
        ]);
        let dirs = from_env(lookup(&env), Some(PathBuf::from("/home/me")));
        assert_eq!(dirs.config_home, PathBuf::from("/tmp/cfg"));
        assert_eq!(dirs.data_home, PathBuf::from("/tmp/data"));
        assert_eq!(dirs.cache_home, PathBuf::from("/tmp/cache"));
    }

    #[test]
    fn relative_or_empty_xdg_is_ignored() {
        let env = HashMap::from([
            ("XDG_CONFIG_HOME", "relative-config"),
            ("XDG_DATA_HOME", ""),
        ]);
        let dirs = from_env(lookup(&env), Some(PathBuf::from("/home/me")));
        assert!(dirs.config_home.is_absolute() || dirs.config_home == Path::new("."));
        assert_ne!(dirs.config_home, Path::new("relative-config"));
        assert_ne!(dirs.data_home.as_os_str(), "");
    }

    #[test]
    fn missing_home_falls_back_to_dot() {
        let empty: HashMap<&str, &str> = HashMap::new();
        let dirs = from_env(lookup(&empty), None);
        assert_eq!(dirs.config_home, PathBuf::from("."));
        assert_eq!(dirs.data_home, PathBuf::from("."));
        assert_eq!(dirs.cache_home, PathBuf::from("."));
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    #[test]
    fn unix_platform_defaults() {
        let empty: HashMap<&str, &str> = HashMap::new();
        let dirs = from_env(lookup(&empty), Some(PathBuf::from("/home/me")));
        assert_eq!(dirs.config_home, PathBuf::from("/home/me/.config"));
        assert_eq!(dirs.data_home, PathBuf::from("/home/me/.local/share"));
        assert_eq!(dirs.cache_home, PathBuf::from("/home/me/.cache"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_platform_defaults() {
        let empty: HashMap<&str, &str> = HashMap::new();
        let dirs = from_env(lookup(&empty), Some(PathBuf::from("/Users/me")));
        assert_eq!(
            dirs.config_home,
            PathBuf::from("/Users/me/Library/Application Support")
        );
        assert_eq!(
            dirs.data_home,
            PathBuf::from("/Users/me/Library/Application Support")
        );
        assert_eq!(dirs.cache_home, PathBuf::from("/Users/me/Library/Caches"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_env_defaults() {
        let env = HashMap::from([
            ("APPDATA", r"C:\Users\me\AppData\Roaming"),
            ("LOCALAPPDATA", r"C:\Users\me\AppData\Local"),
        ]);
        let dirs = from_env(lookup(&env), Some(PathBuf::from(r"C:\Users\me")));
        assert_eq!(
            dirs.config_home,
            PathBuf::from(r"C:\Users\me\AppData\Roaming")
        );
        assert_eq!(
            dirs.data_home,
            PathBuf::from(r"C:\Users\me\AppData\Roaming")
        );
        assert_eq!(dirs.cache_home, PathBuf::from(r"C:\Users\me\AppData\Local"));
    }

    #[test]
    fn xdg_override_rejects_relative() {
        assert!(xdg_override(Some(OsString::from("rel"))).is_none());
        assert!(xdg_override(Some(OsString::from(""))).is_none());
        assert!(xdg_override(None).is_none());
        #[cfg(unix)]
        assert_eq!(
            xdg_override(Some(OsString::from("/abs"))),
            Some(PathBuf::from("/abs"))
        );
    }
}
