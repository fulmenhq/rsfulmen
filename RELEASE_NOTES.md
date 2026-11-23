# Release Notes

This document tracks release notes for rsfulmen releases.

> **Convention**: Keep only the latest 3 releases here to prevent file bloat. Older releases are archived in `docs/releases/`.

## [0.3.1] - 2026-11-23

### foundry-core on yamlprims, Crucible v0.5.1, hardening

**Release Type**: Patch Release

#### Overview

Patch release. MSRV remains 1.88. The embedded Crucible catalog is now
v0.5.1. 0.3.1 is the first 0.3 release on crates.io.

#### Highlights

- **foundry-core without libyaml.** The embedded catalogs load with
  `yamlprims`. Builds that enable only `foundry-core` no longer depend on
  `serde_yaml_ng` or `unsafe-libyaml`. Default and `full` feature sets and
  the public API are unchanged.
- **Crucible v0.5.1.** SSOT assets and `CRUCIBLE_VERSION` report v0.5.1.
  The foundry catalogs are unchanged.
- **Signals.** A second SIGINT inside the double-tap window now calls the
  `set_force_exit` hook with the double-tap exit code. Without a hook it
  still calls `process::exit`.
- **fulpack.** Extract and verify reject entry names with a drive letter in
  any segment as absolute paths on every platform, and extract rejects any
  joined path outside the destination.
- **Redaction.** `Authorization`, `Proxy-Authorization`, and `Cookie` /
  `Set-Cookie` values are redacted to the end of the line. Long secret keys
  also match as field-name suffixes. Route normalization maps email-shaped
  and token-shaped segments.
- **Config and logging.** Boolean env parse errors omit the raw input.
  Simple-profile lines escape control characters and Unicode line
  separators.

#### Behavior Change

- Double-tap force exit goes through `set_force_exit` when a hook is set
  (0.3.0 always called `process::exit`).

#### Requirements

- **Rust**: 1.88+ (MSRV, unchanged)
- **Crucible**: v0.5.1 (embedded)

---

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
