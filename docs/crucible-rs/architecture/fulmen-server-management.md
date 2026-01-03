# Fulmen Server Management – Architecture Guide

**Version:** v1.0.0
**Date:** 2025-11-03
**Status:** Active Standard
**Related Standards:** [Foundry Library](../standards/library/foundry/README.md), [Exit Codes](../standards/fulmen/exit-codes/README.md)

---

## Introduction

This guide defines the Fulmen standard for orchestrating local development, testing, and preview servers with predictable ports, health checks, and cleanup routines. Originally developed for Forge Codex Pulsar, this pattern is now a first-class Fulmen standard applicable to all web templates and applications.

**Scope**: Applies to web forges, services with local dev servers, and applications requiring multiple server configurations (dev, test, a11y, preview, prod-like).

**Principles**:

- **Predictable Ports**: Named configuration classes with defined port ranges prevent conflicts
- **Health Checks**: Standardized readiness checks ensure servers are fully operational before proceeding
- **Environment Overrides**: Allow CI/CD and local customization via environment variables
- **Clean Shutdown**: PID tracking and cleanup routines ensure graceful termination
- **Exit Code Alignment**: Server failures map to Foundry exit codes for consistent error handling

---

## Configuration Classes

Server configurations are organized into **named classes**, each with a specific purpose:

### Standard Classes

| Class       | Purpose                                     | Default Port Range | Health Check Path |
| ----------- | ------------------------------------------- | ------------------ | ----------------- |
| `dev`       | Local development with hot reload           | 4321-4322          | `/`               |
| `test`      | Test/CI environments                        | 4380-4389          | `/health`         |
| `a11y`      | Accessibility testing (multiple instances)  | 4323-4340          | `/`               |
| `preview`   | Preview/staging builds                      | 4341-4350          | `/health`         |
| `prod_like` | Production-like verification before release | 4351-4360          | `/health`         |

### Custom Classes

Applications can define additional configurations using the `x-` prefix:

```yaml
additionalConfigurations:
  x-storybook:
    preferredPort: 6006
    range:
      min: 6006
      max: 6010
    # ... rest of configuration
```

**Naming Convention**: Custom classes must match `^x-[a-z0-9][a-z0-9_-]*$` to avoid conflicts with future standard classes.

---

## Schema Structure

**Schema Location:** `schem../protocol/management/v1.0.0/server-management.schema.json`
**Default Config:** `conf../protocol/management/server-management.yaml`

### Core Fields

#### envPrefix (string)

Environment variable prefix derived from application identity:

```yaml
envPrefix: FULMEN_PULSAR
```

Used to construct environment variable names: `${envPrefix}_${CONFIG_CLASS}_${SETTING}`

**Pattern**: `^[A-Z][A-Z0-9_]*$`
**Default**: `FULMEN_APP`

#### configurations (object)

Required standard configurations (dev, test, a11y, preview, prod_like):

```yaml
configurations:
  dev:
    preferredPort: 4321
    range:
      min: 4321
      max: 4322
    healthCheck:
      method: GET
      path: /
      timeout: 5000
      retries: 3
      interval: 1000
    exitBehavior:
      portInUse: 11
      healthCheckFailed: 50
      startupTimeout: 52
    envOverrides:
      - FULMEN_APP_DEV_PORT
      - FULMEN_APP_DEV_RANGE_MIN
      - FULMEN_APP_DEV_RANGE_MAX
    pidFile: .server/dev.pid
    logFile: .server/dev.log
```

### Server Configuration Fields

#### preferredPort (integer, required)

Preferred port number within the defined range. If unavailable, server should try other ports in range.

- **Range**: 1024-65535
- **Must be within**: `range.min` ≤ `preferredPort` ≤ `range.max`

#### range (object, required)

Allowed port range for this configuration:

```yaml
range:
  min: 4321
  max: 4322
```

- **min/max**: 1024-65535
- **Validation**: `min` ≤ `max`

