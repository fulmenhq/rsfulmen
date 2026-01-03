---
title: "FulmenHQ Repository Operations SOP"
description: "Standard operating procedure for repository operations, including safety protocols and guardrails"
author: "Pipeline Architect"
date: "2025-10-06"
last_updated: "2025-10-06"
status: "approved"
tags: ["sop", "operations", "safety", "repository"]
---

# FulmenHQ Repository Operations SOP

## Overview

This SOP defines the operational guidelines, safety protocols, and guardrails for all FulmenHQ repositories. It serves as the canonical standard for repository operations to ensure consistency, reliability, and security across the ecosystem.

**When performing repository operations**, follow these guidelines to prevent accidental drift, data loss, or unauthorized releases.

## General Rules

1. **Human Oversight**: No merges, tags, or package publishes without explicit approval from @3leapsdave.
2. **Command Discipline**: Prefer `make` targets and bundled scripts over ad-hoc commands to ensure language wrappers stay in sync.
3. **Plan Before Action**: Record work plans in `.plans/` (gitignored) or session transcripts before making structural changes.

## High-Risk Operations

| Operation                              | Risk                          | Protocol                                                                                                    |
| -------------------------------------- | ----------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Editing schemas/docs/templates in root | Breaking downstream consumers | Update root, run `bun run sync:to-lang`, verify tests, and ensure `make release-check` passes before merge. |
| Version bumps                          | Package/version drift         | Use `bun run version:update` or `make version-set`. Never edit `VERSION` manually.                          |
| Publishing Go/npm packages             | Releasing stale assets        | Confirm release checklist complete, run `make release-prepare`, obtain human approval, then tag/publish.    |
| Deleting schemas or standards          | Downstream breakage           | Requires issue + maintainer review. Provide migration plan and version bump.                                |
| Modifying CI workflows                 | Broken automation             | Review with @3leapsdave. Test in branch before merging.                                                     |

## Commit and Push Checklists

### Pre-Commit Checklist

**Before running `git commit`:**

- [ ] Run `make check-all` and verify all quality gates pass
- [ ] Check working tree state with `git status`
  - **Clean commit (preferred)**: No unstaged files present
  - **Partial commit**: Explicit approval obtained and documented for committing with unstaged files
