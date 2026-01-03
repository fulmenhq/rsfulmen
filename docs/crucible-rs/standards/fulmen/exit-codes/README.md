---
title: "Exit Codes Application Guide"
description: "Comprehensive guide for application developers using Fulmen standardized exit codes"
audience: "application-developers"
version: "v1.0.0"
last_updated: "2025-11-02"
status: "stable"
tags: ["fulmen", "exit-codes", "application-standards", "signals"]
---

# Exit Codes Application Guide

**Audience**: Application developers using gofulmen, pyfulmen, or tsfulmen helper libraries.

**Purpose**: This guide explains how to use Fulmen's standardized exit codes in your applications for consistent, observable error signaling across the Fulmen ecosystem.

---

## Platform-Specific Behavior

### Windows Signal Exit Codes

> ⚠️ **Important**: POSIX signal exits (128+N) are NOT available on native Windows.

Windows does not surface Unix signal exit codes (SIGTERM=143, SIGINT=130, etc.) because it uses a different process termination model. This affects applications targeting cross-platform deployment:

**Platform Behavior**:

- **Native Windows**: Signal exit codes (128-165) will never occur
- **WSL/Cygwin**: MAY support signal exits but emit telemetry warnings
- **Linux/macOS**: Full signal exit code support

**Recommendations for Cross-Platform Applications**:

#### 1. Use Simplified Mode on Windows

```python
# Python example
import platform
from pyfulmen.foundry import ExitCode, map_to_simplified, SimplifiedMode

exit_code = ExitCode.EXIT_PORT_IN_USE  # 10

if platform.system() == "Windows":
    exit_code = map_to_simplified(exit_code, SimplifiedMode.BASIC)
    # Returns 1 (ERROR) instead of 10

sys.exit(exit_code)
```

```go
// Go example
import (
    "runtime"
    "github.com/fulmenhq/gofulmen/foundry"
)

exitCode := foundry.ExitPortInUse  // 10

if runtime.GOOS == "windows" {
    exitCode = foundry.MapToSimplified(exitCode, foundry.SimplifiedModeBasic)
    // Returns 1 (ERROR) instead of 10
}
os.Exit(exitCode)
```

```typescript
// TypeScript example
import {
  exitCodes,
  mapToSimplified,
  SimplifiedMode,
} from "@fulmenhq/tsfulmen/foundry";

let exitCode = exitCodes.EXIT_PORT_IN_USE; // 10

if (process.platform === "win32") {
  exitCode = mapToSimplified(exitCode, SimplifiedMode.BASIC);
  // Returns 1 (ERROR) instead of 10
}

process.exit(exitCode);
```

#### 2. Check Platform Capabilities

> **Note**: Platform capability probes (`supportsSignalExitCodes()`, `getPlatformInfo()`) will be available in helper library v0.2.0 (post-Crucible v0.2.3). Until then, use runtime platform detection from standard libraries.

**Current approach** (using standard library platform detection):

```go
// Go - Check platform manually
import "runtime"

if runtime.GOOS == "windows" {
    // Use simplified mode or standard codes only on Windows
    logger.Warn("Signal exit codes unavailable on Windows")
    exitCode = foundry.MapToSimplified(exitCode, foundry.SimplifiedModeBasic)
}
```

```python
# Python - Check platform manually
import platform

if platform.system() == "Windows":
    logger.warning("Signal exit codes unavailable on Windows")
    exit_code = map_to_simplified(exit_code, SimplifiedMode.BASIC)
```

```typescript
// TypeScript - Check platform manually
if (process.platform === "win32") {
  logger.warn("Signal exit codes unavailable on Windows");
  exitCode = mapToSimplified(exitCode, SimplifiedMode.BASIC);
}
```

**Future (helper library v0.2.0)**:

```python
# Will be available in pyfulmen v0.2.0
from pyfulmen.foundry import supports_signal_exit_codes, get_platform_info

if not supports_signal_exit_codes():
    logger.warning("Signal exit codes unavailable on this platform")
```

#### 3. Structured Logging for Platform Differences

When your application detects platform limitations, emit structured telemetry:

