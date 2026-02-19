---
title: "Fulmen Fixture Standard"
description: "Standard structure and requirements for Fulmen Fixture repositories - test infrastructure providing real-but-test-purpose servers, clients, and datastores"
author: "Schema Cartographer (@schema-cartographer)"
date: "2026-01-06"
last_updated: "2026-01-12"
status: "active"
tags: ["architecture", "fixture", "testing", "infrastructure", "v0.4.6"]
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
fixture-<mode>-<category>-<name>
```

All components are required.

### Components

| Component    | Constraint              | Description                                                   |
| ------------ | ----------------------- | ------------------------------------------------------------- |
| `fixture`    | Literal                 | Repository type prefix                                        |
| `<mode>`     | Enum                    | `server`, `client`, `datastore`, `identity`                   |
| `<category>` | Enum                    | `proving`, `utility`, `chaos`                                 |
| `<name>`     | `^[a-z][a-z0-9]{1,20}$` | Registered name from fixture catalog (no hyphens/underscores) |

**Full regex**: `^fixture-(server|client|datastore|identity)-(proving|utility|chaos)-[a-z][a-z0-9]{1,20}$`

### Variant Strategy

Variants (different configurations of the same fixture concept) are handled via:

1. **Docker tags** for versioned variants: `fixture-server-proving-gauntlet:oauth-minimal`
2. **Scenario files** for runtime configuration: `scenarios/oauth-full.yaml`
3. **Future semantic variants** may use descriptive suffixes if needed (e.g., `fixture-server-proving-gauntlet-extended`)

This approach simplifies naming while preserving flexibility. Most fixtures need only one repository with multiple scenarios.

### Modes

| Mode        | Purpose                            | Examples                          |
| ----------- | ---------------------------------- | --------------------------------- |
| `server`    | Backend APIs (REST, gRPC, GraphQL) | Protected backend, echo server    |
| `client`    | Clients for server/API testing     | OAuth tester, load generator      |
| `datastore` | Databases, caches, message queues  | S3-compatible storage, Redis mock |
| `identity`  | IdP/authentication (OIDC, SAML)    | OIDC provider with test users     |

**Note**: `identity` mode is planned for v0.4.3.

### Binary Naming

The binary/executable name uses only the `<name>` component, without mode or category:

| Repository Name                  | Binary Name | Rationale                                                               |
| -------------------------------- | ----------- | ----------------------------------------------------------------------- |
| `fixture-server-proving-rampart` | `rampart`   | Short, clean for `docker exec` and local use                            |
| `fixture-server-utility-echo`    | `echo`      | Name only (conflicts with shell `echo` acceptable in container context) |

**Identity information** appears in:

- Repository/image name: `ghcr.io/fulmenhq/fixture-server-proving-rampart`
- App identity: `.fulmen/app.yaml`
- Version output: `rampart v1.0.0`
- `/version` endpoint response

### Examples

| Full Name                          | Mode      | Category | Name     | Description                   |
| ---------------------------------- | --------- | -------- | -------- | ----------------------------- |
| `fixture-server-proving-gauntlet`  | server    | proving  | gauntlet | Protected backend, OAuth2     |
| `fixture-server-proving-rampart`   | server    | proving  | rampart  | HTTP protocol testing server  |
| `fixture-server-chaos-gremlin`     | server    | chaos    | gremlin  | Failure injection server      |
| `fixture-server-utility-echo`      | server    | utility  | echo     | Simple request echo           |
| `fixture-client-proving-probe`     | client    | proving  | probe    | OAuth client tester           |
| `fixture-datastore-utility-bucket` | datastore | utility  | bucket   | S3-compatible minimal storage |
| `fixture-identity-proving-warden`  | identity  | proving  | warden   | Full OIDC with test users     |

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

**Important**: The fixture catalog serves as the authoritative registry for fixture names. Create the catalog entry BEFORE creating the repository.

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
    scenarios:
      - name: "oauth-minimal"
        description: "OAuth2 with basic scopes"
      - name: "oauth-full"
        description: "OAuth2 with PKCE, refresh, introspection"
```

Scenarios document the different configurations available within a single fixture repository.

### i18n Support

