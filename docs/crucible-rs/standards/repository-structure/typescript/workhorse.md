---
title: "TypeScript Workhorse Application Structure"
description: "Repository structure and patterns for TypeScript workhorse (CLI + HTTP/gRPC) applications"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags:
  ["standards", "repository-structure", "typescript", "workhorse", "fastify"]
---

# TypeScript Workhorse Application Structure

## Overview

This guide defines the TypeScript implementation of the Fulmen workhorse pattern (CLI + HTTP server with optional
gRPC). Read the cross-language taxonomy first:

1. [Cross-Language Repository Structure Patterns](../README.md)
2. [TypeScript Repository Structure Standards](README.md)

The workhorse category inherits all CLI requirements and adds HTTP/gRPC runtime expectations.

## Directory Layout

```
project/
├── src/
│   ├── cli/                 # CLI entry points (Commander or custom Bun CLI)
│   ├── server/
│   │   ├── http.ts          # Fastify server bootstrap
│   │   ├── routes/          # Route handlers and schemas
│   │   └── grpc.ts          # Optional gRPC bootstrap (@grpc/grpc-js)
│   ├── core/                # Shared business logic (transport-agnostic)
│   ├── config/              # Config loading (three-layer helpers)
│   └── observability/       # Logging/metrics/tracing setup
├── proto/                   # gRPC service definitions (compiled to src/gen)
├── src/gen/                 # Generated gRPC types (excluded from lint formatting)
├── test/                    # Bun/Vitest tests
├── examples/
├── bunfig.toml
├── biome.json
├── tsconfig.json
└── Makefile
```

## HTTP Server (Fastify)

- Use **Fastify** (`fastify@^5`) with `@fastify/ajv-compiler` for schema validation.
- Register routes under../protocol/v1/\*`and health endpoints at`/health/live`, `/health/ready`, `/health/startup`.
- Expose `/metrics` using `@fastify/metrics` (Prometheus format).
- Integrate logging via `pino` with serializers that emit Crucible-compliant log events.
- Example bootstrap (`s../protocol/http.ts`):

```ts
import fastify from "fastify";
import { registerRoutes } from "./routes/index.js";
import { buildLogger } from "../observability/logger.js";

export async function createServer() {
  const app = fastify({ logger: buildLogger() });
  await registerRoutes(app);
  return app;
}
```

## gRPC (Optional)

- Use `@grpc/grpc-js` + `@grpc/proto-loader` for runtime; generate TypeScript types with `ts-proto` or `protobufjs`.
- Implement services in `s../protocol/grpc.ts`, sharing core logic with HTTP handlers.
- Expose standard health service (`grpc.health.v1.Health`) alongside custom services.
- Propagate correlation IDs via metadata keys `x-request-id` and `x-correlation-id`.

## CLI Entry Point

- Provide `bun run cli serve` (or `make serve`) that boots Fastify and optional gRPC server.
- CLI should reuse core logic for non-server commands (e.g., `validate`, `generate`).
- Use Commander or lightweight Bun CLI wrappers; ensure CLI flags align with cross-language taxonomy (host, port, workers, reload).

## Configuration

- Implement three-layer config using helpers from `@fulmenhq/crucible` once available, or local utilities following
  `docs/standards/library/modules/enterprise-three-layer-config.md`.
- Read defaults from embedded YAML, merge user overrides (XDG/Bun config dir), apply environment/CLI overrides.
- Validate merged config with AJV and Fulmen schemas.

## Observability

- Logging: Use `pino` with a custom transport that formats events per Crucible logging schema (include correlationId, requestId, etc.).
- Metrics: Expose Prometheus metrics via `/metrics` using `@fastify/metrics`.
- Tracing: Optional OpenTelemetry integration via `@opentelemetry/sdk-node`.

## Testing

- CLI: Use Bun test runner to execute command handlers.
- HTTP: Use `@fastify/request` or `light-my-request` to test routes without binding to a port.
- gRPC: Use `@grpc/grpc-js` in-process clients with ephemeral server instances.
- Maintain ≥85% coverage on exported APIs (per module manifest targets).

## Makefile Targets

- `make bootstrap` – install Bun deps + generated artifacts.
- `make serve` – start development server with auto-reload (`bun run dev`).
- `make lint`, `make test`, `make typecheck` – delegate to Bun/tsc/biome.
- `make generate` – regenerate gRPC stubs from `proto/` definitions.

## References

- `docs/standards/protocol/http-rest-standards.md`
- `docs/standards/protocol/grpc-standards.md`
- `docs/standards/library/README.md` (module spec index)
- `docs/standards/observability/logging.md`
