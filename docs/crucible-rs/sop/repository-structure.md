---
title: "FulmenHQ Repository Structure SOP"
description: "Standard operating procedure for repository structure across all FulmenHQ projects"
author: "@3leapsdave"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "approved"
tags: ["sop", "repository", "structure", "standards"]
---

# FulmenHQ Repository Structure SOP

## Overview

This SOP defines the required structure, files, and conventions for all repositories in the FulmenHQ ecosystem. Following this structure ensures consistency, discoverability, and maintainability across the organization.

**When creating a new repository**, use this document as a checklist to ensure all required components are in place.

## Required Root Files

All FulmenHQ repositories MUST include these files in the repository root:

| File                             | Required       | Purpose                                                 | Reference                                                                |
| -------------------------------- | -------------- | ------------------------------------------------------- | ------------------------------------------------------------------------ |
| `README.md`                      | ✅ Yes         | Project overview, quick start, installation             | [README Requirements](#readmemd)                                         |
| `VERSION`                        | ✅ Yes         | Single source of truth for version                      | [Repository Versioning](../standards/repository-versioning.md)           |
| `LICENSE`                        | ✅ Yes         | Legal terms and usage rights                            | [License Requirements](#license)                                         |
| `CONTRIBUTING.md`                | ✅ Yes         | Contribution guidelines and workflow                    | [Contributing Requirements](#contributingmd)                             |
| `CHANGELOG.md`                   | ✅ Yes         | Release history and changes                             | [Changelog Requirements](#changelogmd)                                   |
| `.gitignore`                     | ✅ Yes         | Ignored files and directories                           | Language-specific                                                        |
| `Makefile`                       | ✅ Yes         | Standard automation entry points                        | [Makefile Standard](../standards/makefile-standard.md)                   |
| `RELEASE_CHECKLIST.md`           | ✅ Yes         | Reusable release playbook template                      | [Release Checklist Standard](../standards/release-checklist-standard.md) |
| `AGENTS.md`                      | ✅ Yes         | AI agent startup guide and interface registry           | [AI Agent Collaboration](../standards/ai-agents.md)                      |
| `MAINTAINERS.md`                 | ✅ Yes         | Human + AI maintainer roster, handles, supervision      | [AI Agent Collaboration](../standards/ai-agents.md)                      |
| `REPOSITORY_SAFETY_PROTOCOLS.md` | ⚠️ Recommended | Operational guardrails and escalation paths             | [AI Agent Collaboration](../standards/ai-agents.md)                      |
| `.goneat/tools.yaml`             | ✅ Yes         | External tool manifest consumed by make bootstrap/tools | [Makefile Standard](../standards/makefile-standard.md)                   |
| `SECURITY.md`                    | ⚠️ Recommended | Security policy and reporting                           | GitHub standard                                                          |
| `.github/`                       | ⚠️ Recommended | GitHub Actions workflows, templates                     | [GitHub Integration](#github-integration)                                |

## File-by-File Requirements

### README.md

**Purpose**: First impression and primary entry point for users and contributors.

**Required sections:**

1. **Title and Tagline**

   ```markdown
   # Project Name

   Brief tagline explaining what the project does
   ```

2. **Badges** (optional but recommended)
   - Release version
   - License
   - CI/CD status
   - Language version

   ```markdown
   [![Release](https://img.shields.io/github/v/release/fulmenhq/project)](...)
   [![License](https://img.shields.io/badge/License-...)](LICENSE)
   ```

3. **Quick Start / TL;DR**
   - Installation in 3-5 steps
   - Basic usage example
   - Link to detailed docs

4. **Overview / What is X?**
   - Problem being solved
   - How this project solves it
   - Key features and benefits

5. **Installation**
   - Multiple installation methods
   - Prerequisites
   - Verification steps

6. **Usage**
   - Basic examples
   - Common use cases
   - Links to detailed documentation

7. **Documentation**
   - Link to full documentation
   - Architecture overview
   - API reference

8. **Contributing**
   - Link to CONTRIBUTING.md
   - Quick contribution guidelines

9. **License**
   - Link to LICENSE file
   - Brief license summary

10. **Status** (if applicable)
    - Lifecycle phase (alpha, beta, stable)
    - Release phase (rc, production)
    - Known limitations

**Format guidelines:**

- Use clear headers (H1 for title, H2 for main sections)
- Keep quick start under 5 minutes
- Include visual examples where helpful
- Link to detailed docs rather than including everything
- Use code blocks with language syntax highlighting

**Example structure:**

```markdown
# Project Name

Tagline

## Quick Start

## What is X?

## Installation

## Usage

## Documentation

## Contributing

## License
```

### VERSION

**Purpose**: Single source of truth for repository version.

**Format:**

```
2025.10.0
```

**Requirements:**

- Single line with version number
- No `v` prefix
- No trailing newline or whitespace
- Must follow chosen versioning strategy (SemVer or CalVer)

**References:**

- [Repository Versioning Standard](../standards/repository-versioning.md)
- [Repository Version Adoption SOP](repository-version-adoption.md)

**Choosing your strategy:**

- **SemVer** (`MAJOR.MINOR.PATCH`): Libraries, tools, APIs
- **CalVer** (`YYYY.0M.MICRO`): Documentation, schemas, standards

### LICENSE

**Purpose**: Legal terms governing use, modification, and distribution.

**FulmenHQ License Options:**

1. **MIT License** (most common)
   - Standard permissive license
   - Use for libraries, tools, applications
   - Simple, well-understood

2. **Apache 2.0 License**
   - Permissive with patent grant
   - Use for projects with patent concerns
   - Includes contributor license agreement

3. **Hybrid License** (specialized)
   - MIT for code
   - CC0 1.0 for creative works/data
   - Use for schema/documentation repositories
   - Example: `crucible`

**Required elements:**

- Copyright notice: `Copyright (c) 2025 3 Leaps, LLC`
- Full license text
- Trademark notice (see crucible LICENSE for template)

**Template locations:**

- MIT: Standard template from OSI
- Apache 2.0: Standard template from Apache Foundation
- Hybrid: Use `fulmenhq/crucible` LICENSE as template

### CONTRIBUTING.md

**Purpose**: Guide contributors through the contribution process.

**Required sections:**

1. **How to Contribute**
   - Reporting issues
   - Submitting changes
   - Pull request process

2. **Versioning**
   - Chosen strategy (SemVer or CalVer)
   - Link to Repository Versioning Standard
   - Version management commands

3. **Development Setup**
   - Prerequisites
   - Installation steps
   - Running tests
   - Building the project

4. **Code Standards**
   - Link to coding standards
   - Linting and formatting requirements
   - Testing requirements

5. **Commit Convention**
   - Conventional Commits format
   - Commit message examples

   ```
   - feat: New features
   - fix: Bug fixes
   - docs: Documentation changes
   - chore: Maintenance tasks
   ```

6. **License**
   - License summary
   - Contributor agreement statement

**Template:**

```markdown
# Contributing to [Project Name]

## How to Contribute

### Reporting Issues

### Submitting Changes

## Versioning

This repository uses [SemVer|CalVer] for version management.
See [Repository Versioning Standard](...).

## Development Setup

## Code Standards

## Commit Convention

## License
```

### CHANGELOG.md

**Purpose**: Track changes across versions for users and maintainers.

**Format**: Based on [Keep a Changelog](https://keepachangelog.com/)

**Structure:**

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added

- New features in progress

### Changed

- Changes to existing functionality

### Deprecated

- Soon-to-be removed features

### Removed

- Removed features

### Fixed

- Bug fixes

### Security

- Security improvements

## [1.0.0] - 2025-10-02

### Added

- Initial release
```

**Guidelines:**

- Use SemVer/CalVer version numbers as headers
- Include dates in ISO 8601 format (YYYY-MM-DD)
- Group changes by type (Added, Changed, Fixed, etc.)
- Write for users, not developers (focus on impact)
- Update `[Unreleased]` section during development
- Move to versioned section on release

**For CalVer repositories:**

```markdown
## [2025.10.0] - 2025-10-15

### Added

- Terminal schema v1.0.0 revision 3
- Pathfinder schema v1.0.0
```

### AGENTS.md

**Purpose**: Orient AI assistants before they contribute.

**Requirements:**

- List every supported agentic interface (Codex CLI, Claude Code, Cursor, etc.) with the authoritative prompt file.
- Provide a "Read First" note that directs agents to `MAINTAINERS.md`, `REPOSITORY_SAFETY_PROTOCOLS.md`, and relevant SOPs.
- Document session initialization expectations, attribution rules, and any required scripts (e.g., `bun run sync:to-lang`).
- Link to interface-specific adapters (e.g., `CLAUDE.md`, `.cline/rules/PROJECT.md`) and require they reference this standard.

### MAINTAINERS.md

**Purpose**: Declare the humans and AI co-maintainers responsible for the repository.

**Requirements:**

- Separate sections for human maintainers and AI agents.
- Every AI agent entry must include identity name, emoji, canonical handle (`@code-scout`), specialization, supervising human, established date, attribution string, and contact email (use `noreply@3leaps.net` for internal agents).
- Note Mattermost channel(s) for cross-agent coordination when available.
- Cross-link to [Agentic Attribution Standard](../standards/agentic-attribution.md) so contributors use the correct signature.

### REPOSITORY_SAFETY_PROTOCOLS.md

**Purpose**: Capture high-risk operations, guardrails, and escalation paths.

**Expectations:**

- Outline dangerous commands, required approvals, rollback sequences, and contact points.
- When a repository is truly low-risk and omits this file, document the rationale in `MAINTAINERS.md` and the README’s “Status” section.
- Update alongside major infrastructure or automation changes.

## Language-Specific Structure

### Go Projects

**Required files:**

```
project/
├── README.md
├── VERSION
├── LICENSE
├── CONTRIBUTING.md
├── CHANGELOG.md
├── go.mod
├── go.sum
├── main.go (or cmd/)
├── .gitignore
└── .github/
```

**Additional requirements:**

- `go.mod` with module path: `module github.com/fulmenhq/project`
- `.gitignore` for Go (binaries, vendor/, coverage files)
- Follow [Go Coding Standards](../standards/coding/go.md)

**Common directory structure:**

```
project/
├── cmd/              # Command-line tools
│   └── projectname/
│       └── main.go
├── pkg/              # Public libraries
│   └── feature/
├── internal/         # Private code
├── docs/             # Documentation
├── scripts/          # Build/utility scripts
└── testdata/         # Test fixtures
```

### TypeScript/JavaScript Projects

**Required files:**

```
project/
├── README.md
├── VERSION
├── LICENSE
├── CONTRIBUTING.md
├── CHANGELOG.md
├── package.json
├── tsconfig.json (TypeScript)
├── .gitignore
└── .github/
```

**Additional requirements:**

- `package.json` with correct metadata
  - `name`: `@fulmenhq/project` (if scoped)
  - `version`: Must sync with VERSION file
  - `description`, `license`, `repository`
- `.gitignore` for Node.js (node_modules/, dist/, coverage/)
- Follow [TypeScript Coding Standards](../standards/coding/typescript.md)

**Common directory structure:**

```
project/
├── src/              # Source code
│   ├── index.ts
│   └── lib/
├── test/             # Tests
├── dist/             # Built output (gitignored)
├── docs/             # Documentation
└── scripts/          # Build/utility scripts
```

**Package.json version sync:**

```json
{
  "name": "@fulmenhq/project",
  "version": "1.0.0",
  "scripts": {
    "version:sync": "bun run scripts/version.ts sync"
  }
}
```

### Multi-Language Projects

**Structure (Asymmetric Pattern):**

As of v2025.10.5, Crucible uses an asymmetric structure with Go at root and other languages in `lang/` subdirectories:

```
crucible/
├── README.md         # Overall project docs
├── VERSION           # Repository version
├── LICENSE
├── CONTRIBUTING.md
├── CHANGELOG.md
├── go.mod            # Go module at root (v2025.10.5+)
├── go.sum
├── *.go              # Go sources at root
├── lang/             # Non-Go language implementations
│   ├── go/           # Breadcrumb README only (see ADR-0009)
│   ├── python/
│   │   ├── README.md
│   │   ├── pyproject.toml
│   │   └── ...
│   └── typescript/
│       ├── README.md
│       ├── package.json
│       └── ...
├── schemas/          # SSOT schemas (Go embeds directly)
├── docs/             # SSOT documentation (Go embeds directly)
├── config/           # SSOT configuration
└── scripts/          # Cross-language utilities
```

**Rationale for Asymmetric Pattern:**

- **Go at root**: Enables standard external installation (`go get github.com/fulmenhq/crucible`), supports `go:embed` directives for direct SSOT access, and follows Go ecosystem conventions
- **Python/TypeScript in lang/**: Standard package structures with synced SSOT assets
- **Industry precedent**: Terraform, Kubernetes, NATS, and etcd use similar patterns

See [ADR-0009](../architecture/decisions/ADR-0009-go-module-root-relocation.md) for detailed rationale and migration notes.

**SSOT Sync Rules:**

- Root assets (`schemas/`, `docs/`, `config/`) are the only source of truth
- **Go**: Embeds directly from root via `//go:embed` directives (no sync needed)
- **Python/TypeScript**: Receive synced copies via `bun run sync:to-lang`
- Never edit `lang/python/` or `lang/typescript/` contents manually—edit root SSOT and run sync
- Keep planning briefs in `.plans/` (gitignored) or similar so release artifacts stay clean

## Documentation Structure

**Recommended docs/ structure:**

```
docs/
├── README.md                   # Documentation index
├── architecture/               # Architecture decisions and design
│   ├── decisions/             # ADRs (Architecture Decision Records)
│   └── diagrams/
├── standards/                  # Project-specific standards
│   ├── coding/
│   └── api/
├── guides/                     # How-to guides
│   ├── getting-started.md
│   ├── development.md
│   └── deployment.md
├── reference/                  # API reference, schemas
└── sop/                        # Standard operating procedures
```

**Documentation requirements:**

- All markdown files MUST include frontmatter (see [Frontmatter Standard](../standards/frontmatter-standard.md))
- Use relative links between docs
- Include table of contents for long documents
- Keep language clear and concise

## GitHub Integration

### Required GitHub Files

```
.github/
├── workflows/              # GitHub Actions
│   ├── test.yml
│   ├── lint.yml
│   └── release.yml
├── ISSUE_TEMPLATE/         # Issue templates
│   ├── bug_report.md
│   └── feature_request.md
├── PULL_REQUEST_TEMPLATE.md
└── dependabot.yml          # Dependency updates
```

### Recommended Workflows

**1. Test workflow** (`.github/workflows/test.yml`):

- Run on push and pull requests
- Execute test suite
- Report coverage

**2. Lint workflow** (`.github/workflows/lint.yml`):

- Run linters and formatters
- Check code style
- Validate documentation

**3. Version validation** (`.github/workflows/validate-version.yml`):

- Verify VERSION file exists
- Check version format
- Ensure consistency with package files

**4. Release workflow** (`.github/workflows/release.yml`):

- Trigger on version tags
- Build artifacts
- Create GitHub release
- Publish packages (npm, Go modules, etc.)

## Repository Settings

### Branch Protection

**Main branch requirements:**

- Require pull request reviews (1+ approvers)
- Require status checks to pass
- Require branches to be up to date
- Include administrators in restrictions

### Tags and Releases

**Version tagging:**

- Format: `v{VERSION}` (e.g., `v1.0.0`, `v2025.10.0`)
- Match VERSION file exactly
- Create GitHub releases for tags

## Validation Checklist

Use this checklist when creating or auditing a repository:

### Required Files

- [ ] README.md with all required sections
- [ ] VERSION file with valid version
- [ ] LICENSE with appropriate license
- [ ] CONTRIBUTING.md with version strategy documented
- [ ] CHANGELOG.md following Keep a Changelog format
- [ ] .gitignore appropriate for language
- [ ] Makefile with required Fulmen targets
- [ ] RELEASE_CHECKLIST.md present and aligned with the release checklist standard
- [ ] AGENTS.md outlining interfaces and startup protocol
- [ ] MAINTAINERS.md listing human + AI maintainers with canonical handles
- [ ] REPOSITORY_SAFETY_PROTOCOLS.md created or rationale documented elsewhere
- [ ] .goneat/tools.yaml manifest present and validated against tooling schema

### Documentation

- [ ] All markdown files have frontmatter
- [ ] docs/ directory exists with structure
- [ ] Architecture and design documented
- [ ] Contributing guidelines are clear

### Version Management

- [ ] VERSION file matches versioning strategy
- [ ] package.json version synced (if applicable)
- [ ] go.mod version synced (if applicable)
- [ ] Git tags match VERSION

### GitHub Integration

- [ ] CI/CD workflows present
- [ ] Issue templates configured
- [ ] PR template present
- [ ] Branch protection enabled

### Code Quality

- [ ] Coding standards followed
- [ ] Tests present and passing
- [ ] Linting configured
- [ ] Documentation complete

## Examples from Ecosystem

### CalVer Information Architecture

**Repository**: `fulmenhq/crucible`

- CalVer versioning (`2025.10.0`)
- Hybrid MIT/CC0 license
- Multi-language wrappers
- Schema and documentation SSOT

### SemVer Go Tool

**Repository**: `fulmenhq/goneat`

- SemVer versioning (`v0.2.11`)
- Apache 2.0 license
- Single-language Go project
- CLI tool with comprehensive docs

### SemVer TypeScript Service

**Repository**: `fulmenhq/brooklyn-mcp`

- SemVer versioning (`v0.2.0-rc.2`)
- MIT license
- TypeScript MCP server
- Enterprise-grade structure

## Related Standards and SOPs

- [Repository Versioning Standard](../standards/repository-versioning.md)
- [Repository Version Adoption SOP](repository-version-adoption.md)
- [Frontmatter Standard](../standards/frontmatter-standard.md)
- [Go Coding Standards](../standards/coding/go.md)
- [TypeScript Coding Standards](../standards/coding/typescript.md)
- [Agentic Attribution Standard](../standards/agentic-attribution.md)
- [Release Checklist Standard](../standards/release-checklist-standard.md)

## Maintenance

This SOP evolves with the FulmenHQ ecosystem. Updates are reviewed quarterly or as new patterns emerge.

Submit proposed changes via PR to `fulmenhq/crucible`.

---

**Status**: Approved  
**Last Updated**: 2025-10-02  
**Author**: @3leapsdave  
**Effective Date**: 2025-10-02
