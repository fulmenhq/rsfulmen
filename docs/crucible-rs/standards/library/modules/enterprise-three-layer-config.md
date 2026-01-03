---
title: "Enterprise Three-Layer Configuration Standard"
description: "Layered configuration loading contract for enterprise-grade Fulmen applications (workhorses, complex CLIs)"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-11-10"
status: "draft"
tags: ["standards", "library", "configuration", "enterprise", "2025.10.2"]
---

# Enterprise Three-Layer Configuration Standard

## Purpose

Define how helper libraries load and merge configuration from Crucible defaults, user overrides, and runtime inputs for **enterprise-grade applications**. This three-layer pattern is designed for workhorse services, complex CLI tools, and applications requiring ecosystem-wide standardization.

**When to use Enterprise Three-Layer Config**:

- ✅ Workhorse services (HTTP APIs, background workers)
- ✅ Complex CLI tools with multiple configuration sources
- ✅ Applications requiring Crucible SSOT defaults for compliance

**When NOT to use** (use Simple Config Pattern instead):

- ❌ Microtools (single-purpose CLI tools) - use two-layer config (defaults + user overrides only)
- ❌ Simple applications with minimal configuration
- ❌ Prototypes and experiments

## Layer Model (The "Three Layers")

This pattern is called "Enterprise **Three-Layer** Config" because configuration is composed from three distinct layers:

| Priority | Layer                          | Source                                               | Description                                 |
| -------- | ------------------------------ | ---------------------------------------------------- | ------------------------------------------- |
| 3 (base) | **Layer 1: SSOT Defaults**     | Crucible assets (`config/<category>/vX.Y.Z/*.yaml`). | Ecosystem-wide standards and best practices |
| 2        | **Layer 2: User Config**       | Files in `GetAppConfigDir`.                          | Application-specific settings               |
| 1 (top)  | **Layer 3: Runtime Overrides** | Environment variables, CLI flags, API inputs.        | Dynamic configuration at execution time     |

**Precedence**: Layer 3 (runtime) > Layer 2 (user) > Layer 1 (SSOT defaults)

Higher priority layers overwrite conflicting keys from lower layers. Arrays MUST replace the prior value unless the specification for a given schema explicitly opts into merge semantics.

## Requirements

1. Provide helper functions to load each layer independently and merge into a single configuration object.
2. Validate merged configuration against the relevant schema before exposing it to consumers.
3. Surface provenance metadata (which layer provided which key) for diagnostics when possible.
4. Expose hooks for applications to supply additional runtime layers (e.g., secrets fetched at startup).

## Merge Semantics

- **Scalars/Objects**: last writer wins (`runtime > user > defaults`).
- **Arrays**: replaced entirely by the highest-priority layer unless schema defines a merge strategy. Document
  merge strategies within the schema (e.g., unique union for plugin lists).
- **Null Values**: treat explicit `null` in higher layers as a request to delete the key (omit from final
  configuration) unless prohibited by schema.

## Language Implementation Notes

- **Go**: Use `gopkg.in/yaml.v3` for parsing defaults/user layers, convert to `map[string]any`, merge, then
  validate via Goneat (`goneat schema validate-data`). Provide `LoaderOptions` for injecting additional tiers.
- **Python**: Use `ruamel.yaml` for round-trip support, merge using recursive dict logic, then validate with
  `jsonschema` or the `goneat` CLI via subprocess.
- **TypeScript**: Use `yaml` package, merge with `deepmerge-ts` (custom config to replace arrays), validate with
  `ajv`. Provide ESM-friendly APIs.

## Environment Variable Overrides

Helper libraries MUST expose a mapping from environment variables to configuration keys (document in README).
Prefer prefixing env vars with the application name (`APP_*`). Boolean values accept `true|false|1|0`.

## CLI Overrides

CLI tools SHOULD expose `--config`, `--set key=value`, or similar flags that inject runtime overrides at the top
layer. Helper libraries provide parsing utilities to merge these values safely.

## Testing Requirements

- Unit tests for each merge scenario (scalar, object, array, null deletion).
- Schema validation tests ensuring invalid configs raise descriptive errors.
- Integration tests verifying environment variable and CLI flag overrides.

## Related Documents

- `./config-path-api.md`
- `docs/standards/config/fulmen-config-paths.md`
- `schemas/taxonomy/repository-category/v1.0.0/category-key.schema.json`
