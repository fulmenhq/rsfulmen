//! JSON and Prometheus text exporters (no HTTP server).
//!
//! Canonical events stay JSON ([`MetricsEvent`]). Prometheus text is a
//! derived scrape format the caller can serve. Auth, TLS, and rate-limit
//! stay with the caller — same split as `POST /admin/signal`.

use std::collections::BTreeMap;
use std::io::Write;
use std::sync::Mutex;
use std::time::Instant;

use super::{HistogramSummary, MetricUnit, MetricValue, Metrics, MetricsEvent, TelemetryError};

/// Sink that pulls a snapshot from [`Metrics`].
pub trait Exporter {
    /// Collect and write/render the current snapshot.
    fn export(&self, metrics: &Metrics) -> Result<(), TelemetryError>;
}

/// JSON-lines writer (`application/jsonl`).
pub struct JsonLinesExporter<W> {
    writer: Mutex<W>,
}

impl<W: Write + Send> JsonLinesExporter<W> {
    /// Wrap an existing writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer: Mutex::new(writer),
        }
    }
}

impl<W: Write + Send> Exporter for JsonLinesExporter<W> {
    fn export(&self, metrics: &Metrics) -> Result<(), TelemetryError> {
        let text = render_json_lines(&metrics.export()?)?;
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| TelemetryError::Io("json-lines writer lock poisoned".to_string()))?;
        writer
            .write_all(text.as_bytes())
            .map_err(|err| TelemetryError::Io(err.to_string()))?;
        writer
            .flush()
            .map_err(|err| TelemetryError::Io(err.to_string()))?;
        Ok(())
    }
}

/// Prometheus text exposition (`text/plain; version=0.0.4`).
///
/// Renders a snapshot into `writer`. Does not open a listen socket.
pub struct PrometheusTextExporter<W> {
    writer: Mutex<W>,
}

impl<W: Write + Send> PrometheusTextExporter<W> {
    /// Wrap an existing writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer: Mutex::new(writer),
        }
    }
}

impl<W: Write + Send> Exporter for PrometheusTextExporter<W> {
    fn export(&self, metrics: &Metrics) -> Result<(), TelemetryError> {
        let text = metrics.export_prometheus_text()?;
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| TelemetryError::Io("prometheus writer lock poisoned".to_string()))?;
        writer
            .write_all(text.as_bytes())
            .map_err(|err| TelemetryError::Io(err.to_string()))?;
        writer
            .flush()
            .map_err(|err| TelemetryError::Io(err.to_string()))?;
        Ok(())
    }
}

/// Serialize events as a JSON array.
pub fn render_json_array(events: &[MetricsEvent]) -> Result<String, TelemetryError> {
    serde_json::to_string(events).map_err(|err| TelemetryError::Io(err.to_string()))
}

/// Serialize events as JSON lines (one object per line, trailing newline).
pub fn render_json_lines(events: &[MetricsEvent]) -> Result<String, TelemetryError> {
    let mut out = String::new();
    for event in events {
        let line =
            serde_json::to_string(event).map_err(|err| TelemetryError::Io(err.to_string()))?;
        out.push_str(&line);
        out.push('\n');
    }
    Ok(out)
}

