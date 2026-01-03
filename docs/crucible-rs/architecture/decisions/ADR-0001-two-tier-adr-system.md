---
id: "ADR-0001"
title: "Two-Tier ADR System for Fulmen Ecosystem"
status: "accepted"
date: "2025-10-15"
last_updated: "2025-10-15"
deciders:
  - "@3leapsdave"
  - "@pipeline-architect"
scope: "Ecosystem"
supersedes: []
superseded_by: ""
deprecation_date: ""
tags:
  - "architecture"
  - "standards"
  - "documentation"
  - "governance"
related_adrs: []
adoption:
  gofulmen: "planned"
  pyfulmen: "planned"
  tsfulmen: "planned"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-0001: Two-Tier ADR System for Fulmen Ecosystem

## Status

**Current Status**: Accepted

This ADR establishes the Architecture Decision Record (ADR) system for the Fulmen ecosystem, including format, lifecycle management, and governance.

## Context

The Fulmen ecosystem consists of a standards SSOT (Crucible) and multiple language-specific foundation libraries (gofulmen, pyfulmen, tsfulmen, etc.). We needed a consistent way to document architectural decisions at two levels:

1. **Ecosystem-wide decisions** affecting multiple libraries (e.g., catalog indexing strategies, logging profiles)
2. **Library-specific decisions** unique to one language implementation (e.g., Go-specific optimizations)

### Requirements

- Single source of truth for cross-language decisions
- Independent decision-making for language-specific concerns
- Clear promotion path when local decisions reveal ecosystem impact
- Schema-driven lifecycle management enabling tooling
- Adoption tracking across libraries
- Integration with existing Crucible sync infrastructure

### Challenges

- Avoiding filename collisions between ecosystem and local ADRs
- Maintaining clarity about which ADRs apply to which libraries
- Enabling comparative operations (e.g., "show all stable ADRs")
- Supporting superseding relationships across repositories
- Keeping documentation synchronized via `make sync`

## Decision

We adopt a **two-tier ADR system** with schema-driven lifecycle management:

### Tier 1: Ecosystem ADRs (Crucible)

**Location**: `docs/architecture/decisions/` in Crucible SSOT

**Scope**: Cross-language decisions affecting multiple libraries

**Sync Behavior**: Automatically synced to all language wrappers via `make sync`

**Format**:

- Filename: `ADR-XXXX-kebab-case-title.md`
- Frontmatter: `id: "ADR-XXXX"`, `scope: "Ecosystem"`
- Sequential numbering starting at 0001

### Tier 2: Local ADRs (Per Library)

**Location**: `docs/development/adr/` in each foundation library

**Scope**: Language/implementation-specific decisions

**Sync Behavior**: Not synced; maintained independently by library teams

**Format**:

- Filename: `ADR-XXXX-kebab-case-title.md` (same format as ecosystem)
- Frontmatter: `id: "ADR-XXXX"`, `scope: "{library-name}"`
- Independent sequential numbering per library
- **Note**: Local ADRs can omit the `adoption` field since it's only meaningful for cross-library tracking

### Key Design Decisions

**1. Consistent Filename Format**

All ADRs use `ADR-XXXX-title.md` format regardless of scope. Location determines whether it's ecosystem or local, not filename.

**Rationale**: Easier discovery (`find . -name "ADR-*"`), simpler tooling, reduced cognitive load.

**2. Schema-Driven Lifecycle Management**

Three JSON schemas define the system:

- `adr-lifecycle-status.json`: 7 lifecycle stages (proposal → experimental → accepted → stable → deprecated → superseded → retired) with numeric sort order
- `adr-adoption-status.json`: 6 adoption statuses (planned → in-progress → implemented → verified) with sort order
- `adr-frontmatter.schema.json`: Complete frontmatter validation

**Rationale**: Enables comparative operations, CI validation, automated index generation, and adoption dashboards.

**3. Scoped References for Superseding**

ADRs can reference other ADRs using:

- Ecosystem ADRs: `ADR-0003` (unscoped)
- Local ADRs from other repos: `gofulmen:ADR-0007` (scoped)

**Rationale**: Enables ecosystem ADRs to consolidate multiple local ADRs while maintaining traceability.

