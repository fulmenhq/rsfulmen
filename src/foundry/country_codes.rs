//! ISO 3166 Country Code Catalog
//!
//! Provides country code lookups by alpha-2, alpha-3, and numeric codes.
//! All lookups are case-insensitive for alphabetic codes and handle
//! numeric code normalization (e.g., "76" → "076").
//!
//! ## Example
//!
//! ```rust
//! use rsfulmen::foundry::country_codes::{lookup_by_alpha2, lookup_by_alpha3, lookup_by_numeric};
//!
//! // Lookup by alpha-2 code (case-insensitive)
//! let usa = lookup_by_alpha2("us").unwrap();
//! assert_eq!(usa.name, "United States of America");
//!
//! // Lookup by alpha-3 code
//! let japan = lookup_by_alpha3("JPN").unwrap();
//! assert_eq!(japan.alpha2, "JP");
//!
//! // Lookup by numeric code (handles padding)
//! let brazil = lookup_by_numeric("76").unwrap();
//! assert_eq!(brazil.alpha2, "BR");
//! ```

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{FoundryError, FoundryResult};

/// ISO 3166-1 country code entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    /// ISO 3166-1 alpha-2 code (2 letters, e.g., "US")
    pub alpha2: String,
    /// ISO 3166-1 alpha-3 code (3 letters, e.g., "USA")
    pub alpha3: String,
    /// ISO 3166-1 numeric code (3 digits, e.g., "840")
    pub numeric: String,
    /// Short name (e.g., "United States of America")
    pub name: String,
    /// Official name (e.g., "United States of America")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub official_name: Option<String>,
}

/// Country code catalog with version information.
#[derive(Debug, Clone, Deserialize)]
struct CountryCatalog {
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    description: String,
    countries: Vec<Country>,
}

/// Embedded country codes YAML from Crucible.
const COUNTRY_CODES_YAML: &str =
    include_str!("../../config/crucible-rs/library/foundry/country-codes.yaml");

/// Lazily initialized country code indexes.
static CATALOGS: Lazy<CountryIndexes> =
    Lazy::new(|| CountryIndexes::load().expect("Failed to load embedded country codes catalog"));

/// Pre-computed indexes for fast lookups.
struct CountryIndexes {
    /// All countries in the catalog
    all: Vec<Country>,
    /// Index by uppercase alpha-2 code
    by_alpha2: HashMap<String, usize>,
    /// Index by uppercase alpha-3 code
    by_alpha3: HashMap<String, usize>,
    /// Index by normalized numeric code (3 digits with leading zeros)
    by_numeric: HashMap<String, usize>,
}

impl CountryIndexes {
    fn load() -> FoundryResult<Self> {
        let catalog: CountryCatalog = serde_yaml::from_str(COUNTRY_CODES_YAML).map_err(|e| {
            FoundryError::LoadError(format!("Failed to parse country codes: {}", e))
        })?;

        let mut by_alpha2 = HashMap::new();
        let mut by_alpha3 = HashMap::new();
        let mut by_numeric = HashMap::new();

        for (index, country) in catalog.countries.iter().enumerate() {
            // Alpha-2 index (uppercase)
            by_alpha2.insert(country.alpha2.to_uppercase(), index);

            // Alpha-3 index (uppercase)
            by_alpha3.insert(country.alpha3.to_uppercase(), index);

            // Numeric index (normalized to 3 digits)
            let normalized = normalize_numeric(&country.numeric);
            by_numeric.insert(normalized, index);
        }

        Ok(Self {
            all: catalog.countries,
            by_alpha2,
            by_alpha3,
            by_numeric,
        })
    }

    fn get_by_alpha2(&self, code: &str) -> Option<&Country> {
        let normalized = code.to_uppercase();
        self.by_alpha2.get(&normalized).map(|&i| &self.all[i])
    }

    fn get_by_alpha3(&self, code: &str) -> Option<&Country> {
        let normalized = code.to_uppercase();
        self.by_alpha3.get(&normalized).map(|&i| &self.all[i])
    }

    fn get_by_numeric(&self, code: &str) -> Option<&Country> {
        let normalized = normalize_numeric(code);
        self.by_numeric.get(&normalized).map(|&i| &self.all[i])
    }
}

/// Normalize a numeric code to 3 digits with leading zeros.
/// "76" → "076", "076" → "076", "840" → "840"
fn normalize_numeric(code: &str) -> String {
    format!("{:0>3}", code.trim())
}

/// Look up a country by its ISO 3166-1 alpha-2 code.
///
/// Lookup is case-insensitive ("us", "US", "Us" all work).
///
/// # Arguments
///
/// * `code` - The 2-letter alpha-2 code (e.g., "US", "CA", "JP")
///
/// # Returns
///
/// The country if found, or `None` if not found.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::country_codes::lookup_by_alpha2;
///
/// let usa = lookup_by_alpha2("US").unwrap();
/// assert_eq!(usa.name, "United States of America");
///
/// // Case-insensitive
/// let also_usa = lookup_by_alpha2("us").unwrap();
/// assert_eq!(also_usa.alpha2, "US");
/// ```
pub fn lookup_by_alpha2(code: &str) -> Option<&'static Country> {
    CATALOGS.get_by_alpha2(code)
}

