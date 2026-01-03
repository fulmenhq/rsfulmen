---
title: "Go CLI Application Structure (Cobra)"
description: "Repository structure and patterns for Cobra-based CLI applications including command organization, configuration, and testing patterns"
author: "Forge Master"
author_of_record: "Dave Thompson (https://github.com/3leapsdave)"
supervised_by: "@3leapsdave"
date: "2025-10-08"
last_updated: "2025-10-08"
status: "draft"
tags: ["standards", "repository-structure", "go", "cli", "cobra", "variant"]
related_docs: ["README.md", "library.md", "../../coding/go.md"]
---

# Go CLI Application Structure (Cobra)

## Overview

This guide establishes the **repository structure and patterns** for Cobra-based CLI applications in the FulmenHQ ecosystem. It builds upon the [Go Repository Structure Standards](README.md) with CLI-specific patterns.

**Target Use Cases**:

- Command-line tools and utilities (microtool pattern)
- CLI-based automation tools
- Developer tooling (DX tools like goneat, fuldx)
- System administration utilities
- CLI/server hybrids (CLI with optional HTTP server)

**Example**: Goneat (FulmenHQ's release automation tool)

---

## Prerequisites

Read and apply **[Go Repository Structure Standards](README.md)** first. This document extends those requirements.

**Required Tooling** (from base standards):

- Go 1.23+
- Go Modules
- golangci-lint
- Go testing + testify

**Additional CLI-Specific Dependencies**:

- Cobra (CLI framework) - `github.com/spf13/cobra`
- Viper (configuration) - `github.com/spf13/viper`
- Optional: pterm (rich terminal output) - `github.com/pterm/pterm`

---

## Project Structure

### Microtool Pattern (Single Focused Tool)

```
cli-tool/
├── .golangci.yml            # Linter configuration
├── go.mod                   # Module definition
├── go.sum                   # Dependency checksums
├── Makefile                 # Development commands
├── README.md
├── LICENSE
├── VERSION                  # Version file
├── CHANGELOG.md
│
├── cmd/
│   └── tool-name/           # Binary entry point
│       └── main.go          # Minimal main (delegates to internal/cmd)
│
├── internal/
│   ├── cmd/                 # Cobra command definitions
│   │   ├── root.go          # Root command + global flags
│   │   ├── version.go       # version subcommand
│   │   ├── process.go       # process subcommand
│   │   └── config.go        # config subcommand group
│   ├── core/                # Business logic (CLI-agnostic)
│   │   ├── processor.go
│   │   ├── processor_test.go
│   │   ├── validator.go
│   │   └── validator_test.go
│   ├── config/              # Configuration handling
│   │   ├── config.go        # Config struct + defaults
│   │   ├── loader.go        # Load from file/env
│   │   └── config_test.go
│   └── utils/               # Internal utilities
│       ├── logger.go        # Logging setup
│       ├── output.go        # Output formatting
│       └── errors.go        # Error handling
│
├── testdata/                # Test fixtures
│   ├── fixtures/
│   │   ├── valid_config.yaml
│   │   └── sample_data.json
│   └── golden/
│       └── expected_output.txt
│
└── schemas/                 # JSON/YAML schemas (optional)
    └── config.schema.json
```

### CLI/Server Hybrid Pattern (Workhorse Tool)

```
tool-name/
├── cmd/
│   └── tool-name/
│       └── main.go
│
├── internal/
│   ├── cmd/                 # CLI commands
│   │   ├── root.go
│   │   ├── serve.go         # HTTP server command
│   │   ├── process.go       # CLI processing command
│   │   └── version.go
│   ├── server/              # HTTP server implementation
│   │   ├── server.go        # Server setup
│   │   ├── handlers.go      # HTTP handlers
│   │   ├── middleware.go    # Middleware
│   │   └── routes.go        # Route configuration
│   ├── core/                # Shared business logic
│   │   ├── processor.go     # Used by both CLI and server
│   │   └── validator.go
│   ├── api/                 # API models (if exposing HTTP API)
│   │   ├── models.go
│   │   └── responses.go
│   └── config/
│       ├── cli_config.go    # CLI-specific config
│       └── server_config.go # Server-specific config
│
└── testdata/
```

---

## Key Components

### 1. Entry Point (cmd/tool-name/main.go)

**Microtool Pattern**:

```go
package main

import (
    "os"
    "github.com/fulmenhq/tool-name/internal/cmd"
)

// Version information (set via ldflags)
var (
    version   = "dev"
    commit    = "unknown"
    buildDate = "unknown"
)

func main() {
    // Set version info for commands to access
    cmd.SetVersionInfo(version, commit, buildDate)

    // Execute root command
    if err := cmd.Execute(); err != nil {
        os.Exit(1)
    }
}
```

**Build with version info**:

```bash
go build -ldflags="-X main.version=1.0.0 -X main.commit=$(git rev-parse --short HEAD) -X main.buildDate=$(date -u +%Y-%m-%dT%H:%M:%SZ)" -o bin/tool-name ./cmd/tool-name
```

### 2. Root Command (internal/cmd/root.go)

```go
package cmd

import (
    "fmt"
    "os"

    "github.com/spf13/cobra"
    "github.com/spf13/viper"
    "github.com/fulmenhq/tool-name/internal/config"
    "github.com/fulmenhq/tool-name/internal/utils"
)

var (
    cfgFile string
    verbose bool

    // Version info set by main package
    versionInfo struct {
        Version   string
        Commit    string
        BuildDate string
    }
)

// SetVersionInfo is called by main package to set version information
func SetVersionInfo(version, commit, buildDate string) {
    versionInfo.Version = version
    versionInfo.Commit = commit
    versionInfo.BuildDate = buildDate
}

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
    Use:   "tool-name",
    Short: "Brief description of the tool",
    Long: `Longer description of the tool's purpose and capabilities.

This tool provides functionality for X, Y, and Z.
Use the subcommands to perform specific operations.`,
    SilenceUsage:  true,  // Don't show usage on errors
    SilenceErrors: true,  // We'll handle error output ourselves
}

// Execute adds all child commands to the root command and sets flags appropriately.
func Execute() error {
    return rootCmd.Execute()
}

func init() {
    cobra.OnInitialize(initConfig)

    // Global flags
    rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file (default is $HOME/.tool-name.yaml)")
    rootCmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")

    // Bind flags to viper
    viper.BindPFlag("verbose", rootCmd.PersistentFlags().Lookup("verbose"))
}

func initConfig() {
    if cfgFile != "" {
        // Use config file from flag
        viper.SetConfigFile(cfgFile)
    } else {
        // Search for config in home directory
        home, err := os.UserHomeDir()
        if err != nil {
            fmt.Fprintf(os.Stderr, "Error finding home directory: %v\n", err)
            os.Exit(1)
        }

        viper.AddConfigPath(home)
        viper.SetConfigType("yaml")
        viper.SetConfigName(".tool-name")
    }

    // Read in environment variables
    viper.SetEnvPrefix("TOOLNAME")
    viper.AutomaticEnv()

    // If config file is found, read it
    if err := viper.ReadInConfig(); err == nil {
        if verbose {
            utils.Logger.Info("Using config file", "path", viper.ConfigFileUsed())
        }
    }
}
```

### 3. Version Command (internal/cmd/version.go)

```go
package cmd

import (
    "fmt"
    "github.com/spf13/cobra"
)

var versionCmd = &cobra.Command{
    Use:   "version",
    Short: "Print version information",
    Long:  `Print detailed version information including commit hash and build date.`,
    Run: func(cmd *cobra.Command, args []string) {
        fmt.Printf("tool-name %s\n", versionInfo.Version)
        fmt.Printf("Commit: %s\n", versionInfo.Commit)
        fmt.Printf("Built: %s\n", versionInfo.BuildDate)
    },
}

func init() {
    rootCmd.AddCommand(versionCmd)
}
```

### 4. Process Command (internal/cmd/process.go)

```go
package cmd

import (
    "context"
    "fmt"
    "os"
    "time"

    "github.com/spf13/cobra"
    "github.com/fulmenhq/tool-name/internal/core"
    "github.com/fulmenhq/tool-name/internal/config"
    "github.com/fulmenhq/tool-name/internal/utils"
)

var (
    outputFile string
    format     string
    dryRun     bool
)

var processCmd = &cobra.Command{
    Use:   "process [target]",
    Short: "Process the specified target",
    Long: `Process the specified target file or directory.

The process command analyzes the target and produces results
based on the configured options.`,
    Args: cobra.ExactArgs(1),
    RunE: func(cmd *cobra.Command, args []string) error {
        target := args[0]

        // Load configuration
        cfg, err := config.Load()
        if err != nil {
            return fmt.Errorf("failed to load configuration: %w", err)
        }

        // Create processor
        processor := core.NewProcessor(cfg)

        // Setup context with timeout if specified
        ctx := context.Background()
        if cfg.Timeout > 0 {
            var cancel context.CancelFunc
            ctx, cancel = context.WithTimeout(ctx, cfg.Timeout)
            defer cancel()
        }

        // Process target
        utils.Logger.Info("Processing target", "target", target, "dry-run", dryRun)
        result, err := processor.Process(ctx, target)
        if err != nil {
            return fmt.Errorf("processing failed: %w", err)
        }

        // Output results
        if err := utils.OutputResult(result, format, outputFile); err != nil {
            return fmt.Errorf("failed to output results: %w", err)
        }

        // Exit with appropriate code
        if !result.Success {
            utils.Logger.Warn("Processing completed with issues", "count", len(result.Issues))
            os.Exit(1)
        }

        utils.Logger.Info("Processing completed successfully")
        return nil
    },
}

func init() {
    rootCmd.AddCommand(processCmd)

    processCmd.Flags().StringVarP(&outputFile, "output", "o", "", "output file (default is stdout)")
    processCmd.Flags().StringVarP(&format, "format", "f", "text", "output format (text, json, yaml)")
    processCmd.Flags().BoolVar(&dryRun, "dry-run", false, "perform dry run without making changes")
}
```

### 5. Command Group Example (internal/cmd/config.go)

```go
package cmd

import (
    "fmt"
    "github.com/spf13/cobra"
    "github.com/fulmenhq/tool-name/internal/config"
)

var configCmd = &cobra.Command{
    Use:   "config",
    Short: "Manage configuration",
    Long:  `View and manage tool configuration.`,
}

var configShowCmd = &cobra.Command{
    Use:   "show",
    Short: "Show current configuration",
    RunE: func(cmd *cobra.Command, args []string) error {
        cfg, err := config.Load()
        if err != nil {
            return err
        }

        fmt.Printf("Configuration:\n")
        fmt.Printf("  Timeout: %s\n", cfg.Timeout)
        fmt.Printf("  Workers: %d\n", cfg.Workers)
        return nil
    },
}

var configValidateCmd = &cobra.Command{
    Use:   "validate",
    Short: "Validate configuration file",
    RunE: func(cmd *cobra.Command, args []string) error {
        cfg, err := config.Load()
        if err != nil {
            return fmt.Errorf("configuration validation failed: %w", err)
        }

        fmt.Println("✓ Configuration is valid")
        return nil
    },
}

func init() {
    rootCmd.AddCommand(configCmd)
    configCmd.AddCommand(configShowCmd)
    configCmd.AddCommand(configValidateCmd)
}
```

### 6. Configuration Handling (internal/config/config.go)

```go
package config

import (
    "fmt"
    "time"

    "github.com/spf13/viper"
)

// Config holds application configuration
type Config struct {
    Timeout  time.Duration `mapstructure:"timeout"`
    Workers  int           `mapstructure:"workers"`
    LogLevel string        `mapstructure:"log_level"`
    OutputDir string       `mapstructure:"output_dir"`
}

// Default returns default configuration
func Default() *Config {
    return &Config{
        Timeout:  30 * time.Second,
        Workers:  4,
        LogLevel: "info",
        OutputDir: "./output",
    }
}

// Load loads configuration from viper
func Load() (*Config, error) {
    cfg := Default()

    if err := viper.Unmarshal(cfg); err != nil {
        return nil, fmt.Errorf("failed to unmarshal config: %w", err)
    }

    if err := cfg.Validate(); err != nil {
        return nil, fmt.Errorf("invalid configuration: %w", err)
    }

    return cfg, nil
}

// Validate validates configuration
func (c *Config) Validate() error {
    if c.Workers < 1 || c.Workers > 16 {
        return fmt.Errorf("workers must be between 1 and 16, got %d", c.Workers)
    }

    if c.Timeout < time.Second {
        return fmt.Errorf("timeout must be at least 1 second")
    }

    validLogLevels := map[string]bool{"debug": true, "info": true, "warn": true, "error": true}
    if !validLogLevels[c.LogLevel] {
        return fmt.Errorf("invalid log level: %s", c.LogLevel)
    }

    return nil
}
```

### 7. Logging Setup (internal/utils/logger.go)

```go
package utils

import (
    "log/slog"
    "os"
)

var Logger *slog.Logger

func init() {
    // Default logger (can be reconfigured based on flags)
    Logger = slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{
        Level: slog.LevelInfo,
    }))
}

// SetupLogger configures logger based on verbose flag and log level
func SetupLogger(verbose bool, level string) {
    var slogLevel slog.Level

    if verbose {
        slogLevel = slog.LevelDebug
    } else {
        switch level {
        case "debug":
            slogLevel = slog.LevelDebug
        case "info":
            slogLevel = slog.LevelInfo
        case "warn":
            slogLevel = slog.LevelWarn
        case "error":
            slogLevel = slog.LevelError
        default:
            slogLevel = slog.LevelInfo
        }
    }

    Logger = slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{
        Level: slogLevel,
    }))
}
```

### 8. Output Formatting (internal/utils/output.go)

```go
package utils

import (
    "encoding/json"
    "fmt"
    "os"

    "gopkg.in/yaml.v3"
)

// Result represents processing result
type Result struct {
    Success bool                   `json:"success" yaml:"success"`
    Message string                 `json:"message" yaml:"message"`
    Data    map[string]interface{} `json:"data,omitempty" yaml:"data,omitempty"`
    Issues  []Issue                `json:"issues,omitempty" yaml:"issues,omitempty"`
}

type Issue struct {
    File     string `json:"file" yaml:"file"`
    Line     int    `json:"line,omitempty" yaml:"line,omitempty"`
    Severity string `json:"severity" yaml:"severity"`
    Message  string `json:"message" yaml:"message"`
}

// OutputResult outputs result in specified format
func OutputResult(result *Result, format string, outputFile string) error {
    var output []byte
    var err error

    switch format {
    case "json":
        output, err = json.MarshalIndent(result, "", "  ")
    case "yaml":
        output, err = yaml.Marshal(result)
    case "text":
        output = []byte(formatText(result))
    default:
        return fmt.Errorf("unsupported format: %s", format)
    }

    if err != nil {
        return fmt.Errorf("failed to format output: %w", err)
    }

    // Write to file or stdout
    if outputFile != "" {
        return os.WriteFile(outputFile, output, 0644)
    }

    // Output to stdout (structured data)
    fmt.Println(string(output))
    return nil
}

func formatText(result *Result) string {
    text := fmt.Sprintf("Success: %v\n", result.Success)
    text += fmt.Sprintf("Message: %s\n", result.Message)

    if len(result.Issues) > 0 {
        text += fmt.Sprintf("\nIssues (%d):\n", len(result.Issues))
        for _, issue := range result.Issues {
            text += fmt.Sprintf("  - [%s] %s:%d - %s\n",
                issue.Severity, issue.File, issue.Line, issue.Message)
        }
    }

    return text
}
```

---

## CLI/Server Hybrid Pattern

### Server Command (internal/cmd/serve.go)

```go
package cmd

import (
    "context"
    "fmt"
    "os"
    "os/signal"
    "syscall"
    "time"

    "github.com/spf13/cobra"
    "github.com/fulmenhq/tool-name/internal/server"
    "github.com/fulmenhq/tool-name/internal/config"
    "github.com/fulmenhq/tool-name/internal/utils"
)

var (
    serverPort int
    serverHost string
)

var serveCmd = &cobra.Command{
    Use:   "serve",
    Short: "Start HTTP server",
    Long:  `Start an HTTP server that exposes the tool's functionality via REST API.`,
    RunE: func(cmd *cobra.Command, args []string) error {
        // Load configuration
        cfg, err := config.Load()
        if err != nil {
            return fmt.Errorf("failed to load configuration: %w", err)
        }

        // Create server
        srv := server.New(cfg, serverHost, serverPort)

        // Start server in goroutine
        errChan := make(chan error, 1)
        go func() {
            utils.Logger.Info("Starting HTTP server", "host", serverHost, "port", serverPort)
            if err := srv.Start(); err != nil {
                errChan <- err
            }
        }()

        // Wait for interrupt signal
        sigChan := make(chan os.Signal, 1)
        signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)

        select {
        case err := <-errChan:
            return fmt.Errorf("server error: %w", err)
        case sig := <-sigChan:
            utils.Logger.Info("Received signal, shutting down", "signal", sig)

            // Graceful shutdown
            ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
            defer cancel()

            if err := srv.Shutdown(ctx); err != nil {
                return fmt.Errorf("server shutdown failed: %w", err)
            }

            utils.Logger.Info("Server stopped gracefully")
        }

        return nil
    },
}

