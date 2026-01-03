# Crucible Architecture Documentation

Architectural decisions, design patterns, and technical standards that govern the Fulmen ecosystem.

## Overview

This directory contains the authoritative architecture documentation for Crucible and the broader Fulmen ecosystem, including the layer cake model, SSOT synchronization patterns, and cross-language library standards.

## Core Architecture Documents

### Ecosystem Foundation

#### [Fulmen Technical Manifesto](fulmen-technical-manifesto.md)

Core principles and tenets guiding all Fulmen development.

**Key Principles**:

- Start Fast, Thrive on Scale
- Be Persnickety About Code
- Build for Clarity
- Fail Fast, Recover Faster
- Documentation as Code

**When to read**: Understanding the "why" behind Fulmen architectural decisions.

#### [Fulmen Ecosystem Guide](fulmen-ecosystem-guide.md)

Complete overview of the Fulmen layer cake and how components interact.

**Covers**:

- Layer 0: Crucible (Infoarch SSOT)
- Layer 1: Helper Libraries (gofulmen, pyfulmen, tsfulmen)
- Layer 2: Templates (Forge Workhorses)
- Layer 3: DX/Dev Tools (goneat, fulward, pathfinder)
- Layer 4: Apps/Services (brooklyn-mcp, sumpter, analytics)
- Virtuous flywheel and feedback loops

**When to read**: Onboarding to the Fulmen ecosystem or understanding component relationships.

### Repository & Sync Architecture

#### [Pseudo-Monorepo](pseudo-monorepo.md)

Crucible's asymmetric repository structure balancing SSOT discipline with language ecosystem conventions.

**Topics**:

- Why asymmetric (Go at root, Python/TypeScript in `lang/`)
- SSOT authority (schemas/, docs/, config/)
- Sync automation vs embedding (`//go:embed` vs copied assets)
- Unified versioning strategy

**When to read**: Understanding why Crucible's structure differs from typical monorepos.

#### [Sync Model](sync-model.md)

Detailed explanation of how SSOT assets propagate from Crucible to language wrappers and downstream projects.

**Topics**:

- Sync producers (Crucible maintainers)
- Sync consumers (gofulmen, pyfulmen, tsfulmen, templates)
- Sync mechanisms (embedding, copying, pull scripts)
- Version alignment strategies
- Conflict resolution

**When to read**: Implementing sync in a new library or debugging sync issues.

#### [Library Ecosystem](library-ecosystem.md)

Multi-language support architecture and cross-language consistency patterns.

**Topics**:

- Schema normalization contract (camelCase → language conventions)
- Type-safe accessor patterns
- Embedded vs synced assets
- Testing strategies across languages

**When to read**: Adding language support or ensuring cross-language compatibility.

### Library Standards

#### [Fulmen Helper Library Standard](fulmen-helper-library-standard.md)

Comprehensive standard for building helper libraries (gofulmen, pyfulmen, tsfulmen).

**Requirements**:

- Repository structure and naming
- Module organization
- Crucible integration
- Testing and quality gates
- Documentation requirements
- Version management

**When to read**: Creating or auditing a Fulmen helper library.

#### [Fulmen Forge Workhorse Standard](fulmen-forge-workhorse-standard.md)

Standard for backend service templates (Groningen, Percheron).

**Requirements**:

- Service architecture patterns
- API standards (REST, gRPC)
- Observability integration
- Configuration management
- Deployment patterns

**When to read**: Building a new Forge Workhorse or standardizing an existing service.

## Architecture Decision Records (ADRs)

Architectural decisions affecting the entire ecosystem are documented in [decisions/](decisions/README.md).

**Recent significant ADRs**:

- [ADR-0010: Semantic Versioning Adoption](decisions/ADR-0010-semantic-versioning-adoption.md) - Switch from CalVer to SemVer
- [ADR-0009: Go Module Root Relocation](decisions/ADR-0009-go-module-root-relocation.md) - Move Go module to repository root
- [ADR-0008: Helper Library Instrumentation Patterns](decisions/ADR-0008-helper-library-instrumentation-patterns.md) - Telemetry integration
- [ADR-0004: Schema-Driven Config Hydration](decisions/ADR-0004-schema-driven-config-hydration.md) - Type-safe config patterns
- [ADR-0001: Two-Tier ADR System](decisions/ADR-0001-two-tier-adr-system.md) - Ecosystem vs library-specific ADRs

See [decisions/README.md](decisions/README.md) for the complete ADR index.

## Domain Architecture

- [Enact Overview](enact/overview.md) - What Enact is and how it works.
- [Enact Schema Architecture](enact/schema-architecture.md) - Layer model + schema tree map.
- [Enact Schema Examples](enact/schema-examples.md) - Example instance documents.

## Module Documentation

Standards for specific library modules (Config Path API, Crucible Shim, Docscribe, etc.) are in [modules/](modules/README.md).

## Quick Reference

### By Topic

**Understanding the Ecosystem**:

1. [Fulmen Technical Manifesto](fulmen-technical-manifesto.md) - Core principles
2. [Fulmen Ecosystem Guide](fulmen-ecosystem-guide.md) - Layer cake overview
3. [Pseudo-Monorepo](pseudo-monorepo.md) - Repository structure

**Building Libraries**:

1. [Fulmen Helper Library Standard](fulmen-helper-library-standard.md) - Requirements
2. [Library Ecosystem](library-ecosystem.md) - Cross-language patterns
3. [Sync Model](sync-model.md) - SSOT integration

**Decision History**:

- [decisions/README.md](decisions/README.md) - ADR index

### By Role

**Crucible Maintainer**:

- All documents, especially [Sync Model](sync-model.md) and [Pseudo-Monorepo](pseudo-monorepo.md)

**Library Developer**:

- [Fulmen Helper Library Standard](fulmen-helper-library-standard.md)
- [Library Ecosystem](library-ecosystem.md)
- [Sync Model](sync-model.md)

**Application Developer**:

- [Fulmen Ecosystem Guide](fulmen-ecosystem-guide.md)
- [Fulmen Technical Manifesto](fulmen-technical-manifesto.md)

**New to Fulmen**:

1. [Fulmen Technical Manifesto](fulmen-technical-manifesto.md)
2. [Fulmen Ecosystem Guide](fulmen-ecosystem-guide.md)
3. Browse [decisions/README.md](decisions/README.md)

## Related Documentation

### Standards

- [Coding Standards](../standards/coding/README.md) - Language-specific best practices
- [Library Standards](../standards/library/README.md) - Module-specific standards
- [API Standards](../standar../protocol/README.md) - REST/gRPC patterns

### Guides

- [Integration Guide](../guides/integration-guide.md) - Using Crucible in your project
- [Library Bootstrap Guide](../guides/fulmen-library-bootstrap-guide.md) - Creating new libraries

### Operations

- [Repository Operations SOP](../sop/repository-operations-sop.md) - Day-to-day procedures
- [Repository Structure SOP](../sop/repository-structure.md) - Layout requirements

## Contributing

Architecture decisions require broader discussion and approval.

**Process**:

1. Open a discussion: https://github.com/fulmenhq/crucible/discussions
2. Propose an ADR using [decisions/template.md](decisions/template.md)
3. Submit PR for review by @3leapsdave and ecosystem maintainers
4. Once approved, the ADR becomes authoritative

**For questions**: https://github.com/fulmenhq/crucible/issues

---

**Note**: Architecture documentation is maintained in the Crucible SSOT and automatically synced to language wrappers (`lang/*/docs/architecture/`).
