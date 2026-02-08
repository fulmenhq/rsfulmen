//! Structured logging with progressive profiles.
//!
//! This module provides a structured logger that matches the Crucible observability
//! taxonomy. It exposes rsfulmen's own types for cross-language API consistency
//! with tsfulmen and pyfulmen.
//!
//! ## Profiles
//!
//! - [`Profile::Simple`] -- Human-readable, space-aligned console output on stderr.
//! - [`Profile::Structured`] -- JSON lines for machine consumption (log aggregators,
//!   cloud logging).
//!
//! ## Quick Start
//!
//! ```rust
//! use rsfulmen::logging::{new_cli, Severity};
//!
//! let log = new_cli("myapp", Severity::Info);
//! log.info("server started", &[("port", "8080")]);
//! log.warn("high latency", &[("method", "GET"), ("duration_ms", "523")]);
//! ```
//!
//! ## Child Loggers
//!
//! Use [`Logger::with_fields`] and [`Logger::with_component`] to create child
//! loggers that carry additional context:
//!
//! ```rust
//! use rsfulmen::logging::{new_cli, Severity};
//!
//! let log = new_cli("myapp", Severity::Debug);
//! let req_log = log.with_fields(&[("request_id", "abc-123")]);
//! req_log.info("handling request", &[("path", "/api/health")]);
//!
//! let db_log = log.with_component("db");
//! db_log.debug("query executed", &[("rows", "42")]);
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Log severity levels matching the Crucible observability taxonomy.
///
/// Variants are ordered from least to most severe. The discriminant values
/// are spaced by 10 to leave room for future intermediate levels while
/// remaining compatible with the cross-language Fulmen convention.
///
/// [`Severity::None`] is a sentinel that disables all logging when used as
/// the minimum level in a [`LoggerConfig`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Severity {
    /// Extremely verbose tracing information.
    Trace = 0,
    /// Diagnostic information useful during development.
    Debug = 10,
    /// Normal operational messages.
    Info = 20,
    /// Potentially harmful situations.
    Warn = 30,
    /// Error events that might still allow the application to continue.
    Error = 40,
    /// Severe error events that will likely cause the application to abort.
    Fatal = 50,
    /// Sentinel level: disables all logging output.
    None = 60,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Severity::Trace => "TRACE",
            Severity::Debug => "DEBUG",
            Severity::Info => "INFO",
            Severity::Warn => "WARN",
            Severity::Error => "ERROR",
            Severity::Fatal => "FATAL",
            Severity::None => "NONE",
        };
        f.write_str(label)
    }
}

/// Logging profile determining output format.
///
/// The profile controls how log records are serialized when written to the
/// output sink. Both profiles emit one record per line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Profile {
    /// Human-readable, space-aligned console output.
    Simple,
    /// JSON lines for machine consumption.
    Structured,
}

/// Logger configuration.
///
/// Combines the application name, minimum severity filter, and output profile
/// into a single value that can be serialized/deserialized for config files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    /// Application or component name emitted in every log record.
    pub name: String,
    /// Minimum severity to emit. Messages below this level are discarded.
    pub level: Severity,
    /// Output profile controlling the serialization format.
    pub profile: Profile,
}

// ---------------------------------------------------------------------------
// Logger
// ---------------------------------------------------------------------------

