//! Logging middleware pipeline (redaction, sensitive keys, correlation, throttle).
//!
//! Crucible describes Go `func(*Event) skip` and TS `(event) => event | null`.
//! Rust uses a [`Middleware`] trait over an owned [`LogRecord`]. Middleware is
//! opt-in and does not change the default logger.

use std::sync::Mutex;
use std::time::Instant;

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::Severity;

/// Replacement used when a secret is stripped from a field or message.
pub const DEFAULT_REDACTION_REPLACEMENT: &str = "[REDACTED]";

/// Default field names (case-insensitive) aligned with gofulmen / tsfulmen.
pub const DEFAULT_SECRET_KEYS: &[&str] = &[
    "password",
    "token",
    "apikey",
    "api_key",
    "authorization",
    "secret",
    "cardnumber",
    "card_number",
    "cvv",
    "ssn",
    "accesstoken",
    "access_token",
    "refreshtoken",
    "refresh_token",
];

/// Default regex patterns aligned with gofulmen / tsfulmen.
pub const DEFAULT_SECRET_PATTERNS: &[&str] = &[
    r"SECRET_[A-Z0-9_]+",
    r"[A-Z0-9_]*TOKEN[A-Z0-9_]*",
    r"[A-Z0-9_]*KEY[A-Z0-9_]*",
    r"[A-Za-z0-9+/]{40,}={0,2}",
    r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
    r"\b\d{13,19}\b",
];

const MAX_PATTERN_SCAN_SIZE: usize = 10 * 1024;

/// Outcome of one middleware step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiddlewareAction {
    /// Continue the chain and emit if nothing later drops the record.
    Emit,
    /// Discard the record (throttle / filter).
    Drop,
}

/// Structured record passed through the middleware chain before formatting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    /// RFC3339 timestamp.
    pub timestamp: String,
    /// Severity of the record.
    pub severity: Severity,
    /// Human-readable message (may be redacted).
    pub message: String,
    /// Logger name (`service` or `service:component`).
    pub logger: String,
    /// Optional component tag.
    pub component: Option<String>,
    /// Structured fields (key/value). Values may be redacted in place.
    pub fields: Vec<(String, String)>,
    /// Cross-service correlation id (UUIDv7 when `foundry-correlation` is on).
    pub correlation_id: Option<String>,
    /// Redaction indicators (`redacted`, later `pii` / `secret`).
    pub redaction_flags: Vec<String>,
    /// Set when a throttle drop is applied (or a near-drop annotation).
    pub throttle_bucket: Option<String>,
}

/// Processor applied to each [`LogRecord`] before emission.
pub trait Middleware: Send + Sync {
    /// Mutate `record`. Return [`MiddlewareAction::Drop`] to skip the sink.
    fn process(&self, record: &mut LogRecord) -> MiddlewareAction;
}

/// Ordered middleware chain. Cheap to clone (`Arc` internally).
#[derive(Clone, Default)]
pub struct MiddlewareChain {
    steps: Vec<std::sync::Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    /// Empty chain (current default logger behavior).
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Append a step. Lower-priority / earlier steps should be pushed first
    /// (Crucible: lower `priority` runs first; redaction before correlation).
    pub fn push<M: Middleware + 'static>(&mut self, step: M) {
        self.steps.push(std::sync::Arc::new(step));
    }

    /// Builder-style append.
    pub fn with<M: Middleware + 'static>(mut self, step: M) -> Self {
        self.push(step);
        self
    }

    /// Run the chain. Returns `false` when a step drops the record.
    pub fn apply(&self, record: &mut LogRecord) -> bool {
        for step in &self.steps {
            if matches!(step.process(record), MiddlewareAction::Drop) {
                return false;
            }
        }
        true
    }

    /// Number of configured steps.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Whether the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// Field-name redaction (case-insensitive). No pattern scan.
pub struct SensitiveKeyMiddleware {
    keys_lower: Vec<String>,
    replacement: String,
}

impl SensitiveKeyMiddleware {
    /// Redact the default secret key set.
    pub fn defaults() -> Self {
        Self::new(DEFAULT_SECRET_KEYS, DEFAULT_REDACTION_REPLACEMENT)
    }

    /// Redact the given keys (case-insensitive).
    pub fn new(keys: &[&str], replacement: &str) -> Self {
        Self {
            keys_lower: keys.iter().map(|k| k.to_ascii_lowercase()).collect(),
            replacement: replacement.to_string(),
        }
    }

