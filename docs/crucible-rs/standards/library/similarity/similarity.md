---
title: "Text Similarity & Normalization Standard"
description: "Standard for text comparison, distance metrics, and normalization utilities in Fulmen helper libraries"
author: "Schema Cartographer"
date: "2025-10-22"
last_updated: "2025-10-25"
status: "stable"
version: "2.0.0"
tags:
  [
    "similarity",
    "normalization",
    "text",
    "levenshtein",
    "damerau_osa",
    "damerau_unrestricted",
    "jaro-winkler",
    "substring",
    "multiline",
    "2025.10.3",
  ]
---

# Text Similarity & Normalization Standard

## Overview

The similarity and normalization utilities provide standardized text comparison capabilities across all Fulmen helper libraries. This enables consistent fuzzy matching, "Did you mean...?" suggestions, and Unicode-aware text processing for Crucible shim error messages, CLI search, and Docscribe lookups.

## Scope

Helper libraries MUST implement:

1. **Distance Metrics** - Multiple algorithms for different use cases:
   - Levenshtein distance (standard edit distance)
   - Damerau-Levenshtein distance (OSA and unrestricted variants)
   - Jaro-Winkler similarity (prefix-weighted, good for short strings)
   - Substring scoring (longest common substring with position weighting)
2. **Similarity scoring** - Normalized distance as 0.0–1.0 score
3. **Suggestion API** - Generate ranked suggestions from candidate lists with configurable metrics
4. **Normalization presets** - Pre-configured pipelines for common use cases (default, aggressive, minimal, none)
5. **Unicode-aware text processing** - NFC/NFKD normalization, case folding, accent stripping, punctuation removal

## Non-Goals (2025.10.3)

- Token-based similarity (token-set, token-sort) - deferred to 2025.11.x
- Language-specific tokenization beyond Unicode normalization
- Locale-aware collation beyond Unicode case folding
- ML-based ranking or embeddings
- Phonetic matching (Soundex, Metaphone)

## API Surface

### Distance Metrics

All distance/similarity functions accept an optional `metric` parameter to select the algorithm.

**Metric Types**:

| Metric String            | Algorithm                                      | Use Case                                                  |
| ------------------------ | ---------------------------------------------- | --------------------------------------------------------- |
| `"levenshtein"`          | Standard edit distance                         | General-purpose, default for backward compatibility       |
| `"damerau_osa"`          | Damerau-Levenshtein (Optimal String Alignment) | Typo correction with adjacent transpositions in hot paths |
| `"damerau_unrestricted"` | Damerau-Levenshtein (unrestricted)             | Full edit distance with transpositions for complex text   |
| `"jaro_winkler"`         | Jaro-Winkler similarity                        | Short strings, prefix-sensitive matching                  |
| `"substring"`            | Longest common substring                       | Partial matches, path/command suggestions                 |

**Default**: When `metric` is not specified or `null/None/undefined`, defaults to `"levenshtein"` for backward compatibility.

### Distance & Similarity Functions

| Operation  | Go                                                        | Python                                                                    | TypeScript                                                           |
| ---------- | --------------------------------------------------------- | ------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| Distance   | `similarity.Distance(a, b string, metric ...string) int`  | `similarity.distance(a: str, b: str, metric: str = "levenshtein") -> int` | `similarity.distance(a: string, b: string, metric?: string): number` |
| Similarity | `similarity.Score(a, b string, metric ...string) float64` | `similarity.score(a: str, b: str, metric: str = "levenshtein") -> float`  | `similarity.score(a: string, b: string, metric?: string): number`    |

**Levenshtein Distance**:

- Uses dynamic programming Wagner-Fischer algorithm
- Early-exit optimization when length difference exceeds threshold
- Returns edit distance as non-negative integer
- Empty strings: `Distance("", "", "levenshtein") == 0`

**Damerau-Levenshtein (OSA) Distance**:

- Extends Levenshtein with adjacent transposition support using the Optimal String Alignment (OSA) variant
- Prevents editing the same substring twice; faster and sufficient for common typo correction
- Transposition ("ab" → "ba") counts as 1 edit instead of 2
- Returns edit distance as non-negative integer

**Damerau-Levenshtein (Unrestricted) Distance**:

- Implements the full Damerau-Levenshtein distance without the OSA restriction
- Allows multiple edits of the same substring; captures transformations that OSA cannot (e.g., `"CA"` → `"ABC"`
  with distance 2 instead of 3)
- Preferred for general-purpose similarity, document comparison, and scientific workloads (e.g., DNA sequencing)
- Returns edit distance as non-negative integer

**Variant Selection Guidance**:

- Default to `"damerau_osa"` for CLI fuzzy matching and typo correction (matches rapidfuzz `distance.OSA` and
  Rust `strsim::osa_distance`)
