---
title: "Crucible Integration Guide"
description: "How to integrate Crucible schemas, standards, and templates into your project"
author: "@3leapsdave"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "approved"
tags: ["guide", "integration", "development"]
---

# Crucible Integration Guide

This guide explains how to integrate Crucible into your project for production use and local development.

## Quick Start by Use Case

Before diving into use cases, make sure you are familiar with the sync workflows documented in the [Sync Consumers Guide](sync-consumers-guide.md) (and, if you publish SSOT assets, the [Sync Producers Guide](sync-producers-guide.md)).

### I'm Building a Library/Tool (Go or TypeScript)

**Use**: Published packages  
**Why**: Schemas embedded, version managed, no sync needed

→ See [Production Integration](#production-integration)

### I'm Building a Fulmen Template

**Use**: FulDX sync  
**Why**: Need to vendor schemas/docs/config defaults

→ See [Template Integration](#template-integration)

### I'm Developing Crucible or a Library Using It

**Use**: Local package references  
**Why**: Test changes without publishing

→ See [Development Workflow](#development-workflow)

## Production Integration

### Go Projects

**1. Add dependency:**

```bash
go get github.com/fulmenhq/crucible@latest
```

**2. Use embedded schemas:**

```go
package main

import (
    "fmt"
    "log"

    "github.com/fulmenhq/crucible"
)

func main() {
    // Access schemas
    schema, err := crucible.SchemaRegistry.Pathfinder().V1_0_0().FindQuery()
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Loaded schema: %d bytes\n", len(schema))

    // Load terminal catalog
    catalog, err := crucible.LoadTerminalCatalog()
    if err != nil {
        log.Fatal(err)
    }

    // Get specific terminal
    iterm, err := crucible.GetTerminalConfig("iTerm2")
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("iTerm2 emoji width: %d\n", iterm.Overrides.EmojiWidth)

    // Access standards documentation
    goStandards, err := crucible.StandardsRegistry.Coding().Go()
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Go standards: %d bytes\n", len(goStandards))
}
```

**3. Pin version in production:**

```bash
# Pin to specific version
go get github.com/fulmenhq/crucible@v2025.10.0

# Or use go.mod
require github.com/fulmenhq/crucible v2025.10.0
```

### TypeScript Projects

**1. Add dependency:**

```bash
bun add @fulmenhq/crucible
# or
npm install @fulmenhq/crucible
```

**2. Use schemas:**

```typescript
import {
  schemas,
  standards,
  loadTerminalCatalog,
  getTerminalConfig,
  VERSION,
} from "@fulmenhq/crucible";

// Access schemas
const pathfinderSchema = schemas.pathfinder().v1_0_0().findQuery();
console.log("Pathfinder schema:", pathfinderSchema);

// Load terminal catalog
const catalog = loadTerminalCatalog();
console.log(`Loaded ${catalog.size} terminal configs`);

// Get specific terminal
const iterm = getTerminalConfig("iTerm2");
console.log("iTerm2 emoji width:", iterm?.overrides.emoji_width);

// Access standards documentation
const goStandards = standards.coding().go();
console.log(`Go standards: ${goStandards.length} bytes`);

// Check crucible version
console.log("Crucible version:", VERSION);
```

**3. Pin version in production:**

```json
{
  "dependencies": {
    "@fulmenhq/crucible": "2025.10.0"
  }
}
```

## Template Integration

For Fulmen templates or projects that need to vendor schemas/docs/config defaults we recommend using **FulDX**.

1. **Install FulDX (tool bootstrap)**

```bash
make bootstrap   # installs ./bin/fuldx via .goneat/tools.yaml
```

2. **Create a sync manifest** (`.fuldx/sync-consumer.yaml` is a common location):

```yaml
version: "2025.10.0"
sources:
  - id: crucible.schemas.pathfinder
    include:
      - v1.0.0/*.schema.json
    output: .crucible/schemas/pathfinder
  - id: crucible.docs
    include:
      - guides/**/*.md
    output: docs/crucible
  - id: crucible.config.terminal
    include:
      - v1.0.0/terminal-overrides-defaults.yaml
    output: config/crucible
```

Manifest schema: `schemas/config/sync-consumer-config.yaml`. Recommended keys are published in `config/sync/sync-keys.yaml`.

3. **Pull assets**

```bash
./bin/fuldx sync pull --manifest .fuldx/sync-consumer.yaml
```

FulDX will validate and download assets into the specified output directories (future releases add automatic schema validation).

4. **Use pulled assets**

```typescript
import { readFileSync } from "fs";
import { join } from "path";

const schema = JSON.parse(
  readFileSync(
    join(
      process.cwd(),
      ".crucible/schemas/pathfinder/v1.0.0/find-query.schema.json",
    ),
    "utf-8",
  ),
);
```

See the [Sync Consumers Guide](sync-consumers-guide.md) for additional patterns and the [Sync Producers Guide](sync-producers-guide.md) if you are publishing new SSOT assets.

## Development Workflow

### Go Development with Local Crucible

When developing a library that uses crucible and you need to test local changes:

**1. Use go.mod replace directive:**

In your library's `go.mod`:

```go
module github.com/fulmenhq/gofulmen

go 1.21

require (
    github.com/fulmenhq/crucible v2025.10.0
)

// Development: use local crucible
replace github.com/fulmenhq/crucible => ../crucible/lang/go
```

**2. Directory structure:**

```
dev/fulmenhq/
├── crucible/           # Cloned from fulmenhq/crucible
│   └── lang/go/       # Go package
└── gofulmen/          # Your library
    └── go.mod         # Uses replace directive
```

**3. Make changes in crucible:**

```bash
cd crucible
# Edit schemas, sync to lang wrappers
bun run sync:to-lang

# Test in your library
cd ../gofulmen
go test ./...
```

**4. Before committing:**
Remove or comment out the replace directive:

```go
// replace github.com/fulmenhq/crucible => ../crucible/lang/go
```

**5. Alternative: Install from local path:**

```bash
cd gofulmen
go mod edit -replace=github.com/fulmenhq/crucible=../crucible/lang/go
go mod tidy
```

### TypeScript Development with Local Crucible

**Option A: npm/bun link**

```bash
# In crucible TypeScript package
cd crucible/lang/typescript
bun link

# In your library
cd ../../gofulmen  # or your project
bun link @fulmenhq/crucible
```

**Option B: package.json local path**

```json
{
  "dependencies": {
    "@fulmenhq/crucible": "file:../crucible/lang/typescript"
  }
}
```

**Option C: Use pull script with ref:**

```bash
# Pull from local crucible (if cloned sibling)
bun run scripts/crucible-pull.ts --ref=main --force
```

**Before committing:**

```json
{
  "dependencies": {
    "@fulmenhq/crucible": "2025.10.0"
  }
}
```

### Testing Local Changes End-to-End

**Workflow for library developers:**

```bash
# 1. Clone both repos as siblings
cd ~/dev/fulmenhq
git clone https://github.com/fulmenhq/crucible.git
git clone https://github.com/fulmenhq/gofulmen.git

# 2. Make changes in crucible
cd crucible
vim schemas/pathfinder/v1.0.0/find-query.schema.json

# 3. Sync to language wrappers
bun run sync:to-lang

# 4. Test in your library (Go example)
cd ../gofulmen
# Add replace directive to go.mod
echo 'replace github.com/fulmenhq/crucible => ../crucible/lang/go' >> go.mod
go test ./pathfinder/...

# 5. When satisfied, prepare for PR
cd ../crucible
git add schemas/ lang/
git commit -m "feat: add new pathfinder query options"
git push origin feature/new-query-options

# 6. In your library, remove replace and use version
cd ../gofulmen
# Remove replace directive
vim go.mod  # Delete replace line
# Wait for crucible PR to merge and tag
go get github.com/fulmenhq/crucible@v2025.10.1
```

## Schema Validation

### Go Validation Example

```go
import (
    "encoding/json"

    "github.com/fulmenhq/crucible"
    "github.com/xeipuuv/gojsonschema"
)

func validateFindQuery(query map[string]interface{}) error {
    // Get schema
    schemaBytes, err := crucible.SchemaRegistry.Pathfinder().V1_0_0().FindQuery()
    if err != nil {
        return err
    }

    // Parse as JSON schema
    schema, err := crucible.ParseJSONSchema(schemaBytes)
    if err != nil {
        return err
    }

    // Validate
    schemaLoader := gojsonschema.NewGoLoader(schema)
    documentLoader := gojsonschema.NewGoLoader(query)

    result, err := gojsonschema.Validate(schemaLoader, documentLoader)
    if err != nil {
        return err
    }

    if !result.Valid() {
        return fmt.Errorf("validation failed: %v", result.Errors())
    }

    return nil
}
```

### TypeScript Validation Example

```typescript
import Ajv from "ajv";
import { schemas } from "@fulmenhq/crucible";

const ajv = new Ajv();

function validateFindQuery(query: unknown): boolean {
  const schema = schemas.pathfinder().v1_0_0().findQuery();
  const validate = ajv.compile(schema);

  if (!validate(query)) {
    console.error("Validation errors:", validate.errors);
    return false;
  }

  return true;
}

// Usage
const query = {
  root: "/home/user/project",
  include: ["**/*.ts"],
  maxDepth: 5,
};

if (validateFindQuery(query)) {
  console.log("Query is valid!");
}
```

## Version Management

### Checking Crucible Version

**Go:**

```go
import "github.com/fulmenhq/crucible"

fmt.Println("Crucible version:", crucible.Version)
```

**TypeScript:**

```typescript
import { VERSION } from "@fulmenhq/crucible";

console.log("Crucible version:", VERSION);
```

### Updating Crucible

**Go:**

```bash
# Update to latest
go get -u github.com/fulmenhq/crucible

# Update to specific version
go get github.com/fulmenhq/crucible@v2025.11.0

# Verify
go mod graph | grep crucible
```

**TypeScript:**

```bash
# Update to latest
bun update @fulmenhq/crucible

# Update to specific version
bun add @fulmenhq/crucible@2025.11.0

# Verify
bun pm ls | grep crucible
```

## Troubleshooting

### Go: "ambiguous import" or "multiple packages"

**Problem**: Multiple versions of crucible in module cache

**Solution:**

```bash
go clean -modcache
go mod tidy
go build ./...
```

### TypeScript: "Cannot find module '@fulmenhq/crucible'"

**Problem**: Package not installed or wrong registry

**Solution:**

```bash
# Clear cache
rm -rf node_modules
bun install

# Or rebuild
bun install --force
```

### Schemas are empty/undefined

**Problem**: Using package before sync or wrong import path

**Solution (Go):**

```bash
cd crucible
bun run sync:to-lang
cd lang/go
go build ./...
```

**Solution (TypeScript):**

```bash
cd crucible
bun run sync:to-lang
cd lang/typescript
bun run build
```

### Local development changes not reflecting

**Problem**: Using cached published version instead of local

**Solution (Go):**

```bash
# Verify replace directive is active
go mod edit -print | grep replace

# Force rebuild
go clean -cache
go build -a ./...
```

**Solution (TypeScript):**

```bash
# Verify link is active
bun link --list

# Reinstall
bun install --force
```

## Best Practices

### Production

- ✅ Pin specific versions: `v2025.10.0`
- ✅ Test version updates in staging first
- ✅ Use published packages, not local paths
- ✅ Validate inputs against schemas
- ✅ Check for schema updates quarterly

### Development

- ✅ Use replace directives for local testing
- ✅ Remove replace directives before committing
- ✅ Sync schemas after every crucible change
- ✅ Test with multiple schema versions
- ✅ Document schema version requirements

### CI/CD

- ✅ Pin crucible version in CI
- ✅ Cache dependencies
- ✅ Run schema validation tests
- ✅ Alert on failed validations
- ✅ Test crucible updates in PR builds

## Related Documentation

- [Sync Strategy Guide](sync-strategy.md) - Pull scripts and vendoring
- [Sync Model Architecture](../architecture/sync-model.md) - How sync works
- [Repository Structure SOP](../sop/repository-structure.md) - Project setup
- [Pull Script README](../../scripts/pull/README.md) - Pull script usage

---

**Status**: Approved  
**Last Updated**: 2025-10-02  
**Author**: @3leapsdave
