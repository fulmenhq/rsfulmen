---
title: "Fulmen Language Taxonomy"
description: "Canonical JSON Schema definitions for Fulmen helper language keys and metadata"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["schema", "taxonomy", "language", "2025.10.2"]
---

# Fulmen Language Taxonomy

This directory contains machine-readable definitions for the Fulmen helper language catalog. The schemas
provide reusable `$id` URLs so downstream manifests can reference the same enums for validation.

## Files

- `language-key.schema.json` – Enumerates the canonical helper language keys (`go`, `python`, `typescript`, `rust`, `csharp`).
- `language-metadata.schema.json` – Describes the metadata object captured in `config/taxonomy/languages.yaml`.

## Usage

```bash
goneat schema validate-data \
  --schema schemas/taxonomy/language/v1.0.0/language-metadata.schema.json \
  --data config/taxonomy/languages.yaml
```

Schemas target the 2020-12 draft and expose stable `$id` values for reuse by the library module manifest and
other standards artifacts introduced in the v2025.10.2 cycle.
