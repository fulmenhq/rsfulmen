//! Enterprise Three-Layer Configuration Loader
//!
//! Implements the Fulmen Enterprise Three-Layer Configuration Standard:
//!
//! - Layer 1: Embedded SSOT defaults (from Crucible)
//! - Layer 2: User config file (from `get_app_config_dir(app_name)`)
//! - Layer 3: Runtime overrides (BYOC, CLI flags, env-derived values)
//!
//! Precedence: runtime > user > defaults.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_yaml::{Mapping, Value};

use crate::config::get_app_config_dir;

/// Configuration layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigLayer {
    /// Layer 1: Crucible defaults.
    Defaults,
    /// Layer 2: User config file.
    User,
    /// Layer 3: Runtime overrides.
    Runtime,
}

/// Provenance map for debugging which layer last wrote a key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Provenance {
    /// JSON pointer-like paths to the winning layer.
    pub paths: BTreeMap<String, ConfigLayer>,
}

/// Result of a three-layer load.
#[derive(Debug, Clone, PartialEq)]
pub struct LayeredConfig {
    /// Final merged configuration.
    pub config: Value,
    /// Provenance of merged keys.
    pub provenance: Provenance,
}

/// Errors that can occur while loading layered configuration.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ThreeLayerError {
    /// Defaults asset was not found in embedded Crucible config.
    #[error("defaults not found in embedded config: {0}")]
    DefaultsNotFound(String),

    /// YAML parsing failed.
    #[error("invalid yaml at {path}: {message}")]
    InvalidYaml {
        /// Path context.
        path: String,
        /// Error details.
        message: String,
    },

    /// File IO error.
    #[error("io error reading {path}: {message}")]
    Io {
        /// Path context.
        path: String,
        /// Error details.
        message: String,
    },

    /// Schema validation requested but not available.
    #[error("schema validation requested but feature 'schema-validation' is disabled")]
    SchemaValidationUnavailable,

    /// Schema validation failed.
    #[cfg(feature = "schema-validation")]
    #[error("schema validation failed with {0} issue(s)")]
    SchemaValidationFailed(usize),
}

/// Options for loading a three-layer configuration.
#[derive(Debug, Clone)]
pub struct ThreeLayerOptions {
    /// Application name (determines the user config directory).
    pub app_name: String,

    /// Embedded defaults config path under `config/crucible-rs/`.
    ///
    /// Example: `terminal/v1.0.0/terminal-overrides-defaults.yaml`
    pub defaults_path: String,

    /// Relative path under the app config directory for the user config.
    ///
    /// Example: `terminal/overrides.yaml`
    pub user_config_path: String,

    /// Optional runtime overrides.
    pub runtime_overrides: Option<Value>,

    /// Optional schema path under `schemas/crucible-rs/` for validation.
    ///
    /// If set and the `schema-validation` feature is enabled, the final merged
    /// config will be validated.
    pub schema_path: Option<String>,
}

impl ThreeLayerOptions {
    /// Construct options with no runtime overrides or schema validation.
    pub fn new(
        app_name: impl Into<String>,
        defaults_path: impl Into<String>,
        user_config_path: impl Into<String>,
    ) -> Self {
        Self {
            app_name: app_name.into(),
            defaults_path: defaults_path.into(),
            user_config_path: user_config_path.into(),
            runtime_overrides: None,
            schema_path: None,
        }
    }
}

/// Load, merge, and (optionally) validate three-layer configuration.
pub fn load_three_layer(options: &ThreeLayerOptions) -> Result<LayeredConfig, ThreeLayerError> {
    let defaults = load_embedded_defaults(&options.defaults_path)?;

    let user_path = get_app_config_dir(&options.app_name).join(&options.user_config_path);
    let user = load_yaml_file_optional(&user_path)?;

    let mut provenance = Provenance::default();
    let mut merged = defaults;

    provenance_mark_root(&mut provenance, ConfigLayer::Defaults);

    if let Some(user) = user {
        merged = merge_with_provenance(merged, user, ConfigLayer::User, &mut provenance, "");
    }

    if let Some(runtime) = options.runtime_overrides.clone() {
        merged = merge_with_provenance(merged, runtime, ConfigLayer::Runtime, &mut provenance, "");
    }

    if let Some(schema_path) = options.schema_path.as_deref() {
        validate_if_enabled(schema_path, &merged)?;
    }

    Ok(LayeredConfig {
        config: merged,
        provenance,
    })
}

/// Load embedded defaults YAML from the Crucible synced config tree.
pub fn load_embedded_defaults(config_path: &str) -> Result<Value, ThreeLayerError> {
    let bytes = crate::crucible::open_config_bytes(config_path)
        .ok_or_else(|| ThreeLayerError::DefaultsNotFound(config_path.to_string()))?;

    serde_yaml::from_slice(bytes).map_err(|e| ThreeLayerError::InvalidYaml {
        path: format!("embedded:{config_path}"),
        message: e.to_string(),
    })
}