- Use `"damerau_unrestricted"` when absolute edit accuracy is required (`rapidfuzz.distance.DamerauLevenshtein`,
  `strsim::damerau_levenshtein`)
- Fixture cases tagged `osa_distinction` / `unrestricted_distinction` demonstrate where the algorithms differ

**Jaro-Winkler Similarity**:

- Returns similarity score in range [0.0, 1.0] (not distance)
- Higher scores for strings with common prefixes
- Configurable prefix scale (default: 0.1) and max prefix length (default: 4)
- Good for short strings like names, commands, identifiers
- Score calculation: `jaro + (prefix_length * prefix_scale * (1 - jaro))`

**Substring Similarity**:

- Finds longest common substring using dynamic programming
- Score = `lcs_length / max(len(a), len(b))`
- Returns score in range [0.0, 1.0]
- Good for partial matches in paths, URLs, long text

**Score Function**:

- For `levenshtein` and `damerau`: `1 - distance / max(len(a), len(b))`
- For `jaro_winkler` and `substring`: Direct similarity score (already in [0.0, 1.0])
- Range: `0.0` (completely different) to `1.0` (identical)
- Empty strings: `Score("", "", metric) == 1.0`

### Normalization Presets (v2.0)

Normalization presets provide pre-configured pipelines for common text processing scenarios. All presets apply operations in a defined order to ensure consistent results across languages.

**Preset Definitions**:

| Preset         | Operations (in order)                                                                  | Use Case                                           |
| -------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------- |
| `"none"`       | No normalization                                                                       | Raw string comparison, preserves all characters    |
| `"minimal"`    | NFC → trim whitespace                                                                  | Canonical Unicode form, minimal processing         |
| `"default"`    | NFC → Unicode casefold → trim whitespace                                               | Case-insensitive matching, preserves accents       |
| `"aggressive"` | NFKD → Unicode casefold → strip combining marks → remove punctuation → trim whitespace | Fuzzy matching, typo tolerance, accent-insensitive |

**Operation Details**:

- **NFC**: Unicode Normalization Form Composed - canonically equivalent strings become identical
- **NFKD**: Unicode Normalization Form Compatibility Decomposed - enables accent stripping
- **Unicode casefold**: Language-aware case folding per Unicode standard (handles Turkish İ/ı, German ß, etc.)
- **Strip combining marks**: Remove Unicode category Mn (Nonspacing_Mark) characters after NFKD
- **Remove punctuation**: Remove Unicode general categories `Pc`, `Pd`, `Pe`, `Pf`, `Pi`, `Po`, `Ps` (e.g., `.,;:!?()[]{}-_`)
- **Trim whitespace**: Remove leading/trailing whitespace

**Locale Handling**:

- All presets use Unicode default case folding (no explicit locale parameter in v2.0)
- Turkish İ/ı and German ß are handled correctly via Unicode casefold tables
- For locale-specific requirements beyond Unicode standard, callers should pre-normalize input (future enhancement tracked)

**Example Transformations**:

```
Input: "  Café-Zürich!  "

none:       "  Café-Zürich!  "       (unchanged)
minimal:    "Café-Zürich!"           (NFC + trim)
default:    "café-zürich!"           (+ casefold)
aggressive: "cafezurich"             (+ strip accents + remove punctuation)
```

### Multi-Line String Handling (v2.0)

#### Line Breaks as Characters

All similarity metrics treat line breaks (`\n`, `\r\n`, `\r`) as ordinary characters:

- `"hello\nworld"` vs. `"hello world"` → distance `1` (newline substituted for space)
- `"text\r\n"` vs. `"text\n"` → distance `1` (CRLF vs. LF)
- `"line1\nline2"` vs. `"line1\nline2"` → distance `0` (identical multi-line strings)

This mirrors reference implementations (Rust `strsim`, Python `rapidfuzz`) and ensures language-agnostic behavior.
Line breaks are not collapsed or treated specially by default.

#### Line Ending Normalization Recommendations

Applications comparing cross-platform content SHOULD normalize line endings prior to similarity checks when
consistent behavior is required:

```python
# Python – normalize to LF
text = text.replace('\r\n', '\n').replace('\r', '\n')
```

```go
// Go – normalize to LF
text = strings.ReplaceAll(text, "\r\n", "\n")
text = strings.ReplaceAll(text, "\r", "\n")
```

```typescript
// TypeScript – normalize to LF
const normalized = text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
```

Skip normalization when exact line ending preservation is required (e.g., checksum verification or CRLF-sensitive
file formats).

#### Normalization Presets and Line Breaks

Built-in presets **preserve line breaks**:

- `none`: leaves newlines untouched
- `minimal`: trims edges but keeps all line breaks
- `default`: preserves line breaks while casefolding + trimming
- `aggressive`: preserves line breaks while stripping punctuation/accents

Example:

```
Input: "Line 1\n\nLine 2"

none:       "Line 1\n\nLine 2"
minimal:    "Line 1\n\nLine 2"
default:    "line 1\n\nline 2"
aggressive: "line 1\n\nline 2"
```

Callers that require single-line comparison must pre-process:

```python
flat = text.replace('\r', ' ').replace('\n', ' ')
normalized = similarity.normalize(flat, preset="aggressive")
```

#### Trailing Newline Considerations

Editors frequently append trailing newlines (`"text"` vs. `"text\n"` → distance `1`). Decide whether to strip them
based on context:

```python
# Strip trailing newlines for document comparison
clean = text.rstrip('\r\n')
```

#### Test Coverage

Fixtures tagged `multiline` and `line_endings` validate the behaviors above; implementations MUST pass these cases to
guarantee cross-language parity.

### Suggestion API (v2.0)

Generate ranked suggestions from a candidate list based on similarity to input. v2.0 adds metric selection, normalization presets, and richer suggestion metadata.

**Signature** (language-idiomatic):

```go
// Go
func Suggest(input string, candidates []string, opts SuggestOptions) []Suggestion

type Suggestion struct {
    Value           string
    Score           float64
    MatchedRange    *Range // nil for distance metrics, set for substring
    Reason          string // Optional: "prefix_match", "typo_correction", etc.
    NormalizedValue string // Result of applying normalization preset
}

type Range struct {
    Start int // Inclusive, 0-indexed character position
    End   int // Exclusive, one past last matched character
}

type SuggestOptions struct {
    MinScore         float64 // Default: 0.6
    MaxSuggestions   int     // Default: 3
    Metric           string  // Default: "levenshtein"
    NormalizePreset  string  // Default: "default"
    PreferPrefix     bool    // Default: false (bonus for prefix matches)
    JaroPrefixScale  float64 // Default: 0.1 (only for jaro_winkler)
    JaroMaxPrefix    int     // Default: 4 (only for jaro_winkler)
}
```

```python
# Python
def suggest(
    input: str,
    candidates: list[str],
    *,
    min_score: float = 0.6,
    max_suggestions: int = 3,
    metric: str = "levenshtein",
    normalize_preset: str = "default",
    prefer_prefix: bool = False,
    jaro_prefix_scale: float = 0.1,
    jaro_max_prefix: int = 4
) -> list[Suggestion]:
    ...

@dataclass
class Suggestion:
    value: str
    score: float
    matched_range: tuple[int, int] | None = None  # [start, end) or None
    reason: str | None = None
    normalized_value: str | None = None
```

```typescript
// TypeScript
function suggest(
  input: string,
  candidates: string[],
  options?: SuggestOptions,
): Suggestion[];

interface Suggestion {
  value: string;
  score: number;
  matchedRange?: [number, number]; // [start, end) or undefined
  reason?: string;
  normalizedValue?: string;
}

interface SuggestOptions {
  minScore?: number; // Default: 0.6
  maxSuggestions?: number; // Default: 3
  metric?: string; // Default: "levenshtein"
  normalizePreset?: string; // Default: "default"
  preferPrefix?: boolean; // Default: false
  jaroPrefixScale?: number; // Default: 0.1
  jaroMaxPrefix?: number; // Default: 4
}
```

**Behavior**:

1. Apply normalization preset to input and candidates
2. Calculate similarity score for each candidate using selected metric
3. Apply prefix bonus if `preferPrefix: true` (multiply score by 1.1 if candidate starts with input)
4. Filter candidates with score ≥ `minScore`
5. Sort by score (descending), then alphabetically for ties
6. Return top `maxSuggestions` results
7. For `substring` metric, populate `matchedRange` with position of longest common substring

**Default Parameters Rationale**:

- `MinScore = 0.6` - Empirically balances helpful suggestions vs. noise
- `MaxSuggestions = 3` - Avoids overwhelming users with too many options
- `Metric = "levenshtein"` - Backward compatible default, general-purpose
- `NormalizePreset = "default"` - Case-insensitive matching, preserves accents for better precision
- `PreferPrefix = false` - Opt-in for CLI/autocomplete scenarios
- `JaroPrefixScale = 0.1`, `JaroMaxPrefix = 4` - Standard Jaro-Winkler parameters matching Rust strsim

### Matched Range Semantics (v2.0)

When using the `substring` metric or when `matchedRange` is populated in suggestions, the following semantics apply:

**Character Indexing**:

- All indices represent **character positions** (Unicode scalar values), not byte offsets
- Ranges use **half-open intervals** `[start, end)`:
  - `start`: Inclusive, 0-indexed position of first matched character
  - `end`: Exclusive, position one past the last matched character
  - Empty match: `[0, 0)` or `null/None/undefined` depending on context