```python
import platform

if platform.system() == "Windows":
    logger.warning(
        "Platform does not support signal exit codes",
        extra={
            'platform_os': platform.system(),
            'platform_arch': platform.machine(),
            'exit_code_strategy': 'simplified',
            'signal_support': False
        }
    )
```

**Key Takeaways**:

- ✅ Use standard codes (0-99) for cross-platform compatibility
- ✅ Use simplified modes on Windows for cleaner exit semantics
- ✅ Check platform (runtime.GOOS, platform.system(), process.platform) before relying on signal exits
- ✅ Log platform capabilities for observability
- ⚠️ Never assume signal exit codes work on all platforms
- 📅 Platform capability probes coming in helper library v0.2.0

---

## Quick Start

### Go (gofulmen)

```go
import "github.com/fulmenhq/gofulmen/foundry"

func main() {
    if err := validateConfig(); err != nil {
        log.Printf("Configuration validation failed: %v", err)
        os.Exit(foundry.ExitConfigInvalid)
    }

    if err := startServer(); err != nil {
        log.Printf("Server startup failed: %v", err)
        os.Exit(foundry.ExitPortInUse)
    }

    os.Exit(foundry.ExitSuccess)
}
```

### Python (pyfulmen)

```python
from pyfulmen.foundry import ExitCode
import sys

def main():
    if not validate_config():
        print("Configuration validation failed", file=sys.stderr)
        sys.exit(ExitCode.EXIT_CONFIG_INVALID)

    if not start_server():
        print("Server startup failed", file=sys.stderr)
        sys.exit(ExitCode.EXIT_PORT_IN_USE)

    sys.exit(ExitCode.EXIT_SUCCESS)
```

### TypeScript (tsfulmen)

```typescript
import { exitCodes } from "@fulmenhq/tsfulmen/foundry";

async function main() {
  if (!(await validateConfig())) {
    console.error("Configuration validation failed");
    process.exit(exitCodes.EXIT_CONFIG_INVALID);
  }

  if (!(await startServer())) {
    console.error("Server startup failed");
    process.exit(exitCodes.EXIT_PORT_IN_USE);
  }

  process.exit(exitCodes.EXIT_SUCCESS);
}
```

---

## Exit Code Categories

Fulmen provides **54 standardized exit codes** across **11 categories**:

### Standard (0-1)

- `EXIT_SUCCESS` (0) - Successful execution
- `EXIT_FAILURE` (1) - Generic failure

**Use**: Default success/failure. Use specific codes below when applicable.

---

### Networking (10-19)

| Code | Constant                     | When to Use                       |
| ---- | ---------------------------- | --------------------------------- |
| 10   | `EXIT_PORT_IN_USE`           | Server port already bound         |
| 11   | `EXIT_CONNECTION_TIMEOUT`    | Network connection timeout        |
| 12   | `EXIT_CONNECTION_REFUSED`    | Remote service refused connection |
| 13   | `EXIT_DNS_RESOLUTION_FAILED` | Cannot resolve hostname           |
| 14   | `EXIT_NETWORK_UNREACHABLE`   | Network route unavailable         |
| 15   | `EXIT_TLS_HANDSHAKE_FAILED`  | TLS/SSL negotiation failed        |

**Use**: Network-related failures during startup or operation.

**Retry Hint**: Most networking codes suggest `investigate` - check logs, verify config, ensure services are up.

---

### Configuration (20-29)

| Code | Constant                        | When to Use                           |
| ---- | ------------------------------- | ------------------------------------- |
| 20   | `EXIT_CONFIG_INVALID`           | Config file malformed/invalid         |
| 21   | `EXIT_CONFIG_MISSING`           | Required config file not found        |
| 22   | `EXIT_DEPENDENCY_UNAVAILABLE`   | Required external service unavailable |
| 23   | `EXIT_SSOT_MISMATCH`            | Config doesn't match canonical source |
| 24   | `EXIT_SCHEMA_VALIDATION_FAILED` | Data doesn't validate against schema  |

**Use**: Configuration and validation failures during startup.

**Retry Hint**: `no_retry` - fix configuration before restarting.

---

