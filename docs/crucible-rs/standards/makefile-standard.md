---
title: "FulmenHQ Makefile Standard"
description: "Baseline make targets every Fulmen repository must implement"
author: "Codex Assistant"
date: "2025-10-02"
last_updated: "2025-11-07"
status: "approved"
tags: ["standards", "build", "cicd", "make"]
---

# Makefile Standard

## Why a Makefile Everywhere?

A shared `Makefile` gives Fulmen automation a consistent entry point across languages. Regardless of whether a project is Go, TypeScript, Python, or multi-language, the same targets exist so CI/CD pipelines, local scripts, and agent tooling can rely on them.

## Required Targets

## Compliance Requirements

**Full compliance** with the FulmenHQ Makefile Standard requires implementation of all required make targets. This ensures consistent tooling across the ecosystem, regardless of primary language or build system.

- **Makefile as Standard Interface**: The Makefile provides the canonical entry point for all standard operations (bootstrap, lint, test, build, etc.).
- **Package.json Scripts**: Optional for local development convenience, but **must not** duplicate or override Makefile logic. If present, package.json scripts should delegate to Makefile targets (e.g., `"test": "make test"`).
- **No Direct Dependencies in Scripts**: Package.json scripts cannot express dependencies directly. For full compliance, rely on Makefile for dependency management.
- **SSOT Consumers**: Repositories that receive synchronized assets (docs, schemas, configs) from Crucible or other SSOT sources MUST expose `make sync` so CI pipelines and agents can refresh upstream artifacts consistently.

### Language-Specific Notes

- **TypeScript/JavaScript Linting**: Use Biome for linting TypeScript and JavaScript files. Goneat does not currently support these languages adequately, so the `make lint` target uses Biome for TS/JS and Goneat for other languages (Go, YAML, schemas).
- **Go Linting**: Goneat handles Go linting via golangci-lint in the appropriate module directory.

### Enforcement

- CI/CD pipelines **must** use Makefile targets exclusively.
- Local development may use either, but Makefile is preferred for consistency.
- Repositories without a Makefile implementing all required targets are considered non-compliant.

Every repository **MUST** implement the following make targets:

| Target                                  | Purpose                                                                                                          |
| --------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `make help`                             | List available targets with short descriptions.                                                                  |
| `make bootstrap`                        | Install external prerequisites listed in `.goneat/tools.yaml` (installers must use the tooling manifest schema). |
| `make tools`                            | Verify external tools are present; may be a no-op if none are required.                                          |
| `make sync`                             | For repositories that consume SSOT artifacts, run the canonical sync pipeline (e.g., `goneat ssot sync`).        |
| `make lint`                             | Run lint/format/style checks.                                                                                    |
| `make test`                             | Execute the full test suite.                                                                                     |
| `make build`                            | Produce distributable artifacts/binaries for current platform.                                                   |
| `make build-all`                        | Build multi-platform binaries (Linux, macOS, Windows) and generate SHA256SUMS.txt checksums.                     |
| `make clean`                            | Remove build artifacts, caches, temp files.                                                                      |
| `make fmt`                              | Apply formatting (language-specific).                                                                            |
| `make version`                          | Print current repository version from `VERSION`.                                                                 |
| `make version-set VERSION=x`            | Update `VERSION` and any derived metadata. Must call repo-appropriate scripts (e.g., `bun run version:update`).  |
| `make version-bump-{major,minor,patch}` | Bump version according to strategy (SemVer or CalVer) and regenerate derived files.                              |
| `make release-check`                    | Run the release checklist validation (tests, lint, sync scripts).                                                |
| `make release-prepare`                  | Sequence of commands to ready a release (sync, tests, version bump).                                             |
| `make release-build`                    | Build release artifacts (binaries + checksums) for distribution.                                                 |
| `make prepush`                          | Run pre-push hooks (stub for now).                                                                               |
| `make precommit`                        | Run pre-commit hooks (stub for now).                                                                             |
| `make check-all`                        | Run all checks (lint, test, typecheck).                                                                          |

Repositories may add additional targets (e.g., `make docs`, `make package`). Required targets must NOT be renamed.

## Implementation Guidance

- Allow forcing bootstrap via `make bootstrap FORCE=1` or a `bootstrap-force` alias. Document this so CI/developers can reinstall tools when needed.

