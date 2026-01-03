---
title: "External Tools Manifest v1.0.0"
description: "Schema for declaring external tool prerequisites in Fulmen repositories"
author: "Pipeline Architect"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "draft"
tags: ["tooling", "bootstrap", "schema"]
---

# External Tools Manifest

**Schema**: `external-tools-manifest.schema.yaml`  
**Purpose**: Define CLI tool prerequisites for repository bootstrap  
**Location**: `.goneat/tools.yaml`  
**Script**: `scripts/bootstrap-tools.ts`

## Overview

This schema validates manifests that declare external tooling dependencies for Fulmen repositories. Tools are installed via `make bootstrap` and verified via `make tools`.

## Install Types

### `go` - Go Installable Tools

Install tools via `go install`:

```yaml
tools:
  - id: goneat
    description: Fulmen DX CLI
    required: true
    install:
      type: go
      module: github.com/fulmenhq/goneat
      version: latest
      binName: goneat
      destination: ./bin
```

**Fields**:

- `module`: Go module path (required)
- `version`: Version tag or `latest` (required)
- `binName`: Binary name (defaults to tool id)
- `destination`: Install directory (defaults to manifest `binDir`)

### `verify` - System-Installed Tools

Verify tools already on PATH:

```yaml
tools:
  - id: curl
    description: HTTP client
    required: true
    install:
      type: verify
      command: curl
```

**Fields**:

- `command`: Command to check (defaults to tool id)

### `download` - Binary Downloads

Download and install prebuilt binaries:

```yaml
tools:
  - id: shfmt
    description: Shell formatter
    required: true
    install:
      type: download
      url: https://github.com/mvdan/sh/releases/download/v3.8.0/shfmt_v3.8.0_darwin_amd64
      binName: shfmt
      destination: ./bin
      checksum: c0218b47a0301bb006f49fad85d2c08de23df303472834faf5639d04121320f8
```

**Fields**:

- `url`: Direct download URL (required)
- `binName`: Binary name (defaults to tool id)
- `destination`: Install directory (defaults to manifest `binDir`)
- `checksum`: SHA-256 hex checksum (strongly recommended for security)

**Security Note**: Always provide `checksum` for downloaded binaries to verify integrity and prevent tampering.

## Usage

### Bootstrap Tools

```bash
make bootstrap          # Install all required tools
make tools              # Verify tools are available
```

### Manual Invocation

```bash
bun run scripts/bootstrap-tools.ts --install
bun run scripts/bootstrap-tools.ts --verify
bun run scripts/bootstrap-tools.ts --install --force  # Reinstall even if present
```

## Example Manifest

```yaml
version: v1.0.0
binDir: ./bin

tools:
  # Go tool
  - id: goneat
    description: Fulmen DX CLI for formatting, linting, security checks
    required: true
    install:
      type: go
      module: github.com/fulmenhq/goneat
      version: latest
      binName: goneat
      destination: ./bin

  # Downloaded binary
  - id: shfmt
    description: Shell script formatter
    required: true
    install:
      type: download
      url: https://github.com/mvdan/sh/releases/download/v3.8.0/shfmt_v3.8.0_darwin_amd64
      checksum: c0218b47a0301bb006f49fad85d2c08de23df303472834faf5639d04121320f8
      destination: ./bin

  # System tool
  - id: curl
    description: HTTP client used for fetching remote assets
    required: true
    install:
      type: verify
      command: curl
```

## Relationship to Goneat Tools

**Different Use Cases**:

| Feature        | External Tools Manifest               | Goneat Tools Config                         |
| -------------- | ------------------------------------- | ------------------------------------------- |
| **Purpose**    | Repo-specific bootstrap               | Ecosystem-wide catalog                      |
| **Schema**     | `external-tools-manifest.schema.yaml` | `goneat-tools-config.schema.yaml`           |
| **Location**   | `.goneat/tools.yaml`                  | Goneat internal/custom catalogs             |
| **Complexity** | Minimal (3 install types)             | Full-featured (scopes, versions, platforms) |
| **Command**    | `make bootstrap`                      | `goneat doctor`                             |

See [Goneat Tools Config](../../goneat-tools/v1.0.0/) for the full-featured tool catalog schema.

## References

- [Makefile Standard](../../../../docs/standards/makefile-standard.md)
- [Goneat Tools Config](../../goneat-tools/v1.0.0/)
- [Repository Safety Protocols](../../../../REPOSITORY_SAFETY_PROTOCOLS.md)