/// The structured logger.
///
/// Thread-safe ([`Send`] + [`Sync`]). Cloneable for sharing across threads.
/// Cloned loggers share the same underlying writer so output remains
/// interleaved-safe.
///
/// By default writes to stderr. For testing, accepts a custom writer via the
/// internal [`Logger::with_writer`] constructor.
#[derive(Clone)]
pub struct Logger {
    config: LoggerConfig,
    fields: Vec<(String, String)>,
    component: Option<String>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

// Manual Debug because `dyn Write` is not Debug.
impl fmt::Debug for Logger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Logger")
            .field("config", &self.config)
            .field("fields", &self.fields)
            .field("component", &self.component)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// Free-standing constructors (module-level API)
// ---------------------------------------------------------------------------

/// Create a logger for CLI usage with the [`Profile::Simple`] profile and
/// stderr output.
///
/// This is the recommended constructor for command-line tools that emit
/// human-readable diagnostics.
///
/// # Examples
///
/// ```rust
/// use rsfulmen::logging::{new_cli, Severity};
///
/// let log = new_cli("mytool", Severity::Info);
/// log.info("starting", &[]);
/// ```
pub fn new_cli(name: &str, level: Severity) -> Logger {
    let config = LoggerConfig {
        name: name.to_string(),
        level,
        profile: Profile::Simple,
    };
    new(config)
}

/// Create a logger with full configuration, writing to stderr.
///
/// # Examples
///
/// ```rust
/// use rsfulmen::logging::{new, LoggerConfig, Severity, Profile};
///
/// let config = LoggerConfig {
///     name: "myservice".to_string(),
///     level: Severity::Debug,
///     profile: Profile::Structured,
/// };
/// let log = new(config);
/// log.debug("booting", &[("version", "0.1.2")]);
/// ```
pub fn new(config: LoggerConfig) -> Logger {
    Logger {
        config,
        fields: Vec::new(),
        component: None,
        writer: Arc::new(Mutex::new(Box::new(std::io::stderr()))),
    }
}

/// Return default configuration for a given application name.
///
/// Defaults:
/// - level: [`Severity::Info`]
/// - profile: [`Profile::Simple`]
///
/// # Examples
///
/// ```rust
/// use rsfulmen::logging::{default_config, Severity, Profile};
///
/// let cfg = default_config("myapp");
/// assert_eq!(cfg.name, "myapp");
/// assert_eq!(cfg.level, Severity::Info);
/// assert_eq!(cfg.profile, Profile::Simple);
/// ```
pub fn default_config(name: &str) -> LoggerConfig {
    LoggerConfig {
        name: name.to_string(),
        level: Severity::Info,
        profile: Profile::Simple,
    }
}

// ---------------------------------------------------------------------------
// Logger implementation
// ---------------------------------------------------------------------------

impl Logger {
    /// Create a logger that writes to a custom writer (for testing).
    ///
    /// The writer is wrapped in an `Arc<Mutex<...>>` so that child loggers
    /// created via [`with_fields`](Logger::with_fields) or
    /// [`with_component`](Logger::with_component) share the same output.
    #[cfg(test)]
    fn with_writer<W: Write + Send + 'static>(config: LoggerConfig, writer: W) -> Self {
        Logger {
            config,
            fields: Vec::new(),
            component: None,
            writer: Arc::new(Mutex::new(Box::new(writer))),
        }
    }

    // -- logging methods ---------------------------------------------------

    /// Emit a log record at [`Severity::Trace`] level.
    pub fn trace(&self, msg: &str, fields: &[(&str, &str)]) {
        self.log(Severity::Trace, msg, fields);
    }

    /// Emit a log record at [`Severity::Debug`] level.
    pub fn debug(&self, msg: &str, fields: &[(&str, &str)]) {
        self.log(Severity::Debug, msg, fields);
    }

    /// Emit a log record at [`Severity::Info`] level.
    pub fn info(&self, msg: &str, fields: &[(&str, &str)]) {
        self.log(Severity::Info, msg, fields);
    }

    /// Emit a log record at [`Severity::Warn`] level.
    pub fn warn(&self, msg: &str, fields: &[(&str, &str)]) {
        self.log(Severity::Warn, msg, fields);
    }

    /// Emit a log record at [`Severity::Error`] level.
    pub fn error(&self, msg: &str, fields: &[(&str, &str)]) {
        self.log(Severity::Error, msg, fields);
    }

    // -- child loggers -----------------------------------------------------

