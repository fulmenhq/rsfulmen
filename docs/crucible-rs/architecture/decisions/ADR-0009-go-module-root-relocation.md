---
id: "ADR-0009"
title: "Go Module Root Relocation for External Installation and Embed Support"
status: "accepted"
date: "2025-10-29"
last_updated: "2025-10-29"
deciders:
  - "@3leapsdave"
  - "@pipeline-architect"
  - "@fulmen-ea-steward"
scope: "Ecosystem"
tags:
  - "go"
  - "structure"
  - "ssot"
  - "embed"
  - "installation"
related_adrs: []
adoption:
  gofulmen: "required"
  pyfulmen: "not-applicable"
  tsfulmen: "not-applicable"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-0009: Go Module Root Relocation for External Installation and Embed Support

## Status

**Current Status**: Accepted
**Implemented**: v2025.10.5

## Context

Crucible serves as the Single Source of Truth (SSOT) for FulmenHQ schemas, standards, and templates across multiple programming languages. Prior to v2025.10.5, the repository maintained a symmetric structure with all language implementations in `lang/<language>/` subdirectories:

```
crucible/
├── lang/
│   ├── go/           # Go module here
│   ├── python/       # Python package here
│   └── typescript/   # TypeScript package here
├── schemas/          # SSOT
├── docs/             # SSOT
└── config/           # SSOT
```

This structure worked well for local development and monorepo workflows with `replace` directives in `go.mod`. However, when Crucible went public in v2025.10.4, two critical issues emerged:

### Issue 1: External Go Module Installation Blocked

Go's module system does not support subdirectory modules. When external projects attempted to install gofulmen (which depends on Crucible), they encountered errors:

```bash
$ go get github.com/fulmenhq/gofulmen
go: github.com/fulmenhq/gofulmen/crucible imports
    github.com/fulmenhq/crucible: module github.com/fulmenhq/crucible@latest found (v2025.10.4+incompatible),
    but does not contain package github.com/fulmenhq/crucible
```

The Go toolchain expected to find `go.mod` and package sources at `github.com/fulmenhq/crucible`, not at `github.com/fulmenhq/crucible/lang/go`. This blocked:

- External consumers installing gofulmen via standard `go get`
- forge-workhorse templates using gofulmen
- Any third-party Go project depending on Crucible

### Issue 2: go:embed Requires Root-Relative Paths

Go's `//go:embed` directive embeds files relative to the package location. With Go code in `lang/go/`, embed directives required navigating up to access root SSOT:

```go
//go:embed ../../schemas  // Fragile, path-dependent
//go:embed ../../docs
```

This pattern:

- Created fragile path dependencies
- Complicated testing and local development
- Violated the principle that SSOT assets should be embedded directly from their canonical location
- Required syncing SSOT to `lang/go/schemas/`, `lang/go/docs/`, `lang/go/config/` before embedding

### SSOT Sync Overhead

The symmetric structure required syncing SSOT assets to all three language directories before publishing. For Go, this meant:

1. Root SSOT modified
2. `make sync` copies to `lang/go/schemas/`, etc.
3. Go embeds from the copied location
4. Package published

This introduced unnecessary intermediate copies and sync overhead for the Go wrapper.

## Decision

**Move Go module to repository root** while preserving `lang/` structure for Python and TypeScript.

### New Repository Structure

```
crucible/
├── go.mod            # Go module at root
├── go.sum
├── *.go              # All Go sources at root
├── schemas/          # SSOT (Go embeds directly)
├── docs/             # SSOT (Go embeds directly)
├── config/           # SSOT (Go embeds directly)
├── lang/
│   ├── go/           # Breadcrumb README only
│   ├── python/       # Python package (synced)
│   └── typescript/   # TypeScript package (synced)
├── scripts/
├── Makefile
└── ...
```

### Key Changes

1. **Go Module at Root**: `go.mod`, `go.sum`, and all `*.go` files moved to repository root
2. **Direct Embed**: Go embeds from root SSOT via `//go:embed schemas`, `//go:embed docs`
3. **No Go Sync**: `scripts/sync-to-lang.ts` no longer syncs to `lang/go/` (Go accesses root SSOT directly)
4. **Breadcrumb README**: `lang/go/README.md` explains relocation and points to root module
5. **Makefile Updates**: Go targets run from root (`go test ./...` instead of `cd lang/go && go test`)

### Asymmetric Pattern Rationale

This creates an intentionally asymmetric structure:

- **Go at root**: Supports standard module installation and direct SSOT embedding
- **Python in lang/python/**: Standard Python package structure with synced assets
- **TypeScript in lang/typescript/**: Standard npm package structure with synced assets

This pattern follows industry precedent:

- **Terraform**: Root-level Go module, `website/` and `docs/` subdirectories
- **Kubernetes**: Root-level Go modules, separate `cmd/` and `pkg/` organization
- **NATS**: Root-level Go, other language clients in subdirectories
- **etcd**: Root-level Go code, documentation in `docs/`

The primary or "first-class" language typically occupies the root in multi-language monorepos, with other languages in subdirectories.

## Rationale

### Benefits of Root-Level Go Module

1. **Standard External Installation**

   ```bash
   go get github.com/fulmenhq/crucible@v2025.10.5
   # Works immediately, no replace directives needed
   ```

2. **Clean Embed Directives**

   ```go
   //go:embed schemas  // Clean, root-relative
   //go:embed docs     // No path navigation
   ```

3. **Eliminates Go Sync Overhead**
   - Python/TypeScript still sync from root SSOT (they package assets differently)
   - Go accesses root SSOT directly via embed
   - Simpler build process for Go consumers

4. **Industry Standard Pattern**
   - Aligns with Go ecosystem conventions
   - Familiar structure for Go developers
   - Better pkg.go.dev documentation generation

5. **Unblocks Ecosystem Growth**
   - External projects can use gofulmen
   - forge-workhorse templates work out of the box
   - No local clones or replace directives required

### Trade-offs Accepted

1. **Asymmetric Language Structure**
   - **Impact**: Contributors may find it surprising initially
   - **Mitigation**:
     - Comprehensive breadcrumb README in `lang/go/`
     - Documentation explaining the pattern and rationale
     - Industry precedent demonstrates this is acceptable

2. **Root Directory Has More Files**
   - **Impact**: Root directory contains Go sources alongside other top-level files
   - **Mitigation**:
     - Go files follow clear naming conventions
     - Root README documents the structure
     - Standard Go tooling handles this naturally

3. **One-Time Migration Complexity**
   - **Impact**: Existing local clones need cleanup, gofulmen needs update
   - **Mitigation**:
     - Clear migration documentation
     - Version bump signals the change
     - Coordinated release with gofulmen v0.1.6

## Alternatives Considered

### Alternative 1: Separate crucible-go Repository

**Description**: Create a standalone `fulmenhq/crucible-go` repository containing only Go wrapper code.

**Pros**:

- Complete structural independence
- Can version Go module separately from schemas
- No impact on Crucible structure

**Cons**:

- **Fragmentation**: Need to sync schemas/docs between repositories
- **More Maintenance**: Two repositories to maintain, two release processes
- **Loses Monorepo Benefits**: Can't atomically update schemas and Go code
- **Coordination Overhead**: Schema changes require coordinated releases
- **Not SSOT Compliant**: Violates the principle of single source of truth

**Decision**: Rejected - Fragmentation costs outweigh structural simplicity.

### Alternative 2: Go Subdirectory with Replace Directives

**Description**: Keep Go in `lang/go/` and document that external consumers must use `replace` directives.

**Pros**:

- Maintains symmetric structure
- No structural changes needed

**Cons**:

- **Poor User Experience**: Every consumer needs manual replace directive
- **Not Standard Go**: Violates Go module conventions
- **Breaks External Installation**: `go get` fundamentally doesn't work
- **Documentation Burden**: Must document workarounds extensively
- **Fragile**: Local path dependencies break easily

**Decision**: Rejected - Unacceptable user experience and violates Go conventions.

### Alternative 3: Use Go Workspace with Subdirectory

**Description**: Use Go 1.18+ workspace feature to reference `lang/go/` subdirectory.

**Pros**:

- Modern Go feature
- Could maintain symmetric structure

**Cons**:

- **Requires go.work File**: All consumers must maintain workspace files
- **Still Non-Standard**: Subdirectory modules aren't idiomatic
- **Complexity**: Adds workspace overhead for simple use case
- **External Installation Still Broken**: `go get` still expects root module

**Decision**: Rejected - Adds complexity without solving core issue.

## Consequences

### Positive

- ✅ **External Installation Works**: `go get github.com/fulmenhq/crucible` works immediately
- ✅ **No Replace Directives**: gofulmen and downstream projects use standard module references
- ✅ **Clean Embed Paths**: `//go:embed schemas` directly accesses SSOT
- ✅ **Eliminates Go Sync**: Simpler build process, no intermediate copies
- ✅ **Standard Go Practice**: Aligns with ecosystem conventions
- ✅ **Better Documentation**: pkg.go.dev automatically indexes and documents the module
- ✅ **Unblocks Ecosystem**: forge-workhorse and other external consumers can proceed

### Negative

- ⚠️ **Asymmetric Structure**: Go at root, others in `lang/` subdirectories
  - Mitigated by documentation and industry precedent
- ⚠️ **One-Time Migration**: Existing consumers must update to v2025.10.5
  - Mitigated by clear migration notes and version signaling
- ⚠️ **Root Directory Density**: More files at root level
  - Mitigated by clear organization and standard Go conventions

### Neutral

- ℹ️ **Python/TypeScript Unchanged**: Sync process continues for non-Go languages
- ℹ️ **SSOT Integrity Preserved**: Root schemas/docs/config remain authoritative
- ℹ️ **Build Process**: Go builds from root, others from subdirectories (documented in Makefile)

## Implementation

### Version 2025.10.5

**Migration Steps**:

1. ✅ Move Go files: `lang/go/*.go`, `lang/go/go.mod`, `lang/go/go.sum` → root
2. ✅ Update version constant: `schemas.go` Version = "2025.10.5"
3. ✅ Remove synced directories: `lang/go/schemas/`, `lang/go/docs/`, `lang/go/config/`
4. ✅ Create breadcrumb: `lang/go/README.md` with migration notes
5. ✅ Update sync script: Remove `syncToGo()` from `scripts/sync-to-lang.ts`
6. ✅ Update Makefile: Remove `cd lang/go` from test/build/lint targets
7. ✅ Documentation: ADR-0009, README updates, repository structure SOP

**Validation**:

- `go mod tidy` succeeds
- `go build ./...` succeeds
- `go test ./...` passes
- `//go:embed` directives work correctly
- Python/TypeScript builds unaffected
- External `go get github.com/fulmenhq/crucible@v2025.10.5` works

### Downstream Coordination

**gofulmen v0.1.6** (coordinated release):

```go
// Remove replace directive
// replace github.com/fulmenhq/crucible => ../crucible/lang/go

// Update to v2025.10.5
require github.com/fulmenhq/crucible v2025.10.5
```

**forge-workhorse-groningen**:

- Can now use `go get github.com/fulmenhq/gofulmen` without local clones
- Standard Go module workflow enabled

## Cross-Language Impact

### Python (lang/python/)

**Impact**: None
**Rationale**: Python packaging ignores `.go` files at root. Sync process continues as before.

### TypeScript (lang/typescript/)

**Impact**: None
**Rationale**: TypeScript tooling ignores `.go` files. Sync process continues as before.

### Future Languages (Rust, C#)

**Pattern**: Will follow Python/TypeScript model in `lang/<language>/` subdirectories unless they have similar module system constraints requiring root placement.

## Documentation Updates

### Required Updates

1. ✅ **ADR-0009**: This document
2. **Root README.md**: Note Go module structure, link to ADR-0009
3. **docs/sop/repository-structure.md**: Document asymmetric pattern
4. **CHANGELOG.md**: Note v2025.10.5 structural change
5. **lang/go/README.md**: Breadcrumb with migration guide (✅ completed)

### Migration Communication

**CHANGELOG Entry** (v2025.10.5):

```markdown
### Changed

- **BREAKING**: Go module relocated to repository root for standard external installation
  - `go get github.com/fulmenhq/crucible` now works without replace directives
  - Go code embeds directly from root SSOT (schemas/, docs/, config/)
  - See ADR-0009 and lang/go/README.md for migration details
  - Update to v2025.10.5 and remove any replace directives from go.mod
```

**For gofulmen**:

- Update `go.mod` to require v2025.10.5
- Remove replace directive
- Test external installation works
- Release v0.1.6 with updated dependency

## Review and Approval

**Reviewed by**:

- @3leapsdave (Project Lead)
- @pipeline-architect (DevOps & Tooling)
- @fulmen-ea-steward (Ecosystem Architecture)

**Approved**: 2025-10-29
**Implemented**: v2025.10.5

---

**References**:

- [Repository Structure SOP](../../sop/repository-structure.md)
- [Makefile Standard](../../standards/makefile-standard.md)
- [Go Module Documentation](https://go.dev/ref/mod)
- [lang/go/README.md](../../../lang/go/README.md) (Migration Guide)