| Field          | Constraint            | Notes                                         |
| -------------- | --------------------- | --------------------------------------------- |
| `default_lang` | Required, ISO 639-1   | Author's primary language                     |
| `summary`      | Required, UTF-8       | In `default_lang`                             |
| `summary_i18n` | Optional, map         | lang code -> string (excludes `default_lang`) |
| `description`  | Optional, UTF-8       | In `default_lang`                             |
| `aliases`      | Optional, UTF-8 array | i18n nicknames, typo corrections              |

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

## CLI Requirements

Fixtures MUST implement a minimal CLI surface for operational diagnostics. This is mandatory, not optional.

### Required Commands

#### Default Behavior (No Arguments)

```bash
rampart        # Starts server
rampart serve  # Optional explicit alias
```

Fixtures MUST start serving when invoked without arguments. This maintains Docker compatibility where `ENTRYPOINT ["/rampart"]` should just work.

#### Version Command

```bash
rampart version           # Standard output
rampart version extended  # Extended build info
```

**Standard output**:

```
rampart v1.0.0
```

**Extended output**:

```
rampart v1.0.0
  Commit:     abc1234
  Built:      2026-01-08T12:00:00Z
  Go:         go1.21.5
  Platform:   linux/amd64
  Crucible:   0.4.4

Fulmen - Thrive on Scale
https://3leaps.net
```

**Requirements**:

- MUST include version string
- MUST include Crucible version (for SSOT alignment debugging)
- SHOULD include commit SHA and build date
- Extended version SHOULD include ecosystem branding
- Exit code: 0 on success

#### Doctor Command

```bash
rampart doctor
```

**Purpose**: Pre-flight check before serving. Answers "can I run?"

**Standard output** (ready):

```
rampart doctor

Configuration
  Port:       8080 (from PORT env, default: 8080)
  Host:       127.0.0.1 (from HOST env, default: 127.0.0.1)
  Log Level:  info (from LOG_LEVEL env, default: info)
  Scenario:   default (from SCENARIO env, default: default)

Port Check
  ✓ Port 8080 is available

Environment
  ✓ Running as non-root (uid=1000)
  ✓ Writable stderr for logging

Ready to serve.
```

**Failure output** (port in use):

```
rampart doctor

Configuration
  Port:       8080 (from PORT env, default: 8080)
  ...

Port Check
  ✗ Port 8080 is in use
    PID 12345: /usr/bin/rampart (if detectable)

Not ready. Fix issues above before starting.
```

**Requirements**:

- MUST check if configured port is available
- MUST show resolved configuration with sources (env var vs default)
- SHOULD detect what process holds port (platform-dependent, best effort)
- SHOULD check basic runtime requirements (stderr writable, etc.)
- Exit code: 0 if ready, non-zero if issues found
- Output MUST be human-readable (not JSON) - this is operator-facing

### Exit Codes

