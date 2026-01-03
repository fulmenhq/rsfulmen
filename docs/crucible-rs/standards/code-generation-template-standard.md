---
title: "Code Generation Template Standard"
description: "Common structure and workflow for publishing language bindings derived from Crucible catalogs"
author: "Fulmen Enterprise Architect (@fulmen-ea-steward)"
date: "2025-10-31"
last_updated: "2025-10-31"
status: "draft"
tags: ["standards", "codegen", "templates", "2025.10.3"]
---

# Code Generation Template Standard

This standard defines how Crucible publishes language-specific code generation templates for assets such as exit codes, HTTP status catalogs, or other SSOT datasets. The goal is to keep helper libraries in sync with a single source of truth, avoid hand-crafted drift, and provide a repeatable workflow that future catalog additions can reuse.

## Scope

- Applies to templates that generate language bindings or helper constants from Crucible data (YAML/JSON catalogs, schemas, configuration).
- Templates live in the `scripts/codegen/` directory of Crucible and are consumed by downstream repositories (helper libraries, codex forges, etc.).
- Covers Go, Python, TypeScript initially; other languages may be added following the same pattern.

## Tooling

- Code generation scripts MUST execute with **Bun ≥ 1.2**. Node.js 20+ may serve as a compatibility fallback but Bun is the supported runtime for new templates.
- Recommended template engines:
  - Go: built-in `text/template`
  - Python: Jinja2 rendered via Bun (e.g., Nunjucks) or delegated to Python if necessary
  - TypeScript/JavaScript: EJS/Handlebars (or similar) executed from Bun

## ⚠️ CRITICAL: Go Module Architecture (ADR-0009 Compliance)

**All Go code generation MUST comply with [ADR-0009: Go Module Root Relocation](../architecture/decisions/ADR-0009-go-module-root-relocation.md).**

### Go-Specific Requirements

**Go files MUST be generated at repository root**, not in subdirectories:

```
✅ CORRECT: foundry/exit_codes.go           (at root, package foundry)
❌ WRONG:   lang/go/pkg/foundry/exit_codes.go  (subdirectory - violates ADR-0009)
```

**Why**: Go's module system requires all Go code at repository root for:

- External `go get` installation to work
- Direct SSOT embedding via `//go:embed schemas`
- Standard Go toolchain compatibility

**Python and TypeScript**: Remain in `lang/` subdirectories (unchanged).

### Output Path Examples (metadata.json)

```json
{
  "languages": {
    "go": {
      "template": "template.tmpl",
      "output_path": "foundry/exit_codes.go", // ← Root-relative, creates foundry/ subpackage
      "postprocess": "postprocess.sh"
    },
    "python": {
      "template": "template.jinja",
      "output_path": "lang/python/src/pyfulmen/foundry/exit_codes.py", // ← lang/ subdirectory
      "postprocess": "postprocess.sh"
    },
    "typescript": {
      "template": "template.ejs",
      "output_path": "lang/typescript/src/foundry/exitCodes.ts", // ← lang/ subdirectory
      "postprocess": "postprocess.sh"
    }
  }
}
```

**Key Principle**: Go output paths are **root-relative** and create subpackages under the main `crucible` module. Python and TypeScript output paths include `lang/<language>/` prefix.

### Common Mistakes to Avoid

❌ **DON'T**:

- Generate Go code in `lang/go/` (violates ADR-0009)
- Use `pkg/` prefix for Go paths (Go code lives at root, not in pkg/)
- Reference `../` paths for Go outputs

✅ **DO**:

- Generate Go code at root: `foundry/exit_codes.go`
- Use subpackages for organization: `foundry/`, `signals/`, `identity/`
- Keep Python/TypeScript in `lang/` subdirectories

**Reference**: See [ADR-0009](../architecture/decisions/ADR-0009-go-module-root-relocation.md) for complete rationale and architectural decisions.

---

## Directory Structure

```
scripts/
  codegen/
    README.md                          # Overview and usage instructions
    <catalog-key>/                     # slug identifying the catalog (e.g., exit-codes)
      metadata.json                    # Required metadata (see below)
      go/
        template.tmpl                  # Go template
        postprocess.sh                 # Optional formatter (gofmt, goimports)
      python/
        template.jinja                 # Python template
        postprocess.sh                 # Optional formatter (ruff, mypy)
      typescript/
        template.ejs                   # TypeScript template
        postprocess.sh                 # Optional formatter (biome, prettier)
```

- `<catalog-key>` MUST be lowercase alphanumeric with optional hyphen.
- Each language directory contains the template plus optional helper scripts for formatting or additional processing.

### metadata.json

Every catalog directory MUST include `metadata.json` with at least:

```json
{
  "catalog": "exit-codes",
  "catalog_path": "config/library/foundry/exit-codes.yaml",
  "snapshot_path": "config/library/foundry/exit-codes.snapshot.json",
  "owner": "@fulmen-ea-steward",
  "last_reviewed": "2025-10-31",
  "languages": {
    "go": {
      "template": "template.tmpl",
      "output_path": "foundry/exit_codes.go",
      "postprocess": "postprocess.sh"
    },
    "python": {
      "template": "template.jinja",
      "output_path": "lang/python/src/pyfulmen/foundry/exit_codes.py",
      "postprocess": "postprocess.sh"
    },
    "typescript": {
      "template": "template.ejs",
      "output_path": "lang/typescript/src/foundry/exitCodes.ts",
      "postprocess": "postprocess.sh"
    }
  }
}
```

