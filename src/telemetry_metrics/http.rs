//! HTTP server metrics helpers (no framework middleware).
//!
//! Records the Crucible `http_*` taxonomy. The caller supplies method, a
//! **templated** route (or a raw path that [`normalize_route`] will rewrite),
//! status, and duration. No `axum` / `tower` / listen socket.

use std::time::Duration;

use super::{Metrics, TelemetryError};

const OTHER_ROUTE: &str = "__other__";
const STANDARD_METHODS: &[&str] = &[
    "GET", "HEAD", "POST", "PUT", "DELETE", "CONNECT", "OPTIONS", "TRACE", "PATCH",
];

/// Default duration buckets for `http_request_duration_seconds` (seconds).
pub const HTTP_DURATION_BUCKETS_S: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

/// Default size buckets for request/response body histograms (bytes).
pub const HTTP_SIZE_BUCKETS_BYTES: &[f64] = &[
    1024.0,
    10_240.0,
    102_400.0,
    1_048_576.0,
    10_485_760.0,
    104_857_600.0,
];

/// Outcome label derived from an HTTP status code (`1xx`…`5xx`).
pub fn outcome_from_status(status: u16) -> &'static str {
    match status {
        100..=199 => "1xx",
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "unknown",
    }
}

/// Replace high-cardinality path segments (UUIDs, numeric IDs, ObjectIds).
///
/// Query strings are stripped. Use a framework route template when you have
/// one; this is the fallback the standard requires.
pub fn normalize_route(path: &str) -> String {
    let path = path.split('?').next().unwrap_or(path);
    if path.is_empty() {
        return "/".to_string();
    }
    let leading = path.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        if segment.is_empty() {
            continue;
        }
        parts.push(if is_uuid(segment) {
            ":uuid"
        } else if is_object_id(segment) {
            ":objectid"
        } else if is_numeric_id(segment) {
            ":id"
        } else if is_token_segment(segment) {
            ":token"
        } else {
            segment
        });
    }
    let mut out = if leading {
        String::from("/")
    } else {
        String::new()
    };
    out.push_str(&parts.join("/"));
    if out.is_empty() {
        "/".to_string()
    } else {
        out
    }
}

fn is_token_segment(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    if bytes.len() < 16 {
        return false;
    }
    if bytes.iter().all(|b| b.is_ascii_hexdigit()) {
        return true;
    }
    let base64ish = bytes
        .iter()
        .all(|b| b.is_ascii_alphanumeric() || matches!(*b, b'+' | b'/' | b'-' | b'_' | b'='));
    let has_digit = bytes.iter().any(|b| b.is_ascii_digit());
    base64ish && has_digit
}

fn canonical_method(method: &str) -> String {
    let upper = method.to_ascii_uppercase();
    if STANDARD_METHODS.iter().any(|item| *item == upper) {
        upper
    } else {
        "OTHER".to_string()
    }
}

fn is_numeric_id(segment: &str) -> bool {
    !segment.is_empty() && segment.bytes().all(|b| b.is_ascii_digit())
}

fn is_object_id(segment: &str) -> bool {
    segment.len() == 24 && segment.bytes().all(|b| b.is_ascii_hexdigit())
}