/// Load an optional YAML file.
///
/// - Missing file => `Ok(None)`
/// - Invalid YAML => error
pub fn load_yaml_file_optional(path: &Path) -> Result<Option<Value>, ThreeLayerError> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(ThreeLayerError::Io {
                path: path.display().to_string(),
                message: e.to_string(),
            })
        }
    };

    let value: Value =
        serde_yaml::from_slice(&bytes).map_err(|e| ThreeLayerError::InvalidYaml {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;

    Ok(Some(value))
}

fn validate_if_enabled(schema_path: &str, merged: &Value) -> Result<(), ThreeLayerError> {
    #[cfg(feature = "schema-validation")]
    {
        let yaml = serde_yaml::to_string(merged).map_err(|e| ThreeLayerError::InvalidYaml {
            path: "<merged>".to_string(),
            message: e.to_string(),
        })?;

        let issues = crate::schema_validation::validate_bytes(schema_path, yaml.as_bytes())
            .map_err(|e| ThreeLayerError::InvalidYaml {
                path: schema_path.to_string(),
                message: e.to_string(),
            })?;

        if !issues.is_empty() {
            return Err(ThreeLayerError::SchemaValidationFailed(issues.len()));
        }

        Ok(())
    }

    #[cfg(not(feature = "schema-validation"))]
    {
        let _ = schema_path;
        let _ = merged;
        Err(ThreeLayerError::SchemaValidationUnavailable)
    }
}

fn provenance_mark_root(provenance: &mut Provenance, layer: ConfigLayer) {
    provenance.paths.insert("".to_string(), layer);
}

fn merge_with_provenance(
    base: Value,
    overlay: Value,
    layer: ConfigLayer,
    provenance: &mut Provenance,
    path: &str,
) -> Value {
    match (base, overlay) {
        (Value::Mapping(mut base_map), Value::Mapping(overlay_map)) => {
            merge_maps(&mut base_map, overlay_map, layer, provenance, path);
            Value::Mapping(base_map)
        }
        (Value::Sequence(_), Value::Sequence(overlay_seq)) => {
            provenance.paths.insert(path.to_string(), layer);
            Value::Sequence(overlay_seq)
        }
        (Value::Null, overlay) => {
            provenance.paths.insert(path.to_string(), layer);
            overlay
        }
        (_, Value::Null) => {
            // Null deletes are handled at the parent map level. If we get here,
            // treat as null overwrite.
            provenance.paths.insert(path.to_string(), layer);
            Value::Null
        }
        (_, overlay) => {
            provenance.paths.insert(path.to_string(), layer);
            overlay
        }
    }
}

fn merge_maps(
    base: &mut Mapping,
    overlay: Mapping,
    layer: ConfigLayer,
    provenance: &mut Provenance,
    path: &str,
) {
    for (k, v) in overlay {
        // Only track provenance for string keys (Fulmen configs should use these).
        let key_str = k.as_str().map(|s| s.to_string());
        let child_path = if let Some(key_str) = &key_str {
            join_path(path, key_str)
        } else {
            path.to_string()
        };

        if v.is_null() {
            base.remove(&k);
            provenance.paths.insert(child_path, layer);
            continue;
        }

        if let Some(existing) = base.remove(&k) {
            let merged = merge_with_provenance(existing, v, layer, provenance, &child_path);
            base.insert(k, merged);
        } else {
            provenance.paths.insert(child_path.clone(), layer);
            base.insert(k, v);
        }
    }
}

fn join_path(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        format!("/{key}")
    } else {
        format!("{prefix}/{key}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        std::env::temp_dir().join(format!("rsfulmen-{name}-{ts}"))
    }

    #[test]
    fn test_merge_precedence_and_arrays_replace() {
        let defaults: Value = serde_yaml::from_str(
            r#"
key: a
nested:
  x: 1
  list: [1, 2]
"#,
        )
        .unwrap();

        let user: Value = serde_yaml::from_str(
            r#"
key: b
nested:
  y: 2
  list: [9]
"#,
        )
        .unwrap();

        let runtime: Value = serde_yaml::from_str(
            r#"
key: c
nested:
  x: 10
"#,
        )
        .unwrap();

        let mut provenance = Provenance::default();
        provenance_mark_root(&mut provenance, ConfigLayer::Defaults);

        let merged = merge_with_provenance(defaults, user, ConfigLayer::User, &mut provenance, "");
        let merged =
            merge_with_provenance(merged, runtime, ConfigLayer::Runtime, &mut provenance, "");

        let expected: Value = serde_yaml::from_str(
            r#"
key: c
nested:
  x: 10
  y: 2
  list: [9]
"#,
        )
        .unwrap();

        assert_eq!(merged, expected);
        assert_eq!(provenance.paths.get("/key"), Some(&ConfigLayer::Runtime));
        assert_eq!(
            provenance.paths.get("/nested/x"),
            Some(&ConfigLayer::Runtime)
        );
        assert_eq!(provenance.paths.get("/nested/y"), Some(&ConfigLayer::User));
        assert_eq!(
            provenance.paths.get("/nested/list"),
            Some(&ConfigLayer::User)
        );
    }

    #[test]
    fn test_null_deletes_key() {
        let defaults: Value = serde_yaml::from_str("a: 1\nb: 2\n").unwrap();
        let user: Value = serde_yaml::from_str("b: null\n").unwrap();

        let mut provenance = Provenance::default();
        provenance_mark_root(&mut provenance, ConfigLayer::Defaults);
        let merged = merge_with_provenance(defaults, user, ConfigLayer::User, &mut provenance, "");

        let expected: Value = serde_yaml::from_str("a: 1\n").unwrap();
        assert_eq!(merged, expected);
        assert_eq!(provenance.paths.get("/b"), Some(&ConfigLayer::User));
    }

    #[test]
    fn test_load_three_layer_reads_optional_user_file() {
        // Arrange: create an app config dir under temp and write overrides.
        let tmp = temp_dir("three-layer");
        fs::create_dir_all(&tmp).unwrap();

        // Our loader uses get_app_config_dir(app_name), so we can't redirect it
        // without setting env. Instead, validate the file loader directly.
        let user_file = tmp.join("config.yaml");
        fs::write(&user_file, "b: 2\n").unwrap();

        let loaded = load_yaml_file_optional(&user_file).unwrap().unwrap();
        let expected: Value = serde_yaml::from_str("b: 2\n").unwrap();
        assert_eq!(loaded, expected);

        let missing = load_yaml_file_optional(&tmp.join("missing.yaml")).unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn test_load_yaml_file_optional_invalid_yaml() {
        let tmp = temp_dir("three-layer-invalid-yaml");
        fs::create_dir_all(&tmp).unwrap();

        let file = tmp.join("bad.yaml");
        fs::write(&file, ": not valid yaml: [").unwrap();

        let err = load_yaml_file_optional(&file).unwrap_err();
        assert!(matches!(err, ThreeLayerError::InvalidYaml { .. }));
    }

    #[test]
    fn test_load_embedded_defaults_known_path() {
        // Smoke-test: ensure we can read a known defaults asset.
        let defaults = load_embedded_defaults("terminal/v1.0.0/terminal-overrides-defaults.yaml")
            .expect("embedded defaults should load");

        assert!(matches!(defaults, Value::Mapping(_)));
    }

    #[test]
    fn test_load_three_layer_defaults_only_with_runtime_overrides() {
        // Use a unique app name + non-existent config file to avoid pulling any user config.
        let options = ThreeLayerOptions {
            app_name: format!("rsfulmen-test-{}", std::process::id()),
            defaults_path: "terminal/v1.0.0/terminal-overrides-defaults.yaml".to_string(),
            user_config_path: "does-not-exist.yaml".to_string(),
            runtime_overrides: Some(serde_yaml::from_str("runtimeOnly: true\n").unwrap()),
            schema_path: None,
        };

        let out = load_three_layer(&options).unwrap();
        assert_eq!(
            out.provenance.paths.get("/runtimeOnly"),
            Some(&ConfigLayer::Runtime)
        );

        let mapping = out.config.as_mapping().unwrap();
        assert!(mapping.contains_key(Value::String("runtimeOnly".to_string())));
    }

    #[cfg(feature = "schema-validation")]
    #[test]
    fn test_schema_validation_failure_returns_error() {
        // Deliberately validate an unrelated config against the mime-types schema.
        let options = ThreeLayerOptions {
            app_name: format!("rsfulmen-test-{}", std::process::id()),
            defaults_path: "terminal/v1.0.0/terminal-overrides-defaults.yaml".to_string(),
            user_config_path: "does-not-exist.yaml".to_string(),
            runtime_overrides: None,
            schema_path: Some("library/foundry/v1.0.0/mime-types.schema.json".to_string()),
        };

        let err = load_three_layer(&options).unwrap_err();
        assert!(matches!(err, ThreeLayerError::SchemaValidationFailed(_)));
    }

    #[cfg(not(feature = "schema-validation"))]
    #[test]
    fn test_schema_validation_unavailable_feature_gate() {
        let options = ThreeLayerOptions {
            app_name: format!("rsfulmen-test-{}", std::process::id()),
            defaults_path: "terminal/v1.0.0/terminal-overrides-defaults.yaml".to_string(),
            user_config_path: "does-not-exist.yaml".to_string(),
            runtime_overrides: None,
            schema_path: Some("library/foundry/v1.0.0/mime-types.schema.json".to_string()),
        };

        let err = load_three_layer(&options).unwrap_err();
        assert!(matches!(err, ThreeLayerError::SchemaValidationUnavailable));
    }
}
