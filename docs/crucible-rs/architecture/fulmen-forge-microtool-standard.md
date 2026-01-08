---
title: "Fulmen Forge Microtool Standard"
description: "Standard structure and capabilities for Fulmen Microtool forges - production-ready templates for ultra-narrow, single-purpose CLI deployment tools"
author: "Schema Cartographer (@schema-cartographer)"
date: "2025-11-15"
last_updated: "2025-12-18"
status: "draft"
tags: ["architecture", "forge", "microtool", "template", "2025.10.2"]
---

# Fulmen Forge Microtool Standard

This document defines the standardized structure and pre-integrated capabilities for Fulmen Microtool forges. Microtool forges provide production-ready templates for ultra-narrow, single-purpose CLI deployment and automation tools (e.g., fixture deployment, config synchronization, asset management) that must remain focused and lightweight. They embody the CDRL philosophy (Clone → Degit → Refit → Launch) and align with the repository category taxonomy (`microtool` key from [repository-categories.yaml](config/taxonomy/repository-categories.yaml)).

Microtools are distinguished from other categories by their **architectural constraints**:

- **Single primary purpose** (if scope grows → promote to `cli` category)
- **No library exports** (no `pkg/` directory, CLI-only interface)
- **Helper library dependency** (SHOULD import gofulmen/tsfulmen/rsfulmen)
- **One-way dependency flow** (microtool → helper → SSOT, prevents circular dependencies)
- **No web server endpoints** (CLI stdin/stdout/stderr only)

Microtool forges use **tool/instrument** themed names to distinguish from workhorse (horse breeds) and codex (documentation) templates.

The canonical list of forge categories and statuses is maintained in the [Repository Category Taxonomy](config/taxonomy/repository-categories.yaml); consult that before proposing new forges or changing lifecycle states.

For a quick view of module requirements across forge categories, see the [Module Compliance Matrix](./module-compliance-matrix.md). CDRL expectations for all templates are defined in the [Fulmen Template CDRL Standard](./fulmen-template-cdrl-standard.md); microtool forges must satisfy those requirements in addition to the guidance here.

## Reference Implementation Requirement

Microtool forges are **reference implementations**, not hollow templates. Every published microtool MUST:

- Build, lint, and test successfully immediately after cloning.
- Include working example commands demonstrating the single primary purpose.
- Ship with sane defaults so maintainers can `clone → make check-all → run` before performing any refit.

CDRL (Clone → Degit → Refit → Launch) happens **after** a team builds and validates the reference implementation. “Template” in this context means “ready-to-run forge that you refit,” not “collection of placeholders.” Any TODO-only files, stubbed logic, or unimplemented commands violate this standard.

When creating a new microtool forge, ensure CI proves the repo is runnable prior to documentation mentioning CDRL. Refit scripts (rename, identity updates) exist to adapt a working tool—not to finish unfinished code.

## Scope

Applies to Microtool-specific forge templates (e.g., `forge-microtool-anvil`, `forge-microtool-chisel`). Microtools MUST remain narrow in scope - a tool that grows beyond single purpose violates the category and must be promoted to `cli`. Forges are not SSOT repos or full applications but starters that integrate Fulmen helper libraries (gofulmen, pyfulmen, tsfulmen) to enforce standards while staying lightweight.

Core philosophy: Ship minimal, focused templates that handle authentication, configuration, logging, and graceful shutdown consistently - then get out of the way for single-purpose logic. No "useful" functionality (no domain-specific code); just standardized foundations for deployment automation.

**CDRL Guide**: Users follow Clone → Degit → Refit → Launch; see `docs/development/fulmen_cdrl_guide.md` for details. During refit, users rename the tool identifier (e.g., `anvil` → `my-deployer`) throughout the codebase.

Implementers MUST comply with ecosystem standards in Crucible's `docs/standards/` (e.g., exit codes, logging conventions, signal handling) to ensure consistency.

## Language Constraints

**REQUIRED**: Microtools MUST be written in **Go, TypeScript/Bun, or Rust only**.

No other languages are permitted. This constraint ensures:

- Helper library availability (gofulmen, tsfulmen, rsfulmen)
- Single-binary compilation capabilities
- Consistent ecosystem tooling

