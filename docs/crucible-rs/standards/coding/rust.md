---
title: "Rust Coding Standards for FulmenHQ"
description: "Rust-specific coding standards including ownership patterns, error handling, async patterns, testing, and logging for enterprise-grade Rust development"
author: "EA Steward"
author_of_record: "Dave Thompson (https://github.com/3leapsdave)"
supervised_by: "@3leapsdave"
date: "2025-11-30"
last_updated: "2025-11-30"
status: "draft"
tags: ["standards", "coding", "rust", "async", "error-handling", "testing"]
related_docs: ["README.md"]
---

# Rust Coding Standards for FulmenHQ

## Overview

This document establishes coding standards for FulmenHQ Rust projects, ensuring consistency, quality, and adherence to enterprise-grade practices. As tools designed for scale, FulmenHQ projects require rigorous standards to maintain reliability and structured output integrity.

**Core Principle**: Write idiomatic Rust code that is safe, performant, and maintainable, with strict output hygiene and proper error handling.

**Foundation**: This guide builds upon **[Cross-Language Coding Standards](README.md)** which establishes patterns for:

- Output hygiene (STDERR for logs, STDOUT for data)
- RFC3339 timestamps
- Schema validation with goneat
- CLI exit codes
- Logging standards
- Security practices

Read the cross-language standards first, then apply the Rust-specific patterns below.

---

## 1. Critical Rules (Zero-Tolerance)

### 1.1 Rust Version and MSRV

**Minimum Supported Rust Version (MSRV)**: 1.70

**Why 1.70**:

- Stable `OnceCell` and `OnceLock` in std
- Improved async trait support
- Modern error handling patterns
- Wide toolchain availability

```toml
# Cargo.toml
[package]
rust-version = "1.70"
edition = "2021"
```

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.70"
components = ["rustfmt", "clippy"]
```

### 1.2 Output Hygiene - CRITICAL

**Rule**: Output streams must remain clean for structured output (JSON, YAML) consumed by tools and automation.

**DO**: Use `tracing` macros for all diagnostic output

```rust
use tracing::{debug, info, warn, error};

// Correct logging - goes to STDERR via tracing subscriber
debug!("Processing {} files", file_count);
info!(duration_ms = elapsed.as_millis(), "Operation completed");
warn!("Configuration file not found, using defaults");
error!(error = ?err, "Failed to process file");
```

**DO NOT**: Pollute output streams with print macros in library code

```rust
// CRITICAL ERROR: Breaks structured output
println!("DEBUG: Processing {}", filename);  // Never in library code
print!("Status: {}", status);                 // Never in library code
eprintln!("Error: {}", error);                // Use tracing::error! instead

// Exception: Binary entrypoints may use println! for final structured output
fn main() {
    // ... processing ...
    println!("{}", serde_json::to_string(&result).unwrap()); // OK for final output
}
```

**Why Critical**: FulmenHQ tools produce structured output consumed by:

- CI/CD pipelines expecting clean data
- Automated tools parsing results
- Agentic systems processing structured data
- Pre-commit/pre-push hooks expecting parseable output

### 1.3 No `unwrap()` or `expect()` in Library Code

```rust
// WRONG - Panics on error
let config = load_config(path).unwrap();
let value = map.get("key").expect("key should exist");

// CORRECT - Propagate errors
let config = load_config(path)?;
let value = map.get("key").ok_or_else(|| Error::MissingKey("key"))?;

// Exception: Tests and examples may use unwrap/expect with clear context
#[test]
fn test_config_loading() {
    let config = load_config(test_path()).expect("test config should load");
    assert_eq!(config.port, 8080);
}
```

### 1.4 No `unsafe` Without Documentation

```rust
// WRONG - Undocumented unsafe
unsafe {
    ptr::write(dest, value);
}

// CORRECT - Documented safety invariants
// SAFETY: `dest` is a valid, aligned pointer obtained from Box::into_raw()
// and has not been deallocated. We have exclusive access via &mut self.
unsafe {
    ptr::write(dest, value);
}
```

**Rule**: Every `unsafe` block must have a `// SAFETY:` comment explaining why the operation is sound.

