//! Schema Validation
//!
//! Provides helpers to discover embedded Crucible schemas and validate JSON/YAML
//! documents against JSON Schema draft 2020-12 by default (supports draft-07 when
//! declared by the schema).

use std::fs;
use std::path::Path;

use jsonschema::{Draft, JSONSchema, SchemaResolver, SchemaResolverError};
use serde::Deserialize;
use std::sync::Arc;
use url::Url;

/// A validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    /// JSON pointer to the failing instance location.
    pub pointer: String,
    /// Human-readable error message.
    pub message: String,
    /// JSON Schema keyword that triggered the failure.
    pub keyword: Option<String>,
    /// Severity (`ERROR` for now).
    pub severity: Severity,
    /// Validation source.
    pub source: ValidationSource,
}

/// Validation severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Validation error.
    Error,
    /// Validation warning (reserved).
    Warn,
}

/// Source of validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSource {
    /// Language-native validation (jsonschema crate).
    Native,
    /// Validation performed by goneat CLI (future).
    Goneat,
}

/// Embedded schema metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaInfo {
    /// Path relative to `schemas/crucible-rs/`.
    pub path: String,
    /// `$id` of the schema (if present).
    pub id: Option<String>,
    /// Title.
    pub title: Option<String>,
    /// Description.
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SchemaHeader {
    #[serde(rename = "$id")]
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
}

/// Errors from schema validation operations.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum SchemaValidationError {
    /// Schema asset was not found.
    #[error("schema not found: {0}")]
    SchemaNotFound(String),

    /// Schema could not be parsed as JSON.
    #[error("invalid schema JSON at {path}: {message}")]
    InvalidSchemaJson {
        /// Schema path.
        path: String,
        /// Error details.
        message: String,
    },

    /// Payload could not be parsed.
    #[error("invalid payload: {0}")]
    InvalidPayload(String),

    /// Schema compilation failed.
    #[error("schema compile failed for {path}: {message}")]
    SchemaCompileFailed {
        /// Schema path.
        path: String,
        /// Error details.
        message: String,
    },
}

/// List available schema assets.
///
/// Optionally filter by prefix (path prefix under `schemas/crucible-rs/`).
pub fn list_schemas(prefix: Option<&str>) -> Vec<SchemaInfo> {
    let mut out = Vec::new();

    for asset in crate::crucible::list_schemas() {
        let path = asset.path;
        if let Some(prefix) = prefix {
            if !path.starts_with(prefix) {
                continue;
            }
        }

        // Index JSON/YAML schema assets (YAML allows comments).
        let is_schema =
            path.ends_with(".json") || path.ends_with(".yaml") || path.ends_with(".yml");
        if !is_schema {
            continue;
        }

        let header =
            crate::crucible::open_schemas(path).and_then(|b| schema_header_from_bytes(b).ok());

        out.push(SchemaInfo {
            path: path.to_string(),
            id: header.as_ref().and_then(|h| h.id.clone()),
            title: header.as_ref().and_then(|h| h.title.clone()),
            description: header.as_ref().and_then(|h| h.description.clone()),
        });
    }

    out
}

/// Load an embedded schema by path.
///
/// Schemas may be stored as JSON (`.json`) or YAML (`.yaml`/`.yml`).
pub fn load_schema(path: &str) -> Result<serde_json::Value, SchemaValidationError> {
    let (resolved_path, bytes) = open_schema_bytes(path)?;

    parse_schema_bytes(&resolved_path, bytes)
}

fn open_schema_bytes(path: &str) -> Result<(String, &'static [u8]), SchemaValidationError> {
    if let Some(bytes) = crate::crucible::open_schemas(path) {
        return Ok((path.to_string(), bytes));
    }

    // Convenience fallbacks for callers that omit extensions.
    let candidates = [
        format!("{path}.json"),
        format!("{path}.schema.json"),
        format!("{path}.yaml"),
        format!("{path}.schema.yaml"),
        format!("{path}.yml"),
        format!("{path}.schema.yml"),
    ];

    for candidate in candidates {
        if let Some(bytes) = crate::crucible::open_schemas(&candidate) {
            return Ok((candidate, bytes));
        }
    }

    Err(SchemaValidationError::SchemaNotFound(path.to_string()))
}

