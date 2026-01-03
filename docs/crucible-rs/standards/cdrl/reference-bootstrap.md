# Reference Bootstrap Pattern

**Version**: v0.2.8
**Status**: Recommended
**Audience**: Template maintainers

## Overview

This document defines the **reference bootstrap pattern** for Fulmen forge templates, enabling reproducible local development environments through version-pinned tool installation.

**Problem**: Templates need development tools (goneat, language-specific utilities) but system-wide installation causes version drift and "works on my machine" issues.

**Solution**: Manifest-driven local bootstrap that installs pinned tool versions to `bin/` directory.

**CDRL Integration**: Bootstrap is the first step of the **Launch** stage (Clone → Degit → Refit → **Launch**).

---

## Pattern Overview

### Architecture

```
.goneat/
  tools.yaml              # Production manifest (committed, version-pinned)
  tools.local.yaml        # Local dev override (gitignored, optional)

scripts/
  bootstrap.py            # Bootstrap implementation (Python example)
  bootstrap.ts            # Bootstrap implementation (TypeScript example)

bin/                      # Local tool binaries (gitignored)
  goneat                  # Installed via bootstrap

Makefile:
  bootstrap: ## Install dependencies and development tools
    @uv run python scripts/bootstrap.py
```

### Workflow

1. **Developer clones template** → No tools installed yet
2. **Run `make bootstrap`** → Reads `.goneat/tools.yaml`, downloads tools to `bin/`
3. **Local development** → Uses `bin/goneat` (pinned version, reproducible)
4. **CI/CD** → Runs same `make bootstrap`, identical environment

---

## Manifest Schema

### `.goneat/tools.yaml`

Declarative manifest defining required tools and installation metadata.

**Format**: Goneat Tools v0.3.0+ schema

**Example**:

```yaml
# .goneat/tools.yaml
version: v0.3.3
binDir: ./bin
tools:
  - id: goneat
    description: Fulmen schema validation and automation CLI
    required: true
    install:
      type: download
      url: https://github.com/fulmenhq/goneat/releases/download/v0.3.3/goneat_v0.3.3_{{os}}_{{arch}}.tar.gz
      binName: goneat
      destination: ./bin
      checksum:
        darwin-arm64: "5549d578d4d9680a0404775e7043b14a8f37c32c043a025bab56978e1d894a18"
        darwin-amd64: "96ec9fbfbbc7203ca5bbdcef78bbdf735e9081942552b94bede8a0b52eee64aa"
        linux-amd64: "31326788c7375797c54c48b6748a56ab6c8f197ca3ebadb346865c1efcb6887a"
        linux-arm64: "8b8ba91c2cd890c5a13f610d35885c907f3bb4717c7793f1db5525eff8dd84c8"
      # SHA256 checksums from: https://github.com/fulmenhq/goneat/releases/download/v0.3.3/SHA256SUMS
```

**Schema Fields**:

| Field                         | Type    | Required       | Description                                            |
| ----------------------------- | ------- | -------------- | ------------------------------------------------------ |
| `version`                     | string  | Yes            | Goneat tools manifest version                          |
| `binDir`                      | string  | Yes            | Local directory for installed tools (usually `./bin`)  |
| `tools`                       | array   | Yes            | List of tool definitions                               |
| `tools[].id`                  | string  | Yes            | Tool identifier (e.g., `goneat`)                       |
| `tools[].description`         | string  | No             | Human-readable description                             |
| `tools[].required`            | boolean | No             | Fail bootstrap if installation fails (default: false)  |
| `tools[].install.type`        | string  | Yes            | Installation method: `download`, `link`                |
| `tools[].install.url`         | string  | Yes (download) | Download URL with `{{os}}` and `{{arch}}` placeholders |
| `tools[].install.binName`     | string  | Yes            | Binary name (e.g., `goneat`)                           |
| `tools[].install.destination` | string  | Yes            | Installation directory (matches `binDir`)              |
| `tools[].install.checksum`    | object  | No             | SHA256 checksums by platform (e.g., `darwin-arm64`)    |

**Platform Substitutions**:

- `{{os}}`: darwin, linux, windows
- `{{arch}}`: amd64, arm64

### `.goneat/tools.local.yaml` (Optional)

Local development override, **never committed** (add to `.gitignore`).

**Use Cases**:

- Link to locally built goneat during development
- Override version for testing unreleased features
- Platform-specific workarounds

**Example**:

```yaml
version: v0.3.3-dev
binDir: ./bin
tools:
  - id: goneat
    description: Local development build
    required: true
    install:
      type: link
      source: ~/dev/goneat/bin/goneat # Link to local build
      binName: goneat
```

**Priority**: Bootstrap script MUST prefer `tools.local.yaml` over `tools.yaml` if present.