fn is_uuid(segment: &str) -> bool {
    let b = segment.as_bytes();
    if b.len() != 36 {
        return false;
    }
    for (i, byte) in b.iter().enumerate() {
        match i {
            8 | 13 | 18 | 23 => {
                if *byte != b'-' {
                    return false;
                }
            }
            _ => {
                if !byte.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}

/// One completed HTTP request to record.
#[derive(Debug, Clone)]
pub struct HttpRecord {
    /// HTTP method (uppercased on emit).
    pub method: String,
    /// Route template or raw path.
    pub route: String,
    /// HTTP status code.
    pub status: u16,
    /// Duration in **seconds** (taxonomy unit).
    pub duration_secs: f64,
    /// Optional request body size in bytes.
    pub request_bytes: Option<u64>,
    /// Optional response body size in bytes.
    pub response_bytes: Option<u64>,
    /// When true (default), run [`normalize_route`] on `route`.
    pub normalize: bool,
}

impl HttpRecord {
    /// Build a record with zero duration and no sizes.
    pub fn new(method: impl Into<String>, route: impl Into<String>, status: u16) -> Self {
        Self {
            method: method.into(),
            route: route.into(),
            status,
            duration_secs: 0.0,
            request_bytes: None,
            response_bytes: None,
            normalize: true,
        }
    }

    /// Set duration from a [`Duration`] (already seconds-based).
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration_secs = duration.as_secs_f64();
        self
    }

    /// Set duration already expressed in seconds.
    pub fn duration_secs(mut self, secs: f64) -> Self {
        self.duration_secs = secs;
        self
    }

    /// Set duration from milliseconds. Converts to seconds (`ms / 1000`).
    pub fn duration_ms(mut self, ms: f64) -> Self {
        self.duration_secs = ms / 1000.0;
        self
    }

    /// Attach request/response sizes in bytes.
    pub fn sizes(mut self, request_bytes: u64, response_bytes: u64) -> Self {
        self.request_bytes = Some(request_bytes);
        self.response_bytes = Some(response_bytes);
        self
    }

    /// Skip [`normalize_route`] when `route` is already a template.
    pub fn already_normalized(mut self) -> Self {
        self.normalize = false;
        self
    }
}

/// Records `http_*` taxonomy metrics for one service.
#[derive(Debug, Clone)]
pub struct HttpRecorder {
    metrics: Metrics,
    service: String,
    track_sizes: bool,
}

impl HttpRecorder {
    /// Bind a recorder to a [`Metrics`] registry and service name.
    pub fn new(metrics: Metrics, service: impl Into<String>) -> Self {
        Self {
            metrics,
            service: service.into(),
            track_sizes: false,
        }
    }

    /// Also emit request/response size histograms when sizes are present.
    pub fn with_sizes(mut self) -> Self {
        self.track_sizes = true;
        self
    }

    /// Increment `http_active_requests`. Decrement on drop.
    pub fn enter(&self) -> Result<ActiveHttpRequest, TelemetryError> {
        self.metrics
            .gauge("http_active_requests")?
            .with_tag("service", self.service.clone())
            .inc(None)?;
        Ok(ActiveHttpRequest {
            metrics: self.metrics.clone(),
            service: self.service.clone(),
        })
    }

    /// Record a completed request (counter + duration, optional sizes).
    ///
    /// Label sets share one cap on the [`Metrics`] registry. Past the cap,
    /// the route collapses to `__other__` and the status collapses to its
    /// outcome class so request-controlled labels cannot open more series.
    pub fn record(&self, record: HttpRecord) -> Result<(), TelemetryError> {
        let method = canonical_method(&record.method);
        let mut route = if record.normalize {
            normalize_route(&record.route)
        } else {
            record.route
        };
        let outcome = outcome_from_status(record.status);
        let mut status = record.status.to_string();
        let service = self.service.as_str();
        let signature = format!("{service}\n{method}\n{route}\n{status}\n{outcome}");
        if !self.metrics.admit_http_series(&signature) {
            route = OTHER_ROUTE.to_string();
            status = outcome.to_string();
        }

        self.metrics
            .counter("http_requests_total")?
            .with_tags([
                ("method", method.as_str()),
                ("route", route.as_str()),
                ("status", status.as_str()),
                ("service", service),
                ("outcome", outcome),
            ])
            .inc(None)?;

        self.metrics
            .histogram("http_request_duration_seconds")?
            .with_tags([
                ("method", method.as_str()),
                ("route", route.as_str()),
                ("status", status.as_str()),
                ("service", service),
                ("outcome", outcome),
            ])
            .observe(record.duration_secs)?;

        if self.track_sizes {
            if let Some(bytes) = record.request_bytes {
                self.metrics
                    .histogram("http_request_size_bytes")?
                    .with_tags([
                        ("method", method.as_str()),
                        ("route", route.as_str()),
                        ("service", service),
                    ])
                    .observe(bytes as f64)?;
            }
            if let Some(bytes) = record.response_bytes {
                self.metrics
                    .histogram("http_response_size_bytes")?
                    .with_tags([
                        ("method", method.as_str()),
                        ("route", route.as_str()),
                        ("status", status.as_str()),
                        ("service", service),
                        ("outcome", outcome),
                    ])
                    .observe(bytes as f64)?;
            }
        }
        Ok(())
    }
}

/// Guard that decrements `http_active_requests` when dropped.
#[derive(Debug)]
pub struct ActiveHttpRequest {
    metrics: Metrics,
    service: String,
}

impl Drop for ActiveHttpRequest {
    fn drop(&mut self) {
        if let Ok(gauge) = self.metrics.gauge("http_active_requests") {
            let _ = gauge.with_tag("service", self.service.clone()).dec(None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry_metrics::{MetricValue, Metrics};

    #[test]
    fn normalize_rewrites_ids_and_strips_query() {
        assert_eq!(
            normalize_route("/users/123/orders/456?x=1"),
            "/users/:id/orders/:id"
        );
        assert_eq!(
            normalize_route("/users/550e8400-e29b-41d4-a716-446655440000"),
            "/users/:uuid"
        );
        assert_eq!(
            normalize_route("/docs/507f1f77bcf86cd799439011"),
            "/docs/:objectid"
        );
        assert_eq!(
            normalize_route("/password-reset/9f8e7d6c5b4a39281716151413121110"),
            "/password-reset/:token"
        );
        assert!(
            !normalize_route("/password-reset/9f8e7d6c5b4a39281716151413121110").contains("9f8e")
        );
        assert_eq!(normalize_route("/"), "/");
    }

    #[test]
    fn outcome_groups_status() {
        assert_eq!(outcome_from_status(201), "2xx");
        assert_eq!(outcome_from_status(404), "4xx");
        assert_eq!(outcome_from_status(503), "5xx");
        assert_eq!(outcome_from_status(99), "unknown");
    }

    #[test]
    fn duration_ms_converts_to_seconds() {
        let rec = HttpRecord::new("get", "/users/9", 200).duration_ms(1500.0);
        assert!((rec.duration_secs - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn record_emits_normalized_labels() {
        let metrics = Metrics::new();
        let rec = HttpRecorder::new(metrics.clone(), "api").with_sizes();
        rec.record(
            HttpRecord::new("get", "/users/42", 201)
                .duration_ms(20.0)
                .sizes(128, 256),
        )
        .unwrap();

        let events = metrics.export().unwrap();
        let total = events
            .iter()
            .find(|e| e.name == "http_requests_total")
            .unwrap();
        let tags = total.tags.as_ref().unwrap();
        assert_eq!(tags.get("method").map(String::as_str), Some("GET"));
        assert_eq!(tags.get("route").map(String::as_str), Some("/users/:id"));
        assert_eq!(tags.get("status").map(String::as_str), Some("201"));
        assert_eq!(tags.get("outcome").map(String::as_str), Some("2xx"));
        assert_eq!(tags.get("service").map(String::as_str), Some("api"));

        let duration = events
            .iter()
            .find(|e| e.name == "http_request_duration_seconds")
            .unwrap();
        let MetricValue::Histogram(h) = &duration.value else {
            panic!("expected histogram");
        };
        assert_eq!(h.count, 1);
        assert!((h.sum - 0.02).abs() < 1e-9);
    }

    #[test]
    fn label_sets_collapse_past_cap_and_unknown_methods() {
        let metrics = Metrics::new();
        metrics.set_http_series_cap(3);
        let rec = HttpRecorder::new(metrics.clone(), "api");
        for i in 0..5 {
            rec.record(HttpRecord::new("GET", format!("/only-{i}"), 404))
                .unwrap();
        }
        rec.record(HttpRecord::new("NOPE", "/only-0", 404)).unwrap();
        let text = metrics.export_prometheus_text().unwrap();
        assert!(text.contains("route=\"/only-0\""), "{text}");
        assert!(text.contains("route=\"__other__\""), "{text}");
        assert!(!text.contains("/only-3"), "{text}");
        assert!(!text.contains("/only-4"), "{text}");
        assert!(text.contains("method=\"OTHER\""), "{text}");
        assert!(!text.contains("9f8e"), "{text}");
    }

    #[test]
    fn recorders_share_one_registry_cap() {
        let metrics = Metrics::new();
        metrics.set_http_series_cap(3);
        let left = HttpRecorder::new(metrics.clone(), "api");
        let right = HttpRecorder::new(metrics.clone(), "api");
        left.record(HttpRecord::new("GET", "/ra-0", 200)).unwrap();
        right.record(HttpRecord::new("GET", "/rb-0", 200)).unwrap();
        left.record(HttpRecord::new("GET", "/ra-1", 200)).unwrap();
        right.record(HttpRecord::new("GET", "/rb-1", 200)).unwrap();
        let text = metrics.export_prometheus_text().unwrap();
        let series = text
            .lines()
            .filter(|line| line.starts_with("http_requests_total{"))
            .count();
        assert!(series <= 3, "{series}\n{text}");
        assert!(text.contains("route=\"__other__\""), "{text}");
        let present = ["/ra-0", "/rb-0", "/ra-1", "/rb-1"]
            .into_iter()
            .filter(|route| text.contains(route))
            .count();
        assert_eq!(present, 2, "{text}");
    }

    #[test]
    fn overflow_collapses_status_to_outcome() {
        let metrics = Metrics::new();
        metrics.set_http_series_cap(3);
        let rec = HttpRecorder::new(metrics.clone(), "api");
        rec.record(HttpRecord::new("GET", "/keep-a", 200)).unwrap();
        rec.record(HttpRecord::new("GET", "/keep-b", 200)).unwrap();
        for status in [201u16, 404, 401, 500, 503, 302, 418] {
            rec.record(HttpRecord::new("GET", format!("/past-{status}"), status))
                .unwrap();
        }
        let text = metrics.export_prometheus_text().unwrap();
        assert!(text.contains("route=\"/keep-a\""), "{text}");
        assert!(text.contains("route=\"/keep-b\""), "{text}");
        assert!(text.contains("route=\"__other__\""), "{text}");
        assert!(!text.contains("/past-"), "{text}");
        for status in ["201", "404", "401", "500", "503", "302", "418"] {
            assert!(!text.contains(&format!("status=\"{status}\"")), "{text}");
        }
        assert!(text.contains("status=\"4xx\""), "{text}");
        assert!(text.contains("status=\"5xx\""), "{text}");
        let series = text
            .lines()
            .filter(|line| line.starts_with("http_requests_total{"))
            .count();
        // Two admitted routes plus one overflow series per outcome class.
        assert!(series <= 6, "{series}\n{text}");
    }

    #[test]
    fn already_normalized_route_is_kept() {
        let metrics = Metrics::new();
        let rec = HttpRecorder::new(metrics.clone(), "api");
        rec.record(HttpRecord::new("POST", "/users/:id", 200).already_normalized())
            .unwrap();
        let events = metrics.export().unwrap();
        let total = events
            .iter()
            .find(|e| e.name == "http_requests_total")
            .unwrap();
        assert_eq!(
            total
                .tags
                .as_ref()
                .unwrap()
                .get("route")
                .map(String::as_str),
            Some("/users/:id")
        );
    }

    #[test]
    fn enter_tracks_active_requests() {
        let metrics = Metrics::new();
        let rec = HttpRecorder::new(metrics.clone(), "api");
        {
            let _guard = rec.enter().unwrap();
            let events = metrics.export().unwrap();
            let last = events
                .iter()
                .rev()
                .find(|e| e.name == "http_active_requests")
                .unwrap();
            assert_eq!(last.value, MetricValue::Scalar(1.0));
        }
        let events = metrics.export().unwrap();
        let last = events
            .iter()
            .rev()
            .find(|e| e.name == "http_active_requests")
            .unwrap();
        assert_eq!(last.value, MetricValue::Scalar(0.0));
    }
}