fn parse_schema_bytes(
    path: &str,
    bytes: &[u8],
) -> Result<serde_json::Value, SchemaValidationError> {
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) {
        return Ok(value);
    }

    let yaml_value: serde_yaml::Value =
        serde_yaml::from_slice(bytes).map_err(|e| SchemaValidationError::InvalidSchemaJson {
            path: path.to_string(),
            message: e.to_string(),
        })?;

    serde_json::to_value(yaml_value).map_err(|e| SchemaValidationError::InvalidSchemaJson {
        path: path.to_string(),
        message: e.to_string(),
    })
}

fn schema_header_from_bytes(bytes: &[u8]) -> Result<SchemaHeader, SchemaValidationError> {
    if let Ok(h) = serde_json::from_slice::<SchemaHeader>(bytes) {
        return Ok(h);
    }

    let yaml_value: serde_yaml::Value =
        serde_yaml::from_slice(bytes).map_err(|e| SchemaValidationError::InvalidSchemaJson {
            path: "<header>".to_string(),
            message: e.to_string(),
        })?;

    let json_value =
        serde_json::to_value(yaml_value).map_err(|e| SchemaValidationError::InvalidSchemaJson {
            path: "<header>".to_string(),
            message: e.to_string(),
        })?;

    serde_json::from_value::<SchemaHeader>(json_value).map_err(|e| {
        SchemaValidationError::InvalidSchemaJson {
            path: "<header>".to_string(),
            message: e.to_string(),
        }
    })
}

/// Validate an in-memory JSON value against an embedded schema.
pub fn validate_data(
    schema_path: &str,
    data: &serde_json::Value,
) -> Result<Vec<ValidationIssue>, SchemaValidationError> {
    let schema_json = load_schema(schema_path)?;

    let resolver = EmbeddedSchemaResolver;

    let compiled = JSONSchema::options()
        .with_draft(select_draft(&schema_json))
        .with_resolver(resolver)
        .compile(&schema_json)
        .map_err(|e| SchemaValidationError::SchemaCompileFailed {
            path: schema_path.to_string(),
            message: e.to_string(),
        })?;

    let mut issues = Vec::new();

    if let Err(errors) = compiled.validate(data) {
        for error in errors {
            let schema_path = error.schema_path.to_string();
            issues.push(ValidationIssue {
                pointer: error.instance_path.to_string(),
                message: error.to_string(),
                keyword: keyword_from_schema_path(&schema_path),
                severity: Severity::Error,
                source: ValidationSource::Native,
            });
        }
    }

    Ok(issues)
}

fn select_draft(schema: &serde_json::Value) -> Draft {
    let schema_uri = schema.get("$schema").and_then(|v| v.as_str()).unwrap_or("");

    if schema_uri.contains("draft-07") {
        Draft::Draft7
    } else if schema_uri.contains("2020-12") {
        Draft::Draft202012
    } else {
        // Fulmen default.
        Draft::Draft202012
    }
}

fn keyword_from_schema_path(schema_path: &str) -> Option<String> {
    let keyword = schema_path
        .trim_start_matches('/')
        .split('/')
        .next_back()?
        .trim();

    if keyword.is_empty() {
        None
    } else {
        Some(keyword.to_string())
    }
}

/// Validate a JSON or YAML document from bytes.
pub fn validate_bytes(
    schema_path: &str,
    bytes: &[u8],
) -> Result<Vec<ValidationIssue>, SchemaValidationError> {
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) {
        return validate_data(schema_path, &value);
    }

    let yaml_value: serde_yaml::Value = serde_yaml::from_slice(bytes)
        .map_err(|e| SchemaValidationError::InvalidPayload(e.to_string()))?;
    let json_value = serde_json::to_value(yaml_value)
        .map_err(|e| SchemaValidationError::InvalidPayload(e.to_string()))?;

    validate_data(schema_path, &json_value)
}

