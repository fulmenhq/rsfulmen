---
title: "Application Identity Module Standard"
description: "Standardized application identity metadata for Fulmen ecosystem projects"
author: "Schema Cartographer"
date: "2025-11-03"
last_updated: "2025-12-23"
status: "approved"
version: "v1.0.0"
tags: ["app-identity", "modules", "configuration", "standards"]
---

# Application Identity Module Standard

## Purpose

Provide a single source of truth for application identity metadata across the Fulmen ecosystem. Every project—from helper libraries to production forges—derives binary names, vendor namespaces, environment-variable prefixes, and config directories from `.fulmen/app.yaml`.

## Background

Fulmen templates (groningen, percheron, future breeds) historically embedded binary names and vendor prefixes throughout code, docs, and tooling, requiring manual search-and-replace during CDRL refits. This led to:

- Hardcoded values scattered across 20+ locations
- Environment variable prefix inconsistencies
- Config directory path duplication
- Documentation drift during refits
- No validation to catch regressions

The App Identity module solves this by providing a canonical metadata file that helper libraries consume programmatically.

## Requirements

### For Crucible (SSOT)

1. **Schema**: Publish JSON Schema at `schemas/config/repository/app-identity/v1.0.0/app-identity.schema.json`
2. **Example**: Provide reference example at `config/repository/app-identity/app-identity.example.yaml`
3. **Fixtures**: Maintain valid/invalid test fixtures for cross-language parity
4. **Documentation**: This standard document
5. **Sync**: Schema syncs to helper libraries via standard sync pipeline

### For Helper Libraries (gofulmen, pyfulmen, tsfulmen)

1. **Load identity**: Discover and parse `.fulmen/app.yaml` from repository
2. **Validate**: Enforce schema rules at load time
3. **Cache**: Read-once per process, immutable thereafter
4. **Integrate**: Provide identity to config paths, logging, metrics, CLI modules
5. **Test utilities**: Support fixtures for unit testing without filesystem
6. **Parity**: Match behavior across all languages (Go, Python, TypeScript)

### For Applications

1. **Include**: Place `.fulmen/app.yaml` in repository root (or per-binary in monorepos)
2. **Use**: Consume identity via helper library APIs instead of hardcoded strings
3. **Validate**: Add `make validate-app-identity` to CI pipeline
4. **Document**: Reference identity file in CDRL refit instructions

## Schema Definition

**Location**: `schemas/config/repository/app-identity/v1.0.0/app-identity.schema.json`

**Required Fields** (`app` object):

| Field         | Type   | Pattern                          | Description                                               |
| ------------- | ------ | -------------------------------- | --------------------------------------------------------- |
| `binary_name` | string | `^[a-z][a-z0-9-]{0,62}[a-z0-9]$` | Lowercase kebab-case binary/executable name               |
| `vendor`      | string | `^[a-z0-9]{2,64}$`               | Lowercase alphanumeric vendor namespace (no hyphens)      |
| `env_prefix`  | string | `^[A-Z][A-Z0-9_]*_$`             | Uppercase environment variable prefix (must end with `_`) |
| `config_name` | string | `^[a-z][a-z0-9-]{0,62}[a-z0-9]$` | Filesystem-safe config directory name                     |
| `description` | string | 10-200 chars                     | One-line application description                          |

