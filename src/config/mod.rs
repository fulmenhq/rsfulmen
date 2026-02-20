//! Configuration path utilities following the Fulmen Config Path Standard.
//!
//! This module provides XDG-compliant configuration, data, and cache directory
//! resolution across Linux, macOS, and Windows platforms.
//!
//! ## Overview
//!
//! The Fulmen Config Path Standard ensures consistent file locations across all
//! Fulmen ecosystem tools and libraries. This module implements the standard
//! defined in Crucible's `docs/standards/config/fulmen-config-paths.md`.
//!
//! ## Platform Defaults
//!
//! | Platform | Config | Data | Cache |
//! |----------|--------|------|-------|
//! | Linux/Unix | `~/.config/fulmen` | `~/.local/share/fulmen` | `~/.cache/fulmen` |
//! | macOS | `~/Library/Application Support/Fulmen` | `~/Library/Application Support/Fulmen` | `~/Library/Caches/Fulmen` |
//! | Windows | `%APPDATA%\Fulmen` | `%APPDATA%\Fulmen` | `%LOCALAPPDATA%\Fulmen\Cache` |
//!
//! ## Example
//!
//! ```rust
//! use rsfulmen::config::{get_fulmen_config_dir, get_app_config_dir};
//!
//! // Get Fulmen ecosystem config directory
//! let fulmen_config = get_fulmen_config_dir();
//!
//! // Get config directory for a custom app
//! let my_app_config = get_app_config_dir("myapp");
//! ```

use std::path::PathBuf;

pub mod env;
#[cfg(feature = "three-layer-config")]
pub mod three_layer;

/// XDG base directories structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XdgBaseDirs {
    /// Configuration home (e.g., `~/.config`)
    pub config_home: PathBuf,
    /// Data home (e.g., `~/.local/share`)
    pub data_home: PathBuf,
    /// Cache home (e.g., `~/.cache`)
    pub cache_home: PathBuf,
}

/// Get XDG base directories, respecting environment overrides.
///
/// Returns platform-appropriate directories:
/// - Linux: Respects `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME`
/// - macOS: Uses `~/Library/Application Support` and `~/Library/Caches`
/// - Windows: Uses `%APPDATA%` and `%LOCALAPPDATA%`
pub fn get_xdg_base_dirs() -> XdgBaseDirs {
    XdgBaseDirs {
        config_home: dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")),
        data_home: dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")),
        cache_home: dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".")),
    }
}

/// Get the configuration directory for an arbitrary application.
///
/// # Arguments
///
/// * `app_name` - The application name (used as subdirectory)
///
/// # Example
///
/// ```rust
/// use rsfulmen::config::get_app_config_dir;
///
/// let config_dir = get_app_config_dir("myapp");
/// // Linux: ~/.config/myapp
/// // macOS: ~/Library/Application Support/myapp
/// // Windows: %APPDATA%\myapp
/// ```
pub fn get_app_config_dir(app_name: &str) -> PathBuf {
    get_xdg_base_dirs().config_home.join(app_name)
}

/// Get the data directory for an arbitrary application.
///
/// # Arguments
///
/// * `app_name` - The application name (used as subdirectory)
pub fn get_app_data_dir(app_name: &str) -> PathBuf {
    get_xdg_base_dirs().data_home.join(app_name)
}

/// Get the cache directory for an arbitrary application.
///
/// # Arguments
///
/// * `app_name` - The application name (used as subdirectory)
pub fn get_app_cache_dir(app_name: &str) -> PathBuf {
    get_xdg_base_dirs().cache_home.join(app_name)
}

/// Get ordered list of configuration search paths.
///
/// Returns paths in priority order (first = highest priority):
/// 1. Current app config directory
/// 2. Legacy app directories (if provided)
///
/// # Arguments
///
/// * `app_name` - The current application name
/// * `legacy_names` - Optional legacy application names to search
pub fn get_app_config_paths(app_name: &str, legacy_names: &[&str]) -> Vec<PathBuf> {
    let mut paths = vec![get_app_config_dir(app_name)];
    for legacy in legacy_names {
        paths.push(get_app_config_dir(legacy));
    }
    paths
}

// Fulmen ecosystem helpers

/// Get the Fulmen ecosystem configuration directory.
///
/// Returns the standard Fulmen config location:
/// - Linux: `~/.config/fulmen`
/// - macOS: `~/Library/Application Support/Fulmen`
/// - Windows: `%APPDATA%\Fulmen`
pub fn get_fulmen_config_dir() -> PathBuf {
    get_app_config_dir("fulmen")
}

/// Get the Fulmen ecosystem data directory.
///
/// Returns the standard Fulmen data location:
/// - Linux: `~/.local/share/fulmen`
/// - macOS: `~/Library/Application Support/Fulmen`
/// - Windows: `%APPDATA%\Fulmen`
pub fn get_fulmen_data_dir() -> PathBuf {
    get_app_data_dir("fulmen")
}

/// Get the Fulmen ecosystem cache directory.
///
/// Returns the standard Fulmen cache location:
/// - Linux: `~/.cache/fulmen`
/// - macOS: `~/Library/Caches/Fulmen`
/// - Windows: `%LOCALAPPDATA%\Fulmen\Cache`
pub fn get_fulmen_cache_dir() -> PathBuf {
    get_app_cache_dir("fulmen")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_xdg_base_dirs() {
        let dirs = get_xdg_base_dirs();
        // Just verify we get some paths back
        assert!(!dirs.config_home.as_os_str().is_empty());
        assert!(!dirs.data_home.as_os_str().is_empty());
        assert!(!dirs.cache_home.as_os_str().is_empty());
    }

    #[test]
    fn test_get_app_config_dir() {
        let path = get_app_config_dir("testapp");
        assert!(path.ends_with("testapp"));
    }

    #[test]
    fn test_get_fulmen_config_dir() {
        let path = get_fulmen_config_dir();
        assert!(path.ends_with("fulmen"));
    }

    #[test]
    fn test_get_app_config_paths() {
        let paths = get_app_config_paths("newapp", &["oldapp", "legacyapp"]);
        assert_eq!(paths.len(), 3);
        assert!(paths[0].ends_with("newapp"));
        assert!(paths[1].ends_with("oldapp"));
        assert!(paths[2].ends_with("legacyapp"));
    }
}
