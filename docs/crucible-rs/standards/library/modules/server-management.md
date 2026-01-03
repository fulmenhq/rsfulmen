---
title: "Server Management Module"
description: "Utilities for orchestrating local servers with health checks, port management, and lifecycle control"
author: "Schema Cartographer"
date: "2025-11-03"
last_updated: "2025-11-03"
status: "draft"
tags:
  [
    "standards",
    "library",
    "server-management",
    "health-checks",
    "port-management",
    "v0.2.3",
  ]
---

# Server Management Module

## Purpose

Provide consistent server orchestration utilities across Fulmen helper libraries (gofulmen, pyfulmen, tsfulmen) for local development, testing, and preview environments. Enables templates (Forge Codex Pulsar, future web forges) to manage multiple server configurations (dev, test, a11y, preview, prod-like) with predictable ports, health checks, and cleanup routines.

**Status**: Schema and configuration SSOT established in v0.2.3. Helper library implementations planned for v0.2.4+.

## Implementation Strategy

**Reference Harness Pattern**: Helper libraries ship **configurable orchestration harnesses** that handle cross-cutting infrastructure (port discovery, health checks, PID files, exit codes) while accepting application-specific server commands as configuration.

### Division of Responsibility

**Crucible (SSOT)**:

- Defines configuration schema (`schemas/server/management/v1.0.0/`)
- Provides default configuration (`config/server/management/server-management.yaml`)
- Specifies exit codes, health check formats, requirements

**Helper Libraries**:

- Implement orchestration harness (CLI tool or importable package)
- Read Crucible config from embedded SSOT
- Handle port management, health checks, PID files
- Exit with standardized codes

**Applications**:

- Specify their server command (e.g., `bun run dev`, `uvicorn app:app`)
- Wire command into library harness via Makefile or config
- Inherit consistent orchestration behavior

### Harness Implementation Requirements

Each helper library MUST provide:

1. **CLI Tool or Package**: Ecosystem-appropriate interface
   - TypeScript: `tsfulmen server` CLI (Bun-native)
   - Python: `pyfulmen server` CLI or `pyfulmen.server` module
   - Go: `gofulmen/server` importable package
2. **Configuration Resolution**: Read and merge Crucible config with environment overrides
3. **Port Discovery**: Check preferred port, scan range if unavailable
4. **Process Management**: Start command, write PID, handle signals
5. **Health Checks**: Poll endpoint with retry logic from config
6. **Exit Code Compliance**: Use Foundry exit codes (EXIT_PORT_IN_USE=11, EXIT_HEALTH_CHECK_FAILED=50, etc.)

### Example: TypeScript Harness

**Library provides** (`@fulmenhq/tsfulmen`):

```typescript
// bin/tsfulmen-server.ts
import {
  resolveServerConfig,
  findAvailablePort,
  waitForHealth,
} from "./server";
import { spawnServer } from "./process";

async function start(configClass: string, command: string) {
  const config = await resolveServerConfig(configClass);
  const port = await findAvailablePort(config.range, config.preferredPort);

  if (!port) {
    process.exit(11); // EXIT_PORT_IN_USE
  }

  const envVars = {
    [`${config.envPrefix}_${configClass.toUpperCase()}_PORT`]: port,
  };
  const pid = await spawnServer(command, envVars, config.pidFile);

  const healthy = await waitForHealth(config.healthCheck, port);
  if (!healthy) {
    process.exit(50); // EXIT_HEALTH_CHECK_FAILED
  }

  console.log(`Server started (PID ${pid}) on port ${port}`);
}
```

**Application uses** (Forge Codex Pulsar):

```makefile
.PHONY: server-start-%
server-start-%:
	@tsfulmen server start --config $* --command "bun run dev"
```

**What happens**:

- `tsfulmen` handles: config resolution, port discovery, health checks, PID files
- Application specifies: the actual command (`bun run dev`)
- Result: Consistent orchestration, no copy/paste logic

### Benefits

**DRY Principle**: Port discovery logic written once per ecosystem, not per application

**Centralized Fixes**: Bug in health check retry? Fix library, all apps benefit

**Consistent Behavior**: All Fulmen servers use same exit codes, PID conventions, health patterns

