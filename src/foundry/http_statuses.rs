//! HTTP Status Code Catalog
//!
//! Provides HTTP status code lookups with standard reason phrases and group information.
//!
//! ## Example
//!
//! ```rust
//! use rsfulmen::foundry::http_statuses::{lookup_status, StatusGroup};
//!
//! let ok = lookup_status(200).unwrap();
//! assert_eq!(ok.reason, "OK");
//! assert_eq!(ok.group, StatusGroup::Success);
//!
//! let not_found = lookup_status(404).unwrap();
//! assert_eq!(not_found.reason, "Not Found");
//! ```

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{FoundryError, FoundryResult};

/// HTTP status code group identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatusGroup {
    /// 1xx - Informational responses
    Informational,
    /// 2xx - Successful responses
    Success,
    /// 3xx - Redirection responses
    Redirect,
    /// 4xx - Client error responses
    ClientError,
    /// 5xx - Server error responses
    ServerError,
}

impl StatusGroup {
    /// Get the human-readable name of the status group.
    pub fn name(&self) -> &'static str {
        match self {
            StatusGroup::Informational => "Informational Responses",
            StatusGroup::Success => "Successful Responses",
            StatusGroup::Redirect => "Redirection Responses",
            StatusGroup::ClientError => "Client Error Responses",
            StatusGroup::ServerError => "Server Error Responses",
        }
    }

    /// Get the description of the status group.
    pub fn description(&self) -> &'static str {
        match self {
            StatusGroup::Informational => "1xx HTTP status codes indicating provisional responses.",
            StatusGroup::Success => "2xx HTTP status codes indicating successful requests.",
            StatusGroup::Redirect => "3xx HTTP status codes indicating redirects.",
            StatusGroup::ClientError => "4xx HTTP status codes indicating client errors.",
            StatusGroup::ServerError => "5xx HTTP status codes indicating server errors.",
        }
    }

    /// Determine the group from a status code value.
    pub fn from_code(code: u16) -> Option<Self> {
        match code {
            100..=199 => Some(StatusGroup::Informational),
            200..=299 => Some(StatusGroup::Success),
            300..=399 => Some(StatusGroup::Redirect),
            400..=499 => Some(StatusGroup::ClientError),
            500..=599 => Some(StatusGroup::ServerError),
            _ => None,
        }
    }
}

/// HTTP status code entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpStatus {
    /// The numeric status code (e.g., 200, 404, 500)
    pub code: u16,
    /// The standard reason phrase (e.g., "OK", "Not Found")
    pub reason: String,
    /// The status group this code belongs to
    pub group: StatusGroup,
}

/// Raw status code entry from YAML.
#[derive(Debug, Clone, Deserialize)]
struct RawStatusCode {
    value: u16,
    reason: String,
}

/// Raw status group from YAML.
#[derive(Debug, Clone, Deserialize)]
struct RawStatusGroup {
    id: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    description: String,
    codes: Vec<RawStatusCode>,
}

/// HTTP status catalog from YAML.
#[derive(Debug, Clone, Deserialize)]
struct HttpStatusCatalog {
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    description: String,
    groups: Vec<RawStatusGroup>,
}

/// Embedded HTTP statuses YAML from Crucible.
const HTTP_STATUSES_YAML: &str =
    include_str!("../../config/crucible-rs/library/foundry/http-statuses.yaml");

/// Lazily initialized HTTP status indexes.
static CATALOGS: Lazy<HttpStatusIndexes> =
    Lazy::new(|| HttpStatusIndexes::load().expect("Failed to load embedded HTTP statuses catalog"));

/// Pre-computed indexes for fast lookups.
struct HttpStatusIndexes {
    /// All status codes
    all: Vec<HttpStatus>,
    /// Index by status code value
    by_code: HashMap<u16, usize>,
}

fn parse_group_id(id: &str) -> Option<StatusGroup> {
    match id {
        "informational" => Some(StatusGroup::Informational),
        "success" => Some(StatusGroup::Success),
        "redirect" => Some(StatusGroup::Redirect),
        "client-error" => Some(StatusGroup::ClientError),
        "server-error" => Some(StatusGroup::ServerError),
        _ => None,
    }
}

impl HttpStatusIndexes {
    fn load() -> FoundryResult<Self> {
        let catalog: HttpStatusCatalog = serde_yaml::from_str(HTTP_STATUSES_YAML).map_err(|e| {
            FoundryError::LoadError(format!("Failed to parse HTTP statuses: {}", e))
        })?;

        let mut all = Vec::new();
        let mut by_code = HashMap::new();

        for group in catalog.groups {
            let group_id = parse_group_id(&group.id).ok_or_else(|| {
                FoundryError::LoadError(format!("Unknown group id: {}", group.id))
            })?;

            for code in group.codes {
                let index = all.len();
                all.push(HttpStatus {
                    code: code.value,
                    reason: code.reason,
                    group: group_id,
                });
                by_code.insert(code.value, index);
            }
        }

        Ok(Self { all, by_code })
    }
}

/// Look up an HTTP status by its numeric code.
///
/// # Arguments
///
/// * `code` - The HTTP status code (e.g., 200, 404, 500)
///
/// # Returns
///
/// The status if found, or `None` if not a standard status code.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::http_statuses::lookup_status;
///
/// let ok = lookup_status(200).unwrap();
/// assert_eq!(ok.reason, "OK");
///
/// let not_found = lookup_status(404).unwrap();
/// assert_eq!(not_found.reason, "Not Found");
/// ```
pub fn lookup_status(code: u16) -> Option<&'static HttpStatus> {
    CATALOGS.by_code.get(&code).map(|&i| &CATALOGS.all[i])
}

