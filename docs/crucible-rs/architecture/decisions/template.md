---
id: "ADR-XXXX"
title: "Brief Descriptive Title (5-100 characters)"
status: "proposal"
date: "YYYY-MM-DD"
last_updated: "YYYY-MM-DD"
deciders:
  - "@github-username"
scope: "Ecosystem"
supersedes: [] # Single ID or array. Ecosystem: "ADR-0003" or ["ADR-0003", "ADR-0005"]. Local: ["gofulmen:0007", "pyfulmen:0012"]
superseded_by: ""
deprecation_date: ""
tags:
  - "tag1"
  - "tag2"
related_adrs:
  - "ADR-YYYY"
adoption:
  gofulmen: "planned"
  pyfulmen: "planned"
  tsfulmen: "planned"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-XXXX: Brief Descriptive Title

<!--
INSTRUCTIONS:
1. Filename: Use ADR-XXXX-kebab-case-title.md format (e.g., ADR-0001-two-tier-adr-system.md)
2. Frontmatter id: Must match filename pattern ADR-XXXX (e.g., "ADR-0001")
3. Update all required frontmatter fields (see schema: schemas/config/standards/v1.0.0/adr-frontmatter.schema.json)
4. Remove unused optional fields (supersedes, superseded_by, deprecation_date)
5. Delete this comment block before committing
6. See lifecycle stages in schemas/config/standards/v1.0.0/adr-lifecycle-status.json
7. See adoption statuses in schemas/config/standards/v1.0.0/adr-adoption-status.json

NAMING CONVENTION:
- All ADRs use ADR-XXXX prefix in both filename and frontmatter id
- Location determines scope (ecosystem vs local), not filename format
- Ecosystem ADRs: docs/architecture/decisions/ADR-XXXX-slug-title.md
- Local ADRs: docs/development/adr/ADR-XXXX-slug-title.md

SLUG RULES for title portion:
- Lowercase alphanumeric characters and hyphens only
- No spaces (use hyphens instead)
- No consecutive hyphens (e.g., "two--hyphens" is invalid)
- No underscores (use hyphens instead)
- Examples: "two-tier-adr-system", "triple-index-catalog", "logging-profiles"

LIFECYCLE STATUS VALUES:
- proposal (10): Under evaluation, not yet accepted
- experimental (20): Approved for trial use, may change
- accepted (30): Approved and recommended for active use
- stable (40): Widely adopted, battle-tested, best practice
- deprecated (50): Discouraged, migration path available
- superseded (60): Replaced by another ADR
- retired (70): No longer relevant, historical reference only

ADOPTION STATUS VALUES (per library):
- not-applicable (0): Does not apply to this library/language
- deferred (5): Postponed, include rationale
- planned (10): Planned but not started
- in-progress (20): Active implementation underway
- implemented (30): Fully implemented, ready for validation
- verified (40): Implemented and validated
-->

## Status

**Current Status**: [Proposal | Experimental | Accepted | Stable | Deprecated | Superseded | Retired]

<!-- If superseded: Add note here -->
<!-- ⚠️ **Superseded by [ADR-YYYY: Title](./YYYY-title.md)** -->

