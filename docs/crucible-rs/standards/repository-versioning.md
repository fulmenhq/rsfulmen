---
title: "Repository Versioning Standard"
description: "Versioning strategies for FulmenHQ repositories with SemVer and CalVer implementations"
author: "@3leapsdave"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "approved"
tags: ["standards", "versioning", "semver", "calver"]
---

# Repository Versioning Standard

## Overview

FulmenHQ repositories use standardized versioning strategies to provide clear, predictable release management. This standard defines two supported approaches: **Semantic Versioning (SemVer)** and **Calendar Versioning (CalVer)**, along with implementation patterns using a canonical `VERSION` file as the single source of truth.

## Supported Versioning Strategies

### 1. Semantic Versioning (SemVer)

**Format**: `MAJOR.MINOR.PATCH` (e.g., `1.2.3`)

**When to use:**

- Libraries and packages with APIs
- Tools with clear breaking/non-breaking change boundaries
- Projects where dependency resolution matters
- Code that consumers import or depend on

**Version semantics:**

- **MAJOR**: Breaking changes (incompatible API changes)
- **MINOR**: New features (backward-compatible additions)
- **PATCH**: Bug fixes (backward-compatible fixes)

**Examples:**

- `gofulmen` - Go library package
- `@fulmenhq/tsfulmen` - TypeScript library package
- `goneat` - CLI tool with public API
- `fulward` - Infrastructure protection tool

**References:**

