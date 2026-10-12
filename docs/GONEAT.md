# Goneat Integration Guide

How [goneat](https://github.com/fulmenhq/goneat) is used in rsfulmen for bootstrap,
Crucible SSOT sync, formatting assessment, and version operations.

## Overview

rsfulmen uses the goneat CLI for standardized development operations:

- **Bootstrap** — install foundation tools (`ripgrep`, `jq`, `yq`, formatters)
- **Sync** — pull Crucible schemas, docs, and config into `*-crucible-rs/` trees
- **Assess** — format / lint / security checks used by `make fmt`, hooks, and pre-push
- **Version** — `goneat version set|bump` (release-pack work only; not this feature track)

goneat is a **binary** installed via [sfetch](https://github.com/3leaps/sfetch)
(`make bootstrap`). rsfulmen does not `go install` goneat. There is no crate-level
circular dependency (goneat is Go; this repo is a Rust library).

## Pin

```
GONEAT_VERSION ?= v0.6.3   # Makefile
```

Override locally if you must:

```bash
make bootstrap GONEAT_VERSION=v0.6.3
# or force reinstall
make bootstrap-force
```

`goneat v0.6.2+` has no `darwin/amd64` asset. That drop is estate-wide; do not
re-add that triple to any matrix.

v0.6.3 skips gosec / govulncheck unless the tree has a `go.mod`. Rust-only
hooks therefore pass the security category without `--no-verify` (the v0.6.2
false positive on a Rust adapter is fixed).

Confirm the binary:

```bash
goneat --version
# or: goneat version
```

## Bootstrap

`make bootstrap` (see the Makefile):

1. Installs **sfetch** (trust anchor) into `$(BINDIR)` (default `$HOME/.local/bin`).
2. Installs **goneat** at `GONEAT_VERSION` via
   `sfetch -repo fulmenhq/goneat -tag <pin> -dest-dir $(BINDIR)`.
3. Verifies `cargo` / `rustc`.
4. Installs cargo tools (`cargo-tarpaulin`, `cargo-audit`, `cargo-deny`, `cargo-msrv`).
5. Runs `goneat doctor tools --scope foundation --install` when goneat is on `PATH`.

Ensure `$(BINDIR)` is on `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
make tools
```

Tool scopes and installers live in [`.goneat/tools.yaml`](../.goneat/tools.yaml).
That file is the goneat doctor manifest (ripgrep, jq, yamlfmt, cargo-deny, …),
not a second goneat version pin. The version pin is **only** `GONEAT_VERSION`
in the Makefile.

## Crucible SSOT sync

Consumer config: [`.goneat/ssot-consumer.yaml`](../.goneat/ssot-consumer.yaml).

```yaml
sources:
  - name: crucible
    repo: fulmenhq/crucible
    ref: v0.4.19
```

```bash
make sync          # goneat ssot sync --force-remote + scripts/sync-crucible-version.sh
```

Synced trees (do not edit by hand):

| Tree                   | Contents                         |
| ---------------------- | -------------------------------- |
| `config/crucible-rs/`  | catalogs, taxonomy, templates    |
| `schemas/crucible-rs/` | JSON schemas                     |
| `docs/crucible-rs/`    | standards and architecture docs  |
| `.crucible/metadata/`  | sync provenance                  |

If a Crucible asset is wrong, document the proposed fix locally (gitignored
`.plans/memos/`) and take it upstream. After Crucible publishes a new tag,
bump `ref` in `.goneat/ssot-consumer.yaml` and run `make sync`.

Provenance after a sync:

```bash
cat .goneat/ssot/provenance.json
cat .crucible/metadata/metadata.yaml
```

## Makefile targets that invoke goneat

| Target        | Role                                              |
| ------------- | ------------------------------------------------- |
| `bootstrap`   | Install sfetch + pinned goneat + cargo tools      |
| `tools`       | Verify goneat and the Rust toolchain              |
| `sync`        | `goneat ssot sync --force-remote`                 |
| `fmt`         | rustfmt + `goneat assess --categories format --fix` |
| `fmt-check`   | rustfmt `--check` + goneat format assess          |
| `precommit`   | rustfmt, clippy, goneat assess (critical)         |
| `prepush`     | rustfmt, clippy, tests, goneat assess (high)      |
| `version-*`   | goneat version set/bump (release-pack only)       |

`make check-all` is `fmt-check` + clippy + tests + `make deny` (read-only;
`cargo deny` does not mutate the tree). CI also runs deny as its own job.

## CI alignment

rsfulmen CI (`.github/workflows/ci.yml`) runs on `ubuntu-latest` **inside**
`ghcr.io/fulmenhq/goneat-tools-runner-glibc:v0.5.9` (digest-pinned; fulmen-toolbox
v0.5.9, `--user 1001`). That image ships goneat **v0.6.3**, rustup, and
`cargo-deny`. Jobs still install `dtolnay/rust-toolchain@1.98.0` (MSRV: 1.88.0)
so clippy matches `rust-toolchain.toml`. rustup/cargo homes are
`/tmp/rsfulmen-*` because UID 1001 cannot write the image `/opt/rustup`. Do not
add `darwin/amd64` to any matrix.

When local `goneat` and CI disagree on YAML format/lint, compare the local pin
(`GONEAT_VERSION`) to the runner pin (`v0.5.9` / goneat v0.6.3) before changing
repo policy.

## Daily workflow

```bash
make bootstrap     # once per machine
make sync          # after Crucible bumps, or when catalogs look stale
make fmt
make test
make deny          # licenses, bans, advisories, sources
make check-all
```

Hooks install on first `make build` (`goneat hooks generate/install` when
`.git/hooks/pre-commit` is missing). Manifest: [`.goneat/hooks.yaml`](../.goneat/hooks.yaml).

## Troubleshooting

### goneat not found

```bash
make bootstrap
export PATH="$HOME/.local/bin:$PATH"
goneat --version
```

### Pin / version drift

`make bootstrap` skips install when *any* goneat is already on `PATH`. To force
the Makefile pin:

```bash
make bootstrap-force
```

### Sync fails

1. Confirm goneat is the pinned binary (`goneat --version`).
2. Confirm GitHub can reach `fulmenhq/crucible`.
3. Confirm `.goneat/ssot-consumer.yaml` `ref` is a published Crucible tag.

### cargo-deny missing locally

```bash
make bootstrap-cargo-tools
make deny
```

CI installs deny via `EmbarkStudios/cargo-deny-action`; local `make deny`
requires the cargo plugin.

## Relation to release

`make release-publish` exists on rsfulmen. Other language foundations still need
parity for that path. **Feature PRs must not tag, release, or edit
`VERSION` / `CHANGELOG.md` / `RELEASE_NOTES.md` / `docs/releases/`.** The next
release pack waits on Dave.
