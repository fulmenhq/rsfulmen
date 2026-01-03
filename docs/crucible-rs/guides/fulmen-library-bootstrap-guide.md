---
title: "Fulmen Library Bootstrap Guide"
description: "Bootstrap a new Fulmen ecosystem library using goneat and Crucible"
author: "Schema Cartographer"
author_of_record: "Dave Thompson (https://github.com/3leapsdave)"
supervised_by: "@3leapsdave"
date: "2025-10-08"
last_updated: "2025-10-28"
status: "active"
tags: ["bootstrap", "goneat", "ssot", "setup", "new-repository", "library"]
---

# Fulmen Library Bootstrap Guide

This guide walks you through creating a new Fulmen ecosystem library (Python, Go, TypeScript, or other languages) that syncs standards and schemas from Crucible.

**Goal**: Get from empty repository to working library with Crucible integration in under 10 minutes.

**Audience**: Developers creating new libraries like `pyfulmen`, `gofulmen`, `tsfulmen`, or custom language-specific helpers.

## Philosophy

Fulmen libraries follow a consistent pattern:

1. **Goneat for tooling** - SSOT sync, version management, formatting, quality gates
2. **Crucible for standards** - Schemas, coding standards, documentation
3. **Language-specific conventions** - Idiomatic testing, linting, building
4. **Makefile Standard compliance** - Required targets for quality gates

## Prerequisites

- Git repository created on GitHub
- Local clone of the repository
- Go 1.21+ installed (to install goneat)
- Command line access (bash/zsh)
- (Optional) Local Crucible clone at `../crucible` for faster development

## Installation: Goneat

Choose one of these methods:

### Method A: System Install (Recommended for Production)

```bash
# Install goneat globally
go install github.com/fulmenhq/goneat/cmd/goneat@latest

# Verify
goneat version
```

**Pros**: Simple, works everywhere, easy updates  
**Cons**: Team members might have different versions

### Method B: Local Development Link

```bash
# Clone goneat repo (sibling to your library)
git clone https://github.com/fulmenhq/goneat.git ../goneat
cd ../goneat
make build  # Creates dist/goneat

# Link in your library repo
cd ../your-library
ln -s ../goneat/dist/goneat goneat

# Use via local link
./goneat version
```

**Pros**: Test goneat changes locally  
**Cons**: Requires goneat repo checkout

### Method C: Repository-Managed Bootstrap (Advanced)

Go and TypeScript libraries can include bootstrap tooling that installs goneat to `./bin/`.

See language-specific sections below for details.

## Quick Start

```bash
# 1. Install goneat (Method A or B)
go install github.com/fulmenhq/goneat/cmd/goneat@latest

# 2. Create repository structure
mkdir -p .goneat src tests docs
echo "0.1.0" > VERSION

# 3. Configure SSOT sync
cat > .goneat/ssot-consumer.yaml <<EOF
version: 1.0
sources:
  - name: crucible
    repo: fulmenhq/crucible
    ref: main
    keys:
      - crucible.schemas.*
      - crucible.docs.standards
      - crucible.docs.guides
EOF

# 4. (Optional) Local override for development
cat > .goneat/ssot-consumer.local.yaml <<EOF
version: 1.0
sources:
  - name: crucible
    localPath: ../crucible
EOF

# 5. Sync Crucible assets
goneat ssot sync

# 6. Verify
cat .goneat/ssot/provenance.json
ls .crucible/
```

## Directory Structure

After bootstrap, your repository should look like:

```
your-library/
├── .goneat/
│   ├── ssot-consumer.yaml        # SSOT sync configuration
│   ├── ssot-consumer.local.yaml  # Local overrides (gitignored)
│   ├── tools.yaml                # Tool definitions (optional)
│   ├── hooks.yaml                # Git hooks configuration (optional)
│   ├── version-policy.yaml       # Version management policy (optional)
│   └── ssot/
│       └── provenance.json       # Sync metadata (generated)
├── .crucible/
│   └── metadata/
│       ├── sync-keys.yaml        # Available sync keys
│       └── metadata.yaml         # Source metadata
├── schemas/crucible-{lang}/      # Synced schemas
├── docs/crucible-{lang}/         # Synced documentation
├── config/crucible-{lang}/       # Synced config defaults
├── src/                          # Your library code
├── tests/                        # Your tests
├── VERSION                       # Version file (0.1.0)
├── Makefile                      # Standard targets
├── .gitignore                    # Ignore patterns
└── README.md                     # Documentation
```