- Use phony targets (`.PHONY`) for commands that do not produce physical files.
- Keep commands quiet when appropriate (prefix with `@`) but print meaningful status lines.
- `make version-set` should delegate to the language-appropriate script (e.g., `bun run version:update` in Crucible) so wrappers stay synced.
- `make bootstrap`/`make tools` should call a script that reads `.goneat/tools.yaml` (validated against `schemas/tooling/external-tools/v1.0.0/external-tools-manifest.schema.yaml`).
- For `make build-all`, implement cross-compilation for common platforms (Linux amd64/arm64, macOS amd64/arm64, Windows amd64). Output binaries to `dist/` directory and generate `SHA256SUMS.txt` with checksums for all artifacts.
- `make release-build` should depend on `build-all` and may include additional packaging steps.
- Include a `help` target that parses comments, for example:
  ```makefile
  .PHONY: help
  help:
  	@grep -E '^\.PHONY: [a-zA-Z_-]+ ##' Makefile | sed -e 's/\.PHONY: //' -e 's/ ##/\t/'
  ```
- Document deviations or extensions in `CONTRIBUTING.md`.

### SSOT Sync Hooks

- Any repository designated as an SSOT consumer MUST implement `make sync` so downstream tooling can refresh Crucible assets consistently. The target should invoke the canonical sync tool (`goneat ssot sync` in current libraries such as `pyfulmen`) and may wrap additional validation.
- Repositories MAY add post-sync helper targets (for example, `make sync-foundry-assets` in `pyfulmen`) to copy artifacts or trigger language-specific regeneration. Document these hooks alongside the primary target (see `pyfulmen/docs/development/foundry-asset-sync.md` for the reference implementation).
- The required `make sync` target should remain idempotent and safe to run in CI; auxiliary hooks can depend on it or be chained via `make sync && make sync-foundry-assets`.

### Multi-Platform Build Example

For Go-based tools, implement `build-all` as follows:

```makefile
BIN_NAME := mytool
VERSION := $(shell cat VERSION)
DIST_DIR := dist

.PHONY: build-all
build-all: ## Build multi-platform binaries and generate checksums
	@echo "Building $(BIN_NAME) $(VERSION) for multiple platforms..."
	@mkdir -p $(DIST_DIR)
	@for os in linux darwin windows; do \
		for arch in amd64 arm64; do \
			if [ "$$os" = "windows" ] && [ "$$arch" = "arm64" ]; then continue; fi; \
			echo "Building $$os/$$arch..."; \
			EXT=""; \
			if [ "$$os" = "windows" ]; then EXT=".exe"; fi; \
			GOOS=$$os GOARCH=$$arch go build -ldflags="-X main.version=$(VERSION)" -o $(DIST_DIR)/$(BIN_NAME)-$$os-$$arch$$EXT ./cmd; \
		done; \
	done
	@echo "Generating SHA256SUMS.txt..."
	@cd $(DIST_DIR) && sha256sum * > SHA256SUMS.txt
	@echo "✅ Multi-platform build complete - artifacts in $(DIST_DIR)/"
```

### Clean Target Best Practices

The `make clean` target removes **build artifacts** only. It must **never** delete working directories containing user-created content.

**What to Delete** (safe, reproducible artifacts):

- Compiled binaries: `dist/`, `build/`, language-specific output directories
- Generated code: Auto-generated files with headers indicating generation
- Downloaded dependencies: `node_modules/` (restorable via `npm install`), language package caches
- Build caches: `.cache/`, language-specific build caches
- External tool caches: `bin/` directory from `make bootstrap` (restorable via re-bootstrap)

**What NOT to Delete** (user workspace, non-reproducible):

- `.plans/` - Local planning workspace with session notes, incident logs, user-created content
- `.env` - User's local environment configuration
- `.vscode/`, `.idea/` - User's IDE settings
- User configuration files not tracked in git
- Any gitignored directory containing user-authored content

**Example Clean Target:**

```makefile
clean: ## Clean build artifacts only
	@echo "Cleaning artifacts..."
	@rm -rf dist/ lang/*/dist/ bin/ .cache/
	@echo "✅ Clean completed"
```

**Guideline**: If a directory is gitignored because it contains **user content** (not because it contains **generated artifacts**), it should not be in the clean target.

**Common Mistake**: Including `.plans/` in clean target. This directory is gitignored for workspace privacy, not because it's a build artifact. Deleting it destroys non-recoverable planning work.

## Verification

