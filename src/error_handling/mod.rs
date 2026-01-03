//! Error Handling & Propagation
//!
//! Implements the Crucible **Error Handling & Propagation** standard by extending
//! the Pathfinder error envelope with optional telemetry fields.
//!
//! This module is intentionally **data-model-first** (ADR-0006): the canonical
//! error contract is represented as a serializable struct rather than a
//! hierarchy of Rust error types.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Embedded schema path for validating error payloads.
pub const ERROR_RESPONSE_SCHEMA_PATH: &str = "error-handling/v1.0.0/error-response.schema.json";

/// Errors produced by this module.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ErrorHandlingError {
    /// Exit codes must fit in the 0-255 range.
    #[error("exit_code out of range (0-255): {0}")]
    ExitCodeOutOfRange(i32),

    /// Severity levels must be 0-4.
    #[error("severity_level out of range (0-4): {0}")]
    SeverityLevelOutOfRange(u8),

    /// Timestamp formatting failed.
    #[error("timestamp formatting failed: {0}")]
    TimestampFormat(String),

    /// JSON serialization failed.
    #[error("json serialization failed: {0}")]
    JsonSerialization(String),
}

/// Assessment-aligned severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational (0).
    Info,
    /// Low severity (1).
    Low,
    /// Medium severity (2).
    Medium,
    /// High severity (3).
    High,
    /// Critical severity (4).
    Critical,
}

impl Severity {
    /// Returns the canonical numeric level (info=0 ... critical=4).
    pub const fn level(self) -> u8 {
        match self {
            Severity::Info => 0,
            Severity::Low => 1,
            Severity::Medium => 2,
            Severity::High => 3,
            Severity::Critical => 4,
        }
    }

    /// Convert a numeric severity level back into the enum.
    pub const fn from_level(level: u8) -> Option<Self> {
        match level {
            0 => Some(Severity::Info),
            1 => Some(Severity::Low),
            2 => Some(Severity::Medium),
            3 => Some(Severity::High),
            4 => Some(Severity::Critical),
            _ => None,
        }
    }
}

/// Context value types permitted by the error-handling schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContextValue {
    /// String value.
    String(String),
    /// Numeric value.
    Number(serde_json::Number),
    /// Boolean value.
    Bool(bool),
    /// Array of strings.
    StringArray(Vec<String>),
}

impl From<String> for ContextValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for ContextValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<bool> for ContextValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i32> for ContextValue {
    fn from(value: i32) -> Self {
        Self::Number(serde_json::Number::from(value))
    }
}

impl From<i64> for ContextValue {
    fn from(value: i64) -> Self {
        Self::Number(serde_json::Number::from(value))
    }
}

impl From<u64> for ContextValue {
    fn from(value: u64) -> Self {
        Self::Number(serde_json::Number::from(value))
    }
}

impl From<Vec<String>> for ContextValue {
    fn from(value: Vec<String>) -> Self {
        Self::StringArray(value)
    }
}

impl From<Vec<&str>> for ContextValue {
    fn from(value: Vec<&str>) -> Self {
        Self::StringArray(value.into_iter().map(|s| s.to_string()).collect())
    }
}

/// Optional serialized form of a wrapped/original error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OriginalValue {
    /// String representation.
    String(String),
    /// Object representation.
    Object(BTreeMap<String, Value>),
}

/// Pathfinder error response envelope.
///
/// This matches `schemas/crucible-rs/pathfinder/v1.0.0/error-response.schema.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathfinderErrorResponse {
    /// Error code identifier.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Additional structured details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<BTreeMap<String, Value>>,
    /// Path that caused the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// RFC3339 timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl PathfinderErrorResponse {
    /// Create a minimal Pathfinder error response.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            path: None,
            timestamp: None,
        }
    }

    /// Set `timestamp` to the current UTC time (RFC3339, seconds precision).
    pub fn set_timestamp_now_utc(&mut self) -> Result<(), ErrorHandlingError> {
        self.timestamp = Some(now_rfc3339()?);
        Ok(())
    }
}

/// Fulmen error response envelope.
///
/// This extends Pathfinder's envelope by adding optional telemetry properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code identifier.
    pub code: String,
    /// Human-readable error message.
    pub message: String,

    /// Additional structured details (unrestricted object).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<BTreeMap<String, Value>>,

    /// Path that caused the error (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// RFC3339 timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Severity name (info/low/medium/high/critical).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,

    /// Numeric severity (info=0 ... critical=4).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity_level: Option<u8>,

    /// Correlation identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,

    /// Tracing identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    /// Optional process exit code (0-255).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<u8>,

    /// Structured non-sensitive context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<BTreeMap<String, ContextValue>>,

    /// Optional serialized original error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original: Option<OriginalValue>,

    /// Additional extension fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl From<PathfinderErrorResponse> for ErrorResponse {
    fn from(value: PathfinderErrorResponse) -> Self {
        Self {
            code: value.code,
            message: value.message,
            details: value.details,
            path: value.path,
            timestamp: value.timestamp,
            severity: None,
            severity_level: None,
            correlation_id: None,
            trace_id: None,
            exit_code: None,
            context: None,
            original: None,
            extra: BTreeMap::new(),
        }
    }
}