func init() {
    rootCmd.AddCommand(serveCmd)

    serveCmd.Flags().IntVarP(&serverPort, "port", "p", 8080, "server port")
    serveCmd.Flags().StringVar(&serverHost, "host", "localhost", "server host")
}
```

---

## Testing

### Command Testing (internal/cmd/process_test.go)

```go
package cmd

import (
    "bytes"
    "testing"

    "github.com/spf13/cobra"
    "github.com/stretchr/testify/assert"
    "github.com/stretchr/testify/require"
)

func TestProcessCommand(t *testing.T) {
    tests := []struct {
        name        string
        args        []string
        expectError bool
    }{
        {
            name:        "valid_target",
            args:        []string{"process", "../../testdata/fixtures/valid_config.yaml"},
            expectError: false,
        },
        {
            name:        "missing_target",
            args:        []string{"process"},
            expectError: true,
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            // Create root command
            cmd := rootCmd

            // Capture output
            buf := new(bytes.Buffer)
            cmd.SetOut(buf)
            cmd.SetErr(buf)

            // Set args
            cmd.SetArgs(tt.args)

            // Execute
            err := cmd.Execute()

            if tt.expectError {
                assert.Error(t, err)
            } else {
                assert.NoError(t, err)
            }
        })
    }
}
```

---

## Best Practices

### 1. Separate CLI from Core Logic

```go
// ✅ GOOD - CLI delegates to core
func (cmd *processCmd) RunE(c *cobra.Command, args []string) error {
    processor := core.NewProcessor(config)
    result, err := processor.Process(ctx, args[0])
    return handleResult(result, err)
}

