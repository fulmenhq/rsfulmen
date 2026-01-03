---
title: "Fulpack Archive Module Standard"
description: "Canonical API specification for archive operations with Pathfinder integration"
author: "Schema Cartographer"
date: "2025-11-15"
version: "1.0.0"
status: "active"
tier: "common"
tags: ["archive", "compression", "pathfinder", "tar", "zip", "gzip"]
---

# Fulpack Archive Module Standard

## Overview

**Fulpack** is a Common-tier helper library module providing canonical archive operations (create, extract, scan, verify, info) with **mandatory Pathfinder integration** for glob-based file discovery within archives. This specification defines the cross-language API contract that all Fulmen helper libraries must implement.

**Module Tier**: Common
**Version**: 1.0.0
**Added**: Crucible v0.2.10
**Dependencies**:

- pathfinder (required) - Glob-based file discovery
- fulhash (required) - Checksum generation and verification

## Architecture Principles

### Canonical Façade Principle

Per the [Fulmen Helper Library Standard](../../../architecture/fulmen-helper-library-standard.md#canonical-façade-principle), fulpack provides **canonical façades** wrapping standard library functionality to ensure:

1. **Cross-Language Interface Consistency** - Same operations, same error envelopes, same behavior
2. **Pathfinder Integration** - Unified glob search across filesystems and archives
3. **Security by Default** - Path traversal protection, decompression bomb detection
4. **Taxonomy-Driven Design** - Archive formats, operations, and entry types defined in SSOT

### Why Common Tier?

**Common tier assignment rationale**:

- **Universal need**: Most applications need basic archive handling (tar, tar.gz, zip, gzip)
- **Zero external dependencies**: Uses language stdlib only (tarfile, zipfile, gzip in Python; archive/tar, archive/zip, compress/gzip in Go; tar-stream, archiver in TypeScript)
- **Pathfinder integration**: Enables unified file discovery API across filesystem and archives
- **Security requirement**: Consistent path traversal protection and bomb detection

**This is NOT a specialized module** despite wrapping stdlib—the Canonical Façade Principle mandates façades for universal capabilities regardless of implementation strategy.

## Taxonomy-Driven Design

### Archive Formats Taxonomy

**Location**: `schemas/taxonomy/library/fulpack/archive-formats/v1.0.0/formats.yaml`

**Supported formats** (v1.0.0):

- `tar` - POSIX tar (uncompressed)
- `tar.gz` - POSIX tar with gzip compression
- `zip` - ZIP archive with deflate compression
- `gzip` - GZIP compressed single file

**Rationale for uncompressed tar support**: While less common at large scale, uncompressed tar archives are used for:

- Pre-compressed data (images, videos, already-compressed files)
- Streaming scenarios where compression CPU overhead is undesirable
- Compatibility with legacy systems and tools
- Maximum speed operations (no compression/decompression overhead)

**Format features**:

```yaml
tar:
  preserves_permissions: true
  preserves_timestamps: true
  supports_symlinks: true
  supports_directories: true
  compression: none
  use_case: "Maximum speed, pre-compressed data, streaming"

tar.gz:
  preserves_permissions: true
  preserves_timestamps: true
  supports_symlinks: true
  supports_directories: true
  compression: gzip
  use_case: "General purpose, best compatibility"

zip:
  preserves_permissions: false # Limited on Windows
  preserves_timestamps: true
  supports_symlinks: false
  supports_directories: true
  compression: deflate
  use_case: "Windows compatibility, random access"

gzip:
  preserves_permissions: false
  preserves_timestamps: true
  supports_symlinks: false
  supports_directories: false # Single file only
  compression: gzip
  use_case: "Single file compression only"
```

Libraries generate enums from this taxonomy:

```python
# Generated in pyfulmen
class ArchiveFormat(Enum):
    TAR = "tar"
    TAR_GZ = "tar.gz"
    ZIP = "zip"
    GZIP = "gzip"
```

### Operations Taxonomy

**Location**: `schemas/taxonomy/library/fulpack/operations/v1.0.0/operations.yaml`

**Five canonical operations**:

1. `create` - Create archive from source files/directories
2. `extract` - Extract archive contents to destination
3. `scan` - List archive entries (for Pathfinder integration)
4. `verify` - Validate archive integrity and checksums
5. `info` - Get archive metadata without extraction

### Entry Types Taxonomy

**Location**: `schemas/taxonomy/library/fulpack/entry-types/v1.0.0/types.yaml`

**Canonical entry types**:

- `file` - Regular file with data
- `directory` - Directory/folder entry
- `symlink` - Symbolic link (requires security validation)

## Canonical API Specification

### 1. create() - Create Archive

**Signature** (TypeScript pseudocode):

```typescript
create(
  source: string | string[],
  output: string,
  format: ArchiveFormat,
  options?: CreateOptions
): ArchiveInfo
```

**Parameters**:

- `source`: Single path or array of paths to archive
- `output`: Output archive file path
- `format`: `ArchiveFormat.TAR`, `ArchiveFormat.TAR_GZ`, `ArchiveFormat.ZIP`, or `ArchiveFormat.GZIP`
- `options`: Optional creation options

**CreateOptions** (from `schemas/library/fulpack/v1.0.0/create-options.schema.json`):

```typescript
{
  compression_level?: number,       // 1-9 (default: 6), ignored for uncompressed tar
  include_patterns?: string[],      // e.g., ["**/*.py", "**/*.md"]
  exclude_patterns?: string[],      // e.g., ["**/__pycache__", "**/.git"]
  checksum_algorithm?: string,      // "xxh3-128" | "sha256" | "sha512" | "sha1" | "md5" (default: "sha256")
                                    // Standard: xxh3-128 (fast), sha256 (cryptographic) via fulhash module
                                    // Optional: sha512, sha1, md5 (may require extensions)
  preserve_permissions?: boolean,   // default: true
  follow_symlinks?: boolean         // default: false
}
```

**Note**: `compression_level` is ignored for `ArchiveFormat.TAR` (uncompressed) and `ArchiveFormat.GZIP` (uses default level 6).

**Returns**: `ArchiveInfo` with metadata (entry_count, sizes, checksums)

**Security**:

- Validates paths before archiving
- Applies path traversal protection
- Symlinks: Only followed if `follow_symlinks: true`

**Example** (Python):

```python
from pyfulmen import fulpack
from pyfulmen.fulpack import ArchiveFormat

# Compressed archive (most common)
info = fulpack.create(
    source=["src/", "docs/", "README.md"],
    output="release.tar.gz",
    format=ArchiveFormat.TAR_GZ,
    options={
        "exclude_patterns": ["**/__pycache__", "**/.git"],
        "compression_level": 9
    }
)
print(f"Created archive with {info.entry_count} entries, {info.compression_ratio}x compression")

# Uncompressed tar (maximum speed, pre-compressed data)
info_tar = fulpack.create(
    source=["videos/", "images/"],  # Already compressed media files
    output="media.tar",
    format=ArchiveFormat.TAR,
    options={
        "preserve_permissions": True
        # compression_level ignored for uncompressed tar
    }
)
print(f"Created uncompressed tar: {info_tar.total_size} bytes (compression_ratio: {info_tar.compression_ratio})")
# compression_ratio will be 1.0 for uncompressed tar
```

---

### 2. extract() - Extract Archive

**Signature**:

```typescript
extract(
  archive: string,
  destination: string,
  options?: ExtractOptions
): ExtractResult
```

**Parameters**:

- `archive`: Archive file path
- `destination`: Target directory (must be explicit, no CWD extraction)
- `options`: Optional extraction options

**ExtractOptions** (from `schemas/library/fulpack/v1.0.0/extract-options.schema.json`):

```typescript
{
  overwrite?: "error" | "skip" | "overwrite",  // default: "error"
  verify_checksums?: boolean,                   // default: true
  preserve_permissions?: boolean,               // default: true
  include_patterns?: string[],                  // e.g., ["**/*.csv"]
  max_size?: number,                            // default: 1GB (bomb protection)
  max_entries?: number                          // default: 10000 (bomb protection)
}
```

**Returns**: `ExtractResult` with extracted_count, skipped, errors

**Security** (MANDATORY for all implementations):

- **Path traversal protection**: Reject `../`, absolute paths, escapes outside destination
- **Symlink validation**: Reject symlinks targeting outside destination
- **Decompression bomb protection**: Enforce `max_size` and `max_entries` limits
- **Checksum verification**: Verify checksums if present (unless disabled)

**Example** (Go):

```go
import "github.com/fulmenhq/gofulmen/fulpack"

result, err := fulpack.Extract(
    "data.tar.gz",
    "/tmp/extracted",
    &fulpack.ExtractOptions{
        IncludePatterns: []string{"**/*.csv"},
        VerifyChecksums: true,
    },
)
if err != nil {
    return err
}
fmt.Printf("Extracted %d files, skipped %d\n", result.ExtractedCount, result.SkippedCount)
```

---

### 3. scan() - Scan Archive (Pathfinder Integration)

**Signature**:

```typescript
scan(
  archive: string,
  options?: ScanOptions
): ArchiveEntry[]
```

**Parameters**:

- `archive`: Archive file path
- `options`: Optional scan options

**ScanOptions** (from `schemas/library/fulpack/v1.0.0/scan-options.schema.json`):

```typescript
{
  include_metadata?: boolean,       // default: true
  entry_types?: string[],           // ["file", "directory", "symlink"]
  max_depth?: number | null,        // default: null (unlimited)
  max_entries?: number              // default: 100000 (safety limit)
}
```

**Returns**: Array of `ArchiveEntry` objects with:

- `path` - Normalized entry path
- `type` - "file" | "directory" | "symlink"
- `size` - Uncompressed size
- `compressed_size` - Compressed size (if available)
- `modified` - Modification timestamp
- `checksum` - SHA-256 checksum (if available)
- `mode` - Unix permissions (if available)

**Purpose**: Enables Pathfinder glob searches within archives without extraction

**Performance**: Lazy evaluation—reads archive TOC/directory, does NOT extract files

**Pathfinder Integration Pattern**:

```python
from pyfulmen import fulpack, pathfinder

# Pathfinder delegates to fulpack for archives
entries = fulpack.scan("data.tar.gz")
matches = pathfinder.glob(entries, "**/*.csv")  # Pathfinder applies glob

# Or use Pathfinder's unified API
results = pathfinder.find(
    source="data.tar.gz",
    pattern="**/*.csv"
)
# Pathfinder detects archive, calls fulpack.scan(), applies pattern
```

**Example** (TypeScript):

```typescript
import { fulpack, ArchiveFormat } from "@fulmenhq/tsfulmen/fulpack";

const entries = fulpack.scan("data.tar.gz", {
  includeMetadata: true,
  entryTypes: ["file"], // Only files, no directories
});

// Filter entries by pattern (or let Pathfinder do it)
const csvFiles = entries.filter((e) => e.path.endsWith(".csv"));
console.log(`Found ${csvFiles.length} CSV files`);
```

---

### 4. verify() - Verify Archive Integrity

**Signature**:

```typescript
verify(
  archive: string,
  options?: VerifyOptions
): ValidationResult
```

**Parameters**:

- `archive`: Archive file path
- `options`: Optional verification options (future use)

**Returns**: `ValidationResult` with:

- `valid` - Boolean indicating if archive is intact
- `errors` - Array of validation errors
- `warnings` - Array of warnings (e.g., missing checksums)
- `entry_count` - Number of entries validated
- `checksums_verified` - Count of checksum validations
- `checks_performed` - List of checks (structure_valid, checksums_verified, no_path_traversal, no_decompression_bomb, symlinks_safe)

**Security checks**:

- Archive structure integrity
- Checksum verification (if present)
- Path traversal detection
- Decompression bomb detection (compression ratio, entry count)
- Symlink safety (targets within bounds)

**Example** (Python):

```python
result = fulpack.verify("data.tar.gz")
if not result.valid:
    print(f"Archive validation failed: {result.errors}")
else:
    print(f"Archive valid: {result.entry_count} entries, {result.checksums_verified} checksums verified")
    if result.warnings:
        print(f"Warnings: {result.warnings}")
```

---

### 5. info() - Get Archive Metadata

**Signature**:

```typescript
info(
  archive: string
): ArchiveInfo
```

**Parameters**:

- `archive`: Archive file path

**Returns**: `ArchiveInfo` with:

- `format` - Detected archive format (enum)
- `compression` - Compression algorithm (enum)
- `entry_count` - Total entries
- `total_size` - Uncompressed total size
- `compressed_size` - Archive file size
- `compression_ratio` - Ratio calculation
- `has_checksums` - Boolean
- `checksum_algorithm` - Algorithm used (if has_checksums)
- `created` - Archive creation timestamp (if available)

**Use cases**:

- Quick inspection without extraction
- Format detection
- Size estimation before extraction
- Compression ratio analysis

**Example** (Go):

```go
info, err := fulpack.Info("release.tar.gz")
if err != nil {
    return err
}
fmt.Printf("Format: %s, Entries: %d, Compression: %.1fx\n",
    info.Format, info.EntryCount, info.CompressionRatio)
```

---

## Streaming API (Planned - Implementation Deferred)

**Status**: API reserved, implementation deferred to v0.2.11+

**Why plan now**: Ensures block API doesn't prevent streaming later; reserves method names; defines resource cleanup patterns

**Streaming operations** (future):

- `create_stream()` - Streaming archive creation
- `extract_stream()` - Streaming extraction
- `scan_stream()` - Streaming scan (large archives)

**Language-specific patterns**:

- **Python**: Context managers (`with`), generators, async iterators
- **Go**: `io.Reader`/`io.Writer`, `defer` cleanup, channel-based iteration
- **TypeScript**: Async iterators, `ReadableStream`/`WritableStream`

### Forward Compatibility Confirmation

**Schema Compatibility**: Current schemas are designed to support streaming without breaking changes

**No schema migrations required** when adding streaming in v0.2.11:

1. **Operation schemas remain unchanged**: Streaming variants use same option/result schemas
   - `CreateOptions` works for both `create()` and `create_stream()`
   - `ExtractResult` works for both `extract()` and `extract_stream()`

2. **New method names**: Streaming uses distinct names (`*_stream`)
   - No conflicts with existing methods
   - Both APIs can coexist in same module

3. **Resource cleanup**: Schemas don't dictate cleanup patterns
   - Python: Add context manager protocol to stream objects
   - Go: Add `Close()` method to stream types
   - TypeScript: Add `finally()` to promise chains

4. **Validation unchanged**: Same schemas validate both modes
   - Block API: Full object validation
   - Stream API: Per-entry validation

**Example (forward-compatible implementation)**:

```python
# v1.0.0 (block API)
result = fulpack.extract("archive.tar.gz", "/dest", options)

# v2.0.0 (streaming API added, no schema changes)
with fulpack.extract_stream("archive.tar.gz", "/dest", options) as stream:
    for entry in stream:
        # Process entry with same ArchiveEntry schema
        pass
# Returns same ExtractResult schema
```

**Conclusion**: Current v1.0.0 schemas are streaming-ready. No breaking changes needed for v0.2.11 streaming implementation.

See feature brief for detailed streaming API specification.

---

## Security Model

### Mandatory Protections

**ALL implementations MUST enforce these protections**:

#### 1. Path Traversal Protection

- Normalize all entry paths
- Reject absolute paths (e.g., `/etc/passwd`)
- Reject paths containing `../` (parent directory traversal)
- Validate symlink targets are within archive/destination bounds
- Enforce destination directory bounds during extraction

#### 2. Decompression Bomb Protection

- Max entry size limit (default: 1GB)
- Max total size limit (default: 10GB)
- Max entries limit (default: 100k)
- Monitor compression ratio (warn if >100:1)

#### 3. Checksum Verification (via fulhash module)

- Use fulhash module for checksum generation and verification
- **Standard algorithms**: `xxh3-128` (fast, non-cryptographic), `sha256` (cryptographic)
- **Optional algorithms**: `sha512`, `sha1`, `md5` (may require extension modules)
- Verify checksums if present in archive
- Report missing checksums as warnings (not errors)
- Fail extraction on checksum mismatch (unless `verify_checksums: false`)
- Default algorithm: `sha256` (balance of security and compatibility)

#### 4. Safe Defaults

- No extraction to CWD (require explicit destination)
- Error on overwrite conflicts (unless `overwrite: "skip"` or `"overwrite"`)
- Preserve permissions only if requested
- Don't follow symlinks unless `follow_symlinks: true`

### Security Test Requirements

All implementations MUST pass security tests for:

- Path traversal attempts (`../../../etc/passwd`)
- Absolute path attacks (`/etc/passwd`)
- Symlink escapes (symlink targeting outside bounds)
- Decompression bombs (10MB → 10GB expansion)
- Checksum tampering detection

---

## Pathfinder Integration Specification

### Integration Pattern

**Pathfinder** provides a unified file discovery API that works seamlessly across:

- Local filesystems
- Archives (via fulpack)
- Network resources (future)

**Workflow**:

1. User calls `pathfinder.find("data.tar.gz", "**/*.csv")`
2. Pathfinder detects archive format
3. Pathfinder calls `fulpack.scan("data.tar.gz", options)`
4. Fulpack returns `ArchiveEntry[]` with normalized paths
5. Pathfinder applies glob pattern to entry paths
6. Pathfinder returns matching entries

**No extraction required**: `scan()` only reads archive TOC/directory

**Performance target**: <1s for TOC read, regardless of archive size

### Edge Case Handling (Cross-Language Determinism)

**CRITICAL**: All implementations MUST handle these edge cases identically to ensure Pathfinder returns the same results across Go/Python/TypeScript:

#### 1. Symlinks in Archives

**Behavior**: `scan()` MUST include symlink entries in results with `type: "symlink"`

- Include `symlink_target` field with original target path (not resolved)
- DO NOT follow symlinks during scan (no recursive resolution)
- DO NOT validate symlink targets during scan (validation happens in `verify()` or `extract()`)
- Pathfinder applies glob to symlink paths, not targets

**Rationale**: Symlink validation is security-critical but separate from discovery. `scan()` provides raw TOC; `verify()`/`extract()` enforce security.

**Example**:

```yaml
# Archive contains: docs/link.md -> ../secret.txt
# scan() returns:
- path: "docs/link.md"
  type: "symlink"
  symlink_target: "../secret.txt" # Original target, not resolved
  size: 0
```

#### 2. Invalid UTF-8 Paths

**Behavior**: `scan()` MUST handle invalid UTF-8 in entry paths deterministically

- **Preferred**: Normalize to UTF-8 using replacement character (U+FFFD)
- **Alternative**: Base64-encode invalid paths with prefix `base64:` (for exact preservation)
- **Document choice**: Each language documents its approach in implementation notes
- **Cross-language tests**: Fixtures MUST include invalid UTF-8 paths to verify consistency

**Rationale**: Archives may contain non-UTF-8 paths (legacy encodings, binary names). Implementations must handle gracefully without crashing.

**Example (replacement character approach)**:

```yaml
# Archive contains: data/file_\xFF\xFE.txt (invalid UTF-8)
# scan() returns:
- path: "data/file_��.txt" # U+FFFD replacement characters
  type: "file"
```

#### 3. Absolute Paths

**Behavior**: `scan()` MUST normalize absolute paths to relative

- Strip leading `/` or drive letters (Windows: `C:/`)
- Emit warning in scan results
- Include in results (don't skip) so Pathfinder can match
- `extract()` and `verify()` MUST reject absolute paths (security)

**Rationale**: `scan()` is discovery; `extract()` is security enforcement. Separation of concerns.

#### 4. Path Traversal Attempts

**Behavior**: `scan()` MUST include `../` paths in results

- DO NOT normalize or reject during scan
- Include in results so Pathfinder can discover them
- `verify()` MUST flag as security warnings
- `extract()` MUST reject with error

**Rationale**: Security tests need to verify that archives with malicious paths are detected and rejected.

### Example Integration

```python
from pyfulmen import pathfinder

# Unified API - pathfinder handles routing
files = pathfinder.find(
    source="data.tar.gz",
    pattern="**/*.csv",
    options={"include_metadata": True}
)

for file in files:
    print(f"{file.path}: {file.size} bytes")
```

---

## Implementation Guidance

### Language-Specific Notes

**Go** (`github.com/fulmenhq/gofulmen/fulpack`):

- Use `archive/tar`, `archive/zip`, `compress/gzip` from stdlib
- Errors returned as `error` type with wrapping
- Enums generated from taxonomy YAML

**Python** (`pyfulmen.fulpack`):

- Use `tarfile`, `zipfile`, `gzip` from stdlib
- Enums generated from taxonomy YAML
- Context managers for file handles
- Exceptions raised for errors

**TypeScript** (`@fulmenhq/tsfulmen/fulpack`):

- Use `tar-stream` and `archiver` for cross-platform compatibility
- Enums generated from taxonomy YAML
- Promises for async operations
- Errors thrown as `Error` instances

### Error Handling

All implementations MUST:

- Use Foundry error schemas for consistency
- Provide clear error messages with context
- Distinguish between validation errors (invalid input) and runtime errors (I/O failures)
- Return structured error envelopes with exit codes

#### Canonical Error Envelope

All fulpack errors MUST use this envelope structure (compatible with Foundry error schemas):

```typescript
interface FulpackError {
  code: string; // Canonical error code (see below)
  message: string; // Human-readable message
  path?: string; // Entry path that caused error (if applicable)
  archive?: string; // Archive file path
  operation: string; // Operation name (create, extract, scan, verify, info)
  details?: {
    // Optional context
    entry_index?: number;
    compression_ratio?: number;
    actual_size?: number;
    max_size?: number;
  };
}
```

#### Canonical Error Codes

**Validation Errors** (invalid input):

- `INVALID_ARCHIVE_FORMAT` - Archive format not recognized
- `INVALID_PATH` - Entry path contains invalid characters
- `INVALID_OPTIONS` - Invalid options passed to operation

**Security Errors** (protection triggered):

- `PATH_TRAVERSAL` - Entry path attempts directory traversal (`../`)
- `ABSOLUTE_PATH` - Entry path is absolute (`/etc/passwd`)
- `SYMLINK_ESCAPE` - Symlink target outside archive/destination bounds
- `DECOMPRESSION_BOMB` - Archive exceeds size/entry limits
- `CHECKSUM_MISMATCH` - Entry checksum verification failed

**Runtime Errors** (I/O failures):

- `ARCHIVE_NOT_FOUND` - Archive file does not exist
- `ARCHIVE_CORRUPT` - Archive structure is invalid/corrupted
- `EXTRACTION_FAILED` - Failed to write extracted file
- `PERMISSION_DENIED` - Insufficient permissions to read/write
- `DISK_FULL` - Insufficient disk space for extraction

**Example Usage**:

```python
# Decompression bomb detection
raise FulpackError(
    code="DECOMPRESSION_BOMB",
    message="Archive exceeds maximum size limit",
    archive="malicious.tar.gz",
    operation="extract",
    details={
        "actual_size": 10737418240,  # 10GB
        "max_size": 1073741824,      # 1GB limit
        "compression_ratio": 1000
    }
)

# Checksum mismatch
raise FulpackError(
    code="CHECKSUM_MISMATCH",
    message="Entry checksum verification failed",
    archive="data.tar.gz",
    path="data/corrupted.csv",
    operation="extract",
    details={"entry_index": 42}
)
```

### Testing Requirements

All implementations MUST provide:

- **Unit tests**: Each operation tested in isolation
- **Security tests**: Path traversal, bombs, checksums
- **Integration tests**: Pathfinder integration
- **Fixture tests**: Using fixtures from `config/library/fulpack/fixtures/`
- **Portable testing compliance**: Follow [Portable Testing Practices](../../testing/portable-testing-practices.md)
  - **Critical for fulpack**: Temp file cleanup, in-memory testing where possible, capability detection for filesystem features
  - See [Coding Standards](../../coding/README.md) for language-specific testing patterns

**Test coverage target**: ≥70%

**Sandbox audit requirements**:

Fulpack implementations MUST pass tests in restricted environments (no network, limited filesystem access) to support security audits and sandboxed CI/CD pipelines.

- **Network isolation**: Tests MUST NOT require network/DNS access
  - Pathological archive tests (malicious paths, bombs) run entirely local
  - No external downloads or validation services
- **Filesystem capability detection**: Use skip helpers for optional features
  - Symlink support: Skip symlink tests if OS doesn't support them
  - Compression algorithms: Skip tests for unavailable algorithms (e.g., xz, zstd)
  - Permissions: Skip permission tests on Windows FAT32 filesystems
- **Temporary file cleanup**: MUST clean up ALL temp files in test teardown
  - Go: Use `t.Cleanup()` to register cleanup handlers
  - Python: Use pytest `tmp_path` fixture or `@pytest.fixture` with cleanup
  - TypeScript: Use `afterEach()` to remove temp directories
  - **Why critical**: Temp file leaks cause audit failures and disk exhaustion
- **Memory limits**: Decompression bomb tests MUST verify memory limits work
  - Test that `max_size` limit prevents OOM in bomb scenarios
  - Use in-memory archives for unit tests (avoid disk I/O)
- **Deterministic extraction**: Same archive → same output across environments
  - Verify timestamp preservation, permission handling, path normalization
  - Test cross-platform compatibility (Windows paths, Unix permissions)

---

## Test Fixtures

**Location**: `config/library/fulpack/fixtures/`

**Three canonical fixtures**:

1. **basic.tar.gz** - Normal archive structure
   - 10 files in simple directory tree
   - All formats supported (tar.gz, zip versions)
   - Used for basic operation testing

2. **nested.zip** - 3-level directory nesting
   - Tests deep directory traversal
   - Tests path normalization
   - Tests scan with `max_depth` option

3. **pathological.tar.gz** - Security test cases
   - Contains path traversal attempts (`../../../etc/passwd`)
   - Contains absolute paths (`/etc/passwd`)
   - Contains symlink escapes
   - MUST be rejected by extract/scan operations

### Fixture Governance

**Adding New Fixtures**:

1. **Naming Convention**: `{category}-{description}.{format}`
   - Categories: `basic`, `nested`, `pathological`, `utf8`, `symlink`, `large`, `corrupt`
   - Examples: `pathological-traversal.tar.gz`, `utf8-invalid-paths.zip`, `large-10k-entries.tar.gz`

2. **Approval Process**:
   - Create fixture locally and test in your library
   - Document fixture purpose and expected behavior in PR description
   - Add fixture to `config/library/fulpack/fixtures/`
   - Add test cases to validate fixture behavior
   - Request review from Schema Cartographer before merging to Crucible

3. **Documentation**: Each fixture should have an accompanying `.txt` or `.md` file describing:
   - Purpose (what it tests)
   - Expected behavior (pass/fail conditions)
   - Contents summary (number of files, structure)
   - Special characteristics (invalid UTF-8, symlinks, etc.)

4. **Size Limits**:
   - Basic fixtures: <10KB
   - Pathological fixtures: <50KB
   - Large fixtures (if needed): <1MB, must justify in PR

5. **Cross-Language Parity**:
   - New fixtures MUST work identically across all language implementations
   - Include fixture in parity test suites
   - Document any language-specific behavior (e.g., UTF-8 handling differences)

**Example fixture documentation** (`pathological-traversal.tar.gz.txt`):

```
Fixture: pathological-traversal.tar.gz
Purpose: Test path traversal protection
Expected: extract() and verify() MUST reject this archive
Contents:
  - safe.txt (normal file)
  - ../../../etc/passwd (traversal attempt)
  - /root/.ssh/id_rsa (absolute path)
Behavior:
  - scan() MUST list all entries (including malicious paths)
  - verify() MUST return valid=false with PATH_TRAVERSAL errors
  - extract() MUST fail with PATH_TRAVERSAL error
```

---

## Schema References

**Taxonomy schemas**:

- `schemas/taxonomy/library/fulpack/archive-formats/v1.0.0/formats.yaml`
- `schemas/taxonomy/library/fulpack/operations/v1.0.0/operations.yaml`
- `schemas/taxonomy/library/fulpack/entry-types/v1.0.0/types.yaml`

**Data structure schemas**:

- `schemas/library/fulpack/v1.0.0/archive-info.schema.json`
- `schemas/library/fulpack/v1.0.0/archive-entry.schema.json`
- `schemas/library/fulpack/v1.0.0/archive-manifest.schema.json`
- `schemas/library/fulpack/v1.0.0/validation-result.schema.json`
- `schemas/library/fulpack/v1.0.0/create-options.schema.json`
- `schemas/library/fulpack/v1.0.0/extract-options.schema.json`
- `schemas/library/fulpack/v1.0.0/scan-options.schema.json`
- `schemas/library/fulpack/v1.0.0/extract-result.schema.json`

---

## Version History

- **1.0.0** (2025-11-15) - Initial specification
  - 4 archive formats (tar, tar.gz, zip, gzip)
  - 5 operations (create, extract, scan, verify, info)
  - Pathfinder integration
  - Security model defined
  - Streaming API reserved (implementation deferred)
  - Telemetry specification added (2025-11-12)

---

## Future Enhancements

**Planned for fulpack v2.0.0+**:

- Streaming API implementation (create_stream, extract_stream, scan_stream)
- Archive composition (nested archives)
- Additional stdlib-compatible compression formats (tar.bz2 using bz2 stdlib module)

**Deferred to fulpack-formats (Specialized tier module)**:

- Modern compression formats requiring external dependencies:
  - `tar.zst` (Zstandard compression - requires external library)
  - `tar.xz` (LZMA/XZ compression - requires external library)
  - `7z` (7-Zip format - requires external library)
  - `rar` (RAR format - requires external library, licensing restrictions)
- **Rationale**: These formats require non-stdlib dependencies and are less common in production environments

**Deferred to fulcloak (Specialized tier module - Encryption & Security)**:

- **File and archive encryption**:
  - Encrypted ZIP (AES-256 encryption within ZIP format)
  - GPG/PGP encryption (asymmetric encryption with key management)
  - Age encryption (modern alternative to GPG)
  - Encrypted tar streams (tar.gz.gpg, tar.age)
- **Digital signatures and verification**:
  - Detached signatures (GPG .sig files, OpenSSL signatures)
  - In-archive signature verification
  - Certificate chain validation
- **Key management integration**:
  - Hardware security module (HSM) support
  - Cloud KMS integration (AWS KMS, GCP KMS, Azure Key Vault)
  - Local keyring integration
- **Rationale**: Encryption and cryptographic operations require:
  - Complex key management and secure credential handling
  - External cryptographic libraries with careful version management
  - Compliance considerations (FIPS 140-2, export controls)
  - Specialized security expertise for safe implementation
  - Separate security audit and vulnerability management

**Clear separation of concerns**:

- **fulpack (Common tier)**: Plain archives with checksum integrity (SHA-256)
- **fulcloak (Specialized tier)**: Cryptographic operations requiring key management and compliance considerations

**Note**: Applications needing encrypted archives TODAY should use external tools (gpg, age, openssl) to wrap fulpack operations until fulcloak is available.

---

## Telemetry Specification

**Module**: Uses Core-tier `telemetry` module for metrics instrumentation

All fulpack implementations MUST instrument the following metrics using the helper library's telemetry module. Metrics enable performance monitoring, security alerting, and usage analytics in production.

### Core Operation Metrics

**Operation Duration** (Histogram):

```
fulpack.operation.duration_seconds
Labels: operation={create|extract|scan|verify|info}, format={tar|tar.gz|zip|gzip}, status={success|error}
Buckets: [0.1, 0.5, 1, 2, 5, 10, 30, 60]
Purpose: Track operation latency for performance monitoring and SLO tracking
Note: Expect tar (uncompressed) to be faster than tar.gz for create/extract operations
```

**Operation Count** (Counter):

```
fulpack.operation.total
Labels: operation={create|extract|scan|verify|info}, format={tar|tar.gz|zip|gzip}, status={success|error}
Purpose: Track operation volume and success/failure rates
```

**Example usage**:

```python
from pyfulmen import telemetry, fulpack

with telemetry.histogram("fulpack.operation.duration_seconds",
                          labels={"operation": "extract", "format": "tar.gz"}):
    result = fulpack.extract(archive, destination, options)
    telemetry.counter("fulpack.operation.total",
                      labels={"operation": "extract", "format": "tar.gz", "status": "success"})
```

### Archive Characteristics Metrics

**Archive Size** (Histogram):

```
fulpack.archive.size_bytes
Labels: type={compressed|uncompressed}, format={tar|tar.gz|zip|gzip}
Buckets: [1024, 10240, 102400, 1048576, 10485760, 104857600, 1073741824, 10737418240]
Purpose: Track archive size distribution for capacity planning and anomaly detection
Note: For format=tar, compressed and uncompressed sizes are equal (compression_ratio=1.0)
```

**Entry Count** (Histogram):

```
fulpack.archive.entries
Labels: format={tar|tar.gz|zip|gzip}
Buckets: [10, 50, 100, 500, 1000, 5000, 10000, 50000, 100000]
Purpose: Track archive complexity for performance tuning and bomb detection thresholds
```

**Compression Ratio** (Histogram):

```
fulpack.archive.compression_ratio
Labels: format={tar|tar.gz|zip|gzip}
Buckets: [1, 2, 5, 10, 20, 50, 100, 200]
Purpose: Monitor compression efficiency and detect potential decompression bombs
Note: For format=tar (uncompressed), ratio is always 1.0; useful for comparing format choices
```

### Security Metrics

**Security Violations Detected** (Counter):

```
fulpack.security.violations_total
Labels: type={path_traversal|absolute_path|symlink_escape|decompression_bomb|checksum_mismatch}, operation={extract|verify|scan}
Purpose: Track security threats detected for alerting and audit logging
Alert threshold: Any non-zero value in extract/verify operations
```

**Checksum Verification Results** (Counter):

```
fulpack.checksum.verifications_total
Labels: result={verified|failed|missing}, algorithm={xxh3-128|sha256|sha512|sha1|md5}
Purpose: Track data integrity validation coverage and failure rates
Note: xxh3-128 and sha256 are standard via fulhash module; others may require extensions
```

**Example security monitoring**:

```python
# In extract() implementation
if path_traversal_detected:
    telemetry.counter("fulpack.security.violations_total",
                      labels={"type": "path_traversal", "operation": "extract"})
    raise FulpackError(code="PATH_TRAVERSAL", ...)
```

### Performance Metrics

**Bytes Processed** (Counter):

```
fulpack.bytes_processed_total
Labels: operation={create|extract}, direction={read|write}
Purpose: Track I/O volume for throughput analysis and cost allocation
```

**Entries Processed** (Counter):

```
fulpack.entries_processed_total
Labels: operation={create|extract|scan}, status={success|skipped|error}
Purpose: Track granular operation progress and failure patterns
```

**Scan TOC Read Duration** (Histogram):

```
fulpack.scan.toc_read_seconds
Labels: format={tar|tar.gz|zip|gzip}
Buckets: [0.01, 0.05, 0.1, 0.5, 1, 2, 5]
Purpose: Monitor scan() performance against <1s target
Alert threshold: p95 > 1.0 seconds
Note: Expect tar (uncompressed) scans to be fastest (no decompression overhead)
```

### Error Metrics

**Errors by Code** (Counter):

```
fulpack.errors_total
Labels: code={INVALID_ARCHIVE_FORMAT|INVALID_PATH|PATH_TRAVERSAL|ARCHIVE_NOT_FOUND|ARCHIVE_CORRUPT|EXTRACTION_FAILED|PERMISSION_DENIED|DISK_FULL|etc}, operation={create|extract|scan|verify|info}
Purpose: Track error distribution for debugging and reliability monitoring
```

**Error Rate by Operation** (Gauge):

```
fulpack.errors_rate
Labels: operation={create|extract|scan|verify|info}
Computation: errors_total / operation.total (rolling 5m window)
Purpose: Monitor operation reliability and trigger alerts
Alert threshold: error_rate > 0.05 (5%)
```

### Pathfinder Integration Metrics

**Pathfinder Delegation Count** (Counter):

```
fulpack.pathfinder.delegations_total
Labels: pattern_type={glob|regex}, result={found|not_found|error}
Purpose: Track Pathfinder integration usage and success rates
```

**Archive Glob Match Duration** (Histogram):

```
fulpack.pathfinder.glob_match_seconds
Labels: format={tar|tar.gz|zip|gzip}, entries_scanned={<100|100-1000|1000-10000|>10000}
Buckets: [0.01, 0.05, 0.1, 0.5, 1, 2, 5]
Purpose: Monitor glob pattern matching performance within archives
```

### Resource Utilization Metrics

**Memory Usage During Operation** (Gauge):

```
fulpack.memory.bytes_used
Labels: operation={create|extract|scan}, phase={reading|processing|writing}
Purpose: Track memory consumption for capacity planning
Alert threshold: memory_bytes_used > 1GB (potential memory leak)
```

**Temporary File Usage** (Gauge):

```
fulpack.temp_files.count
Labels: operation={create|extract}
Purpose: Monitor temporary file cleanup and detect leaks
Alert threshold: temp_files.count > 0 after operation completion
```

### Instrumentation Requirements

**ALL implementations MUST**:

1. **Instrument at operation entry/exit**:
   - Record operation start timestamp
   - Record operation end timestamp and compute duration
   - Increment operation counter with status label

2. **Record security events**:
   - Increment security violation counter on detection
   - Log security violations with full context (archive path, entry path, violation type)

3. **Track resource metrics**:
   - Sample memory usage during long-running operations
   - Report bytes processed and entry counts

4. **Use consistent labels**:
   - Format: Use canonical format enum values (`tar`, `tar.gz`, `zip`, `gzip`)
   - Operation: Use canonical operation names (lowercase)
   - Status: Use `success`, `error`, or `skipped` (consistent with other modules)

5. **Follow telemetry module conventions**:
   - Use helper library's telemetry module (don't instrument directly)
   - Respect telemetry sampling configuration
   - Handle telemetry failures gracefully (never fail operation due to telemetry)

### Alerting Recommendations

**Critical Alerts** (page immediately):

- `fulpack.security.violations_total{type="decompression_bomb"}` > 0
- `fulpack.errors_rate` > 0.10 (10% error rate)
- `fulpack.scan.toc_read_seconds` p95 > 5.0 seconds

**Warning Alerts** (notify on-call):

- `fulpack.security.violations_total{type="path_traversal"}` > 10/hour
- `fulpack.errors_rate` > 0.05 (5% error rate)
- `fulpack.scan.toc_read_seconds` p95 > 1.0 seconds
- `fulpack.memory.bytes_used` > 1GB

**Info Alerts** (log only):

- `fulpack.checksum.verifications_total{result="missing"}` > 50% (low coverage)
- `fulpack.archive.compression_ratio` > 100 (suspicious but not necessarily malicious)

### Example: Complete Operation Instrumentation

```python
from pyfulmen import telemetry, fulpack
from pyfulmen.fulpack import ArchiveFormat, FulpackError

def extract_with_telemetry(archive: str, destination: str, options: dict):
    """Example showing complete telemetry instrumentation."""

    # Detect format for labels
    format_str = detect_format(archive)
    labels = {"operation": "extract", "format": format_str}

    # Track operation duration
    start_time = time.time()
    try:
        # Perform extraction
        result = fulpack.extract(archive, destination, options)

        # Record success metrics
        duration = time.time() - start_time
        telemetry.histogram("fulpack.operation.duration_seconds", duration, labels={**labels, "status": "success"})
        telemetry.counter("fulpack.operation.total", labels={**labels, "status": "success"})

        # Record processing metrics
        telemetry.counter("fulpack.entries_processed_total",
                          result.extracted_count,
                          labels={"operation": "extract", "status": "success"})
        telemetry.counter("fulpack.entries_processed_total",
                          result.skipped_count,
                          labels={"operation": "extract", "status": "skipped"})

        # Record checksum metrics if verified
        if result.checksums_verified:
            telemetry.counter("fulpack.checksum.verifications_total",
                              result.checksums_verified,
                              labels={"result": "verified", "algorithm": "sha256"})

        return result

    except FulpackError as e:
        # Record error metrics
        duration = time.time() - start_time
        telemetry.histogram("fulpack.operation.duration_seconds", duration, labels={**labels, "status": "error"})
        telemetry.counter("fulpack.operation.total", labels={**labels, "status": "error"})
        telemetry.counter("fulpack.errors_total", labels={"code": e.code, "operation": "extract"})

        # Record security violations
        if e.code in ["PATH_TRAVERSAL", "ABSOLUTE_PATH", "SYMLINK_ESCAPE", "DECOMPRESSION_BOMB", "CHECKSUM_MISMATCH"]:
            telemetry.counter("fulpack.security.violations_total",
                              labels={"type": e.code.lower(), "operation": "extract"})

        raise
```

### Testing Telemetry

**All implementations MUST test**:

1. **Metric emission**: Verify each metric is emitted with correct labels
2. **Security event tracking**: Verify security violations increment counters
3. **Error code coverage**: Verify all error codes are tracked
4. **Performance tracking**: Verify duration histograms capture operation timing

**Use telemetry module's test utilities**:

```python
# Python example
from pyfulmen.telemetry.testing import MetricsCollector

def test_extract_metrics():
    with MetricsCollector() as metrics:
        fulpack.extract("test.tar.gz", "/tmp/dest")

    assert metrics.histogram("fulpack.operation.duration_seconds") > 0
    assert metrics.counter("fulpack.operation.total", labels={"operation": "extract", "status": "success"}) == 1
```

---

## Related Standards

- [Fulmen Helper Library Standard](../../../architecture/fulmen-helper-library-standard.md)
- [Canonical Façade Principle](../../../architecture/fulmen-helper-library-standard.md#canonical-façade-principle)
- [Pathfinder Module Standard](pathfinder.md) - Required dependency for glob-based archive scanning
- [FulHash Module Standard](fulhash.md) - Required dependency for checksum generation and verification
- [Portable Testing Practices](../../testing/portable-testing-practices.md) - **Required for all implementations** (especially temp file cleanup, sandbox compatibility)
- [Coding Standards](../../coding/README.md) - Language-specific implementation and testing patterns
- [Module Registry](../../../../config/taxonomy/library/platform-modules/v1.0.0/modules.yaml)

---

**Status**: Active
**Tier**: Common
**Version**: 1.0.0
**Last Updated**: 2025-11-15
