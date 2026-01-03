---
title: "Repository Category Standards"
description: "Category-specific standards for Fulmen repository types"
author: "Schema Cartographer"
date: "2025-11-03"
last_updated: "2026-01-06"
status: "active"
tags: ["standards", "repository-category", "organization", "v0.4.2"]
---

# Repository Category Standards

## Purpose

This directory contains standards, configuration schemas, and documentation specific to each repository category in the Fulmen ecosystem. These category-specific standards complement ecosystem-wide standards and provide specialized guidance for each repository type.

## Repository Categories

Repository categories are defined in the taxonomy: `schemas/taxonomy/repository-category/v1.0.0/category-key.schema.json`

Current categories: `cli`, `codex`, `doc-host`, `fixture`, `library`, `microtool`, `missive`, `pipeline`, `sdk`, `service`, `spec-host`, `workhorse`

## Structure

```
repository-category/
├── codex/
│   └── config-standard.md          # Codex configuration schema standard
├── doc-host/
│   └── README.md                   # Doc-host category requirements
├── fixture/
│   └── README.md                   # Fixture category requirements
├── missive/
│   └── README.md                   # Missive category requirements
├── spec-host/
│   └── README.md                   # Spec-host category requirements
├── workhorse/
│   └── (future workhorse-specific standards)
├── helper-library/
│   └── (future library-specific standards)
└── README.md                        # This file
```

## Parallel Schema Organization

Category-specific schemas are organized under `schemas/config/repository-category/`:

```
schemas/config/repository-category/
├── codex/v1.0.0/
│   └── codex-config.schema.json
├── workhorse/v1.0.0/
│   └── (future schemas)
└── helper-library/v1.0.0/
    └── (future schemas)
```

This creates symmetry between schemas and documentation:

- **Schemas**: `schemas/config/repository-category/{category}/v1.0.0/`
- **Documentation**: `docs/standards/repository-category/{category}/`

## Current Standards

### Codex

**Category**: `codex` (human-first documentation sites and knowledge hubs)

**Summary**: Documentation-first static sites with rich browsing, search, and navigation. For machine-first spec hosting, see spec-host.

**Standards**:

- [Codex Configuration Standard](codex/config-standard.md) - Configuration schema and governance

**Schemas**:

- `schemas/config/repository-category/codex/v1.0.0/codex-config.schema.json` - Configuration validation schema

**Examples**:

- `examples/config/repository-category/codex/v1.0.0/codex-config.example.json` - Reference configuration

**Applies To**:

- Forge Codex Pulsar (active)
- Future codex templates (Aurora, Nebula, etc.)

**Note**: Codex sites may layer browsable UI over a spec-host or doc-host corpus.

### Spec-Host

**Category**: `spec-host` (machine-first self-describing specification hosting)

**Summary**: Static hosting for self-describing specification artifacts (JSON Schema, OpenAPI, AsyncAPI) where assets contain embedded canonical IDs that must resolve over HTTPS.

**Standards**:

- [Spec-Host Category Standards](spec-host/README.md) - Category requirements
- [Spec Publishing Standard](../publishing/spec-publishing.md) - Publishing workflow contract

**Key Invariant**: Every `$id` (JSON Schema) or `x-fulmen-id` (OpenAPI/AsyncAPI) MUST resolve over HTTPS. Assets are validated against industry-standard meta-schemas.

**Applies To**:

- Crucible schema publishing (planned)
- Future `forge-spec-host-*` templates

**Note**: For path-addressed assets without embedded IDs, see doc-host.

### Doc-Host

**Category**: `doc-host` (machine-first path-addressed asset hosting)

**Summary**: Static hosting for documentation, configuration, and reference assets where canonical URLs are derived from file paths, not embedded identifiers. No meta-validation against industry-standard specifications is required.

**Standards**:

- [Doc-Host Category Standards](doc-host/README.md) - Category requirements
- [Canonical URI Resolution Standard](../publishing/canonical-uri-resolution.md) - URI structure and resolver contract

**Key Invariant**: Published file paths directly determine canonical URLs. No embedded identifiers required.

**Applies To**:

- Crucible documentation publishing (planned: `docs.fulmenhq.dev`)
- Crucible configuration publishing (planned: `config.fulmenhq.dev`)
- Future `forge-doc-host-*` templates

**Note**: Doc-host and spec-host are complementary. A Codex site may layer browsable UI over either.

### Fixture

**Category**: `fixture` (test infrastructure with real-but-test-purpose implementations)

**Summary**: Controlled test infrastructure providing real servers, clients, or datastores with synthetic data. Distinct from mocks (simulated responses) - fixtures execute real code paths.

