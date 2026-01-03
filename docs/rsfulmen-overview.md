# rsfulmen Overview

rsfulmen is the Rust helper library for the Fulmen ecosystem. It embeds Crucible SSOT assets (schemas/docs/config) and exposes stable, typed APIs for common catalogs and helpers.

## Feature Gates (Why They Matter)

rsfulmen supports minimal installs for lightweight consumers (e.g. sysprims) by splitting functionality across Cargo features.

### Minimal install: signals + exit codes

```toml
[dependencies]
rsfulmen = { version = "0.1.0", default-features = false, features = ["foundry-core"] }
```

Example:

```rust
use rsfulmen::foundry::signals;
use rsfulmen::foundry::exit_codes;

let sigkill = signals::SIGKILL;
let exit_137 = exit_codes::EXIT_SIGNAL_KILL;
assert_eq!(exit_codes::get_signal_from_exit(exit_137), Some(sigkill));
```

### Add MIME types

```toml
rsfulmen = { version = "0.1.0", default-features = false, features = ["foundry-mime-types"] }
```

### Add patterns (regex/glob)

```toml
rsfulmen = { version = "0.1.0", default-features = false, features = ["foundry-patterns"] }
```

### Similarity (standalone)

```toml
rsfulmen = { version = "0.1.0", default-features = false, features = ["similarity"] }
```

API:

- `rsfulmen::similarity` (preferred)
- `rsfulmen::foundry::similarity` (compat re-export)

### Schema validation (heavy)

```toml
rsfulmen = { version = "0.1.0", default-features = false, features = ["schema-validation"] }
```

This module supports offline validation using embedded meta-schemas (Draft 2020-12 default; Draft-07 supported when declared), and supports schemas stored as YAML (comments allowed).

## Crucible module registry

Crucible v0.4.x introduces module/catelog registries describing weight (`light|heavy`) and `default_inclusion` guidance.
rsfulmen maps those concepts to Cargo features but keeps Rust-specific implementation details in this repository.

See:
- `docs/crucible-rs/standards/fulmen/module-registry.md`
