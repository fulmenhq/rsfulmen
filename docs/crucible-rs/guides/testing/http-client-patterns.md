---
title: "HTTP Client Testing Patterns"
description: "Compliance routing and practical patterns for HTTP client library and middleware testing"
author: "entarch (AI)"
date: "2026-01-09"
last_updated: "2026-01-09"
status: "active"
tags: ["guides", "testing", "http", "client", "v0.4.5"]
---

# HTTP Client Testing Patterns

This guide helps developers testing HTTP clients, middleware, and proxies discover applicable standards and implement robust test suites. It serves as a **compliance routing document** - directing you to normative standards while providing practical examples using rampart and gauntlet fixtures.

## Compliance Requirements

Before implementing HTTP client tests, ensure compliance with these standards:

| Standard                                                                            | Scope                      | Key Requirements                                         |
| ----------------------------------------------------------------------------------- | -------------------------- | -------------------------------------------------------- |
| [HTTP REST Standard](../../standards/protocol/http-rest-standards.md)               | All HTTP clients           | Expected response formats, status code handling, headers |
| [Go Coding Standards](../../standards/coding/go.md)                                 | Go implementations         | Error handling, context propagation                      |
| [Python Coding Standards](../../standards/coding/python.md)                         | Python implementations     | Async patterns, timeout handling                         |
| [TypeScript Coding Standards](../../standards/coding/typescript.md)                 | TypeScript implementations | Promise handling, AbortController                        |
| [Portable Testing Practices](../../standards/testing/portable-testing-practices.md) | All languages              | Deterministic tests, CI-friendly patterns                |

### Pre-Implementation Checklist

- [ ] Read HTTP REST Standard sections on status codes and headers
- [ ] Read language-specific coding standard for error handling patterns
- [ ] Identify which fixtures cover your test scenarios (see Quick Reference below)
- [ ] Review timeout tier patterns if testing timeouts
- [ ] Plan correlation ID propagation for log-based assertions

### Pre-Review Checklist (for devrev)

- [ ] Tests use fixtures rather than mocks for integration scenarios
- [ ] Timeout tests distinguish connect vs header vs body timeouts
- [ ] Redirect tests validate max hop limits and loop detection
- [ ] Retry logic follows idempotency rules
- [ ] Correlation IDs used for log-based assertions
- [ ] Tests are deterministic and CI-friendly

## Timeout Testing

### The Three Timeout Tiers

Most HTTP clients conflate different timeout types. Testing each tier separately reveals edge cases:

| Tier                | What It Measures          | Failure Mode                      |
| ------------------- | ------------------------- | --------------------------------- |
| **Connect timeout** | TCP handshake completion  | Server unreachable, firewall drop |
| **Header timeout**  | Time to first byte (TTFB) | Slow backend processing           |
| **Body timeout**    | Full response completion  | Large payloads, slow streaming    |

### Testing with Fixtures

| Scenario                   | Fixture  | Endpoint                        | Behavior                                               |
| -------------------------- | -------- | ------------------------------- | ------------------------------------------------------ |
| Header/read timeout (TTFB) | rampart  | `/timeout`                      | TCP accepts, but never sends data (tests read timeout) |
| Header timeout             | rampart  | `/delay/{ms}/headers`           | Delays before sending headers                          |
| Body timeout               | rampart  | `/delay/{ms}/body`              | Sends headers immediately, delays body                 |
| Combined                   | gauntlet | `/drip?duration={ms}&bytes={n}` | Slow body streaming                                    |

**Note on connect timeout testing**: True connect timeouts require the TCP handshake to never complete (unreachable host, firewall DROP, or blackhole). Fixture endpoints cannot simulate this because the connection succeeds. For connect timeout testing:

- Use an unroutable IP: `203.0.113.1:81` (TEST-NET-3, guaranteed unroutable)
- Use Docker network policies or iptables DROP rules (environment-dependent)
- Use `localhost:1` (port 1 is typically closed, gives connection refused - not quite a timeout but fast failure)

