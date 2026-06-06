# rsfulmen Overview

rsfulmen is the Rust helper library for the Fulmen ecosystem. It embeds Crucible SSOT assets (schemas/docs/config) and exposes stable, typed APIs for common catalogs and helpers.

## When to Use rsfulmen

### Use rsfulmen when you need:

| Need                    | Feature             | Why rsfulmen                                         |
| ----------------------- | ------------------- | ---------------------------------------------------- |
| Consistent exit codes   | `foundry-core`      | Same codes as your Go/Python/TypeScript services     |
| HTTP status helpers     | `foundry-core`      | Grouping (2xx/4xx/5xx) matches cross-language        |
| Country code validation | `foundry-core`      | ISO 3166-1 with triple-index (alpha2/alpha3/numeric) |
| XDG config paths        | `config`            | Cross-platform without reimplementing                |
| Structured errors       | `error-handling`    | Schema-valid envelopes with correlation IDs          |
| Metrics emission        | `telemetry-metrics` | Taxonomy-backed, ready for your observability stack  |

### Consider alternatives when:

| Scenario                       | Alternative            | Why                                           |
| ------------------------------ | ---------------------- | --------------------------------------------- |
| Single HTTP status check       | Inline match           | Don't add a dependency for one lookup         |
| Custom exit code scheme        | Own constants          | rsfulmen assumes Crucible's 54-code catalog   |
| No cross-language requirements | Language-native crates | If you're Rust-only, `http` crate may suffice |

## Feature Selection Guide

Choose features based on your dependency budget and needs:

```
                           ┌─────────────────────────────────────┐
                           │ What do you need?                   │
                           └─────────────────────────────────────┘
                                          │
              ┌───────────────────────────┼───────────────────────────┐
              ▼                           ▼                           ▼
   ┌──────────────────┐        ┌──────────────────┐        ┌──────────────────┐
   │ Just catalogs    │        │ Text processing  │        │ Full validation  │
   │ (exit codes,     │        │ (patterns,       │        │ (JSON schemas,   │
   │ countries, HTTP) │        │ similarity)      │        │ error envelopes) │
   └──────────────────┘        └──────────────────┘        └──────────────────┘
              │                           │                           │
              ▼                           ▼                           ▼
   ┌──────────────────┐        ┌──────────────────┐        ┌──────────────────┐
   │ foundry-core     │        │ foundry-patterns │        │ schema-validation│
   │                  │        │ similarity       │        │ error-handling   │
   │ Zero heavy deps  │        │ Adds regex/glob  │        │ Adds jsonschema  │
   └──────────────────┘        └──────────────────┘        └──────────────────┘
```

## Feature Gates (Why They Matter)

rsfulmen supports minimal installs for lightweight consumers (e.g. sysprims) by splitting functionality across Cargo features.

### Minimal install: signals + exit codes

**Best for**: CLI tools, system utilities, small services

```toml
[dependencies]
rsfulmen = { version = "0.1", default-features = false, features = ["foundry-core"] }
```

**What you get**: Exit codes, signals, country codes, HTTP statuses — all with zero heavy dependencies.

Example:

```rust
use rsfulmen::foundry::signals;
use rsfulmen::foundry::exit_codes;

let sigkill = signals::SIGKILL;
let exit_137 = exit_codes::EXIT_SIGNAL_KILL;
assert_eq!(exit_codes::get_signal_from_exit(exit_137), Some(sigkill));
```

### Add MIME types

**Best for**: File processing, content detection

```toml
rsfulmen = { version = "0.1", default-features = false, features = ["foundry-mime-types"] }
```

**Trade-off**: Adds `serde_json` dependency.

### Add patterns (regex/glob)

**Best for**: Configuration validation, file matching

```toml
rsfulmen = { version = "0.1", default-features = false, features = ["foundry-patterns"] }
```

**Trade-off**: Adds `regex` and `glob` dependencies.

### Similarity (standalone)

**Best for**: Fuzzy matching, typo correction, search suggestions

```toml
rsfulmen = { version = "0.1", default-features = false, features = ["similarity"] }
```

API:

- `rsfulmen::similarity` (preferred)
- `rsfulmen::foundry::similarity` (compat re-export)

**Trade-off**: Adds `strsim` and unicode normalization dependencies.

### Schema validation (heavy)

**Best for**: Configuration validation, API response validation

```toml
rsfulmen = { version = "0.1", default-features = false, features = ["schema-validation"] }
```

This module supports offline validation using embedded meta-schemas (Draft 2020-12 default; Draft-07 supported when declared), and supports schemas stored as YAML (comments allowed).

**Trade-off**: Adds `jsonschema` and `url` dependencies — significant binary size increase.

## Dependency Impact

| Feature              | Added Dependencies | Approx. Binary Impact |
| -------------------- | ------------------ | --------------------- |
| `foundry-core`       | None (beyond std)  | Minimal               |
| `foundry-mime-types` | serde_json         | ~200KB                |
| `foundry-patterns`   | regex, glob        | ~500KB                |
| `similarity`         | strsim, unicode-\* | ~300KB                |
| `schema-validation`  | jsonschema, url    | ~1MB                  |
| `error-handling`     | serde_json         | ~200KB                |
| `telemetry-metrics`  | serde_json         | ~200KB                |
| `fulpack`            | tar, flate2, zip   | ~600KB                |

_Binary sizes are approximate and depend on optimization settings._

## Crucible Module Registry

Crucible v0.4.x introduces module/catalog registries describing weight (`light|heavy`) and `default_inclusion` guidance. rsfulmen maps those concepts to Cargo features but keeps Rust-specific implementation details in this repository.

See:

- `docs/crucible-rs/standards/fulmen/module-registry.md`

## Cross-Language Parity

rsfulmen's catalogs match the other Fulmen helper libraries exactly:

| Catalog       | rsfulmen         | gofulmen         | pyfulmen         | tsfulmen         |
| ------------- | ---------------- | ---------------- | ---------------- | ---------------- |
| Exit codes    | 54 codes         | 54 codes         | 54 codes         | 54 codes         |
| HTTP statuses | Full registry    | Full registry    | Full registry    | Full registry    |
| Country codes | ISO 3166-1       | ISO 3166-1       | ISO 3166-1       | ISO 3166-1       |
| Signals       | POSIX + platform | POSIX + platform | POSIX + platform | POSIX + platform |

This parity is enforced by Crucible SSOT — all libraries sync from the same source.
