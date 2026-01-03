---
id: "ADR-0010"
title: "Semantic Versioning Adoption for Go Module Compatibility"
status: "accepted"
date: "2025-10-29"
last_updated: "2025-10-29"
deciders:
  - "@3leapsdave"
  - "@schema-cartographer"
  - "@fulmen-ea-steward"
scope: "Ecosystem"
tags:
  - "versioning"
  - "semver"
  - "go"
  - "calver"
  - "release-process"
related_adrs:
  - "ADR-0009"
adoption:
  gofulmen: "required"
  pyfulmen: "planned"
  tsfulmen: "planned"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-0010: Semantic Versioning Adoption for Go Module Compatibility

## Status

**Current Status**: Accepted
**Implemented**: v0.2.0 (transition release)

## Context

Following ADR-0009's relocation of the Go module to repository root (v2025.10.5), external Go module installation became possible. However, a critical compatibility issue emerged: **Go's module system interprets version tags as Semantic Versioning**, not Calendar Versioning.

### The CalVer-Go Module Path Conflict

Crucible's CalVer scheme (`v2025.10.5`) follows the pattern `vYYYY.MM.PATCH`. Go's module system interprets this as:

- Major version: 2025
- Minor version: 10
- Patch version: 5

Per [Go module version requirements](https://go.dev/doc/modules/major-version), **any module with major version ≥2 must include the major version in its module path**:

```go
// For v2025.10.5, Go expects:
module github.com/fulmenhq/crucible/v2025
```

Without this path adjustment, external installation fails:

```bash
$ go get github.com/fulmenhq/crucible
go: downloading github.com/fulmenhq/crucible v2025.10.5+incompatible
error: invalid version: module contains a go.mod file, so module path must match
       major version ("github.com/fulmenhq/crucible/v2025")
```

### Discovery Timeline

- **v2025.10.4**: Repository made public
- **v2025.10.5**: Go module moved to root per ADR-0009
- **Post-v2025.10.5**: External consumers (gofulmen, forge-workhorse-groningen) encountered versioning errors
- **2025-10-29**: Issue analyzed, SemVer transition planned

### Why CalVer Was Originally Chosen

Crucible initially adopted CalVer to:

- Provide temporal context in version numbers (immediately see when a release was made)
- Align with project management practices emphasizing release cadence
- Avoid semantic meaning debates ("Is this breaking? Minor? Patch?")

However, these benefits assumed Crucible would remain private with `replace` directives, not serve as a public Go module.

## Decision

**Adopt Semantic Versioning (SemVer) for all Crucible releases**, starting with v0.2.0.

### Version Scheme

- **Pre-1.0 (current)**: `v0.x.y` while APIs are stabilizing
- **Post-1.0**: `v1.x.y` once interfaces are stable
- **Breaking changes**: Bump major version, requiring module path updates (`/v2`, `/v3`, etc.)

### Retroactive Tag Mapping

To preserve chronological continuity and provide migration clarity, **add parallel SemVer tags to existing CalVer releases**:

| CalVer Tag | SemVer Tag | Commit SHA | Release Date | Notes                     |
| ---------- | ---------- | ---------- | ------------ | ------------------------- |
| v2025.10.1 | v0.1.0     | (same)     | 2025-10-25   | Initial public release    |
| v2025.10.2 | v0.1.1     | (same)     | 2025-10-26   | Documentation polish      |
| v2025.10.3 | v0.1.2     | (same)     | 2025-10-28   | SSOT provenance schemas   |
| v2025.10.4 | v0.1.3     | (same)     | 2025-10-28   | Bootstrap guide rewrite   |
| v2025.10.5 | v0.1.4     | (same)     | 2025-10-29   | Go module root relocation |
| (new)      | v0.2.0     | (new)      | 2025-10-29   | SemVer transition release |

**Tagging Strategy**:

- Preserve existing CalVer tags for historical record (do not delete)
- Add SemVer tags pointing to identical commits
- Both tag sets remain accessible during transition period
- Future releases use **only SemVer tags**

### Versioning Policy Going Forward

**Pre-1.0 Phase** (current):

- `v0.x.0`: Minor version bumps for feature additions, non-breaking enhancements
- `v0.x.y`: Patch version bumps for bug fixes, documentation updates
- Breaking changes acceptable during v0.x (per SemVer spec)

**Post-1.0 Phase** (future):

- `v1.0.0`: Stable API contract locked
- `v1.x.0`: Minor version for backward-compatible features
- `v1.0.y`: Patch version for backward-compatible fixes
- `v2.0.0`: Breaking changes (requires `/v2` module path)

## Rationale

### Why SemVer is the Right Choice

1. **Go Module Compatibility** (Critical)
   - Works immediately with `go get` and Go proxy
   - No module path gymnastics (`/v2025`, `/v2026`, etc.)
   - Standard Go ecosystem practice

2. **Eliminates Annual Breaking Changes**
   - CalVer with Go compliance would require yearly module path updates:
     ```go
     import "github.com/fulmenhq/crucible/v2025"  // 2025
     import "github.com/fulmenhq/crucible/v2026"  // 2026 (breaking!)
     ```
   - SemVer keeps module path stable until actual breaking changes

3. **Ecosystem Standard**
   - Go, Python (PEP 440), JavaScript (npm), Rust (Cargo) all use SemVer
   - Familiar to all developers
   - Tooling (dependency bots, security scanners) understand SemVer

4. **Semantic Clarity**
   - `v0.2.0` signals "pre-stable, minor iteration"
   - `v1.0.0` will signal "stable API contract"
   - Breaking changes explicit via major version bump

5. **Preserves Temporal Context**
   - Release notes and CHANGELOG.md still include dates
   - Git tags retain timestamps
   - Release process can document "Released 2025-10-29"

### Trade-offs Accepted

1. **Loses Date-at-a-Glance**
   - **Impact**: Can't immediately see release date from version number
   - **Mitigation**: CHANGELOG.md includes dates, GitHub releases show timestamps, version numbers still convey progression

2. **Breaking from FulmenHQ CalVer Preference**
   - **Impact**: Crucible diverges from initial versioning vision
   - **Mitigation**: This is a pragmatic correction based on ecosystem realities; other FulmenHQ projects can choose differently unless they're Go modules

3. **Retroactive Tag Complexity**
   - **Impact**: Two tag sets exist temporarily during migration
   - **Mitigation**: Clear documentation, parallel tags aid migration rather than hinder it

## Alternatives Considered

### Alternative 1: v0 CalVer (Hybrid Versioning)

**Description**: Keep CalVer spirit with v0 prefix to satisfy Go: `v0.202510.5`

**Pros**:

- Preserves date-based versioning intent
- Works with Go modules (no path changes needed)
- Simple to implement

**Cons**:

- **Non-standard**: Not CalVer, not SemVer—confusing hybrid
- **Awkward Semantics**: What does "202510" mean as a minor version?
- **Annual Coordination**: Still need to coordinate version bumps across ecosystem
- **Loses Ecosystem Familiarity**: Developers won't recognize the pattern

**Decision**: Rejected - Solving technical problem but creating semantic confusion.

### Alternative 2: Annual Module Path Bumps (CalVer + Go Compliance)

**Description**: Embrace CalVer fully with yearly `/v2025`, `/v2026` module paths.

**go.mod**:

```go
module github.com/fulmenhq/crucible/v2025

go 1.23
```

**Pros**:

- True CalVer with clear temporal meaning
- Go modules technically compliant

**Cons**:

- **Annual Breaking Changes**: Every year, import path changes
  ```go
  import "github.com/fulmenhq/crucible/v2025"  // This year
  import "github.com/fulmenhq/crucible/v2026"  // Next year (breaking!)
  ```
- **Massive Ecosystem Disruption**: All downstream consumers must update imports annually
- **Multiple Major Versions Simultaneously**: Libraries support multiple Crucible years
- **Not Actually Semantic**: Module path changes don't reflect API changes

**Decision**: Rejected - Unacceptable churn and ecosystem disruption.

### Alternative 3: Keep CalVer with +incompatible

**Description**: Tag as `v2025.10.5`, acknowledge with `+incompatible` suffix.

**go.mod** (consumers):

```go
require github.com/fulmenhq/crucible v2025.10.5+incompatible
```

**Pros**:

- No changes to Crucible needed
- Preserves pure CalVer

**Cons**:

- **Doesn't Actually Work**: External `go get` still fails with 404
- **Signals "Broken"**: `+incompatible` tells ecosystem "this module doesn't follow conventions"
- **Not a Solution**: Acknowledging the problem doesn't fix it

**Decision**: Rejected - Not a viable solution.

### Alternative 4: Separate crucible-go Repository

**Description**: Split Go wrapper into `fulmenhq/crucible-go` with independent versioning.

**Pros**:

- Go versioning independent of SSOT
- No structural impact on Crucible

**Cons**:

- **Violates SSOT Principle**: Schemas must sync between repos
- **Fragmentation**: Two repos, two release processes
- **Coordination Overhead**: Schema updates require coordinated releases
- **Loses Monorepo Benefits**: Can't atomically update schemas and code

**Decision**: Rejected - Same rationale as ADR-0009 Alternative 1.

## Consequences

### Positive

- ✅ **Standard External Installation**: `go get github.com/fulmenhq/crucible` works immediately without workarounds
- ✅ **No Annual Breaking Changes**: Module path stays stable (`github.com/fulmenhq/crucible`) until actual API changes
- ✅ **Ecosystem Familiarity**: Developers recognize and understand SemVer
- ✅ **Tooling Compatibility**: Dependency management tools, security scanners, and bots understand SemVer
- ✅ **Clear Stability Signal**: v0.x indicates pre-stable, v1.0.0 will signal production-ready
- ✅ **Downstream Simplicity**: gofulmen, pyfulmen, tsfulmen can use standard version pinning
- ✅ **Migration Path Provided**: Retroactive tags ease consumer transition

### Negative

- ⚠️ **Loses Temporal Clarity**: Version number no longer indicates release date
  - Mitigated by CHANGELOG.md dates, GitHub release timestamps, and documentation
- ⚠️ **Two Tag Sets Temporarily**: CalVer and SemVer tags coexist during transition
  - Mitigated by clear documentation and explicit migration guidance
- ⚠️ **Versioning Policy Change**: Existing consumers must understand the shift
  - Mitigated by ADR documentation, CHANGELOG entry, and ecosystem communication

### Neutral

- ℹ️ **Release Cadence Unchanged**: SemVer doesn't dictate release frequency
- ℹ️ **SSOT Integrity Preserved**: Versioning scheme doesn't affect schema governance
- ℹ️ **Makefile Targets Updated**: `make version-set VERSION=0.2.0` supports SemVer (goneat already compatible)

## Implementation

### Transition Release: v0.2.0

**Purpose**: Formal adoption of SemVer with no functional changes.

**Steps**:

1. **Retroactive Tagging** (one-time operation):

   ```bash
   # Add SemVer tags to existing CalVer commits
   git tag v0.1.0 v2025.10.1
   git tag v0.1.1 v2025.10.2
   git tag v0.1.2 v2025.10.3
   git tag v0.1.3 v2025.10.4
   git tag v0.1.4 v2025.10.5
   git push origin v0.1.0 v0.1.1 v0.1.2 v0.1.3 v0.1.4
   ```

2. **Update VERSION File**:

   ```bash
   make version-set VERSION=0.2.0
   ```

3. **Update CHANGELOG.md**:
   - Add `## [0.2.0] - 2025-10-29` entry
   - Document SemVer adoption and retroactive tag mapping
   - Update versioning adherence statement from CalVer to SemVer

4. **Update README.md** (if versioning section exists):
   - Remove CalVer references
   - Document SemVer scheme (v0.x.y pre-stable, v1.x.y stable)

5. **Update Documentation**:
   - ADR-0010 (this document)
   - docs/ops/repository/release-checklist.md (if CalVer-specific)
   - docs/sop/repository-operations-sop.md (if CalVer-specific)

6. **Commit and Tag**:

   ```bash
   git add VERSION CHANGELOG.md README.md docs/
   git commit -m "chore: adopt semantic versioning (v0.2.0)

   Replace CalVer with SemVer for Go module compatibility and ecosystem
   alignment. Add retroactive SemVer tags (v0.1.0-v0.1.4) to existing
   CalVer releases for migration continuity.

   🎯 Changes:
   - Update VERSION to 0.2.0
   - Add ADR-0010 documenting versioning decision
   - Update CHANGELOG.md with SemVer adherence
   - Retroactively tag v0.1.0 through v0.1.4 for historical releases

   Generated by Schema Cartographer ([Claude Code](https://claude.com/claude-code)) under supervision of @3leapsdave

   Co-Authored-By: Schema Cartographer <noreply@3leaps.net>
   Committer-of-Record: Dave Thompson <dave.thompson@3leaps.net> [@3leapsdave]"

   git tag -a v0.2.0 -m "Release v0.2.0: Semantic Versioning Adoption

   Formal transition from CalVer to SemVer for Go module compatibility.
   See ADR-0010 and CHANGELOG.md for details."

   git push origin main
   git push origin v0.2.0
   ```

### Downstream Coordination

**gofulmen**:

```go
// go.mod - update from:
require github.com/fulmenhq/crucible v2025.10.5
// to:
require github.com/fulmenhq/crucible v0.1.4  // or v0.2.0 after transition
```

**pyfulmen / tsfulmen**:

- Update dependency version references if hardcoded
- Align on SemVer for ecosystem consistency (recommended)

**goneat**:

- No changes needed (already supports both CalVer and SemVer validation)

### Validation Checklist

- [ ] `goneat version validate 0.2.0` succeeds
- [ ] `make version-set VERSION=0.2.0` succeeds
- [ ] `make version` shows `0.2.0`
- [ ] CHANGELOG.md includes v0.2.0 entry
- [ ] Retroactive tags point to correct commits
- [ ] All tests pass: `make test`
- [ ] External `go get github.com/fulmenhq/crucible@v0.2.0` works

## Cross-Language Coordination

### Go (Crucible root module)

**Impact**: Required
**Implementation**: Immediate (v0.2.0)
**Rationale**: Driven by Go module versioning requirements

### Python (lang/python/)

**Impact**: Recommended
**Implementation**: Next release (align on SemVer for consistency)
**Rationale**: No technical requirement, but ecosystem consistency valuable

### TypeScript (lang/typescript/)

**Impact**: Recommended
**Implementation**: Next release (align on SemVer for consistency)
**Rationale**: npm ecosystem uses SemVer; natural fit

### Future Languages (Rust, C#)

**Pattern**: Start with SemVer from the beginning to avoid versioning migrations

## References

- [ADR-0009: Go Module Root Relocation](./ADR-0009-go-module-root-relocation.md) - Context for this decision
- [Go Modules Version Numbers](https://go.dev/doc/modules/version-numbers) - Official Go versioning documentation
- [Go Major Version Requirements](https://go.dev/doc/modules/major-version) - Module path requirements for v2+
- [Semantic Versioning 2.0.0](https://semver.org/) - SemVer specification
- [Calendar Versioning](https://calver.org/) - CalVer specification (previous approach)
- [gofulmen memo: CalVer vs Go Module Path](../../.plans/memos/crucible/2025-10-29-calver-go-module-path-issue.md) - Analysis that led to this decision

## Revision History

| Date       | Status Change | Summary                    | Updated By           |
| ---------- | ------------- | -------------------------- | -------------------- |
| 2025-10-29 | → accepted    | Initial draft and approval | @schema-cartographer |
| 2025-11-04 | clarification | CalVer tag cleanup         | @schema-cartographer |

### 2025-11-04 Update: CalVer Tag Cleanup

Following v0.2.4 release, the legacy CalVer tags (`v2025.10.0` through `v2025.10.5`) were deleted to eliminate tag sorting confusion. The original "preserve for historical record" guidance (line 108) was written at the transition point, but after several successful SemVer releases, the CalVer tags became a source of confusion rather than clarity.

**Rationale for Deletion:**

- CalVer tags sort alphabetically above SemVer tags, causing confusion in `git tag -l` output
- All CalVer→SemVer mappings permanently documented in ADR-0010 table (lines 94-101)
- Git history preserves evidence of CalVer tags in reflog
- Minimal external exposure (repository was private through most CalVer period)
- Clean SemVer-only tag list benefits all future consumers

**Deleted Tags:**

- `v2025.10.0` (pre-public release, no SemVer equivalent)
- `v2025.10.1` (≡ v0.1.0)
- `v2025.10.2` (≡ v0.1.1)
- `v2025.10.3` (≡ v0.1.2)
- `v2025.10.4` (≡ v0.1.3)
- `v2025.10.5` (≡ v0.1.4)

**Documentation:** See [Annex B: CalVer Tag Cleanup (2025-11-04)](../../ops/repository/memos/2025-10-07-initial-commit-history-squash.md#annex-b-calver-tag-cleanup---november-4-2025) for operational details.

---

**Schema Conformance**: This ADR conforms to [ADR Frontmatter Schema v1.0.0](https://schemas.fulmenhq.dev/standards/adr-frontmatter-v1.0.0.json)
