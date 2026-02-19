---
title: "Fulencode Module Standard (DRAFT)"
description: "Canonical API specification for encoding/decoding operations with security and normalization"
author: "Schema Cartographer"
date: "2025-11-15"
version: "1.0.0"
status: "draft"
tier: "common"
target_path: "docs/standards/library/modules/fulencode.md"
tags: ["encoding", "decoding", "unicode", "normalization", "base64", "utf8"]
---

# Fulencode Module Standard

**Target Documentation Path**: `docs/standards/library/modules/fulencode.md`

## Overview

**Fulencode** is a Common-tier helper library module providing canonical encoding/decoding operations (Base64, Base32, Hex, UTF detection, Unicode normalization, BOM handling) with **mandatory security protections** and configurable error recovery. This specification defines the cross-language API contract that all Fulmen helper libraries must implement.

**Module Tier**: Common
**Version**: 1.0.0
**Added**: Crucible v0.2.12
**Dependencies**:

- fulhash (required) - Checksum generation and verification for large payloads
- telemetry (required) - Metrics instrumentation

**Language-Specific Dependencies** (Common Tier):

- **Python**: Pure stdlib (`base64`, `binascii`, `unicodedata`, `codecs`) - Zero PyPI packages
- **TypeScript**: Pure stdlib (`Buffer`, `String.prototype.normalize()`) - Zero npm packages
- **Go**: Stdlib + `golang.org/x/text/unicode/norm` - Extended stdlib for Unicode normalization (already vendored by gofulmen)
  - Note: Go stdlib lacks native Unicode normalization; `x/text` is de facto standard and not counted as "external dependency"

## Architecture Principles

### Canonical Façade Principle

