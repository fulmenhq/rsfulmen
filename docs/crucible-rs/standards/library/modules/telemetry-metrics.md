---
title: "Telemetry & Metrics Export"
description: "Lightweight counter and histogram export aligned with the observability logging pipeline."
status: "draft"
last_updated: "2025-10-24"
tags: ["standards", "library", "observability", "metrics"]
---

# Telemetry & Metrics Export

## Overview

This extension module adds a minimal metrics surface to Fulmen helper libraries so that critical paths can be
instrumented from day zero. Events are emitted as JSON (stdout or structured logs) and validated against
`schemas/observability/metrics/v1.0.0/metrics-event.schema.json`, which in turn references the shared
metrics taxonomy (`config/taxonomy/metrics.yaml`).

## Principles

- **Taxonomy-Backed Identifiers** – Metric names/units come from the taxonomy file; libraries MUST not invent
  ad-hoc identifiers without upstreaming them into Crucible.
- **Scalar or Histogram** – `value` MAY be a scalar number (counter/gauge) or a histogram summary object with
  OTLP-style buckets (`{count,sum,buckets[]}`).
- **JSON-First Export** – Metrics are serialised to JSON for pipelines/agents; libraries may adapt the data to
  other sinks but the canonical format is JSON.
- **No External Dependencies** – Implementations use standard library data structures and ship offline.
- **Optional Adoption** – Module is recommended (extension tier) but not mandatory; coverage targets set
  expectations per language.

## API Surface

- `metrics.counter(name).inc(delta?)`
- `metrics.gauge(name).set(value)`
- `metrics.histogram(name).observe(value)`
- `metrics.export()` → array of schema-compliant events
- `metrics.flush()` → emit & clear events, optionally publishing through the logging pipeline

## Default Histogram Buckets (ADR-0007)

Per [ADR-0007](../../architecture/decisions/ADR-0007-telemetry-default-histogram-buckets.md) Fulmen libraries
MUST use the following default bucket boundaries (in **milliseconds**) for any histogram whose metric name ends
with the `_ms` suffix. Implementations MAY allow overrides on a per-metric basis, but the defaults ensure
consistent aggregation and dashboards across languages.

```
[1, 5, 10, 50, 100, 500, 1000, 5000, 10000]
```

These buckets cover fast in-memory operations (≤10 ms), moderate operations (10 – 100 ms), slower I/O/network
workloads (100 – 1 000 ms), and long-running tasks (1 – 10 s). Libraries SHOULD expose helpers that pick the
default buckets automatically when the metric unit in the taxonomy is `ms`.

## Instrumentation Patterns by Module Type

Helper libraries share common module shapes. Pick the instrumentation pattern that balances observability with
runtime overhead so latency-heavy drivers get histograms, hot loops avoid unnecessary cost, and
security-sensitive actions retain the audit signals they need.

### Standard Pattern (Histogram + Counter)

Use for moderate-frequency operations where latency variability is meaningful:

- File and config I/O (config loaders, schema reads)
- Network-bound discovery or sync operations
- Validation flows and other work that may regress silently

Pattern guidance:

1. Capture start time at the beginning of the operation.
2. Use `try/finally` (Python/TypeScript) or `defer` (Go) to record elapsed time into a histogram metric.
3. Increment a success counter in the normal path and an error counter inside exception handling.

This pattern surfaces both throughput and latency information, making regressions easy to detect.

### Performance-Sensitive Pattern (Counter-Only)

Use when the operation is invoked in tight loops or high-volume pipelines where histogram timing cost outweighs
its value:

- Hash computations and encoder/decoder loops (FulHash)
- In-memory catalog lookups (Foundry taxonomy, key/value caches)
- Text normalization and tokenisation hot paths (Docscribe, similarity scoring)

Guidance:

- Increment a counter for successful operations; optionally add an error counter.
- Skip histogram timing; in benchmarked helper libraries the counter cost is <10 ns, while histogram timing
  adds 50 – 100 ns per invocation.
- Document the choice in module-specific ADRs when additional context (call rates, SLAs) is helpful.

### Audit & Compliance Pattern (Counter + Audit Event)

Use when operations must produce audit trails or may leak sensitive data through timing side-channels:

- Authentication/authorization checks and credential refresh
- Privileged configuration or cache mutation paths (e.g., forced sync, flush)
- Security posture probes or rate-limit enforcement

Guidance:

- Emit attempt, success, and failure counters so monitoring captures drift in behaviour.
- Produce a structured audit log event per ADR-0003 with sensitive values redacted; correlate via request or
  operation IDs.
- Avoid histogram timing unless explicitly approved—latency distributions can reveal sensitive workload details.
- Where ongoing state matters (e.g., active sessions), expose a gauge metric updated alongside counters.

### Module-Specific Recommendations

