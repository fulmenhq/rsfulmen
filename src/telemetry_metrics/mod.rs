//! Telemetry & Metrics Export
//!
//! Implements the Crucible **Telemetry & Metrics Export** standard.
//!
//! The canonical export format is JSON, validated against
//! `schemas/crucible-rs/observability/metrics/v1.0.0/metrics-event.schema.json`.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Embedded schema path for validating metrics event payloads.
pub const METRICS_EVENT_SCHEMA_PATH: &str =
    "observability/metrics/v1.0.0/metrics-event.schema.json";

const METRICS_TAXONOMY_YAML: &str = include_str!("../../config/crucible-rs/taxonomy/metrics.yaml");

/// Errors produced by telemetry operations.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum TelemetryError {
    /// Metric identifier does not exist in the taxonomy.
    #[error("unknown metric name: {0}")]
    UnknownMetricName(String),

    /// Histogram bucket boundaries were invalid.
    #[error("invalid histogram buckets: {0}")]
    InvalidHistogramBuckets(String),

    /// Timestamp formatting failed.
    #[error("timestamp formatting failed: {0}")]
    TimestampFormat(String),
}

/// Metric unit (aligned with the metrics taxonomy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricUnit {
    /// Count.
    Count,
    /// Milliseconds.
    Ms,
    /// Bytes.
    Bytes,
    /// Percent.
    Percent,
    /// Seconds.
    S,
}

impl MetricUnit {
    fn from_taxonomy_unit(unit: &str) -> Option<Self> {
        match unit {
            "count" => Some(MetricUnit::Count),
            "ms" => Some(MetricUnit::Ms),
            "bytes" => Some(MetricUnit::Bytes),
            "percent" => Some(MetricUnit::Percent),
            "s" => Some(MetricUnit::S),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct MetricsTaxonomy {
    version: String,
    #[serde(default)]
    defaults: Option<TaxonomyDefaults>,
    metrics: Vec<TaxonomyMetric>,
}

#[derive(Debug, Clone, Deserialize)]
struct TaxonomyDefaults {
    histogram_buckets: Option<TaxonomyHistogramBuckets>,
}

#[derive(Debug, Clone, Deserialize)]
struct TaxonomyHistogramBuckets {
    ms_metrics: Option<Vec<f64>>,
    seconds_metrics: Option<Vec<f64>>,
    bytes_metrics: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Deserialize)]
struct TaxonomyMetric {
    name: String,
    unit: String,
}

#[derive(Debug, Clone)]
struct TaxonomyRegistry {
    #[allow(dead_code)]
    version: String,
    units: HashMap<String, MetricUnit>,
    buckets_ms: Vec<f64>,
    buckets_s: Vec<f64>,
    buckets_bytes: Vec<f64>,
}

impl TaxonomyRegistry {
    fn load() -> Self {
        let taxonomy: MetricsTaxonomy = serde_yaml::from_str(METRICS_TAXONOMY_YAML)
            .expect("embedded metrics taxonomy must parse");

        let mut units = HashMap::new();
        for metric in &taxonomy.metrics {
            if let Some(unit) = MetricUnit::from_taxonomy_unit(&metric.unit) {
                units.insert(metric.name.clone(), unit);
            }
        }

        let buckets_ms = taxonomy
            .defaults
            .as_ref()
            .and_then(|d| d.histogram_buckets.as_ref())
            .and_then(|b| b.ms_metrics.clone())
            .unwrap_or_else(|| vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0, 10000.0]);

        let buckets_s = taxonomy
            .defaults
            .as_ref()
            .and_then(|d| d.histogram_buckets.as_ref())
            .and_then(|b| b.seconds_metrics.clone())
            .unwrap_or_else(|| {
                vec![
                    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
                ]
            });

        let buckets_bytes = taxonomy
            .defaults
            .as_ref()
            .and_then(|d| d.histogram_buckets.as_ref())
            .and_then(|b| b.bytes_metrics.clone())
            .unwrap_or_else(|| {
                vec![
                    1024.0,
                    10240.0,
                    102400.0,
                    1048576.0,
                    10485760.0,
                    104857600.0,
                ]
            });

        Self {
            version: taxonomy.version,
            units,
            buckets_ms,
            buckets_s,
            buckets_bytes,
        }
    }

    fn unit_for(&self, name: &str) -> Option<MetricUnit> {
        self.units.get(name).copied()
    }

    fn default_buckets_for(&self, name: &str, unit: MetricUnit) -> Vec<f64> {
        match unit {
            MetricUnit::Ms => self.buckets_ms.clone(),
            MetricUnit::S => self.buckets_s.clone(),
            MetricUnit::Bytes => self.buckets_bytes.clone(),
            // Not meaningful for histogram by default.
            MetricUnit::Count | MetricUnit::Percent => {
                if name.ends_with("_ms") {
                    self.buckets_ms.clone()
                } else {
                    Vec::new()
                }
            }
        }
    }
}

static TAXONOMY: once_cell::sync::Lazy<TaxonomyRegistry> =
    once_cell::sync::Lazy::new(TaxonomyRegistry::load);

/// A telemetry event emitted by the metrics module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricsEvent {
    /// RFC3339 timestamp of emission.
    pub timestamp: String,
    /// Metric identifier.
    pub name: String,
    /// Event value (scalar or histogram summary).
    pub value: MetricValue,
    /// Optional tags (dimensions) for the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<BTreeMap<String, String>>,
    /// Optional unit (defaults to taxonomy unit when omitted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<MetricUnit>,
}

