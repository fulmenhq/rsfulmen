//! Standard Exit Code Catalog
//!
//! Provides exit code lookups with standard names, descriptions, and categories.
//!
//! ## Example
//!
//! ```rust
//! use rsfulmen::foundry::exit_codes::{lookup_exit_code, ExitCategory, EXIT_SUCCESS, EXIT_FAILURE};
//!
//! assert_eq!(EXIT_SUCCESS, 0);
//! assert_eq!(EXIT_FAILURE, 1);
//!
//! let not_found = lookup_exit_code(404);
//! assert!(not_found.is_none()); // HTTP status != exit code
//!
//! let ok = lookup_exit_code(0).unwrap();
//! assert_eq!(ok.name, "EXIT_SUCCESS");
//! ```

use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

use super::{FoundryError, FoundryResult};

// Standard exit code constants
/// Successful execution (0)
pub const EXIT_SUCCESS: i32 = 0;
/// Generic failure (1)
pub const EXIT_FAILURE: i32 = 1;
/// Port is already in use (10)
pub const EXIT_PORT_IN_USE: i32 = 10;
/// Configuration file invalid (20)
pub const EXIT_CONFIG_INVALID: i32 = 20;
/// Health check failed (30)
pub const EXIT_HEALTH_CHECK_FAILED: i32 = 30;
/// Invalid command-line argument (40)
pub const EXIT_INVALID_ARGUMENT: i32 = 40;
/// Permission denied (50)
pub const EXIT_PERMISSION_DENIED: i32 = 50;
/// Data validation failed (60)
pub const EXIT_DATA_INVALID: i32 = 60;
/// Command usage error - BSD sysexits.h compatible (64)
pub const EXIT_USAGE: i32 = 64;
/// Authentication failed (70)
pub const EXIT_AUTHENTICATION_FAILED: i32 = 70;
/// SIGINT received (130)
pub const EXIT_SIGNAL_INT: i32 = 130;
/// SIGTERM received (143)
pub const EXIT_SIGNAL_TERM: i32 = 143;
/// SIGKILL received (137)
pub const EXIT_SIGNAL_KILL: i32 = 137;

/// Command timed out before completion (124)
pub const EXIT_TIMEOUT: i32 = 124;
/// Timeout utility itself failed (125)
pub const EXIT_TIMEOUT_INTERNAL: i32 = 125;
/// Command found but could not be executed (126)
pub const EXIT_CANNOT_EXECUTE: i32 = 126;
/// Command not found (127)
pub const EXIT_NOT_FOUND: i32 = 127;

/// Exit code category identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExitCategory {
    /// Standard POSIX codes (0-1)
    Standard,
    /// Network and port management (10-19)
    Networking,
    /// Configuration and validation (20-29)
    Configuration,
    /// Runtime errors (30-39)
    Runtime,
    /// Command-line usage errors (40-49, 64)
    Usage,
    /// File and permission errors (50-59)
    Permissions,
    /// Data and parsing errors (60-69)
    Data,
    /// Security and authentication (70-79)
    Security,
    /// Observability and monitoring (80-89)
    Observability,
    /// Testing and validation (91-99)
    Testing,
    /// Shell & process conventions (124-127)
    Shell,
    /// Signal-induced exits (128+)
    Signals,
}

impl ExitCategory {
    /// Get the human-readable name of the category.
    pub fn name(&self) -> &'static str {
        match self {
            ExitCategory::Standard => "Standard Exit Codes",
            ExitCategory::Networking => "Networking & Port Management",
            ExitCategory::Configuration => "Configuration & Validation",
            ExitCategory::Runtime => "Runtime Errors",
            ExitCategory::Usage => "Command-Line Usage Errors",
            ExitCategory::Permissions => "Permissions & File Access",
            ExitCategory::Data => "Data & Processing Errors",
            ExitCategory::Security => "Security & Authentication",
            ExitCategory::Observability => "Observability & Monitoring",
            ExitCategory::Testing => "Testing & Validation",
            ExitCategory::Shell => "Shell & Process Control",
            ExitCategory::Signals => "Signal-Induced Exits",
        }
    }

    /// Determine the category from an exit code value.
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0..=1 => Some(ExitCategory::Standard),
            10..=19 => Some(ExitCategory::Networking),
            20..=29 => Some(ExitCategory::Configuration),
            30..=39 => Some(ExitCategory::Runtime),
            40..=49 | 64 => Some(ExitCategory::Usage), // 64 = BSD EX_USAGE
            50..=59 => Some(ExitCategory::Permissions),
            60..=63 | 65..=69 => Some(ExitCategory::Data), // Excluding 64
            70..=79 => Some(ExitCategory::Security),
            80..=89 => Some(ExitCategory::Observability),
            91..=99 => Some(ExitCategory::Testing),
            124..=127 => Some(ExitCategory::Shell),
            128..=165 => Some(ExitCategory::Signals),
            _ => None,
        }
    }
}