**Note**: Go output path is root-relative (`foundry/exit_codes.go`) per [ADR-0009](../architecture/decisions/ADR-0009-go-module-root-relocation.md). Python and TypeScript paths include `lang/<language>/` prefix.

Automation reads ownership/version data from this file—do not rename or omit it.

## Template Expectations

1. **Source of Truth**: Templates must render code directly from the canonical catalog (YAML/JSON) stored in `config/` or `schemas/`. Avoid duplicating business logic inside templates.
2. **Formatting**: Generated code must pass language-formatting tools. Provide a post-processing script (`gofmt`, `ruff format`, `biome format`, etc.) when necessary.
3. **Idempotency**: Re-running the generator with unchanged input must produce identical output (after formatting) to ensure clean diffs.
4. **Determinism**: Do not inject timestamps or environment-dependent values into generated code unless explicitly required.
5. **Language Conventions**: Document required idioms. For example, TypeScript templates MUST emit `export const exitCodes = {...} as const;` and `export type ExitCode = typeof exitCodes[keyof typeof exitCodes];` to preserve literal types and tree-shaking.

## Generation Workflow

1. **Entry Point**: Provide a generator script (e.g., `scripts/codegen/generate-exit-codes.ts`) that reads the catalog, applies the template, writes the output file, and runs optional post-processing.
2. **CLI Shape**: Generators SHOULD expose a consistent interface:
   ```bash
   bun run scripts/codegen/generate-exit-codes \
     --lang <language> \
     --out <output-path> \
     [--format]      # run post-processing
     [--verify]      # optional parity/compile check
   ```
   Additional filters (e.g., `--include-categories`) are allowed as long as defaults reproduce the full catalog.
3. **CI Integration**: Integrate generation into `make sync` or equivalent so helper repos refresh bindings automatically.
4. **Error Handling**: Fail fast if the catalog schema changes, templates are missing, or post-processing fails.

## Ownership & Review

- `metadata.json` is the authoritative record of catalog ownership, review cadence, and output targets. Update it whenever templates or maintainers change.
- Helper library maintainers (e.g., gofulmen, pyfulmen, tsfulmen) must review template updates affecting their language to ensure compatibility with local coding guidelines.
- Coordinate template changes with corresponding updates to consuming repositories (preferably in the same release cycle).

## Consumption Guidelines

Downstream repositories must:

1. **Pin Version**: Document which Crucible release (or commit) generated the current binding. Ideally, embed the catalog version in the generated artifact.
2. **Format/Lint**: Run language-specific formatters after generation to catch errors early.
3. **CI Enforcement**: Add parity tests or snapshot comparisons to detect drift between locally generated bindings and the canonical catalog.
4. **Documentation**: Update README/changelog when regenerating bindings so consumers know when new codes or fields are available.
5. **SSOT Sync Configuration** (TypeScript and Python libraries only): Helper libraries that consume generated code via SSOT sync (e.g., tsfulmen, pyfulmen) **MUST** ensure their `.crucible/metadata/sync-keys.yaml` includes the `src/` directory from the appropriate language wrapper. Go libraries do not need this as they embed directly from the Crucible module root.

### Example: Python Library Sync Configuration

```yaml
- id: crucible.lang.python.src
  description: "Python generated code (exit codes, etc.)"
  basePath: lang/python/src/
  recommendedOutput: src/
  tags: [language, "lang:python", generated]
  metadata:
    sourceRepo: https://github.com/fulmenhq/crucible.git
    sourcePathBase: lang/python/src
```

### Example: TypeScript Library Sync Configuration

```yaml
- id: crucible.lang.typescript.src
  description: "TypeScript generated code (exit codes, etc.)"
  basePath: lang/typescript/src/
  recommendedOutput: src/
  tags: [language, "lang:ts", generated]
  metadata:
    sourceRepo: https://github.com/fulmenhq/crucible.git
    sourcePathBase: lang/typescript/src
```

**Important**: Without `src/` sync configuration, helper libraries will receive schemas, config, and docs from Crucible, but will **not** receive the generated code bindings (exit codes, etc.).

## Verification & CI Integration

Crucible itself enforces code generation parity through:

1. **Verification Script**: `scripts/codegen/verify-exit-codes.ts` (or similar per catalog) regenerates all language bindings to a temporary location, compares with checked-in files, validates compilation, and checks parity with canonical snapshots.
2. **Make Target**: `make verify-codegen` runs all verification scripts and fails if drift is detected.
3. **CI Integration**: `make verify-codegen` is part of `make check-all` (run by `make precommit` and `make prepush`), ensuring generated code stays synchronized with catalogs before every commit.

Downstream repositories should expose similar verification targets (e.g., `make verify-codegen`) that regenerate bindings and run parity checks to prevent drift.

## Future Enhancements

- Extend templates for additional languages (Rust, C#, etc.) as helper libraries adopt them.
- Explore provenance/signing (checksums, SLSA attestations) for generated bindings in future iterations to satisfy locked-environment requirements.
- Create a unified orchestration script (`scripts/codegen/generate-all.ts`) that discovers and runs all catalog generators in one invocation.

## References

- [Fulmen Forge Codex Standard](../architecture/fulmen-forge-codex-standard.md)
- [Makefile Standard](./makefile-standard.md)
- [Fulmen Ecosystem Guide](../architecture/fulmen-ecosystem-guide.md)
