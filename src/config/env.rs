//! Environment variable override loading for layered config.
//!
//! Maps environment variables into a nested `serde_yaml::Value` tree that can
//! be passed into `ThreeLayerOptions::runtime_overrides`.

use std::env;

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

/// How to parse the raw env var string value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvVarType {
    /// Pass through as-is.
    String,
    /// Parse via `str::parse::<i64>()`.
    Int,
    /// Parse via `str::parse::<f64>()`.
    Float,
    /// Parse as a flexible boolean.
    Bool,
}

impl EnvVarType {
    fn as_name(self) -> &'static str {
        match self {
            EnvVarType::String => "string",
            EnvVarType::Int => "int",
            EnvVarType::Float => "float",
            EnvVarType::Bool => "bool",
        }
    }
}

/// Maps an environment variable to a config key path.
#[derive(Debug, Clone)]
pub struct EnvVarSpec {
    /// Canonical env var name (for example: `APP_RETRIES`).
    pub name: String,
    /// Config key path (for example: `["settings", "retries"]`).
    pub path: Vec<String>,
    /// How to parse the raw string value.
    pub var_type: EnvVarType,
    /// Alternative env var names. First set alias wins.
    pub aliases: Vec<String>,
}

/// Which env var source provided the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvVarSource {
    /// The canonical `name` field.
    Canonical,
    /// One of the `aliases`.
    Alias,
}

/// Record of an env var that was applied as a config override.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVarApplied {
    /// Canonical name from the spec.
    pub spec_name: String,
    /// Actual env var name that provided the value.
    pub chosen_name: String,
    /// Whether the canonical name or an alias was used.
    pub source: EnvVarSource,
    /// Config key path where the value was injected.
    pub path: Vec<String>,
}

/// Conflict detected between canonical and alias env vars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVarConflict {
    /// Canonical env var name.
    pub canonical_name: String,
    /// Alias env var name that conflicted.
    pub alias_name: String,
    /// Canonical value (or `"[set]"` if masked).
    pub canonical_value: String,
    /// Alias value (or `"[set]"` if masked).
    pub alias_value: String,
    /// Which env var name won resolution.
    pub chosen_name: String,
    /// Whether values were masked due to sensitive names.
    pub masked: bool,
}

/// Diagnostic report from env var override loading.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvOverrideReport {
    /// Merged override map suitable for `runtime_overrides`.
    pub overrides: Value,
    /// Which env vars were actually applied.
    pub applied: Vec<EnvVarApplied>,
    /// Detected canonical-vs-alias conflicts.
    pub conflicts: Vec<EnvVarConflict>,
}

/// Error type for env var override loading.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum EnvOverrideError {
    /// Env var value failed to parse to its declared type.
    #[error("failed to parse env var {name}={value:?} as {expected_type}: {message}")]
    ParseError {
        /// The env var name that failed.
        name: String,
        /// Value captured for diagnostics (masked when sensitive).
        value: String,
        /// Expected type name.
        expected_type: String,
        /// Parse failure details.
        message: String,
    },
}

const SENSITIVE_KEYWORDS: &[&str] = &[
    "TOKEN",
    "SECRET",
    "PASSWORD",
    "PASSWD",
    "PWD",
    "API_KEY",
    "PRIVATE_KEY",
    "CREDENTIAL",
    "AUTHORIZATION",
];

/// Load environment variables into a nested override map.
pub fn load_env_overrides(specs: &[EnvVarSpec]) -> Result<Value, EnvOverrideError> {
    Ok(load_env_overrides_with_report(specs)?.overrides)
}

