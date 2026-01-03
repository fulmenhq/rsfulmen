---
title: "Cross-Language Coding Standards"
description: "Language-agnostic coding standards for output hygiene, timestamps, schema validation, exit codes, and logging across all FulmenHQ projects"
author: "Pipeline Architect"
author_of_record: "Dave Thompson (https://github.com/3leapsdave)"
supervised_by: "@3leapsdave"
date: "2025-10-08"
last_updated: "2025-11-30"
status: "approved"
tags:
  [
    "standards",
    "coding",
    "cross-language",
    "logging",
    "validation",
    "exit-codes",
  ]
---

# Cross-Language Coding Standards

## Overview

This document establishes **language-agnostic coding standards** that apply to all FulmenHQ projects regardless of implementation language. These patterns ensure consistency, reliability, and interoperability across the entire ecosystem.

**Language-Specific Standards**: See language-specific guides for implementation details:

- **[Go Coding Standards](go.md)**
- **[Python Coding Standards](python.md)**
- **[Rust Coding Standards](rust.md)**
- **[TypeScript Coding Standards](typescript.md)**

---

## 1. Output Hygiene and Stream Management

### 1.1 STDERR for Logs, STDOUT for Data ⚠️ **CRITICAL**

**Rule**: All diagnostic output (logs, debug info, status messages) MUST go to STDERR. STDOUT is reserved exclusively for structured data output.

**Why Critical**:

- Pipelines expect clean data on STDOUT
- CI/CD tools parse STDOUT for results
- STDERR can be suppressed/redirected independently
- Unix philosophy: data flows through STDOUT

**Implementation by Language**:

```go
// Go - Use logger package (writes to STDERR)
logger.Info("Processing completed: %d items", count)
logger.Debug("File opened: %s", filename)

// ✅ STDOUT for structured output only
fmt.Println(jsonOutput)  // Structured data

// ❌ NEVER: Diagnostic output to STDOUT
fmt.Printf("Processing file %s...\n", filename)
```

```typescript
// TypeScript - Use logger/console.error
logger.info("Processing completed", { count });
console.error("Debug: File opened");

// ✅ STDOUT for structured output only
console.log(JSON.stringify(result));

// ❌ NEVER: Diagnostic output to STDOUT
console.log("Processing file...");
```

```python
# Python - Use logging module (writes to STDERR)
logger.info("Processing completed: %d items", count)
logger.debug("File opened: %s", filename)

# ✅ STDOUT for structured output only
print(json.dumps(result))

# ❌ NEVER: Diagnostic output to STDOUT
print(f"Processing file {filename}...")
```

**Exception**: Applications with specific requirements (e.g., interactive CLIs) may use STDOUT for user-facing messages, but this MUST be documented in the project README.

### 1.2 Structured Output Formats

**Rule**: When emitting structured data, use JSON or YAML with proper schema validation.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "timestamp": "2025-10-08T15:30:00Z",
  "status": "success",
  "data": {
    "processed": 42,
    "errors": 0
  }
}
```

**No contamination**: STDOUT must contain ONLY the structured output, no debug messages, no status updates.

---

## 2. Timestamps and Date Handling

### 2.1 RFC3339 Timestamps (REQUIRED)

**Rule**: All timestamps MUST use RFC3339 format with timezone information.

**Format**: `YYYY-MM-DDTHH:MM:SSZ` or `YYYY-MM-DDTHH:MM:SS±HH:MM`

**Examples**:

- UTC: `2025-10-08T15:30:00Z`
- With timezone: `2025-10-08T15:30:00-04:00`
- With milliseconds: `2025-10-08T15:30:00.123Z`

**Why RFC3339**:

- Unambiguous timezone handling
- Sortable as strings
- ISO 8601 compliant
- Universal parser support
- No locale-dependent formatting

**Implementation**:

```go
// Go
timestamp := time.Now().Format(time.RFC3339)
// "2025-10-08T15:30:00Z"
```

```typescript
// TypeScript
const timestamp = new Date().toISOString();
// "2025-10-08T15:30:00.123Z"
```

```python
# Python
from datetime import datetime, timezone

timestamp = datetime.now(timezone.utc).isoformat()
# "2025-10-08T15:30:00.123456+00:00"