/// Exit code entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitCode {
    /// The numeric exit code
    pub code: i32,
    /// Symbolic name (e.g., "EXIT_SUCCESS")
    pub name: String,
    /// Description
    pub description: String,
    /// The category this code belongs to
    pub category: ExitCategory,
}

// Raw types for YAML parsing
#[derive(Debug, Clone, Deserialize)]
struct RawCode {
    code: i32,
    name: String,
    description: String,
    #[allow(dead_code)]
    context: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawCategory {
    id: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    description: String,
    codes: Vec<RawCode>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExitCodeCatalog {
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    description: String,
    categories: Vec<RawCategory>,
}

/// Embedded exit codes YAML from Crucible.
const EXIT_CODES_YAML: &str =
    include_str!("../../config/crucible-rs/library/foundry/exit-codes.yaml");

/// Lazily initialized exit code indexes.
static CATALOGS: Lazy<ExitCodeIndexes> =
    Lazy::new(|| ExitCodeIndexes::load().expect("Failed to load embedded exit codes catalog"));

/// Pre-computed indexes for fast lookups.
struct ExitCodeIndexes {
    /// All exit codes
    all: Vec<ExitCode>,
    /// Index by code value
    by_code: HashMap<i32, usize>,
}

fn parse_category_id(id: &str) -> Option<ExitCategory> {
    match id {
        "standard" => Some(ExitCategory::Standard),
        "networking" => Some(ExitCategory::Networking),
        "configuration" => Some(ExitCategory::Configuration),
        "runtime" => Some(ExitCategory::Runtime),
        "usage" => Some(ExitCategory::Usage),
        "permissions" => Some(ExitCategory::Permissions),
        "data" => Some(ExitCategory::Data),
        "security" => Some(ExitCategory::Security),
        "observability" => Some(ExitCategory::Observability),
        "testing" => Some(ExitCategory::Testing),
        "shell" => Some(ExitCategory::Shell),
        "signals" => Some(ExitCategory::Signals),
        _ => None,
    }
}

impl ExitCodeIndexes {
    fn load() -> FoundryResult<Self> {
        let catalog: ExitCodeCatalog = serde_yaml::from_str(EXIT_CODES_YAML)
            .map_err(|e| FoundryError::LoadError(format!("Failed to parse exit codes: {}", e)))?;

        let mut all = Vec::new();
        let mut by_code = HashMap::new();

        for cat in catalog.categories {
            let category = parse_category_id(&cat.id)
                .ok_or_else(|| FoundryError::LoadError(format!("Unknown category: {}", cat.id)))?;

            for code in cat.codes {
                let index = all.len();
                all.push(ExitCode {
                    code: code.code,
                    name: code.name,
                    description: code.description,
                    category,
                });
                by_code.insert(code.code, index);
            }
        }

        Ok(Self { all, by_code })
    }
}

/// Look up an exit code by its numeric value.
///
/// # Arguments
///
/// * `code` - The exit code value
///
/// # Returns
///
/// The exit code entry if found, or `None`.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::exit_codes::lookup_exit_code;
///
/// let success = lookup_exit_code(0).unwrap();
/// assert_eq!(success.name, "EXIT_SUCCESS");
/// ```
pub fn lookup_exit_code(code: i32) -> Option<&'static ExitCode> {
    CATALOGS.by_code.get(&code).map(|&i| &CATALOGS.all[i])
}

/// Get the symbolic name for an exit code.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::exit_codes::get_exit_name;
///
/// assert_eq!(get_exit_name(0), Some("EXIT_SUCCESS"));
/// assert_eq!(get_exit_name(1), Some("EXIT_FAILURE"));
/// ```
pub fn get_exit_name(code: i32) -> Option<&'static str> {
    lookup_exit_code(code).map(|e| e.name.as_str())
}

/// List all exit codes in a specific category.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::exit_codes::{list_by_category, ExitCategory};
///
/// let standard = list_by_category(ExitCategory::Standard);
/// assert!(standard.iter().any(|e| e.code == 0));
/// ```
pub fn list_by_category(category: ExitCategory) -> Vec<&'static ExitCode> {
    CATALOGS
        .all
        .iter()
        .filter(|e| e.category == category)
        .collect()
}

/// List all exit codes in the catalog.
pub fn list_exit_codes() -> &'static [ExitCode] {
    &CATALOGS.all
}

