---
title: "Fulmen Forge Workhorse Standard"
description: "Standard structure and capabilities for Fulmen Workhorse forges - production-ready templates for robust, general-purpose applications"
author: "Fulmen Enterprise Architect (@fulmen-ea-steward)"
date: "2025-10-20"
last_updated: "2025-12-18"
status: "draft"
tags: ["architecture", "forge", "workhorse", "template", "2025.10.2"]
---

# Fulmen Forge Workhorse Standard

This document defines the standardized structure and pre-integrated capabilities for Fulmen Workhorse forges. Workhorse forges provide production-ready templates for robust, general-purpose applications (e.g., servers, workers, long-running processes) that require reliable tooling out-of-the-box. They embody the CRDL philosophy (Clone → Degit → Refit → Launch) and align with the repository category taxonomy (`workhorse` key from [category-key.schema.json](schemas/taxonomy/repository-category/v1.0.0/category-key.schema.json)).

Workhorse forges are distinguished from other categories (e.g., `cli` for command-line tools, `service` for microservices) by their focus on durable, scalable backends with emphasis on observability, config management, and error resilience. Canonical implementations use horse breed names (`groningen` for Go, `percheron` for Python) to identify language-specific variants while maintaining consistent standards across the ecosystem.

The canonical list of forge categories and statuses is maintained in the [Repository Category Taxonomy](schemas/taxonomy/repository-category/v1.0.0/README.md); consult that before proposing new forges or changing lifecycle states.

## Scope

Applies to Workhorse-specific forge templates (e.g., `forge-workhorse-groningen`, `forge-workhorse-percheron`). Workhorse forges use horse breed names (e.g., groningen, percheron, clydesdale) as distinctive identifiers, with binaries named using the breed name only (not `workhorse-{breed}`). Excludes other categories (e.g., `cli` for interactive tools, `library` for reusable code). Forges are not SSOT repos or full applications but starters that integrate Fulmen ecosystem components (Crucible via helpers, goneat optional) to accelerate development while enforcing standards.

Core philosophy: Ship "batteries-included" templates that handle 80% of boilerplate (logging, config, telemetry, bootstrap) so users focus on business logic. No "useful" functionality (e.g., no domain-specific code); just scalable foundations.

**CDRL Guide**: Users follow Clone → Degit → Refit → Launch; see `docs/development/fulmen_cdrl_guide.md` for details. During refit, users rename the breed identifier (e.g., `groningen` → `myapi`) throughout the codebase.

Implementers MUST comply with ecosystem standards in Crucible's `docs/standards/` (e.g., coding conventions, API patterns, repository structure) to ensure consistency.

## Required Library Modules

Workhorse forges MUST integrate these Fulmen helper library modules to ensure ecosystem compliance. All modules are accessed via the language-specific helper library (e.g., gofulmen, pyfulmen, tsfulmen) - no direct Crucible dependencies. Each module reference includes the compliance requirement (REQUIRED vs RECOMMENDED) and links to canonical specifications.

### Core Identity & Configuration Modules

1. **App Identity Module** (REQUIRED)
   - **Purpose**: Standardized application metadata (binary name, vendor, environment prefix)
   - **Spec**: [App Identity Module](../standards/library/modules/app-identity.md)
   - **Compliance**: MUST implement `.fulmen/app.yaml` with:
     - `binary_name`: Breed name for template (users rename during CDRL refit)
     - `vendor`: Default `fulmenhq` (users customize)
     - `env_prefix`: Environment variable prefix (e.g., `PERCHERON_`)
     - `config_name`: Config directory name (usually same as binary_name)
   - **Helper API**: `app_identity.load()` → AppIdentity object
   - **Addresses Gap**: Binary name parameterization (see percheron workhorse-standard-gaps.md)
   - **CDRL Workflow**: Users update `.fulmen/app.yaml` FIRST, then run `make validate-app-identity` to find hardcoded references
   - **Distributed artifact requirement**: Workhorse forges MUST embed identity for standalone binaries (see [App Identity Module](../standards/library/modules/app-identity.md))
   - **Required Make targets**: forges MUST provide `make sync-embedded-identity` and `make verify-embedded-identity` (see [Fulmen Template CDRL Standard](fulmen-template-cdrl-standard.md))

