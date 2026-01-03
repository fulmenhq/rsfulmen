---
title: "Pathfinder Extension"
description: "Optional helper module for path discovery, filesystem traversal, checksum support, and repository root discovery"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-11-16"
status: "draft"
tags:
  [
    "standards",
    "library",
    "extensions",
    "pathfinder",
    "fulhash",
    "checksums",
    "repository-root",
    "upward-traversal",
    "2025.11.2",
  ]
---

# Pathfinder Extension

## Scope

Deliver ergonomic helpers for scanning filesystem trees, applying inclusion/exclusion globs, and producing
metadata used by Fulmen tools (e.g., goneat). Pathfinding remains optional but widely useful for CLI tools.

## Capabilities

- Recursive scanning with inclusive/exclusive glob patterns.
- Ability to honor `.fulmenignore`-style files.
- Metadata collection: file size, modification time, and optional checksums.
- Hooks for pluggable processors (e.g., apply validation per file).
- Optional checksum calculation using [FulHash](../modules/fulhash.md) for integrity verification and change detection.
- Safe upward filesystem traversal for repository root discovery (v0.2.15+).

## Interoperability

- **Error Handling**: The shared error module (`schemas/error-handling/v1.0.0/error-response.schema.json`) extends the Pathfinder error envelope via `$ref`. Libraries emitting Pathfinder errors gain the optional telemetry fields automatically when they adopt the wrapper—no breaking changes for existing consumers.
- **Logging**: When Pathfinder operations surface errors, coordinate with Observability Logging to propagate `correlation_id` and `severity` so downstream pipelines can link events.

## Implementation Notes

- **Go**: Build atop `filepath.WalkDir` with concurrency controls and context cancellation.
- **Python**: Use `pathlib.Path.rglob` / `os.scandir`. Provide async variant when running under asyncio.
- **TypeScript**: Use `fast-glob` or `@nodelib/fs.walk` for efficient traversal.

## Testing

- Fixture-based tests with nested directories verifying glob matching.
- Performance benchmarks to guard against regressions.
- Windows path handling tests (drive letters, UNC paths).

## Checksum Support

**Version**: 2025.10.3+

Pathfinder integrates with [FulHash](../modules/fulhash.md) to provide optional file checksum calculation during traversal.

### Configuration

Extend Pathfinder configuration with checksum options:

```jsonc
{
  "includePatterns": ["**/*.go", "**/*.ts"],
  "excludePatterns": ["**/node_modules/**"],
  "calculateChecksums": false, // Enable checksum calculation
  "checksumAlgorithm": "xxh3-128", // Default: xxh3-128, also supports: sha256
  "checksumEncoding": "hex", // Default: hex (lowercase)
}
```

**Fields**:

- `calculateChecksums` (boolean): Enable/disable checksum calculation. Default: `false`
- `checksumAlgorithm` (string): Hash algorithm. Options: `"xxh3-128"` (default), `"sha256"`
- `checksumEncoding` (string): Output encoding. Currently only `"hex"` supported (lowercase)

### Metadata Schema

When checksums enabled, `PathResult` metadata includes:

```json
{
  "path": "src/main.go",
  "metadata": {
    "size": 12345,
    "modified": "2025-10-23T17:20:00Z",
    "checksum": "xxh3-128:abc123def456...",
    "checksumAlgorithm": "xxh3-128"
  }
}
```

**New Fields**:

- `checksum` (string, optional): Formatted checksum with algorithm prefix (`<algorithm>:<hex>`)
- `checksumAlgorithm` (string, optional): Algorithm identifier for quick filtering

**Backward Compatibility**: When `calculateChecksums: false`, these fields are omitted and existing metadata structure unchanged.

### Performance Considerations

- **Streaming**: Implementations MUST use streaming hashing for files to avoid loading entire file into memory
- **Target Overhead**: <10% performance overhead on mixed workloads (small/large files)
- **Buffer Management**: Reuse buffers to minimize allocations during traversal
- **Concurrency**: May parallelize checksum calculation across files (language-dependent)

### Error Handling

**Checksum Calculation Failures**:

When checksum calculation fails (permissions, I/O error):

1. Log warning with file path and error reason
2. Continue traversal (do not fail entire operation)
3. Omit checksum fields from metadata for affected files
4. Optionally include `checksumError` field in metadata:

```json
{
  "path": "protected/file.bin",
  "metadata": {
    "size": 5678,
    "modified": "2025-10-23T12:00:00Z",
    "checksumError": "Permission denied"
  }
}
```

### Implementation Requirements

**FulHash Integration**:

```python
# Python example
from pyfulmen.fulhash import hash_file, Algorithm

if config.calculate_checksums:
    try:
        digest = await hash_file(
            path,
            algorithm=Algorithm[config.checksum_algorithm.upper().replace('-', '_')]
        )
        metadata.checksum = digest.formatted
        metadata.checksum_algorithm = config.checksum_algorithm
    except OSError as e:
        logger.warning(f"Checksum failed for {path}: {e}")
        metadata.checksum_error = str(e)
```

**Cross-Language Validation**:

- All implementations MUST produce identical checksums for same file
- Use FulHash shared fixtures for testing
- Include integration tests with known files

### Testing Requirements

**Unit Tests**:

- Checksum calculation enabled/disabled
- Both algorithms (`xxh3-128`, `sha256`)
- Error handling (permission denied, missing file)
- Streaming correctness (large files)

**Integration Tests**:

- Cross-language checksum parity
- Performance benchmarks within target overhead
- Concurrent traversal with checksums

**Fixtures**:
Use FulHash shared fixtures (`config/library/fulhash/fixtures.yaml`) for validation.

## Repository Root Discovery

**Version**: 2025.11.2+ (v0.2.15)

Pathfinder provides safe upward filesystem traversal to locate repository markers (`.git`, `go.mod`, `package.json`, etc.), eliminating duplicated "walk up until marker found" implementations across Fulmen consumers.

### Use Cases

- Finding repository root from nested source directories
- Locating project configuration files (`.fulmen/app.yaml`, `go.mod`, `package.json`)
- Establishing path context for relative imports/paths
- Tool initialization (determining workspace boundaries)

### API Signature

**Go**:

```go
package pathfinder

// FindRepositoryRoot searches upward from startPath looking for marker files/directories.
// Returns the directory containing the first marker found, or error if not found.
func FindRepositoryRoot(startPath string, markers []string, opts ...FindOption) (string, error)

// Predefined marker sets for common repository types
var (
    GitMarkers      = []string{".git"}
    GoModMarkers    = []string{"go.mod"}
    NodeMarkers     = []string{"package.json"}
    PythonMarkers   = []string{"pyproject.toml", "setup.py"}
    MonorepoMarkers = []string{".git", "pnpm-workspace.yaml", "lerna.json"}
)

// FindOptions configures safety boundaries and behavior
type FindOptions struct {
    // StopAtFirst stops at first marker found (default: true)
    StopAtFirst bool

    // MaxDepth limits upward traversal (default: 10 directories)
    MaxDepth int

    // Boundary sets absolute ceiling path, never traverse above
    // Default: user home directory ($HOME, %USERPROFILE%)
    Boundary string

    // RespectConstraints integrates with PathConstraint if configured (default: true)
    RespectConstraints bool

    // FollowSymlinks whether to follow symlinks during traversal (default: false)
    FollowSymlinks bool
}
```

**Python**:

```python
from pyfulmen.pathfinder import find_repository_root, GitMarkers, GoModMarkers

# Returns path string or raises RepositoryNotFoundError
root = find_repository_root(
    start_path=".",
    markers=GitMarkers,
    max_depth=10,
    boundary=os.path.expanduser("~")
)
```

**TypeScript**:

```typescript
import { findRepositoryRoot, GitMarkers } from "@fulmenhq/tsfulmen/pathfinder";

// Returns path string or throws RepositoryNotFoundError
const root = await findRepositoryRoot(".", GitMarkers, {
  maxDepth: 10,
  boundary: os.homedir(),
});
```

### Safety Model

**CRITICAL**: This function MUST traverse above `pwd()` to be useful – that is its entire purpose (finding repository root when executing from nested subdirectories like `src/internal/config/`).

Safety comes not from **whether** we traverse upward, but from **where we stop**:

#### Default Safety Boundaries

