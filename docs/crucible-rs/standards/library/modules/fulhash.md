---
title: "FulHash Module Standard"
description: "Shared hashing API for block and streaming checksums across Fulmen helper libraries"
author: "Schema Cartographer"
date: "2025-11-21"
last_updated: "2025-11-21"
status: "approved"
tags:
  [
    "standards",
    "library",
    "fulhash",
    "checksums",
    "xxh3-128",
    "sha256",
    "crc32",
    "crc32c",
    "2025.11.20",
  ]
---

# FulHash Module Standard

## Purpose

Provide a consistent, performant hashing API across all Fulmen helper libraries (gofulmen, pyfulmen, tsfulmen) for:

1. **File integrity verification** (Pathfinder checksums, release artifacts)
2. **Cache key generation** (Docscribe, config hydration)
3. **Change detection** (sync workflows, build systems)
4. **Tamper detection** (optional cryptographic verification)

By standardizing on a curated set of algorithms (speed, security, compatibility) with identical metadata formatting, FulHash ensures cross-language interoperability and prevents implementation drift.

## Core Capabilities

### 1. Block Hashing

One-shot hashing for in-memory data (strings, byte arrays, buffers).

**Go**:

```go
import "github.com/fulmenhq/gofulmen/fulhash"

digest, err := fulhash.Hash(data, fulhash.XXH3_128)
hex := digest.Hex()  // "abc123..."
formatted := digest.String()  // "xxh3-128:abc123..."
```

**Python**:

```python
from pyfulmen.fulhash import hash, Algorithm

digest = hash(data, algorithm=Algorithm.XXH3_128)
print(digest.hex)       # "abc123..."
print(digest.formatted) # "xxh3-128:abc123..."
```

**TypeScript**:

```typescript
import { hash, Algorithm } from "@fulmenhq/tsfulmen/fulhash";

const digest = await hash(data, { algorithm: Algorithm.XXH3_128 });
console.log(digest.hex); // "abc123..."
console.log(digest.formatted); // "xxh3-128:abc123..."
```

### 2. Streaming Hashing

Incremental hashing for large files or streams without loading entire content into memory.

**Go**:

```go
hasher := fulhash.NewStreamHasher(fulhash.XXH3_128)
io.Copy(hasher, file)
digest := hasher.Sum()
```

**Python**:

```python
hasher = fulhash.stream(algorithm=Algorithm.XXH3_128)
for chunk in file:
    hasher.update(chunk)
digest = hasher.digest()
```

**TypeScript**:

```typescript
const hasher = fulhash.createStream({ algorithm: Algorithm.XXH3_128 });
for await (const chunk of stream) {
  hasher.update(chunk);
}
const digest = hasher.digest();
```

### 3. Metadata Formatting

Standardized checksum representation: `<algorithm>:<lowercase-hex>` (see [`checksum-string.schema.json`](../../../schemas/library/fulhash/v1.0.0/checksum-string.schema.json))

**Format Helper**:

```python
checksum = "xxh3-128:a1b2c3d4e5f6"
algo, hex_value = fulhash.parse_checksum(checksum)
```

**Validation**:

- Algorithm prefix MUST be from supported set (`xxh3-128`, `sha256`)
- Hex MUST be lowercase
- Separator MUST be single colon (`:`)

### 4. Convenience Interfaces

#### MultiHash

Calculate multiple digests in a single pass (I/O optimization).

**Signature**: `MultiHash(data/reader, algorithms[]) -> Map<Algorithm, Digest>`

**Use Case**: Calculate `sha256` for security and `crc32` for legacy metadata simultaneously while reading a stream once.

**Go**:

```go
digests, err := fulhash.MultiHashString("content", []Algorithm{XXH3_128, SHA256})
fmt.Println(digests[XXH3_128].Hex)
fmt.Println(digests[SHA256].Hex)
```

#### Verify

Helper to verify data against an expected digest.

**Signature**: `Verify(reader, expected_digest) -> bool`

**Behavior**:

- Returns `true` if hash matches.
- Returns `false` if hash mismatches.
- Raises `IntegrityError` (or language equivalent) on I/O failure or malformed digest.

**Python**:

```python
valid = fulhash.verify(file_path, "sha256:e3b0c44...")
```

## Code Generation

FulHash types and enums are **auto-generated** to ensure SSOT compliance.

