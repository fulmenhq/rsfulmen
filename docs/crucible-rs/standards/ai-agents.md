---
title: "AI Agent Collaboration Standard"
description: "Operating model and identity conventions for AI assistants working in Fulmen repositories"
author: "entarch"
author_of_record: "Dave Thompson (https://github.com/3leapsdave)"
supervised_by: "@3leapsdave"
date: "2025-10-02"
last_updated: "2026-01-01"
status: "approved"
tags: ["standards", "ai", "agents", "collaboration", "roles"]
---

# AI Agent Collaboration Standard

## Purpose

Define the operating model, role configuration, and attribution requirements that allow AI assistants to collaborate safely alongside human maintainers in Fulmen repositories.

## Operating Modes

### Supervised Mode

Human reviews and approves before commit:

| Aspect          | Description                  |
| --------------- | ---------------------------- |
| Accountability  | Human (Committer-of-Record)  |
| GitHub Account  | Uses human's credentials     |
| Review Required | Yes, before every commit     |
| Use Case        | Default for all repositories |

### Autonomous Mode

Agent operates independently within defined boundaries:

| Aspect          | Description                             |
| --------------- | --------------------------------------- |
| Accountability  | Organization via Escalation-Contact     |
| GitHub Account  | Dedicated `@<org>-agent-<role>` account |
| Review Required | Post-commit audit                       |
| Use Case        | CI/CD automation, scheduled tasks       |

**Note**: Account presence implies autonomous capability. Repositories using supervised mode only do not configure autonomous agent accounts.

## Role-Based Identity

Agents operate in **roles**, not named personalities. Roles define:

- What the agent is responsible for
- How to approach problems (mindset)
- When to escalate
- What's out of scope

### Available Roles

| Role                  | Identifier | Scope                                      |
| --------------------- | ---------- | ------------------------------------------ |
| Development Lead      | `devlead`  | Implementation, architecture, feature work |
| Development Reviewer  | `devrev`   | Code review, bug finding, four-eyes audit  |
| Information Architect | `infoarch` | Documentation, schemas, standards          |
| Enterprise Architect  | `entarch`  | Cross-repo coordination, API parity        |
| CI/CD Automation      | `cicd`     | Pipelines, builds, automation              |
| Security Review       | `secrev`   | Security analysis, vulnerability review    |
| Data Engineering      | `dataeng`  | Database design, data pipelines            |

See [Role Catalog](../catalog/agentic/roles/README.md) for complete role definitions.

### Role vs Named Identity

**Previous approach** (deprecated):

- Named personalities: "Schema Cartographer", "Pipeline Architect"
- Emoji identifiers
- Persistent handles: `@schema-cartographer`
- Risk of anthropomorphization

**Current approach** (role-based):

- Functional roles: `infoarch`, `cicd`, `devlead`
- Model name in attribution: "Claude Opus 4.5"
- Role context per session
- Clear separation of model, interface, and function

## Required Artifacts

Every repository MUST include:

| File             | Purpose                                                   |
| ---------------- | --------------------------------------------------------- |
| `AGENTS.md`      | Operating model, role configuration, session protocol     |
| `MAINTAINERS.md` | Human maintainers, autonomous agents (if any), governance |

### Optional Artifacts

| File                                         | Purpose                                         |
| -------------------------------------------- | ----------------------------------------------- |
| `REPOSITORY_SAFETY_PROTOCOLS.md`             | Safety boundaries, guardrails, escalation paths |
| Interface adapters (`CLAUDE.md`, `CODEX.md`) | Interface-specific configuration                |

## AGENTS.md Structure

