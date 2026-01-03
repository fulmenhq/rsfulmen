---
title: "Crucible Configuration Defaults"
description: "Default configuration assets mirrored from schemas"
author: "Schema Cartographer"
date: "2025-10-03"
last_updated: "2025-10-03"
status: "draft"
tags: ["config", "defaults", "ssot"]
---

# Crucible Configuration Defaults

Crucible now mirrors certain schema families with canonical default configuration files. These defaults live under `config/` and follow the same directory hierarchy and versioning pattern as `schemas/`.

## Layout

```
config/
├── terminal/
│   └── v1.0.0/
│       └── terminal-overrides-defaults.yaml
└── ... (future categories)
```

## Naming Convention

- Files must use the suffix `-defaults.yaml` (or `.json`) to distinguish SSOT defaults from user-provided configuration.
- Version directories (`v1.0.0`) align with the corresponding schema versions.

## Usage

Language foundations (gofulmen, tsfulmen, etc.) embed these files and layer user/application overrides on top:

1. **Library defaults** – from this directory.
2. **User overrides** – e.g., `~/.config/fulmen/*.yaml`.
3. **Application-provided config** – BYOC path determined by the consuming app.

See the [Fulmen Config Path Standard](../docs/standards/config/fulmen-config-paths.md) for runtime path discovery.
