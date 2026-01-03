---
title: "Goneat Bootstrap Guide"
description: "Installing the goneat CLI via package managers or local overrides"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["bootstrap", "goneat", "tools"]
---

# Goneat Bootstrap Guide

Goneat is the primary CLI for schema validation, SSOT sync, and release automation across the [fulmen ecosystem](https://github.com/fulmenhq) (FulmenHQ). This guide
explains how to install goneat, how to keep a repository-local override, and where to place tooling manifests if
manual bootstraps are needed.

## 1. Preferred Installation (Package Managers)

As soon as taps/buckets are live, install goneat via your platform package manager:

```bash
# macOS (Homebrew)
brew install goneat

# Windows (Scoop)
scoop install goneat

# Linux (Homebrew on Linux or distro-specific packages)
brew install goneat
```

Package manager installs keep goneat on `$PATH` for CI/CD and local development. Scripts should invoke `goneat`
directly rather than assuming `./bin/goneat`.

You can also manually download and place in a folder that is in your machine's `$PATH` as well.

## 2. Bootstrap Fallback (Manual Download)

Until package managers are ready—or when you need a pinned binary—use the repository bootstrap script hosted at
`scripts/bootstrap-tools.ts`.

### 2.1 Requirements

- **Bun ≥ 1.2** (used to run the bootstrap script)
- `./bin/` directory (created automatically)
- `.goneat/tools.yaml` manifest describing the binaries to install

Example `.goneat/tools.yaml` entry:

```yaml
version: v0.3.0
binDir: ./bin
tools:
  - id: goneat
    description: Fulmen schema validation and automation CLI
    required: true
    install:
      type: download
      url: https://github.com/fulmenhq/goneat/releases/download/<actual version>/goneat-{{os}}-{{arch}}
      binName: goneat
      destination: ./bin
      checksum:
        darwin-arm64: "<sha256-checksum>"
        darwin-amd64: "<sha256-checksum>"
        linux-amd64: "<sha256-checksum>"
        linux-arm64: "<sha256-checksum>"
```

> Replace `<sha256-checksum>` values with the published checksums from the goneat GitHub release you plan to
> consume. To determine the latest, visit the [goneat releases page](https://github.com/fulmenhq/goneat/releases)
> or clone the repository and run `make dist` locally.

### 2.2 Running the Bootstrap

```bash
make bootstrap

# or explicitly invoke the script
bun run scripts/bootstrap-tools.ts --install
```

Either command downloads the binaries defined in `.goneat/tools.yaml` (or `.goneat/tools.local.yaml` if present),
places them in `./bin`, and marks them executable. Use `--verify` to confirm the binaries exist and are callable.

## 3. Local Development Override

When iterating on goneat itself (or testing a custom build), create a gitignored override manifest that points to
your local binary:

```bash
cp .goneat/tools.local.yaml.example .goneat/tools.local.yaml
```

Example override entry:

```yaml
version: v0.0.0-dev
tools:
  - id: goneat
    install:
      type: link
      source: /Users/you/dev/goneat/dist/goneat
      binName: goneat
      destination: ./bin
```

Run the bootstrap script again—`tools.local.yaml` takes precedence over the committed manifest.

## 4. Post-Install Checklist

```bash
goneat version

# Validate a schema to ensure dependencies are satisfied
goneat schema validate-schema schemas/observability/logging/v1.0.0/log-event.schema.json
```

If these commands succeed, the bootstrap is complete. CI workflows should run either the package manager install
step or `bun run scripts/bootstrap-tools.ts --install` before invoking goneat commands.

## 5. Tooling Manifest Notes

- Keep `.goneat/tools.yaml` under version control; it documents the expected tooling for contributors and CI.
- `.goneat/tools.local.yaml` should remain gitignored and exist only on machines that need overrides.
- `schemas/config/sync*.yaml` and related config assets stay in Crucible so other tooling can reuse them without
  depending on goneat directly.

## 6. Looking Up Goneat Releases

To find the latest released version:

1. Visit the [goneat releases page](https://github.com/fulmenhq/goneat/releases) for version numbers and checksums.
2. Or clone the repository and run its release workflow (`make dist` or `make release:preview`) to produce local
   binaries in `dist/` for use with the override manifest.

Once package manager taps are published, update this guide with exact commands and any platform-specific flags.