**Helper Library Import**:

- **SHOULD** import helper library (gofulmen, tsfulmen, or rsfulmen)
- **If NOT imported**: MUST NOT recreate any helper functionality
  - Exit codes, logger, signals, config loading must be implemented independently
  - Cannot copy or replicate helper library patterns
  - Must still obey Fulmen standards (exit codes, logging format, signal handling)

**Rationale**: Importing the helper library is the easiest path to compliance. Implementing independently is permitted but requires significantly more work to meet standards.

## Required Library Modules

Microtool forges that import the helper library MUST integrate these modules to ensure ecosystem compliance. All modules are accessed via the language-specific helper library (gofulmen, pyfulmen, tsfulmen) - no direct Crucible dependencies.

**Note**: If a microtool does NOT import the helper library, it must implement equivalent functionality independently while adhering to the same standards.

### Core Identity & Configuration Modules

1. **App Identity Module** (REQUIRED)
   - **Purpose**: Standardized application metadata (binary name, vendor, environment prefix)
   - **Spec**: [App Identity Module](../standards/library/modules/app-identity.md)
   - **Compliance**: MUST implement `.fulmen/app.yaml` with:
     - `binary_name`: Tool name for template (users rename during CDRL refit)
     - `vendor`: Default `fulmenhq` (users customize)
     - `env_prefix`: Environment variable prefix (e.g., `GIMLET_`)
     - `config_name`: Config directory name (usually same as binary_name)
   - **Helper API**: `appidentity.Get(ctx)` → `(*appidentity.Identity, error)`
   - **Identity fields** (not methods): `identity.BinaryName`, `identity.Vendor`, `identity.EnvPrefix`, `identity.ConfigName`, `identity.Description`
   - **Example**:

     ```go
     import "github.com/fulmenhq/gofulmen/appidentity"

     identity, err := appidentity.Get(ctx)
     if err != nil {
         return fmt.Errorf("failed to load app identity: %w", err)
     }

     // Fields, not methods
     fmt.Println(identity.BinaryName)   // ✅ Correct
     fmt.Println(identity.BinaryName()) // ❌ Compile error
     ```

   - **CDRL Workflow**: Users update `.fulmen/app.yaml` FIRST, then run `make validate-app-identity`
   - **Distributed artifact requirement**: Microtools MUST embed identity so the binary can run from any directory (see [App Identity Module](../standards/library/modules/app-identity.md))
   - **Required Make targets**: forges MUST provide `make sync-embedded-identity` and `make verify-embedded-identity` (see [Fulmen Template CDRL Standard](fulmen-template-cdrl-standard.md))

