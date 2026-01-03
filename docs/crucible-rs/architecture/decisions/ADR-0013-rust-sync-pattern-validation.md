---
id: "ADR-0013"
title: "Rust Language Support Uses Sync Pattern (Not Root Relocation)"
status: "accepted"
date: "2025-11-30"
last_updated: "2025-11-30"
deciders:
  - "@3leapsdave"
  - "@fulmen-ea-steward"
  - "@schema-cartographer"
scope: "Ecosystem"
tags:
  - "rust"
  - "structure"
  - "ssot"
  - "sync"
  - "embedding"
related_adrs:
  - "ADR-0009"
adoption:
  gofulmen: "not-applicable"
  pyfulmen: "not-applicable"
  tsfulmen: "not-applicable"
  rsfulmen: "required"
  csfulmen: "informative"
---

# ADR-0013: Rust Language Support Uses Sync Pattern (Not Root Relocation)

## Status

**Current Status**: Accepted
**Target Release**: v0.2.21

## Context

With the addition of Rust as a first-class language in the FulmenHQ ecosystem (rsfulmen), we needed to determine whether Rust should follow:

1. **Go's pattern**: Root-level module with direct SSOT embedding (per ADR-0009)
2. **Python/TypeScript pattern**: Subdirectory package with synced SSOT assets

ADR-0009 documented that Go required root relocation due to fundamental constraints in Go's module system. Before implementing Rust support, we investigated whether Rust has similar constraints.

### Go's Constraints (ADR-0009 Summary)

Go was relocated to the repository root because of two critical issues:

1. **Module System Limitation**: Go's `go get` command fundamentally does not support subdirectory modules. External consumers could not install Crucible via `go get github.com/fulmenhq/crucible` when `go.mod` was in `lang/go/`.

2. **Embed Path Fragility**: Go's `//go:embed` directive requires paths relative to the package location. From `lang/go/`, embedding root SSOT required `//go:embed ../../schemas` which is fragile and violates SSOT principles.

### Rust Investigation

We investigated whether Rust (Cargo) has similar constraints:

#### Cargo Package Location

Unlike Go, Cargo fully supports packages in subdirectories:

- **Workspaces**: Cargo workspaces support member crates in any subdirectory
- **Git Dependencies**: Cargo supports `{ git = "...", path = "lang/rust" }` syntax
- **crates.io Publishing**: `cargo publish` works from any directory with a valid `Cargo.toml`

External installation of a subdirectory crate works:

```toml
# In consumer's Cargo.toml
[dependencies]
crucible-codegen = { git = "https://github.com/fulmenhq/crucible", path = "lang/rust" }
```

#### Asset Embedding

Rust's `include_str!`, `include_bytes!`, and the `include_dir` crate have similar path behavior to Go's `go:embed`:

- Paths are relative to the source file or `CARGO_MANIFEST_DIR`
- Parent directory access (`../../schemas`) is problematic when packaging

However, unlike Go, this is **not a blocking issue** because:

1. The sync pattern places assets **within** the crate directory (`lang/rust/schemas/`)
2. Embedding then uses clean paths: `include_dir!("$CARGO_MANIFEST_DIR/schemas")`
3. Assets are properly included when publishing to crates.io

From the Rust community documentation:

> "When trying to include files from parent directories using `include_bytes!("../../assets/...")` and adding the path to `Cargo.toml`'s `[package] include` field, the file may not be included properly when packaging."

This confirms that syncing assets INTO the crate directory is the correct approach.

## Decision

**Rust will use the sync pattern** (like Python/TypeScript), NOT root relocation (like Go).

### Repository Structure

```
crucible/
├── go.mod              # Go at root (per ADR-0009)
├── *.go                # Go sources at root
├── schemas/            # SSOT
├── docs/               # SSOT
├── config/             # SSOT
├── lang/
│   ├── go/             # Breadcrumb README only
│   ├── python/         # Python package (synced)
│   ├── typescript/     # TypeScript package (synced)
│   └── rust/           # Rust crate (synced) ← NEW
│       ├── Cargo.toml
│       ├── src/
│       ├── schemas/    # Synced from root
│       ├── config/     # Synced from root
│       └── docs/       # Synced from root (excluding ops/)
└── scripts/
    └── sync-to-lang.ts # Updated to include Rust
```

### Sync Pipeline Update

`scripts/sync-to-lang.ts` will be extended to include a `syncToRust()` function that mirrors the existing `syncToPython()` and `syncToTypeScript()` implementations:

```typescript
async function syncToRust(options: SyncOptions) {
  console.log("📦 Rust crate...");
  const rustRoot = join(ROOT, "lang/rust");
  await syncDirectory(
    join(ROOT, "schemas"),
    join(rustRoot, "schemas"),
    "schemas/",
    options,
  );
  await syncDirectory(
    join(ROOT, "config"),
    join(rustRoot, "config"),
    "config/",
    options,
  );
  await syncDirectory(
    join(ROOT, "docs"),
    join(rustRoot, "docs"),
    "docs/",
    options,
    ["ops"],
  );
}
```

### Asset Embedding in Rust

With synced assets in `lang/rust/`, embedding is clean and reliable:

```rust
use include_dir::{include_dir, Dir};

// Clean path - assets are within crate directory
static SCHEMAS: Dir = include_dir!("$CARGO_MANIFEST_DIR/schemas");
static CONFIG: Dir = include_dir!("$CARGO_MANIFEST_DIR/config");
```

## Rationale