1. **User Home Directory Ceiling**
   - **Default**: Never traverse above `$HOME` (Linux/macOS) or `%USERPROFILE%` (Windows) unless explicitly overridden
   - **Rationale**: Prevents escaping user-controlled filesystem space into system directories
   - **Example**: Search started in `/home/user/projects/myapp/src` stops at `/home/user`, never checks `/home` or `/`
   - **Windows-Specific**:
     - Never traverse past drive root (`C:\`, `D:\`) even if home directory is below it
     - Never traverse past UNC share root (`\\server\share\`) - treat share as boundary
     - Home directory on Windows: `%USERPROFILE%` (typically `C:\Users\{username}`)
   - **Container/CI Edge Case**: If `$HOME` is `/root` or `/`, fall back to current working directory as boundary (don't traverse entire filesystem)

2. **Filesystem Root**
   - **Absolute Ceiling**: `/` (Unix) or drive root `C:\` (Windows) or UNC share root (`\\server\share\`)
   - **Implementation**: Use `filepath.VolumeName()` (Go) or equivalent to detect filesystem boundaries
   - **Never traverse above**: Volume roots on any platform

3. **Max Depth Guard**
   - **Default**: 10 directories upward from start path
   - **Rationale**: Prevents infinite loops, runaway traversal, or symlink cycles
   - **Example**: Starting from `/a/b/c/d/e/f/g/h/i/j/k` with max_depth=10 stops at `/a`, never checks root

4. **PathConstraint Integration**
   - **When Configured**: Respect existing PathConstraint boundaries if application has configured them
   - **Example**: If PathConstraint defines repository root at `/home/user/monorepo`, stop there even if `.git` isn't found yet

#### Why Traversing Above pwd() Is Safe

**Common Misconception**: "Walking up the directory tree is dangerous."

**Reality**: Upward traversal is **inherently safer** than downward traversal because:

1. **Finite Search Space**: Walking up has exactly N steps where N = depth from filesystem root. Walking down is exponential (branching factor).

2. **Predictable Paths**: Each step up is deterministic (exactly one parent). Downward traversal requires glob matching and inclusion/exclusion logic.

3. **User-Owned Paths**: By default (home directory boundary), we only traverse directories the current user owns/controls. Downward traversal can encounter arbitrary file permissions, symlinks to sensitive paths, etc.

4. **Existing Precedent**: Standard tools do this safely:
   - `git status` walks up looking for `.git`
   - `npm install` walks up looking for `package.json`
   - Editor integrations (VS Code, JetBrains) walk up finding project roots
   - Language toolchains (`go build`, `cargo`, `poetry`) walk up finding manifests

5. **Read-Only Operation**: FindRepositoryRoot only **reads** directory entries looking for markers. It never writes, modifies, or executes anything. The attack surface is minimal (directory listing permissions).

6. **Bounded by Design**: Multiple safety nets (home directory, max depth, PathConstraint) ensure traversal cannot escape intended boundaries even in edge cases (symlinks, mount points, unusual filesystem structures).

**The Risk We're Managing**: Not that upward traversal is inherently dangerous, but that **unbounded** traversal could walk into system directories, traverse symlinks into unrelated subtrees, hit permission errors, or never terminate if markers not found.

**Our Mitigation**: Default home directory boundary + max depth guard + PathConstraint integration provide defense-in-depth against all these scenarios.

### Marker Matching Behavior

**To prevent implementation divergence across languages, the matching algorithm is specified**:

1. **Upward Traversal**: Start from `startPath`, walk upward one directory at a time
2. **Check Each Level**: At each parent directory, check if ANY marker from the list exists
3. **First Match Wins**: Stop at first directory containing ANY marker (closest to start)
4. **Return Directory**: Return the directory path, not the marker file path

**Example**:

```go
// Directory structure:
// /home/user/.git
// /home/user/projects/myapp/go.mod
// /home/user/projects/myapp/src/main.go  <- start here

root := FindRepositoryRoot(
    "/home/user/projects/myapp/src",
    []string{".git", "go.mod"},
)
// Returns: "/home/user/projects/myapp" (first parent with go.mod)
// Never checks /home/user because we stop at first match
```

**Marker List Ordering**: The order of markers in the array does NOT affect matching priority. At each directory level, we check for existence of ANY marker. The list order is purely for documentation/convention (recommend listing most-specific markers first).

**Multiple Markers in Same Directory**: If a directory contains multiple markers from the list (e.g., both `.git` and `go.mod`), return that directory immediately. No preference between markers.

### Error Handling

Errors conform to `schemas/pathfinder/v1.0.0/error-response.schema.json`:

```json
{
  "code": "REPOSITORY_NOT_FOUND",
  "message": "No repository markers found within search boundaries",
  "details": {
    "startPath": "/home/user/projects/myapp/src/internal/config",
    "markers": [".git", "go.mod"],
    "searchedDepth": 10,
    "stoppedAt": "/home/user",
    "reason": "boundary_reached"
  }
}
```

**Error Scenarios**:

1. **No Marker Found Within Boundaries**: Return `REPOSITORY_NOT_FOUND` with context
2. **Permission Denied on Parent**: Log warning, continue upward (partial traversal OK)
3. **Start Path Invalid/Missing**: Return `INVALID_START_PATH` immediately
4. **Boundary Misconfiguration**: Return `INVALID_BOUNDARY` if boundary path invalid
5. **Symlink Loop Detected**: Return `TRAVERSAL_LOOP` if `FollowSymlinks=true` and cycle detected

### Performance Considerations

- **Early Termination**: Stop immediately when first marker found (default `StopAtFirst: true`)
- **Minimal I/O**: Only check directory entries for marker names, no full file reads
- **No Recursion**: Iterative loop implementation to avoid stack depth issues
- **Platform-Native**: Use `filepath.WalkDir` style iteration (Go), `pathlib` (Python), `fs.promises` (TypeScript)
- **Expected Performance**: <5ms typical case (3-5 directories up), <20ms worst case (max depth reached)

### Integration with Existing Features

**PathConstraint Integration**:

When application has configured PathConstraint boundaries, FindRepositoryRoot respects them:

```go
// Application configures repository constraint
constraint := pathfinder.PathConstraint{
    Root:             "/home/user/monorepo",
    Type:            "repository",
    EnforcementLevel: "strict",
}

// FindRepositoryRoot respects this boundary
root, err := pathfinder.FindRepositoryRoot(
    "/home/user/monorepo/services/api/internal/config",
    pathfinder.GitMarkers,
    pathfinder.WithConstraint(constraint), // stops at /home/user/monorepo
)
```

**Identity Discovery Integration**:

Existing `.fulmen/app.yaml` ancestor search (v0.2.4) can migrate to use FindRepositoryRoot:

```go
import "github.com/fulmenhq/gofulmen/pathfinder"

func findAppYaml(start string) (string, error) {
    root, err := pathfinder.FindRepositoryRoot(start, []string{".fulmen"})
    if err != nil {
        return "", err
    }
    return filepath.Join(root, ".fulmen", "app.yaml"), nil
}
```

### Testing Requirements

**Unit Tests**:

- Boundary enforcement (home directory, explicit boundary, max depth)
- Marker matching (immediate parent, N levels up, first marker wins)
- Platform compatibility (Windows drive letters, UNC paths, Unix paths, symlinks)
- Error handling (invalid start path, permission denied, symlink loops, boundary misconfiguration)

**Integration Tests**:

- Cross-language parity (Go, Python, TypeScript produce identical results)
- Real repository detection (`.git`, `go.mod`, `package.json`, `pyproject.toml`)
- Monorepo scenarios with multiple markers

**Performance Tests**:

- Typical case (3-5 directories up): <5ms
- Max depth case (10 directories): <20ms
- No regression vs. existing pathfinder operations

### Usage Examples

**Go Example**:

```go
package main

import (
    "fmt"
    "github.com/fulmenhq/gofulmen/pathfinder"
)

func main() {
    // Find Git repository root
    root, err := pathfinder.FindRepositoryRoot(".", pathfinder.GitMarkers)
    if err != nil {
        fmt.Printf("Not in a Git repository: %v\n", err)
        return
    }
    fmt.Printf("Repository root: %s\n", root)

    // Find Go module root with custom boundary
    root, err = pathfinder.FindRepositoryRoot(
        "internal/config",
        pathfinder.GoModMarkers,
        pathfinder.WithMaxDepth(5),
        pathfinder.WithBoundary("/home/user/projects"),
    )
}
```

**Python Example**:

```python
from pyfulmen.pathfinder import (
    find_repository_root,
    GitMarkers,
    RepositoryNotFoundError
)

try:
    root = find_repository_root(".", GitMarkers)
    print(f"Repository root: {root}")
except RepositoryNotFoundError as e:
    print(f"Not in a Git repository: {e}")
```

**TypeScript Example**:

```typescript
import { findRepositoryRoot, GitMarkers } from "@fulmenhq/tsfulmen/pathfinder";

async function findRoot() {
  try {
    const root = await findRepositoryRoot(".", GitMarkers);
    console.log(`Repository root: ${root}`);
  } catch (err) {
    console.error(`Not in a Git repository: ${err.message}`);
  }
}
```

### Implementation Notes

- **Go**: Use iterative parent directory traversal with `filepath.Dir()`, check marker existence with `os.Stat()`
- **Python**: Use `pathlib.Path.parent` iteration, check markers with `Path.exists()`
- **TypeScript**: Use `path.dirname()` iteration, check with `fs.promises.access()` or `fs.promises.stat()`
- **All**: Pre-validate boundary path exists and is absolute before traversal begins

## Status

- Optional; recommended for CLI-heavy foundations. Document adoption in module manifest overrides.
- Checksum support added in 2025.10.3 via FulHash integration.
- Repository root discovery added in 2025.11.2 (v0.2.15) for safe upward traversal.
