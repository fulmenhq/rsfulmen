---
title: "Fulmen Fixture Standard"
description: "Standard structure and requirements for Fulmen Fixture repositories - test infrastructure providing real-but-test-purpose servers, clients, and datastores"
author: "Schema Cartographer (@schema-cartographer)"
date: "2026-01-06"
last_updated: "2026-01-06"
status: "draft"
tags: ["architecture", "fixture", "testing", "infrastructure", "v0.4.2"]
---

# Fulmen Fixture Standard

This document defines the standardized structure, naming conventions, and requirements for Fulmen Fixture repositories. Fixtures provide controlled test infrastructure - real implementations with test-purpose configuration - enabling integration testing of complex authentication/authorization flows, API interactions, and data pipelines.

## Overview

### What Fixtures Are

Fixtures are **real implementations with test-purpose configuration**, NOT mocks (simulated responses). They:

- Execute real logic (actual HTTP servers, real token validation, genuine database queries)
- Use synthetic/test-purpose data (no PII, no production secrets)
- Provide controlled, reproducible behavior for integration testing
- Run as containers with scenario-driven configuration

### What Fixtures Are NOT

| Term     | Meaning                                                               | Fixture Difference                                         |
| -------- | --------------------------------------------------------------------- | ---------------------------------------------------------- |
| **Mock** | Simulated response, no real execution (e.g., unittest.mock, WireMock) | Fixtures execute real code                                 |
| **Stub** | Minimal implementation returning canned responses                     | Fixtures have full logic paths                             |
| **Fake** | Simplified implementation (e.g., in-memory database)                  | Fixtures may be production-grade software with test config |

### Layer Placement

**Layer 2 (Applications)** - Fixtures are specialized applications that:

- Consume helper libraries (gofulmen, tsfulmen, pyfulmen)
- Do NOT export code (not libraries)
- Run as containers or processes
- Serve a testing purpose rather than production traffic

This parallels workhorse placement but with testing-specific constraints.

## Naming Convention

### Pattern

```
fixture-<mode>-<category>-<name>-<variant>
```

All components are required. No implicit defaults.

### Components

| Component    | Constraint              | Description                                                   |
| ------------ | ----------------------- | ------------------------------------------------------------- |
| `fixture`    | Literal                 | Repository type prefix                                        |
| `<mode>`     | Enum                    | `server`, `client`, `datastore`, `identity`                   |
| `<category>` | Enum                    | `proving`, `utility`, `chaos`                                 |
| `<name>`     | `^[a-z][a-z0-9]{1,20}$` | Registered name from fixture catalog (no hyphens/underscores) |
| `<variant>`  | `^[0-9]{3}$`            | 3-digit zero-padded variant code                              |

**Full regex**: `^fixture-(server|client|datastore|identity)-(proving|utility|chaos)-[a-z][a-z0-9]{1,20}-[0-9]{3}$`

### Modes

| Mode        | Purpose                            | Examples                          |
| ----------- | ---------------------------------- | --------------------------------- |
| `server`    | Backend APIs (REST, gRPC, GraphQL) | Protected backend, echo server    |
| `client`    | Clients for server/API testing     | OAuth tester, load generator      |
| `datastore` | Databases, caches, message queues  | S3-compatible storage, Redis mock |
| `identity`  | IdP/authentication (OIDC, SAML)    | OIDC provider with test users     |

**Note**: `identity` mode is planned for v0.4.3.

### Variant Codes

- `001` is the reference/default implementation
- `000` is reserved (unused)
- Sequential assignment within a fixture name
- Variants represent different configurations or capabilities of the same fixture concept

### Examples

| Full Name                              | Mode      | Category | Name     | Variant | Description                           |
| -------------------------------------- | --------- | -------- | -------- | ------- | ------------------------------------- |
| `fixture-server-proving-gauntlet-001`  | server    | proving  | gauntlet | 001     | Protected backend, OAuth2 minimal     |
| `fixture-server-proving-gauntlet-002`  | server    | proving  | gauntlet | 002     | Protected backend, OAuth2 full + PKCE |
| `fixture-server-chaos-gremlin-001`     | server    | chaos    | gremlin  | 001     | Random latency injection              |
| `fixture-server-utility-echo-001`      | server    | utility  | echo     | 001     | Simple request echo                   |
| `fixture-client-proving-probe-001`     | client    | proving  | probe    | 001     | OAuth client tester                   |
| `fixture-datastore-utility-bucket-001` | datastore | utility  | bucket   | 001     | S3-compatible minimal storage         |
| `fixture-identity-proving-warden-001`  | identity  | proving  | warden   | 001     | Full OIDC with test users             |