The `/timeout` endpoint tests **read/header timeout** (connection succeeds, no response), which is the more common client timeout failure mode.

### Go Example: Testing Header vs Body Timeout

```go
func TestClient_TimeoutTiers(t *testing.T) {
    tests := []struct {
        name           string
        endpoint       string
        headerTimeout  time.Duration
        bodyTimeout    time.Duration
        expectError    bool
        errorContains  string
    }{
        {
            name:          "header_timeout_triggers",
            endpoint:      "/delay/500/headers",  // 500ms header delay
            headerTimeout: 100 * time.Millisecond,
            bodyTimeout:   5 * time.Second,
            expectError:   true,
            errorContains: "timeout",
        },
        {
            name:          "body_timeout_triggers",
            endpoint:      "/delay/500/body",  // 500ms body delay
            headerTimeout: 5 * time.Second,
            bodyTimeout:   100 * time.Millisecond,
            expectError:   true,
            errorContains: "timeout",
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            client := &http.Client{
                Timeout: tt.headerTimeout + tt.bodyTimeout,
            }

            ctx, cancel := context.WithTimeout(context.Background(), tt.headerTimeout)
            defer cancel()

            req, _ := http.NewRequestWithContext(ctx, "GET", fixtureURL+tt.endpoint, nil)
            resp, err := client.Do(req)

            if tt.expectError {
                require.Error(t, err)
                assert.Contains(t, err.Error(), tt.errorContains)
            } else {
                require.NoError(t, err)
                defer resp.Body.Close()
            }
        })
    }
}
```

### Python Example: Timeout with httpx

```python
import httpx
import pytest

@pytest.mark.parametrize("endpoint,timeout,should_fail", [
    ("/delay/500/headers", 0.1, True),   # Header delay > timeout
    ("/delay/100/headers", 1.0, False),  # Header delay < timeout
])
async def test_header_timeout(fixture_url, endpoint, timeout, should_fail):
    async with httpx.AsyncClient(timeout=httpx.Timeout(timeout)) as client:
        if should_fail:
            with pytest.raises(httpx.TimeoutException):
                await client.get(f"{fixture_url}{endpoint}")
        else:
            resp = await client.get(f"{fixture_url}{endpoint}")
            assert resp.status_code == 200
```

### Common Pitfalls

- **Confusing connect vs read timeout**: Many clients have separate settings
- **Not testing slow body scenarios**: Headers arrive, body doesn't
- **Ignoring Keep-Alive timeout**: Connection reuse can mask issues

## Redirect Handling

### Redirect Types and Semantics

| Status | Meaning   | Method Change?      | Body Forwarded? |
| ------ | --------- | ------------------- | --------------- |
| 301    | Permanent | GET only            | No              |
| 302    | Found     | GET only (browsers) | No              |
| 303    | See Other | Always GET          | No              |
| 307    | Temporary | Never               | Yes             |
| 308    | Permanent | Never               | Yes             |

### Testing with Fixtures

| Scenario           | Fixture | Endpoint                 | Behavior               |
| ------------------ | ------- | ------------------------ | ---------------------- |
| Redirect chain     | rampart | `/redirect/{n}`          | N redirects before 200 |
| Absolute redirects | rampart | `/redirect/{n}/absolute` | Absolute URL redirects |
| Relative redirects | rampart | `/redirect/{n}/relative` | Relative URL redirects |
| Infinite loop      | rampart | `/redirect/loop`         | Redirects to self      |

### TypeScript Example: Redirect Handling

```typescript
import { describe, it, expect } from "vitest";

describe("redirect handling", () => {
  it("follows redirect chain up to limit", async () => {
    const response = await fetch(`${FIXTURE_URL}/redirect/3`, {
      redirect: "follow",
    });

    expect(response.ok).toBe(true);
    expect(response.redirected).toBe(true);
  });

  it("detects redirect loop", async () => {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 5000);

    try {
      await fetch(`${FIXTURE_URL}/redirect/loop`, {
        redirect: "follow",
        signal: controller.signal,
      });
      expect.fail("Should have thrown on redirect loop");
    } catch (error) {
      // Expected: either max redirects or abort
      expect(error).toBeDefined();
    } finally {
      clearTimeout(timeout);
    }
  });
});
```

