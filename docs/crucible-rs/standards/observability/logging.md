---
title: "Fulmen Logging Standard"
description: "Cross-language logging requirements within the observability program"
author: "Codex Assistant"
date: "2025-10-02"
last_updated: "2025-11-16"
status: "draft"
tags: ["observability", "logging", "telemetry"]
---

# Logging Standard

> Status: Draft – targeting first release with the logging/telemetry initiative.

## Scope

This standard governs structured logging across Fulmen repositories. It defines the event envelope, severity model, configuration structure, runtime expectations, and packaging strategy. Logging is a sibling within the broader observability program (metrics, tracing, etc.) and may be consumed independently.

## Event Envelope

All log events MUST emit JSON with the following shape (additional fields allowed unless noted):

| Field            | Type    | Required | Notes                                                                                      |
| ---------------- | ------- | -------- | ------------------------------------------------------------------------------------------ |
| `timestamp`      | string  | ✅       | RFC3339Nano UTC timestamp.                                                                 |
| `severity`       | string  | ✅       | Enum value (`TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL`, `NONE`).                   |
| `severityLevel`  | integer | ✅       | Numeric representation (TRACE=0, DEBUG=10, INFO=20, WARN=30, ERROR=40, FATAL=50, NONE=60). |
| `message`        | string  | ✅       | Human-readable message.                                                                    |
| `service`        | string  | ✅       | Service/application name.                                                                  |
| `component`      | string  | ⚠️       | Subsystem/component name; empty string discouraged.                                        |
| `logger`         | string  | ⚠️       | Logger instance identifier (e.g., `gofulmen.pathfinder`).                                  |
| `environment`    | string  | ⚠️       | Deployment environment tag (e.g., `production`, `staging`).                                |
| `context`        | object  | ⚠️       | Arbitrary key/value map (emit `{}` when empty).                                            |
| `contextId`      | string  | ⚠️       | Execution context identifier (job, pipeline, CLI invocation).                              |
| `requestId`      | string  | ⚠️       | Per-request identifier (HTTP `X-Request-ID` header).                                       |
| `correlationId`  | string  | ⚠️       | Cross-service correlation UUID (UUIDv7 generated if caller omits).                         |
| `traceId`        | string  | ⚠️       | REQUIRED when tracing enabled; OpenTelemetry trace identifier.                             |
| `spanId`         | string  | ⚠️       | REQUIRED when tracing enabled; span identifier.                                            |
| `parentSpanId`   | string  | ⚠️       | Optional parent span identifier for nested operations.                                     |
| `operation`      | string  | ⚠️       | Logical operation or handler name (CLI command, HTTP route, job step).                     |
| `durationMs`     | number  | ⚠️       | Operation duration in milliseconds.                                                        |
| `userId`         | string  | ⚠️       | Authenticated user identifier when available.                                              |
| `error`          | object  | ⚠️       | When present: `{ "message": string, "type"?: string, "stack"?: string }`.                  |
| `tags`           | array   | ⚠️       | Optional string array for ad-hoc filtering.                                                |
| `eventId`        | string  | ⚠️       | Optional unique identifier assigned by the producer.                                       |
| `throttleBucket` | string  | ⚠️       | Set when throttling drops are applied.                                                     |
| `redactionFlags` | array   | ⚠️       | Redaction indicators emitted by middleware (e.g., `["pii"]`).                              |

JSON output MUST be newline-delimited when written to files/streams.

> **Error Integration**: Structured errors emitted via the error-handling module reuse the Pathfinder envelope and may populate `severity`, `severityLevel`, `correlationId`, and `traceId`. When logging those errors, prefer the numeric `severityLevel` for comparisons and carry through `correlationId` so telemetry remains linkable.

## Progressive Profiles

Fulmen helper libraries expose a progressive configuration surface defined by `schemas/observability/logging/v1.0.0/logger-config.schema.json`. The `profile` field selects the appropriate complexity for the application:

