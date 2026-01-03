---
title: "Repository Version Adoption SOP"
description: "Standard operating procedure for adopting version management in FulmenHQ repositories"
author: "@3leapsdave"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "approved"
tags: ["sop", "versioning", "process"]
---

# Repository Version Adoption SOP

## Policy

**All FulmenHQ repositories MUST adopt one of the standardized versioning strategies** defined in the [Repository Versioning Standard](../standards/repository-versioning.md).

This is a **mandatory requirement** for all repositories under the `fulmenhq` GitHub organization.

## Supported Strategies

Each repository must choose **one** of the following:

1. **Semantic Versioning (SemVer)** - `MAJOR.MINOR.PATCH`
2. **Calendar Versioning (CalVer)** - `YYYY.0M.MICRO`

See [Repository Versioning Standard](../standards/repository-versioning.md) for detailed guidance on which to choose.

## Implementation Requirements

### Mandatory Artifacts

Every FulmenHQ repository MUST have:

1. **VERSION file** in repository root
   - Single source of truth for version
   - Format depends on chosen strategy
   - No `v` prefix, just the version number

2. **Documentation of choice** in `CONTRIBUTING.md`:

   ```markdown
   ## Versioning

   This repository uses [SemVer|CalVer] for version management.
   See the [Repository Versioning Standard](https://github.com/fulmenhq/crucible/blob/main/docs/standards/repository-versioning.md).
   ```

3. **Version management scripts** (recommended pattern):
   - `scripts/version.ts` or equivalent
   - Package scripts for version bumps
   - Embedding scripts if needed

4. **Git tags** matching VERSION file:
   - Format: `v{VERSION}`
   - Created on each release

### Optional Enhancements

- CI/CD validation of version consistency
- Automated changelog generation
- Release automation scripts
- Version embedding in binaries/packages

## Adoption Checklist

Use this checklist when adopting version management in a repository:

### Initial Setup

- [ ] **Choose strategy**: Decide between SemVer or CalVer
- [ ] **Create VERSION file**: Add to repository root with initial version
- [ ] **Document choice**: Update `CONTRIBUTING.md` with version strategy
- [ ] **Implement scripts**: Add version management scripts
- [ ] **Update package files**: Sync `package.json`, `go.mod`, etc. with VERSION
- [ ] **Create git tag**: Tag current state with `v{VERSION}`
- [ ] **Update CI/CD**: Add version validation to pipelines

### Migration (Existing Repositories)

- [ ] **Audit current state**: Find all version references in codebase
- [ ] **Determine current version**: Establish baseline version
- [ ] **Create VERSION file**: Add with current version
- [ ] **Implement sync mechanism**: Ensure all version refs read from VERSION
- [ ] **Document migration**: Add note to CHANGELOG
- [ ] **Test build process**: Ensure version embedding works
- [ ] **Update CI/CD**: Add version consistency checks
- [ ] **Validate**: Check all version references are synchronized

### Validation

- [ ] **VERSION file exists** and contains valid version
- [ ] **Format matches strategy**: SemVer or CalVer format is correct
- [ ] **package.json synced** (if applicable)
- [ ] **go.mod synced** (if applicable)
- [ ] **Git tags present** and match VERSION
- [ ] **CI/CD validation** in place
- [ ] **Documentation complete** in CONTRIBUTING.md

## Strategy Selection Guide

### Use SemVer when:

✅ Repository is a **library or package** consumed by other code  
✅ Repository has a **public API** with clear compatibility boundaries  
✅ Repository is a **tool** where breaking changes are well-defined  
✅ Semantic versioning helps **dependency resolution**

**Examples**: `gofulmen`, `tsfulmen`, `goneat`, `fulward`

### Use CalVer when:

✅ Repository is **documentation or information architecture**
✅ Repository contains **schemas, standards, templates**
✅ **Temporal context** matters more than API compatibility
✅ "Breaking changes" are **hard to define** (e.g., doc rewording)

**Examples**: `fulmen-cosmography`

**Note**: Crucible originally used CalVer but transitioned to SemVer (v0.2.0) for Go module compatibility. See [ADR-0010](../architecture/decisions/ADR-0010-semantic-versioning-adoption.md).

## Version Management Workflow

### Standard Development Cycle