**Vendor Pattern Note**: The `vendor` field permits leading digits (e.g., `3leaps`, `37signals`, `8x8`) because vendor names are used for filesystem paths and configuration directories—not as language-specific package identifiers. See [Vendor Pattern Cross-Language Safety](#vendor-pattern-cross-language-safety) for details.

**Optional Fields** (`metadata` object):

| Field                 | Type   | Description                                                                           |
| --------------------- | ------ | ------------------------------------------------------------------------------------- |
| `project_url`         | URI    | Primary project URL (repository, docs)                                                |
| `support_email`       | email  | Support/contact email                                                                 |
| `license`             | string | SPDX license identifier (MIT, Apache-2.0, etc.)                                       |
| `repository_category` | enum   | From Fulmen taxonomy: cli, workhorse, service, library, pipeline, codex, sdk          |
| `telemetry_namespace` | string | Namespace for metrics/logging (defaults to binary_name)                               |
| `registry_id`         | UUID   | Optional UUIDv7 for future registry (experimental)                                    |
| `python`              | object | Python-specific packaging metadata (distribution_name, package_name, console_scripts) |
| _(custom)_            | any    | Additional properties allowed for extensibility                                       |

**Full schema**: See `schemas/config/repository/app-identity/v1.0.0/app-identity.schema.json`

## File Location

**Single-binary applications**:

```
myapp/
├── .fulmen/
│   └── app.yaml              # Identity for myapp
├── src/
└── README.md
```

**Monorepo applications** (multiple binaries):

```
myproject/
├── .fulmen/
│   └── app.yaml              # Shared/default identity
├── cmd/
│   ├── api/
│   │   └── .fulmen/app.yaml  # API-specific identity
│   └── worker/
│       └── .fulmen/app.yaml  # Worker-specific identity
└── lib/                      # Uses nearest parent identity
```

**Library + CLI** (common in Python):

```
pyfulmen/
├── .fulmen/
│   └── app.yaml              # Single identity for both
├── src/pyfulmen/             # Library code
│   └── cli.py                # CLI entry point
└── pyproject.toml            # Derives from identity
```

## Discovery Precedence

Helper libraries MUST follow this discovery order:

1. **Explicit path parameter**: `LoadFrom(path)` / `load_identity(path=...)` / `loadFrom(path)`
   - Highest priority: Caller explicitly specifies path
   - Use case: Tests, multi-binary explicit selection
   - Behavior: Error if file doesn't exist

2. **Environment variable override**: `FULMEN_APP_IDENTITY_PATH`
   - Second priority: Environment explicitly specifies path
   - Use case: CI/CD, containers, deployment overrides
   - Behavior: Error if file doesn't exist

3. **Filesystem discovery**: Walk upward from CWD
   - Third priority: Search from `os.Getcwd()` / `process.cwd()` upward
   - Stop at first `.fulmen/app.yaml` found
   - Walk to filesystem root (or max 20 levels)
   - Behavior: Error if not found (list searched paths)
   - Optional: Implementations MAY include an executable-directory fallback as part of filesystem discovery, but it MUST run after the CWD ancestor walk.

4. **Embedded identity fallback** (REQUIRED for distributed artifacts)
   - Used only when explicit path/env var are not set and filesystem discovery fails
   - Ensures standalone binaries/packages know their identity outside the repo

5. **Test/fixture injection**: Via test utilities only
   - `WithIdentity(ctx, fixture)` (Go)
   - `override_identity(fixture)` (Python)
   - `withTestIdentity(fixture, fn)` (TypeScript)
   - Never used in production code

## Embedded Identity Fallback (Distributed Artifacts)

### Requirements

- `.fulmen/app.yaml` MUST NOT contain secrets.
- Helper libraries MUST support an embedded identity fallback mechanism for distributed artifacts (binaries, packages, container entrypoints).
- Applications/templates MUST embed identity at build/package time.
- Embedded identity MUST be treated as build-time provenance and read-only at runtime.

### Drift Prevention (Required for Templates)

Templates MUST provide build targets to keep the embedded mirror in sync:

- `make sync-embedded-identity`
  - Copies `.fulmen/app.yaml` to an embeddable mirror path (language-specific)
  - Fails if `.fulmen/app.yaml` is missing
- `make verify-embedded-identity`
  - Fails if the embedded mirror differs from `.fulmen/app.yaml`
  - SHOULD run as part of `make test`, `make precommit`, and CI

### Acceptance Criteria (App/Template Layer)

- **AC1 — Standalone binary works outside repo**
  - `make build`
  - `cp ./bin/<tool> /tmp/<tool>`
  - `/tmp/<tool> version`
  - Must succeed without any `.fulmen/app.yaml` on disk.
- **AC2 — `--help` must not depend on external identity**
  - same copy-to-`/tmp` pattern, run `/tmp/<tool> --help`
- **AC3 — Drift prevention**
  - `.fulmen/app.yaml` and the embedded mirror (e.g., `internal/assets/appidentity/app.yaml`) must be identical
  - `make verify-embedded-identity` must fail when they differ

## Dependency Layering

To prevent import cycles, app identity is Layer 0 with zero Fulmen module dependencies:

```
Layer 0: appidentity (pure metadata, stdlib + YAML parser only)
         ↓
Layer 1: configpaths (consumes appidentity for vendor/app slugs)
         ↓
Layer 2: three-layer config, logging, telemetry (consume both)
```

**Rules**:

- `appidentity` MUST NOT import from `config`, `configpaths`, `logging`, or `telemetry`
- Higher layers consume identity via explicit parameters, not global state
- Config modules accept `Identity` struct as optional parameter (graceful degradation)

## Runtime Caching

All language implementations MUST follow these caching guarantees:

1. **Read once per process**: First load reads from filesystem
2. **Cached thereafter**: Subsequent calls return cached instance (no filesystem I/O)
3. **Cache scope**: Process-level (not global/shared across processes)
4. **Thread/concurrency safe**: Cached identity is immutable
5. **No hot-reload**: Changes to `.fulmen/app.yaml` NOT reflected until process restart
6. **Test utilities bypass cache**: Inject fixtures without touching cache

## Library Implementation

### Go (`gofulmen/appidentity`)

```go
package main

import (
    "context"
    "github.com/fulmenhq/gofulmen/appidentity"
    "github.com/fulmenhq/gofulmen/configpaths"
)

func main() {
    ctx := context.Background()

    // Load identity (cached after first call)
    identity, err := appidentity.Get(ctx)
    if err != nil {
        log.Fatal(err)
    }

    // Use for config paths
    configDir, _ := configpaths.GetAppConfigDir(identity.Vendor, identity.ConfigName)

    // Use for environment variables
    logLevel := os.Getenv(identity.EnvPrefix + "LOG_LEVEL")
}
```

**Testing**:

```go
func TestFeature(t *testing.T) {
    fixture := appidentity.Identity{
        BinaryName: "testapp",
        Vendor: "testvendor",
        EnvPrefix: "TESTAPP_",
        ConfigName: "testapp",
        Description: "Test app",
    }
    ctx := appidentitytest.WithIdentity(context.Background(), fixture)

    // Test code using identity
    result := myFeature(ctx)
    assert.NotNil(t, result)
}
```

### Python (`pyfulmen.appidentity`)

```python
from pyfulmen.appidentity import get_identity
from pyfulmen.configpaths import get_app_config_dir

def main():
    # Load identity (cached after first call)
    identity = get_identity()

    # Use for config paths
    config_dir = get_app_config_dir(identity.vendor, identity.config_name)

    # Use for environment variables
    log_level = os.getenv(f"{identity.env_prefix}LOG_LEVEL", "INFO")
```

**Testing**:

```python
from pyfulmen.appidentity import override_identity, AppIdentity

def test_feature():
    fixture = AppIdentity(
        binary_name="testapp",
        vendor="testvendor",
        env_prefix="TESTAPP_",
        config_name="testapp",
        description="Test app"
    )

    with override_identity(fixture):
        result = my_feature()
        assert result is not None
```

### TypeScript (`@fulmenhq/crucible/appidentity`)

```typescript
import { loadIdentity } from "@fulmenhq/crucible/appidentity";
import { getAppConfigDir } from "@fulmenhq/crucible/configpaths";

async function main() {
  // Load identity (cached after first call)
  const identity = await loadIdentity();

  // Use for config paths
  const configDir = await getAppConfigDir(identity.vendor, identity.configName);

  // Use for environment variables
  const logLevel = process.env[`${identity.envPrefix}LOG_LEVEL`] || "INFO";
}
```

**Testing**:

```typescript
import { withTestIdentity } from "@fulmenhq/crucible/appidentity";

describe("my feature", () => {
  it("should work", async () => {
    const fixture = {
      binaryName: "testapp",
      vendor: "testvendor",
      envPrefix: "TESTAPP_",
      configName: "testapp",
      description: "Test app",
    };

    await withTestIdentity(fixture, async () => {
      const result = await myFeature();
      expect(result).toBeDefined();
    });
  });
});
```

## Validation

### Schema Validation

```bash
# Validate identity file against schema
goneat schema validate-data \
  --schema schemas/config/repository/app-identity/v1.0.0/app-identity.schema.json \
  --data .fulmen/app.yaml
```

### Project Validation

Add to Makefile:

```makefile
.PHONY: validate-app-identity
validate-app-identity:
\t@echo "Validating app identity..."
\t@goneat schema validate-data \
\t  --schema $(CRUCIBLE)/schemas/config/repository/app-identity/v1.0.0/app-identity.schema.json \
\t  --data .fulmen/app.yaml
\t@echo "✓ App identity valid"
```

### CI Integration

```yaml
# .github/workflows/validate.yml
- name: Validate App Identity
  run: make validate-app-identity
```

## Testing Requirements

### Parity Tests

All language implementations MUST pass parity tests against canonical snapshot:

**Location**: `config/repository/app-identity/parity-snapshot.json`

**Test**: Load each fixture, compare output to snapshot expected values

```go
// Go
func TestParitySnapshot(t *testing.T) {
    snapshot := loadParitySnapshot("config/repository/app-identity/parity-snapshot.json")
    for name, tc := range snapshot.Valid {
        identity, err := appidentity.LoadFrom(tc.InputFile)
        require.NoError(t, err)
        assert.Equal(t, tc.ExpectedOutput, toMap(identity))
    }
}
```

### Test Coverage

Each language MUST cover:

- ✅ Valid fixtures pass validation
- ✅ Invalid fixtures fail with specific errors
- ✅ Discovery algorithm (CWD, parent walk, env override)
- ✅ Caching behavior
- ✅ Concurrent access safety
- ✅ Test utilities work correctly
- ✅ Integration with config paths
- ✅ Parity with other languages

## Examples

### Complete Example

See `config/repository/app-identity/app-identity.example.yaml`

### Minimal Example

```yaml
app:
  binary_name: myapp
  vendor: acmecorp
  env_prefix: MYAPP_
  config_name: myapp
  description: My application description
```

### With Python Packaging

```yaml
app:
  binary_name: pyfulmen
  vendor: fulmenhq
  env_prefix: PYFULMEN_
  config_name: pyfulmen
  description: Python Fulmen helper library and CLI tools

metadata:
  license: MIT
  python:
    distribution_name: pyfulmen
    package_name: pyfulmen
    console_scripts:
      - name: pyfulmen
        entry_point: pyfulmen.cli:main
```

## Error Handling

### Common Errors

**Identity Not Found**:

```
Error: App identity not found
  Searched paths:
    - /Users/dev/myproject/.fulmen/app.yaml
    - /Users/dev/.fulmen/app.yaml
    - /Users/.fulmen/app.yaml

  Solution: Create .fulmen/app.yaml in your repository root
  Documentation: https://docs.fulmenhq.io/app-identity
```

**Schema Validation Failure**:

```
Error: Invalid app identity: /path/to/.fulmen/app.yaml
  Validation errors:
    - app.env_prefix: must end with underscore (got: "MYAPP")
    - app.binary_name: must be lowercase (got: "MyApp")

  Documentation: https://docs.fulmenhq.io/app-identity#validation
```

### Best Practices

**For applications**:

- Fail fast at startup if identity missing/invalid
- Provide actionable error messages with documentation links
- Include searched paths in "not found" errors

**For libraries**:

- Return detailed errors (not panic/exit)
- Support optional identity (graceful degradation)
- Cache results to avoid repeated file I/O

**For tests**:

- Always inject identity explicitly (don't rely on filesystem)
- Use test utilities to avoid test interference
- Validate error messages in negative test cases

## CDRL Workflow

### Before (Manual Search/Replace)

1. Search for hardcoded `percheron` → replace with `newapp`
2. Search for `PERCHERON_` → replace with `NEWAPP_`
3. Update config directory paths
4. Update README, docs, `.env` files
5. Hope you didn't miss any references

### After (Identity-Driven)

1. Edit `.fulmen/app.yaml` with new values
2. Run `make validate-app-identity`
3. Libraries automatically use new identity
4. Regenerate docs if needed

**All references stay synchronized automatically.**

## Vendor Pattern Cross-Language Safety

The `vendor` field pattern (`^[a-z0-9]{2,64}$`) permits leading digits. This section documents why this is safe across all supported languages.

### Why Leading Digits Are Safe

The `vendor` field is used for:

1. **Filesystem paths**: `~/.config/<vendor>/` - All filesystems accept leading digits
2. **Configuration directories**: No language restrictions apply
3. **Namespace grouping**: Organizational identifier, not a code symbol

The `vendor` field is **NOT** used for:

- Go package names (use `binary_name` or module path)
- Python import identifiers (use `metadata.python.package_name`)
- TypeScript/npm package names (use separate package.json configuration)

### Language-Specific Analysis

| Language   | Concern                              | Vendor Impact | Notes                               |
| ---------- | ------------------------------------ | ------------- | ----------------------------------- |
| Go         | Module paths can start with digits   | Safe          | `github.com/3leaps/pkg` works       |
| Python     | Identifiers cannot start with digits | Safe          | Vendor not used as import name      |
| TypeScript | npm scopes can start with digits     | Safe          | `@3leaps/pkg` is valid              |
| Rust       | Crate names can start with digits    | Safe          | Cargo accepts numeric prefixes      |
| Filesystem | All platforms accept leading digits  | Safe          | `~/.config/3leaps/` works on all OS |

### Real-World Examples

Valid vendor names with leading digits:

- `3leaps` - 3 Leaps, LLC
- `37signals` - Basecamp/HEY creators
- `8x8` - Communications platform
- `1password` - Password manager vendor

### Related Fields

Fields that **do** require alphabetic first character:

| Field                          | Pattern                          | Reason                         |
| ------------------------------ | -------------------------------- | ------------------------------ |
| `binary_name`                  | `^[a-z][a-z0-9-]{0,62}[a-z0-9]$` | Shell command conventions      |
| `env_prefix`                   | `^[A-Z][A-Z0-9_]*_$`             | Environment variable standards |
| `config_name`                  | `^[a-z][a-z0-9-]{0,62}[a-z0-9]$` | Consistent with binary_name    |
| `metadata.python.package_name` | `^[a-z][a-z0-9_]{0,62}[a-z0-9]$` | Python identifier requirements |
| `metadata.telemetry_namespace` | `^[a-z][a-z0-9_]{0,62}[a-z0-9]$` | Metrics system compatibility   |

## Security Considerations

1. **Path traversal protection**: Validate paths, no `../` escapes
2. **YAML parsing safety**: Use safe loaders, limit file size (< 10KB)
3. **Environment injection**: Prefix already validated by schema
4. **File permissions**: Identity files should be world-readable (0644), no secrets

## Migration Plan

**Phase 1: Crucible (v0.2.4)**

- ✅ Schema published
- ✅ Example and fixtures created
- ✅ Documentation written
- ✅ Parity snapshot defined

**Phase 2: Helper Libraries (v0.2.x)**

- Implement `appidentity` module in gofulmen, pyfulmen, tsfulmen
- Full test coverage including parity tests
- Integration with configpaths module

**Phase 3: Templates (v0.3.x)**

- Add `.fulmen/app.yaml` to groningen, percheron templates
- Replace hardcoded references with helper API calls
- Add validation to Makefiles and CI

**Phase 4: Adoption (v0.3.x+)**

- Roll out to existing projects
- CDRL documentation updates
- Monitor and iterate based on feedback

## Related Documentation

- [Fulmen Ecosystem Guide](../../architecture/fulmen-ecosystem-guide.md)
- [Config Path API Standard](./config-paths.md)
- [Three-Layer Config Standard](./enterprise-three-layer-config.md)
- [Repository Category Standards](../repository-category/README.md)

## References

- **Schema**: `schemas/config/repository/app-identity/v1.0.0/app-identity.schema.json`
- **Example**: `config/repository/app-identity/app-identity.example.yaml`
- **Fixtures**: `config/repository/app-identity/fixtures/{valid,invalid}/`
- **Parity Snapshot**: `config/repository/app-identity/parity-snapshot.json`

---

**Status**: Approved (v1.0.0)

**Maintainers**: Schema Cartographer, Crucible Team

**Last Updated**: 2025-12-23