| Profile        | Typical Use Cases                                    | Features Enabled                                                    | Required Configuration                                                   |
| -------------- | ---------------------------------------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| **SIMPLE**     | CLI tooling, scripts, local experiments              | Console output only, minimal configuration                          | `service` (all other fields optional)                                    |
| **STRUCTURED** | API services, background jobs, dev/staging workloads | Structured sinks, static fields, correlation IDs                    | `service`, `sinks`                                                       |
| **ENTERPRISE** | Workhorse services, production environments          | Multiple sinks, middleware pipeline, throttling, policy enforcement | `service`, `sinks`, `middleware` (optionally `throttling`, `policyFile`) |
| **CUSTOM**     | Specialized logging adapters or legacy integrations  | Full control via `customConfig` payload                             | `service`, `customConfig`                                                |

Profiles build upon each other; moving from SIMPLE to ENTERPRISE adds capabilities without breaking compatibility.

### Configuration Examples

```yaml
# SIMPLE profile (defaults are enough for CLI tooling)
profile: SIMPLE
service: mycli
defaultLevel: INFO

# STRUCTURED profile (service with console and file sinks)
profile: STRUCTURED
service: api-gateway
sinks:
  - type: console
    format: json
  - type: file
    path: logs/gateway.log

# ENTERPRISE profile with policy enforcement
profile: ENTERPRISE
service: datawhirl
policyFile: /org/logging-policy.yaml
sinks:
  - type: console
    format: json
  - type: rolling-file
    path: logs/datawhirl.log
middleware:
  - name: redact-secrets
  - name: correlation
throttling:
  enabled: true
  maxRate: 1000
  burstSize: 100
```

### Policy Enforcement

Organizations MAY provide a YAML policy file (validated by `schemas/observability/logging/v1.0.0/logging-policy.schema.json`) to enforce governance rules:

```yaml
allowedProfiles: [SIMPLE, STRUCTURED, ENTERPRISE]
requiredProfiles:
  workhorse: [ENTERPRISE]
environmentRules:
  production: [STRUCTURED, ENTERPRISE]
profileRequirements:
  ENTERPRISE:
    requiredFeatures: [correlation, middleware, throttling]
auditSettings:
  logPolicyViolations: true
  enforceStrictMode: false
```

Policy files are resolved in the following order:

1. `.fulmen/logging-policy.yaml` (repository-local development)
2. `/etc/fulmen/logging-policy.yaml` (system-wide default)
3. `$FULMEN_ORG_PATH/logging-policy.yaml` (organization-managed baseline, default `/opt/fulmen/`)

Libraries MUST validate logger configuration against the policy (when present) during initialization. Violations MUST be logged and, when `enforceStrictMode` is true, MUST prevent the logger from starting.

> Goneat tooling can generate starter policy files via `goneat bootstrap --generate-policy`.

### Correlation & Context Propagation

- **Correlation ID (`correlationId`)**: generate a UUIDv7 when the caller does not provide one. Propagate inbound
  values across HTTP (`X-Correlation-ID`) and gRPC metadata. UUIDv7 ensures time-sortable identifiers for Splunk
  and Datadog searches.
- **Request ID (`requestId`)**: represent the current transport request. For HTTP, read/emit `X-Request-ID`.
  For CLI workflows, generate an operation-scoped UUID (prefix optional) and surface it in human output.
- **Context ID (`contextId`)**: tie together larger execution scopes (batch pipeline run, scheduled job, CLI
  session). CLI tools SHOULD reuse a single context ID for the entire invocation while generating distinct
  request IDs per sub-command when appropriate.
- **Tracing IDs**: when OpenTelemetry (or another tracer) is enabled, emit `traceId`, `spanId`, and
  `parentSpanId` for every event within the span. Absence of tracing MUST fall back to correlation/request IDs
  so downstream systems still link records.
- **Operation metadata**: populate `operation`, `durationMs`, and `userId` when available so dashboards can
  aggregate latency and audit activity.

## Severity Enum & Filtering

