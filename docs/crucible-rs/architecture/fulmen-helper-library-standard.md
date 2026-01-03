---
title: "Fulmen Helper Library Standard"
description: "Standard structure and capabilities for gofulmen, tsfulmen, and future language helpers"
author: "Schema Cartographer"
date: "2025-10-02"
last_updated: "2025-11-11"
status: "draft"
tags: ["architecture", "helper-library", "multi-language", "local-development"]
---

# Fulmen Helper Library Standard

This document formalizes the expectations outlined in the gofulmen proposal so that every `*fulmen`
language foundation delivers the same core capabilities. The canonical list of languages and statuses
is maintained in the [Library Ecosystem taxonomy](library-ecosystem.md#language-foundations-taxonomy);
consult that table before proposing new foundations or changing lifecycle state.

## Scope

Applies to language-specific Fulmen helper libraries (gofulmen, tsfulmen, pyfulmen, csfulmen, rufulmen, etc.). Excludes SSOT repos (Crucible, Cosmography) and application/tool repos (Fulward, goneat).

## Canonical Façade Principle

**ARCHITECTURAL DECREE**: Helper libraries MUST provide canonical façades for all modules, regardless of whether the underlying functionality wraps standard library features, third-party libraries, or custom implementations.

### Rationale

The Fulmen ecosystem prioritizes **cross-language interface consistency** over implementation details. When applications use helper library modules, they should encounter the same API surface, error handling patterns, and behavioral contracts across Go, Python, and TypeScript—even when the underlying implementations differ significantly.

### Core Tenets

1. **Façades Are Mandatory**: Every module in the helper library registry MUST ship a façade, even if the implementation simply wraps standard library functionality with minimal adaptation.

2. **Implementation Details Are Separate**: Whether a module wraps `stdlib`, uses third-party dependencies, or provides custom logic is documented in the `implementation` field of the module registry—it does NOT determine tier assignment.

3. **Tier Assignment Is About Use Case**: A module's tier (Core, Common, Specialized) reflects:
   - **Universality**: How many applications need this capability
   - **Dependency footprint**: External dependencies beyond stdlib
   - **Adoption patterns**: Expected usage across the ecosystem

4. **stdlib Wrapping Is Common**: If stdlib provides baseline functionality that most applications need, wrapping it in a Common tier module with a consistent façade is the CORRECT pattern. Examples:
   - `config`: Wraps `os`, `path/filepath` (Go), `os`, `pathlib` (Python), `fs`, `path` (Node.js)
   - `fulpack`: Wraps `archive/tar`, `archive/zip` (Go), `tarfile`, `zipfile` (Python), `tar-stream`, `archiver` (TypeScript)
   - `logging`: Wraps `log/slog` (Go), `logging` (Python), `console` with structured output (TypeScript)

5. **Cross-Language Orchestration**: The power of façades emerges when:
   - Python developers call `pyfulmen.fulpack.create_tar_gz()`
   - Go developers call `gofulmen.fulpack.CreateTarGz()`
   - TypeScript developers call `@fulmenhq/tsfulmen/fulpack.createTarGz()`

   ...and all three receive the same error envelope structure, the same checksum verification behavior, and the same path traversal protections, regardless of whether the implementation uses stdlib, third-party libs, or custom code.

### Implementation Transparency

The `implementation` field in the module registry provides full transparency about how each language implements a module:

```yaml
languages:
  go:
    status: available
    package: github.com/fulmenhq/gofulmen/fulpack
    version: "0.1.0"
    implementation: "Wraps stdlib archive/tar, archive/zip, compress/gzip"
  python:
    status: available
    package: pyfulmen.fulpack
    version: "0.1.0"
    implementation: "Wraps stdlib tarfile, zipfile, gzip"
  typescript:
    status: available
    package: "@fulmenhq/tsfulmen/fulpack"
    version: "0.1.0"
    implementation: "Wraps tar-stream and archiver for cross-platform compatibility"
```

This transparency allows developers to understand:

- Performance characteristics per language
- Dependency requirements
- Platform compatibility
- Maintenance complexity

### Anti-Patterns

❌ **DO NOT** skip façades because "stdlib already provides this"
✅ **DO** wrap stdlib to ensure consistent error handling and API surface

❌ **DO NOT** make every stdlib wrapper a Specialized module
✅ **DO** use Common tier for widely-needed stdlib wrappers (tar/zip, basic encoding, signals)

❌ **DO NOT** assume "no external deps = no façade needed"
✅ **DO** provide façades to orchestrate cross-language interface consistency

❌ **DO NOT** document implementation strategy as an override reason
✅ **DO** use the `implementation` field to explain how the module is built

### When Specialized Tier IS Appropriate

Specialized tier is for:

- **Niche use cases** (<50% of applications need it)
- **Heavy external dependencies** (multiple third-party libs, large runtime footprint)
- **Advanced/exotic capabilities** (e.g., `fulpack-formats` for 7z/rar/brotli, `fulencoding-advanced` for 200+ character sets)

Examples:

- `fulpack` (Common): Basic tar.gz and zip using stdlib
- `fulpack-formats` (Specialized): Exotic formats (7z, rar, brotli) requiring heavy dependencies

### References

- [Extension Framework v2 Clarifications](../../.plans/memos/helperlibs/2025-11-15-extension-framework-clarifications-v2.md)
- [Team Responses Synthesis](../../.plans/memos/helperlibs/2025-11-15-team-responses-synthesis.md)
- [Module Registry](../../config/taxonomy/library/platform-modules/v1.0.0/modules.yaml)

## Mandatory Capabilities

1. **Goneat Bootstrap Pattern**
   - Follow the [Goneat Bootstrap Guide](../guides/bootstrap-goneat.md) so contributors and CI can install the CLI via package manager or `go install github.com/fulmenhq/goneat/cmd/goneat@latest`.
   - Provide a `make bootstrap` target (or equivalent script) that verifies goneat is available and documents the fallback steps in `docs/`.
   - Keep `.goneat/tools.yaml` and `.goneat/tools.local.yaml.example` templates for teams that still need download-based bootstraps; ensure `.goneat/tools.local.yaml` remains gitignored.
   - Once goneat is installed, use `goneat ssot sync` with a repository-specific `.goneat/ssot-consumer.yaml` (see below) to pull Crucible assets. Avoid bundling legacy FulDX binaries.

2. **SSOT Synchronization**
   - Include `.goneat/ssot-consumer.yaml` describing the Crucible assets your library consumes. Recommended shape:
     ```yaml
     version: "2025.10.2"
     sync_path_base: lang/<language> # e.g., lang/typescript, lang/python
     sources:
       - id: crucible
         repo: https://github.com/fulmenhq/crucible.git
         ref: main
         include:
           - docs/**/*.md
           - schemas/**/*
           - config/**/*.yaml
           - config/sync/sync-keys.yaml
         notes: "Use sync keys defined in config/sync/sync-keys.yaml"
     ```
     `sync_path_base` MUST match the directory under `lang/` (for example `lang/typescript`) so that goneat writes the synced artifacts to the correct tree.
   - Create `.goneat/ssot-consumer.local.yaml` for local development (gitignored) with overrides such as `local_path: ../crucible`.
   - Add to `.gitignore`:
     ```
     .goneat/*.local.yaml
     ```
   - Commit synced assets—for example `docs/crucible-<lang>`, `schemas/crucible-<lang>`, `config/crucible-<lang>`, and `.crucible/metadata`—so consumers can work offline.
   - Provide a `make sync` target (or equivalent) that runs `goneat ssot sync --manifest .goneat/ssot-consumer.yaml`.
   - Use glob patterns such as `schemas/**/*` to capture both `.json` and `.yaml` assets.
   - Refer to the [SSOT Sync Standard](../standards/library/modules/ssot-sync.md) for command surface and testing guidance.

3. **Crucible Shim**
   - Provide idiomatic access to Crucible assets (docs, schemas, config defaults).
   - Re-export version constants so consumers can log/report underlying Crucible snapshot.
   - Discover available categories (`ListAvailableDocs()`, `ListAvailableSchemas()`) via embedded metadata or generated index.
   - Refer to the [Crucible Shim Standard](../standards/library/modules/crucible-shim.md).

4. **Config Path API**
   - Implement `GetAppConfigDir`, `GetAppDataDir`, `GetAppCacheDir`, `GetAppConfigPaths`, and `GetXDGBaseDirs` (naming per language).
   - Expose Fulmen-specific helpers (`GetFulmenConfigDir`, etc.) aligned with [Fulmen Config Path Standard](../standards/config/fulmen-config-paths.md).
   - Respect platform defaults (Linux/macOS/Windows) and environment overrides.
   - Refer to the [Config Path API Standard](../standards/library/modules/config-path-api.md).

5. **Three-Layer Config Loading**
   - Layer 1: Embed Crucible defaults from `config/{category}/vX.Y.Z/*-defaults.yaml`.
   - Layer 2: Merge user overrides from `GetFulmenConfigDir()`.
   - Layer 3: Allow application-provided config (BYOC) with explicit API hooks.
   - Refer to the [Three-Layer Configuration Standard](../standards/library/modules/enterprise-three-layer-config.md).

6. **Schema Validation Utilities**
   - Provide helpers to load, parse, and validate schemas shipped in Crucible.
   - Optional but recommended: integrate with language-native validation libraries.
   - Refer to the [Schema Validation Helper Standard](../standards/library/modules/schema-validation.md).

7. **Docscribe Module**
   - Provide APIs for accessing Crucible documentation assets, including frontmatter extraction and clean content reads.
   - Integrate with Crucible Shim for asset discovery and Schema Validation for config processing.
   - Refer to the [Docscribe Module Standard](../standards/library/modules/docscribe.md).

8. **Error Handling Propagation**
   - Implement the canonical error contract as a schema-backed data model per [ADR-0006](decisions/ADR-0006-error-data-models.md).
   - Expose helpers (`wrap`, `validate`, `exitWithError`) that operate on the shared data shape, allowing language-native wrappers as optional extras.
   - Refer to the [Error Handling & Propagation Standard](../standards/library/modules/error-handling-propagation.md).

9. **Telemetry & Metrics Export**
   - Provide counter, gauge, and histogram helpers aligned with the metrics taxonomy.
   - Use the default millisecond histogram buckets defined in [ADR-0007](decisions/ADR-0007-telemetry-default-histogram-buckets.md) unless overridden explicitly.
   - Refer to the [Telemetry & Metrics Standard](../standards/library/modules/telemetry-metrics.md).

10. **Module Registry Compliance**

- Read and use the definitive module registry SSOT at `config/taxonomy/library/modules/v1.0.0/modules.yaml`.
- Implement modules according to their assigned tier (Core, Common, Specialized).
- Respect language-specific tier overrides when present (e.g., Common in one language, Specialized in another).
- For Specialized modules, implement optional installation patterns per language:
  - Python: `pip install pyfulmen[extras]`
  - TypeScript: Peer dependencies or optional dependencies
  - Go: Separate import paths with tree-shaking
- Validate module implementation matches registry metadata (tier, dependencies, version).
- Report any registry inconsistencies to Crucible maintainers for correction.
- Refer to the [Module Registry](config/taxonomy/library/modules/v1.0.0/modules.yaml) and [Extension Tier Standard](../standards/library/modules/extension-tiering.md) (available v0.2.11+).

## Ecosystem Tool Integration

Helper libraries serve as the primary access point for Crucible assets in the broader Fulmen ecosystem. Tools and applications MUST access Crucible indirectly through helper libraries to ensure version alignment and consistent APIs.

### Tool Integration Pattern

1. **Depend on Helper Library**: Tools declare dependency on appropriate helper library (pyfulmen, gofulmen, tsfulmen)
2. **Use Shim APIs**: Access Crucible assets through library-provided APIs (see API Surface Requirements)
3. **Delegate Bootstrap**: Helper library handles goneat installation and SSOT sync
4. **Version Reporting**: Tools can report embedded Crucible version for compatibility

### Asset Discovery & Access

Tools discover and access Crucible content through standardized shim APIs:

**Finding Content:**

```python
# List available categories and assets
categories = crucible.list_categories()  # ['docs', 'schemas', 'config']
docs = crucible.list_assets('docs')      # Asset metadata array

# Search by pattern
logging_docs = [asset for asset in docs if 'logging' in asset.id]
```

**Accessing Content:**

```python
# Direct content access
doc_content = crucible.get_documentation('standards/observability/logging')
schema = crucible.load_schema('observability', 'logging', 'v1.0.0', 'logger-config')

# Streaming for large content
with crucible.open_asset('docs/architecture/fulmen-technical-manifesto.md') as stream:
    for chunk in stream:
        process(chunk)
```

**Error Handling:**

```python
try:
    doc = crucible.get_documentation('standards/observability/missing-doc')
except AssetNotFoundError as e:
    print(f"Suggestions: {e.suggestions}")  # Helpful suggestions provided
```

### Prohibited Patterns

- Direct imports of Crucible assets
- Custom SSOT sync configurations in tools
- Version pinning that bypasses helper library coordination
- Manual file system access to embedded assets

## Optional (Recommended) Capabilities

- Pathfinder & ASCII helpers (for languages that can support them).
- Cosmography shims once that SSOT expands.
- Registry API clients if SSOT repos expose HTTP endpoints in the future.

Module tiers, language-specific implementations, and cross-language status are tracked in the
definitive module registry at `config/taxonomy/library/modules/v1.0.0/modules.yaml` (validated by
`schemas/taxonomy/library/modules/v1.0.0/module-entry.schema.json`). This registry is the canonical
SSOT for all helper library modules and their tier assignments.

## Directory Structure

### Required Directories

```
<foundation-repo>/
├── .goneat/
│   ├── tools.yaml                    # Production tooling manifest (goneat ensured on PATH)
│   ├── tools.local.yaml.example      # Local override template (committed)
│   ├── ssot-consumer.yaml            # SSOT sync configuration (committed)
│   └── ssot-consumer.local.yaml.example  # Local override template (committed)
├── .crucible/
│   └── metadata/                     # Sync metadata produced by goneat ssot sync
├── docs/
│   └── crucible-<lang>/              # Synced docs (committed, regenerated via sync)
├── schemas/
│   └── crucible-<lang>/              # Synced schemas (committed, regenerated via sync)
├── config/
│   └── crucible-<lang>/              # Synced config defaults (committed, regenerated)
└── bin/                              # Optional vendored tools (gitignored if used)
    └── goneat
```

### Namespace Guidance

- SSOT-specific helpers should live under namespaces such as `crucible/logging`, `crucible/terminal`, `cosmography/maps`.
- Language-only utilities (e.g., Go reflection helpers) may live under `foundation/` namespaces.
- Provide package-level READMEs describing available modules.

## Bootstrap Strategy

### The Problem: Bootstrap the Bootstrap

Foundation libraries face a unique challenge:

- They're dependencies for sophisticated tools (goneat, fulward)
- They need DX tooling themselves (goneat for validation, SSOT sync, and release automation)
- They cannot create circular dependencies
- They must work in CI/CD without manual installation steps

### The Solution: Minimal Goneat Bootstrap + Synced Assets

1. **Commit synced assets** - Docs, schemas, configs from Crucible are committed and regenerated via `make sync`.
2. **Single bootstrap entry** - `.goneat/tools.yaml` ensures goneat is installed (via package manager or download).
3. **Local override pattern** - `.goneat/tools.local.yaml` (gitignored) for development iteration.
4. **Automated sync** - `goneat ssot sync --manifest .goneat/ssot-consumer.yaml` keeps mirror directories current.

### Workflows

**CI/CD (Production):**

```bash
git clone <foundation-repo>  # Includes synced assets
make bootstrap               # Installs/validates goneat
make sync                    # Optional: update from Crucible via goneat ssot sync
make test
```

**Local Development (Iteration):**

```bash
# Copy local override template
cp .goneat/tools.local.yaml.example .goneat/tools.local.yaml

# Edit to point to local goneat build if necessary
# source: ../goneat/dist/goneat

# Bootstrap uses local override (creates symlink)
make bootstrap
```

**Important**: The `type: link` installation method MUST create symbolic links, not copies. This ensures that `bin/goneat` automatically tracks the latest build from `../goneat/dist/goneat` without requiring `make bootstrap-force` after every rebuild.

**Bootstrap Script Requirements for `type: link`:**

- Use `os.symlink()` (Python), `ln -s` (shell), or equivalent
- Remove existing file/symlink before creating new symlink
- Verify source exists and is a file before linking
- Output should indicate symlink creation: `🔗 Creating symlink to <path>`

### Safety: Preventing Local Path Leaks

- Add `.goneat/tools.local.yaml` to `.gitignore`
- Implement precommit hook to validate no local paths in `tools.yaml`
- Provide `tools.local.yaml.example` as template

## Testing Expectations

- Each language foundation owns its unit/integration tests.
- Crucible supplies schemas and config defaults but does not ship tests for the shims.
- Tests should cover:
  - Config path resolution (including legacy fallbacks).
  - Embedding/parsing of Crucible defaults.
  - Schema validation wrappers.
  - Logging severity/middleware mapping.
  - Foundry catalog helpers normalizing alpha-2/alpha-3/numeric country codes with case-insensitive lookups backed by precomputed secondary indexes.
  - Bootstrap script functionality:
    - `type: link` creates symlinks (not copies) for local development
    - `type: download` fetches and verifies checksums for production
    - Symlinks automatically track source updates without re-bootstrap

All helper tests MUST follow the cross-language [Portable Testing Practices](../standards/testing/portable-testing-practices.md). Provide shared capability checks/skip helpers (network, filesystem, DNS) so suites behave deterministically in CI and sandboxed agent environments without hiding real regressions.

## Module Implementation Guidelines

When implementing a new module or capability that requires telemetry, follow this checklist to ensure proper taxonomy coordination:

### Telemetry-Enabled Module Checklist

**Before Implementation**:

1. **Draft Metrics List**
   - Identify all metrics your module will emit
   - Follow naming conventions: `module_operation_unit` (e.g., `pathfinder_find_ms`, `config_load_errors`)
   - Use standard units: `count`, `ms`, `bytes`, `percent`
   - Use semantic suffixes: `_ms` (duration), `_errors` (failures), `_warnings` (non-fatal), `_count` (totals)

2. **Submit Taxonomy Update Request**
   - Create memo in Crucible's `.plans/active/libraries/` directory
   - Format: `YYYYMMDD-<module>-metrics-request.md`
   - Include:
     - List of proposed metrics with names, units, descriptions
     - Business justification for each metric
     - Module context (core vs extension)
     - Impact on other language libraries

3. **Await Crucible Approval**
   - Schema Cartographer reviews request
   - All library teams provide feedback (24-48 hour window)
   - Metrics added to `config/taxonomy/metrics.yaml`
   - Crucible syncs to all lang wrappers

4. **Sync Updated Taxonomy**
   - Pull latest Crucible: `make sync` (or `goneat ssot sync`)
   - Verify new metrics appear in `docs/crucible-<lang>/config/taxonomy/metrics.yaml`

5. **Implement Module**
   - Use approved metric names (exact match required)
   - Emit metrics via library's telemetry module
   - Schema validation will pass on first try

6. **Validate**
   - Verify schema validation passes for emitted metrics
   - Test metric recording and export
   - Document metrics in module's API reference

**Why This Matters**:

- **Cross-library consistency**: All libraries use same metric names
- **Schema validation**: Metrics taxonomy is SSOT, schema references it via `$ref`
- **Prevents rework**: Pre-approval avoids failed validation during implementation
- **Historical tracking**: Memos provide audit trail for metric additions

**Example Workflow**:

```
1. GoFulmen plans Pathfinder module with 4 metrics
2. Creates .plans/active/libraries/20251024-pathfinder-metrics-request.md
3. Schema Cartographer reviews, gets team feedback
4. Crucible updated with new metrics (30 min)
5. GoFulmen syncs Crucible, sees new metrics in taxonomy
6. GoFulmen implements, schema validation passes ✅
```

**Common Mistakes to Avoid**:

- ❌ Implementing first, requesting taxonomy update after (causes validation failures)
- ❌ Using custom metric names not in taxonomy (schema rejects them)
- ❌ Assuming pyfulmen/tsfulmen metrics will work for gofulmen (taxonomy may be stale)
- ✅ Following this checklist ensures smooth implementation

**Reference**: See `.plans/active/libraries/20251024-telemetry-schema-fix.md` for real-world example of this process.

## Documentation Requirements

- README with installation, quick start, links to Crucible standards, **and a prominent link to the overview document described below.**
- **Crucible Version Section**: README MUST include a dedicated section (e.g., "Crucible Version") explaining how to determine the embedded Crucible version. This section MUST:
  1. Show code example using the Crucible Shim API (e.g., `crucible.get_version()`, `crucible.Version()`, or equivalent)
  2. Explain the metadata fields: `version` (CalVer), `commit`, `dirty` flag, `syncedAt`, `syncMethod`
  3. Link to `.crucible/metadata.yaml` location for manual inspection
  4. Note that `dirty: true` indicates sync from uncommitted changes (development workflow)
- Goneat bootstrap notes (or reference this standard / `docs/guides/bootstrap-goneat.md`).
- Bootstrap strategy documentation (e.g., `docs/BOOTSTRAP-STRATEGY.md`).
- API reference comments/docstrings per language norms.
- Notes on dependency flow (SSOT → foundation → consumer) to prevent circular imports.
- **Crucible Overview**: Every library's overview document MUST include a "Crucible Overview" section explaining (1) what Crucible is, (2) why the shim/docscribe module matters, and (3) where to learn more. This gives downstream users essential context without spawning additional top-level documentation.
- Architecture overview stored at `docs/<lang>fulmen_overview.md` (for example, `docs/pyfulmen_overview.md`). Use the following template to ensure consistency:

  ```markdown
  # <Language> Fulmen Overview

  ## Purpose & Scope

  - Brief summary of the foundation library and supported environments.

  ## Crucible Overview

  **What is Crucible?**

  Crucible is the FulmenHQ single source of truth (SSOT) for schemas, standards, and configuration templates. It ensures consistent APIs, documentation structures, and behavioral contracts across all language foundations (gofulmen, pyfulmen, tsfulmen, etc.).

  **Why the Shim & Docscribe Module?**

  Rather than copying Crucible assets into every project, helper libraries provide idiomatic access through shim APIs. This keeps your application lightweight, versioned correctly, and aligned with ecosystem-wide standards. The docscribe module lets you discover, parse, and validate Crucible content programmatically without manual file management.

  **Where to Learn More:**

  - [Crucible Repository](https://github.com/fulmenhq/crucible) - SSOT schemas, docs, and configs
  - [Fulmen Technical Manifesto](../crucible-<lang>/architecture/fulmen-technical-manifesto.md) - Philosophy and design principles
  - [SSOT Sync Standard](../crucible-<lang>/standards/library/modules/ssot-sync.md) - How libraries stay synchronized

  ## Module Catalog

  | Module          | Tier | Summary                                                                                 | Spec Link                                                       |
  | --------------- | ---- | --------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
  | config-path-api | Core | Platform-aware config/data/cache helpers                                                | [config-path-api](standards/library/modules/config-path-api.md) |
  | docscribe       | Core | Access and processing of Crucible documentation assets including frontmatter extraction | [docscribe](standards/library/modules/docscribe.md)             |
  | ...             | ...  | ...                                                                                     | ...                                                             |

  ## Observability & Logging Integration

  - Default logging profile, policy search order, notable middleware/sinks enabled by default.

  ## Dependency Map

  | Artifact          | Description                | Source             |
  | ----------------- | -------------------------- | ------------------ |
  | Crucible schemas  | Synced SSOT assets         | `goneat ssot sync` |
  | Published package | e.g., `@fulmenhq/tsfulmen` | npm (link)         |
  | Optional extras   | e.g., telemetry plugins    | link               |

  ## Roadmap & Known Gaps

  - Bullet list of planned enhancements or known limitations.
  ```

- **Asset Catalog**: Document available Crucible assets with examples of shim API usage
- **Integration Examples**: Code samples showing common tool integration patterns (see Ecosystem Tool Integration section)
- **Version Compatibility**: Guidance on handling Crucible version changes
- **Docscribe Integration**: Demonstrate how the standalone `docscribe` module is used alongside crucible-shim for frontmatter parsing, header extraction, and outline generation.

- Development operations documentation located under `docs/development/`. Every library MUST provide:

  ```markdown
  docs/
  ├── <lang>fulmen*overview.md
  ├── development/
  │ ├── README.md # Index linking to operations/testing/bootstrap docs
  │ └── operations.md # Required operations guide
  └── crucible*<lang>/ # Synced Crucible docs (as today)
  ```

  `docs/development/operations.md` MUST cover:
  1. Development workflow (primary `make` targets, lint/typecheck commands)
  2. Release process (versioning strategy, changelog expectations, required checks)
  3. Testing strategy (coverage targets from the module manifest, tooling)
  4. Tooling reference (bootstrap, goneat usage, language-specific helpers)
  5. Community & support notes (contribution guidelines, communication channels)
  6. Security expectations (dependency scanning, vulnerability reporting)

  Libraries MAY add additional documents (`testing.md`, `release-process.md`, `bootstrap.md`, etc.) referenced from `docs/development/README.md`.

## Architecture Decision Records (ADRs)

Helper libraries operate under a two-tier ADR system to track both ecosystem-wide and language-specific architectural decisions. This system enables consistent decision tracking while allowing flexibility for language-specific implementations.

### Tier 1: Ecosystem ADRs (from Crucible)

**Location**: `docs/crucible-{lang}/architecture/decisions/`

**Source**: Synced from Crucible SSOT via `make sync`

**Scope**: Cross-language patterns, contracts, and foundational decisions that affect multiple libraries

**Status**: Read-only reference - propose changes upstream to Crucible

**Examples**:

- ADR-0001: Triple-Index Catalog Strategy
- ADR-0002: Progressive Logging Profiles
- ADR-0003: Schema-Driven Config Hydration
- ADR-0004: CamelCase to Language Convention Mapping

**Lifecycle Management**: Ecosystem ADRs use the [ADR Lifecycle Status Schema](https://schemas.fulmenhq.dev/standards/adr-lifecycle-status-v1.0.0.json) with stages from `proposal` through `stable` to `deprecated/superseded/retired`. Each ADR includes adoption tracking showing implementation status across all language libraries.

### Tier 2: Local ADRs (library-specific)

**Location**: `docs/development/adr/`

**Scope**: Language/implementation-specific decisions that don't affect other languages

**Maintained By**: Library maintainers (not synced from Crucible)

**Examples**:

- gofulmen: `ADR-0001-use-sync-pool-for-event-buffers.md`
- pyfulmen: `ADR-0001-fulmencatalogmodel-populate-by-name.md`
- tsfulmen: `ADR-0001-use-proxy-for-lazy-loading.md`

**Naming Convention**: All ADRs use consistent `ADR-XXXX-kebab-case-title.md` filename format and `id: "ADR-XXXX"` frontmatter, regardless of scope. Location determines whether it's ecosystem or local.

See [Crucible ADR README](https://github.com/fulmenhq/crucible/blob/main/docs/architecture/decisions/README.md) for complete guide including lifecycle stages, adoption tracking, and when to promote local decisions to ecosystem ADRs.

### Directory Structure

```
docs/
├── <lang>fulmen_overview.md      # Library overview
├── development/                   # LOCAL: Library-specific documentation
│   ├── README.md                  # Index linking to operations/testing/ADR docs
│   ├── operations.md              # Required operations guide
│   └── adr/                       # LOCAL: Library-specific ADRs
│       ├── README.md              # ADR index, references ecosystem ADRs
│       ├── ADR-0001-title.md      # Local ADRs (library-maintained)
│       └── ...
└── crucible-{lang}/               # SYNCED: From Crucible SSOT (read-only)
    ├── standards/                 # Synced standards
    ├── config/                    # Synced config defaults
    └── architecture/
        └── decisions/             # Ecosystem ADRs (reference only)
            ├── README.md
            ├── template.md
            └── ADR-0001-*.md      # Ecosystem ADRs (all use ADR- prefix)
```

### Path Ownership

- `docs/development/` - **Maintained by library team**, not synced from Crucible
- `docs/crucible-{lang}/` - **Synced from Crucible**, do not edit directly

### Local ADR Requirements

**README.md** (in `docs/development/adr/`) MUST include:

1. ADR index table with ID, title, status, date
2. When to write a local ADR vs. promoting to ecosystem
3. Link to ecosystem ADRs in `docs/crucible-{lang}/architecture/decisions/`
4. Contribution guidelines for proposing new ADRs

**When to Write Local ADR**:

- ✅ Implementation detail unique to language
- ✅ Tooling/dependency choice (e.g., which JSON library)
- ✅ Performance optimization specific to runtime
- ✅ Language idiom preference
- ✅ Test framework choice
- ✅ Build/packaging decisions

**When to Promote to Ecosystem ADR**:

- ✅ Decision affects API contracts between languages
- ✅ Pattern must be consistent across Go/Python/TypeScript
- ✅ Schema structure or field naming is involved
- ✅ Other languages must implement same behavior

**ADR Format**: Use Crucible standard template from `docs/crucible-{lang}/architecture/decisions/template.md`

### Adoption Tracking

Libraries SHOULD track their implementation status for ecosystem ADRs in the library's local documentation. This enables ecosystem-wide visibility into ADR adoption progress.

**Example** (`docs/development/adr/ecosystem-adoption-status.md`):

| Ecosystem ADR                  | Status      | Notes                                   | Related Local ADRs             |
| ------------------------------ | ----------- | --------------------------------------- | ------------------------------ |
| ADR-0001: Triple-Index Catalog | verified    | Implemented with sync.Pool optimization | ADR-0003-sync-pool-buffers.md  |
| ADR-0002: Progressive Logging  | implemented | SIMPLE/STRUCTURED profiles complete     | ADR-0005-structured-logging.md |
| ADR-0003: Schema-Driven Config | in-progress | Layer 1 & 2 done, Layer 3 pending       | -                              |
| ADR-0004: CamelCase Mapping    | planned     | Scheduled for v0.5.0                    | -                              |

**Adoption Status Values** (from [ADR Adoption Status Schema](https://schemas.fulmenhq.dev/standards/adr-adoption-status-v1.0.0.json)):

- `not-applicable` (0): Does not apply to this library
- `deferred` (5): Postponed with documented rationale
- `planned` (10): Implementation planned but not started
- `in-progress` (20): Active implementation underway
- `implemented` (30): Fully implemented, ready for validation
- `verified` (40): Implemented and validated through tests/production use

### Cross-Referencing

When a local ADR relates to an ecosystem ADR, include a cross-reference using your library's language-specific path:

```markdown
## Related Ecosystem ADRs

- [ADR-0001: Triple-Index Catalog Strategy](../crucible-{lang}/architecture/decisions/ADR-0001-triple-index-catalog-strategy.md)

This local decision implements the ecosystem strategy using {language}-specific patterns.
```

**Path is Language-Specific**:

- gofulmen: `../crucible-go/architecture/decisions/`
- pyfulmen: `../crucible-py/architecture/decisions/`
- tsfulmen: `../crucible-ts/architecture/decisions/`

Each library sees Crucible content synced to its own language namespace, so always use the path matching your library's language.

### Promotion Path

When a local ADR reveals cross-language impact during implementation or review:

1. **Recognize**: Decision affects other language implementations
2. **Propose**: Create proposal in Crucible `.plans/` referencing the local ADR
3. **Coordinate**: Discuss in #fulmen-architecture channel, get buy-in from other library maintainers
4. **Promote**: Create ecosystem ADR in Crucible `docs/architecture/decisions/` with next available ADR-XXXX number
5. **Update Local ADR**: Mark as "Superseded by [ADR-XXXX]" with clear link
6. **Sync**: Run `make sync` in Crucible to propagate to all language wrappers

See `.plans/active/2025.10.2/library-adr-brief.md` for complete promotion workflow and lifecycle management details.

## Version Alignment

- Foundations MUST pin the Crucible version they embed and expose it publicly.
- Consumers depend on the foundation version; no direct Crucible import required.
- When Crucible publishes new assets, foundations should sync and bump versions in tandem.
- Use the repo-provided CalVer automation (for example `bun run scripts/update-version.ts --bump <patch|minor|major>`) to keep artifacts aligned.
- Declare repository maturity via `LIFECYCLE_PHASE`, following the [Repository Lifecycle Standard](../standards/repository-lifecycle.md). Coverage gates and CI policies MUST respect the thresholds defined for each phase.

## Makefile Targets (Required)

In addition to the standard targets, support overriding bootstrap with `FORCE=1` or provide a `bootstrap-force` alias so tool reinstall can be forced when iterating on local builds.

All foundation libraries MUST provide these targets:

```makefile
bootstrap:      # Install or verify goneat availability
sync:           # Sync assets from Crucible SSOT via goneat
version-bump:   # Bump version (requires TYPE/BUMP parameter)
test:           # Run all tests
fmt:            # Format code
lint:           # Lint/style checks
```

## Common Pitfalls & Solutions

### Pitfall 1: Syncing Only JSON Schemas

- **Issue**: Using `schemas/**/*.json` misses `.yaml` schemas.
- **Solution**: Use `schemas/**/*` or `schemas/**/*.{json,yaml}`.

### Pitfall 2: Outdated Goneat Version

- **Issue**: Older goneat binaries may lack `ssot sync` enhancements.
- **Solution**: Re-run `make bootstrap` or reinstall goneat (`go install ...@latest`, `brew install goneat`, etc.) before syncing.

### Pitfall 3: Local Overrides Committed

- **Issue**: `.goneat/tools.local.yaml` accidentally committed.
- **Solution**: Provide `.goneat/tools.local.yaml.example`, add `.goneat/tools.local.yaml` to `.gitignore`, enforce pre-commit checks.

### Pitfall 4: Circular Bootstrap Dependencies

- **Issue**: Using project-internal tools before goneat is installed can create cycles.
- **Solution**: Keep bootstrap focused on ensuring goneat is available; delegate other tooling installs to repo-specific scripts executed after goneat is ready.

### Pitfall 5: Missing Maintainer Docs

- **Issue**: Ops/ADR info lost in commits.
- **Solution**: Maintain `ops/` directory with ADRs, runbooks, bootstrap strategy.

### Pitfall 6: Bootstrap Script Copying Instead of Symlinking

- **Issue**: Bootstrap script copies tools instead of symlinking when using `type: link`, causing stale binaries that don't reflect rebuilds.
- **Symptom**: After rebuilding goneat locally, `bin/goneat --version` shows old version. Must re-run `make bootstrap` or `make bootstrap-force` after every rebuild.
- **Solution**: Implement proper symlink creation:
  - Python: Use `os.symlink(src, dest)` instead of `shutil.copy()`
  - Shell: Use `ln -s "$source" "$dest"` instead of `cp`
  - Verify existing file/link is removed before creating symlink
  - Check source exists before creating symlink
- **Verification**: `ls -la bin/goneat` should show symlink arrow `bin/goneat -> ../goneat/dist/goneat`
- **Test**: After creating symlink, rebuild goneat in source location - `bin/goneat --version` should immediately reflect new version without re-bootstrap

## References

- [Fulmen Library Ecosystem](library-ecosystem.md)
- [Fulmen Config Path Standard](../standards/config/fulmen-config-paths.md)
- [Config Defaults README](../../config/README.md)
- [Goneat Repository](https://github.com/fulmenhq/goneat)
- `.plans/crucible/fulmen-helper-library-specification.md` (original proposal)

## Implementation Examples

- **gofulmen** - Reference implementation with Go-based bootstrap
- **tsfulmen** - TypeScript implementation (in development)

## Changelog

- **2025-10-10** - Migrated bootstrap + sync guidance from FulDX to goneat
- **2025-10-07** - Added FulDX bootstrap pattern, SSOT sync requirements, directory structure
- **2025-10-03** - Initial draft