```markdown
# Repository – AI Agents Startup Guide

## Read First

1. Check AGENTS.local.md if it exists
2. Read MAINTAINERS.md for contacts
3. Read REPOSITORY_SAFETY_PROTOCOLS.md
4. Understand project scope before changes

## Operating Model

| Aspect         | Setting                                  |
| -------------- | ---------------------------------------- | ------------------ | --------------- |
| Mode           | Supervised (human reviews before commit) |
| Classification | <code-substantive                        | security-sensitive | data-integrity> |
| Role Required  | Yes                                      |

## Roles

| Role      | Prompt                           | Notes       |
| --------- | -------------------------------- | ----------- |
| `devlead` | [devlead.md](path/to/devlead.md) | Development |
| `secrev`  | [secrev.md](path/to/secrev.md)   | Security    |

## Commit Attribution

[Attribution format and examples]

## Session Protocol

[Before changes, before committing, quality gates]
```

## MAINTAINERS.md Structure

```markdown
# Repository – Maintainers

## Human Maintainers

| Name | GitHub  | Email | Role            |
| ---- | ------- | ----- | --------------- |
| Name | @handle | email | Lead maintainer |

## Autonomous Agents

_None configured. This repository uses supervised mode only._

OR (if autonomous agents are configured):

| Account         | Role | Escalation Contact |
| --------------- | ---- | ------------------ |
| @org-agent-cicd | cicd | @human-maintainer  |

## AI-Assisted Development

This repository uses AI assistants in **supervised mode**.
See [AGENTS.md](AGENTS.md) for configuration.
```

## Attribution Format

### Commit Attribution

```
<type>(<scope>): <subject>

<body>

Changes:
- <change 1>
- <change 2>

Generated by <Model> via <Interface> under supervision of @<maintainer>

Co-Authored-By: <Model> <noreply@3leaps.net>
Role: <role>
Committer-of-Record: <Human> <email> [@handle]
```

### Required Elements

1. **Model**: Claude Opus 4.5, Claude Sonnet, GPT-5.2
2. **Interface**: Claude Code, Cursor, Codex CLI
3. **Role**: devlead, infoarch, secrev, etc.
4. **Committer-of-Record**: Human accountable for the commit

See [Agentic Attribution Standard](agentic-attribution.md) for full specification.

## Four-Eyes Pattern

Use `devlead` + `devrev` for code review:

- `devlead`: Writes implementation ("build the solution")
- `devrev`: Reviews for correctness ("find the problems")
- `secrev`: Reviews for security (for security-sensitive changes)

Different models can fill different roles in the same workflow.

## Interface Adapters

Repositories may provide interface-specific configuration:

| Interface   | Config File               | Notes                    |
| ----------- | ------------------------- | ------------------------ |
| Claude Code | `CLAUDE.md`               | Primary for FulmenHQ     |
| Codex CLI   | `CODEX.md`                |                          |
| Cursor      | `AGENTS.md`               |                          |
| Cline       | `.cline/rules/PROJECT.md` | Must reference AGENTS.md |
| OpenCode    | `AGENTS.md`               |                          |

Each adapter should reference `AGENTS.md` for role configuration.

## Session Protocol

1. **Context Review**: Read MAINTAINERS.md, understand operating model
2. **Role Selection**: Confirm which role applies to the task
3. **Quality Gates**: Run `make precommit` before commits
4. **Attribution**: Include proper trailers with role

## Migration from Named Identities

Repositories migrating from named identities:

| Previous            | New Role   |
| ------------------- | ---------- |
| Schema Cartographer | `infoarch` |
| Pipeline Architect  | `cicd`     |
| EA Steward          | `entarch`  |
| Code Scout          | `devlead`  |

### Elements to Remove

- Emoji assignments
- Named identity @handles
- "Established" dates
- "AI Co-Maintainers" section in MAINTAINERS.md

### Elements to Keep

- Model name in attribution
- Human supervision relationships
- Quality protocols
- Interface adapter documentation

## Related Documents

- [Role Catalog](../catalog/agentic/roles/README.md) - Role definitions
- [Git Commit Attribution](../catalog/agentic/attribution/git-commit.md) - Commit template
- [Agentic Attribution Standard](agentic-attribution.md) - Full attribution spec
- [Frontmatter Standard](frontmatter-standard.md) - Document metadata

---

**Standard Version**: 2.0.0
**Status**: Approved - Role-based identity model