| Module Type           | Recommended Pattern                         | Metrics                                                                  | Rationale                                                    |
| --------------------- | ------------------------------------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------ |
| File / Config I/O     | Standard (histogram + counter)              | `*_ms` latency histogram, success/error counters                         | Latency variance indicates filesystem or network issues      |
| Validation            | Standard (histogram + counter)              | Per-operation latency, error counter                                     | Catch schema drifts and slow validation runs                 |
| Hashing               | Performance-sensitive (counter-only)        | Success counter, error counter                                           | Called tens of thousands of times; timing adds notable cost  |
| Catalog Lookup        | Performance-sensitive (counter-only)        | Lookup counter                                                           | Pure in-memory; latency already microseconds                 |
| Text Processing       | Performance-sensitive (counter-only)        | Operation counter, optional error counter                                | High-frequency workloads; counts sufficient for profiling    |
| Security / Audit Flow | Audit & compliance (counters + audit event) | Attempt/success/failure counters, gauge if applicable, audit log payload | Requires traceable audit trail without revealing timing data |

Apply these recommendations consistently across gofulmen, pyfulmen, tsfulmen, and future foundations so
observability behaviour remains predictable regardless of language.

## HTTP Server Metrics

Applications building HTTP servers using Fulmen helpers SHOULD instrument their HTTP handlers with standardized HTTP server metrics to enable consistent observability across services. These metrics use the `http_*` namespace and are designed for application-level HTTP servers (Express, Fastify, Go net/http, Python FastAPI/Flask, etc.).

### HTTP Metrics Specification

| Metric Name                     | Unit    | Type      | Description                                           | Required Labels                        | Optional Labels |
| ------------------------------- | ------- | --------- | ----------------------------------------------------- | -------------------------------------- | --------------- |
| `http_requests_total`           | `count` | Counter   | Total HTTP requests received by the server.           | `method`, `route`, `status`, `service` | `outcome`       |
| `http_request_duration_seconds` | `s`     | Histogram | Request duration in seconds from receipt to response. | `method`, `route`, `status`, `service` | `outcome`       |
| `http_request_size_bytes`       | `bytes` | Histogram | Request body size in bytes.                           | `method`, `route`, `service`           | None            |
| `http_response_size_bytes`      | `bytes` | Histogram | Response body size in bytes.                          | `method`, `route`, `status`, `service` | `outcome`       |
| `http_active_requests`          | `count` | Gauge     | Number of HTTP requests currently being processed.    | `service`                              | None            |

### Label Value Standards

| Label     | Description                                                    | Examples                                | Cardinality Warning                                        |
| --------- | -------------------------------------------------------------- | --------------------------------------- | ---------------------------------------------------------- |
| `method`  | HTTP method (uppercase)                                        | `GET`, `POST`, `PUT`, `DELETE`          | Low (typically <10 values)                                 |
| `route`   | **Templated/normalized** route path (NOT literal request path) | `/users/:id`, `/api/v1/orders/:orderId` | **CRITICAL**: Use templates to avoid cardinality explosion |
| `status`  | HTTP status code (numeric)                                     | `200`, `404`, `500`                     | Low (typically <50 values)                                 |
| `service` | Service identifier                                             | `api-gateway`, `auth-service`           | Low (one per service)                                      |
| `outcome` | Optional HTTP status group derived from `status`               | `2xx`, `4xx`, `5xx`                     | Very low (3-5 values); use for simplified dashboards       |

### Route Normalization (CRITICAL)

**DO NOT use literal request paths as the `route` label value.** Literal paths create unbounded cardinality:

```typescript
// ❌ WRONG - Creates one time series per user ID
http_requests_total{method="GET", route="/users/123", status="200"} 1
http_requests_total{method="GET", route="/users/456", status="200"} 1
http_requests_total{method="GET", route="/users/789", status="200"} 1
// Result: Thousands/millions of time series = metric series explosion

// ✅ CORRECT - Uses route template
http_requests_total{method="GET", route="/users/:id", status="200"} 3
// Result: Single time series aggregating all user requests
```

**Implementation Guidance**:

1. **Express (TypeScript/Node.js)**: Extract from `req.route.path`

   ```typescript
   // Express: req.route.path gives "/users/:id"
   const route = req.route?.path || "unknown";
   metrics
     .counter("http_requests_total")
     .labels({
       method: req.method,
       route,
       status: res.statusCode,
       service: "api",
     })
     .inc();
   ```

2. **Fastify (TypeScript/Node.js)**: Extract from `request.routeOptions.url`

   ```typescript
   // Fastify: request.routeOptions.url gives "/users/:id"
   const route = request.routeOptions?.url || "unknown";
   metrics
     .counter("http_requests_total")
     .labels({
       method: request.method,
       route,
       status: reply.statusCode,
       service: "api",
     })
     .inc();
   ```

3. **Go chi router**: Extract from `RouteContext`

   ```go
   // chi: chi.RouteContext(r.Context()).RoutePattern()
   route := chi.RouteContext(r.Context()).RoutePattern()
   metrics.Counter("http_requests_total").
     WithLabels(map[string]string{
       "method": r.Method,
       "route": route, // "/users/{id}"
       "status": strconv.Itoa(statusCode),
       "service": "api",
     }).Inc()
   ```