/// Metric payload value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    /// Scalar measurement.
    Scalar(f64),
    /// Histogram summary.
    Histogram(HistogramSummary),
}

/// Histogram summary payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistogramSummary {
    /// Total number of observations.
    pub count: u64,
    /// Sum of observed values.
    pub sum: f64,
    /// Cumulative buckets.
    pub buckets: Vec<HistogramBucket>,
}

/// Histogram bucket entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistogramBucket {
    /// Upper bound (less-than-or-equal).
    pub le: f64,
    /// Cumulative count up to and including this bucket.
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MetricKey {
    name: String,
    tags: Vec<(String, String)>,
}

impl MetricKey {
    fn new(name: &str, tags: &BTreeMap<String, String>) -> Self {
        Self {
            name: name.to_string(),
            tags: tags.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        }
    }
}

#[derive(Debug, Clone)]
struct HistogramState {
    boundaries: Vec<f64>,
    cumulative_counts: Vec<u64>,
    count: u64,
    sum: f64,
}

impl HistogramState {
    fn new(boundaries: Vec<f64>) -> Result<Self, TelemetryError> {
        if boundaries.is_empty() {
            return Err(TelemetryError::InvalidHistogramBuckets(
                "bucket list is empty".to_string(),
            ));
        }

        for window in boundaries.windows(2) {
            if window[0] >= window[1] {
                return Err(TelemetryError::InvalidHistogramBuckets(
                    "buckets must be strictly increasing".to_string(),
                ));
            }
        }

        Ok(Self {
            cumulative_counts: vec![0; boundaries.len()],
            boundaries,
            count: 0,
            sum: 0.0,
        })
    }

    fn observe(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;

        for (i, le) in self.boundaries.iter().enumerate() {
            if value <= *le {
                for c in &mut self.cumulative_counts[i..] {
                    *c += 1;
                }
                return;
            }
        }

        // Value above max boundary: treat the final bucket as +Inf so the last
        // bucket count always equals the total observation count.
        if let Some(last) = self.cumulative_counts.last_mut() {
            *last += 1;
        }
    }

    fn snapshot(&self) -> HistogramSummary {
        HistogramSummary {
            count: self.count,
            sum: self.sum,
            buckets: self
                .boundaries
                .iter()
                .cloned()
                .zip(self.cumulative_counts.iter().cloned())
                .map(|(le, count)| HistogramBucket { le, count })
                .collect(),
        }
    }
}

#[derive(Debug, Default)]
struct Inner {
    events: Vec<MetricsEvent>,
    gauges: HashMap<MetricKey, f64>,
    histograms: HashMap<MetricKey, HistogramState>,
}

/// In-memory telemetry registry.
///
/// This is intentionally lightweight and offline; callers can export to JSON and
/// forward to logs/stdout.
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    inner: Arc<Mutex<Inner>>,
}

impl Metrics {
    /// Create a new metrics registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Acquire a counter instrument.
    pub fn counter(&self, name: &str) -> Result<Counter, TelemetryError> {
        let unit = TAXONOMY
            .unit_for(name)
            .ok_or_else(|| TelemetryError::UnknownMetricName(name.to_string()))?;

        Ok(Counter {
            metrics: self.clone(),
            name: name.to_string(),
            unit,
            tags: BTreeMap::new(),
        })
    }

