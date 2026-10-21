# Release Notes

This document tracks release notes for rsfulmen releases.

> **Convention**: Keep only the latest 3 releases here to prevent file bloat. Older releases are archived in `docs/releases/`.

## [0.3.0] - 2026-10-20

### Catalogs, telemetry, redaction, std config paths

**Release Type**: Minor Release

#### Overview

Minor release after 0.2.1. MSRV remains 1.88. Embedded Crucible remains
v0.4.19. Behavior changes cover signal resolution, non-exhaustive catalog
types, Prometheus export, std-only config paths, secret redaction, and
bounded HTTP metric labels.

#### Highlights

- **Std-only config paths.** Absolute `XDG_*_HOME` values apply on every
  platform. The `dirs` crate is gone.
- **Telemetry.** Histogram `+Inf` is only the overflow bucket. One name is
  one instrument kind. HTTP label sets share a 10_000 registry cap.
- **Redaction.** Assignment values for token, secret, password, and API-key
  fields are redacted in log and signal runtime messages, including a
  quoted value with backslash escapes and an authorization scheme plus
  its token. A bare `Bearer` or `Basic` token is removed only when the
  `logging` feature is enabled. Best-effort.
- **Signals.** Listener shutdown does not block a second SIGINT. Double-tap
  calls `process::exit` and does not use `set_force_exit`.
- **Publish.** `make release-publish` binds the upload to the signed tag.
  SSH-alias origins set `RSFULMEN_RELEASE_REPO=fulmenhq/rsfulmen` and push
  the tag first.

#### Breaking Changes

- New `TelemetryError::InstrumentKindConflict`.
- `render_prometheus_text` returns `Result`.
- `resolve_signal("-15")` returns `None`.
- Several catalog types are `#[non_exhaustive]`.
- Absolute XDG env vars override macOS and Windows fallbacks.

#### Requirements

- **Rust**: 1.88+ (MSRV, unchanged)
- **Crucible**: v0.4.19 (embedded)

---

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
