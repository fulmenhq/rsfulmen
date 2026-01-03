---
title: "Fulmen Repository Lifecycle Standard"
description: "Canonical lifecycle phases and quality expectations for every Fulmen codebase"
author: "Schema Cartographer"
date: "2025-10-12"
last_updated: "2025-10-12"
status: "draft"
tags: ["standards", "lifecycle", "quality", "coverage"]
---

# Repository Lifecycle Standard

Every Fulmen code surface MUST declare its maturity stage so that automation, documentation, and release workflows can make consistent decisions. This standard defines the lifecycle phases, the required metadata, and the quality thresholds attached to each phase.

## Canonical Metadata

- **File**: `LIFECYCLE_PHASE` (root-level text file containing a single phase string)
- **Schema**: `schemas/config/repository/v1.0.0/lifecycle-phase.json`
- **Value**: One of `experimental`, `alpha`, `beta`, `rc`, `ga`, `lts`

Repositories MAY surface the phase in additional metadata (badges, docs, build info), but `LIFECYCLE_PHASE` is the single source of truth consumed by tooling.

### Phase Definitions & Quality Bars

| Phase          | Description                                                   | Minimum Coverage\* | Expectations & Notes                                                        |
| -------------- | ------------------------------------------------------------- | ------------------ | --------------------------------------------------------------------------- |
| `experimental` | Initial exploration; APIs unstable, breaking changes expected | 0%                 | No external consumers; tooling may skip quality gates.                      |
| `alpha`        | Early adopters; rapidly evolving features                     | 30%                | Document major gaps; CI should at least build and run smoke tests.          |
| `beta`         | Feature-complete; stabilizing behavior                        | 60%                | CRITICAL bugs addressed promptly; documentation kept current.               |
| `rc`           | Release candidate; locking down for GA                        | 70%                | No known blocking defects; only risk mitigation changes allowed.            |
| `ga`           | General availability; production ready                        | 75%                | Follows published release process; full test suite required; docs polished. |
| `lts`          | Long-term support; critical fixes only                        | 80%                | Security fixes prioritized; maintain backward compatibility guarantees.     |

\*Coverage thresholds represent the **minimum** required by CI/Makefile gates; projects may exceed them. Additional quality gates (lint, type-check, security scans) MUST align with the lifecycle phase.

## Usage Requirements

1. **Maintain `LIFECYCLE_PHASE`** in the repository root. Update the value when the project graduates or changes posture.
2. **CI & Makefiles** MUST read `LIFECYCLE_PHASE` to determine test coverage gates and other protections.
3. **README/Docs** SHOULD reference the current phase to set contributor/user expectations.
4. **Release Notes** SHOULD mention lifecycle transitions (e.g., “project entering GA”).

## Relationship to Release Phase

Lifecycle phase and release phase serve distinct purposes:

| Aspect      | Lifecycle Phase                                         | Release Phase                                     |
| ----------- | ------------------------------------------------------- | ------------------------------------------------- |
| **Tracks**  | Product/code maturity                                   | Deployment cycle state                            |
| **Changes** | Rarely (project matures over time)                      | Frequently (each release cycle)                   |
| **Scope**   | Repository-level                                        | Branch/release-level                              |
| **File**    | `LIFECYCLE_PHASE`                                       | Typically in CI/tooling config                    |
| **Schema**  | `schemas/config/repository/v1.0.0/lifecycle-phase.json` | `schemas/config/goneat/v1.0.0/release-phase.json` |
| **Values**  | experimental, alpha, beta, rc, ga, lts                  | dev, rc, ga, release, hotfix                      |

**Note:** `release` is semantically equivalent to `ga` in release phase. Tooling normalizes `release` → `ga` for comparison. The `hotfix` value indicates urgent production fix state.

### Migration Guidance

- `RELEASE_PHASE` files are deprecated for new repositories. Use CI/tooling configuration or SemVer prerelease suffixes to signal release state.
- Existing repositories using `RELEASE_PHASE` files MAY keep them temporarily, but MUST maintain `LIFECYCLE_PHASE` as the source of truth for maturity.
- For smaller projects that don't change release phase frequently, `ga` or `release` is typically sufficient.

## Tooling Integration

- Refer to the schema at `schemas/config/repository/v1.0.0/lifecycle-phase.json` for validation.
- Example Makefile snippet:
  ```makefile
  LIFECYCLE := $(shell cat LIFECYCLE_PHASE 2>/dev/null || echo experimental)
  # map lifecycle to coverage thresholds
  ```
- Goneat and future CLI tooling SHOULD expose commands to inspect lifecycle state (e.g., `goneat repo lifecycle`).

## Change Control

- New lifecycle values or schema changes require approval from the Fulmen architecture group (@schema-cartographer, @3leapsdave).
- Update the schema and this document together to keep automation and documentation aligned.

---

_Generated by Schema Cartographer under supervision of @3leapsdave._
