# Module Compliance Matrix: Forge Categories

**Purpose**: Quick reference for module requirements across Workhorse, Codex, and Microtool forges.

**Last Updated**: 2025-11-09

---

## Module Requirements by Category

| Module                 | Workhorse   | Codex       | Microtool   | Notes                               |
| ---------------------- | ----------- | ----------- | ----------- | ----------------------------------- |
| **App Identity**       | REQUIRED    | REQUIRED    | REQUIRED    | All categories need identity        |
| **Crucible Shim**      | REQUIRED    | REQUIRED    | RECOMMENDED | Microtools: only if using SSOT      |
| **Three-Layer Config** | REQUIRED    | REQUIRED    | RECOMMENDED | Microtools: only if config-heavy    |
| **Config Path API**    | REQUIRED    | REQUIRED    | RECOMMENDED | Microtools: only if using config    |
| **Schema Validation**  | REQUIRED    | REQUIRED    | OPTIONAL    | Microtools: only if reading/writing |
| **Logging**            | REQUIRED    | OPTIONAL    | REQUIRED    | Codex: build-time only              |
| **Exit Codes**         | N/A         | N/A         | REQUIRED    | Microtools: critical for CI/CD      |
| **Signal Handling**    | REQUIRED    | N/A         | REQUIRED    | Codex: static builds don't need     |
| **Error Handling**     | REQUIRED    | N/A         | REQUIRED    | Codex: uses logging instead         |
| **Telemetry/Metrics**  | REQUIRED    | N/A         | OPTIONAL    | Microtools: skip for short-lived    |
| **Server Management**  | REQUIRED    | N/A         | N/A         | Workhorse only (HTTP endpoints)     |
| **Docscribe**          | REQUIRED    | REQUIRED    | N/A         | Microtools: don't serve docs        |
| **Foundry Catalogs**   | RECOMMENDED | RECOMMENDED | N/A         | Reference data (HTTP, MIME, etc.)   |
| **FulHash**            | RECOMMENDED | RECOMMENDED | N/A         | Hashing utilities                   |

---

## Workhorse Module Compliance (10 REQUIRED)

**Spec**: [fulmen-forge-workhorse-standard.md](./fulmen-forge-workhorse-standard.md)

### REQUIRED (10 total)

1. App Identity
2. Crucible Shim
3. Three-Layer Config
4. Config Path API
5. Schema Validation
6. Telemetry/Metrics
7. Logging
8. Error Handling & Propagation
9. Signal Handling
10. Docscribe

### RECOMMENDED

- Foundry (catalog display)
- FulHash (content hashing)

### Critical Features

- HTTP `/health`, `/version`, `/metrics` endpoints
- Request correlation IDs
- Prometheus metrics export
- Graceful shutdown (SIGINT/SIGTERM)
- Config reload (SIGHUP)

---

## Codex Module Compliance (6 REQUIRED)

**Spec**: [fulmen-forge-codex-standard.md](./fulmen-forge-codex-standard.md)

### REQUIRED (6 total)

1. App Identity
2. Crucible Shim
3. Three-Layer Config
4. Config Path API
5. Schema Validation
6. Docscribe

### RECOMMENDED

- Foundry (catalog display)
- FulHash (content hashing, cache busting)

### OPTIONAL

- Logging (build pipeline diagnostics)

### NOT Required

- ❌ Signal Handling (static builds)
- ❌ Telemetry/Metrics (use analytics integrations)
- ❌ Error Handling & Propagation (build errors via logging)
- ❌ Server Management (not server applications)

### Critical Features

- Static-first architecture (Astro)
- Build-time SSOT ingestion
- Frontmatter parsing
- Site metadata from `.fulmen/app.yaml`

---

## Microtool Module Compliance (5 REQUIRED)

**Spec**: [fulmen-forge-microtool-standard.md](./fulmen-forge-microtool-standard.md)

### REQUIRED (5 total)

1. App Identity
2. Logging
3. Exit Codes
4. Signal Handling
5. Error Handling

### RECOMMENDED (3 total)

6. Crucible Shim (if using schemas/configs)
7. Three-Layer Config (if config-heavy)
8. Config Path API (if using config files)

### OPTIONAL (2 total)

9. Schema Validation (if reading/writing YAML/JSON)
10. Telemetry/Metrics (if long-running—rare)

### NOT Required

- ❌ Server Management (no HTTP endpoints)
- ❌ Docscribe (don’t serve documentation)

### Prohibited Features

- Multiple unrelated commands
- Library exports (`pkg/`)
- Long-running daemons

### Critical Features

- Single purpose enforcement
- Helper library integration (gofulmen/tsfulmen/rsfulmen)
- CI/CD-friendly exit codes
- Graceful shutdown
- No circular dependencies

---

## Usage Guidelines

### When Building a Workhorse

**Example**: `forge-workhorse-percheron` (Python/FastAPI)

- Must implement `/health`, `/version`, `/metrics` endpoints
- Must handle signals gracefully (SIGTERM, SIGINT, SIGHUP)
- Must export Prometheus metrics
- Must use structured logging
- Must have request correlation

### When Building a Codex

**Example**: `forge-codex-pulsar` (TypeScript/Astro)

- Must ingest Crucible SSOT at build time
- Must parse documentation frontmatter
- Must validate schemas
- Must display site metadata from `.fulmen/app.yaml`
- Must support three-layer config (build settings, deployment config)

### When Building a Microtool

**Example**: `forge-microtool-gimlet` (Go), producing `microtool-fulmen-fixtures`

- Start with the five required modules
- Add optional modules only when scoped use cases demand them
- Enforce reference-implementation requirement (build/test/run before CDRL; see [Fulmen Template CDRL Standard](./fulmen-template-cdrl-standard.md))

---

## Implementation Guidance

1. **Phase 1 – Foundation**: App Identity + helper library wiring.
2. **Phase 2 – Observability**: Logging, metrics (if required), error handling.
3. **Phase 3 – Configuration**: Three-layer config, config paths, Crucible shim.
4. **Phase 4 – Resilience**: Signal handling, server management (workhorse).
5. **Phase 5 – Content/Validation**: Docscribe, schema validation, Foundry/FulHash where applicable.

---

## Reference Links

- [Workhorse Standard](./fulmen-forge-workhorse-standard.md)
- [Codex Standard](./fulmen-forge-codex-standard.md)
- [Microtool Standard](./fulmen-forge-microtool-standard.md)
- [Fulmen Template CDRL Standard](./fulmen-template-cdrl-standard.md)
- [Module Manifest YAML](../../config/library/v1.0.0/module-manifest.yaml)

---

**Status**: Active Reference  
**Version**: 1.0.0  
**Owner**: Architecture Committee
