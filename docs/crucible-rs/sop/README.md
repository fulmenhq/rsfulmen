---
title: "FulmenHQ SOP Index"
description: "Quick reference for standard operating procedures across Fulmen repositories"
author: "Codex Assistant"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "draft"
tags: ["sop", "process", "repository"]
---

# Standard Operating Procedures

Standard operating procedures (SOPs) capture repeatable workflows that keep Fulmen projects consistent. Use this index to find the process documentation you need when remediating repos or planning releases.

## Available SOPs

| Document                                                          | Purpose                                                                                            |
| ----------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| [Repository Structure SOP](repository-structure.md)               | Required files, directories, conventions, and pseudo-monorepo rules every Fulmen repo must follow. |
| [Repository Version Adoption SOP](repository-version-adoption.md) | Step-by-step process for migrating or adopting CalVer/SemVer in existing repositories.             |
| [CI/CD Operations SOP](cicd-operations.md)                        | Standard automation flows and required make targets for pipelines.                                 |
| [Config Path Migration SOP](config-path-migration.md)             | Guidance for moving tools to the shared Fulmen config directories.                                 |

## When to Add a New SOP

Create an SOP when:

- A workflow spans multiple teams/repos and we need consistent execution (e.g., release management, dependency audits).
- The process has clear owners and success criteria.
- We can describe inputs/outputs in a way that is automation-friendly.

Remember to add frontmatter, update this index, and link the SOP from relevant standards (README, contributing guides, etc.).

## Related References

- [Standards Directory](../standards/README.md)
- [Crucible Sync Model Architecture](../architecture/sync-model.md)
- [Release Checklist Standard](../standards/release-checklist-standard.md)
