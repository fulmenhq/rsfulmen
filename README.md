# rsfulmen

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust: 1.83+](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Crucible: v0.4.2](https://img.shields.io/badge/crucible-v0.4.2-purple.svg)](https://github.com/fulmenhq/crucible)

**Curated Libraries for Scale**

Rust Fulmen helper library for enterprise-scale development.

**Lifecycle Phase**: `alpha` | **Version**: 0.1.0

📖 **[Read the complete rsfulmen overview](docs/rsfulmen-overview.md)** for comprehensive documentation including module catalog and roadmap.

## Overview

rsfulmen provides consistent, high-quality implementations of common functionality across the FulmenHQ ecosystem. Built on Crucible's schemas and standards, it ensures uniformity and reliability with idiomatic Rust APIs.

> **Alpha Status**: Early adopters; API may evolve before 1.0. See [Repository Lifecycle Standard](docs/crucible-rs/standards/repository-lifecycle.md).

## Crucible Integration

**What is Crucible?**

Crucible is the FulmenHQ single source of truth (SSOT) for schemas, standards, and configuration templates. It ensures consistent APIs, documentation structures, and behavioral contracts across all language foundations (gofulmen, pyfulmen, tsfulmen, rsfulmen).

**Why rsfulmen?**

Rather than copying Crucible assets into every Rust project, rsfulmen provides idiomatic access through type-safe APIs. This keeps your application lightweight, versioned correctly, and aligned with ecosystem-wide standards.

**Where to Learn More:**

- [Crucible Repository](https://github.com/fulmenhq/crucible) — SSOT schemas, docs, and configs
- [Fulmen Technical Manifesto](docs/crucible-rs/architecture/fulmen-technical-manifesto.md) — Philosophy and design principles
- [gofulmen](https://github.com/fulmenhq/gofulmen) — Go reference implementation

## Modules

### Config (`config`)

Configuration path utilities following the [Fulmen Config Path Standard](docs/crucible-rs/standards/config/fulmen-config-paths.md).

- XDG Base Directory compliance (Linux, macOS, Windows)
- Application-specific config/data/cache directories
- Fulmen ecosystem directory helpers
- Legacy config path fallback support

```rust
use rsfulmen::config::{get_fulmen_config_dir, get_app_config_dir, get_xdg_base_dirs};

// Get Fulmen ecosystem config directory
let fulmen_config = get_fulmen_config_dir();
// Linux: ~/.config/fulmen
// macOS: ~/Library/Application Support/Fulmen
// Windows: %APPDATA%\Fulmen

// Get app-specific directories
let app_config = get_app_config_dir("myapp");
let xdg = get_xdg_base_dirs();
println!("Config home: {:?}", xdg.config_home);
```

### Foundry (`foundry`)

Enterprise-grade foundation utilities providing consistent cross-language implementations from Crucible catalogs. All data is embedded at compile time — no network dependencies required.

#### Country Codes

ISO 3166-1 country code lookups with triple-index support.

```rust
use rsfulmen::foundry::country_codes::{lookup_by_alpha2, lookup_by_alpha3, lookup_by_numeric};

let usa = lookup_by_alpha2("US").unwrap();
assert_eq!(usa.name, "United States of America");
assert_eq!(usa.alpha3, "USA");

// Case-insensitive lookups
let japan = lookup_by_alpha3("jpn").unwrap();
assert_eq!(japan.alpha2, "JP");

// Numeric codes (auto zero-padded)
let germany = lookup_by_numeric("276").unwrap();
assert_eq!(germany.name, "Germany");
```

#### HTTP Status Codes

HTTP status code registry with grouping helpers.

```rust
use rsfulmen::foundry::http_statuses::{lookup_status, get_reason, is_success, StatusGroup};

let ok = lookup_status(200).unwrap();
assert_eq!(ok.reason, "OK");
assert_eq!(ok.group, StatusGroup::Success);

assert!(is_success(201));
assert!(!is_success(404));

let reason = get_reason(404).unwrap();
assert_eq!(reason, "Not Found");
```

#### Exit Codes

Standardized exit codes with categories and signal handling.

```rust
use rsfulmen::foundry::exit_codes::{
    lookup_exit_code, is_signal_exit, get_signal_from_exit,
    EXIT_SUCCESS, EXIT_CONFIG_INVALID, ExitCategory,
};

// Use standard constants
std::process::exit(EXIT_SUCCESS);

// Look up exit code metadata
let code = lookup_exit_code(20).unwrap();
assert_eq!(code.name, "EXIT_CONFIG_INVALID");
assert_eq!(code.category, ExitCategory::Configuration);

// Signal detection (128+)
assert!(is_signal_exit(130));  // SIGINT
let signal = get_signal_from_exit(130).unwrap();
assert_eq!(signal, 2);  // SIGINT = 2
```

### Error Handling (`error_handling`)

Canonical error envelope that extends Pathfinder's schema with optional telemetry
fields (`severity`, `correlation_id`, `exit_code`, etc.). Payloads are JSON
serializable and can be validated offline when `schema-validation` is enabled.

```rust
use rsfulmen::error_handling::{ErrorResponse, PathfinderErrorResponse, Severity, WrapOptions};

let base = PathfinderErrorResponse::new("CONFIG_INVALID", "Config load failed");
let err = ErrorResponse::wrap(
    base,
    WrapOptions {
        severity: Some(Severity::High),
        exit_code: Some(20),
        ..WrapOptions::default()
    },
)
.unwrap();

println!("{}", err.to_json_string_pretty().unwrap());
```

### Telemetry Metrics (`telemetry_metrics`)

Taxonomy-backed counters, gauges, and histograms exported as schema-valid JSON
events.

```rust
use rsfulmen::telemetry_metrics::Metrics;

let metrics = Metrics::new();
metrics.counter("schema_validations").unwrap().inc(None).unwrap();

let events = metrics.flush().unwrap();
assert!(!events.is_empty());
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rsfulmen = "0.1"
```

### Feature Flags

rsfulmen supports **minimal installs** for lightweight consumers (e.g. sysprims).

```toml
[dependencies]
# All features (default)
rsfulmen = "0.1"

# Foundry core only (signals, exit-codes, countries, http-statuses)
rsfulmen = { version = "0.1", default-features = false, features = ["foundry-core"] }

# Add MIME types (adds serde_json)
rsfulmen = { version = "0.1", default-features = false, features = ["foundry-mime-types"] }

# Add patterns (adds regex/glob)
rsfulmen = { version = "0.1", default-features = false, features = ["foundry-patterns"] }

# Similarity (standalone module; heavy)
rsfulmen = { version = "0.1", default-features = false, features = ["similarity"] }

# Schema validation (heavy)
rsfulmen = { version = "0.1", default-features = false, features = ["schema-validation"] }
```

#### Feature Matrix

| Feature | Includes | Notes |
|--------|----------|------|
| `foundry-core` | signals, exit-codes, countries, http-statuses | Minimal catalog install |
| `foundry-mime-types` | mime-types | Adds `serde_json` |
| `foundry-patterns` | patterns | Adds `regex` + `glob` |
| `similarity` | `rsfulmen::similarity` (+ foundry re-export) | Heavy deps (strsim/unicode) |
| `schema-validation` | `rsfulmen::schema_validation` | Heavy deps (jsonschema/url) |
| `error-handling` | `rsfulmen::error_handling` | Canonical error envelope (adds `serde_json`) |
| `telemetry-metrics` | `rsfulmen::telemetry_metrics` | Metrics export (schema-valid JSON events) |
| `crucible` | `rsfulmen::crucible` | Embedded SSOT access |
| `docscribe` | `rsfulmen::docscribe` | Doc access + frontmatter parsing |

## Development

### Prerequisites

- Rust 1.83+
- [goneat](https://github.com/fulmenhq/goneat) for SSOT sync (installed via `make bootstrap`)

### Quick Start

```bash
# Install development tools
make bootstrap

# Sync Crucible assets
make sync

# Run tests
make test

# Run all quality checks
make check-all
```

### Makefile Targets

| Target | Description |
|--------|-------------|
| `bootstrap` | Install dependencies and external tools |
| `sync` | Sync assets from Crucible SSOT |
| `build` | Build library |
| `test` | Run all tests |
| `lint` | Run clippy with strict warnings |
| `fmt` | Format code with rustfmt |
| `check-all` | fmt-check + lint + test |
| `doc` | Generate rustdoc documentation |
| `version` | Print current version |

### Crucible Sync

rsfulmen syncs schemas, documentation, and configuration from [Crucible](https://github.com/fulmenhq/crucible):

```bash
# Update to latest Crucible
make sync

# Check sync provenance
cat .goneat/ssot/provenance.json
```

Synced assets are stored in:
- `config/crucible-rs/` — Configuration files and foundry catalogs
- `schemas/crucible-rs/` — JSON schemas
- `docs/crucible-rs/` — Documentation and standards

## Ecosystem

rsfulmen is part of the Fulmen helper library family:

| Library | Language | Status |
|---------|----------|--------|
| [gofulmen](https://github.com/fulmenhq/gofulmen) | Go | Reference implementation |
| [tsfulmen](https://github.com/fulmenhq/tsfulmen) | TypeScript | Stable |
| [pyfulmen](https://github.com/fulmenhq/pyfulmen) | Python | Stable |
| rsfulmen | Rust | Experimental |

All libraries sync from [Crucible](https://github.com/fulmenhq/crucible) and follow the [Fulmen Helper Library Standard](docs/crucible-rs/architecture/fulmen-helper-library-standard.md).

## Contributing

Contributions are welcome! Please ensure:

- Code follows Rust idioms and conventions
- Tests are included for new functionality
- Documentation is updated
- Changes are consistent with Crucible standards
- `make check-all` passes before submitting

## License

Licensed under the MIT License. See [LICENSE](LICENSE) file for details.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.