### Runtime (30-39)

| Code | Constant                    | When to Use                             |
| ---- | --------------------------- | --------------------------------------- |
| 30   | `EXIT_HEALTH_CHECK_FAILED`  | Application health check failure        |
| 31   | `EXIT_DATABASE_UNAVAILABLE` | Database connection lost                |
| 32   | `EXIT_CACHE_UNAVAILABLE`    | Cache service unavailable               |
| 33   | `EXIT_RESOURCE_EXHAUSTED`   | Out of memory, disk, or other resources |
| 34   | `EXIT_OPERATION_TIMEOUT`    | Long-running operation timed out        |

**Use**: Runtime failures during operation (not startup).

**Retry Hint**: Mix of `retry` (temporary issues) and `investigate` (persistent problems).

---

### Usage (40-49)

| Code | Constant                     | When to Use                    |
| ---- | ---------------------------- | ------------------------------ |
| 40   | `EXIT_INVALID_ARGUMENTS`     | CLI arguments invalid          |
| 41   | `EXIT_REQUIRED_FLAG_MISSING` | Required CLI flag not provided |
| 64   | `EXIT_USAGE`                 | BSD EX_USAGE compatibility     |

**Use**: User-facing CLI errors.

**Retry Hint**: `no_retry` - user must fix arguments.

**Note**: Exit code 64 is preserved for BSD `sysexits.h` compatibility.

---

### Permissions (50-59)

| Code | Constant                   | When to Use                               |
| ---- | -------------------------- | ----------------------------------------- |
| 50   | `EXIT_PERMISSION_DENIED`   | Insufficient OS permissions               |
| 51   | `EXIT_FILE_NOT_FOUND`      | Required file missing                     |
| 52   | `EXIT_FILE_READ_ERROR`     | Cannot read file (permissions/corruption) |
| 53   | `EXIT_FILE_WRITE_ERROR`    | Cannot write file (permissions/disk full) |
| 54   | `EXIT_DIRECTORY_NOT_FOUND` | Required directory missing                |

**Use**: Filesystem and permission errors.

**Retry Hint**: `no_retry` - fix permissions/paths before restarting.

---

### Data (60-69)

| Code | Constant                          | When to Use                  |
| ---- | --------------------------------- | ---------------------------- |
| 60   | `EXIT_DATA_VALIDATION_FAILED`     | Input data validation failed |
| 61   | `EXIT_DATA_PARSING_FAILED`        | Cannot parse input format    |
| 62   | `EXIT_DATA_TRANSFORMATION_FAILED` | Data transformation error    |
| 63   | `EXIT_DATA_CORRUPTION`            | Data integrity check failed  |

**Use**: Data processing errors.

**Retry Hint**: `no_retry` - fix input data.

---

### Security (70-79)

| Code | Constant                     | When to Use                        |
| ---- | ---------------------------- | ---------------------------------- |
| 70   | `EXIT_AUTHENTICATION_FAILED` | Auth credentials invalid           |
| 71   | `EXIT_AUTHORIZATION_FAILED`  | User lacks required permissions    |
| 72   | `EXIT_SECURITY_VIOLATION`    | Security policy violation detected |
| 73   | `EXIT_CERTIFICATE_INVALID`   | TLS certificate invalid/expired    |

**Use**: Security and authentication failures.

**Retry Hint**: `no_retry` - security issues require investigation.

---

### Observability (80-89)

| Code | Constant                     | When to Use                  |
| ---- | ---------------------------- | ---------------------------- |
| 80   | `EXIT_METRICS_UNAVAILABLE`   | Metrics endpoint unreachable |
| 81   | `EXIT_TRACING_UNAVAILABLE`   | Tracing backend unreachable  |
| 82   | `EXIT_LOGGING_FAILED`        | Log destination unreachable  |
| 83   | `EXIT_ALERT_DELIVERY_FAILED` | Cannot send alerts           |
| 84   | `EXIT_TELEMETRY_BUFFER_FULL` | Telemetry buffer overflow    |

**Use**: Observability infrastructure failures.

**Retry Hint**: Mix - depends on criticality of observability.

---

### Testing (90-99)