/// Validate a file on disk (JSON or YAML).
pub fn validate_file(
    schema_path: &str,
    path: &Path,
) -> Result<Vec<ValidationIssue>, SchemaValidationError> {
    let bytes = fs::read(path).map_err(|e| SchemaValidationError::InvalidPayload(e.to_string()))?;
    validate_bytes(schema_path, &bytes)
}

/// Compare an embedded schema to another JSON document.
pub fn compare_schema(
    schema_path: &str,
    other_schema_json: &serde_json::Value,
) -> Result<bool, SchemaValidationError> {
    let embedded = load_schema(schema_path)?;
    Ok(&embedded == other_schema_json)
}

/// Minimal `$ref` resolver that maps canonical schema URLs to embedded assets.
///
/// Per the Canonical URI Resolution Standard (Crucible v0.4.2+), Crucible-hosted
/// schemas use module-qualified IDs:
///
/// `https://schemas.fulmenhq.dev/crucible/<topic>/<version>/<filename>`
///
/// rsfulmen embeds only the `crucible` module schemas; other modules are not
/// resolvable offline by this resolver.
///
/// Note: internal JSON pointer refs (`#/...`) are handled by the validator.
#[derive(Default, Clone, Copy)]
struct EmbeddedSchemaResolver;

impl EmbeddedSchemaResolver {
    fn host_supported(host: &str) -> bool {
        host == "schemas.fulmenhq.dev" || host == "schemas.3leaps.dev" || host == "json-schema.org"
    }

    fn url_to_embedded_path(url: &Url) -> Option<String> {
        let host = url.host_str()?;
        if !Self::host_supported(host) {
            return None;
        }

        let path = url.path().trim_start_matches('/');
        if path.is_empty() {
            return None;
        }

        if host == "json-schema.org" {
            return Self::json_schema_org_to_embedded_path(path);
        }

        // Canonical module-qualified schema IDs (Crucible v0.4.2+).
        // Only resolve the module we embed: `crucible`.
        path.strip_prefix("crucible/")
            .map(|rest| rest.to_string())
            .or_else(|| {
                // Some config schemas (e.g., taxonomy) are hosted outside the `crucible/` prefix.
                if path.starts_with("config/") {
                    Some(path.to_string())
                } else {
                    None
                }
            })
    }

    fn json_schema_org_to_embedded_path(path: &str) -> Option<String> {
        // Draft-07 meta schema
        if let Some(rest) = path.strip_prefix("draft-07/") {
            if rest == "schema" {
                return Some("meta/draft-07/schema.json".to_string());
            }
        }

        // Draft 2020-12 meta schemas
        if let Some(rest) = path.strip_prefix("draft/2020-12/") {
            if rest == "schema" {
                return Some("meta/draft-2020-12/schema.json".to_string());
            }

            if let Some(meta) = rest.strip_prefix("meta/") {
                let mut p = format!("meta/draft-2020-12/meta/{meta}");
                if !p.ends_with(".json") {
                    p.push_str(".json");
                }
                return Some(p);
            }
        }

        None
    }
}

