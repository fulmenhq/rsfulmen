---
title: "Fulmen API Standards"
description: "Cross-language HTTP and gRPC standards for Fulmen repositories"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["standards", "api", "http", "grpc", "2025.10.2"]
---

# Fulmen API Standards

This directory collects cross-language API standards for Fulmen services and workhorse applications. The initial
scope covers HTTP/REST and gRPC transports, aligned with the repository category taxonomy (`workhorse`,
`service`).

## Documents

- `http-rest-standards.md` – RESTful conventions, endpoint design, request/response schemas, error handling (schemas under `schemas/protocol/http/v1.0.0/`).
- `grpc-standards.md` – gRPC service design, protobuf style guide, interceptor requirements, health checks.

Future additions (GraphQL, WebSockets) will follow the same pattern.