Per the [Fulmen Helper Library Standard](../../../architecture/fulmen-helper-library-standard.md#canonical-façade-principle), fulencode provides **canonical façades** wrapping standard library and third-party functionality to ensure:

1. **Cross-Language Interface Consistency** - Same operations, same error envelopes, same behavior
2. **Security by Default** - Buffer overflow protection, invalid UTF-8 handling, normalization attack prevention
3. **Configurable Error Recovery** - Support both strict validation (for round-trip integrity) and correction modes (for resilient parsing)
4. **Taxonomy-Driven Design** - Encoding families, normalization profiles, and detection confidence levels defined in SSOT

### Why Common Tier?

**Common tier assignment rationale**:

- **Universal need**: Most applications need basic encoding (Base64, Hex), UTF validation, and normalization
- **Minimal external dependencies**:
  - Python/TypeScript: Pure stdlib (zero package manager dependencies)
  - Go: Stdlib + `golang.org/x/text` (extended stdlib, widely vendored for Unicode normalization)
- **Security requirement**: Consistent buffer limits, UTF validation, and error handling across languages
- **Integration enabler**: Required by fulpack (file name normalization), nimbus (cloud payload encoding), pathfinder (manifest encoding detection)

**This is NOT a specialized module** despite wrapping stdlib—the Canonical Façade Principle mandates façades for universal capabilities regardless of implementation strategy.

**Dependency Philosophy** (per gofulmen feedback):

- "Zero external dependencies" = zero package manager deps (PyPI/npm)
- Go's `golang.org/x/text` is acceptable because:
  - Part of Go extended standard library ecosystem
  - De facto standard for Unicode normalization (stdlib gap)
  - Already vendored by gofulmen and other Go projects
  - Considered baseline requirement, not "external dependency"

## Generated Types Integration

**All helper libraries MUST use generated types from Crucible** to ensure cross-language consistency and prevent drift.

### Import Patterns (per EA Steward feedback)

**TypeScript**:

```typescript
// Import generated types from crucible package
import type {
  EncodingFormat,
  NormalizationProfile,
  ConfidenceLevel,
} from "@fulmenhq/crucible/fulencode/types";

// Use in function signatures
export function encode(
  data: Uint8Array,
  format: EncodingFormat,
  options?: EncodeOptions,
): string {
  // Implementation validates format at runtime
  if (!isValidEncodingFormat(format)) {
    throw new FulencodeError(
      "INVALID_FORMAT",
      `Unknown encoding format: ${format}`,
    );
  }
  // ...
}
```

**Python**:

```python
# Import generated enums from crucible
from crucible.fulencode import (
    EncodingFormat,
    NormalizationProfile,
    ConfidenceLevel,
)

# Use in type annotations and runtime validation
def encode(
    data: bytes,
    format: EncodingFormat,
    options: EncodeOptions | None = None
) -> str:
    """Encode binary data to text using specified format."""
    # Enum validation happens automatically via type system
    # Runtime validation for string inputs:
    try:
        format_enum = EncodingFormat(format)
    except ValueError:
        raise FulencodeError('INVALID_FORMAT', f'Unknown encoding format: {format}')
```

**Go**:

```go
// Import generated types from crucible
import "github.com/fulmenhq/crucible/fulencode"

// Use with validators
func Encode(data []byte, format fulencode.EncodingFormat, options *EncodeOptions) (string, error) {
    // Validate format using generated validator
    if err := fulencode.ValidateEncodingFormat(format); err != nil {
        return "", NewFulencodeError("INVALID_FORMAT", err.Error())
    }
    // ...
}
```

### Type Generation System

**Generator**: `scripts/codegen/generate-fulencode-types.ts`
**Verifier**: `scripts/codegen/verify-fulencode-types.ts`
**Makefile**: `make codegen-fulencode`, `make verify-codegen`

**Generated files**:

- Go: `fulencode/types.go` - Type aliases with const enums and validators
- Python: `lang/python/src/crucible/fulencode/enums.py` - str-based Enum classes
- TypeScript: `lang/typescript/src/fulencode/types.ts` - String literal union types

**DO NOT** hand-write these types in helper libraries. Import from Crucible to ensure consistency.

## Taxonomy-Driven Design

### Encoding Families Taxonomy

**Location**: `schemas/taxonomy/library/fulencode/encoding-families/v1.0.0/families.yaml`

**Supported encodings** (v1.0.0):

**Binary-to-Text Encodings (Common tier)**:

- `base64` - Standard Base64 (RFC 4648 §4, uses `+/` characters)
- `base64url` - URL-safe Base64 (RFC 4648 §5, uses `-_` characters)
- `base64_raw` - Base64 without padding (no `=` padding chars)
- `base32` - Base32 standard (RFC 4648 §6)
- `base32hex` - Base32 with hex alphabet (RFC 4648 §7)
- `hex` - Hexadecimal encoding (Base16, RFC 4648 §8)

**Character Encodings (Common tier - detection and transcoding)**:

- `utf-8` - Unicode UTF-8 (RFC 3629)
- `utf-16le` - Unicode UTF-16 Little Endian
- `utf-16be` - Unicode UTF-16 Big Endian
- `iso-8859-1` - ISO 8859-1 (Latin-1) - canonical name, `latin-1` as alias
- `cp1252` - Windows-1252 (Western European, superset of ISO-8859-1)
- `ascii` - ASCII (7-bit, printable subset of UTF-8)

**Specialized encodings** (deferred to extension modules):

- `base58` - Base58 (Bitcoin, IPFS variants) - requires specialized module
- `base85` - Base85 (RFC 1924, Adobe Ascii85) - requires specialized module
- `multibase` - Self-describing base encodings (IPFS) - detection supported, encoding deferred
- `utf-32le`, `utf-32be` - UTF-32 variants - deferred to specialized
- Legacy code pages (Shift-JIS, EBCDIC, GB2312, Big5) - deferred to specialized

**Format features** (Common tier encodings):

```yaml
base64:
  type: binary_to_text
  alphabet_size: 64
  padding: true
  padding_char: "="
  characters: "A-Za-z0-9+/"
  use_case: "General purpose binary encoding, email attachments"

base64url:
  type: binary_to_text
  alphabet_size: 64
  padding: optional # Can be omitted for URL safety
  padding_char: "="
  characters: "A-Za-z0-9-_"
  use_case: "URL-safe tokens, JWT, query parameters"

hex:
  type: binary_to_text
  alphabet_size: 16
  case_variants: [lowercase, uppercase]
  padding: false
  characters: "0-9a-f" # or "0-9A-F"
  use_case: "Checksums, debug output, color codes"

utf-8:
  type: character_encoding
  variable_width: true # 1-4 bytes per codepoint
  bom_required: false
  bom_bytes: [0xEF, 0xBB, 0xBF]
  max_codepoint: 0x10FFFF
  use_case: "Universal text encoding, web, APIs"

utf-16le:
  type: character_encoding
  variable_width: true # 2 or 4 bytes per codepoint
  bom_required: false # But strongly recommended
  bom_bytes: [0xFF, 0xFE]
  max_codepoint: 0x10FFFF
  use_case: "Windows internals, Java string storage"
```

Libraries generate enums from this taxonomy:

```python
# Generated in pyfulmen
from enum import Enum

class EncodingFormat(Enum):
    # Binary-to-text
    BASE64 = "base64"
    BASE64URL = "base64url"
    BASE64_RAW = "base64_raw"
    BASE32 = "base32"
    BASE32HEX = "base32hex"
    HEX = "hex"

    # Character encodings
    UTF8 = "utf-8"
    UTF16LE = "utf-16le"
    UTF16BE = "utf-16be"
    ISO_8859_1 = "iso-8859-1"
    CP1252 = "cp1252"
    ASCII = "ascii"
```

### Normalization Profiles Taxonomy

**Location**: `schemas/taxonomy/library/fulencode/normalization-profiles/v1.0.0/profiles.yaml`

**Canonical normalization forms** (Unicode Standard Annex #15):

```yaml
profiles:
  nfc:
    name: "Canonical Composition (NFC)"
    description: "Compose combining characters (é = U+00E9)"
    use_case: "Text storage, comparison, most general purpose"
    semantic_preserving: true

  nfd:
    name: "Canonical Decomposition (NFD)"
    description: "Decompose to base + combining marks (é = U+0065 U+0301)"
    use_case: "Text processing, searching, sorting"
    semantic_preserving: true

  nfkc:
    name: "Compatibility Composition (NFKC)"
    description: "Decompose compatibility chars then compose (ﬁ → fi)"
    use_case: "Identifiers, case-insensitive search"
    semantic_preserving: false # ⚠️ WARNING: Changes meaning (ligatures, etc.)

  nfkd:
    name: "Compatibility Decomposition (NFKD)"
    description: "Full decomposition including compatibility chars"
    use_case: "Advanced text analysis, full-text search"
    semantic_preserving: false

  # Custom profiles for specific use cases
  safe_identifiers:
    base_form: nfkc
    strip_accents: false
    case_fold: false
    allowed_categories: ["Lu", "Ll", "Lt", "Lm", "Lo", "Nd", "Pc"] # Letters, numbers, connectors
    reject_zero_width: true
    use_case: "Programming identifiers, usernames"

  search_optimized:
    base_form: nfkd
    strip_accents: true
    case_fold: true
    remove_punctuation: true
    compress_whitespace: true
    use_case: "Full-text search indexing"

  filename_safe:
    base_form: nfc
    strip_accents: false
    case_fold: false
    reject_control_chars: true
    reject_path_separators: true
    normalize_spaces: true
    use_case: "Cross-platform file names"

  text_safe:
    base_form: nfc
    reject_control_chars: true
    reject_zero_width: true
    reject_bidi_controls: true
    compress_whitespace: optional
    use_case: "Log-safe and UI-safe text display"

See `docs/standards/library/modules/fulencode-text-safe.md` for detailed rationale and algorithm notes.

  legacy_compatible:
    base_form: nfc
    strip_nonascii: false
    transliterate: optional
    use_case: "Legacy system integration"
```

#### Custom Profile Transformation Rules (per pyfulmen feedback)

**Explicit transformation algorithms for consistent cross-language implementation:**

**safe_identifiers** (base: NFKC):

```python
# Step-by-step transformation
def safe_identifiers(text: str, options: dict) -> str:
    # 1. Apply NFKC normalization
    text = unicodedata.normalize('NFKC', text)

    # 2. Reject zero-width characters (security)
    zero_width_chars = ['\u200B', '\u200C', '\u200D', '\uFEFF']
    if any(c in text for c in zero_width_chars):
        raise FulencodeError('ZERO_WIDTH_REJECTED', 'Zero-width characters not allowed in identifiers')

    # 3. Check combining marks limit (prevent stacking attacks)
    max_combining = options.get('max_combining_marks', 3)
    for i, char in enumerate(text):
        if unicodedata.category(char) in ('Mn', 'Mc', 'Me'):  # Combining marks
            # Count consecutive combining marks
            combining_count = 1
            j = i + 1
            while j < len(text) and unicodedata.category(text[j]) in ('Mn', 'Mc', 'Me'):
                combining_count += 1
                j += 1
            if combining_count > max_combining:
                raise FulencodeError('EXCESSIVE_COMBINING_MARKS',
                                     f'More than {max_combining} combining marks at position {i}')

    # 4. Validate character categories (only allowed Unicode categories)
    allowed_categories = ['Lu', 'Ll', 'Lt', 'Lm', 'Lo', 'Nd', 'Pc']  # Letters, numbers, connectors
    for i, char in enumerate(text):
        category = unicodedata.category(char)
        if category not in allowed_categories:
            raise FulencodeError('INVALID_CHARACTER',
                                 f'Character {char!r} (category {category}) not allowed at position {i}')

    return text
```

**search_optimized** (base: NFKD):

```python
def search_optimized(text: str, options: dict) -> str:
    # 1. Apply NFKD normalization (full decomposition)
    text = unicodedata.normalize('NFKD', text)

    # 2. Strip accents (optional, enabled by default)
    if options.get('strip_accents', True):
        # Remove combining marks (accents/diacritics)
        text = ''.join(c for c in text if unicodedata.category(c) != 'Mn')

    # 3. Case folding (lowercase)
    if options.get('case_fold', True):
        text = text.casefold()  # Unicode-aware case folding

    # 4. Remove punctuation (optional)
    if options.get('remove_punctuation', True):
        text = ''.join(c for c in text if unicodedata.category(c) not in ('P', 'S'))

    # 5. Compress whitespace
    if options.get('compress_whitespace', True):
        text = ' '.join(text.split())  # Collapse multiple spaces to single

    return text
```

**filename_safe** (base: NFC):

```python
def filename_safe(text: str, options: dict) -> str:
    # 1. Apply NFC normalization (for macOS/Windows compatibility)
    text = unicodedata.normalize('NFC', text)

    # 2. Reject control characters (0x00-0x1F, 0x7F-0x9F)
    if options.get('reject_control_chars', True):
        for i, char in enumerate(text):
            if ord(char) < 0x20 or (0x7F <= ord(char) < 0xA0):
                raise FulencodeError('CONTROL_CHARACTER',
                                     f'Control character U+{ord(char):04X} at position {i}')

    # 3. Reject path separators (security)
    if options.get('reject_path_separators', True):
        path_separators = ['/', '\\', '\x00']
        for sep in path_separators:
            if sep in text:
                raise FulencodeError('PATH_SEPARATOR',
                                     f'Path separator {sep!r} not allowed in file names')

    # 4. Normalize spaces (collapse multiple spaces, trim)
    if options.get('normalize_spaces', True):
        text = ' '.join(text.split())  # Collapse and trim

    # 5. Optional: reject reserved names (Windows)
    if options.get('reject_reserved_windows', False):
        reserved = ['CON', 'PRN', 'AUX', 'NUL', 'COM1', 'COM2', 'LPT1', 'LPT2']
        if text.upper() in reserved or any(text.upper().startswith(r + '.') for r in reserved):
            raise FulencodeError('RESERVED_NAME', f'Windows reserved name: {text}')

    return text
```

**legacy_compatible** (base: NFC):

```python
def legacy_compatible(text: str, options: dict) -> str:
    # 1. Apply NFC normalization
    text = unicodedata.normalize('NFC', text)

    # 2. Optional transliteration (lossy)
    if options.get('transliterate', False):
        # Simple ASCII transliteration (language-agnostic)
        # This is lossy and should be avoided if possible
        transliterations = {
            'é': 'e', 'è': 'e', 'ê': 'e', 'ë': 'e',
            'á': 'a', 'à': 'a', 'â': 'a', 'ä': 'a',
            'ñ': 'n', 'ç': 'c', 'ü': 'u', 'ö': 'o',
            # Add more as needed
        }
        text = ''.join(transliterations.get(c, c) for c in text)

    # 3. Optional: reject non-ASCII (for pure ASCII output)
    if options.get('strip_nonascii', False):
        text = ''.join(c if ord(c) < 128 else '' for c in text)

    return text
```

**Implementation notes**:

- All profiles MUST raise `FulencodeError` on violations (don't silently skip)
- Use Unicode category checks (`unicodedata.category()`) for consistent behavior
- Document which transformations are lossy (strip_accents, transliterate)
- Track semantic changes in `NormalizationResult.semantic_changes[]`

### Detection Confidence Taxonomy

**Location**: `schemas/taxonomy/library/fulencode/detection-confidence/v1.0.0/levels.yaml`

```yaml
confidence_levels:
  high:
    threshold: 0.90 # ≥90%
    description: "Strong BOM present or unambiguous byte patterns"
    examples:
      - "UTF-8 BOM detected"
      - "All bytes valid UTF-8, no NULL bytes, common Unicode ranges"
      - "UTF-16 BOM detected"
      - "Even-offset NULL bytes consistent with UTF-16LE"

  medium:
    threshold: 0.50 # 50-89%
    description: "Heuristic match with some uncertainty"
    examples:
      - "Valid UTF-8 but short sample size"
      - "Byte frequency suggests CP1252 but overlaps with ISO-8859-1"
      - "No BOM but NULL byte pattern suggests UTF-16"

  low:
    threshold: 0.00 # <50%
    description: "Multiple possibilities, defaulting to UTF-8"
    examples:
      - "All bytes valid ASCII (subset of UTF-8, ISO-8859-1, CP1252)"
      - "Binary data, no text patterns detected"
      - "Conflicting signals from different heuristics"
```

---

## Canonical API Specification

### 1. encode() - Binary-to-Text Encoding

**Signature** (TypeScript pseudocode):

```typescript
encode(
  data: Uint8Array | string,
  format: EncodingFormat,
  options?: EncodeOptions
): EncodingResult
```

**Parameters**:

- `data`: Raw bytes (Uint8Array) or string (for text encodings)
- `format`: Target encoding format (enum)
- `options`: Optional encoding options

**EncodeOptions** (from `schemas/library/fulencode/v1.0.0/encode-options.schema.json`):

```typescript
{
  // Binary-to-text encoding options
  padding?: boolean,              // Base64 padding (default: true)
  case?: "upper" | "lower",       // Hex case (default: "lower")
  line_length?: number | null,    // Wrap at N chars (default: null = no wrapping)
  line_ending?: "\n" | "\r\n",    // Line ending style (default: "\n")

  // Security and validation
  max_encoded_size?: number,      // Max output size in bytes (default: 500MB)

  // Integrity options (via Fulhash integration)
  compute_checksum?: string,      // "xxh3-128" | "sha256" | "sha512" (default: null)
  embed_checksum?: boolean,       // Append checksum in footer (default: false)

  // Error handling
  on_error?: "strict" | "replace" | "ignore",  // For text encoding (default: "strict")
}
```

**Returns**: `EncodingResult`

```typescript
interface EncodingResult {
  data: string; // Encoded output
  format: string; // Format used
  input_size: number; // Input bytes
  output_size: number; // Output bytes
  checksum?: string; // Optional checksum (if computed)
  checksum_algorithm?: string; // Algorithm used
  warnings: string[]; // Non-fatal warnings
}
```

**Security**:

- Enforces `max_encoded_size` limit (default: 500MB)
- Monitors memory allocation during encoding
- Rejects invalid input for target format

**Example** (Python):

```python
from pyfulmen import fulencode
from pyfulmen.fulencode import EncodingFormat

# Basic Base64 encoding
result = fulencode.encode(
    data=b"Hello, World!",
    format=EncodingFormat.BASE64,
    options={"padding": True}
)
print(result.data)  # "SGVsbG8sIFdvcmxkIQ=="

# URL-safe Base64 with checksum (for JWT, API tokens)
result_urlsafe = fulencode.encode(
    data=token_bytes,
    format=EncodingFormat.BASE64URL,
    options={
        "padding": False,              # Common for URLs
        "compute_checksum": "xxh3-128",
        "embed_checksum": True
    }
)

# Hex encoding with uppercase (for display)
hex_result = fulencode.encode(
    data=binary_data,
    format=EncodingFormat.HEX,
    options={"case": "upper"}
)
print(hex_result.data)  # "48656C6C6F"
```

**Example** (Go):

```go
import "github.com/fulmenhq/gofulmen/fulencode"

// Base64 with line wrapping (MIME style)
result, err := fulencode.Encode(
    data,
    fulencode.BASE64,
    &fulencode.EncodeOptions{
        Padding:    true,
        LineLength: 76,      // MIME line length
        LineEnding: "\r\n",
    },
)
if err != nil {
    return err
}
fmt.Println(result.Data)
```

**Example** (TypeScript):

```typescript
import { fulencode, EncodingFormat } from "@fulmenhq/tsfulmen/fulencode";

// Base64URL for URL parameters
const result = await fulencode.encode(
  new Uint8Array(data),
  EncodingFormat.BASE64URL,
  {
    padding: false,
    computeChecksum: "sha256",
  },
);
console.log(`Encoded: ${result.data}`);
```

---

### 2. decode() - Binary-from-Text Decoding

**Signature**:

```typescript
decode(
  data: string | Uint8Array,
  format: EncodingFormat,
  options?: DecodeOptions
): DecodingResult
```

**Parameters**:

- `data`: Encoded string or bytes
- `format`: Source encoding format (enum)
- `options`: Optional decoding options

**DecodeOptions** (from `schemas/library/fulencode/v1.0.0/decode-options.schema.json`):

```typescript
{
  // Validation options
  verify_checksum?: boolean,      // Verify embedded checksum (default: true if present)
  compute_checksum?: string,      // Compute checksum of decoded output (default: null)

  // Security limits
  max_decoded_size?: number,      // Max output size in bytes (default: 100MB)

  // Error handling (CRITICAL for round-trip integrity)
  on_error?: "strict" | "replace" | "ignore" | "fallback",  // (default: "strict")
  fallback_formats?: string[],    // Attempt these if primary fails (default: [])

  // Binary-to-text options
  ignore_whitespace?: boolean,    // Ignore \n, \r, \t, space (default: true for Base64/Hex)
  validate_padding?: boolean,     // Require correct padding (default: true for Base64)
}
```

**Returns**: `DecodingResult`

```typescript
interface DecodingResult {
  data: Uint8Array; // Decoded bytes
  format: string; // Format detected/used
  input_size: number; // Input bytes
  output_size: number; // Output bytes
  checksum?: string; // Verified or computed checksum
  checksum_verified?: boolean; // True if checksum was verified
  checksum_algorithm?: string; // Algorithm used
  warnings: string[]; // Non-fatal warnings (e.g., missing padding)
  corrections_applied: number; // Count of error corrections (0 in strict mode)
}
```

**Security** (MANDATORY for all implementations):

- **Buffer overflow protection**: Enforce `max_decoded_size` limit (default: 100MB)
- **Encoding bomb protection**: Monitor expansion ratio (e.g., 1KB Base64 → 1GB decoded)
- **Checksum verification**: Verify checksums if present (unless disabled)
- **Invalid input detection**: Reject malformed encodings in strict mode

**Error handling modes** (critical for round-trip integrity):

```yaml
strict (default):
  - Raise error on any invalid byte
  - Best for: Data integrity, round-trip serialization, cryptographic operations
  - UTF-8: Reject overlong sequences, surrogates, invalid continuations
  - UTF-16: Reject unpaired surrogates, invalid byte order
  - Base64: Reject invalid characters, incorrect padding

replace:
  - Replace invalid sequences with U+FFFD (UTF) or skip (binary encodings)
  - Best for: Resilient parsing, user input, legacy data migration
  - UTF-8: Invalid bytes → U+FFFD
  - UTF-16: Invalid surrogates → U+FFFD
  - Base64: Skip invalid characters (whitespace already ignored)

ignore:
  - Skip invalid sequences entirely
  - Best for: Lossy parsing where some data loss is acceptable
  - WARNING: Output may be corrupted

fallback:
  - Attempt alternative encodings from fallback_formats list
  - Best for: Auto-detection scenarios where format is uncertain
  - Try each encoding until one succeeds
```

**⚠️ CRITICAL**: Applications requiring **perfect round-trip integrity** (e.g., cryptographic signatures, legal documents, data synchronization) MUST use `on_error: "strict"`. Correction modes permanently alter data and prevent exact reconstruction.

**Example** (Python):

```python
from pyfulmen import fulencode
from pyfulmen.fulencode import EncodingFormat

# Strict decoding (default) - errors on invalid input
try:
    result = fulencode.decode(
        data="SGVsbG8sIFdvcmxkIQ==",
        format=EncodingFormat.BASE64,
        options={"on_error": "strict"}  # Explicit for clarity
    )
    print(result.data)  # b"Hello, World!"
except fulencode.FulencodeError as e:
    if e.code == "INVALID_ENCODING":
        print(f"Invalid Base64: {e.message}")

# Resilient decoding with correction (for user input)
result_resilient = fulencode.decode(
    data=user_provided_text,
    format=EncodingFormat.UTF8,
    options={
        "on_error": "replace",         # Replace invalid UTF-8 with U+FFFD
        "fallback_formats": ["cp1252", "iso-8859-1"]  # Try these if UTF-8 fails
    }
)
if result_resilient.corrections_applied > 0:
    print(f"⚠️ Applied {result_resilient.corrections_applied} corrections - data modified")

# Verify checksum (if embedded)
result_checked = fulencode.decode(
    data=base64_with_checksum,
    format=EncodingFormat.BASE64,
    options={
        "verify_checksum": True,
        "compute_checksum": "sha256"
    }
)
if result_checked.checksum_verified:
    print(f"✓ Checksum verified: {result_checked.checksum}")
```

**Example** (Go with UTF-16 error handling):

```go
import "github.com/fulmenhq/gofulmen/fulencode"

// Strict UTF-16LE decoding (cryptographic use case)
result, err := fulencode.Decode(
    utf16LEBytes,
    fulencode.UTF16LE,
    &fulencode.DecodeOptions{
        OnError: fulencode.Strict,  // Error on unpaired surrogates
    },
)
if err != nil {
    // Data is corrupted - do not proceed
    return fmt.Errorf("UTF-16 integrity check failed: %w", err)
}

// Resilient UTF-16BE decoding (legacy data migration)
resultResilient, err := fulencode.Decode(
    utf16BEBytes,
    fulencode.UTF16BE,
    &fulencode.DecodeOptions{
        OnError: fulencode.Replace,  // Replace invalid surrogates with U+FFFD
    },
)
if resultResilient.CorrectionsApplied > 0 {
    log.Printf("⚠️ Applied %d corrections to UTF-16 data", resultResilient.CorrectionsApplied)
}
```

---

### 3. detect() - Encoding Detection

**Signature**:

```typescript
detect(
  data: Uint8Array,
  options?: DetectOptions
): DetectionResult
```

**Parameters**:

- `data`: Byte sequence to analyze (minimum 32 bytes recommended for accuracy)
- `options`: Optional detection options

**DetectOptions** (from `schemas/library/fulencode/v1.0.0/detect-options.schema.json`):

```typescript
{
  // Detection scope
  candidate_encodings?: string[],  // Limit to these encodings (default: all Common tier)
  max_sample_size?: number,        // Analyze first N bytes (default: 8192)

  // Confidence thresholds
  min_confidence?: number,         // Reject results below threshold (default: 0.5 = medium)

  // BOM handling
  check_bom?: boolean,             // Check for BOM first (default: true)

  // Multibase support
  recognize_multibase?: boolean,   // Detect multibase prefix (default: false)
}
```

**Returns**: `DetectionResult`

```typescript
interface DetectionResult {
  encoding: string; // Detected encoding (canonical name)
  confidence: number; // Confidence score (0.0 - 1.0)
  confidence_level: string; // "high" | "medium" | "low"
  bom_detected?: boolean; // True if BOM present
  bom_bytes?: number[]; // BOM byte sequence if detected
  multibase_prefix?: string; // Multibase prefix if recognized (e.g., "f" for hex)
  candidates: DetectionCandidate[]; // Alternative possibilities
  sample_size: number; // Bytes analyzed
  warnings: string[]; // Detection warnings
}

interface DetectionCandidate {
  encoding: string;
  confidence: number;
  reason: string; // Why this is a candidate
}
```

**Detection algorithm** (in order of precedence):

1. **BOM detection** (100% confidence if present)
   - UTF-8: `0xEF 0xBB 0xBF`
   - UTF-16LE: `0xFF 0xFE`
   - UTF-16BE: `0xFE 0xFF`
   - UTF-32LE: `0xFF 0xFE 0x00 0x00`
   - UTF-32BE: `0x00 0x00 0xFE 0xFF`

2. **Multibase prefix detection** (if enabled)
   - Single-character prefix indicates encoding (IPFS standard)

3. **NULL byte pattern analysis** (for UTF-16 detection)
   - Even-offset NULLs → UTF-16LE (high confidence if consistent)
   - Odd-offset NULLs → UTF-16BE (high confidence if consistent)

4. **UTF-8 validation** (high confidence if all valid)
   - All bytes form valid UTF-8 sequences
   - No overlong encodings
   - No surrogate codepoints
   - Common Unicode ranges present (if sample large enough)

5. **Statistical analysis** (medium confidence)
   - Byte frequency distribution
   - Common byte patterns for CP1252 vs ISO-8859-1
   - Heuristics for legacy encodings

6. **ASCII subset** (low confidence - ambiguous)
   - All bytes < 0x80 → could be UTF-8, ASCII, ISO-8859-1, CP1252
   - Default to UTF-8 with low confidence

#### Minimum Detection Algorithm for Common Tier (per pyfulmen feedback)

**Clarification**: The Common tier detection algorithm is deliberately minimal to avoid external dependencies.

**Common tier MUST implement** (Phase 1):

1. **BOM detection** (steps 1-2 above)
   - Check first 2-4 bytes for BOM signatures
   - Return HIGH confidence with detected encoding
2. **UTF-8 validation** (step 4 above)
   - Validate all bytes form legal UTF-8 sequences
   - Check for overlong encodings (security)
   - Check for surrogate codepoints (U+D800-U+DFFF)
   - Return HIGH confidence if all bytes valid UTF-8
3. **NULL byte pattern** (step 3 above)
   - Scan for regular NULL byte patterns indicating UTF-16
   - Return HIGH confidence if consistent pattern found
4. **ASCII fallback** (step 6 above)
   - If all bytes < 0x80, return "utf-8" with LOW confidence
   - Note: ASCII is subset of UTF-8, ISO-8859-1, CP1252

**Common tier MUST NOT implement** (Specialized tier only):

- **Statistical byte frequency analysis** (step 5)
  - Requires large lookup tables or ML models
  - Deferred to `fulencode-detect-pro` module
  - Distinguishes CP1252 smart quotes from ISO-8859-1
  - Detects CJK encodings (Shift-JIS, GB2312, etc.)

**Implementation pseudocode** (Common tier):

```python
def detect_common_tier(data: bytes) -> DetectionResult:
    # Step 1: Check BOM
    bom_result = check_bom(data)
    if bom_result.detected:
        return DetectionResult(
            encoding=bom_result.encoding,
            confidence=1.0,
            confidence_level="high",
            bom_detected=True
        )

    # Step 2: Try UTF-8 validation
    if validate_utf8(data):
        return DetectionResult(
            encoding="utf-8",
            confidence=0.95,  # High but not 100% (no BOM)
            confidence_level="high"
        )

    # Step 3: Check for UTF-16 NULL patterns
    utf16_result = check_null_patterns(data)
    if utf16_result.confidence > 0.8:
        return utf16_result

    # Step 4: ASCII fallback (ambiguous)
    if all(b < 0x80 for b in data):
        return DetectionResult(
            encoding="utf-8",  # Default to UTF-8
            confidence=0.4,
            confidence_level="low",
            warnings=["Pure ASCII - could be UTF-8, ISO-8859-1, or CP1252"]
        )

    # Step 5: Binary or unknown
    return DetectionResult(
        encoding="unknown",
        confidence=0.0,
        confidence_level="low",
        warnings=["Invalid UTF-8, no BOM, no recognizable pattern"]
    )
```

**Specialized tier additions** (`fulencode-detect-pro`):

- Byte frequency histograms
- N-gram analysis
- Language models (chardet, ICU, cchardet)
- CJK encoding detection
- Latin-1/CP1252 disambiguation

**Example** (Python):

```python
from pyfulmen import fulencode

# Detect encoding with BOM check
with open("mystery_file.txt", "rb") as f:
    data = f.read()

result = fulencode.detect(
    data,
    options={
        "check_bom": True,
        "min_confidence": 0.5  # Reject if < 50% confident
    }
)

print(f"Detected: {result.encoding} (confidence: {result.confidence:.2%})")
if result.bom_detected:
    print(f"  BOM found: {result.bom_bytes}")

# Show alternative candidates
if result.confidence_level == "medium" or result.confidence_level == "low":
    print("Alternative possibilities:")
    for candidate in result.candidates[:3]:
        print(f"  - {candidate.encoding}: {candidate.confidence:.2%} ({candidate.reason})")

# Decode using detected encoding with fallback
decoded = fulencode.decode(
    data,
    format=result.encoding,
    options={
        "on_error": "fallback",
        "fallback_formats": [c.encoding for c in result.candidates]
    }
)
```

**Example** (TypeScript with multibase):

```typescript
import { fulencode } from "@fulmenhq/tsfulmen/fulencode";

// Detect with multibase support (for IPFS data)
const result = await fulencode.detect(data, {
  recognizeMultibase: true,
});

if (result.multibasePrefix) {
  console.log(
    `Multibase format: ${result.multibasePrefix} (${result.encoding})`,
  );
  // Specialized module needed for encoding
  console.warn("⚠️ Multibase encoding requires fulencode-multibase extension");
}
```

---

### 4. normalize() - Unicode Normalization

**Signature**:

```typescript
normalize(
  text: string,
  profile: string | NormalizationProfile,
  options?: NormalizeOptions
): NormalizationResult
```

**Parameters**:

- `text`: Unicode string to normalize
- `profile`: Normalization profile name or custom profile object
- `options`: Optional normalization options

**NormalizeOptions** (from `schemas/library/fulencode/v1.0.0/normalize-options.schema.json`):

```typescript
{
  // Security options
  warn_semantic_change?: boolean,   // Warn if normalization changes meaning (default: true)
  reject_zero_width?: boolean,      // Reject zero-width chars (security) (default: false)
  max_combining_marks?: number,     // Max combining marks per base char (default: 10)

  // Custom transformations (for custom profiles)
  strip_accents?: boolean,          // Remove diacritics (default: false)
  case_fold?: boolean,              // Case-insensitive fold (default: false)
  remove_punctuation?: boolean,     // Remove punctuation (default: false)
  compress_whitespace?: boolean,    // Collapse multiple spaces (default: false)
}
```

**Returns**: `NormalizationResult`

```typescript
interface NormalizationResult {
  text: string; // Normalized output
  profile: string; // Profile used
  input_length: number; // Input codepoints
  output_length: number; // Output codepoints
  transformations_applied: string[]; // List of transformations
  semantic_changes: SemanticChange[]; // Semantic-altering changes (if any)
  warnings: string[]; // Normalization warnings
}

interface SemanticChange {
  position: number; // Codepoint offset
  original: string; // Original character
  normalized: string; // Normalized character
  reason: string; // Why this is semantic-changing
}
```

**Security considerations**:

- **NFKC/NFKD warnings**: These forms change semantic meaning (e.g., ligatures `ﬁ` → `fi`, superscripts `²` → `2`)
- **Combining mark limits**: Reject deeply nested marks (quadratic blowup attack)
- **Zero-width characters**: Can hide malicious content (U+200B, U+200C, U+200D, U+FEFF)

**Example** (Python):

```python
from pyfulmen import fulencode

# Standard NFC normalization (safe, semantic-preserving)
result = fulencode.normalize(
    text="café",  # é as combining chars: e + ´
    profile="nfc",
    options={"warn_semantic_change": True}
)
print(result.text)  # café (é as single codepoint U+00E9)

# NFKC with semantic change warnings (for identifiers)
result_nfkc = fulencode.normalize(
    text="Café ﬁle №2",  # Contains ligature ﬁ and superscript
    profile="nfkc",
    options={"warn_semantic_change": True}
)
print(result_nfkc.text)  # "Café file No2" (ligature decomposed, № → No)
if result_nfkc.semantic_changes:
    for change in result_nfkc.semantic_changes:
        print(f"⚠️ '{change.original}' → '{change.normalized}' at position {change.position}: {change.reason}")

# Custom profile for search (aggressive normalization)
result_search = fulencode.normalize(
    text="Café, naïve résumé!",
    profile="search_optimized",
    options={
        "strip_accents": True,
        "case_fold": True,
        "remove_punctuation": True,
        "compress_whitespace": True,
    }
)
print(result_search.text)  # "cafe naive resume" (search-friendly)
```

**Example** (Go for identifiers):

```go
import "github.com/fulmenhq/gofulmen/fulencode"

// Safe identifier normalization (rejects dangerous chars)
result, err := fulencode.Normalize(
    "user_name_123",
    "safe_identifiers",
    &fulencode.NormalizeOptions{
        RejectZeroWidth:   true,  // Security
        MaxCombiningMarks: 2,     // Prevent stacking attacks
    },
)
if err != nil {
    return fmt.Errorf("identifier contains invalid characters: %w", err)
}
```

---

### 5. BOM Handling Operations

**Purpose**: Provide middleware-like helpers to "just handle" BOM framing issues that break XML parsers, CSV readers, and other text processors.

#### 5a. detect_bom() - Detect BOM

**Signature**:

```typescript
detect_bom(
  data: Uint8Array
): BOMResult
```

**Returns**: `BOMResult`

```typescript
interface BOMResult {
  bom_type: string | null; // "utf-8" | "utf-16le" | "utf-16be" | "utf-32le" | "utf-32be" | null
  byte_length: number; // BOM length in bytes (0 if no BOM)
  encoding_implied: string; // Encoding indicated by BOM
}
```

**Example**:

```python
from pyfulmen import fulencode

data = b"\xef\xbb\xbfHello"  # UTF-8 BOM + text

bom = fulencode.detect_bom(data)
print(bom.bom_type)         # "utf-8"
print(bom.byte_length)      # 3
print(bom.encoding_implied) # "utf-8"
```

#### 5b. remove_bom() - Remove BOM

**Signature**:

```typescript
remove_bom(
  data: Uint8Array,
  encoding?: string
): Uint8Array
```

**Parameters**:

- `data`: Byte sequence (may contain BOM)
- `encoding`: Expected encoding (optional) - if provided, validates BOM matches

**Returns**: Byte sequence with BOM removed

**Security**: If `encoding` provided, raises error if BOM doesn't match

**Example**:

```python
# Auto-detect and remove
data_clean = fulencode.remove_bom(data)

# Validate BOM matches expected encoding
try:
    data_clean = fulencode.remove_bom(data, encoding="utf-8")
except fulencode.FulencodeError as e:
    if e.code == "BOM_MISMATCH":
        print(f"BOM doesn't match UTF-8: {e.details}")
```

#### 5c. add_bom() - Add BOM

**Signature**:

```typescript
add_bom(
  data: Uint8Array,
  encoding: string
): Uint8Array
```

**Parameters**:

- `data`: Byte sequence (without BOM)
- `encoding`: Target encoding (determines BOM bytes)

**Returns**: Byte sequence with BOM prepended

**Security warning**: BOM injection can confuse parsers

**Example**:

```python
# Add UTF-8 BOM for Windows compatibility
data_with_bom = fulencode.add_bom(utf8_bytes, encoding="utf-8")
```

#### 5d. validate_bom() - Validate BOM

**Signature**:

```typescript
validate_bom(
  data: Uint8Array,
  expected_encoding: string
): boolean
```

**Returns**: True if BOM matches expected encoding or no BOM (for encodings that don't require it)

**Example**:

```python
if fulencode.validate_bom(data, "utf-16le"):
    print("✓ BOM valid for UTF-16LE")
else:
    print("✗ BOM mismatch or missing")
```

#### 5e. correct_bom() - Middleware Pattern (Auto-Fix)

**Signature**:

```typescript
correct_bom(
  source: string | URI | Uint8Array,
  options?: CorrectBOMOptions
): Uint8Array
```

**Purpose**: "Just handle it" - detect, validate, and correct BOM issues without complaint

**CorrectBOMOptions**:

```typescript
{
  expected_encoding?: string,     // If known, validate against this
  prefer_no_bom?: boolean,        // Remove BOM if present (default: false)
  add_if_missing?: boolean,       // Add BOM if expected but missing (default: false)
  on_mismatch?: "error" | "fix" | "ignore",  // BOM mismatch handling (default: "fix")
}
```

**Use case**: Context manager / middleware for XML parsers, CSV readers, etc.

**Example** (Python with context manager):

```python
# Middleware pattern for XML parser
with fulencode.correct_bom_context("data.xml", expected_encoding="utf-8") as corrected_data:
    tree = ET.parse(io.BytesIO(corrected_data))  # Parser doesn't choke on BOM

# Explicit correction
corrected = fulencode.correct_bom(
    source="user_upload.csv",
    options={
        "expected_encoding": "utf-8",
        "prefer_no_bom": True,      # Most CSV parsers don't expect BOM
        "on_mismatch": "fix"        # Auto-fix if wrong BOM present
    }
)
df = pd.read_csv(io.BytesIO(corrected))  # Clean data
```

**Example** (Go middleware):

```go
// Middleware for HTTP response bodies
func WithBOMCorrection(handler http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        // Read body
        body, _ := io.ReadAll(r.Body)

        // Correct BOM issues
        corrected, err := fulencode.CorrectBOM(body, &fulencode.CorrectBOMOptions{
            PreferNoBOM: true,
            OnMismatch:  fulencode.Fix,
        })

        // Replace body with corrected version
        r.Body = io.NopCloser(bytes.NewReader(corrected))

        handler.ServeHTTP(w, r)
    })
}
```

---

## (CONTINUED IN NEXT MESSAGE - HIT TOKEN LIMIT FOR SINGLE WRITE)

I'll continue in the next file...

## Streaming API (Planned - Implementation Deferred to v1.1.0)

**Status**: API reserved, implementation deferred per fulpack pattern

**Why plan now**: Ensures block API doesn't prevent streaming later; reserves method names; defines resource cleanup patterns

**Streaming operations** (future v1.1.0):

- `encode_stream()` - Streaming encoding for large payloads
- `decode_stream()` - Streaming decoding for large payloads
- `detect_stream()` - Progressive detection with streaming input

### Forward Compatibility Confirmation

**Schema Compatibility**: Current schemas are designed to support streaming without breaking changes

**No schema migrations required** when adding streaming in v1.1.0:

1. **Operation schemas remain unchanged**: Streaming variants use same option/result schemas
   - `EncodeOptions` works for both `encode()` and `encode_stream()`
   - `DecodingResult` works for both `decode()` and `decode_stream()`

2. **New method names**: Streaming uses distinct names (`*_stream`)
   - No conflicts with existing methods
   - Both APIs can coexist in same module

3. **Resource cleanup**: Schemas don't dictate cleanup patterns
   - Python: Add context manager protocol to stream objects
   - Go: Add `Close()` method to stream types
   - TypeScript: Add `finally()` to promise chains or async iterators

4. **Validation unchanged**: Same schemas validate both modes
   - Block API: Full object validation
   - Stream API: Per-chunk validation

**Example (forward-compatible API)** (v1.1.0 proposal):

```python
# v1.0.0 (block API)
result = fulencode.decode("SGVsbG8sIFdvcmxkIQ==", "base64", options)

# v1.1.0 (streaming API added, no schema changes)
with fulencode.decode_stream(input_stream, "base64", options) as stream:
    for chunk in stream:
        # Process chunk with same DecodingResult schema (partial)
        process(chunk.data)
# Returns same DecodingResult schema (aggregate)
```

**See**: `.plans/active/v0.2.x-plans/streaming-api-feature-brief.md` (to be created) for detailed streaming specification.

---

## Security Model

### Mandatory Protections

**ALL implementations MUST enforce these protections**:

#### 1. Buffer Overflow Protection

- **Encoding limits**: `max_encoded_size` (default: 500MB)
- **Decoding limits**: `max_decoded_size` (default: 100MB)
- **Memory monitoring**: Track allocations during encode/decode
- **Expansion ratio monitoring**: Warn if ratio > 10:1 (potential encoding bomb)

**Configuration**: Limits are configurable via module config schema (see Config Schema section)

```python
# Override defaults via config
fulencode_config = {
    "limits": {
        "max_decoded_size": 50 * 1024 * 1024,  # 50MB for constrained environments
        "max_encoded_size": 200 * 1024 * 1024,  # 200MB
        "max_expansion_ratio": 20               # Stricter bomb detection
    }
}

result = fulencode.decode(data, "base64", config=fulencode_config)
```

#### 2. Invalid UTF-8 Handling (Cross-Language Determinism)

**CRITICAL**: All implementations MUST handle invalid UTF-8 identically to ensure consistent behavior.

**Strategy**: Use **replacement character (U+FFFD)** in `replace` mode

```yaml
Invalid Byte Sequences (UTF-8):
  overlong_encodings:
    example: 0xC0 0x80 (overlong encoding of NULL)
    strict_mode: Error with INVALID_UTF8
    replace_mode: Single U+FFFD

  invalid_continuations:
    example: 0x80 (continuation byte without start)
    strict_mode: Error with INVALID_UTF8
    replace_mode: Single U+FFFD per invalid byte

  surrogate_codepoints:
    example: 0xED 0xA0 0x80 (U+D800)
    strict_mode: Error with INVALID_UTF8
    replace_mode: Single U+FFFD

  out_of_range:
    example: 0xF4 0x90 0x80 0x80 (U+110000, beyond Unicode)
    strict_mode: Error with INVALID_UTF8
    replace_mode: Single U+FFFD

  truncated_sequences:
    example: 0xE2 0x82 (incomplete 3-byte sequence)
    strict_mode: Error with INVALID_UTF8
    replace_mode: Single U+FFFD
```

**Example**:

```python
# Strict mode (for round-trip integrity)
try:
    result = fulencode.decode(bad_utf8, "utf-8", options={"on_error": "strict"})
except fulencode.FulencodeError as e:
    assert e.code == "INVALID_UTF8"
    assert e.details["byte_offset"] == 5  # Where error occurred

# Replace mode (for resilient parsing)
result = fulencode.decode(bad_utf8, "utf-8", options={"on_error": "replace"})
assert "�" in result.data.decode("utf-8")  # U+FFFD replacement character
assert result.corrections_applied > 0
```

#### 3. Invalid UTF-16 Handling (Unpaired Surrogates)

**UTF-16 specifics**: Surrogates must be paired correctly

```yaml
Invalid Surrogate Sequences (UTF-16):
  unpaired_high_surrogate:
    example: 0xD800 0x0041 (high surrogate followed by non-surrogate)
    strict_mode: Error with INVALID_UTF16
    replace_mode: U+FFFD for unpaired surrogate, keep 0x0041

  unpaired_low_surrogate:
    example: 0xDC00 (low surrogate without preceding high)
    strict_mode: Error with INVALID_UTF16
    replace_mode: U+FFFD

  reversed_surrogates:
    example: 0xDC00 0xD800 (low before high)
    strict_mode: Error with INVALID_UTF16
    replace_mode: Two U+FFFD characters

  truncated_input:
    example: File ends mid-surrogate-pair
    strict_mode: Error with INVALID_UTF16
    replace_mode: U+FFFD for incomplete pair
```

**Example** (Go):

```go
// Strict UTF-16LE for cryptographic signatures (must round-trip perfectly)
result, err := fulencode.Decode(utf16Data, fulencode.UTF16LE, &fulencode.DecodeOptions{
    OnError: fulencode.Strict,
})
if err != nil {
    var fulencodeErr *fulencode.FulencodeError
    if errors.As(err, &fulencodeErr) && fulencodeErr.Code == "INVALID_UTF16" {
        return fmt.Errorf("UTF-16 integrity compromised at offset %d: %w",
            fulencodeErr.Details["byte_offset"], err)
    }
    return err
}
// Safe to use for signature verification

// Replace mode for legacy data migration
resultResilient, err := fulencode.Decode(legacyUTF16, fulencode.UTF16BE, &fulencode.DecodeOptions{
    OnError: fulencode.Replace,
})
if resultResilient.CorrectionsApplied > 0 {
    log.Printf("⚠️ Migrated legacy data with %d corrections - verify manually",
        resultResilient.CorrectionsApplied)
}
```

#### 4. Normalization Attack Prevention

**NFKC/NFKD semantic changes** (security-critical):

```yaml
Semantic-Changing Normalizations:
  ligatures:
    original: "ﬁle" # U+FB01 (fi ligature)
    nfkc: "file" # Two separate characters
    risk: "Filename collisions, identifier confusion"

  superscripts_subscripts:
    original: "x²" # U+00B2 (superscript 2)
    nfkc: "x2" # Digit 2
    risk: "Mathematical expressions lose meaning"

  roman_numerals:
    original: "Ⅷ" # U+2167 (Roman numeral eight)
    nfkc: "VIII" # Four ASCII characters
    risk: "Identifier length changes"

  circled_numbers:
    original: "①" # U+2460 (circled digit one)
    nfkc: "1" # ASCII digit
    risk: "List markers become regular numbers"
```

**Protection strategy**:

```python
# Warn on semantic-changing normalizations
result = fulencode.normalize(
    text="File №2: ﬁnal résumé.pdf",
    profile="nfkc",
    options={"warn_semantic_change": True}
)

if result.semantic_changes:
    for change in result.semantic_changes:
        print(f"⚠️ Semantic change at position {change.position}:")
        print(f"   '{change.original}' → '{change.normalized}'")
        print(f"   Reason: {change.reason}")

    # Application decision:
    # - Allow for search/comparison (lossy OK)
    # - Reject for identifiers (preserve original)
    if use_case == "identifier":
        raise ValueError("NFKC changes semantic meaning - use NFC for identifiers")
```

#### 5. Combining Mark Limits (Quadratic Blowup)

**Attack vector**: Deeply nested combining marks cause quadratic processing time

```python
# Malicious input: base char + 100 combining marks
malicious = "e" + "\u0301" * 100  # e + 100 acute accents

# Protection: Limit combining marks per base character
try:
    result = fulencode.normalize(
        malicious,
        profile="nfc",
        options={"max_combining_marks": 10}  # Reasonable limit
    )
except fulencode.FulencodeError as e:
    assert e.code == "EXCESSIVE_COMBINING_MARKS"
```

#### 6. Zero-Width Character Detection (Hiding Attacks)

**Security risk**: Zero-width characters hide malicious content

```yaml
Zero-Width Characters:
  U+200B: ZERO WIDTH SPACE
  U+200C: ZERO WIDTH NON-JOINER
  U+200D: ZERO WIDTH JOINER
  U+FEFF: ZERO WIDTH NO-BREAK SPACE (BOM in middle of text)

Attack Examples:
  - Hidden commands in filenames: "delete.sh\u200Bsafe.txt"
  - Homograph attacks: "pay\u200Bpal.com" (looks like paypal.com)
  - Obfuscated code: "eval\u200B(user_input)"
```

**Protection**:

```python
# Reject zero-width characters for identifiers
result = fulencode.normalize(
    username,
    profile="safe_identifiers",
    options={"reject_zero_width": True}
)
# Raises error if zero-width chars present
```

### Security Test Requirements

All implementations MUST pass security tests for:

- **Invalid UTF-8**: Overlong encodings, surrogates, truncated sequences, invalid continuations
- **Invalid UTF-16**: Unpaired surrogates, reversed pairs, truncated input
- **Encoding bombs**: 1KB Base64 → 1GB decoded (should hit max_decoded_size limit)
- **Normalization attacks**: NFKC semantic changes, excessive combining marks, zero-width chars
- **BOM injection**: Multiple BOMs, mismatched BOMs, BOM in middle of text

---

## Error Handling

### Canonical Error Envelope

All fulencode errors MUST use this envelope structure (compatible with Foundry error schemas):

```typescript
interface FulencodeError {
  code: string; // Canonical error code (see below)
  message: string; // Human-readable message
  operation: string; // Operation name (encode, decode, detect, normalize, bom)
  input_format?: string; // Source format (if applicable)
  output_format?: string; // Target format (if applicable)
  details?: {
    byte_offset?: number; // Where error occurred
    codepoint_offset?: number; // Unicode position
    detected_encoding?: string; // What was detected
    confidence?: number; // Detection confidence
    invalid_bytes?: number[]; // Invalid byte sequence
    expected?: string; // What was expected
    actual?: string; // What was found
  };
}
```

### Canonical Error Codes

**Validation Errors** (invalid input):

- `INVALID_ENCODING` - Input not valid for specified format
  - Example: Invalid Base64 character, non-hex digit in hex string

- `UNSUPPORTED_FORMAT` - Format not implemented
  - Example: Requesting Base58 without specialized module

- `INVALID_OPTIONS` - Invalid options passed to operation
  - Example: Negative line_length, invalid normalization profile

**UTF Validation Errors**:

- `INVALID_UTF8` - UTF-8 validation failed
  - Subcodes: `overlong_encoding`, `invalid_continuation`, `surrogate_codepoint`, `out_of_range`, `truncated_sequence`

- `INVALID_UTF16` - UTF-16 validation failed
  - Subcodes: `unpaired_high_surrogate`, `unpaired_low_surrogate`, `reversed_surrogates`, `truncated_input`

**Security Errors** (protection triggered):

- `BUFFER_OVERFLOW` - Decoded size exceeds limit
  - Details include `actual_size`, `max_size`

- `ENCODING_BOMB` - Expansion ratio exceeds threshold
  - Details include `expansion_ratio`, `max_ratio`

- `EXCESSIVE_COMBINING_MARKS` - Too many combining marks per base character
  - Details include `mark_count`, `max_marks`

- `ZERO_WIDTH_CHARACTER` - Zero-width character detected in strict mode
  - Details include `character`, `codepoint`, `position`

**BOM Errors**:

- `BOM_MISMATCH` - Detected BOM doesn't match declared encoding
  - Details include `detected_bom`, `expected_encoding`

- `MULTIPLE_BOMS` - Multiple BOMs detected in text
  - Details include `bom_positions`

**Detection Errors**:

- `DETECTION_FAILED` - Encoding detection confidence too low
  - Details include `confidence`, `min_confidence`, `candidates`

- `AMBIGUOUS_ENCODING` - Multiple encodings equally likely
  - Details include `candidates` with equal confidence

**Normalization Errors**:

- `NORMALIZATION_ERROR` - Normalization transformation failed
  - Subcodes: `invalid_codepoint`, `profile_not_found`, `semantic_change_rejected`

**Runtime Errors** (I/O failures):

- `FILE_NOT_FOUND` - Input file does not exist
- `PERMISSION_DENIED` - Insufficient permissions
- `READ_ERROR` - Failed to read input
- `WRITE_ERROR` - Failed to write output

### Example Error Handling

```python
from pyfulmen import fulencode
from pyfulmen.fulencode import FulencodeError

# Catch specific error codes
try:
    result = fulencode.decode(untrusted_input, "base64")
except FulencodeError as e:
    if e.code == "ENCODING_BOMB":
        log.error(f"Potential attack: {e.details['expansion_ratio']}x expansion")
        alert_security_team(e)
    elif e.code == "INVALID_ENCODING":
        log.warning(f"Invalid Base64 at offset {e.details['byte_offset']}")
        # Maybe try fallback encoding
    else:
        raise  # Unexpected error

# UTF-8 validation with detailed error
try:
    result = fulencode.decode(data, "utf-8", options={"on_error": "strict"})
except FulencodeError as e:
    if e.code == "INVALID_UTF8":
        subcode = e.details.get("subcode")
        offset = e.details.get("byte_offset")
        invalid_bytes = e.details.get("invalid_bytes")

        print(f"UTF-8 validation failed: {subcode} at offset {offset}")
        print(f"Invalid bytes: {' '.join(f'{b:02x}' for b in invalid_bytes)}")

        # Decision: retry with replace mode or reject entirely
        if allow_correction:
            result = fulencode.decode(data, "utf-8", options={"on_error": "replace"})
            log.warning(f"Corrected {result.corrections_applied} UTF-8 errors")
        else:
            raise ValueError("UTF-8 integrity required but validation failed")
```

---

## Configuration Schema

**Location**: `schemas/library/fulencode/v1.0.0/fulencode-config.schema.json`

**Purpose**: Allow applications to configure default limits and behaviors module-wide (especially for testing and constrained environments)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://schemas.fulmenhq.dev/library/fulencode/fulencode-config-v1.0.0.json",
  "title": "Fulencode Module Configuration",
  "description": "Configuration schema for fulencode module limits and defaults",
  "type": "object",
  "properties": {
    "limits": {
      "type": "object",
      "properties": {
        "max_decoded_size": {
          "type": "integer",
          "description": "Maximum decoded output size in bytes",
          "default": 104857600,
          "minimum": 1024
        },
        "max_encoded_size": {
          "type": "integer",
          "description": "Maximum encoded output size in bytes",
          "default": 524288000,
          "minimum": 1024
        },
        "max_expansion_ratio": {
          "type": "number",
          "description": "Maximum decode expansion ratio (encoding bomb detection)",
          "default": 10.0,
          "minimum": 1.0
        },
        "max_combining_marks": {
          "type": "integer",
          "description": "Maximum combining marks per base character",
          "default": 10,
          "minimum": 1,
          "maximum": 100
        }
      }
    },
    "defaults": {
      "type": "object",
      "properties": {
        "on_error": {
          "type": "string",
          "enum": ["strict", "replace", "ignore", "fallback"],
          "description": "Default error handling mode",
          "default": "strict"
        },
        "checksum_algorithm": {
          "type": "string",
          "enum": ["xxh3-128", "sha256", "sha512", "sha1", "md5"],
          "description": "Default checksum algorithm for integrity checks",
          "default": "sha256"
        },
        "normalization_profile": {
          "type": "string",
          "description": "Default normalization profile",
          "default": "nfc"
        }
      }
    },
    "detection": {
      "type": "object",
      "properties": {
        "min_confidence": {
          "type": "number",
          "description": "Minimum detection confidence threshold",
          "default": 0.5,
          "minimum": 0.0,
          "maximum": 1.0
        },
        "max_sample_size": {
          "type": "integer",
          "description": "Maximum bytes to analyze for detection",
          "default": 8192,
          "minimum": 32
        }
      }
    }
  }
}
```

**Usage**:

```python
# Load config for testing (smaller limits)
test_config = {
    "limits": {
        "max_decoded_size": 10 * 1024 * 1024,  # 10MB for tests
        "max_encoded_size": 50 * 1024 * 1024,  # 50MB
        "max_expansion_ratio": 5.0              # Stricter
    },
    "defaults": {
        "on_error": "replace"  # Resilient mode for tests
    }
}

# Apply config module-wide
fulencode.configure(test_config)

# Or per-operation
result = fulencode.decode(data, "base64", config=test_config)
```

---

## Testing Requirements

### Mandatory Test Coverage

All implementations MUST provide:

1. **Unit tests**: Each operation tested in isolation
2. **Security tests**: Invalid UTF-8/UTF-16, encoding bombs, normalization attacks, BOM injection
3. **Cross-language parity tests**: Same input → same output across Go/Python/TypeScript
4. **Integration tests**: Fulhash integration, Fulpack integration (file name normalization)
5. **Fixture tests**: Using fixtures from `config/library/fulencode/fixtures/`
6. **Portable testing compliance**: Follow [Portable Testing Practices](../../testing/portable-testing-practices.md)

**Test coverage target**: ≥70%

### Portable Testing Requirements (Critical for Fulencode)

**Sandbox compatibility** (no network, limited filesystem):

- **No network access**: All encoding tests local (no downloads, no external validation services)
- **Deterministic behavior**: Same bytes → same result across environments
- **Temp file cleanup**: MUST clean up ALL temp files in test teardown
  - Go: Use `t.Cleanup()` to register cleanup handlers
  - Python: Use pytest `tmp_path` fixture or `@pytest.fixture` with cleanup
  - TypeScript: Use `afterEach()` to remove temp directories
- **Memory limits**: Encoding bomb tests MUST verify memory limits work
  - Test that `max_decoded_size` prevents OOM
  - Use in-memory streams for unit tests (avoid disk I/O)
- **Capability detection**: Skip tests for optional features
  - Specialized encodings (Base58, Shift-JIS) - skip if not available
  - Statistical detection libraries - skip if not installed
  - Normalization (Python): Skip if unicodedata missing (shouldn't happen but defensive)

**Example capability detection**:

```python
import pytest

# Skip if specialized module not available
@pytest.mark.skipif(
    not fulencode.has_extension("base58"),
    reason="Base58 specialized module not installed"
)
def test_base58_encoding():
    result = fulencode.encode(data, "base58")
    assert result.format == "base58"

# Skip if statistical detection not available
@pytest.mark.skipif(
    not fulencode.has_feature("statistical_detection"),
    reason="Statistical detection (chardet) not available"
)
def test_advanced_detection():
    result = fulencode.detect(ambiguous_data, options={"use_statistical": True})
    assert result.confidence > 0.8
```

### Security Test Cases (Mandatory)

**Invalid UTF-8** (MUST detect and handle):

```python
def test_invalid_utf8_overlong_encoding():
    """Overlong encoding of NULL (security vulnerability)"""
    invalid = bytes([0xC0, 0x80])  # Overlong encoding of U+0000

    # Strict mode: must error
    with pytest.raises(FulencodeError) as exc_info:
        fulencode.decode(invalid, "utf-8", options={"on_error": "strict"})
    assert exc_info.value.code == "INVALID_UTF8"
    assert exc_info.value.details["subcode"] == "overlong_encoding"

    # Replace mode: must use U+FFFD
    result = fulencode.decode(invalid, "utf-8", options={"on_error": "replace"})
    assert "\ufffd" in result.data.decode("utf-8")
    assert result.corrections_applied == 1

def test_invalid_utf8_surrogate():
    """UTF-16 surrogate in UTF-8 (invalid)"""
    invalid = bytes([0xED, 0xA0, 0x80])  # U+D800 (high surrogate)

    with pytest.raises(FulencodeError) as exc_info:
        fulencode.decode(invalid, "utf-8", options={"on_error": "strict"})
    assert exc_info.value.code == "INVALID_UTF8"
    assert exc_info.value.details["subcode"] == "surrogate_codepoint"
```

**Invalid UTF-16** (MUST detect and handle):

```python
def test_invalid_utf16_unpaired_high_surrogate():
    """High surrogate not followed by low surrogate"""
    invalid = bytes([0x00, 0xD8, 0x00, 0x41])  # U+D800 U+0041 (big-endian)

    with pytest.raises(FulencodeError) as exc_info:
        fulencode.decode(invalid, "utf-16be", options={"on_error": "strict"})
    assert exc_info.value.code == "INVALID_UTF16"
    assert exc_info.value.details["subcode"] == "unpaired_high_surrogate"

    # Replace mode: U+FFFD for surrogate, keep U+0041
    result = fulencode.decode(invalid, "utf-16be", options={"on_error": "replace"})
    assert result.data.decode("utf-16be") == "\ufffdA"
```

**Encoding bombs** (MUST detect):

```python
def test_encoding_bomb_base64():
    """1KB Base64 → 1GB decoded (should hit limit)"""
    # Create 1GB of zeros
    bomb_data = b"\x00" * (1024 * 1024 * 1024)  # 1GB
    # Encode to Base64 (compresses well, ~1KB)
    bomb_b64 = fulencode.encode(bomb_data, "base64").data

    # Decode with default limit (100MB) - should fail
    with pytest.raises(FulencodeError) as exc_info:
        fulencode.decode(bomb_b64, "base64")

    assert exc_info.value.code == "BUFFER_OVERFLOW"
    assert exc_info.value.details["actual_size"] > exc_info.value.details["max_size"]
```

**Normalization attacks** (MUST detect):

```python
def test_normalization_excessive_combining_marks():
    """Quadratic blowup attack with combining marks"""
    malicious = "e" + "\u0301" * 100  # e + 100 acute accents

    with pytest.raises(FulencodeError) as exc_info:
        fulencode.normalize(
            malicious,
            profile="nfc",
            options={"max_combining_marks": 10}
        )

    assert exc_info.value.code == "EXCESSIVE_COMBINING_MARKS"
    assert exc_info.value.details["mark_count"] == 100
    assert exc_info.value.details["max_marks"] == 10

def test_normalization_semantic_change_detection():
    """NFKC semantic changes detected"""
    text = "Café ﬁle №2"  # Contains ligature and special chars

    result = fulencode.normalize(
        text,
        profile="nfkc",
        options={"warn_semantic_change": True}
    )

    assert len(result.semantic_changes) > 0
    # Check ligature was flagged
    ligature_change = [c for c in result.semantic_changes if "ligature" in c.reason.lower()]
    assert len(ligature_change) > 0
```

**BOM attacks** (MUST detect):

```python
def test_bom_mismatch():
    """UTF-8 BOM with UTF-16 content"""
    utf8_bom = bytes([0xEF, 0xBB, 0xBF])
    utf16_content = "test".encode("utf-16le")
    malicious = utf8_bom + utf16_content

    with pytest.raises(FulencodeError) as exc_info:
        fulencode.decode(malicious, "utf-8", options={"on_error": "strict"})

    # Should fail UTF-8 validation after BOM
    assert exc_info.value.code in ["INVALID_UTF8", "BOM_MISMATCH"]

def test_multiple_boms():
    """Multiple BOMs in text"""
    utf8_bom = bytes([0xEF, 0xBB, 0xBF])
    malicious = utf8_bom + b"Hello" + utf8_bom + b"World"

    result = fulencode.detect_bom(malicious)
    # Should only detect first BOM
    assert result.bom_type == "utf-8"

    # Validation should warn about second BOM
    # (Implementation-specific: might be MULTIPLE_BOMS error or warning)
```

---

## Telemetry Testing Requirements

### Purpose

All fulencode implementations MUST test telemetry instrumentation to ensure (at minimum, Wave 1 metrics):

1. Metrics are emitted correctly with proper labels
2. Security violations increment appropriate counters
3. Security violations increment appropriate counters
4. Performance metrics capture operation timing for core operations

Wave 2/3 metrics SHOULD have tests when implemented, but they MUST NOT block initial module adoption.

**Rationale**: Specialized modules often skip telemetry testing, leading to blind spots in production monitoring. Making telemetry testing mandatory prevents this.

### Test Helper Requirements by Language

#### Python (pyfulmen)

**Use telemetry module's test collector**:

```python
from pyfulmen.telemetry.testing import MetricsCollector
from pyfulmen import fulencode
from pyfulmen.fulencode import EncodingFormat, FulencodeError
import pytest

def test_encode_telemetry():
    """Verify encode() emits correct telemetry."""
    with MetricsCollector() as metrics:
        # Perform encode operation
        result = fulencode.encode(
            data=b"Hello, World!",
            format=EncodingFormat.BASE64
        )

    # Verify operation metrics
    assert metrics.histogram("fulencode_operation_duration_seconds") > 0
    assert metrics.counter(
        "fulencode_operation_total",
        labels={"operation": "encode", "format": "base64", "result": "success"}
    ) == 1

    # Verify data volume metrics
    assert metrics.histogram(
        "fulencode_bytes_processed_total",
        labels={
            "operation": "encode",
            "direction": "in",
            "format": "base64",
            "result": "success",
        }
    ) == 13  # Length of "Hello, World!"

def test_security_violation_telemetry():
    """Verify security violations are tracked."""
    with MetricsCollector() as metrics:
        # Attempt encoding bomb (should fail with BUFFER_OVERFLOW)
        with pytest.raises(FulencodeError) as exc_info:
            # 1GB of data (exceeds default 100MB limit)
            fulencode.decode(
                "A" * 1000000000,  # Fake base64
                format=EncodingFormat.BASE64
            )

        assert exc_info.value.code == "BUFFER_OVERFLOW"

    # Verify security violation was tracked
    assert metrics.counter(
        "fulencode_security_violations_total",
        labels={"type": "encoding_bomb", "operation": "decode"}
    ) == 1

def test_detection_confidence_telemetry():
    """Verify detection outcomes are categorized correctly."""
    with MetricsCollector() as metrics:
        # Detect UTF-8 with BOM (should be high confidence)
        utf8_with_bom = b"\xef\xbb\xbfHello"
        result = fulencode.detect(utf8_with_bom)

        assert result.confidence_level == "high"

    # Verify detection was tracked
    assert metrics.counter(
        "fulencode_detect_result_total",
        labels={"encoding": "utf-8", "confidence": "high", "result": "success"}
    ) == 1
```

**Test coverage requirements**:

- ✅ Every operation emits duration and count metrics
- ✅ Security violations tracked for all error codes
- ✅ Bytes processed tracked for encode/decode
- ✅ Detection confidence buckets populated correctly
- ⏸️ Normalization semantic changes tracked (Wave 2)

#### Go (gofulmen)

**Use in-memory telemetry sink**:

```go
package fulencode_test

import (
    "testing"

    "github.com/fulmenhq/gofulmen/fulencode"
    "github.com/fulmenhq/gofulmen/telemetry"
    "github.com/fulmenhq/gofulmen/telemetry/testing"
)

func TestEncodeTelemetry(t *testing.T) {
    // Setup in-memory sink
    sink := telemetry_testing.NewMemorySink()
    telemetry.SetGlobalSink(sink)
    defer telemetry.SetGlobalSink(nil)  // Cleanup

    // Perform encode operation
    data := []byte("Hello, World!")
    result, err := fulencode.Encode(data, fulencode.BASE64, nil)
    if err != nil {
        t.Fatalf("encode failed: %v", err)
    }

    // Verify operation metrics
    metrics := sink.GetMetrics()

    if !metrics.HasHistogram("fulencode_operation_duration_seconds") {
        t.Error("missing operation duration metric")
    }

    if count := metrics.GetCounter("fulencode_operation_total", map[string]string{
        "operation": "encode",
        "format":    "base64",
        "result":    "success",
    }); count != 1 {
        t.Errorf("expected 1 operation, got %d", count)
    }

    // Verify bytes processed
    if bytes := metrics.GetHistogramValue("fulencode_bytes_processed_total", map[string]string{
        "operation": "encode",
        "direction": "in",
        "format":    "base64",
        "result":    "success",
    }); bytes != int64(len(data)) {
        t.Errorf("expected %d bytes processed, got %d", len(data), bytes)
    }
}

func TestSecurityViolationTelemetry(t *testing.T) {
    sink := telemetry_testing.NewMemorySink()
    telemetry.SetGlobalSink(sink)
    defer telemetry.SetGlobalSink(nil)

    // Attempt invalid UTF-8 in strict mode
    invalidUTF8 := []byte{0xC0, 0x80}  // Overlong encoding

    _, err := fulencode.Decode(invalidUTF8, fulencode.UTF8, &fulencode.DecodeOptions{
        OnError: fulencode.Strict,
    })

    if err == nil {
        t.Fatal("expected error for invalid UTF-8")
    }

    // Verify security violation tracked
    metrics := sink.GetMetrics()
    if count := metrics.GetCounter("fulencode_security_violations_total", map[string]string{
        "type":      "invalid_utf8",
        "operation": "decode",
    }); count != 1 {
        t.Errorf("expected 1 security violation, got %d", count)
    }
}
```

**Test requirements**:

- Use `telemetry/testing.MemorySink` for all telemetry tests
- Clear sink between test functions
- Test all metric types (counters, histograms, gauges)
- Verify label correctness

#### TypeScript (tsfulmen)

**Use telemetry test utilities**:

```typescript
import {
  fulencode,
  EncodingFormat,
  FulencodeError,
} from "@fulmenhq/tsfulmen/fulencode";
import { telemetry } from "@fulmenhq/tsfulmen/telemetry";
import { MemoryMetricsCollector } from "@fulmenhq/tsfulmen/telemetry/testing";

describe("Fulencode Telemetry", () => {
  let collector: MemoryMetricsCollector;

  beforeEach(() => {
    collector = new MemoryMetricsCollector();
    telemetry.setCollector(collector);
  });

  afterEach(() => {
    telemetry.setCollector(null); // Reset
  });

  test("encode emits operation metrics", async () => {
    // Perform encode
    const result = await fulencode.encode(
      new Uint8Array([72, 101, 108, 108, 111]), // "Hello"
      EncodingFormat.BASE64,
    );

    // Verify metrics
    expect(collector.hasHistogram("fulencode_operation_duration_seconds")).toBe(
      true,
    );

    expect(
      collector.getCounter("fulencode_operation_total", {
        operation: "encode",
        format: "base64",
        result: "success",
      }),
    ).toBe(1);

    expect(
      collector.getHistogramValue("fulencode_bytes_processed_total", {
        operation: "encode",
        direction: "in",
        format: "base64",
        result: "success",
      }),
    ).toBe(5);
  });

  test("detection confidence tracked correctly", async () => {
    // UTF-8 with BOM (high confidence)
    const utf8WithBom = new Uint8Array([
      0xef, 0xbb, 0xbf, 72, 101, 108, 108, 111,
    ]);

    const detected = await fulencode.detect(utf8WithBom);

    expect(detected.confidenceLevel).toBe("high");

    expect(
      collector.getCounter("fulencode_detect_result_total", {
        encoding: "utf-8",
        confidence: "high",
        result: "success",
      }),
    ).toBe(1);
  });

  test("normalization semantic changes tracked", async () => {
    // NFKC normalization (semantic-changing)
    const text = "Café ﬁle"; // Contains ligature

    const result = await fulencode.normalize(text, "nfkc", {
      warnSemanticChange: true,
    });

    if (result.semanticChanges.length > 0) {
      expect(
        collector.getCounter("fulencode_normalize_semantic_changes_total", {
          profile: "nfkc",
          change_type: "ligatures",
        }),
      ).toBeGreaterThan(0);
    }
  });

  test("security violations tracked", async () => {
    // Encoding bomb attempt
    const hugeData = "A".repeat(1000000000); // 1GB

    await expect(
      fulencode.decode(hugeData, EncodingFormat.BASE64),
    ).rejects.toThrow(FulencodeError);

    expect(
      collector.getCounter("fulencode_security_violations_total", {
        type: "buffer_overflow",
        operation: "decode",
      }),
    ).toBe(1);
  });
});
```

**Test requirements**:

- Use `MemoryMetricsCollector` from telemetry testing package
- Clear collector in `beforeEach` / `afterEach`
- Test async operations with `await`
- Verify all label combinations

### Cross-Language Telemetry Parity Tests

**Purpose**: Ensure telemetry behaves identically across Go/Python/TypeScript.

**Shared test fixtures** (in `config/library/fulencode/fixtures/telemetry/`):

```yaml
# telemetry-test-cases.yaml
test_cases:
  - name: "basic_encode"
    operation: "encode"
    input:
      data: "SGVsbG8sIFdvcmxkIQ==" # Base64 of "Hello, World!"
      format: "base64"
    expected_metrics:
      - metric: "fulencode_operation_total"
        labels: { operation: "encode", format: "base64", result: "success" }
        value: 1
      - metric: "fulencode_bytes_processed_total"
        labels: { direction: "encode", format: "base64" }
        value: 13

  - name: "invalid_utf8_security_violation"
    operation: "decode"
    input:
      data: [0xC0, 0x80] # Overlong encoding
      format: "utf-8"
      options: { on_error: "strict" }
    expected_error: "INVALID_UTF8"
    expected_metrics:
      - metric: "fulencode_security_violations_total"
        labels: { type: "invalid_utf8", operation: "decode" }
        value: 1

  - name: "high_confidence_detection"
    operation: "detect"
    input:
      data: [0xEF, 0xBB, 0xBF, 72, 101, 108, 108, 111] # UTF-8 BOM + "Hello"
    expected_metrics:
      - metric: "fulencode_detect_result_total"
        labels: { encoding: "utf-8", confidence: "high", result: "success" }
        value: 1
```

**Each language implements**:

```python
# Python
def test_telemetry_parity():
    """Run all telemetry test cases from fixtures."""
    with open("config/library/fulencode/fixtures/telemetry/telemetry-test-cases.yaml") as f:
        test_cases = yaml.safe_load(f)["test_cases"]

    for case in test_cases:
        with MetricsCollector() as metrics:
            # Run operation
            run_test_case(case)

            # Verify metrics
            for expected_metric in case["expected_metrics"]:
                actual = metrics.get_metric(
                    expected_metric["metric"],
                    expected_metric["labels"]
                )
                assert actual == expected_metric["value"], \
                    f"Metric mismatch for {case['name']}: {expected_metric['metric']}"
```

### Telemetry Test Coverage Requirements

**Minimum coverage for each language**:

1. ✅ **Operation metrics** - Every operation (encode, decode, detect, normalize, bom)
2. ✅ **Security violations** - All security error codes tracked
3. ✅ **Data volume** - Bytes processed and expansion ratios
4. ✅ **Detection outcomes** - All confidence buckets (low, medium, high)
5. ✅ **Normalization tracking** - Semantic changes detected
6. ✅ **Error tracking** - All error codes increment counters
7. ✅ **BOM operations** - All BOM operation types tracked
8. ✅ **Cross-language parity** - Shared fixtures produce same metrics

**CI/CD enforcement**:

```yaml
# .github/workflows/test.yml (example)
- name: Test Telemetry Instrumentation
  run: |
    # Run telemetry-specific tests
    pytest tests/telemetry/  # Python
    go test -run Telemetry ./...  # Go
    npm test -- --testPathPattern=telemetry  # TypeScript

    # Verify all operations emit metrics
    ./scripts/verify-telemetry-coverage.sh
```

### Documentation Requirements

**Each language's README MUST document**:

1. How to enable telemetry in tests (`MetricsCollector`, `MemorySink`, etc.)
2. How to verify metrics in tests (assertion helpers)
3. Example test showing complete instrumentation
4. Link to telemetry test fixtures

**Example (pyfulmen README.md)**:

```markdown
## Testing Telemetry

Pyfulmen provides `MetricsCollector` for testing telemetry instrumentation:

\`\`\`python
from pyfulmen.telemetry.testing import MetricsCollector

def test_my_operation():
with MetricsCollector() as metrics: # Perform operation
result = fulencode.encode(data, "base64")

        # Verify metrics
        assert metrics.counter("fulencode_operation_total") == 1

\`\`\`

See `tests/telemetry/` for complete examples and `config/library/fulencode/fixtures/telemetry/` for shared test cases.
```

---

## Test Fixtures

**Location**: `config/library/fulencode/fixtures/`

**Size constraint**: Keep total fixture size under 1MB (per v0.2.12 guidance)

**Canonical fixtures** (v1.0.0):

### 1. Valid Encodings (`valid-encodings/`)

- `utf8-valid.txt` (5KB) - Normal UTF-8 text with emoji, accents, CJK
- `utf16le-bom.txt` (3KB) - UTF-16LE with BOM
- `utf16be-bom.txt` (3KB) - UTF-16BE with BOM
- `iso-8859-1-valid.txt` (2KB) - ISO-8859-1 (Latin-1) text
- `cp1252-curly-quotes.txt` (2KB) - Windows-1252 with smart quotes
- `ascii-pure.txt` (1KB) - Pure ASCII (7-bit)

### 2. Invalid Encodings (`invalid-encodings/`)

- `utf8-overlong.bin` (1KB) - Overlong UTF-8 encodings
- `utf8-surrogates.bin` (1KB) - UTF-16 surrogates in UTF-8
- `utf8-truncated.bin` (500B) - Incomplete multi-byte sequences
- `utf16le-unpaired-surrogates.bin` (2KB) - Unpaired UTF-16 surrogates
- `mojibake-double-encoded.txt` (3KB) - Double-encoded text (UTF-8 → Latin-1 → UTF-8)

### 3. Normalization Test Cases (`normalization/`)

- `nfc-nfd-pairs.txt` (5KB) - Characters with NFC/NFD differences (é vs e+´)
- `nfkc-semantic-change.txt` (3KB) - NFKC semantic changes (ﬁ, ², №, etc.)
- `combining-marks.txt` (4KB) - Various combining diacritics
- `zero-width-chars.txt` (1KB) - Zero-width characters (U+200B, etc.)

### 4. BOM Test Cases (`bom/`)

- `utf8-bom.txt` (2KB) - UTF-8 with BOM
- `utf16le-bom.txt` (2KB) - UTF-16LE with BOM
- `utf16be-bom.txt` (2KB) - UTF-16BE with BOM
- `no-bom-utf8.txt` (2KB) - UTF-8 without BOM
- `bom-mismatch.bin` (1KB) - Wrong BOM for content encoding
- `multiple-boms.bin` (2KB) - Multiple BOMs in same file

### 5. Pathological Cases (`pathological/`)

- `encoding-bomb.b64` (50KB) - Base64 encoding of highly compressible data
- `excessive-combining.txt` (5KB) - Excessive combining marks per character
- `deeply-nested-normalization.txt` (10KB) - Normalization quadratic blowup

### 6. Detection Test Cases (`detection/`)

- `ambiguous-ascii.txt` (2KB) - Pure ASCII (could be UTF-8/ISO-8859-1/CP1252)
- `utf8-no-bom.txt` (3KB) - UTF-8 without BOM (high confidence via validation)
- `cp1252-vs-iso-8859-1.txt` (3KB) - Ambiguous between CP1252 and ISO-8859-1
- `binary-data.bin` (5KB) - Non-text binary data

**Total**: ~70KB (well under 1MB limit)

### Fixture Governance

**Adding new fixtures** (same process as fulpack):

1. **Naming convention**: `{category}-{description}.{ext}`
   - Categories: `valid`, `invalid`, `normalization`, `bom`, `pathological`, `detection`
   - Examples: `invalid-utf8-overlong.bin`, `bom-multiple.txt`, `pathological-combining-marks.txt`

2. **Approval process**:
   - Create fixture locally and test in your library
   - Document purpose and expected behavior in PR description
   - Add fixture to `config/library/fulencode/fixtures/`
   - Add test cases validating fixture behavior
   - Request review from Schema Cartographer before merging to Crucible

3. **Documentation**: Each fixture SHOULD have an accompanying `.txt` description:

```
Fixture: invalid-utf8-overlong.bin
Purpose: Test overlong UTF-8 encoding detection (security vulnerability)
Expected: decode() in strict mode MUST raise INVALID_UTF8 error
Contents:
  - Overlong encoding of NULL (0xC0 0x80 instead of 0x00)
  - Overlong encoding of ASCII "A" (0xC1 0x81 instead of 0x41)
  - Overlong 3-byte encoding (0xE0 0x80 0x80)
Behavior:
  - detect() should detect as invalid UTF-8
  - decode(strict) MUST error with INVALID_UTF8, subcode="overlong_encoding"
  - decode(replace) MUST replace each overlong with single U+FFFD
```

4. **Size limits**:
   - Valid/invalid encodings: <5KB each
   - Normalization fixtures: <5KB each
   - Pathological fixtures: <50KB each (if larger, justify in PR)
   - Total: <1MB across all fulencode fixtures

5. **Cross-language parity**:
   - New fixtures MUST work identically across all implementations
   - Include in parity test suites
   - Document language-specific behavior if unavoidable

**Note**: For large-scale encoding test suites, use the separate fixtures repository (planned for next week). Crucible fixtures are for core functionality and portability testing only.

---

## Schema References

### Taxonomy Schemas

**Explicit paths to taxonomy definitions**:

- `schemas/taxonomy/library/fulencode/encoding-families/v1.0.0/families.yaml`
  - Defines all supported encodings with metadata (alphabet size, padding, use cases)
  - Binary-to-text: base64, base64url, base64_raw, base32, base32hex, hex
  - Character encodings: utf-8, utf-16le, utf-16be, iso-8859-1, cp1252, ascii

- `schemas/taxonomy/library/fulencode/normalization-profiles/v1.0.0/profiles.yaml`
  - Defines normalization forms (NFC, NFD, NFKC, NFKD)
  - Custom profiles (safe_identifiers, search_optimized)

- `schemas/taxonomy/library/fulencode/detection-confidence/v1.0.0/levels.yaml`
  - Confidence thresholds (high: ≥90%, medium: 50-89%, low: <50%)
  - Detection sources (BOM, validation, statistical, heuristic)

### Data Structure Schemas

**Operation input/output schemas**:

- `schemas/library/fulencode/v1.0.0/encode-options.schema.json`
  - Options for encode() operation
  - Fields: padding, case, line_length, max_encoded_size, compute_checksum, on_error

- `schemas/library/fulencode/v1.0.0/decode-options.schema.json`
  - Options for decode() operation
  - Fields: verify_checksum, max_decoded_size, on_error, fallback_formats, ignore_whitespace

- `schemas/library/fulencode/v1.0.0/detect-options.schema.json`
  - Options for detect() operation
  - Fields: candidate_encodings, max_sample_size, min_confidence, check_bom, recognize_multibase

- `schemas/library/fulencode/v1.0.0/normalize-options.schema.json`
  - Options for normalize() operation
  - Fields: warn_semantic_change, reject_zero_width, max_combining_marks, custom transformations

- `schemas/library/fulencode/v1.0.0/bom-options.schema.json`
  - Options for BOM operations (correct_bom, etc.)
  - Fields: expected_encoding, prefer_no_bom, add_if_missing, on_mismatch

**Result schemas**:

- `schemas/library/fulencode/v1.0.0/encoding-result.schema.json`
  - Result structure for encode() operation
  - Fields: data, format, input_size, output_size, checksum, warnings

- `schemas/library/fulencode/v1.0.0/decoding-result.schema.json`
  - Result structure for decode() operation
  - Fields: data, format, input_size, output_size, checksum_verified, corrections_applied, warnings

- `schemas/library/fulencode/v1.0.0/detection-result.schema.json`
  - Result structure for detect() operation
  - Fields: encoding, confidence, confidence_level, bom_detected, candidates, sample_size

- `schemas/library/fulencode/v1.0.0/normalization-result.schema.json`
  - Result structure for normalize() operation
  - Fields: text, profile, input_length, output_length, semantic_changes, warnings

- `schemas/library/fulencode/v1.0.0/bom-result.schema.json`
  - Result structure for BOM operations
  - Fields: bom_type, byte_length, encoding_implied

**Supporting schemas**:

- `schemas/library/fulencode/v1.0.0/detection-candidate.schema.json`
  - Individual detection candidate
  - Fields: encoding, confidence, reason

- `schemas/library/fulencode/v1.0.0/semantic-change.schema.json`
  - Semantic change descriptor for normalization
  - Fields: position, original, normalized, reason

- `schemas/library/fulencode/v1.0.0/fulencode-error.schema.json`
  - Error envelope structure
  - Fields: code, message, operation, input_format, output_format, details

- `schemas/library/fulencode/v1.0.0/fulencode-config.schema.json`
  - Module configuration schema
  - Fields: limits (max sizes, ratios), defaults (error handling), detection settings

---

## Implementation Guidance

### Language-Specific Notes

**Go** (`github.com/fulmenhq/gofulmen/fulencode`):

**Standard library usage**:

- `encoding/base64`, `encoding/hex` for binary-to-text encodings
- `unicode/utf8`, `unicode/utf16` for UTF validation
- `golang.org/x/text/unicode/norm` for normalization
- `golang.org/x/text/encoding` for legacy encodings (Specialized tier)

**Error handling**:

- Return errors as `error` type with wrapping
- Provide typed error structs implementing `error` interface:

  ```go
  type FulencodeError struct {
      Code      string
      Message   string
      Operation string
      Details   map[string]interface{}
  }

  func (e *FulencodeError) Error() string {
      return fmt.Sprintf("fulencode.%s: %s (code: %s)", e.Operation, e.Message, e.Code)
  }
  ```

**Enums from taxonomy**:

- Generate Go constants/types from YAML via codegen

  ```go
  type EncodingFormat string

  const (
      BASE64     EncodingFormat = "base64"
      BASE64URL  EncodingFormat = "base64url"
      HEX        EncodingFormat = "hex"
      // ... etc
  )
  ```

**Example implementation pattern**:

```go
package fulencode

import (
    "encoding/base64"
    "fmt"
)

func Encode(data []byte, format EncodingFormat, opts *EncodeOptions) (*EncodingResult, error) {
    if opts == nil {
        opts = &EncodeOptions{}  // Use defaults
    }

    // Security: Check size limit
    if opts.MaxEncodedSize == 0 {
        opts.MaxEncodedSize = DefaultMaxEncodedSize  // 500MB
    }

    var encoded string
    switch format {
    case BASE64:
        if opts.Padding == nil || *opts.Padding {
            encoded = base64.StdEncoding.EncodeToString(data)
        } else {
            encoded = base64.RawStdEncoding.EncodeToString(data)
        }
    case BASE64URL:
        if opts.Padding == nil || *opts.Padding {
            encoded = base64.URLEncoding.EncodeToString(data)
        } else {
            encoded = base64.RawURLEncoding.EncodeToString(data)
        }
    case HEX:
        encoded = hex.EncodeToString(data)
        if opts.Case != nil && *opts.Case == "upper" {
            encoded = strings.ToUpper(encoded)
        }
    default:
        return nil, &FulencodeError{
            Code:      "UNSUPPORTED_FORMAT",
            Message:   fmt.Sprintf("format %s not supported", format),
            Operation: "encode",
        }
    }

    // Check output size
    if len(encoded) > opts.MaxEncodedSize {
        return nil, &FulencodeError{
            Code:      "BUFFER_OVERFLOW",
            Message:   "encoded output exceeds max size",
            Operation: "encode",
            Details: map[string]interface{}{
                "actual_size": len(encoded),
                "max_size":    opts.MaxEncodedSize,
            },
        }
    }

    result := &EncodingResult{
        Data:       encoded,
        Format:     string(format),
        InputSize:  len(data),
        OutputSize: len(encoded),
    }

    // Optional: Compute checksum via fulhash
    if opts.ComputeChecksum != "" {
        checksum, err := fulhash.Compute([]byte(encoded), opts.ComputeChecksum)
        if err == nil {
            result.Checksum = checksum
            result.ChecksumAlgorithm = opts.ComputeChecksum
        }
    }

    return result, nil
}
```

---

**Python** (`pyfulmen.fulencode`):

**Standard library usage**:

- `base64`, `binascii` for binary-to-text encodings
- `codecs` module for character encoding detection and transcoding
- `unicodedata` for normalization

**Error handling**:

- Raise exceptions for errors
- Provide custom exception classes inheriting from `Exception`:
  ```python
  class FulencodeError(Exception):
      def __init__(self, code, message, operation, input_format=None, output_format=None, details=None):
          self.code = code
          self.message = message
          self.operation = operation
          self.input_format = input_format
          self.output_format = output_format
          self.details = details or {}
          super().__init__(f"fulencode.{operation}: {message} (code: {code})")
  ```

**Enums from taxonomy**:

- Generate Python Enums from YAML via codegen

  ```python
  from enum import Enum

  class EncodingFormat(str, Enum):
      BASE64 = "base64"
      BASE64URL = "base64url"
      HEX = "hex"
      # ... etc
  ```

**Type hints**:

- Use `TypedDict` for options (Python 3.8+)

  ```python
  from typing import TypedDict, Optional

  class EncodeOptions(TypedDict, total=False):
      padding: bool
      case: str  # "upper" | "lower"
      line_length: Optional[int]
      max_encoded_size: int
      compute_checksum: Optional[str]
  ```

**Example implementation pattern**:

```python
import base64
import binascii
from typing import Optional

from pyfulmen.fulencode.types import EncodingFormat, EncodeOptions, EncodingResult
from pyfulmen.fulencode.errors import FulencodeError

DEFAULT_MAX_ENCODED_SIZE = 500 * 1024 * 1024  # 500MB

def encode(
    data: bytes,
    format: EncodingFormat,
    options: Optional[EncodeOptions] = None
) -> EncodingResult:
    """
    Encode binary data to text format.

    Args:
        data: Raw bytes to encode
        format: Target encoding format
        options: Optional encoding options

    Returns:
        EncodingResult with encoded data and metadata

    Raises:
        FulencodeError: If encoding fails or exceeds limits
    """
    if options is None:
        options = {}

    # Security: Check size limit
    max_size = options.get("max_encoded_size", DEFAULT_MAX_ENCODED_SIZE)

    # Encode based on format
    if format == EncodingFormat.BASE64:
        if options.get("padding", True):
            encoded = base64.b64encode(data).decode("ascii")
        else:
            encoded = base64.b64encode(data).rstrip(b"=").decode("ascii")

    elif format == EncodingFormat.BASE64URL:
        if options.get("padding", True):
            encoded = base64.urlsafe_b64encode(data).decode("ascii")
        else:
            encoded = base64.urlsafe_b64encode(data).rstrip(b"=").decode("ascii")

    elif format == EncodingFormat.HEX:
        encoded = binascii.hexlify(data).decode("ascii")
        if options.get("case") == "upper":
            encoded = encoded.upper()

    else:
        raise FulencodeError(
            code="UNSUPPORTED_FORMAT",
            message=f"format {format} not supported",
            operation="encode",
            output_format=format
        )

    # Check output size
    if len(encoded) > max_size:
        raise FulencodeError(
            code="BUFFER_OVERFLOW",
            message="encoded output exceeds max size",
            operation="encode",
            details={
                "actual_size": len(encoded),
                "max_size": max_size,
            }
        )

    result = EncodingResult(
        data=encoded,
        format=format,
        input_size=len(data),
        output_size=len(encoded),
        warnings=[]
    )

    # Optional: Compute checksum via fulhash
    if checksum_alg := options.get("compute_checksum"):
        from pyfulmen import fulhash
        result.checksum = fulhash.compute(encoded.encode("ascii"), checksum_alg)
        result.checksum_algorithm = checksum_alg

    return result
```

**Package extras for specialized encodings**:

```python
# pyproject.toml
[project.optional-dependencies]
base58 = ["base58>=2.1.0"]
legacy = ["chardet>=5.0.0"]
advanced = ["charset-normalizer>=3.0.0"]

# Install with extras
pip install pyfulmen[base58,legacy]
```

---

**TypeScript** (`@fulmenhq/tsfulmen/fulencode`):

**Node.js Buffer usage**:

- Use `Buffer.from()` and `Buffer.toString()` for encodings
- Polyfill `TextEncoder`/`TextDecoder` for browser compatibility

**Error handling**:

- Throw `Error` instances for errors
- Provide custom error classes:
  ```typescript
  export class FulencodeError extends Error {
    constructor(
      public code: string,
      message: string,
      public operation: string,
      public details?: Record<string, any>,
    ) {
      super(`fulencode.${operation}: ${message} (code: ${code})`);
      this.name = "FulencodeError";
    }
  }
  ```

**Enums from taxonomy**:

- Generate TypeScript enums from YAML via codegen
  ```typescript
  export enum EncodingFormat {
    BASE64 = "base64",
    BASE64URL = "base64url",
    HEX = "hex",
    // ... etc
  }
  ```

**Type safety**:

- Use Zod schemas for runtime validation (optional)

  ```typescript
  import { z } from "zod";

  const EncodeOptionsSchema = z.object({
    padding: z.boolean().optional(),
    case: z.enum(["upper", "lower"]).optional(),
    lineLength: z.number().optional(),
    maxEncodedSize: z.number().optional(),
    computeChecksum: z.string().optional(),
  });

  export type EncodeOptions = z.infer<typeof EncodeOptionsSchema>;
  ```

**Example implementation pattern**:

```typescript
import { FulencodeError } from "./errors";
import { EncodingFormat, EncodeOptions, EncodingResult } from "./types";

const DEFAULT_MAX_ENCODED_SIZE = 500 * 1024 * 1024; // 500MB

export function encode(
  data: Uint8Array | Buffer,
  format: EncodingFormat,
  options: EncodeOptions = {},
): EncodingResult {
  // Security: Check size limit
  const maxSize = options.maxEncodedSize ?? DEFAULT_MAX_ENCODED_SIZE;

  // Convert to Buffer if Uint8Array
  const buffer = Buffer.from(data);

  let encoded: string;

  switch (format) {
    case EncodingFormat.BASE64: {
      encoded = buffer.toString("base64");
      if (options.padding === false) {
        encoded = encoded.replace(/=+$/, ""); // Remove padding
      }
      break;
    }

    case EncodingFormat.BASE64URL: {
      encoded = buffer.toString("base64url");
      if (options.padding === false) {
        encoded = encoded.replace(/=+$/, "");
      }
      break;
    }

    case EncodingFormat.HEX: {
      encoded = buffer.toString("hex");
      if (options.case === "upper") {
        encoded = encoded.toUpperCase();
      }
      break;
    }

    default:
      throw new FulencodeError(
        "UNSUPPORTED_FORMAT",
        `format ${format} not supported`,
        "encode",
        { format },
      );
  }

  // Check output size
  if (encoded.length > maxSize) {
    throw new FulencodeError(
      "BUFFER_OVERFLOW",
      "encoded output exceeds max size",
      "encode",
      {
        actualSize: encoded.length,
        maxSize,
      },
    );
  }

  const result: EncodingResult = {
    data: encoded,
    format,
    inputSize: buffer.length,
    outputSize: encoded.length,
    warnings: [],
  };

  // Optional: Compute checksum via fulhash
  if (options.computeChecksum) {
    const fulhash = require("@fulmenhq/tsfulmen/fulhash");
    result.checksum = fulhash.compute(
      Buffer.from(encoded, "utf8"),
      options.computeChecksum,
    );
    result.checksumAlgorithm = options.computeChecksum;
  }

  return result;
}
```

---

## Integration Specifications

Fulencode is designed to integrate seamlessly with other Fulmen helper modules. This section provides concrete examples of cross-module usage patterns.

### Integration Architecture

```
┌─────────────┐
│   Nimbus    │  Cloud storage operations
│ (Future)    │  → download/upload streams
└──────┬──────┘
       │
       ↓ (Base64 decode, BOM correction)
┌─────────────┐
│  Fulencode  │  Encoding/decoding operations
│  (v0.2.12)  │  → normalize text, validate UTF
└──────┬──────┘
       │
       ↓ (File name normalization)
┌─────────────┐
│   Fulpack   │  Archive operations
│ (v0.2.11)   │  → create/extract archives
└──────┬──────┘
       │
       ↓ (Glob patterns from normalized manifests)
┌─────────────┐
│ Pathfinder  │  File discovery
│  (v1.0.0)   │  → find files in archives/fs
└─────────────┘
```

**Integration dependencies**:

- Fulencode → Fulhash (checksum verification)
- Fulpack → Fulencode (file name normalization, metadata transcoding)
- Nimbus → Fulencode (cloud payload encoding, BOM handling)
- Pathfinder → Fulencode (manifest encoding detection)

### Fulpack Integration

#### Use Case 1: Archive Creation with UTF-8 Normalization

**Problem**: Archive file names from mixed sources (Windows, macOS, Linux) may have different Unicode normalizations, causing extraction issues.

**Solution**: Normalize all file names to NFC before archiving.

```python
from pyfulmen import fulpack, fulencode
from pyfulmen.fulpack import ArchiveFormat
import os

def create_normalized_archive(source_paths: list[str], output: str) -> fulpack.ArchiveInfo:
    """
    Create archive with NFC-normalized file names for cross-platform compatibility.

    Args:
        source_paths: Files/directories to archive
        output: Output archive path

    Returns:
        Archive metadata with entry count and sizes
    """
    # Step 1: Detect and normalize file names
    normalized_entries = []

    for path in source_paths:
        # Get filename
        filename = os.path.basename(path)

        # Detect encoding if needed (for non-UTF-8 filesystems)
        filename_bytes = filename.encode('utf-8', errors='surrogateescape')
        detected = fulencode.detect(filename_bytes, options={
            "check_bom": False,  # Filenames don't have BOMs
            "min_confidence": 0.7
        })

        if detected.encoding != "utf-8" or detected.confidence < 0.9:
            # Transcode to UTF-8
            filename_clean = fulencode.decode(
                filename_bytes,
                format=detected.encoding,
                options={"on_error": "replace"}  # Replace invalid chars
            ).data.decode("utf-8")
        else:
            filename_clean = filename

        # Step 2: Normalize to NFC (canonical composition)
        filename_normalized = fulencode.normalize(
            filename_clean,
            profile="nfc",
            options={"warn_semantic_change": False}  # NFC is semantic-preserving
        ).text

        normalized_entries.append((path, filename_normalized))

        if filename != filename_normalized:
            print(f"Normalized: '{filename}' → '{filename_normalized}'")

    # Step 3: Create archive with normalized names
    info = fulpack.create(
        source=source_paths,
        output=output,
        format=ArchiveFormat.TAR_GZ,
        options={
            "exclude_patterns": ["**/.DS_Store", "**/__pycache__"],
            "preserve_permissions": True,
            "checksum_algorithm": "sha256"
        }
    )

    print(f"✓ Created {output}: {info.entry_count} entries, {info.compression_ratio:.1f}x compression")
    return info

# Usage
create_normalized_archive(
    source_paths=["docs/", "src/", "README.md"],
    output="release.tar.gz"
)
```

#### Use Case 2: Extract Archive with BOM-Aware Text Processing

**Problem**: Extracted text files may have BOMs that break parsers (CSV readers, XML parsers).

**Solution**: Use fulencode BOM correction after extraction.

```python
import io
import csv
import xml.etree.ElementTree as ET
from pathlib import Path

def extract_and_process_archive(archive_path: str, dest: str):
    """
    Extract archive and process text files with automatic BOM handling.
    """
    # Step 1: Extract archive
    result = fulpack.extract(
        archive=archive_path,
        destination=dest,
        options={"verify_checksums": True}
    )

    print(f"Extracted {result.extracted_count} files")

    # Step 2: Process CSV files with BOM correction
    for csv_file in Path(dest).glob("**/*.csv"):
        print(f"Processing: {csv_file}")

        # Correct BOM issues automatically
        with fulencode.correct_bom_context(
            csv_file,
            options={
                "expected_encoding": "utf-8",
                "prefer_no_bom": True,      # CSV parsers prefer no BOM
                "on_mismatch": "fix"        # Auto-fix mismatched BOMs
            }
        ) as corrected_data:
            # Parse CSV with clean data (no BOM surprises)
            df = csv.DictReader(io.StringIO(corrected_data.decode("utf-8")))
            for row in df:
                process_row(row)

    # Step 3: Process XML files with BOM correction
    for xml_file in Path(dest).glob("**/*.xml"):
        with fulencode.correct_bom_context(xml_file) as corrected_xml:
            # Parse XML without BOM issues
            tree = ET.parse(io.BytesIO(corrected_xml))
            process_xml(tree)

# Usage
extract_and_process_archive("data.tar.gz", "/tmp/extracted")
```

#### Use Case 3: Archive Metadata Transcoding

**Problem**: Legacy archives may have non-UTF-8 metadata (file comments, extra fields).

**Solution**: Detect and transcode metadata during extraction.

```go
package main

import (
    "github.com/fulmenhq/gofulmen/fulpack"
    "github.com/fulmenhq/gofulmen/fulencode"
)

func extractWithTranscoding(archivePath, destDir string) error {
    // Step 1: Scan archive to analyze metadata
    entries, err := fulpack.Scan(archivePath, &fulpack.ScanOptions{
        IncludeMetadata: true,
    })
    if err != nil {
        return err
    }

    // Step 2: Detect metadata encoding (check for non-UTF-8)
    var needsTranscoding bool
    var detectedEncoding string

    for _, entry := range entries {
        if entry.Comment != "" {
            // Detect encoding of comment field
            result, err := fulencode.Detect(
                []byte(entry.Comment),
                &fulencode.DetectOptions{
                    MinConfidence: 0.5,
                },
            )
            if err == nil && result.Encoding != "utf-8" {
                needsTranscoding = true
                detectedEncoding = result.Encoding
                break
            }
        }
    }

    // Step 3: Extract with transcoding if needed
    if needsTranscoding {
        log.Printf("Detected %s metadata, transcoding to UTF-8", detectedEncoding)

        // Custom extraction with transcoding
        for _, entry := range entries {
            // Transcode comment to UTF-8
            if entry.Comment != "" {
                decoded, err := fulencode.Decode(
                    []byte(entry.Comment),
                    fulencode.EncodingFormat(detectedEncoding),
                    &fulencode.DecodeOptions{
                        OnError: fulencode.Replace,  // Replace invalid chars
                    },
                )
                if err == nil {
                    entry.Comment = string(decoded.Data)
                }
            }

            // Extract file with updated metadata
            extractEntry(entry, destDir)
        }
    } else {
        // Standard extraction (metadata already UTF-8)
        _, err := fulpack.Extract(archivePath, destDir, nil)
        return err
    }

    return nil
}
```

### Nimbus Integration (Future Module)

**Note**: Nimbus is a planned module for cloud storage operations. These examples show anticipated integration patterns.

#### Use Case 1: Download and Decode Base64 Cloud Objects

**Problem**: S3/GCS objects may be Base64-encoded to work around binary restrictions.

**Solution**: Stream download → decode → process pipeline.

```python
from pyfulmen import nimbus, fulencode
from pyfulmen.fulencode import EncodingFormat

async def process_encoded_object(bucket: str, key: str, output_path: str):
    """
    Download Base64-encoded object from cloud storage and decode streaming.

    Avoids loading entire object into memory.
    """
    # Stream download from cloud
    async with nimbus.download_stream(f"s3://{bucket}/{key}") as s3_stream:
        # Decode Base64 streaming
        async with fulencode.decode_stream(
            s3_stream,
            format=EncodingFormat.BASE64,
            options={
                "verify_checksum": True,        # Verify if checksum embedded
                "max_decoded_size": 100*1024*1024,  # 100MB limit
                "ignore_whitespace": True       # Skip newlines in Base64
            }
        ) as decoder:
            # Write decoded data to file
            with open(output_path, "wb") as f:
                async for chunk in decoder:
                    f.write(chunk.data)

    print(f"✓ Downloaded and decoded: {output_path}")

# Usage
await process_encoded_object("my-bucket", "data.tar.gz.b64", "/tmp/data.tar.gz")
```

#### Use Case 2: Upload with Encoding Detection and Normalization

**Problem**: Uploading text files with unknown encoding or incorrect BOMs.

**Solution**: Detect → normalize → upload pipeline.

```typescript
import { nimbus } from "@fulmenhq/tsfulmen/nimbus";
import { fulencode, EncodingFormat } from "@fulmenhq/tsfulmen/fulencode";
import * as fs from "fs";

async function uploadNormalizedText(
  localPath: string,
  bucket: string,
  key: string,
): Promise<void> {
  // Step 1: Read and detect encoding
  const fileBytes = fs.readFileSync(localPath);

  const detected = await fulencode.detect(fileBytes, {
    checkBom: true,
    minConfidence: 0.7,
  });

  console.log(
    `Detected encoding: ${detected.encoding} (${detected.confidence.toFixed(2)})`,
  );

  // Step 2: Decode to UTF-8 if needed
  let textContent: string;

  if (detected.encoding !== "utf-8") {
    const decoded = await fulencode.decode(
      fileBytes,
      EncodingFormat[detected.encoding.toUpperCase().replace("-", "")],
      {
        onError: "replace", // Replace invalid characters
        fallbackFormats: ["cp1252", "iso-8859-1"], // Try these if detection wrong
      },
    );

    textContent = new TextDecoder("utf-8").decode(decoded.data);
    console.log(`✓ Transcoded from ${detected.encoding} to UTF-8`);
  } else {
    textContent = new TextDecoder("utf-8").decode(fileBytes);
  }

  // Step 3: Normalize Unicode (NFC for storage)
  const normalized = await fulencode.normalize(textContent, "nfc", {
    warnSemanticChange: true,
  });

  if (normalized.transformationsApplied.length > 0) {
    console.log(
      `✓ Applied normalizations: ${normalized.transformationsApplied.join(", ")}`,
    );
  }

  // Step 4: Remove BOM if present (cloud storage prefers no BOM)
  const cleanBytes = fulencode.removeBom(
    Buffer.from(normalized.text, "utf-8"),
    "utf-8",
  );

  // Step 5: Upload to cloud
  await nimbus.upload(`s3://${bucket}/${key}`, cleanBytes, {
    contentType: "text/plain; charset=utf-8",
    metadata: {
      originalEncoding: detected.encoding,
      normalized: "nfc",
      bomRemoved: detected.bomDetected ? "true" : "false",
    },
  });

  console.log(`✓ Uploaded normalized text to s3://${bucket}/${key}`);
}