4. **Go gin framework**: Extract from `c.FullPath()`

   ```go
   // gin: c.FullPath() gives "/users/:id"
   route := c.FullPath()
   metrics.Counter("http_requests_total").
     WithLabels(map[string]string{
       "method": c.Request.Method,
       "route": route, // "/users/:id"
       "status": strconv.Itoa(c.Writer.Status()),
       "service": "api",
     }).Inc()
   ```

5. **Go httprouter**: Store pattern during route registration

   ```go
   // httprouter: Must store pattern when registering route
   // In route registration:
   router.GET("/users/:id", func(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
       // Store pattern in context or use middleware wrapper
       route := "/users/:id" // Pattern from registration
       metrics.Counter("http_requests_total").
         WithLabels(map[string]string{
           "method": r.Method,
           "route": route,
           "status": strconv.Itoa(statusCode),
           "service": "api",
         }).Inc()
   })
   ```

6. **Python FastAPI**: Use `request.scope["route"]`

   ```python
   # FastAPI: request.scope["route"] provides route pattern
   route = request.scope.get("route", "unknown")
   metrics.counter("http_requests_total").labels(
       method=request.method,
       route=route,  # "/users/{id}"
       status=response.status_code,
       service="api"
   ).inc()
   ```

7. **Fallback normalization**: If route template unavailable, use canonical helper

   ```typescript
   // Canonical normalizeRoute helper for frameworks without route introspection
   function normalizeRoute(path: string): string {
     return path
       .replace(
         /\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/gi,
         "/:uuid",
       ) // UUIDs
       .replace(/\/\d+/g, "/:id") // Numeric IDs
       .replace(/\/[0-9a-f]{24}/g, "/:objectid"); // MongoDB ObjectIDs
   }
   ```

**Recommendation**: Fulmen helper libraries implementing HTTP metrics SHOULD provide a framework-specific route extractor with fallback to a `normalizeRoute()` utility function per language idioms.

### Histogram Bucket Defaults

**HTTP Request Duration** (`http_request_duration_seconds`):

- **Unit**: Seconds (helpers exposing milliseconds MUST document conversion)
- **Buckets**: `[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10]`
- **Coverage**: 5ms (fast in-memory) to 10s (slow backend calls)

**HTTP Request/Response Size** (`http_request_size_bytes`, `http_response_size_bytes`):

- **Unit**: Bytes
- **Buckets**: `[1024, 10240, 102400, 1048576, 10485760, 104857600]`
- **Coverage**: 1KB to 100MB (1KB, 10KB, 100KB, 1MB, 10MB, 100MB)

### Unit Conversion Requirements

**CRITICAL**: The taxonomy defines metric units as **seconds** (not milliseconds) for duration and **bytes** for sizes. Helper libraries MUST emit values in these canonical units to ensure bucket alignment across languages.

**Duration metrics** (`http_request_duration_seconds`):

- Taxonomy unit: **seconds**
- Helper APIs MAY expose millisecond interfaces for developer convenience
- **MUST convert milliseconds to seconds before calling histogram.observe()**

**Size metrics** (`http_request_size_bytes`, `http_response_size_bytes`):

- Taxonomy unit: **bytes**
- Helpers MUST emit byte values directly (no conversion needed)
- Avoid kilobytes, megabytes, or other units

#### TypeScript/Node.js Conversion Example

```typescript
import { metrics } from "@fulmenhq/tsfulmen";

// ❌ WRONG - Emitting milliseconds into seconds histogram
const startMs = Date.now();
// ... handle request ...
const durationMs = Date.now() - startMs;
metrics.histogram("http_request_duration_seconds").observe(durationMs); // WRONG!

// ✅ CORRECT - Convert milliseconds to seconds before emitting
const startMs = Date.now();
// ... handle request ...
const durationMs = Date.now() - startMs;
const durationSeconds = durationMs / 1000; // Convert to seconds
metrics.histogram("http_request_duration_seconds").observe(durationSeconds);

// ✅ ALTERNATIVE - Helper wrapper with automatic conversion
function observeDuration(metricName: string, durationMs: number) {
  const durationSeconds = durationMs / 1000;
  metrics.histogram(metricName).observe(durationSeconds);
}

const startMs = Date.now();
// ... handle request ...
observeDuration("http_request_duration_seconds", Date.now() - startMs);
```

#### Go Conversion Example

```go
import (
	"time"
	"github.com/fulmenhq/gofulmen/telemetry"
)

// ❌ WRONG - Emitting milliseconds or nanoseconds
start := time.Now()
// ... handle request ...
durationMs := time.Since(start).Milliseconds()
telemetry.Histogram("http_request_duration_seconds").Observe(float64(durationMs)) // WRONG!

// ✅ CORRECT - Use time.Since().Seconds() for automatic conversion
start := time.Now()
// ... handle request ...
durationSeconds := time.Since(start).Seconds() // Returns float64 in seconds
telemetry.Histogram("http_request_duration_seconds").Observe(durationSeconds)

// ✅ ALTERNATIVE - Explicit conversion if needed
duration := time.Since(start)
durationSeconds := duration.Seconds() // Or: float64(duration) / float64(time.Second)
telemetry.Histogram("http_request_duration_seconds").Observe(durationSeconds)
```

