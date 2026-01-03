---
title: "Module Telemetry Coordination SOP"
description: "Process for coordinating metrics taxonomy updates across language libraries"
author: "Schema Cartographer"
date: "2025-10-24"
status: "approved"
category: "sop"
tags: ["telemetry", "metrics", "taxonomy", "coordination", "cross-library"]
---

# Module Telemetry Coordination SOP

## Purpose

This SOP defines the process for adding new metrics to Crucible's canonical taxonomy when implementing telemetry-enabled modules in language libraries (gofulmen, pyfulmen, tsfulmen).

**Why This Matters**: The metrics taxonomy (`config/taxonomy/metrics.yaml`) is the single source of truth for metric identifiers. Schema validation enforces this via `$ref`, so metrics must be added to taxonomy BEFORE implementation.

## Scope

Applies to:

- Module implementations that emit telemetry
- Extension of existing modules with new metrics
- Cross-library telemetry standardization

Does NOT apply to:

- Application-specific metrics (not in helper library scope)
- Metrics consumed but not emitted by libraries

## Process

### Phase 1: Before Implementation (Library Team)

**Trigger**: Planning to implement module with telemetry

**Actions**:

1. **Draft Metrics List**
   - Identify all metrics the module will emit
   - Follow naming conventions:
     - Format: `module_operation_unit`
     - Examples: `pathfinder_find_ms`, `config_load_errors`
   - Use standard units: `count`, `ms`, `bytes`, `percent`
   - Use semantic suffixes:
     - `_ms` - Duration measurements
     - `_errors` - Failure counts
     - `_warnings` - Non-fatal issue counts
     - `_count` - Success/total counts

2. **Create Request Memo** (Temporary, in `.plans/`)
   - Location: `.plans/active/libraries/YYYYMMDD-<module>-metrics-request.md`
   - Include:
     - List of proposed metrics (name, unit, description)
     - Business justification for each metric
     - Module context (core vs extension)
     - Impact assessment on other language libraries
   - Template available in this document (see Appendix)

3. **Share with Teams**
   - Post memo location to library maintainer channels
   - Request feedback from:
     - All language library teams (gofulmen, pyfulmen, tsfulmen)
     - Schema Cartographer
     - @3leapsdave (final approval)
   - Set 24-48 hour feedback window

**Deliverable**: Request memo with team feedback

---

### Phase 2: Crucible Update (Schema Cartographer)

**Trigger**: Request approved by teams + @3leapsdave

**Actions**:

1. **Update Taxonomy**
   - File: `config/taxonomy/metrics.yaml`
   - Add metrics to `$defs.metricName.enum` section
   - Add metric definitions to `metrics` array
   - Maintain alphabetical grouping by module prefix

2. **Validate Schema**
   - Run `bun run scripts/validate-schemas.ts`
   - Verify no schema errors
   - Confirm `$ref` resolution works

3. **Run Precommit**
   - Execute `make precommit`
   - Ensure all quality gates pass
   - Verify sync to lang wrappers completes

4. **Commit & Sync**
   - Commit taxonomy changes with attribution
   - Include reference to request memo
   - Run `make sync` if not included in precommit

**Deliverable**: Updated taxonomy synced to all language wrappers

---

### Phase 3: Library Sync (Requesting Library Team)

**Trigger**: Crucible taxonomy updated

**Actions**:

1. **Pull Latest Crucible**
   - In library repo: `make sync` or `goneat ssot sync`
   - Verify new metrics appear in `docs/crucible-<lang>/config/taxonomy/metrics.yaml`

2. **Implement Module**
   - Use approved metric names (exact match required)
   - Emit metrics via library's telemetry module
   - Schema validation will pass on first try

3. **Validate**
   - Run library tests with telemetry enabled
   - Verify schema validation passes for emitted metrics
   - Test metric recording and export

4. **Document**
   - Update module API reference with metrics list
   - Note which metrics are emitted when
   - Link to Crucible taxonomy for canonical definitions

**Deliverable**: Module implementation with validated telemetry

---

### Phase 4: Other Libraries (Optional)

**Trigger**: When implementing same module in other languages

**Actions**:

1. **Check Taxonomy**
   - Verify metrics already exist in Crucible taxonomy
   - Pull latest Crucible if stale

2. **Implement**
   - Use existing metric names (no new taxonomy update needed)
   - Follow same validation steps as requesting library

**Deliverable**: Cross-language telemetry parity

---

## Naming Conventions

### Metric Name Format

```
<module>_<operation>_<unit_suffix>
```

**Examples**:

- `pathfinder_find_ms` - Pathfinder module, find operation, milliseconds
- `config_load_errors` - Config module, load operation, error count
- `schema_validations` - Schema module, validations, count (implied)

### Standard Unit Suffixes

| Suffix      | Unit         | Meaning             | Example                        |
| ----------- | ------------ | ------------------- | ------------------------------ |
| `_ms`       | milliseconds | Duration            | `pathfinder_find_ms`           |
| `_errors`   | count        | Failure count       | `config_load_errors`           |
| `_warnings` | count        | Non-fatal issues    | `pathfinder_security_warnings` |
| `_count`    | count        | Total/success count | `foundry_lookup_count`         |
| _(none)_    | count        | Implied count       | `schema_validations`           |

### Standard Units (Taxonomy Enum)

