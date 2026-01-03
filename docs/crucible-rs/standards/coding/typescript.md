---
title: "TypeScript Coding Standards for FulmenHQ"
description: "TypeScript-specific coding standards including type safety, error handling, testing patterns, and logger initialization for enterprise-grade TypeScript development"
author: "Code Scout"
date: "2025-10-01"
last_updated: "2025-10-08"
status: "approved"
tags: ["standards", "coding", "typescript", "type-safety", "testing"]
related_docs: ["README.md"]
---

# TypeScript Coding Standards for FulmenHQ

## Overview

This document establishes coding standards for FulmenHQ TypeScript projects, ensuring consistency, quality, and adherence to enterprise-grade practices. As tools designed for scale, FulmenHQ projects require rigorous standards to maintain reliability and structured output integrity.

**Core Principle**: Write idiomatic TypeScript code that is simple, readable, and maintainable, with strict type safety and clean output.

**Foundation**: This guide builds upon **[Cross-Language Coding Standards](README.md)** which establishes patterns for:

- Output hygiene (STDERR for logs, STDOUT for data)
- RFC3339 timestamps
- Schema validation with goneat
- CLI exit codes
- Logging standards
- Security practices

Read the cross-language standards first, then apply the TypeScript-specific patterns below.

---

## 1. Critical Rules (Zero-Tolerance)

### 1.1 Logger Initialization - NEVER at Module Level

```typescript
// ❌ WRONG - Will crash bundled binaries
import { getLogger } from "../shared/pino-logger.js";
const logger = getLogger("my-module"); // 💥 BOOM

// ✅ CORRECT - Lazy initialization
import { getLogger } from "../shared/pino-logger.js";

let logger: ReturnType<typeof getLogger> | null = null;
function ensureLogger() {
  if (!logger) {
    logger = getLogger("my-module");
  }
  return logger;
}
```

**Why Critical**: Module-level code runs during bundling. Logger registry isn't initialized yet. Your binary crashes.

### 1.2 Database Types - Use InValue

```typescript
// ❌ WRONG - TypeScript will complain
async execute(sql: string, params?: any[]): Promise<ResultSet>
async execute(sql: string, params?: unknown[]): Promise<ResultSet>

// ✅ CORRECT - Use libSQL types
import type { InValue } from "@libsql/client";
async execute(sql: string, params?: InValue[]): Promise<ResultSet>
```

**Why Critical**: libSQL expects specific types. `InValue = null | string | number | bigint | ArrayBuffer | boolean | Uint8Array | Date`

### 1.3 Database Results - Always Use Bracket Notation

```typescript
// ❌ WRONG - Will fail at runtime
const result = await db.execute("SELECT COUNT(*) as count FROM users");
const count = result.rows[0].count; // 💥 TypeScript error

// ✅ CORRECT - Bracket notation
const count = result.rows[0]?.["count"] as number;
```

**Why Critical**: libSQL returns results that require bracket notation for field access.

---

## 2. Code Organization and Structure

### 2.1 Monorepo TypeScript Configuration

#### Base Configuration

Use a shared base TypeScript configuration for consistent settings across packages:

```json
{
  "$schema": "https://json.schemastore.org/tsconfig",
  "compilerOptions": {
    "esModuleInterop": true,
    "skipLibCheck": true,
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "resolveJsonModule": true,
    "allowJs": true,
    "strict": true,
    "noEmit": true,
    "isolatedModules": true,
    "incremental": true,
    "baseUrl": ".",
    "paths": {
      "@fulmenhq/*": ["packages/*/src"]
    }
  }
}
```

#### Package Configuration

Keep package-specific TypeScript configurations minimal:

```json
{
  "extends": "@fulmenhq/typescript-config",
  "include": ["src"],
  "exclude": ["node_modules", "dist"],
  "compilerOptions": {
    "types": ["bun-types"]
  }
}
```

### 2.2 Module and Object Commenting

#### JSDoc Guidelines

