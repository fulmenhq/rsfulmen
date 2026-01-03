---
id: "ADR-0011"
title: "Temporary SemVer Flexibility for L'Orage Central DevSecOps Schemas"
status: "experimental"
date: "2025-11-10"
last_updated: "2025-11-10"
deciders:
  - "@3leapsdave"
  - "@schema-cartographer"
scope: "Ecosystem"
superseded_by: ""
deprecation_date: ""
tags:
  - "lorage-central"
  - "devsecops"
  - "schemas"
  - "semver"
  - "experimental"
related_adrs:
  - "ADR-0010"
adoption:
  gofulmen: "in-progress"
  pyfulmen: "planned"
  tsfulmen: "planned"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-0011: Temporary SemVer Flexibility for L'Orage Central DevSecOps Schemas

## Status

**Current Status**: Experimental

**Expiration Condition**: This ADR expires and will be superseded once L'Orage Central CLI reaches v0.2.0. At that milestone, all schemas will transition to standard Crucible SemVer discipline.

## Context

L'Orage Central is a new hybrid CLI/REPL/server workhorse for secure, schema-driven DevSecOps orchestration (see [Fulmen Forge Workhorse Standard](../fulmen-forge-workhorse-standard.md)). It introduces a new suite of schemas and configuration defaults under `devsecops/lorage-central/*` along with new taxonomies (`auth-methods`, `geo`, `infra-phases`, `infra-providers`).

### The Iteration Challenge

During the L'Orage Central application build (targeting v0.1.0 MVP), the following schema areas require rapid iteration:

- **Policy schemas**: Session TTL, MFA requirements, tenant isolation rules
- **Recipe schemas**: IaC orchestration DSL structure and validation
- **Runbook schemas**: Serialization format for procedural operations
- **Tenant schemas**: Registry structure and data sensitivity guards
- **Activity schemas**: Audit record formats for observability
- **Credential schemas**: Secrets reference patterns
- **New taxonomies**: Auth methods, geo regions, infra phases/providers

The CLI is not yet public and has no external consumers. Enforcing strict Crucible SemVer rules (additive-only changes, deprecation cycles for removals) would significantly slow experimentation and refinement of these DevSecOps abstractions.

### Scope of Experimental Status

This flexibility applies **only** to:

- `schemas/devsecops/lorage-central/**/v1.0.0/`
- `config/devsecops/lorage-central/**/v1.0.0/`
- `schemas/taxonomy/devsecops/auth-methods/v1.0.0/`
- `schemas/taxonomy/devsecops/geo/v1.0.0/`
- `schemas/taxonomy/devsecops/infra-phases/v1.0.0/`
- `schemas/taxonomy/devsecops/infra-providers/v1.0.0/`
- `config/taxonomy/devsecops/` (mirroring schema structure)

All other Crucible schemas and configurations continue to follow standard SemVer discipline per [ADR-0010: Semantic Versioning Adoption](./ADR-0010-semantic-versioning-adoption.md).

**Rationale for DevSecOps Taxonomy Grouping**: The new taxonomies are domain-specific to infrastructure orchestration and security concerns, distinguishing them from ecosystem-wide taxonomies (`language`, `repository-category`). This grouping provides:

- Clear domain boundaries and ownership
- Scalability for future DevSecOps taxonomies
- Namespace collision protection
- Reusability signal for other infrastructure/security tooling beyond L'Orage Central

## Decision

**Grant temporary SemVer flexibility for L'Orage Central DevSecOps schemas until the CLI reaches v0.2.0 stabilization milestone.**

### Specific Provisions

1. **Mark schemas as experimental**: All affected schemas include `status: experimental` in YAML frontmatter and reference this ADR (e.g., `# Per ADR-0011-lorage-devsecops-schemas-semver-experimental`)

2. **Allow breaking changes to v1.0.0**: Breaking changes (field removals, type changes, constraint tightening) are permitted within the v1.0.0 version during the experimental window