impl ErrorResponse {
    /// Create a minimal Fulmen error response.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        PathfinderErrorResponse::new(code, message).into()
    }

    /// Set `timestamp` to the current UTC time (RFC3339, seconds precision).
    pub fn set_timestamp_now_utc(&mut self) -> Result<(), ErrorHandlingError> {
        self.timestamp = Some(now_rfc3339()?);
        Ok(())
    }

    /// Wrap a base error payload with telemetry and optional metadata.
    pub fn wrap<B: Into<Self>>(base: B, options: WrapOptions) -> Result<Self, ErrorHandlingError> {
        let mut out = base.into();

        if let Some(severity) = options.severity {
            out.set_severity(severity);
        }

        if let Some(level) = options.severity_level {
            out.set_severity_level(level)?;
        }

        if let Some(correlation_id) = options.correlation_id {
            out.correlation_id = Some(correlation_id);
        }

        if let Some(trace_id) = options.trace_id {
            out.trace_id = Some(trace_id);
        }

        if let Some(exit_code) = options.exit_code {
            out.exit_code = Some(exit_code);
        }

        if let Some(context) = options.context {
            out.context
                .get_or_insert_with(BTreeMap::new)
                .extend(context);
        }

        if let Some(original) = options.original {
            out.original = Some(original);
        }

        Ok(out)
    }

    /// Set both `severity` and `severity_level` consistently.
    pub fn set_severity(&mut self, severity: Severity) {
        self.severity = Some(severity);
        self.severity_level = Some(severity.level());
    }

    /// Set `severity_level` and (when possible) derive `severity`.
    pub fn set_severity_level(&mut self, severity_level: u8) -> Result<(), ErrorHandlingError> {
        let severity = Severity::from_level(severity_level)
            .ok_or(ErrorHandlingError::SeverityLevelOutOfRange(severity_level))?;

        self.severity_level = Some(severity_level);
        self.severity.get_or_insert(severity);
        Ok(())
    }

    /// Set `exit_code` from an `i32`, validating the 0-255 range.
    pub fn set_exit_code_i32(&mut self, exit_code: i32) -> Result<(), ErrorHandlingError> {
        let exit_code_u8 = u8::try_from(exit_code).map_err(|_| {
            // Note: try_from covers negative numbers and >255.
            ErrorHandlingError::ExitCodeOutOfRange(exit_code)
        })?;

        self.exit_code = Some(exit_code_u8);
        Ok(())
    }

    /// Insert a single context key/value.
    pub fn insert_context(&mut self, key: impl Into<String>, value: impl Into<ContextValue>) {
        self.context
            .get_or_insert_with(BTreeMap::new)
            .insert(key.into(), value.into());
    }

    /// Serialize to a `serde_json::Value`.
    pub fn to_json_value(&self) -> Result<Value, ErrorHandlingError> {
        serde_json::to_value(self).map_err(|e| ErrorHandlingError::JsonSerialization(e.to_string()))
    }

    /// Serialize to a JSON string.
    pub fn to_json_string(&self) -> Result<String, ErrorHandlingError> {
        serde_json::to_string(self)
            .map_err(|e| ErrorHandlingError::JsonSerialization(e.to_string()))
    }

    /// Serialize to a pretty-printed JSON string.
    pub fn to_json_string_pretty(&self) -> Result<String, ErrorHandlingError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| ErrorHandlingError::JsonSerialization(e.to_string()))
    }

    /// Validate this payload against the embedded Crucible schema.
    #[cfg(feature = "schema-validation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "schema-validation")))]
    pub fn validate(
        &self,
    ) -> Result<
        Vec<crate::schema_validation::ValidationIssue>,
        crate::schema_validation::SchemaValidationError,
    > {
        let value = self.to_json_value().map_err(|e| {
            crate::schema_validation::SchemaValidationError::InvalidPayload(e.to_string())
        })?;

        crate::schema_validation::validate_data(ERROR_RESPONSE_SCHEMA_PATH, &value)
    }

    /// Attach a `did_you_mean` context array from similarity suggestions.
    ///
    /// Returns the formatted "Did you mean ...?" message for convenience.
    #[cfg(feature = "similarity")]
    #[cfg_attr(docsrs, doc(cfg(feature = "similarity")))]
    pub fn attach_did_you_mean(&mut self, input: &str, candidates: &[&str]) -> Option<String> {
        let suggestions = crate::similarity::suggest_simple(input, candidates);
        if suggestions.is_empty() {
            return None;
        }

        let values: Vec<String> = suggestions.iter().map(|s| s.value.clone()).collect();
        self.insert_context("did_you_mean", ContextValue::StringArray(values));

        crate::similarity::format_did_you_mean(&suggestions)
    }

    /// Resolve the symbolic exit code name using the embedded Foundry catalog.
    #[cfg(any(
        feature = "foundry",
        feature = "foundry-core",
        feature = "foundry-mime-types",
        feature = "foundry-patterns",
    ))]
    #[cfg_attr(docsrs, doc(cfg(feature = "foundry-core")))]
    pub fn exit_code_name(&self) -> Option<&'static str> {
        crate::foundry::exit_codes::get_exit_name(i32::from(self.exit_code?))
    }
}