---

## 2. Code Organization and Structure

### 2.1 Project Structure

Follow FulmenHQ's established structure:

```
project/
├── src/
│   ├── lib.rs              # Library root (pub mod declarations)
│   ├── error.rs            # Error types (thiserror)
│   ├── config.rs           # Configuration handling
│   └── modules/
│       ├── mod.rs          # Module declarations
│       └── feature.rs      # Feature implementations
├── tests/
│   ├── integration/        # Integration tests
│   └── fixtures/           # Test data
├── benches/                # Benchmarks (criterion)
├── examples/               # Usage examples
├── Cargo.toml
├── rust-toolchain.toml
└── .gitignore
```

### 2.2 Naming Conventions

- **Crates/Modules**: `snake_case` (e.g., `config_loader`, `file_processor`)
- **Types/Traits**: `PascalCase` (e.g., `ConfigLoader`, `FileProcessor`)
- **Functions/Methods**: `snake_case` (e.g., `load_config`, `process_file`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RETRIES`, `DEFAULT_TIMEOUT`)
- **Type Parameters**: Single uppercase letter or `PascalCase` (e.g., `T`, `Config`)
- **Lifetimes**: Short lowercase (e.g., `'a`, `'de` for deserialize)

### 2.3 Module Organization

```rust
// lib.rs - Public API surface

// Re-exports for convenient access
pub use config::Config;
pub use error::{Error, Result};

// Public modules
pub mod config;
pub mod error;
pub mod foundry;

// Internal modules (not part of public API)
mod internal;
```

### 2.4 Visibility Guidelines

```rust
// CORRECT - Minimal visibility
pub struct Config {           // Public: part of API
    pub(crate) inner: Inner,  // Crate-visible: internal use
    port: u16,                // Private: implementation detail
}

impl Config {
    pub fn new() -> Self { }           // Public constructor
    pub(crate) fn validate(&self) { }  // Internal validation
    fn compute_hash(&self) { }         // Private helper
}
```

**Rule**: Use the minimum visibility required. Prefer `pub(crate)` over `pub` for internal APIs.

---

## 3. Error Handling

### 3.1 Error Type Design with `thiserror`

```rust
use thiserror::Error;

/// Errors that can occur during configuration loading
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration file not found: {path}")]
    NotFound { path: std::path::PathBuf },

    #[error("invalid configuration format: {0}")]
    InvalidFormat(#[from] serde_yaml::Error),

    #[error("validation failed: {message}")]
    Validation { message: String, field: Option<String> },

    #[error("I/O error reading configuration")]
    Io(#[from] std::io::Error),
}
```

### 3.2 Result Type Alias

```rust
// error.rs
pub type Result<T> = std::result::Result<T, Error>;

// Usage
pub fn load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&content)?;
    config.validate()?;
    Ok(config)
}
```

### 3.3 Error Context with `anyhow` (Applications Only)

```rust
// For applications/binaries - use anyhow for rich context
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config(&args.config_path)
        .with_context(|| format!("failed to load config from {:?}", args.config_path))?;

    process_files(&config)
        .context("file processing failed")?;

    Ok(())
}
```

**Rule**: Use `thiserror` for libraries, `anyhow` for applications.

### 3.4 Error Propagation Patterns

```rust
// CORRECT - Use ? operator for propagation
fn process(input: &str) -> Result<Output> {
    let parsed = parse(input)?;
    let validated = validate(parsed)?;
    let result = transform(validated)?;
    Ok(result)
}

// CORRECT - Add context when propagating
fn load_user_config() -> Result<Config> {
    let path = get_config_path()?;
    let content = std::fs::read_to_string(&path)
        .map_err(|e| ConfigError::Io { path: path.clone(), source: e })?;
    parse_config(&content)
}
```

---

## 4. Type Safety and Patterns

### 4.1 Ownership and Borrowing

```rust
// CORRECT - Take ownership when storing
pub struct Processor {
    config: Config,  // Owned
}

impl Processor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