## SSOT Sync Configuration

### Basic Configuration (`.goneat/ssot-consumer.yaml`)

```yaml
version: 1.0
sources:
  - name: crucible
    repo: fulmenhq/crucible
    ref: main # Or specific tag like v2025.10.3
    keys:
      # Standards and guides (all libraries should sync these)
      - crucible.docs.standards
      - crucible.docs.guides
      - crucible.docs.sop

      # Schemas (choose what you need)
      - crucible.schemas.observability.logging
      - crucible.schemas.pathfinder
      - crucible.schemas.terminal

      # Config defaults
      - crucible.config.library
```

**Available Keys**: Run `goneat ssot keys` to see all registered sync keys.

### Local Development Override (`.goneat/ssot-consumer.local.yaml`)

```yaml
version: 1.0
sources:
  - name: crucible
    localPath: ../crucible # Points to local clone
```

**Important**:

- This file is **gitignored** (never commit)
- Overrides only `localPath` from main config
- Each developer can point to their own Crucible clone
- Syncs from filesystem (instant) instead of GitHub API

### Version Pinning

Pin to specific Crucible version in production:

```yaml
version: 1.0
sources:
  - name: crucible
    repo: fulmenhq/crucible
    ref: v2025.10.3 # Specific tag instead of 'main'
    keys:
      - crucible.docs.standards
```

## Makefile Standard

All Fulmen libraries **must** implement these targets:

```makefile
# Fulmen Library Makefile
# Repository: $(notdir $(CURDIR))

.PHONY: help
help: ## Show available targets
	@echo "Required targets:"
	@echo "  bootstrap    - Install dependencies and tools"
	@echo "  tools        - Verify external tools available"
	@echo "  sync         - Sync Crucible assets"
	@echo "  lint         - Run linting"
	@echo "  fmt          - Format code"
	@echo "  test         - Run tests"
	@echo "  build        - Build artifacts"
	@echo "  check-all    - Run all quality checks"
	@echo "  precommit    - Pre-commit quality gates"
	@echo "  prepush      - Pre-push quality gates"
	@echo "  clean        - Remove build artifacts"

.PHONY: tools
tools: ## Verify goneat is available
	@command -v goneat >/dev/null 2>&1 || { \
		echo "❌ goneat not found"; \
		echo "Install: go install github.com/fulmenhq/goneat/cmd/goneat@latest"; \
		exit 1; \
	}
	@echo "✅ goneat: $$(goneat version 2>&1 | head -1)"

.PHONY: bootstrap
bootstrap: tools ## Bootstrap development environment
	@echo "Bootstrapping..."
	# Add language-specific commands
	@$(MAKE) sync
	@echo "✅ Bootstrap complete"

.PHONY: sync
sync: ## Sync Crucible assets
	@goneat ssot sync
	@echo "✅ Crucible synced"

.PHONY: fmt
fmt: ## Format code
	@goneat format
	@echo "✅ Code formatted"

.PHONY: lint
lint: ## Run linting
	@goneat assess --categories lint
	# Add language-specific linting
	@echo "✅ Linting passed"

.PHONY: test
test: ## Run tests
	# Add language-specific test runner
	@echo "✅ Tests passed"

.PHONY: build
build: sync ## Build artifacts
	# Add language-specific build commands
	@echo "✅ Build complete"

.PHONY: check-all
check-all: build fmt lint test ## Run all quality checks
	@echo "✅ All checks passed"

.PHONY: precommit
precommit: fmt lint ## Pre-commit hooks
	@echo "✅ Pre-commit checks passed"

.PHONY: prepush
prepush: check-all ## Pre-push hooks
	@echo "✅ Pre-push checks passed"

.PHONY: clean
clean: ## Clean build artifacts
	@rm -rf dist/ build/ bin/
	@echo "✅ Clean complete"
```

## Gitignore Patterns

Add to `.gitignore`:

```gitignore
# Goneat local overrides (development only)
.goneat/*.local.yaml
.goneat/*.local.json
.goneat/ssot/provenance.json  # Generated during sync

# Local goneat link (if using Method B)
/goneat

# Crucible synced assets (regenerated from SSOT)
/.crucible/
!/.crucible/metadata/
/schemas/crucible-*/
/docs/crucible-*/
/config/crucible-*/

# Build artifacts
/dist/
/build/
/bin/
*.egg-info

# Language-specific
__pycache__/
*.pyc
.pytest_cache/
node_modules/
.coverage
```

