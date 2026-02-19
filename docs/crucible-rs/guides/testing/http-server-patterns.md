---
title: "HTTP Server Testing Patterns"
description: "Compliance routing and practical patterns for HTTP server and fixture implementations"
author: "entarch (AI)"
date: "2026-01-09"
last_updated: "2026-01-12"
status: "active"
tags: ["guides", "testing", "http", "server", "fixture", "openapi", "v0.4.6"]
---

# HTTP Server Testing Patterns

This guide helps developers implementing HTTP servers (including fixtures) discover applicable standards and avoid common anti-patterns. It serves as a **compliance routing document** - directing you to normative standards while providing practical examples.

## Compliance Requirements

Before implementing HTTP server code, ensure compliance with these standards. Note that **workhorse/services** and **fixtures** have different envelope requirements.

| Standard                                                                   | Scope                       | Key Requirements                                                  |
| -------------------------------------------------------------------------- | --------------------------- | ----------------------------------------------------------------- |
| [HTTP REST Standard](../../standards/protocol/http-rest-standards.md)      | **Workhorse/services only** | Response envelope, versioned paths (`/api/v1/`), health endpoints |
| [Go Coding Standards](../../standards/coding/go.md)                        | Go implementations          | Handler error patterns, output hygiene, context handling          |
| [Python Coding Standards](../../standards/coding/python.md)                | Python implementations      | Async patterns, error handling                                    |
| [TypeScript Coding Standards](../../standards/coding/typescript.md)        | TypeScript implementations  | Type safety, error handling                                       |
| [Fixture Standard](../../architecture/fulmen-fixture-standard.md)          | Fixture repos only          | CLI commands, port management, scenarios, observability           |
| [Observability Logging Standard](../../standards/observability/logging.md) | All servers                 | Structured logging, log levels, correlation IDs                   |

### Workhorse vs Fixture Envelope Requirements

| Aspect            | Workhorse/Services                                       | Fixtures                                                                                       |
| ----------------- | -------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Response envelope | MUST use `{success, data, error}` per HTTP REST Standard | MAY diverge where test purpose requires (binary streams, raw bytes, simple `{error, message}`) |
| Versioned paths   | MUST use `/api/v1/` prefix                               | NOT required (endpoint paths serve test semantics)                                             |
| Health endpoints  | MUST follow standard schema                              | SHOULD follow standard for `/health`, but endpoint-specific responses are acceptable           |
| Correlation IDs   | MUST propagate                                           | SHOULD propagate                                                                               |

**Rationale**: Fixtures exist to test client behavior, not to model production APIs. Binary endpoints, intentionally malformed responses, and test-specific error formats serve legitimate test purposes.

### Pre-Implementation Checklist

- [ ] Identify server type: workhorse/service OR fixture
- [ ] If workhorse: Read HTTP REST Standard sections 2-4 (endpoints, schema, status codes)
- [ ] Read language-specific coding standard (Go §8 for anti-patterns)
- [ ] If building a fixture: Read Fixture Standard (CLI, port management, scenarios)
- [ ] Review anti-pattern table below
- [ ] Confirm health endpoint compliance (`/health`, `/health/ready`)

### Pre-Review Checklist (for devrev)

- [ ] **Workhorse only**: Response envelope matches HTTP REST Standard schema
- [ ] **Fixtures**: Response format documented in endpoint docs; divergence from standard justified
- [ ] Correlation ID propagation implemented (accept, generate if missing, return, log)
- [ ] Health endpoints return correct format
- [ ] Structured logging includes: method, path, status, duration, correlation_id
- [ ] No bare `time.Sleep()` in handlers (context-aware delays only)
- [ ] Error handling follows language coding standard
- [ ] If fixture: CLI commands (`version`, `doctor`) implemented per Fixture Standard

## Go HTTP Handler Anti-Patterns

These patterns were derived from 25+ lint fixes during rampart/gauntlet fixture development.

### Error Handling in Response Writers

