# Signal Handling Module Standard

**Version:** v1.0.0
**Status:** Stable
**Layer:** Foundry + Module (Hybrid)
**Adoption:** Required for Workhorses, Optional for CLIs

## Overview

The Signal Handling module provides standardized, cross-platform signal handling for the Fulmen ecosystem. It uses a hybrid architecture with a Foundry-layer catalog (SSOT for signal semantics) and Module-layer implementations in helper libraries (reusable code).

### Purpose

- **Consistency**: Same signal behavior across Go, Python, TypeScript, Rust, C#
- **OS Abstraction**: Unix signals map transparently to Windows console events
- **Testability**: Signal injection APIs for unit/integration tests without OS signals
- **Observability**: Structured logging and telemetry for signal events
- **Container-Ready**: SIGTERM graceful shutdown and HTTP `/admin/signal` endpoint

### Hybrid Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Foundry Layer (Crucible SSOT)                               │
├─────────────────────────────────────────────────────────────┤
│ • config/library/foundry/signals.yaml                       │
│   - 8 standard signals (TERM, INT, HUP, QUIT, etc.)         │
│   - OS mappings (Unix → Windows)                            │
│   - Behavior definitions (graceful_shutdown, etc.)          │
│   - Exit code mappings (128+N pattern)                      │
│ • schemas/library/foundry/v1.0.0/signals.schema.json        │
├─────────────────────────────────────────────────────────────┤
│ Module Layer (Helper Libraries)                             │
├─────────────────────────────────────────────────────────────┤
│ • gofulmen/pkg/signals                                      │
│ • pyfulmen/signals                                          │
│ • tsfulmen/signals                                          │
│   - Handler registration APIs                               │
│   - Cleanup function chains                                 │
│   - Context/cancellation integration                        │
│   - Testing utilities (signal injection)                    │
└─────────────────────────────────────────────────────────────┘
```

### Dependency Flow

**CRITICAL**: Signal handling is a low-level module that **MUST NOT** depend on higher-level modules:

```
signals → config → bootstrap → app
```

- ✅ **Correct**: Config module imports signals for graceful reload
- ❌ **Wrong**: Signals module imports config (creates cycle)
- ✅ **Pattern**: Applications provide callbacks; signals calls them

This is particularly critical in Go where import cycles cause compilation errors.

## Foundry Component: Signal Catalog

### Catalog Structure

**File**: `config/library/foundry/signals.yaml`
**Schema**: `schemas/library/foundry/v1.0.0/signals.schema.json`

The catalog defines 8 standard signals with complete metadata:

| Signal  | Unix # | Windows Event    | Behavior                          | Exit Code |
| ------- | ------ | ---------------- | --------------------------------- | --------- |
| SIGTERM | 15     | CTRL_CLOSE_EVENT | graceful_shutdown                 | 143       |
| SIGINT  | 2      | CTRL_C_EVENT     | graceful_shutdown_with_double_tap | 130       |
| SIGHUP  | 1      | null             | reload_via_restart                | 129       |
| SIGQUIT | 3      | CTRL_BREAK_EVENT | immediate_exit                    | 131       |
| SIGPIPE | 13     | null             | observe_only                      | 141       |
| SIGALRM | 14     | null             | custom                            | 142       |
| SIGUSR1 | 10/30  | null             | custom                            | 138/158   |
| SIGUSR2 | 12/31  | null             | custom                            | 140/159   |

**Note**: SIGUSR1/SIGUSR2 have platform-specific signal numbers:

- Linux: 10, 12 (exit codes 138, 140)
- macOS/FreeBSD: 30, 31 (exit codes 158, 159)

### Platform Mapping Strategy

Helper libraries **MUST** handle platform differences transparently:

1. **Unix (Linux, macOS, FreeBSD)**: Native signal support
2. **Windows**: Map to console events where possible; use HTTP fallback for unsupported signals

**Windows Fallback Behavior**:

Helper libraries **MUST** implement standardized logging and telemetry when unsupported signals are registered on Windows:

**Standardized Logging Contract**:

- **Log Level**: `INFO` (not WARN - this is expected behavior on Windows, not a warning)
- **Format**: Structured JSON with consistent fields across all languages
- **Template**: `signal=${signal} platform=${platform} fallback=${fallback_behavior} message='${operation_hint}'`
- **Example**: `signal=SIGHUP platform=windows fallback=http_admin_endpoint message='POST /admin/signal with signal=HUP'`

**Standardized Telemetry Contract**:

- **Event Name**: `fulmen.signal.unsupported` (fixed constant, all languages)
- **Required Tags**:
  - `signal`: Signal name (e.g., "SIGHUP")
  - `platform`: Operating system (e.g., "windows")
  - `fallback_behavior`: Fallback type from catalog (e.g., "http_admin_endpoint", "exception_handling", "timer_api")

**Reference Implementation** (Python):

```python
def handle_signal_registration(signal_name, handler):
    if platform.system() == "Windows" and signal_name in UNSUPPORTED_SIGNALS:
        fallback = get_fallback_from_catalog(signal_name)

        # Structured logging at INFO level
        logger.info(
            "Signal unavailable on Windows",
            extra={
                "signal": signal_name,
                "platform": "windows",
                "fallback": fallback["fallback_behavior"],
                "operation_hint": fallback["operation_hint"]
            }
        )

        # Standardized telemetry emission
        telemetry.emit("fulmen.signal.unsupported", {
            "signal": signal_name,
            "platform": "windows",
            "fallback_behavior": fallback["fallback_behavior"]
        })

        return  # Return gracefully, do not raise error
