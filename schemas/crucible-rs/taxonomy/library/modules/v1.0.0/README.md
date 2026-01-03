# Helper Library Module Taxonomy Schema

**Version**: 1.0.0
**Status**: Active
**Owner**: Schema Cartographer

## Purpose

This schema defines the structure for individual module entries in the Fulmen Helper Library Module Registry (`config/taxonomy/library/modules/v1.0.0/modules.yaml`).

## Files

- **`module-entry.schema.json`**: JSON Schema for individual module entries

## Module Tiers

### Core

- **Definition**: Always present in all languages, zero optional dependencies
- **Rules**: Must be `available` in all languages (no `na` status, no `tier_override`)
- **Examples**: config, logging, schema

### Common

- **Definition**: Default install, lightweight
- **Rules**: Can have `tier_override` to `specialized` or `na` (with rationale)
- **Examples**: pathfinder, ascii, foundry

### Specialized

- **Definition**: Opt-in via extras/optional installs
- **Rules**: Requires `graduation_metrics` and `sunset_date`
- **Examples**: fulencoding (planned v0.2.11)

## Language-Specific Tier Overrides

Modules can have different tiers per language based on implementation realities:

**Example**: Archive handling module

- Go: `common` (stdlib `archive/tar`, `archive/zip`)
- Python: `common` (stdlib `tarfile`, `zipfile`)
- TypeScript: `specialized` (requires external `node-tar`, `archiver` packages)

Overrides require documented `rationale` for transparency.

## Evidence Pointers

Modules include pointers to their artifacts for automated validation:

- `has_schema`: Whether module has JSON schemas
- `schema_path`: Path to schema directory (e.g., `schemas/pathfinder/v1.0.0/`)
- `has_config`: Whether module has config files
- `config_path`: Path to config directory (e.g., `config/library/foundry/`)

## Validation

The module registry is validated by `scripts/validate-module-registry.ts` which:

1. Scans Crucible for evidence of modules (schemas, configs, packages)
2. Detects orphaned schemas (no registry entry)
3. Detects dead entries (registry entry with no evidence)
4. Validates evidence pointers match reality
5. Validates cross-language status matches actual packages
6. Enforces core universality rules
7. Validates module cross-references

## Usage

The registry is consumed by:

- Helper libraries (gofulmen, pyfulmen, tsfulmen) to know which modules to implement
- Validation scripts to ensure registry accuracy
- Documentation generators for module listings
- Parity tracking tools for cross-language status

## Related

- **Registry**: `config/taxonomy/library/modules/v1.0.0/modules.yaml`
- **Validation**: `scripts/validate-module-registry.ts`
- **Standard**: `docs/standards/library/modules/extension-tiering.md` (v0.2.11+)
- **Helper Library Standard**: `docs/architecture/fulmen-helper-library-standard.md`

---

_Schema created v0.2.10 by Schema Cartographer under supervision of @3leapsdave_
