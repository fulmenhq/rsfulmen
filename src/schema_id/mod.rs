//! Schema ID Helpers
//!
//! Helpers for emitting and resolving `schema_id` fields in JSON outputs.
//!
//! This follows the Crucible Canonical URI Resolution Standard:
//!
//! `https://schemas.<org>.dev/<module>/<topic>/<version>/<filename>`
//!
//! For Crucible-synced schemas, the module is `crucible`.

use serde::{Deserialize, Serialize};

/// Canonical field name used in Fulmen JSON payloads.
pub const SCHEMA_ID_FIELD: &str = "schema_id";

/// Wraps a serializable value with a `schema_id` field.
///
/// The wrapped `value` is flattened so the JSON output looks like:
///
/// ```json
/// { "schema_id": "https://...", ... }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaTagged<T> {
    /// Canonical schema identifier.
    pub schema_id: &'static str,

    /// Payload value.
    #[serde(flatten)]
    pub value: T,
}

/// Convenience constructor for [`SchemaTagged`].
pub fn tag<T>(schema_id: &'static str, value: T) -> SchemaTagged<T> {
    SchemaTagged { schema_id, value }
}

/// Parsed canonical schema URI components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaUri {
    /// Module namespace (e.g. `crucible`, `rsfulmen`).
    pub module: String,
    /// Topic path segments joined with `/`.
    pub topic: String,
    /// Version directory (e.g. `v1.0.0`).
    pub version: String,
    /// Filename with extension.
    pub filename: String,
}

impl SchemaUri {
    /// Return the path portion after the module.
    ///
    /// Example: `library/foundry/v1.0.0/mime-types.schema.json`.
    pub fn embedded_schema_path(&self) -> String {
        format!("{}/{}/{}", self.topic, self.version, self.filename)
    }
}

/// Errors from schema_id URI parsing or resolution.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum SchemaIdError {
    /// Not a supported schema URI.
    #[error("invalid schema uri: {0}")]
    InvalidSchemaUri(String),

    /// Module is not available locally.
    #[error("schema module not available locally: {0}")]
    ModuleNotAvailable(String),

    /// Schema not found in embedded assets.
    #[error("schema not found for uri: {0}")]
    SchemaNotFound(String),

    /// Embedded schema has no `$id`.
    #[error("embedded schema missing $id: {0}")]
    MissingId(String),

    /// `$id` mismatch.
    #[error("schema id mismatch: expected {expected}, got {actual}")]
    IdMismatch {
        /// Expected ID.
        expected: String,
        /// Actual ID.
        actual: String,
    },

    /// Schema JSON parse failed.
    #[error("invalid schema json: {0}")]
    InvalidSchemaJson(String),
}

/// Parse a canonical schema URI.
///
/// Expected format:
/// `https://schemas.<org>.dev/<module>/<topic>/<version>/<filename>`
pub fn parse_schema_uri(uri: &str) -> Result<SchemaUri, SchemaIdError> {
    let uri = uri.trim();

    let prefix = "https://schemas.";
    let after_prefix = uri
        .strip_prefix(prefix)
        .ok_or_else(|| SchemaIdError::InvalidSchemaUri(uri.to_string()))?;

    // Accept any host of the form `schemas.<org>.dev`.
    let (host_rest, rest) = after_prefix
        .split_once(".dev/")
        .ok_or_else(|| SchemaIdError::InvalidSchemaUri(uri.to_string()))?;

    if host_rest.is_empty() {
        return Err(SchemaIdError::InvalidSchemaUri(uri.to_string()));
    }

    // Drop fragment (we only care about the canonical path).
    let rest = rest.split('#').next().unwrap_or(rest);

    let mut parts: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() < 4 {
        return Err(SchemaIdError::InvalidSchemaUri(uri.to_string()));
    }

    let module = parts.remove(0).to_string();
    let filename = parts.pop().unwrap().to_string();
    let version = parts.pop().unwrap().to_string();
    let topic = parts.join("/");

    Ok(SchemaUri {
        module,
        topic,
        version,
        filename,
    })
}

/// Returns true if rsfulmen can resolve the given schema URI locally.
#[cfg(feature = "crucible")]
pub fn can_resolve_schema_uri(uri: &str) -> bool {
    resolve_schema_uri(uri).is_ok()
}