| Code | Constant                             | When to Use                     |
| ---- | ------------------------------------ | ------------------------------- |
| 90   | `EXIT_TEST_SUCCESS`                  | All tests passed                |
| 91   | `EXIT_TEST_FAILURE`                  | One or more tests failed        |
| 92   | `EXIT_TEST_ERROR`                    | Test framework error            |
| 93   | `EXIT_TEST_INTERRUPTED`              | Tests interrupted (user/signal) |
| 94   | `EXIT_TEST_COVERAGE_BELOW_THRESHOLD` | Coverage below required minimum |
| 95   | `EXIT_TEST_TIMEOUT`                  | Test execution timeout          |

**Use**: Test runners and CI/CD pipelines.

---

### Signals (128-165)

Unix signal termination follows the `128 + signal_number` pattern:

| Code | Constant       | Signal       | When to Use               |
| ---- | -------------- | ------------ | ------------------------- |
| 129  | `EXIT_SIGHUP`  | SIGHUP (1)   | Terminal hangup           |
| 130  | `EXIT_SIGINT`  | SIGINT (2)   | User interrupt (Ctrl+C)   |
| 131  | `EXIT_SIGQUIT` | SIGQUIT (3)  | User quit (Ctrl+\)        |
| 137  | `EXIT_SIGKILL` | SIGKILL (9)  | Forceful termination      |
| 141  | `EXIT_SIGPIPE` | SIGPIPE (13) | Broken pipe               |
| 142  | `EXIT_SIGALRM` | SIGALRM (14) | Alarm/timeout             |
| 143  | `EXIT_SIGTERM` | SIGTERM (15) | Graceful shutdown request |
| 158  | `EXIT_SIGUSR1` | SIGUSR1 (30) | User-defined signal 1     |
| 159  | `EXIT_SIGUSR2` | SIGUSR2 (31) | User-defined signal 2     |

**Use**: Signal handlers (see Signal Handling section below).

**Python Note**: Python's `signal` module uses platform-specific signal numbers. Use helper library's signal mapping utilities.

---

## Signal Handling Best Practices

### Go Example

```go
import (
    "os"
    "os/signal"
    "syscall"
    "github.com/fulmenhq/gofulmen/foundry"
)

func main() {
    sigChan := make(chan os.Signal, 1)
    signal.Notify(sigChan, syscall.SIGTERM, syscall.SIGINT)

    go func() {
        sig := <-sigChan
        log.Printf("Received signal: %v", sig)

        // Graceful shutdown
        if err := shutdown(); err != nil {
            log.Printf("Shutdown error: %v", err)
        }

        switch sig {
        case syscall.SIGTERM:
            os.Exit(foundry.ExitSIGTERM)
        case syscall.SIGINT:
            os.Exit(foundry.ExitSIGINT)
        }
    }()

    // Run application
    run()
}
```

### Python Example

```python
import signal
import sys
from pyfulmen.foundry import ExitCode

def signal_handler(signum, frame):
    print(f"Received signal: {signum}", file=sys.stderr)
    shutdown()

    if signum == signal.SIGTERM:
        sys.exit(ExitCode.EXIT_SIGTERM)
    elif signum == signal.SIGINT:
        sys.exit(ExitCode.EXIT_SIGINT)

signal.signal(signal.SIGTERM, signal_handler)
signal.signal(signal.SIGINT, signal_handler)
```

### TypeScript Example

```typescript
import { exitCodes } from "@fulmenhq/tsfulmen/foundry";

process.on("SIGTERM", async () => {
  console.error("Received SIGTERM");
  await shutdown();
  process.exit(exitCodes.EXIT_SIGTERM);
});

process.on("SIGINT", async () => {
  console.error("Received SIGINT");
  await shutdown();
  process.exit(exitCodes.EXIT_SIGINT);
});
```

---

## Application-Specific Exit Codes

Applications can define custom codes in **reserved ranges**:

### Range 100-127 (28 codes)

**Use**: Application-specific exit codes.

**Example**:

```go
const (
    ExitBatchJobPartialSuccess = 100
    ExitBatchJobNoRecords      = 101
    ExitWorkflowStepSkipped    = 102
)
```

