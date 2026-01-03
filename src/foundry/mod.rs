//! Foundry catalog data structures.
//!
//! This module provides access to standardized catalogs synced from Crucible,
//! including country codes, HTTP statuses, MIME types, and more.
//!
//! ## Overview
//!
//! Foundry catalogs provide pre-validated, production-ready reference data
//! that is consistent across all Fulmen ecosystem libraries (tsfulmen, pyfulmen,
//! gofulmen, rsfulmen).
//!
//! ## Available Catalogs
//!
//! | Catalog | Description | Status |
//! |---------|-------------|--------|
//! | [`country_codes`] | ISO 3166 country codes (alpha-2, alpha-3, numeric) | ✅ |
//! | [`http_statuses`] | HTTP status code registry | ✅ |
//! | [`exit_codes`] | Standard exit code definitions | ✅ |
//! | [`signals`] | Signal definitions and platform support | ✅ |
//! | [`mime_types`] | Common MIME types | ✅ (`foundry-mime-types`) |
//! | [`patterns`] | Validated regex patterns | ✅ (`foundry-patterns`) |
//! | [`similarity`] | Text similarity + "did you mean?" helpers | ✅ (`similarity`) |
//!
//! ## Example
//!
//! ```rust
//! use rsfulmen::foundry::country_codes::lookup_by_alpha2;
//!
//! let country = lookup_by_alpha2("US").unwrap();
//! assert_eq!(country.name, "United States of America");
//! ```
//!
//! ## Data Sources
//!
//! Catalog data is synced from Crucible via `make sync` and stored in
//! `config/crucible-rs/library/foundry/`. The data is embedded at compile time for
//! zero-runtime-cost access.

// Catalog modules
pub mod country_codes;
pub mod exit_codes;
pub mod http_statuses;
pub mod signals;

#[cfg(feature = "foundry-mime-types")]
pub mod mime_types;

#[cfg(feature = "foundry-patterns")]
pub mod patterns;

// Compatibility re-export for older callers.
#[cfg(feature = "similarity")]
pub mod similarity;

use thiserror::Error;

/// Errors that can occur when working with foundry catalogs.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FoundryError {
    /// The requested item was not found in the catalog.
    #[error("item not found: {0}")]
    NotFound(String),

    /// The catalog data could not be loaded or parsed.
    #[error("catalog load error: {0}")]
    LoadError(String),

    /// Invalid input provided for lookup.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

/// Result type for foundry operations.
pub type FoundryResult<T> = Result<T, FoundryError>;

/// Marker trait for catalog entries that can be looked up by string key.
pub trait CatalogEntry {
    /// The primary key type for this entry (e.g., country code, status code).
    type Key;

    /// Get the primary key for this entry.
    fn key(&self) -> &Self::Key;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foundry_error_display() {
        let err = FoundryError::NotFound("US".to_string());
        assert_eq!(err.to_string(), "item not found: US");
    }

    #[test]
    fn test_foundry_error_variants() {
        let not_found = FoundryError::NotFound("test".to_string());
        let load_error = FoundryError::LoadError("parse failed".to_string());
        let invalid = FoundryError::InvalidInput("bad format".to_string());

        assert!(matches!(not_found, FoundryError::NotFound(_)));
        assert!(matches!(load_error, FoundryError::LoadError(_)));
        assert!(matches!(invalid, FoundryError::InvalidInput(_)));
    }
}