// CORRECT - Borrow when only reading
impl Processor {
    pub fn process(&self, input: &str) -> Result<Output> {
        // Borrow self and input
    }
}

// CORRECT - Use Cow for flexible ownership
use std::borrow::Cow;

pub fn normalize(input: &str) -> Cow<'_, str> {
    if input.contains('\t') {
        Cow::Owned(input.replace('\t', "    "))
    } else {
        Cow::Borrowed(input)
    }
}
```

### 4.2 Lifetime Annotations

```rust
// CORRECT - Explicit lifetimes when needed
pub struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    pub fn next_token(&mut self) -> Option<&'a str> {
        // Returns slice from input with same lifetime
    }
}
```

### 4.3 Generic Constraints

```rust
// CORRECT - Minimal bounds
pub fn process<T: AsRef<str>>(input: T) -> Result<Output> {
    let s = input.as_ref();
    // ...
}

// CORRECT - Where clause for complex bounds
pub fn serialize<T>(value: &T) -> Result<String>
where
    T: Serialize + Debug,
{
    serde_json::to_string(value).map_err(Into::into)
}
```

### 4.4 Builder Pattern

```rust
#[derive(Debug, Clone)]
pub struct Config {
    host: String,
    port: u16,
    timeout: Duration,
}

#[derive(Debug, Default)]
pub struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<Duration>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn build(self) -> Result<Config, ConfigError> {
        Ok(Config {
            host: self.host.unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
        })
    }
}
```

### 4.5 Schema-Driven Configuration Hydration

- Provide a centralized deserialization layer using `serde` that maps schema-authored camelCase keys to Rust snake_case fields via `#[serde(rename_all = "camelCase")]` or field-level `#[serde(rename = "...")]`.
- Use `#[serde(default)]` for optional fields with sensible defaults; validate required fields at construction time rather than relying on `Option<T>` everywhere.
- Implement `TryFrom<RawConfig>` or a dedicated `normalize()` function that validates, coerces enums, and flattens nested structures before constructing typed instances.
- Test hydration with fixtures covering every field, including zero/empty values, enum variants, and nested structures. Validate against Crucible schemas using `jsonschema` crate or equivalent.
- Keep the normalization logic pure and deterministic for cross-language parity.

---

## 5. Async and Concurrency

### 5.1 Async Runtime

**Standard**: Use `tokio` as the async runtime for FulmenHQ Rust projects.

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Application entry point
}

// Or for libraries providing async APIs
pub async fn fetch_data(url: &str) -> Result<Data> {
    let response = reqwest::get(url).await?;
    let data = response.json().await?;
    Ok(data)
}
```

### 5.2 Send and Sync Bounds

```rust
// CORRECT - Ensure futures are Send for multi-threaded runtimes
pub async fn process<T>(input: T) -> Result<Output>
where
    T: AsRef<str> + Send,
{
    // ...
}

// CORRECT - Use Arc for shared ownership across threads
use std::sync::Arc;

pub struct SharedState {
    data: Arc<RwLock<HashMap<String, Value>>>,
}
```

### 5.3 Avoiding Async Pitfalls

```rust
// WRONG - Holding lock across await point
async fn bad_pattern(state: &Mutex<State>) {
    let guard = state.lock().await;
    do_async_work().await;  // Lock held across await!
    drop(guard);
}

// CORRECT - Release lock before await
async fn good_pattern(state: &Mutex<State>) {
    let data = {
        let guard = state.lock().await;
        guard.clone()
    };  // Lock released here
    do_async_work_with(data).await;
}
```

### 5.4 Cancellation Safety

```rust
use tokio::select;
use tokio_util::sync::CancellationToken;

pub async fn cancellable_operation(cancel: CancellationToken) -> Result<Output> {
    select! {
        result = do_work() => result,
        _ = cancel.cancelled() => Err(Error::Cancelled),
    }
}
```

---

## 6. Logging and Observability

### 6.1 Tracing Setup

```rust
use tracing::{debug, info, warn, error, instrument, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env()
            .add_directive(Level::INFO.into()))
        .init();
}
```

### 6.2 Structured Logging

```rust
use tracing::{info, instrument, Span};