#### Python Conversion Example

```python
import time
from fulmenhq.pyfulmen import metrics

# ❌ WRONG - Emitting milliseconds
start = time.time()
# ... handle request ...
duration_ms = (time.time() - start) * 1000
metrics.histogram("http_request_duration_seconds").observe(duration_ms)  # WRONG!

# ✅ CORRECT - time.time() already returns seconds
start = time.time()
# ... handle request ...
duration_seconds = time.time() - start  # Already in seconds
metrics.histogram("http_request_duration_seconds").observe(duration_seconds)

# ✅ ALTERNATIVE - Explicit conversion if using perf_counter
start = time.perf_counter()
# ... handle request ...
duration_seconds = time.perf_counter() - start  # perf_counter also returns seconds
metrics.histogram("http_request_duration_seconds").observe(duration_seconds)
```

**Size Handling** (no conversion needed):

```typescript
// TypeScript: Content-Length headers are already in bytes
const reqSize = parseInt(req.get("content-length") || "0", 10); // Bytes
metrics.histogram("http_request_size_bytes").observe(reqSize); // ✅ Correct
```

```go
// Go: ContentLength is already in bytes
reqSize := r.ContentLength // int64, bytes
telemetry.Histogram("http_request_size_bytes").Observe(float64(reqSize)) // ✅ Correct
```

```python
# Python: Content-Length headers are already in bytes
req_size = int(request.headers.get("content-length", 0))  # Bytes
metrics.histogram("http_request_size_bytes").observe(req_size)  # ✅ Correct
```

### Outcome Label (Optional)

The `outcome` label groups HTTP status codes into coarse categories for simplified dashboards:

| Outcome | Status Codes | Meaning      |
| ------- | ------------ | ------------ |
| `2xx`   | 200-299      | Success      |
| `4xx`   | 400-499      | Client error |
| `5xx`   | 500-599      | Server error |

**When to use**:

- ✅ High-level dashboards showing success/error rates
- ✅ Alerts based on error rate thresholds

**When NOT to use**:

- ❌ Detailed debugging (use `status` label for specific codes)
- ❌ When you need to distinguish 404 vs 403, 500 vs 503, etc.

**Derivation**: Helpers MAY automatically populate `outcome` from `status`:

```typescript
function deriveOutcome(status: number): string {
  if (status >= 200 && status < 300) return "2xx";
  if (status >= 400 && status < 500) return "4xx";
  if (status >= 500 && status < 600) return "5xx";
  return "other";
}
```

### Implementation Examples

**TypeScript (Express)**:

```typescript
import { metrics } from "@fulmenhq/tsfulmen";

app.use((req, res, next) => {
  const start = Date.now();

  // Track active requests
  metrics.gauge("http_active_requests").labels({ service: "api" }).inc();

  // Capture request size
  const reqSize = parseInt(req.get("content-length") || "0", 10);
  metrics
    .histogram("http_request_size_bytes")
    .labels({
      method: req.method,
      route: req.route?.path || "unknown",
      service: "api",
    })
    .observe(reqSize);

  res.on("finish", () => {
    const duration = (Date.now() - start) / 1000; // Convert ms to seconds
    const route = req.route?.path || "unknown";
    const status = res.statusCode.toString();

    // Record request
    metrics
      .counter("http_requests_total")
      .labels({ method: req.method, route, status, service: "api" })
      .inc();

    // Record duration
    metrics
      .histogram("http_request_duration_seconds")
      .labels({ method: req.method, route, status, service: "api" })
      .observe(duration);

    // Record response size
    const resSize = parseInt(res.get("content-length") || "0", 10);
    metrics
      .histogram("http_response_size_bytes")
      .labels({ method: req.method, route, status, service: "api" })
      .observe(resSize);

    // Decrement active requests
    metrics.gauge("http_active_requests").labels({ service: "api" }).dec();
  });

  next();
});
```

**Go (net/http with chi router)**:

```go
import (
	"github.com/go-chi/chi/v5"
	"github.com/fulmenhq/gofulmen/telemetry"
)

func httpMetricsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()

		// Track active requests
		telemetry.Gauge("http_active_requests").
			WithLabels(map[string]string{"service": "api"}).
			Inc()

		// Capture request size
		reqSize := r.ContentLength
		if reqSize > 0 {
			route := chi.RouteContext(r.Context()).RoutePattern()
			telemetry.Histogram("http_request_size_bytes").
				WithLabels(map[string]string{
					"method": r.Method,
					"route": route,
					"service": "api",
				}).
				Observe(float64(reqSize))
		}

		// Wrap response writer to capture status and size
		wrapped := &responseWriter{ResponseWriter: w, statusCode: 200}
		next.ServeHTTP(wrapped, r)

		// Record metrics after response
		duration := time.Since(start).Seconds()
		route := chi.RouteContext(r.Context()).RoutePattern()
		status := strconv.Itoa(wrapped.statusCode)

		telemetry.Counter("http_requests_total").
			WithLabels(map[string]string{
				"method": r.Method,
				"route": route,
				"status": status,
				"service": "api",
			}).
			Inc()

		telemetry.Histogram("http_request_duration_seconds").
			WithLabels(map[string]string{
				"method": r.Method,
				"route": route,
				"status": status,
				"service": "api",
			}).
			Observe(duration)

		telemetry.Histogram("http_response_size_bytes").
			WithLabels(map[string]string{
				"method": r.Method,
				"route": route,
				"status": status,
				"service": "api",
			}).
			Observe(float64(wrapped.size))

		telemetry.Gauge("http_active_requests").
			WithLabels(map[string]string{"service": "api"}).
			Dec()
	})
}

type responseWriter struct {
	http.ResponseWriter
	statusCode int
	size       int
}

func (rw *responseWriter) WriteHeader(code int) {
	rw.statusCode = code
	rw.ResponseWriter.WriteHeader(code)
}

func (rw *responseWriter) Write(b []byte) (int, error) {
	n, err := rw.ResponseWriter.Write(b)
	rw.size += n
	return n, err
}
```

**Python (FastAPI)**:

```python
from fastapi import FastAPI, Request
from fulmenhq.pyfulmen import metrics
import time

app = FastAPI()

@app.middleware("http")
async def http_metrics_middleware(request: Request, call_next):
    start = time.time()

    # Track active requests
    metrics.gauge("http_active_requests").labels(service="api").inc()

    # Capture request size
    req_size = int(request.headers.get("content-length", 0))
    route = request.scope.get("route", "unknown")
    if req_size > 0:
        metrics.histogram("http_request_size_bytes").labels(
            method=request.method,
            route=route,
            service="api"
        ).observe(req_size)

    # Process request
    response = await call_next(request)

    # Record metrics after response
    duration = time.time() - start
    status = str(response.status_code)

    metrics.counter("http_requests_total").labels(
        method=request.method,
        route=route,
        status=status,
        service="api"
    ).inc()

    metrics.histogram("http_request_duration_seconds").labels(
        method=request.method,
        route=route,
        status=status,
        service="api"
    ).observe(duration)

    # Response size (if available)
    res_size = int(response.headers.get("content-length", 0))
    if res_size > 0:
        metrics.histogram("http_response_size_bytes").labels(
            method=request.method,
            route=route,
            status=status,
            service="api"
        ).observe(res_size)

    metrics.gauge("http_active_requests").labels(service="api").dec()

    return response
```

### Cardinality Budget

Assuming typical application parameters:

- Methods: 5 (GET, POST, PUT, DELETE, PATCH)
- Routes: 50 (templated routes)
- Status codes: 20 (200, 201, 400, 401, 403, 404, 500, 503, etc.)
- Services: 1 per deployment

**Estimated time series per metric**:

- `http_requests_total`: 5 × 50 × 20 × 1 = **5,000 series**
- `http_request_duration_seconds`: 5 × 50 × 20 × 1 × 11 buckets = **55,000 series**
- `http_request_size_bytes`: 5 × 50 × 1 × 6 buckets = **1,500 series**
- `http_response_size_bytes`: 5 × 50 × 20 × 1 × 6 buckets = **30,000 series**
- `http_active_requests`: 1 = **1 series**

**Total**: ~92,000 time series per service

**Mitigation**: This is acceptable for modern Prometheus/Grafana deployments. If cardinality becomes an issue:

1. Reduce route count by grouping similar endpoints
2. Use `outcome` instead of `status` for less critical metrics
3. Sample high-cardinality routes (e.g., only instrument top N routes)

## Prometheus Exporter Metrics

Helper libraries shipping Prometheus exporters (gofulmen, pyfulmen ≥v0.1.10, tsfulmen ≥v0.1.8) emit
exporter-specific telemetry to monitor exporter health and HTTP exposition performance. These metrics use the
`prometheus_exporter_*` namespace reserved for infrastructure internals.

### Exporter Metrics Specification

| Metric Name                                    | Unit    | Type      | Description                                                                   | Required Labels   | Optional Labels |
| ---------------------------------------------- | ------- | --------- | ----------------------------------------------------------------------------- | ----------------- | --------------- |
| `prometheus_exporter_refresh_duration_seconds` | `s`     | Histogram | Time to refresh/export registry data. Uses ADR-0007 buckets ÷ 1000 (seconds). | `phase`, `result` | `error_type`    |
| `prometheus_exporter_refresh_total`            | `count` | Counter   | Total refresh cycles attempted (success/error breakdown via `result` label).  | `result`          | `error_type`    |
| `prometheus_exporter_refresh_errors_total`     | `count` | Counter   | Failed refresh cycles with error classification.                              | `error_type`      | `phase`         |
| `prometheus_exporter_refresh_inflight`         | `count` | Gauge     | Concurrent refresh operations currently running.                              | None              | `phase`         |
| `prometheus_exporter_http_requests_total`      | `count` | Counter   | HTTP scrape/exposition requests handled by exporter.                          | `status`, `path`  | `client`        |
| `prometheus_exporter_http_errors_total`        | `count` | Counter   | HTTP exposition failures (5xx, auth failures, timeouts).                      | `status`, `path`  | `client`        |
| `prometheus_exporter_restarts_total`           | `count` | Counter   | Exporter refresh loop or HTTP server restarts.                                | `reason`          | None            |

