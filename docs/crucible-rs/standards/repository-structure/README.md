---
title: "Cross-Language Repository Structure Patterns"
description: "Canonical taxonomy for Fulmen application templates and cross-language baseline requirements"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["standards", "repository", "structure", "taxonomy", "2025.10.2"]
---

# Cross-Language Repository Structure Patterns

This standard defines the canonical taxonomy for Fulmen application and template structures and documents
shared requirements that apply before language-specific standards add further guidance. Language variants
(`docs/standards/repository-structure/<language>/…`) MUST reference this document in their prerequisites
section. For the authoritative language foundation list see
`docs/architecture/library-ecosystem.md#language-foundations-taxonomy`; machine-readable category metadata
is stored in `config/taxonomy/repository-categories.yaml`.

## Repository Category Taxonomy

| Category    | Summary                                                                 | Primary Entry Points                                           | Typical Consumers                    |
| ----------- | ----------------------------------------------------------------------- | -------------------------------------------------------------- | ------------------------------------ |
| `cli`       | Command-line utilities with no resident server component                | Executables exposed via Cobra/Click/Fastify CLI                | Developer tools, build pipeline jobs |
| `workhorse` | Hybrid CLI + HTTP (and optional gRPC) runtime with shared core logic    | CLI subcommand `serve`, HTTP `/health/*` endpoints             | Automation services, MCP servers     |
| `service`   | Long-running HTTP/gRPC service without CLI UX beyond lifecycle commands | Systemd/service host, Kubernetes deployment                    | Backend microservices, internal APIs |
| `library`   | Importable modules or packages with no executable                       | Language runtime package manager                               | Shared helper libraries, SDKs        |
| `pipeline`  | Batch/stream processing workloads orchestrated via schedulers           | CLI driver plus job orchestration entry points                 | Data engineering, ETL jobs           |
| `codex`     | Documentation or static site repositories published via CI              | Static site generators, Markdown tooling                       | Docs portals, standards handbooks    |
| `sdk`       | Client SDKs targeting external services or APIs                         | Package manager entry points, language-specific init scaffolds | External developers, partner teams   |

Categories are mutually exclusive for automation purposes—projects MAY layer features (e.g., a workhorse
exports an SDK package), but the repository-level designation drives the minimum requirements captured
below.

## Baseline Requirements (All Categories)

- `VERSION` file aligned to the repository’s CalVer or SemVer strategy (see `docs/standards/repository-versioning.md`).
- Structured logging per `docs/standards/observability/logging.md` with RFC3339 timestamps and STDERR routing.
- Three-layer configuration model (embedded defaults → user config → environment variables/CLI overrides).
- Makefile compliance (`make bootstrap`, `make lint`, `make test`, `make release:check`).
- Test coverage expectations published in the module manifest once language-specific tooling is ready (interim target: ≥80% statement coverage on public API).
- README/CONTRIBUTING entries linking to the category definition in this document.

## Category Requirements

### CLI (`cli`)

- Required flags: `--help/-h`, `--version/-v`, `--config`, `--log-level`, `--log-format`, and `--output` when the tool renders data.
- Exit codes: `0` success, `1` general failure, `2` usage error, `3` configuration error. Reserve `> 128` for signal exits.
- Output hygiene: data on STDOUT, diagnostics on STDERR; JSON output must honor the schema published for the command when available.
- Subcommands MUST include `help` and `version`; additional subcommands follow language standards (e.g., Cobra command factories, Click groups).

### Workhorse (`workhorse`)