#[instrument(skip(config), fields(user_id = %user.id))]
pub async fn process_request(user: &User, config: &Config) -> Result<Response> {
    info!(endpoint = %config.endpoint, "Starting request processing");

    let result = fetch_data(&config.endpoint).await?;

    info!(
        items_processed = result.len(),
        duration_ms = elapsed.as_millis(),
        "Request processing complete"
    );

    Ok(result)
}
```

### 6.3 Span Context

```rust
use tracing::{info_span, Instrument};

pub async fn batch_process(items: Vec<Item>) -> Result<Vec<Output>> {
    let mut results = Vec::with_capacity(items.len());

    for (idx, item) in items.into_iter().enumerate() {
        let span = info_span!("process_item", item_index = idx);
        let result = process_single(item)
            .instrument(span)
            .await?;
        results.push(result);
    }

    Ok(results)
}
```

### 6.4 Log Levels

| Level    | Usage                         | Example                               |
| -------- | ----------------------------- | ------------------------------------- |
| `error!` | Errors that prevent operation | "Failed to connect to database"       |
| `warn!`  | Warning conditions, non-fatal | "Config file missing, using defaults" |
| `info!`  | Operational messages          | "Server started on port 8080"         |
| `debug!` | Detailed diagnostic info      | "Parsed 42 records from input"        |
| `trace!` | Very detailed tracing         | "Entering function with args: ..."    |

---

## 7. Testing Standards

### 7.1 Test Organization

```rust
// Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_input() {
        let result = parse("valid input");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_input() {
        let result = parse("");
        assert!(matches!(result, Err(Error::EmptyInput)));
    }
}
```

```
// Integration tests in tests/ directory
tests/
├── integration/
│   ├── mod.rs
│   ├── config_loading.rs
│   └── end_to_end.rs
└── fixtures/
    ├── valid_config.yaml
    └── invalid_config.yaml
```

### 7.2 Test Naming

```rust
#[test]
fn parse_returns_error_for_empty_input() { }

#[test]
fn config_loads_from_valid_yaml_file() { }

#[test]
fn process_handles_unicode_correctly() { }
```

**Convention**: `{function_under_test}_{expected_behavior}_{scenario}`

### 7.3 Async Tests

```rust
#[tokio::test]
async fn fetch_returns_data_for_valid_url() {
    let data = fetch_data("https://api.example.com/data").await;
    assert!(data.is_ok());
}

#[tokio::test]
async fn fetch_returns_error_for_invalid_url() {
    let result = fetch_data("not-a-url").await;
    assert!(matches!(result, Err(Error::InvalidUrl(_))));
}
```

### 7.4 Test Fixtures

```rust
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn load_config_from_fixture() {
    let path = fixtures_dir().join("valid_config.yaml");
    let config = Config::load(&path).expect("fixture should load");
    assert_eq!(config.port, 8080);
}
```

### 7.5 Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_never_panics(input in ".*") {
        let _ = parse(&input);  // Should not panic
    }

    #[test]
    fn roundtrip_preserves_data(original: Config) {
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Config = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}
```

### 7.6 Schema Contract Fixtures & Golden Events

- Maintain canonical fixtures under `tests/fixtures/` covering every configuration variant. Tag fixtures with the Crucible schema version they target.
- Require tests to load fixtures through the normalization layer and validate both hydrated configs and emitted events against schemas using `jsonschema` crate.
- Use `insta` or similar for snapshot testing to detect behavioral drift.
- Coordinate fixture updates with other language foundations for cross-language parity.

---

## 8. Security and Validation

### 8.1 Input Validation

```rust
use std::path::{Path, PathBuf};

pub fn validate_path(path: &Path) -> Result<PathBuf> {
    let canonical = path.canonicalize()
        .map_err(|e| Error::InvalidPath { path: path.to_owned(), source: e })?;

    // Check for path traversal
    if canonical.components().any(|c| c == std::path::Component::ParentDir) {
        return Err(Error::PathTraversal { path: path.to_owned() });
    }

    Ok(canonical)
}
```