// Usage
await uploadNormalizedText(
  "legacy-data.txt",
  "my-bucket",
  "data/normalized.txt",
);
```

#### Use Case 3: Batch Encoding Detection for Cloud Migration

**Problem**: Migrating thousands of legacy files to cloud with unknown encodings.

**Solution**: Parallel detection → report → selective transcode.

```go
package main

import (
    "context"
    "sync"

    "github.com/fulmenhq/gofulmen/nimbus"
    "github.com/fulmenhq/gofulmen/fulencode"
)

type EncodingReport struct {
    Path       string
    Encoding   string
    Confidence float64
    NeedsFixing bool
}

func analyzeCloudFiles(ctx context.Context, bucket string, prefix string) ([]EncodingReport, error) {
    // List all files
    objects, err := nimbus.List(ctx, fmt.Sprintf("s3://%s/%s", bucket, prefix))
    if err != nil {
        return nil, err
    }

    // Parallel detection (bounded concurrency)
    const workers = 10
    results := make(chan EncodingReport, len(objects))
    sem := make(chan struct{}, workers)
    var wg sync.WaitGroup

    for _, obj := range objects {
        wg.Add(1)
        go func(objKey string) {
            defer wg.Done()
            sem <- struct{}{}        // Acquire
            defer func() { <-sem }() // Release

            // Download sample (first 8KB)
            sample, err := nimbus.DownloadRange(ctx,
                fmt.Sprintf("s3://%s/%s", bucket, objKey),
                0, 8192)
            if err != nil {
                return
            }

            // Detect encoding
            detection, err := fulencode.Detect(sample, &fulencode.DetectOptions{
                CheckBOM: true,
                MinConfidence: 0.5,
            })
            if err != nil {
                return
            }

            // Flag files needing transcoding
            needsFix := detection.Encoding != "utf-8" ||
                       detection.Confidence < 0.9 ||
                       detection.BOMDetected

            results <- EncodingReport{
                Path:       objKey,
                Encoding:   detection.Encoding,
                Confidence: detection.Confidence,
                NeedsFixing: needsFix,
            }
        }(obj.Key)
    }

    // Wait and collect results
    go func() {
        wg.Wait()
        close(results)
    }()

    var reports []EncodingReport
    for report := range results {
        reports = append(reports, report)
    }

    return reports, nil
}

