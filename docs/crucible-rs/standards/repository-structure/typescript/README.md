---
title: "TypeScript Repository Structure Standards"
description: "Mandatory tooling and baseline requirements for FulmenHQ TypeScript projects"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["standards", "repository-structure", "typescript", "bun", "tooling"]
---

# TypeScript Repository Structure Standards

These guidelines capture the baseline expectations for FulmenHQ TypeScript repositories. Read the
cross-language taxonomy in [`docs/standards/repository-structure/README.md`](../README.md) first to determine the
correct repository category (`cli`, `workhorse`, `library`, etc.), then apply the category-specific guide (e.g.,
[`workhorse.md`](workhorse.md)).

## Mandatory Tooling

- **Runtime/Package Manager**: Bun ≥ 1.2
- **Lint/Format**: Biome (`bunx biome check` / `bunx biome format`)
- **Type Checking**: `bunx tsc --noEmit`
- **Testing**: Bun test runner (`bun test`) or Vitest where appropriate
- **Schema Validation**: goneat CLI for Fulmen schemas

Ensure the project Makefile delegates to these tools and aligns with the Fulmen Makefile standard.

## Directory Layout

```
project/
├── src/
│   └── index.ts                # Primary entry point
├── test/                       # Vitest/Bun tests
├── examples/                   # Optional usage samples
├── schemas/                    # Embedded schemas (if applicable)
├── bunfig.toml                 # Bun configuration
├── biome.json                  # Lint/format config
├── tsconfig.json               # Compiler options
└── Makefile
```

Category-specific guides detail additional structure (e.g., Fastify server layout for workhorse repos).