---

## Reference Implementations

### Canonical Implementations by Language

| Language       | Template                                                                           | Implementation                                                                                                 | Status            |
| -------------- | ---------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- | ----------------- |
| **Python**     | [forge-workhorse-percheron](https://github.com/fulmenhq/forge-workhorse-percheron) | [`scripts/bootstrap.py`](https://github.com/fulmenhq/forge-workhorse-percheron/blob/main/scripts/bootstrap.py) | ✅ Canonical      |
| **TypeScript** | forge-codex-aurora (TBD)                                                           | `scripts/bootstrap.ts` (TBD)                                                                                   | 🔜 In Development |
| **Go**         | [forge-workhorse-groningen](https://github.com/fulmenhq/forge-workhorse-groningen) | N/A - Uses native Go dependency management                                                                     | ⚪ Not Applicable |

### Why Go Templates Don't Need Bootstrap

**Reason**: Go templates can add goneat as a native Go dependency in `go.mod`:

```go
// go.mod
module github.com/mycompany/myapp

require (
    github.com/fulmenhq/goneat v0.3.3
)
```

Then install via standard Go tooling:

```bash
go install github.com/fulmenhq/goneat@v0.3.3
```

**Trade-off**: Goneat binary installed to `$GOPATH/bin` (system-wide) rather than `bin/` (local). Acceptable for Go ecosystem conventions.

### Python Reference: `scripts/bootstrap.py`

**Source**: [percheron/scripts/bootstrap.py](https://github.com/fulmenhq/forge-workhorse-percheron/blob/main/scripts/bootstrap.py)

**Features**:

- Parses `.goneat/tools.yaml` with PyYAML
- Supports `download` and `link` install types
- Downloads archives (tar.gz, zip) and extracts binaries
- Verifies SHA256 checksums for security
- Makes binaries executable (`chmod 0o755`)
- Prefers `tools.local.yaml` for local dev
- Provides `--force` flag to reinstall

**Dependencies**: PyYAML (usually in template's dev dependencies)

**Usage**:

```bash
# Standard bootstrap
python scripts/bootstrap.py

# Force reinstall
python scripts/bootstrap.py --force
```

**Makefile Integration**:

```makefile
.PHONY: bootstrap
bootstrap:  ## Install dependencies and development tools
	@echo "🔧 Bootstrapping development environment..."
	@command -v uv >/dev/null 2>&1 || (echo "❌ uv not found" && exit 1)
	@echo "🛠️  Installing external tools (goneat)..."
	@uv run python scripts/bootstrap.py
	@echo "📦 Installing Python dependencies..."
	@uv sync --all-extras
	@echo "✅ Bootstrap complete!"

bin/goneat:
	@echo "⚠️  Goneat not found. Run 'make bootstrap' first."
	@exit 1
```

### TypeScript Reference: `scripts/bootstrap.ts` (Future)

**Status**: TBD - Will be implemented in first codex template (aurora)

**Expected Features**:

- Parse YAML with `js-yaml`
- Download and extract with native Node.js APIs
- Verify checksums with `crypto` module
- Cross-platform support (darwin, linux, windows)

**Anticipated Usage**:

```bash
# Standard bootstrap (via Bun)
bun run scripts/bootstrap.ts

# Force reinstall
bun run scripts/bootstrap.ts --force
```

**Makefile Integration** (expected):

```makefile
.PHONY: bootstrap
bootstrap:  ## Install dependencies and development tools
	@echo "🔧 Bootstrapping development environment..."
	@command -v bun >/dev/null 2>&1 || (echo "❌ Bun not found" && exit 1)
	@echo "🛠️  Installing external tools (goneat)..."
	@bun run scripts/bootstrap.ts
	@echo "📦 Installing TypeScript dependencies..."
	@bun install
	@echo "✅ Bootstrap complete!"
```

---

## Adaptation Guidance

### For Template Maintainers

When implementing bootstrap for a new template:

#### 1. Choose Implementation Language

**Guideline**: Match template's primary language ecosystem.

| Template Language     | Bootstrap Language    | Rationale                                     |
| --------------------- | --------------------- | --------------------------------------------- |
| Python                | Python                | Native PyYAML, developers have Python         |
| TypeScript/JavaScript | TypeScript/JavaScript | Native YAML parsing, developers have Node/Bun |
| Go                    | N/A                   | Use native Go dependency management           |
| Rust                  | Rust                  | Native TOML/YAML, developers have Rust        |

#### 2. Copy Canonical Implementation

**Python Templates**:

```bash
# Copy from percheron
curl -o scripts/bootstrap.py \
  https://raw.githubusercontent.com/fulmenhq/forge-workhorse-percheron/main/scripts/bootstrap.py
```

**TypeScript Templates** (when available):

```bash
# Copy from aurora (TBD)
curl -o scripts/bootstrap.ts \
  https://raw.githubusercontent.com/fulmenhq/forge-codex-aurora/main/scripts/bootstrap.ts
```

#### 3. Create `.goneat/tools.yaml`

Get latest goneat version from: https://github.com/fulmenhq/goneat/releases

**Populate checksums**:

```bash
# Download SHA256SUMS from release
curl -L https://github.com/fulmenhq/goneat/releases/download/v0.3.3/SHA256SUMS

# Extract checksums for your platforms
grep "darwin_arm64" SHA256SUMS
grep "darwin_amd64" SHA256SUMS
grep "linux_amd64" SHA256SUMS
grep "linux_arm64" SHA256SUMS
```

#### 4. Update `.gitignore`

```gitignore
# Local tool binaries (installed via make bootstrap)
/bin/

# Local dev tool overrides (never commit)
/.goneat/tools.local.yaml
```

#### 5. Integrate with Makefile

**Required Targets**:

```makefile
.PHONY: bootstrap
bootstrap:  ## Install dependencies and development tools
	# ... language-specific implementation

.PHONY: bootstrap-force
bootstrap-force:  ## Force reinstall all dependencies and tools
	@echo "🔄 Force bootstrap..."
	@rm -rf .venv bin/goneat  # Adjust paths as needed
	# ... run bootstrap with --force

bin/goneat:
	@echo "⚠️  Goneat not found. Run 'make bootstrap' first."
	@exit 1

.PHONY: tools
tools: bin/goneat  ## Verify external tools are present
	@echo "Verifying external tools..."
	@bin/goneat version > /dev/null && echo "✓ Goneat: $$(bin/goneat version)" || exit 1
```

**Makefile Standard Compliance**: See [Makefile Standard](../makefile-standard.md) for complete requirements.

#### 6. Document in `tooling-guide.md` (Optional)

If template provides `docs/development/tooling-guide.md`, document the bootstrap process:

````markdown
## Bootstrap Process

Percheron uses a manifest-driven bootstrap to install development tools locally.

### Quick Start

```bash
make bootstrap
```
````

This installs:

- **Goneat** (v0.3.3) - Fulmen schema validation and automation CLI
- **Python dependencies** (via uv)

### How It Works

See [Crucible Reference Bootstrap Pattern](https://github.com/fulmenhq/crucible/blob/main/docs/standards/cdrl/reference-bootstrap.md) for implementation details.

### Local Development Override

To use a locally built goneat:

```yaml
# .goneat/tools.local.yaml
version: v0.3.3-dev
binDir: ./bin
tools:
  - id: goneat
    install:
      type: link
      source: ~/dev/goneat/bin/goneat
      binName: goneat
```

Run `make bootstrap` to activate.

````

---

## Integration Points

### CDRL Workflow

Bootstrap is the **first substep** of the **Launch** stage.

**Location in CDRL**: Clone → Degit → Refit → **Launch** (Step 4.1)

**Workflow**:
1. Developer completes Refit (App Identity, module path, config)
2. **Run `make bootstrap`** → Install tools, dependencies
3. Run `make doctor` → Validate CDRL completeness
4. Run `make test` → Verify tests pass
5. Run `make build && make run` → Launch application

**Reference**: See [CDRL Workflow Guide](workflow-guide.md#step-4-launch) for complete Launch stage.

### Required Makefile Targets

Bootstrap integrates with these Makefile targets:

| Target | Purpose | Bootstrap Dependency |
|--------|---------|---------------------|
| `make bootstrap` | Install tools and dependencies | N/A (entry point) |
| `make fmt` | Format code | Requires `bin/goneat` |
| `make polish` | Run goneat polish | Requires `bin/goneat` |
| `make tools` | Verify tools installed | Requires `bin/goneat` |

**Pattern**: Targets that need goneat should depend on `bin/goneat` target (fails with helpful message if not bootstrapped).

### Template Documentation Structure

Bootstrap documentation appears in:

| Document | Section | Content |
|----------|---------|---------|
| **`fulmen_cdrl_guide.md`** (REQUIRED) | Step 4: Launch | Brief mention of `make bootstrap` command |
| **`tooling-guide.md`** (OPTIONAL) | Bootstrap Process | Detailed explanation, manifest structure, local overrides |
| **`README.md`** (RECOMMENDED) | Quick Start | Include `make bootstrap` in initial setup steps |

**Separation of Concerns**:
- **CDRL Guide**: Focus on WHAT to run (`make bootstrap`)
- **Tooling Guide**: Focus on HOW it works (manifest, checksums, local dev)

---

## Security Considerations

### Checksum Verification

**REQUIRED**: Bootstrap scripts MUST verify SHA256 checksums for downloaded binaries.

**Rationale**: Prevents supply chain attacks via compromised downloads.

**Implementation**:
```python
import hashlib

def compute_sha256(file_path: Path) -> str:
    sha256 = hashlib.sha256()
    with open(file_path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            sha256.update(chunk)
    return sha256.hexdigest()

# Verify
if actual_checksum != expected_checksum:
    raise ValueError(f"Checksum mismatch! Expected: {expected}, Got: {actual}")
````

**Provisional Checksums**: During development, use placeholder `"0" * 64` (all zeros). Bootstrap SHOULD skip verification for provisional checksums and warn user.

### HTTPS Downloads

**REQUIRED**: All download URLs MUST use HTTPS.

**Bad**:

```yaml
url: http://github.com/fulmenhq/goneat/releases/... # ❌ Insecure
```

**Good**:

```yaml
url: https://github.com/fulmenhq/goneat/releases/... # ✅ Secure
```

### Executable Permissions

**REQUIRED**: Bootstrap scripts MUST set executable permissions on installed binaries.

**Implementation**:

```python
dest_file.chmod(0o755)  # rwxr-xr-x
```

---

## Testing

### Template Maintainer Checklist

Before releasing template with bootstrap:

- [ ] `make bootstrap` completes successfully on clean clone
- [ ] `bin/goneat version` reports expected version
- [ ] Checksums verified for all platforms (darwin-arm64, darwin-amd64, linux-amd64, linux-arm64)
- [ ] `.gitignore` excludes `bin/` and `tools.local.yaml`
- [ ] `make bootstrap-force` reinstalls correctly
- [ ] `bin/goneat` target fails with helpful message if not bootstrapped
- [ ] `make help` documents bootstrap targets
- [ ] CI/CD runs `make bootstrap` before other commands

### CI/CD Integration

**GitHub Actions Example**:

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install uv (Python templates)
        run: curl -LsSf https://astral.sh/uv/install.sh | sh

      - name: Bootstrap development environment
        run: make bootstrap

      - name: Verify tools
        run: make tools

      - name: Run tests
        run: make test
```

**Verification**: Bootstrap in CI should produce identical `bin/goneat` version as local development (reproducibility).

---

## FAQ

### Why not install goneat system-wide via Brew?

**Answer**: Version pinning and reproducibility. Different projects may require different goneat versions. Local `bin/` installation ensures:

- ✅ Each project has its required version
- ✅ CI/CD matches local development exactly
- ✅ No conflicts between projects

**Note**: Developers CAN install goneat via Brew for convenience, but `make bootstrap` ensures pinned version is available.

### Why prefer `tools.local.yaml` over `tools.yaml`?

**Answer**: Enables local development without committing changes:

- ✅ Test unreleased goneat features by linking local build
- ✅ Override platform-specific settings without affecting team
- ✅ Temporary workarounds don't pollute git history

### What if download fails?

**Answer**: Bootstrap script should provide actionable error messages:

```
❌ Failed to download goneat from https://github.com/...
   Check your internet connection and try again.
   Alternative: Install goneat via Brew: brew install fulmenhq/tap/goneat
```

### Do I need PyYAML/js-yaml as a production dependency?

**Answer**: No. Bootstrap dependencies are **development-only**:

**Python** (pyproject.toml):

```toml
[project.optional-dependencies]
dev = [
    "PyYAML>=6.0",
    # ... other dev deps
]
```

**TypeScript** (package.json):

```json
{
  "devDependencies": {
    "js-yaml": "^4.1.0"
  }
}
```

---

## References

- [CDRL Architectural Standard](../../architecture/fulmen-template-cdrl-standard.md) - Template CDRL compliance requirements
- [CDRL Workflow Guide](workflow-guide.md) - User-facing CDRL workflow (Launch stage)
- [Makefile Standard](../makefile-standard.md) - Required Makefile targets and conventions
- [Workhorse Standard](../../architecture/fulmen-forge-workhorse-standard.md) - Backend service template requirements
- [Codex Standard](../../architecture/fulmen-forge-codex-standard.md) - Documentation site template requirements
- [Goneat GitHub Repository](https://github.com/fulmenhq/goneat) - Latest releases and SHA256SUMS
- [Percheron Template](https://github.com/fulmenhq/forge-workhorse-percheron) - Python canonical implementation
- [Groningen Template](https://github.com/fulmenhq/forge-workhorse-groningen) - Go template (native dependency management)

---

## Version History

| Version | Date       | Changes                                       |
| ------- | ---------- | --------------------------------------------- |
| v0.2.8  | 2025-11-07 | Initial version documenting bootstrap pattern |

---

**Maintained by**: Fulmen HQ
**Questions**: See [Crucible SSOT Repository](https://github.com/fulmenhq/crucible)