Severity values and numeric order:

| Name    | Numeric | Description                                       |
| ------- | ------- | ------------------------------------------------- |
| `TRACE` | 0       | Highly verbose diagnostics.                       |
| `DEBUG` | 10      | Debug-level details.                              |
| `INFO`  | 20      | Core operational events.                          |
| `WARN`  | 30      | Something unusual but not breaking.               |
| `ERROR` | 40      | Request/operation failure (recoverable).          |
| `FATAL` | 50      | Unrecoverable failure; program exit expected.     |
| `NONE`  | 60      | Explicitly disable emission (sink-level filters). |

Comparisons (e.g., `< INFO`, `>= WARN`) MUST operate on numeric levels. `NONE` is treated as "filter everything" when used as a minimum level.

## Configuration Model

Configuration is authored in YAML and normalized to JSON. Top-level fields:

- `defaultLevel` – minimum severity (enum).
- `sinks[]` – array of sink entries with `type`, `level`, `options`, `middleware`, and `throttling`.
- `middleware[]` – global middleware chain definitions.
- `encoders` – named encoder configs (e.g., JSON, NDJSON with additional formatting).
- `fields` – static (`fields.static`) and dynamic (`fields.dynamic`) attributes appended to events.
- `throttling` – global defaults (`mode`, `bufferSize`, `dropPolicy`).
- `exports` – optional remote sink definitions (future use).

### Sink Options

Each sink entry includes:

```yaml
- name: console
  type: console
  level: INFO
  encoder: json
  middleware: [redact-secrets]
  throttling:
    mode: non-blocking
    bufferSize: 1000
    dropPolicy: drop-oldest
  options:
    stderrOnly: true
```

Supported sink types: `console`, `file`, `rolling-file`, `memory`, `external` (future). Console sinks MUST force `stderrOnly: true`. File sinks define path, rotation, retention.

### Middleware

Middleware entries define processors applied before emission. Interface semantics:

- **Go**: `type Middleware func(event *Event) (skip bool)` executed sequentially.
- **TypeScript**: `(event: LogEvent) => LogEvent | null` where `null` indicates drop.
- **Rust/Python/C#**: Align with language idioms (e.g., `Layer` in `tracing`, processor list in structlog, `Enricher`/`Filter` in Serilog).

Recommended built-ins: `redact-secrets`, `redact-pii`, `request-context` (injects correlation/request IDs),
`annotate-trace`, `throttle` (wraps queue logic).

#### Redaction Middleware (Optional - All Profiles)

Redaction middleware is **available** for SIMPLE, STRUCTURED, and ENTERPRISE profiles as an **opt-in capability**. Implementations follow the middleware configuration schema at `schemas/observability/logging/v1.0.0/middleware-config.schema.json`.

**Availability**:

- **Profiles**: SIMPLE, STRUCTURED, ENTERPRISE
- **Default**: Disabled (no redaction applied)
- **Enabled via**: LoggerConfig `middleware` array with `type: "redaction"`
- **Schema**: `middleware-config.schema.json#/$defs/redactionConfig`

**Purpose**: Mask sensitive data (tokens, PII, credentials, secrets) in log messages and structured fields before emission to sinks.

**Configuration Structure** (per schema):

```yaml
middleware:
  - type: redaction
    enabled: true
    priority: 10 # Lower priority runs first (before correlation, etc.)
    redaction:
      # Regex patterns to match and redact
      patterns:
        - "SECRET_[A-Z0-9_]+" # Environment variables like SECRET_KEY
        - "[A-Z0-9_]*TOKEN[A-Z0-9_]*" # API tokens like GITHUB_TOKEN
        - "[A-Z0-9_]*KEY[A-Z0-9_]*" # Keys like API_KEY, DATABASE_KEY
        - "[A-Za-z0-9+/]{40,}={0,2}" # Base64-encoded secrets (40+ chars)

      # Specific field names to redact completely
      fields:
        - password
        - token
        - apiKey
        - authorization
        - secret

      # Replacement string for redacted content
      replacement: "[REDACTED]"
```