/// Look up a country by its ISO 3166-1 alpha-3 code.
///
/// Lookup is case-insensitive ("usa", "USA", "Usa" all work).
///
/// # Arguments
///
/// * `code` - The 3-letter alpha-3 code (e.g., "USA", "CAN", "JPN")
///
/// # Returns
///
/// The country if found, or `None` if not found.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::country_codes::lookup_by_alpha3;
///
/// let japan = lookup_by_alpha3("JPN").unwrap();
/// assert_eq!(japan.alpha2, "JP");
/// ```
pub fn lookup_by_alpha3(code: &str) -> Option<&'static Country> {
    CATALOGS.get_by_alpha3(code)
}

/// Look up a country by its ISO 3166-1 numeric code.
///
/// Numeric codes are normalized to 3 digits with leading zeros.
/// "76", "076", and "76 " all resolve to "076" (Brazil).
///
/// # Arguments
///
/// * `code` - The numeric code as a string (e.g., "840", "076", "76")
///
/// # Returns
///
/// The country if found, or `None` if not found.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::country_codes::lookup_by_numeric;
///
/// // All of these find Brazil (numeric code 076)
/// assert!(lookup_by_numeric("076").is_some());
/// assert!(lookup_by_numeric("76").is_some());
/// ```
pub fn lookup_by_numeric(code: &str) -> Option<&'static Country> {
    CATALOGS.get_by_numeric(code)
}

/// List all countries in the catalog.
///
/// Returns a slice of all country entries.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::country_codes::list_countries;
///
/// let countries = list_countries();
/// assert!(!countries.is_empty());
/// ```
pub fn list_countries() -> &'static [Country] {
    &CATALOGS.all
}

/// Get the number of countries in the catalog.
pub fn country_count() -> usize {
    CATALOGS.all.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_by_alpha2() {
        let usa = lookup_by_alpha2("US").expect("US should exist");
        assert_eq!(usa.alpha2, "US");
        assert_eq!(usa.alpha3, "USA");
        assert_eq!(usa.name, "United States of America");
    }

    #[test]
    fn test_lookup_by_alpha2_case_insensitive() {
        let usa1 = lookup_by_alpha2("us").expect("lowercase should work");
        let usa2 = lookup_by_alpha2("US").expect("uppercase should work");
        let usa3 = lookup_by_alpha2("Us").expect("mixed case should work");
        assert_eq!(usa1.alpha2, usa2.alpha2);
        assert_eq!(usa2.alpha2, usa3.alpha2);
    }

    #[test]
    fn test_lookup_by_alpha3() {
        let japan = lookup_by_alpha3("JPN").expect("JPN should exist");
        assert_eq!(japan.alpha2, "JP");
        assert_eq!(japan.name, "Japan");
    }

    #[test]
    fn test_lookup_by_alpha3_case_insensitive() {
        let jp1 = lookup_by_alpha3("jpn").expect("lowercase should work");
        let jp2 = lookup_by_alpha3("JPN").expect("uppercase should work");
        assert_eq!(jp1.alpha2, jp2.alpha2);
    }

    #[test]
    fn test_lookup_by_numeric() {
        let brazil = lookup_by_numeric("076").expect("076 should exist");
        assert_eq!(brazil.alpha2, "BR");
        assert_eq!(brazil.name, "Brazil");
    }

    #[test]
    fn test_lookup_by_numeric_normalization() {
        // All should find Brazil (076)
        let br1 = lookup_by_numeric("076").expect("076 should work");
        let br2 = lookup_by_numeric("76").expect("76 should work");
        assert_eq!(br1.alpha2, br2.alpha2);
    }

    #[test]
    fn test_lookup_not_found() {
        assert!(lookup_by_alpha2("ZZ").is_none());
        assert!(lookup_by_alpha3("ZZZ").is_none());
        assert!(lookup_by_numeric("999").is_none());
    }

    #[test]
    fn test_list_countries() {
        let countries = list_countries();
        assert!(!countries.is_empty());
        // We know there are at least 5 countries in the sample catalog
        assert!(countries.len() >= 5);
    }

    #[test]
    fn test_country_count() {
        assert!(country_count() >= 5);
    }

    #[test]
    fn test_normalize_numeric() {
        assert_eq!(normalize_numeric("76"), "076");
        assert_eq!(normalize_numeric("076"), "076");
        assert_eq!(normalize_numeric("840"), "840");
        assert_eq!(normalize_numeric("1"), "001");
    }
}