    /// Return a child logger with additional fields appended.
    ///
    /// The child shares the same writer as the parent. Fields from the parent
    /// are preserved and appear before the new ones.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rsfulmen::logging::{new_cli, Severity};
    ///
    /// let log = new_cli("myapp", Severity::Debug);
    /// let child = log.with_fields(&[("request_id", "abc-123")]);
    /// child.info("handled", &[("status", "200")]);
    /// ```
    pub fn with_fields(&self, fields: &[(&str, &str)]) -> Logger {
        let mut new_fields = self.fields.clone();
        for (k, v) in fields {
            new_fields.push(((*k).to_string(), (*v).to_string()));
        }
        Logger {
            config: self.config.clone(),
            fields: new_fields,
            component: self.component.clone(),
            writer: Arc::clone(&self.writer),
        }
    }

    /// Return a child logger tagged with a component name.
    ///
    /// The component name is included in the logger name field as
    /// `name:component`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rsfulmen::logging::{new_cli, Severity};
    ///
    /// let log = new_cli("myapp", Severity::Debug);
    /// let db = log.with_component("db");
    /// db.debug("pool ready", &[("size", "10")]);
    /// ```
    pub fn with_component(&self, component: &str) -> Logger {
        Logger {
            config: self.config.clone(),
            fields: self.fields.clone(),
            component: Some(component.to_string()),
            writer: Arc::clone(&self.writer),
        }
    }

    /// Flush buffered output to the underlying writer.
    pub fn sync(&self) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.flush();
        }
    }

    // -- internal ----------------------------------------------------------

    /// Resolved logger name, incorporating the component if present.
    fn logger_name(&self) -> String {
        match &self.component {
            Some(c) => format!("{}:{}", self.config.name, c),
            None => self.config.name.clone(),
        }
    }

    /// Core log dispatch. Checks severity filter, formats, and writes.
    fn log(&self, severity: Severity, msg: &str, extra_fields: &[(&str, &str)]) {
        if severity < self.config.level {
            return;
        }

        let timestamp = format_rfc3339(SystemTime::now());
        let logger_name = self.logger_name();

        let line = match self.config.profile {
            Profile::Simple => {
                self.format_simple(&timestamp, severity, &logger_name, msg, extra_fields)
            }
            Profile::Structured => {
                self.format_structured(&timestamp, severity, &logger_name, msg, extra_fields)
            }
        };

        if let Ok(mut w) = self.writer.lock() {
            let _ = writeln!(w, "{}", line);
        }
    }

    /// Format a record in the Simple (human-readable) profile.
    ///
    /// Format: `{timestamp} {LEVEL:5} [{name}] {message} {key=value}...`
    fn format_simple(
        &self,
        timestamp: &str,
        severity: Severity,
        logger_name: &str,
        msg: &str,
        extra_fields: &[(&str, &str)],
    ) -> String {
        let level_str = format!("{:<5}", severity);

        let mut buf = format!("{} {} [{}] {}", timestamp, level_str, logger_name, msg);

        // Append base fields (from with_fields)
        for (k, v) in &self.fields {
            buf.push(' ');
            append_kv_simple(&mut buf, k, v);
        }

        // Append per-call fields
        for (k, v) in extra_fields {
            buf.push(' ');
            append_kv_simple(&mut buf, k, v);
        }

        buf
    }

    /// Format a record in the Structured (JSON) profile.
    fn format_structured(
        &self,
        timestamp: &str,
        severity: Severity,
        logger_name: &str,
        msg: &str,
        extra_fields: &[(&str, &str)],
    ) -> String {
        let mut map = serde_json::Map::new();

        // Append base fields (from with_fields)
        for (k, v) in &self.fields {
            if is_structured_canonical_key(k) {
                continue;
            }
            map.insert(k.clone(), serde_json::Value::String(v.clone()));
        }

        // Append per-call fields
        for (k, v) in extra_fields {
            if is_structured_canonical_key(k) {
                continue;
            }
            map.insert(
                (*k).to_string(),
                serde_json::Value::String((*v).to_string()),
            );
        }

        map.insert(
            "timestamp".to_string(),
            serde_json::Value::String(timestamp.to_string()),
        );
        map.insert(
            "level".to_string(),
            serde_json::Value::String(severity.to_string()),
        );
        map.insert(
            "logger".to_string(),
            serde_json::Value::String(logger_name.to_string()),
        );
        map.insert(
            "message".to_string(),
            serde_json::Value::String(msg.to_string()),
        );

        if let Some(component) = &self.component {
            map.insert(
                "component".to_string(),
                serde_json::Value::String(component.clone()),
            );
        }

        serde_json::to_string(&serde_json::Value::Object(map)).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Append a `key=value` pair in simple format. Values containing spaces are
/// quoted with double-quotes.
fn append_kv_simple(buf: &mut String, key: &str, value: &str) {
    if should_quote_simple_value(value) {
        buf.push_str(key);
        buf.push_str("=\"");
        buf.push_str(&escape_simple_value(value));
        buf.push('"');
    } else {
        buf.push_str(&format!("{}={}", key, value));
    }
}

/// Canonical structured output keys that caller fields must not overwrite.
fn is_structured_canonical_key(key: &str) -> bool {
    matches!(key, "timestamp" | "level" | "logger" | "message")
}

/// Decide whether a simple-profile value requires quoting.
fn should_quote_simple_value(value: &str) -> bool {
    value.chars().any(|c| {
        c.is_whitespace()
            || c == '"'
            || c == '\\'
            || c == '\n'
            || c == '\r'
            || c == '\t'
            || c.is_control()
    })
}

/// Escape special characters for simple-profile quoted values.
fn escape_simple_value(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04x}", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped
}