    fn is_secret(keys_lower: &[String], key: &str) -> bool {
        let lower = key.to_ascii_lowercase();
        keys_lower.iter().any(|k| k == &lower)
    }
}

impl Middleware for SensitiveKeyMiddleware {
    fn process(&self, record: &mut LogRecord) -> MiddlewareAction {
        let mut hit = false;
        for (key, value) in &mut record.fields {
            if Self::is_secret(&self.keys_lower, key) {
                *value = self.replacement.clone();
                hit = true;
            }
        }
        if hit {
            push_flag(&mut record.redaction_flags, "redacted");
        }
        MiddlewareAction::Emit
    }
}

/// Field + pattern redaction. Secrets must never reach the sink.
pub struct RedactSecretsMiddleware {
    keys_lower: Vec<String>,
    patterns: Vec<Regex>,
    replacement: String,
}

impl RedactSecretsMiddleware {
    /// Default keys and patterns from gofulmen / tsfulmen.
    pub fn defaults() -> Result<Self, regex::Error> {
        Self::new(
            DEFAULT_SECRET_KEYS,
            DEFAULT_SECRET_PATTERNS,
            DEFAULT_REDACTION_REPLACEMENT,
        )
    }

    /// Custom keys and regex pattern strings.
    pub fn new(keys: &[&str], patterns: &[&str], replacement: &str) -> Result<Self, regex::Error> {
        let compiled = patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            keys_lower: keys.iter().map(|k| k.to_ascii_lowercase()).collect(),
            patterns: compiled,
            replacement: replacement.to_string(),
        })
    }

    fn redact_string(&self, value: &str) -> String {
        if value.len() > MAX_PATTERN_SCAN_SIZE {
            return value.to_string();
        }
        let mut out = value.to_string();
        for pattern in &self.patterns {
            out = pattern
                .replace_all(&out, self.replacement.as_str())
                .into_owned();
        }
        out
    }
}

impl Middleware for RedactSecretsMiddleware {
    fn process(&self, record: &mut LogRecord) -> MiddlewareAction {
        let mut hit = false;
        let original_message = record.message.clone();
        record.message = self.redact_string(&record.message);
        if record.message != original_message {
            hit = true;
        }
        for (key, value) in &mut record.fields {
            if SensitiveKeyMiddleware::is_secret(&self.keys_lower, key) {
                *value = self.replacement.clone();
                hit = true;
            } else {
                let redacted = self.redact_string(value);
                if redacted != *value {
                    hit = true;
                }
                *value = redacted;
            }
        }
        if hit {
            push_flag(&mut record.redaction_flags, "redacted");
        }
        MiddlewareAction::Emit
    }
}

/// Inject a correlation id when the caller omitted one.
pub struct CorrelationMiddleware {
    header_field: String,
}

impl Default for CorrelationMiddleware {
    fn default() -> Self {
        Self {
            header_field: "correlationId".to_string(),
        }
    }
}

impl CorrelationMiddleware {
    /// Use the Crucible envelope field `correlationId`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Middleware for CorrelationMiddleware {
    fn process(&self, record: &mut LogRecord) -> MiddlewareAction {
        if record.correlation_id.is_some() {
            return MiddlewareAction::Emit;
        }
        if let Some((_, value)) = record
            .fields
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("correlationid") || k == "correlation_id")
        {
            record.correlation_id = Some(value.clone());
            return MiddlewareAction::Emit;
        }
        let id = generate_correlation_id();
        record.correlation_id = Some(id.clone());
        record.fields.push((self.header_field.clone(), id));
        MiddlewareAction::Emit
    }
}

fn generate_correlation_id() -> String {
    #[cfg(feature = "foundry-correlation")]
    {
        crate::foundry::correlation::generate()
    }
    #[cfg(not(feature = "foundry-correlation"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("corr-{nanos:x}")
    }
}

/// Token-bucket throttle. Drops records when the bucket is empty.
pub struct ThrottleMiddleware {
    state: Mutex<Bucket>,
    max_rate: f64,
    burst: u32,
    bucket_name: String,
}

struct Bucket {
    tokens: f64,
    last: Instant,
}

