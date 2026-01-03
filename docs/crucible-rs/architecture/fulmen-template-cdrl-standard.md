---
title: "Fulmen Template CDRL Standard"
description: "Architectural standard for Clone → Degit → Refit → Launch (CDRL) workflow compliance in Fulmen forge templates"
author: "Schema Cartographer (supervised by @3leapsdave)"
date: "2025-11-07"
last_updated: "2025-12-18"
status: "approved"
tags: ["architecture", "cdrl", "templates", "forge", "workhorse", "codex"]
---

# Fulmen Template CDRL Standard

## Status

**Approved** - 2025-11-07

## Overview

CDRL (Clone → Degit → Refit → Launch) is the canonical workflow for customizing Fulmen forge templates into production applications. This standard defines the architectural requirements templates MUST satisfy to be CDRL-ready, ensuring predictable, repeatable customization with minimal friction and maximum safety.

**Key Principle**: Customization should flow from a **single source of truth** (`.fulmen/app.yaml`) rather than scattered hardcoded values throughout the codebase.

**Important**: Fulmen forges are **working reference implementations**, not collections of `.template` files with placeholder strings. The template IS a buildable, runnable application that users customize by editing `.fulmen/app.yaml` and following the CDRL workflow. Think of `forge-workhorse-groningen` as the canonical example: it builds and runs as-is, with "groningen" throughout the codebase.

## Context

### Problem Statement

Early Fulmen templates (pre-CDRL) required error-prone manual find-replace across dozens of files:

- Binary names hardcoded in CLI help, HTTP responses, logging, metrics
- Environment variable prefixes scattered in code
- Configuration paths duplicated throughout
- No validation to detect incomplete customization

**Result**: High error rates, inconsistent deployments, poor developer experience.

### CDRL Solution

Centralize customization in **App Identity Module** (`.fulmen/app.yaml`), provide validation tooling, and standardize refit workflow across all forge types.

### Proof of Concept

