# rsfulmen Development Guide

This directory contains local development documentation for rsfulmen. For Crucible ecosystem-wide standards, see [`../crucible-rs/`](../crucible-rs/).

## Contents

| Document                       | Description                                       |
| ------------------------------ | ------------------------------------------------- |
| [operations.md](operations.md) | Build, test, release, and operations runbook      |
| [adr/](adr/)                   | Architecture Decision Records (rsfulmen-specific) |

## Quick Start

```bash
# Install dependencies and tools
make bootstrap

# Sync Crucible assets
make sync

# Run all quality checks
make check-all

# Build
make build
```

## Key Resources

- [rsfulmen Overview](../rsfulmen-overview.md) - Module catalog and feature gates
- [README](../../README.md) - Installation and usage
- [CHANGELOG](../../CHANGELOG.md) - Version history

## Crucible Ecosystem

rsfulmen syncs from the [Crucible SSOT](https://github.com/fulmenhq/crucible). Synced assets are read-only:

- `docs/crucible-rs/` - Standards, architecture, decisions
- `schemas/crucible-rs/` - JSON schemas
- `config/crucible-rs/` - Configuration templates and catalogs

To update synced assets:

```bash
make sync
```

See [SSOT Sync Standard](../crucible-rs/standards/library/modules/ssot-sync.md) for details.
