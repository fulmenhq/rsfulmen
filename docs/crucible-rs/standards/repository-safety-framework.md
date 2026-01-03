---
title: "Fulmen Repository Safety Framework"
description: "Universal safety requirements and behavioral patterns for all Fulmen repositories"
author: "Schema Cartographer"
date: "2025-10-03"
last_updated: "2025-10-03"
status: "draft"
tags: ["safety", "repository", "compliance"]
---

# Fulmen Repository Safety Framework

**Framework**: Universal safety requirements and behavioral patterns that **every** repository in the Fulmen ecosystem must implement through its local `REPOSITORY-SAFETY-PROTOCOLS.md`.

> This document mirrors the policy authored in Fulmen Codex so engineering teams can reference the canonical requirements directly from Crucible. Domain-specific templates and the more thorough policy narrative still live in Codex; this copy is the standards-view for quick access.

## 🎯 Purpose

Establish consistent safety foundations across all Fulmen repositories while allowing domain-specific customization via structured templates and local implementation.

## 🏗️ Three-Tier Safety Architecture

### Tier 1 – Universal Foundation (This document)

- Mandatory patterns every repository must implement.
- Non-negotiable safety behaviors and quality standards.
- Universal emergency procedures and escalation protocols.

### Tier 2 – Domain Templates (see Fulmen Codex)

- Pre-built safety templates for common system types (CLI, services, proxies, etc.).
- Domain-specific risk patterns and mitigation strategies.
- Customizable frameworks for consistent implementation.

### Tier 3 – Repository Implementation

- Repository-specific `REPOSITORY-SAFETY-PROTOCOLS.md`.
- Must reference (and comply with) Tier 1 and the applicable Tier 2 template.
- Adds local customization for unique risks and emergency procedures.

## 📋 Mandatory Universal Requirements

### Requirement 1 – Risk Classification System

All repositories **must** implement a three-level risk classification:

```markdown
## OPERATIONAL DANGER CLASSIFICATION

### Level 1: CATASTROPHIC – never execute without user confirmation

- [List repository-specific catastrophic operations]

### Level 2: HIGH RISK – validate before execution

- [List high-risk operations with validation requirements]

### Level 3: MEDIUM RISK – proceed with caution

- [List routine operations with minor risk]
```

Each level must document specific operations, behavioral requirements, and escalation guidance (Level 1 incidents require immediate maintainer involvement).

### Requirement 2 – Explicit Authorization Protocol

All repositories **must** document and enforce the following flow for Level 1 and Level 2 operations:

```markdown
1. STOP – pause and assess the operation completely
2. DESCRIBE – explain exactly what will be performed and why
3. CLASSIFY – identify risk level and potential impacts
4. ASK – request explicit user confirmation with full context
5. WAIT – do not proceed until explicit authorization is received
6. CONFIRM – repeat back what was authorized before execution
7. EXECUTE – perform only the specific authorized operation
8. AUDIT – document authorization and results
```

Prohibited behaviors (universal):

- Automatic execution of Level 1 operations.
- Chaining high-risk operations with `&&`, `;`, or bulk scripts.
- Proceeding without explicit authorization.

### Requirement 3 – Quality Gate Standards

All repositories **must** maintain effective quality validation:

- Formatting and linting for all supported languages/content.
- Tests (unit/integration) meeting minimum coverage targets (default 80%).
- Security scanning (static analysis, dependency audit).
- Build/compilation success.
- Documentation consistency checks.

Quality gates may only be bypassed with explicit maintainer approval and incident tracking.

#### Commit Operation Standards

**Pre-Commit Requirements:**

- **Option A (Clean Commit):** Repository working tree is clean with no unstaged files. All changes staged and quality gates passed.
- **Option B (Partial Commit):** Explicit approval obtained to commit with unstaged files present. Approval must document reason and scope of partial commit.
- **Prohibited:** Committing without running quality gates (`make check-all` or equivalent) unless emergency bypass approval granted.

**Post-Commit Verification:**

- Always run `git status` after commit attempt to verify expected state.
- Check commit operation return code; failure requires analysis and remediation before proceeding.
- Verify commit message attribution and formatting meet standards.

#### Push Operation Standards

**Pre-Push Requirements:**

- Repository working tree **must** be clean (no unstaged files) unless emergency bypass approval obtained.
- Run `make prepush` or `make check-all` target to validate all quality gates pass.
- Verify commit history includes proper attribution for all commits in push range.
- **Prohibited:** Pushing with unstaged changes or failed quality gates without explicit emergency approval.

**Post-Push Verification:**

- Check push operation return code and output for errors or warnings.
- Verify remote branch state matches expected commits with `git log origin/branch`.
- Run `git status` to confirm clean state post-push.

### Requirement 4 – Attribution Standards

Repositories must enforce the [Agentic Attribution Standard](agentic-attribution.md):

```markdown
Generated by [Agent Name] ([Interface](interface-url)) under supervision of [@maintainer](https://github.com/maintainer)

Co-Authored-By: [Agent Name] <noreply@fulmenhq.dev>
Co-Authored-By: [Human Name] <email@domain.ext>
```

Ensure interface prompt files (e.g., `CLAUDE.md`, `CODEX.md`, `.cline/rules/PROJECT.md`) point back to `AGENTS.md` so agent workflows remain traceable.

## 🚨 Emergency Preparedness (Universal Expectations)

Every repository must define:

- Emergency contact and escalation procedures.
- Incident response steps and rollback plans for Level 1 operations.
- Backup/restore or recovery paths where applicable.
- Communication plan for user/customer impact.

See Fulmen Codex “Emergency Procedures” for reference templates.

## 📦 Repository Deliverables

Each repository must maintain:

1. **`REPOSITORY-SAFETY-PROTOCOLS.md`** – references this framework, implements relevant domain template, documents repository-specific risks, and is kept current.
2. **Quality infrastructure** – CI/CD enforcing quality gates, pre-commit hooks, coverage/reporting, and security scans.
3. **Emergency readiness** – contact information, response steps, and tested recovery procedures.

## ✅ Compliance & Audit

Suggested cadence:

- Monthly – safety protocol compliance check.
- Quarterly – review quality gate effectiveness.
- Annually – full safety framework audit and update.

Compliance metrics:

- ≥95% commits pass quality gates on first attempt.
- Zero Level 1 operations executed without written authorization.
- 100% attribution compliance.
- Mean time to resolve safety incidents < 4 hours.

## 🔧 Implementation Checklist (Maintainers)

- [ ] `REPOSITORY-SAFETY-PROTOCOLS.md` references this framework and applicable template.
- [ ] Risk classification table populated with repository-specific operations.
- [ ] Authorization protocol documented and enforced.
- [ ] Quality gates configured and passing.
- [ ] Attribution standard integrated (commit templates/pre-commit).
- [ ] Emergency procedures documented with contacts.
- [ ] Agent interface files created and linked to `AGENTS.md`.
- [ ] Team training/acknowledgment recorded.

Auditors should validate the same checklist plus confirm quality gates/emergency drills are actually exercised.

## 🔗 Related References

- [Agentic Attribution Standard](agentic-attribution.md)
- [Consuming Crucible Assets Guide](../guides/consuming-crucible-assets.md) (for SSOT asset synchronization)
- Fulmen Codex policy: `policies/repository-safety-framework`
- Fulmen Codex templates: `templates/safety-protocols/*`
