---
id: "ADR-0002"
title: "Triple-Index Catalog Strategy"
status: "proposal"
date: "2025-10-15"
last_updated: "2025-10-15"
deciders:
  - "@schema-cartographer"
  - "@pipeline-architect"
scope: "Ecosystem"
supersedes: []
tags:
  - "catalog"
  - "foundry"
  - "ssot"
related_adrs:
  - "ADR-0001"
adoption:
  gofulmen: "verified"
  pyfulmen: "implemented"
  tsfulmen: "planned"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-0002: Triple-Index Catalog Strategy

## Status

**Current Status**: Proposal – pending maintainer sign-off across libraries.

## Context

Fulmen helper libraries ship read-only catalogs sourced from Crucible (e.g., ISO country codes, HTTP status groups, MIME types). Consumers rely on these catalogs for deterministic, low-latency lookups in multiple identifier formats. The ISO 3166-1 country catalog in particular must resolve:

- **Alpha-2** codes (`"US"`, `"CA"`)
- **Alpha-3** codes (`"USA"`, `"CAN"`)
- **Numeric** codes (`"840"`, `"124"`)

The catalog must guarantee:

- **O(1) lookup performance** without linear scans
- **Case-insensitive matching** for alpha codes
- **Canonical numeric handling** (zero-padding)
- **Consistency across languages** so downstream applications behave identically irrespective of helper library

PyFulmen and gofulmen independently converged on a triple-index approach during their v0.1.x implementations. This ADR codifies the strategy as the ecosystem contract.

## Decision

Every helper library MUST materialize **three dedicated indexes** when loading SSOT catalogs that expose multiple identifier forms:

1. `alpha2Index` keyed by upper-cased Alpha-2 codes (`"US"`, `"CA"`)
2. `alpha3Index` keyed by upper-cased Alpha-3 codes (`"USA"`, `"CAN"`)
3. `numericIndex` keyed by zero-padded numeric strings (`"840"`, `"076"`)

### Required Behaviours

- Normalize input tokens before lookup:
  - Trim whitespace
  - Upper-case alphabetic tokens
  - Left-pad numeric tokens to three digits (`"76"` → `"076"`)
- Build indexes once during catalog load (lazy-loading permitted) and reuse for all lookups.
- Provide explicit accessors (e.g., `GetCountry(code)`, `GetCountryByAlpha3(code)`, `GetCountryByNumeric(code)`).
- Expose validation helpers (`ValidateCountryCode`) that can consult the indexes sequentially.
- Maintain parity across other multi-key catalogs (future MIME types, currencies, etc.) when multiple identifier forms exist.
- When new multi-identifier catalogs land (for example, MIME types with ID, mime string, and extensions), their ecosystem specs MUST reference this ADR (alongside the Foundry catalog standard) and declare the indexes they require so downstream foundations implement them consistently.

### Reference Implementation Sketch

```go
type countryCatalog struct {
    alpha2  map[string]*Country
    alpha3  map[string]*Country
    numeric map[string]*Country
}

func newCountryCatalog(countries []Country) *countryCatalog {
    alpha2 := make(map[string]*Country, len(countries))
    alpha3 := make(map[string]*Country, len(countries))
    numeric := make(map[string]*Country, len(countries))

    for i := range countries {
        c := &countries[i]
        alpha2[strings.ToUpper(c.Alpha2)] = c
        alpha3[strings.ToUpper(c.Alpha3)] = c
        numeric[fmt.Sprintf("%03s", c.Numeric)] = c
    }

    return &countryCatalog{alpha2: alpha2, alpha3: alpha3, numeric: numeric}
}
```

Language-specific code may vary, but the indexing semantics MUST match.

## Rationale

- **Performance**: Avoids repeated O(n) scans; dict/map lookups remain constant time even as catalogs expand to hundreds of entries.
- **Correctness**: Eliminates format-detection heuristics — each identifier is resolved through an explicit index.
- **Clarity**: Dedicated maps communicate intent and are easier to maintain than composite-key or prefix hacks.
- **Parity**: Aligns the gofulmen and pyfulmen implementations that already use this technique, simplifying downstream audits.
- **Extensibility**: Additional indexes (e.g., by localized name) can be added without disrupting existing callers.

## Alternatives Considered

### Alternative 1: Single Map with Composite Keys

