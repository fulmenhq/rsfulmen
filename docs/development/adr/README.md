# Architecture Decision Records (rsfulmen)

This directory contains **local ADRs** for rsfulmen-specific decisions. For ecosystem-wide ADRs that apply across all Fulmen helper libraries, see [`../../crucible-rs/architecture/decisions/`](../../crucible-rs/architecture/decisions/).

## ADR Index

| ID  | Title               | Status | Date |
| --- | ------------------- | ------ | ---- |
| -   | (No local ADRs yet) | -      | -    |

## When to Write a Local ADR

Write a local ADR when making decisions that are:

- Implementation details unique to Rust
- Tooling/dependency choices (e.g., which crate to use)
- Performance optimizations specific to the Rust runtime
- Rust idiom preferences
- Test framework choices
- Build/packaging decisions

## When to Promote to Ecosystem ADR

Promote to an ecosystem ADR (in Crucible) when:

- Decision affects API contracts between languages
- Pattern must be consistent across Go/Python/TypeScript/Rust
- Schema structure or field naming is involved
- Other languages must implement the same behavior

## ADR Format

Use the template from [0001-template.md](0001-template.md) for new ADRs.

Template is based on the Crucible standard: [`../../crucible-rs/architecture/decisions/template.md`](../../crucible-rs/architecture/decisions/template.md)

## Ecosystem ADR Adoption Status

Track implementation status of ecosystem ADRs here:

| Ecosystem ADR                                                                                                                 | Status      | Notes                                                                            | Related Local ADRs |
| ----------------------------------------------------------------------------------------------------------------------------- | ----------- | -------------------------------------------------------------------------------- | ------------------ |
| [ADR-0002: Triple-Index Catalog Strategy](../../crucible-rs/architecture/decisions/ADR-0002-triple-index-catalog-strategy.md) | verified    | Implemented in foundry catalogs                                                  | -                  |
| [ADR-0003: Progressive Logging Profiles](../../crucible-rs/architecture/decisions/ADR-0003-progressive-logging-profiles.md)   | implemented | logging module with SIMPLE/STRUCTURED profiles (v0.1.3)                          | -                  |
| [ADR-0005: CamelCase Mapping](../../crucible-rs/architecture/decisions/ADR-0005-camelcase-to-language-conventions.md)         | verified    | snake_case per Rust conventions; serde camelCase in pathfinder, fulhash (v0.1.3) | -                  |
| [ADR-0006: Error Data Models](../../crucible-rs/architecture/decisions/ADR-0006-error-data-models.md)                         | implemented | error_handling module                                                            | -                  |
| [ADR-0010: Semantic Versioning Adoption](../../crucible-rs/architecture/decisions/ADR-0010-semantic-versioning-adoption.md)   | verified    | Cargo.toml + VERSION file, signed tag workflow                                   | -                  |
| [ADR-0013: Rust Sync Pattern Validation](../../crucible-rs/architecture/decisions/ADR-0013-rust-sync-pattern-validation.md)   | verified    | patterns module with regex/glob                                                  | -                  |

**Status values:**

- `not-applicable` - Does not apply to rsfulmen
- `deferred` - Postponed with documented rationale
- `planned` - Implementation planned but not started
- `in-progress` - Active implementation underway
- `implemented` - Fully implemented, ready for validation
- `verified` - Implemented and validated through tests