impl ThrottleMiddleware {
    /// `max_rate` tokens per second, `burst` capacity.
    pub fn new(max_rate: f64, burst: u32) -> Self {
        let burst = burst.max(1);
        Self {
            state: Mutex::new(Bucket {
                tokens: f64::from(burst),
                last: Instant::now(),
            }),
            max_rate: max_rate.max(0.0),
            burst,
            bucket_name: "default".to_string(),
        }
    }

    /// Label written to `throttleBucket` when a record is dropped.
    pub fn with_bucket_name(mut self, name: impl Into<String>) -> Self {
        self.bucket_name = name.into();
        self
    }
}

impl Middleware for ThrottleMiddleware {
    fn process(&self, record: &mut LogRecord) -> MiddlewareAction {
        let Ok(mut bucket) = self.state.lock() else {
            return MiddlewareAction::Emit;
        };
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(bucket.last);
        bucket.last = now;
        let refill = elapsed.as_secs_f64() * self.max_rate;
        bucket.tokens = (bucket.tokens + refill).min(f64::from(self.burst));
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            MiddlewareAction::Emit
        } else {
            record.throttle_bucket = Some(self.bucket_name.clone());
            MiddlewareAction::Drop
        }
    }
}

fn push_flag(flags: &mut Vec<String>, flag: &str) {
    if !flags.iter().any(|f| f == flag) {
        flags.push(flag.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(msg: &str, fields: &[(&str, &str)]) -> LogRecord {
        LogRecord {
            timestamp: "2026-10-12T00:00:00Z".to_string(),
            severity: Severity::Info,
            message: msg.to_string(),
            logger: "test".to_string(),
            component: None,
            fields: fields
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
            correlation_id: None,
            redaction_flags: Vec::new(),
            throttle_bucket: None,
        }
    }

    #[test]
    fn sensitive_key_never_emits_secret() {
        let mw = SensitiveKeyMiddleware::defaults();
        let mut record = rec("ok", &[("password", "hunter2"), ("user", "ada")]);
        assert_eq!(mw.process(&mut record), MiddlewareAction::Emit);
        assert_eq!(record.fields[0].1, DEFAULT_REDACTION_REPLACEMENT);
        assert_eq!(record.fields[1].1, "ada");
        assert!(!record.fields.iter().any(|(_, v)| v.contains("hunter2")));
        assert!(record.redaction_flags.contains(&"redacted".to_string()));
    }

    #[test]
    fn redact_patterns_strip_token_and_email() {
        let mw = RedactSecretsMiddleware::defaults().unwrap();
        let mut record = rec(
            "login GITHUB_TOKEN=abc user@example.com",
            &[("note", "SECRET_KEY=zzzz")],
        );
        mw.process(&mut record);
        assert!(
            !record.message.contains("GITHUB_TOKEN"),
            "message leaked: {}",
            record.message
        );
        assert!(
            !record.message.contains("user@example.com"),
            "email leaked: {}",
            record.message
        );
        assert!(
            !record.fields[0].1.contains("SECRET_KEY"),
            "field leaked: {}",
            record.fields[0].1
        );
        assert!(record.message.contains(DEFAULT_REDACTION_REPLACEMENT));
    }

    #[test]
    fn correlation_fills_missing_id() {
        let mw = CorrelationMiddleware::new();
        let mut record = rec("hi", &[]);
        mw.process(&mut record);
        assert!(record.correlation_id.is_some());
        assert!(record.fields.iter().any(|(k, _)| k == "correlationId"));
    }

    #[test]
    fn throttle_drops_after_burst() {
        let mw = ThrottleMiddleware::new(0.0, 1).with_bucket_name("test");
        let mut a = rec("a", &[]);
        let mut b = rec("b", &[]);
        assert_eq!(mw.process(&mut a), MiddlewareAction::Emit);
        assert_eq!(mw.process(&mut b), MiddlewareAction::Drop);
        assert_eq!(b.throttle_bucket.as_deref(), Some("test"));
    }

    #[test]
    fn chain_redacts_then_correlates() {
        let chain = MiddlewareChain::new()
            .with(RedactSecretsMiddleware::defaults().unwrap())
            .with(CorrelationMiddleware::new());
        let mut record = rec("ok", &[("token", "super-secret-value")]);
        assert!(chain.apply(&mut record));
        assert_eq!(record.fields[0].1, DEFAULT_REDACTION_REPLACEMENT);
        assert!(!format!("{record:?}").contains("super-secret-value"));
        assert!(record.correlation_id.is_some());
    }
}