### 8.2 Secrets Management

```rust
// CORRECT - Environment variables for secrets
let api_key = std::env::var("API_KEY")
    .map_err(|_| Error::MissingEnvVar("API_KEY"))?;

// CORRECT - Use secrecy crate for sensitive data
use secrecy::{Secret, ExposeSecret};

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

impl Credentials {
    pub fn authenticate(&self) -> bool {
        // Access secret only when needed
        check_password(self.password.expose_secret())
    }
}

// WRONG - Hardcoded secrets
let api_key = "sk_live_12345...";  // NEVER DO THIS
```

### 8.3 Safe Defaults

```rust
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),  // Bind to localhost by default
            port: 8080,
            max_connections: 100,  // Reasonable limit
        }
    }
}
```

---

## 9. Code Style and Formatting

### 9.1 Rustfmt Configuration

FulmenHQ Rust projects use a standard `rustfmt.toml` configuration aligned with MSRV 1.70.

```toml
# rustfmt.toml
edition = "2021"
newline_style = "Unix"
max_width = 100
tab_spaces = 4

# Consistency
wrap_comments = true
comment_width = 80
normalize_doc_attributes = true

# Unstable options (commented out for MSRV 1.70 compatibility)
# imports_granularity = "Crate"
# group_imports = "StdExternalCrate"
```

**Reference implementation:** `lang/rust/rustfmt.toml` in Crucible.

### 9.2 Documentation Comments

````rust
/// Loads configuration from the specified path.
///
/// This function reads a YAML configuration file and validates its contents
/// against the expected schema.
///
/// # Arguments
///
/// * `path` - Path to the configuration file
///
/// # Returns
///
/// Returns the parsed configuration on success, or an error if the file
/// cannot be read or contains invalid data.
///
/// # Errors
///
/// This function will return an error if:
/// - The file does not exist
/// - The file contains invalid YAML
/// - The configuration fails validation
///
/// # Examples
///
/// ```
/// use mylib::Config;
///
/// let config = Config::load("config.yaml")?;
/// println!("Server port: {}", config.port);
/// ```
pub fn load(path: impl AsRef<Path>) -> Result<Config> {
    // ...
}
````

### 9.3 Clippy Configuration

**MSRV Note:** The `[lints]` table in `Cargo.toml` requires Rust 1.74+. For MSRV 1.70 compatibility, use crate-level attributes in `lib.rs`.

**Crate-level attributes (`src/lib.rs`):**

```rust
// Standard Safety & Quality
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

// Clippy Groups
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]

// Allowances (with justification)
#![allow(clippy::module_name_repetitions)]  // Common in module::Type patterns
#![allow(clippy::multiple_crate_versions)]  // Dependency tree may require
#![allow(clippy::match_same_arms)]          // Acceptable for category grouping
#![allow(clippy::doc_markdown)]             // See .clippy.toml for valid-idents
```

**Clippy configuration (`.clippy.toml`):**

```toml
# Clippy configuration
cognitive-complexity-threshold = 30

# Words that should be considered valid identifiers in documentation
doc-valid-idents = [
    "FulmenHQ",
    "Crucible",
    "Fulmen",
    "Fulpack",
    "Fulencode",
    "Fulhash",
    "Opencode",
    "Lorage",
    "DevSecOps",
    "SSOT",
    "Codex",
    "Foundry",
    "GitHub",
    "OAuth",
    "OpenID",
    "OIDC",
    "SAML",
    "JWT",
    "mTLS",
    "gRPC",
]
```

**CI enforcement:** Run `cargo clippy -- -D warnings` to treat all warnings as errors.

**Reference implementation:** `lang/rust/src/lib.rs` and `lang/rust/.clippy.toml` in Crucible.

---

## 10. Common Anti-Patterns to Avoid

### 10.1 Clone Overuse

```rust
// WRONG - Unnecessary clones
fn process(data: String) {
    let copy = data.clone();
    do_something(&copy);
}

// CORRECT - Borrow instead
fn process(data: &str) {
    do_something(data);
}
```