2. **Crucible Shim Module** (REQUIRED)
   - **Purpose**: Access Crucible SSOT assets (schemas, standards, documentation, configs, taxonomies) without direct sync
   - **Spec**: [Crucible Shim](../standards/library/modules/crucible-shim.md)
   - **Compliance**:
     - **Schemas**: `crucible.GetSchema()`, `crucible.GetLoggingEventSchema()`
     - **Documentation**: `crucible.GetDoc("standards/observability/logging.md")`
     - **Configs**: `crucible.GetConfig()`
     - **Standards**: `crucible.GetGoStandards()`, `crucible.GetTypeScriptStandards()`
   - **Exposure**: `/version` endpoint MUST include Crucible version from `crucible.GetVersion()`
   - **Documentation access**: Workhorses may need to access standards for runtime compliance validation, API documentation generation, or operational playbooks. See [Crucible Shim - Accessing General Documentation](../standards/library/modules/crucible-shim.md#accessing-general-documentation) for examples.

3. **Enterprise Three-Layer Config Module** (REQUIRED)
   - **Purpose**: Layered configuration (Crucible SSOT defaults → User config → Runtime overrides)
   - **Spec**: [Enterprise Three-Layer Config](../standards/library/modules/enterprise-three-layer-config.md)
   - **Compliance**:
     - Layer 1: Crucible defaults via helper (e.g., `workhorse/v1.0.0/defaults`)
     - Layer 2: User config at `~/.config/{vendor}/{app}/config.yaml`
     - Layer 3: Runtime dict/env var overrides
   - **Standard Env Vars** (REQUIRED):
     - `{PREFIX}PORT` - Server port (default: 8080)
     - `{PREFIX}HOST` - Server host (default: 0.0.0.0)
     - `{PREFIX}LOG_LEVEL` - Log level (trace|debug|info|warn|error, default: info)
     - `{PREFIX}CONFIG_PATH` - Config file path override
     - `{PREFIX}METRICS_PORT` - Metrics port (optional, default: same as PORT)
     - `{PREFIX}HEALTH_PORT` - Health port (optional, default: same as PORT)
   - **Precedence**: CLI flags → Env vars → Config file → Defaults

4. **Config Path API Module** (REQUIRED)
   - **Purpose**: Discover Fulmen config directories (user, system, app-specific)
   - **Spec**: [Config Path API](../standards/library/modules/config-path-api.md)
   - **Compliance**: Use `get_app_config_dir({app_name})` from App Identity for Layer 2 paths

5. **Schema Validation Module** (REQUIRED)
   - **Purpose**: Runtime validation of configs, requests, responses against Crucible schemas
   - **Spec**: [Schema Validation](../standards/library/modules/schema-validation.md)
   - **Compliance**: Validate config files on load, API payloads on ingress

### Observability & Resilience Modules

6. **Telemetry/Metrics Module** (REQUIRED)
   - **Purpose**: Prometheus-compatible metrics export (counters, gauges, histograms)
   - **Spec**: [Telemetry/Metrics](../standards/library/modules/telemetry-metrics.md)
   - **Compliance**:
     - Implement Prometheus exporter metrics: `prometheus_exporter_refresh_total`, `prometheus_exporter_http_requests_total`, etc.
     - Auto-emit module metrics (Foundry, Error Handling, FulHash) if modules used
     - Expose `/metrics` endpoint (Prometheus text format)
     - Use ADR-0007 histogram buckets
   - **Application Metrics**: Use binary-prefixed names (e.g., `percheron_task_duration_ms`, `groningen_request_latency_ms`)

7. **Logging Module** (REQUIRED)
   - **Purpose**: Structured logging with Crucible schema compliance
   - **Spec**: [Observability Logging](../standards/observability/logging.md)
   - **Compliance**:
     - Use SIMPLE or STRUCTURED profile from Crucible logging schemas
     - Service name from App Identity (`binary_name`)
     - Default middleware: Request ID correlation, severity mapping
     - Support `{PREFIX}LOG_LEVEL` env var

8. **Error Handling & Propagation Module** (REQUIRED)
   - **Purpose**: Standardized error types with severity, correlation, context wrapping
   - **Spec**: [Error Handling Propagation](../standards/library/modules/error-handling-propagation.md)
   - **Compliance**:
     - Use helper's error types (extend Pathfinder patterns)
     - Wrap errors uniformly for logging/JSON export
     - Auto-emit `error_handling_wraps_total` metric

9. **Signal Handling Module** (REQUIRED)
   - **Purpose**: Graceful shutdown, Ctrl+C double-tap, config reload via signals
   - **Spec**: [Signal Handling](../standards/library/modules/signal-handling.md)
   - **Compliance**:
     - Handle SIGTERM/SIGINT (graceful shutdown with cleanup)
     - Implement Ctrl+C double-tap pattern (2s window, force quit on second)
     - Support SIGHUP for config reload (validate → restart)
     - Expose `/admin/signal` HTTP endpoint for containerized environments
     - Use exit codes from Crucible signals catalog (e.g., SIGTERM=143)
   - **Runtime Pattern**: Context cancellation, resource cleanup chains

### Content & Documentation Modules

10. **Docscribe Module** (REQUIRED)
    - **Purpose**: Access Crucible documentation with frontmatter parsing
    - **Spec**: [Docscribe](../standards/library/modules/docscribe.md)
    - **Compliance**:
      - Embed examples via `docscribe.GetDoc(path)`
      - Optional: `/docs` endpoint for runtime doc serving

### Data Processing Modules (Conditional)

11. **Foundry Module** (RECOMMENDED for data-heavy workhorses)
    - **Purpose**: Catalogs for country codes, HTTP statuses, MIME types, text similarity
    - **Spec**: [Foundry Catalogs](../standards/library/foundry/README.md)
    - **Compliance**: Use `foundry.GetCountryCode()`, `foundry.GetHTTPStatus()`, etc. instead of hardcoded lookups
    - **Auto-Metrics**: Emits `foundry_mime_detections_total_*`, `foundry_mime_detection_ms_*` if MIME detection used

12. **FulHash Module** (RECOMMENDED for content hashing)
    - **Purpose**: Standardized hashing (XXH3-128 for performance, SHA256 for security)
    - **Spec**: [FulHash](../standards/library/modules/fulhash.md)
    - **Compliance**: Use helper's hash APIs instead of language-native hashlib
    - **Auto-Metrics**: Emits `fulhash_operations_total_*`, `fulhash_bytes_hashed_total`, `fulhash_operation_ms`

13. **Server Management Module** (RECOMMENDED for complex server orchestration)
    - **Purpose**: Multi-server orchestration (dev, test, preview, prod-like configs)
    - **Spec**: [Server Management](../standards/library/modules/server-management.md)
    - **Compliance**: Use helper's orchestration harness for multi-process coordination
    - **Use Case**: Workhorses with multiple background services or complex dev environments

### Module Integration Summary

| Module                        | Status      | Purpose                                  | Auto-Metrics                       | Spec Link                                                                                         |
| ----------------------------- | ----------- | ---------------------------------------- | ---------------------------------- | ------------------------------------------------------------------------------------------------- |
| App Identity                  | REQUIRED    | Binary name, env prefix, vendor metadata | None                               | [app-identity.md](../standards/library/modules/app-identity.md)                                   |
| Crucible Shim                 | REQUIRED    | SSOT asset access                        | None                               | [crucible-shim.md](../standards/library/modules/crucible-shim.md)                                 |
| Enterprise Three-Layer Config | REQUIRED    | Layered configuration                    | None                               | [enterprise-three-layer-config.md](../standards/library/modules/enterprise-three-layer-config.md) |
| Config Path API               | REQUIRED    | Config directory discovery               | None                               | [config-path-api.md](../standards/library/modules/config-path-api.md)                             |
| Schema Validation             | REQUIRED    | Runtime schema validation                | None                               | [schema-validation.md](../standards/library/modules/schema-validation.md)                         |
| Telemetry/Metrics             | REQUIRED    | Prometheus metrics export                | Yes (7 exporter metrics)           | [telemetry-metrics.md](../standards/library/modules/telemetry-metrics.md)                         |
| Logging                       | REQUIRED    | Structured logging                       | None                               | [logging.md](../standards/observability/logging.md)                                               |
| Error Handling                | REQUIRED    | Error wrapping, propagation              | Yes (`error_handling_wraps_total`) | [error-handling-propagation.md](../standards/library/modules/error-handling-propagation.md)       |
| Signal Handling               | REQUIRED    | Graceful shutdown, signals               | None                               | [signal-handling.md](../standards/library/modules/signal-handling.md)                             |
| Docscribe                     | REQUIRED    | Documentation access                     | None                               | [docscribe.md](../standards/library/modules/docscribe.md)                                         |
| Foundry                       | RECOMMENDED | Catalogs (country, HTTP, MIME)           | Yes (12 MIME detection metrics)    | [foundry/README.md](../standards/library/foundry/README.md)                                       |
| FulHash                       | RECOMMENDED | Content hashing                          | Yes (5 hash operation metrics)     | [fulhash.md](../standards/library/modules/fulhash.md)                                             |
| Server Management             | RECOMMENDED | Multi-server orchestration               | None                               | [server-management.md](../standards/library/modules/server-management.md)                         |

**Total Auto-Emitted Metrics**: 24 (7 exporter + 1 error handling + 12 MIME + 4 hash) when all modules active.

## Mandatory Capabilities

Workhorse forges MUST pre-integrate these ecosystem components, providing a launch-ready skeleton. Crucible access is indirect via the language-specific helper library (e.g., pyfulmen for Python forges), eliminating direct SSOT sync. Goneat is optional for DX tooling but not required for core bootstrap. All SSOT assets (Crucible, Cosmography, etc.) accessed via helper library shims; extend helpers for new SSOT (e.g., Cosmography shim for data ELT/analytics).

Implementers MUST comply with ecosystem standards in Crucible's `docs/standards/` (e.g., coding conventions, API patterns, repository structure) to ensure consistency.

1. **Helper Library Integration (Primary Bootstrap)**
   - Depend on and bootstrap via language-specific Fulmen helper library (e.g., `go install gofulmen` or `uv add pyfulmen`).
   - Use helper library's Crucible Shim for all asset access (schemas, docs, configs)—no direct Crucible sync or goneat SSOT in forges.
   - Pre-configure Three-Layer Config (embed defaults via helper, load user overrides, support BYOC), Schema Validation, and Documentation Module.
   - Include a simple `make bootstrap` script that installs the helper library and verifies Crucible access (e.g., `crucible.GetVersion()`).
   - Pin SSOT versions (e.g., Crucible v2025.10.2) in deps/lockfiles; expose via `/version` endpoint.
   - For emerging SSOT like Cosmography, use/extend helper shims (e.g., `cosmography.GetMap(id)`); no direct repo access.
   - Refer to [Fulmen Helper Library Standard](docs/architecture/fulmen-helper-library-standard.md) for integration patterns.

2. **Makefile (Mandatory)**
   - Always include Makefile following [Makefile Standard](docs/standards/makefile-standard.md).
   - Targets: `bootstrap` (helper install), `run` (CLI serve), `build` (binary), `test`, `lint`, `version-bump` (CalVer).
   - No SSOT sync targets; optional `dx` for goneat if included.
   - Ensure cross-platform (even for Python/TS via shell fallbacks).

3. **Goneat as Optional DX Tool**
   - Optionally include `.goneat/tools.yaml` for local development (e.g., linting, validation via goneat tasks)—no `ssot-consumer.yaml` or sync config.
   - Provide `make bootstrap-dx` for goneat installation if desired, but do NOT implement SSOT sync targets (`make sync-ssot` prohibited to avoid confusion with libraries).
   - Local overrides (`.goneat/tools.local.yaml`) gitignored; use only for non-Crucible tooling.
   - Refer to [Goneat Bootstrap Guide](docs/guides/bootstrap-goneat.md) for optional setup.

4. **Observability & Telemetry**
   - Pre-wire structured logging using Crucible logging schemas (SIMPLE/STRUCTURED profiles) via helper library.
   - Integrate metrics export (counters/gauges/histograms) via Telemetry/Metrics module.
   - Default middleware: Request ID correlation, severity mapping, throttling.
   - Expose health/version endpoints per API standards.
   - Refer to [Observability Logging](docs/standards/observability/logging.md) and [Telemetry/Metrics](docs/standards/library/modules/telemetry-metrics.md).

5. **Error Handling & Propagation**
   - Use standardized error types from Error Handling module (extend Pathfinder with severity/correlation).
   - Wrap errors uniformly for logging/export (JSON responses for APIs).
   - Refer to [Error Handling Standard](docs/standards/library/modules/error-handling-propagation.md).

6. **Config Path & Management**
   - Use Config Path API from helper library for discovering Fulmen/app directories.
   - Implement Three-Layer Config explicitly: Layer 1 (Crucible defaults via helper), Layer 2 (user from app dir), Layer 3 (runtime BYOC).
   - Pre-load/validate configs against schemas; support env var overrides (e.g., `FULMEN_CONFIG_HOME`).
   - Refer to [Config Path API](docs/standards/library/modules/config-path-api.md) and [Three-Layer Config](docs/standards/library/modules/enterprise-three-layer-config.md).

7. **Env Var & .env Support**
   - Use a required env var prefix based on breed name (e.g., `{BREED_NAME}_` where BREED*NAME is uppercase, default `GRONINGEN*` for groningen breed).
   - Include `.env.example` with standard vars (e.g., `GRONINGEN_PORT=8080`, `GRONINGEN_LOG_LEVEL=info`, `GRONINGEN_CONFIG_PATH=./config/groningen.yaml`); gitcommitted, user copies to `.env` (gitignored).
   - Load .env via three-layer (Layer 2: from app config dir; parse with helper or lang-native like python-dotenv).
   - In fulmen*cdrl_guide.md, instruct users to rename prefix (e.g., change `GRONINGEN*`to`MYAPI\_` in code/.env.example).
   - Validate prefix in CLI (`--env-prefix` flag optional); env vars override config (Layer 3).
   - Standard vars: Port, log level, metrics port, health port, config path; extend for app-specific.
   - Refer to Three-Layer Config for integration.

8. **Docscribe Module Integration**
   - Embed examples using docscribe module for frontmatter parsing and clean doc reads.
   - Include runtime doc serving (e.g., /docs endpoint) for self-documenting apps.
   - Refer to [Docscribe Standard](docs/standards/library/modules/docscribe.md).

9. **Standard Endpoints & Message Patterns**
   - **HTTP/gRPC Patterns**: Implement REST/gRPC backends with standard routes/methods:
     - `/health`: Liveness/readiness (JSON: `{status: "healthy", version: str}`).
     - `/version`: Full version info (integrate Crucible/SSOT versions from helper).
     - `/metrics`: Prometheus/OpenTelemetry export.
     - `/openapi.yaml`: OpenAPI specification (SHOULD serve if publishing HTTP API).
     - Error responses: JSON per [API HTTP Standards](docs/standards/protocol/http-rest-standards.md) (e.g., `{error: {code: str, message: str, details: any}}`).
     - gRPC: Use proto defs from Crucible schemas; unary/streaming with metadata propagation.
   - **Messages**: Structured payloads validated against schemas (e.g., log events, metrics). Use helper's Foundry for patterns (e.g., HTTP status groups, MIME types).
   - **OpenAPI Documentation**: Workhorses exposing HTTP APIs SHOULD publish OpenAPI specs with coverage testing per [ADR-0014](decisions/ADR-0014-openapi-spec-coverage.md). See [HTTP Server Patterns](../guides/testing/http-server-patterns.md) for implementation guidance.
   - Refer to [API Standards](docs/standards/protocol/README.md).

10. **CLI Surface for Server Invocation**
    - Provide a standard CLI wrapper (e.g., via cobra/click/argparse) for backend server:
      - `{breed-name} serve [flags]`: Starts server (e.g., `groningen serve`, `percheron serve`).
      - Standard flags: `--config <path>` (Three-Layer), `--port <int>`, `--log-level <str>` (trace/debug/info/warn/error), `--metrics-port <int>`, `--health-port <int>`, `--env-prefix <str>` (default from breed name), `--version` (print and exit), `--help`.
      - Subcommands:
        - `serve` (default): Starts server.
        - `version` / `version --extended`: Basic version; extended shows full info (app version, SSOT/Crucible versions from helper, build date, git commit).
        - `envinfo`: Dumps effective env vars (filtered by prefix), config layers, SSOT versions (helpful for debugging).
        - `doctor`: Runs checks/scaffolding (health self-test, config validation, missing deps, suggest fixes; e.g., "Missing .env? Copy .env.example").
        - `health`: Self-check (mirrors /health endpoint).
      - Defaults from Crucible configs via helper; env var overrides (e.g., `PORT=8080`); load .env via three-layer.
    - Binary name MUST follow the Binary Naming Convention (breed name only, see dedicated section).
    - Integrate with helper library for config/logging/SSOT info.
    - Refer to [Repository Structure: Workhorse](docs/standards/repository-structure/typescript/workhorse.md) for patterns; see goneat CLI for version/envinfo/doctor examples.

11. **CI/CD & Release Readiness**
    - Include basic GitHub Actions for lint/test/build.
    - Pre-commit hooks via helper library or simple scripts (no goneat dependency).
    - Versioning: CalVer support with `make version-bump`.
    - Refer to [Repository Lifecycle](docs/standards/repository-lifecycle.md).

## Runtime Patterns

Workhorse forges MUST implement these for production reliability:

- **Graceful Shutdown**: Handle OS signals (SIGINT/SIGTERM) with context cancellation; allow clean resource cleanup (e.g., DB connections, goroutines in Go; asyncio in Python).
- **Monitoring & Debug**: Expose /debug/pprof (Go) or equivalent; integrate with observability for runtime metrics (CPU/memory, goroutine count).
- **Migration Support**: If DB-integrated, include migration tooling (e.g., goose for Go, alembic for Python); config flag for auto-migrate on start.
- **DB Integration**: Standard config for connections (DSN via three-layer); pool management; health checks for DB readiness.

Refer to language standards for implementation (e.g., Go context, Python signal handling).

## UX Support

- **Browser-Based UX Flag**: Config `enable_ui: bool` (default false) to toggle /ui endpoint (e.g., simple dashboard for metrics/health using embedded assets).
- If enabled, serve static UI from /ui (use helper's Documentation for dynamic content); secure with auth if production.
- CDRL guide: Instruct on enabling/customizing UI post-refit.

## Refactoring Guide (Sumpter Example)

To refactor existing tools like sumpter (Go CLI for data extraction/inspection) to workhorse standard:

- **Helper Integration**: Add gofulmen dep; replace direct schemas/docs with Crucible Shim (e.g., load extract schemas via `crucible.LoadSchema("extract", "v0.1.0", "record-match")`); use three-layer for config (embed defaults, user .yaml, env).
- **Remove Direct SSOT**: Eliminate goneat SSOT sync; embed via gofulmen (internal/assets now from helper).
- **Binary Naming**: Rename to breed name only (e.g., if refactoring as `forge-workhorse-sumpter`, binary becomes `sumpter`, not `workhorse-sumpter`).
- **Standardize CLI**: Sumpter already has version/envinfo/doctor—extend version --extended for SSOT/build info; add serve subcommand for backend mode (e.g., API for extract/validate).
- **Add Endpoints**: Implement /health, /version, /metrics; /extract (POST for recipes) using gRPC/HTTP patterns.
- **Env/ .env**: Adopt breed name prefix (e.g., `SUMPTER_`); add .env.example with DSN/log-level.
- **Runtime**: Add graceful shutdown (context in main); pprof on /debug; migration if DB (sumpter has none, but future-proof).
- **UX**: Config enable_ui for /ui (e.g., inspect results dashboard).
- **Makefile**: Ensure compliance (bootstrap installs gofulmen, no sync).

This refactoring reduces boilerplate, aligns with ecosystem, adds server capabilities without breaking CLI focus.

4. **CI/CD & Release Readiness**
   - Include basic GitHub Actions or equivalent for lint/test/build.
   - Pre-commit hooks via goneat (format, validate schemas).
   - Versioning: CalVer support with `make version-bump`.
   - Refer to [Repository Lifecycle](docs/standards/repository-lifecycle.md).

## Directory Structure

Workhorse forges MUST follow this skeleton for consistency (Python example with `groningen` breed; adapt for Go/TS):

```
forge-workhorse-groningen/
├── .fulmen/                          # Fulmen application metadata (REQUIRED)
│   └── app.yaml                      # App Identity config (binary name, vendor, env prefix)
├── .goneat/                          # Optional DX tooling only
│   ├── tools.yaml                    # Linting/validation (no SSOT)
│   └── tools.local.yaml.example      # Local template
├── .github/
│   └── workflows/                    # CI/CD pipelines (test, release)
│       ├── ci.yaml
│       └── release.yaml
├── src/
│   └── groningen/                    # Main entrypoint (CLI + server) - breed name only
│       ├── __init__.py
│       ├── main.py                   # CLI surface: serve, version, health, envinfo, doctor
│       └── internal/                 # App logic (server handlers, config loaders)
├── config/
│   └── groningen.yaml                # App-specific defaults (Layer 2/3; Layer 1 via helper)
├── docs/
│   ├── README.md                     # Forge overview
│   ├── DEVELOPMENT.md                # Local setup, contribution
│   └── development/
│       └── fulmen_cdrl_guide.md      # CDRL instructions for users (naming/root conventions)
├── pyproject.toml                    # Deps: pyfulmen (mandatory), binary: groningen
├── .env.example                      # Standard env vars (committed): GRONINGEN_PORT, etc.
├── Makefile                          # Standard targets (bootstrap, run, build, test)
└── .gitignore                        # Ignore .env, builds, local configs
```

- **Primary Entry**: `src/{breed-name}/main.py` launches the workhorse (breed name only, no `workhorse_` prefix).
- **No Direct Synced Assets**: All Crucible/SSOT via helper library shims; no `docs/crucible-*`, `config/crucible-*`, or SSOT folders.
- **No Domain Code**: Placeholders (e.g., echo server in internal/server.py) to demonstrate; users refit via CDRL.

## Binary Naming Convention

Workhorse forges MUST follow this naming pattern to support clean CDRL refit workflows:

**Rule**: The binary name MUST be the distinctive identifier (e.g., the horse breed name like `groningen`, `percheron`, `clydesdale`), excluding the `workhorse` category prefix.

**Rationale**:

- Templates should model final usage patterns. During CDRL refit, users rename binaries to their application name (e.g., `myapi`, `analytics-engine`), not `workhorse-myapi`.
- The repository name provides full context (`forge-workhorse-{breed}`), while the binary name serves as a placeholder for the user's final application identity.
- This separation ensures the refit process is straightforward: users replace the breed name with their app name in a single refactoring pass.

**Examples**:

| Repository Name             | Binary Name           | After User Refit   | Status              |
| --------------------------- | --------------------- | ------------------ | ------------------- |
| `forge-workhorse-groningen` | `groningen`           | `analytics-engine` | ✅                  |
| `forge-workhorse-percheron` | `percheron`           | `data-processor`   | ✅                  |
| `forge-workhorse-groningen` | `workhorse-groningen` | N/A                | ❌ Redundant prefix |

**Implementation Notes**:

- **CLI Entry Point**: Name the main executable/entry point using only the breed name (e.g., `groningen serve`, not `workhorse-groningen serve`).
- **Module/Package Names**: Follow language conventions while maintaining the breed identifier:
  - **Go**: Package `main`, binary output `groningen` (via `go build -o groningen`)
  - **Python**: Package `groningen` (not `workhorse_groningen`), entry point via `pyproject.toml`: `groningen = "groningen.main:cli"`
  - **TypeScript**: Package name can be `@forge/workhorse-groningen` for npm scope, but binary via `package.json` scripts should be `groningen`
- **Environment Variables**: Use breed name as prefix (e.g., `GRONINGEN_PORT`, `GRONINGEN_LOG_LEVEL`). Users update this during refit (e.g., to `MYAPI_PORT`).
- **Config Files**: Name using breed (e.g., `config/groningen.yaml`). CDRL guide instructs users to rename.

**Cross-Language Consistency**: All language implementations of a given workhorse breed MUST use identical binary names to maintain ecosystem coherence.

## Bootstrap Strategy

Forges bootstrap like libraries but add app-specific setup:

1. **Initial Clone**: Users `git clone` or degit; run `make bootstrap` to install goneat + deps.
2. **Sync Assets**: `make sync` pulls/updates Crucible.
3. **Launch**: `make run` starts with defaults; `make build` for production binary.
4. **Local Overrides**: `.goneat/tools.local.yaml` for dev iteration.

**Important**: Use symlinks for local tools; ensure `type: link` creates proper links (see helper library pitfalls).

## Testing Expectations

- Unit: Cover config loading (three-layer), logging setup, error wrapping (70%+).
- Integration: End-to-end via helper (asset access, validation); mock telemetry.
- E2E: Basic health checks, config validation, CLI invocation.
- Coverage: Enforce via CI; align with module manifest thresholds.

All forge tests must follow the cross-repo [Portable Testing Practices](../standards/testing/portable-testing-practices.md) so workhorse templates run cleanly in CI and sandboxed environments. Provide shared skip helpers for network/file-system dependent suites and reference them in `DEVELOPMENT.md`.

## Documentation Requirements

- **README.md**: CDRL guide (link to fulmen_cdrl_guide.md), quick start (`make bootstrap && make run`), ecosystem links.
- **DEVELOPMENT.md**: Local workflow, testing, release process.
- **Architecture Overview**: `docs/workhorse-overview.md` with module catalog template (adapt from helper standard).
- **Examples**: Inline code for config/telemetry setup.
- **Frontmatter**: All docs use YAML headers for metadata.

Adapt helper library template:

```markdown
# Workhorse Forge Overview

## Purpose & Scope

- Template for durable backends; pre-wires observability/config.

## Integrated Modules

| Module        | Tier | Summary      | Spec Link |
| ------------- | ---- | ------------ | --------- |
| crucible-shim | Core | Asset access | [link]    |
| ...           | ...  | ...          | ...       |

## Launch Guide

- Follow fulmen_cdrl_guide.md → `make bootstrap` → `make run`
```

## CDRL Compliance

Workhorse forges MUST comply with the [Fulmen Template CDRL Standard](fulmen-template-cdrl-standard.md) to ensure predictable, repeatable customization workflows for downstream users.

### Required CDRL Implementation

1. **App Identity Module** (PRIMARY CUSTOMIZATION POINT)
   - Implement `.fulmen/app.yaml` as documented in [App Identity Module](../standards/library/modules/app-identity.md)
   - All parameterization points (binary name, env prefix, config paths, telemetry namespaces) MUST derive from App Identity
   - No hardcoded breed names in source code (except `.fulmen/app.yaml` itself)

2. **CDRL Validation Targets** (REQUIRED MAKEFILE TARGETS)
   - Implement `make validate-app-identity` per [Makefile Standard Annex B](../standards/makefile-standard.md#annex-b-template-repository-cdrl-targets)
   - Implement `make doctor` (or `make validate-cdrl-ready`) for comprehensive refit validation
   - Both targets MUST be documented in Makefile help output

3. **CDRL Workflow Guide** (REQUIRED DOCUMENTATION)
   - Provide `docs/development/fulmen_cdrl_guide.md` with template-specific CDRL instructions
   - Document all parameterization points (binary name, module path, env vars, config files)
   - Include verification checklist and troubleshooting guide
   - Link to ecosystem CDRL guide: [CDRL Workflow Guide](../standards/cdrl/workflow-guide.md)

4. **Directory Structure CDRL Readiness**
   - `.fulmen/app.yaml` MUST exist with breed name as default identity
   - `.env.example` MUST use breed-prefixed environment variables
   - `config/{breed}.yaml` MUST be named with breed identifier (users rename during refit)
   - All config paths MUST derive from App Identity, not hardcoded strings

5. **Bootstrap Script** (RECOMMENDED)
   - Provide `scripts/bootstrap.py` (Python) or `scripts/bootstrap.ts` (TypeScript) for reproducible tool installation
   - Implement manifest-driven bootstrap reading `.goneat/tools.yaml` (version-pinned tools)
   - Integrate with `make bootstrap` target
   - See [Reference Bootstrap Pattern](../standards/cdrl/reference-bootstrap.md) for implementation guidance
   - **Note**: Go templates can use native dependency management instead

### CDRL Parameterization Points

Templates MUST document these customization points in their CDRL guide:

- **Binary Name**: CLI commands, HTTP server responses, process names, telemetry service names
- **Module Path**: `go.mod` module directive, `package.json` name, `pyproject.toml` name
- **Environment Variables**: All env vars MUST use `{ENV_PREFIX}*` from App Identity
- **Configuration Paths**: Layer 2 config at `~/.config/{vendor}/{config_name}/config.yaml`
- **Telemetry Namespaces**: Logging service names, metric prefixes, trace service names

### Validation Requirements

**Pre-Release Gate**: Templates MUST pass all CDRL validation before being published:

```bash
# Required validation sequence
make validate-app-identity  # Exit 0: No hardcoded breed names
make doctor                 # Exit 0: Comprehensive validation passed
make test                   # Exit 0: All tests pass
```

**Example Refit Test**: Template maintainers MUST periodically test full CDRL workflow:

1. Clone template to temporary directory
2. Edit `.fulmen/app.yaml` with test identity (e.g., breed → `testapp`)
3. Run `make validate-app-identity` → SHOULD detect hardcoded references
4. Fix all violations
5. Run `make doctor` → MUST pass
6. Run `make test` → MUST pass
7. Run `make build && make run` → MUST work with test identity

### CDRL References

- [Fulmen Template CDRL Standard](fulmen-template-cdrl-standard.md) - Architectural requirements
- [CDRL Workflow Guide](../standards/cdrl/workflow-guide.md) - User-facing step-by-step instructions
- [Makefile Standard Annex B](../standards/makefile-standard.md#annex-b-template-repository-cdrl-targets) - Required validation targets
- [Repository Category Taxonomy](../../config/taxonomy/repository-categories.yaml) - CDRL compliance requirements by category

## Version Alignment & Lifecycle

- Pin helper libraries/Crucible versions.
- CalVer releases; changelog via release-notes.
- Lifecycle: `prototype` → `stable` per taxonomy.

## Common Pitfalls & Solutions

- **Missing Sync**: Always run `make sync` post-clone.
- **Dep Conflicts**: Lock helper lib versions.
- **Over-Refit**: Keep domain placeholders minimal.

## References

- [Fulmen Helper Library Standard](fulmen-helper-library-standard.md)
- [Repository Category Taxonomy](../../schemas/taxonomy/repository-category/v1.0.0/README.md)
- [Ecosystem Brand Summary](../../config/branding/ecosystem.yaml) - For `version --extended` or `about` command/endpoint
- [Technical Manifesto](fulmen-technical-manifesto.md)
- [Binary Naming Convention](#binary-naming-convention) (this document)
- Prototype: forge-workhorse-groningen (Go), forge-workhorse-groningen-py (Python)

## Changelog

- **2025-10-20**: Initial draft for workhorse category.