- [SemVer 2.0.0 Specification](https://semver.org)

### 2. Calendar Versioning (CalVer)

**Format**: `YYYY.0M.MICRO` (e.g., `2025.10.0`)

**When to use:**

- Documentation repositories
- Information architecture (schemas, standards, templates)
- Content-driven projects
- Projects where temporal context matters more than API compatibility

**Version semantics:**

- **YYYY**: Four-digit year
- **0M**: Zero-padded month (01-12)
- **MICRO**: Incremental release counter (starts at 0)
  - `.0` = Primary release of the month
  - `.1+` = Hotfixes or rapid iterations within the month

**Examples:**

- `fulmen-cosmography` - Galaxy taxonomy and documentation

**Note**: Crucible originally used CalVer but transitioned to SemVer (v0.2.0) for Go module compatibility. See [ADR-0010](../architecture/decisions/ADR-0010-semantic-versioning-adoption.md) for the decision rationale.

**Benefits over SemVer for content:**

- No need to define "breaking changes" in documentation
- Temporal clarity: "I'm using the October 2025 standards"
- Natural fit for living documentation
- Avoids artificial semantic interpretation

## VERSION File Pattern

All FulmenHQ repositories MUST maintain a `VERSION` file in the repository root as the single source of truth.

### File Format

```
2025.10.0
```

**Requirements:**

- Single line containing the version string
- No prefix (no `v` character)
- No trailing newline or whitespace
- Must be valid according to chosen strategy

### Reading VERSION

**Shell:**

```bash
VERSION=$(cat VERSION)
echo "Current version: $VERSION"
```

**Bun/TypeScript:**

```typescript
import { readFileSync } from "fs";
const version = readFileSync("VERSION", "utf-8").trim();
```

**Go:**

```go
import "os"

version, err := os.ReadFile("VERSION")
if err != nil {
    panic(err)
}
versionStr := strings.TrimSpace(string(version))
```

### Updating VERSION

**❌ NEVER edit VERSION file manually**

Use version management scripts (see Implementation Patterns below).

## Pre-Release Versions

Both strategies support pre-release versions:

**SemVer:**

```
1.2.3-rc.1
1.2.3-beta.1
1.2.3-alpha.1
```

**CalVer:**

```
2025.10.0-rc.1
2025.10.0-beta.1
```

**Git tags only:**

- Pre-release versions appear in git tags: `v2025.10.0-rc.1`
- VERSION file contains only the target release: `2025.10.0`
- Published packages use pre-release suffix: `@fulmenhq/crucible@2025.10.0-rc.1`

## Implementation Patterns

### Pattern A: Script-Based (Recommended)

**Directory structure:**

```
scripts/
  version.ts          # Version bump commands
  embed-version.ts    # Embed version in source files
```

**Package scripts:**

```json
{
  "scripts": {
    "version:bump:major": "bun run scripts/version.ts bump major",
    "version:bump:minor": "bun run scripts/version.ts bump minor",
    "version:bump:patch": "bun run scripts/version.ts bump patch",
    "version:set": "bun run scripts/version.ts set",
    "version:embed": "bun run scripts/embed-version.ts"
  }
}
```

**Usage:**

```bash
# Bump version
bun run version:bump:patch    # 1.0.0 → 1.0.1 (SemVer)
bun run version:bump:patch    # 2025.10.0 → 2025.10.1 (CalVer)

# Set specific version
bun run version:set 2025.10.0

# Embed in source files
bun run version:embed
```

### Pattern B: Version Library Integration

FulmenHQ maintains version management libraries in sibling repositories:

**Go version library:**

```go
import "github.com/fulmenhq/gofulmen/version"

// Read VERSION file
v, err := version.ReadVersionFile()

// Bump version
newV, err := version.Bump(v, version.Patch)

// Write VERSION file
err = version.WriteVersionFile(newV)
```

**TypeScript version library:**

```typescript
import {
  readVersion,
  bumpVersion,
  writeVersion,
} from "@fulmenhq/tsfulmen/version";

// Read VERSION file
const v = await readVersion();

// Bump version
const newV = bumpVersion(v, "patch");

// Write VERSION file
await writeVersion(newV);
```

## Version Coordination

### Repository VERSION vs Asset Versions

**For content repositories (CalVer):**

- Repository VERSION: `2025.10.2` (coordination version)
- Individual assets track their own revisions via frontmatter:

```yaml
---
title: "Terminal Schema"
version: "v1.0.0" # Schema version
revision: 3 # Revision number
last_updated: "2025-10-15" # Last changed date
---
```

**Publishing behavior:**

- Repository version indicates "release snapshot"
- All assets published together, even if unchanged
- Consumers get atomic snapshot: "everything as of 2025.10.2"
- CHANGELOG documents which assets changed

### Git Tagging

**Format:**

```
v{VERSION}
```

**Examples:**

```bash
git tag v1.2.3        # SemVer
git tag v2025.10.0    # CalVer
```

**Tag creation:**

```bash
# Automated (recommended)
bun run release         # Creates tag from VERSION file

# Manual
VERSION=$(cat VERSION)
git tag "v${VERSION}"
git push origin "v${VERSION}"
```

## Development Workflow

### Standard Release Flow

```bash
# 1. Bump version
bun run version:bump:patch      # or minor/major

# 2. Embed version in source files (if applicable)
bun run version:embed

# 3. Build
bun run build

# 4. Test
bun run test

# 5. Commit
git add VERSION package.json src/  # Include embedded files
git commit -m "chore: bump version to $(cat VERSION)"

# 6. Tag
git tag "v$(cat VERSION)"

# 7. Push
git push origin main --tags
```

### Hotfix Flow (CalVer)

```bash
# Released 2025.10.0, discovered issue

# 1. Fix the issue
# 2. Bump micro version
bun run version:set 2025.10.1

# 3. Standard release flow
bun run build && bun run test
git commit -am "fix: critical issue in schema validation"
git tag v2025.10.1
git push origin main --tags
```

## Migration Guide

### Adopting This Standard

**For new repositories:**

1. Choose SemVer or CalVer based on repository type
2. Create `VERSION` file with initial version
3. Implement version scripts using patterns above
4. Document choice in `CONTRIBUTING.md`

**For existing repositories:**

1. Audit current version locations (package.json, etc.)
2. Create VERSION file with current version
3. Implement sync scripts to read from VERSION
4. Update CI/CD to use VERSION file
5. Document migration in CHANGELOG

## Quality Gates

### Pre-Commit Requirements

- [ ] VERSION file exists and is valid
- [ ] All version references are synchronized
- [ ] Version follows chosen strategy (SemVer or CalVer)
- [ ] Git tag matches VERSION (on release)

### CI/CD Validation

```bash
# Version consistency check
if [ "$(cat VERSION)" != "$(jq -r .version package.json)" ]; then
  echo "❌ Version mismatch between VERSION and package.json"
  exit 1
fi

# Version format validation
VERSION=$(cat VERSION)
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "❌ Invalid version format"
  exit 1
fi
```

## Related Standards

- [Frontmatter Standard](frontmatter-standard.md) - Document metadata and revision tracking
- [Agentic Attribution Standard](agentic-attribution.md) - AI agent contributions
- [Repository Version Adoption SOP](../sop/repository-version-adoption.md) - Mandatory adoption process

## Examples from Ecosystem

### SemVer Implementation: Brooklyn MCP

**Repository**: `brooklyn-mcp`  
**Strategy**: SemVer  
**Pattern**: Script-based with static embedding

```bash
VERSION file: 1.1.8

bun run version:bump:patch  # → 1.1.9
bun run version:embed       # Embeds in source files
bun run build               # Builds with embedded version
```

### SemVer Implementation: Crucible (Post-Transition)

**Repository**: `crucible`
**Strategy**: SemVer (transitioned from CalVer at v0.2.0)
**Pattern**: VERSION file with asset revision tracking

```bash
VERSION file: 0.2.1

# Schemas track independent versions:
schemas/terminal/v1.0.0/schema.json (revision 3)
schemas/pathfinder/v1.0.0/schema.json (revision 1)

# Repository version = release snapshot with Go module compatibility
```

**Historical Note**: Crucible originally used CalVer (`2025.10.x`) but adopted SemVer for Go module compatibility per [ADR-0010](../architecture/decisions/ADR-0010-semantic-versioning-adoption.md). CalVer tags (v2025.10.1-v2025.10.5) were retroactively mapped to SemVer equivalents (v0.1.0-v0.1.4).

---

**Status**: Approved  
**Last Updated**: 2025-10-02  
**Author**: @3leapsdave