# Or with pendulum
import pendulum
timestamp = pendulum.now().to_rfc3339_string()
# "2025-10-08T15:30:00Z"
```

**Do NOT use**:

- Unix timestamps (hard to read, timezone ambiguous)
- Locale-specific formats (`MM/DD/YYYY`, `DD/MM/YYYY`)
- Custom formats without timezone
- Ambiguous formats (`2025-10-08 15:30:00`)

### 2.2 Duration Formats

**Rule**: Use ISO 8601 duration format for time periods.

**Format**: `P[n]Y[n]M[n]DT[n]H[n]M[n]S`

**Examples**:

- 5 seconds: `PT5S`
- 2 hours 30 minutes: `PT2H30M`
- 1 day: `P1D`
- 1 year 2 months: `P1Y2M`

**Alternative**: Human-readable strings with units for logs

- `"5s"`, `"2h30m"`, `"1d"` (for log output)

---

## 3. Schema Validation

### 3.1 Input/Output Validation (REQUIRED)

**Rule**: All formal inputs and outputs MUST be validated against published JSON Schemas using FulmenHQ validation standards.

**FulmenHQ Validator**: `goneat` binary

```bash
# Validate input data against schema
goneat schema validate-data \
  --schema schemas/config/v1.0.0/config.schema.json \
  --data config.yaml

# Validate schema itself
goneat schema validate-schema \
  schemas/config/v1.0.0/config.schema.json
```

**Schema Standards**:

- Use JSON Schema Draft 2020-12 (unless compatibility requires Draft-07)
- Store schemas in `schemas/` directory
- Version schemas semantically
- Include `$schema` and `$id` in all schemas

**What to Validate**:

- Configuration files (startup)
- API request/response bodies
- File formats (import/export)
- CLI command options (when structured)
- Inter-service message payloads

**What NOT to Validate** (performance reasons):

- High-frequency internal function calls
- Simple primitive values
- Already-validated data in hot paths

### 3.2 Schema Location and Naming

**Convention**: `schemas/<category>/<version>/<name>.schema.json`

```
schemas/
├── config/
│   └── v1.0.0/
│       └── app-config.schema.json
├── api/
│   └── v1.0.0/
│       ├── request.schema.json
│       └── response.schema.json
└── terminal/
    └── v1.0.0/
        └── schema.json
```

---

## 4. Exit Codes and Error Handling

### 4.1 CLI Exit Codes (REQUIRED for CLI tools)

**Rule**: Use conventional Unix exit codes for CLI applications.

| Code  | Meaning                | Usage                            |
| ----- | ---------------------- | -------------------------------- |
| 0     | Success                | Operation completed successfully |
| 1     | General error          | Catch-all for general errors     |
| 2     | Misuse                 | Invalid command-line arguments   |
| 3     | Configuration error    | Invalid or missing configuration |
| 4     | Input error            | Invalid input data or file       |
| 5     | Output error           | Cannot write output              |
| 6-125 | Application-specific   | Document in README               |
| 126   | Command cannot execute | Permission/binary issues         |
| 127   | Command not found      | Missing dependency               |
| 128+N | Fatal signal           | Killed by signal N               |
| 130   | SIGINT (Ctrl+C)        | User interrupted                 |

**Implementation**:

```go
// Go
if err != nil {
    logger.Error("Configuration error: %v", err)
    os.Exit(3)
}
```

```typescript
// TypeScript
if (error) {
  logger.error("Configuration error", { error });
  process.exit(3);
}
```

```python
# Python
import sys

if error:
    logger.error("Configuration error: %s", error)
    sys.exit(3)
