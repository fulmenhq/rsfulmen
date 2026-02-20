//! Correlation ID helpers.
//!
//! Provides UUIDv7 generation, parsing, validation, and a validated
//! [`CorrelationId`] newtype for cross-module request correlation.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use uuid::Uuid;

/// A validated UUIDv7 correlation identifier.
///
/// Correlation IDs are stored and displayed in canonical lowercase hyphenated
/// UUID format. Uppercase input is accepted and normalized during parse and
/// deserialize.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorrelationId(String);

impl CorrelationId {
    /// Generate a new UUIDv7 correlation ID.
    pub fn new() -> Self {
        Self(generate())
    }

    /// Validate that this ID remains a well-formed UUIDv7.
    pub fn is_valid(&self) -> bool {
        is_valid(&self.0)
    }

    /// Return the canonical UUID string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for CorrelationId {
    type Err = CorrelationIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parsed = parse_uuid_v7(value)?;
        Ok(Self(parsed.hyphenated().to_string()))
    }
}

impl Serialize for CorrelationId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CorrelationId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        CorrelationId::from_str(&raw).map_err(serde::de::Error::custom)
    }
}

/// Error type for correlation ID parsing and validation.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum CorrelationIdError {
    /// Input was empty.
    #[error("empty correlation ID")]
    Empty,
    /// Input was not a valid UUID representation.
    #[error("invalid UUID format: {0}")]
    InvalidFormat(String),
    /// UUID version was not v7.
    #[error("not a UUIDv7 (version {found}, expected 7)")]
    WrongVersion {
        /// The parsed UUID version number.
        found: u8,
    },
}

/// Generate a new UUIDv7 correlation ID string.
pub fn generate() -> String {
    Uuid::now_v7().hyphenated().to_string()
}

/// Parse and validate a UUIDv7 correlation ID.
pub fn parse(value: &str) -> Result<CorrelationId, CorrelationIdError> {
    CorrelationId::from_str(value)
}

/// Return true if the input is a valid UUIDv7 correlation ID.
pub fn is_valid(value: &str) -> bool {
    parse_uuid_v7(value).is_ok()
}

fn parse_uuid_v7(value: &str) -> Result<Uuid, CorrelationIdError> {
    if value.is_empty() {
        return Err(CorrelationIdError::Empty);
    }

    let parsed =
        Uuid::parse_str(value).map_err(|err| CorrelationIdError::InvalidFormat(err.to_string()))?;

    let version = parsed.get_version_num();
    if version != 7 {
        return Err(CorrelationIdError::WrongVersion {
            found: u8::try_from(version).unwrap_or(u8::MAX),
        });
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::str::FromStr;

    use super::{parse, CorrelationId};
    use crate::foundry::correlation::{generate, is_valid, CorrelationIdError};

    #[test]
    fn test_generate_returns_valid_uuidv7() {
        let id = generate();
        assert!(is_valid(&id));
        let parsed = uuid::Uuid::parse_str(&id).unwrap();
        assert_eq!(parsed.get_version_num(), 7);
    }

    #[test]
    fn test_generate_is_unique() {
        let ids: Vec<String> = (0..100).map(|_| generate()).collect();
        let unique: HashSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn test_generate_version_is_7() {
        for _ in 0..50 {
            let id = generate();
            let parsed = uuid::Uuid::parse_str(&id).unwrap();
            assert_eq!(parsed.get_version_num(), 7);
        }
    }

    #[test]
    fn test_is_valid_rejects_v4() {
        assert!(!is_valid("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_is_valid_rejects_garbage() {
        assert!(!is_valid(""));
        assert!(!is_valid("not-a-uuid"));
        assert!(!is_valid("12345"));
    }

    #[test]
    fn test_parse_returns_correlation_id() {
        let id = generate();
        let parsed = parse(&id).unwrap();
        assert_eq!(parsed.as_str(), id);
    }

    #[test]
    fn test_correlation_id_newtype_serde_roundtrip() {
        let id = CorrelationId::new();
        let yaml = serde_yaml::to_string(&id).unwrap();
        let parsed: CorrelationId = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_correlation_id_rejects_v4_on_deserialize() {
        let yaml = "'550e8400-e29b-41d4-a716-446655440000'";
        let result: Result<CorrelationId, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_correlation_id_from_str() {
        let id = CorrelationId::new();
        let parsed = CorrelationId::from_str(id.as_str()).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_correlation_id_from_str_rejects_empty() {
        let err = CorrelationId::from_str("").unwrap_err();
        assert_eq!(err, CorrelationIdError::Empty);
    }

    #[test]
    fn test_uppercase_input_normalized_to_lowercase() {
        let uppercase = "019503E7-2C4A-7000-8000-A1B2C3D4E5F6";
        let parsed = CorrelationId::from_str(uppercase).unwrap();
        assert_eq!(parsed.as_str(), "019503e7-2c4a-7000-8000-a1b2c3d4e5f6");
        assert_eq!(parsed.to_string(), "019503e7-2c4a-7000-8000-a1b2c3d4e5f6");
    }
}