```

**Required Behavior**:

- Return gracefully (no error/exception raised)
- Do not register a no-op handler (would be misleading)
- Applications can use HTTP `/admin/signal` endpoint as operational fallback

### Exit Code Integration

Signal-induced exits **MUST** use the 128+N pattern per POSIX:

- Exit code = 128 + signal number
- Cross-referenced in `config/library/foundry/exit-codes.yaml`
- See Signal Catalog for mappings

## Module API Requirements

### Handler Registration

Helper libraries **MUST** provide clean registration APIs:

**Go Example** (`gofulmen/pkg/signals`):

```go
signals.Handle(syscall.SIGTERM, func(sig os.Signal) {
    log.Info("Received SIGTERM, shutting down gracefully")
    cleanup()
    os.Exit(143)
})
```

**Python Example** (`pyfulmen/signals`):

```python
from pyfulmen import signals

signals.handle(signal.SIGTERM, lambda sig: (
    logger.info("Received SIGTERM, shutting down gracefully"),
    cleanup(),
    sys.exit(143)
))
```

**TypeScript Example** (`tsfulmen/signals`):

```typescript
import { signals } from "tsfulmen";

signals.handle("SIGTERM", (sig) => {
  logger.info("Received SIGTERM, shutting down gracefully");
  cleanup();
  process.exit(143);
});
```

### Platform Introspection

Helper libraries **MUST** expose a `supports_signal()` API for runtime platform detection:

**Purpose**:

- Enable applications to check signal support before registration
- Allow conditional logic based on platform capabilities
- Support cross-platform testing with deterministic behavior
- Enable dynamic capability reporting in documentation/help text

**Go Example** (`gofulmen/pkg/signals`):

```go
if signals.SupportsSignal(syscall.SIGHUP) {
    signals.Handle(syscall.SIGHUP, reloadConfigHandler)
    log.Info("Config reload: send SIGHUP to reload")
} else {
    // Windows: HTTP endpoint fallback
    log.Info("Config reload: POST /admin/signal with signal=HUP")
}
```

**Python Example** (`pyfulmen/signals`):

```python
from pyfulmen import signals
import signal

if signals.supports_signal(signal.SIGHUP):
    signals.handle(signal.SIGHUP, reload_config_handler)
    print("Config reload: kill -HUP <pid>")
else:
    # Windows: HTTP endpoint fallback
    print("Config reload: POST /admin/signal with signal=HUP")
```

**TypeScript Example** (`tsfulmen/signals`):

```typescript
import { signals } from "tsfulmen";