## Language-Specific Setup

### Python

**File**: `pyproject.toml`

```toml
[project]
name = "your-library"
version = "0.1.0"
description = "Description"
requires-python = ">=3.11"
dependencies = []

[project.optional-dependencies]
dev = [
    "pytest>=8.0.0",
    "ruff>=0.1.0",
]

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.ruff]
line-length = 100
target-version = "py311"
```

**Makefile additions**:

```makefile
.PHONY: bootstrap
bootstrap: tools
	@pip install -e .[dev]
	@$(MAKE) sync

.PHONY: lint
lint:
	@ruff check src/ tests/
	@goneat assess --categories lint

.PHONY: test
test:
	@pytest tests/ -v

.PHONY: build
build: sync
	@python -m build
```

**See pyfulmen for complete reference implementation.**

### Go

**File**: `go.mod`

```go
module github.com/fulmenhq/your-library

go 1.21

require (
    // Dependencies
)
```

**Makefile additions**:

```makefile
.PHONY: bootstrap
bootstrap: tools
	@go mod tidy
	@$(MAKE) sync

.PHONY: fmt
fmt:
	@gofmt -w .
	@goneat format

.PHONY: lint
lint:
	@go vet ./...
	@goneat assess --categories lint

.PHONY: test
test:
	@go test ./... -v

.PHONY: build
build: sync
	@go build ./...
```

**Bootstrap Package Pattern** (optional):

Gofulmen includes `cmd/bootstrap` that manages tool installation:

```bash
# Bootstrap installs tools from .goneat/tools.yaml to ./bin/
make bootstrap
# Runs: go run ./cmd/bootstrap --install --verbose
```

**See gofulmen for complete reference implementation.**

### TypeScript

**File**: `package.json`

```json
{
  "name": "your-library",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "build": "tsup",
    "test": "vitest run",
    "typecheck": "tsc --noEmit"
  },
  "devDependencies": {
    "@biomejs/biome": "^2.2.5",
    "tsup": "^8.0.0",
    "typescript": "^5.7.0",
    "vitest": "^2.1.0"
  }
}
```

**Makefile additions**:

```makefile
.PHONY: bootstrap
bootstrap: tools
	@bun install
	@$(MAKE) sync

.PHONY: fmt
fmt:
	@bunx biome format --write .
	@goneat format

.PHONY: lint
lint:
	@bunx biome check .
	@goneat assess --categories lint

.PHONY: typecheck
typecheck:
	@bunx tsc --noEmit

.PHONY: test
test:
	@bunx vitest run

.PHONY: check-all
check-all: build fmt lint typecheck test
	@echo "✅ All checks passed"
```

**Bootstrap Script Pattern** (optional):

TSFulmen includes `scripts/bootstrap-tools.ts` using Bun runtime:

```bash
# Bootstrap installs tools from .goneat/tools.yaml to ./bin/
make bootstrap
# Runs: bun run scripts/bootstrap-tools.ts --install --verbose
```

**See tsfulmen for complete reference implementation.**

## Provenance Tracking

Goneat automatically records provenance when syncing SSOT sources.

After `goneat ssot sync`, check `.goneat/ssot/provenance.json`:

```json
{
  "schema": {
    "name": "goneat.ssot.provenance",
    "version": "v1"
  },
  "generated_at": "2025-10-28T18:59:49Z",
  "sources": [
    {
      "name": "crucible",
      "method": "local_path",
      "repo_url": "https://github.com/fulmenhq/crucible",
      "local_path": "../crucible",
      "ref": "main",
      "commit": "14540342ad9defcedcd4a2f9c6a167bd10f7cb5d",
      "dirty": false,
      "version": "2025.10.3",
      "outputs": {
        "schema": "schemas/crucible-py",
        "doc": "docs/crucible-py",
        "config": "config/crucible-py",
        "metadata": ".crucible/metadata"
      }
    }
  ]
}
```

**Use Cases**:

- Verify which Crucible version is synced
- Audit SSOT integrity in CI/CD
- Debug sync issues (check commit hash, dirty status)
- Document dependencies in release notes

## Bootstrap Checklist