```bash
# 1. Feature development on branch
git checkout -b feature/new-capability

# 2. Commit changes (no version bump yet)
git commit -m "feat: add new capability"

# 3. Merge to main
git checkout main
git merge feature/new-capability

# 4. Bump version (choose appropriate bump)
bun run version:bump:patch   # or minor/major

# 5. Embed version (if applicable)
bun run version:embed

# 6. Commit version bump
git add VERSION package.json
git commit -m "chore: bump version to $(cat VERSION)"

# 7. Tag release
git tag "v$(cat VERSION)"

# 8. Push with tags
git push origin main --tags
```

### Hotfix Workflow

```bash
# 1. Fix critical issue
git checkout -b hotfix/critical-bug
# ... make fix ...
git commit -m "fix: critical bug in validation"

# 2. Merge to main
git checkout main
git merge hotfix/critical-bug

# 3. Bump patch/micro version
bun run version:bump:patch   # or version:set for CalVer

# 4. Tag and release
git tag "v$(cat VERSION)"
git push origin main --tags
```

## CI/CD Integration

### Required Validations

All FulmenHQ repositories should validate version consistency in CI:

```yaml
# .github/workflows/validate-version.yml
name: Validate Version

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Check VERSION file exists
        run: |
          if [ ! -f VERSION ]; then
            echo "❌ VERSION file missing"
            exit 1
          fi

      - name: Validate version format
        run: |
          VERSION=$(cat VERSION)
          # Add format validation based on strategy
          echo "Version: $VERSION"

      - name: Check version consistency
        run: |
          # Example: Check package.json matches VERSION
          VERSION=$(cat VERSION)
          PKG_VERSION=$(jq -r .version package.json)
          if [ "$VERSION" != "$PKG_VERSION" ]; then
            echo "❌ Version mismatch"
            exit 1
          fi
```

### Release Automation

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - "v*"

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Verify tag matches VERSION
        run: |
          VERSION=$(cat VERSION)
          TAG=${GITHUB_REF#refs/tags/v}
          if [ "$VERSION" != "$TAG" ]; then
            echo "❌ Tag $TAG does not match VERSION $VERSION"
            exit 1
          fi

      - name: Create GitHub Release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ github.ref }}
```

## Enforcement

### Repository Audit

The FulmenHQ organization performs periodic audits to ensure compliance:

- **Quarterly reviews** of all repositories
- **Automated checks** via GitHub Actions organization-wide workflows
- **New repository checklist** includes version adoption

### Non-Compliance

Repositories not following this SOP:

1. Will be flagged in organization audit reports
2. May be marked as "non-compliant" in repository metadata
3. Must be brought into compliance before next major release
4. May block promotion to "production-ready" status

### Exceptions

Exceptions to this policy require:

1. **Written justification** in repository README
2. **Approval** from @3leapsdave or designated architecture lead
3. **Documentation** of alternative approach
4. **Review** at each quarterly audit

Example valid exceptions:

- Experimental/prototype repositories (clearly marked)
- Archived repositories (no active development)
- Special-purpose repositories with domain-specific versioning needs

## Support and Resources

### Getting Help

- **Documentation**: [Repository Versioning Standard](../standards/repository-versioning.md)
- **Examples**: See `crucible` (CalVer) and `brooklyn-mcp` (SemVer)
- **Questions**: Open issue in `fulmenhq/crucible` with tag `question`

### Version Libraries

FulmenHQ provides version management libraries:

- **Go**: `github.com/fulmenhq/gofulmen/version`
- **TypeScript**: `@fulmenhq/tsfulmen/version`

Use these for consistent version handling across the ecosystem.

### Reference Implementations

**SemVer reference**: `fulmenhq/brooklyn-mcp`

- Script-based version management
- Static version embedding
- Full CI/CD validation

**SemVer reference (post-transition)**: `fulmenhq/crucible`

- VERSION file as SSOT
- Asset-level revision tracking
- Schema/doc coordination
- Transitioned from CalVer to SemVer per ADR-0010

## Review and Updates

This SOP is reviewed:

- **Quarterly** as part of organization standards review
- **On-demand** when versioning issues arise
- **With ecosystem growth** as new patterns emerge

Proposed changes should be submitted as PRs to `fulmenhq/crucible`.

---

**Status**: Approved  
**Last Updated**: 2025-10-02  
**Author**: @3leapsdave  
**Effective Date**: 2025-10-02