3. **Maintain change documentation**: Every breaking change must be documented in the Crucible CHANGELOG.md with `[EXPERIMENTAL]` prefix, even during the flexible period

4. **Restrict consumer adoption**: No FulmenHQ applications other than L'Orage Central should depend on these schemas until the experimental status is removed

5. **Clear termination criteria**:
   - **Trigger**: L'Orage Central CLI ships v0.2.0 (after public alpha cycle at v0.1.x)
   - **Action**: Remove `status: experimental` from all affected schemas
   - **Effect**: All subsequent changes follow standard Crucible SemVer rules (additive changes only, deprecation cycles for breaking changes)

6. **ADR retirement**: Once experimental status is removed, this ADR will be marked `status: superseded` with supersession date noted

### What This Enables

- Iterative refinement of policy DSL during REPL development
- Schema field renames based on CLI usability testing
- Taxonomy adjustments as infrastructure patterns emerge
- Recipe structure evolution as IaC abstractions are validated

### What This Does NOT Enable

- Silent breaking changes (all must be documented)
- Experimental status for schemas outside the specified scope
- Indefinite flexibility (strict termination at v0.2.0)
- Public API instability after L'Orage Central stabilization

## Rationale

### Why Experimental Status is Appropriate

1. **No External Consumers**: L'Orage Central is under active development with no public releases. The only consumer is internal development, where breaking changes are manageable.

2. **Schema-Driven Development Requires Iteration**: DevSecOps abstractions (policies, recipes, runbooks) benefit from tight feedback loops between schema design and CLI implementation. Strict versioning creates artificial barriers.

3. **Clear Stabilization Target**: The v0.2.0 milestone provides an unambiguous point where "experimentation ends, stability begins." This aligns with L'Orage Central's public alpha completion.

4. **Documented Trail Preserved**: Requiring CHANGELOG entries maintains visibility into evolution without blocking rapid iteration.

5. **Isolated Blast Radius**: Restricting experimental status to lorage-specific schemas protects the broader Crucible ecosystem from instability.

### Why This is Safe

- L'Orage Central is the sole consumer during the experimental window
- Foundation helpers (gofulmen, pyfulmen, tsfulmen) will not integrate these schemas until experimental status is removed
- All changes remain traceable through CHANGELOG and git history
- EA Steward oversight ensures no ecosystem-wide schemas inadvertently gain experimental status

## Alternatives Considered

### Alternative 1: Strict SemVer from Day One

**Description**: Require full SemVer compliance (v1.0.0, v2.0.0, etc.) for all lorage schemas from initial commit.

**Pros**:

- Uniform versioning policy across all Crucible schemas
- No special cases to track
- Immediate stability signal to potential consumers

**Cons**:

- **Slows Development**: Every field rename requires a new major version (v1 → v2 → v3 rapidly)
- **Premature Commitment**: Locks in abstractions before CLI validates them with real usage
- **Version Number Inflation**: Could reach v5.0.0 before L'Orage Central ships v0.1.0
- **Coordination Overhead**: Major version bumps require downstream updates even during private development

**Decision**: Rejected - Optimization for stability without consumers creates unnecessary friction.

### Alternative 2: v0.x Versioning for Schemas

**Description**: Use v0.1.0, v0.2.0, v0.3.0 for schemas (SemVer pre-1.0 phase allows breaking changes).

**Pros**:

- Follows SemVer pre-stability conventions
- Permits breaking changes legitimately
- No special "experimental" designation needed

**Cons**:

- **Confusing for Ecosystem**: Crucible schemas typically start at v1.0.0
- **Version Misalignment**: Schema v0.3.0 may be used by CLI v0.1.0 (no clear correlation)
- **Migration Burden**: Eventually need v0.x → v1.0.0 schema migration when stabilizing
- **Signals Instability Longer**: v0.x implies "not ready" even after L'Orage Central stabilizes