- CI pipelines should call `make lint`, `make test`, and `make release-check`.
- For repositories producing binaries, CI should also call `make build-all` to verify cross-compilation works.
- `make release-check` must cover all items in `RELEASE_CHECKLIST.md` (or invoke a script that does).
- Release workflows should use `make release-build` to produce distributable artifacts with checksums.

---

## Annex A: Server Orchestration Targets

**Applicability**: Required for repositories implementing server functionality (workhorses, services, servers).

Repositories that implement HTTP/gRPC/WebSocket servers **MUST** expose the following additional targets to support server orchestration. These targets enable local development, testing, accessibility verification, and preview environments using the configuration classes defined in the [Fulmen Server Management module](library/modules/server-management.md).

### Required Server Targets

| Target                  | Purpose                                                                             |
| ----------------------- | ----------------------------------------------------------------------------------- |
| `make server-start-%`   | Start server in specified configuration class (dev, test, a11y, preview, prod_like) |
| `make server-stop-%`    | Stop server for specified configuration class using graceful shutdown (SIGTERM)     |
| `make server-status-%`  | Check if server is running and healthy for specified configuration class            |
| `make server-restart-%` | Restart server (stop + start) for specified configuration class                     |
| `make server-logs-%`    | Tail or display logs for specified configuration class                              |

**Pattern**: The `%` wildcard represents the configuration class (`dev`, `test`, `a11y`, `preview`, `prod_like`).

### Example Usage

```bash
# Start server in dev configuration (uses dev.preferredPort, dev.healthCheck settings)
make server-start-dev

# Check if test server is running and healthy
make server-status-test

# Stop accessibility testing server
make server-stop-a11y

# Restart preview server (for reviewing feature branches)
make server-restart-preview

# View logs for production-like testing server
make server-logs-prod_like
```

### Implementation Requirements

**Server Start** (`server-start-%`):

1. Resolve server configuration from Crucible schemas (`schemas/server/management/v1.0.0/server-management.schema.json`)
2. Check if preferred port is available; if not, find available port in configured range
3. Start server process with appropriate environment variables (honor `envOverrides` from configuration)
4. Write PID to configured `pidFile` (if specified in configuration)
5. Poll health endpoint using retry logic from `healthCheck.retries` and `healthCheck.interval`
6. Exit with appropriate code on failure:
   - `EXIT_PORT_IN_USE` (10) if no ports available
   - `EXIT_HEALTH_CHECK_FAILED` (30) if health check fails after retries
   - `EXIT_OPERATION_TIMEOUT` (34) if server doesn't start within timeout window

**Server Stop** (`server-stop-%`):

1. Read PID from `pidFile` (if exists)
2. Send SIGTERM to server process (graceful shutdown)
3. Wait for process to exit (with timeout)
4. Clean up PID file
5. Optionally clean up log files if specified in configuration

**Server Status** (`server-status-%`):

1. Check if PID file exists and process is running
2. Perform health check against configured endpoint
3. Report status (running + healthy, running + unhealthy, stopped)
4. Exit with 0 if healthy, non-zero otherwise

**Server Restart** (`server-restart-%`):

1. Call `server-stop-%` target
2. Call `server-start-%` target
3. Propagate exit codes appropriately

**Server Logs** (`server-logs-%`):

1. Read log file path from configuration (if specified)
2. Tail logs using appropriate command (`tail -f`, `less +F`, etc.)
3. If no log file configured, indicate where logs are being written (stdout, syslog, etc.)

### Configuration Classes

Server targets must support the following standard configuration classes:

- **dev**: Local development (hot reload, verbose logging, debug endpoints enabled)
- **test**: Automated testing (predictable port ranges, test fixtures, fast startup)
- **a11y**: Accessibility testing (same as dev but dedicated port range to avoid conflicts)
- **preview**: Feature branch preview (production-like config but isolated port ranges)
- **prod_like**: Production-like testing (mimics production settings for final validation)

**Environment Variable Overrides**: Each configuration class specifies `envOverrides` that take precedence over defaults (e.g., `FULMEN_PULSAR_DEV_PORT`, `FULMEN_PULSAR_TEST_PORT`). Derive the prefix from application identity where possible and ensure Makefile targets honor these overrides.

### Integration with Fulmen Standards

- **Exit Codes**: Use standardized exit codes from `config/library/foundry/exit-codes.yaml` (see [Exit Codes README](fulmen/exit-codes/README.md))
- **Health Endpoints**: Validate responses against `schemas/protocol/http/v1.0.0/health-response.schema.json`
- **Logging**: Server logs should conform to `schemas/observability/logging/v1.0.0/log-event.schema.json`

