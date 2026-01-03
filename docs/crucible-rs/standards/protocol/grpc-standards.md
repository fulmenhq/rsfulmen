---
title: "gRPC Standard"
description: "Fulmen conventions for designing and implementing gRPC services"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["standards", "api", "grpc", "2025.10.2"]
---

# gRPC Standard

## Scope

Applies to Fulmen repositories exposing gRPC endpoints (workhorse optional feature, service category required when
declared). Covers protobuf structure, service design, interceptors, and operational requirements.

## Protobuf Guidelines

- Package names follow `fulmen.<area>.v1`.
- Use `proto3` syntax with explicit `go_package`, `java_package`, `csharp_namespace`, etc.
- Reserve field numbers for future extension; document deprecation in comments.
- Reuse shared messages from `schem../protocol/grpc/` when available (health, version, pagination).

## Service Design

- Prefer unary RPCs unless streaming necessary (document use cases for clie../protocol/bidi streams).
- Include standard health service implementing `grpc.health.v1.Health` alongside custom services.
- Expose version service (`GetVersion`) returning commit/ref for diagnostics.

## Interceptors / Middleware

- **Unary**: chain interceptors for request logging, correlation ID propagation, panic recovery, validation.
- **Streaming**: apply stream interceptors mirroring unary behaviors (especially logging + auth).
- Provide hooks for tracing (OpenTelemetry instrumentation) and metrics (`grpc_server_handled_total`).

## Error Handling

- Use canonical status codes (`INVALID_ARGUMENT`, `NOT_FOUND`, `FAILED_PRECONDITION`, `INTERNAL`).
- Attach structured error details via `google.rpc.Status` and `BadRequest` extras when needed.
- Propagate `requestId`/`correlationId` via metadata for parity with HTTP.

## Security

- Default to TLS. Mutual TLS recommended for internal services.
- Auth via metadata (`authorization` header) or mTLS identities; document expected peer cert attributes.

## Tooling

- Generate language bindings during `make build` (Go, Python, TypeScript via Proto compile). Store in `gen/`.
- Ensure generated code checked in when required by downstream build systems.
- Validate protobuf definitions using `buf lint` (future integration) and `goneat` for schema compliance where applicable.

## Related Documents

- `docs/standards/protocol/http-rest-standards.md`
- `docs/standards/observability/logging.md`
- `docs/architecture/fulmen-helper-library-standard.md`