**Decision**: Rejected - Creates long-term version management complexity and ecosystem confusion.

### Alternative 3: Separate lorage-schemas Repository

**Description**: Host L'Orage Central schemas in a dedicated repository outside Crucible until stable.

**Pros**:

- Complete independence from Crucible versioning
- No impact on Crucible stability
- Freedom to iterate without ADR process

**Cons**:

- **Violates SSOT Principle**: Crucible is the canonical schema source for FulmenHQ
- **Fragmentation**: DevSecOps schemas scattered across repositories
- **Integration Complexity**: gofulmen must pull from multiple schema sources
- **Duplication Risk**: Taxonomies like geo/auth-methods would need replication
- **Migration Overhead**: Eventually merge into Crucible, creating significant coordination work

**Decision**: Rejected - Undermines Crucible's role as ecosystem SSOT.

### Alternative 4: Fork-Per-Change Versioning

**Description**: Create new schema versions (v1.0.0, v1.1.0, v2.0.0) on every change, but don't enforce gofulmen updates until stabilization.

**Pros**:

- Technically follows SemVer
- Full version history preserved
- No special experimental designation

**Cons**:

- **Coordination Burden**: Every schema change requires version bump coordination
- **Version Explosion**: 20+ schema versions during 3-month development cycle
- **Directory Bloat**: `policy/v1.0.0/`, `policy/v1.1.0/`, `policy/v2.0.0/`, etc.
- **Migration Complexity**: L'Orage Central must constantly update schema references
- **Wasted Effort**: Creating migration paths for versions with zero external consumers

**Decision**: Rejected - Creates process overhead without providing value to anyone.

## Consequences

### Positive

- ✅ **Accelerated Schema Iteration**: Breaking changes can be made quickly based on CLI development feedback
- ✅ **Simplified Development**: L'Orage Central team doesn't fight versioning during rapid prototyping
- ✅ **Clear Stabilization Signal**: v0.2.0 milestone unambiguously ends experimental phase
- ✅ **Paper Trail Maintained**: CHANGELOG preserves evolution history for future reference
- ✅ **Ecosystem Protection**: Other applications explicitly warned not to adopt experimental schemas
- ✅ **ADR Transparency**: Decision documented, time-boxed, and reversible
- ✅ **Foundation Helper Safety**: gofulmen/pyfulmen/tsfulmen can defer integration until stability confirmed

### Negative

- ⚠️ **Two-Tier Schema Governance**: Creates temporary exception to Crucible's uniform versioning policy
  - Mitigated by: Clear scope limitations, automatic expiration, EA oversight
- ⚠️ **Potential for Confusion**: Developers might assume all schemas have experimental flexibility
  - Mitigated by: Explicit `status: experimental` in schema frontmatter, README warnings, ADR references
- ⚠️ **Breaking Change Risk**: Schema changes could inadvertently break gofulmen if boundaries are unclear
  - Mitigated by: gofulmen avoids experimental schemas, EA review gates any integration attempts

### Neutral

- ℹ️ **Temporary by Design**: Experimental period is time-boxed to L'Orage Central development cycle
- ℹ️ **Precedent for Future Workhorses**: Similar pattern could apply to other greenfield workhorse schemas
- ℹ️ **Does Not Affect Existing Schemas**: Library modules (app-identity, telemetry, etc.) unaffected

## Implementation

### Schema Frontmatter Annotations

All experimental schemas include:

```yaml
---
title: Schema Title
description: Brief description
author: Guest Architect Black
date: 2025-11-10
status: experimental  # Per ADR-0011-lorage-devsecops-schemas-semver-experimental
tags: [taxonomy, domain, v1.0.0]
---

# Rationale
Description of schema purpose and experimental status.

# References
- ADR-0011: Temporary SemVer flexibility for L'Orage Central DevSecOps schemas
```

### README Warnings