- **Source**: `schemas/taxonomy/library/fulhash/algorithms/v1.0.0/algorithms.yaml`
- **Command**: `make codegen-fulhash`
- **Outputs**:
  - Go: `fulhash/types.go`
  - Python: `src/crucible/fulhash/types.py`
  - TypeScript: `src/fulhash/types.ts`

Implementers MUST use these generated types for `Algorithm` enums and `Digest` structs.

## Algorithms

| Algorithm  | Bit Width | Purpose                 | Default | Performance       |
| ---------- | --------- | ----------------------- | ------- | ----------------- |
| `xxh3-128` | 128-bit   | Fast change detection   | ✅ Yes  | ~50-100 GB/s      |
| `crc32`    | 32-bit    | Archive compatibility   | No      | ~1-5 GB/s         |
| `crc32c`   | 32-bit    | Cloud/HW accelerated    | No      | ~10-30 GB/s       |
| `sha256`   | 256-bit   | Cryptographic integrity | No      | ~500 MB/s - 2GB/s |

**Selection Criteria**:

- **xxh3-128**: Default for Pathfinder, Docscribe, cache keys. Non-cryptographic but excellent collision resistance for integrity checks.
- **crc32/crc32c**: Use when interoperability with legacy systems (ZIP, GZIP) or specific cloud APIs (GCS) is required. NOT suitable for collision resistance against malicious inputs.
- **sha256**: Opt-in for tamper detection, release artifact verification, security-sensitive workflows.

**Future Algorithms**: BLAKE3, SHA-512 may be added in future revisions; implementations MUST reject unsupported algorithms with clear error messages.

## Type Definitions

### Digest

Represents a computed hash with metadata.

| Field       | Type   | Description                                                            |
| ----------- | ------ | ---------------------------------------------------------------------- |
| `algorithm` | string | Algorithm identifier (`"xxh3-128"`, `"crc32"`, `"crc32c"`, `"sha256"`) |
| `hex`       | string | Lowercase hexadecimal representation                                   |
| `bytes`     | bytes  | Raw hash bytes (language-specific type)                                |
| `formatted` | string | Prefixed format: `"<algorithm>:<hex>"`                                 |

Canonical schema: [`digest.schema.json`](../../../schemas/library/fulhash/v1.0.0/digest.schema.json)

### StreamHasher

Incremental hasher supporting `update()`, `digest()`, and optional `reset()`.

**Required Methods**:

- `update(data)`: Add chunk to running hash
- `digest()`: Finalize and return Digest
- `reset()`: Clear state for reuse (optional, recommended)

## Error Handling

| Error Condition         | Behavior                                                  |
| ----------------------- | --------------------------------------------------------- |
| Unsupported algorithm   | Raise error with list of supported algorithms             |
| Invalid checksum format | Raise error indicating expected format                    |
| I/O error during stream | Propagate I/O error with context (file path if available) |
| Empty input             | Return valid digest for zero-length input (per algorithm) |

**Error Messages MUST**:

- Include algorithm/checksum string in context
- Suggest valid alternatives for unsupported algorithm
- Reference FulHash documentation for format specification

## Shared Fixtures

Helper libraries MUST validate against shared test fixtures ensuring cross-language parity.

**Fixture Location**: `config/library/fulhash/fixtures.yaml` (synced to lang wrappers)

**Fixture Structure**:

```yaml
version: "1.0.0"
fixtures:
  - name: "empty-input"
    input: ""
    encoding: "utf-8"
    xxh3_128: "xxh3-128:99aa06d3014798d86001c324468d497f"
    sha256: "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

  - name: "hello-world"
    input: "Hello, World!"
    encoding: "utf-8"
    xxh3_128: "xxh3-128:..."
    sha256: "sha256:..."

  - name: "streaming-test"
    description: "Multi-chunk streaming validation"
    chunks: ["Hello, ", "World!"]
    encoding: "utf-8"
    xxh3_128: "xxh3-128:..."
    sha256: "sha256:..."
```

## Performance Requirements

| Workload    | Target                                                      |
| ----------- | ----------------------------------------------------------- |
| Small files | <1ms overhead for files <10KB                               |
| Large files | <10% overhead vs raw I/O for streaming                      |
| Memory      | O(1) memory usage for streaming (constant buffer size)      |
| Concurrency | Thread-safe Digest type, StreamHasher stateful per instance |