## Behavioral Categories

Categories describe the behavioral purpose of a fixture and appear in the repository name for discoverability at scale.

| Category    | Behavior                                            | Purpose                              | Naming Theme                                                 |
| ----------- | --------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------ |
| **proving** | Real execution, test-purpose data, validates caller | Integration testing, AAA validation  | Challenge/trial names (gauntlet, sentinel, bastion, citadel) |
| **utility** | Real execution, trivial but reliable service        | Convenience, bootstrapping, demos    | Functional/descriptive (echo, static, relay, cache)          |
| **chaos**   | Real execution, deliberately unreliable/hostile     | Resilience testing, failure handling | Mischief names (gremlin, jinx, havoc, mayhem)                |

Categories are part of the naming pattern to enable sorting and identification at a glance when managing many fixtures.

## Fixture Catalog Registry

All fixture names MUST be registered in `config/taxonomy/fixture-catalog.yaml` before creating a repository.

### Registration Process

1. Propose fixture name and category in PR to Crucible
2. Add entry to `config/taxonomy/fixture-catalog.yaml`
3. Create fixture repository with registered name
4. Add variants as needed (increment variant code)

### Registry Schema

```yaml
fixtures:
  gauntlet:
    category: proving
    mode: server
    default_lang: en
    summary: "Protected backend with mixed auth requirements"
    summary_i18n:
      ja: "混合認証要件を持つ保護されたバックエンド"
    aliases:
      - "gantlet" # Common misspelling
    variants:
      "001":
        name: "oauth-minimal"
        description: "OAuth2 with basic scopes"
      "002":
        name: "oauth-full"
        description: "OAuth2 with PKCE, refresh, introspection"
```

### i18n Support

| Field              | Constraint            | Notes                                         |
| ------------------ | --------------------- | --------------------------------------------- |
| `default_lang`     | Required, ISO 639-1   | Author's primary language                     |
| `summary`          | Required, UTF-8       | In `default_lang`                             |
| `summary_i18n`     | Optional, map         | lang code -> string (excludes `default_lang`) |
| `description`      | Optional, UTF-8       | In `default_lang`, at variant level           |
| `description_i18n` | Optional, map         | lang code -> string                           |
| `aliases`          | Optional, UTF-8 array | i18n nicknames, typo corrections              |

**Key principle**: Canonical names are ASCII (Docker/DNS safe). All human-readable metadata supports full UTF-8. Authors can write in their primary language; English is not forced.

## Security Constraints (Inviolate)

These constraints are NOT negotiable and apply to ALL public fixture repositories:

| Constraint                      | Rationale                               | Enforcement        |
| ------------------------------- | --------------------------------------- | ------------------ |
| No PII                          | Test data must be synthetic/anonymized  | PR review, CI scan |
| No NPI/MNPI                     | Regulatory/legal exposure (SEC, GDPR)   | PR review          |
| No non-public interface tooling | Prevents disclosure of proprietary APIs | PR review          |
| Observability required          | Ensures structured logging for audit    | CI check           |

**Private repositories** may relax these constraints with explicit documentation, but public fixtures in fulmenhq org MUST comply.

### Data Requirements

- All user data MUST be synthetic (generated names, emails, etc.)
- Credentials MUST be well-known test values (not production secrets)
- API keys MUST be clearly fake (`sk-test-xxx`, not real keys)
- No customer data, transaction records, or business-sensitive information

## Technical Requirements

### Container-First

Fixtures MUST be runnable via `docker compose up`:

```yaml
# docker-compose.yml (required at repo root)
services:
  fixture:
    build: .
    ports:
      - "8080:8080"
    environment:
      - SCENARIO_PATH=/scenarios/default.yaml
      - LOG_LEVEL=info
    volumes:
      - ./scenarios:/scenarios:ro
```

### Scenario-Driven Configuration

Behavior MUST be configurable via YAML/JSON, not code changes:

```yaml
# scenarios/protected-backend.yaml
name: "Protected Backend"
description: "API with mixed auth requirements"
endpoints:
  - path: /health
    method: GET
    auth: optional
    responses:
      authenticated:
        status: 200
        body: { "status": "healthy", "details": { "db": "ok", "cache": "ok" } }
      anonymous:
        status: 200
        body: { "status": "healthy" }
  - path: /api/users
    method: GET
    auth: required
    responses:
      authenticated:
        status: 200
        body: { "users": [] }
      anonymous:
        status: 401
        body: { "error": "unauthorized" }
```

### Observability

Fixtures SHOULD use helper library logging for structured output:

```go
// Structured logging for test assertions
logger.Info("request processed",
    zap.String("path", r.URL.Path),
    zap.String("method", r.Method),
    zap.Bool("authenticated", isAuth),
    zap.Int("status", statusCode),
    zap.String("correlation_id", correlationID),
)
```

If helper library is not used (thin wrapper around existing image), fixture MUST still produce structured logs (JSON) to stderr.

### Stateless Default

- No persistence between restarts (clean slate)
- Optional persistence via volume mounts for specific scenarios
- Ensures test isolation and reproducibility

### Helper Library Usage

**SHOULD** (not MUST) use helper library. Rationale:

- Many fixtures are thin wrappers around existing images (authentik, keycloak)
- Adding gofulmen to third-party images is impractical
- Key requirement is observability, not implementation method

If helper library is NOT used:

- MUST still implement structured logging
- SHOULD follow exit code conventions
- SHOULD handle signals gracefully

## Directory Structure

```
fixture-server-proving-gauntlet-001/
├── .fulmen/
│   └── app.yaml                    # App Identity manifest (if using helper lib)
├── scenarios/
│   ├── default.yaml                # Default scenario
│   ├── oauth-minimal.yaml          # Scenario variants
│   └── oauth-full.yaml
├── docker-compose.yml              # Required: primary entry point
├── Dockerfile                      # Container build
├── src/                            # Implementation (language-specific)
│   └── ...
├── README.md                       # Usage, scenarios, endpoints
├── INTEGRATION.md                  # Required: external dependencies & contracts
├── LICENSE
└── docs/
    └── scenarios.md                # Detailed scenario documentation
```

## Docker Image Versioning

Fixture identity and version are separate concerns:

| Concern          | Where It Lives  | Example                               |
| ---------------- | --------------- | ------------------------------------- |
| Fixture identity | Repo/image name | `fixture-server-proving-gauntlet-001` |
| Fixture version  | Docker tag      | `v1.2.3`, `latest`, `sha-abc123`      |

```bash
# Full image reference
docker pull ghcr.io/fulmenhq/fixture-server-proving-gauntlet-001:v1.2.3
docker pull ghcr.io/fulmenhq/fixture-server-proving-gauntlet-001:latest
```

Never encode version in repository or image name.

### Image Publishing

Each fixture repository publishes its own image to `ghcr.io/fulmenhq/fixture-*`. The `fixture-` prefix is a reserved namespace - fulmen-toolbox and other publishing repos MUST NOT create packages with this prefix.

For composed test stacks (e.g., "authentik + gauntlet + postgres"), use docker-compose files that reference individual fixture images rather than creating combined images.

## Transport Standards

All transports are allowed. Document which your fixture supports:

| Transport | Typical Use             |
| --------- | ----------------------- |
| HTTP/1.1  | Universal compatibility |
| HTTP/2    | Modern APIs             |
| gRPC      | Protobuf services       |
| WebSocket | Real-time testing       |
| Raw TCP   | Datastore protocols     |

## Documentation Requirements

### README.md (Required)

- Fixture purpose (one sentence)
- Quick start (`docker compose up`)
- Available scenarios
- Endpoint documentation
- Authentication details (if applicable)
- Environment variables

### INTEGRATION.md (Required)

Every fixture MUST include an `INTEGRATION.md` file documenting external dependencies and integration requirements. This is mandatory even for fixtures with no external dependencies (to explicitly state "none required").

**Required Template:**

```markdown
# Integration Requirements

## Overview

Brief description of what external services this fixture requires or provides.

## External Dependencies

| Dependency | Required | Purpose | Notes     |
| ---------- | -------- | ------- | --------- |
| (service)  | Yes/No   | (why)   | (details) |

<!-- If no dependencies, use: -->
<!-- | None | - | This fixture has no external dependencies | - | -->

## Environment Variables

### Required

| Variable | Description | Example   |
| -------- | ----------- | --------- |
| (var)    | (desc)      | (example) |

<!-- If no required variables, state: "No required environment variables." -->

### Optional

| Variable     | Description           | Default   |
| ------------ | --------------------- | --------- |
| `PORT`       | Listen port           | `8080`    |
| `LOG_LEVEL`  | Logging verbosity     | `info`    |
| `LOG_FORMAT` | Output format         | `json`    |
| `SCENARIO`   | Scenario file to load | `default` |

## Compose Integration

Example docker-compose snippet for integrating this fixture:

\`\`\`yaml
services:
fixture-name:
image: ghcr.io/fulmenhq/fixture-...:latest
environment: # Required # Optional
ports: - "8080:8080"
\`\`\`

## Health Check Contract

| Endpoint        | Method | Auth | Success Response        |
| --------------- | ------ | ---- | ----------------------- |
| `/health`       | GET    | None | `{"status": "healthy"}` |
| `/health/ready` | GET    | None | `{"status": "ready"}`   |

## Test Credentials

<!-- If fixture provides test users/credentials, document them here -->
<!-- All credentials MUST be synthetic/well-known test values -->

| Identity   | Secret  | Roles   | Notes   |
| ---------- | ------- | ------- | ------- |
| (user/key) | (value) | (roles) | (notes) |

<!-- If no test credentials, state: "This fixture does not manage credentials." -->
```