/// Get the reason phrase for an HTTP status code.
///
/// # Arguments
///
/// * `code` - The HTTP status code
///
/// # Returns
///
/// The reason phrase if found, or `None`.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::http_statuses::get_reason;
///
/// assert_eq!(get_reason(200), Some("OK"));
/// assert_eq!(get_reason(404), Some("Not Found"));
/// ```
pub fn get_reason(code: u16) -> Option<&'static str> {
    lookup_status(code).map(|s| s.reason.as_str())
}

/// List all HTTP status codes in a specific group.
///
/// # Arguments
///
/// * `group` - The status group to filter by
///
/// # Returns
///
/// A vector of status codes in the specified group.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::http_statuses::{list_by_group, StatusGroup};
///
/// let success_codes = list_by_group(StatusGroup::Success);
/// assert!(success_codes.iter().any(|s| s.code == 200));
/// ```
pub fn list_by_group(group: StatusGroup) -> Vec<&'static HttpStatus> {
    CATALOGS.all.iter().filter(|s| s.group == group).collect()
}

/// List all HTTP status codes in the catalog.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::http_statuses::list_statuses;
///
/// let all = list_statuses();
/// assert!(!all.is_empty());
/// ```
pub fn list_statuses() -> &'static [HttpStatus] {
    &CATALOGS.all
}

/// Check if a status code indicates success (2xx).
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::http_statuses::is_success;
///
/// assert!(is_success(200));
/// assert!(is_success(201));
/// assert!(!is_success(404));
/// ```
pub fn is_success(code: u16) -> bool {
    (200..300).contains(&code)
}

/// Check if a status code indicates a client error (4xx).
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::http_statuses::is_client_error;
///
/// assert!(is_client_error(400));
/// assert!(is_client_error(404));
/// assert!(!is_client_error(500));
/// ```
pub fn is_client_error(code: u16) -> bool {
    (400..500).contains(&code)
}

/// Check if a status code indicates a server error (5xx).
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::http_statuses::is_server_error;
///
/// assert!(is_server_error(500));
/// assert!(is_server_error(503));
/// assert!(!is_server_error(404));
/// ```
pub fn is_server_error(code: u16) -> bool {
    (500..600).contains(&code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_status_200() {
        let status = lookup_status(200).expect("200 should exist");
        assert_eq!(status.code, 200);
        assert_eq!(status.reason, "OK");
        assert_eq!(status.group, StatusGroup::Success);
    }

    #[test]
    fn test_lookup_status_404() {
        let status = lookup_status(404).expect("404 should exist");
        assert_eq!(status.code, 404);
        assert_eq!(status.reason, "Not Found");
        assert_eq!(status.group, StatusGroup::ClientError);
    }

    #[test]
    fn test_lookup_status_500() {
        let status = lookup_status(500).expect("500 should exist");
        assert_eq!(status.code, 500);
        assert_eq!(status.reason, "Internal Server Error");
        assert_eq!(status.group, StatusGroup::ServerError);
    }

    #[test]
    fn test_lookup_not_found() {
        assert!(lookup_status(999).is_none());
    }

    #[test]
    fn test_get_reason() {
        assert_eq!(get_reason(200), Some("OK"));
        assert_eq!(get_reason(404), Some("Not Found"));
        assert_eq!(get_reason(999), None);
    }

    #[test]
    fn test_list_by_group() {
        let success = list_by_group(StatusGroup::Success);
        assert!(success.iter().any(|s| s.code == 200));
        assert!(success.iter().any(|s| s.code == 201));
        assert!(!success.iter().any(|s| s.code == 404));
    }

    #[test]
    fn test_is_success() {
        assert!(is_success(200));
        assert!(is_success(201));
        assert!(is_success(299));
        assert!(!is_success(100));
        assert!(!is_success(300));
        assert!(!is_success(404));
    }

    #[test]
    fn test_is_client_error() {
        assert!(is_client_error(400));
        assert!(is_client_error(404));
        assert!(is_client_error(499));
        assert!(!is_client_error(200));
        assert!(!is_client_error(500));
    }

    #[test]
    fn test_is_server_error() {
        assert!(is_server_error(500));
        assert!(is_server_error(503));
        assert!(is_server_error(599));
        assert!(!is_server_error(200));
        assert!(!is_server_error(404));
    }

    #[test]
    fn test_status_group_from_code() {
        assert_eq!(
            StatusGroup::from_code(100),
            Some(StatusGroup::Informational)
        );
        assert_eq!(StatusGroup::from_code(200), Some(StatusGroup::Success));
        assert_eq!(StatusGroup::from_code(301), Some(StatusGroup::Redirect));
        assert_eq!(StatusGroup::from_code(404), Some(StatusGroup::ClientError));
        assert_eq!(StatusGroup::from_code(500), Some(StatusGroup::ServerError));
        assert_eq!(StatusGroup::from_code(600), None);
    }

    #[test]
    fn test_list_statuses() {
        let all = list_statuses();
        assert!(!all.is_empty());
        // There should be at least 50 status codes
        assert!(all.len() >= 50);
    }
}
