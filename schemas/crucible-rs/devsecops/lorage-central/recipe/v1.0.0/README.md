---
title: "L'Orage Central Recipe Schema"
description: "Schema for declarative infrastructure deployment and seeding recipes with phase-based orchestration"
author: "Schema Cartographer"
date: "2025-11-10"
last_updated: "2025-11-10"
status: "experimental"
tags:
  [
    "schema",
    "lorage-central",
    "recipe",
    "devsecops",
    "v1.0.0",
    "experimental",
    "iac",
    "deployment",
  ]
---

# L'Orage Central Recipe Schema

⚠️ **EXPERIMENTAL STATUS**: This schema is under active development for L'Orage Central (v0.1.x → v0.2.0 cycle).
Breaking changes may occur without version bumps per [ADR-0011](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md).

**Do not depend on this schema in production applications** until experimental status is removed (target: L'Orage Central v0.2.0).

## Purpose

Defines declarative recipes for L'Orage Central infrastructure deployments and data seeding. Recipes are loaded via the gofulmen Crucible shim and executed in the REPL with schema-driven validation. Supports both declarative IaC patterns (components, secrets, validation) and procedural operations (bootstrap scripts, SDK calls) when needed.

## Key Features

### Recipe Types

- **deploy**: Infrastructure stack deployment (e.g., Mattermost on DO, OpenEdX on K3s)
- **seed**: Dataset population with immutable diff tracking for versioning

### Phase-Based Orchestration

- Components reference infra-phases taxonomy (bootstrap → secrets → network → storage → compute)
- Automatic DAG ordering based on phases and explicit `dependsOn` relationships
- Parallel execution support within phases

### Declarative Components

- Container/image specifications with ports and environment variables
- Secret references (decrypted via policy backend at runtime)
- Module support for Tofu/Pulumi integrations

### Procedural Actions (Optional)

- **script**: Execute bash/python for pain points (e.g., GPG key generation)
- **sdk-call**: Direct provider API calls for complex networking
- **bootstrap**: Infrastructure initialization steps

### Multi-Provider Support

- Target specification with provider (from infra-providers taxonomy)
- Region selection with geo/cloud validation
- Backend toolchain choice (OpenTofu, Terraform, Pulumi)

### Validation & Testing

- Post-deploy health checks (HTTP endpoints)
- Connection validation (database, API)
- Custom validation scripts

## Files

- `recipe.schema.json` – Recipe schema structure

## Usage

```bash
goneat schema validate-schema schemas/devsecops/lorage-central/recipe/v1.0.0/recipe.schema.json
```

## Schema Structure

### Deploy Recipe Example

```json
{
  "name": "mattermost-stack",
  "type": "deploy",
  "target": {
    "provider": "doc",
    "region": "nyc3",
    "backend": "opentofu"
  },
  "components": [
    {
      "name": "postgres",
      "image": "postgres:15",
      "phase": "storage",
      "ports": [5432],
      "env": {
        "POSTGRES_DB": "mattermost"
      },
      "secrets": [
        {
          "ref": "gpg://keyring/acme/db-pass",
          "injectAs": "POSTGRES_PASSWORD"
        }
      ]
    },
    {
      "name": "mattermost",
      "image": "mattermost/mattermost-team-edition:latest",
      "phase": "compute",
      "ports": [8065],
      "dependsOn": ["postgres"],
      "env": {
        "MM_SQLSETTINGS_DRIVERNAME": "postgres",
        "MM_SQLSETTINGS_DATASOURCE": "postgres://postgres@postgres:5432/mattermost"
      }
    }
  ],
  "validate": [
    {
      "type": "health",
      "endpoint": "http://localhost:8065/api/v4/system/ping"
    }
  ]
}
```

### Seed Recipe with Diff Example

```json
{
  "name": "acme-tenant-seed",
  "type": "seed",
  "target": {
    "provider": "doc",
    "region": "nyc3"
  },
  "components": [
    {
      "name": "turso-import",
      "image": "turso-cli:latest",
      "phase": "storage"
    }
  ],
  "diff": {
    "from": "v1-0",
    "to": "v1-1",
    "changes": [
      {
        "op": "add",
        "path": "/tenants/acme-prod-us",
        "value": {
          "publicId": "tnt-uuid-123-prod-us",
          "purpose": "production-api"
        }
      }
    ]
  }
}
```

## Integration with L'Orage Central

- **REPL Execution**: Recipes are validated and executed in the L'Orage Central REPL
- **Phase Ordering**: Topological sort based on infra-phases taxonomy and dependsOn relationships
- **Secret Management**: Secrets decrypted at runtime via policy-defined backend
- **Multi-Tenant**: Recipe instances can be scoped to specific tenants
- **Validation**: Pre-deploy schema validation and post-deploy health checks

## Dependencies

This schema references:

- [Infra Phases Taxonomy](../../../../taxonomy/devsecops/infra-phases/v1.0.0/README.md) – Phase identifiers and ordering
- [Infra Providers Taxonomy](../../../../taxonomy/devsecops/infra-providers/v1.0.0/README.md) – Provider and region validation
- [Policy Schema](../policy/v1.0.0/README.md) – Secret backend configuration
- [Credentials Schema](../credentials/v1.0.0/README.md) – Credential reference patterns

## Recipe Execution Flow

1. **Load & Validate**: REPL loads recipe and validates against schema (slugs, phases, refs)
2. **Unlock Secrets**: Policy backend decrypts secret references
3. **Build DAG**: Topological sort of components/actions by phase and dependsOn
4. **Execute**: Shim to Tofu/SDK for each component/action in order
5. **Validate**: Run post-deploy checks (health endpoints, connection tests)
6. **Audit**: Log execution to activity schema

## Future Enhancements

Planned for post-experimental:

- **migrate**: Diff-based upgrade recipes with rollback support
- **$ref validation**: Cross-validate provider regions against infra-providers metadata
- **Conditional execution**: Branching based on geo/cloud/tenant context
- **Metrics collection**: Resource usage and performance tracking

## References

- [ADR-0011: Temporary SemVer Flexibility for L'Orage Central DevSecOps Schemas](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md)
- [L'Orage Central Feature Brief](.plans/fulmen-lorage-central/feature-brief.md)
- [Infra Phases Taxonomy](../../../../taxonomy/devsecops/infra-phases/v1.0.0/README.md)
- [Infra Providers Taxonomy](../../../../taxonomy/devsecops/infra-providers/v1.0.0/README.md)