**Guidelines**:

- Document in your app's README
- Don't conflict with Fulmen standard codes (0-99, 128-165)
- Use descriptive names following Fulmen conventions

### Range 166-191 (26 codes)

**Reserved**: Future signal extensions. Do not use.

### Range 192-254 (63 codes)

**Use**: User-defined codes for scripts, automation, internal tooling.

**Note**: Exit code 255 is reserved (often used by shells for exit code overflow).

---

## Simplified Modes

For novice users or simple CLI tools, Fulmen provides **simplified exit code modes** that collapse detailed codes into basic categories.

### Basic Mode (3 codes)

Collapses all errors into just 3 codes:

- **0**: Success
- **1**: Error (any failure)
- **2**: Usage error (CLI argument problems)

**Use Case**: Simple scripts, beginner-friendly CLIs.

**Configuration** (helper libraries provide API):

```go
// Go
foundry.SetSimplifiedMode(foundry.BasicMode)

// Python
from pyfulmen.foundry import set_simplified_mode, SimplifiedMode
set_simplified_mode(SimplifiedMode.BASIC)

// TypeScript
import { setSimplifiedMode, SimplifiedMode } from '@fulmenhq/tsfulmen/foundry';
setSimplifiedMode(SimplifiedMode.Basic);
```

### Severity Mode (8 codes)

Groups errors by severity:

- **0**: Success
- **1**: User error (arguments, input)
- **2**: Configuration error
- **3**: Runtime error
- **4**: System error (resources, permissions)
- **5**: Security error
- **6**: Test failure
- **7**: Observability failure

**Use Case**: Applications that want semantic grouping without full detail.

---

## Exit Code Metadata

Helper libraries provide metadata accessors for runtime inspection:

### Go

```go
info := foundry.GetExitCodeInfo(foundry.ExitPortInUse)
fmt.Printf("Code: %d\n", info.Code)
fmt.Printf("Name: %s\n", info.Name)
fmt.Printf("Category: %s\n", info.Category)
fmt.Printf("Description: %s\n", info.Description)
fmt.Printf("Retry Hint: %s\n", info.RetryHint)
```

### Python

```python
from pyfulmen.foundry import get_exit_code_info, ExitCode

info = get_exit_code_info(ExitCode.EXIT_PORT_IN_USE)
print(f"Code: {info['code']}")
print(f"Name: {info['name']}")
print(f"Category: {info['category']}")
print(f"Description: {info['description']}")
if 'retry_hint' in info:
    print(f"Retry Hint: {info['retry_hint']}")
```

### TypeScript

```typescript
import { getExitCodeInfo, exitCodes } from "@fulmenhq/tsfulmen/foundry";

const info = getExitCodeInfo(exitCodes.EXIT_PORT_IN_USE);
console.log(`Code: ${info.code}`);
console.log(`Name: ${info.name}`);
console.log(`Category: ${info.category}`);
console.log(`Description: ${info.description}`);
console.log(`Retry Hint: ${info.retryHint}`);
```

**Use Cases**:

- Logging structured exit code metadata
- Building restart/retry logic based on `retry_hint`
- Generating user-friendly error messages
- Telemetry correlation

---

## Retry Logic Patterns

Exit codes include `retry_hint` metadata to guide automation:

### Retry Hints

- **`retry`**: Transient error, safe to retry immediately
- **`no_retry`**: Permanent error, fix required before retry
- **`investigate`**: Ambiguous, requires human review

### Example Retry Logic (Go)

```go
func runWithRetry(cmd string, maxAttempts int) error {
    for attempt := 1; attempt <= maxAttempts; attempt++ {
        exitCode := runCommand(cmd)

        if exitCode == foundry.ExitSuccess {
            return nil
        }

        info := foundry.GetExitCodeInfo(exitCode)

        switch info.RetryHint {
        case "retry":
            log.Printf("Attempt %d failed (code %d), retrying...", attempt, exitCode)
            time.Sleep(time.Second * time.Duration(attempt)) // Exponential backoff
            continue
        case "no_retry":
            return fmt.Errorf("permanent failure (code %d): %s", exitCode, info.Description)
        case "investigate":
            return fmt.Errorf("ambiguous failure (code %d): %s - manual investigation required", exitCode, info.Description)
        }
    }
    return fmt.Errorf("max retry attempts exceeded")
}
```

