# rsfulmen Integration Guide

**For library consumers.** This guide shows how to add rsfulmen to a Rust application
and pick the features you actually need. Behaviour contracts come from
[Crucible](https://github.com/fulmenhq/crucible) (embedded snapshot **v0.4.19**).
gofulmen and tsfulmen are worked examples of the same contracts; Rust APIs are
idiomatic (`Result`, feature flags, rustdoc) and are not a Go-shaped port.

For contributor / SSOT tooling, see [GONEAT.md](GONEAT.md). For the module catalog
and feature-gate trade-offs, see [rsfulmen-overview.md](rsfulmen-overview.md).

## What is rsfulmen?

rsfulmen is the **Rust helper library** for the Fulmen ecosystem:

- **Foundry catalogs** — exit codes, HTTP statuses, country codes, signals, MIME types, patterns
- **Config paths** — XDG-compliant config/data/cache directories
- **Logging** — Simple (human) and Structured (JSON) profiles from the Crucible observability standard
- **Pathfinder** — safe filesystem discovery with glob filters and repo-root detection
- **Fulhash** — portable `algo:hex` digests (xxh3-128 default, SHA-256)
- **Crucible access** — embedded schemas, standards, and metadata via a single crate

Catalog data is embedded at compile time. There is no runtime network call for
reference data.

## Installation

Add to `Cargo.toml`. Default features match a full helper-library install.

```toml
[dependencies]
rsfulmen = "0.2"
```

### Minimal / feature-gated installs

Keep the core crate lean. Enable only what you use:

```toml
[dependencies]
# Catalogs only (signals, exit-codes, countries, HTTP statuses)
rsfulmen = { version = "0.2", default-features = false, features = ["foundry-core"] }

# Catalogs + XDG paths
rsfulmen = { version = "0.2", default-features = false, features = ["foundry-core", "config"] }

# Host-binary identity from an application build.rs (not a runtime feature)
rsfulmen = { version = "0.2", default-features = false, features = ["host-identity-producer"] }
```

See the feature matrix in the [README](../README.md#feature-flags).

## Quick start

```rust
use rsfulmen::config::{get_app_config_dir, get_fulmen_config_dir};
use rsfulmen::foundry::exit_codes::{lookup_exit_code, EXIT_CONFIG_INVALID};
use rsfulmen::logging::{self, Severity};

fn main() {
    let log = logging::new_cli("myapp", Severity::Info);
    log.info("service started", &[("config", &format!("{:?}", get_app_config_dir("myapp")))]);

    let fulmen_dir = get_fulmen_config_dir();
    log.info("fulmen config dir", &[("path", &format!("{:?}", fulmen_dir))]);

    let code = lookup_exit_code(EXIT_CONFIG_INVALID).expect("catalog embed");
    log.info("canonical exit", &[("name", code.name), ("code", "20")]);
}
```

## Available modules

### Config (`config`)

XDG Base Directory paths for the current platform. Fulmen ecosystem tools share
`~/.config/fulmen` (Linux); macOS and Windows use the platform equivalents.

```rust
use rsfulmen::config::{get_app_config_dir, get_fulmen_config_dir, get_xdg_base_dirs};

let app = get_app_config_dir("myapp");
let fulmen = get_fulmen_config_dir();
let xdg = get_xdg_base_dirs();
```

Environment-variable overlays live under `rsfulmen::config::env` (`EnvVarSpec`,
`load_env_overrides_with_report`). Three-layer loading is the `three-layer-config`
feature.

Public `config::merge` / `config::deep_copy` operate on `serde_yaml::Value`:
mappings merge recursively, sequences replace, and `null` deletes a mapping
key. Inputs are borrowed; the result does not alias them.

```rust
use rsfulmen::config::{deep_copy, merge};

let base = serde_yaml::from_str("a: 1\nnested: {x: 1}").unwrap();
let overlay = serde_yaml::from_str("nested: {y: 2}\nb: null").unwrap();
let merged = merge(&base, &overlay);
let _independent = deep_copy(&merged);
```

### Foundry catalogs (`foundry` / `foundry-core`)

Same catalog data as gofulmen / tsfulmen / pyfulmen, from Crucible v0.4.19.

```rust
use rsfulmen::foundry::country_codes::lookup_by_alpha2;
use rsfulmen::foundry::exit_codes::{EXIT_SUCCESS, lookup_exit_code};
use rsfulmen::foundry::http_statuses::{is_success, lookup_status};

let usa = lookup_by_alpha2("US").unwrap();
let ok = lookup_status(200).unwrap();
assert!(is_success(201));
assert_eq!(lookup_exit_code(EXIT_SUCCESS).unwrap().name, "EXIT_SUCCESS");
```

Optional foundry extras (each adds deps):

| Feature               | Adds                          |
| --------------------- | ----------------------------- |
| `foundry-mime-types`  | `serde_json`                  |
| `foundry-patterns`    | `regex`, `glob`               |
| `foundry-correlation` | `uuid` (UUIDv7 correlation IDs) |

### Logging (`logging`)

Crucible observability profiles. `Simple` is human-readable on stderr;
`Structured` is JSON lines.

```rust
use rsfulmen::logging::{self, Severity};

let log = logging::new_cli("myapp", Severity::Info);
log.info("request handled", &[("path", "/health"), ("status", "200")]);

let req = log.with_fields(&[("request_id", "abc-123")]);
req.warn("slow query", &[("duration_ms", "523")]);
```

Library crates should accept an optional logger (or a `Severity::None` no-op)
rather than installing a global logger.

### Pathfinder (`pathfinder`)

Safe discovery with glob include/exclude, traversal protection, and repository
root detection.

`find_repository_root` is the historical `.git`-only unbounded `Option` walk.
`discover_repository_root` is the bounded marker API: markers, max depth
(default 10), home or explicit ancestor ceiling, and structured errors.
`HOME=/` or `/root` falls back to the current working directory. A home
that is not an ancestor stops with `BoundaryReached`. PathConstraint /
RespectConstraints is deferred; pass `boundary` instead.
The default `follow_symlinks: false` mode rejects `..` components in start and
boundary paths and hides repository markers behind symlinked directories below
the ceiling. Aliases at or above the ceiling remain allowed.

```rust
use rsfulmen::pathfinder::{
    discover_repository_root, find_repository_root, RepositoryRootOptions,
};
use std::path::Path;

let _ = find_repository_root(Path::new("."));

let opts = RepositoryRootOptions::with_markers([".git", "Cargo.toml"]);
let _ = discover_repository_root(Path::new("."), &opts);
```

### Fulhash (`fulhash`)

Portable `algo:hex` digests shared with gofulmen / tsfulmen / pyfulmen.
Algorithms: xxh3-128 (default), sha256, crc32 (IEEE), crc32c (Castagnoli).
`multi_hash` / `multi_hash_reader` compute several digests in one pass.

```rust
use rsfulmen::fulhash::{self, Algorithm};

let digest = fulhash::hash_string("hello", None).unwrap();
assert!(fulhash::verify(b"hello", &digest));

let many = fulhash::multi_hash(b"hello", &[Algorithm::Sha256, Algorithm::Crc32]).unwrap();
assert_eq!(many.len(), 2);
```

### Signals (`foundry-core`)

Runtime signal manager: ordered cleanup, reload on SIGHUP, optional double-tap
Ctrl+C. Unix uses `signal-hook`; Windows uses `ctrlc`.

Handler panics are isolated (`catch_unwind` + `AssertUnwindSafe`). The payload
is logged after redaction (`key=value` secret assignments; plus logging-crate
pattern redaction when the `logging` feature is on). Later handlers still run,
and TERM/INT **always** enter the shutdown chain.

Shutdown hooks run under the catalog `timeout_seconds` grace period:

| Signal | Catalog `timeout_seconds` | Runtime |
| --- | --- | --- |
| SIGTERM | 30 | wait 30s for hooks, then force-exit (143) |
| SIGINT | 5 | wait 5s for hooks, then force-exit (130) |
| field absent | — | **30s** default (`DEFAULT_SHUTDOWN_GRACE`) |
| `0` | immediate | force-exit immediately; hooks are not run |

Override with `SignalManager::set_shutdown_grace`. The production force-exit
path is `process::exit`; tests/embedders can intercept it with
`set_force_exit`.

`POST /admin/signal` is a request helper, not a server. Auth, TLS, and 429
rate-limit stay with the caller. The JSON adapter
(`SignalManager::handle_admin_signal_http`) needs the `json` feature
(pulled by `logging` and other JSON modules). `foundry-core` alone keeps the
typed `handle_admin_signal_request` path and stays free of `serde_json`.

```rust
use std::time::Duration;
use rsfulmen::signals::SignalManager;

let manager = SignalManager::new();
manager.on_shutdown(|| { /* cleanup */ Ok(()) });
manager.on_reload(|| { /* reread config */ Ok(()) });
// Optional: override catalog grace (None restores catalog / 30s default).
manager.set_shutdown_grace(Some(Duration::from_secs(15)));

// Typed body (foundry-core). HTTP adapter is feature `json`.
let _ = manager.handle_admin_signal_request(rsfulmen::signals::SignalEndpointRequest {
    signal: "HUP".into(),
    reason: Some("reload".into()),
    correlation_id: None,
});
```

### Telemetry (`telemetry-metrics`)

JSON events are canonical. Prometheus text is a derived scrape format — this
crate does not listen on `/metrics`. HTTP helpers record the `http_*` taxonomy
and normalize raw paths (`/users/42` → `/users/:id`).

```rust
use rsfulmen::telemetry_metrics::{HttpRecord, HttpRecorder, Metrics};

let metrics = Metrics::new();
let http = HttpRecorder::new(metrics.clone(), "api").with_sizes();
let _inflight = http.enter().unwrap();
http.record(
    HttpRecord::new("GET", "/users/42", 200)
        .duration_ms(12.0)
        .sizes(128, 256),
)
.unwrap();
let _ = metrics.export_prometheus_text().unwrap();
```

Duration histograms use **seconds**. `HttpRecord::duration_ms` divides by 1000.

### Crucible metadata (`crucible`)

```rust
use rsfulmen::crucible;

let meta = crucible::metadata();
println!("crucible {}", meta.version);
```

Embedded snapshot is **v0.4.19**. Query `crucible::version()` at runtime rather
than hard-coding the pin in application logic.

### Other modules

| Feature              | Module                   | Typical use                          |
| -------------------- | ------------------------ | ------------------------------------ |
| `error-handling`     | `error_handling`         | Canonical error envelope             |
| `telemetry-metrics`  | `telemetry_metrics`      | JSON / Prometheus text + HTTP helpers |
| `appidentity`        | `appidentity`            | `.fulmen/app.yaml` upward search     |
| `ascii`              | `ascii`                  | Display width, box drawing           |
| `similarity`         | `similarity`             | Fuzzy match / "did you mean?"        |
| `schema-validation`  | `schema_validation`      | Offline JSON Schema (Draft 2020-12)  |
| `fulencode`          | `fulencode`              | Encode/decode, NFC, BOM              |
| `fulpack`            | `fulpack`                | tar/zip inspect/create/extract       |
| `docscribe`          | `docscribe`              | Markdown frontmatter                 |
| `host-identity-producer` | `buildinfo::producer` | Stamp host identity from `build.rs` |

## Integration patterns

### CLI

Redaction is fully supported and **off by default**. `new_cli` / `new` use an
empty middleware chain. Enable it by attaching `RedactSecretsMiddleware` first:

```rust
use rsfulmen::logging::{self, MiddlewareChain, RedactSecretsMiddleware, Severity};

fn main() {
    let log = logging::new_cli("my-tool", Severity::Info);
    let chain = MiddlewareChain::new().with(RedactSecretsMiddleware::defaults().unwrap());
    let log = log.with_middleware_chain(chain);
    if let Err(err) = run(&log) {
        log.error("command failed", &[("error", &err.to_string())]);
        std::process::exit(1);
    }
}

fn run(log: &rsfulmen::logging::Logger) -> Result<(), Box<dyn std::error::Error>> {
    log.info("ready", &[]);
    Ok(())
}
```

### Library crate

Do not create a process-wide logger inside a library. Accept one, or stay silent:

```rust
use rsfulmen::logging::{self, Logger, Severity};

pub struct Client {
    log: Logger,
}

impl Client {
    pub fn new(log: Option<Logger>) -> Self {
        Self {
            log: log.unwrap_or_else(|| logging::new_cli("mylib", Severity::None)),
        }
    }
}
```

### Host binary identity

Product CLIs should report **the binary they are**, not rsfulmen's git. In the
application `build.rs`, depend with `default-features = false` and
`features = ["host-identity-producer"]`, then call
`rsfulmen::buildinfo::producer::emit_host_identity()`. At runtime:

```rust
let info = rsfulmen::host_identity!();
println!("{}", info.format_basic("mycli"));
```

`host_identity!()` is compile-stamp-only. Missing stamps use documented defaults
and never read process `FULMEN_HOST_*`.

## Configuration locations

**Fulmen ecosystem tools** share the Fulmen directory:

```rust
let shared = rsfulmen::config::get_fulmen_config_dir();
```

**Independent applications** use their own app name:

```rust
let config = rsfulmen::config::get_app_config_dir("myapp");
let data = rsfulmen::config::get_app_data_dir("myapp");
let cache = rsfulmen::config::get_app_cache_dir("myapp");
```

## Best practices

1. **Feature-gate** — depend on `foundry-core` (or a short list) unless you need the default set.
2. **Prefer structured fields** — `log.info("msg", &[("key", "value")])` over interpolated strings.
3. **Use catalog constants** — `EXIT_CONFIG_INVALID` rather than a magic `20`, so alerting matches Go/TS/Python.
4. **Validate user paths** with pathfinder rather than concatenating strings.
5. **Log Crucible version** on service startup (`crucible::version()`).
6. **Do not** treat rsfulmen as a global process manager. Signals, logging, and identity are façades you compose.

## Troubleshooting

### Feature not found / module missing

The type lives behind a Cargo feature. Check the [feature matrix](../README.md#feature-flags)
and enable it explicitly when using `default-features = false`.

### Catalog mismatch with gofulmen / tsfulmen

Confirm the embedded Crucible version:

```rust
assert!(rsfulmen::crucible::version().contains("0.4.19"));
```

All language foundations should embed the same Crucible tag for catalog parity.

### Config directory not where you expect

Config paths follow the platform conventions documented in
[Fulmen Config Paths](crucible-rs/standards/config/fulmen-config-paths.md).
Resolution is std-only (no `dirs` crate). Absolute `XDG_*` environment
variables win on every platform; otherwise Unix, macOS, and Windows fallbacks
from that table apply.

## Support

- Issues: https://github.com/fulmenhq/rsfulmen/issues
- Crucible SSOT: https://github.com/fulmenhq/crucible
- Sister libraries: [gofulmen](https://github.com/fulmenhq/gofulmen),
  [tsfulmen](https://github.com/fulmenhq/tsfulmen),
  [pyfulmen](https://github.com/fulmenhq/pyfulmen)

## Next steps

1. Add `rsfulmen` with the smallest feature set that covers your use case.
2. Read rustdoc (`cargo doc --open --package rsfulmen`) for the modules you enabled.
3. If you contribute to rsfulmen itself, see [GONEAT.md](GONEAT.md) and
   [development/operations.md](development/operations.md).