### Common Pitfalls

- **No max redirect limit**: Infinite loops hang forever
- **Method change on 302/303**: POST becomes GET
- **Cookie/header propagation**: Security implications across origins

## Retry and Backoff

### When to Retry

| Condition                           | Retry? | Notes                        |
| ----------------------------------- | ------ | ---------------------------- |
| 429 Too Many Requests               | Yes    | Respect `Retry-After` header |
| 503 Service Unavailable             | Yes    | Temporary overload           |
| 5xx with idempotent method          | Yes    | GET, HEAD, PUT, DELETE       |
| Network error (connect timeout)     | Yes    | Transient failure            |
| 4xx (except 429)                    | No     | Client error, won't change   |
| Non-idempotent without confirmation | No     | May cause duplicates         |

### Testing with Fixtures

```go
func TestRetryBehavior(t *testing.T) {
    // Test that client respects Retry-After if fixture provides it.
    // NOTE: Not all /status/{code} endpoints synthesize Retry-After headers.
    // Check your fixture's documentation or add a dedicated endpoint.
    t.Run("respects_retry_after", func(t *testing.T) {
        // If your fixture provides Retry-After on 429:
        start := time.Now()

        resp, err := clientWithRetry.Get(fixtureURL + "/status/429")
        require.NoError(t, err)
        defer resp.Body.Close()

        // Only assert timing if Retry-After was provided
        if retryAfter := resp.Header.Get("Retry-After"); retryAfter != "" {
            elapsed := time.Since(start)
            // Parse retryAfter and verify elapsed >= parsed value
            assert.GreaterOrEqual(t, elapsed, 1*time.Second, "should have waited for Retry-After")
        }
    })
}
```

### Exponential Backoff Pattern

```go
func retryWithBackoff(ctx context.Context, fn func() error) error {
    backoff := 100 * time.Millisecond
    maxBackoff := 10 * time.Second
    maxRetries := 5

    for attempt := 0; attempt < maxRetries; attempt++ {
        err := fn()
        if err == nil {
            return nil
        }

        if !isRetryable(err) {
            return err
        }

        select {
        case <-ctx.Done():
            return ctx.Err()
        case <-time.After(backoff):
            backoff = min(backoff*2, maxBackoff)
        }
    }

    return fmt.Errorf("max retries exceeded")
}
```

## Observability and Assertions

### Correlation ID Round-Trip

Use correlation IDs to connect client requests to fixture logs:

```python
import uuid

def test_correlation_id_propagation(fixture_url):
    correlation_id = str(uuid.uuid4())

    response = requests.get(
        f"{fixture_url}/echo",
        headers={"X-Correlation-ID": correlation_id}
    )

    # Verify round-trip
    assert response.headers.get("X-Correlation-ID") == correlation_id

    # Can now grep fixture logs for this correlation_id
    # docker logs gauntlet 2>&1 | jq 'select(.correlation_id == "...")'
```

### Log-Based Assertions

```bash
# Filter fixture logs by correlation ID
docker logs gauntlet 2>&1 | jq 'select(.correlation_id == "test-123")'

# Verify request was logged with expected fields
docker logs gauntlet 2>&1 | jq 'select(.correlation_id == "test-123") | {method, path, status}'
```

## Auth Failure Handling

### 401 vs 403 Semantics

| Status           | Meaning              | Client Action                       |
| ---------------- | -------------------- | ----------------------------------- |
| 401 Unauthorized | "Who are you?"       | Authenticate (login, refresh token) |
| 403 Forbidden    | "I know you, but no" | Don't retry, permission denied      |

### WWW-Authenticate Header Parsing

