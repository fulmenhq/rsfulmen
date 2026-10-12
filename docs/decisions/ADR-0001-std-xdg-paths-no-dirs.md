---
id: "ADR-0001"
title: "Replace dirs with a std XDG/home wrapper"
status: "accepted"
date: "2026-10-12"
last_updated: "2026-10-12"
deciders:
  - "@3leapsdave"
  - "Wright"
scope: "rsfulmen config paths / supply-chain licenses"
tags:
  - "config"
  - "xdg"
  - "dependencies"
  - "licenses"
relates-to:
  - "crucible Fulmen Config Path Standard"
  - "3leaps/crucible ADR-0003 (*DR taxonomy)"
---

# ADR-0001: Replace `dirs` with a std XDG/home wrapper

## Status

**Accepted.** Dave ruled that `dirs` must be replaced so `deny.toml` does not
keep a long-term `MPL-2.0` allow. The thin wrapper is the clear winner among
the options below.

## Context

`dirs 5` → `dirs-sys` → `option-ext 0.2.0` is **MPL-2.0 only**. Estate policy
is MIT/Apache-compatible licenses; no copyleft (including MPL). rsfulmen only
needs three homes (`config`, `data`, `cache`) already specified by the Fulmen
Config Path Standard.

## Options considered

| Option | License | Cross-platform | Notes |
| ------ | ------- | -------------- | ----- |
| Keep `dirs` + `MPL-2.0` allow | MPL via `option-ext` | yes | Rejected. Dave: do not keep MPL long-term. |
| `directories` | same `dirs-sys` / `option-ext` stack | yes | Same MPL edge. |
| `etcetera` | MIT OR Apache-2.0 | yes | Extra crate. Apple strategy uses `~/Library/Preferences`, which would **change** macOS config vs Fulmen's `Application Support` table unless we force XDG-everywhere. MSRV 1.87 on 0.11. |
| `xdg` / `xdge` | MIT OR Apache-2.0 | Unix only | No Windows/macOS native fallbacks. |
| Estate crate (`fulmenhq/*`, `3leaps/*`, sysprims) | — | — | No XDG/home crate found that helps this must-have. Do not add speculatively. |
| **Thin std wrapper** | MIT (this crate) | yes | Zero new deps. Matches gofulmen's env+home helpers and the Fulmen platform table. |

## Decision

Implement `src/config/xdg.rs` with `std` only:

1. Absolute `XDG_CONFIG_HOME` / `XDG_DATA_HOME` / `XDG_CACHE_HOME` win (relative
   or empty values ignored, per XDG).
2. Else platform fallbacks from the Fulmen Config Path Standard:
   Unix `~/.config`, `~/.local/share`, `~/.cache`; macOS Application Support /
   Caches; Windows `%APPDATA%` / `%LOCALAPPDATA%`.
3. Missing home falls back to `"."` (same as the previous `dirs` unwrap).
4. Drop the `dirs` dependency and remove `MPL-2.0` from `deny.toml`.

Public APIs (`get_xdg_base_dirs`, `get_app_*_dir`, `get_fulmen_*_dir`) do not
change shape.

## Consequences

- `option-ext` leaves the graph. `cargo deny check licenses` no longer needs MPL.
- Config feature is std-only.
- Windows uses `APPDATA` / `LOCALAPPDATA` rather than Known Folder COM APIs.
  That matches the Fulmen table; it is slightly less precise than `dirs-sys` if
  those env vars are unset (home-relative `AppData\Roaming` / `AppData\Local`
  fallback).
- XDG overrides now apply on macOS/Windows when set to an absolute path. That
  follows Fulmen principle 1 and is a deliberate, small behavior change vs
  `dirs` (which ignored XDG on those platforms).

## References

- [Fulmen Config Path Standard](../crucible-rs/standards/config/fulmen-config-paths.md)
- gofulmen `config/xdg.go` (env + `$HOME` helpers; Unix-shaped)
- [SecDR-0001](SecDR-0001-r-efi-permissive-or.md) (remaining dual-license OR)