See [Server Management Architecture](../../../architecture/fulmen-server-management.md#reference-implementation-pattern) for high-level overview and application examples.

## Core Capabilities (Planned v0.2.4)

### 1. Configuration Resolution

Load and merge server configurations from Crucible SSOT with environment variable overrides.

**Go**:

```go
import "github.com/fulmenhq/gofulmen/server"

config, err := server.ResolveConfig("dev")
// Returns ServerConfig with preferredPort, range, healthCheck, etc.
// Honors FULMEN_APP_DEV_PORT, FULMEN_APP_DEV_RANGE_MIN env vars
```

**Python**:

```python
from pyfulmen.server import resolve_config

config = resolve_config("dev")
# Merges defaults from Crucible with environment overrides
print(config.preferred_port)  # 4321 or env override
```

**TypeScript**:

```typescript
import { resolveServerConfig } from "@fulmenhq/tsfulmen/server";

const config = await resolveServerConfig("dev");
// Returns ServerConfiguration with environment overrides applied
console.log(config.preferredPort); // 4321 or env override
```

### 2. Port Management

Discover available ports within configured ranges, honoring preferred ports when available.

**Go**:

```go
port, err := server.FindAvailablePort(config.Range)
// Tries preferredPort first, then scans [min, max]
```

**Python**:

```python
from pyfulmen.server import find_available_port

port = find_available_port(config.range)
# Returns first available port in range
```

**TypeScript**:

```typescript
import { findAvailablePort } from "@fulmenhq/tsfulmen/server";

const port = await findAvailablePort(config.range);
// Scans range for available port
```

### 3. Health Check Utilities

Wait for server readiness using retry logic and standardized health endpoints.

**Go**:

```go
err := server.WaitForServer(config)
// Polls health endpoint with retry/timeout from config
// Uses Foundry HTTP status codes for validation
```

**Python**:

```python
from pyfulmen.server import wait_for_server

await wait_for_server(config)
# Async polling with configurable timeout/retries
```

**TypeScript**:

```typescript
import { waitForServer } from "@fulmenhq/tsfulmen/server";

await waitForServer(config);
// Polls config.healthCheck.path with retry logic
// Throws on timeout or health check failure
```

**Health Check Implementation**:

Helper libraries SHOULD use the standardized health response schema for validation:

- **Schema**: `schemas/protocol/http/v1.0.0/health-response.schema.json`
- **Expected Response**: `{ "status": "pass" | "warn" | "fail", "service": "...", ... }`
- **HTTP Status Codes**: Use Foundry catalog `config/library/foundry/http-statuses.yaml` (200 OK for pass, 503 Service Unavailable for fail)
- **Standard**: See [HTTP REST Standard](../../protocol/http-rest-standards.md) for complete health endpoint conventions

### 4. Environment Variable Handling

Derive environment variable prefixes from application identity and resolve overrides.

**Go**:

```go
prefix := server.GetEnvPrefix()
// Returns "FULMEN_APP" or derives from appidentity module (v0.2.4+)

envVars := server.GetEnvOverrides("dev")
// Returns ["FULMEN_APP_DEV_PORT", "FULMEN_APP_DEV_RANGE_MIN", ...]
```

**Python**:

```python
from pyfulmen.server import get_env_prefix, get_env_overrides

prefix = get_env_prefix()
# Derives from appidentity if available

env_vars = get_env_overrides("dev")
# Returns list of env var names for dev config
```

**TypeScript**:

```typescript
import { getEnvPrefix, getEnvOverrides } from "@fulmenhq/tsfulmen/server";

const prefix = getEnvPrefix();
// Returns "FULMEN_APP" or derives from app identity

const envVars = getEnvOverrides("dev");
// Returns array of environment variable names
```

**Environment Variable Pattern**:

```
${envPrefix}_${CONFIG_CLASS}_${SETTING}
```

Examples:

```bash
FULMEN_PULSAR_DEV_PORT=4321
FULMEN_PULSAR_A11Y_RANGE_MIN=4323
FULMEN_PULSAR_A11Y_RANGE_MAX=4340
FULMEN_PULSAR_TEST_HEALTH_PATH=/health
```

See [Server Management Architecture](../../../architecture/fulmen-server-management.md#environment-variable-pattern) for complete documentation.

### 5. PID File Management

Track server processes using PID files for graceful shutdown and cleanup.

**Go**:

```go
err := server.WritePid(config.PidFile, os.Getpid())
// Writes PID to .server/dev.pid

pid, err := server.ReadPid(config.PidFile)
// Reads PID from file

err = server.CleanupPid(config.PidFile)
// Removes PID file after shutdown
```

**Python**:

```python
from pyfulmen.server import write_pid, read_pid, cleanup_pid

write_pid(config.pid_file, os.getpid())
# Writes PID to configured path

pid = read_pid(config.pid_file)
# Returns PID as integer

cleanup_pid(config.pid_file)
# Removes PID file
```

**TypeScript**:

```typescript
import { writePid, readPid, cleanupPid } from "@fulmenhq/tsfulmen/server";

await writePid(config.pidFile, process.pid);
// Writes PID to .server/dev.pid

const pid = await readPid(config.pidFile);
// Returns PID as number

await cleanupPid(config.pidFile);
// Removes PID file
```

## Configuration Classes

Helper libraries MUST support the five standard configuration classes defined in the schema:

| Class       | Purpose                                     | Default Port Range | Health Check Path |
| ----------- | ------------------------------------------- | ------------------ | ----------------- |
| `dev`       | Local development with hot reload           | 4321-4322          | `/`               |
| `test`      | Test/CI environments                        | 4380-4389          | `/health`         |
| `a11y`      | Accessibility testing (multiple instances)  | 4323-4340          | `/`               |
| `preview`   | Preview/staging builds                      | 4341-4350          | `/health`         |
| `prod_like` | Production-like verification before release | 4351-4360          | `/health`         |

**Custom Classes**: Templates MAY define additional configurations using the `x-` prefix (e.g., `x-storybook`). Helper libraries SHOULD support loading custom configurations from the `additionalConfigurations` field.

## Exit Code Alignment

Server failures SHOULD map to Foundry exit codes for consistent error handling:

| Failure Condition   | Exit Code | Foundry Name             |
| ------------------- | --------- | ------------------------ |
| Port already in use | 11        | EXIT_PORT_IN_USE         |
| Health check failed | 50        | EXIT_HEALTH_CHECK_FAILED |
| Startup timeout     | 52        | EXIT_TIMEOUT             |
| Permission denied   | 13        | EXIT_PERMISSION_DENIED   |

See [Foundry Exit Codes](../../fulmen/exit-codes/README.md) for complete catalog.

## Integration with HTTP Standards

Server management utilities SHOULD integrate with existing Foundry HTTP standards:

### Health Endpoints

- **Schema**: `schemas/protocol/http/v1.0.0/health-response.schema.json`
- **Standard Paths**:
  - `/health/live` - Liveness probe (always returns 200 if process is running)
  - `/health/ready` - Readiness probe (returns 200 when dependencies are available)
  - `/health/startup` - Startup probe (returns 200 when initialization complete)
- **Response Format**:
  ```json
  {
    "status": "pass",
    "service": "pulsar-dev",
    "timestamp": "2025-11-03T10:30:00Z",
    "uptimeSeconds": 42,
    "checks": [
      {
        "name": "database",
        "status": "pass",
        "observedAt": "2025-11-03T10:30:00Z"
      }
    ]
  }
  ```

### Version Endpoints

- **Schema**: `schemas/protocol/http/v1.0.0/version-response.schema.json`
- **Standard Path**: `/version`
- **Response Format**:
  ```json
  {
    "success": true,
    "data": {
      "version": "0.2.3",
      "commit": "dea9c66",
      "buildDate": "2025-11-03T10:00:00Z",
      "language": "typescript"
    }
  }
  ```

### HTTP Status Codes

Use Foundry catalog `config/library/foundry/http-statuses.yaml` for consistent status code handling:

- **200 OK** - Successful health check, resource available
- **503 Service Unavailable** - Health check failed, dependencies unavailable
- **500 Internal Server Error** - Unexpected server error
- **400 Bad Request** - Invalid health check parameters

See [HTTP REST Standard](../../protocol/http-rest-standards.md) for complete conventions.

## Instrumentation Patterns

Follow standard telemetry patterns for server management operations:

### Port Discovery

**Pattern**: Counter-only (performance-sensitive)

```python
from pyfulmen.server import find_available_port
from pyfulmen.metrics import counter

with counter("server.port_discovery_attempts"):
    port = find_available_port(config.range)
```

### Health Checks

**Pattern**: Histogram + Counter (latency variability)

```typescript
import { waitForServer } from "@fulmenhq/tsfulmen/server";
import { histogram, counter } from "@fulmenhq/tsfulmen/metrics";

const start = Date.now();
try {
  await waitForServer(config);
  histogram("server.health_check_duration_ms").observe(Date.now() - start);
  counter("server.health_checks_success").inc();
} catch (error) {
  counter("server.health_checks_failed").inc();
  throw error;
}
```

**Metrics**: Use existing taxonomy entries:

- `server.port_discovery_attempts` - Counter for port discovery attempts
- `server.health_check_duration_ms` - Histogram for health check latency
- `server.health_checks_success` - Counter for successful health checks
- `server.health_checks_failed` - Counter for failed health checks

See [Telemetry & Metrics](./telemetry-metrics.md) for complete instrumentation patterns.

## Schema & Configuration SSOT

### Schema Location

- **Schema**: `schem../protocol/management/v1.0.0/server-management.schema.json`
- **Version**: v1.0.0 (stable)
- **Validation**: Draft 2020-12

### Default Configuration

- **Config**: `conf../protocol/management/server-management.yaml`
- **envPrefix**: `FULMEN_APP` (override per application)
- **Configurations**: Complete defaults for all 5 standard classes

### Consuming from Helper Libraries

**Go** (embeds directly from Crucible):

```go
import "github.com/fulmenhq/crucible"

schema, err := crucible.SchemaRegistry.Server().Management()
config, err := crucible.ConfigRegistry.Server().Management()
```

**TypeScript** (synced to wrapper):

```typescript
import serverManagementSchema from "@fulmenhq/tsfulmen/schem../protocol/management/v1.0.0/server-management.schema.json";
import serverManagementDefaults from "@fulmenhq/tsfulmen/conf../protocol/management/server-management.yaml";
```

**Python** (synced to wrapper):

```python
from pyfulmen.config.server.management import load_server_management_config

config = load_server_management_config()
# Returns parsed YAML with defaults
```

See [Consuming Crucible Assets](../../../../guides/consuming-crucible-assets.md) for complete integration guide.

## Template Integration

Templates (Forge Codex Pulsar, future web forges) SHOULD:

1. **Copy Config**: Stage `server-management.yaml` from Crucible to project config
2. **Update envPrefix**: Customize to match application identity (e.g., `FULMEN_PULSAR`)
3. **Adjust Port Ranges**: Modify if needed to avoid conflicts with other templates
4. **Implement Makefile Targets**:
   - `make server-start-%` - Start server in given configuration class
   - `make server-stop-%` - Stop server for given configuration class
   - `make server-status-%` - Check server status
   - `make server-clean` - Clean all server artifacts (PIDs, logs)
5. **Add Health Check Endpoints**: Implement `/health` and `/version` endpoints per HTTP standards
6. **Integrate with CI/CD**: Use `test` configuration class for CI environments
7. **Add Cleanup to Hooks**: Include `make server-clean` in pre-commit hooks

**Example Makefile Integration**:

```makefile
.PHONY: server-start-%
server-start-%:
	@echo "Starting server in $* configuration..."
	@tsfulmen server:start --config $*

.PHONY: server-stop-%
server-stop-%:
	@echo "Stopping server in $* configuration..."
	@tsfulmen server:stop --config $*

.PHONY: server-clean
server-clean:
	@echo "Cleaning server artifacts..."
	@rm -rf .server/
```

See [Server Management Architecture](../../../architecture/fulmen-server-management.md#makefile-targets) for complete implementation guidance.

## Implementation Checklist

### For Helper Libraries (v0.2.4)

- [ ] Implement `resolveServerConfig(configClass)` utility
- [ ] Add `findAvailablePort(range)` helper
- [ ] Implement `waitForServer(config)` with retry logic
- [ ] Add `getEnvPrefix()` with appidentity integration
- [ ] Implement PID file management utilities
- [ ] Add validation against server-management schema
- [ ] Document environment variable patterns in library README
- [ ] Add examples for all configuration classes
- [ ] Integrate with telemetry module for instrumentation
- [ ] Add tests for all utilities with mocked servers

### For Templates (v0.2.4)

- [ ] Copy `server-management.yaml` from Crucible to project config
- [ ] Update `envPrefix` to match application identity
- [ ] Implement Makefile targets (`server-start-%`, `server-stop-%`, etc.)
- [ ] Add health check endpoints (`/health`, `/health/ready`, `/health/startup`)
- [ ] Add version endpoint (`/version`)
- [ ] Integrate with CI/CD (use `test` configuration class)
- [ ] Add cleanup to pre-commit hooks (`make server-clean`)
- [ ] Document server management workflow in template README

## Future Enhancements (v0.3.0+)

- **CLI Commands**: `tsfulmen server:start`, `pyfulmen server --config dev`
- **Process Management**: Daemonization, automatic restart on failure
- **Log Streaming**: Tail logs from configured `logFile` paths
- **Multi-Server Orchestration**: Manage multiple servers simultaneously (e.g., API + frontend)
- **Docker Integration**: Container lifecycle management with health checks
- **Monitoring Dashboards**: Prometheus/Grafana integration for server metrics

## Related Documentation

- **Architecture**: [Server Management Architecture](../../../architecture/fulmen-server-management.md) - Complete design and implementation guide
- **HTTP Standards**: [HTTP REST Standard](../../protocol/http-rest-standards.md) - Health endpoint conventions
- **Exit Codes**: [Foundry Exit Codes](../../fulmen/exit-codes/README.md) - Error handling catalog
- **Telemetry**: [Telemetry & Metrics](./telemetry-metrics.md) - Instrumentation patterns
- **Integration Guide**: [Consuming Crucible Assets](../../../../guides/consuming-crucible-assets.md) - SSOT consumption patterns

---

**Status**: Draft - Schema and configuration SSOT established in v0.2.3. Helper library implementations planned for v0.2.4+.
**Maintainer**: Schema Cartographer
**Next Review**: 2025-12-01