Each affected schema directory includes README with:

```markdown
# L'Orage Central Schemas

⚠️ **EXPERIMENTAL STATUS**: These schemas are under active development for L'Orage Central (v0.1.x → v0.2.0 cycle).
Breaking changes may occur without version bumps per [ADR-0011](../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md).

**Do not depend on these schemas in production applications** until experimental status is removed (target: L'Orage Central v0.2.0).
```

### CHANGELOG Documentation

Breaking changes during experimental window:

```markdown
## [0.2.9] - 2025-11-10

### Changed (Experimental)

- [EXPERIMENTAL] `devsecops/lorage-central/policy/v1.0.0`: Renamed `session.ttl.ops` to `session.ttl.overrides` (ADR-0011)
- [EXPERIMENTAL] `taxonomy/devsecops/auth-methods/v1.0.0`: Added `impl-langs` required field (ADR-0011)
```

### Stabilization Checklist (v0.2.0 Target)

When L'Orage Central reaches v0.2.0:

1. **Remove Experimental Markers**:
   - Delete `status: experimental` from all schema frontmatter
   - Remove ADR-0011 references from comments
   - Update READMEs to remove experimental warnings

2. **Update ADR-0011**:
   - Change status from `experimental` to `superseded`
   - Add supersession date to frontmatter
   - Document final stabilization in revision history

3. **Document Final State**:
   - CHANGELOG entry noting experimental phase completion
   - List any schemas that were withdrawn (if applicable)
   - Confirm EA Steward sign-off on stabilization

4. **Enable Downstream Adoption**:
   - Notify gofulmen, pyfulmen, tsfulmen teams that schemas are stable
   - Publish integration guide for L'Orage Central schema consumption
   - Update module compliance matrix with adoption status

5. **Enforce Standard SemVer**:
   - All subsequent changes require version bumps per ADR-0010
   - Breaking changes require new major versions (v2.0.0, v3.0.0, etc.)
   - Deprecation cycles required for field removals

## Cross-Language Coordination

### Go (gofulmen)

**Impact**: Required (primary consumer)
**Implementation**: In-progress (v0.1.x development)
**Coordination**:

- gofulmen's crucible shim loads experimental schemas with warning logs
- Schema changes trigger recompilation but no version constraints during experimental window
- After stabilization: Pin to stable schema versions with SemVer constraints

### Python (pyfulmen)

**Impact**: Planned (v0.2.x+)
**Implementation**: Deferred until schemas stabilize
**Coordination**:

- Avoid integration until experimental status removed
- Document lorage schema consumption patterns once stable

### TypeScript (tsfulmen)

**Impact**: Planned (v0.2.x+)
**Implementation**: Deferred until schemas stabilize
**Coordination**:

- Avoid integration until experimental status removed
- Document lorage schema consumption patterns once stable

## References

- [ADR-0010: Semantic Versioning Adoption](./ADR-0010-semantic-versioning-adoption.md) - Standard Crucible versioning policy
- [Fulmen Forge Workhorse Standard](../fulmen-forge-workhorse-standard.md) - L'Orage Central's architectural foundation
- [L'Orage Central Feature Brief](.plans/fulmen-lorage-central/feature-brief.md) - Complete application specification
- [Crucible Schema Normalization Standard](../../standards/schema-normalization.md) - Schema design guidelines

## Revision History

| Date       | Status Change  | Summary                                                       | Updated By           |
| ---------- | -------------- | ------------------------------------------------------------- | -------------------- |
| 2025-11-10 | → experimental | Initial draft and approval                                    | @schema-cartographer |
| 2025-11-10 | clarification  | Added devsecops/ grouping for taxonomy schemas (pre-creation) | @schema-cartographer |

---

**Schema Conformance**: This ADR conforms to [ADR Frontmatter Schema v1.0.0](https://schemas.fulmenhq.dev/standards/adr-frontmatter-v1.0.0.json)