**Schema Reference** (`middleware-config.schema.json`):

- `patterns` (array of strings): Regex patterns to match and redact in message text
- `fields` (array of strings): Structured field names to redact completely
- `replacement` (string, default `"[REDACTED]"`): Text replacing redacted content

**Pattern Examples** (implementation-specific defaults may vary):

| Pattern Category      | Regex Example                                    | Matches                                      |
| --------------------- | ------------------------------------------------ | -------------------------------------------- |
| Environment Variables | `SECRET_[A-Z0-9_]+`                              | `SECRET_KEY`, `SECRET_DATABASE_URL`          |
| API Tokens            | `[A-Z0-9_]*TOKEN[A-Z0-9_]*`                      | `GITHUB_TOKEN`, `API_TOKEN`, `AUTH_TOKEN`    |
| Keys                  | `[A-Z0-9_]*KEY[A-Z0-9_]*`                        | `API_KEY`, `DATABASE_KEY`, `ENCRYPTION_KEY`  |
| Base64 Secrets        | `[A-Za-z0-9+/]{40,}={0,2}`                       | Long base64-encoded strings (40+ characters) |
| Email Addresses       | `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}` | `user@example.com`                           |
| Credit Cards          | `\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}`         | `4111-1111-1111-1111`                        |

**NOTE**: Patterns shown are examples. Helper library implementations (gofulmen, tsfulmen, pyfulmen) MAY provide these as defaults but MUST allow full configuration via `patterns` array per schema. No patterns are hardcoded in the Crucible schema.

**Behavior**:

- **Execution Order**: Controlled by `priority` field (lower values run first). If multiple middleware have the same priority, execution order is undefined (implementation-specific). Recommend assigning distinct priorities to avoid ambiguity.
- **Scope**: Applies to message text AND structured context fields
- **Field Matching**: Field names SHOULD be matched case-insensitively (e.g., `password`, `Password`, `PASSWORD` all match field `"password"`)
- **Replacement**: Matched content replaced with `replacement` string
- **Flags**: Sets `redactionFlags` array in event envelope. Initial implementations SHOULD use generic `["redacted"]` flag; future versions MAY categorize by pattern type (`["pii"]`, `["secret"]`, `["pci"]`)
- **Idempotent**: Running multiple times produces same output
- **Safe**: Never modifies original log call arguments, only emitted events

**Compatibility**:

- ✅ Console sinks (stderr JSON)
- ✅ File sinks (JSON, NDJSON)
- ✅ All profiles (SIMPLE, STRUCTURED, ENTERPRISE)
- ✅ Combines with correlation, throttling, sampling middleware

**Profile-Specific Recommendations**:

| Profile        | When to Enable Redaction                                                    | Typical Configuration                                               |
| -------------- | --------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| **SIMPLE**     | CLI tools handling secrets (deployment scripts, Fulmen-Secrets)             | Minimal patterns targeting known env vars (`SECRET_*`, `*_TOKEN`)   |
| **STRUCTURED** | Services processing user data or credentials                                | Field-based redaction (`password`, `token`) + common token patterns |
| **ENTERPRISE** | Production environments with compliance requirements (GDPR, HIPAA, PCI-DSS) | Comprehensive patterns + field redaction + audit flags              |

**Example Configurations**:

**SIMPLE Profile** (CLI tool with secrets):

```yaml
profile: SIMPLE
service: deployment-tool
middleware:
  - type: redaction
    enabled: true
    redaction:
      patterns:
        - "SECRET_[A-Z0-9_]+"
        - "[A-Z0-9_]*TOKEN[A-Z0-9_]*"
      replacement: "***"
```

**STRUCTURED Profile** (API service):