/// Render aggregated Prometheus text from a JSON-event snapshot.
///
/// Counters (`*_total`, `*_count`, `*_errors`, plus remaining `count` units)
/// are summed. Gauges (`active` / `inflight` / `percent`) keep the last value.
/// Histograms emit `_bucket`, `_sum`, `_count`, and `le="+Inf"`.
pub fn render_prometheus_text(events: &[MetricsEvent]) -> String {
    let mut counters: BTreeMap<(String, BTreeMap<String, String>), f64> = BTreeMap::new();
    let mut gauges: BTreeMap<(String, BTreeMap<String, String>), f64> = BTreeMap::new();
    let mut hists: BTreeMap<(String, BTreeMap<String, String>), HistogramSummary> = BTreeMap::new();
    let mut types: BTreeMap<String, PromType> = BTreeMap::new();

    for event in events {
        let labels = event.tags.clone().unwrap_or_default();
        let key = (event.name.clone(), labels);
        match &event.value {
            MetricValue::Histogram(summary) => {
                types.insert(event.name.clone(), PromType::Histogram);
                hists.insert(key, summary.clone());
            }
            MetricValue::Scalar(value) => match classify_scalar(&event.name, event.unit) {
                PromType::Gauge => {
                    types.insert(event.name.clone(), PromType::Gauge);
                    gauges.insert(key, *value);
                }
                PromType::Counter => {
                    types.insert(event.name.clone(), PromType::Counter);
                    *counters.entry(key).or_insert(0.0) += *value;
                }
                PromType::Histogram => {}
            },
        }
    }

    let mut out = String::new();
    for (name, kind) in &types {
        out.push_str("# TYPE ");
        out.push_str(name);
        out.push(' ');
        out.push_str(kind.as_str());
        out.push('\n');

        match kind {
            PromType::Counter => {
                for ((metric, labels), value) in &counters {
                    if metric == name {
                        write_sample(&mut out, metric, labels, *value);
                    }
                }
            }
            PromType::Gauge => {
                for ((metric, labels), value) in &gauges {
                    if metric == name {
                        write_sample(&mut out, metric, labels, *value);
                    }
                }
            }
            PromType::Histogram => {
                for ((metric, labels), summary) in &hists {
                    if metric != name {
                        continue;
                    }
                    let mut last_count = 0;
                    for bucket in &summary.buckets {
                        let mut bucket_labels = labels.clone();
                        bucket_labels.insert("le".to_string(), format_prom_number(bucket.le));
                        write_sample(
                            &mut out,
                            &format!("{metric}_bucket"),
                            &bucket_labels,
                            bucket.count as f64,
                        );
                        last_count = bucket.count;
                    }
                    let mut inf_labels = labels.clone();
                    inf_labels.insert("le".to_string(), "+Inf".to_string());
                    write_sample(
                        &mut out,
                        &format!("{metric}_bucket"),
                        &inf_labels,
                        summary.count.max(last_count) as f64,
                    );
                    write_sample(&mut out, &format!("{metric}_sum"), labels, summary.sum);
                    write_sample(
                        &mut out,
                        &format!("{metric}_count"),
                        labels,
                        summary.count as f64,
                    );
                }
            }
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromType {
    Counter,
    Gauge,
    Histogram,
}

impl PromType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::Gauge => "gauge",
            Self::Histogram => "histogram",
        }
    }
}

fn classify_scalar(name: &str, unit: Option<MetricUnit>) -> PromType {
    let lower = name.to_ascii_lowercase();
    if lower.contains("active") || lower.contains("inflight") {
        return PromType::Gauge;
    }
    if matches!(unit, Some(MetricUnit::Percent)) {
        return PromType::Gauge;
    }
    PromType::Counter
}

fn write_sample(out: &mut String, name: &str, labels: &BTreeMap<String, String>, value: f64) {
    out.push_str(name);
    if !labels.is_empty() {
        out.push('{');
        let mut first = true;
        for (key, val) in labels {
            if !first {
                out.push(',');
            }
            first = false;
            out.push_str(key);
            out.push_str("=\"");
            out.push_str(&escape_label(val));
            out.push('"');
        }
        out.push('}');
    }
    out.push(' ');
    out.push_str(&format_prom_number(value));
    out.push('\n');
}

fn escape_label(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out
}

fn format_prom_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_positive() {
            "+Inf"
        } else {
            "-Inf"
        }
        .to_string();
    }
    let formatted = format!("{value}");
    if formatted.contains('.') || formatted.contains('e') || formatted.contains('E') {
        formatted
    } else {
        format!("{value:.1}")
    }
}

impl Metrics {
    /// Export the current snapshot as a JSON array.
    pub fn export_json_array(&self) -> Result<String, TelemetryError> {
        render_json_array(&self.export()?)
    }

    /// Export the current snapshot as JSON lines.
    pub fn export_json_lines(&self) -> Result<String, TelemetryError> {
        render_json_lines(&self.export()?)
    }