**forge-workhorse-groningen** successfully prototyped CDRL with full App Identity integration. See [Groningen CDRL Guide](https://github.com/fulmenhq/forge-workhorse-groningen/blob/main/docs/development/fulmen_cdrl_guide.md) for reference implementation.

## Scope

This standard applies to:

- **Workhorse Templates**: `forge-workhorse-*` (Groningen, Percheron, Sumpter, etc.)
- **Codex Templates**: `forge-codex-*` (Aurora, future documentation sites)
- **Future Template Categories**: Gymnasium experiments, specialized forks

Forges MUST comply with this standard to be considered production-ready templates.

## CDRL Workflow Definition

### Four Stages

**1. Clone** - Acquire template

```bash
git clone https://github.com/fulmenhq/forge-workhorse-groningen.git my-app
cd my-app
```

**2. Degit** - Remove template history

```bash
rm -rf .git
git init
```

**3. Refit** - Customize for application

```bash
# PRIMARY: Edit app identity
vim .fulmen/app.yaml

# VALIDATION: Detect incomplete refit
make validate-app-identity

# SECONDARY: Update business logic, docs, metadata
```

**4. Launch** - Run customized application

```bash
make bootstrap
make run
```

### Workflow Characteristics

- **Fast**: < 10 minutes from clone to running custom app
- **Safe**: Validation catches incomplete customization
- **Predictable**: Same workflow across all forge types
- **Repeatable**: Documented, testable, automatable

## Required Template Capabilities

### 1. App Identity Module (REQUIRED)

Templates MUST implement [App Identity Module](../standards/library/modules/app-identity.md) with `.fulmen/app.yaml` as the customization SSOT.

**Minimum Fields**:

```yaml
vendor: fulmenhq # Organization namespace
binary_name: groningen # Application identifier (breed name for templates)
service_type: workhorse # Template category (workhorse | codex | gymnasium)
env_prefix: GRONINGEN_ # Environment variable prefix
config_name: groningen # Configuration file/directory name
description: "HTTP API workhorse for production services"
version: 1.0.0 # Application version (independent of Crucible)
```

**Compliance**:

- Helper library MUST load identity via `app_identity.load()`
- All parameterization points MUST derive from identity fields
- No hardcoded breed/site names in codebase (except `.fulmen/app.yaml`)
- **Distributed artifacts MUST embed identity** and support the embedded fallback behavior defined in the [App Identity Module](../standards/library/modules/app-identity.md) standard

**Why This Matters**: Single-file customization reduces refit errors from ~30% to <5% (observed in Groningen pilot).

### 2. Parameterization Points (REQUIRED)

These are NOT string literals to find-replace. Instead, they are the **locations where the template's breed/site name naturally appears** in working code. The CDRL workflow treats them as:

1. **Edit `.fulmen/app.yaml` first** (single source of truth)
2. **Run `make validate-app-identity`** (finds hardcoded breed name)
3. **Update identified locations** (module path, imports, directory names)

Templates MUST document these mandatory refit points:

**Workhorse Templates**:

- **Binary Name**: CLI help, HTTP responses, process names, telemetry service names
- **Module Path**: `go.mod` module directive, Python package names, TypeScript package names
- **Environment Variables**: All env vars MUST use `{ENV_PREFIX}*` pattern from identity
- **Configuration Paths**: Layer 2 config at `~/.config/{vendor}/{config_name}/config.yaml`
- **Telemetry Namespaces**: Logging service names, metric prefixes, trace service names

**Codex Templates**:

- **Site Name**: HTML title, navigation, footer, meta tags
- **Canonical URL**: `site_url` in Layer 2 config
- **Analytics**: Provider IDs, tracking codes
- **Branding**: Favicon, logos, theme colors (reference identity or Layer 2 config)

**Documentation Requirement**: Each template MUST maintain a CDRL guide (see [Workflow Guide](../standards/cdrl/workflow-guide.md)) listing all parameterization points.

### 3. Validation Targets (REQUIRED)

Templates MUST provide Makefile targets for CDRL validation:

#### `make validate-app-identity`

**Purpose**: Detect hardcoded breed/site names in codebase

**Behavior**:

- Scan source files for breed name strings (e.g., `groningen`, `percheron`)
- Exclude `.fulmen/app.yaml`, `README.md`, `CHANGELOG.md`, docs
- Report files containing hardcoded references
- Exit code 1 if violations found, 0 if clean

**Example Implementation** (pseudocode):

```bash
grep -r "groningen" --exclude-dir=".fulmen" --exclude="*.md" src/
```

#### `make sync-embedded-identity`

**Purpose**: Create/update the embedded identity mirror for distributed artifacts

**Behavior**:

- Copy `.fulmen/app.yaml` → an embeddable mirror path (language-specific)
- Fail if `.fulmen/app.yaml` is missing
- Do not mutate formatting/content (byte-for-byte intent)

#### `make verify-embedded-identity`

**Purpose**: Prevent identity drift between source and embedded mirror

**Behavior**:

- Fail if `.fulmen/app.yaml` differs from the embedded mirror
- SHOULD run as part of `make test`, `make precommit`, and CI

#### Acceptance Criteria (App/Template Layer)

- **AC1 — Standalone binary works outside repo**
  - `make build`
  - `cp ./bin/<tool> /tmp/<tool>`
  - `/tmp/<tool> version`
  - Must succeed without any `.fulmen/app.yaml` on disk.
- **AC2 — `--help` must not depend on external identity**
  - same copy-to-`/tmp` pattern, run `/tmp/<tool> --help`
- **AC3 — Drift prevention**
  - `.fulmen/app.yaml` and the embedded mirror must be identical
  - `make verify-embedded-identity` must fail when they differ

#### `make doctor` (or `make validate-cdrl-ready`)

**Purpose**: Comprehensive refit completeness check

**Checks**:

- App identity file exists and validates
- Environment variable prefix consistency
- Configuration paths match identity
- Module path updated (language-specific)
- README customized (no template placeholders)
- Tests pass

**Exit Codes**:

- 0: CDRL refit complete
- 1: Warnings (non-blocking issues)
- 2: Errors (blocking issues)

### 4. Standard Directory Structure (REQUIRED)

#### Template Files: Not `.template` Suffixes

Fulmen forges do NOT use `.template` file suffixes (e.g., `app.yaml.template`, `go.mod.template`). Instead:

- ✅ **Ship working files**: `go.mod` with real module path, `.fulmen/app.yaml` with breed/site name
- ✅ **Users edit in place**: During CDRL refit, users modify these working files
- ✅ **Validation catches missed renames**: `make validate-app-identity` finds hardcoded references

**Why**: Templates must be buildable and testable in CI. Using `.template` files would prevent builds and IDE type-checking.

**Example - Groningen Pattern (Correct)**:

```
forge-workhorse-groningen/
├── .fulmen/app.yaml          # binary_name: groningen (users edit to: myapi)
├── go.mod                    # module github.com/fulmenhq/forge-workhorse-groningen
├── cmd/groningen/main.go     # Real working code
```

**Anti-Pattern (Avoid)**:

```
forge-workhorse-groningen/
├── .fulmen/app.yaml.template    # ❌ Not buildable
├── go.mod.template              # ❌ IDE can't check
├── cmd/{{BREED}}/main.go        # ❌ Won't compile
```

#### Required Directory Structure

```
forge-*/
├── .fulmen/                          # Fulmen metadata (REQUIRED)
│   └── app.yaml                      # App Identity config (REQUIRED)
├── docs/
│   ├── development/
│   │   ├── README.md                 # Development overview (RECOMMENDED)
│   │   ├── fulmen_cdrl_guide.md      # Template-specific CDRL guide (REQUIRED)
│   │   ├── tooling-guide.md          # DX tools, goneat, hooks (OPTIONAL)
│   │   └── customization-advanced.md # Post-refit patterns (OPTIONAL)
│   └── architecture/                 # Template architecture docs
├── config/
│   └── {breed}.yaml                  # Default config (users rename during refit)
├── .env.example                      # Environment variable template (REQUIRED)
├── Makefile                          # MUST include CDRL validation targets
├── README.md                         # MUST document CDRL workflow
├── MAINTAINERS.md                    # Template maintainers
├── LICENSE                           # Template license
├── scripts/
│   └── bootstrap.py                  # Tool installation script (RECOMMENDED)
└── src/ (or internal/)               # Application source
```

### 5. Bootstrap Script (RECOMMENDED)

Templates SHOULD provide a `scripts/bootstrap.{py,ts,sh}` script that installs development tools locally.

**Purpose**: Reproducible local development environments through version-pinned tool installation.

**Pattern**: Manifest-driven bootstrap reads `.goneat/tools.yaml` and installs tools (goneat, etc.) to `bin/` directory.

**Key Features**:

- Version pinning for reproducibility
- SHA256 checksum verification for security
- Platform-aware (darwin/linux/windows, amd64/arm64)
- Local dev override support (`.goneat/tools.local.yaml`)

**Language-Specific Guidance**:

- **Python templates**: Implement `scripts/bootstrap.py` (see percheron canonical)
- **TypeScript templates**: Implement `scripts/bootstrap.ts` (see aurora canonical)
- **Go templates**: Use native Go dependency management (no bootstrap needed)

**Makefile Integration**:

```makefile
.PHONY: bootstrap
bootstrap:  ## Install dependencies and development tools
	@uv run python scripts/bootstrap.py  # Or: bun run scripts/bootstrap.ts
```

**Reference**: See [Reference Bootstrap Pattern](../standards/cdrl/reference-bootstrap.md) for complete implementation guidance, manifest schema, and canonical implementations.

### 6. Documentation Requirements (REQUIRED)

Each template MUST include:

**README.md**:

- CDRL quick start (< 5 minutes to custom app)
- Link to Crucible CDRL standards
- Template-specific customization notes

**docs/development/fulmen_cdrl_guide.md** (REQUIRED - PRIMARY CDRL DOCUMENTATION):

Must follow this canonical outline structure:

```markdown
# [Template Name] CDRL Guide

## Overview

- What is this template (workhorse/codex, language, key capabilities)
- What is CDRL (Clone → Degit → Refit → Launch)
- Link to Crucible CDRL standard and workflow guide

## Prerequisites

- Runtime requirements (Go, Python, Node/Bun, etc.)
- Build tools (make, language-specific package managers)
- Recommended development environment

## CDRL Workflow

### Step 1: Clone

- Command to clone template
- What you get after cloning

### Step 2: Degit

- Remove template git history
- Initialize fresh repository

### Step 3: Refit (PRIMARY SECTION)

#### 3.1 Update App Identity (REQUIRED FIRST STEP)

- Edit `.fulmen/app.yaml`
- Explain each field (vendor, binary_name, env_prefix, config_name)

#### 3.2 Validate App Identity

- Run `make validate-app-identity`
- Expected output (success/failure examples)

#### 3.3 Update Module Path

- Language-specific: go.mod, pyproject.toml, package.json
- Show before/after examples

#### 3.4 Update Environment Variables

- Rename prefix in `.env.example` → `.env`
- List all standard env vars for this template

#### 3.5 Rename Configuration Files

- Which files need renaming (e.g., `config/groningen.yaml` → `config/myapi.yaml`)

#### 3.6 Customize Business Logic (OPTIONAL)

- Remove template placeholders
- Add your application code

#### 3.7 Update Documentation

- README.md, LICENSE, MAINTAINERS.md

#### 3.8 Final Validation

- Run `make doctor`
- Expected output

### Step 4: Launch

- `make bootstrap`
- `make run` or `make build`
- Verify customization (version endpoint, logs, etc.)

## Parameterization Reference

### All Parameterization Points

Table listing every place the template identity appears:
| Location | Before (Template) | After (Your App) | Required |
|----------|-------------------|------------------|----------|
| `.fulmen/app.yaml` | `binary_name: groningen` | `binary_name: myapi` | Yes |
| `go.mod` | `module github.com/fulmenhq/...` | `module github.com/me/myapi` | Yes |
| ... | ... | ... | ... |

## Verification Checklist

- [ ] App Identity loads correctly
- [ ] Environment variables use correct prefix
- [ ] Config files renamed
- [ ] Module path updated
- [ ] `make validate-app-identity` passes
- [ ] `make doctor` passes
- [ ] `make test` passes
- [ ] Application runs with custom identity

## Common Customization Patterns (RECOMMENDED)

- Web API service example
- Background worker example
- CLI tool example
- (Template-specific patterns)

## Troubleshooting (RECOMMENDED)

- App Identity not loading
- Environment variables not working
- Config file not found
- Validation failures
- (Template-specific issues)

## Next Steps (RECOMMENDED)

- Where to add features
- How to deploy
- Link to advanced customization docs
- Link to tooling guide (goneat, hooks, DX)
```

**Template-Specific Supplementary Documentation** (OPTIONAL):

Templates MAY provide additional development guides in `docs/development/`:

- **`tooling-guide.md`** - DX tools (goneat, hooks, make targets, switching approaches)
- **`customization-advanced.md`** - Post-refit feature additions, architectural patterns
- **`distribution-options.md`** - Packaging strategies, deployment, release workflows
- **`README.md`** - Development overview, entry point directing to `fulmen_cdrl_guide.md`

**MAINTAINERS.md**:

- Template stewards (responsible for CDRL compliance)
- Contribution guidelines
- Support channels

## Forge-Specific Variations

### Workhorse Templates

**Primary Refit Points**:

1. Binary name → CLI commands, HTTP server, metrics
2. Module path → `go.mod`, `package.json`, `pyproject.toml`
3. Environment prefix → All `{PREFIX}_*` variables
4. Config files → Rename `config/{breed}.yaml` to `config/{app}.yaml`

**Validation Focus**:

- Server starts with custom identity
- `/version` endpoint reflects custom app
- Logs use custom service name
- Metrics use custom prefixes

**Example**: Groningen → User Service

```yaml
# .fulmen/app.yaml
vendor: acme
binary_name: user-service
env_prefix: USERSVC_
config_name: user-service
```

### Codex Templates

**Primary Refit Points**:

1. Site name → HTML, navigation, meta tags
2. Site URL → Canonical links, sitemap
3. Analytics → Provider IDs, tracking codes
4. Branding → Assets, theme, colors

**Validation Focus**:

- Site builds with custom branding
- Internal links resolve correctly
- Analytics configured correctly
- Search indexes custom content

**Example**: Aurora → Company Docs

```yaml
# .fulmen/app.yaml
vendor: acme
binary_name: acme-docs
config_name: site
description: "Acme Corporation documentation portal"
```

## Quality Gates

### Template CDRL Readiness Checklist

Templates MUST pass all checks before being considered production-ready:

- [ ] **App Identity Module implemented** per [module spec](../standards/library/modules/app-identity.md)
- [ ] **Parameterization points documented** in CDRL guide
- [ ] **Validation targets present**: `make validate-app-identity` and `make doctor`
- [ ] **CDRL guide present** at `docs/development/cdrl-guide.md`
- [ ] **Example customization succeeds** (maintainers MUST test full CDRL workflow)
- [ ] **No hardcoded breed/site names** in source code (validation passes)
- [ ] **Environment variables use prefix** from App Identity
- [ ] **Configuration paths derive** from App Identity
- [ ] **README documents CDRL** with quick start
- [ ] **Tests pass** after example refit

### Compliance Verification

Crucible maintainers MAY audit templates for CDRL compliance:

```bash
# Clone template
git clone https://github.com/fulmenhq/forge-workhorse-{breed}.git audit
cd audit

# Verify structure
test -f .fulmen/app.yaml || echo "FAIL: Missing App Identity"
test -f docs/development/*cdrl*.md || echo "FAIL: Missing CDRL guide"

# Verify Makefile targets
make validate-app-identity || echo "FAIL: No validation target"
make doctor || echo "FAIL: No doctor target"

# Test example refit
sed -i 's/breed/testapp/g' .fulmen/app.yaml
make validate-app-identity  # Should detect hardcoded "breed" references
```

## Working Implementation Pattern

Fulmen forges follow the **working implementation pattern**:

### Build Requirements

- **MUST compile/build as-is**: `go build`, `npm run build`, `cargo build` succeeds
- **MUST pass tests as-is**: `make test` succeeds without customization
- **MUST run as-is**: Binary executes with `--help`, `--version` working

### IDE Experience

- **Syntax highlighting works**: No placeholder syntax breaking parsers
- **Type checking works**: IDEs can validate imports and types
- **Auto-complete works**: Developers get full IDE support when refitting

### CI/CD Requirements

- **Template repositories run CI**: Linting, testing, building validate template quality
- **Quality gates enforce standards**: Pre-commit hooks, format checks, security scans

### Comparison: Template Repository vs Template Files

| Aspect      | Fulmen Forge (✅)    | Literal Templates (❌)                 |
| ----------- | -------------------- | -------------------------------------- |
| **Files**   | `go.mod`, `app.yaml` | `go.mod.template`, `app.yaml.template` |
| **Build**   | Builds immediately   | Requires preprocessing                 |
| **IDE**     | Full type checking   | No validation                          |
| **CI**      | Can run tests        | Cannot test                            |
| **Pattern** | Groningen, Percheron | Cookiecutter, Yeoman                   |

**Reference**: See `forge-workhorse-groningen` as the canonical example of this pattern.

## Anti-Patterns

### ❌ Hardcoded Identifiers

```go
// BAD: Hardcoded breed name
func main() {
    log.Info("Starting groningen server...")
}

// GOOD: Load from App Identity
func main() {
    identity := appIdentity.Load()
    log.Info("Starting %s server...", identity.BinaryName)
}
```

### ❌ Manual Find-Replace Instructions

```markdown
<!-- BAD: Error-prone manual steps -->

1. Find all occurrences of "groningen" and replace with your app name
2. Update go.mod module path
3. Search for "GRONINGEN\_" and update env vars
```

### ❌ No Validation

```bash
# BAD: User has no way to verify refit completeness
make run  # May fail mysteriously if refit incomplete
```

### ❌ Scattered Configuration

```yaml
# BAD: Identity spread across multiple files
# config/server.yaml
app_name: groningen

# config/logging.yaml
service_name: groningen

# config/metrics.yaml
prefix: groningen_
```

## Integration with Other Standards

### Workhorse Standard

[Fulmen Forge Workhorse Standard](fulmen-forge-workhorse-standard.md) requires App Identity Module and CDRL compliance.

### Codex Standard

[Fulmen Forge Codex Standard](fulmen-forge-codex-standard.md) requires App Identity Module and CDRL compliance.

### Makefile Standard

[Makefile Standard](../standards/makefile-standard.md) defines CDRL validation targets for template repositories.

### Repository Categories

[Repository Categories Taxonomy](../../config/taxonomy/repository-categories.yaml) defines `forge-workhorse`, `forge-codex`, and `forge-gymnasium` categories with CDRL requirements.

## Implementation Guidance

### For Template Authors

**When creating a new forge**:

1. Implement App Identity Module first (`.fulmen/app.yaml`)
2. Use helper library's `app_identity.load()` everywhere
3. Add validation targets to Makefile
4. Write CDRL guide documenting refit points
5. Test full CDRL workflow with example customization
6. Submit for Crucible review

**When updating existing forge**:

1. Add `.fulmen/app.yaml` with current breed as default
2. Refactor hardcoded identifiers to use App Identity
3. Add validation targets
4. Update README with CDRL quick start
5. Write CDRL guide
6. Increment template version

### For Template Users

**When customizing a template**:

1. Follow template's CDRL guide (linked from README)
2. Edit `.fulmen/app.yaml` as primary customization
3. Run `make validate-app-identity` to detect missed references
4. Complete secondary customization (business logic, docs)
5. Run `make doctor` for comprehensive validation
6. Launch with `make bootstrap && make run`

See [CDRL Workflow Guide](../standards/cdrl/workflow-guide.md) for detailed step-by-step instructions.

## Future Enhancements

### Planned (v0.3.0+)

- **Automated Refit Script**: `make cdrl-refit` prompts for identity fields and updates files
- **CDRL Testing Framework**: Automated compliance testing for templates
- **Multi-Template Refit**: Merge features from multiple forge templates
- **CDRL Catalog**: Registry of CDRL-ready templates with compliance badges

### Under Consideration

- **Helper Library CDRL Module**: Shared validation logic for all templates
- **Interactive Refit Tool**: TUI for guided CDRL workflow
- **Template Versioning**: CalVer/SemVer coordination between template and Crucible

## References

- [CDRL Workflow Guide](../standards/cdrl/workflow-guide.md) - Step-by-step user instructions
- [App Identity Module Spec](../standards/library/modules/app-identity.md) - Technical specification
- [Fulmen Forge Workhorse Standard](fulmen-forge-workhorse-standard.md) - Workhorse template requirements
- [Fulmen Forge Codex Standard](fulmen-forge-codex-standard.md) - Codex template requirements
- [Makefile Standard](../standards/makefile-standard.md) - CDRL validation targets
- [Repository Categories Taxonomy](../../config/taxonomy/repository-categories.yaml) - Template category definitions
- [Groningen CDRL Guide](https://github.com/fulmenhq/forge-workhorse-groningen/blob/main/docs/development/fulmen_cdrl_guide.md) - Reference implementation

## Revision History

| Version | Date       | Changes                                                                  |
| ------- | ---------- | ------------------------------------------------------------------------ |
| 1.0.0   | 2025-11-07 | Initial standard based on Groningen prototype and v0.2.8 forge standards |

---

**Maintainers**: Schema Cartographer, @3leapsdave
**Status**: Approved for v0.2.8 release
**Next Review**: 2025-12-07 (30 days)