impl SchemaResolver for EmbeddedSchemaResolver {
    fn resolve(
        &self,
        _root_schema: &serde_json::Value,
        url: &Url,
        _original_reference: &str,
    ) -> Result<Arc<serde_json::Value>, SchemaResolverError> {
        match url.scheme() {
            "http" | "https" => {
                let Some(path) = Self::url_to_embedded_path(url) else {
                    return Err(anyhow::anyhow!("unsupported schema URL: {url}"));
                };

                let bytes = crate::crucible::open_schemas(&path)
                    .or_else(|| {
                        if path.ends_with(".json") {
                            None
                        } else {
                            crate::crucible::open_schemas(&format!("{path}.json"))
                        }
                    })
                    .or_else(|| {
                        // Some schemas reference config-hosted taxonomy YAML via canonical URLs.
                        path.strip_prefix("config/")
                            .and_then(|rest| crate::crucible::open_config_bytes(rest))
                    })
                    .ok_or_else(|| {
                        anyhow::anyhow!("schema not found in embedded assets: {path}")
                    })?;

                let value: serde_json::Value = if let Ok(v) = serde_json::from_slice(bytes) {
                    v
                } else {
                    let yaml: serde_yaml::Value = serde_yaml::from_slice(bytes)?;
                    serde_json::to_value(yaml)?
                };

                Ok(Arc::new(value))
            }
            // This scheme is used when resolving relative external refs without a root `$id`.
            "json-schema" => Err(anyhow::anyhow!(
                "cannot resolve relative external schema without root schema ID"
            )),
            "file" => Err(anyhow::anyhow!(
                "file:// schema resolution is not supported in embedded mode"
            )),
            other => Err(anyhow::anyhow!("unsupported schema scheme: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIME_SCHEMA: &str = "library/foundry/v1.0.0/mime-types.schema.json";
    const META_2020_12: &str = "meta/draft-2020-12/schema.json";
    const SYNC_KEYS_SCHEMA: &str = "config/sync-keys.schema.yaml";

    #[test]
    fn test_validate_known_good_yaml_against_schema() {
        let yaml = include_bytes!("../../config/crucible-rs/library/foundry/mime-types.yaml");
        let issues = validate_bytes(MIME_SCHEMA, yaml).unwrap();
        assert!(issues.is_empty(), "expected no issues, got {issues:?}");
    }

    #[test]
    fn test_validation_reports_missing_required_field() {
        let value = serde_json::json!({"version": "v0.1.0"});
        let issues = validate_data(MIME_SCHEMA, &value).unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.pointer.is_empty()));
    }

    #[test]
    fn test_list_schemas_filters_prefix() {
        let foundry = list_schemas(Some("library/foundry/"));
        assert!(!foundry.is_empty());
        assert!(foundry
            .iter()
            .all(|s| s.path.starts_with("library/foundry/")));
    }

    #[test]
    fn test_meta_validate_schema_offline() {
        let schema = load_schema(MIME_SCHEMA).unwrap();
        let issues = validate_data(META_2020_12, &schema).unwrap();
        assert!(issues.is_empty(), "expected no meta issues, got {issues:?}");
    }

    #[test]
    fn test_resolver_strips_crucible_module_prefix() {
        let url = Url::parse(
            "https://schemas.fulmenhq.dev/crucible/library/foundry/v1.0.0/mime-types.schema.json",
        )
        .unwrap();

        let embedded = EmbeddedSchemaResolver::url_to_embedded_path(&url).unwrap();
        assert_eq!(
            embedded,
            "library/foundry/v1.0.0/mime-types.schema.json".to_string()
        );

        // Ensure the mapped path is actually present.
        let bytes = crate::crucible::open_schemas(&embedded).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_load_yaml_schema_and_validate_minimal_yaml_data() {
        // Schema is stored as YAML (with potential comments) and must validate YAML data.
        let schema = load_schema(SYNC_KEYS_SCHEMA).unwrap();
        let meta_issues = validate_data(META_2020_12, &schema).unwrap();
        assert!(
            meta_issues.is_empty(),
            "expected yaml schema to be meta-valid, got {meta_issues:?}"
        );

        // Minimal compliant sample.
        let data = br#"version: "2025.10.0"
keys:
  - id: crucible.docs
    description: General documentation
    basePath: docs/
  - id: crucible.schemas.terminal
    description: Terminal schemas
    basePath: schemas/terminal/
"#;

        let issues = validate_bytes(SYNC_KEYS_SCHEMA, data).unwrap();
        assert!(issues.is_empty(), "expected no issues, got {issues:?}");
    }

    #[test]
    fn test_synced_sync_keys_yaml_validates_against_schema() {
        let data = include_bytes!("../../config/crucible-rs/sync/sync-keys.yaml");
        let issues = validate_bytes(SYNC_KEYS_SCHEMA, data).unwrap();
        assert!(issues.is_empty(), "expected no issues, got {issues:?}");
    }
}