1. Use JSDoc comments for all public APIs:

```typescript
/**
 * Base interface for orderable status values
 * @template T - The string literal type for status values
 */
interface OrderedStatus<T extends string> {
  readonly value: T;
  readonly order: number;
}
```

2. Include code examples using @example tags:

````typescript
/**
 * Compare two status values
 * @param a - First status value
 * @param b - Second status value
 * @returns Negative if a < b, 0 if equal, positive if a > b
 *
 * @example
 * ```typescript
 * const status = RegistryStatusEnum.instance;
 * if (status.compare(RegistryStatus.Beta, RegistryStatus.Alpha) > 0) {
 *   // Beta is more advanced than Alpha
 * }
 * ```
 */
compare(a: T, b: T): number;
````

### 2.3 Import Organization

```typescript
// ❌ WRONG - Mixed up imports
import { User } from "./types";
import { readFile } from "node:fs/promises";
import { z } from "zod";

// ✅ CORRECT - Node → Third-party → Local (with blank lines)
import { readFile } from "node:fs/promises";

import { z } from "zod";

import type { User } from "./types";
```

### 2.4 Type Imports - Be Explicit

```typescript
// ❌ WRONG - Value import for types
import { User, Config } from "./types";

// ✅ CORRECT - Type imports
import type { User, Config } from "./types";

// ✅ CORRECT - Mixed imports
import { createUser, type User } from "./user-service";
```

---

## 3. Type Safety and Patterns

### 3.1 No `any` Types - Ever

```typescript
// ❌ WRONG - The forbidden type
function processData(data: any): any {}
const config: Record<string, any> = {};

// ✅ CORRECT - Be specific or use unknown
function processData(data: unknown): string {}
interface Config {
  port: number;
  host: string;
}
```

### 3.2 Promise Return Types

```typescript
// ❌ WRONG - void in Promise union
async function maybeReturn(): Promise<Response | void> {}

// ✅ CORRECT - undefined in Promise union
async function maybeReturn(): Promise<Response | undefined> {
  return undefined; // Explicit return
}
```

### 3.3 Environment Variables - Bracket Notation

```typescript
// ❌ WRONG - Dot notation
const port = process.env.PORT;
const apiKey = process.env.API_KEY;

// ✅ CORRECT - Always brackets
const port = process.env["PORT"];
const apiKey = process.env["API_KEY"];
```

### 3.4 String Literals vs Templates

```typescript
// ❌ WRONG - Unnecessary backticks
const message = `Hello world`;
const url = `https://api.example.com`;

// ✅ CORRECT - Double quotes for simple strings
const message = "Hello world";
const url = "https://api.example.com";

// ✅ CORRECT - Backticks ONLY for templates
const greeting = `Hello, ${name}!`;
const apiUrl = `https://api.example.com/users/${userId}`;
```

### 3.5 Schema-Driven Configuration Hydration

- Centralize logger configuration mapping inside a pure helper (for example, `normalizeLoggerConfig(raw: unknown): LoggerConfig`) that converts schema-authored camelCase keys into idiomatic TypeScript casing, coerces enum values, and flattens nested objects such as `middleware[].config` before constructing typed instances.
- Load and enforce policy files inside the helper. Honour `enforceStrictMode` by throwing when a policy denies a configuration—do not leave permissive placeholders or allow silent fallbacks.
- Add unit tests that exercise every field, including optional numbers/booleans, throttle metadata, middleware ordering, and sink-specific options. Ensure zero/empty values survive serialization and round-trip through JSON.
- Validate normalized configs and emitted events against the Crucible schemas (`logger-config.schema.json`, `log-event.schema.json`) during tests using AJV (or equivalent). Treat schema validation failures as hard test failures.
- Keep the helper deterministic so fixtures shared across languages produce identical results.

---

## 4. Error Handling

### 4.1 Type Guards for Error Types

Use type guards to ensure type safety when handling errors:

```typescript
// Avoid - unsafe error name checking
if (error.name === "ValidationError") {
  handleValidationError(error); // error type is still 'unknown'
}