**Cross-Language Implementation Guidance**:

| Language   | String Model       | Implementation Notes                                                                  |
| ---------- | ------------------ | ------------------------------------------------------------------------------------- |
| Go         | UTF-8 bytes        | Convert byte offsets to rune indices using `utf8.RuneCountInString()` or rune slicing |
| Python     | Unicode codepoints | Direct indexing works (already codepoint-based)                                       |
| TypeScript | UTF-16 code units  | Use spread operator `[...str]` for proper surrogate pair handling                     |

**Conformance Test Cases**:

Implementations MUST correctly handle:

- ASCII strings: `"hello"[1:4] == "ell"`
- Astral plane emoji: `"Hi🔥World"`[2:3] == `"🔥"` (single character despite multi-byte encoding)
- Emoji with ZWJ sequences: `"A👩‍🚀B"`[1:2] == `"👩‍🚀"` (single grapheme cluster)
- Multi-script text: `"Hello日本語World"`[5:8] == `"日本語"`

**No Match Representation**:

| Language   | No Match Value  |
| ---------- | --------------- |
| Go         | `nil` (\*Range) |
| Python     | `None`          |
| TypeScript | `undefined`     |

### Normalization Helpers

**Note**: In v2.0, direct use of normalization helpers is discouraged. Use **normalization presets** instead via the `NormalizePreset` option in suggestion/distance functions.

For advanced use cases, the following low-level helpers remain available:

| Helper                            | Purpose                                                                   |
| --------------------------------- | ------------------------------------------------------------------------- |
| `Normalize(value, preset)`        | Apply a normalization preset ("none", "minimal", "default", "aggressive") |
| `Casefold(value)`                 | Unicode-aware case folding per Unicode standard                           |
| `EqualsIgnoreCase(a, b, preset?)` | Compare strings using normalization preset                                |
| `StripAccents(value)`             | Remove diacritics (NFKD decomposition + filter combining marks)           |

**Normalization Preset Application**:

Use the `Normalize(value, preset)` function to apply a preset pipeline:

```go
// Go
normalized := similarity.Normalize("  Café-Zürich!  ", "aggressive")
// Result: "cafezurich"
```

```python
# Python
normalized = similarity.normalize("  Café-Zürich!  ", preset="aggressive")
# Result: "cafezurich"
```

```typescript
// TypeScript
const normalized = similarity.normalize("  Café-Zürich!  ", "aggressive");
// Result: "cafezurich"
```

**Preset Error Handling**:

- Unknown preset values MUST trigger an error (no silent fallback to default)
- Error types: Go `error`, Python `ValueError`, TypeScript `Error`

### Option Validation (v2.0)

All option parameters MUST be validated with fail-fast behavior (no silent clamping or fallback).

**Validation Rules**:

| Option            | Valid Values / Range                                                                        | Default         | Invalid Behavior          |
| ----------------- | ------------------------------------------------------------------------------------------- | --------------- | ------------------------- |
| `metric`          | `"levenshtein"`, `"damerau_osa"`, `"damerau_unrestricted"`, `"jaro_winkler"`, `"substring"` | `"levenshtein"` | Error                     |
| `normalizePreset` | `"none"`, `"minimal"`, `"default"`, `"aggressive"`                                          | `"default"`     | Error                     |
| `jaroPrefixScale` | `[0.0, 0.25]` (float)                                                                       | `0.1`           | Error if outside range    |
| `jaroMaxPrefix`   | `[1, 8]` (integer)                                                                          | `4`             | Error if outside range    |
| `minScore`        | `[0.0, 1.0]` (float)                                                                        | `0.6`           | Error if outside range    |
| `maxSuggestions`  | `≥ 0` (integer)                                                                             | `3`             | Error if negative         |
| `preferPrefix`    | boolean                                                                                     | `false`         | Type error if not boolean |

**Error Messages**:

Implementations SHOULD provide clear error messages:

- `"Invalid metric 'xyz': must be one of: levenshtein, damerau, jaro_winkler, substring"`
- `"jaroPrefixScale must be in range [0.0, 0.25], got: 0.5"`
- `"Unknown normalization preset 'custom': must be one of: none, minimal, default, aggressive"`

**Language-Specific Error Types**:

| Language   | Error Type   | Example                                                |
| ---------- | ------------ | ------------------------------------------------------ |
| Go         | `error`      | `return nil, fmt.Errorf("invalid metric: %s", metric)` |
| Python     | `ValueError` | `raise ValueError(f"Invalid metric: {metric}")`        |
| TypeScript | `Error`      | `throw new Error(`Invalid metric: ${metric}`)`         |