**Benchmarking**: Each implementation MUST include benchmarks comparing:

- Block hashing (small/medium/large inputs)
- Streaming hashing (chunked vs single read)
- Memory allocation profiles

## Integration Points

### Pathfinder

Pathfinder uses FulHash for file checksum metadata:

```jsonc
// Pathfinder config
{
  "calculateChecksums": false,
  "checksumAlgorithm": "xxh3-128"
}

// PathResult metadata
{
  "path": "src/main.go",
  "metadata": {
    "size": 12345,
    "checksum": "xxh3-128:abc123...",
    "checksumAlgorithm": "xxh3-128"
  }
}
```

See [Pathfinder Checksum Integration](../extensions/pathfinder.md#checksum-support) for complete specification.

### Docscribe

Optional integrity verification for synced documentation:

```python
doc = docscribe.get_document(id)
digest = fulhash.hash(doc.content, algorithm=Algorithm.XXH3_128)
if digest.formatted != doc.metadata.checksum:
    raise IntegrityError("Document checksum mismatch")
```

### Cache Keys

Consistent cache key generation across tools:

```go
key := fulhash.HashString(configPath + version, fulhash.XXH3_128).Hex()
cache.Get(key)
```

## Implementation Guidance

### Go (gofulmen)

**Dependencies**:

- `github.com/zeebo/xxh3` (pure Go, MIT license)
- `crypto/sha256` (stdlib)

**Streaming**:

```go
type StreamHasher interface {
    io.Writer
    Sum() Digest
    Reset()
}
```

**Thread Safety**: Digest immutable, StreamHasher instance-local state

### Python (pyfulmen)

**Dependencies**:

- `xxhash` package (PyPI, BSD-2-Clause)
- `hashlib` (stdlib)
- `google-crc32c` (Optional but Recommended for CRC32C performance)

**Async Support**: Provide async variants for streaming:

```python
async def hash_file(path: Path, algorithm: Algorithm) -> Digest:
    hasher = fulhash.stream(algorithm)
    async with aiofiles.open(path, 'rb') as f:
        async for chunk in f:
            hasher.update(chunk)
    return hasher.digest()
```

### TypeScript (tsfulmen)

**Dependencies**:

- `xxhash-wasm` (NPM, MIT license) for Node.js/browser
- `crypto` (Node.js stdlib) or `crypto.subtle` (browser) for SHA-256

**Promise-based**:

```typescript
async function hashFile(path: string, algorithm: Algorithm): Promise<Digest> {
  const hasher = fulhash.createStream({ algorithm });
  const stream = fs.createReadStream(path);
  for await (const chunk of stream) {
    hasher.update(chunk);
  }
  return hasher.digest();
}
```

## Testing Requirements

### Unit Tests

- ✅ Block hashing for each algorithm with known fixtures
- ✅ Streaming hashing with single/multi-chunk inputs
- ✅ Metadata formatting and parsing
- ✅ Error conditions (unsupported algorithm, invalid format)
- ✅ Empty input handling

### Integration Tests

- ✅ Cross-language fixture validation (all implementations produce identical output)
- ✅ Large file handling (>100MB without memory issues)
- ✅ Concurrent hasher instances (no shared state corruption)

### Benchmarks

- ✅ Block hashing throughput (MB/s)
- ✅ Streaming overhead vs raw I/O
- ✅ Memory allocation profile

## Versioning & Evolution

**Current Version**: 1.0.0 (2025.10.3)

**API Stability**: Core functions (`hash`, `stream`, `Digest` type) are stable. New algorithms may be added without breaking changes.

**Deprecation Policy**: Algorithm removal requires:

1. Deprecation notice in release notes (minimum 2 releases)
2. Migration guide for affected consumers
3. Fixture retention for legacy validation

## References

- [Module Manifest Entry](../../config/library/v1.0.0/module-manifest.yaml#fulhash)
- [FulHash Fixtures](../../config/library/fulhash/fixtures.yaml)
- [Pathfinder Checksum Integration](.plans/active/2025.10.3/pathfinder-fulhash-integration-brief.md)
- [xxHash Algorithm](https://xxhash.com/)
- [SHA-256 Specification](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf)

## Changelog

- **2025-11-21**: Added CRC32, CRC32C, MultiHash, Verify, and Codegen requirements (v0.2.20)
- **2025-10-23**: Initial draft for 2025.10.3 release