    /// Acquire a gauge instrument.
    pub fn gauge(&self, name: &str) -> Result<Gauge, TelemetryError> {
        let unit = TAXONOMY
            .unit_for(name)
            .ok_or_else(|| TelemetryError::UnknownMetricName(name.to_string()))?;

        Ok(Gauge {
            metrics: self.clone(),
            name: name.to_string(),
            unit,
            tags: BTreeMap::new(),
        })
    }

    /// Acquire a histogram instrument using taxonomy/ADR default buckets.
    pub fn histogram(&self, name: &str) -> Result<Histogram, TelemetryError> {
        let unit = TAXONOMY
            .unit_for(name)
            .ok_or_else(|| TelemetryError::UnknownMetricName(name.to_string()))?;

        let buckets = TAXONOMY.default_buckets_for(name, unit);
        Histogram::new(self.clone(), name.to_string(), unit, buckets)
    }

    /// Acquire a histogram instrument with explicit bucket boundaries.
    pub fn histogram_with_buckets(
        &self,
        name: &str,
        buckets: Vec<f64>,
    ) -> Result<Histogram, TelemetryError> {
        let unit = TAXONOMY
            .unit_for(name)
            .ok_or_else(|| TelemetryError::UnknownMetricName(name.to_string()))?;

        Histogram::new(self.clone(), name.to_string(), unit, buckets)
    }

    /// Export current metric events (does not clear state).
    pub fn export(&self) -> Result<Vec<MetricsEvent>, TelemetryError> {
        let now = now_rfc3339()?;
        let mut out = Vec::new();

        let inner = self.inner.lock().expect("metrics lock poisoned");
        out.extend(inner.events.iter().cloned());

        for (key, hist) in &inner.histograms {
            out.push(MetricsEvent {
                timestamp: now.clone(),
                name: key.name.clone(),
                value: MetricValue::Histogram(hist.snapshot()),
                tags: if key.tags.is_empty() {
                    None
                } else {
                    Some(key.tags.iter().cloned().collect())
                },
                unit: TAXONOMY.unit_for(&key.name),
            });
        }

        Ok(out)
    }

    /// Export and clear all buffered events and aggregations.
    pub fn flush(&self) -> Result<Vec<MetricsEvent>, TelemetryError> {
        let now = now_rfc3339()?;
        let mut inner = self.inner.lock().expect("metrics lock poisoned");

        let mut out = Vec::new();
        out.append(&mut inner.events);

        for (key, hist) in inner.histograms.drain() {
            let unit = TAXONOMY.unit_for(&key.name);
            let name = key.name;
            let tags = key.tags;

            out.push(MetricsEvent {
                timestamp: now.clone(),
                name,
                value: MetricValue::Histogram(hist.snapshot()),
                tags: if tags.is_empty() {
                    None
                } else {
                    Some(tags.into_iter().collect())
                },
                unit,
            });
        }

        inner.gauges.clear();

        Ok(out)
    }

    /// Validate an exported metrics payload against the embedded schema.
    #[cfg(feature = "schema-validation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "schema-validation")))]
    pub fn validate_events(
        events: &[MetricsEvent],
    ) -> Result<
        Vec<crate::schema_validation::ValidationIssue>,
        crate::schema_validation::SchemaValidationError,
    > {
        let mut issues = Vec::new();

        for event in events {
            let value = serde_json::to_value(event).map_err(|e| {
                crate::schema_validation::SchemaValidationError::InvalidPayload(e.to_string())
            })?;

            issues.extend(crate::schema_validation::validate_data(
                METRICS_EVENT_SCHEMA_PATH,
                &value,
            )?);
        }

        Ok(issues)
    }
}

/// Counter instrument.
#[derive(Debug, Clone)]
pub struct Counter {
    metrics: Metrics,
    name: String,
    unit: MetricUnit,
    tags: BTreeMap<String, String>,
}

impl Counter {
    /// Attach tags to the instrument.
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Attach multiple tags to the instrument.
    pub fn with_tags<I, K, V>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (k, v) in tags {
            self.tags.insert(k.into(), v.into());
        }
        self
    }

    /// Increment the counter by `delta` (defaults to 1).
    pub fn inc(&self, delta: Option<u64>) -> Result<(), TelemetryError> {
        let delta = delta.unwrap_or(1) as f64;
        let event = MetricsEvent {
            timestamp: now_rfc3339()?,
            name: self.name.clone(),
            value: MetricValue::Scalar(delta),
            tags: if self.tags.is_empty() {
                None
            } else {
                Some(self.tags.clone())
            },
            unit: Some(self.unit),
        };

        let mut inner = self.metrics.inner.lock().expect("metrics lock poisoned");
        inner.events.push(event);
        Ok(())
    }
}

