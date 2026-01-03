---
title: "FulmenHQ CI/CD Operations SOP"
description: "Standard pipeline expectations and required automation entry points"
author: "Codex Assistant"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "draft"
tags: ["sop", "cicd", "devsecops", "automation"]
---

# CI/CD Operations SOP

## Purpose

Define the minimum automation contract for all Fulmen repositories so that shared tooling, agents, and pipelines can operate predictably. This SOP applies to **every** repository, regardless of language.

## Core Requirements

1. **Makefile Compliance**
   - Implement the targets defined in the [Makefile Standard](../standards/makefile-standard.md).
   - Pipelines SHOULD call `make lint`, `make test`, and `make release-check`.
2. **Release Checklist Integration**
   - `make release-check` must cover or call every gate in `RELEASE_CHECKLIST.md`.
3. **Version Synchronization**
   - Use the standard scripts (`bun run version:update`, etc.) so `VERSION`, language packages, and metadata stay aligned.
4. **Schema/Docs Sync** (for SSOT repos like Crucible)
   - Run `bun run sync:to-lang` (or repo equivalent) in CI to detect unsynced changes.
5. **Security Gates**
   - Include lint, dependency audit, and static analysis appropriate for the language stack.

## Recommended Pipeline Flow

1. `make bootstrap`
2. `make lint`
3. `make fmt` (optional in CI, but run locally)
4. `make test`
5. `make build`
6. `make release-check`

## Standard Validation Hooks

Repositories SHOULD expose the following commands so CI can hook in easily:

- `make ci` – Runs bootstrap, lint, test, build.
- `make ci-quick` – Minimal smoke tests for PRs.
- `make ci-full` – Full release validation (includes `release-check`).
- `make tools` – Lightweight verification of external dependencies (optional in fast paths).

## DevSecOps Expectations

- Align with forthcoming standards under `docs/standards/` (e.g., dependency scanning, SBOM generation).
- Document secrets management and required environment variables in `REPOSITORY_SAFETY_PROTOCOLS.md`.
- When using external services (npm publish, Go module proxy), ensure tokens are stored in GitHub secrets with consistent names (`FULMEN_NPM_TOKEN`, etc.).

## Related Standards

- [Makefile Standard](../standards/makefile-standard.md)
- [Release Checklist Standard](../standards/release-checklist-standard.md)
- [AI Agent Collaboration Standard](../standards/ai-agents.md)
- [Agentic Attribution Standard](../standards/agentic-attribution.md)

## Future Work

- Add language-specific appendices (Go, TypeScript, Python) with extra targets or tooling requirements.
- Provide reusable GitHub Actions workflows that rely on the standardized make targets.
- Integrate DevSecOps scanners (SAST/DAST) as mandatory gates.
