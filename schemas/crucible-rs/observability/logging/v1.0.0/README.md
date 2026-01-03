# Logging Schemas v1.0.0

JSON Schemas for standardized logging across Fulmen libraries.

## Schemas

- **definitions.schema.json** - Shared type definitions (severity levels, operators, field types)
- **log-event.schema.json** - Structured log event envelope
- **logger-config.schema.json** - Logger configuration
- **middleware-config.schema.json** - Middleware configuration
- **severity-filter.schema.json** - Severity filtering with comparison operators

### v1.0.0 Update (2025-10-09)

- Added correlation context fields (`contextId`, `requestId`, `correlationId`, `parentSpanId`, `operation`,
  `durationMs`, `userId`) to `log-event.schema.json`.
- Introduced reusable definitions for UUIDv7 and request/context identifiers.

## Severity Level Handling (Option 2: Hybrid Approach)

### Configuration (User-Facing)

Users specify severity levels as **strings** in config files:

```yaml
defaultLevel: "INFO"
sinks:
  - type: console
    level: "WARN"
```

### Runtime (Implementation-Required)

Implementations MUST:

1. Convert `severityName` strings to `severityLevel` integers using the canonical mapping in `definitions.schema.json#/$defs/severityMapping`
2. Use **numeric comparison** for all filtering operations (except EQ/NE which can use either)
3. Emit events with **both** `severity` (string) and `severityLevel` (integer)

### Canonical Mapping

| Name  | Level |
| ----- | ----- |
| TRACE | 0     |
| DEBUG | 10    |
| INFO  | 20    |
| WARN  | 30    |
| ERROR | 40    |
| FATAL | 50    |
| NONE  | 60    |

### Comparison Examples

```go
// User config:
filter := SeverityFilter{Operator: "GE", Level: "WARN"}

// Runtime conversion:
filterLevel := ParseSeverity("WARN")  // => {Name: "WARN", Level: 30}

// Comparison (MUST use numeric):
currentLevel := ParseSeverity("ERROR")  // => {Name: "ERROR", Level: 40}
passes := currentLevel.Level >= filterLevel.Level  // 40 >= 30 => true

// Emitted event:
{
  "severity": "ERROR",
  "severityLevel": 40,
  "message": "..."
}
```

### Cross-Language Consistency

All implementations (Go, TypeScript, Python, etc.) MUST:

- Use identical name→level mapping
- Perform numeric comparison for GE/LE/GT/LT operators
- Emit both `severity` and `severityLevel` in log events
- Accept string severity names in configuration

See individual schema files for detailed specifications.
