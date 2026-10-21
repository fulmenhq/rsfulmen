//! # rsfulmen
//!
//! Rust helper library for the [Fulmen ecosystem](https://github.com/fulmenhq).
//!
//! rsfulmen provides foundry catalogs, config utilities, and cross-platform helpers
//! that follow the same patterns as [tsfulmen](https://github.com/fulmenhq/tsfulmen)
//! and [pyfulmen](https://github.com/fulmenhq/pyfulmen).
//!
//! ## Features
//!
//! - **foundry** (default) - Foundry catalogs (core + opt-in heavy catalogs)
//! - **foundry-core** - Lightweight catalogs (signals, exit-codes, etc.)
//! - **config** (default) - XDG-compliant configuration path utilities
//! - **crucible** (default) - Embedded Crucible assets + metadata shim
//! - **docscribe** (default) - Doc access + frontmatter parsing
//! - **three-layer-config** (default) - Enterprise layered config loading
//! - **schema-id** (default) - schema_id tagging + offline resolution
//! - **similarity** (default) - Text similarity + suggestions (heavy)
//! - **schema-validation** (default) - JSON Schema validation helpers (heavy)
//! - **error-handling** (default) - Canonical error envelope + propagation
//! - **telemetry-metrics** (default) - Telemetry + metrics export
//! - **appidentity** (default) - App identity discovery from `.fulmen/app.yaml`
//! - **logging** (default) - Structured logging with SIMPLE/STRUCTURED profiles
//! - **fulhash** (default) - Canonical hashing (xxh3-128, SHA-256, CRC-32, CRC-32C)
//! - **pathfinder** (default) - Safe filesystem discovery with glob patterns
//! - **ascii** (default) - Terminal utilities + Unicode-aware string handling
//! - **fulencode** (default) - Encoding/decoding, detection, normalization, BOM helpers
//! - **buildinfo** (always) - Host binary identity for `version --extended`
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use rsfulmen::config::get_fulmen_config_dir;
//!
//! // Get the Fulmen config directory for the current platform
//! let config_dir = get_fulmen_config_dir();
//! println!("Fulmen config: {:?}", config_dir);
//! ```
//!
//! ## Crucible Integration
//!
//! rsfulmen syncs schemas, documentation, and configuration defaults from
//! [Crucible](https://github.com/fulmenhq/crucible), the Fulmen SSOT repository.
//!
//! Run `make sync-ssot` to update synced assets.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod secret_text;

/// Configuration path utilities following the Fulmen Config Path Standard.
///
/// Provides XDG-compliant configuration, data, and cache directory resolution
/// across Linux, macOS, and Windows platforms.
#[cfg(feature = "config")]
#[cfg_attr(docsrs, doc(cfg(feature = "config")))]
pub mod config;

/// App identity discovery from `.fulmen/app.yaml`.
///
/// Provides upward directory search and explicit-file loading with optional
/// `FULMEN_APP_IDENTITY_FILE` override.
#[cfg(feature = "appidentity")]
#[cfg_attr(docsrs, doc(cfg(feature = "appidentity")))]
pub mod appidentity;

/// Structured logging with SIMPLE and STRUCTURED profiles.
///
/// Provides leveled logging (Trace through Fatal) with configurable output
/// format, component context, structured fields, and an opt-in middleware
/// chain (redaction, correlation, throttle) plus organizational policy load.
#[cfg(feature = "logging")]
#[cfg_attr(docsrs, doc(cfg(feature = "logging")))]
pub mod logging;

/// Canonical hashing with xxh3-128 (default), SHA-256, CRC-32, and CRC-32C.
///
/// Provides content-addressable digests in `algo:hex` format, streaming
/// hash computation, and verification helpers.
#[cfg(feature = "fulhash")]
#[cfg_attr(docsrs, doc(cfg(feature = "fulhash")))]
pub mod fulhash;

/// Safe filesystem discovery with glob patterns.
///
/// Provides file search with glob matching, path traversal protection,
/// repository root discovery, and config file scanning.
#[cfg(feature = "pathfinder")]
#[cfg_attr(docsrs, doc(cfg(feature = "pathfinder")))]
pub mod pathfinder;