#### healthCheck (object, required)

Health check configuration for verifying server readiness:

```yaml
healthCheck:
  method: GET # HTTP method (GET, HEAD, POST)
  path: /health # Endpoint path (must start with /)
  timeout: 5000 # Timeout in milliseconds (100-30000)
  retries: 3 # Retry attempts (0-10)
  interval: 1000 # Interval between retries in ms (100-10000)
```

**Required**: `path`
**Defaults**: `method: GET`, `timeout: 5000`, `retries: 3`, `interval: 1000`

#### exitBehavior (object, optional)

Exit code mappings for common server failure scenarios:

```yaml
exitBehavior:
  portInUse: 11 # EXIT_PORT_IN_USE (Foundry)
  healthCheckFailed: 50 # EXIT_HEALTH_CHECK_FAILED (Foundry)
  startupTimeout: 52 # EXIT_TIMEOUT (Foundry)
```

Maps to [Foundry Exit Codes](../standards/fulmen/exit-codes/README.md) for consistent error handling across Fulmen applications.

#### envOverrides (array, optional)

Environment variables that can override configuration settings:

```yaml
envOverrides:
  - FULMEN_APP_DEV_PORT
  - FULMEN_APP_DEV_RANGE_MIN
  - FULMEN_APP_DEV_RANGE_MAX
```

**Pattern**: `^[A-Z][A-Z0-9_]*$` (uppercase with underscores)
**Unique**: No duplicate variable names

#### pidFile (string, optional)

Path to PID file for process tracking (relative to project root):

```yaml
pidFile: .server/dev.pid
```

**Pattern**: `^[^/].*\.pid$` (relative path ending in .pid)

#### logFile (string, optional)

Path to log file for server output (relative to project root):

```yaml
logFile: .server/dev.log
```

---

## Environment Variable Pattern

Environment variables follow a consistent naming strategy:

```
${envPrefix}_${CONFIG_CLASS}_${SETTING}
```

### Examples

For `envPrefix: FULMEN_PULSAR` and `dev` configuration:

```bash
FULMEN_PULSAR_DEV_PORT=4321
FULMEN_PULSAR_DEV_RANGE_MIN=4321
FULMEN_PULSAR_DEV_RANGE_MAX=4322
FULMEN_PULSAR_DEV_HEALTH_PATH=/
FULMEN_PULSAR_DEV_HEALTH_TIMEOUT=5000
```

For `a11y` configuration:

```bash
FULMEN_PULSAR_A11Y_PORT=4325
FULMEN_PULSAR_A11Y_RANGE_MIN=4323
FULMEN_PULSAR_A11Y_RANGE_MAX=4340
```

### Deriving envPrefix

The `envPrefix` should be derived from application identity:

```typescript
// Future (helper library v0.2.4+)
import { getAppIdentity } from "tsfulmen/appidentity";

const identity = getAppIdentity();
const envPrefix = `FULMEN_${identity.slug.toUpperCase()}`;
// → FULMEN_PULSAR, FULMEN_GRONINGEN, etc.
```

**Current Approach** (until appidentity module lands):

```typescript
// Explicit in config
const config = {
  envPrefix: "FULMEN_PULSAR",
  // ...
};
```

---

## Makefile Targets

Templates implementing server functionality **MUST** expose standardized server orchestration targets. These targets enable consistent local development, testing, and preview workflows across all Fulmen server implementations.