/// Resolve a canonical schema URI to embedded schema bytes.
///
/// Supported modules:
///
/// - `crucible`: resolves to embedded Crucible schemas under `schemas/crucible-rs/`
///
/// Note: rsfulmen does not currently ship library-native schemas under module `rsfulmen`.
#[cfg(feature = "crucible")]
pub fn resolve_schema_uri(uri: &str) -> Result<&'static [u8], SchemaIdError> {
    let parsed = parse_schema_uri(uri)?;

    match parsed.module.as_str() {
        "crucible" => {
            let path = parsed.embedded_schema_path();
            crate::crucible::open_schemas(&path)
                .ok_or_else(|| SchemaIdError::SchemaNotFound(uri.to_string()))
        }
        other => Err(SchemaIdError::ModuleNotAvailable(other.to_string())),
    }
}

/// Assert that a schema `$id` matches the expected canonical schema URI.
///
/// This is intended for CI/unit tests to ensure `schema_id` constants match SSOT.
#[cfg(feature = "crucible")]
pub fn assert_schema_id_matches_embedded_schema(
    schema_path: &str,
    expected_schema_id: &str,
) -> Result<(), SchemaIdError> {
    let bytes = crate::crucible::open_schemas(schema_path)
        .ok_or_else(|| SchemaIdError::SchemaNotFound(schema_path.to_string()))?;

    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| SchemaIdError::InvalidSchemaJson(e.to_string()))?;

    let actual = value
        .get("$id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| SchemaIdError::MissingId(schema_path.to_string()))?;

    if actual == expected_schema_id {
        Ok(())
    } else {
        Err(SchemaIdError::IdMismatch {
            expected: expected_schema_id.to_string(),
            actual: actual.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "schema-validation")]
    #[test]
    fn test_schema_tagged_serializes_schema_id() {
        #[derive(Serialize)]
        struct Example {
            name: &'static str,
        }

        let tagged = tag(
            "https://schemas.fulmenhq.dev/sysprims/example/v1.0.0/example.schema.json",
            Example { name: "x" },
        );
        let json = serde_json::to_value(&tagged).unwrap();

        assert_eq!(
            json.get(SCHEMA_ID_FIELD).and_then(|v| v.as_str()),
            Some("https://schemas.fulmenhq.dev/sysprims/example/v1.0.0/example.schema.json")
        );
        assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("x"));
    }

    #[test]
    fn test_parse_schema_uri() {
        let uri =
            "https://schemas.fulmenhq.dev/crucible/library/foundry/v1.0.0/mime-types.schema.json";
        let parsed = parse_schema_uri(uri).unwrap();
        assert_eq!(parsed.module, "crucible");
        assert_eq!(parsed.topic, "library/foundry");
        assert_eq!(parsed.version, "v1.0.0");
        assert_eq!(parsed.filename, "mime-types.schema.json");
        assert_eq!(
            parsed.embedded_schema_path(),
            "library/foundry/v1.0.0/mime-types.schema.json"
        );

        let uri = "https://schemas.3leaps.dev/sysprims/timeout/v1.0.0/timeout-result.schema.json";
        let parsed = parse_schema_uri(uri).unwrap();
        assert_eq!(parsed.module, "sysprims");
        assert_eq!(parsed.topic, "timeout");
        assert_eq!(parsed.version, "v1.0.0");
        assert_eq!(parsed.filename, "timeout-result.schema.json");
    }

    #[cfg(feature = "crucible")]
    #[test]
    fn test_resolve_schema_uri_to_embedded_bytes() {
        let uri =
            "https://schemas.fulmenhq.dev/crucible/library/foundry/v1.0.0/mime-types.schema.json";
        let bytes = resolve_schema_uri(uri).unwrap();
        assert!(!bytes.is_empty());

        // Ensure it matches direct open by embedded path.
        let direct =
            crate::crucible::open_schemas("library/foundry/v1.0.0/mime-types.schema.json").unwrap();
        assert_eq!(bytes, direct);
    }

    #[cfg(feature = "schema-validation")]
    #[test]
    fn test_assert_schema_id_matches_embedded_schema_round_trip() {
        let schema_path = "library/foundry/v1.0.0/mime-types.schema.json";
        let bytes = crate::crucible::open_schemas(schema_path).unwrap();
        let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        let id = value.get("$id").and_then(|v| v.as_str()).unwrap();

        assert_schema_id_matches_embedded_schema(schema_path, id).unwrap();
    }
}