### Error Handling

All functions MUST handle edge cases gracefully:

- Empty strings: Valid inputs, return appropriate defaults (distance=0, score=1.0)
- Null/undefined/nil: Raise appropriate language-specific error
- Non-string types: Type system enforcement (compile-time or runtime)
- Invalid options: Fail-fast with clear error messages (see Option Validation above)

## Shared Fixtures (v2.0)

Crucible provides cross-language validation fixtures:

**Schema**: `schemas/library/foundry/v2.0.0/similarity.schema.json`  
**Fixtures**: `config/library/foundry/similarity-fixtures.yaml`

### Fixture Categories

1. **Levenshtein Distance/Score** – ASCII, Unicode, and multi-line edge cases (`multiline`, `line_endings` tags)
2. **Damerau-Levenshtein (OSA)** – Adjacent swaps and hot-path typo corrections (`osa_distinction`)
3. **Damerau-Levenshtein (Unrestricted)** – True Damerau distance showcasing divergence from OSA (`unrestricted_distinction`)
4. **Jaro-Winkler Similarity** – Prefix-sensitive name/command examples
5. **Substring Scoring** – Path components, partial matches, and no-match cases
6. **Normalization Presets** – Preset behavior on Unicode, punctuation, and multi-line strings (`multiline` tags)
7. **Suggestion Scenarios** – CLI typo correction, normalization impact, and metric overrides

### Fixture Format

```yaml
$schema: https://schemas.fulmenhq.dev/library/foundry/v2.0.0/similarity.schema.json
version: 2025.10.3
test_cases:
  - category: damerau_osa
    cases:
      - input_a: "abcd"
        input_b: "abdc"
        expected_distance: 1
        expected_score: 0.75
        description: "Basic transposition"
        tags: ["transposition"]

  - category: damerau_unrestricted
    cases:
      - input_a: "CA"
        input_b: "ABC"
        expected_distance: 2
        expected_score: 0.33333333333333337
        description: "Unrestricted variant lower cost"
        tags: ["unrestricted_distinction"]

  - category: levenshtein
    cases:
      - input_a: "Line1\nLine2"
        input_b: "Line1 Line2"
        expected_distance: 1
        expected_score: 0.9047619047619048
        description: "Newline substitution"
        tags: ["multiline"]

  - category: normalization_presets
    cases:
      - input: "Line 1\r\n\nLine 2"
        preset: "default"
        expected: "line 1\r\n\nline 2"
        description: "Preserve CRLF while casefolding"
        tags: ["multiline", "line_endings"]

  - category: suggestions
    cases:
      - input: "docscrib"
        candidates: ["docscribe", "crucible-shim", "config-path-api"]
        options:
          metric: "levenshtein"
          normalize_preset: "default"
        expected:
          - value: "docscribe"
            score: 0.8889 # ≈1 - 1/9 (implementations may emit higher precision)
        description: "Typo correction"
```

## Algorithm Implementations (v2.0)

This section provides implementation guidance for each metric. Implementations MAY use optimized libraries but MUST produce results matching the reference algorithms described here.

### Substring Scoring Algorithm

**Algorithm**: Longest Common Substring (LCS) with position tie-breakers

**Input**: Two strings `needle` and `haystack` (after normalization)

**Output**: Score (float), matched range `[start, end)`

**Steps**:

1. Apply normalization preset to both strings
2. Compute longest common substring length using dynamic programming (O(n·m) complexity)
3. Calculate score: `lcs_length / max(len(needle), len(haystack))`
4. Find position of first occurrence of the maximal substring in haystack
5. If multiple start positions tie (same LCS length), prefer the lowest start index
6. If same start index, prefer the lowest end index
7. Return score and matched range relative to **normalized** haystack

**Pseudocode**:

```
function substringScore(needle, haystack):
  if len(needle) == 0 or len(haystack) == 0:
    return {score: 0.0, range: nil}

  # DP matrix: lcs[i][j] = length of LCS ending at needle[i-1], haystack[j-1]
  lcs = matrix[len(needle)+1][len(haystack)+1] initialized to 0
  max_length = 0
  end_positions = []

  for i in 1..len(needle):
    for j in 1..len(haystack):
      if needle[i-1] == haystack[j-1]:
        lcs[i][j] = lcs[i-1][j-1] + 1
        if lcs[i][j] > max_length:
          max_length = lcs[i][j]
          end_positions = [j]
        elif lcs[i][j] == max_length:
          end_positions.append(j)

  if max_length == 0:
    return {score: 0.0, range: nil}

  # Find earliest start position
  earliest_end = min(end_positions)
  start = earliest_end - max_length
  end = earliest_end

  score = max_length / max(len(needle), len(haystack))
  return {score: score, range: [start, end)}
```

