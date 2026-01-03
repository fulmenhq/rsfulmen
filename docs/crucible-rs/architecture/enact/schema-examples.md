---
title: "Enact Schema Examples"
description: "Realistic example documents for the Enact schema suite"
author: "Schema Cartographer"
date: "2025-12-17"
last_updated: "2025-12-17"
status: "draft"
tags: ["architecture", "enact", "schemas", "examples"]
---

# Enact Schema Examples

These examples demonstrate **schema-valid instance documents** for the Enact schema suite.

They are intentionally **vendor-neutral** and are meant to illustrate the data model.
They do **not** imply that specific Enact plugins (providers, components) are implemented yet.

## Structure

Examples are versioned and grouped by intent:

- `examples/enact/v1.0.0/minimal/` - smallest structurally-valid instances
- `examples/enact/v1.0.0/features/` - showcases of specific schema capabilities
- `examples/enact/v1.0.0/invalid/` - intentionally invalid cases (for validator testing)
- `examples/enact/v1.0.0/realistic/` - production-like examples by domain

## Example Scenario (Realistic)

A vendor-neutral collaboration suite at `example.education`.

- Recipe: `examples/enact/v1.0.0/realistic/collaboration-suite/recipe.yaml`
- Inventory: `examples/enact/v1.0.0/realistic/collaboration-suite/inventory.yaml`
- Runlog: `examples/enact/v1.0.0/realistic/collaboration-suite/runlog.yaml`
- Health report: `examples/enact/v1.0.0/realistic/collaboration-suite/health-report.yaml`

## Notes

- The `realistic/*/recipe.yaml` files describe intent only; they don’t imply provider plugins are already implemented.

## Related

- Enact overview: `docs/architecture/enact/overview.md`
- Enact schema architecture: `docs/architecture/enact/schema-architecture.md`
- Enact schema source: `schemas/enact/v1.0.0/`
