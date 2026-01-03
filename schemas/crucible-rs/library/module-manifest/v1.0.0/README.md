---
title: "Library Module Manifest Schema"
description: "JSON Schema for Fulmen helper module manifest files"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["schema", "library", "manifest", "2025.10.2"]
---

# Library Module Manifest Schema

Defines the structure of `config/library/v1.0.0/module-manifest.yaml`, the SSOT describing core and extension
modules across Fulmen helper libraries.

## Files

- `module-manifest.schema.json` – Main manifest schema.

## Validation

```
goneat validate data \
  --schema-file schemas/library/module-manifest/v1.0.0/module-manifest.schema.json \
  --data config/library/v1.0.0/module-manifest.yaml
```

The schema depends on taxonomy enums defined in:

- `schemas/taxonomy/language/v1.0.0/language-key.schema.json`
- `schemas/taxonomy/repository-category/v1.0.0/category-key.schema.json`