**Optimization Notes**:

- For needles ≤256 characters: use full O(n·m) DP
- For haystacks >10k characters: future enhancement may add windowing/early exit (not required in v2.0)
- Space optimization: use two-row approach for O(min(m,n)) memory instead of O(m·n)

**Complexity**:

- Time: O(n·m) where n = len(needle), m = len(haystack)
- Space: O(m·n) naive, O(min(m,n)) optimized

### Damerau-Levenshtein Algorithm

Extend standard Levenshtein with transposition support. An adjacent transposition (swap of two consecutive characters) counts as a single edit operation.

**Restricted vs. Optimal String Alignment Distance (OSA)**:

- v2.0 uses **OSA variant** (simpler, O(n·m) space)
- True Damerau-Levenshtein allows unlimited transpositions (requires O(n·m·|alphabet|) space) - deferred to future enhancement

**Pseudocode** (OSA variant):

```
function damerauDistance(a, b):
  if len(a) == 0: return len(b)
  if len(b) == 0: return len(a)

  d = matrix[len(a)+1][len(b)+1]

  for i in 0..len(a):
    d[i][0] = i
  for j in 0..len(b):
    d[0][j] = j

  for i in 1..len(a):
    for j in 1..len(b):
      cost = 0 if a[i-1] == b[j-1] else 1

      d[i][j] = min(
        d[i-1][j] + 1,        // deletion
        d[i][j-1] + 1,        // insertion
        d[i-1][j-1] + cost    // substitution
      )

      # Transposition check
      if i > 1 and j > 1 and a[i-1] == b[j-2] and a[i-2] == b[j-1]:
        d[i][j] = min(d[i][j], d[i-2][j-2] + 1)  // transposition

  return d[len(a)][len(b)]
```

### Jaro-Winkler Algorithm

Jaro-Winkler is a variant of Jaro distance with prefix bonus.

**Parameters**:

- `prefix_scale`: Scaling factor for common prefix bonus (default: 0.1, max: 0.25)
- `max_prefix`: Maximum prefix length to consider (default: 4, range: [1, 8])

**Formula**:

```
jaro_winkler = jaro + (prefix_length * prefix_scale * (1 - jaro))

where:
  jaro = (matches/len(a) + matches/len(b) + (matches - transpositions)/matches) / 3
  prefix_length = min(max_prefix, length of common prefix up to 4 characters)
```

**Reference Implementation**: Rust `strsim` v0.11+ provides the canonical implementation for fixture generation.

## Performance Requirements (v2.0)

### Benchmark Targets

Measured on reference hardware (Apple M1, 8-core, Go 1.21, Python 3.12, Node 20):

| Metric       | String Length  | Go (p95) | Python (p95) | TypeScript (p95) | Notes                                     |
| ------------ | -------------- | -------- | ------------ | ---------------- | ----------------------------------------- |
| Levenshtein  | ≤128 chars     | ≤0.5ms   | ≤1.0ms       | ≤1.0ms           | Baseline                                  |
| Damerau      | ≤128 chars     | ≤0.55ms  | ≤1.1ms       | ≤1.1ms           | ≤10% regression vs Levenshtein            |
| Jaro-Winkler | ≤50 chars      | ≤0.3ms   | ≤0.6ms       | ≤0.6ms           | Faster than Levenshtein for short strings |
| Substring    | ≤256/10k chars | ≤1.0ms   | ≤2.0ms       | ≤2.0ms           | Needle ≤256, haystack ≤10k                |

**Performance Variance**:

- ±20% variance between languages is acceptable due to runtime differences
- Python may use C-accelerated libraries (e.g., `rapidfuzz`) to meet targets
- TypeScript may use WASM or native bindings for performance-critical paths

**Degradation for Large Strings**:

- For strings >1000 chars, document as "best effort" with note on O(n·m) complexity
- No hard SLA for large strings in v2.0
- Future enhancements may add early-exit heuristics

**Measurement Methodology**:

- Use high-resolution timers (Go: `testing.B`, Python: `timeit`, TypeScript: `perf_hooks`)
- Run 10,000 iterations per test
- Report p50 and p95 latencies
- Test with 128-character strings (typical CLI/asset name length)
- Document exact CPU model in benchmark results

### Optimization Strategies

**Required**:

- Early-exit when length difference exceeds remaining budget
- Single-pass normalization (avoid redundant string copies)

**Optional**:

- Wagner-Fischer algorithm with linear space optimization
- SIMD for character comparisons (if language/platform supports)
- Memoization for repeated candidates (caller-side caching)

## Implementation Guidelines

### Algorithm Choice

Use **Wagner-Fischer dynamic programming** for Levenshtein distance:

```
distance(a, b):
  if len(a) == 0: return len(b)
  if len(b) == 0: return len(a)

  matrix[0..len(a)+1][0..len(b)+1]
  for i in 0..len(a):
    matrix[i][0] = i
  for j in 0..len(b):
    matrix[0][j] = j

  for i in 1..len(a):
    for j in 1..len(b):
      cost = 0 if a[i-1] == b[j-1] else 1
      matrix[i][j] = min(
        matrix[i-1][j] + 1,      // deletion
        matrix[i][j-1] + 1,      // insertion
        matrix[i-1][j-1] + cost  // substitution
      )

  return matrix[len(a)][len(b)]
```

**Space Optimization**: Use two-row approach (O(min(m, n)) space instead of O(m\*n))

### Unicode Handling

- **Go**: Use `unicode` package for case folding, `norm` for NFD/NFC
- **Python**: Use `unicodedata` module for normalization, `str.casefold()` for case folding
- **TypeScript**: Use `Intl.Collator` or polyfill for case folding, normalize via `String.prototype.normalize()`

**Character Counting**:

- Use **grapheme clusters** (visible characters) not byte/code point length
- Go: `utf8.RuneCountInString()`
- Python: `len(str)` (already counts graphemes)
- TypeScript: `[...str].length` or grapheme segmentation library

### Testing Requirements

Each helper library MUST:

1. **Pass all shared fixtures** from `config/library/foundry/similarity-fixtures.yaml`
2. **Benchmark against targets** and document results in library-specific docs
3. **Property-based tests** (if framework available):
   - `Distance(a, a) == 0` (identity)
   - `Distance(a, b) == Distance(b, a)` (symmetry)
   - `Distance(a, c) <= Distance(a, b) + Distance(b, c)` (triangle inequality)
   - `Score(a, b) in [0.0, 1.0]` (range bounds)

## Integration Examples

### Crucible Shim Error Messages

```python
# pyfulmen example
from fulmen.foundry import similarity
from fulmen.crucible_shim import AssetNotFoundError

def get_asset(asset_id: str) -> Asset:
    try:
        return shim.read_asset(asset_id)
    except AssetNotFoundError:
        candidates = shim.list_assets()
        suggestions = similarity.suggest(
            asset_id,
            [c.id for c in candidates],
            min_score=0.6,
            max_suggestions=3
        )

        if suggestions:
            hint = f"Did you mean: {', '.join(s.value for s in suggestions)}?"
            raise AssetNotFoundError(f"Asset '{asset_id}' not found. {hint}")
        else:
            raise
```

### Docscribe Document Search

```typescript
// tsfulmen example
import { similarity } from "@fulmen/foundry";
import { docscribe } from "@fulmen/docscribe";

function findDocument(query: string): Document | null {
  const docs = docscribe.listDocuments();

  const suggestions = similarity.suggest(
    query,
    docs.map((d) => d.title),
    {
      minScore: 0.5,
      maxSuggestions: 5,
      normalize: true,
    },
  );

  return suggestions.length > 0
    ? docs.find((d) => d.title === suggestions[0].value)
    : null;
}
```

### CLI Command Suggestions

```go
// gofulmen example
import (
    "github.com/fulmenhq/gofulmen/foundry/similarity"
)

func SuggestCommand(input string, commands []string) {
    suggestions := similarity.Suggest(input, commands, similarity.SuggestOptions{
        MinScore:       0.6,
        MaxSuggestions: 3,
        Normalize:      true,
    })

    if len(suggestions) > 0 {
        fmt.Printf("Unknown command '%s'. Did you mean:\n", input)
        for _, s := range suggestions {
            fmt.Printf("  - %s (%.0f%% match)\n", s.Value, s.Score*100)
        }
    }
}
```

## Module Manifest Entry

The Foundry module entry in `config/library/v1.0.0/module-manifest.yaml` documents similarity capabilities:

```yaml
- id: foundry
  tier: core
  requirement: mandatory
  description: >
    Common reference data, patterns, and text utilities including similarity
    scoring and normalization for fuzzy matching and suggestions.
  capabilities:
    - country_codes
    - http_statuses
    - mime_types
    - patterns
    - text_similarity
    - text_normalization
  coverage:
    - language: go
      target: 95
    - language: python
      target: 95
    - language: typescript
      target: 95
```

## Versioning & Evolution

**Current Version**: 2.0.0 (2025.10.3)

**Stability Promise**:

- API signatures stable for v2.x
- Backward compatibility: v1.0 behavior preserved when options not specified
- Fixture format backward compatible (v1 fixtures remain valid)
- Performance targets may tighten in minor versions

**v2.0 Changes from v1.0**:

1. **Added Metrics**: Damerau-Levenshtein, Jaro-Winkler, Substring scoring
2. **Added Normalization Presets**: `none`, `minimal`, `default`, `aggressive`
3. **Extended Suggestion API**: Added `metric`, `normalizePreset`, `preferPrefix`, Jaro-Winkler tuning
4. **Richer Suggestion Results**: Added `matchedRange`, `reason`, `normalizedValue` fields
5. **Stricter Option Validation**: Fail-fast on invalid parameters (no silent fallback)

**Backward Compatibility**:

- All v1.0 function calls work identically in v2.0 when new options not provided
- `Distance(a, b)` defaults to Levenshtein (same as v1.0)
- `Score(a, b)` defaults to Levenshtein-based scoring (same as v1.0)
- `Suggest(input, candidates)` with old options struct works unchanged
- Normalization behavior unchanged when using boolean `normalize` flag (maps to `default` preset)

**Future Enhancements** (post-2025.10.3):

- Token-based similarity (token-set, token-sort) for set-like comparisons
- Configurable edit costs (insertion/deletion/substitution/transposition)
- Phonetic matching (Soundex, Metaphone)
- Locale-specific normalization beyond Unicode casefold
- Custom preprocessing hooks (synonym dictionaries)

## Migration Guide (v1.0 → v2.0)

### No Changes Required (Backward Compatible)

If your code only uses basic distance/score/suggest functions without custom options, no changes are required:

```go
// These work identically in v1.0 and v2.0
dist := similarity.Distance("hello", "hallo")
score := similarity.Score("hello", "hallo")
suggestions := similarity.Suggest("schem", candidates, similarity.SuggestOptions{
    MinScore: 0.6,
    MaxSuggestions: 3,
    Normalize: true,  // Maps to "default" preset in v2.0
})
```

### Opt-In to New Features

To use new metrics and presets:

```go
// Use Damerau-Levenshtein for transposition-tolerant matching
dist := similarity.Distance("abcd", "abdc", "damerau")  // Returns 1 (vs. 2 for Levenshtein)

// Use aggressive normalization for fuzzy matching
suggestions := similarity.Suggest("cafe", candidates, similarity.SuggestOptions{
    MinScore:        0.7,
    MaxSuggestions:  5,
    Metric:          "jaro_winkler",
    NormalizePreset: "aggressive",  // Strips accents, removes punctuation
    PreferPrefix:    true,          // Bonus for prefix matches
})

// Use substring matching for partial path search
suggestions := similarity.Suggest("schemas", candidates, similarity.SuggestOptions{
    Metric: "substring",
    // matchedRange will be populated in results
})
```

### Handle New Suggestion Fields

v2.0 Suggestion struct has optional new fields:

```go
for _, s := range suggestions {
    fmt.Printf("Suggestion: %s (score: %.2f)\n", s.Value, s.Score)

    // v2.0 additions (check for nil/None/undefined):
    if s.MatchedRange != nil {
        fmt.Printf("  Matched at: [%d:%d)\n", s.MatchedRange.Start, s.MatchedRange.End)
    }
    if s.Reason != "" {
        fmt.Printf("  Reason: %s\n", s.Reason)
    }
}
```

### Normalization API Changes

v1.0 used boolean flags:

```go
// v1.0 (deprecated but still works)
normalized := similarity.Normalize(value, similarity.NormalizeOptions{
    StripAccents: true,
})
```

v2.0 recommends presets:

```go
// v2.0 (recommended)
normalized := similarity.Normalize(value, "aggressive")
```

## References

- [Levenshtein Distance (Wikipedia)](https://en.wikipedia.org/wiki/Levenshtein_distance)
- [Damerau-Levenshtein Distance (Wikipedia)](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance)
- [Jaro-Winkler Similarity (Wikipedia)](https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance)
- [Longest Common Substring (Wikipedia)](https://en.wikipedia.org/wiki/Longest_common_substring_problem)
- [Wagner-Fischer Algorithm](https://en.wikipedia.org/wiki/Wagner%E2%80%93Fischer_algorithm)
- [Unicode Normalization Forms (TR15)](https://unicode.org/reports/tr15/)
- [Unicode Case Folding (TR21)](https://www.unicode.org/reports/tr21/tr21-5.html)
- [Rust strsim (Reference Implementation)](https://docs.rs/strsim/latest/strsim/)
- Shared Fixtures: `config/library/foundry/similarity-fixtures.yaml`
- Fixture Schema: `schemas/library/foundry/v1.0.0/similarity-fixtures.schema.json`

---

**Conformance**: This standard is part of the Fulmen Helper Library Standard (2025.10.3) and synchronizes to all language wrappers.

**Fixture Generation**: Values in `similarity-fixtures.yaml` are computed using Rust `strsim` 0.11.x as canonical reference, with cross-validation against Python `rapidfuzz` 3.x. Human spot-checked for edge cases.
