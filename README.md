# rsfulmen

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust: 1.83+](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Crucible: v0.4.2](https://img.shields.io/badge/crucible-v0.4.2-purple.svg)](https://github.com/fulmenhq/crucible)

**Stop reinventing catalogs. Start shipping.**

Every team writes their own HTTP status helpers, exit code enums, and country code lookups. rsfulmen provides production-grade Rust implementations derived from a single source of truth—so your Rust services use the same codes as your Go, Python, and TypeScript services.

- **Zero runtime dependencies**: All catalogs embedded at compile time
- **Cross-language parity**: Same exit codes, signals, and schemas as gofulmen, pyfulmen, tsfulmen
- **Minimal footprint**: Feature flags let you include only what you need

**Lifecycle Phase**: `alpha` | **Version**: 0.1.1

📖 **[Read the complete rsfulmen overview](docs/rsfulmen-overview.md)** for comprehensive documentation including module catalog and roadmap.

## Overview

rsfulmen provides consistent, high-quality implementations of common functionality across the FulmenHQ ecosystem. Built on Crucible's schemas and standards, it ensures uniformity and reliability with idiomatic Rust APIs.

> **Alpha Status**: API may evolve before 1.0. However:
> - **Catalog data is stable**: Exit codes, signals, country codes, and HTTP statuses derive from Crucible SSOT and won't change without ecosystem-wide coordination
> - **Breaking changes documented**: All API changes noted in [CHANGELOG.md](CHANGELOG.md)
> - **Production-viable for catalogs**: The `foundry-core` feature is suitable for production use
>
> See [Repository Lifecycle Standard](docs/crucible-rs/standards/repository-lifecycle.md) for quality expectations.

## Who Should Use This

**Platform Engineers & SREs**: Standardize exit codes across all services so alerting thresholds and runbooks work consistently—whether the service is written in Rust, Go, Python, or TypeScript.

**Security & Compliance Teams**: Fewer runtime dependencies means a smaller attack surface. Embedded catalogs eliminate network calls for reference data. Audit the dependency tree once with `cargo tree`.

**Polyglot Teams**: When your organization runs multiple languages, rsfulmen ensures your Rust services speak the same language as the rest of your stack. Same HTTP status groupings. Same signal handling semantics. Same error codes.

**Library Authors**: Build on rsfulmen's catalogs instead of maintaining your own. The feature flag system lets you depend on only `foundry-core` (zero heavy deps) while your consumers can add features as needed.

## Crucible Integration

**What is Crucible?**

Crucible is the FulmenHQ single source of truth (SSOT) for schemas, standards, and configuration templates. It ensures consistent APIs, documentation structures, and behavioral contracts across all language foundations (gofulmen, pyfulmen, tsfulmen, rsfulmen).

**Why rsfulmen?**

Rather than copying Crucible assets into every Rust project, rsfulmen provides idiomatic access through type-safe APIs. This keeps your application lightweight, versioned correctly, and aligned with ecosystem-wide standards.

**Where to Learn More:**

- [Crucible Repository](https://github.com/fulmenhq/crucible) — SSOT schemas, docs, and configs
- [Fulmen Technical Manifesto](docs/crucible-rs/architecture/fulmen-technical-manifesto.md) — Philosophy and design principles
- [gofulmen](https://github.com/fulmenhq/gofulmen) — Go reference implementation

### Crucible Version

rsfulmen embeds a snapshot of Crucible assets at build time. You can query the embedded version programmatically:

```rust
use rsfulmen::crucible;

fn main() {
    // Get full metadata
    let meta = crucible::metadata();
    println!("Crucible version: {}", meta.version);
    println!("Commit: {}", meta.commit);
    println!("Synced at: {}", meta.synced_at);
    println!("Sync method: {}", meta.sync_method);

    // Or just the version string (with 'v' prefix)
    println!("Version: {}", crucible::version());
}
```

**Metadata fields:**

| Field | Description |
|-------|-------------|
| `version` | CalVer Crucible version (e.g., `0.4.2`) |
| `commit` | Git commit SHA of the synced Crucible snapshot |
| `dirty` | `true` if synced from uncommitted changes (development only) |
| `synced_at` | RFC3339 timestamp when sync occurred |
| `sync_method` | Sync method used (e.g., `git_ref`) |

For manual inspection, see [`.crucible/metadata/metadata.yaml`](.crucible/metadata/metadata.yaml).

## Modules

### Config (`config`)

*Use case: Locate configuration files consistently across Linux, macOS, and Windows without reimplementing XDG logic.*

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

*Use case: Access standardized catalogs (countries, HTTP statuses, exit codes) that match your Go/Python/TypeScript services exactly.*

Enterprise-grade foundation utilities providing consistent cross-language implementations from Crucible catalogs. All data is embedded at compile time — no network dependencies required.

#### Country Codes

*Use case: Validate and normalize country codes in API requests without maintaining your own ISO 3166-1 dataset.*

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

*Use case: Categorize responses for metrics and logging with consistent groupings across all your services.*

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

*Use case: Return meaningful exit codes so monitoring systems can distinguish configuration errors from runtime failures.*

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

*Use case: Return structured errors with correlation IDs and severity levels that integrate with your observability stack.*

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

*Use case: Emit metrics that conform to your organization's taxonomy without building a custom metrics framework.*

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

rsfulmen is part of the Fulmen helper library family. All libraries derive their catalogs from [Crucible](https://github.com/fulmenhq/crucible), ensuring cross-language consistency:

| Library | Language | Status | Crucible Version |
|---------|----------|--------|------------------|
| [gofulmen](https://github.com/fulmenhq/gofulmen) | Go | Reference impl | v0.4.x |
| [tsfulmen](https://github.com/fulmenhq/tsfulmen) | TypeScript | Stable | v0.4.x |
| [pyfulmen](https://github.com/fulmenhq/pyfulmen) | Python | Stable | v0.4.x |
| rsfulmen | Rust | Alpha | v0.4.2 |

**Why this matters**: A Rust service using `EXIT_CONFIG_INVALID` (code 20) will match a Go service using the same exit code. Your alerting rules and runbooks work across the entire stack.

All libraries sync from [Crucible](https://github.com/fulmenhq/crucible) and follow the [Fulmen Helper Library Standard](docs/crucible-rs/architecture/fulmen-helper-library-standard.md).

## Supply Chain & Security

rsfulmen is designed for environments where dependency hygiene matters.

**Dependency Transparency:**
- **Minimal by default**: `foundry-core` feature has zero heavy dependencies
- **Auditable**: Run `cargo tree` to inspect the full dependency graph
- **SBOM-ready**: Compatible with `cargo sbom` and `cargo cyclonedx`
- **License-clean**: All dependencies use MIT, Apache-2.0, or compatible licenses

**Embedded Data:**
- All Crucible catalogs (country codes, exit codes, HTTP statuses) are embedded at compile time
- No runtime network calls for reference data
- Version and provenance tracked in `.crucible/metadata/metadata.yaml`

**Security Practices:**
- No `unsafe` code in core modules
- Pattern matching uses bounded execution (no ReDoS vulnerabilities)
- Vulnerability scanning via `cargo audit`

**Audit Commands:**
```bash
# View dependency tree
cargo tree

# Check for known vulnerabilities
cargo audit

# Generate SBOM
cargo sbom > sbom.json
```

See [SECURITY.md](SECURITY.md) for vulnerability reporting and our full security policy.

## Contributing

Contributions are welcome! Please ensure:

- Code follows Rust idioms and conventions
- Tests are included for new functionality
- Documentation is updated
- Changes are consistent with Crucible standards
- `make check-all` passes before submitting

See [MAINTAINERS.md](MAINTAINERS.md) for governance and [SECURITY.md](SECURITY.md) for vulnerability reporting.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) file for details.

**Trademarks**: "Fulmen" and "3 Leaps" are trademarks of 3 Leaps, LLC. While code is open source, please use distinct names for derivative works to prevent confusion.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

---

<div align="center">

**Built by the [3 Leaps](https://3leaps.net) team**

Part of the [Fulmen Ecosystem](https://github.com/fulmenhq) — Enterprise-grade libraries that thrive on scale

</div>