### Why Sync Pattern Works for Rust

| Factor                        | Go (Root Required)                 | Rust (Subdirectory OK)           |
| ----------------------------- | ---------------------------------- | -------------------------------- |
| External package installation | `go get` broken for subdirectories | `cargo add` with git path works  |
| Package manager support       | No subdirectory module support     | Full workspace/path support      |
| crates.io/pkg.go.dev          | Expects root module                | Accepts subdirectory crates      |
| Embed with synced assets      | N/A (uses root)                    | Works with `$CARGO_MANIFEST_DIR` |

### Why NOT Root Relocation for Rust

1. **No Technical Necessity**: Unlike Go, Cargo has no fundamental limitation requiring root placement

2. **Preserves Consistency**: Maintains the established pattern where non-Go languages live in `lang/<language>/`

3. **Avoids Root Clutter**: Adding `Cargo.toml`, `Cargo.lock`, and `*.rs` files to root alongside Go's `go.mod`, `go.sum`, and `*.go` would create confusion

4. **ADR-0009 Precedent**: ADR-0009 explicitly states future languages should "follow Python/TypeScript model in `lang/<language>/` subdirectories unless they have similar module system constraints requiring root placement"

5. **Industry Alignment**: Mixed-language monorepos typically keep each language's package in its own subdirectory unless there's a technical requirement otherwise

## Alternatives Considered

### Alternative 1: Root-Level Rust (Like Go)

**Description**: Place `Cargo.toml` and Rust sources at repository root alongside Go.

**Pros**:

- Direct SSOT embedding without sync
- Symmetric with Go

**Cons**:

- **No Technical Justification**: Cargo doesn't require root placement
- **Root Clutter**: Two languages' build files at root
- **Violates ADR-0009 Guidance**: Would relocate without necessity
- **Confusion**: Which language "owns" the root?

**Decision**: Rejected - No technical requirement, would add complexity.

### Alternative 2: No Asset Embedding (Runtime Loading)

**Description**: Don't embed assets; load schemas/config at runtime from filesystem paths.

**Pros**:

- No sync needed
- Assets always fresh

**Cons**:

- **Deployment Complexity**: Must ship asset files alongside binary
- **Not Rust Idiomatic**: Rust prefers compile-time embedding for bundled assets
- **Inconsistent**: Go and generated code embed at compile time

**Decision**: Rejected - Compile-time embedding is preferred for reliability.

### Alternative 3: Build Script Asset Copying

**Description**: Use `build.rs` to copy assets from `../../` at build time.

**Pros**:

- No manual sync step
- Assets copied automatically

**Cons**:

- **Packaging Issues**: Parent paths may not be available when building from crates.io
- **Workspace Assumptions**: Assumes specific repository structure
- **Fragile**: Depends on relative paths being stable

**Decision**: Rejected - Sync is more reliable and explicit.

## Consequences

### Positive

- ✅ **Consistent Pattern**: Rust follows established Python/TypeScript model
- ✅ **Clean Embedding**: `$CARGO_MANIFEST_DIR/schemas` is reliable and portable
- ✅ **External Installation**: Git dependencies with path work correctly
- ✅ **crates.io Ready**: Subdirectory publishing is fully supported
- ✅ **Root Simplicity**: No additional files at repository root
- ✅ **Clear Ownership**: Each language has its own directory

### Negative

- ⚠️ **Sync Required**: Must run `make sync` before building/publishing Rust crate
  - Mitigated by `make precommit` including sync step
- ⚠️ **Duplicate Assets**: schemas/config/docs exist in multiple locations
  - Mitigated by clear SSOT documentation and gitignore patterns

### Neutral

- ℹ️ **Go Remains Special Case**: Go at root is the exception, not the rule
- ℹ️ **Future Languages**: C# and others should also follow sync pattern unless constraints emerge

## Implementation

### v0.2.21 Implementation

1. **Create `lang/rust/` scaffold** (W1)
   - `Cargo.toml` with serde, thiserror dependencies
   - `rust-toolchain.toml` pinning MSRV 1.70
   - `src/lib.rs` with module structure
   - `.gitignore` for `target/`

2. **Update sync script** (W1)
   - Add `syncToRust()` function to `scripts/sync-to-lang.ts`
   - Include Rust in sync pipeline

3. **Add codegen templates** (W2)
   - Rust EJS templates for exit-codes, fulpack, fulencode, fulhash
   - Generated files output to `lang/rust/src/foundry/`

4. **Wire Makefile targets** (W3)
   - `test-rust`, `lint-rust`, `fmt-rust`, `build-rust`
   - Include Rust in `check-all` and `precommit`

### Validation

- `cargo build` succeeds in `lang/rust/`
- `cargo test` passes
- Synced assets appear correctly after `make sync`
- `include_dir!("$CARGO_MANIFEST_DIR/schemas")` compiles and includes files

## References

- [ADR-0009: Go Module Root Relocation](ADR-0009-go-module-root-relocation.md)
- [The Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [include_dir crate](https://docs.rs/include_dir/latest/include_dir/)
- [Rust Forum - Parent Directory Includes](https://users.rust-lang.org/t/including-files-from-parent-directory-in-package/88969)
- [Feature Brief: Rust Language Enablement](../../../.plans/active/v0.2.21/rust-language-feature-brief.md)

---

_Generated by Fulmen Enterprise Architect (@fulmen-ea-steward) ([Claude Code](https://claude.com/claude-code)) under supervision of @3leapsdave._