func main() {
    ctx := context.Background()

    // Analyze all files
    reports, err := analyzeCloudFiles(ctx, "legacy-bucket", "documents/")
    if err != nil {
        log.Fatal(err)
    }

    // Generate migration report
    fmt.Printf("Analyzed %d files\n", len(reports))

    var needsFixCount int
    encodingDist := make(map[string]int)

    for _, report := range reports {
        encodingDist[report.Encoding]++
        if report.NeedsFixing {
            needsFixCount++
            fmt.Printf("⚠️  %s: %s (%.0f%% confidence)\n",
                report.Path, report.Encoding, report.Confidence*100)
        }
    }

    fmt.Printf("\nEncoding distribution:\n")
    for enc, count := range encodingDist {
        fmt.Printf("  %s: %d files\n", enc, count)
    }

    fmt.Printf("\nFiles needing transcoding: %d (%.1f%%)\n",
        needsFixCount, float64(needsFixCount)/float64(len(reports))*100)
}
```

### Pathfinder Integration

#### Use Case: Encoding-Aware Manifest Processing

**Problem**: `.gitignore`-style manifest files may have non-UTF-8 encodings or incorrect normalizations.

**Solution**: Detect → decode → normalize → use with Pathfinder.

```python
from pyfulmen import pathfinder, fulencode
import yaml