| Command           | Success      | Failure                |
| ----------------- | ------------ | ---------------------- |
| `version`         | 0            | 1 (shouldn't fail)     |
| `doctor`          | 0 (ready)    | 1 (not ready)          |
| `serve` / default | 0 (shutdown) | See foundry exit codes |
| Port in use       | -            | 10 (foundry standard)  |

### Implementation Notes

Fixtures should NOT require a CLI framework (cobra, urfave/cli). Simple `os.Args` parsing is sufficient:

```go
func main() {
    if len(os.Args) > 1 {
        switch os.Args[1] {
        case "version":
            printVersion(len(os.Args) > 2 && os.Args[2] == "extended")
            return
        case "doctor":
            os.Exit(runDoctor())
        case "serve":
            // fall through to serve
        default:
            fmt.Fprintf(os.Stderr, "unknown command: %s\n", os.Args[1])
            os.Exit(1)
        }
    }
    // default: serve
    serve()
}
```

## OpenAPI Publication

Fixtures that expose HTTP endpoints MUST publish an OpenAPI specification to enable client generation, documentation, and contract testing.

### Requirements

| Requirement   | Level | Details                                       |
| ------------- | ----- | --------------------------------------------- |
| Generation    | MUST  | `make openapi` produces `dist/openapi.yaml`   |
| Serving       | MUST  | `GET /openapi.yaml` returns the spec          |
| Coverage test | MUST  | CI verifies spec covers all registered routes |
| CI workflow   | MUST  | `make openapi` runs before `make test`        |

### Build Artifact

Fixtures MAY use either of these equivalent patterns:

| Pattern        | Spec Location                    | Serving Mechanism               | Example  |
| -------------- | -------------------------------- | ------------------------------- | -------- |
| Build artifact | `dist/openapi.yaml`              | Read from filesystem at runtime | gauntlet |
| Embedded       | `internal/handlers/openapi.yaml` | `//go:embed` into binary        | rampart  |

Both patterns are acceptable as long as:

- `GET /openapi.yaml` returns a deterministic, complete spec
- The coverage test validates the published contract
- `make openapi` generates/updates the spec; `make build` SHOULD depend on `openapi`

### Coverage Testing

Fixtures MUST include an automated coverage test that compares registered routes against the OpenAPI spec. See [ADR-0014: OpenAPI Spec Coverage Tests](decisions/ADR-0014-openapi-spec-coverage.md) for rationale and implementation patterns.

The coverage test:

1. Parses registered routes from the router source-of-truth
2. Parses `paths` + methods from the generated spec
3. Fails if any route is missing from the spec (unless explicitly excluded)

**Intentional exclusions**: Maintainers MAY exclude experimental, internal, or self-referential endpoints (e.g., `/openapi.yaml` itself). All "going concern" endpoints intended for consumer use SHOULD be documented.

### Release Assets

Repository maintainers SHOULD decide whether the OpenAPI spec belongs in release assets (`dist/release/`). If included:

- Copy `dist/openapi.yaml` to `dist/release/openapi.yaml` during `make release-prepare`
- The spec becomes a signed release artifact alongside binaries
- Document availability in release notes

### Provenance Metadata

OpenAPI specs SHOULD include provenance using the standard `info.x-*` extension mechanism (does NOT break OpenAPI tooling):

```yaml
openapi: "3.1.0"
info:
  title: "Rampart HTTP Testing Fixture"
  version: "1.0.0"
  x-generated-by: "swaggo/swag"
  x-generated-at: "2026-01-12T10:00:00Z"
  x-source-repo: "github.com/fulmenhq/fixture-server-proving-rampart"
  x-crucible-version: "0.4.6"
```

Do NOT add YAML frontmatter or comments before the `openapi:` key—this breaks spec parsers.

### Related Documentation

- [ADR-0014: OpenAPI Spec Coverage Tests](decisions/ADR-0014-openapi-spec-coverage.md) - Decision rationale
- [HTTP Server Patterns Guide](../guides/testing/http-server-patterns.md) - Implementation patterns
- [HTTP REST Standard](../standards/protocol/http-rest-standards.md) - HTTP API conventions

## Port Management Tiers

Fixtures operate at different complexity levels for port management. Choose the tier appropriate for your fixture.

### Tier 1: Simple (Recommended)

Single service with environment variable configuration.

| Variable | Purpose      | Default     |
| -------- | ------------ | ----------- |
| `PORT`   | Listen port  | `8080`      |
| `HOST`   | Bind address | `127.0.0.1` |

**Example**:

```bash
PORT=9090 HOST=0.0.0.0 rampart
```

**Use when**: Single HTTP endpoint, local development, simple CI.

**Reference implementation**: `fixture-server-proving-rampart`

### Tier 2: Multi-Port

Services exposing multiple ports (e.g., HTTP + metrics, HTTP + gRPC).

| Variable       | Purpose          | Default     |
| -------------- | ---------------- | ----------- |
| `PORT`         | Primary port     | `8080`      |
| `METRICS_PORT` | Metrics endpoint | `9090`      |
| `GRPC_PORT`    | gRPC endpoint    | `50051`     |
| `HOST`         | Bind address     | `127.0.0.1` |

**Example**:

```bash
PORT=8080 METRICS_PORT=9091 GRPC_PORT=50052 gauntlet
```

**Use when**: Multiple protocols, metrics exposure, health separated from main API.

### Tier 3: Port Ranges (Future)

Reserved for orchestrated multi-fixture environments where port allocation is managed centrally.

| Variable          | Purpose                   | Example |
| ----------------- | ------------------------- | ------- |
| `PORT_RANGE_BASE` | Start of allocated range  | `8000`  |
| `PORT_RANGE_SIZE` | Number of ports allocated | `10`    |

This tier is **planned** for fixture composition scenarios where a controller allocates port ranges to avoid conflicts.

### Host Binding Default

**Default: `127.0.0.1`** (localhost only)

**Rationale**:

- Prevents accidental network exposure during local development
- Safe default for `make run` scenarios
- Container deployments explicitly set `HOST=0.0.0.0`

**Docker Compose files SHOULD include**:

```yaml
environment:
  - HOST=0.0.0.0 # Allow external access in container
```

### Port Conflict Detection

Fixtures MUST detect port conflicts before binding:

1. **On startup**: Check if port is available before attempting to bind
2. **Exit code**: Use exit code 10 (foundry standard) for port-in-use
3. **Error message**: Include port number and (if detectable) conflicting process

```
Error: port 8080 is already in use
  Process: rampart (PID 12345)
  Hint: Stop the existing process or use PORT=<other> to change port
```

## Directory Structure

```
fixture-server-proving-gauntlet/
├── .fulmen/
│   └── app.yaml                    # App Identity manifest (if using helper lib)
├── scenarios/
│   ├── default.yaml                # Default scenario
│   ├── oauth-minimal.yaml          # Scenario configurations
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

| Concern          | Where It Lives  | Example                           |
| ---------------- | --------------- | --------------------------------- |
| Fixture identity | Repo/image name | `fixture-server-proving-gauntlet` |
| Fixture version  | Docker tag      | `v1.2.3`, `latest`, `sha-abc123`  |

```bash
# Full image reference
docker pull ghcr.io/fulmenhq/fixture-server-proving-gauntlet:v1.2.3
docker pull ghcr.io/fulmenhq/fixture-server-proving-gauntlet:latest
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

### Naming & Registration

- [ ] Name follows pattern: `fixture-<mode>-<category>-<name>`
- [ ] Name registered in `config/taxonomy/fixture-catalog.yaml`

### Security

- [ ] No PII in test data
- [ ] No NPI/MNPI
- [ ] No non-public interface tooling (in public repos)

### Infrastructure

- [ ] Container-first: `docker-compose.yml` at root
- [ ] Scenario-driven configuration (YAML/JSON)
- [ ] Structured logging (JSON to stderr)
- [ ] Stateless by default

### CLI Commands (Mandatory)

- [ ] `version` command implemented
- [ ] `doctor` command implemented
- [ ] Default behavior (no args) starts server

### Documentation

- [ ] README with usage examples
- [ ] INTEGRATION.md with required template sections
- [ ] Scenarios documented

## Roadmap

### v0.4.4 (Current)

- Simplified naming (dropped variant suffix)
- CLI requirements (`version`, `doctor` commands)
- Port management tiers
- Host binding default (127.0.0.1)

### v0.4.3

- Modes: `server`, `client`, `datastore`
- Categories: `proving`, `utility`, `chaos`
- Fixture catalog registry
- i18n support for metadata

### Future

- Add `identity` mode for OIDC/SAML fixtures
- Alias resolver implementation
- CI validation: repo name matches catalog entry
- Scenario schema standardization (after patterns emerge)
- Fixture composition (multi-fixture test environments)

## Related Documentation

- [Repository Categories Taxonomy](../../config/taxonomy/repository-categories.yaml)
- [Fixture Catalog](../../config/taxonomy/fixture-catalog.yaml)
- [Ecosystem Brand Summary](../../config/branding/ecosystem.yaml) - For `version --extended` or `about` command
- [Repository Category Standards](../standards/repository-category/README.md)
- [Fulmen Ecosystem Guide](./fulmen-ecosystem-guide.md)
- [Fulmen Forge Workhorse Standard](./fulmen-forge-workhorse-standard.md) - Layer 2 peer
- [Fulmen Forge Microtool Standard](./fulmen-forge-microtool-standard.md) - Similar constraints model
- [HTTP Server Patterns Guide](../guides/testing/http-server-patterns.md) - Implementation patterns and compliance checklists for fixture authors
- [HTTP Client Patterns Guide](../guides/testing/http-client-patterns.md) - Testing patterns for fixture consumers

---

**Document Status**: Active
**Last Updated**: 2026-01-08
**Maintained By**: Schema Cartographer
**Approval Required From**: EA Steward, Crucible Maintainers