if (signals.supportsSignal("SIGHUP")) {
  signals.handle("SIGHUP", reloadConfigHandler);
  console.log("Config reload: kill -HUP <pid>");
} else {
  // Windows: HTTP endpoint fallback
  console.log("Config reload: POST /admin/signal with signal=HUP");
}
```

**API Contract**:

- **Function Signature**:
  - Go: `SupportsSignal(sig os.Signal) bool`
  - Python: `supports_signal(sig: signal.Signals) -> bool`
  - TypeScript: `supportsSignal(sig: string) -> boolean`
- **Return Value**:
  - `true` / `True` if signal is natively supported on current platform
  - `false` / `False` if signal uses Windows fallback mechanism
- **Implementation Note**: Check against platform support matrix in `signals.yaml`

### Cleanup Function Chains

Support ordered cleanup handler registration:

```go
signals.OnShutdown(func() {
    closeConnections()
})
signals.OnShutdown(func() {
    flushBuffers()
})
signals.OnShutdown(func() {
    removePidFile()
})
// Executed in reverse order: removePidFile, flushBuffers, closeConnections
```

### Context/Cancellation Integration

Integrate with language-specific cancellation patterns:

**Go (context.Context)**:

```go
ctx, cancel := context.WithCancel(context.Background())
signals.Handle(syscall.SIGTERM, func(sig os.Signal) {
    cancel() // Cancel all operations using this context
})
```

**Python (asyncio)**:

```python
loop = asyncio.get_event_loop()
signals.handle(signal.SIGTERM, lambda sig: loop.stop())
```

**TypeScript (AbortController)**:

```typescript
const controller = new AbortController();
signals.handle("SIGTERM", () => controller.abort());
```

### Testing Utilities

Provide signal injection for tests:

**Go**:

```go
signals.InjectForTest(syscall.SIGTERM) // Trigger handlers without OS signal
```

**Python**:

```python
signals.inject_for_test(signal.SIGTERM)
```

**TypeScript**:

```typescript
signals.injectForTest("SIGTERM");
```

**Test Mode Behavior**:

- Signal injection triggers handlers but **DOES NOT call `os.Exit`**
- Handlers receive signal normally; tests verify behavior
- Production mode: handlers exit normally

## Ctrl+C Double-Tap Pattern

### Specification

**Requirement**: SIGINT (Ctrl+C) **MUST** implement graceful shutdown with force-quit option.

**Behavior**:

1. **First Ctrl+C**: Start graceful shutdown, print hint message
2. **Second Ctrl+C** (within 2 seconds): Force immediate exit
3. Exit code: 130 for both graceful and forced modes

### Implementation Requirements

```python
first_sigint_time = None

def handle_sigint(sig):
    global first_sigint_time
    now = time.time()

    if first_sigint_time is None:
        # First SIGINT
        first_sigint_time = now
        print("Press Ctrl+C again within 2s to force quit")
        graceful_shutdown()
    elif (now - first_sigint_time) < 2.0:
        # Second SIGINT within window
        print("Force quitting...")
        sys.exit(130)
    else:
        # SIGINT after window expired - treat as first
        first_sigint_time = now
        print("Press Ctrl+C again within 2s to force quit")
        graceful_shutdown()
```

### Testing Requirements

- Verify first Ctrl+C starts graceful shutdown
- Verify second Ctrl+C (within 2s) forces immediate exit
- Verify Ctrl+C after window (>2s) resets and starts new graceful shutdown

## Config Reload Validation

### Restart-Based Reload Mandate

**Requirement**: Configuration reload via SIGHUP **MUST** use restart-based pattern with mandatory schema validation.

**Rationale**: In-process hot reload bypasses schema validation and risks configuration corruption. Restart-based reload ensures:

- New config validated against Crucible schema before acceptance
- Process supervisor (systemd, K8s) handles restart
- Invalid config logged and rejected without disruption

### Implementation Pattern

```python
def handle_sighup(sig):
    logger.info("Received SIGHUP, validating new config")

    try:
        new_config = load_config("config.yaml")
        validate_against_schema(new_config, "app-config.schema.json")
    except ValidationError as e:
        logger.error(f"Config validation failed: {e}")
        logger.error("Continuing with current config (no restart)")
        return

    # Valid config - proceed with restart
    logger.info("Config valid, exiting for restart")
    graceful_shutdown()
    sys.exit(129)  # 128 + 1
    # Process supervisor restarts with new config
```

### Three-Strikes Failure Policy

**Optional Enhancement**: Track reload failures and take action:

```python
reload_failures = 0

def handle_sighup(sig):
    global reload_failures

    if validate_new_config():
        reload_failures = 0  # Reset on success
        sys.exit(129)
    else:
        reload_failures += 1
        logger.error(f"Reload failure #{reload_failures}")

        if reload_failures >= 3:
            logger.critical("3 consecutive reload failures - check config source")
            emit_alert("config_reload_failures")