def load_manifest_with_encoding_detection(manifest_path: str) -> dict:
    """
    Load Pathfinder manifest with automatic encoding detection and normalization.

    Handles:
    - Non-UTF-8 encoded manifests (legacy systems)
    - BOM issues in YAML files
    - Unicode normalization mismatches
    """
    # Step 1: Read manifest bytes
    with open(manifest_path, "rb") as f:
        manifest_bytes = f.read()

    # Step 2: Detect encoding
    detected = fulencode.detect(
        manifest_bytes,
        options={
            "check_bom": True,
            "min_confidence": 0.6
        }
    )

    print(f"Detected manifest encoding: {detected.encoding} ({detected.confidence:.2%} confidence)")

    # Step 3: Decode with fallback
    decoded = fulencode.decode(
        manifest_bytes,
        format=detected.encoding,
        options={
            "on_error": "fallback",
            "fallback_formats": ["utf-8", "cp1252", "iso-8859-1"]
        }
    )

    manifest_text = decoded.data.decode("utf-8")

    # Step 4: Normalize glob patterns (NFC for consistent matching)
    manifest_dict = yaml.safe_load(manifest_text)

    if "patterns" in manifest_dict:
        normalized_patterns = []
        for pattern in manifest_dict["patterns"]:
            normalized = fulencode.normalize(
                pattern,
                profile="nfc",
                options={"warn_semantic_change": False}
            )
            normalized_patterns.append(normalized.text)

        manifest_dict["patterns"] = normalized_patterns

    return manifest_dict