**See**: [Makefile Standard - Annex A: Server Orchestration Targets](../standards/makefile-standard.md#annex-a-server-orchestration-targets) for:

- Required target specifications (`server-start-%`, `server-stop-%`, `server-status-%`, `server-restart-%`, `server-logs-%`)
- Implementation requirements (port management, health checks, PID file handling, exit codes)
- Example Makefile implementations for TypeScript/Python/Go
- Integration points with Crucible schemas and helper libraries

**Quick Reference** - Required targets:

| Target                  | Purpose                                                                             |
| ----------------------- | ----------------------------------------------------------------------------------- |
| `make server-start-%`   | Start server in specified configuration class (dev, test, a11y, preview, prod_like) |
| `make server-stop-%`    | Stop server using graceful shutdown (SIGTERM)                                       |
| `make server-status-%`  | Check if server is running and healthy                                              |
| `make server-restart-%` | Restart server (stop + start)                                                       |
| `make server-logs-%`    | Display or tail logs                                                                |

**Example Usage**:

```bash
# Start dev server (uses dev.preferredPort from server-management.yaml)
make server-start-dev

# Check test server health
make server-status-test

# Restart preview server
make server-restart-preview
```

---

## Reference Implementation Pattern

**Division of Responsibility**: Server orchestration logic is **cross-cutting infrastructure**, not application-specific code. Helper libraries provide reference implementations to avoid copy/paste drift across templates.

### Three-Layer Model

**1. Crucible (SSOT) - Defines "What"**:

- Schema: `schemas/server/management/v1.0.0/server-management.schema.json`
- Config: `config/server/management/server-management.yaml`
- Standards: Exit codes, health check formats, configuration classes
- Requirements: Port ranges, retry logic, PID file conventions

**2. Helper Libraries - Implement "How"**:

- **TypeScript**: `@fulmenhq/tsfulmen` ships CLI harness (`tsfulmen server start --config dev`)
- **Python**: `pyfulmen` provides subprocess-based orchestration (`pyfulmen server start --config dev`)
- **Go**: `gofulmen/server` package exports orchestration functions

**Harness Responsibilities**:

- Read `server-management.yaml` from Crucible SSOT
- Resolve configuration for specified class (dev, test, a11y, preview, prod_like)
- Discover available port (check preferred, fallback to range scan)
- Start server process with environment variable overrides
- Poll health endpoint using retry logic from config
- Write PID file for process management
- Exit with standardized exit codes on failure (EXIT_PORT_IN_USE, EXIT_HEALTH_CHECK_FAILED, etc.)

**What Libraries Don't Know**: The actual server command to run (that's application-specific).

**3. Applications - Configure "What Command"**:

Applications specify their server command in configuration, which the library harness executes.

**Example (TypeScript/Bun application)**:

```yaml
# app-specific-config.yaml (or passed via CLI args)
serverCommand: "bun run dev"
envPrefix: "MY_APP"
```

**Makefile delegates to library harness**:

```makefile
.PHONY: server-start-%
server-start-%:
	@tsfulmen server start --config $* --command "bun run dev"
```

**What Happens**:

1. `tsfulmen` reads `server-management.yaml` from Crucible (synced to `node_modules/@fulmenhq/tsfulmen/config/`)
2. Resolves dev configuration (preferred port 3000, range 3000-3099, health check settings)
3. Checks if port 3000 available; if not, scans range
4. Runs `bun run dev` with `DEV_PORT=3000` environment variable
5. Polls `http://localhost:3000/health/live` until server responds with `{"status": "pass"}`
6. Writes PID to `var/run/dev.pid`
7. Exits with code 0 on success, or appropriate error code (11 for port conflict, 50 for health check failure)

### Benefits of Reference Implementations

**Centralized Maintenance**:

- Bug in port discovery logic? Fix once in library, all applications benefit
- Need to improve health check retry backoff? Update library, applications inherit improvement

**Consistent Behavior**:

- All Fulmen servers use same exit codes, same health check patterns, same PID file conventions
- DevOps teams learn orchestration once, applies across all services

**DRY Principle**:

- No copy/paste of port management logic across 50+ templates
- Reduces drift: applications can't accidentally implement incompatible variations

**Ecosystem-Idiomatic Tooling**:

- TypeScript apps get Bun-native CLI (`tsfulmen server`)
- Python apps get subprocess wrapper (`pyfulmen.server.start()`)
- Go apps get importable package (`gofulmen/server.Start()`)