```yaml
profile: STRUCTURED
service: user-api
middleware:
  - type: redaction
    enabled: true
    priority: 5
    redaction:
      fields:
        - password
        - accessToken
        - refreshToken
      patterns:
        - "[A-Za-z0-9+/]{40,}={0,2}" # Base64 tokens
      replacement: "[REDACTED]"
  - type: augmentation
    priority: 10
    augmentation:
      fields:
        environment: production
```

**ENTERPRISE Profile** (compliance-driven):

```yaml
profile: ENTERPRISE
service: payment-processor
policyFile: /etc/fulmen/logging-policy.yaml
middleware:
  - type: redaction
    enabled: true
    priority: 1 # Run before all other middleware
    redaction:
      patterns:
        - "\\d{4}[\\s-]?\\d{4}[\\s-]?\\d{4}[\\s-]?\\d{4}" # Credit cards
        - "\\d{3}-\\d{2}-\\d{4}" # SSN
        - "[A-Za-z0-9+/]{40,}={0,2}" # Tokens
      fields:
        - cardNumber
        - cvv
        - ssn
        - password
      replacement: "[PCI-REDACTED]"
```

**Recommended Default Priorities** (for cross-language consistency):

| Middleware Type | Recommended Priority | Rationale                                                   |
| --------------- | -------------------- | ----------------------------------------------------------- |
| Redaction       | 10                   | Run early to mask secrets before other middleware processes |
| Augmentation    | 15                   | Add fields after redaction, before correlation              |
| Correlation     | 20                   | Generate IDs after augmentation                             |
| Sampling        | 25                   | Sample after correlation IDs assigned                       |
| Throttling      | 30                   | Run last to control output rate                             |

**Note**: These are recommendations, not requirements. Applications MAY assign different priorities based on specific needs. When middleware have the same priority, execution order is undefined.

**Performance Considerations**:

- **Regex Overhead**: Each pattern adds ~0.1-1ms per log event (depends on message length)
- **Recommendation**: Use specific patterns, avoid greedy regex like `.*`
- **Optimization**: Helper libraries MUST pre-compile all regex patterns at logger initialization (not per-event). Store compiled patterns in middleware struct. Fail fast on invalid patterns with clear error messages.
- **Trade-off**: Opt-in by default because not all applications handle secrets

**Schema Validation**:

- Middleware configuration MUST validate against `middleware-config.schema.json`
- Invalid patterns (malformed regex) MUST fail at logger initialization
- Unknown fields in `redaction` config MUST be rejected per schema `additionalProperties: false`

**Testing**:

- Helper libraries MUST include redaction test fixtures
- Tests MUST verify patterns match expected inputs (see schema examples)
- Cross-language parity tests ensure identical redaction behavior

### Throttling / Backpressure

Configuration keys:

- `mode`: `blocking` | `non-blocking`.
- `bufferSize`: integer (required when `blocking`).
- `dropPolicy`: `drop-oldest` | `drop-newest` (for non-blocking) | `block`.
- `flushInterval`: optional duration for background flush in non-blocking mode.

Underlying libraries must map these semantics appropriately (see implementation notes).

## Output Channels

- Console sink writes to `stderr` only. Duplicating to `stdout` is forbidden to preserve CLI/streaming guarantees.
- Application output intended for users or upstream pipelines continues to use `stdout` outside of the logging pipeline.

## Runtime API Expectations

Language packages MUST expose:

- Constructors accepting `LoggerOptions` (service, component, min level, middleware list, sinks, throttling config).
- Methods: `Trace`, `Debug`, `Info`, `Warn`, `Error`, `Fatal`, `WithFields`, `WithError`, `Sync` (or idiomatic equivalents).
- Middleware registration API (chain composition).
- Graceful shutdown via `Sync` to flush buffers.

## Cross-Language Implementation

