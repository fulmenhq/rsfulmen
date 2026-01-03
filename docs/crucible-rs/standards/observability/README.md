---
title: "Fulmen Observability Standards"
description: "Umbrella standards for logging, metrics, traces, and telemetry pipelines"
author: "Codex Assistant"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "draft"
tags: ["observability", "logging", "metrics", "telemetry"]
---

# Observability Standards

The observability program spans structured logging, metrics, tracing, and telemetry distribution. This directory hosts sibling standards that share common goals but remain independently consumable.

## Available Standards

| Document                       | Scope                                                                      |
| ------------------------------ | -------------------------------------------------------------------------- |
| [Logging Standard](logging.md) | Log schema, severity enums, sink/middleware guidance (current initiative). |
| _Metrics Standard (planned)_   | Metric schema, collection/export rules (future).                           |
| _Tracing Standard (planned)_   | Trace/span schema, instrumentation guidance (future).                      |

Add new standards here as observability capabilities expand.

## Relationship to Other Docs

- `docs/sop/cicd-operations.md` – pipelines must run logging validation targets.
- `docs/standards/makefile-standard.md` – defines make targets used by observability tooling.
- `docs/standards/ai-agents.md` – AI maintainers follow the same attribution rules when touching observability assets.

## Schema Layout

Observability schemas live under `schemas/observability/` with subdirectories per capability, e.g., `schemas/observability/logging/v1.0.0/`.

## Contact

- Human lead: @3leapsdave
- AI steward: @schema-cartographer
