---
title: "Fulmen Library Extensions"
description: "Optional helper modules available to Fulmen foundation libraries"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["standards", "library", "extensions", "2025.10.2"]
---

# Fulmen Library Extensions

Extension modules provide optional capabilities that helper libraries MAY implement when language ecosystems
or product requirements justify the additional surface area. Each extension specification documents the
expected APIs, configuration, and testing patterns.

| Module        | Summary                                            | Spec                                 |
| ------------- | -------------------------------------------------- | ------------------------------------ |
| Cloud Storage | Common blob storage helpers (S3, GCS, Azure)       | [cloud-storage.md](cloud-storage.md) |
| Pathfinder    | Cross-platform path discovery & manipulation       | [pathfinder.md](pathfinder.md)       |
| ASCII Helpers | Console formatting utilities (box drawing, tables) | [ascii-helpers.md](ascii-helpers.md) |

Future extensions must be added here and referenced from the machine-readable manifest so tooling can detect
language-level opt-ins.
