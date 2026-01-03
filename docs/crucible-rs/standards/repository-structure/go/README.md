---
title: "Go Repository Structure Standards"
description: "Mandatory tooling and broad requirements for all FulmenHQ Go projects regardless of application type"
author: "Forge Master"
author_of_record: "Dave Thompson (https://github.com/3leapsdave)"
supervised_by: "@3leapsdave"
date: "2025-10-08"
last_updated: "2025-10-08"
status: "draft"
tags: ["standards", "repository-structure", "go", "tooling", "modules"]
related_docs: ["cli-cobra.md", "library.md", "../../coding/go.md"]
---

# Go Repository Structure Standards

## Overview

This document establishes **mandatory tooling and broad requirements** for all FulmenHQ Go projects. These are non-negotiable standards that apply regardless of application category.

Consult the cross-language taxonomy in [`docs/standards/repository-structure/README.md`](../README.md)
before picking a template so the repository’s declared category stays aligned with the canonical set
(`cli`, `workhorse`, `service`, `codex`, etc.).

For **application-specific patterns** (CLI, gRPC workhorse, library, etc.), see the variant-specific guides in this directory after confirming the category definition.

---

## Mandatory Tooling

### Go Version: 1.23+ (REQUIRED)

**Why Go 1.23**:

- Range-over-func iteration (iterator pattern)
- Improved error handling patterns
- Performance improvements
- Enhanced type inference
- Modern standard library features

**Minimum Version**: 1.23  
**Recommended**: 1.23.x (latest patch)

**Version Pinning**:

```go
// go.mod
module github.com/fulmenhq/project-name

go 1.23

toolchain go1.23.2
```

**Go Version Management**: Use official Go installer or `asdf` with go plugin

---

### Module Management: Go Modules (REQUIRED)

**Why Go Modules**:

- Official Go dependency management
- Built into Go toolchain
- Semantic import versioning
- Reproducible builds via go.sum
- No external tools required

**Initialization**:

```bash
# Initialize new module
go mod init github.com/fulmenhq/project-name

# Add dependencies
go get github.com/spf13/cobra@latest

# Tidy dependencies
go mod tidy
```

**Do NOT use**: dep, glide, govendor, or other legacy tools

---

### Linting: golangci-lint (REQUIRED)

**Why golangci-lint**:

- Fast parallel execution
- Aggregates dozens of linters
- Highly configurable
- Industry standard
- CI/CD integration

**Installation**:

```bash
# macOS/Linux
brew install golangci-lint

# Or via go install
go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
```

**Configuration**: Use `.golangci.yml` file

**Minimum `.golangci.yml`**:

```yaml
run:
  timeout: 5m
  go: "1.23"

linters:
  enable:
    - errcheck # Check error handling
    - gosimple # Simplify code
    - govet # Go vet
    - ineffassign # Detect ineffectual assignments
    - staticcheck # Advanced static analysis
    - typecheck # Type checking
    - unused # Detect unused code
    - gofmt # Format checking
    - goimports # Import organization
    - misspell # Spell checking
    - revive # Drop-in replacement for golint

linters-settings:
  errcheck:
    check-type-assertions: true
    check-blank: true

  govet:
    check-shadowing: true

  revive:
    confidence: 0.8

issues:
  exclude-use-default: false
  max-issues-per-linter: 0
  max-same-issues: 0
```

**Running**:

```bash
# Lint all code
golangci-lint run

# Auto-fix issues where possible
golangci-lint run --fix
```

**Do NOT use**: Individual linters (golint, go vet alone) as primary tooling

---

### Testing: Go Testing + testify (REQUIRED)

**Why Go testing**:

- Built into Go toolchain
- Fast parallel execution
- Table-driven test support
- Coverage reporting
- Benchmark support

**Why testify**:

- Rich assertion library
- Suite support
- Mocking utilities
- Clear test failures

**Installation**:

```bash
go get github.com/stretchr/testify
```

**Running Tests**:

```bash
# Run all tests
go test ./...

# Run with coverage
go test -cover ./...

# Generate coverage report
go test -coverprofile=coverage.out ./...
go tool cover -html=coverage.out
```

---

## Project Structure Standards

### Required Files

Every Go project MUST have:

```
project/
├── .golangci.yml            # Linter configuration
├── go.mod                   # Module definition
├── go.sum                   # Dependency checksums
├── README.md                # Project documentation
├── LICENSE                  # License file
├── .gitignore               # Git ignore patterns
├── Makefile                 # Common development commands
├── cmd/                     # Application entry points
│   └── project-name/
│       └── main.go
├── internal/                # Private application code
│   └── ...
├── pkg/                     # Public library code (optional)
│   └── ...
└── testdata/                # Test fixtures (gitignored data files)
```

### Optional Files (Recommended)