### Example Makefile Implementation

```makefile
# Server orchestration targets
.PHONY: server-start-%
server-start-%:
	@echo "🚀 Starting server in $* configuration..."
	@ts-node scripts/server/start.ts --config $*

.PHONY: server-stop-%
server-stop-%:
	@echo "🛑 Stopping server in $* configuration..."
	@ts-node scripts/server/stop.ts --config $*

.PHONY: server-status-%
server-status-%:
	@echo "🔍 Checking server status for $* configuration..."
	@ts-node scripts/server/status.ts --config $*

.PHONY: server-restart-%
server-restart-%: server-stop-% server-start-%
	@echo "♻️  Server restarted in $* configuration"

.PHONY: server-logs-%
server-logs-%:
	@echo "📋 Displaying logs for $* configuration..."
	@ts-node scripts/server/logs.ts --config $*
```

**Note**: The actual implementation logic should be in scripts (e.g., `scripts/server/start.ts` for TypeScript, `scripts/server/start.py` for Python, etc.) that leverage helper library modules (gofulmen, tsfulmen, pyfulmen) for configuration resolution, port management, and health checks.

### Related Documentation

- [Fulmen Server Management Architecture](../architecture/fulmen-server-management.md) – High-level server orchestration patterns
- [Server Management Module Spec](library/modules/server-management.md) – Detailed module specification
- [HTTP REST Standards](protocol/http-rest-standards.md) – Health endpoint standards

---

## Annex B: Template Repository CDRL Targets

**Applicability**: Required for Fulmen forge templates (workhorse, codex, gymnasium) that support CDRL (Clone → Degit → Refit → Launch) workflow.

Repositories categorized as `forge-workhorse`, `forge-codex`, or `forge-gymnasium` in the [Repository Categories Taxonomy](../../config/taxonomy/repository-categories.yaml) **MUST** implement the following validation targets to support CDRL compliance per the [Fulmen Template CDRL Standard](../architecture/fulmen-template-cdrl-standard.md).

### Required CDRL Validation Targets

| Target                                        | Purpose                                          | Exit Codes                       |
| --------------------------------------------- | ------------------------------------------------ | -------------------------------- |
| `make validate-app-identity`                  | Detect hardcoded breed/site names in source code | 0=clean, 1=violations            |
| `make doctor` (or `make validate-cdrl-ready`) | Comprehensive CDRL refit completeness check      | 0=complete, 1=warnings, 2=errors |

### Target Specifications

#### `validate-app-identity`

**Purpose**: Scans source code for hardcoded template identifiers that should derive from `.fulmen/app.yaml`.

**Implementation Requirements**:

- Scan all source files for breed/site name (from App Identity `binary_name`)
- EXCLUDE from scan: `.fulmen/`, `docs/`, `README.md`, `CHANGELOG.md`, `.git/`
- Report files and line numbers containing violations
- Exit 0 if no violations, exit 1 if violations found

**Example Implementation** (workhorse with breed "groningen"):

```makefile
.PHONY: validate-app-identity
validate-app-identity: ## Detect hardcoded breed references
	@echo "🔍 Scanning for hardcoded breed references..."
	@if grep -r "groningen" \
		--exclude-dir=".git" \
		--exclude-dir=".fulmen" \
		--exclude-dir="docs" \
		--exclude="*.md" \
		--exclude="Makefile" \
		src/ internal/ cmd/ 2>/dev/null | grep -v "\.fulmen/app\.yaml"; then \
		echo "❌ Found hardcoded 'groningen' references (see above)"; \
		echo "   ACTION: Replace with App Identity references"; \
		exit 1; \
	else \
		echo "✅ No hardcoded breed references found"; \
	fi
```

**Language-Specific Notes**:

- **Go**: Scan `cmd/`, `internal/`, `pkg/` directories
- **Python**: Scan `src/`, `tests/` directories
- **TypeScript**: Scan `src/`, `lib/` directories

#### `doctor` (or `validate-cdrl-ready`)

**Purpose**: Comprehensive validation of CDRL refit completeness.

**Implementation Requirements**:

- Check App Identity file exists and validates
- Verify environment variable prefix consistency
- Verify configuration paths match App Identity
- Verify module path updated (Go `go.mod`, Python `pyproject.toml`, TypeScript `package.json`)
- Verify tests pass
- Report status with actionable suggestions

**Exit Codes**:

- `0`: All checks passed, CDRL refit complete
- `1`: Warnings detected (non-blocking issues)
- `2`: Errors detected (blocking issues)

**Example Implementation**:

```makefile
.PHONY: doctor
doctor: ## Validate CDRL refit completeness
	@echo "🏥 Running CDRL completeness check..."
	@./scripts/cdrl-doctor.sh  # Delegates to script
	@echo "✅ Doctor check complete"
```

**Doctor Script Requirements** (language-agnostic checks):

1. **App Identity Validation**:
   - File exists at `.fulmen/app.yaml`
   - YAML parses correctly
   - Required fields present: `vendor`, `binary_name`, `env_prefix`, `config_name`

2. **Environment Variable Consistency**:
   - All env vars in `.env` use prefix from App Identity
   - No stray template env vars (e.g., `GRONINGEN_*` when breed changed)

3. **Configuration Path Validation**:
   - Config files renamed to match `config_name`
   - Config directory structure consistent

4. **Module Path Validation** (language-specific):
   - Go: `go.mod` module path updated from template
   - Python: `pyproject.toml` name field updated
   - TypeScript: `package.json` name field updated

5. **Test Suite**:
   - Run `make test` as final validation
   - Report failures with actionable suggestions

**Example Output** (success):

```
🏥 Running CDRL completeness check...
✅ App Identity: Valid (.fulmen/app.yaml)
✅ Environment Variables: Consistent prefix (MYAPI_)
✅ Configuration Paths: Match identity (config_name: myapi)
✅ Module Path: Updated (github.com/mycompany/myapi)
✅ Tests: Passing (10 tests, 0 failures)
🎉 CDRL refit complete - ready to launch!
```

**Example Output** (errors):

```
🏥 Running CDRL completeness check...
✅ App Identity: Valid (.fulmen/app.yaml)
❌ Environment Variables: Found GRONINGEN_ prefix (expected MYAPI_)
   ACTION: Update .env file with correct prefix
⚠️  Configuration Paths: Config file not renamed
   ACTION: mv config/groningen.yaml config/myapi.yaml
❌ Module Path: Still using template path (github.com/fulmenhq/forge-workhorse-groningen)
   ACTION: Update go.mod module directive
❌ Tests: 3 failures
   ACTION: Fix test failures before launching
🛑 CDRL refit incomplete - fix errors above
```

### Optional CDRL Targets

Templates MAY provide additional convenience targets:

| Target             | Purpose                                                             |
| ------------------ | ------------------------------------------------------------------- |
| `make cdrl-refit`  | Interactive script prompting for identity fields and updating files |
| `make cdrl-clean`  | Remove template artifacts (examples, placeholder logic)             |
| `make cdrl-verify` | Alias for `make validate-app-identity && make doctor`               |

### Template Documentation Requirements

Templates implementing these targets MUST document:

1. CDRL workflow in `README.md` (quick start)
2. Detailed CDRL guide at `docs/development/*cdrl*.md`
3. Makefile target descriptions in `make help` output

### Validation in CI

Template CI pipelines SHOULD include CDRL readiness checks:

```yaml
- name: Validate CDRL Readiness
  run: |
    make validate-app-identity
    make doctor
```

### Related Documentation

- [Fulmen Template CDRL Standard](../architecture/fulmen-template-cdrl-standard.md) – Architectural requirements
- [CDRL Workflow Guide](cdrl/workflow-guide.md) – User-facing step-by-step instructions
- [App Identity Module](library/modules/app-identity.md) – Technical specification
- [Repository Categories Taxonomy](../../config/taxonomy/repository-categories.yaml) – Template category definitions

---

## Relationship to Other Standards

- [Release Checklist Standard](release-checklist-standard.md) – Make targets drive release validation.
- [Repository Structure SOP](../sop/repository-structure.md) – References the requirement for a Makefile and tooling manifest.
- [CI/CD Operations SOP](../sop/cicd-operations.md) – Pipelines must invoke `make bootstrap` before validation steps.
- Language-specific coding standards can recommend additional targets (e.g., `make docs`, `make fmt-go`).

## Future Enhancements

- Provide a template `Makefile` generator in Crucible with standard implementations for `build-all` and `release:build`.
- Add CI linters to ensure required targets exist and validate multi-platform build outputs.
- Extend to support remote execution wrappers (e.g., `make plan` for infrastructure).
- Standardize artifact naming conventions and checksum formats across repositories.