**Description**: Store entries in a single dictionary keyed by tuples (`("alpha2","US")`, `("alpha3","USA")`, `("numeric","840")`).

**Pros**:

- Single data structure to manage
- Lower memory overhead

**Cons**:

- Requires callers to provide format hints or run detection logic
- Harder to enumerate by identifier type
- Less readable; tuple keys increase cognitive load

**Decision**: Rejected – sacrifices ergonomics without delivering meaningful benefits.

### Alternative 2: Unified Map with Mixed Keys

**Description**: Insert Alpha-2, Alpha-3, and numeric codes directly into one map (`"US"`, `"USA"`, `"840"`).

**Pros**:

- Simplest lookup call-site
- Lowest initialization code

**Cons**:

- Risk of collisions between Alpha-3 and numeric codes
- Impossible to determine which identifier matched
- Enumeration by type requires additional bookkeeping

**Decision**: Rejected – correctness risks outweigh minor setup savings.

### Alternative 3: Linear Scan Per Lookup

**Description**: Keep a single list/slice of countries and iterate to find matches.

**Pros**:

- Minimal upfront memory
- Straightforward to implement

**Cons**:

- O(n) per lookup; unacceptable for catalogs with hundreds of entries
- Encourages call-sites to cache results, fragmenting logic

**Decision**: Rejected – fails the performance requirement.

## Consequences

### Positive

- ✅ Consistent O(1) lookup performance across all helper libraries.
- ✅ Shared mental model for maintainers and auditors.
- ✅ Straightforward validation helpers that reuse the same indexes.

### Negative

- ⚠️ Additional memory overhead (roughly 3× number of countries). For ≤300 entries, the cost remains <100 KB.
- ⚠️ Slightly more initialization code (three assignments per catalog entry).

### Neutral

- ℹ️ Lazy-loading remains permitted; implementations can defer index construction until first use.
- ℹ️ Catalog updates require maintaining all three indexes in sync, but the workflow mirrors current practice.

## Implementation

- Update catalog loaders in each language to materialize the three indexes when reading Crucible assets.
- Ensure validation helpers (`ValidateCountryCode`, etc.) reuse the indexes instead of performing ad-hoc scans.
- Add unit tests covering:
  - Case-insensitive alpha lookups
  - Numeric zero-padding behaviour
  - Enumeration parity across the three maps
- Document the triple-index contract in language-specific API references.

### Implementation Status

| Library  | Status         | Notes                                                                | PR/Issue |
| -------- | -------------- | -------------------------------------------------------------------- | -------- |
| gofulmen | verified       | Production-ready in `foundry/catalog.go` v0.1.1, 89.3% test coverage | N/A      |
| pyfulmen | implemented    | Implemented in `foundry/catalog.py` v0.1.2                           | N/A      |
| tsfulmen | planned        | Will adopt during Foundry MVP                                        | N/A      |
| rsfulmen | not-applicable | No runtime planned for Rust foundation yet                           | N/A      |
| csfulmen | not-applicable | .NET foundation not in scope for this phase                          | N/A      |

## Cross-Language Coordination

- Align naming conventions (`alpha2`, `alpha3`, `numeric`) to simplify documentation and parity tests.
- Share fixture catalogs in Crucible (e.g., 5-country preview dataset) so integration tests assert identical behaviour.
- Coordinate benchmark expectations: document target lookup latency (<500 ns) to catch regressions.
- When syncing new catalog data, ensure `make sync` updates downstream language wrappers before publishing releases.

## References

- PyFulmen ADR-0003 (local) – source material for this decision.
- Gofulmen Foundry catalog implementation (`foundry/catalog.go`).
- Crucible Foundry standard (`docs/standards/library/foundry/README.md`) – updated responsibilities/tests.
- Upcoming catalog fixtures (`schemas/library/foundry/country-codes.yaml`).

## Revision History

| Date       | Status Change      | Summary                                       | Updated By           |
| ---------- | ------------------ | --------------------------------------------- | -------------------- |
| 2025-10-15 | gofulmen: verified | Upgrade gofulmen to verified (89.3% coverage) | @gofulmen            |
| 2025-10-15 | → proposal         | Initial ecosystem draft (for review)          | @schema-cartographer |

---

**Schema Conformance**: This ADR conforms to [ADR Frontmatter Schema v1.0.0](https://schemas.fulmenhq.dev/standards/adr-frontmatter-v1.0.0.json)
