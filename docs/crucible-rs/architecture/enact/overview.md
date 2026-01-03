---
title: "Enact Overview"
description: "Guided, imperative, idempotent deployment model and the SSOT schema foundation"
author: "Schema Cartographer"
date: "2025-12-17"
last_updated: "2025-12-17"
status: "draft"
tags: ["architecture", "enact", "devsecops", "deployment"]
---

# Enact Overview

Enact is a guided deployment model for infrastructure and service delivery: **imperative execution with idempotency guarantees**, backed by schemas and operator-visible artifacts.

It targets the middle ground between:

- Ad-hoc scripts (fast, but brittle and non-portable)
- Full declarative IaC (powerful, but slow-to-adopt and often overkill)

## The Core Loop

1. **Recipe** (intent): operator declares what they want.
2. **Plan** (preview): runner computes the actions required.
3. **Apply** (execute): runner executes phases with human checkpoints.
4. **Inventory** (state): runner records what exists.
5. **Runlog + Health** (evidence): runner records what happened and what is healthy.

## SSOT: Enact Schemas

Enact is intentionally **schema-first**. The schemas define the contracts for:

- Taxonomy vocabulary (phases, components, variants, providers)
- Operator inputs (recipes, secrets documentation)
- Runner state (inventory)
- Operational evidence (runlogs, health reports, metrics definitions)

**Schema source (Crucible)**: `schemas/enact/v1.0.0/`

**Published namespace (target)**: `https://schemas.fulmenhq.dev/enact/`

See: [Enact Schema Architecture](schema-architecture.md)

## What Enact Is (and isn’t)

- Enact is a **domain architecture** and schema namespace in Crucible.
- Enact may later have helper-library modules, but there is no stable cross-language module contract yet.
- Enact is designed to be **extended by plugins** (domain orgs provide phase implementations, recipes, and docs) while sharing a common schema foundation.

## Operator Workspace (Concept)

A typical deployment workspace is expected to be portable and git-friendly:

- `recipe.yaml` (committed)
- `.enact/inventory.yaml` (committed)
- `.enact/runlog/` and `.enact/checks/` (often gitignored)
- `secrets.*` (encrypted, committed only when appropriate)

The exact workspace standard can evolve independently from schema evolution, but should remain schema-backed.
