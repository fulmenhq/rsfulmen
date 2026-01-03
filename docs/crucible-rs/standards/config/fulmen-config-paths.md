---
title: "Fulmen Config Path Standard"
description: "XDG-aligned configuration location rules for Fulmen libraries"
author: "Schema Cartographer"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "draft"
tags: ["config", "xdg", "standard", "fulmen"]
---

# Fulmen Config Path Standard

## Purpose

Provide a consistent, cross-language contract for where Fulmen tools read/write configuration, data, and cache files. This standard is implemented first in gofulmen and will be adopted across tsfulmen, pyfulmen, and other ecosystem libraries.

## Key Principles

1. **XDG Compliance** – Respect `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, and `XDG_CACHE_HOME` (with platform-specific fallbacks).
2. **Ecosystem Namespace** – Shared configs live under `fulmen` (e.g., `~/.config/fulmen`).
3. **App Flexibility** – Libraries expose helpers that accept an `appName` so non-Fulmen consumers can choose their own namespace.
4. **Legacy Migration** – Tools may search legacy paths (e.g., `~/.config/goneat`) after new paths; migrations should be documented in per-repo SOPs.

## Required APIs

Every Fulmen language foundation MUST expose the following functions (idiomatic naming allowed):

| Function                                     | Description                                                                      |
| -------------------------------------------- | -------------------------------------------------------------------------------- |
| `GetAppConfigDir(appName)`                   | Returns config directory for an arbitrary app.                                   |
| `GetAppDataDir(appName)`                     | Returns data directory for an arbitrary app.                                     |
| `GetAppCacheDir(appName)`                    | Returns cache directory for an arbitrary app.                                    |
| `GetAppConfigPaths(appName, legacyNames...)` | Returns ordered list of config search paths (new first).                         |
| `GetXDGBaseDirs()`                           | Returns `{ configHome, dataHome, cacheHome }`, respecting environment overrides. |

Ecosystem helpers SHOULD be provided:

| Helper                 | Returns                                                    |
| ---------------------- | ---------------------------------------------------------- |
| `GetFulmenConfigDir()` | `$XDG_CONFIG_HOME/fulmen` (default `~/.config/fulmen`).    |
| `GetFulmenDataDir()`   | `$XDG_DATA_HOME/fulmen` (default `~/.local/share/fulmen`). |
| `GetFulmenCacheDir()`  | `$XDG_CACHE_HOME/fulmen` (default `~/.cache/fulmen`).      |

## Platform Defaults

| Platform     | Config                                 | Data                                   | Cache                         |
| ------------ | -------------------------------------- | -------------------------------------- | ----------------------------- |
| Linux / Unix | `~/.config/fulmen`                     | `~/.local/share/fulmen`                | `~/.cache/fulmen`             |
| macOS        | `~/Library/Application Support/Fulmen` | `~/Library/Application Support/Fulmen` | `~/Library/Caches/Fulmen`     |
| Windows      | `%APPDATA%\Fulmen`                     | `%APPDATA%\Fulmen`                     | `%LOCALAPPDATA%\Fulmen\Cache` |

Language foundations MUST implement these defaults when XDG variables are absent.

## Configuration Defaults in Crucible

Crucible mirrors certain schema families with canonical defaults under the `config/` directory. For example, terminal overrides live at `config/terminal/v1.0.0/terminal-overrides-defaults.yaml`. Language foundations SHOULD embed these defaults as the first layer in their configuration stack.

## JSON Schema

Tooling manifests referencing config locations SHOULD use the schema at `schemas/config/fulmen-ecosystem/v1.0.0/fulmen-config-paths.schema.json`.

## Migration Guidance

Repositories migrating from custom paths should follow `docs/sop/config-path-migration.md`. Goneat and other CLI tools SHOULD maintain legacy lookups until users migrate their files.

## References

- [Library Ecosystem Architecture](../../architecture/library-ecosystem.md)
- [Config Path Migration SOP](../../sop/config-path-migration.md)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)