// ❌ BAD - Business logic in CLI
func (cmd *processCmd) RunE(c *cobra.Command, args []string) error {
    // ... complex processing logic here
}
```

### 2. Use RunE for Error Handling

```go
// ✅ GOOD - Return errors, let Cobra handle them
var myCmd = &cobra.Command{
    RunE: func(cmd *cobra.Command, args []string) error {
        if err := doSomething(); err != nil {
            return fmt.Errorf("operation failed: %w", err)
        }
        return nil
    },
}

// ❌ BAD - Manual error handling
var myCmd = &cobra.Command{
    Run: func(cmd *cobra.Command, args []string) {
        if err := doSomething(); err != nil {
            fmt.Fprintf(os.Stderr, "Error: %v\n", err)
            os.Exit(1)
        }
    },
}
```

### 3. Configuration Hierarchy

**Priority** (highest to lowest):

1. Command-line flags
2. Environment variables
3. Config file
4. Defaults

**Implementation**:

```go
func init() {
    // Flag (highest priority)
    rootCmd.PersistentFlags().IntP("workers", "w", 0, "number of workers")

    // Bind to viper (enables env var and config file)
    viper.BindPFlag("workers", rootCmd.PersistentFlags().Lookup("workers"))

    // Environment variable (TOOLNAME_WORKERS)
    viper.SetEnvPrefix("TOOLNAME")
    viper.AutomaticEnv()

    // Config file loaded in initConfig()

    // Default set in config.Default()
}
```

---

## Makefile

```makefile
.PHONY: build test lint clean install

