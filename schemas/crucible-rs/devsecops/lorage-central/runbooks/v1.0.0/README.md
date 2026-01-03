---
title: "L'Orage Central Runbook Schema"
description: "Schema for operational runbooks serializing Markdown procedures into structured, executable REPL workflows"
author: "Schema Cartographer"
date: "2025-11-10"
last_updated: "2025-11-10"
status: "experimental"
tags:
  [
    "schema",
    "lorage-central",
    "runbook",
    "devsecops",
    "v1.0.0",
    "experimental",
    "operations",
  ]
---

# L'Orage Central Runbook Schema

⚠️ **EXPERIMENTAL STATUS**: This schema is under active development for L'Orage Central (v0.1.x → v0.2.0 cycle).
Breaking changes may occur without version bumps per [ADR-0011](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md).

**Do not depend on this schema in production applications** until experimental status is removed (target: L'Orage Central v0.2.0).

## Purpose

Defines structured operational runbooks for L'Orage Central. Runbooks serialize Markdown-based procedures (from prototypes) into YAML/JSON for REPL execution. They document and automate operational procedures like tenant provisioning, global network setup, and infrastructure bootstrapping with phase-based organization and dependency tracking.

## Key Features

### Structured Procedure Execution

- **Phase-based organization**: Uses infra-phases taxonomy for ordered execution
- **Step types**: Text, tables, scripts, recipe actions, diagrams (ASCII/Mermaid)
- **Dependency tracking**: Steps can depend on prior steps (within or across phases)
- **Parallel execution**: Optional concurrent execution for independent steps

### Multi-Tenant Scoping

- Runbooks can target specific tenants or apply globally
- Tenant public IDs from tenant registry for scoping
- Supports multi-tenant operations (e.g., cross-region networking)

### Markdown Serialization

- Converts operational Markdown prototypes into structured JSON/YAML
- Preserves tables, code blocks, and diagrams as content
- Human-readable format for documentation and execution

### Integration with Recipes & Credentials

- **Action steps**: Reference recipe IDs for infrastructure operations
- **Credential refs**: Bootstrap steps can reference credential schemas
- **Validation**: Post-step checks using recipe validation patterns

### Audit Trail

- Configurable audit backend (local-json, postgres, loki)
- Retention policies for compliance
- Step execution logging via activity schema

## Files

- `runbook.schema.json` – Runbook schema structure

## Usage

```bash
goneat schema validate-schema schemas/devsecops/lorage-central/runbooks/v1.0.0/runbook.schema.json
```

## Schema Structure

### Global Network Runbook Example

```json
{
  "id": "global-network",
  "title": "Global Enterprise Network Setup",
  "tenantScope": ["tnt-fulmenhq-prod", "tnt-fulmenhq-dev"],
  "description": "Establishes zero-trust networking backbone across all tenants with VPC peering and Tailscale mesh.",
  "phases": [
    {
      "id": "network",
      "title": "Core Networking",
      "description": "Networking: Zero-trust backbone with VPC peering and Tailscale mesh",
      "steps": [
        {
          "id": "tenant-table",
          "type": "table",
          "content": "| Tenant | Type | Focus | Clouds |\n|--------|------|-------|--------|\n| fulmenhq | Ecosystem | Tonnerre | AWS/GCP |\n| 3leaps | Consulting | Édlair | DO/AWS |"
        },
        {
          "id": "vpc-create",
          "type": "script",
          "content": "doctl compute vpc create fulmen-global-vpc --region nyc3 --ip-range 10.0.0.0/16",
          "dependsOn": ["tenant-table"]
        },
        {
          "id": "vpc-peering",
          "type": "action",
          "ref": "vpc-peering-recipe",
          "dependsOn": ["vpc-create"],
          "validate": {
            "type": "connect",
            "endpoint": "http://10.0.1.1:8080/health"
          }
        }
      ]
    },
    {
      "id": "compute",
      "title": "Shared Services",
      "steps": [
        {
          "id": "mattermost-deploy",
          "type": "action",
          "ref": "mattermost-stack",
          "dependsOn": ["vpc-peering"]
        }
      ]
    }
  ],
  "audit": {
    "backend": "postgres",
    "retention": 730
  }
}
```

### Tenant Bootstrap Runbook Example

```json
{
  "id": "tenant-bootstrap",
  "title": "New Tenant Provisioning",
  "tenantScope": ["tnt-uuid-new-client-prod"],
  "description": "Bootstrap a new tenant with secrets, networking, and base services.",
  "phases": [
    {
      "id": "bootstrap",
      "title": "Foundation",
      "steps": [
        {
          "id": "gpg-keys",
          "type": "script",
          "content": "gpg --batch --gen-key /configs/gpg-keygen-params.txt"
        },
        {
          "id": "credentials-seed",
          "type": "action",
          "ref": "credentials-seed-recipe",
          "dependsOn": ["gpg-keys"]
        }
      ]
    },
    {
      "id": "network",
      "title": "Networking",
      "steps": [
        {
          "id": "vpc-setup",
          "type": "action",
          "ref": "tenant-vpc-recipe"
        }
      ]
    }
  ]
}
```

## Step Types

### text

Markdown content for documentation and instructions.

```json
{
  "id": "overview",
  "type": "text",
  "content": "## Networking Overview\n\nThis section establishes the zero-trust backbone..."
}
```

### table

Tabular data (e.g., tenant listings, configuration matrices).

```json
{
  "id": "tenant-list",
  "type": "table",
  "content": "| Tenant | Region | Provider |\n|--------|--------|----------|\n| acme-prod | us-east-1 | aws |"
}
```

### script

Executable bash/python scripts for procedural operations.

```json
{
  "id": "init-keys",
  "type": "script",
  "content": "gpg --gen-key --batch /tmp/gpg-params.txt"
}
```

### action

References to recipe IDs for infrastructure operations.

```json
{
  "id": "deploy-stack",
  "type": "action",
  "ref": "mattermost-stack"
}
```

### diagram

ASCII art or Mermaid diagrams for visualization.

````json
{
  "id": "network-diagram",
  "type": "diagram",
  "content": "```mermaid\ngraph TD\n  A[VPC] --> B[Subnet]\n```"
}
````

## Integration with L'Orage Central

- **REPL Execution**: Runbooks are loaded and executed step-by-step in the REPL
- **Phase Ordering**: Steps execute in phase order with dependency resolution
- **Recipe Integration**: Action steps invoke recipes for infrastructure operations
- **Tenant Scoping**: REPL validates tenant scope before execution
- **Audit Logging**: All step execution logged to activity schema

## Dependencies

This schema references:

- [Infra Phases Taxonomy](../../../../taxonomy/devsecops/infra-phases/v1.0.0/README.md) – Phase identifiers
- [Tenant Schema](../tenant/v1.0.0/README.md) – Tenant public ID validation
- [Recipe Schema](../recipe/v1.0.0/README.md) – Action step references and validation patterns
- [Activity Schema](../activity/v1.0.0/README.md) – Audit logging

## Runbook Execution Flow

1. **Load**: REPL loads runbook and validates against schema
2. **Scope Check**: Verify tenant scope matches execution context
3. **Build DAG**: Topological sort of steps by phase and dependsOn
4. **Execute Steps**:
   - **text/table/diagram**: Display to user
   - **script**: Execute in REPL shell context
   - **action**: Invoke referenced recipe
5. **Validate**: Run post-step checks if defined
6. **Audit**: Log step execution to activity backend

## Future Enhancements

Planned for post-experimental:

- **Conditional steps**: Branch execution based on geo/cloud/tenant context
- **Step rollback**: Automatic or manual rollback for failed steps
- **Mermaid rendering**: Visual diagram generation in REPL
- **Interactive mode**: Prompt for user confirmation at key steps
- **Metrics collection**: Step execution time and resource usage

## References

- [ADR-0011: Temporary SemVer Flexibility for L'Orage Central DevSecOps Schemas](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md)
- [L'Orage Central Feature Brief](.plans/fulmen-lorage-central/feature-brief.md)
- [Infra Phases Taxonomy](../../../../taxonomy/devsecops/infra-phases/v1.0.0/README.md)
- [Recipe Schema](../recipe/v1.0.0/README.md)