```

**Best Practices**:

- Exit 0 ONLY on complete success
- Exit 1 for general errors when specific code doesn't apply
- Document application-specific codes (6-125) in README
- Use exit 2 for argument parsing errors (matches GNU standards)
- Log error details to STDERR before exiting

### 4.2 Error Context and Wrapping

**Rule**: Errors MUST include context about what operation failed and why.

```go
// Go - Error wrapping
return fmt.Errorf("failed to load config from %s: %w", path, err)
```

```typescript
// TypeScript - Error context
throw new ConfigurationError(
  `Failed to load config from ${path}: ${err.message}`,
);
```

```python
# Python - Exception chaining
raise ConfigurationError(f"Failed to load config from {path}") from err
```

**Error Messages Should**:

- Describe the operation that failed
- Include relevant context (file path, resource ID, etc.)
- Preserve the original error (wrapping/chaining)
- Be actionable (user can fix the problem)

---

## 5. Logging Standards

### 5.1 Log Levels

**Standard Levels** (in order of severity):

| Level          | Usage                                | Example                                    |
| -------------- | ------------------------------------ | ------------------------------------------ |
| DEBUG          | Detailed diagnostic info             | "Opening file: /path/to/file.txt"          |
| INFO           | General operational messages         | "Service started on port 8080"             |
| WARN           | Warning, non-fatal issues            | "Config file missing, using defaults"      |
| ERROR          | Error conditions, may cause failures | "Failed to connect to database: timeout"   |
| FATAL/CRITICAL | Application cannot continue          | "Required dependency unavailable, exiting" |

**Guidelines**:

- DEBUG: For developers troubleshooting issues
- INFO: For operators monitoring health
- WARN: For potential problems that don't stop execution
- ERROR: For failures that need attention
- FATAL: For failures requiring immediate action/restart

### 5.2 Structured Logging Context

**Rule**: Include relevant context in log messages.

```go
// Go
logger.Info("Request processed",
    zap.String("method", "GET"),
    zap.String("path",../protocol/users"),
    zap.Duration("duration", elapsed),
    zap.Int("status", 200),
)
```

```typescript
// TypeScript
logger.info("Request processed", {
  method: "GET",
  path:../protocol/users",
  duration: elapsed,
  status: 200,
});
```

```python
# Python
logger.info("Request processed", extra={
    "method": "GET",
    "path":../protocol/users",
    "duration": elapsed,
    "status": 200
})
```

**Context to Include**:

- Operation identifier (request ID, job ID)
- Resource identifiers (user ID, file path)
- Timing information (duration, timestamp)
- Outcome (success/failure, status code)

### 5.3 Log Format

**Recommended Format**: Structured JSON for production, human-readable for development

```json
{
  "timestamp": "2025-10-08T15:30:00Z",
  "level": "INFO",
  "message": "Request processed",
  "context": {
    "method": "GET",
    "path":../protocol/users",
    "duration_ms": 45,
    "status": 200
  }
}
```

---

## 6. Security Standards

### 6.1 Secrets Management

**Rule**: NEVER hardcode secrets in source code or configuration files committed to version control.

**Do**:

- Use environment variables
- Use secret management systems (HashiCorp Vault, AWS Secrets Manager)
- Use `.env` files (gitignored) for local development
- Rotate secrets regularly

**Do NOT**:

- Hardcode API keys, passwords, tokens
- Commit `.env` files to version control
- Log secret values
- Include secrets in error messages

```python
# ✅ CORRECT
import os
api_key = os.environ["API_KEY"]

# ❌ WRONG
api_key = "sk_live_1234567890abcdef"
```

### 6.2 Input Validation

**Rule**: Validate and sanitize all external input.

**Validate**:

- User input (CLI arguments, API requests)
- File paths (prevent path traversal)
- File contents (schema validation)
- Environment variables

**Example - Path Validation**:

```go
// Go
absPath, err := filepath.Abs(userPath)
if strings.Contains(absPath, "..") {
    return errors.New("path traversal detected")
}
```

```python
# Python
from pathlib import Path

def validate_path(user_path: Path) -> Path:
    abs_path = user_path.resolve()
    if ".." in abs_path.parts:
        raise ValueError("Path traversal detected")
    return abs_path
```

---

## 7. Versioning and Compatibility

### 7.1 Semantic Versioning

**Rule**: Use Semantic Versioning (SemVer) or Calendar Versioning (CalVer) consistently.

**SemVer**: `MAJOR.MINOR.PATCH`

- MAJOR: Breaking changes
- MINOR: New features, backward compatible
- PATCH: Bug fixes, backward compatible

**CalVer**: `YYYY.MM.MICRO` (Crucible uses this)

- YYYY: Year
- MM: Month (no leading zero)
- MICRO: Incremental within month

**Implementation**:

```python
# Python - CalVer
__version__ = "2025.10.2"

# Python - SemVer
__version__ = "1.2.3"
```

### 7.2 API Compatibility

**Rule**: Maintain backward compatibility within major versions.

**Breaking Changes Require**:

- Major version bump (SemVer)
- Or new year/month (CalVer)
- Migration guide in CHANGELOG
- Deprecation warnings before removal

---

## 8. Testing Standards

### 8.1 Test Coverage

**Minimum Requirements**:

- Public API: ≥90% coverage
- Critical paths: 100% coverage
- Error handling: Test failure cases

### 8.2 Test Organization

**Structure**:

```
tests/
├── unit/          # Fast, isolated tests
├── integration/   # Tests with external dependencies
└── fixtures/      # Test data
```

### 8.3 Test Naming

**Convention**: `test_<function_name>_<scenario>`

```python
def test_load_config_valid_file():
    """Test loading a valid configuration file."""
    pass

def test_load_config_missing_file():
    """Test that missing config raises appropriate error."""
    pass
```

### 8.4 Additional Testing Guidance

For comprehensive testing standards covering portable testing practices and language-specific patterns, see the [Testing Standards](../testing/README.md) directory:

- **[Portable Testing Practices](../testing/portable-testing-practices.md)** - Cross-language practices for deterministic, sandbox-friendly test suites
- **[Language-Specific Testing Patterns](../testing/language-testing-patterns.md)** - CLI testing isolation patterns for Go (Cobra), Python (Typer/Click), TypeScript (Commander/oclif), Rust, and C#

---

## 9. Documentation Standards

### 9.1 README.md Requirements

Every project MUST have a README.md with:

- Brief description (1-2 sentences)
- Installation instructions
- Quick start example
- Configuration options
- Development setup
- License information

### 9.2 Code Documentation

**Function/Method Documentation**:

- Purpose (what it does)
- Parameters (with types)
- Return value (with type)
- Errors/Exceptions raised
- Example usage (for public APIs)

**Example Formats**:

```go
// Go - GoDoc format
// LoadConfig loads configuration from the specified file.
// It returns an error if the file cannot be read or is invalid.
func LoadConfig(path string) (*Config, error)
```

```typescript
/**
 * Load configuration from the specified file.
 *
 * @param path - Path to configuration file
 * @returns Parsed configuration object
 * @throws ConfigurationError if file is invalid
 */
function loadConfig(path: string): Config;
```

```python
def load_config(path: Path) -> Config:
    """Load configuration from the specified file.

    Args:
        path: Path to configuration file.

    Returns:
        Parsed configuration object.

    Raises:
        ConfigurationError: If file cannot be read or is invalid.

    Example:
        >>> config = load_config(Path("config.yaml"))
        >>> print(config.host)
        localhost
    """
```

---

## 10. Code Review Checklist

Before submitting code for review, verify:

**Output and Logging**:

- [ ] No diagnostic output to STDOUT (logs go to STDERR)
- [ ] Structured output is clean (JSON/YAML only)
- [ ] Log messages include context (operation, resource IDs)
- [ ] Timestamps use RFC3339 format

**Error Handling**:

- [ ] Errors include context (what failed, why)
- [ ] Exit codes appropriate for failure type
- [ ] Errors properly wrapped/chained

**Security**:

- [ ] No hardcoded secrets
- [ ] Input validation on external data
- [ ] Path traversal prevention
- [ ] No secrets in logs or errors

**Validation**:

- [ ] Schemas defined for formal inputs/outputs
- [ ] Data validated against schemas
- [ ] Schema versioning followed

**Testing**:

- [ ] Tests cover happy path and error cases
- [ ] Public API coverage ≥90%
- [ ] Test names descriptive

**Documentation**:

- [ ] README updated if public API changed
- [ ] Function/method documentation complete
- [ ] CHANGELOG updated

---

## 11. Language-Specific Standards

These cross-language patterns are implemented in language-specific ways. See:

- **[Go Coding Standards](go.md)** - Go implementation details
- **[Python Coding Standards](python.md)** - Python implementation details
- **[Rust Coding Standards](rust.md)** - Rust implementation details
- **[TypeScript Coding Standards](typescript.md)** - TypeScript implementation details

Each language guide builds on these foundations with language-specific:

- Type systems and patterns
- Tooling requirements
- Testing frameworks
- Code organization
- Idioms and best practices

---

## Conclusion

These cross-language standards ensure FulmenHQ projects are consistent, interoperable, and maintainable regardless of implementation language. They provide the foundation upon which language-specific standards build.

**Remember**: Clean streams, validated data, meaningful errors, and proper logging enable reliable automation and seamless integration across the entire ecosystem.

_Adherence to these standards ensures enterprise-grade reliability and operational excellence._
