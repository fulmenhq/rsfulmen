---
title: "Config Path API Standard"
description: "Cross-language helper contract for discovering Fulmen configuration, data, and cache directories"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["standards", "library", "config", "paths", "2025.10.2"]
---

# Config Path API Standard

## Purpose

Provide a consistent helper API for resolving configuration, data, and cache directories across platforms.
Every Fulmen helper library MUST expose this contract so applications can locate Crucible defaults, local
overrides, and runtime caches without duplicating OS-specific logic.

## Responsibilities

1. Discover platform-specific base directories using the Fulmen Config Path Standard.
2. Expose helper functions that return:
   - Fulmen-specific config/data/cache roots.
   - Application-scoped directories (`<vendor>/<app>` conventions).
   - Ordered search paths for configuration files.
3. Support overrides via environment variables documented in `docs/standards/config/fulmen-config-paths.md`.
4. Guarantee directories exist (create lazily) when the consumer requests `Ensure*` variants.

## API Contract

Language bindings MUST provide the following surface (idiomatic naming allowed):

| Function                    | Return Type | Description                                                |
| --------------------------- | ----------- | ---------------------------------------------------------- |
| `GetFulmenConfigDir()`      | string/path | Base directory for Fulmen config (`~/.config/fulmen`).     |
| `GetFulmenDataDir()`        | string/path | Base directory for Fulmen data (`~/.local/share/fulmen`).  |
| `GetFulmenCacheDir()`       | string/path | Base directory for Fulmen cache (`~/.cache/fulmen`).       |
| `GetOrgConfigDir()`         | string/path | Organization-level config directory (`/opt/fulmen`).       |
| `GetAppConfigDir(app)`      | string/path | App-specific config directory under vendor namespace.      |
| `GetAppDataDir(app)`        | string/path | App-specific data directory.                               |
| `GetAppCacheDir(app)`       | string/path | App-specific cache directory.                              |
| `GetConfigSearchPaths(app)` | list[path]  | Ordered list of directories (defaults → user → overrides). |
| `EnsureDir(path)`           | error/bool  | Create directory (with parents) if missing.                |

`app` parameters MUST accept a struct/object containing vendor + application identifiers. Helpers MUST enforce
lowercase kebab-case (`fulmenhq`, `gofulmen`) to align with filesystem conventions.

## Platform Behavior

| Platform | Base Config                       | Base Data                            | Base Cache                      | Notes                                                                    |
| -------- | --------------------------------- | ------------------------------------ | ------------------------------- | ------------------------------------------------------------------------ |
| Linux    | `$XDG_CONFIG_HOME` or `~/.config` | `$XDG_DATA_HOME` or `~/.local/share` | `$XDG_CACHE_HOME` or `~/.cache` | Respect XDG variables when set.                                          |
| macOS    | `~/Library/Application Support`   | `~/Library/Application Support`      | `~/Library/Caches`              | Config and data share root per Apple guidelines.                         |
| Windows  | `%APPDATA%`                       | `%LOCALAPPDATA%`                     | `%LOCALAPPDATA%/Cache`          | Use `SHGetKnownFolderPath` in Go/C#, `Pathlib` with `APPDATA` in Python. |

Fulmen-specific directories append `/fulmen` to the base path. Application-scoped directories append
`/<vendor>/<app>` (e.g., `~/.config/fulmen/fulmenhq/gofulmen`).

## Overrides

Honor environment variables defined in the config path standard (e.g., `FULMEN_CONFIG_HOME`,
`FULMEN_DATA_HOME`, `FULMEN_CACHE_HOME`, `FULMEN_ORG_PATH`). Helpers MUST validate overrides to prevent traversal outside the
user's home directory unless explicitly allowed via documented configuration flags. `FULMEN_ORG_PATH` defaults to `/opt/fulmen` when unset.

## Language Implementation Notes

- **Go**: Provide functions in `foundation/config/paths`. Use `os/user` for home fallback, `os.MkdirAll` for
  ensuring directories, and return `path/filepath` values.
- **Python**: Provide module `pyfulmen.config.paths` with functions returning `pathlib.Path`. Use `platformdirs`
  for base directories and wrap exceptions with `FulmenPathError`.
- **TypeScript**: Implement under `@fulmenhq/crucible/config/paths`. Use `env-paths` or custom resolution and
  return POSIX-style paths (ensure Windows normalization via `path.win32`).
- **Rust/C#** (planned): Provide asynchronous-safe helpers using standard crates/`Environment.SpecialFolder`.

## Testing Requirements

- Unit tests covering Linux/macOS/Windows permutations (mock environment variables).
- Tests ensuring overrides respect validation rules and fall back correctly when unset.
- Integration tests verifying directories are created when `EnsureDir` is invoked (use temp directories).

## Related Documents

- `docs/standards/config/fulmen-config-paths.md`
- `config/taxonomy/languages.yaml`
- `.plans/active/2025.10.2/library-module-specification-architecture-v2.md`
