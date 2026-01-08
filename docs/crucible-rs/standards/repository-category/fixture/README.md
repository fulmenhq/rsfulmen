---
title: "Fixture Category Standards"
description: "Standards and requirements for Fulmen fixture repositories"
author: "Schema Cartographer"
date: "2026-01-06"
last_updated: "2026-01-08"
status: "active"
tags: ["standards", "repository-category", "fixture", "testing", "v0.4.4"]
---

# Fixture Category Standards

## Purpose

This document defines the category-specific standards for `fixture` repositories in the Fulmen ecosystem. Fixtures provide controlled test infrastructure - real implementations with test-purpose configuration - for integration testing.

## Category Definition

**Category Key**: `fixture`

**Summary**: Test infrastructure providing real-but-test-purpose servers, clients, or datastores.

**Layer**: 2 (Applications) - Fixtures are specialized applications that consume helper libraries but do not export code.

## Key Distinction: Fixtures vs Mocks

| Term        | Behavior                                        | Example                         |
| ----------- | ----------------------------------------------- | ------------------------------- |
| **Fixture** | Real execution, real code paths, synthetic data | OAuth2 server with test users   |
| **Mock**    | Simulated response, no real execution           | `unittest.mock`, WireMock stubs |

Fixtures execute real logic - they are production-grade software configured for testing, not simulations.

## Naming Convention

### Pattern

```
fixture-<mode>-<category>-<name>
```

### Components

| Component  | Constraint                                                        | Example    |
| ---------- | ----------------------------------------------------------------- | ---------- |
| `mode`     | `server`, `client`, `datastore`, `identity`                       | `server`   |
| `category` | `proving`, `utility`, `chaos`                                     | `proving`  |
| `name`     | ASCII `[a-z][a-z0-9]{1,20}`, no separators, registered in catalog | `gauntlet` |

### Full Examples

- `fixture-server-proving-gauntlet` - Protected backend, OAuth2
- `fixture-server-proving-rampart` - HTTP protocol testing server
- `fixture-server-chaos-gremlin` - Chaos server, failure injection
- `fixture-datastore-utility-bucket` - S3-compatible storage

### Variant Strategy

Different configurations of the same fixture are handled via:

- **Scenario files** for runtime configuration (`scenarios/oauth-full.yaml`)
- **Docker tags** for versioned variants (`fixture-server-proving-gauntlet:oauth-minimal`)

## Modes

| Mode        | Purpose                            | Status         |
| ----------- | ---------------------------------- | -------------- |
| `server`    | Backend APIs (REST, gRPC, GraphQL) | Active         |
| `client`    | Clients for server/API testing     | Active         |
| `datastore` | Databases, caches, message queues  | Active         |
| `identity`  | IdP/authentication (OIDC, SAML)    | Planned v0.4.3 |

## Behavioral Categories

Categories describe behavioral purpose and appear in the repository name for discoverability:

| Category    | Behavior                            | Naming Theme                |
| ----------- | ----------------------------------- | --------------------------- |
| **proving** | Validates caller, test-purpose data | gauntlet, sentinel, bastion |
| **utility** | Trivial but reliable service        | echo, static, relay         |
| **chaos**   | Deliberately unreliable/hostile     | gremlin, jinx, havoc        |

Including category in the name enables sorting and identification at a glance when managing many fixtures.

## Security Constraints (Inviolate)

These constraints are NOT negotiable for public fixtures:

| Constraint                 | Requirement                                             |
| -------------------------- | ------------------------------------------------------- |
| **No PII**                 | All data must be synthetic/anonymized                   |
| **No NPI/MNPI**            | No non-public information (regulatory risk)             |
| **No proprietary tooling** | No non-public interface implementations in public repos |

### Data Requirements

- User data: Generated names, emails, identifiers
- Credentials: Well-known test values (`sk-test-xxx`)
- API keys: Clearly fake, non-functional
- Business data: Synthetic, non-sensitive

## Technical Requirements

### Container-First

Every fixture MUST be runnable via:

```bash
docker compose up
```

Required file: `docker-compose.yml` at repository root.

### Scenario-Driven Configuration

Behavior MUST be configurable via YAML/JSON files, not code changes:

```yaml
# scenarios/default.yaml
name: "Default Scenario"
endpoints:
  - path: /health
    auth: optional
```

### Observability

Fixtures SHOULD produce structured JSON logs to stderr:

```json
{
  "level": "info",
  "path": "/api/users",
  "method": "GET",
  "authenticated": true,
  "status": 200
}
```

### Stateless Default

- No persistence between restarts
- Clean slate ensures test isolation
- Optional persistence via volume mounts

## Registration Requirement

All fixture names MUST be registered in `config/taxonomy/fixture-catalog.yaml` before creating a repository.

### Process

1. Submit PR to Crucible adding entry to fixture catalog
2. Get catalog entry approved
3. Create fixture repository with registered name
4. Add variants as needed

## Helper Library Usage

**SHOULD** (not MUST) use helper library.

Rationale: Many fixtures wrap existing images (authentik, keycloak) where adding gofulmen is impractical. The key requirement is observability, not implementation method.

If NOT using helper library:

- MUST produce structured logs (JSON)
- SHOULD follow exit code conventions
- SHOULD handle signals gracefully

## Directory Structure

```
fixture-server-proving-gauntlet/
├── docker-compose.yml      # Required: primary entry
├── Dockerfile
├── INTEGRATION.md          # Required: external dependencies & contracts
├── scenarios/
│   ├── default.yaml
│   └── oauth-full.yaml
├── src/
├── README.md
└── docs/
    └── scenarios.md
```

## Documentation Requirements

### README.md

- Purpose (one sentence)
- Quick start (`docker compose up`)
- Available scenarios
- Endpoints and auth requirements
- Environment variables

### Scenario Documentation

Each scenario should document:

- Purpose and use case
- Endpoint behaviors
- Request/response examples

## Versioning

| Concern          | Location              |
| ---------------- | --------------------- |
| Fixture identity | Repository/image name |
| Fixture version  | Docker tag            |

```bash
docker pull ghcr.io/fulmenhq/fixture-server-proving-gauntlet:v1.2.3
```

Never encode version in repository name.

## Compliance Checklist

### Naming & Registration

- [ ] Name follows pattern: `fixture-<mode>-<category>-<name>`
- [ ] Registered in `config/taxonomy/fixture-catalog.yaml`

### Security

- [ ] No PII in test data
- [ ] No NPI/MNPI

### Infrastructure

- [ ] `docker-compose.yml` at root
- [ ] `INTEGRATION.md` with required template sections
- [ ] Scenario-driven configuration
- [ ] Structured logging
- [ ] Stateless by default

### CLI Commands (Mandatory)

- [ ] `version` command implemented
- [ ] `doctor` command implemented
- [ ] Default behavior (no args) starts server

### Documentation

- [ ] README with usage examples
- [ ] Scenarios documented

## Related Documentation

- [Fulmen Fixture Standard](../../../architecture/fulmen-fixture-standard.md) - Full specification
- [Fixture Catalog](../../../../config/taxonomy/fixture-catalog.yaml) - Name registry
- [Fixture Catalog Schema](../../../../schemas/taxonomy/fixture/v1.0.0/fixture-catalog.schema.json)
- [Repository Categories](../../../../config/taxonomy/repository-categories.yaml)

---

**Status**: Active (v0.4.4+)
**Maintainers**: Crucible Team
