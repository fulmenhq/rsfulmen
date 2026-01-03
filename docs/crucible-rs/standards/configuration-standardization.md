---
title: "Configuration Standardization"
description: "Standard approaches for configuration management across Fulmen language implementations"
author: "@gofulmen-team"
date: "2025-10-02"
last_updated: "2025-10-07"
status: "approved"
tags: ["standards", "configuration", "cross-language", "local-overrides"]
---

# Configuration Standardization

This document defines standard approaches for configuration management across Fulmen language implementations.

## Configuration Sources (Priority Order)

1. **Command-line arguments** (highest priority)
2. **Environment variables**
3. **Configuration files** (JSON, YAML, TOML)
4. **Default values** (lowest priority)

## Environment Variables

### Naming Convention

- Prefix: `FULMEN_`
- Component: `PATHFINDER_`, `ASCII_`, etc.
- Setting: Uppercase with underscores
- Example: `FULMEN_PATHFINDER_MAX_WORKERS`

### Common Variables

- `FULMEN_CONFIG_FILE`: Path to configuration file
- `FULMEN_LOG_LEVEL`: Logging verbosity (DEBUG, INFO, WARN, ERROR)
- `FULMEN_LOG_FORMAT`: Log output format (json, text)

### FulDX SSOT Sync Variables

- `FULDX_{SOURCE}_LOCAL_PATH`: Override local path for SSOT source (e.g., `FULDX_CRUCIBLE_LOCAL_PATH`)
- `FULDX_SYNC_CONSUMER_CONFIG`: Path to sync consumer configuration file
- `FULDX_OUTPUT_DIR`: Base directory for synced assets

### Pathfinder Variables

- `FULMEN_PATHFINDER_MAX_WORKERS`: Maximum concurrent workers
- `FULMEN_PATHFINDER_CACHE_ENABLED`: Enable caching
- `FULMEN_PATHFINDER_CACHE_TTL`: Cache time-to-live in seconds
- `FULMEN_PATHFINDER_VALIDATE_INPUTS`: Enable input validation
- `FULMEN_PATHFINDER_VALIDATE_OUTPUTS`: Enable output validation

## Configuration Files

### Supported Formats

- JSON (`.json`)
- YAML (`.yaml`, `.yml`)
- TOML (`.toml`)

### Local Override Pattern

For machine-specific configurations (like local development paths), use the `.local.*` pattern:

- **Main Config** (committed): `.fuldx/sync-consumer.yaml`
- **Local Override** (gitignored): `.fuldx/sync-consumer.local.yaml`

**Loading Priority:**

1. Load main configuration file
2. Merge local override file (if exists)
3. Apply environment variable overrides
4. Apply convention-based defaults

**Example `.gitignore` entry:**

```
# Local configuration overrides
*.local.yaml
*.local.json
*.local.toml
.fuldx/sync-consumer.local.yaml
```

### Search Paths (in order)

1. Current working directory: `./fulmen.json`
2. User config directory: `~/.config/fulmen/config.json`
3. XDG config home: `$XDG_CONFIG_HOME/fulmen/config.json`
4. System config: `/etc/fulmen/config.json`

### File Structure

```json
{
  "pathfinder": {
    "maxWorkers": 4,
    "cacheEnabled": false,
    "validateInputs": false,
    "validateOutputs": false
  },
  "ascii": {
    "defaultBoxStyle": "single"
  },
  "logging": {
    "level": "info",
    "format": "text"
  }
}
```

## Configuration Loading

### Go Implementation

```go
type Config struct {
    Pathfinder PathfinderConfig `json:"pathfinder"`
    ASCII      ASCIIConfig      `json:"ascii"`
    Logging    LoggingConfig    `json:"logging"`
}

func LoadConfig() (*Config, error) {
    cfg := DefaultConfig()

    // Load from files
    if err := loadConfigFile(cfg); err != nil {
        return nil, err
    }

    // Override with environment
    if err := loadFromEnv(cfg); err != nil {
        return nil, err
    }

    return cfg, nil
}
```

### TypeScript Implementation

```typescript
interface Config {
  pathfinder: PathfinderConfig;
  ascii: ASCIIConfig;
  logging: LoggingConfig;
}

function loadConfig(): Config {
  const cfg = defaultConfig();

  // Load from files
  loadConfigFile(cfg);

  // Override with environment
  loadFromEnv(cfg);

  return cfg;
}
```

### Python Implementation (Future)

```python
@dataclass
class Config:
    pathfinder: PathfinderConfig
    ascii: ASCIIConfig
    logging: LoggingConfig

def load_config() -> Config:
    cfg = default_config()

    # Load from files
    load_config_file(cfg)

    # Override with environment
    load_from_env(cfg)

    return cfg
```

## Validation

- **Validate on load**: Check configuration values when loading
- **Clear error messages**: Provide actionable feedback for invalid settings
- **Partial configuration**: Use defaults for missing values
- **Schema validation**: Validate against JSON schemas where applicable

## Runtime Configuration

- **Reload support**: Allow configuration reload without restart where possible
- **Introspection APIs**: Provide APIs to inspect current configuration
- **Change notifications**: Support callbacks/events for configuration changes
- **Validation before apply**: Validate configuration changes before applying

## Security Considerations

- **Sensitive values**: Avoid logging sensitive configuration values
- **Credential handling**: Use secure methods for credential configuration (keychain, secrets manager)
- **File permissions**: Validate file permissions on configuration files
- **Encryption support**: Consider supporting encrypted configuration files
- **Environment isolation**: Be cautious with environment variable precedence in production

## Testing Configuration

### Unit Tests

- Test configuration loading from each source
- Test priority/override behavior
- Test validation logic
- Test default values

### Integration Tests

- Test configuration discovery
- Test environment-specific configurations
- Test configuration reload
- Test invalid configuration handling

## Related Standards

- [Error Handling Patterns](./error-handling-patterns.md)
- [Frontmatter Standard](./frontmatter-standard.md)

## Examples

### Pathfinder Configuration

**finder-config.json**:

```json
{
  "maxWorkers": 8,
  "cacheEnabled": true,
  "cacheTTL": 3600,
  "constraint": {
    "root": "/home/user/project",
    "type": "repository",
    "enforcementLevel": "strict"
  },
  "loaderType": "local"
}
```

See `schemas/pathfinder/v1.0.0/finder-config.schema.json` for validation.

---

**Status**: Approved  
**Last Updated**: 2025-10-02  
**Original Author**: @gofulmen-team  
**Integrated By**: @3leapsdave