- [ ] Verify commit message follows attribution standards (see [Agentic Attribution Standard](../standards/agentic-attribution.md))
- [ ] Commit message is concise and scannable (avoid excessive emoji sections, see [Commit Message Style](#commit-message-style))
- [ ] Review staged changes with `git diff --staged` to confirm intended scope

**After `git commit` attempt:**

- [ ] Check commit command return code (zero = success, non-zero = failure requiring investigation)
- [ ] Run `git status` to verify expected repository state
- [ ] Review commit with `git log -1 --stat` to confirm files and message
- [ ] **DO NOT PROCEED** to additional commits or push if working tree state is unexpected

### Pre-Push Checklist

**Before running `git push`:**

- [ ] Verify working tree is clean with `git status` (no unstaged or uncommitted files)
  - **Exception**: Emergency bypass with explicit maintainer approval and incident tracking
- [ ] Run `make prepush` or `make check-all` to validate quality gates
- [ ] Review commit history to be pushed: `git log origin/main..HEAD`
- [ ] Verify all commits have proper attribution (agent + supervisor)
- [ ] Confirm push target is correct branch: `git branch --show-current`
- [ ] Obtain explicit approval from @3leapsdave for pushes to `main`

**After `git push` attempt:**

- [ ] Check push command return code and output for errors or warnings
- [ ] Verify remote state matches local: `git log origin/main -3`
- [ ] Run `git status` to confirm clean working tree post-push
- [ ] Monitor CI/CD pipeline for any failures triggered by push

### Common Pre-Operation Mistakes to Avoid

| Mistake                                      | Impact                                             | Prevention                                                                   |
| -------------------------------------------- | -------------------------------------------------- | ---------------------------------------------------------------------------- |
| Committing with unstaged files (no approval) | Incomplete changes committed, confusion in history | Always check `git status` before commit; obtain approval for partial commits |
| Pushing with dirty working tree              | Local changes not in remote, desync risk           | Run `git status` before push; ensure clean state                             |
| Skipping quality gates                       | Broken code in main branch, failing CI             | Always run `make check-all` or `make prepush`                                |
| Not verifying operation results              | Failed operations go unnoticed, compounding issues | Always check return codes and run `git status` after operations              |
| Committing without format/sync               | Format churn in subsequent builds                  | Ensure `make build` includes sync + format steps                             |

## Commit Message Style

**Principle**: Commit messages should be readable in `git log --oneline` and provide necessary context in detail view. Extensive documentation belongs in code comments, `docs/`, or linked issues/ADRs.

### Good - Concise and Scannable

```
fix: create symlinks instead of copies for type:link in bootstrap

The installLink() function was copying files instead of creating symlinks
for type:link installations, causing bin/goneat to become stale after
source rebuilds. Now properly uses symlinkSync() per Fulmen Helper
Library Standard requirement (lines 166-174).

Verified: bin/goneat now tracks source automatically without re-bootstrap.

[Attribution trailers per agentic-attribution.md]
```

**Why this is good**:

- Concise subject line (50 chars)
- Paragraph format body explaining what/why
- Specific file/line references when helpful
- Verification notes included
- Readable in `git log --oneline` and GitHub UI

### Avoid - Excessive Emoji Sections

```
fix: create symlinks instead of copies for type:link in bootstrap

🎯 Changes:
- Import symlinkSync, unlinkSync, and resolve
- Replace copyFileSync() with symlinkSync()
- Update console output

🐛 Bug Details:
- Before: bin/goneat was a copy (Module: Oct 10 version)
- After: bin/goneat is a symlink (Module: Oct 16 version)
- Symptom: Needed make bootstrap-force after every rebuild
- Root cause: Line 215-216 used copyFileSync

✅ Verification:
- ls -la shows symlink
- Versions match exactly
- Symlink tracks automatically

📋 Compliance: Fulmen Helper Library Standard lines 166-174

[Attribution trailers]
```

**Why to avoid**:

- Multiple emoji-prefixed sections create noise in commit history
- Not appropriate for public OSS repositories
- Better suited for internal documentation, ADRs, or issue discussions
- Makes `git log` and GitHub UI difficult to scan

### Commit Message Structure

1. **Subject line**: Type prefix + imperative mood summary (≤50 chars)
   - Conventional commit types: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`

2. **Body**: Paragraph format explaining what and why (wrap at 72 chars)
   - Focus on user-visible changes and rationale
   - Reference specific files/lines when helpful
   - Include verification notes if applicable
   - Keep it concise - 2-4 paragraphs maximum

3. **Trailers**: Agentic attribution per [Agentic Attribution Standard](../standards/agentic-attribution.md)
   - Generated by [Agent Identity] ([Interface](url)) under supervision of [@maintainer]
   - Co-Authored-By: [Agent Identity] <noreply@3leaps.net>
   - Committer-of-Record: [Human Name] <email> [@maintainer]

### Quick Style Check

Before committing, verify:

- [ ] Subject line ≤50 characters
- [ ] Body uses paragraph format (not emoji sections)
- [ ] Includes what changed and why
- [ ] References specific files/lines when helpful
- [ ] Attribution trailers complete per standard

## Required Commands & Tools

- `make bootstrap`, `make lint`, `make test`, `make release-check`
- `bun run sync:to-lang`
- `bun run scripts/update-version.ts`
- `bun run scripts/crucible-pull.ts --validate` (for pull-script verification)

## Forbidden Actions

- Force pushes to `main` or release branches.
- Publishing packages from unreviewed branches.
- Storing secrets or credentials in the repository.

## Incident Response

1. **Assess**: Capture logs, diffs, and current state.
2. **Notify**: Ping @3leapsdave (and other maintainers as needed) on Mattermost / GitHub.
3. **Mitigate**: Revert offending commits or tags (`git revert`, `git tag -d` etc.) under supervision.
4. **Document**: Record the incident and remediation in `.plans/incident-logs/` (gitignored) and update relevant SOPs if needed.

## Environment & Credentials

- GitHub tokens: stored in repository secrets (`FULMEN_NPM_TOKEN`, `FULMEN_GO_PROXY_TOKEN` – future use).
- Mattermost: use assigned agent handles once channels go live.
- Local scripts assume no network access beyond public GitHub unless explicitly approved.

## Related Standards and SOPs

- [Repository Structure SOP](repository-structure.md)
- [CI/CD Operations SOP](cicd-operations.md)
- [Makefile Standard](../standards/makefile-standard.md)
- [Release Checklist Standard](../standards/release-checklist-standard.md)
- [AI Agent Collaboration Standard](../standards/ai-agents.md)
- [Agentic Attribution Standard](../standards/agentic-attribution.md)

---

**Status**: Approved  
**Last Updated**: 2025-10-06  
**Author**: Pipeline Architect  
**Effective Date**: 2025-10-06