/// Gauge instrument.
#[derive(Debug, Clone)]
pub struct Gauge {
    metrics: Metrics,
    name: String,
    unit: MetricUnit,
    tags: BTreeMap<String, String>,
}

impl Gauge {
    /// Attach tags to the instrument.
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Attach multiple tags to the instrument.
    pub fn with_tags<I, K, V>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (k, v) in tags {
            self.tags.insert(k.into(), v.into());
        }
        self
    }

    /// Set the gauge to an explicit value.
    pub fn set(&self, value: f64) -> Result<(), TelemetryError> {
        self.emit_and_store(value)
    }

    /// Increment the gauge by `delta`.
    pub fn inc(&self, delta: Option<f64>) -> Result<(), TelemetryError> {
        let delta = delta.unwrap_or(1.0);

        let mut inner = self.metrics.inner.lock().expect("metrics lock poisoned");
        let key = MetricKey::new(&self.name, &self.tags);
        let current = inner.gauges.get(&key).copied().unwrap_or(0.0);
        let next = current + delta;
        inner.gauges.insert(key, next);

        inner.events.push(MetricsEvent {
            timestamp: now_rfc3339()?,
            name: self.name.clone(),
            value: MetricValue::Scalar(next),
            tags: if self.tags.is_empty() {
                None
            } else {
                Some(self.tags.clone())
            },
            unit: Some(self.unit),
        });

        Ok(())
    }

    /// Decrement the gauge by `delta`.
    pub fn dec(&self, delta: Option<f64>) -> Result<(), TelemetryError> {
        self.inc(Some(-delta.unwrap_or(1.0)))
    }

    fn emit_and_store(&self, value: f64) -> Result<(), TelemetryError> {
        let mut inner = self.metrics.inner.lock().expect("metrics lock poisoned");
        let key = MetricKey::new(&self.name, &self.tags);
        inner.gauges.insert(key, value);

        inner.events.push(MetricsEvent {
            timestamp: now_rfc3339()?,
            name: self.name.clone(),
            value: MetricValue::Scalar(value),
            tags: if self.tags.is_empty() {
                None
            } else {
                Some(self.tags.clone())
            },
            unit: Some(self.unit),
        });

        Ok(())
    }
}

/// Histogram instrument.
#[derive(Debug, Clone)]
pub struct Histogram {
    metrics: Metrics,
    name: String,
    unit: MetricUnit,
    tags: BTreeMap<String, String>,
    boundaries: Vec<f64>,
}

impl Histogram {
    fn new(
        metrics: Metrics,
        name: String,
        unit: MetricUnit,
        boundaries: Vec<f64>,
    ) -> Result<Self, TelemetryError> {
        // Validate boundaries up-front.
        let _ = HistogramState::new(boundaries.clone())?;

        Ok(Self {
            metrics,
            name,
            unit,
            tags: BTreeMap::new(),
            boundaries,
        })
    }

    /// Attach tags to the instrument.
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Attach multiple tags to the instrument.
    pub fn with_tags<I, K, V>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (k, v) in tags {
            self.tags.insert(k.into(), v.into());
        }
        self
    }

    /// Record a new observation.
    pub fn observe(&self, value: f64) -> Result<(), TelemetryError> {
        let mut inner = self.metrics.inner.lock().expect("metrics lock poisoned");
        let key = MetricKey::new(&self.name, &self.tags);

        let hist = inner
            .histograms
            .entry(key)
            .or_insert_with(|| HistogramState::new(self.boundaries.clone()).expect("validated"));

        hist.observe(value);
        Ok(())
    }

    /// Snapshot the histogram state as an event.
    pub fn snapshot(&self) -> Result<MetricsEvent, TelemetryError> {
        let inner = self.metrics.inner.lock().expect("metrics lock poisoned");
        let key = MetricKey::new(&self.name, &self.tags);

        let state =
            inner.histograms.get(&key).cloned().unwrap_or_else(|| {
                HistogramState::new(self.boundaries.clone()).expect("validated")
            });

        Ok(MetricsEvent {
            timestamp: now_rfc3339()?,
            name: self.name.clone(),
            value: MetricValue::Histogram(state.snapshot()),
            tags: if self.tags.is_empty() {
                None
            } else {
                Some(self.tags.clone())
            },
            unit: Some(self.unit),
        })
    }
}

fn now_rfc3339() -> Result<String, TelemetryError> {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| TelemetryError::TimestampFormat(e.to_string()))?;

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