// Prefer - type guard for error types
function isNamedError(error: unknown, name: string): error is Error {
  return error instanceof Error && error.name === name;
}

if (isNamedError(error, "ValidationError")) {
  handleValidationError(error); // error type is now 'Error'
}
```

### 4.2 Structured Error Handling

```typescript
try {
  await doSomething();
} catch (error) {
  // Handle specific error types
  if (error instanceof ValidationError) {
    handleValidation(error);
    return;
  }

  // Handle error-like objects
  if (error && typeof error === "object" && "code" in error) {
    switch (error.code) {
      case "AUTH_FAILED":
        handleAuth();
        return;
      case "INVALID_INPUT":
        handleInvalid();
        return;
    }
  }

  // Handle unknown errors
  logger.error("Unexpected error", { error });
  throw error;
}
```

---

## 5. Testing Standards

### 5.1 Test Organization

```
test/
├── unit/                           # Unit tests
│   ├── packages/                   # Package tests
│   │   ├── core/                  # Core package tests
│   │   │   ├── logger/
│   │   │   │   ├── constants.test.ts
│   │   │   │   └── index.test.ts
│   │   │   └── parser/
│   │   └── config/               # Config package tests
│   └── apps/                      # Application tests
│       ├── cli/                  # CLI app tests
│       └── api/                  # API app tests
├── integration/                    # Integration tests
├── fixtures/                      # Test data
│   └── markdown/                 # Markdown test files
└── helpers/                      # Test utilities
    ├── common/                   # Shared utilities
    ├── cli/                     # CLI-specific helpers
    └── api/                     # API-specific helpers
```

### 5.2 Test Implementation

```typescript
import { describe, expect, test, spyOn } from "bun:test";

describe("component", () => {
  test("should perform specific action", () => {
    const result = operation();
    expect(result).toBe(expected);
  });
});
```

### 5.3 Never Use Non-Null Assertions in Tests

```typescript
// ❌ WRONG - Will break when property is undefined
expect(response.data!.items!.length).toBe(3);
expect(user.profile!.address!.city).toBe("NYC");

// ✅ CORRECT - Optional chaining
expect(response.data?.items?.length).toBe(3);
expect(user.profile?.address?.city).toBe("NYC");
```

### 5.4 Mocking and Spies

```typescript
describe("feature", () => {
  let outputs: string[] = [];
  let logSpy: ReturnType<typeof spyOn>;

  beforeEach(() => {
    outputs = [];
    logSpy = spyOn(console, "log").mockImplementation((msg) => {
      outputs.push(String(msg));
    });
  });

  afterEach(() => {
    logSpy.mockRestore();
  });
});
```

### 5.5 Schema Contract Fixtures & Golden Events

- Store canonical fixtures in `test/fixtures/logging/` covering every supported profile, sink type, and middleware combination. Tag fixtures with the Crucible schema version they target.
- Require tests to load each fixture through `normalizeLoggerConfig`, instantiate middleware via the registry, emit sample events, and validate both the hydrated config and output JSON against the schemas. Snapshot or approval tests SHOULD guard against behavioural drift.
- Exercise policy enforcement permutations (allow/deny, missing policy, strict mode) with table-driven unit tests so regressions fail fast.
- Coordinate fixture updates with other language foundations to keep the cross-language parity checklist accurate.

---

## 6. Logging and Output

### 6.1 Structured Logging

Use structured logging with metadata:

```typescript
logger.info("Operation completed", {
  component: "documentGenerator",
  timestamp: new Date().toISOString(),
  version: process.env.npm_package_version,
});
```

### 6.2 Log Levels

- `error`: Errors that prevent normal operation
- `warn`: Issues that don't stop execution but need attention
- `info`: Important state changes and operations
- `debug`: Detailed information for troubleshooting

---

## 7. Configuration Management

### 7.1 Environment Variables

Use Zod for schema validation:

```typescript
const envSchema = z.object({
  APP_ENV: z
    .enum(["development", "production", "test"])
    .default("development")
    .describe("Runtime environment"),
  LOG_LEVEL: z
    .enum(["error", "warn", "info", "debug"])
    .default("info")
    .describe("Logging verbosity level"),
});
```

### 7.2 Version Management

```typescript
// version.ts
export const version = {
  major: 0,
  minor: 1,
  patch: 0,
  toString: () => "0.1.0",
};
```

---

## 8. Iteration Patterns

### 8.1 Prefer `for...of` Over `forEach`

```typescript
// Avoid
items.forEach((item) => {
  processItem(item);
});

