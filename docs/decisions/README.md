# Decisions

This directory is **this repo's chosen home** for decision and governance
records. Types, naming, and lifecycle follow
[3leaps/crucible ADR-0003](https://github.com/3leaps/crucible/blob/main/docs/decisions/ADR-0003-decision-record-taxonomy.md)
(the `*DR` family). The standard mandates no storage location;
`docs/decisions/` is the home rsfulmen picked.

Do not invent a parallel taxonomy. The supported type set is
`{ADR, DDR, SecDR, PDR, EPR}`. `SDR` is not a type (renamed to **SecDR**).

Local Rust-only notes that predate this lane remain under
[`docs/development/adr/`](../development/adr/). New records go here.

## Naming

`{TYPE}-{NNNN}-{kebab-title}.md` — 4-digit, zero-padded, **per type**,
repo-global per type. Numbers are never reused.

## Status

`proposed` → `accepted` → (`superseded` | `rejected` | `withdrawn`)

## Index

| ID | Title | Status | Date |
| -- | ----- | ------ | ---- |
| [ADR-0001](ADR-0001-std-xdg-paths-no-dirs.md) | Replace `dirs` with a std XDG/home wrapper | accepted | 2026-10-12 |
| [SecDR-0001](SecDR-0001-r-efi-permissive-or.md) | Accept `r-efi` via MIT/Apache side of SPDX OR | accepted | 2026-10-12 |