    /// Export Prometheus text and record exporter refresh self-metrics.
    ///
    /// Self-metrics (`prometheus_exporter_refresh_*`) land on the *next*
    /// snapshot so this scrape stays stable.
    pub fn export_prometheus_text(&self) -> Result<String, TelemetryError> {
        let started = Instant::now();
        let result = self.export().map(|events| render_prometheus_text(&events));
        let secs = started.elapsed().as_secs_f64();
        match &result {
            Ok(_) => {
                let _ = self
                    .counter("prometheus_exporter_refresh_total")
                    .and_then(|c| c.with_tag("result", "success").inc(None));
                if let Ok(hist) = self.histogram("prometheus_exporter_refresh_duration_seconds") {
                    let _ = hist
                        .with_tag("phase", "export")
                        .with_tag("result", "success")
                        .observe(secs);
                }
            }
            Err(_) => {
                let _ = self
                    .counter("prometheus_exporter_refresh_total")
                    .and_then(|c| c.with_tag("result", "error").inc(None));
                let _ = self
                    .counter("prometheus_exporter_refresh_errors_total")
                    .and_then(|c| c.with_tag("error_type", "other").inc(None));
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn json_lines_are_one_object_per_line() {
        let metrics = Metrics::new();
        metrics
            .counter("schema_validations")
            .unwrap()
            .inc(None)
            .unwrap();
        let text = metrics.export_json_lines().unwrap();
        let line = text.trim();
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(
            parsed.get("name").and_then(|v| v.as_str()),
            Some("schema_validations")
        );
    }

    #[test]
    fn prometheus_sums_counters_and_keeps_last_gauge() {
        let metrics = Metrics::new();
        metrics
            .counter("http_requests_total")
            .unwrap()
            .with_tags([
                ("method", "GET"),
                ("route", "/users/:id"),
                ("status", "200"),
                ("service", "api"),
            ])
            .inc(None)
            .unwrap();
        metrics
            .counter("http_requests_total")
            .unwrap()
            .with_tags([
                ("method", "GET"),
                ("route", "/users/:id"),
                ("status", "200"),
                ("service", "api"),
            ])
            .inc(None)
            .unwrap();
        metrics
            .gauge("http_active_requests")
            .unwrap()
            .with_tag("service", "api")
            .set(3.0)
            .unwrap();

        let text = render_prometheus_text(&metrics.export().unwrap());
        assert!(text.contains("# TYPE http_requests_total counter"));
        assert!(text.contains(
            "http_requests_total{method=\"GET\",route=\"/users/:id\",service=\"api\",status=\"200\"} 2.0"
        ));
        assert!(text.contains("# TYPE http_active_requests gauge"));
        assert!(text.contains("http_active_requests{service=\"api\"} 3.0"));
    }

    #[test]
    fn prometheus_histogram_emits_inf_bucket() {
        let metrics = Metrics::new();
        let hist = metrics.histogram("http_request_duration_seconds").unwrap();
        hist.observe(0.01).unwrap();
        let text = render_prometheus_text(&metrics.export().unwrap());
        assert!(text.contains("# TYPE http_request_duration_seconds histogram"));
        assert!(text.contains("le=\"+Inf\""));
        assert!(text.contains("http_request_duration_seconds_count"));
        assert!(text.contains("http_request_duration_seconds_sum"));
    }

    #[test]
    fn json_lines_exporter_writes() {
        use std::sync::Arc;

        #[derive(Clone)]
        struct Shared(Arc<Mutex<Vec<u8>>>);
        impl Write for Shared {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().expect("buf").write(buf)
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let metrics = Metrics::new();
        metrics
            .counter("foundry_lookup_count")
            .unwrap()
            .inc(None)
            .unwrap();
        let buf = Shared(Arc::new(Mutex::new(Vec::new())));
        let exporter = JsonLinesExporter::new(buf.clone());
        exporter.export(&metrics).unwrap();
        let text = String::from_utf8(buf.0.lock().expect("buf").clone()).unwrap();
        assert!(text.contains("foundry_lookup_count"));
        assert!(text.ends_with('\n'));
    }

    #[test]
    fn prometheus_export_records_refresh_self_metrics() {
        let metrics = Metrics::new();
        metrics
            .counter("schema_validations")
            .unwrap()
            .inc(None)
            .unwrap();
        let first = metrics.export_prometheus_text().unwrap();
        assert!(first.contains("schema_validations"));
        assert!(!first.contains("prometheus_exporter_refresh_total"));
        let second = metrics.export_prometheus_text().unwrap();
        assert!(second.contains("prometheus_exporter_refresh_total"));
    }
}
