---
title: "Go Coding Standards for FulmenHQ"
description: "Go-specific coding standards including output hygiene, error handling, concurrency patterns, and testing for enterprise-grade Go development"
author: "Code Scout"
date: "2025-10-01"
last_updated: "2025-10-08"
status: "approved"
tags: ["standards", "coding", "go", "concurrency", "testing"]
related_docs: ["README.md"]
---

# Go Coding Standards for FulmenHQ

## Overview

This document establishes coding standards for FulmenHQ Go projects, ensuring consistency, quality, and adherence to enterprise-grade practices. As tools designed for scale, FulmenHQ projects require rigorous standards to maintain reliability and structured output integrity.

**Core Principle**: Write idiomatic Go code that is simple, readable, and maintainable, with strict output hygiene for structured data integrity.

**Foundation**: This guide builds upon **[Cross-Language Coding Standards](README.md)** which establishes patterns for:

- Output hygiene (STDERR for logs, STDOUT for data)
- RFC3339 timestamps
- Schema validation with goneat
- CLI exit codes
- Logging standards
- Security practices

Read the cross-language standards first, then apply the Go-specific patterns below.

---

## 1. Critical Rules (Zero-Tolerance)

### 1.1 Output Hygiene ⚠️ **CRITICAL**

**Rule**: Output streams must remain clean for structured output (JSON, YAML) consumed by tools and automation.

**DO**: Use appropriate logging packages for all output

```go
import "github.com/fulmenhq/goneat/pkg/logger" // or equivalent

// ✅ Correct logging
logger.Debug("assessment: detected uncommitted files")
logger.Info("Operation completed in 50ms: 3 issues found")
logger.Error("Failed to process: %v", err)
logger.Warn("Configuration file not found, using defaults")
```

**DO NOT**: Pollute output streams with direct writes

```go
// ❌ CRITICAL ERROR: Breaks structured output
fmt.Printf("DEBUG: Creating issue for %d files\n", count)
fmt.Println("Processing files...")
log.Printf("Status: %v", status)
println("Debug info")

// ❌ These break structured output consumed by CI/CD tools
os.Stdout.WriteString("Status message\n")
```

**Why Critical**: FulmenHQ tools produce structured output consumed by:

- CI/CD pipelines expecting clean data
- Automated tools parsing results
- Agentic systems processing structured data
- Pre-commit/pre-push hooks expecting parseable output

**Enforcement**: Any direct output writes in core logic will fail code review.

### 1.2 Error Handling

**DO**: Always handle errors explicitly

```go
// ✅ Proper error handling
result, err := process(ctx, target, config)
if err != nil {
    return fmt.Errorf("processing failed for %s: %w", category, err)
}
```

**DO NOT**: Ignore errors or use blank identifiers unnecessarily

```go
// ❌ Never ignore errors
result, _ := process(ctx, target, config)

// ❌ Don't ignore critical errors
os.ReadFile(configFile) // Missing error check
```

### 1.3 Interface Contracts

Implement interfaces correctly and completely:

```go
type Processor interface {
    Process(ctx context.Context, target string, config Config) (*Result, error)
    CanRunInParallel() bool
    GetCategory() Category
    GetEstimatedTime(target string) time.Duration
    IsAvailable() bool
}
```

**Success Logic**:

```go
// ✅ Correct success determination
success := len(issues) == 0

return &Result{
    Success: success,
    Issues:  issues,
    // ...
}
```

---

## 2. Code Organization and Structure

### 2.1 Project Structure

Follow FulmenHQ's established structure:

```
project/
├── cmd/                    # CLI commands
├── internal/
│   ├── core/              # Core business logic
│   ├── assets/            # Embedded assets
│   └── utils/             # Internal utilities
├── pkg/
│   ├── config/            # Configuration handling
│   ├── logger/            # Logging utilities
│   └── schema/            # Schema validation
└── schemas/               # JSON/YAML schemas
```

### 2.2 Naming Conventions

- **Types**: PascalCase (e.g., `Processor`, `CategoryResult`)
- **Functions**: camelCase (e.g., `validateConfig`, `buildMetrics`)
- **Constants**: PascalCase for exported, camelCase for unexported
- **Files**: snake_case with descriptive names (e.g., `repo_status_processor.go`)

---

## 3. Logging and Output Standards

### 3.1 Logging Levels

```go
// Debug: Detailed information for troubleshooting
logger.Debug("processing %d files for changes", fileCount)

// Info: General operational messages
logger.Info("Operation completed in %v: %d issues found", duration, issueCount)

// Warn: Warning conditions that don't stop execution
logger.Warn("Configuration file not found, using defaults")

// Error: Error conditions that may cause failures
logger.Error("Failed to process: %v", err)
```

### 3.2 Structured Logging Context

Include relevant context in log messages:

```go
// ✅ Good: Contextual information
logger.Info("Operation completed",
    zap.String("category", "status"),
    zap.Duration("duration", elapsed),
    zap.Int("issues_found", len(issues)),
)

// ❌ Bad: Missing context
logger.Info("Operation completed")
```

### 3.3 Structured Output Integrity

Never contaminate structured output streams:

```go
// ✅ Correct: Clean structured output
func GenerateReport() (*Report, error) {
    // All logging goes to stderr via logger package
    logger.Debug("generating report")

    report := &Report{
        Metadata: metadata,
        Summary:  summary,
        // ...
    }

    return report, nil // Clean return for marshaling
}
```

### 3.4 Schema-Driven Configuration Hydration

- Funnel all schema-authored logger configuration through a dedicated normalizer (for example, `NormalizeLoggerConfig(raw map[string]any) (Config, error)`) that uppercases profile identifiers, converts camelCase keys to struct fields, and flattens nested maps such as `middleware[].config` into strongly typed options.
- Enforce policy files inside the normalizer: resolve the configured search order, merge overrides, and reject configurations immediately when `enforceStrictMode` is enabled. Production code MUST NOT ship with placeholder policy loaders.
- Cover the normalizer with table-driven tests that assert every field round-trips correctly, including zero values, enum casing, middleware ordering, throttle metadata, and sink-specific options.
- Validate hydrated configs and emitted events against the Crucible schemas (`logger-config`, `log-event`) during tests using `gojsonschema` (or equivalent). Treat schema validation failures as build breakers.
- Keep normalization logic deterministic and side-effect free so other language foundations can mirror the behaviour.

---

## 4. Concurrency and Performance

### 4.1 Goroutine Management

Use proper synchronization for concurrent operations:

```go
// ✅ Proper goroutine management
func runConcurrentOperations(categories []Category) {
    var wg sync.WaitGroup
    results := make(chan CategoryResult, len(categories))

    for _, category := range categories {
        wg.Add(1)
        go func(cat Category) {
            defer wg.Done()
            result := runCategory(cat)
            results <- result
        }(category)
    }

    wg.Wait()
    close(results)
}
```

### 4.2 Context Handling

Always respect context cancellation:

```go
func Process(ctx context.Context, target string, config Config) (*Result, error) {
    // Check context early
    select {
    case <-ctx.Done():
        return nil, ctx.Err()
    default:
    }

    // Long-running operations should check context periodically
    for _, file := range files {
        select {
        case <-ctx.Done():
            return nil, ctx.Err()
        default:
            // Process file
        }
    }
}
```

---

## 5. Testing Standards

### 5.1 Test Organization

```
internal/core/
├── processor.go
├── processor_test.go    # Unit tests
└── testdata/            # Test fixtures
    └── fixtures/
        ├── clean/
        └── invalid/
```

### 5.2 Table-Driven Tests

Use table-driven tests:

```go
func TestProcessor_Process(t *testing.T) {
    tests := []struct {
        name           string
        input          string
        expectedIssues int
        expectedSuccess bool
    }{
        {"valid_input", "clean", 0, true},
        {"invalid_input", "invalid", 1, false},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            processor := NewProcessor()
            result, err := processor.Process(ctx, tt.input, config)

            assert.NoError(t, err)
            assert.Equal(t, tt.expectedSuccess, result.Success)
            assert.Len(t, result.Issues, tt.expectedIssues)
        })
    }
}
```

### 5.3 Test Data Management

Use `testdata/` directories for fixtures:

```go
func setupTestData(t *testing.T, state string) string {
    t.Helper()
    testDir := filepath.Join("testdata", "fixtures", state)
    // Setup test data
    return testDir
}
```

### 5.4 Schema Contract Fixtures & Golden Events

- Maintain shared fixtures under `internal/logger/testdata/` (or similar) that include schema-valid configurations for each profile, sink type, and middleware chain. Commit these fixtures and reference their Crucible schema version.
- Wire CI tests that load every fixture through the normalizer, instantiate middleware via the registry, emit a sample event, and validate both config and output JSON against the schemas. Snapshot the resulting payloads so regressions surface immediately.
- Exercise policy enforcement with table-driven tests that cover allow/deny cases, missing policies, and strict-mode failures.
- Coordinate fixture updates with other language foundations to keep cross-language parity auditable.

---

## 6. Security and Validation

### 6.1 Input Validation

Validate all inputs, especially file paths:

```go
func validateTarget(target string) error {
    // Ensure target is within expected bounds
    absTarget, err := filepath.Abs(target)
    if err != nil {
        return fmt.Errorf("invalid target path: %w", err)
    }

    // Check for path traversal
    if strings.Contains(absTarget, "..") {
        return errors.New("path traversal detected")
    }

    return nil
}
```

### 6.2 File Operations

Use secure file operations with proper permissions:

```go
// ✅ Secure file operations
func writeConfigFile(filename string, data []byte) error {
    // Create with restrictive permissions
    return os.WriteFile(filename, data, 0640)
}
```

---

## 7. Best Practices

### 7.1 Issue Creation

Create consistent, actionable issues:

```go
func createIssue(file string, severity Severity, message string, category Category) Issue {
    return Issue{
        File:        file,
        Line:        0, // Set if specific line
        Severity:    severity,
        Message:     message,
        Category:    category,
        AutoFixable: false, // Set true if fixable
    }
}
```

### 7.2 Metrics Collection

Include useful metrics for reporting:

```go
metrics := map[string]interface{}{
    "items_checked":    itemCount,
    "issues_found":     len(issues),
    "execution_time_ms": time.Since(startTime).Milliseconds(),
}

// Add category-specific metrics
if specificData != nil {
    metrics["specific_metric"] = specificData
}
```

### 7.3 Error Recovery

Handle errors gracefully:

```go
func Process(ctx context.Context, target string, config Config) (*Result, error) {
    defer func() {
        if r := recover(); r != nil {
            logger.Error("Process panic recovered: %v", r)
        }
    }()

    // Logic with proper error handling
}
```

---

## 8. Common Anti-Patterns to Avoid

**See also**: [HTTP Server Patterns Guide](../../guides/testing/http-server-patterns.md) for HTTP-specific anti-patterns derived from rampart/gauntlet fixture development (json.Encoder handling, response body close, context cancellation in delay handlers).

### 8.1 Output Contamination

```go
// ❌ NEVER: Contaminates structured output
fmt.Printf("DEBUG: Processing %s\n", filename)

// ✅ ALWAYS: Use logger
logger.Debug("processing file", zap.String("filename", filename))
```

### 8.2 Hardcoded Values

```go
// ❌ Bad: Hardcoded paths
configPath := "/home/user/.config/app.yaml"

// ✅ Good: Dynamic paths
configPath := filepath.Join(homeDir, ".config", "app.yaml")
```

### 8.3 Ignored Errors

```go
// ❌ Bad: Ignored error
file, _ := os.Open(filename)

// ✅ Good: Proper error handling
file, err := os.Open(filename)
if err != nil {
    return fmt.Errorf("failed to open file %s: %w", filename, err)
}
defer file.Close()
```

---

## 9. Code Review Checklist

Before submitting code, verify:

- [ ] No direct output writes in core logic
- [ ] All errors are properly handled and wrapped
- [ ] Logger is used for all debug/info/error output
- [ ] Tests cover happy path and error conditions
- [ ] Interfaces are implemented correctly
- [ ] Success flag logic is correct
- [ ] Context cancellation is respected
- [ ] File operations use proper permissions
- [ ] No hardcoded paths or values

---

## 10. Tools and Enforcement

### 10.1 Required Tools

- `golangci-lint` with appropriate configuration
- `go fmt` for consistent formatting
- `go vet` for static analysis

### 10.2 Pre-commit Hooks

Use appropriate hooks to enforce standards.

### 10.3 CI Integration

Ensure CI pipelines check for output contamination:

```bash
# Check for forbidden patterns
if grep -r "fmt\.Print" internal/; then
    echo "ERROR: fmt.Print* found in core code"
    exit 1
fi
```

---

## 11. Portable Testing

Fulmen Go projects must keep `go test ./...` deterministic across laptops, CI, and sandboxed environments. Follow the cross-language [Portable Testing Practices](../testing/portable-testing-practices.md) and adopt these Go-specific patterns:

- Bind sockets with `net.Listen("127.0.0.1:0")` (or `tcp4`) and inject listeners into `httptest.Server` to avoid IPv6-only or privileged port failures.
- Seed randomness deterministically (`rand.New(rand.NewSource(seed))`) so fixtures are reproducible.
- Prefer gofulmen's in-memory telemetry emitter for unit tests; guard real exporters behind capability checks (e.g., a shared `RequireNetwork(t *testing.T)` helper that skips with a clear message).
- Use `t.Cleanup` to close servers, goroutines, and temp dirs to keep tests isolated.
- Provide shared skip helpers (network, DNS, filesystem) so skips are explicit and actionable.

**CLI Testing**: For Cobra-based CLIs (e.g., goneat, workhorse forges), follow the Cobra command isolation pattern in [Language-Specific Testing Patterns](../testing/language-testing-patterns.md#go--cobra-command-isolation) to prevent state pollution between tests.

---

## Conclusion

These standards ensure FulmenHQ Go projects maintain reliability as production-grade tools. The emphasis on output hygiene is critical for structured data integrity and integration with automated systems.

**Remember**: Clean output = Clean data = Happy automation.

_Adherence to these standards ensures enterprise-grade reliability and seamless integration across development workflows._