<!-- If deprecated: Add deprecation notice -->
<!-- ⚠️ **Deprecated**: This decision is discouraged. Plan to migrate by [YYYY-MM-DD].
     See [Migration Guide](#migration-path) below. -->

## Context

<!--
Describe the problem, forces, and constraints that led to this decision.

Include:
- What problem are we solving?
- What are the requirements and constraints?
- What are the success criteria?
- What is the current situation that needs to change?
- What alternatives were considered initially?
-->

## Decision

<!--
State the decision clearly and concisely.

Include:
- What we decided to do
- Key implementation approach
- Code examples or patterns (if relevant)
- How this decision should be applied
- Boundaries and limitations
-->

## Rationale

<!--
Explain WHY this is the right choice.

Include:
- Primary benefits and advantages
- How this aligns with ecosystem goals
- Why this is better than alternatives
- What evidence or experience supports this
- What trade-offs are acceptable
-->

## Alternatives Considered

### Alternative 1: [Name]

**Description**: [Brief description of this alternative]

**Pros**:

- [Advantage 1]
- [Advantage 2]

**Cons**:

- [Disadvantage 1]
- [Disadvantage 2]

**Decision**: [Rejected | Deferred] - [Reason for rejection or deferral]

### Alternative 2: [Name]

**Description**: [Brief description of this alternative]

**Pros**:

- [Advantage 1]
- [Advantage 2]

**Cons**:

- [Disadvantage 1]
- [Disadvantage 2]

**Decision**: [Rejected | Deferred] - [Reason for rejection or deferral]

## Consequences

### Positive

- ✅ [Benefit 1: Describe positive impact]
- ✅ [Benefit 2: Describe positive impact]

### Negative

- ⚠️ [Trade-off 1: Describe cost or limitation]
- ⚠️ [Trade-off 2: Describe cost or limitation]

### Neutral

- ℹ️ [Impact 1: Neither clearly positive nor negative]
- ℹ️ [Impact 2: Neither clearly positive nor negative]

## Implementation

<!--
Describe how this decision should be implemented.

Include:
- Files or modules affected
- Implementation steps or phases
- Dependencies or prerequisites
- Validation approach
- Testing requirements
- Documentation updates needed
-->

### Implementation Status

<!-- Update adoption frontmatter as implementation progresses in each library -->

| Library  | Status         | Notes                | PR/Issue     |
| -------- | -------------- | -------------------- | ------------ |
| gofulmen | planned        | [Additional context] | [#123](link) |
| pyfulmen | planned        | [Additional context] | [#456](link) |
| tsfulmen | planned        | [Additional context] | [#789](link) |
| rsfulmen | not-applicable | [Why not applicable] | N/A          |
| csfulmen | not-applicable | [Why not applicable] | N/A          |

## Cross-Language Coordination

<!--
FOR ECOSYSTEM ADRs:
Describe how this decision should be implemented across languages.

Include:
- Language-specific considerations
- How patterns translate to different languages
- Test/validation parity requirements
- Documentation synchronization needs
- Timeline for cross-language adoption
-->

<!--
FOR LOCAL ADRs:
Link to related ecosystem ADRs and explain relationship.

Example:
This local decision implements [ADR-0001: Triple-Index Catalog Strategy](../crucible-go/architecture/decisions/0001-triple-index-catalog-strategy.md)
using Go-specific patterns (sync.Pool for buffer management).
-->

## Migration Path

<!--
REQUIRED IF STATUS IS 'deprecated':
Provide clear migration guidance.

Include:
- What users need to change
- Step-by-step migration instructions
- Automated migration tools (if available)
- Timeline and deprecation schedule
- Support resources
- Link to replacement ADR (if applicable)
-->

## References

<!--
Link to supporting materials.

Include:
- Related specifications or standards
- Implementation examples
- External documentation
- Research papers or blog posts
- Discussion threads or RFCs
- Related code or PRs
-->

- [Related Document 1](link)
- [Related Document 2](link)
- [Specification Reference](link)

## Revision History

<!--
Track major updates to this ADR.
Minor edits (typos, formatting) don't need entries.
-->

| Date       | Status Change  | Summary                              | Updated By |
| ---------- | -------------- | ------------------------------------ | ---------- |
| YYYY-MM-DD | → proposal     | Initial draft                        | @username  |
| YYYY-MM-DD | → experimental | Approved for trial in gofulmen       | @username  |
| YYYY-MM-DD | → accepted     | Implemented in 2+ libraries          | @username  |
| YYYY-MM-DD | → stable       | Proven in production, widely adopted | @username  |

---

**Schema Conformance**: This ADR conforms to [ADR Frontmatter Schema v1.0.0](https://schemas.fulmenhq.dev/standards/adr-frontmatter-v1.0.0.json)

<!--
For more information:
- ADR Format Guide: docs/architecture/decisions/README.md
- Lifecycle Management: .plans/active/2025.10.2/library-adr-brief.md
- Helper Library Standard: docs/architecture/fulmen-helper-library-standard.md
-->