### Implementation Details

See [Server Management Module Spec](../standards/library/modules/server-management.md) for:

- Required helper library interfaces
- Configuration resolution algorithm
- Port discovery implementation
- Health check retry logic
- Exit code mapping
- Language-specific examples (TypeScript, Python, Go)

---

## Makefile Targets

Templates implementing server functionality **MUST** expose standardized server orchestration targets. These targets enable consistent local development, testing, and preview workflows across all Fulmen server implementations.

**See**: [Makefile Standard - Annex A: Server Orchestration Targets](../standards/makefile-standard.md#annex-a-server-orchestration-targets) for:

- Required target specifications (`server-start-%`, `server-stop-%`, `server-status-%`, `server-restart-%`, `server-logs-%`)
- Implementation requirements (port management, health checks, PID file handling, exit codes)
- Example Makefile implementations for TypeScript/Python/Go
- Integration points with Crucible schemas and helper libraries

**Quick Reference** - Required targets:

| Target                  | Purpose                                                                             |
| ----------------------- | ----------------------------------------------------------------------------------- |
| `make server-start-%`   | Start server in specified configuration class (dev, test, a11y, preview, prod_like) |
| `make server-stop-%`    | Stop server using graceful shutdown (SIGTERM)                                       |
| `make server-status-%`  | Check if server is running and healthy                                              |
| `make server-restart-%` | Restart server (stop + start)                                                       |
| `make server-logs-%`    | Display or tail logs                                                                |

**Example Usage**:

```bash
# Start dev server (uses dev.preferredPort from server-management.yaml)
make server-start-dev

# Check test server health
make server-status-test

# Restart preview server
make server-restart-preview

# Clean all server artifacts (PIDs, logs)
.PHONY: server-clean
server-clean:
	@echo "Cleaning server artifacts..."
	rm -rf .server/
```

### Example Usage

```bash
# Start dev server
make server-start-dev

# Check status
make server-status-dev

# Stop server
make server-stop-dev

# Clean all artifacts
make server-clean
```

---

## Health Check Workflow

Health checks verify server readiness before proceeding with tests or deployment:

### Implementation Pattern

```typescript
async function waitForServer(config: ServerConfiguration): Promise<void> {
  const { healthCheck, preferredPort } = config;
  const url = `http://localhost:${preferredPort}${healthCheck.path}`;

  for (let attempt = 0; attempt <= healthCheck.retries; attempt++) {
    try {
      const response = await fetch(url, {
        method: healthCheck.method,
        signal: AbortSignal.timeout(healthCheck.timeout),
      });

      if (response.ok) {
        console.log(`Server healthy at ${url}`);
        return;
      }
    } catch (error) {
      if (attempt < healthCheck.retries) {
        await new Promise((resolve) =>
          setTimeout(resolve, healthCheck.interval),
        );
        continue;
      }

      // Final attempt failed
      console.error(`Health check failed after ${attempt + 1} attempts`);
      process.exit(config.exitBehavior.healthCheckFailed); // EXIT_HEALTH_CHECK_FAILED
    }
  }
}
```

### Failure Modes

| Condition           | Exit Code | Foundry Name             |
| ------------------- | --------- | ------------------------ |
| Port already in use | 11        | EXIT_PORT_IN_USE         |
| Health check failed | 50        | EXIT_HEALTH_CHECK_FAILED |
| Startup timeout     | 52        | EXIT_TIMEOUT             |
| Permission denied   | 13        | EXIT_PERMISSION_DENIED   |

---

## PID Registry & Cleanup

### PID File Management

**Location**: Defined in `pidFile` (e.g., `.server/dev.pid`)
**Format**: Plain text file containing process ID

```bash
# Write PID on startup
echo $$ > .server/dev.pid