**Problem**: `json.Encoder.Encode()` returns an error that golangci-lint catches when ignored.

**Anti-pattern** (10+ occurrences fixed):

```go
w.Header().Set("Content-Type", "application/json")
w.WriteHeader(http.StatusOK)
json.NewEncoder(w).Encode(response)  // Error return value not checked
```

**Correct pattern** (explicit ignore after headers sent):

```go
w.Header().Set("Content-Type", "application/json")
w.WriteHeader(http.StatusOK)
_ = json.NewEncoder(w).Encode(response)  // Explicit ignore signals intent
```

**Why explicit ignore?** After `WriteHeader()` is called, HTTP status is committed. If `Encode()` fails (client disconnects), we can't change the response. The blank identifier signals intentional ignore to linters and maintainers.

**Alternative** (pre-header error handling):

```go
data, err := json.Marshal(response)
if err != nil {
    http.Error(w, "internal error", http.StatusInternalServerError)
    return
}
w.Header().Set("Content-Type", "application/json")
w.WriteHeader(http.StatusOK)
_, _ = w.Write(data)
```

### Response Body Close in Tests

**Problem**: `resp.Body.Close()` returns an error that golangci-lint catches.

**Anti-pattern** (15+ occurrences fixed):

```go
resp := w.Result()
defer resp.Body.Close()  // Error return value not checked
```

**Correct pattern** (per-file test helper):

```go
// closeBody is a test helper that closes a response body and fails on error
func closeBody(t *testing.T, body io.Closer) {
    t.Helper()
    if err := body.Close(); err != nil {
        t.Errorf("failed to close response body: %v", err)
    }
}

// Usage
func TestMyHandler(t *testing.T) {
    resp := w.Result()
    defer closeBody(t, resp.Body)  // Error handled, test fails on close error
    // ... assertions ...
}
```

**Why per-file helpers?** Go's `_test.go` external package pattern means test helpers aren't shared across files. Options:

1. **Per-file helpers** (simplest): Name uniquely (`closeBodyEcho`, `closeBodyStatus`)
2. **Shared test package**: Create `internal/testutil/helpers.go`
3. **Test main**: Use `TestMain` for shared setup

### Context Cancellation in Delay Handlers

**Problem**: Handlers that sleep must respect context cancellation for graceful shutdown.

**Anti-pattern**:

```go
func DelayHandler(w http.ResponseWriter, r *http.Request) {
    time.Sleep(time.Duration(ms) * time.Millisecond)  // Ignores cancellation
    // ... respond ...
}
```

**Correct pattern**:

```go
func DelayHandler(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()
    select {
    case <-time.After(time.Duration(ms) * time.Millisecond):
        // Timer completed, send response
    case <-ctx.Done():
        // Client disconnected or server shutting down
        return  // Don't send response to dead connection
    }
    // ... respond ...
}
```

**Helper function** (used in rampart):

```go
// sleepWithContext sleeps for duration, returns true if completed, false if cancelled.
// Uses time.NewTimer with proper cleanup to avoid goroutine leaks in long-lived servers.
func sleepWithContext(ctx context.Context, d time.Duration) bool {
    timer := time.NewTimer(d)
    defer timer.Stop()  // Critical: prevents timer goroutine leak

    select {
    case <-timer.C:
        return true
    case <-ctx.Done():
        return false
    }
}
```

**Note**: Prefer `time.NewTimer` + `defer timer.Stop()` over `time.After` in long-lived servers. `time.After` creates a timer that can't be stopped if the context cancels first, leading to goroutine accumulation under load.

### Body Limits and Safe Reads

**Problem**: Unbounded `io.ReadAll` on request body enables memory DoS.

**Anti-pattern**:

```go
body, err := io.ReadAll(r.Body)  // Unbounded - attacker sends 2GB
```

**Correct pattern** (bounded reads):

```go
const maxBodySize = 1 << 20  // 1MB

limited := io.LimitReader(r.Body, maxBodySize+1)
body, err := io.ReadAll(limited)
if err != nil {
    http.Error(w, "read error", http.StatusInternalServerError)
    return
}
if len(body) > maxBodySize {
    http.Error(w, "request too large", http.StatusRequestEntityTooLarge)
    return
}
```