# Usage
manifest = load_manifest_with_encoding_detection(".pathfinder.yaml")

# Use normalized patterns with Pathfinder
results = pathfinder.find(
    source="/data",
    patterns=manifest["patterns"],
    options=manifest.get("options", {})
)

for result in results:
    print(f"Found: {result.path}")
```

---

## Telemetry Specification (Enhanced)

**Module**: Uses Core-tier `telemetry` module for metrics instrumentation

Fulencode telemetry is implemented in waves to avoid blocking early helper library adoption.

- Wave 1 metrics are required when telemetry is enabled.
- Wave 2 metrics are recommended and should be added once implementations stabilize.
- Wave 3 metrics are optional and typically added after error taxonomy/label conventions settle.

All fulencode implementations MUST support telemetry hooks and MUST NOT error when telemetry is disabled/unconfigured.

### Core Operation Metrics

**Operation Duration** (Histogram):

```
fulencode_operation_duration_seconds
Tags: operation={encode|decode|detect|normalize|bom}, format={base64|utf-8|etc}, result={success|error}
Buckets: Use default seconds buckets from `config/taxonomy/metrics.yaml` (unit `s`).
Purpose: Track operation latency for performance monitoring
```

**Operation Count** (Counter):

```
fulencode_operation_total
Tags: operation={encode|decode|detect|normalize|bom}, format={...}, result={success|error}
Purpose: Track operation volume and success/failure rates
```

**Example usage**:

```python
from pyfulmen import telemetry, fulencode