- [ ] Repository created and cloned
- [ ] Goneat installed (verify with `goneat version`)
- [ ] `VERSION` file created (0.1.0)
- [ ] `.goneat/ssot-consumer.yaml` configured
- [ ] (Optional) `.goneat/ssot-consumer.local.yaml` for local development
- [ ] `Makefile` created with all required targets
- [ ] `.gitignore` updated
- [ ] Language-specific files created
- [ ] `make bootstrap` runs successfully
- [ ] `make sync` runs successfully
- [ ] `.goneat/ssot/provenance.json` exists and is valid
- [ ] `make check-all` passes
- [ ] Initial commit pushed

## Common Workflows

### Version Management

```bash
# Bump patch (0.1.0 → 0.1.1)
goneat version bump patch

# Bump minor (0.1.1 → 0.2.0)
goneat version bump minor

# Bump major (0.2.0 → 1.0.0)
goneat version bump major

# Propagate VERSION to package manifests
goneat version propagate
```

### Sync Workflows

```bash
# Sync from GitHub (uses .goneat/ssot-consumer.yaml)
make sync

# Sync from local Crucible (uses .local.yaml if present)
make sync

# Check available sync keys
goneat ssot keys
```

### Quality Gates

```bash
# Format code
make fmt

# Run linting
make lint

# Run tests
make test

# Run all checks (required before commit)
make check-all

# Pre-commit checks
make precommit

# Pre-push checks (comprehensive)
make prepush
```

## Advanced Topics

### Tools Manifest (`.goneat/tools.yaml`)

Define external tools for bootstrap automation:

```yaml
version: v0.3.0
binDir: ./bin
tools:
  - id: goneat
    description: Fulmen schema validation and automation CLI
    required: true
    install:
      type: download
      url: https://github.com/fulmenhq/goneat/releases/download/v0.3.0/goneat-{{os}}-{{arch}}
      binName: goneat
      destination: ./bin
      checksum:
        darwin-arm64: "sha256-hash-here"
```

Bootstrap scripts can read this manifest to install tools locally.

### Local Overrides (`.local.yaml` pattern)

Goneat supports `.local.yaml` overrides for:

- `.goneat/ssot-consumer.local.yaml` (SSOT sources)
- `.goneat/tools.local.yaml` (tool definitions)

These files are **always gitignored** and let developers customize their environment without affecting the team.

### Multi-Source Sync

Sync from multiple SSOT repositories:

```yaml
version: 1.0
sources:
  - name: crucible
    repo: fulmenhq/crucible
    ref: main
    keys:
      - crucible.docs.standards
      - crucible.schemas.*

  - name: cosmography
    repo: fulmenhq/cosmography
    ref: main
    keys:
      - cosmography.data.topologies
```

## Troubleshooting

### Goneat Not Found

```bash
# Check if installed
command -v goneat

# Install
go install github.com/fulmenhq/goneat/cmd/goneat@latest

# Verify Go bin in PATH
echo $GOPATH/bin
export PATH=$PATH:$(go env GOPATH)/bin
```

### Sync Fails

```bash
# Check connectivity
curl -I https://api.github.com/repos/fulmenhq/crucible

# Use local clone
echo -e "version: 1.0\nsources:\n  - name: crucible\n    localPath: ../crucible" > .goneat/ssot-consumer.local.yaml

# Check available keys
goneat ssot keys
```

### Provenance Shows Dirty

```bash
# Check git status in Crucible clone
cd ../crucible
git status

# Clean or commit changes
git add .
git commit -m "..."
```

## Reference Implementations

- **pyfulmen** - Python reference (goneat v0.2.11+)
- **gofulmen** - Go reference (goneat v0.2.11+, bootstrap package)
- **tsfulmen** - TypeScript reference (goneat v0.3.0, Bun bootstrap)

See respective repositories for complete working examples.

## See Also

- [Bootstrap Goneat Guide](bootstrap-goneat.md) - Installing goneat tooling
- [Agentic Interface Adoption Guide](agentic-interface-adoption.md) - Adopting role catalog and attribution baseline
- [Sync Model Architecture](../architecture/sync-model.md) - How Crucible distribution works
- [Makefile Standard](../standards/makefile-standard.md) - Required targets
- [Repository Versioning](../standards/repository-versioning.md) - CalVer vs SemVer
- [Pull Script README](../../scripts/pull/README.md) - Alternative sync approach

---

**Note**: This guide reflects goneat v0.2.11+ patterns. Earlier versions used different directory structures. For library-specific details (bootstrap scripts, post-sync hooks, quality checklists), see individual library repositories.