**Implementation note**: Type shown for clarity; metric type remains implicit in taxonomy (inferred from API: `counter()`, `histogram()`, `gauge()`).

### Label Value Standards

Standardized label values ensure portable dashboards and alerts:

| Label        | Allowed Values                          | Used By Metrics                                                                            | Notes                                       |
| ------------ | --------------------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------- |
| `phase`      | `collect`, `convert`, `export`          | `prometheus_exporter_refresh_duration_seconds`, `prometheus_exporter_refresh_errors_total` | Refresh pipeline stages                     |
| `result`     | `success`, `error`                      | `prometheus_exporter_refresh_duration_seconds`, `prometheus_exporter_refresh_total`        | Operation outcome                           |
| `error_type` | `validation`, `io`, `timeout`, `other`  | `prometheus_exporter_refresh_errors_total`, optionally on `refresh_*` metrics              | Error classification                        |
| `status`     | `200`, `500`, `503`, etc.               | `prometheus_exporter_http_requests_total`, `prometheus_exporter_http_errors_total`         | HTTP status codes                           |
| `path`       | `/metrics`, `/health`, etc.             | `prometheus_exporter_http_requests_total`, `prometheus_exporter_http_errors_total`         | HTTP endpoint paths                         |
| `client`     | `<ip>`, `<service_name>`, etc.          | Optional for HTTP metrics                                                                  | Scraper ID (high cardinality—use sparingly) |
| `reason`     | `config_change`, `error`, `manual`, etc | `prometheus_exporter_restarts_total`                                                       | Restart trigger                             |

**Cardinality warning**: The `client` label creates one time series per unique client. Only use when multiple scrapers require differentiation; omit otherwise to prevent metric series explosion.

### Histogram Bucket Conversion

**Critical**: Prometheus histograms use **seconds**, not milliseconds.

ADR-0007 default buckets (milliseconds):

```
[1, 5, 10, 50, 100, 500, 1000, 5000, 10000]
```

**Exporters MUST divide by 1000 before emitting** to Prometheus:

```
[0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]  (seconds)
```

**Warning**: Do not double-convert; exporters should convert once at emission time. The `_seconds` suffix in the metric name signals that values are already in seconds.

### Dual Counter Emission Rules

Exporters emit both `prometheus_exporter_refresh_total` and `prometheus_exporter_refresh_errors_total` to serve different query patterns:

**Emission Rules**:

1. **Always emit**: `prometheus_exporter_refresh_total{result="success"}` for successful refreshes
2. **Always emit**: `prometheus_exporter_refresh_total{result="error"}` for failed refreshes
3. **Additionally emit**: `prometheus_exporter_refresh_errors_total{error_type="..."}` for errors with detailed classification
4. **Invariant**: `sum(prometheus_exporter_refresh_errors_total) MUST equal sum(prometheus_exporter_refresh_total{result="error"})`

**Use case separation**:

- Use `refresh_total` for overall throughput and success rate dashboards
- Use `refresh_errors_total` for detailed error breakdown by type and phase

**Example PromQL**:

```promql
# Overall success rate
rate(prometheus_exporter_refresh_total{result="success"}[5m])
  / rate(prometheus_exporter_refresh_total[5m])

# Error breakdown by type
sum by (error_type) (rate(prometheus_exporter_refresh_errors_total[5m]))
```

### Prometheus Client Default Metrics

⚠️ **CRITICAL WARNING for TypeScript/Node.js (prom-client)**: The `prom-client` library automatically emits `http_requests_total` and `http_request_duration_seconds` when using Express middleware, which **COLLIDES with taxonomy HTTP metrics**. This creates duplicate time series with different label sets, breaking dashboards and queries. You MUST disable prom-client defaults before instrumenting with taxonomy metrics.

Many Prometheus client libraries emit default metrics:

- Node.js (`prom-client`): `http_requests_total`, `http_request_duration_seconds`, `process_*`, `nodejs_*` ⚠️ **Collides with HTTP taxonomy metrics**
- Python (`prometheus-client`): `process_*`, `python_gc_*`, platform metrics (no HTTP collision)
- Go (`prometheus/client_golang`): `go_*`, `process_*` metrics (no HTTP collision)

**Recommendation**: Disable client library defaults to prevent duplicate time series and reduce cardinality.

**TypeScript/Node.js (prom-client)**:

```typescript
import { register } from "prom-client";

// Option A: Clear all default metrics
register.clear();

// Option B: Disable default collection
import { collectDefaultMetrics } from "prom-client";
collectDefaultMetrics({ register: null });
```

**Python (prometheus-client)**:

```python
from prometheus_client import REGISTRY, PROCESS_COLLECTOR, PLATFORM_COLLECTOR, GC_COLLECTOR

# Clear default collectors
REGISTRY.unregister(PROCESS_COLLECTOR)
REGISTRY.unregister(PLATFORM_COLLECTOR)
REGISTRY.unregister(GC_COLLECTOR)
```

**Go (prometheus/client_golang)**:

```go
// Use custom registry instead of default
registry := prometheus.NewRegistry()
// Register only taxonomy metrics, not default Go collectors
```

**If defaults must remain**: Document namespace distinction clearly:

- Client defaults: `http_*`, `process_*`, `go_*`, etc.
- Taxonomy metrics: `prometheus_exporter_http_*` (exporter-specific)

Dashboards should reference taxonomy metrics for consistency across languages.

### Compatibility Matrix

| Library  | Prometheus Exporter | Taxonomy Support | Notes                  |
| -------- | ------------------- | ---------------- | ---------------------- |
| gofulmen | ✅ Implemented      | ✅ v0.2.7+       | Uses taxonomy metrics  |
| pyfulmen | ✅ Implemented      | ✅ v0.1.10+      | Uses taxonomy metrics  |
| tsfulmen | 🔄 In progress      | ✅ v0.1.8+       | Implementing in v0.1.8 |

**Minimum Crucible version**: ≥v0.2.7 for `prometheus_exporter_*` metrics and `s` (seconds) unit support.

## Metric Namespace Governance

### Module Prefix Reservation

**Module prefixes are reserved by Crucible** to ensure cross-language consistency and prevent namespace collisions. Each Fulmen module reserves a prefix for its metrics:

**Reserved Prefixes**:

- `foundry_*` — Foundry catalog and pattern matching operations
- `pathfinder_*` — File discovery and validation
- `error_handling_*` — Error wrapping and propagation
- `fulhash_*` — Hashing operations
- `config_load_*` — Configuration loading
- `schema_validation_*` — Schema validation
- `logging_emit_*` — Log emission
- `prometheus_exporter_*` — Prometheus exporter internals
- _(Additional modules reserve prefixes as they're added)_

**Purpose**: Reserved prefixes enable ecosystem-wide dashboards and alerts that compare module behavior across:

- Languages (gofulmen vs pyfulmen vs tsfulmen)
- Services (multiple apps using the same module)
- Environments (dev vs staging vs production)

### Required Module Metrics

Each Fulmen module SHOULD define **required metrics** in the taxonomy (`config/taxonomy/metrics.yaml`) to enable cross-language observability. When a module is implemented in multiple languages, all implementations MUST emit the required metrics with identical names, labels, and semantics.

**Example: Foundry Module**

```yaml
# Required metrics (in taxonomy)
- foundry_lookup_count # Catalog lookup operations
- foundry_mime_detections_total # MIME type detections
- foundry_mime_detection_ms # MIME detection latency
```

Libraries implementing Foundry (gofulmen, pyfulmen, tsfulmen) MUST emit these metrics for module compliance and cross-language consistency.

### Implementer Extensions

**Library implementers MAY add additional metrics** for their module implementations beyond taxonomy requirements, recognizing they may need to adapt if Crucible later adds metrics with the same names.

✅ **Allowed**:

```python
# pyfulmen adds Python-specific Foundry metric (not in taxonomy)
metrics.counter("foundry_cache_hits_total").inc()
```

⚠️ **Caveat**: If Crucible later adds `foundry_cache_hits_total` to the taxonomy with different semantics, pyfulmen must adapt its implementation to match.

**Best Practice**: Petition Crucible (via PR to `config/taxonomy/metrics.yaml`) to add metrics that would benefit ecosystem-wide observability. Metrics useful across all language implementations should be standardized.

**Petition Process**:

1. Open PR adding metric to `config/taxonomy/metrics.yaml`
2. Include description, unit, required labels
3. Provide rationale for cross-language value
4. Wait for approval from `MAINTAINERS.md` before emitting in production

### Application Metric Rules

**Applications MUST NOT use module prefixes** for their business logic metrics unless they meet specific criteria.

**Allowed Application Prefixes**:

- Binary name: `percheron_*`, `groningen_*`, `sumpter_*`
- Organization: `acmecorp_*`, `example_*`
- Service: `api_gateway_*`, `auth_service_*`

**Forbidden**:

- Module prefixes: `foundry_*`, `pathfinder_*`, etc. (reserved for Fulmen modules)
- Infrastructure prefixes: `prometheus_exporter_*` (reserved for exporters)

**Exception**: If an application binary name matches a Fulmen module name (e.g., app named `foundry`), the application:

1. MUST obtain maintainer approval from `MAINTAINERS.md`
2. MUST emit all required module metrics with standard semantics
3. MUST NOT add non-standard metrics using the module prefix

**Application Metric Emission**:
Applications emit their metrics via **Prometheus client libraries directly**, NOT through `metrics-event.schema.json`:

```typescript
// Application code - use Prometheus client directly
import { Counter } from "prom-client";
const jobsProcessed = new Counter({
  name: "percheron_jobs_processed_total",
  help: "Total jobs processed by Percheron",
});
jobsProcessed.inc();
```

**Not**:

```typescript
// Wrong - applications don't use taxonomy for business metrics
metrics.counter("percheron_jobs_processed_total").inc(); // Won't work - not in taxonomy
```

### Three-Tier Metric Architecture

| Tier | Scope                   | In Taxonomy | Emitted Via                 | Prefix Examples                           | Purpose                             |
| ---- | ----------------------- | ----------- | --------------------------- | ----------------------------------------- | ----------------------------------- |
| 1    | **Module Internals**    | ✅ Yes      | `metrics-event.schema.json` | `foundry_*`, `pathfinder_*`, `fulhash_*`  | Cross-language module observability |
| 2    | **Infrastructure**      | ✅ Yes      | `metrics-event.schema.json` | `prometheus_exporter_*`                   | Infrastructure health monitoring    |
| 3    | **Application Metrics** | ❌ No       | Prometheus client libraries | `percheron_*`, `groningen_*`, `sumpter_*` | Application business logic          |

**Key Distinction**:

- **Tier 1 & 2**: Module/infrastructure metrics standardized across all helper libraries → in taxonomy
- **Tier 3**: Application-specific business metrics → NOT in taxonomy, apps choose their own

### Collision Prevention

| Scenario                                    | Rule                                 | Example                                                         |
| ------------------------------------------- | ------------------------------------ | --------------------------------------------------------------- |
| Fulmen module                               | Prefix reserved, metrics in taxonomy | `foundry_lookup_count` (in taxonomy)                            |
| Helper library extending module             | Can add metrics with adaptation risk | `foundry_cache_hits_total` (pyfulmen-specific, not in taxonomy) |
| Application (different name)                | Use binary prefix, NOT in taxonomy   | `percheron_job_duration_ms` (emitted by app)                    |
| Application (same name as module)           | Requires approval + conformance      | App named `foundry` must emit standard `foundry_*`              |
| Application attempting to use module prefix | ❌ Forbidden without approval        | App cannot emit `foundry_custom_metric` arbitrarily             |

**Example Collision Scenario**:

```python
# pyfulmen implements Foundry module
metrics.counter("foundry_lookup_count").inc()  # ✅ Required metric (in taxonomy)
metrics.counter("foundry_cache_hits_total").inc()  # ✅ Extension (not in taxonomy, adaptation risk)

# Application using pyfulmen
from prom_client import Counter
app_jobs = Counter("percheron_jobs_total")  # ✅ Application metric (binary prefix)
app_jobs.inc()

# Wrong - application trying to use module prefix
foundry_custom = Counter("foundry_custom_metric")  # ❌ Forbidden - module prefix
```

## Language Examples

**Go**

```go
metrics.Counter("schema_validations").Inc()
metrics.Histogram("config_load_ms").Observe(42)
if err := telemetry.Validate(metrics.Export()); err != nil {
    log.Fatal(err)
}
metrics.Flush()
```

**TypeScript**

```typescript
metrics.counter("schema_validations").inc();
metrics.histogram("config_load_ms").observe(42);
assert(validateMetrics(metrics.export()), "invalid payload");
metrics.flush();
```

**Python**

```python
metrics.counter("schema_validations").inc()
metrics.histogram("config_load_ms").observe(42)
assert validate_metrics(metrics.export())
metrics.flush()
```

## Testing Expectations

- **Coverage**: ≥85 % covering counters, gauges, histograms, tags, and flush behaviour.
- **Fixtures**: `tests/fixtures/metrics/` should include scalar and histogram examples plus failure cases (unknown
  metric, invalid buckets).
- **Integration**: Validate alongside logging middleware (metrics events can be appended to log payloads) and
  schema validation flows.
- **Tooling**: goneat `schema validate` using the updated metrics schema and taxonomy definitions.

## Ecosystem Ties

- **SSOT**: This document (`docs/standards/library/modules/telemetry-metrics.md`) is the **canonical reference** for telemetry implementation guidance. Library teams MUST reference this document when implementing telemetry modules.
- **Taxonomy**: Metric names, units, and required metrics defined in `config/taxonomy/metrics.yaml` (synced to language wrappers via `make sync`).
- **Logging**: Metrics events may be embedded inside progressive logging payloads for SIMPLE/STRUCTURED profiles.
- **Assessment**: Future goneat assess categories can rely on the taxonomy to audit mandatory metrics.
- **Extensibility**: New metrics are proposed via Crucible PRs updating `config/taxonomy/metrics.yaml` so all
  languages stay in sync.
- **ADR Mapping**: Default histogram buckets (ADR-0007), instrumentation patterns (ADR-0008).
