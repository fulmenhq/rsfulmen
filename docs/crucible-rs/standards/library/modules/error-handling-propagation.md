---
title: "Error Handling & Propagation"
description: "Uniform error wrapper built on the Pathfinder error envelope with optional telemetry metadata."
status: "draft"
last_updated: "2025-10-23"
tags: ["standards", "library", "errors", "pathfinder", "observability"]
---

# Error Handling & Propagation

## Overview

This standard defines a consistent, schema-driven error model across Fulmen helper libraries. Instead of
introducing a new envelope, it **extends the existing Pathfinder error response** via `$ref`, layering optional
telemetry properties (`severity`, `severity_level`, `correlation_id`, `trace_id`, `exit_code`) while keeping all
current payloads valid. Libraries can adopt the richer fields incrementally without breaking consumers.

## Principles

- **Pathfinder-Compatible** – Base fields (`code`, `message`, `details`, `timestamp`) come directly from
  `schemas/pathfinder/v1.0.0/error-response.schema.json` to preserve existing contracts.
- **Optional Telemetry** – Additional metadata is optional and ignored by legacy clients; new consumers should
  prefer the numeric `severity_level` for comparisons.
- **Structured Context** – `context` (`additionalProperties: true`) captures non-sensitive debugging details.
- **JSON-First** – Errors MUST serialise to JSON for pipelines and agents; human-facing renderers may format
  the payload differently.
- **Extensible & Offline** – `additionalProperties: true` allows future metadata; validation happens offline via
  goneat/AJV/gojsonschema (no network calls).

## API Surface

Every language provides idiomatic helpers built around the shared schema:

- `wrap(baseError, options)` – Accepts a Pathfinder error (or equivalent shape) and augments it with context,
  telemetry, and the original error object.
- `toJSON()/to_dict()` – Produces a schema-compliant JSON payload.
- `validate(payload)` – Uses the shared schema to confirm validity.
- `exitWithError(exitCode, error)` – Logs, sets `exit_code`, and exits with the provided code.

## Canonical Data Model (ADR-0006)

Per [ADR-0006](../../architecture/decisions/ADR-0006-error-data-models.md) all Fulmen libraries MUST implement
the error contract as a **data model/struct/interface**, not as language-native exception classes. Exceptions
or language-specific wrappers MAY be layered on top, but the canonical representation MUST preserve the fields
defined below so that serialisation, schema validation, and cross-language interoperability remain consistent.

| Field            | Type              | Notes                                                              |
| ---------------- | ----------------- | ------------------------------------------------------------------ |
| `code`           | string (required) | Pathfinder error code                                              |
| `message`        | string (required) | Human-readable description                                         |
| `details`        | object            | Additional structured information (optional)                       |
| `path`           | string            | Resource path or identifier (optional)                             |
| `timestamp`      | RFC3339 string    | Time error occurred (optional but recommended)                     |
| `severity`       | enum              | `"info" \| "low" \| "medium" \| "high" \| "critical"` (optional)   |
| `severity_level` | integer           | Numeric severity (0-4) aligned with assessment taxonomy (optional) |
| `correlation_id` | string            | Request/trace correlation ID (optional)                            |
| `trace_id`       | string            | Tracing identifier (optional)                                      |
| `exit_code`      | integer           | Recommended process exit code (optional)                           |
| `context`        | object            | Non-sensitive debugging context (optional)                         |
| `original`       | string or object  | Serialized original error or payload (optional)                    |

Implementations MUST expose constructors/factories that accept these fields, and helpers MUST emit JSON that
validates against `schemas/error-handling/v1.0.0/error-response.schema.json`. Wrappers that integrate with
language-native error mechanisms (e.g., `raise`, `throw`, Go `error`) SHOULD embed or reference the canonical
data model rather than duplicating fields.

### Language Examples

**Go**

```go
base := pathfinder.NewError("CONFIG_INVALID", "Config load failed")
err := errorhandling.Wrap(base, errorhandling.Options{
    Context: map[string]any{"path": "/app.yaml"},
    Original: originalErr,
    Severity: errorhandling.SeverityHigh,
    CorrelationID: logger.CorrelationID(),
})
if !errorhandling.Validate(err) {
    log.Fatal("invalid error payload")
}
errorhandling.ExitWithError(3, err)
```

**TypeScript**

```typescript
const base = new PathfinderError("CONFIG_INVALID", "Config load failed");
const err = FulmenError.wrap(base, {
  context: { path: "/app.yaml" },
  original: originalErr,
  severity: "high",
  correlationId: currentCorrelationId(),
});
if (!FulmenError.validate(err)) throw new Error("Invalid payload");
FulmenError.exitWithError(3, err);
```

**Python**

```python
base = PathfinderError(code="CONFIG_INVALID", message="Config load failed")
err = FulmenError.wrap(
    base,
    context={"path": "/app.yaml"},
    original=original_err,
    severity="high",
    correlation_id=current_correlation(),
)
if not FulmenError.validate(err):
    raise ValueError("Invalid payload")
FulmenError.exit_with_error(3, err)
```

## Testing Expectations

- **Coverage**: ≥95 % branches covering wrapping, validation, serialisation, and optional telemetry paths.
- **Fixtures**: `tests/fixtures/errors/` SHOULD include both legacy Pathfinder payloads and extended payloads
  (severity + correlation) to guarantee backward compatibility.
- **Integration**: Validate alongside logging (ensuring severity maps into progressive logger events) and schema
  validation flows.
- **Tooling**: goneat `schema validate` must pass against `schemas/error-handling/v1.0.0/error-response.schema.json`.

## Ecosystem Ties

- **Observability**: Optional severity/severity_level align with the shared assessment severity mapping used by
  DevSecOps policies and logging.
- **API Standards**: HTTP/gRPC layers SHOULD reuse this envelope for structured error responses.
- **ADR Mapping**: Document adoption via ecosystem ADR-0002 (triple-index) and ADR-0006 (error data models).
