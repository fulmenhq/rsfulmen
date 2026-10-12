---
id: "SecDR-0001"
title: "Accept r-efi via the MIT/Apache side of its SPDX OR"
status: "accepted"
date: "2026-10-12"
last_updated: "2026-10-12"
deciders:
  - "@3leapsdave"
  - "Wright"
scope: "rsfulmen supply-chain license posture"
tags:
  - "licenses"
  - "cargo-deny"
  - "r-efi"
relates-to:
  - "3leaps/crucible ADR-0003 (*DR taxonomy; SecDR not SDR)"
  - "rsfulmen ADR-0001 (dirs/MPL removal)"
---

# SecDR-0001: Accept `r-efi` via the MIT/Apache side of its SPDX OR

## Status

**Accepted.** Dave: the permissive OR side is allowed **only if** the pinned
versions are 100% MIT/Apache-choosable.

## Context

`cargo deny list` reports `LGPL-2.1-or-later` on `r-efi` 5.3.0 and 6.0.0.
Those crates are transitive (`getrandom` → `r-efi` on UEFI-capable targets).
Estate policy forbids copyleft (GPL/LGPL/AGPL/MPL/…). A dual-license crate is
acceptable only when a consumer can choose MIT or Apache-2.0 with no LGPL
obligation.

## Evidence (pinned lockfile versions)

| Crate | Lock pin | crates.io / crate SPDX |
| ----- | -------- | ---------------------- |
| `r-efi` | 5.3.0 | `MIT OR Apache-2.0 OR LGPL-2.1-or-later` |
| `r-efi` | 6.0.0 | `MIT OR Apache-2.0 OR LGPL-2.1-or-later` |

This is a true SPDX **OR**, not AND. rsfulmen (MIT) elects **MIT** (Apache-2.0
is equally allowed). No LGPL source is required, and `deny.toml` does **not**
allow `LGPL-2.1-or-later`. cargo-deny accepts the crate because one allowed
license in the OR expression is sufficient.

`r-efi` has no further third-party deps in the lockfile (optional
`rustc-std-workspace-core` only when built as rustc-dep-of-std).

## Decision

1. Keep the pinned `r-efi` 5.3.0 and 6.0.0 versions.
2. Do not add LGPL (or MPL) to the cargo-deny allow list.
3. Re-check this SecDR if a newer `r-efi` drops the MIT/Apache alternatives or
   switches to AND.

## Consequences

- License policy stays MIT/Apache-compatible.
- UEFI-only `r-efi` types do not change the runtime license of Unix/Windows
  builds; the crate remains in the graph as a `getrandom` target-cfg dep.

## References

- https://crates.io/crates/r-efi/5.3.0
- https://crates.io/crates/r-efi/6.0.0
- https://spdx.org/licenses/ (OR vs AND)