```
├── CHANGELOG.md             # Version history
├── CONTRIBUTING.md          # Contribution guidelines
├── VERSION                  # Version file (for CalVer/SemVer)
├── .github/
│   └── workflows/
│       └── test.yml         # CI/CD workflow
├── docs/                    # Documentation
│   └── ...
└── schemas/                 # JSON/YAML schemas
    └── ...
```

---

## Standard Directory Layout

### cmd/ - Application Entry Points

**Purpose**: Command-line application entry points (main packages)

```
cmd/
├── project-name/            # Primary binary
│   └── main.go
└── helper-tool/             # Additional utilities
    └── main.go
```

**main.go example**:

```go
package main

import (
    "github.com/fulmenhq/project-name/internal/cmd"
)

func main() {
    cmd.Execute()
}
```

**Rules**:

- Keep main.go minimal (delegates to internal/cmd or pkg/)
- One binary per subdirectory
- Binary name matches directory name

---

### internal/ - Private Application Code

**Purpose**: Private code that cannot be imported by other projects

```
internal/
├── cmd/                     # CLI command implementations
│   ├── root.go             # Root command
│   ├── version.go          # Version command
│   └── process.go          # Process command
├── core/                    # Business logic
│   ├── processor.go
│   ├── validator.go
│   └── processor_test.go
├── config/                  # Configuration handling
│   ├── config.go
│   └── loader.go
└── utils/                   # Internal utilities
    ├── logger.go
    └── output.go
```