/// Load environment variables into overrides and return diagnostics.
pub fn load_env_overrides_with_report(
    specs: &[EnvVarSpec],
) -> Result<EnvOverrideReport, EnvOverrideError> {
    let mut root = Value::Mapping(Mapping::new());
    let mut applied = Vec::new();
    let mut conflicts = Vec::new();

    for spec in specs {
        let canonical = env::var(&spec.name).ok();
        let alias_values = collect_alias_values(&spec.aliases);
        let chosen = choose_source(spec, canonical.as_deref(), &alias_values);

        if let Some((chosen_name, source, raw)) = chosen {
            let parsed = parse_value(spec.var_type, &chosen_name, &raw)?;
            insert_override(&mut root, &spec.path, parsed);

            applied.push(EnvVarApplied {
                spec_name: spec.name.clone(),
                chosen_name: chosen_name.clone(),
                source,
                path: spec.path.clone(),
            });

            if let Some(canonical_raw) = canonical.as_deref() {
                for (alias_name, alias_raw) in &alias_values {
                    if canonical_raw.trim() == alias_raw.trim() {
                        continue;
                    }

                    let (canonical_value, canonical_masked) =
                        mask_env_value(&spec.name, canonical_raw);
                    let (alias_value, alias_masked) = mask_env_value(alias_name, alias_raw);

                    conflicts.push(EnvVarConflict {
                        canonical_name: spec.name.clone(),
                        alias_name: alias_name.clone(),
                        canonical_value,
                        alias_value,
                        chosen_name: chosen_name.clone(),
                        masked: canonical_masked || alias_masked,
                    });
                }
            }
        }
    }

    Ok(EnvOverrideReport {
        overrides: root,
        applied,
        conflicts,
    })
}

fn collect_alias_values(aliases: &[String]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for alias in aliases {
        if let Ok(value) = env::var(alias) {
            out.push((alias.clone(), value));
        }
    }
    out
}

fn choose_source(
    spec: &EnvVarSpec,
    canonical: Option<&str>,
    alias_values: &[(String, String)],
) -> Option<(String, EnvVarSource, String)> {
    if let Some((alias, value)) = alias_values.first() {
        return Some((alias.clone(), EnvVarSource::Alias, value.clone()));
    }

    canonical.map(|value| {
        (
            spec.name.clone(),
            EnvVarSource::Canonical,
            value.to_string(),
        )
    })
}

fn parse_value(var_type: EnvVarType, name: &str, raw: &str) -> Result<Value, EnvOverrideError> {
    match var_type {
        EnvVarType::String => Ok(Value::String(raw.to_string())),
        EnvVarType::Int => {
            let parsed = raw
                .trim()
                .parse::<i64>()
                .map_err(|err| parse_error(name, raw, var_type, err.to_string()))?;
            Ok(Value::Number(parsed.into()))
        }
        EnvVarType::Float => {
            let parsed = raw
                .trim()
                .parse::<f64>()
                .map_err(|err| parse_error(name, raw, var_type, err.to_string()))?;
            serde_yaml::to_value(parsed)
                .map_err(|err| parse_error(name, raw, var_type, err.to_string()))
        }
        EnvVarType::Bool => {
            let parsed = parse_flexible_bool(raw)
                .map_err(|err| parse_error(name, raw, var_type, err.to_string()))?;
            Ok(Value::Bool(parsed))
        }
    }
}

fn parse_error(name: &str, raw: &str, var_type: EnvVarType, message: String) -> EnvOverrideError {
    let (masked_value, _) = mask_env_value(name, raw);
    EnvOverrideError::ParseError {
        name: name.to_string(),
        value: masked_value,
        expected_type: var_type.as_name().to_string(),
        message,
    }
}

fn parse_flexible_bool(raw: &str) -> Result<bool, ParseBoolError> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "1" | "t" | "true" | "yes" | "y" => Ok(true),
        "0" | "f" | "false" | "no" | "n" => Ok(false),
        _ => Err(ParseBoolError {
            value: raw.trim().to_string(),
        }),
    }
}

fn insert_override(root: &mut Value, path: &[String], value: Value) {
    if path.is_empty() {
        *root = value;
        return;
    }

    if !matches!(root, Value::Mapping(_)) {
        *root = Value::Mapping(Mapping::new());
    }

    if let Value::Mapping(mapping) = root {
        insert_mapping(mapping, path, value);
    }
}