/// Canonical archive operations (tar, tar.gz, zip, gzip).
#[cfg(feature = "fulpack")]
#[cfg_attr(docsrs, doc(cfg(feature = "fulpack")))]
pub mod fulpack;

/// Terminal utilities and Unicode-aware string handling.
///
/// Provides box drawing, display width calculation, string analysis,
/// and Unicode-safe truncation/padding.
#[cfg(feature = "ascii")]
#[cfg_attr(docsrs, doc(cfg(feature = "ascii")))]
pub mod ascii;

/// Canonical encoding/decoding, detection, normalization, and BOM helpers.
#[cfg(feature = "fulencode")]
#[cfg_attr(docsrs, doc(cfg(feature = "fulencode")))]
pub mod fulencode;

/// Foundry catalog data structures.
///
/// Provides access to standardized catalogs including:
/// - Country codes (ISO 3166)
/// - HTTP status codes
/// - MIME types
/// - Common regex patterns
/// - File magic numbers
/// - Exit codes
#[cfg(any(
    feature = "foundry",
    feature = "foundry-core",
    feature = "foundry-mime-types",
    feature = "foundry-patterns",
))]
#[cfg_attr(docsrs, doc(cfg(feature = "foundry")))]
pub mod foundry;

/// Runtime signal handling plus signal catalog helpers.
///
/// Re-exports all foundry signal catalog helpers (`lookup_signal`, constants,
/// platform support metadata) and adds runtime handling primitives via
/// [`signals::SignalManager`], including `POST /admin/signal` request helpers
/// and (with the `json` feature) a thin HTTP method/path/body adapter.
#[cfg(any(
    feature = "foundry",
    feature = "foundry-core",
    feature = "foundry-mime-types",
    feature = "foundry-patterns",
))]
#[cfg_attr(docsrs, doc(cfg(feature = "foundry-core")))]
pub mod signals;

/// Embedded Crucible assets and sync metadata.
#[cfg(feature = "crucible")]
#[cfg_attr(docsrs, doc(cfg(feature = "crucible")))]
pub mod crucible;

/// Programmatic access to SSOT module registries.
#[cfg(feature = "crucible")]
#[cfg_attr(docsrs, doc(cfg(feature = "crucible")))]
pub mod module_registry;

/// Access and processing of embedded Crucible documentation assets.
#[cfg(feature = "docscribe")]
#[cfg_attr(docsrs, doc(cfg(feature = "docscribe")))]
pub mod docscribe;

/// schema_id tagging and offline schema URI resolution.
#[cfg(feature = "schema-id")]
#[cfg_attr(docsrs, doc(cfg(feature = "schema-id")))]
pub mod schema_id;

/// Text similarity utilities.
#[cfg(feature = "similarity")]
#[cfg_attr(docsrs, doc(cfg(feature = "similarity")))]
pub mod similarity;

/// Schema validation utilities (embedded Crucible catalogs and file-backed trees).
#[cfg(feature = "schema-validation")]
#[cfg_attr(docsrs, doc(cfg(feature = "schema-validation")))]
pub mod schema_validation;

/// Canonical error envelope + propagation helpers.
#[cfg(feature = "error-handling")]
#[cfg_attr(docsrs, doc(cfg(feature = "error-handling")))]
pub mod error_handling;

/// Telemetry + taxonomy-backed metrics export (JSON, Prometheus text, HTTP helpers).
#[cfg(feature = "telemetry-metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "telemetry-metrics")))]
pub mod telemetry_metrics;

/// Host binary identity (`version` / `version --extended`).
///
/// Resolves the **running product binary** from app-injected `FULMEN_HOST_*`
/// compile stamps (`host_identity!()` is stamp-only). Library git and Crucible
/// SSOT commits are pins, not host commit. See [`host_identity`].
pub mod buildinfo;

/// Library version information.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crucible SSOT version this library was synced from.
///
/// Updated automatically by `make sync-ssot`.
pub const CRUCIBLE_VERSION: &str = "v0.4.19";

/// Crate-wide lock for tests that read or mutate the process environment.
///
/// Prefer [`ProcessEnvGuard`] so changed keys are restored on unwind.
#[cfg(test)]
pub(crate) fn process_env_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