### 10.2 Stringly-Typed APIs

```rust
// WRONG - String for finite set of values
fn set_status(status: &str) {
    match status {
        "pending" | "active" | "done" => { }
        _ => panic!("invalid status"),
    }
}

// CORRECT - Use enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pending,
    Active,
    Done,
}

fn set_status(status: Status) {
    // Compile-time guarantee of valid status
}
```

### 10.3 Ignoring Results

```rust
// WRONG - Silently ignoring errors
let _ = write_file(path, content);

// CORRECT - Handle or propagate
write_file(path, content)?;

// CORRECT - Explicitly ignore with documentation
// We intentionally ignore cleanup errors during shutdown
let _ = cleanup_temp_files();
```

---

## 11. Code Review Checklist

Before submitting code, verify:

- [ ] MSRV compatibility (Rust 1.70+)
- [ ] No `println!`/`print!` in library code (use `tracing`)
- [ ] No `unwrap()`/`expect()` in library code (propagate errors)
- [ ] All `unsafe` blocks have `// SAFETY:` comments
- [ ] Error types use `thiserror` with descriptive messages
- [ ] Public API has documentation with examples
- [ ] Tests cover happy path and error conditions
- [ ] `cargo fmt` produces no changes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] No hardcoded secrets or credentials
- [ ] Path inputs validated against traversal attacks

---

## 12. Tools and Enforcement

### 12.1 Required Tools

- **rustfmt**: Code formatting
- **clippy**: Linting and best practices
- **cargo**: Build and dependency management

### 12.2 Recommended Tools

- **cargo-deny**: License and vulnerability checking
- **cargo-audit**: Security vulnerability scanning
- **criterion**: Benchmarking
- **insta**: Snapshot testing
- **proptest**: Property-based testing

### 12.3 CI Integration

```yaml
# Example GitHub Actions workflow
- name: Format check
  run: cargo fmt --check

- name: Clippy
  run: cargo clippy -- -D warnings

- name: Tests
  run: cargo test

- name: Security audit
  run: cargo audit
```

---

## 13. Portable Testing

Fulmen Rust projects must keep `cargo test` deterministic across laptops, CI, and sandboxed environments. Follow the cross-language [Portable Testing Practices](../testing/portable-testing-practices.md) and apply these Rust patterns:

- Use `tempfile` crate for temporary directories; never write to hardcoded paths.
- Bind sockets with port `0` to let the OS assign ephemeral ports:
  ```rust
  let listener = TcpListener::bind("127.0.0.1:0").await?;
  let port = listener.local_addr()?.port();
  ```
- Provide shared skip helpers that check capabilities and skip with clear messages:
  ```rust
  fn require_network() {
      if std::env::var("SKIP_NETWORK_TESTS").is_ok() {
          eprintln!("Skipping: SKIP_NETWORK_TESTS is set");
          return;
      }
  }
  ```
- Seed randomness deterministically for reproducible tests:
  ```rust
  use rand::{SeedableRng, rngs::StdRng};
  let mut rng = StdRng::seed_from_u64(42);
  ```
- Use `#[ignore]` attribute for tests requiring special capabilities, with documentation:
  ```rust
  #[test]
  #[ignore = "requires network access"]
  fn test_external_api() { }
  ```
- Clean up resources in `Drop` implementations or explicit cleanup blocks to prevent cross-test interference.

**CLI Testing**: For `clap`-based CLIs, construct fresh `Command` instances per test rather than using global state. See [Language-Specific Testing Patterns](../testing/language-testing-patterns.md) for detailed patterns.

---

## Conclusion

These standards ensure FulmenHQ Rust projects maintain reliability as production-grade tools. The emphasis on safety, proper error handling, and output hygiene is critical for maintaining code quality and seamless automation integration.

**Remember**: Rust's compiler is your ally. Let it catch bugs at compile time. Propagate errors properly. Keep output streams clean.

_Adherence to these standards ensures enterprise-grade reliability and seamless integration across development workflows._
