# Agentic Schemas v0

Schemas for AI coding agent collaboration.

**Status**: Unstable (v0) - breaking changes may occur without notice.

## Schemas

| Schema                    | Purpose                            |
| ------------------------- | ---------------------------------- |
| `role-prompt.schema.json` | Role prompt frontmatter validation |

## Usage

### Role Prompts

Role prompts define how AI coding assistants operate within repositories. They use YAML frontmatter validated against `role-prompt.schema.json`:

```yaml
---
slug: devlead
name: Development Lead
description: Architecture, implementation, and code review
version: 1.0.0
author: entarch
status: approved
category: agentic
tags: [role, implementation, architecture]
---

# Role: devlead

One-line description.

## Context
When to use this role...

## Scope
- What this role covers
- Boundaries of responsibility

## Mindset
Focus on:
- Key questions to ask
- Perspective to maintain

## Responsibilities
- Specific duties
- Quality expectations

## Escalates To
- When to escalate
- Who to escalate to

## Does Not
- Explicit exclusions
- Out-of-scope items

## Examples
Commit examples, checklists...
```

### Required Body Sections

By convention (not schema-enforced), role prompts should include:

| Section          | Required    | Purpose                                  |
| ---------------- | ----------- | ---------------------------------------- |
| Context          | Recommended | When to use this role                    |
| Scope            | Yes         | What this role covers                    |
| Mindset          | Recommended | Context engineering for AI effectiveness |
| Responsibilities | Yes         | Specific duties and expectations         |
| Escalates To     | Yes         | When and who to hand off to              |
| Does Not         | Yes         | Explicit boundaries and exclusions       |
| Examples         | Recommended | Commit patterns, checklists              |

## Schema URL

```
https://schemas.3leaps.dev/agentic/v0/role-prompt.schema.json
```

## Related

- [docs/catalog/roles/](../../docs/catalog/roles/) - Baseline role definitions
- [docs/repository/agent-identity.md](../../docs/repository/agent-identity.md) - Identity scheme
- [FulmenHQ Crucible](https://github.com/fulmenhq/crucible) - Enterprise role catalog
