---
title: Pre-Commit Processes
description: Standardized pre-commit hooks and validation for code quality and security.
status: draft
---

# Pre-Commit Processes

Enforce excellence at every commit: format, lint, test subsets, security scans.

## Workflow

1. **Format & Lint**: Run goneat format/assess (fail on medium+ issues).
2. **Type Check**: tsc --noEmit or equivalent (strict mode).
3. **Test Changed Files**: Jest/Cypress on modified paths (coverage >80% for touched code).
4. **Security Scan**: gitleaks/gosec on diffs; block secrets/high vulns.
5. **Schema Validate**: goneat validate on configs/schemas.
6. **Maturity Check**: Ensure phase-aligned (e.g., no dirty git in RC+).

From prototypes: Use .goneatignore for exclusions; parallel sharding for large repos; JSON output for CI gating.

Integrate via goneat hooks: `goneat hooks generate --pre-commit`.

## Best Practices

- Fail-fast: Critical issues block commit.
- Auto-fix where possible (e.g., prettier, gofmt).
- Diff-aware: Focus on changes to reduce noise.
- Human-AI: AI suggests fixes; human reviews/approves via Fulward.
