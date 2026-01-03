---
title: "Fulmen Helper Module Catalog"
description: "Architecture index for cross-language helper modules and links to detailed specifications"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["architecture", "modules", "helper-libraries", "2025.10.2"]
---

# Fulmen Helper Module Catalog

This catalog enumerates the cross-language helper modules every Fulmen foundation library implements or
extends. Each entry links to the normative specification under `docs/standards/library/` (see `modules/`, `foundry/`,
and `extensions/` subdirectories) and identifies
whether the module is considered **core** (mandatory) or **extension** (optional, opt-in per language).

| Module                       | Tier                | Summary                                                                                                                              | Spec                                                                                          |
| ---------------------------- | ------------------- | ------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------- |
| FulDX Bootstrap              | Core                | Shared bootstrap pattern for FulDX + tooling manifest                                                                                | [FulDX Bootstrap](../../standards/library/modules/fuldx-bootstrap.md)                         |
| SSOT Sync                    | Core                | Crucible asset synchronization via FulDX                                                                                             | [SSOT Sync](../../standards/library/modules/ssot-sync.md)                                     |
| Crucible Shim                | Core                | Language accessors for embedded Crucible assets - primary integration point for ecosystem tools accessing docs, schemas, and configs | [Crucible Shim](../../standards/library/modules/crucible-shim.md)                             |
| Docscribe                    | Core                | Access and processing of Crucible documentation assets including frontmatter extraction and clean content reads                      | [Docscribe](../../standards/library/modules/docscribe.md)                                     |
| Config Path API              | Core                | Platform-aware config/data/cache path discovery                                                                                      | [Config Path API](../../standards/library/modules/config-path-api.md)                         |
| Three-Layer Config           | Core                | Default → user → runtime configuration layering                                                                                      | [Three-Layer Config](../../standards/library/modules/enterprise-three-layer-config.md)        |
| Schema Validation            | Core                | Schema discovery and validation helpers                                                                                              | [Schema Validation](../../standards/library/modules/schema-validation.md)                     |
| Foundry Patterns             | Core                | Shared regex/glob/catalog data (patterns, HTTP, ISO; ISO lookups normalize alpha-2/alpha-3/numeric)                                  | [Foundry Patterns](../../standards/library/foundry/README.md)                                 |
| Observability Logging        | Core                | Structured logging alignment with Crucible standards                                                                                 | [Observability Logging](../../standards/observability/logging.md)                             |
| Error Handling & Propagation | Core                | Extends Pathfinder errors via $ref with optional severity/correlation fields for uniform wrapping/export.                            | [Error Handling & Propagation](../../standards/library/modules/error-handling-propagation.md) |
| Cloud Storage                | Extension           | Common blob storage helpers                                                                                                          | [Cloud Storage](../../standards/library/extensions/cloud-storage.md)                          |
| Pathfinder                   | Extension           | Path discovery and helper utilities                                                                                                  | [Pathfinder](../../standards/library/extensions/pathfinder.md)                                |
| ASCII Helpers                | Extension           | Text formatting helpers for consoles                                                                                                 | [ASCII Helpers](../../standards/library/extensions/ascii-helpers.md)                          |
| Telemetry/Metrics Export     | Extension (Logging) | Counter/gauge/histogram events validated against taxonomy-backed schema; JSON export alongside logging.                              | [Telemetry/Metrics Export](../../standards/library/modules/telemetry-metrics.md)              |

> **Note**: Observability logging remains under `docs/standards/observability/` because it spans libraries and
> applications. The module catalog includes it for completeness and to anchor cross-language expectations.
>
> **Ecosystem Integration**: The Crucible Shim module provides the standardized API surface for tools and applications to access Fulmen standards and assets. See [Fulmen Helper Library Standard](../fulmen-helper-library-standard.md#ecosystem-tool-integration) for integration patterns.

Future modules must be registered here after their standards land and the machine-readable manifest in
`config/library/` is updated.