with telemetry.histogram("fulencode_operation_duration_seconds",
                          labels={"operation": "decode", "format": "base64", "result": "success"}):
    result = fulencode.decode(data, "base64", options)
    telemetry.counter("fulencode_operation_total",
                      labels={"operation": "decode", "format": "base64", "result": "success"})
```

### Data Volume Metrics

**Bytes Processed** (Histogram):

```
fulencode_bytes_processed_total
Tags: operation={encode|decode}, direction={in|out}, format={base64|utf-8|etc}, result={success|error}
Buckets: Use default bytes buckets from `config/taxonomy/metrics.yaml` (unit `bytes`).
Purpose: Track payload sizes for capacity planning
```

**Expansion Ratio** (Histogram):

```
fulencode_expansion_ratio_percent
Tags: operation={encode|decode}, format={...}, result={success|error}
Buckets: [0.1, 0.5, 1, 2, 5, 10, 20, 50, 100]
Purpose: Monitor encoding efficiency and detect potential bombs
```

### Detection Metrics

**Detection Outcomes** (Counter):

```
fulencode_detect_result_total
Tags: encoding={utf-8|utf-16le|iso-8859-1|unknown}, confidence={low|medium|high}, result={success|error}
Purpose: Track detection accuracy and encoding distribution
```

**Detection Duration** (Histogram):

```
fulencode_detect_duration_seconds
Tags: sample_size_bucket={<1kb|1-10kb|10-100kb|>100kb}, result={success|error}
Buckets: Use default seconds buckets from `config/taxonomy/metrics.yaml` (unit `s`).
Purpose: Monitor detection performance vs sample size
```

### Normalization Metrics

**Normalization Actions** (Counter):

```
fulencode_normalize_total
Tags: profile={nfc|nfd|nfkc|nfkd|custom}, result={success|error}
Purpose: Track normalization usage and errors
```

**Semantic Changes Detected** (Counter):

```
fulencode_normalize_semantic_changes_total
Tags: profile={nfkc|nfkd}, change_type={ligatures|superscripts_subscripts|compatibility|other}
Purpose: Track NFKC/NFKD semantic changes for security monitoring
```

### Security Metrics

**Security Violations Detected** (Counter):

```
fulencode_security_violations_total
Tags: type={invalid_utf8|invalid_utf16|encoding_bomb|excessive_combining|zero_width|bidi_controls|bom_mismatch}, operation={...}
Purpose: Track security threats for alerting and audit logging
Alert threshold: Any non-zero value in production
```

**Error Corrections Applied** (Counter):

```
fulencode_corrections_total
Tags: error_mode={replace|fallback|ignore}, error_type={invalid_utf8|invalid_utf16|invalid_encoding|etc}
Purpose: Track how often correction modes are triggered (may indicate data quality issues)
```

### BOM Handling Metrics

**BOM Operations** (Counter):

```
fulencode_bom_operations_total
Tags: operation={detect|remove|add|validate|correct}, bom_type={utf-8|utf-16le|utf-16be|utf-32le|utf-32be|none}, result={success|error}
Purpose: Track BOM handling frequency
```

**BOM Mismatches** (Counter):

```
fulencode_bom_mismatches_total
Tags: detected_bom={...}, expected_encoding={...}, action={error|fix|ignore}
Purpose: Security monitoring for BOM injection attempts
```

### Telemetry Implementation Checklist

**Use this checklist to validate metric coverage during implementation** (per EA Steward feedback):

| Metric                                       | Must Emit From                  | Labels Required                               | When to Emit                                      |
| -------------------------------------------- | ------------------------------- | --------------------------------------------- | ------------------------------------------------- |
| `fulencode_operation_duration_seconds`       | All operations                  | `operation`, `format`, `result`               | Start of operation (use timer/context manager)    |
| `fulencode_operation_total`                  | All operations                  | `operation`, `format`, `result`               | After operation completes (success or error)      |
| `fulencode_bytes_processed_total`            | `encode`, `decode`              | `operation`, `direction`, `format`, `result`  | After processing input/output bytes               |
| `fulencode_expansion_ratio_percent`          | `encode`, `decode`              | `operation`, `format`, `result`               | After operation (output_size / input_size)        |
| `fulencode_detect_result_total`              | `detect`                        | `encoding`, `confidence`, `result`            | After detection completes                         |
| `fulencode_detect_duration_seconds`          | `detect`                        | `sample_size_bucket`, `result`                | Start of detection (use timer)                    |
| `fulencode_normalize_total`                  | `normalize`                     | `profile`, `result`                           | After normalization attempt                       |
| `fulencode_normalize_semantic_changes_total` | `normalize` (NFKC/NFKD only)    | `profile`, `change_type`                      | When semantic change detected (ligatures, etc.)   |
| `fulencode_security_violations_total`        | All operations                  | `type`, `operation`                           | **Immediately** when violation detected           |
| `fulencode_corrections_total`                | `encode`, `decode`, `normalize` | `error_mode`, `error_type`                    | When correction applied (replace/fallback/ignore) |
| `fulencode_bom_operations_total`             | `bom_*` functions               | `operation`, `bom_type`, `result`             | After BOM operation                               |
| `fulencode_bom_mismatches_total`             | `bom_validate`, `detect`        | `detected_bom`, `expected_encoding`, `action` | When BOM mismatch detected                        |

**Wave 1 (MVP) Required Metrics** (7 metrics):

- ✅ `fulencode_operation_duration_seconds` - Core performance
- ✅ `fulencode_operation_total` - Core volume
- ✅ `fulencode_bytes_processed_total` - Data volume tracking
- ✅ `fulencode_expansion_ratio_percent` - Bomb detection
- ✅ `fulencode_detect_result_total` - Detection outcomes
- ✅ `fulencode_normalize_total` - Normalization volume
- ✅ `fulencode_security_violations_total` - **CRITICAL** security alerts

**Wave 2 Metrics** (5 metrics):

- ⏸️ `fulencode_detect_duration_seconds` - Detection performance

- ⏸️ `fulencode_normalize_semantic_changes_total` - Advanced normalization
- ⏸️ `fulencode_corrections_total` - Error recovery tracking
- ⏸️ `fulencode_bom_operations_total` - BOM handling volume
- ⏸️ `fulencode_bom_mismatches_total` - BOM security

**Wave 3 Metrics** (optional):

- ⏸️ `fulencode_errors_total` - Error code distribution (emit after error taxonomy + label conventions stabilize)

**Test Validation Pattern**:

```python
# Verify metric emission in tests
def test_encode_emits_telemetry():
    with telemetry_test_context() as metrics:
        fulencode.encode(b"test", "base64")

        # Assert required metrics present
        assert "fulencode_operation_duration_seconds" in metrics
        assert "fulencode_operation_total" in metrics
        assert "fulencode_bytes_processed_total" in metrics
        assert metrics["fulencode_operation_total"].labels["result"] == "success"