BINARY_NAME := tool-name
VERSION := $(shell cat VERSION 2>/dev/null || echo "dev")
COMMIT := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_DATE := $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
LDFLAGS := -X main.version=$(VERSION) -X main.commit=$(COMMIT) -X main.buildDate=$(BUILD_DATE)

build:  ## Build binary for current platform
	go build -ldflags="$(LDFLAGS)" -o bin/$(BINARY_NAME) ./cmd/$(BINARY_NAME)

build-all:  ## Build multi-platform binaries
	GOOS=linux GOARCH=amd64 go build -ldflags="$(LDFLAGS)" -o bin/$(BINARY_NAME)-linux-amd64 ./cmd/$(BINARY_NAME)
	GOOS=darwin GOARCH=amd64 go build -ldflags="$(LDFLAGS)" -o bin/$(BINARY_NAME)-darwin-amd64 ./cmd/$(BINARY_NAME)
	GOOS=darwin GOARCH=arm64 go build -ldflags="$(LDFLAGS)" -o bin/$(BINARY_NAME)-darwin-arm64 ./cmd/$(BINARY_NAME)
	GOOS=windows GOARCH=amd64 go build -ldflags="$(LDFLAGS)" -o bin/$(BINARY_NAME)-windows-amd64.exe ./cmd/$(BINARY_NAME)
	cd bin && sha256sum * > SHA256SUMS.txt

install:  ## Install dependencies
	go mod download
	go mod tidy

test:  ## Run tests
	go test -v -race ./...

test-cov:  ## Run tests with coverage
	go test -v -race -coverprofile=coverage.out ./...
	go tool cover -html=coverage.out -o coverage.html

lint:  ## Run linting
	golangci-lint run

clean:  ## Clean build artifacts
	rm -rf bin/ coverage.out coverage.html
```

---

## Checklist for New CLI Projects

- [ ] Base Go standards applied (see README.md)
- [ ] Cobra for CLI framework
- [ ] Viper for configuration
- [ ] Structured logging to STDERR (slog)
- [ ] Separate cmd/ from internal/cmd/
- [ ] Core business logic in internal/core/
- [ ] Version command with ldflags injection
- [ ] Config file support with validation
- [ ] Output formatting (JSON/YAML/text)
- [ ] Proper exit codes
- [ ] Command tests using Cobra testing utilities

---

## References

- [Cobra Documentation](https://cobra.dev/)
- [Viper Documentation](https://github.com/spf13/viper)
- [Go Repository Structure Standards](README.md)
- [Go Coding Standards](../../coding/go.md)

---

**Remember**: Separate CLI interface from business logic. Core logic should be usable without Cobra.