```

### Testing Requirements

- Verify valid config triggers restart (exit 129)
- Verify invalid config logged and rejected (no restart)
- Verify process continues with current config after rejection

## HTTP Signal Endpoint

### Request/Response Contract

**Endpoint**: `POST /admin/signal`

**Request**:

```json
{
  "signal": "HUP",
  "reason": "config reload requested",
  "correlation_id": "req-123"
}
```

**Response (202 Accepted)**:

```json
{
  "status": "accepted",
  "signal": "HUP",
  "correlation_id": "req-123",
  "message": "Signal will be processed asynchronously"
}
```

**Response (400 Bad Request)**:

```json
{
  "status": "error",
  "error": "invalid_signal",
  "message": "Signal 'FOO' is not recognized. Valid signals: HUP, TERM, INT, QUIT, USR1, USR2"
}
```

### Authentication Requirements

**Requirement**: HTTP signal endpoint **MUST** require authentication.

**Options** (in order of preference):

1. mTLS client certificates
2. Bearer token (API key)
3. Basic auth (acceptable for internal networks)

**Rationale**: Signal injection can terminate processes; authentication prevents abuse.

### Rate Limiting Defaults

- **Default**: 10 requests/minute per client
- **Configurable**: via `admin.signal_rate_limit` config
- **Enforcement**: 429 Too Many Requests response

### Admin Surface Integration

Signal endpoint **MUST** be part of the admin HTTP surface:

- Separate listener or TLS-only route
- Not exposed on public application port
- Cross-reference: Admin Surface Standard (when available)

### Minimum Spec for Helper Library Implementation

**Status**: Interim standard until full Admin Surface Standard published

Helper libraries **MUST** provide signal endpoint utilities meeting these minimum requirements:

**Request Contract**:

- Method: `POST`
- Path: `/admin/signal`
- Content-Type: `application/json`
- Body: `{"signal": "HUP"|"TERM"|"INT"|"QUIT"|"USR1"|"USR2", "reason": string (optional), "correlation_id": string (optional)}`

**Response Contract**:

- Success: `202 Accepted` with `{"status": "accepted", "signal": string, "correlation_id": string, "message": string}`
- Invalid signal: `400 Bad Request` with `{"status": "error", "error": "invalid_signal", "message": string}`
- Authentication failure: `401 Unauthorized` or `403 Forbidden`
- Rate limit exceeded: `429 Too Many Requests`

**Authentication** (Required):

- **Preferred**: mTLS client certificates
- **Alternative**: Bearer token (API key) or Basic auth (internal networks only)
- **Configurable**: Via `admin.signal_endpoint_auth_method` config
- **Default**: mTLS if available, otherwise require Bearer token

**Rate Limiting**:

- **Default**: 10 requests/minute per client (identified by IP or certificate fingerprint)
- **Configurable**: Via `admin.signal_rate_limit` config
- **Enforcement**: Return 429 with `Retry-After` header

**Logging**: All signal endpoint requests **MUST** be logged with correlation ID, source IP, authenticated identity, and signal name

## OS Abstraction

### Platform Support Matrix

| Platform | Support Level | Notes                                  |
| -------- | ------------- | -------------------------------------- |
| Linux    | Full          | All 8 signals natively supported       |
| macOS    | Full          | USR1/USR2 use different signal numbers |
| FreeBSD  | Full          | USR1/USR2 use different signal numbers |
| Windows  | Partial       | TERM/INT/QUIT mapped; others via HTTP  |

### Graceful Degradation Patterns

**Windows Limitations**:

- SIGHUP, SIGPIPE, SIGALRM, SIGUSR1, SIGUSR2 not available
- Fallback: Log warning + emit telemetry + return gracefully
- Applications use HTTP `/admin/signal` as operational alternative

**Example**:

```python
def handle_sigusr1(sig):
    if platform.system() == "Windows":
        logger.warning("SIGUSR1 not available on Windows; use HTTP POST /admin/signal")
        telemetry.emit("fulmen.signal.unsupported", {"signal": "SIGUSR1", "platform": "windows"})
        return
    # Unix implementation
    reopen_log_files()
```

### Testing Requirements Per Platform

- **Unix**: Test all 8 signals
- **Windows**: Test TERM/INT/QUIT mapping; verify others log warnings
- **macOS/FreeBSD**: Test USR1/USR2 platform overrides

## Windows Testing Strategy

**Requirement**: Helper libraries **MUST** implement Windows smoke tests for signal fallback behavior.

### Required Test Coverage

**1. Unsupported Signal Registration Test**:

```python
def test_windows_unsupported_signal_registration():
    """Verify registration logs and emits telemetry correctly"""
    if platform.system() != "Windows":
        pytest.skip("Windows-specific test")

    # Capture logs and telemetry
    with capture_logs() as logs, capture_telemetry() as telemetry:
        signals.handle(signal.SIGHUP, lambda sig: None)

    # Assert structured logging
    assert len(logs) == 1
    assert logs[0]["level"] == "INFO"
    assert logs[0]["signal"] == "SIGHUP"
    assert logs[0]["platform"] == "windows"
    assert logs[0]["fallback"] == "http_admin_endpoint"
    assert "POST /admin/signal" in logs[0]["operation_hint"]

    # Assert standardized telemetry
    assert len(telemetry) == 1
    assert telemetry[0]["event"] == "fulmen.signal.unsupported"
    assert telemetry[0]["tags"]["signal"] == "SIGHUP"
    assert telemetry[0]["tags"]["platform"] == "windows"
    assert telemetry[0]["tags"]["fallback_behavior"] == "http_admin_endpoint"