/// Acquire [`process_env_lock`], recovering a poisoned lock so one failed
/// assertion cannot cascade into unrelated tests.
#[cfg(test)]
pub(crate) fn acquire_process_env_lock() -> std::sync::MutexGuard<'static, ()> {
    process_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

/// Holds the process-env lock and restores every key this guard changed.
///
/// `Drop` runs on unwind, so a panicked writer cannot leak values to later
/// tests. Poison recovery still applies via [`acquire_process_env_lock`].
#[cfg(test)]
pub(crate) struct ProcessEnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    previous: Vec<(std::ffi::OsString, Option<std::ffi::OsString>)>,
}

#[cfg(test)]
impl ProcessEnvGuard {
    pub(crate) fn acquire() -> Self {
        Self {
            _lock: acquire_process_env_lock(),
            previous: Vec::new(),
        }
    }

    fn remember(&mut self, key: &std::ffi::OsStr) {
        if !self.previous.iter().any(|(k, _)| k == key) {
            self.previous
                .push((key.to_os_string(), std::env::var_os(key)));
        }
    }

    pub(crate) fn set(
        &mut self,
        key: impl AsRef<std::ffi::OsStr>,
        value: impl AsRef<std::ffi::OsStr>,
    ) {
        let key = key.as_ref();
        self.remember(key);
        std::env::set_var(key, value);
    }

    pub(crate) fn remove(&mut self, key: impl AsRef<std::ffi::OsStr>) {
        let key = key.as_ref();
        self.remember(key);
        std::env::remove_var(key);
    }
}

#[cfg(test)]
impl Drop for ProcessEnvGuard {
    fn drop(&mut self) {
        for (key, previous) in self.previous.drain(..).rev() {
            match previous {
                Some(value) => std::env::set_var(&key, value),
                None => std::env::remove_var(&key),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_set() {
        // VERSION comes from Cargo.toml, should be semver format
        assert!(VERSION.len() >= 5, "VERSION should be at least x.y.z");
        assert!(VERSION.contains('.'), "VERSION should be semver format");
    }

    #[test]
    fn test_crucible_version_is_set() {
        // CRUCIBLE_VERSION should be vX.Y.Z format
        assert!(
            CRUCIBLE_VERSION.len() >= 6,
            "CRUCIBLE_VERSION should be at least vX.Y.Z"
        );
        assert!(
            CRUCIBLE_VERSION.starts_with('v'),
            "CRUCIBLE_VERSION should start with 'v'"
        );
    }

    #[test]
    fn process_env_guard_restores_env_after_panic() {
        const KEY: &str = "RSFULMEN_TEST_PROCESS_ENV_LOCK";
        let before = std::env::var_os(KEY);
        let panicked = std::panic::catch_unwind(|| {
            let mut guard = ProcessEnvGuard::acquire();
            guard.set(KEY, "during-panic");
            panic!("intentional lock poison");
        });
        assert!(panicked.is_err());
        let _reader = ProcessEnvGuard::acquire();
        assert_eq!(std::env::var_os(KEY), before);
    }

    #[test]
    fn process_env_guard_serializes_reader_until_writer_drops() {
        use std::sync::{Arc, Barrier, TryLockError};

        const KEY: &str = "RSFULMEN_TEST_PROCESS_ENV_BARRIER";
        let before = std::env::var_os(KEY);
        let writer_holding = Arc::new(Barrier::new(2));
        let reader_saw_hold = Arc::new(Barrier::new(2));
        let w_hold = writer_holding.clone();
        let w_seen = reader_saw_hold.clone();
        let writer = std::thread::spawn(move || {
            let mut guard = ProcessEnvGuard::acquire();
            guard.set(KEY, "writer-temp");
            w_hold.wait();
            w_seen.wait();
        });
        writer_holding.wait();
        match process_env_lock().try_lock() {
            Err(TryLockError::WouldBlock) => {}
            other => panic!("reader must not acquire while the writer holds, got {other:?}"),
        }
        reader_saw_hold.wait();
        writer.join().expect("writer thread");
        let _reader = ProcessEnvGuard::acquire();
        assert_eq!(
            std::env::var_os(KEY),
            before,
            "reader must observe the restored value after the writer drops"
        );
    }
}