**Standards**:

- [Fixture Category Standards](fixture/README.md) - Category requirements
- [Fulmen Fixture Standard](../../architecture/fulmen-fixture-standard.md) - Full specification

**Schemas**:

- `schemas/taxonomy/fixture/v1.0.0/fixture-catalog.schema.json` - Fixture registry validation

**Registry**:

- `config/taxonomy/fixture-catalog.yaml` - All fixture names must be registered here

**Key Constraints** (Inviolate):

- No PII (synthetic data only)
- No NPI/MNPI (regulatory exposure)
- No non-public interface tooling in public repos
- Container-first (`docker compose up`)
- Scenario-driven configuration (YAML/JSON)

**Naming Pattern**: `fixture-<mode>-<category>-<name>-<variant>` (e.g., `fixture-server-proving-gauntlet-001`)

**Modes**:

- `server` - Backend APIs (REST, gRPC, GraphQL)
- `client` - Clients for server testing
- `datastore` - Databases, caches, message queues
- `identity` - IdP/authentication (planned v0.4.3)

**Behavioral Categories** (in name for discoverability):

- `proving` - Validates caller (gauntlet, sentinel)
- `utility` - Trivial but reliable (echo, static)
- `chaos` - Deliberately unreliable (gremlin, jinx)

**Applies To**:

- Integration test suites
- CI/CD pipelines
- Local development environments

### Missive

**Category**: `missive` (single-page promotional/CTA sites)

**Summary**: Lightweight single-page sites for event announcements, fundraisers, product launches. Vanilla HTML/CSS first; escalate to Codex when scope grows.

**Standards**:

- [Missive Category Standards](missive/README.md) - Category requirements

**Key Principle**: Start with zero build; add complexity only when justified.

**Applies To**:

- Event announcement pages
- Charity fundraiser sites
- Product launch pages
- Future `forge-missive-*` templates

## Future Standards (Planned)

### Workhorse

**Category**: `workhorse` (application services, API servers)

**Potential Standards**:

- Server configuration patterns
- Deployment manifests
- Health check requirements
- API documentation standards

### Helper Library

**Category**: `library` (language-specific helper libraries)

**Potential Standards**:

- Module implementation patterns
- Testing requirements
- API surface consistency
- Sync procedures

## When to Add Category-Specific Standards

Use category-specific standards when:

1. **Specialized Requirements**: Requirements unique to a category (e.g., codex config, spec-host publishing)
2. **Clear Scope**: Applies to all repositories in a category, not just one instance
3. **Schema-Driven**: Configuration or validation schemas that enforce category standards
4. **Governance Value**: Standardization benefits ecosystem consistency

**Do NOT** add category-specific standards for:

- One-off requirements (use repository-specific docs)
- Ecosystem-wide concerns (use top-level `docs/standards/`)
- Implementation details (use repository README or guides)

## Relationship to Other Standards

**Ecosystem-Wide Standards** (`docs/standards/`):

- Apply to ALL repositories regardless of category
- Examples: coding standards, observability, error handling, publishing

**Category-Specific Standards** (`docs/standards/repository-category/`):

- Apply to all repositories in a specific category
- Examples: codex config, spec-host publishing requirements, missive constraints

**Repository-Specific Documentation** (in each repo):

- Apply only to that specific repository
- Examples: forge-codex-pulsar setup, tsfulmen API reference

## Adding New Category Standards

**Process**:

1. **Identify Need**: Standardization benefits multiple repositories in category
2. **Proposal**: Create issue or PR with proposed standard
3. **Review**: Crucible maintainers review with category experts
4. **Create Schema** (if applicable): Add to `schemas/config/repository-category/{category}/v1.0.0/`
5. **Document**: Create standard doc in `docs/standards/repository-category/{category}/`
6. **Examples**: Add reference implementations to `examples/config/repository-category/{category}/v1.0.0/`
7. **Release**: Include in next Crucible version

## Related Documentation

- [Repository Category Taxonomy](../../../schemas/taxonomy/repository-category/v1.0.0/category-key.schema.json) - Canonical category definitions
- [Repository Categories Config](../../../config/taxonomy/repository-categories.yaml) - Category metadata
- [Fulmen Ecosystem Guide](../../architecture/fulmen-ecosystem-guide.md) - Overview of repository types
- [Repository Structure Standards](../repository-structure/README.md) - Directory structure patterns by language
- [Publishing Standards](../publishing/README.md) - Artifact publishing standards

---

**Status**: Active (v0.4.2+)

**Maintainers**: Crucible Team