fn insert_mapping(mapping: &mut Mapping, path: &[String], value: Value) {
    let key = Value::String(path[0].clone());

    if path.len() == 1 {
        mapping.insert(key, value);
        return;
    }

    let mut child = match mapping.remove(&key) {
        Some(Value::Mapping(existing)) => existing,
        _ => Mapping::new(),
    };

    insert_mapping(&mut child, &path[1..], value);
    mapping.insert(key, Value::Mapping(child));
}

fn is_sensitive_env_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    SENSITIVE_KEYWORDS
        .iter()
        .any(|keyword| upper.contains(keyword))
}

fn mask_env_value(name: &str, raw: &str) -> (String, bool) {
    if is_sensitive_env_name(name) {
        ("[set]".to_string(), true)
    } else {
        (raw.trim().to_string(), false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParseBoolError {
    value: String,
}

impl std::fmt::Display for ParseBoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid boolean value: {}", self.value)
    }
}

impl std::error::Error for ParseBoolError {}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::sync::{Mutex, OnceLock};

    use super::*;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        previous: Vec<(String, Option<OsString>)>,
    }

    impl EnvGuard {
        fn set(values: &[(&str, &str)]) -> Self {
            let mut previous = Vec::with_capacity(values.len());
            for (key, value) in values {
                previous.push(((*key).to_string(), env::var_os(key)));
                env::set_var(key, value);
            }
            Self { previous }
        }

        fn unset(keys: &[&str]) -> Self {
            let mut previous = Vec::with_capacity(keys.len());
            for key in keys {
                previous.push(((*key).to_string(), env::var_os(key)));
                env::remove_var(key);
            }
            Self { previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, previous) in self.previous.iter().rev() {
                match previous {
                    Some(value) => env::set_var(key, value),
                    None => env::remove_var(key),
                }
            }
        }
    }

    #[test]
    fn test_load_env_overrides_string() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[("RSF_ENV_APP_NAME", "myapp")]);
        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_APP_NAME".to_string(),
            path: vec!["app".to_string(), "name".to_string()],
            var_type: EnvVarType::String,
            aliases: vec![],
        }];

        let overrides = load_env_overrides(&specs).unwrap();
        let app = overrides
            .as_mapping()
            .unwrap()
            .get(Value::String("app".to_string()))
            .unwrap()
            .as_mapping()
            .unwrap();
        assert_eq!(
            app.get(Value::String("name".to_string())).unwrap(),
            &Value::String("myapp".to_string())
        );
    }

    #[test]
    fn test_load_env_overrides_int() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[("RSF_ENV_APP_PORT", "8080")]);
        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_APP_PORT".to_string(),
            path: vec!["server".to_string(), "port".to_string()],
            var_type: EnvVarType::Int,
            aliases: vec![],
        }];

        let overrides = load_env_overrides(&specs).unwrap();
        let server = overrides
            .as_mapping()
            .unwrap()
            .get(Value::String("server".to_string()))
            .unwrap()
            .as_mapping()
            .unwrap();
        assert_eq!(
            server.get(Value::String("port".to_string())).unwrap(),
            &Value::Number(8080.into())
        );
    }

    #[test]
    fn test_load_env_overrides_bool_flexible() {
        let _lock = env_lock().lock().unwrap();
        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_APP_FLAG".to_string(),
            path: vec!["debug".to_string()],
            var_type: EnvVarType::Bool,
            aliases: vec![],
        }];

        for value in ["1", "t", "true", "yes", "y", "YES"] {
            let _guard = EnvGuard::set(&[("RSF_ENV_APP_FLAG", value)]);
            let overrides = load_env_overrides(&specs).unwrap();
            assert_eq!(
                overrides
                    .as_mapping()
                    .unwrap()
                    .get(Value::String("debug".to_string()))
                    .unwrap(),
                &Value::Bool(true)
            );
        }

        for value in ["0", "f", "false", "no", "n", "NO"] {
            let _guard = EnvGuard::set(&[("RSF_ENV_APP_FLAG", value)]);
            let overrides = load_env_overrides(&specs).unwrap();
            assert_eq!(
                overrides
                    .as_mapping()
                    .unwrap()
                    .get(Value::String("debug".to_string()))
                    .unwrap(),
                &Value::Bool(false)
            );
        }
    }

    #[test]
    fn test_load_env_overrides_parse_error() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[("RSF_ENV_APP_PORT", "notanumber")]);
        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_APP_PORT".to_string(),
            path: vec!["server".to_string(), "port".to_string()],
            var_type: EnvVarType::Int,
            aliases: vec![],
        }];

        let err = load_env_overrides(&specs).unwrap_err();
        assert!(matches!(err, EnvOverrideError::ParseError { .. }));
    }

    #[test]
    fn test_alias_takes_precedence() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[("RSF_ENV_APP_PORT", "8080"), ("RSF_ENV_PORT", "9090")]);
        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_APP_PORT".to_string(),
            path: vec!["server".to_string(), "port".to_string()],
            var_type: EnvVarType::Int,
            aliases: vec!["RSF_ENV_PORT".to_string()],
        }];

        let overrides = load_env_overrides(&specs).unwrap();
        let server = overrides
            .as_mapping()
            .unwrap()
            .get(Value::String("server".to_string()))
            .unwrap()
            .as_mapping()
            .unwrap();
        assert_eq!(
            server.get(Value::String("port".to_string())).unwrap(),
            &Value::Number(9090.into())
        );
    }

    #[test]
    fn test_conflict_detected_different_values() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[("RSF_ENV_APP_PORT", "8080"), ("RSF_ENV_PORT", "9090")]);
        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_APP_PORT".to_string(),
            path: vec!["server".to_string(), "port".to_string()],
            var_type: EnvVarType::Int,
            aliases: vec!["RSF_ENV_PORT".to_string()],
        }];

        let report = load_env_overrides_with_report(&specs).unwrap();
        assert_eq!(report.conflicts.len(), 1);
        assert_eq!(report.conflicts[0].canonical_name, "RSF_ENV_APP_PORT");
        assert_eq!(report.conflicts[0].alias_name, "RSF_ENV_PORT");
        assert_eq!(report.conflicts[0].chosen_name, "RSF_ENV_PORT");
    }

    #[test]
    fn test_no_conflict_when_values_match_after_trim() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[("RSF_ENV_APP_PORT", "8080"), ("RSF_ENV_PORT", " 8080 ")]);
        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_APP_PORT".to_string(),
            path: vec!["server".to_string(), "port".to_string()],
            var_type: EnvVarType::Int,
            aliases: vec!["RSF_ENV_PORT".to_string()],
        }];

        let report = load_env_overrides_with_report(&specs).unwrap();
        assert!(report.conflicts.is_empty());
    }

    #[test]
    fn test_sensitive_value_masking() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[
            ("RSF_ENV_APP_SECRET_KEY", "mysecret"),
            ("RSF_ENV_SECRET_KEY", "othersecret"),
        ]);
        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_APP_SECRET_KEY".to_string(),
            path: vec!["auth".to_string(), "secret".to_string()],
            var_type: EnvVarType::String,
            aliases: vec!["RSF_ENV_SECRET_KEY".to_string()],
        }];

        let report = load_env_overrides_with_report(&specs).unwrap();
        assert_eq!(report.conflicts.len(), 1);
        let conflict = &report.conflicts[0];
        assert_eq!(conflict.canonical_value, "[set]");
        assert_eq!(conflict.alias_value, "[set]");
        assert!(conflict.masked);
    }

    #[test]
    fn test_sensitive_name_detection() {
        assert!(is_sensitive_env_name("APP_SECRET_KEY"));
        assert!(is_sensitive_env_name("DB_PASSWORD"));
        assert!(is_sensitive_env_name("api_token"));
        assert!(!is_sensitive_env_name("APP_PORT"));
        assert!(!is_sensitive_env_name("APP_NAME"));
    }

    #[test]
    fn test_unset_env_var_skipped() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::unset(&["RSF_ENV_APP_MISSING"]);
        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_APP_MISSING".to_string(),
            path: vec!["missing".to_string()],
            var_type: EnvVarType::String,
            aliases: vec![],
        }];

        let report = load_env_overrides_with_report(&specs).unwrap();
        assert!(report.applied.is_empty());
        assert!(report.overrides.as_mapping().unwrap().is_empty());
    }

    #[test]
    fn test_nested_path_creates_intermediate_maps() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[("RSF_ENV_DB_HOST", "localhost")]);
        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_DB_HOST".to_string(),
            path: vec![
                "database".to_string(),
                "connection".to_string(),
                "host".to_string(),
            ],
            var_type: EnvVarType::String,
            aliases: vec![],
        }];

        let overrides = load_env_overrides(&specs).unwrap();
        let database = overrides
            .as_mapping()
            .unwrap()
            .get(Value::String("database".to_string()))
            .unwrap()
            .as_mapping()
            .unwrap();
        let connection = database
            .get(Value::String("connection".to_string()))
            .unwrap()
            .as_mapping()
            .unwrap();
        assert_eq!(
            connection.get(Value::String("host".to_string())).unwrap(),
            &Value::String("localhost".to_string())
        );
    }

    #[test]
    fn test_report_applied_tracks_source() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[("RSF_ENV_PORT", "8080")]);
        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_APP_PORT".to_string(),
            path: vec!["server".to_string(), "port".to_string()],
            var_type: EnvVarType::Int,
            aliases: vec!["RSF_ENV_PORT".to_string()],
        }];

        let report = load_env_overrides_with_report(&specs).unwrap();
        assert_eq!(report.applied.len(), 1);
        let applied = &report.applied[0];
        assert_eq!(applied.source, EnvVarSource::Alias);
        assert_eq!(applied.chosen_name, "RSF_ENV_PORT");
        assert_eq!(applied.spec_name, "RSF_ENV_APP_PORT");
    }

    #[test]
    fn test_parse_error_masks_sensitive_values() {
        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[("RSF_ENV_DB_PASSWORD", "not-a-bool")]);
        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_DB_PASSWORD".to_string(),
            path: vec!["db".to_string(), "password".to_string()],
            var_type: EnvVarType::Bool,
            aliases: vec![],
        }];

        let err = load_env_overrides(&specs).unwrap_err();
        match err {
            EnvOverrideError::ParseError { value, .. } => assert_eq!(value, "[set]"),
        }
    }

    #[cfg(feature = "three-layer-config")]
    #[test]
    fn test_integration_with_three_layer() {
        use crate::config::three_layer::{load_three_layer, ThreeLayerOptions};

        let _lock = env_lock().lock().unwrap();
        let _guard = EnvGuard::set(&[("RSF_ENV_TERMINAL_LANG", "fr")]);

        let specs = vec![EnvVarSpec {
            name: "RSF_ENV_TERMINAL_LANG".to_string(),
            path: vec!["language".to_string()],
            var_type: EnvVarType::String,
            aliases: vec![],
        }];

        let overrides = load_env_overrides(&specs).unwrap();
        let options = ThreeLayerOptions {
            app_name: format!("rsfulmen-test-{}", std::process::id()),
            defaults_path: "terminal/v1.0.0/terminal-overrides-defaults.yaml".to_string(),
            user_config_path: "does-not-exist.yaml".to_string(),
            runtime_overrides: Some(overrides),
            schema_path: None,
        };

        let loaded = load_three_layer(&options).unwrap();
        let mapping = loaded.config.as_mapping().unwrap();
        assert_eq!(
            mapping.get(Value::String("language".to_string())).unwrap(),
            &Value::String("fr".to_string())
        );
    }
}