// Howard Hinnant's civil-from-days algorithm.
// Converts days since 1970-01-01 (UTC) into year-month-day.
fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u8, u8) {
    // Shift to civil date origin.
    let z = days_since_unix_epoch + 719_468;

    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = mp + if mp < 10 { 3 } else { -9 }; // [1, 12]
    let year = y + if m <= 2 { 1 } else { 0 };

    (year as i32, m as u8, d as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use serde_json::Value;

    #[test]
    fn test_counter_emits_scalar_event() {
        let metrics = Metrics::new();
        let counter = metrics
            .counter("schema_validations")
            .unwrap()
            .with_tags([("module", "schema_validation"), ("service", "api")]);

        counter.inc(None).unwrap();
        let exported = metrics.export().unwrap();

        let event = exported
            .iter()
            .find(|e| e.name == "schema_validations")
            .unwrap();

        let tags = event.tags.as_ref().unwrap();
        assert_eq!(
            tags.get("module").map(String::as_str),
            Some("schema_validation")
        );
        assert_eq!(tags.get("service").map(String::as_str), Some("api"));
    }

    #[test]
    fn test_gauge_inc_dec_updates_value() {
        let metrics = Metrics::new();
        let gauge = metrics.gauge("http_active_requests").unwrap();

        gauge.inc(None).unwrap();
        gauge.inc(None).unwrap();
        gauge.dec(None).unwrap();

        let exported = metrics.export().unwrap();
        let last = exported
            .iter()
            .rev()
            .find(|e| e.name == "http_active_requests")
            .unwrap();

        assert_eq!(last.value, MetricValue::Scalar(1.0));
    }

    #[test]
    fn test_histogram_observe_aggregates_buckets() {
        let metrics = Metrics::new();
        let hist = metrics.histogram("config_load_ms").unwrap();

        hist.observe(1.0).unwrap();
        hist.observe(7.0).unwrap();
        hist.observe(100.0).unwrap();
        hist.observe(50_000.0).unwrap();

        let exported = metrics.export().unwrap();
        let event = exported
            .iter()
            .find(|e| e.name == "config_load_ms" && matches!(e.value, MetricValue::Histogram(_)))
            .unwrap();

        let MetricValue::Histogram(h) = &event.value else {
            panic!("expected histogram");
        };

        assert_eq!(h.count, 4);
        assert_eq!(h.sum, 50_108.0);
        assert_eq!(h.buckets[0].le, 1.0);
        assert_eq!(h.buckets[0].count, 1);

        // <=10 should have 2 observations.
        let le10 = h.buckets.iter().find(|b| b.le == 10.0).unwrap();
        assert_eq!(le10.count, 2);

        // Overflow values are counted in the final bucket (+Inf semantics).
        let last = h.buckets.last().unwrap();
        assert_eq!(last.count, 4);
    }

    #[test]
    fn test_flush_clears_state() {
        let metrics = Metrics::new();
        let counter = metrics.counter("schema_validations").unwrap();
        counter.inc(None).unwrap();

        let out = metrics.flush().unwrap();
        assert!(!out.is_empty());

        let after = metrics.export().unwrap();
        assert!(after.is_empty());
    }

    #[test]
    fn test_unknown_metric_rejected() {
        let metrics = Metrics::new();
        let e = metrics.counter("not_in_taxonomy").unwrap_err();
        assert_eq!(
            e,
            TelemetryError::UnknownMetricName("not_in_taxonomy".to_string())
        );
    }

    #[cfg(feature = "schema-validation")]
    #[test]
    fn test_schema_validation_accepts_scalar_fixture() {
        let fixture = include_str!("../../tests/fixtures/metrics/scalar.json");
        let value: Value = serde_json::from_str(fixture).unwrap();

        let issues = crate::schema_validation::validate_data(METRICS_EVENT_SCHEMA_PATH, &value)
            .expect("schema validation should run");

        assert!(issues.is_empty(), "expected no issues, got {issues:?}");
    }

    #[cfg(feature = "schema-validation")]
    #[test]
    fn test_schema_validation_accepts_histogram_fixture() {
        let fixture = include_str!("../../tests/fixtures/metrics/histogram_ms.json");
        let value: Value = serde_json::from_str(fixture).unwrap();

        let issues = crate::schema_validation::validate_data(METRICS_EVENT_SCHEMA_PATH, &value)
            .expect("schema validation should run");

        assert!(issues.is_empty(), "expected no issues, got {issues:?}");
    }
}