```

### Complete Instrumentation Example

```python
from pyfulmen import telemetry, fulencode
from pyfulmen.fulencode import EncodingFormat, FulencodeError
import time

def decode_with_telemetry(data: bytes, format: str, options: dict):
    """Example showing complete telemetry instrumentation."""

    labels = {"operation": "decode", "format": format}
    start_time = time.time()

    try:
        # Perform decode
        result = fulencode.decode(data, format, options)

        # Record success metrics
        duration = time.time() - start_time
        telemetry.histogram("fulencode_operation_duration_seconds",
                            duration,
                            labels={**labels, "result": "success"})
        telemetry.counter("fulencode_operation_total",
                          labels={**labels, "result": "success"})

        # Record data volume
        telemetry.histogram("fulencode_bytes_processed_total",
                            len(data),
                            labels={
                                "operation": "decode",
                                "direction": "in",
                                "format": format,
                                "result": "success",
                            })

        # Record expansion ratio
        if result.output_size > 0:
            ratio = len(data) / result.output_size
            telemetry.histogram("fulencode_expansion_ratio_percent",
                                ratio * 100,
                                labels={"operation": "decode", "format": format, "result": "success"})

        # Record corrections if applied
        if result.corrections_applied > 0:
            error_mode = options.get("on_error", "strict")
            telemetry.counter("fulencode_corrections_total",
                              result.corrections_applied,
                              labels={"error_mode": error_mode, "error_type": "invalid_utf8"})

        return result

    except FulencodeError as e:
        # Record error metrics
        duration = time.time() - start_time
        telemetry.histogram("fulencode_operation_duration_seconds",
                            duration,
                            labels={**labels, "result": "error"})
        telemetry.counter("fulencode_operation_total",
                          labels={**labels, "result": "error"})

        # Record security violations
        if e.code in ["INVALID_UTF8", "INVALID_UTF16", "ENCODING_BOMB",
                      "EXCESSIVE_COMBINING_MARKS", "ZERO_WIDTH_CHARACTER", "BOM_MISMATCH"]:
            telemetry.counter("fulencode_security_violations_total",
                              labels={"type": e.code.lower(), "operation": "decode"})

        raise
```

### Alerting Recommendations

**Critical Alerts** (page immediately):

- `fulencode_security_violations_total{type="encoding_bomb"}` > 0
- `fulencode_security_violations_total{type="bom_mismatch"}` > 5/hour (possible attack)
- `fulencode_operation_duration_seconds` p99 > 5.0 seconds (performance degradation)

**Warning Alerts** (notify on-call):

- `fulencode_security_violations_total{type="invalid_utf8"}` > 100/hour (data quality issue)
- `fulencode_corrections_total` > 1000/hour (many corrections being applied)
- `fulencode_detect_result_total{confidence="low"}` > 50% of detections (ambiguous data)

**Info Alerts** (log only):

- `fulencode_normalize_semantic_changes_total` > 100/day (tracking NFKC usage)
- `fulencode_bom_operations_total{operation="correct"}` > 100/day (BOM issues common)

---

## Version History

- **1.0.0** (2025-11-XX) - Initial specification
  - 6 encoding families (base64 variants, base32 variants, hex) for binary-to-text
  - 6 character encodings (UTF-8, UTF-16LE/BE, ISO-8859-1, CP1252, ASCII) for detection/transcoding
  - 5 operations (encode, decode, detect, normalize, BOM handling)
  - Fulhash integration for checksums
  - Telemetry specification (14 metric types)
  - Security model with UTF-8/UTF-16 validation, normalization attack prevention
  - Configuration schema for limits and error modes
  - Streaming API reserved (implementation deferred to v1.1.0)

---

## Future Enhancements

**Planned for v1.1.0+**:

- **Streaming API implementation** (encode_stream, decode_stream, detect_stream)
- **Advanced detection** (statistical scoring, confidence intervals)
- **Multibase encoding** (currently detection-only, add encoding support)
- **Additional normalization profiles** (locale-specific, domain-specific)

**Deferred to Specialized Modules**:

**Specialized modules enable optional encodings, advanced detection, and niche use cases** beyond Common tier. Module registry will track these as separate packages with override policies.

### Category 1: Specialized Encoding Families

**Purpose**: Additional binary-to-text encodings requiring external dependencies or serving niche use cases (<50% adoption).

**Modules**:

1. **fulencode-base58** (Specialized tier)
   - **Formats**: Base58 (Bitcoin), Base58Check (Bitcoin checksummed)
   - **Use cases**: Cryptocurrency addresses, IPFS content IDs, short URL encodings
   - **Dependencies**: base58 library (external)
   - **Rationale**: Niche format, external dep, cryptocurrency-specific

2. **fulencode-base85** (Specialized tier)
   - **Formats**: Base85 (RFC 1924), Ascii85 (Adobe), Z85 (ZeroMQ)
   - **Use cases**: PDF embedding, ZeroMQ framing, binary-to-text for protocols
   - **Dependencies**: None (algorithmic), but complex implementations
   - **Rationale**: Less common, multiple incompatible variants

3. **fulencode-multibase** (Specialized tier)
   - **Formats**: Multibase encoding (IPFS self-describing encodings)
   - **Use cases**: IPFS content addressing, self-describing data
   - **Dependencies**: multibase library or manual implementation
   - **Rationale**: IPFS-specific, requires prefix encoding/decoding
   - **Note**: v1.0.0 Common tier includes multibase **detection** (recognize prefix), this module adds **encoding**

### Category 2: Legacy Character Encodings

**Purpose**: Legacy and regional character encodings for data migration and international systems.

**Modules**:

4. **fulencode-legacy-cjk** (Specialized tier)
   - **Formats**: Shift-JIS (Japanese), GB2312/GBK (Chinese), Big5 (Traditional Chinese), EUC-KR (Korean)
   - **Use cases**: Legacy Asian text data, database migrations
   - **Dependencies**: Language-specific codec libraries
   - **Rationale**: Regional-specific, complex mappings, external deps

5. **fulencode-legacy-mainframe** (Specialized tier)
   - **Formats**: EBCDIC variants (IBM mainframe), CP037, CP500
   - **Use cases**: Mainframe data interchange, legacy system integration
   - **Dependencies**: ebcdic codecs (may be stdlib in some languages)
   - **Rationale**: Mainframe-specific, rare outside enterprise

6. **fulencode-legacy-european** (Specialized tier)
   - **Formats**: ISO-8859-2 through ISO-8859-15 (regional Latin variants), KOI8-R (Cyrillic)
   - **Use cases**: Legacy European text, email archive migrations
   - **Dependencies**: None (stdlib in most languages)
   - **Rationale**: Less common than CP1252/ISO-8859-1 (Common tier)

### Category 3: Advanced Detection

**Purpose**: ML-based and statistical detection beyond heuristic BOM/validation.

**Modules**:

7. **fulencode-detect-advanced** (Specialized tier)
   - **Capabilities**: Statistical byte frequency analysis, N-gram models, confidence scoring
   - **Algorithms**: Chi-squared test, entropy analysis, language model scoring
   - **Use cases**: Ambiguous text (no BOM, mixed encodings), low-confidence fallback
   - **Dependencies**: None (pure algorithmic)
   - **Rationale**: Complex algorithms, slower performance, not needed for BOM/UTF-8 cases

8. **fulencode-detect-ml** (Specialized tier)
   - **Capabilities**: ML-based detection, chardet wrapper, charset-normalizer integration
   - **Algorithms**: chardet (Python), ICU (Go/C), charset-normalizer (Python)
   - **Use cases**: High-accuracy detection for unknown data, user uploads
   - **Dependencies**: chardet, charset-normalizer, or ICU libraries (external, large)
   - **Rationale**: External ML models, large dependencies, overkill for BOM/validation

**Note**: Advanced detection may benefit from integration with the [Foundry Similarity module](../foundry/similarity.md) for:

- Fuzzy matching of byte patterns against known encoding signatures
- String distance metrics for confidence scoring
- Similarity-based encoding disambiguation when heuristics are ambiguous

See `docs/standards/library/modules/foundry/similarity.md` for available algorithms and integration patterns.

### Module Registry Entries (Example)

**Common tier (fulencode core)**:

```yaml
fulencode:
  tier: common
  description: "Binary-to-text encoding, UTF detection, Unicode normalization"
  formats:
    - base64, base64url, base32, hex
    - utf-8, utf-16le, utf-16be, iso-8859-1, cp1252, ascii
  operations:
    - encode, decode, detect, normalize, bom_handling
```

**Specialized tier (extensions)**:

```yaml
fulencode_base58:
  tier: specialized
  category: encoding_families
  description: "Base58 and Base58Check encoding (Bitcoin, IPFS)"
  parent_module: fulencode
  override_allowed: true
  override_reason_required: ["missing_dependency", "not_implemented"]

fulencode_detect_ml:
  tier: specialized
  category: advanced_detection
  description: "ML-based encoding detection (chardet, charset-normalizer)"
  parent_module: fulencode
  override_allowed: true
  override_reason_required: ["large_dependency", "platform_unavailable"]
```

### Implementation Priorities

**v1.0.0** (Common tier):

- Core binary-to-text encodings (base64, hex, base32)
- UTF-8/UTF-16 detection and validation
- Common legacy (ISO-8859-1, CP1252)
- BOM handling
- NFC/NFD/NFKC/NFKD normalization

**v1.1.0+** (Streaming API):

- Streaming variants of v1.0.0 operations

**v2.0.0+** (Specialized modules):

- fulencode-base58 (high demand from blockchain projects)
- fulencode-multibase (IPFS integration)
- fulencode-detect-advanced (for ambiguous cases)

**Future** (as needed):

- fulencode-legacy-\* (on-demand for specific projects)
- fulencode-detect-ml (if statistical detection insufficient)

---

## Related Standards

- [Fulmen Helper Library Standard](../../../architecture/fulmen-helper-library-standard.md)
- [Canonical Façade Principle](../../../architecture/fulmen-helper-library-standard.md#canonical-façade-principle) - **Why fulencode wraps stdlib encoders**
- [Portable Testing Practices](../../testing/portable-testing-practices.md) - **Required for all implementations**
- [Coding Standards](../../coding/README.md) - Language-specific implementation and testing patterns
- [Module Registry](../../../../config/taxonomy/library/platform-modules/v1.0.0/modules.yaml)
- [FulHash Module Standard](fulhash.md) - For checksum integration
- [Fulpack Module Standard](fulpack.md) - Integration reference (file name normalization)
- [Unicode Security Considerations (TR#36)](https://unicode.org/reports/tr36/) - Normalization attacks, confusables
- [RFC 3629 (UTF-8)](https://tools.ietf.org/html/rfc3629) - UTF-8 specification
- [RFC 4648 (Base encodings)](https://tools.ietf.org/html/rfc4648) - Base64, Base32, Base16

---

**Status**: Draft (pending Schema Cartographer review and mainteiner approval)
**Tier**: Common
**Version**: 1.0.0
**Last Updated**: 2025-11-15

---

## Summary of Spec Completeness

✅ **API Specification** - All 5 operations with full signatures, parameters, examples (Part 1)
✅ **Security Model** - Buffer overflow, UTF-8/UTF-16 validation, normalization attacks, BOM handling (Part 2)
✅ **Error Handling** - Canonical codes, envelope structure, examples (Part 2)
✅ **Configuration Schema** - Limits, defaults, detection settings (Part 2)
✅ **Testing Requirements** - Unit, security, parity, portable testing compliance (Part 2)
✅ **Test Fixtures** - 6 categories, governance process, under 1MB (Part 2)
✅ **Schema References** - Explicit paths to all taxonomy and data schemas (Part 3)
✅ **Implementation Guidance** - Go, Python, TypeScript with examples (Part 3)
✅ **Integration Specs** - Fulpack, Nimbus, Pathfinder with concrete examples (Part 3)
✅ **Telemetry** - 14 metric types, instrumentation examples, alerting (Part 3)
✅ **Version History** - v1.0.0 scope, future enhancements (Part 3)
✅ **Streaming API** - Streaming-ready v1.0.0 schemas, v1.1.0 implementation deferred (Part 2)
✅ **Cross-References** - Extensive links to related standards and principles (Part 3)

**Ready for**: Maintainer review and feedback before moving from .plans/ to docs/standards/library/modules/