**Standard Environment Variables:**

All fixtures SHOULD support these standard variables:

| Variable     | Purpose                                   | Default   |
| ------------ | ----------------------------------------- | --------- |
| `PORT`       | Listen port                               | `8080`    |
| `LOG_LEVEL`  | `trace`, `debug`, `info`, `warn`, `error` | `info`    |
| `LOG_FORMAT` | `json` or `text`                          | `json`    |
| `SCENARIO`   | Scenario file/name to load                | `default` |

**OIDC-Aware Fixtures** (fixtures that validate tokens):

| Variable        | Purpose                 | Required                 |
| --------------- | ----------------------- | ------------------------ |
| `OIDC_ISSUER`   | IdP issuer URL          | Yes                      |
| `OIDC_AUDIENCE` | Expected audience claim | Yes                      |
| `OIDC_JWKS_URL` | JWKS endpoint override  | No (derived from issuer) |

**OIDC Client Fixtures** (fixtures that act as OIDC clients):

| Variable             | Purpose           | Required        |
| -------------------- | ----------------- | --------------- |
| `OIDC_ISSUER`        | IdP issuer URL    | Yes             |
| `OIDC_CLIENT_ID`     | Client identifier | Yes             |
| `OIDC_CLIENT_SECRET` | Client secret     | Depends on flow |
| `OIDC_REDIRECT_URI`  | Callback URL      | Depends on flow |

**Health Check Contract:**

All fixtures SHOULD implement:

| Endpoint            | Purpose                          | Auth |
| ------------------- | -------------------------------- | ---- |
| `GET /health`       | Basic liveness check             | None |
| `GET /health/ready` | Readiness with dependency checks | None |

Response format:

```json
{
  "status": "healthy",
  "checks": {
    "database": "ok",
    "cache": "ok"
  }
}
```

### Scenario Documentation

Each scenario should document:

- Purpose and use case
- Endpoints and their auth requirements
- Expected request/response examples
- Any special configuration

## Compliance Checklist

- [ ] Name follows pattern: `fixture-<mode>-<category>-<name>-<variant>`
- [ ] Name registered in `config/taxonomy/fixture-catalog.yaml`
- [ ] Variant code is 3-digit zero-padded (`001`, `002`, etc.)
- [ ] No PII in test data
- [ ] No NPI/MNPI
- [ ] No non-public interface tooling (in public repos)
- [ ] Container-first: `docker-compose.yml` at root
- [ ] Scenario-driven configuration (YAML/JSON)
- [ ] Structured logging (JSON to stderr)
- [ ] Stateless by default
- [ ] README with usage examples
- [ ] INTEGRATION.md with required template sections
- [ ] Scenarios documented

## Roadmap

### v0.4.2 (Current)

- Modes: `server`, `client`, `datastore`
- Categories: `proving`, `utility`, `chaos`
- Fixture catalog registry
- i18n support for metadata

### v0.4.3 (Planned)

- Add `identity` mode for OIDC/SAML fixtures
- Alias resolver implementation

### Future

- CI validation: repo name matches catalog entry
- Scenario schema standardization (after patterns emerge)
- Fixture composition (multi-fixture test environments)

## Related Documentation

- [Repository Categories Taxonomy](../../config/taxonomy/repository-categories.yaml)
- [Fixture Catalog](../../config/taxonomy/fixture-catalog.yaml)
- [Repository Category Standards](../standards/repository-category/README.md)
- [Fulmen Ecosystem Guide](./fulmen-ecosystem-guide.md)
- [Fulmen Forge Workhorse Standard](./fulmen-forge-workhorse-standard.md) - Layer 2 peer
- [Fulmen Forge Microtool Standard](./fulmen-forge-microtool-standard.md) - Similar constraints model

---

**Document Status**: Draft
**Last Updated**: 2026-01-06
**Maintained By**: Schema Cartographer
**Approval Required From**: EA Steward, Crucible Maintainers