# Read PID for cleanup
if [ -f .server/dev.pid ]; then
  pid=$(cat .server/dev.pid)
  kill -TERM $pid
  rm .server/dev.pid
fi
```

### Graceful Shutdown

1. **SIGTERM**: Send termination signal to process
2. **Wait**: Allow server to flush buffers, close connections (5-10s timeout)
3. **SIGKILL**: Force kill if still running after timeout
4. **Cleanup**: Remove PID file and temporary artifacts

### Cleanup Routines

**On Normal Exit**:

- Server removes own PID file
- Flushes logs
- Closes connections gracefully

**On Abnormal Exit** (SIGKILL, crash):

- Makefile target cleans stale PID files
- Logs preserved for debugging
- Port released by OS

**Pre-commit Hook**:

```bash
# Clean stale server artifacts before commit
make server-clean
```

---

## Helper Library Integration

### Current State (v0.2.3)

Schemas and defaults available via language wrappers:

```typescript
// TypeScript
import serverManagementDefaults from 'tsfulmen/conf../protocol/management/server-management.yaml';

// Python
from pyfulmen.config.server.management import load_server_management_config
config = load_server_management_config()
```

### Future (v0.2.4+)

Helper libraries will expose utilities:

```typescript
// TypeScript (tsfulmen v0.2.4+)
import { resolveServerConfig, getEnvPrefix } from "tsfulmen/server";

// Resolve configuration for 'dev' class
const devConfig = await resolveServerConfig("dev");
// → merges defaults, applies env overrides, validates

// Get environment prefix
const prefix = getEnvPrefix();
// → derives from appidentity module

// Find available port in range
const port = await findAvailablePort(devConfig.range);
```

```python
# Python (pyfulmen v0.2.4+)
from pyfulmen.server import resolve_server_config, find_available_port

# Resolve configuration
dev_config = resolve_server_config("dev")

# Find port
port = find_available_port(dev_config.range)
```

---

## Implementation Checklist

### For Templates (Forge Codex Pulsar, etc.)

- [ ] Copy `server-management.yaml` from Crucible to project config
- [ ] Update `envPrefix` to match application identity (e.g., `FULMEN_PULSAR`)
- [ ] Adjust port ranges if needed (avoid conflicts with other templates)
- [ ] Implement Makefile targets (`server-start-%`, `server-stop-%`, etc.)
- [ ] Add health check endpoints to server code
- [ ] Integrate with CI/CD (use `test` configuration class)
- [ ] Add cleanup to pre-commit hooks (`make server-clean`)

### For Helper Libraries

- [ ] Expose schema validation (`tsfulmen.schemas.serverManagement.validate(config)`)
- [ ] Implement `resolveServerConfig(configClass)` utility (v0.2.4)
- [ ] Add `findAvailablePort(range)` helper (v0.2.4)
- [ ] Integrate with appidentity module for `envPrefix` derivation (v0.2.4+)
- [ ] Document environment variable patterns in library README

---

## Schema References

| Resource       | Location                                                           |
| -------------- | ------------------------------------------------------------------ |
| Schema         | `schem../protocol/management/v1.0.0/server-management.schema.json` |
| Default Config | `conf../protocol/management/server-management.yaml`                |

**Helper Library Support**:

- **tsfulmen**: Config loading and schema validation (v0.2.3+), utilities (v0.2.4+)
- **pyfulmen**: YAML loading + validation (v0.2.3+), utilities (v0.2.4+)
- **gofulmen**: Future support for Go-based services

---

## Related Standards

- [Foundry Exit Codes](../standards/fulmen/exit-codes/README.md) – Server failure exit codes
- [Forge Codex Standard](fulmen-forge-codex-standard.md) – Web template requirements
- [Application Identity](../standards/fulmen/identity/README.md) – App slug derivation (coming v0.3.0)

---

**Status**: Active Standard
**Next Review**: 2025-12-01
**Maintainer**: Fulmen Architecture Team
