---
title: "Fulmen Repository Category Taxonomy"
description: "Canonical JSON Schema definitions for Fulmen repository category keys and metadata"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["schema", "taxonomy", "repository", "2025.10.2"]
---

# Fulmen Repository Category Taxonomy

Machine-readable enums for Fulmen repository/category templates. These schemas back the
`docs/standards/repository-structure/` taxonomy and can be reused by manifests, scaffolding tools, and CI
validation.

## Files

- `category-key.schema.json` – Enumerates the canonical repository category keys (`cli`, `workhorse`, `service`, `library`, `pipeline`, `codex`, `sdk`).
- `category-metadata.schema.json` – Describes the metadata objects stored in `config/taxonomy/repository-categories.yaml`.

## Usage

```bash
goneat schema validate-data \
  --schema schemas/taxonomy/repository-category/v1.0.0/category-metadata.schema.json \
  --data config/taxonomy/repository-categories.yaml
```

Downstream specs (e.g., library module manifests) can `$ref` `category-key.schema.json` to stay aligned with
the canonical taxonomy.
