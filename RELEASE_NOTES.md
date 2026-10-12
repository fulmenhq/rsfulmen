# Release Notes

This document tracks release notes for rsfulmen releases.

> **Convention**: Keep only the latest 3 releases here to prevent file bloat. Older releases are archived in `docs/releases/`.

## [0.2.1] - 2026-10-12

### goneat v0.6.2, tagger env, deny licenses

**Release Type**: Patch / Maintenance Release

#### Overview

Records goneat v0.6.2 (#18), refreshes `Cargo.lock` within current ranges,
requires signed-tag tagger env, and adds `deny.toml` so `cargo deny check`
licenses passes. MSRV remains 1.88. Embedded Crucible is v0.4.19.

#### Highlights

- **Estate-wide darwin/amd64 drop** — goneat v0.6.2+ has no `darwin/amd64`
  asset. Intel macOS cannot bootstrap that pin (estate-wide, not rsfulmen
  binaries; this crate does not ship binaries).
- **`RSFULMEN_TAGGER_NAME` / `RSFULMEN_TAGGER_EMAIL`** — required for
  `make release-tag` and `make release-verify-tag`. Tag object is stamped
  and verified against env. No identities committed.
- **`deny.toml`** — license allow list; fixes the #18 licenses failure.
- **Lockfile** — xxhash-rust 0.8.19; libc 0.2.190; serde_json 1.0.154.
  uuid held at 1.26.1 (1.28.0 needs rustc 1.89). jsonschema 0.17, dirs 5,
  thiserror 1 held.

#### Breaking Changes

None for the crate API.

#### Testing

- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets --all-features -- -D warnings`
- `cargo test --locked --all-features`
- `cargo deny check`
- `make precommit` / `make prepush`

#### Requirements

- **Rust**: 1.88+ (MSRV, unchanged)
- **Crucible**: v0.4.19 (embedded)

---

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