- `count` - Dimensionless counts
- `ms` - Milliseconds (duration)
- `bytes` - Data size
- `percent` - Percentage (0-100)

---

## Roles & Responsibilities

### Library Team (Requestor)

- ✅ Draft metrics list
- ✅ Create request memo
- ✅ Gather team feedback
- ✅ Pull updated taxonomy after approval
- ✅ Implement with validated metrics

### Schema Cartographer

- ✅ Review request for naming convention compliance
- ✅ Update taxonomy in Crucible
- ✅ Run validation and precommit
- ✅ Commit with proper attribution

### Other Library Teams

- ✅ Provide feedback on request (24-48h window)
- ✅ Flag conflicts or concerns
- ⚠️ Not required to implement metrics (but encouraged for parity)

### @3leapsdave

- ✅ Final approval authority
- ✅ Architectural guidance on metric scope

---

## Common Scenarios

### Scenario 1: New Extension Module (Pathfinder)

**Example**: GoFulmen implementing Pathfinder extension

1. GoFulmen drafts 4 metrics: `pathfinder_find_ms`, `pathfinder_validation_errors`, `pathfinder_security_warnings`, `config_load_errors`
2. Creates memo in `.plans/active/libraries/20251024-pathfinder-metrics-request.md`
3. Shares with teams, gathers feedback (48h)
4. Schema Cartographer updates taxonomy (30 min)
5. GoFulmen syncs Crucible, implements module (days)
6. PyFulmen/TSFulmen can use same metrics when implementing Pathfinder later

### Scenario 2: Extending Existing Module

**Example**: Adding error tracking to existing Config module

1. Library team identifies need for `config_load_errors` metric
2. Checks if `config_load_ms` already exists (yes)
3. Requests addition of paired `_errors` metric
4. Schema Cartographer adds to taxonomy near existing `config_load_ms`
5. All libraries can immediately use new metric

### Scenario 3: Cross-Library Coordination

**Example**: PyFulmen implements Pathfinder after GoFulmen

1. PyFulmen checks taxonomy: Pathfinder metrics already exist ✅
2. No new taxonomy request needed
3. PyFulmen syncs Crucible, uses existing metric names
4. Telemetry parity achieved automatically

---

## Common Mistakes to Avoid

### ❌ Implement First, Request Later

**Problem**: Schema validation fails during implementation

**Solution**: Request taxonomy update BEFORE writing code

### ❌ Use Custom Metric Names

**Problem**: Metrics not in taxonomy get rejected by schema

**Solution**: Use exact names from approved taxonomy

### ❌ Assume Metrics Exist

**Problem**: Library X's metrics may not be in pyfulmen's stale Crucible sync

**Solution**: Always `make sync` before implementing telemetry

### ❌ Skip Team Feedback

**Problem**: Metric names conflict with other libraries' plans

**Solution**: Follow 24-48h feedback window

---

## Timeline Expectations

| Phase                | Owner               | Duration                         |
| -------------------- | ------------------- | -------------------------------- |
| Draft metrics list   | Library team        | 30 min - 2 hours                 |
| Create request memo  | Library team        | 30 min                           |
| Team feedback window | All teams           | 24-48 hours                      |
| Taxonomy update      | Schema Cartographer | 30 min                           |
| Library sync         | Library team        | 5 min                            |
| Implementation       | Library team        | Hours to days (module-dependent) |

**Total**: ~2-3 days from request to implementation start

---

## Appendix: Request Memo Template

```markdown
---
title: "<Module> Metrics Taxonomy Update Request"
description: "Adding metrics for <module> telemetry to Crucible taxonomy"
author: "<Library Team>"
date: "YYYY-MM-DD"
status: "pending-approval"
priority: "high|medium|low"
blocking: ["<library>-<phase>"]
audience: ["gofulmen", "pyfulmen", "tsfulmen"]
---

# <Module> Metrics Taxonomy Update Request

**Requestor**: <Library Team>  
**Priority**: <High|Medium|Low> - <Reason>  
**Status**: Pending Team Approval

## Requested Metrics

| Metric Name | Unit     | Description   | Module   |
| ----------- | -------- | ------------- | -------- |
| `<name>`    | `<unit>` | <description> | <module> |

## Business Justification

1. **`<metric_1>`**: <Why needed>
2. **`<metric_2>`**: <Why needed>

## Impact Assessment

- **<library1>**: <Impact>
- **<library2>**: <Impact>

## Approval

Reply with:
```

Team: <library>
Status: APPROVED / APPROVED_WITH_CHANGES / BLOCKED
Comments: <optional>

```

**Required Approvals**: All library teams + @3leapsdave
```

---

## References

- [Helper Library Standard](../architecture/fulmen-helper-library-standard.md) § Module Implementation Guidelines
- [Telemetry & Metrics Standard](../standards/library/modules/telemetry-metrics.md)
- [Metrics Taxonomy](../../config/taxonomy/metrics.yaml)
- [ADR-0007: Default Histogram Buckets](../architecture/decisions/ADR-0007-telemetry-default-histogram-buckets.md)

---

## Revision History

| Date       | Author              | Changes                                          |
| ---------- | ------------------- | ------------------------------------------------ |
| 2025-10-24 | Schema Cartographer | Initial SOP based on gofulmen Pathfinder request |

---

**Maintainer**: Schema Cartographer (@schema-cartographer)  
**Review Cycle**: Quarterly or as needed when process gaps identified