**4. Adoption Tracking in Frontmatter**

Ecosystem ADRs include per-library adoption status:

```yaml
adoption:
  gofulmen: "verified"
  pyfulmen: "implemented"
  tsfulmen: "in-progress"
```

**Rationale**: Provides ecosystem-wide visibility into implementation progress, enables adoption matrices.

**Note**: This applies to ecosystem ADRs only. Local ADRs can omit the `adoption` block entirely since they apply to a single library. Libraries should not feel compelled to track peer adoption unless specifically coordinating cross-library work.

**5. Promotion Path**

Local ADRs can be promoted to ecosystem ADRs when cross-language impact is discovered. Original local ADR is marked as superseded with link to new ecosystem ADR.

**Rationale**: Allows organic discovery of cross-cutting concerns without requiring upfront coordination.

## Rationale

### Why Two Tiers?

**Single Tier (All in Crucible)** would force all decisions through centralized governance, slowing down language-specific innovation.

**No Central Tier** would lead to duplicated decisions across libraries and difficulty maintaining cross-language consistency.

**Two Tiers** balances autonomy (libraries control local decisions) with consistency (ecosystem decisions synced everywhere).

### Why Schema-Driven?

Inspiration from ThoughtWorks Technology Radar's maturity model, adapted for architectural decisions. Sort order enables:

```typescript
// Filter production-ready ADRs
const productionReady = adrs.filter(
  (adr) => adr.status.sortOrder >= 30 && adr.status.sortOrder <= 40,
);

// Check if ADR is deprecated
if (adr.status.sortOrder >= 50) {
  console.warn(`ADR ${adr.id} is ${adr.status} - migration recommended`);
}
```

### Why Consistent Naming?

Original proposal used `XXXX-title.md` for local ADRs to distinguish them. After team feedback:

- Tooling simplicity: Single pattern for all ADRs
- Discoverability: `git ls-files | grep ADR-` finds everything
- Consistency: No need to remember which format for which tier
- Location already provides clear distinction

## Alternatives Considered

### Alternative 1: Single Repository for All ADRs

**Description**: Maintain all ADRs (ecosystem and local) in Crucible, using directory structure to separate.

**Pros**:

- Single source of truth for all decisions
- Easier cross-ADR searching
- No sync complexity for local ADRs

**Cons**:

- Library teams lose autonomy over local decisions
- Requires PRs to Crucible for language-specific concerns
- Pollutes ecosystem ADR namespace with implementation details
- Slows down local iteration

**Decision**: Rejected - Violates principle of library autonomy

### Alternative 2: No Ecosystem ADRs

**Description**: Each library maintains all its own ADRs, including those implementing cross-language patterns.

**Pros**:

- Maximum library autonomy
- No central coordination needed
- Simple governance

**Cons**:

- Inconsistent implementations of cross-language patterns
- No visibility into which libraries have adopted ecosystem patterns
- Difficult to track breaking changes
- Manual effort to keep libraries in sync

**Decision**: Rejected - Defeats purpose of SSOT architecture

### Alternative 3: Separate Numbering (ADR- vs Local)

**Description**: Ecosystem ADRs use `ADR-XXXX` prefix, local use `XXXX` only.

**Pros**:

- Visual distinction in filenames
- Slightly shorter local filenames

**Cons**:

- Two patterns to remember
- Harder tooling (need two glob patterns)
- Less discoverable (local ADRs don't show up in `find . -name "ADR-*"`)
- Confusing when both exist in same workspace

**Decision**: Rejected - Consistency and tooling simplicity outweigh marginal brevity

### Alternative 4: No Lifecycle Schema

**Description**: Free-form status values like "draft", "active", "old".

**Pros**:

- Simpler to get started
- No schema maintenance
- Flexible wording

**Cons**:

- No comparative operations
- Inconsistent terminology across libraries
- Can't build tooling (dashboards, filters, validators)
- Unclear progression (what comes after "draft"?)

**Decision**: Rejected - Schema-driven approach enables automation and consistency

## Consequences

### Positive

- ✅ **Clear Governance**: Ecosystem decisions centralized in Crucible, local decisions in libraries
- ✅ **Automatic Sync**: Ecosystem ADRs propagate to all libraries via existing sync infrastructure
- ✅ **Adoption Visibility**: Per-library tracking enables ecosystem-wide dashboards
- ✅ **Promotion Path**: Organic discovery of cross-cutting concerns
- ✅ **Tooling Enablement**: Schemas enable CI validation, automated indexes, adoption matrices
- ✅ **Consistent Naming**: Single pattern for all ADRs simplifies discovery and tooling
- ✅ **Sort Order Support**: Enables comparative operations and maturity filtering

### Negative

- ⚠️ **Schema Maintenance**: Three schemas need to be maintained and versioned
- ⚠️ **Learning Curve**: Teams need to understand two-tier system and when to use each
- ⚠️ **Frontmatter Overhead**: Ecosystem ADRs have more required fields than simple markdown
- ⚠️ **Sync Discipline Required**: Libraries must run `make sync` to get latest ecosystem ADRs

### Neutral

- ℹ️ **Numbering Independence**: Same number (e.g., ADR-0003) can exist in Crucible and each library
- ℹ️ **Location-Based Scope**: Where file lives determines scope, not filename format
- ℹ️ **Migration from Existing**: pyfulmen already has ADRs, will need to integrate with ecosystem ADRs

## Implementation

### Files Created

**Schemas** (`schemas/config/standards/v1.0.0/`):

1. `adr-lifecycle-status.json` - Lifecycle stages with sort order
2. `adr-adoption-status.json` - Adoption statuses with sort order
3. `adr-frontmatter.schema.json` - Complete frontmatter validation

**Documentation** (`docs/architecture/decisions/`):

1. `template.md` - Standard ADR template
2. `ADR-0001-two-tier-adr-system.md` - This ADR

**Updated**:

1. `docs/architecture/fulmen-helper-library-standard.md` - Added ADR section

### Validation

- ✅ Schemas validate against JSON Schema draft-2020-12
- ✅ Template conforms to frontmatter schema
- ✅ Sync successfully propagates to all language wrappers
- ✅ All tests pass (`make check-all`)

### Next Steps

1. **Phase 1** (Immediate): Libraries run `make sync` to receive schemas and template
2. **Phase 2** (Week 1): Each library creates `docs/development/adr/README.md`
3. **Phase 3** (Week 2): Backfill key ecosystem ADRs (catalog strategy, logging profiles, config hydration)
4. **Phase 4** (Ongoing): Libraries document local decisions, promote when needed

## Cross-Language Coordination

### Implementation by Library

**gofulmen**:

- Create `docs/development/adr/` structure
- Write README referencing ecosystem ADRs
- Begin documenting Go-specific decisions

**pyfulmen**:

- Integrate existing ADRs with two-tier system
- Link to ecosystem ADRs from local README
- Continue local decision documentation

**tsfulmen**:

- Establish ADR infrastructure
- Reference ecosystem patterns
- Document TypeScript-specific approaches

**rsfulmen/csfulmen**:

- Will adopt when libraries become active
- Infrastructure already in place via sync

### Validation Approach

- CI validates frontmatter against schema
- Pre-commit hooks check ADR filename format
- Quarterly audits of adoption status accuracy
- Annual review of lifecycle progression

## References

- [ADR pattern by Michael Nygard](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [ADR GitHub organization](https://adr.github.io/)
- [ThoughtWorks Technology Radar](https://www.thoughtworks.com/radar) - Maturity model inspiration
- Implementation brief: `.plans/active/2025.10.2/library-adr-brief.md`
- [Fulmen Helper Library Standard](../fulmen-helper-library-standard.md)
- [Crucible Sync Model](../sync-model.md)

## Revision History

| Date       | Status Change | Summary                                         | Updated By          |
| ---------- | ------------- | ----------------------------------------------- | ------------------- |
| 2025-10-15 | → accepted    | Initial implementation with all teams' approval | @pipeline-architect |

---

**Schema Conformance**: This ADR conforms to [ADR Frontmatter Schema v1.0.0](https://schemas.fulmenhq.dev/standards/adr-frontmatter-v1.0.0.json)