```go
func TestAuthFailures(t *testing.T) {
    t.Run("401_includes_www_authenticate", func(t *testing.T) {
        resp, err := client.Get(fixtureURL + "/api/protected")
        require.NoError(t, err)
        defer resp.Body.Close()

        assert.Equal(t, 401, resp.StatusCode)

        wwwAuth := resp.Header.Get("WWW-Authenticate")
        assert.Contains(t, wwwAuth, "Bearer")
        // Parse realm, error, error_description from header
    })

    t.Run("403_no_retry", func(t *testing.T) {
        resp, err := authenticatedClient.Get(fixtureURL + "/api/admin/config")
        require.NoError(t, err)
        defer resp.Body.Close()

        assert.Equal(t, 403, resp.StatusCode)
        // Client should NOT retry 403
    })
}
```

## Streaming and Chunked Responses

### Testing Patterns

| Scenario       | Fixture  | Endpoint                        | Notes            |
| -------------- | -------- | ------------------------------- | ---------------- |
| Chunked stream | gauntlet | `/stream/{n}`                   | N chunks         |
| Slow drip      | gauntlet | `/drip?duration={ms}&bytes={n}` | Timed delivery   |
| Large payload  | gauntlet | `/bytes/{n}`                    | Exact byte count |

### Rust Example: Streaming with reqwest

```rust
use futures_util::StreamExt;

#[tokio::test]
async fn test_streaming_response() {
    let client = reqwest::Client::new();
    let mut stream = client
        .get(format!("{}/stream/10", FIXTURE_URL))
        .send()
        .await
        .unwrap()
        .bytes_stream();

    let mut total_bytes = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        total_bytes += chunk.len();
    }

    assert!(total_bytes > 0);
}
```

## CI/CD Integration

### Health Check Before Tests

```yaml
# Wait for fixture readiness
- name: Wait for fixture
  run: |
    until curl -sf http://localhost:8080/health/ready; do
      sleep 1
    done
```

### Parallel Test Isolation

- Use unique correlation IDs per test for log filtering
- Consider separate fixture instances per test suite
- Use dynamic port allocation to avoid conflicts:

```go
listener, _ := net.Listen("tcp", "127.0.0.1:0")
port := listener.Addr().(*net.TCPAddr).Port
```

## Quick Reference: Test Scenarios

| Testing Scenario    | Fixture  | Endpoint              | Key Assertion                         |
| ------------------- | -------- | --------------------- | ------------------------------------- |
| Read/header timeout | rampart  | `/timeout`            | Error after connect, no data received |
| Header timeout      | rampart  | `/delay/{ms}/headers` | Error before body                     |
| Body timeout        | rampart  | `/delay/{ms}/body`    | Headers received, body timeout        |
| Redirect chain      | rampart  | `/redirect/{n}`       | Final status 200                      |
| Redirect loop       | rampart  | `/redirect/loop`      | Error or max redirects                |
| Status codes        | both     | `/status/{code}`      | Correct status handling               |
| Retry-After         | gauntlet | `/status/429`         | Respects header delay                 |
| Auth required       | gauntlet | `/api/protected`      | 401 + WWW-Authenticate                |
| Permission denied   | gauntlet | `/api/admin/config`   | 403, no retry                         |
| Streaming           | gauntlet | `/stream/{n}`         | Complete stream received              |
| Correlation ID      | both     | `/echo`               | ID round-trips                        |

## Related Documentation

- [HTTP Server Patterns](http-server-patterns.md) - Server-side implementation patterns
- [HTTP REST Standard](../../standards/protocol/http-rest-standards.md) - Normative HTTP requirements
- [Portable Testing Practices](../../standards/testing/portable-testing-practices.md) - Cross-language testing
- [Language Testing Patterns](../../standards/testing/language-testing-patterns.md) - CLI testing patterns
- [rampart README](https://github.com/fulmenhq/fixture-server-proving-rampart) - HTTP protocol fixture
- [gauntlet README](https://github.com/fulmenhq/fixture-server-proving-gauntlet) - Protected backend fixture

---

**Review Required By**: rampart devrev, gauntlet devrev (four-eyes for client-side patterns)