---

## Observability Integration

### Structured Logging

```go
log.WithFields(log.Fields{
    "exit_code": foundry.ExitConfigInvalid,
    "exit_name": "EXIT_CONFIG_INVALID",
    "category":  "configuration",
    "retry_hint": "no_retry",
}).Error("Configuration validation failed")
```

### Metrics/Telemetry

Track exit code distribution in metrics:

```go
metrics.Counter("app.exits",
    "code", strconv.Itoa(exitCode),
    "category", info.Category,
).Inc()
```

**Use Cases**:

- Identify most common failure modes
- Alert on security-related exits (70-79)
- Track signal termination patterns
- Correlate exit codes with retry/restart behavior

---

## BSD Compatibility

Fulmen provides pragmatic mappings to BSD `sysexits.h` for compatibility:

| BSD Code | BSD Name    | Fulmen Mapping                 |
| -------- | ----------- | ------------------------------ |
| 64       | EX_USAGE    | `EXIT_USAGE` (preserved)       |
| 71       | EX_OSERR    | `EXIT_RESOURCE_EXHAUSTED` (33) |
| 74       | EX_IOERR    | `EXIT_FILE_WRITE_ERROR` (54)   |
| 75       | EX_TEMPFAIL | `EXIT_OPERATION_TIMEOUT` (34)  |

**Use**: When migrating from BSD-style exit codes or integrating with legacy systems.

**Access** (helper libraries provide mapping):

```go
bsdCode := foundry.MapToBSD(foundry.ExitResourceExhausted) // Returns 71
```

---

## Testing Exit Codes

### Go Testing

```go
func TestCommandFailsWithConfigError(t *testing.T) {
    cmd := exec.Command("./myapp", "--config", "invalid.yaml")
    err := cmd.Run()

    if exitErr, ok := err.(*exec.ExitError); ok {
        assert.Equal(t, foundry.ExitConfigInvalid, exitErr.ExitCode())
    } else {
        t.Fatal("Expected exit error")
    }
}
```

### Python Testing

```python
from pyfulmen.foundry import ExitCode
import subprocess

def test_command_fails_with_config_error():
    result = subprocess.run(
        ["./myapp", "--config", "invalid.yaml"],
        capture_output=True
    )
    assert result.returncode == ExitCode.EXIT_CONFIG_INVALID
```

### TypeScript Testing (Jest)

```typescript
test("command fails with config error", async () => {
  const { exitCode } = await execa("./myapp", ["--config", "invalid.yaml"], {
    reject: false,
  });
  expect(exitCode).toBe(exitCodes.EXIT_CONFIG_INVALID);
});
```

---

## Migration from Generic Exit Codes

### Before (Generic)

```go
if err := validateConfig(); err != nil {
    log.Printf("Config error: %v", err)
    os.Exit(1) // Generic failure
}
```

### After (Specific)

```go
if err := validateConfig(); err != nil {
    log.Printf("Config error: %v", err)
    os.Exit(foundry.ExitConfigInvalid) // Specific, observable
}
```

**Benefits**:

- Better observability (metrics by exit code)
- Retry automation knows how to handle specific failures
- Debugging is faster (exit code indicates root cause category)
- Consistent error semantics across Fulmen ecosystem

---

## References

- **Schema**: `schemas/library/foundry/v1.0.0/exit-codes.schema.json`
- **Catalog**: `config/library/foundry/exit-codes.yaml`
- **Library Implementation Guide**: `docs/standards/library/foundry/README.md#exit-codes`
- **Signal Handling Guide**: `docs/standards/fulmen/signals/README.md` (coming soon)
- **Application Identity**: `docs/standards/fulmen/identity/README.md` (coming soon)

---

**Version**: v1.0.0
**Effective Date**: 2025-11-02
**Maintained By**: Schema Cartographer, supervised by @3leapsdave
