# Release Notes

This document tracks release notes for rsfulmen releases.

> **Convention**: Keep only the latest 3 releases here to prevent file bloat. Older releases are archived in `docs/releases/`.

## [0.1.7] - 2026-11-22

### foundry-core loads catalogs with yamlprims

**Release Type**: Patch Release

### Changed

- **`foundry-core` loads its embedded catalogs with `yamlprims`.** This covers the
  exit codes, signals, country codes and HTTP statuses catalogs. Builds that enable
  only `foundry-core` no longer depend on `serde_yaml_ng` or `unsafe-libyaml`.
  `serde_yaml_ng` is now an optional dependency. The features that use it enable
  it, so the default and `full` feature sets are unchanged. The public API is
  unchanged.
- A test checks that `yamlprims` and `serde_yaml_ng` load the same data from
  each `foundry-core` catalog. A CI step fails if the `foundry-core` dependency
  tree contains `serde_yaml_ng` or `unsafe-libyaml`.

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