/// Check if an exit code indicates success (code 0).
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::exit_codes::is_success;
///
/// assert!(is_success(0));
/// assert!(!is_success(1));
/// ```
pub fn is_success(code: i32) -> bool {
    code == 0
}

/// Check if an exit code was caused by a signal (128+).
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::exit_codes::is_signal_exit;
///
/// assert!(is_signal_exit(130)); // SIGINT
/// assert!(is_signal_exit(143)); // SIGTERM
/// assert!(!is_signal_exit(1));
/// ```
pub fn is_signal_exit(code: i32) -> bool {
    code >= 128
}

/// Get the signal number from a signal-induced exit code.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::exit_codes::get_signal_from_exit;
///
/// assert_eq!(get_signal_from_exit(130), Some(2)); // SIGINT
/// assert_eq!(get_signal_from_exit(143), Some(15)); // SIGTERM
/// assert_eq!(get_signal_from_exit(1), None); // Not a signal exit
/// ```
pub fn get_signal_from_exit(code: i32) -> Option<i32> {
    if code >= 128 {
        Some(code - 128)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_exit_success() {
        let code = lookup_exit_code(0).expect("0 should exist");
        assert_eq!(code.code, 0);
        assert_eq!(code.name, "EXIT_SUCCESS");
        assert_eq!(code.category, ExitCategory::Standard);
    }

    #[test]
    fn test_lookup_exit_failure() {
        let code = lookup_exit_code(1).expect("1 should exist");
        assert_eq!(code.code, 1);
        assert_eq!(code.name, "EXIT_FAILURE");
    }

    #[test]
    fn test_lookup_signal_term() {
        let code = lookup_exit_code(143).expect("143 should exist");
        assert_eq!(code.name, "EXIT_SIGNAL_TERM");
        assert_eq!(code.category, ExitCategory::Signals);
    }

    #[test]
    fn test_lookup_not_found() {
        assert!(lookup_exit_code(999).is_none());
    }

    #[test]
    fn test_get_exit_name() {
        assert_eq!(get_exit_name(0), Some("EXIT_SUCCESS"));
        assert_eq!(get_exit_name(1), Some("EXIT_FAILURE"));
        assert_eq!(get_exit_name(999), None);
    }

    #[test]
    fn test_is_success() {
        assert!(is_success(0));
        assert!(!is_success(1));
        assert!(!is_success(143));
    }

    #[test]
    fn test_is_signal_exit() {
        assert!(is_signal_exit(128));
        assert!(is_signal_exit(130));
        assert!(is_signal_exit(143));
        assert!(!is_signal_exit(0));
        assert!(!is_signal_exit(1));
        assert!(!is_signal_exit(127));
    }

    #[test]
    fn test_get_signal_from_exit() {
        assert_eq!(get_signal_from_exit(130), Some(2)); // SIGINT
        assert_eq!(get_signal_from_exit(143), Some(15)); // SIGTERM
        assert_eq!(get_signal_from_exit(137), Some(9)); // SIGKILL
        assert_eq!(get_signal_from_exit(1), None);
    }

    #[test]
    fn test_list_by_category() {
        let standard = list_by_category(ExitCategory::Standard);
        assert!(standard.iter().any(|e| e.code == 0));
        assert!(standard.iter().any(|e| e.code == 1));
    }

    #[test]
    fn test_constants() {
        assert_eq!(EXIT_SUCCESS, 0);
        assert_eq!(EXIT_FAILURE, 1);
        assert_eq!(EXIT_PORT_IN_USE, 10);
        assert_eq!(EXIT_CONFIG_INVALID, 20);
        assert_eq!(EXIT_USAGE, 64);
        assert_eq!(EXIT_SIGNAL_INT, 130);
        assert_eq!(EXIT_SIGNAL_TERM, 143);
        assert_eq!(EXIT_SIGNAL_KILL, 137);
        assert_eq!(EXIT_TIMEOUT, 124);
        assert_eq!(EXIT_TIMEOUT_INTERNAL, 125);
        assert_eq!(EXIT_CANNOT_EXECUTE, 126);
        assert_eq!(EXIT_NOT_FOUND, 127);
    }

    #[test]
    fn test_category_from_code() {
        assert_eq!(ExitCategory::from_code(0), Some(ExitCategory::Standard));
        assert_eq!(ExitCategory::from_code(10), Some(ExitCategory::Networking));
        assert_eq!(
            ExitCategory::from_code(20),
            Some(ExitCategory::Configuration)
        );
        assert_eq!(ExitCategory::from_code(64), Some(ExitCategory::Usage));
        assert_eq!(ExitCategory::from_code(124), Some(ExitCategory::Shell));
        assert_eq!(ExitCategory::from_code(143), Some(ExitCategory::Signals));
    }
}