**For sink endpoints** (discard body with 413 on overflow):

```go
// Sink pattern: discard body but enforce limit
limited := io.LimitReader(r.Body, maxBodySize+1)
n, _ := io.Copy(io.Discard, limited)
if n > maxBodySize {
    http.Error(w, "request too large", http.StatusRequestEntityTooLarge)
    return
}
// Body within limit, proceed with response
```

## Correlation ID Propagation

Correlation IDs enable request tracing across logs, responses, and downstream services.

### Requirements

1. **Accept** `X-Correlation-ID` or `X-Request-ID` from client
2. **Generate** if missing (recommend UUIDv7)
3. **Return** `X-Correlation-ID` in response headers
4. **Log** `correlation_id` field in all structured request logs

### Implementation Pattern

```go
// Use a typed context key to avoid collisions
type contextKey struct{ name string }
var correlationKey = contextKey{"correlation_id"}

func correlationMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        correlationID := r.Header.Get("X-Correlation-ID")
        if correlationID == "" {
            correlationID = r.Header.Get("X-Request-ID")
        }
        if correlationID == "" {
            correlationID = generateUUIDv7()  // Or uuid.New().String()
        }

        // Optional: validate format if you want consistency
        // if !isValidUUID(correlationID) { correlationID = generateUUIDv7() }

        // Set response header
        w.Header().Set("X-Correlation-ID", correlationID)

        // Add to context for logging
        ctx := context.WithValue(r.Context(), correlationKey, correlationID)
        next.ServeHTTP(w, r.WithContext(ctx))
    })
}
```

### Common Failure Mode

Enabling correlation in logging middleware config does NOT automatically propagate via HTTP headers. Both are required:

- Logging middleware: Include `correlation_id` in structured logs
- HTTP middleware: Accept/generate/return correlation ID headers

## Streaming Correctness

For endpoints that stream data (e.g., `/drip`, `/stream/{n}`):

### Byte-Count Invariants

- Endpoints defined as "streams {n} bytes" MUST return exactly {n} bytes
- Do NOT append debug footers, newlines, or JSON summaries
- Use HTTP trailers or separate metadata endpoints for end-of-stream stats

### Flush Behavior

```go
func StreamHandler(w http.ResponseWriter, r *http.Request) {
    flusher, ok := w.(http.Flusher)
    if !ok {
        http.Error(w, "streaming not supported", http.StatusInternalServerError)
        return
    }

    for i := 0; i < n; i++ {
        w.Write([]byte{byte(i % 256)})
        flusher.Flush()  // Flush per chunk
    }
}
```

### Client Disconnect Handling

Treat write errors as disconnect - return without attempting error responses:

```go
_, err := w.Write(chunk)
if err != nil {
    return  // Client disconnected, don't try to write error JSON
}
```

## Determinism for Testability

### Range Endpoints

For `/range/{n}` endpoints that support HTTP Range requests:

- Full content and partial ranges MUST align
- Use deterministic pattern: `byte(i) = i % 256`
- Document this explicitly for fixture consumers

**Why?** Clients need to compare full vs partial responses and verify offsets exactly. Random data breaks validation.

### When Randomness is OK

- `/bytes/{n}` may return random data (bulk transfer testing)
- `/range/{n}` should NOT be random (correctness testing)

## OpenAPI Verification

Repositories that publish OpenAPI specifications should verify spec completeness via automated testing. Fixtures MUST implement coverage testing; workhorses SHOULD (if publishing an OpenAPI spec).

### The Spec Drift Problem

A common failure mode:

1. Developer adds new endpoint to router
2. Developer forgets to add OpenAPI annotations
3. Generator produces partial spec (only annotated endpoints)
4. Partial spec ships; consumers assume it's complete

### Coverage Test Pattern

Implement an automated test that compares registered routes against the spec:

```go
func TestOpenAPISpecCoverage(t *testing.T) {
    specPath := "dist/openapi.yaml"
    if _, err := os.Stat(specPath); os.IsNotExist(err) {
        t.Skip("OpenAPI spec not found; run 'make openapi' first")
    }

    // Parse spec paths/methods
    specRoutes := parseOpenAPISpec(t, specPath)

    // Extract routes from router (e.g., parse server.go for mux.HandleFunc calls)
    registeredRoutes := extractRouterRoutes(t, "internal/server/server.go")

    // Compare (respecting intentional exclusions)
    for _, route := range registeredRoutes {
        if isExcluded(route) {
            continue
        }
        if !specRoutes.Contains(route.Method, route.Path) {
            t.Errorf("Route %s %s not documented in OpenAPI spec", route.Method, route.Path)
        }
    }
}
```

### CI Integration

CI pipelines MUST generate the spec before running tests:

```yaml
- run: make openapi
- run: make test # Coverage test now has artifact
```

### Intentional Exclusions

Maintainers MAY exclude endpoints from coverage requirements:

- Experimental endpoints (not yet stable)
- Internal/admin endpoints (`/debug/pprof`)
- Self-referential endpoints (`/openapi.yaml` itself)

**Best practice**: All "going concern" endpoints intended for consumer use SHOULD be documented.

See [ADR-0014: OpenAPI Spec Coverage Tests](../../architecture/decisions/ADR-0014-openapi-spec-coverage.md) for full rationale and cross-language patterns.

## Fixture Author Conformance Checklist

For fixtures specifically, verify before release:

- [ ] Every response includes `X-Correlation-ID`
- [ ] Structured logs include: method, path, status, duration, correlation_id
- [ ] All sleeps are context-aware (no bare `time.Sleep`)
- [ ] Streaming endpoints preserve byte-count contracts
- [ ] Range endpoints are deterministic (full vs partial align)
- [ ] Request-body reads are bounded
- [ ] `version` and `doctor` CLI commands implemented
- [ ] Default behavior (no args) starts server
- [ ] Port conflict detection with exit code 10
- [ ] OpenAPI spec generated and served at `/openapi.yaml`
- [ ] OpenAPI coverage test passes (all routes documented)

## Quick Reference: Endpoints and Patterns

| Testing Scenario | Fixture  | Endpoint                        | Pattern Notes     |
| ---------------- | -------- | ------------------------------- | ----------------- |
| Connect timeout  | rampart  | `/timeout`                      | Never responds    |
| Header timeout   | rampart  | `/delay/{ms}/headers`           | Delayed headers   |
| Body timeout     | rampart  | `/delay/{ms}/body`              | Delayed body      |
| Redirect chain   | rampart  | `/redirect/{n}`                 | N redirects       |
| Redirect loop    | rampart  | `/redirect/loop`                | Infinite loop     |
| Status codes     | both     | `/status/{code}`                | Any HTTP status   |
| Auth failure     | gauntlet | `/api/protected`                | 401 response      |
| Forbidden        | gauntlet | `/api/admin/config`             | 403 response      |
| Streaming        | gauntlet | `/stream/{n}`                   | Chunked encoding  |
| Slow drip        | gauntlet | `/drip?duration={ms}&bytes={n}` | Timed byte stream |

## Related Documentation

- [HTTP Client Patterns](http-client-patterns.md) - Client-side testing patterns
- [HTTP REST Standard](../../standards/protocol/http-rest-standards.md) - Normative HTTP requirements
- [Go Coding Standards](../../standards/coding/go.md) - Go-specific patterns
- [Fixture Standard](../../architecture/fulmen-fixture-standard.md) - Fixture repository requirements
- [Language Testing Patterns](../../standards/testing/language-testing-patterns.md) - CLI testing patterns
- [Observability Logging](../../standards/observability/logging.md) - Structured logging requirements

---

**Review Required By**: rampart devrev, gauntlet devrev (four-eyes for server-side patterns)
