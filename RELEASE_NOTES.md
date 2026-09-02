# Release Notes

This document tracks release notes for rsfulmen releases.

> **Convention**: Keep only the latest 3 releases here to prevent file bloat. Older releases are archived in `docs/releases/`.

## [0.2.0] - 2026-09-02

### Stamp-only host identity, producer, goneat v0.6.0

**Release Type**: Breaking / Feature Release

#### Overview

Host `version --extended` no longer reports a launcher's `FULMEN_HOST_*` when
compile stamps are missing. Application crates can stamp identity from a
`build.rs` producer. goneat is pinned to v0.6.0. MSRV remains 1.88.

Shipped as **0.2.0** because `resolve()` on 0.1.6 read process environment.
Cargo `^0.1` would pull a 0.1.7 and change that behavior.

Public commits: [#15](https://github.com/fulmenhq/rsfulmen/pull/15) (`5d35eae`),
[#16](https://github.com/fulmenhq/rsfulmen/pull/16) (`6cdcfd9`).

#### Highlights

- **Stamp-only `host_identity!()`** — missing stamps use documented defaults;
  process `FULMEN_HOST_*` is never consulted.
- **Producer** (`host-identity-producer` feature) — `emit_host_identity()` for
  application `build.rs`. Git probe uses `CARGO_MANIFEST_DIR` and strips
  locator/index/object/config overrides.
- **`resolve_from_process_env()`** — explicit runtime-environment API.
- **goneat v0.6.0**; flate2 1.1.10; uuid 1.26.0.

#### Breaking Changes

**`resolve()` no longer reads process `FULMEN_HOST_*`.** It returns documented
defaults when no compile stamps are supplied. Callers that used `resolve()` as
a runtime-env fallback must call `resolve_from_process_env()`.

`host_identity!()` is stamp-only (the intended identity path).

#### Testing

- `make check-all` — 450 unit tests, 5 host-identity forging tests, 76 doc tests
- MSRV 1.88 locked build/test
- clippy `-D warnings` on 1.98.0
- `cargo audit` clean on the 0.2.0 lockfile

#### Requirements

- **Rust**: 1.88+ (MSRV, unchanged)
- **Crucible**: v0.4.19 (embedded)

---

## [0.1.6] - 2026-08-20

### File-backed catalogs, fulpack create, lockfile, host identity

**Release Type**: Feature Release

#### Overview

Adds on-disk JSON Schema instance validation, host `version --extended`, and
reproducible CI. Fulpack create no longer substitutes unsupported checksums.
MSRV remains 1.88. Embedded Crucible is v0.4.19.

#### Highlights

- **File-backed schema catalogs** — validate instances against on-disk schema
  trees (offline `$ref`) without vendoring those trees into the crate embed.
  Unix containment uses `openat(2)` + `O_NOFOLLOW`.
- **Host binary identity** (`buildinfo`) — apps stamp `FULMEN_HOST_*`; the
  library formats basic/extended/JSON. Pins are a second block, never host
  `Commit:`.
- **Fulpack create** — sha512/sha1/md5 → `INVALID_OPTIONS`; gzip always level 6.
- **Lockfile** — `Cargo.lock` is committed; CI uses `--locked`.
- **goneat v0.5.16**; clippy toolchain **1.98.0**.

#### Breaking Changes

None for existing embed-id schema APIs or default feature graph. Fulpack
create now **errors** on unsupported checksum algorithms (previously
substituted sha256). Callers that passed sha512/sha1/md5 must switch to
sha256 or xxh3-128.

#### Testing

- `cargo test --locked --all-features` — 438 unit tests, 76 doc tests
- MSRV 1.88 locked build/test
- clippy `-D warnings` on 1.98.0

#### Requirements

- **Rust**: 1.88+ (MSRV, unchanged)
- **Crucible**: v0.4.19 (embedded)

---

## [0.1.5] - 2026-06-05

### Branch/PR workflow, Crucible v0.4.13, dependency hygiene

**Release Type**: Process & Maintenance Release

#### Overview

A process and maintenance release. rsfulmen adopts a branch/PR development workflow (replacing the original micro-team direct-push model), syncs Crucible v0.4.13, and clears dependency and tooling tech debt. No new modules, no public API changes, no breaking changes.

#### Highlights

- **Development workflow** — Moved from direct-push-to-`main` (micro-team) to a branch/PR workflow. Branch protection to follow.
- **Guardian-free hooks** — Removed the guardian browser-intercept on commit and push (obsolete under branch review); hooks regenerated via `goneat hooks generate`.
- **goneat v0.5.13** — Pin bumped from v0.5.1; resolves a format check/apply engine divergence and an `ssot sync` metadata-indentation issue surfaced during this cycle.
- **Crucible v0.4.13 sync** — `devlead`/`devrev`/`qa` role-prompt enrichment (role v1.0.0 -> v1.0.1), upstream schema slimming, and mechanical YAML normalization.
- **serde_yaml_ng migration** — Migrated off the deprecated/archived `serde_yaml` to the maintained, API-compatible `serde_yaml_ng` 0.10 (aliased; no source changes).
- **`.goneatignore`** — Excludes deliberately-malformed negative-test fixtures synced from Crucible from goneat assess.

#### Bug Fixes

- **Flaky appidentity test isolation** — `TestDir` temp dirs could collide under parallel test threads (shared `{pid}-{nanos}`), causing one test to read another's `.fulmen/app.yaml`. Fixed with an `AtomicU64` sequence counter (mirrors the v0.1.4 pathfinder fix).

#### Breaking Changes

None. No public API changes.

#### Testing

- `make check-all` — 362 unit tests, 74 doc tests
- `goneat assess` (format, lint, security) — 100% health

#### Requirements

- **Rust**: 1.88+ (MSRV, unchanged)
- **Crucible**: v0.4.13 (embedded)

---