| Language   | Baseline Library                             | Notes                                                                                 |
| ---------- | -------------------------------------------- | ------------------------------------------------------------------------------------- |
| Go         | `uber-go/zap`                                | Struct-embedded implementation honouring progressive profiles and policy enforcement. |
| TypeScript | `pino`                                       | Discriminated unions map profiles to configuration shapes; transports handle sinks.   |
| Rust       | `tracing` + `tracing-subscriber`             | Profile-to-layer conversion with policy enforcement via tower middleware.             |
| Python     | `structlog` (over stdlib logging)            | Delegation pattern switches implementations per profile; policies validated on init.  |
| C#         | `Serilog` via `Microsoft.Extensions.Logging` | Config binding enforces profiles; enrichers and filters model middleware/throttling.  |

Each package must be installable standalone (e.g., `fulmen-logging` on PyPI) but can be bundled in a future "observability" meta-package.

### Progressive Logging Playbook

| Profile    | Default Sinks                                   | Required Middleware                                                        | Notes                                                                                                                                             |
| ---------- | ----------------------------------------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| SIMPLE     | `console` (stderr JSON)                         | `annotate-trace` (optional), `throttle` (optional), `redact-*` (optional)  | Zero-config path for tooling; inherits defaults from schema. Redaction opt-in for secrets handling (see Redaction Middleware).                    |
| STRUCTURED | `console` + optional `file`                     | `annotate-trace`, `throttle`, `correlation-context`, `redact-*` (optional) | Targets long-running CLIs and services needing durable logs. Redaction opt-in for sensitive data (see Redaction Middleware).                      |
| ENTERPRISE | `console`, `file`, optional external transports | `annotate-trace`, `throttle`, `correlation-context`, `redact-*`            | MUST honour `policyFile` and throttle limits; supports per-sink middleware ordering. Redaction recommended for compliance (GDPR, HIPAA, PCI-DSS). |
| CUSTOM     | Author-defined                                  | Author-defined                                                             | Reserved for advanced scenarios; still must validate against schema + policy contracts.                                                           |

Implementations MUST document how these defaults materialize (zap cores, pino transports, structlog processors, etc.) so auditors can trace configuration to runtime behaviour.

### Schema Field Naming & Hydration

- Schemas publish keys in camelCase; languages MUST map them to idiomatic naming (`snake_case` for Python, exported struct fields for Go, lowerCamelCase properties for TypeScript). A single normalization layer per language performs this conversion, applies defaults, and flattens nested values such as `middleware[].config`.
- Middleware registries accept `(name, config, order, enabled)` and return typed components. Avoid ad-hoc `map[string]any`/`Record<string, unknown>` mutations after normalization.
- Policy files integrate with normalization. Implementations MUST fail fast when `enforceStrictMode` denies a configuration; placeholder loaders are prohibited.

### Shared Fixtures & Golden Events

- Maintain canonical fixtures (for example, `tests/fixtures/logging/simple.yaml`, `enterprise-event.json`) in every language repository. Fixtures reference the Crucible schema version they target.
- CI MUST load each fixture through the normalization layer, instantiate middleware, emit representative events, and validate both hydrated configs and emitted JSON against `logger-config` and `log-event` schemas. Snapshot/approval tests SHOULD guard against regression.
- Coordinate fixture updates across languages (tracked in `.plans/active/.../progressive-logger-crosslang.md` or successor planning docs) so parity gaps stay visible.

## Packaging & Distribution

- Go: `gofulmen` module (`foundation/logging`).
- TypeScript: `@fulmenhq/crucible/logging` entry point.
- Python: `fulmen-logging` PyPI package (optional dependency for full Crucible bundle).
- Rust: `fulmen_logging` crate.
- C#: `Fulmen.Logging` NuGet package.

## Validation & Tooling

- `make release:check` MUST run logging schema validation (AJV or similar) and ensure severity enum alignment.
- Future CLI hook (e.g., via `goneat`) will lint redaction/throttling config.

## Roadmap

- Finalize schema files in `schemas/observability/logging/v1.0.0/`.
- Produce sink capability matrix for documentation.
- Extend to metrics/tracing once logging baseline is shipped.

## Contacts

- Human maintainer: @3leapsdave
- AI steward: @schema-cartographer (🧭)