2. **Crucible Shim Module** (CONDITIONAL)
   - **Purpose**: Access Crucible SSOT assets (schemas, standards, documentation, configs, taxonomies) without requiring crucible checkout
   - **Spec**: [Crucible Shim](../standards/library/modules/crucible-shim.md)
   - **REQUIRED if**: Microtool validates schemas, reads taxonomies, accesses standards/docs, or needs any SSOT assets
   - **Examples**:
     - Schema validators
     - Taxonomy tools
     - Config generators
     - **Commit validation hooks** (accessing agentic-attribution standard)
     - **Documentation generators** (accessing coding standards)
     - **CI compliance checkers** (accessing architecture standards)
     - **Template refit automation** (accessing CDRL workflow)
   - **Skip if**: Tool has no dependencies on Crucible assets (most microtools like gimlet)
   - **Rationale**: Provides offline access without network fragility, local checkout requirements, or vendoring violations
   - **Compliance**:
     - **Schemas**: `crucible.GetSchema()`, `crucible.GetLoggingEventSchema()`
     - **Configs**: `crucible.GetConfig()`
     - **Documentation**: `crucible.GetDoc("standards/agentic-attribution.md")`
     - **Standards**: `crucible.GetGoStandards()`, `crucible.GetTypeScriptStandards()`
     - **Discovery**: `crucible.ListAssets("docs", "standards/")`
     - **Version tracking**: `crucible.GetVersion()`

   **Example - Accessing Standards Documentation:**

   ```go
   import "github.com/fulmenhq/gofulmen/crucible"

   // Get agentic attribution standard for commit validation
   attrStandard, err := crucible.GetDoc("standards/agentic-attribution.md")
   if err != nil {
       logger.Error("failed to load attribution standard", zap.Error(err))
       return foundry.ExitFailure
   }

   // Validate commit message format against standard
   if err := validateCommitAttribution(commitMsg, attrStandard); err != nil {
       logger.Error("commit message validation failed", zap.Error(err))
       return foundry.ExitFailure
   }
   ```

   **Sustainability Note**: Always access Crucible assets through the helper library shim rather than filesystem reads. This ensures:
   - No requirement for local crucible checkout
   - Version-tracked through helper library dependency
   - Offline access to embedded assets
   - Consistent asset access across all ecosystem tools

   **Cross-language pattern**:
   - **Go**: `crucible.GetDoc("path/to/doc.md")`
   - **Python**: `crucible.get_doc("path/to/doc.md")`
   - **TypeScript**: `crucible.getDoc("path/to/doc.md")`

   See [Crucible Shim - Accessing General Documentation](../standards/library/modules/crucible-shim.md#accessing-general-documentation) for comprehensive examples including commit hooks, doc generators, and CI validators.

3. **Simple Config Pattern** (REQUIRED)
   - **Purpose**: Two-layer configuration for single-purpose microtools
   - **Pattern**: Defaults → User Overrides (env vars, flags, optional config file)
   - **Microtools DO NOT use Enterprise Three-Layer Config** (see [Enterprise Three-Layer Config](../standards/library/modules/enterprise-three-layer-config.md))
   - **Rationale**:
     - Microtools don't need Crucible SSOT defaults (Layer 1 of three-layer config)
     - Single-purpose tools need simple, self-contained configuration
     - Enterprise Three-Layer Config is for complex applications requiring ecosystem-wide standardization
   - **Standard Env Vars** (RECOMMENDED):
     - `{PREFIX}LOG_LEVEL` - Log level (trace|debug|info|warn|error, default: info)
     - `{PREFIX}CONFIG_PATH` - Config file path override
   - **Precedence**: CLI flags → Env vars → Config file → Defaults
   - **Example**:

     ```go
     // Simple config loading for microtools
     func LoadConfig(ctx context.Context, identity *appidentity.Identity, logger *logging.Logger) (*Config, error) {
         // 1. Set defaults
         cfg := &Config{
             InputPath:  ".",
             MaxDepth:   10,
         }

         // 2. Try to load optional user config file if exists
         configPath := filepath.Join(os.UserHomeDir(), ".config", identity.Vendor, identity.ConfigName+".yaml")
         if _, err := os.Stat(configPath); err == nil {
             data, _ := os.ReadFile(configPath)
             yaml.Unmarshal(data, cfg)
             logger.Info("loaded config from file", zap.String("path", configPath))
         }

         // 3. Env var overrides
         if envInput := os.Getenv(identity.EnvPrefix + "INPUT_PATH"); envInput != "" {
             cfg.InputPath = envInput
         }

         return cfg, nil
     }
     ```

   - **Graduation criteria**: If your microtool needs Enterprise Three-Layer Config → promote to `cli` category

4. **Config Path API Module** (OPTIONAL - only if using config files)
   - **Purpose**: Discover Fulmen config directories for optional user config
   - **Spec**: [Config Path API](../standards/library/modules/config-path-api.md)
   - **Compliance**: Use `get_app_config_dir({app_name})` from App Identity for user config path construction
   - **When to skip**: Tool doesn't support user config files

### Observability & Resilience Modules

5. **Logging Module** (REQUIRED)
   - **Purpose**: Structured logging with Crucible schema compliance
   - **Spec**: [Observability Logging](../standards/observability/logging.md)
   - **Compliance**:
     - Use SIMPLE or STRUCTURED profile from Crucible logging schemas
     - Tool name from App Identity (`binary_name`)
     - Output to stderr (not stdout - keep stdout clean for data)
   - **Helper API**: `logging.NewCLI(serviceName)` → `(*logging.Logger, error)` for simple CLI tools
   - **Logging calls**: Use `zap.Field` helpers: `zap.String()`, `zap.Int()`, `zap.Error()`
   - **Log level constants**: `logging.TRACE`, `logging.DEBUG`, `logging.INFO`, `logging.WARN`, `logging.ERROR`
   - **Example**:

     ```go
     import (
         "github.com/fulmenhq/gofulmen/logging"
         "go.uber.org/zap"
     )

     logger, err := logging.NewCLI(identity.BinaryName)
     if err != nil {
         return err
     }

     logger.Info("processing file", zap.String("path", filepath), zap.Int("size", size))
     logger.Error("operation failed", zap.Error(err))

     // Set level
     logger.SetLevel(logging.DEBUG)
     ```

6. **Exit Code Module** (REQUIRED)
   - **Purpose**: Standardized exit codes for automation
   - **Spec**: [Exit Code Taxonomy](../standards/exit-codes.md)
   - **Compliance**: Use `foundry.ExitSuccess`, `foundry.ExitFailure`, `foundry.ExitConfigInvalid`, `foundry.ExitInvalidArgument`
   - **Exit**: Call `os.Exit(exitCode)` directly or use foundry helpers
   - **Critical**: Microtools are often invoked by CI/CD - consistent exit codes enable proper error handling

7. **Signal Handling Module** (REQUIRED)
   - **Purpose**: Graceful shutdown on SIGTERM/SIGINT
   - **Spec**: [Signal Handling](../standards/library/modules/signals.md)
   - **Compliance**:
     - Trap SIGTERM, SIGINT
     - Clean up resources (close files, network connections)
     - Exit with appropriate code
   - **Helper API**: `signals.OnShutdown()` for shutdown hooks, `signals.EnableDoubleTap()` for double-tap Ctrl+C, `signals.Listen()` to start listener
   - **Example**:

     ```go
     import "github.com/fulmenhq/gofulmen/signals"

     // Register shutdown hooks
     signals.OnShutdown(func(ctx context.Context) error {
         logger.Info("shutting down gracefully")
         return cleanup(ctx)
     })

     // Enable double-tap Ctrl+C
     signals.EnableDoubleTap(signals.DoubleTapConfig{})

     // Start listener (blocks or run in goroutine)
     go func() {
         if err := signals.Listen(ctx); err != nil {
             logger.Error("signal listener failed", zap.Error(err))
         }
     }()
     ```

8. **Error Handling Module** (REQUIRED)
   - **Purpose**: Structured error propagation
   - **Spec**: [Error Handling](../standards/library/modules/error-handling.md)
   - **Compliance**: Wrap errors with context, use exit codes appropriately

9. **Pathfinder Integration Module** (REQUIRED)
   - **Purpose**: Safe path discovery, checksum calculation, and file selection logic
   - **Spec**: [Pathfinder Extension](../standards/library/extensions/pathfinder.md)
   - **Compliance**:
     - Use `github.com/fulmenhq/gofulmen/pathfinder` (Go) or the language equivalent for all filesystem traversal
     - Expose CLI flags mapping to Pathfinder’s `FindQuery` fields (root, include, exclude, depth, checksum)
     - Never shell out to `find`/`ls`; rely on Pathfinder to enforce traversal guardrails
     - When checksums are needed, use the built-in FulHash integration (`ChecksumAlgorithm`, `CalculateChecksums`)
   - **Rationale**: Microtools routinely inspect files; mandatory Pathfinder usage keeps traversal aligned with Fulmen security policy.

### Optional Modules

10. **Telemetry/Metrics Module** (OPTIONAL)

- **When to use**: If tool runs long enough to benefit from metrics
- **Typically skip**: Most microtools are short-lived CLI operations

11. **Schema Validation Module** (OPTIONAL)
    - **When to use**: If tool reads/writes structured data
    - **Example**: Fixture manifest validation

> **Documentation requirement**: Optional integrations (Crucible shim, telemetry, schema validation, etc.) MUST be documented in `docs/development/fulmen_cdrl_guide.md` with clear “keep vs. remove” guidance for teams refitting the template.

### Module Summary Table

| Module            | Status      | Purpose                            | Typical Use Case                  | Spec Link                                                                                   |
| ----------------- | ----------- | ---------------------------------- | --------------------------------- | ------------------------------------------------------------------------------------------- |
| App Identity      | REQUIRED    | Binary name, env prefix, metadata  | All microtools                    | [app-identity.md](../standards/library/modules/app-identity.md)                             |
| Simple Config     | REQUIRED    | Two-layer config (defaults + user) | All microtools                    | See section above                                                                           |
| Logging           | REQUIRED    | Structured logging                 | All microtools                    | [logging.md](../standards/observability/logging.md)                                         |
| Exit Code         | REQUIRED    | Standardized exit codes            | All microtools (CI/CD)            | [exit-codes/README.md](../standards/fulmen/exit-codes/README.md)                            |
| Signal Handling   | REQUIRED    | Graceful shutdown                  | All microtools                    | [signal-handling.md](../standards/library/modules/signal-handling.md)                       |
| Error Handling    | REQUIRED    | Structured error propagation       | All microtools                    | [error-handling-propagation.md](../standards/library/modules/error-handling-propagation.md) |
| Pathfinder        | REQUIRED    | Safe discovery & checksum data     | Any filesystem interaction        | [pathfinder.md](../standards/library/extensions/pathfinder.md)                              |
| Crucible Shim     | CONDITIONAL | SSOT asset access                  | Schema validators, taxonomy tools | [crucible-shim.md](../standards/library/modules/crucible-shim.md)                           |
| Config Path API   | OPTIONAL    | Config directory discovery         | Tools with optional config files  | [config-path-api.md](../standards/library/modules/config-path-api.md)                       |
| Schema Validation | OPTIONAL    | Data validation                    | Tools reading/writing YAML/JSON   | [schema-validation.md](../standards/library/modules/schema-validation.md)                   |
| Telemetry/Metrics | OPTIONAL    | Metrics export                     | Long-running operations (rare)    | [telemetry-metrics.md](../standards/library/modules/telemetry-metrics.md)                   |

**Key Differences from Workhorse**:

- ❌ No server management module (microtools don't have HTTP endpoints)
- ❌ No Docscribe module (microtools don't serve documentation)
- ❌ NO Enterprise Three-Layer Config (microtools use Simple Config Pattern)
- ✅ Crucible Shim is CONDITIONAL (only for schema validators, taxonomy tools)
- ✅ Config Path API is OPTIONAL (not REQUIRED - only if using config files)
- ✅ Telemetry/Metrics is OPTIONAL (most microtools are short-lived CLI operations)

## Prohibited Features

Microtools MUST NOT include:

- ❌ **Web server endpoints** (HTTP, gRPC) - use `workhorse` or `service` category instead
- ❌ **Long-running daemons** - use `workhorse` or `service` category instead
- ❌ **Exportable packages** - no `pkg/` directory, cannot be imported by other repos
- ❌ **Multi-purpose functionality** - keep single purpose or promote to `cli`
- ❌ **Recreated helper functionality** - if not importing helper, implement independently

## Directory Structure

**Standard Layout**:

```
fulmen-{toolname}/                    # e.g., fulmen-fixtures
├── .fulmen/
│   └── app.yaml                      # App Identity manifest (REQUIRED)
├── cmd/
│   └── {toolname}/                   # e.g., anvil/
│       ├── main.go                   # Entry point
│       ├── generate.go               # Subcommand: generate
│       ├── deploy.go                 # Subcommand: deploy
│       └── verify.go                 # Subcommand: verify
├── internal/                         # Internal implementation (not exported)
│   ├── config/
│   │   └── config.go                 # Tool-specific config
│   ├── client/
│   │   └── storage.go                # S3/storage client
│   └── generator/
│       └── generator.go              # Core generation logic
├── configs/                          # Default configs, recipes, templates
│   └── example.yaml
├── .github/
│   └── workflows/
│       ├── ci.yml                    # Build, test, lint
│       └── release.yml               # Binary releases
├── Makefile                          # Standard targets (see below)
├── go.mod                            # Dependencies (imports gofulmen)
├── go.sum
├── README.md                         # Usage overview, links to docs/development guide
├── LICENSE                           # MIT recommended
└── docs/
    └── development/
        └── fulmen_cdrl_guide.md      # CDRL workflow + optional-module decisions (REQUIRED)
```

**Key Points**:

- NO `pkg/` directory - microtools are not libraries
- ALL implementation in `internal/` (not exported)
- CLI commands in `cmd/{toolname}/` with subcommands as separate files
- Configs/recipes in `configs/` or similar (tool-specific)

**TypeScript/Bun Equivalent**:

```
fulmen-{toolname}/
├── .fulmen/app.yaml
├── src/
│   ├── commands/
│   │   ├── generate.ts
│   │   ├── deploy.ts
│   │   └── verify.ts
│   ├── lib/
│   │   ├── config.ts
│   │   ├── client.ts
│   │   └── generator.ts
│   └── index.ts
├── configs/
├── package.json                      # Imports tsfulmen
├── tsconfig.json
├── README.md
├── LICENSE
└── docs/development/fulmen_cdrl_guide.md
```

## Makefile Standard Targets

**REQUIRED Targets**:

```makefile
# Build
.PHONY: build
build: ## Build binary
	go build -o bin/{toolname} cmd/{toolname}/main.go

# Install locally
.PHONY: install
install: build ## Install to $GOPATH/bin
	go install cmd/{toolname}/main.go

# Test
.PHONY: test
test: ## Run tests
	go test -v ./...

# Lint
.PHONY: lint
lint: ## Run linter
	golangci-lint run

# Format
.PHONY: fmt
fmt: ## Format code
	gofmt -s -w .

# App Identity validation
.PHONY: validate-app-identity
validate-app-identity: ## Validate app identity configuration
	@echo "Checking for hardcoded references to binary name..."
	# Implementation via helper library or script

# Doctor check (CDRL requirement)
.PHONY: doctor
doctor: ## Run health checks
	@echo "Running microtool health checks..."
	@which go || (echo "ERROR: Go not installed" && exit 1)
	@test -f .fulmen/app.yaml || (echo "ERROR: Missing .fulmen/app.yaml" && exit 1)
	@echo "✅ All checks passed"

# Help
.PHONY: help
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "%-20s %s\n", $$1, $$2}'

.DEFAULT_GOAL := help
```

## CLI Command Structure

**Pattern**: Single binary with subcommands

```bash
# Tool name = binary name
{toolname} [global-flags] <command> [command-flags] [arguments]

# Examples
fulmen-fixtures generate --recipe recipe.yaml --output ./fixtures/
fulmen-fixtures deploy --recipe recipe.yaml --target s3://bucket/path
fulmen-fixtures verify --recipe recipe.yaml --source https://example.com
```

### Required Commands

1. **`version`**
   - MUST support an `--extended/-e` flag that prints commit SHA, build date, Go runtime, and helper versions via `crucible.GetVersion()`.
   - Default output: `<binary> <semver>`.
2. **`envinfo`**
   - Summarizes app identity, helper dependency versions, runtime details, and the currently loaded config file.
   - Logs exclusively to stderr using the helper logger.
3. **`doctor`**
   - Performs environment diagnostics (Go version, access to Crucible/gofulmen, config directory health).
   - Returns Fulmen exit codes via `foundry.Exit*`.
   - Serves as the CLI counterpart to the Makefile `doctor` target.

### Global Flags (REQUIRED)

- `--config` – Config file path override
- `--log-level` – Log level override (maps to helper logging severity)
- `--help`
- `--version` (auto-exposed; ensure subcommands honor it)

**Output Conventions**:

- **stdout**: Primary data output (machine-readable)
- **stderr**: Logs, diagnostics, progress (human-readable)
- **exit code**: Status (0 = success, non-zero = error)

**Examples**:

```bash
# Good: Data to stdout, logs to stderr
$ anvil generate --recipe r.yaml > output.json
2025-11-15T12:00:00Z INFO: Generating from recipe
2025-11-15T12:00:01Z INFO: Generated 10 items

# Good: Clean exit codes
$ anvil deploy --target invalid://url
2025-11-15T12:00:00Z ERROR: Invalid target URL
$ echo $?
70  # EXIT_CONFIG_ERROR

# Bad: Mixing logs with data on stdout (DON'T DO THIS)
$ anvil generate --recipe r.yaml  # Logs on stdout = unparseable
```

## Dependencies

**Helper Library** (RECOMMENDED):

```go
// go.mod
require (
    github.com/fulmenhq/gofulmen v0.x.x  // Core helper
)
```

**Common Dependencies** (as needed):

- CLI framework: `github.com/spf13/cobra` (Go) or similar
- Config parsing: `gopkg.in/yaml.v3`, `github.com/BurntSushi/toml`
- HTTP client: Standard library or `github.com/aws/aws-sdk-go-v2` (if S3)
- Testing: Standard library test framework

**Prohibited**:

- ❌ Web frameworks (chi, gin, echo, etc.)
- ❌ Heavy ORMs
- ❌ UI libraries (this is CLI only)

## Testing Requirements

**Unit Tests**:

- Test internal logic (generators, parsers, validators)
- Use helper library's test fixtures
- Aim for >70% coverage

**Integration Tests**:

- Test against real endpoints when possible (S3-compatible storage, etc.)
- Use environment variables for credentials (not hardcoded)
- Mark as integration tests (can skip in CI if credentials unavailable)

**CLI Tests**:

- Test command invocation
- Verify exit codes
- Check stdout/stderr output

Microtool suites must observe the cross-language [Portable Testing Practices](../standards/testing/portable-testing-practices.md). Provide shared skip helpers (network, credentials, filesystem) so tests behave deterministically across CI, sandboxes, and contributor laptops.

**Example** (Go):

```go
func TestGenerateCommand(t *testing.T) {
    cmd := NewGenerateCmd()
    cmd.SetArgs([]string{"--recipe", "testdata/recipe.yaml"})

    err := cmd.Execute()
    if err != nil {
        t.Fatalf("command failed: %v", err)
    }

    // Verify output
}
```

## CDRL Workflow

**Clone → Degit → Refit → Launch**:

1. **Clone**: User clones template

   ```bash
   git clone https://github.com/fulmenhq/forge-microtool-anvil.git my-deployer
   cd my-deployer
   ```

2. **Degit**: Remove template git history

   ```bash
   rm -rf .git
   git init
   ```

3. **Refit**: Customize for their use case

   ```bash
   # Update .fulmen/app.yaml
   vim .fulmen/app.yaml  # Change binary_name, vendor, etc.

   # Validate and find hardcoded references
   make validate-app-identity

   # Rename tool throughout codebase (manual or script-assisted)
   # anvil → my-deployer
   ```

4. **Launch**: Build and use
   ```bash
   make build
   make install
   my-deployer --help
   ```

**Template Responsibilities**:

- Provide `docs/development/fulmen_cdrl_guide.md` with specific refit instructions (Clone → Degit → Refit → Launch plus optional-module guidance)
- Use App Identity for binary name parameterization
- Make refit process as smooth as possible

## Naming Conventions

**Template Naming**: `forge-microtool-{instrument}`

**Language-Matching Pattern**: First letter of instrument matches implementation language for human readability.

**Go (G-instruments)**:

- `gimlet` - Precision boring/ingest tool
- `gauge` - Measurement/validation tool
- `gouge` - Data extraction tool

**TypeScript (T-instruments)**:

- `tongs` - Data handling tool
- `tap` - Threading/integration tool
- `trammel` - Precision layout tool

**Rust (R-instruments)**:

- `rasp` - File processing tool
- `reamer` - Data refinement tool
- `router` - Path/routing tool

**Avoid**:

- Horse breeds (reserved for workhorse category)
- Generic names (tool, util, helper)
- Instruments not matching language first letter

## Security Considerations

**Credentials**:

- NEVER hardcode credentials
- Use environment variables or config files
- Document credential sources in README
- Support multiple cloud providers (AWS, Azure, GCP, etc.) via configuration

**Input Validation**:

- Validate all user inputs
- Sanitize file paths (prevent traversal)
- Validate URLs/endpoints
- Use schemas where applicable

**Network Operations**:

- Use TLS/HTTPS by default
- Validate certificates
- Timeout all operations (no infinite hangs)
- Handle network errors gracefully

## Documentation Requirements

**README.md** (REQUIRED):

- Tool purpose (one sentence)
- Installation instructions
- Usage examples
- Configuration options
- Credential setup
- CDRL guide reference

**docs/development/fulmen_cdrl_guide.md** (REQUIRED):

- Step-by-step refit instructions
- What to rename (binary name, imports, etc.)
- How to validate changes
- Explain optional integrations (Crucible shim, telemetry, schema validation, Pathfinder knobs) and how to keep/remove them
- Common pitfalls

**Command Help** (REQUIRED):

- Every command has `--help`
- Show examples
- Document flags clearly

**godoc / JSDoc** (RECOMMENDED):

- Document exported functions (even in internal/)
- Explain non-obvious logic
- Link to Crucible specs where applicable

## Release & Distribution

**Binary Releases**:

- GitHub Releases with artifacts
- Multiple platforms: Linux (amd64, arm64), macOS (amd64, arm64), Windows (amd64)
- Checksums provided (SHA256)

**Package Managers** (OPTIONAL):

- Homebrew tap (macOS/Linux)
- Scoop bucket (Windows)
- Docker image (if applicable)

**Versioning**:

- Semantic versioning (v1.2.3)
- Tag releases in git
- Maintain CHANGELOG.md

## Examples

**Reference Implementations**:

- `fulmen-fixtures` - Fixture deployment and generation (first microtool)

**Coming Soon**:

- `forge-microtool-anvil` - Template based on fulmen-fixtures

## Graduation Criteria

If a microtool grows beyond single purpose, it must be promoted:

**Indicators**:

- Multiple unrelated features added
- Becomes multi-purpose tool
- Scope creep beyond original purpose

**Action**:

- Promote to `cli` category
- Update repository-categories.yaml
- May require architectural review
- Notify maintainers

**Example**: If `fulmen-fixtures` adds schema validation, code generation, and docs building → too broad, promote to `cli`

## Compliance Checklist

- [ ] Written in Go, TypeScript/Bun, or Rust only
- [ ] Single primary purpose documented
- [ ] Imports helper library (gofulmen/tsfulmen/rsfulmen) OR implements standards independently
- [ ] Uses exit codes from helper (or compliant implementation)
- [ ] Structured logging to stderr
- [ ] Graceful signal handling (SIGTERM, SIGINT)
- [ ] Pathfinder integration for every filesystem traversal
- [ ] App Identity implemented (`.fulmen/app.yaml`)
- [ ] NO `pkg/` directory (not a library)
- [ ] NO web server endpoints
- [ ] Makefile with standard targets
- [ ] CDRL guide (`docs/development/fulmen_cdrl_guide.md`)
- [ ] Help text for all commands
- [ ] README with usage examples
- [ ] Tests (unit + integration)
- [ ] Repository builds/tests successfully before any refit instructions are applied

## Reference Implementation

The `forge-microtool-gimlet` (formerly grinder) repository provides a complete, working reference implementation demonstrating all required integrations:

**Key files to reference:**

- `cmd/gimlet/main.go` - Entry point with appidentity, logging, signals
- `internal/cmd/root.go` - Cobra command setup
- `internal/runtime/logging.go` - Logger initialization
- `internal/config/loader.go` - Simple config loading pattern
- `internal/example/processor.go` - Business logic with structured logging

**Validated against:**

- gofulmen v0.1.10
- forge-workhorse-groningen patterns
- Real build/test/run workflows

**URL**: `https://github.com/fulmenhq/forge-microtool-gimlet` (when published)

**Why this matters**: All API examples in this standard are verified working code from gimlet. When in doubt, reference the implementation.

## See Also

- [Repository Category Taxonomy](../../config/taxonomy/repository-categories.yaml)
- [Ecosystem Brand Summary](../../config/branding/ecosystem.yaml) - For `version --extended` or `about` command
- [Fulmen Template CDRL Standard](fulmen-template-cdrl-standard.md)
- [App Identity Module](../standards/library/modules/app-identity.md)
- [Exit Code Taxonomy](../standards/exit-codes.md)
- [Pathfinder Extension](../standards/library/extensions/pathfinder.md)
- [Observability Logging](../standards/observability/logging.md)
- [Signal Handling](../standards/library/modules/signals.md)
- [Enterprise Three-Layer Config](../standards/library/modules/enterprise-three-layer-config.md) (NOT used by microtools)

---

**Document Status**: Draft
**Last Updated**: 2025-11-15
**Maintained By**: Schema Cartographer
**Approval Required From**: EA Steward, Crucible Maintainers