// Prefer
for (const item of items) {
  processItem(item);
}
```

### 8.2 Optional Chaining and Nullish Coalescing

```typescript
// Avoid
const line =
  node.position && node.position.start ? node.position.start.line : 0;

// Prefer
const line = node.position?.start?.line ?? 0;
```

---

## 9. Regular Expression Patterns

#### String Literal Regex Patterns

```typescript
// ✅ CORRECT - Double backslashes for string literal regex
export const ANSI_COLOR_REGEX =
  /(?:\\x1b\[[0-9;]*m|[\u{1F300}-\u{1F9FF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}])/gu;
```

#### Testing String Literal Patterns

```typescript
describe("ANSI_COLOR_REGEX", () => {
  test("should match ANSI escape sequences", () => {
    const input = "\x1b[32mHello\x1b[0m";
    expect(input.replace(ANSI_COLOR_REGEX, "")).toBe("Hello");
  });
});
```

---

## 10. Best Practices Summary

1. **Type Safety**
   - Always check source enums and types before implementation
   - Use type guards to narrow types safely
   - Avoid type assertions except in specific, documented cases
   - Leverage const assertions for literal types

2. **Error Handling**
   - Handle errors in a structured way with proper type checking
   - Use type guards for error types
   - Include contextual information in error logs

3. **Testing**
   - Use table-driven tests for consistency
   - Mock external dependencies consistently
   - Use optional chaining in test assertions
   - Organize tests with clear structure

4. **Code Organization**
   - Keep related functionality in dedicated modules
   - Export types and interfaces that other modules depend on
   - Use barrel files (`index.ts`) to simplify imports
   - Prefer named exports over default exports

5. **Performance**
   - Use `for...of` loops for better performance and debugging
   - Handle async operations properly
   - Use optional chaining and nullish coalescing for cleaner code

---

## 11. Portable Testing

Fulmen TypeScript projects must keep Jest/Vitest suites deterministic across laptops, CI, and sandboxed environments. Follow the cross-language [Portable Testing Practices](../testing/portable-testing-practices.md) and apply these TypeScript patterns:

- Use `get-port` (or Bun's `Server.serverHTTP`) to allocate ephemeral ports; never bind to fixed privileged ports.
- Clean up mocked timers (`jest.useRealTimers()`, `vi.useRealTimers()`) and HTTP servers in `afterEach` to prevent cross-test interference.
- Seed randomness via deterministic helpers or inject seeds through environment variables for reproducible snapshots.
- Provide shared skip helpers or environment flags (e.g., `SKIP_NETWORK_TESTS`) so integration tests can skip gracefully when capabilities are missing.
- Prefer in-memory mocks for telemetry/logging; guard real network/file integrations behind explicit suites or tags.

**CLI Testing**: For Commander/oclif-based CLIs (e.g., codex forges, tsfulmen tools), follow the factory pattern in [Language-Specific Testing Patterns](../testing/language-testing-patterns.md#typescript--commander--oclif-isolation) to build fresh command instances per test.

---

## Conclusion

These standards ensure FulmenHQ TypeScript projects maintain reliability as production-grade tools. The emphasis on type safety and structured error handling is critical for maintaining code quality and preventing runtime errors.

**Remember**: Type safety prevents bugs before they happen. Follow these patterns consistently.
