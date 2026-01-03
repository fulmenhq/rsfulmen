---
title: "HTTP API Schemas v1.0.0"
description: "JSON Schemas for Fulmen HTTP workhorse/service endpoints"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["schema", "api", "http", "2025.10.2"]
---

# HTTP API Schemas v1.0.0

Canonical JSON Schemas for shared HTTP endpoints used by workhorse and service repositories.

## Schemas

- `health-response.schema.json` – Liveness/readiness/startup response envelope.
- `version-response.schema.json` – Service version endpoint payload.
- `error-response.schema.json` – Standardized error payload aligned with HTTP REST standard.
- `success-response.schema.json` – Generic success envelope used by utility endpoints.

Examples for these schemas live under `examples/api/http/v1.0.0/`.