**Why internal/**:

- Go enforces import restrictions (cannot import from another project's internal/)
- Encourages proper API boundaries
- Prevents accidental public API exposure

---

### pkg/ - Public Library Code (Optional)

**Purpose**: Public libraries that other projects can import

```
pkg/
├── client/                  # API client library
│   ├── client.go
│   └── client_test.go
├── models/                  # Shared data models
│   └── types.go
└── utils/                   # Public utilities
    └── helpers.go
```

**When to use pkg/**:

- Creating a library for external consumption
- Exposing reusable components
- Building SDK or client libraries

**When NOT to use pkg/**:

- Application-only code (use internal/ instead)
- Implementation details
- Code that shouldn't be public API

**Note**: For pure libraries, pkg/ is often unnecessary (put public API at module root)

---

### testdata/ - Test Fixtures

**Purpose**: Test data files (not compiled into binary)

```
testdata/
├── fixtures/
│   ├── valid_config.yaml
│   ├── invalid_config.yaml
│   └── sample_data.json
└── golden/                  # Golden file testing
    └── expected_output.txt
```

**Rules**:

- Go ignores files/directories named `testdata`
- Use for input files, expected outputs, etc.
- Version control test data (unless large binary files)

---

## Module Metadata (go.mod)

### Minimum Required Metadata

```go
module github.com/fulmenhq/project-name

go 1.23

toolchain go1.23.2

require (
    github.com/spf13/cobra v1.8.0
    github.com/spf13/viper v1.18.0
    gopkg.in/yaml.v3 v3.0.1
)

require (
    // Indirect dependencies auto-managed by go mod tidy
    github.com/spf13/pflag v1.0.5 // indirect
)
```

**Key Elements**:

- `module`: Module path (import prefix)
- `go`: Minimum Go version
- `toolchain`: Specific Go toolchain version (optional but recommended)
- `require`: Direct dependencies
- Indirect dependencies managed automatically

---

## Dependency Management

### Adding Dependencies

```bash
# Add latest version
go get github.com/spf13/cobra@latest

# Add specific version
go get github.com/spf13/cobra@v1.8.0

# Add from branch
go get github.com/user/repo@main

# Tidy (remove unused, add missing)
go mod tidy
```

### Updating Dependencies

```bash
# Update all dependencies to latest
go get -u ./...

# Update specific dependency
go get -u github.com/spf13/cobra

# Update to specific version
go get github.com/spf13/cobra@v1.8.1
```

### Vendoring (Optional)

```bash
# Vendor dependencies (creates vendor/ directory)
go mod vendor

# Build using vendored deps
go build -mod=vendor ./cmd/project-name
```

**When to vendor**:

- Ensuring build reproducibility
- Offline builds required
- CI/CD with restricted network

**When NOT to vendor**:

- Default builds (go.sum provides reproducibility)
- Development (adds repository bloat)

---

## Build System

### Building Binaries

```bash
# Build for current platform
go build -o bin/project-name ./cmd/project-name

# Build with version information
go build -ldflags="-X main.version=1.0.0" -o bin/project-name ./cmd/project-name

# Cross-compile
GOOS=linux GOARCH=amd64 go build -o bin/project-name-linux-amd64 ./cmd/project-name
GOOS=darwin GOARCH=arm64 go build -o bin/project-name-darwin-arm64 ./cmd/project-name
GOOS=windows GOARCH=amd64 go build -o bin/project-name-windows-amd64.exe ./cmd/project-name
```

### Makefile Targets

```makefile
.PHONY: build test lint clean install

VERSION ?= $(shell cat VERSION)
LDFLAGS := -X main.version=$(VERSION)

build:  ## Build binary for current platform
	go build -ldflags="$(LDFLAGS)" -o bin/project-name ./cmd/project-name

build-all:  ## Build multi-platform binaries
	GOOS=linux GOARCH=amd64 go build -ldflags="$(LDFLAGS)" -o bin/project-name-linux-amd64 ./cmd/project-name
	GOOS=darwin GOARCH=amd64 go build -ldflags="$(LDFLAGS)" -o bin/project-name-darwin-amd64 ./cmd/project-name
	GOOS=darwin GOARCH=arm64 go build -ldflags="$(LDFLAGS)" -o bin/project-name-darwin-arm64 ./cmd/project-name
	GOOS=windows GOARCH=amd64 go build -ldflags="$(LDFLAGS)" -o bin/project-name-windows-amd64.exe ./cmd/project-name

test:  ## Run tests
	go test -v -race ./...

test-cov:  ## Run tests with coverage
	go test -v -race -coverprofile=coverage.out ./...
	go tool cover -html=coverage.out -o coverage.html

lint:  ## Run linting
	golangci-lint run

fmt:  ## Format code
	gofmt -w .
	goimports -w .

clean:  ## Clean build artifacts
	rm -rf bin/ coverage.out coverage.html

install:  ## Install dependencies
	go mod download
	go mod tidy
```

---

## Common Makefile Targets (FulmenHQ Standard)

```makefile
.PHONY: bootstrap test lint build clean help

help:  ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

bootstrap:  ## Install dependencies and tools
	go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
	go mod download
	go mod tidy

tools:  ## Verify external tools
	@which golangci-lint > /dev/null || (echo "golangci-lint not found" && exit 1)

lint:  ## Run linting
	golangci-lint run

test:  ## Run tests
	go test -v -race ./...

build:  ## Build binary
	go build -o bin/project-name ./cmd/project-name

clean:  ## Clean build artifacts
	rm -rf bin/ coverage.out

check-all:  ## Run all checks
	$(MAKE) lint
	$(MAKE) test

version:  ## Print version
	@cat VERSION
```

---

## Continuous Integration

### GitHub Actions Example

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        go-version: ["1.23"]

    steps:
      - uses: actions/checkout@v4

      - name: Set up Go
        uses: actions/setup-go@v5
        with:
          go-version: ${{ matrix.go-version }}

      - name: Install dependencies
        run: go mod download

      - name: Lint
        uses: golangci/golangci-lint-action@v4
        with:
          version: latest

      - name: Test
        run: go test -v -race -coverprofile=coverage.out ./...

      - name: Upload coverage
        uses: codecov/codecov-action@v4
        with:
          files: ./coverage.out
```

---

## Repository Variants

This document covers **broad requirements** for all Go projects. For **application-specific patterns**, see:

- **[CLI Applications (Cobra)](cli-cobra.md)** - Cobra-based command-line tools
- **[gRPC Services](grpc-service.md)** - gRPC microservices (planned)
- **[Libraries](library.md)** - Pure Go libraries (planned)
- **[HTTP APIs (Gin)](http-api-gin.md)** - HTTP web services (planned)

Each variant builds on these standards with additional patterns specific to that use case.

---

## Migration Guide

### From Older Go Versions

```bash
# Update go.mod
go mod edit -go=1.23
go mod edit -toolchain=go1.23.2

# Update dependencies to compatible versions
go get -u ./...
go mod tidy
```

### From dep/glide to Go Modules

```bash
# Initialize go.mod from existing dependency manager
go mod init github.com/fulmenhq/project-name

# Import dependencies
go mod tidy

# Remove old dependency files
rm -rf vendor/ Gopkg.toml Gopkg.lock glide.yaml glide.lock
```

---

## Checklist for New Projects

- [ ] Go 1.23+ via `go.mod`
- [ ] Go Modules initialized (`go.mod`, `go.sum`)
- [ ] `.golangci.yml` linter configuration
- [ ] Standard directory layout (cmd/, internal/, pkg/)
- [ ] Makefile with standard targets
- [ ] README.md with usage examples
- [ ] LICENSE file
- [ ] .gitignore for Go projects
- [ ] CI workflow (.github/workflows/test.yml)
- [ ] Variant-specific structure (see variant guides)

---

## Exceptions and Approvals

Deviations from these standards require approval from @3leapsdave or repository maintainer. Document exceptions in project README.md with rationale.

**Example exceptions**:

- Legacy projects migrating to Go 1.23 (temporary older version OK)
- Projects requiring older Go for specific platform compatibility
- Monorepos with existing structure conventions

---

## References

- [Go Modules Documentation](https://go.dev/ref/mod)
- [golangci-lint Documentation](https://golangci-lint.run/)
- [Go Testing Documentation](https://pkg.go.dev/testing)
- [Go Project Layout (community)](https://github.com/golang-standards/project-layout)
- [Go Coding Standards](../../coding/go.md)

---

**Remember**: These are the foundations. Build your application structure on top of these standards using the variant guides.