/// Format a [`SystemTime`] as an RFC 3339 UTC timestamp string.
///
/// Produces output like `2026-02-08T10:30:00Z`. Fractional seconds are
/// omitted for brevity (consistent across Simple and Structured profiles).
fn format_rfc3339(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();

    // Civil time decomposition from Unix timestamp (UTC).
    // Uses the well-known algorithm by Howard Hinnant for days -> y/m/d.
    let days_since_epoch = (secs / 86400) as i64;
    let time_of_day = secs % 86400;

    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    // Convert days since 1970-01-01 to (year, month, day).
    // Shift epoch to 0000-03-01 to simplify leap-year handling.
    let z = days_since_epoch + 719468; // days from 0000-03-01 to 1970-01-01
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // month index from March [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hour, minute, second
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Clonable writer that delegates to a shared `Vec<u8>` buffer.
    #[derive(Clone)]
    struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

    impl SharedBuffer {
        fn new() -> Self {
            SharedBuffer(Arc::new(Mutex::new(Vec::new())))
        }

        /// Read the accumulated contents as a UTF-8 string.
        fn contents(&self) -> String {
            String::from_utf8_lossy(&self.0.lock().unwrap()).to_string()
        }
    }

    impl Write for SharedBuffer {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.0.lock().unwrap().flush()
        }
    }

    /// Build a Logger backed by the provided shared buffer.
    fn logger_with_buffer(
        name: &str,
        level: Severity,
        profile: Profile,
        buf: &SharedBuffer,
    ) -> Logger {
        Logger::with_writer(
            LoggerConfig {
                name: name.to_string(),
                level,
                profile,
            },
            buf.clone(),
        )
    }

    // -- construction ------------------------------------------------------

    #[test]
    fn test_new_cli_creates_logger() {
        let log = new_cli("testapp", Severity::Info);
        assert_eq!(log.config.name, "testapp");
        assert_eq!(log.config.level, Severity::Info);
        assert_eq!(log.config.profile, Profile::Simple);
    }

    #[test]
    fn test_new_with_config() {
        let config = LoggerConfig {
            name: "custom".to_string(),
            level: Severity::Debug,
            profile: Profile::Structured,
        };
        let log = new(config);
        assert_eq!(log.config.name, "custom");
        assert_eq!(log.config.level, Severity::Debug);
        assert_eq!(log.config.profile, Profile::Structured);
    }

    #[test]
    fn test_default_config() {
        let cfg = default_config("myapp");
        assert_eq!(cfg.name, "myapp");
        assert_eq!(cfg.level, Severity::Info);
        assert_eq!(cfg.profile, Profile::Simple);
    }

    // -- severity ----------------------------------------------------------

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Trace < Severity::Debug);
        assert!(Severity::Debug < Severity::Info);
        assert!(Severity::Info < Severity::Warn);
        assert!(Severity::Warn < Severity::Error);
        assert!(Severity::Error < Severity::Fatal);
        assert!(Severity::Fatal < Severity::None);
    }

    #[test]
    fn test_severity_values() {
        assert_eq!(Severity::Trace as u8, 0);
        assert_eq!(Severity::Debug as u8, 10);
        assert_eq!(Severity::Info as u8, 20);
        assert_eq!(Severity::Warn as u8, 30);
        assert_eq!(Severity::Error as u8, 40);
        assert_eq!(Severity::Fatal as u8, 50);
        assert_eq!(Severity::None as u8, 60);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::Trace.to_string(), "TRACE");
        assert_eq!(Severity::Debug.to_string(), "DEBUG");
        assert_eq!(Severity::Info.to_string(), "INFO");
        assert_eq!(Severity::Warn.to_string(), "WARN");
        assert_eq!(Severity::Error.to_string(), "ERROR");
        assert_eq!(Severity::Fatal.to_string(), "FATAL");
        assert_eq!(Severity::None.to_string(), "NONE");
    }

    #[test]
    fn test_severity_roundtrip_json() {
        let variants = [
            Severity::Trace,
            Severity::Debug,
            Severity::Info,
            Severity::Warn,
            Severity::Error,
            Severity::Fatal,
            Severity::None,
        ];
        for severity in variants {
            let json = serde_json::to_string(&severity).expect("serialize severity");
            let back: Severity = serde_json::from_str(&json).expect("deserialize severity");
            assert_eq!(back, severity, "roundtrip failed for {severity:?}");
        }
    }

    #[test]
    fn test_profile_roundtrip_json() {
        for profile in [Profile::Simple, Profile::Structured] {
            let json = serde_json::to_string(&profile).expect("serialize profile");
            let back: Profile = serde_json::from_str(&json).expect("deserialize profile");
            assert_eq!(back, profile, "roundtrip failed for {profile:?}");
        }
    }

    #[test]
    fn test_config_roundtrip_yaml() {
        let config = LoggerConfig {
            name: "myservice".to_string(),
            level: Severity::Warn,
            profile: Profile::Structured,
        };
        let yaml = serde_yaml::to_string(&config).expect("serialize config");
        let back: LoggerConfig = serde_yaml::from_str(&yaml).expect("deserialize config");
        assert_eq!(back.name, config.name);
        assert_eq!(back.level, config.level);
        assert_eq!(back.profile, config.profile);
    }

    // -- level filtering ---------------------------------------------------

    #[test]
    fn test_messages_below_level_not_emitted() {
        let buf = SharedBuffer::new();
        let log = logger_with_buffer("app", Severity::Info, Profile::Simple, &buf);
        log.debug("should be filtered", &[]);
        assert!(
            buf.contents().is_empty(),
            "debug message should not be emitted when level is info"
        );
    }

    #[test]
    fn test_messages_at_level_are_emitted() {
        let buf = SharedBuffer::new();
        let log = logger_with_buffer("app", Severity::Info, Profile::Simple, &buf);
        log.info("visible", &[]);
        let output = buf.contents();
        assert!(
            !output.is_empty(),
            "info message should be emitted when level is info"
        );
        assert!(output.contains("visible"));
    }

    #[test]
    fn test_messages_above_level_are_emitted() {
        let buf = SharedBuffer::new();
        let log = logger_with_buffer("app", Severity::Info, Profile::Simple, &buf);
        log.error("boom", &[]);
        let output = buf.contents();
        assert!(
            !output.is_empty(),
            "error message should be emitted when level is info"
        );
        assert!(output.contains("boom"));
    }

    // -- output formats ----------------------------------------------------

    #[test]
    fn test_simple_profile_output_format() {
        let buf = SharedBuffer::new();
        let log = logger_with_buffer("myapp", Severity::Trace, Profile::Simple, &buf);
        log.info("server started", &[("port", "8080")]);
        let output = buf.contents();

        assert!(output.contains("INFO"), "output should contain level");
        assert!(output.contains("[myapp]"), "output should contain [name]");
        assert!(
            output.contains("server started"),
            "output should contain message"
        );
        assert!(output.contains("port=8080"), "output should contain fields");
    }

    #[test]
    fn test_structured_profile_outputs_json() {
        let buf = SharedBuffer::new();
        let log = logger_with_buffer("myapp", Severity::Trace, Profile::Structured, &buf);
        log.info("server started", &[("port", "8080")]);
        let output = buf.contents();

        let parsed: serde_json::Value =
            serde_json::from_str(output.trim()).expect("output should be valid JSON");
        let obj = parsed.as_object().expect("should be a JSON object");

        assert_eq!(obj.get("level").and_then(|v| v.as_str()), Some("INFO"));
        assert_eq!(obj.get("logger").and_then(|v| v.as_str()), Some("myapp"));
        assert_eq!(
            obj.get("message").and_then(|v| v.as_str()),
            Some("server started")
        );
        assert_eq!(obj.get("port").and_then(|v| v.as_str()), Some("8080"));
        assert!(
            obj.contains_key("timestamp"),
            "JSON output should contain timestamp"
        );
    }

    #[test]
    fn test_structured_profile_emits_component_field() {
        let buf = SharedBuffer::new();
        let log = logger_with_buffer("myapp", Severity::Trace, Profile::Structured, &buf);
        let child = log.with_component("db");
        child.info("connected", &[]);
        let output = buf.contents();

        let parsed: serde_json::Value =
            serde_json::from_str(output.trim()).expect("output should be valid JSON");
        let obj = parsed.as_object().expect("should be a JSON object");
        assert_eq!(obj.get("component").and_then(|v| v.as_str()), Some("db"));
    }

    #[test]
    fn test_structured_canonical_fields_cannot_be_overwritten() {
        let buf = SharedBuffer::new();
        let log = logger_with_buffer("myapp", Severity::Trace, Profile::Structured, &buf)
            .with_fields(&[
                ("timestamp", "base-ts"),
                ("level", "DEBUG"),
                ("logger", "base-logger"),
                ("message", "base-message"),
            ]);

        log.info(
            "real-message",
            &[
                ("timestamp", "call-ts"),
                ("level", "TRACE"),
                ("logger", "call-logger"),
                ("message", "call-message"),
            ],
        );
        let output = buf.contents();

        let parsed: serde_json::Value =
            serde_json::from_str(output.trim()).expect("output should be valid JSON");
        let obj = parsed.as_object().expect("should be a JSON object");

        assert_eq!(obj.get("level").and_then(|v| v.as_str()), Some("INFO"));
        assert_eq!(obj.get("logger").and_then(|v| v.as_str()), Some("myapp"));
        assert_eq!(
            obj.get("message").and_then(|v| v.as_str()),
            Some("real-message")
        );
        assert_ne!(
            obj.get("timestamp").and_then(|v| v.as_str()),
            Some("base-ts")
        );
        assert_ne!(
            obj.get("timestamp").and_then(|v| v.as_str()),
            Some("call-ts")
        );
    }

    #[test]
    fn test_fields_appear_in_output() {
        let buf = SharedBuffer::new();
        let log = logger_with_buffer("app", Severity::Trace, Profile::Simple, &buf);
        log.info("msg", &[("region", "us-east-1"), ("count", "42")]);
        let output = buf.contents();

        assert!(output.contains("region=us-east-1"));
        assert!(output.contains("count=42"));
    }

    // -- child loggers -----------------------------------------------------

    #[test]
    fn test_with_fields_returns_new_logger() {
        let buf = SharedBuffer::new();
        let parent = logger_with_buffer("app", Severity::Trace, Profile::Simple, &buf);
        let child = parent.with_fields(&[("request_id", "abc")]);

        // Child has the field, parent does not.
        assert!(child.fields.iter().any(|(k, _)| k == "request_id"));
        assert!(!parent.fields.iter().any(|(k, _)| k == "request_id"));

        child.info("handled", &[]);
        let output = buf.contents();
        assert!(
            output.contains("request_id=abc"),
            "child output should contain inherited fields"
        );
    }

    #[test]
    fn test_with_component_in_output() {
        let buf = SharedBuffer::new();
        let log = logger_with_buffer("myapp", Severity::Trace, Profile::Simple, &buf);
        let child = log.with_component("db");
        child.info("connected", &[]);
        let output = buf.contents();
        assert!(
            output.contains("[myapp:db]"),
            "output should contain name:component"
        );
    }

    // -- edge cases --------------------------------------------------------

    #[test]
    fn test_level_none_disables_all() {
        let buf = SharedBuffer::new();
        let log = logger_with_buffer("app", Severity::None, Profile::Simple, &buf);
        log.trace("t", &[]);
        log.debug("d", &[]);
        log.info("i", &[]);
        log.warn("w", &[]);
        log.error("e", &[]);
        assert!(
            buf.contents().is_empty(),
            "no messages should be emitted when level is None"
        );
    }

    // -- with_writer -------------------------------------------------------

    #[test]
    fn test_with_writer_constructor() {
        let buf: Vec<u8> = Vec::new();
        let config = LoggerConfig {
            name: "test".to_string(),
            level: Severity::Info,
            profile: Profile::Simple,
        };
        let log = Logger::with_writer(config, buf);
        // Should not panic; writer is usable.
        log.info("hello", &[]);
    }

    // -- timestamp ---------------------------------------------------------

    #[test]
    fn test_rfc3339_known_epoch() {
        let ts = format_rfc3339(UNIX_EPOCH);
        assert_eq!(ts, "1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_rfc3339_format_shape() {
        let ts = format_rfc3339(SystemTime::now());
        // Should look like "2026-02-08T...Z"
        assert!(ts.ends_with('Z'), "timestamp should end with Z");
        assert_eq!(
            ts.len(),
            20,
            "timestamp should be 20 chars (YYYY-MM-DDThh:mm:ssZ)"
        );
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
        assert_eq!(&ts[13..14], ":");
        assert_eq!(&ts[16..17], ":");
    }

    // -- simple format quoting ---------------------------------------------

    #[test]
    fn test_simple_format_quotes_values_with_spaces() {
        let buf = SharedBuffer::new();
        let log = logger_with_buffer("app", Severity::Trace, Profile::Simple, &buf);
        log.info("msg", &[("desc", "hello world")]);
        let output = buf.contents();
        assert!(
            output.contains("desc=\"hello world\""),
            "values with spaces should be quoted"
        );
    }

    #[test]
    fn test_simple_format_escapes_quotes_and_newlines() {
        let buf = SharedBuffer::new();
        let log = logger_with_buffer("app", Severity::Trace, Profile::Simple, &buf);
        log.info("msg", &[("desc", "hello \"quoted\"\nworld")]);
        let output = buf.contents();
        assert!(output.contains("desc=\"hello \\\"quoted\\\"\\nworld\""));
    }
}