```

**2. HTTP Endpoint Functional Test**:

```python
def test_http_signal_endpoint():
    """Verify /admin/signal endpoint processes signals correctly"""
    if platform.system() != "Windows":
        pytest.skip("Windows-specific test")

    # Start test server with signal endpoint
    server = start_test_server_with_signal_endpoint()

    # Send signal via HTTP
    response = requests.post(
        "http://localhost:8080/admin/signal",
        json={"signal": "HUP", "reason": "test reload"},
        headers={"Authorization": "Bearer test-token"}
    )

    assert response.status_code == 202
    assert response.json()["status"] == "accepted"
    assert response.json()["signal"] == "HUP"
```

**3. Platform Detection Test**:

```python
def test_platform_detection():
    """Verify correct behavior based on platform"""
    # Unix: handler registered normally
    # Windows: fallback behavior triggered
    handler_called = False

    def test_handler(sig):
        nonlocal handler_called
        handler_called = True

    signals.handle(signal.SIGUSR1, test_handler)

    if platform.system() == "Windows":
        # Handler not registered, fallback logged
        signals.inject_for_test(signal.SIGUSR1)
        assert not handler_called
    else:
        # Handler registered and callable
        signals.inject_for_test(signal.SIGUSR1)
        assert handler_called
```

### CI/CD Guidance

**Windows CI Runners**: Not mandatory, but recommended for comprehensive testing

**Minimum Testing Requirements**:

- **All releases**: Manual verification on Windows (one-time smoke test per release)
- **Test Coverage**: Automated tests for all three scenarios above
- **Documentation**: Clear operational guidance for Windows users

**Acceptable Testing Strategy**:

- Develop and test on Unix/Linux (primary platform)
- Run automated tests on Unix with platform mocks for Windows paths
- Manually verify Windows behavior on each major/minor release
- Document Windows-specific operational procedures in helper library README

### Operational Guidance for Helper Library READMEs

Helper library documentation **MUST** include this operational story:

**Unix/Linux (Preferred)**:

```bash
# Send SIGHUP for config reload
kill -HUP <pid>

# Send SIGTERM for graceful shutdown
kill -TERM <pid>

# Send SIGUSR1 for custom operation
kill -USR1 <pid>
```

**Windows (HTTP Fallback)**:

```powershell
# Config reload via HTTP endpoint
Invoke-WebRequest -Method POST -Uri "http://localhost:8080/admin/signal" `
  -Headers @{"Authorization"="Bearer ${env:ADMIN_TOKEN}"} `
  -Body (@{"signal"="HUP";"reason"="config reload"} | ConvertTo-Json) `
  -ContentType "application/json"

# Graceful shutdown (native)
Stop-Process -Id <pid>  # Sends CTRL_CLOSE_EVENT (SIGTERM equivalent)

# Interrupt (native)
# Ctrl+C in console (sends CTRL_C_EVENT, SIGINT equivalent)
```

**Cross-Platform Automation**:

```python
# Application code detects platform and uses appropriate method
from pyfulmen import signals

if sys.platform == "win32":
    # Windows: Use HTTP endpoint
    send_signal_via_http("HUP")
else:
    # Unix: Use OS signal
    os.kill(pid, signal.SIGHUP)
```

## Cross-References

- **Signal Catalog**: `config/library/foundry/signals.yaml`
- **Signal Schema**: `schemas/library/foundry/v1.0.0/signals.schema.json`
- **Exit Codes Catalog**: `config/library/foundry/exit-codes.yaml`
- **Workhorse Standard**: `docs/architecture/fulmen-forge-workhorse-standard.md`
- **Fixtures**: `config/library/foundry/fixtures/signals/`

## Helper Library Responsibilities

Each language helper library **MUST** implement:

1. **Load signals.yaml**: Expose canonical `signals` module
2. **Register handlers**: Implement graceful shutdown, Ctrl+C double-tap, config reload
3. **Platform abstractions**: Handle Unix vs Windows differences transparently
4. **Test facilities**: Provide `ForTesting` / `inject_signal_for_test` helpers
5. **Documentation**: Update README with signal handling usage examples

Only after helper libraries complete these steps should downstream workhorses (groningen, percheron, etc.) begin adoption.

---

**Standard Version**: 1.0.0
**Effective Date**: Upon Crucible v0.2.5 release
**Review Cycle**: Quarterly or as needed for platform expansion
