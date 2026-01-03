---
title: "Crucible Sync Model Architecture"
description: "Architecture decision record for how Crucible distributes schemas, docs, and config to downstream consumers"
author: "@3leapsdave"
date: "2025-10-02"
last_updated: "2025-11-07"
status: "approved"
tags: ["architecture", "adr", "sync", "distribution", "fuldx"]
---

# Crucible Sync Model Architecture

## Status

**Approved** - 2025-10-02  
**Updated** - 2025-10-28 (Added Python support, TypeScript scripts, modernized tooling)

## Context

Crucible serves as the single source of truth (SSOT) for schemas, standards, config defaults, and documentation across the FulmenHQ ecosystem. We needed to decide how downstream repositories (gofulmen, tsfulmen, pyfulmen, forge repos, tools) would consume these assets.

**Current Implementation**: Dual distribution model—published packages for runtime use, pull scripts for build-time integration. See [Pull Script README](../../scripts/pull/README.md) for usage.

### Requirements

1. **Flexibility**: Downstream repos need control over what they sync and when
2. **Versioning**: Clear version coordination between Crucible and consumers
3. **Performance**: Minimal overhead for consumers (don't force full repo as dependency)
4. **Transparency**: Consumers should see what they're getting
5. **Cross-language**: Support Go, TypeScript, and future languages equally
6. **Offline capability**: Built artifacts should work without network access
7. **Maintainability**: Easy to update and verify sync state

### Constraints

- Crucible uses CalVer versioning (`YYYY.0M.MICRO`)
- Assets have independent lifecycles (schemas can update without affecting standards)
- Some consumers want vendored/committed assets, others prefer gitignored
- Git submodules are too heavyweight and problematic
- Pure CDN/API approach doesn't work for build-time embedding

## Decision

We implement a **dual distribution model** for different consumption patterns:

### 1. Published Packages (Primary for Runtime Use)

**Go Package**: `github.com/fulmenhq/crucible`

```go
import "github.com/fulmenhq/crucible"

schema := crucible.Schemas.Terminal.V1_0_0()
```

**TypeScript Package**: `@fulmenhq/crucible`

```typescript
import { schemas } from "@fulmenhq/crucible";

const schema = schemas.terminal.v1_0_0;
```

**Python Package**: `fulmenhq-crucible`

```python
from crucible import schemas

schema = schemas.terminal.v1_0_0
```

**Characteristics:**

- Monolithic package with internal organization
- Schemas/docs/config embedded at build time
- Versioned with CalVer matching repo VERSION
- Published to standard registries (Go modules, npm, PyPI)
- Tree-shakeable in modern bundlers

### 2. Pull Script (Primary for Build-Time Asset Integration)

**Bun/TypeScript Script**: `scripts/pull/crucible-pull.ts`

```bash
# Copy to your downstream repo
cp scripts/pull/crucible-pull.ts <your-repo>/scripts/

# Pull assets (selective sync supported)
bun run scripts/crucible-pull.ts --schemas terminal
bun run scripts/crucible-pull.ts --version=2025.10.0
```

**Config File Example** (`.crucible-sync.json`):

```json
{
  "version": "2025.10.0",
  "output": ".crucible",
  "include": {
    "schemas": ["terminal", "pathfinder"],
    "docs": ["standards/coding/go"],
    "templates": []
  },
  "gitignore": true
}
```

**Characteristics:**

- Copy-and-adapt model (customize in your repo)
- Selective sync of assets
- Version pinning support
- Config file driven
- Supports both gitignored and committed patterns
- Explicit version tracking via `.crucible-version` file

## Architecture

### Crucible Repository Structure

```
crucible/
├── VERSION                      # CalVer: 2025.10.0
├── schemas/                     # SSOT schemas
│   ├── terminal/v1.0.0/
│   ├── pathfinder/v1.0.0/
│   └── config/
├── docs/                        # SSOT documentation
│   ├── standards/
│   ├── guides/
│   └── architecture/
├── config/                      # SSOT configuration defaults
│   ├── sync/
│   │   └── sync-keys.yaml      # Registered sync keys for FulDX
│   ├── library/
│   └── terminal/
├── lang/                        # Language wrappers (internal)
│   ├── go/                     # Go package
│   │   ├── schemas/            # Synced from root
│   │   ├── docs/               # Synced from root
│   │   ├── config/             # Synced from root
│   │   ├── schemas.go          # Embedded access
│   │   └── go.mod
│   ├── typescript/             # TypeScript package
│   │   ├── schemas/            # Synced from root
│   │   ├── docs/               # Synced from root
│   │   ├── config/             # Synced from root
│   │   └── package.json
│   └── python/                 # Python package
│       ├── schemas/            # Synced from root
│       ├── docs/               # Synced from root
│       ├── config/             # Synced from root
│       └── pyproject.toml
└── scripts/
    ├── sync-to-lang.ts         # Internal: sync root → lang/
    ├── bootstrap-tools.ts      # Installs goneat/fuldx to ./bin/
    └── pull/
        └── crucible-pull.ts    # Legacy reference (use FulDX instead)
```

### Build Process

**Pre-publish (internal to crucible):**

1. **Bootstrap tools** (if needed):
   ```bash
   make bootstrap
   # Installs goneat and other tools to ./bin/
   # See docs/guides/bootstrap-goneat.md for details
   ```
2. **Update root assets** (schemas/, docs/, config/)
3. **Bump VERSION** to new SemVer
4. **Sync to language wrappers** (two-stage pipeline):
   ```bash
   make sync
   # Runs: bun run scripts/sync-to-lang.ts
   #
   # STAGE 1 - SNAPSHOT GENERATION:
   # - Regenerates JSON snapshots from YAML catalogs
   # - config/library/foundry/exit-codes.snapshot.json (from exit-codes.yaml)
   # - config/library/foundry/simplified-modes.snapshot.json
   # - Ensures snapshots are fresh before codegen
   #
   # STAGE 2 - CODE GENERATION & SYNC:
   # - Generates language-native bindings from fresh snapshots
   # - foundry/exit_codes.go
   # - lang/python/src/crucible/foundry/exit_codes.py
   # - lang/typescript/src/foundry/exitCodes.ts
   # - Copies schemas/ → lang/go/schemas/ (via go:embed, no file copy)
   # - Copies schemas/ → lang/typescript/schemas/
   # - Copies schemas/ → lang/python/schemas/
   # Copies docs/ → lang/go/docs/
   # Copies docs/ → lang/typescript/docs/
   # Copies docs/ → lang/python/docs/
   # Copies config/ → lang/go/config/
   # Copies config/ → lang/typescript/config/
   # Copies config/ → lang/python/config/
   ```
5. **Embed in packages**:
   - Go: `//go:embed` directives compile schemas into binary
   - TypeScript: Build process bundles schemas as constants/JSON
6. **Publish packages**:

   ```bash
   # Go: automatic via GitHub (git tag triggers module update)
   git tag v2025.10.0
   git push origin v2025.10.0

   # TypeScript: publish to npm
   cd lang/typescript
   npm publish
   ```

### Consumption Patterns

**Pattern A: Use Published Package (Runtime)**

```typescript
// In gofulmen, forge repos, or application code
import { schemas } from "@fulmenhq/crucible";

// Schemas are embedded, no file I/O needed
const terminalSchema = schemas.terminal.v1_0_0;
```

**Best for**: Runtime schema access, type generation, validation in application code.

**Pattern B: Use Pull Script (Build-Time Integration)**

```bash
# In downstream repo (gofulmen, tsfulmen, forge repos)

# 1. Copy pull script (first time)
cp scripts/pull/crucible-pull.ts <your-repo>/scripts/

# 2. Configure what to sync (.crucible-sync.json)
# See scripts/pull/README.md

# 3. Pull assets from Crucible SSOT
bun run scripts/crucible-pull.ts --version=2025.10.0
```

**Best for**: Syncing docs, config defaults, or schema source files for code generation, keeping local copies in sync with Crucible releases.

**Pattern C: Hybrid (Recommended for Helper Libraries)**

```go
// In gofulmen: use package for runtime access
import "github.com/fulmenhq/crucible"

// Runtime schemas embedded in package
schema := crucible.Schemas.Terminal.V1_0_0()
```

```bash
# Also use pull script to sync source assets for:
# - Code generation from schemas
# - Keeping docs in sync
# - Validating against SSOT during development
bun run scripts/crucible-pull.ts --schemas --docs
```

**Best for**: Helper libraries that both embed Crucible at runtime AND need source assets for development/code-gen.

## Consequences

### Positive

✅ **Flexibility**: Consumers choose package or pull script based on needs  
✅ **Performance**: Embedded assets = zero runtime I/O  
✅ **Versioning**: Clear CalVer snapshots with `.crucible-version` tracking  
✅ **Transparency**: Pull scripts show exactly what's synced  
✅ **Maintainability**: Single source in root, synced to lang wrappers  
✅ **Offline**: Published packages work without network after install  
✅ **Language agnostic**: Same patterns for Go, TypeScript, future languages

### Negative

⚠️ **Duplication**: Root assets duplicated in lang/ directories  
⚠️ **Sync burden**: Must remember to sync root → lang/ before publish  
⚠️ **Package size**: Full schemas/docs embedded increases package size  
⚠️ **Update lag**: Pull script users must manually update

### Mitigations

**For duplication:**

- Automated sync script reduces manual work
- CI validates sync state before publish

**For sync burden:**

- Pre-publish checklist enforces sync
- Git hooks can automate sync on VERSION bump

**For package size:**

- Tree-shaking helps in modern bundlers
- Future: Separate packages for schemas-only if needed

**For update lag:**

- Automated update PRs (Dependabot-style)
- Clear communication of updates via CHANGELOG

## Internal Sync Process (Crucible Repository)

The internal sync process keeps Crucible's language wrappers (`lang/go/`, `lang/typescript/`, `lang/python/`) in sync with root SSOT assets. This is distinct from **external downstream sync** (how consumers like gofulmen pull from Crucible).

### Bootstrap Tools First

Before syncing, ensure tools are installed:

```bash
make bootstrap
# Installs goneat, fuldx, and other tools to ./bin/
# See docs/guides/bootstrap-goneat.md for details
```

### scripts/sync-to-lang.ts

```bash
# Run via Make target
make sync

# Or directly
bun run scripts/sync-to-lang.ts
```

This TypeScript script:

- Copies `schemas/` → `lang/{go,typescript,python}/schemas/`
- Copies `docs/` → `lang/{go,typescript,python}/docs/`
- Copies `config/` → `lang/{go,typescript,python}/config/`
- Preserves directory structure and file metadata

**When to run:**

- Before bumping VERSION
- Before publishing packages
- Automatically via `make precommit` or `make prepush`
- In CI/CD validation workflows

**Why TypeScript, not shell?**

- Cross-platform (Windows, macOS, Linux)
- Safer error handling and path manipulation
- Consistent with other Crucible tooling (bootstrap, validation)

### Embedding Strategy

**Go (compile-time):**

```go
package crucible

import _ "embed"

//go:embed schemas/terminal/v1.0.0/schema.json
var terminalSchemaV1 []byte

func GetTerminalSchemaV1() ([]byte, error) {
    return terminalSchemaV1, nil
}
```

**TypeScript (build-time):**

```typescript
// Generated during build
import terminalSchemaV1 from "./schemas/terminal/v1.0.0/schema.json";

export const schemas = {
  terminal: {
    v1_0_0: terminalSchemaV1,
  },
};
```

## Versioning Strategy

### Repository Version (VERSION file)

```
2025.10.0
```

**Meaning**: Snapshot of all assets at October 2025, revision 0

### Package Versions

```
github.com/fulmenhq/crucible v2025.10.0
@fulmenhq/crucible@2025.10.0
```

**Synchronized with repository VERSION**

### Asset Versions (in frontmatter)

```yaml
---
title: "Terminal Schema"
version: "v1.0.0"
revision: 3
last_updated: "2025-10-15"
---
```

**Independent of repository VERSION**

### Coordination

- Repository version = release snapshot date
- Asset versions = semantic versions of individual artifacts
- Pull script creates `.crucible-version` file for tracking
- Packages expose VERSION constant

## Alternative Approaches Considered

### Git Submodules

**Rejected**: Too heavyweight, difficult to version pin, problematic workflows

### Monorepo with Workspaces

**Rejected**: Doesn't solve cross-repo consumption, increases coupling

### Pure CDN/API Distribution

**Rejected**: Requires network at runtime, complicates offline builds

### Separate Package per Asset Type

**Rejected**: Too much overhead, version coordination complexity

### Go Proxy + npm Registry Only

**Rejected**: Doesn't serve pull script use case, forces package dependency

## Future Considerations

### Multi-Package Strategy (if needed)

If package size becomes problematic:

```
@fulmenhq/crucible-schemas  # Schemas only
@fulmenhq/crucible-docs     # Docs only
@fulmenhq/crucible          # Full bundle (depends on above)
```

### Shell Pull Script

Add `scripts/pull/crucible-pull.sh` for Go developers who prefer shell.

### Automated Sync Validation

CI workflow to ensure root → lang/ sync is current:

```yaml
- name: Validate sync
  run: |
    scripts/sync-to-lang.sh
    git diff --exit-code lang/
```

### Asset Registry

Future: Machine-readable registry of available assets with metadata:

```json
{
  "schemas": {
    "terminal": {
      "versions": ["v1.0.0"],
      "path": "schemas/terminal"
    }
  }
}
```

## References

- [Pull Script README](../../scripts/pull/README.md) - Reference implementation for downstream sync
- [Bootstrap Goneat Guide](../guides/bootstrap-goneat.md) - Installing goneat tooling
- [Repository Versioning Standard](../standards/repository-versioning.md) - CalVer strategy

## Related Standards

- [Repository Structure SOP](../sop/repository-structure.md)
- [Repository Version Adoption SOP](../sop/repository-version-adoption.md)

---

**Decision Date**: 2025-10-02  
**Status**: Approved  
**Author**: @3leapsdave  
**Reviewers**: N/A (Initial architecture)