/// Options for wrapping base errors.
#[derive(Debug, Default, Clone)]
pub struct WrapOptions {
    /// Optional severity name.
    pub severity: Option<Severity>,
    /// Optional numeric severity.
    pub severity_level: Option<u8>,
    /// Optional correlation identifier.
    pub correlation_id: Option<String>,
    /// Optional trace identifier.
    pub trace_id: Option<String>,
    /// Optional exit code.
    pub exit_code: Option<u8>,
    /// Optional structured context.
    pub context: Option<BTreeMap<String, ContextValue>>,
    /// Optional serialized original error.
    pub original: Option<OriginalValue>,
}

/// Emit the error as JSON and exit the current process.
///
/// This helper is intentionally simple: it prints the JSON payload to stderr.
/// Callers that require structured logging should integrate with their logging
/// stack directly.
pub fn exit_with_error(exit_code: u8, mut error: ErrorResponse) -> ! {
    error.exit_code = Some(exit_code);

    let json = error
        .to_json_string()
        .unwrap_or_else(|e| format!("{{\"code\":\"SERIALIZATION_FAILED\",\"message\":\"{e}\"}}"));

    eprintln!("{json}");
    std::process::exit(i32::from(exit_code));
}

fn now_rfc3339() -> Result<String, ErrorHandlingError> {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| ErrorHandlingError::TimestampFormat(e.to_string()))?;

    let secs = dur.as_secs() as i64;
    let days = secs.div_euclid(86_400);
    let secs_of_day = secs.rem_euclid(86_400) as u32;

    let (year, month, day) = civil_from_days(days);

    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;

    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    ))
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u8, u8) {
    let z = days_since_unix_epoch + 719_468;

    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };

    (year as i32, m as u8, d as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_minimal_new_serializes_as_pathfinder_payload() {
        let err = ErrorResponse::new("CONFIG_INVALID", "Config load failed");
        let json = err.to_json_value().unwrap();

        assert_eq!(
            json,
            serde_json::json!({
                "code": "CONFIG_INVALID",
                "message": "Config load failed"
            })
        );
    }

    #[test]
    fn test_wrap_sets_severity_and_exit_code() {
        let base = PathfinderErrorResponse::new("CONFIG_INVALID", "Config load failed");

        let err = ErrorResponse::wrap(
            base,
            WrapOptions {
                severity: Some(Severity::High),
                exit_code: Some(20),
                correlation_id: Some("corr-123".to_string()),
                ..WrapOptions::default()
            },
        )
        .unwrap();

        assert_eq!(err.severity, Some(Severity::High));
        assert_eq!(err.severity_level, Some(3));
        assert_eq!(err.exit_code, Some(20));
        assert_eq!(err.correlation_id.as_deref(), Some("corr-123"));
    }

    #[test]
    fn test_set_exit_code_i32_rejects_out_of_range() {
        let mut err = ErrorResponse::new("X", "Y");

        let e = err.set_exit_code_i32(256).unwrap_err();
        assert_eq!(e, ErrorHandlingError::ExitCodeOutOfRange(256));

        let e = err.set_exit_code_i32(-1).unwrap_err();
        assert_eq!(e, ErrorHandlingError::ExitCodeOutOfRange(-1));
    }

    #[cfg(feature = "similarity")]
    #[test]
    fn test_attach_did_you_mean_populates_context() {
        let mut err = ErrorResponse::new("INVALID_ARGUMENT", "Unknown command");
        let msg = err
            .attach_did_you_mean("buidl", &["build", "test", "lint"])
            .expect("should suggest");

        assert!(msg.starts_with("Did you mean"));

        let ctx = err.context.as_ref().unwrap();
        let ContextValue::StringArray(values) = ctx.get("did_you_mean").unwrap() else {
            panic!("expected did_you_mean array");
        };
        assert!(values.iter().any(|v| v == "build"));
    }

    #[cfg(any(
        feature = "foundry",
        feature = "foundry-core",
        feature = "foundry-mime-types",
        feature = "foundry-patterns",
    ))]
    #[test]
    fn test_exit_code_name_uses_foundry_catalog() {
        let mut err = ErrorResponse::new("CONFIG_INVALID", "Config load failed");
        err.exit_code = Some(20);
        assert_eq!(err.exit_code_name(), Some("EXIT_CONFIG_INVALID"));
    }

    #[cfg(feature = "schema-validation")]
    #[test]
    fn test_fulmen_schema_accepts_pathfinder_base_fixture() {
        let fixture = include_str!("../../tests/fixtures/errors/pathfinder-base.json");
        let value: Value = serde_json::from_str(fixture).unwrap();

        let issues = crate::schema_validation::validate_data(ERROR_RESPONSE_SCHEMA_PATH, &value)
            .expect("schema validation should run");

        assert!(issues.is_empty(), "expected no issues, got {issues:?}");
    }

    #[cfg(feature = "schema-validation")]
    #[test]
    fn test_error_response_validate_passes_for_extended_fixture() {
        let fixture = include_str!("../../tests/fixtures/errors/fulmen-extended.json");
        let mut err: ErrorResponse = serde_json::from_str(fixture).unwrap();

        // Ensure our struct supports unknown additionalProperties.
        err.extra
            .insert("custom".to_string(), Value::String("ok".to_string()));

        let issues = err.validate().unwrap();
        assert!(issues.is_empty(), "expected no issues, got {issues:?}");
    }

    #[cfg(feature = "schema-validation")]
    #[test]
    fn test_error_response_validate_reports_missing_required() {
        let value = serde_json::json!({"message": "missing code"});
        let issues = crate::schema_validation::validate_data(ERROR_RESPONSE_SCHEMA_PATH, &value)
            .expect("schema validation should run");

        assert!(!issues.is_empty());
    }

    #[test]
    fn test_set_severity_level_derives_severity() {
        let mut err = ErrorResponse::new("X", "Y");
        err.set_severity_level(4).unwrap();
        assert_eq!(err.severity, Some(Severity::Critical));
        assert_eq!(err.severity_level, Some(4));

        let e = err.set_severity_level(5).unwrap_err();
        assert_eq!(e, ErrorHandlingError::SeverityLevelOutOfRange(5));
    }

    #[test]
    fn test_pathfinder_error_response_roundtrip() {
        let base = PathfinderErrorResponse::new("CONFIG_INVALID", "Config load failed");
        let json = serde_json::to_string(&base).unwrap();
        let parsed: PathfinderErrorResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, base);
    }

    #[test]
    fn test_original_value_serializes_as_object() {
        let mut err = ErrorResponse::new("X", "Y");
        err.original = Some(OriginalValue::Object(BTreeMap::from([(
            "message".to_string(),
            Value::String("inner".to_string()),
        )])));

        let json = err.to_json_value().unwrap();
        assert_eq!(json["original"]["message"], "inner");
    }

    #[test]
    fn test_timestamp_helpers_generate_rfc3339() {
        let mut base = PathfinderErrorResponse::new("X", "Y");
        base.set_timestamp_now_utc().unwrap();

        let ts = base.timestamp.as_deref().unwrap();
        assert_eq!(ts.len(), 20);
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
        assert_eq!(&ts[13..14], ":");
        assert_eq!(&ts[16..17], ":");
        assert!(ts.ends_with('Z'));

        let mut err = ErrorResponse::new("X", "Y");
        err.set_timestamp_now_utc().unwrap();
        assert!(err.timestamp.as_deref().unwrap().ends_with('Z'));
    }

    #[test]
    fn test_details_serializes_as_object() {
        let mut err = ErrorResponse::new("X", "Y");
        err.details = Some(BTreeMap::from([(
            "foo".to_string(),
            Value::String("bar".to_string()),
        )]));

        let json = err.to_json_value().unwrap();
        assert_eq!(json["details"]["foo"], "bar");
    }

    #[test]
    fn test_context_value_number_serializes() {
        let mut err = ErrorResponse::new("X", "Y");
        err.insert_context("attempt", 2);

        let json = err.to_json_value().unwrap();
        assert_eq!(json["context"]["attempt"], 2);
    }

    #[test]
    fn test_extra_fields_serialize_and_roundtrip() {
        let mut err = ErrorResponse::new("X", "Y");
        err.extra
            .insert("custom".to_string(), Value::String("ok".to_string()));

        let json = err.to_json_value().unwrap();
        assert_eq!(json["custom"], "ok");

        let parsed: ErrorResponse = serde_json::from_value(json).unwrap();
        assert_eq!(
            parsed.extra.get("custom"),
            Some(&Value::String("ok".to_string()))
        );
    }
}
