---
title: "Config Path Migration SOP"
description: "Procedure for moving Fulmen tools to the shared XDG config locations"
author: "Schema Cartographer"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "draft"
tags: ["sop", "config", "migration"]
---

# Config Path Migration SOP

## Purpose

Guide repository maintainers through migrating from tool-specific configuration directories (e.g., `~/.goneat`) to the shared Fulmen XDG locations defined in the [Fulmen Config Path Standard](../standards/config/fulmen-config-paths.md).

## Steps

1. **Adopt Foundation API**
   - Update your project to call the new helpers (`GetFulmenConfigDir`, `GetAppConfigDir` etc.) from gofulmen/tsfulmen/pyfulmen.
2. **Implement Legacy Fallback**
   - When loading configs, check legacy directories (`~/.config/goneat`, etc.) **after** the new paths.
   - Log a deprecation warning when reading from legacy locations.
3. **Handle Migration**
   - Provide CLI instructions or automated migration scripts to move files (e.g., `mv ~/.config/goneat/*.yaml ~/.config/fulmen/`).
   - Document migration steps in project README / release notes.
4. **Update Tests**
   - Add unit tests covering both new and legacy path discovery until migration completes.
5. **Communicate**
   - Announce the change in release notes.
   - Provide timeline for removing legacy paths.
6. **Cleanup**
   - After the deprecation period, remove legacy lookups and update docs accordingly.

## References

- [Fulmen Config Path Standard](../standards/config/fulmen-config-paths.md)
- [Library Ecosystem Architecture](../architecture/library-ecosystem.md)