- Inherits all CLI requirements.
- Mandatory HTTP endpoints: `GET /health/live`, `GET /health/ready`, `GET /health/startup`, `GET /version`, `GET /metrics` (Prometheus format).
- CLI `serve` command MUST support `--host`, `--port`, `--workers`, `--reload` (dev only), and telemetry toggles.
- Response schemas: success envelope `{success:boolean,message:string,data?:object}`; error envelope `{error:string,code:string,details?:object}` with RFC 7807 compatibility.
- Request/trace correlation: accept `X-Request-ID` and `X-Correlation-ID`; generate UUIDv7 identifiers when absent and propagate via logs/traces.
- gRPC interfaces are OPTIONAL by default; when implemented, they MUST expose health reflection and share core logic with HTTP handlers.
- HTTP endpoints MUST follow the [HTTP REST Standard](../protocol/http-rest-standards.md); optional gRPC services follow the [gRPC Standard](../protocol/grpc-standards.md).

### Service (`service`)

- Inherits workhorse server expectations minus interactive CLI subcommands beyond lifecycle helpers.
- Graceful shutdown: handle SIGTERM/SIGINT with configurable drain periods.
- Readiness probes MUST validate dependent services (databases, queues) and degrade gracefully when `critical: false` dependencies fail.
- Authentication/authorization strategies MUST be documented (e.g., mTLS for gRPC, JWT/OIDC for HTTP). Future security standard will elaborate.
- MUST comply with both [HTTP REST Standard](../protocol/http-rest-standards.md) and [gRPC Standard](../protocol/grpc-standards.md) when exposing respective transports.

### Library (`library`)

- Public API MUST follow semantic versioning rules even when the repository uses CalVer packaging.
- Provide generated or hand-authored API documentation (GoDoc, Sphinx/ReadTheDocs, TypeDoc) covering exported types and functions.
- Ship type metadata (`py.typed`, `.d.ts`, or Go interfaces) and include examples demonstrating core modules listed in the manifest.
- No side effects during import beyond idempotent registration.

### Pipeline (`pipeline`)

- CLI driver MUST expose scheduling hooks (cron expression or orchestrator integration flags).
- Metrics MUST cover job duration, throughput, and failure counts.
- Configuration MUST support dry-run/simulation modes.
- Document restart semantics and idempotency expectations for each stage.

### Codex (`codex`)

- Built output (static site) MUST derive from assets under `docs/` or `content/`; no generated files committed unless documented.
- Define publishing flow (e.g., GitHub Pages, S3) and link to `docs/sop/cicd-operations.md` for CI integration.
- Provide navigation metadata so downstream tooling can ingest the site (e.g., sidebar JSON, mkdocs nav).

### SDK (`sdk`)

- Follow `docs/standards/agentic-attribution.md` for generated code markers when scaffolding clients.
- Include compatibility matrix mapping SDK versions to API versions.
- Provide error taxonomy mapping transport-layer faults to language-specific exceptions or error types.
- Document authentication helpers (API keys, OAuth flows) and rate-limit retry strategies.

## Language Variant Integration

Language-specific repository structure documents MUST:

1. Reference this taxonomy in a **Prerequisites** or **Before You Begin** section.
2. Enumerate how each requirement is implemented in the target language (e.g., Click decorators for CLI flags, FastAPI routers for health endpoints).
3. Note any language deviations or pending work (e.g., gRPC planned for tsfulmen Q4).
4. For TypeScript workhorse repositories, follow [`typescript/workhorse.md`](typescript/workhorse.md) (Fastify + optional gRPC).

## Future Additions

- Security profile per category (JWT, API keys, mTLS) will be formalized in an upcoming security standard.
- Additional categories (e.g., `frontend`, `worker`) will be appended when real repositories warrant them; additions must be approved by the architecture team.
- Schema assets for health endpoints, error envelopes, and configuration blocks are tracked in `.plans/active/2025.10.2/` and will be linked here once published.

## References

- `docs/standards/coding/README.md` – Cross-language coding expectations.
- `docs/standards/observability/logging.md` – Structured logging, correlation IDs.
- `docs/standards/config/fulmen-config-paths.md` – Directory discovery and config layering.
- `docs/architecture/fulmen-helper-library-standard.md` – Library capability catalog.
- `.plans/active/2025.10.2/repository-structure-cross-language-patterns.md` – Original feature brief driving this standard.
