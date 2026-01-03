---
title: "Fulmen Library Ecosystem"
description: "Overview of Layer 1 foundations, SSOT integration, and ecosystem dependencies"
author: "Schema Cartographer"
date: "2025-10-28"
last_updated: "2025-10-28"
status: "active"
tags: ["architecture", "ecosystem", "libraries", "layer-1", "observability"]
---

# Fulmen Library Ecosystem

## Overview

The Fulmen library ecosystem centers on Layer 1 foundations (`*fulmen` packages), which wrap Layer 0 SSOT assets (Crucible schemas/standards) for idiomatic use. This prevents direct SSOT imports, ensuring type safety and consistency while avoiding cycles. Libraries enable Layers 2-4 (templates, tools, apps) to consume shared capabilities like observability and config paths.

```
Fulmen Layer Cake (Library Focus)

Layer 0: SSOT (Crucible, Cosmography) ──┐
                                        ├──▶ Layer 1: Foundations (*fulmen)
                                        │     ├── gofulmen (Go)
                                        │     ├── pyfulmen (Python)
                                        │     ├── tsfulmen (TS/JS)
                                        │     └── {rsfulmen, csfulmen} (Planned)
                                        │
                                        └──▶ Layer 2-4: Consumers
                                              ├── Templates (Fulmens: cockpit, runner-forge)
                                              ├── Tools (goneat, fulward, pathfinder)
                                              └── Apps (brooklyn-mcp, sumpter)
```

- **Layer 0 (SSOT)**: Crucible holds schemas, standards, docs; Cosmography (planned) adds data modeling.
- **Layer 1 (Foundations)**: `*fulmen` packages provide language-native APIs for SSOT assets (e.g., schema validation, logging).
- **Consumers (Layers 2-4)**: Depend on foundations, not raw SSOT, for ergonomic access and principle enforcement (e.g., type safety, observability).

## Repository Roles

### Language Foundations Taxonomy

Layer 1 libraries follow a shared roadmap, aligning with Core Fulmen Principles (e.g., type safety, schemas first). This table lists canonical support for 2025.10.4+.

| Language   | Library Name | Status  | Minimum Runtime    | Notes                                                                 |
| ---------- | ------------ | ------- | ------------------ | --------------------------------------------------------------------- |
| Go         | gofulmen     | Active  | Go 1.23+           | Reference for standards; implements config paths, observability.      |
| Python     | pyfulmen     | Active  | Python 3.12+       | Focus on logging/context; Pydantic for schema validation.             |
| TypeScript | tsfulmen     | Active  | TS 5.0+ (Bun/Node) | ESM bundles; Zod for runtime types; aligns with web/service patterns. |
| Rust       | rsfulmen     | Planned | Rust 1.70+         | Post-stabilization; serd/serde for schemas.                           |
| C#         | csfulmen     | Planned | .NET 8.0+          | For ASP.NET/workers; nullable types for safety.                       |

Updates sync from `config/taxonomy/languages.yaml`. New languages require principle-aligned modules (e.g., observability) before standards reference them. See `docs/standards/repository-structure/` for repo expectations.

### Crucible (Layer 0 SSOT)

- Authoritative source for schemas (e.g., observability, pathfinder), standards (coding, security), docs, and configs.
- Foundations import/embed assets (e.g., via sync scripts); not direct consumer dependency.
- Ties to Principles: "Schemas First," "Observability from Day One." Observability under `docs/standards/observability/`.

### Cosmography (Planned Layer 0 Extension)

- Upcoming SSOT for data modeling, topology, and domain-specific schemas (e.g., entity relationships).
- Foundations will wrap for language-native access, extending "Schemas First" to complex data.

### gofulmen (Go Foundation, Layer 1)

- Provides idiomatic Go APIs for Layer 0 assets (e.g., schema validation, logging via Crucible).
- Core modules: config paths (per Fulmen Config Path Standard), observability, Foundry catalogs.
- Avoids cycles: No imports from Layer 3+ (e.g., goneat as CLI only).
- Aligns with Principles: "Type Safety," "DRY with Purpose" via reusable structs/enums.

### tsfulmen (TypeScript Foundation, Layer 1)

- Mirrors gofulmen: Native TS/JS APIs for schemas, logging, config (Zod for validation).
- Supports Bun/Node; ESM bundles for web/services.
- Principles: "Ruthless About Type Safety" (strict TS), "Observability from Day One" (pre-wired metrics).

### pyfulmen (Python Foundation, Layer 1)

- Idiomatic Python APIs for Layer 0 (Pydantic for schemas, logging adapters).
- Focus: Context management, observability; aligns with Python typing standards.
- Principles: "Schemas First" (runtime validation), "Persnickety About Code" (ruff integration via goneat).

### goneat (Layer 3 Tool, Not Library)

- CLI for quality (format, lint, validation); depends on gofulmen for assets.
- External binary only—no library imports to avoid cycles.
- Enforces Principles: "Persnickety About Code" via hooks; integrates with fulward for security.

## Namespace Patterns

Adopt Option B for clarity: Direct SSOT nesting (e.g., `gofulmen/crucible/logging`) with `foundation/` for cross-SSOT utilities (e.g., `foundation/config/paths`).

- Top-level: Reserve for core (e.g., `gofulmen/pathfinder`).
- Re-exports: Convenience for common assets (e.g., `gofulmen/logging` → Crucible schemas).
- Benefits: Short, source-obvious imports; supports "Simplicity" principle without deep nesting.

## Packaging Guidance

- Layers 2-4 depend on `*fulmen` (Layer 1) for assets; pin via CalVer (e.g., `gofulmen@v2025.10.4`).
- Foundations embed/sync Layer 0 (Crucible) versions internally.
- Tools (Layer 3, e.g., goneat) as external CLIs via manifests (`.goneat/tools.yaml`); no lib deps to prevent cycles.
- Principles Tie-In: Ensures "Type Safety" and "DRY" through reusable, versioned APIs.

## Related Docs

- [Fulmen Layer Cake Guide](fulmen-ecosystem-guide.md)
- [Technical Manifesto (Principles)](fulmen-technical-manifesto.md)
- [Helper Library Standard](fulmen-helper-library-standard.md)
- [Config Path Standard](../standards/config/fulmen-config-paths.md)
- [Makefile Standard](../standards/makefile-standard.md)
- [Sync Model](sync-model.md)
- [Pseudo-Monorepo](pseudo-monorepo.md)
- [Observability Standards](../standards/observability/README.md)
- [Repository Structure](../standards/repository-structure/README.md)
