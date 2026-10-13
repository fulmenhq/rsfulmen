//! Runtime signal handling plus foundry signal catalog re-exports.
//!
//! This module preserves the catalog API from `rsfulmen::foundry::signals` and
//! adds a runtime manager for handler dispatch, shutdown/reload chains,
//! `/admin/signal` endpoint utilities, and deterministic test injection.
//!
//! Handler panics are isolated (`catch_unwind`): the payload is logged after
//! redaction, later handlers still run, and TERM/INT always enter the shutdown
//! chain. Shutdown hooks run under the catalog `timeout_seconds` grace period
//! (30s for SIGTERM, 5s for SIGINT; 30s when the field is absent; `0` takes
//! the force-exit path immediately). Force-exit is `process::exit` unless a
//! test/embedder installs [`SignalManager::set_force_exit`].

use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Test helpers for deterministic signal injection.
pub mod testing;

pub use crate::foundry::signals::*;

#[cfg(target_os = "windows")]
static WINDOWS_CTRL_ROUTER: std::sync::OnceLock<Mutex<Option<mpsc::Sender<i32>>>> =
    std::sync::OnceLock::new();
#[cfg(target_os = "windows")]
static WINDOWS_CTRL_INSTALLED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
#[cfg(target_os = "windows")]
static WINDOWS_FALLBACK_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Boxed signal error type used by signal callbacks.
pub type SignalError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Convenience result type for signal callbacks.
pub type SignalResult = Result<(), SignalError>;

/// Handler for a specific signal.
pub type HandlerFn = dyn Fn() -> SignalResult + Send + Sync + 'static;

/// Cleanup function for shutdown chains.
pub type CleanupFn = Box<dyn FnOnce() -> SignalResult + Send + 'static>;

/// Reload function for SIGHUP chains.
pub type ReloadFn = dyn Fn() -> SignalResult + Send + Sync + 'static;

/// Logger callback used for `/admin/signal` endpoint request events.
pub type EndpointLogFn = dyn Fn(&SignalEndpointLogEvent) + Send + Sync + 'static;

/// Logger callback for runtime events (handler panic, grace expiry).
pub type RuntimeLogFn = dyn Fn(&SignalRuntimeLogEvent) + Send + Sync + 'static;

/// Force-exit callback. Production default is [`std::process::exit`].
pub type ForceExitFn = dyn Fn(i32) + Send + Sync + 'static;

/// Shutdown grace used when the catalog omits `timeout_seconds` (SIGTERM).
pub const DEFAULT_SHUTDOWN_GRACE: Duration = Duration::from_secs(30);

/// Replacement written over secret-shaped panic / timeout log text.
const RUNTIME_REDACTION_REPLACEMENT: &str = "[REDACTED]";

/// Field names redacted in `key=value` / `key:value` runtime log lines.
const RUNTIME_SECRET_KEYS: &[&str] = &[
    "password",
    "token",
    "secret",
    "authorization",
    "apikey",
    "api_key",
    "access_token",
    "refresh_token",
];

/// Errors returned by [`SignalManager`] operations.
#[derive(Debug, thiserror::Error)]
pub enum SignalManagerError {
    /// A signal number was not found in the catalog.
    #[error("unknown signal number: {0}")]
    UnknownSignal(i32),

    /// The signal is known but unsupported on this platform.
    #[error("signal {signal} is unsupported on this platform")]
    UnsupportedSignal {
        /// Unsupported signal number.
        signal: i32,
    },

    /// Listener cannot be started more than once at a time.
    #[error("signal listener is already running")]
    ListenAlreadyRunning,

    /// Windows fallback supports only one active listener per process.
    #[error("windows fallback listener is already active for another manager")]
    WindowsFallbackAlreadyActive,

    /// Signal listener backend initialization failed.
    #[error("failed to initialize signal listener: {0}")]
    ListenerInit(String),

    /// Injection channel is closed.
    #[error("signal injector channel is closed")]
    InjectorClosed,

    /// Timed out waiting for the listener to start.
    #[error("timed out waiting for signal listener start after {0:?}")]
    ListenTimeout(Duration),
}

/// Double-tap Ctrl+C configuration.
#[derive(Debug, Clone)]
pub struct DoubleTapConfig {
    /// Window for second tap.
    pub window: Duration,
    /// Message printed on first tap.
    pub message: String,
    /// Exit code on force quit.
    pub exit_code: i32,
}

impl DoubleTapConfig {
    /// Build config from foundry SIGINT metadata with safe fallbacks.
    pub fn from_catalog() -> Self {
        let fallback = Self::default();
        let Some(sigint) = lookup_signal("SIGINT") else {
            return fallback;
        };

        Self {
            window: Duration::from_secs(sigint.double_tap_window_seconds.unwrap_or(2).into()),
            message: sigint
                .double_tap_message
                .clone()
                .unwrap_or(fallback.message),
            exit_code: sigint.exit_code,
        }
    }
}

impl Default for DoubleTapConfig {
    fn default() -> Self {
        Self {
            window: Duration::from_secs(2),
            message: "Press Ctrl+C again within 2s to force quit".to_string(),
            exit_code: EXIT_SIGINT,
        }
    }
}

/// Request payload for `POST /admin/signal`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalEndpointRequest {
    /// Signal token: one of `HUP`, `TERM`, `INT`, `QUIT`, `USR1`, `USR2`.
    pub signal: String,
    /// Optional operator-provided reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Optional request correlation identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

/// Optional endpoint metadata typically derived from the HTTP layer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SignalEndpointMetadata {
    /// Source client IP, if known.
    pub source_ip: Option<String>,
    /// Authenticated principal, if known.
    pub authenticated_identity: Option<String>,
}

/// Success response body for accepted endpoint requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalEndpointAccepted {
    /// Fixed status marker: `accepted`.
    pub status: String,
    /// Echoed short signal token (for example: `HUP`).
    pub signal: String,
    /// Echoed correlation ID (canonicalized when `foundry-correlation` is enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    /// Human-friendly result message.
    pub message: String,
}

/// Error response body for endpoint request failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalEndpointErrorResponse {
    /// Fixed status marker: `error`.
    pub status: String,
    /// Machine-readable error identifier.
    pub error: String,
    /// Human-friendly error detail.
    pub message: String,
}

/// Response wrapper for `/admin/signal` helper processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalEndpointResponse {
    /// Request accepted and queued for async processing.
    Accepted(SignalEndpointAccepted),
    /// Request rejected due to invalid input or processing failure.
    Error(SignalEndpointErrorResponse),
}

impl SignalEndpointResponse {
    /// HTTP status code aligned with the response payload.
    pub fn status_code(&self) -> u16 {
        match self {
            SignalEndpointResponse::Accepted(_) => 202,
            SignalEndpointResponse::Error(err) => match err.error.as_str() {
                "invalid_signal" | "invalid_correlation_id" => 400,
                _ => 500,
            },
        }
    }
}

/// Structured endpoint log event emitted after request processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalEndpointLogEvent {
    /// Resolved short signal token.
    pub signal: String,
    /// Optional request reason.
    pub reason: Option<String>,
    /// Optional correlation identifier.
    pub correlation_id: Option<String>,
    /// Optional source IP from HTTP metadata.
    pub source_ip: Option<String>,
    /// Optional authenticated identity from HTTP metadata.
    pub authenticated_identity: Option<String>,
    /// Outcome status: `accepted` or `error`.
    pub status: String,
    /// Optional error code when status is `error`.
    pub error: Option<String>,
}

/// Kind of a [`SignalRuntimeLogEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalRuntimeLogKind {
    /// A registered signal handler panicked.
    HandlerPanic,
    /// A shutdown hook panicked.
    ShutdownHookPanic,
    /// A reload hook panicked.
    ReloadHookPanic,
    /// Shutdown hooks did not finish before `timeout_seconds`.
    ShutdownGraceExpired,
}

/// Structured runtime log event (panic isolation, grace expiry).
///
/// `message` is already redacted (`key=value` secret assignments; plus
/// logging-crate pattern redaction when the `logging` feature is enabled).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalRuntimeLogEvent {
    /// Event kind.
    pub kind: SignalRuntimeLogKind,
    /// Platform signal number when known.
    pub signal: Option<i32>,
    /// Redacted human-readable detail.
    pub message: String,
}

#[derive(Clone)]
struct RegisteredHandler {
    id: u64,
    handler: Arc<HandlerFn>,
}

struct Inner {
    handlers: Mutex<HashMap<i32, Vec<RegisteredHandler>>>,
    next_handler_id: AtomicU64,
    shutdown_chain: Mutex<Vec<CleanupFn>>,
    reload_chain: Mutex<Vec<Arc<ReloadFn>>>,
    double_tap: Mutex<Option<DoubleTapConfig>>,
    stop_flag: AtomicBool,
    listening: AtomicBool,
    listen_state: Mutex<bool>,
    listen_cv: Condvar,
    inject_tx: mpsc::Sender<i32>,
    inject_rx: Mutex<mpsc::Receiver<i32>>,
    endpoint_logger: Mutex<Option<Arc<EndpointLogFn>>>,
    runtime_logger: Mutex<Option<Arc<RuntimeLogFn>>>,
    force_exit: Mutex<Option<Arc<ForceExitFn>>>,
    shutdown_grace_override: Mutex<Option<Duration>>,
}

impl Inner {
    fn set_listening_state(&self, state: bool) {
        if let Ok(mut lock) = self.listen_state.lock() {
            *lock = state;
            self.listen_cv.notify_all();
        }
    }
}

/// Registration guard returned from [`SignalManager::handle`].
///
/// Dropping this guard unregisters the associated handler.
pub struct SignalRegistration {
    inner: Option<Weak<Inner>>,
    signal: i32,
    handler_id: u64,
}

impl SignalRegistration {
    fn new(inner: &Arc<Inner>, signal: i32, handler_id: u64) -> Self {
        Self {
            inner: Some(Arc::downgrade(inner)),
            signal,
            handler_id,
        }
    }

    #[cfg(target_os = "windows")]
    fn noop() -> Self {
        Self {
            inner: None,
            signal: 0,
            handler_id: 0,
        }
    }
}

impl Drop for SignalRegistration {
    fn drop(&mut self) {
        let Some(inner) = self.inner.as_ref().and_then(Weak::upgrade) else {
            return;
        };

        let Ok(mut handlers) = inner.handlers.lock() else {
            return;
        };

        if let Some(entries) = handlers.get_mut(&self.signal) {
            entries.retain(|entry| entry.id != self.handler_id);
            if entries.is_empty() {
                handlers.remove(&self.signal);
            }
        }
    }
}

/// Runtime signal manager.
#[derive(Clone)]
pub struct SignalManager {
    inner: Arc<Inner>,
}

impl SignalManager {
    /// Create a new signal manager.
    pub fn new() -> Self {
        let (inject_tx, inject_rx) = mpsc::channel();
        Self {
            inner: Arc::new(Inner {
                handlers: Mutex::new(HashMap::new()),
                next_handler_id: AtomicU64::new(1),
                shutdown_chain: Mutex::new(Vec::new()),
                reload_chain: Mutex::new(Vec::new()),
                double_tap: Mutex::new(None),
                stop_flag: AtomicBool::new(false),
                listening: AtomicBool::new(false),
                listen_state: Mutex::new(false),
                listen_cv: Condvar::new(),
                inject_tx,
                inject_rx: Mutex::new(inject_rx),
                endpoint_logger: Mutex::new(None),
                runtime_logger: Mutex::new(None),
                force_exit: Mutex::new(None),
                shutdown_grace_override: Mutex::new(None),
            }),
        }
    }

    /// Register a handler for a specific signal number.
    ///
    /// Returns a registration guard that unregisters on drop.
    pub fn handle<F>(
        &self,
        signal: i32,
        handler: F,
    ) -> Result<SignalRegistration, SignalManagerError>
    where
        F: Fn() -> SignalResult + Send + Sync + 'static,
    {
        let Some(sig_meta) = signal_by_platform_number(signal) else {
            return Err(SignalManagerError::UnknownSignal(signal));
        };
        let storage_signal = canonical_signal_number(signal).unwrap_or(signal);

        if !self.supports(signal) {
            #[cfg(target_os = "windows")]
            {
                log_windows_fallback(sig_meta);
                return Ok(SignalRegistration::noop());
            }

            #[cfg(not(target_os = "windows"))]
            {
                let _ = sig_meta;
                return Err(SignalManagerError::UnsupportedSignal { signal });
            }
        }

        let id = self.inner.next_handler_id.fetch_add(1, Ordering::Relaxed);
        let mut handlers = self
            .inner
            .handlers
            .lock()
            .expect("handlers mutex poisoned unexpectedly");
        handlers
            .entry(storage_signal)
            .or_default()
            .push(RegisteredHandler {
                id,
                handler: Arc::new(handler),
            });

        Ok(SignalRegistration::new(&self.inner, storage_signal, id))
    }

    /// Append a shutdown hook (executed in LIFO order).
    ///
    /// Hooks run under the catalog `timeout_seconds` grace period (see
    /// [`catalog_shutdown_grace`] / [`SignalManager::set_shutdown_grace`]).
    /// A hook panic is isolated and does not skip later hooks.
    pub fn on_shutdown<F>(&self, hook: F)
    where
        F: FnOnce() -> SignalResult + Send + 'static,
    {
        let mut chain = self
            .inner
            .shutdown_chain
            .lock()
            .expect("shutdown mutex poisoned unexpectedly");
        chain.push(Box::new(hook));
    }

    /// Append a reload hook (executed in FIFO order).
    pub fn on_reload<F>(&self, hook: F)
    where
        F: Fn() -> SignalResult + Send + Sync + 'static,
    {
        let mut chain = self
            .inner
            .reload_chain
            .lock()
            .expect("reload mutex poisoned unexpectedly");
        chain.push(Arc::new(hook));
    }

    /// Enable SIGINT double-tap behavior.
    pub fn enable_double_tap(&self, config: DoubleTapConfig) {
        let mut lock = self
            .inner
            .double_tap
            .lock()
            .expect("double_tap mutex poisoned unexpectedly");
        *lock = Some(config);
    }

    /// Disable SIGINT double-tap behavior.
    pub fn disable_double_tap(&self) {
        let mut lock = self
            .inner
            .double_tap
            .lock()
            .expect("double_tap mutex poisoned unexpectedly");
        *lock = None;
    }

    /// Install a callback invoked for every processed `/admin/signal` request.
    pub fn set_endpoint_logger<F>(&self, logger: F)
    where
        F: Fn(&SignalEndpointLogEvent) + Send + Sync + 'static,
    {
        let mut lock = self
            .inner
            .endpoint_logger
            .lock()
            .expect("endpoint logger mutex poisoned unexpectedly");
        *lock = Some(Arc::new(logger));
    }

    /// Remove the endpoint request logger callback.
    pub fn clear_endpoint_logger(&self) {
        let mut lock = self
            .inner
            .endpoint_logger
            .lock()
            .expect("endpoint logger mutex poisoned unexpectedly");
        *lock = None;
    }

    /// Install a callback for handler-panic and grace-expiry events.
    ///
    /// Messages are redacted before the callback runs. Without a callback the
    /// same line is written to stderr.
    pub fn set_runtime_logger<F>(&self, logger: F)
    where
        F: Fn(&SignalRuntimeLogEvent) + Send + Sync + 'static,
    {
        let mut lock = self
            .inner
            .runtime_logger
            .lock()
            .expect("runtime logger mutex poisoned unexpectedly");
        *lock = Some(Arc::new(logger));
    }

    /// Remove the runtime event logger callback.
    pub fn clear_runtime_logger(&self) {
        let mut lock = self
            .inner
            .runtime_logger
            .lock()
            .expect("runtime logger mutex poisoned unexpectedly");
        *lock = None;
    }

    /// Override catalog `timeout_seconds` for TERM/INT shutdown.
    ///
    /// `None` restores catalog-or-default lookup. `Some(Duration::ZERO)` takes
    /// the force-exit path immediately (hooks are not run).
    pub fn set_shutdown_grace(&self, grace: Option<Duration>) {
        let mut lock = self
            .inner
            .shutdown_grace_override
            .lock()
            .expect("shutdown grace mutex poisoned unexpectedly");
        *lock = grace;
    }

    /// Grace period that would apply if `signal` triggered shutdown.
    pub fn shutdown_grace_for(&self, signal: i32) -> Duration {
        let override_grace = self
            .inner
            .shutdown_grace_override
            .lock()
            .expect("shutdown grace mutex poisoned unexpectedly");
        override_grace.unwrap_or_else(|| catalog_shutdown_grace(signal))
    }

    /// Replace `process::exit` on grace expiry (and tests that must not abort).
    pub fn set_force_exit<F>(&self, hook: F)
    where
        F: Fn(i32) + Send + Sync + 'static,
    {
        let mut lock = self
            .inner
            .force_exit
            .lock()
            .expect("force_exit mutex poisoned unexpectedly");
        *lock = Some(Arc::new(hook));
    }

    /// Restore the production force-exit path (`process::exit`).
    pub fn clear_force_exit(&self) {
        let mut lock = self
            .inner
            .force_exit
            .lock()
            .expect("force_exit mutex poisoned unexpectedly");
        *lock = None;
    }

    /// Process a `POST /admin/signal` request body.
    pub fn handle_admin_signal_request(
        &self,
        request: SignalEndpointRequest,
    ) -> SignalEndpointResponse {
        self.handle_admin_signal_request_with_metadata(request, SignalEndpointMetadata::default())
    }

    /// Process a `POST /admin/signal` request body with HTTP-derived metadata.
    pub fn handle_admin_signal_request_with_metadata(
        &self,
        request: SignalEndpointRequest,
        metadata: SignalEndpointMetadata,
    ) -> SignalEndpointResponse {
        let signal_name = match parse_admin_signal_name(&request.signal) {
            Some(name) => name,
            None => {
                let error = SignalEndpointErrorResponse {
                    status: "error".to_string(),
                    error: "invalid_signal".to_string(),
                    message: format!(
                        "Signal '{}' is not recognized. Valid signals: {}",
                        request.signal.trim(),
                        ADMIN_ENDPOINT_SIGNAL_TOKENS.join(", ")
                    ),
                };
                self.emit_endpoint_log_event(SignalEndpointLogEvent {
                    signal: request.signal.trim().to_ascii_uppercase(),
                    reason: request.reason.clone(),
                    correlation_id: request.correlation_id.clone(),
                    source_ip: metadata.source_ip,
                    authenticated_identity: metadata.authenticated_identity,
                    status: "error".to_string(),
                    error: Some("invalid_signal".to_string()),
                });
                return SignalEndpointResponse::Error(error);
            }
        };

        let signal_number = match get_signal_number(signal_name) {
            Some(number) => number,
            None => {
                let error = SignalEndpointErrorResponse {
                    status: "error".to_string(),
                    error: "invalid_signal".to_string(),
                    message: format!(
                        "Signal '{}' is not available on this platform",
                        endpoint_signal_token(signal_name)
                    ),
                };
                self.emit_endpoint_log_event(SignalEndpointLogEvent {
                    signal: endpoint_signal_token(signal_name).to_string(),
                    reason: request.reason.clone(),
                    correlation_id: request.correlation_id.clone(),
                    source_ip: metadata.source_ip,
                    authenticated_identity: metadata.authenticated_identity,
                    status: "error".to_string(),
                    error: Some("invalid_signal".to_string()),
                });
                return SignalEndpointResponse::Error(error);
            }
        };

        let correlation_id =
            match normalize_endpoint_correlation_id(request.correlation_id.as_deref()) {
                Ok(value) => value,
                Err(message) => {
                    let error = SignalEndpointErrorResponse {
                        status: "error".to_string(),
                        error: "invalid_correlation_id".to_string(),
                        message,
                    };
                    self.emit_endpoint_log_event(SignalEndpointLogEvent {
                        signal: endpoint_signal_token(signal_name).to_string(),
                        reason: request.reason.clone(),
                        correlation_id: request.correlation_id.clone(),
                        source_ip: metadata.source_ip,
                        authenticated_identity: metadata.authenticated_identity,
                        status: "error".to_string(),
                        error: Some("invalid_correlation_id".to_string()),
                    });
                    return SignalEndpointResponse::Error(error);
                }
            };

        if self.inject(signal_number).is_err() {
            let error = SignalEndpointErrorResponse {
                status: "error".to_string(),
                error: "inject_failed".to_string(),
                message: "Signal queue is unavailable".to_string(),
            };
            self.emit_endpoint_log_event(SignalEndpointLogEvent {
                signal: endpoint_signal_token(signal_name).to_string(),
                reason: request.reason.clone(),
                correlation_id: correlation_id.clone(),
                source_ip: metadata.source_ip,
                authenticated_identity: metadata.authenticated_identity,
                status: "error".to_string(),
                error: Some("inject_failed".to_string()),
            });
            return SignalEndpointResponse::Error(error);
        }

        let accepted = SignalEndpointAccepted {
            status: "accepted".to_string(),
            signal: endpoint_signal_token(signal_name).to_string(),
            correlation_id: correlation_id.clone(),
            message: "Signal will be processed asynchronously".to_string(),
        };

        self.emit_endpoint_log_event(SignalEndpointLogEvent {
            signal: accepted.signal.clone(),
            reason: request.reason,
            correlation_id,
            source_ip: metadata.source_ip,
            authenticated_identity: metadata.authenticated_identity,
            status: "accepted".to_string(),
            error: None,
        });

        SignalEndpointResponse::Accepted(accepted)
    }

    /// Thin HTTP adapter for `POST /admin/signal`.
    ///
    /// No server crate: the caller supplies method, path, and body. Returns
    /// `(status_code, json_body)`. Other methods/paths are 405/404.
    ///
    /// Authentication, TLS, and rate-limit (401/403/429) stay with the caller.
    /// Requires the `json` feature (`serde_json`); `foundry-core` alone does
    /// not enable this method.
    #[cfg(feature = "json")]
    pub fn handle_admin_signal_http(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
        metadata: SignalEndpointMetadata,
    ) -> (u16, String) {
        let path = path.split('?').next().unwrap_or(path);
        if path != "/admin/signal" {
            return (
                404,
                r#"{"status":"error","error":"not_found","message":"not found"}"#.to_string(),
            );
        }
        if !method.eq_ignore_ascii_case("POST") {
            return (
                405,
                r#"{"status":"error","error":"method_not_allowed","message":"POST required"}"#
                    .to_string(),
            );
        }
        let request = match serde_json::from_slice::<SignalEndpointRequest>(body) {
            Ok(req) => req,
            Err(err) => {
                let resp = SignalEndpointErrorResponse {
                    status: "error".to_string(),
                    error: "invalid_json".to_string(),
                    message: err.to_string(),
                };
                return (
                    400,
                    serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string()),
                );
            }
        };
        let response = self.handle_admin_signal_request_with_metadata(request, metadata);
        let status = response.status_code();
        let body = match response {
            SignalEndpointResponse::Accepted(ok) => serde_json::to_string(&ok),
            SignalEndpointResponse::Error(err) => serde_json::to_string(&err),
        }
        .unwrap_or_else(|_| "{}".to_string());
        (status, body)
    }

    /// Check whether a signal is supported on the current platform.
    pub fn supports(&self, signal: i32) -> bool {
        signal_by_platform_number(signal)
            .map(|sig| is_signal_supported(&sig.name))
            .unwrap_or(false)
    }

    /// Start listening for OS and injected signals.
    ///
    /// This call blocks until [`SignalManager::stop`] is called or a shutdown
    /// signal is processed.
    pub fn listen(&self) -> Result<(), SignalManagerError> {
        if self.inner.listening.swap(true, Ordering::SeqCst) {
            return Err(SignalManagerError::ListenAlreadyRunning);
        }

        self.inner.stop_flag.store(false, Ordering::SeqCst);
        self.inner.set_listening_state(true);
        let _guard = ListeningGuard {
            inner: Arc::clone(&self.inner),
        };

        #[cfg(unix)]
        {
            self.listen_unix()
        }

        #[cfg(not(unix))]
        {
            self.listen_fallback()
        }
    }

    /// Stop the running listener loop.
    pub fn stop(&self) {
        self.inner.stop_flag.store(true, Ordering::SeqCst);
    }

    #[cfg(target_os = "windows")]
    fn listen_fallback(&self) -> Result<(), SignalManagerError> {
        if WINDOWS_FALLBACK_ACTIVE.swap(true, Ordering::SeqCst) {
            return Err(SignalManagerError::WindowsFallbackAlreadyActive);
        }
        let _guard = WindowsFallbackGuard;

        install_windows_ctrl_handler(self.inner.inject_tx.clone())?;
        let mut first_sigint_at = None;
        while !self.inner.stop_flag.load(Ordering::SeqCst) {
            self.drain_injected(&mut first_sigint_at)?;
            thread::sleep(Duration::from_millis(25));
        }
        Ok(())
    }

    #[cfg(all(not(unix), not(target_os = "windows")))]
    fn listen_fallback(&self) -> Result<(), SignalManagerError> {
        let mut first_sigint_at = None;
        while !self.inner.stop_flag.load(Ordering::SeqCst) {
            self.drain_injected(&mut first_sigint_at)?;
            thread::sleep(Duration::from_millis(25));
        }
        Ok(())
    }

    #[cfg(unix)]
    fn listen_unix(&self) -> Result<(), SignalManagerError> {
        use signal_hook::iterator::Signals;

        let registered_signals = {
            let handlers = self
                .inner
                .handlers
                .lock()
                .expect("handlers mutex poisoned unexpectedly");
            handlers.keys().copied().collect::<Vec<_>>()
        };
        let signal_numbers = collect_listenable_signal_numbers(registered_signals);
        let mut signals = Signals::new(signal_numbers)
            .map_err(|e| SignalManagerError::ListenerInit(e.to_string()))?;

        let mut first_sigint_at = None;
        while !self.inner.stop_flag.load(Ordering::SeqCst) {
            for signal in signals.pending() {
                self.dispatch_signal(signal, &mut first_sigint_at);
            }

            self.drain_injected(&mut first_sigint_at)?;
            thread::sleep(Duration::from_millis(25));
        }

        Ok(())
    }

    fn drain_injected(
        &self,
        first_sigint_at: &mut Option<Instant>,
    ) -> Result<(), SignalManagerError> {
        let mut pending = Vec::new();
        {
            let rx = self
                .inner
                .inject_rx
                .lock()
                .expect("inject receiver mutex poisoned unexpectedly");
            loop {
                match rx.try_recv() {
                    Ok(signal) => pending.push(signal),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        return Err(SignalManagerError::InjectorClosed)
                    }
                }
            }
        }

        for signal in pending {
            self.dispatch_signal(signal, first_sigint_at);
        }

        Ok(())
    }

    fn dispatch_signal(&self, signal: i32, first_sigint_at: &mut Option<Instant>) {
        self.run_signal_handlers(signal);

        if signal == sigint_number() {
            self.maybe_handle_double_tap(first_sigint_at);
        }

        if signal == sighup_number() {
            self.run_reload_chain();
            return;
        }

        if signal == sigint_number() || signal == sigterm_number() {
            self.run_shutdown_chain(signal);
            self.stop();
        }
    }

    fn maybe_handle_double_tap(&self, first_sigint_at: &mut Option<Instant>) {
        let config = self
            .inner
            .double_tap
            .lock()
            .expect("double_tap mutex poisoned unexpectedly")
            .clone();

        let Some(config) = config else {
            return;
        };

        let now = Instant::now();
        if let Some(first) = *first_sigint_at {
            if now.duration_since(first) <= config.window {
                std::process::exit(config.exit_code);
            }
        }

        *first_sigint_at = Some(now);
        if !config.message.is_empty() {
            eprintln!("{}", config.message);
        }
    }

    fn run_signal_handlers(&self, signal: i32) {
        let signal = canonical_signal_number(signal).unwrap_or(signal);
        let handlers = {
            let map = self
                .inner
                .handlers
                .lock()
                .expect("handlers mutex poisoned unexpectedly");
            map.get(&signal).cloned().unwrap_or_default()
        };

        for entry in handlers {
            match catch_unwind(AssertUnwindSafe(|| (entry.handler)())) {
                Ok(_) => {}
                Err(payload) => {
                    let detail = panic_payload_to_string(payload);
                    self.emit_runtime_log(SignalRuntimeLogEvent {
                        kind: SignalRuntimeLogKind::HandlerPanic,
                        signal: Some(signal),
                        message: format!("handler panicked: {detail}"),
                    });
                }
            }
        }
    }

    fn run_shutdown_chain(&self, signal: i32) {
        let hooks = {
            let mut chain = self
                .inner
                .shutdown_chain
                .lock()
                .expect("shutdown mutex poisoned unexpectedly");
            std::mem::take(&mut *chain)
        };

        let grace = self.shutdown_grace_for(signal);
        let exit_code = signal_by_platform_number(signal)
            .map(|sig| sig.exit_code)
            .unwrap_or(EXIT_SIGTERM);

        if grace == Duration::ZERO {
            self.emit_runtime_log(SignalRuntimeLogEvent {
                kind: SignalRuntimeLogKind::ShutdownGraceExpired,
                signal: Some(signal),
                message: "shutdown grace is zero; taking force-exit path".to_string(),
            });
            self.invoke_force_exit(exit_code);
            return;
        }

        let (done_tx, done_rx) = mpsc::channel();
        let manager = self.clone();
        thread::spawn(move || {
            for hook in hooks.into_iter().rev() {
                match catch_unwind(AssertUnwindSafe(hook)) {
                    Ok(_) => {}
                    Err(payload) => {
                        let detail = panic_payload_to_string(payload);
                        manager.emit_runtime_log(SignalRuntimeLogEvent {
                            kind: SignalRuntimeLogKind::ShutdownHookPanic,
                            signal: Some(signal),
                            message: format!("shutdown hook panicked: {detail}"),
                        });
                    }
                }
            }
            let _ = done_tx.send(());
        });

        match done_rx.recv_timeout(grace) {
            Ok(()) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => {
                self.emit_runtime_log(SignalRuntimeLogEvent {
                    kind: SignalRuntimeLogKind::ShutdownGraceExpired,
                    signal: Some(signal),
                    message: format!("shutdown hooks exceeded grace {grace:?}"),
                });
                self.invoke_force_exit(exit_code);
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {}
        }
    }

    fn run_reload_chain(&self) {
        let chain = self
            .inner
            .reload_chain
            .lock()
            .expect("reload mutex poisoned unexpectedly")
            .clone();

        for hook in chain {
            match catch_unwind(AssertUnwindSafe(|| hook())) {
                Ok(_) => {}
                Err(payload) => {
                    let detail = panic_payload_to_string(payload);
                    self.emit_runtime_log(SignalRuntimeLogEvent {
                        kind: SignalRuntimeLogKind::ReloadHookPanic,
                        signal: Some(sighup_number()),
                        message: format!("reload hook panicked: {detail}"),
                    });
                }
            }
        }
    }

    fn invoke_force_exit(&self, code: i32) {
        let hook = self
            .inner
            .force_exit
            .lock()
            .expect("force_exit mutex poisoned unexpectedly")
            .clone();
        if let Some(hook) = hook {
            hook(code);
            return;
        }
        std::process::exit(code);
    }

    pub(crate) fn inject(&self, signal: i32) -> Result<(), SignalManagerError> {
        self.inner
            .inject_tx
            .send(signal)
            .map_err(|_| SignalManagerError::InjectorClosed)
    }

    pub(crate) fn wait_for_listen(&self, timeout: Duration) -> Result<(), SignalManagerError> {
        let lock = self
            .inner
            .listen_state
            .lock()
            .expect("listen state mutex poisoned unexpectedly");

        let (guard, wait_result) = self
            .inner
            .listen_cv
            .wait_timeout_while(lock, timeout, |is_running| !*is_running)
            .expect("listen condvar poisoned unexpectedly");

        if *guard {
            Ok(())
        } else {
            let _ = wait_result;
            Err(SignalManagerError::ListenTimeout(timeout))
        }
    }

    fn emit_endpoint_log_event(&self, event: SignalEndpointLogEvent) {
        let logger = self
            .inner
            .endpoint_logger
            .lock()
            .expect("endpoint logger mutex poisoned unexpectedly")
            .clone();

        if let Some(logger) = logger {
            logger(&event);
        }
    }

    fn emit_runtime_log(&self, mut event: SignalRuntimeLogEvent) {
        event.message = redact_runtime_message(&event.message);
        let logger = self
            .inner
            .runtime_logger
            .lock()
            .expect("runtime logger mutex poisoned unexpectedly")
            .clone();

        if let Some(logger) = logger {
            logger(&event);
            return;
        }
        eprintln!(
            "rsfulmen signals: {} {}",
            runtime_log_kind_label(event.kind),
            event.message
        );
    }
}

/// Catalog `timeout_seconds` for `signal`, or [`DEFAULT_SHUTDOWN_GRACE`] when
/// the field is absent. `0` means no grace (force-exit immediately).
pub fn catalog_shutdown_grace(signal: i32) -> Duration {
    match signal_by_platform_number(signal).and_then(|sig| sig.timeout_seconds) {
        Some(0) => Duration::ZERO,
        Some(secs) => Duration::from_secs(u64::from(secs)),
        None => DEFAULT_SHUTDOWN_GRACE,
    }
}

fn runtime_log_kind_label(kind: SignalRuntimeLogKind) -> &'static str {
    match kind {
        SignalRuntimeLogKind::HandlerPanic => "handler_panic",
        SignalRuntimeLogKind::ShutdownHookPanic => "shutdown_hook_panic",
        SignalRuntimeLogKind::ReloadHookPanic => "reload_hook_panic",
        SignalRuntimeLogKind::ShutdownGraceExpired => "shutdown_grace_expired",
    }
}

fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

fn redact_runtime_message(raw: &str) -> String {
    #[cfg(feature = "logging")]
    let out = redact_with_logging_middleware(raw);
    #[cfg(not(feature = "logging"))]
    let out = raw.to_string();
    redact_secret_assignments(&out)
}

#[cfg(feature = "logging")]
fn redact_with_logging_middleware(raw: &str) -> String {
    use crate::logging::{LogRecord, Middleware, RedactSecretsMiddleware, Severity};

    let Ok(mw) = RedactSecretsMiddleware::defaults() else {
        return raw.to_string();
    };
    let mut record = LogRecord {
        timestamp: String::new(),
        severity: Severity::Error,
        message: raw.to_string(),
        logger: "signals".to_string(),
        component: None,
        fields: Vec::new(),
        correlation_id: None,
        redaction_flags: Vec::new(),
        throttle_bucket: None,
    };
    let _ = mw.process(&mut record);
    record.message
}

fn redact_secret_assignments(raw: &str) -> String {
    let chars: Vec<char> = raw.chars().collect();
    let lower: Vec<char> = raw.to_ascii_lowercase().chars().collect();
    let mut keys: Vec<Vec<char>> = RUNTIME_SECRET_KEYS
        .iter()
        .map(|k| k.chars().collect())
        .collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));

    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let mut val_start = None;
        for key in &keys {
            let end = i + key.len();
            if end < lower.len()
                && lower[i..end] == key[..]
                && (i == 0 || !lower[i - 1].is_ascii_alphanumeric())
                && (lower[end] == '=' || lower[end] == ':')
            {
                val_start = Some(end + 1);
                break;
            }
        }
        if let Some(start) = val_start {
            for c in &chars[i..start] {
                out.push(*c);
            }
            out.push_str(RUNTIME_REDACTION_REPLACEMENT);
            let mut j = start;
            while j < chars.len() && !chars[j].is_whitespace() {
                j += 1;
            }
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

impl Default for SignalManager {
    fn default() -> Self {
        Self::new()
    }
}

struct ListeningGuard {
    inner: Arc<Inner>,
}

impl Drop for ListeningGuard {
    fn drop(&mut self) {
        self.inner.listening.store(false, Ordering::SeqCst);
        self.inner.set_listening_state(false);
    }
}

#[cfg(unix)]
fn collect_listenable_signal_numbers<I>(registered_signals: I) -> Vec<i32>
where
    I: IntoIterator<Item = i32>,
{
    use std::collections::BTreeSet;

    let mut set = BTreeSet::new();

    // Core control signals are always listened for.
    for signal_name in ["SIGINT", "SIGTERM", "SIGHUP"] {
        if let Some(sig) = lookup_signal(signal_name) {
            if sig.name != "SIGKILL" && is_signal_supported(&sig.name) {
                set.insert(get_signal_number(&sig.name).unwrap_or(sig.unix_number));
            }
        }
    }

    // Also listen for explicitly handled signals.
    for signal in registered_signals {
        let Some(sig) = signal_by_platform_number(signal) else {
            continue;
        };

        if sig.name == "SIGKILL" || !is_signal_supported(&sig.name) {
            continue;
        }

        set.insert(get_signal_number(&sig.name).unwrap_or(sig.unix_number));
    }

    set.into_iter().collect()
}

fn sigint_number() -> i32 {
    get_signal_number("SIGINT").unwrap_or(SIGINT)
}

fn signal_by_platform_number(signal: i32) -> Option<&'static Signal> {
    if let Some(sig) = lookup_signal_by_number(signal) {
        return Some(sig);
    }

    list_signals()
        .iter()
        .find(|sig| get_signal_number(&sig.name) == Some(signal))
}

fn canonical_signal_number(signal: i32) -> Option<i32> {
    let sig = signal_by_platform_number(signal)?;
    Some(get_signal_number(&sig.name).unwrap_or(sig.unix_number))
}

fn sigterm_number() -> i32 {
    get_signal_number("SIGTERM").unwrap_or(SIGTERM)
}

fn sighup_number() -> i32 {
    get_signal_number("SIGHUP").unwrap_or(SIGHUP)
}

const ADMIN_ENDPOINT_SIGNAL_TOKENS: [&str; 6] = ["HUP", "TERM", "INT", "QUIT", "USR1", "USR2"];

fn parse_admin_signal_name(raw: &str) -> Option<&'static str> {
    let mut normalized = raw.trim().to_ascii_uppercase();
    if normalized.starts_with("SIG") {
        normalized = normalized[3..].to_string();
    }

    match normalized.as_str() {
        "HUP" => Some("SIGHUP"),
        "TERM" => Some("SIGTERM"),
        "INT" => Some("SIGINT"),
        "QUIT" => Some("SIGQUIT"),
        "USR1" => Some("SIGUSR1"),
        "USR2" => Some("SIGUSR2"),
        _ => None,
    }
}

fn endpoint_signal_token(signal_name: &str) -> &'static str {
    match signal_name {
        "SIGHUP" => "HUP",
        "SIGTERM" => "TERM",
        "SIGINT" => "INT",
        "SIGQUIT" => "QUIT",
        "SIGUSR1" => "USR1",
        "SIGUSR2" => "USR2",
        _ => "UNKNOWN",
    }
}

#[cfg(feature = "foundry-correlation")]
fn normalize_endpoint_correlation_id(raw: Option<&str>) -> Result<Option<String>, String> {
    let Some(raw) = raw else {
        return Ok(None);
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    crate::foundry::correlation::parse(trimmed)
        .map(|id| Some(id.to_string()))
        .map_err(|err| format!("Correlation ID is invalid UUIDv7: {err}"))
}

#[cfg(not(feature = "foundry-correlation"))]
fn normalize_endpoint_correlation_id(raw: Option<&str>) -> Result<Option<String>, String> {
    let Some(raw) = raw else {
        return Ok(None);
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(trimmed.to_string()))
}

#[cfg(target_os = "windows")]
fn log_windows_fallback(signal: &Signal) {
    if let Some(fallback) = &signal.windows_fallback {
        eprintln!("{}", fallback.log_message);
        return;
    }
    eprintln!("signal {} unsupported on Windows", signal.name);
}

#[cfg(target_os = "windows")]
fn install_windows_ctrl_handler(sender: mpsc::Sender<i32>) -> Result<(), SignalManagerError> {
    let router = WINDOWS_CTRL_ROUTER.get_or_init(|| Mutex::new(None));
    {
        let mut route = router
            .lock()
            .expect("windows ctrl routing mutex poisoned unexpectedly");
        *route = Some(sender);
    }

    if WINDOWS_CTRL_INSTALLED.get().is_none() {
        ctrlc::set_handler(|| {
            if let Some(router) = WINDOWS_CTRL_ROUTER.get() {
                if let Ok(guard) = router.lock() {
                    if let Some(tx) = &*guard {
                        let _ = tx.send(sigint_number());
                    }
                }
            }
        })
        .map_err(|e| SignalManagerError::ListenerInit(e.to_string()))?;

        let _ = WINDOWS_CTRL_INSTALLED.set(());
    }

    Ok(())
}

#[cfg(target_os = "windows")]
struct WindowsFallbackGuard;

#[cfg(target_os = "windows")]
impl Drop for WindowsFallbackGuard {
    fn drop(&mut self) {
        WINDOWS_FALLBACK_ACTIVE.store(false, Ordering::SeqCst);
        if let Some(router) = WINDOWS_CTRL_ROUTER.get() {
            if let Ok(mut route) = router.lock() {
                *route = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Win32 `ctrlc` fallback is process-global. Parallel `listen()` tests
    /// otherwise trip `WINDOWS_FALLBACK_ACTIVE` and time out in `wait_for_listen`.
    #[cfg(windows)]
    static WINDOWS_LISTEN_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn lock_windows_listen() -> Option<std::sync::MutexGuard<'static, ()>> {
        #[cfg(windows)]
        {
            Some(
                WINDOWS_LISTEN_TEST_LOCK
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()),
            )
        }
        #[cfg(not(windows))]
        {
            None
        }
    }

    #[test]
    fn shutdown_chain_lifo_order() {
        let manager = SignalManager::new();
        let order = Arc::new(Mutex::new(Vec::new()));

        let o1 = Arc::clone(&order);
        manager.on_shutdown(move || {
            o1.lock().expect("order mutex poisoned").push(1usize);
            Ok(())
        });

        let o2 = Arc::clone(&order);
        manager.on_shutdown(move || {
            o2.lock().expect("order mutex poisoned").push(2usize);
            Ok(())
        });

        manager.dispatch_signal(sigterm_number(), &mut None);

        assert_eq!(*order.lock().expect("order mutex poisoned"), vec![2, 1]);
    }

    #[test]
    fn reload_chain_fifo_order() {
        let manager = SignalManager::new();
        let order = Arc::new(Mutex::new(Vec::new()));

        let o1 = Arc::clone(&order);
        manager.on_reload(move || {
            o1.lock().expect("order mutex poisoned").push(1usize);
            Ok(())
        });

        let o2 = Arc::clone(&order);
        manager.on_reload(move || {
            o2.lock().expect("order mutex poisoned").push(2usize);
            Ok(())
        });

        manager.dispatch_signal(sighup_number(), &mut None);

        assert_eq!(*order.lock().expect("order mutex poisoned"), vec![1, 2]);
    }

    #[test]
    fn double_tap_config_from_catalog_has_defaults() {
        let config = DoubleTapConfig::from_catalog();
        assert!(config.window.as_secs() >= 1);
        assert!(!config.message.is_empty());
        assert!(config.exit_code > 0);
    }

    #[test]
    fn supports_sigterm_and_sigint() {
        let manager = SignalManager::new();
        assert!(manager.supports(sigterm_number()));
        assert!(manager.supports(sigint_number()));
    }

    #[cfg(unix)]
    #[test]
    fn supports_sighup_unix() {
        let manager = SignalManager::new();
        assert!(manager.supports(sighup_number()));
    }

    #[cfg(unix)]
    #[test]
    fn listen_registration_is_minimal_plus_explicit_handlers() {
        let usr1 = get_signal_number("SIGUSR1").unwrap_or(SIGUSR1);
        let signals = collect_listenable_signal_numbers(vec![usr1]);

        assert!(signals.contains(&sigint_number()));
        assert!(signals.contains(&sigterm_number()));
        assert!(signals.contains(&sighup_number()));
        assert!(signals.contains(&usr1));

        let sigpipe = get_signal_number("SIGPIPE").unwrap_or(SIGPIPE);
        assert!(
            !signals.contains(&sigpipe),
            "non-core signals should not be intercepted unless explicitly handled"
        );
    }

    #[test]
    fn handler_registration_and_drop_unregisters() {
        let manager = SignalManager::new();
        let called = Arc::new(AtomicUsize::new(0));

        let c = Arc::clone(&called);
        let guard = manager
            .handle(sigterm_number(), move || {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .expect("handler registration should succeed");

        manager.dispatch_signal(sigterm_number(), &mut None);
        assert_eq!(called.load(Ordering::SeqCst), 1);

        drop(guard);

        manager.dispatch_signal(sigterm_number(), &mut None);
        assert_eq!(called.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn injector_wait_for_listen_times_out() {
        let manager = SignalManager::new();
        let injector = testing::SignalInjector::new(&manager);

        let result = injector.wait_for_listen(Duration::from_millis(50));
        assert!(matches!(result, Err(SignalManagerError::ListenTimeout(_))));
    }

    #[cfg(any(target_os = "macos", target_os = "freebsd"))]
    #[test]
    fn overridden_signal_registration_dispatches_via_platform_number() {
        let manager = SignalManager::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = Arc::clone(&called);

        // SIGUSR1 constant is Linux-default (10), while macOS/FreeBSD deliver 30.
        let _guard = manager
            .handle(SIGUSR1, move || {
                called_clone.store(true, Ordering::SeqCst);
                Ok(())
            })
            .expect("registration should succeed");

        let platform_usr1 = get_signal_number("SIGUSR1").expect("platform SIGUSR1 number");
        manager.dispatch_signal(platform_usr1, &mut None);
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn admin_signal_endpoint_accepts_valid_signal() {
        let manager = SignalManager::new();

        let response = manager.handle_admin_signal_request(SignalEndpointRequest {
            signal: "hup".to_string(),
            reason: Some("reload requested".to_string()),
            correlation_id: None,
        });

        match response {
            SignalEndpointResponse::Accepted(body) => {
                assert_eq!(body.status, "accepted");
                assert_eq!(body.signal, "HUP");
                assert!(body.correlation_id.is_none());
                assert_eq!(body.message, "Signal will be processed asynchronously");
            }
            SignalEndpointResponse::Error(_) => panic!("expected accepted response"),
        }
    }

    #[test]
    fn admin_signal_endpoint_rejects_invalid_signal() {
        let manager = SignalManager::new();

        let response = manager.handle_admin_signal_request(SignalEndpointRequest {
            signal: "foo".to_string(),
            reason: None,
            correlation_id: None,
        });

        match response {
            SignalEndpointResponse::Accepted(_) => panic!("expected invalid signal error"),
            SignalEndpointResponse::Error(body) => {
                assert_eq!(body.status, "error");
                assert_eq!(body.error, "invalid_signal");
            }
        }
    }

    #[test]
    fn admin_signal_endpoint_logs_request_event() {
        let manager = SignalManager::new();
        let events = Arc::new(Mutex::new(Vec::<SignalEndpointLogEvent>::new()));
        let captured = Arc::clone(&events);

        manager.set_endpoint_logger(move |event| {
            captured
                .lock()
                .expect("events mutex poisoned unexpectedly")
                .push(event.clone());
        });

        let response = manager.handle_admin_signal_request_with_metadata(
            SignalEndpointRequest {
                signal: "TERM".to_string(),
                reason: Some("maintenance".to_string()),
                correlation_id: Some("019503E7-2C4A-7000-8000-A1B2C3D4E5F6".to_string()),
            },
            SignalEndpointMetadata {
                source_ip: Some("127.0.0.1".to_string()),
                authenticated_identity: Some("ops-admin".to_string()),
            },
        );

        assert_eq!(response.status_code(), 202);

        let events = events.lock().expect("events mutex poisoned unexpectedly");
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.signal, "TERM");
        assert_eq!(event.reason.as_deref(), Some("maintenance"));
        let expected_correlation_id = if cfg!(feature = "foundry-correlation") {
            "019503e7-2c4a-7000-8000-a1b2c3d4e5f6"
        } else {
            "019503E7-2C4A-7000-8000-A1B2C3D4E5F6"
        };
        assert_eq!(
            event.correlation_id.as_deref(),
            Some(expected_correlation_id)
        );
        assert_eq!(event.source_ip.as_deref(), Some("127.0.0.1"));
        assert_eq!(event.authenticated_identity.as_deref(), Some("ops-admin"));
        assert_eq!(event.status, "accepted");
        assert!(event.error.is_none());
    }

    #[cfg(feature = "foundry-correlation")]
    #[test]
    fn admin_signal_endpoint_normalizes_correlation_id_when_enabled() {
        let manager = SignalManager::new();

        let response = manager.handle_admin_signal_request(SignalEndpointRequest {
            signal: "USR1".to_string(),
            reason: None,
            correlation_id: Some("019503E7-2C4A-7000-8000-A1B2C3D4E5F6".to_string()),
        });

        match response {
            SignalEndpointResponse::Accepted(body) => {
                assert_eq!(
                    body.correlation_id.as_deref(),
                    Some("019503e7-2c4a-7000-8000-a1b2c3d4e5f6")
                );
            }
            SignalEndpointResponse::Error(_) => panic!("expected accepted response"),
        }
    }

    #[cfg(feature = "foundry-correlation")]
    #[test]
    fn admin_signal_endpoint_rejects_invalid_correlation_id_when_enabled() {
        let manager = SignalManager::new();

        let response = manager.handle_admin_signal_request(SignalEndpointRequest {
            signal: "INT".to_string(),
            reason: None,
            correlation_id: Some("not-a-uuid".to_string()),
        });

        match response {
            SignalEndpointResponse::Accepted(_) => {
                panic!("expected invalid correlation ID error")
            }
            SignalEndpointResponse::Error(body) => {
                assert_eq!(body.status, "error");
                assert_eq!(body.error, "invalid_correlation_id");
            }
        }
    }

    #[test]
    fn injector_basic_shutdown_dispatch() {
        let _listen_lock = lock_windows_listen();
        let manager = SignalManager::new();
        let injector = testing::SignalInjector::new(&manager);

        let hit = Arc::new(AtomicBool::new(false));
        let hit_clone = Arc::clone(&hit);
        manager.on_shutdown(move || {
            hit_clone.store(true, Ordering::SeqCst);
            Ok(())
        });

        let manager_clone = manager.clone();
        let listener = thread::spawn(move || manager_clone.listen());

        injector
            .wait_for_listen(Duration::from_secs(1))
            .expect("listener should start");

        injector
            .inject(sigterm_number())
            .expect("signal injection should succeed");

        listener
            .join()
            .expect("listener thread should join")
            .expect("listener should exit cleanly");

        assert!(hit.load(Ordering::SeqCst));
    }

    #[test]
    fn multiple_handlers_run_in_registration_order() {
        let manager = SignalManager::new();
        let order = Arc::new(Mutex::new(Vec::new()));

        let o1 = Arc::clone(&order);
        let _g1 = manager
            .handle(sigterm_number(), move || {
                o1.lock().expect("order").push(1usize);
                Ok(())
            })
            .expect("register first");

        let o2 = Arc::clone(&order);
        let _g2 = manager
            .handle(sigterm_number(), move || {
                o2.lock().expect("order").push(2usize);
                Ok(())
            })
            .expect("register second");

        manager.dispatch_signal(sigterm_number(), &mut None);
        assert_eq!(*order.lock().expect("order"), vec![1, 2]);
    }

    #[test]
    fn handler_error_does_not_stop_later_handlers() {
        let manager = SignalManager::new();
        let later = Arc::new(AtomicUsize::new(0));

        let _g1 = manager
            .handle(sigterm_number(), || Err("boom".into()))
            .expect("register failing handler");

        let later_c = Arc::clone(&later);
        let _g2 = manager
            .handle(sigterm_number(), move || {
                later_c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .expect("register later handler");

        manager.dispatch_signal(sigterm_number(), &mut None);
        assert_eq!(later.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn handler_panic_in_middle_continues_and_shuts_down() {
        let manager = SignalManager::new();
        manager.set_force_exit(|_| {});
        manager.set_runtime_logger(|_| {});
        let order = Arc::new(Mutex::new(Vec::new()));
        let shutdown = Arc::new(AtomicUsize::new(0));

        let o1 = Arc::clone(&order);
        let _g1 = manager
            .handle(sigterm_number(), move || {
                o1.lock().expect("order").push(1usize);
                Ok(())
            })
            .expect("register first");

        let _g2 = manager
            .handle(sigterm_number(), || panic!("handler panic in middle"))
            .expect("register panicking handler");

        let o3 = Arc::clone(&order);
        let _g3 = manager
            .handle(sigterm_number(), move || {
                o3.lock().expect("order").push(3usize);
                Ok(())
            })
            .expect("register later handler");

        let shutdown_c = Arc::clone(&shutdown);
        manager.on_shutdown(move || {
            shutdown_c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        manager.dispatch_signal(sigterm_number(), &mut None);
        assert_eq!(*order.lock().expect("order"), vec![1, 3]);
        assert_eq!(shutdown.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn handler_panic_payload_is_logged_and_redacted() {
        let manager = SignalManager::new();
        manager.set_force_exit(|_| {});
        let events = Arc::new(Mutex::new(Vec::<SignalRuntimeLogEvent>::new()));
        let captured = Arc::clone(&events);
        manager.set_runtime_logger(move |event| {
            captured.lock().expect("events").push(event.clone());
        });

        let _g = manager
            .handle(sigterm_number(), || panic!("boom password=hunter2"))
            .expect("register panicking handler");

        manager.dispatch_signal(sigterm_number(), &mut None);

        let events = events.lock().expect("events");
        let panic_logs: Vec<_> = events
            .iter()
            .filter(|e| e.kind == SignalRuntimeLogKind::HandlerPanic)
            .collect();
        assert_eq!(panic_logs.len(), 1);
        assert!(
            panic_logs[0].message.contains("boom"),
            "payload should be logged: {}",
            panic_logs[0].message
        );
        assert!(
            !panic_logs[0].message.contains("hunter2"),
            "secret leaked: {}",
            panic_logs[0].message
        );
        assert!(
            panic_logs[0]
                .message
                .contains(RUNTIME_REDACTION_REPLACEMENT),
            "expected redaction marker: {}",
            panic_logs[0].message
        );
    }

    #[test]
    fn rapid_sighup_is_not_coalesced() {
        let manager = SignalManager::new();
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_c = Arc::clone(&hits);
        manager.on_reload(move || {
            hits_c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        manager.dispatch_signal(sighup_number(), &mut None);
        manager.dispatch_signal(sighup_number(), &mut None);
        assert_eq!(hits.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn double_sigterm_runs_shutdown_hooks_once() {
        let manager = SignalManager::new();
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_c = Arc::clone(&hits);
        manager.on_shutdown(move || {
            hits_c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        manager.dispatch_signal(sigterm_number(), &mut None);
        manager.dispatch_signal(sigterm_number(), &mut None);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn redispatch_from_shutdown_hook_sees_empty_chain() {
        let manager = SignalManager::new();
        let hits = Arc::new(AtomicUsize::new(0));

        let hits_inner = Arc::clone(&hits);
        let manager_inner = manager.clone();
        manager.on_shutdown(move || {
            hits_inner.fetch_add(1, Ordering::SeqCst);
            manager_inner.dispatch_signal(sigterm_number(), &mut None);
            Ok(())
        });

        let hits_late = Arc::clone(&hits);
        manager.on_shutdown(move || {
            hits_late.fetch_add(10, Ordering::SeqCst);
            Ok(())
        });

        manager.dispatch_signal(sigterm_number(), &mut None);
        // LIFO: +10 then +1 (which re-dispatches an already-taken chain).
        assert_eq!(hits.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn handle_from_inside_handler_does_not_deadlock() {
        let manager = SignalManager::new();
        let stored: Arc<Mutex<Option<SignalRegistration>>> = Arc::new(Mutex::new(None));
        let stored_c = Arc::clone(&stored);
        let manager_c = manager.clone();

        let _outer = manager
            .handle(sigterm_number(), move || {
                let inner = manager_c
                    .handle(sigterm_number(), || Ok(()))
                    .expect("inner register during dispatch");
                *stored_c.lock().expect("stored") = Some(inner);
                Ok(())
            })
            .expect("outer register");

        manager.dispatch_signal(sigterm_number(), &mut None);
        assert!(
            stored.lock().expect("stored").is_some(),
            "inner registration should complete"
        );
    }

    #[test]
    fn handle_unknown_signal_number_errors() {
        let manager = SignalManager::new();
        let err = match manager.handle(i32::MAX, || Ok(())) {
            Ok(_) => panic!("unknown signal should not register"),
            Err(e) => e,
        };
        assert!(matches!(err, SignalManagerError::UnknownSignal(_)));
    }

    #[test]
    fn listen_rejects_second_caller() {
        let _listen_lock = lock_windows_listen();
        let manager = SignalManager::new();
        let injector = testing::SignalInjector::new(&manager);
        let manager_listen = manager.clone();
        let listener = thread::spawn(move || manager_listen.listen());

        injector
            .wait_for_listen(Duration::from_secs(1))
            .expect("listener should start");

        let err = manager.listen().expect_err("second listen");
        assert!(matches!(err, SignalManagerError::ListenAlreadyRunning));

        manager.stop();
        listener
            .join()
            .expect("listener thread should join")
            .expect("listener should exit cleanly");
    }

    #[cfg(windows)]
    #[test]
    fn windows_listen_can_start_after_previous_listener_stops() {
        let _listen_lock = lock_windows_listen();

        let first = SignalManager::new();
        let inj1 = testing::SignalInjector::new(&first);
        let first_c = first.clone();
        let t1 = thread::spawn(move || first_c.listen());
        inj1.wait_for_listen(Duration::from_secs(1))
            .expect("first listener should start");
        first.stop();
        t1.join()
            .expect("first listener thread should join")
            .expect("first listener should exit cleanly");

        let second = SignalManager::new();
        let inj2 = testing::SignalInjector::new(&second);
        let second_c = second.clone();
        let t2 = thread::spawn(move || second_c.listen());
        inj2.wait_for_listen(Duration::from_secs(1))
            .expect("second listen after first stop");
        second.stop();
        t2.join()
            .expect("second listener thread should join")
            .expect("second listener should exit cleanly");
    }

    #[cfg(windows)]
    #[test]
    fn windows_unsupported_signal_is_noop_registration() {
        let manager = SignalManager::new();
        assert!(!manager.supports(SIGHUP));
        let guard = manager
            .handle(SIGHUP, || Ok(()))
            .expect("Windows returns a noop registration, not an error");
        drop(guard);
    }

    #[test]
    fn shutdown_hook_error_does_not_stop_later_hooks() {
        let manager = SignalManager::new();
        let later = Arc::new(AtomicUsize::new(0));

        manager.on_shutdown(|| Err("first hook failed".into()));

        let later_c = Arc::clone(&later);
        manager.on_shutdown(move || {
            later_c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        // LIFO: later hook (+1) runs first, then the erroring hook. Both must run.
        manager.dispatch_signal(sigterm_number(), &mut None);
        assert_eq!(later.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn reload_hook_error_does_not_stop_later_hooks() {
        let manager = SignalManager::new();
        let later = Arc::new(AtomicUsize::new(0));

        manager.on_reload(|| Err("reload failed".into()));

        let later_c = Arc::clone(&later);
        manager.on_reload(move || {
            later_c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        manager.dispatch_signal(sighup_number(), &mut None);
        assert_eq!(later.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn first_sigint_with_double_tap_runs_shutdown_without_force_exit() {
        let manager = SignalManager::new();
        manager.enable_double_tap(DoubleTapConfig {
            window: Duration::from_secs(2),
            message: String::new(),
            exit_code: 99,
        });

        let hits = Arc::new(AtomicUsize::new(0));
        let hits_c = Arc::clone(&hits);
        manager.on_shutdown(move || {
            hits_c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        manager.dispatch_signal(sigint_number(), &mut None);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn second_sigint_after_double_tap_window_does_not_force_exit() {
        let manager = SignalManager::new();
        manager.enable_double_tap(DoubleTapConfig {
            window: Duration::from_millis(20),
            message: String::new(),
            exit_code: 99,
        });

        let mut first_sigint_at = None;
        manager.dispatch_signal(sigint_number(), &mut first_sigint_at);
        assert!(first_sigint_at.is_some());

        thread::sleep(Duration::from_millis(50));
        // Window expired: second SIGINT must not call process::exit.
        manager.dispatch_signal(sigint_number(), &mut first_sigint_at);
    }

    #[test]
    fn handler_registered_after_listen_receives_injected_signal() {
        let _listen_lock = lock_windows_listen();
        let manager = SignalManager::new();
        let injector = testing::SignalInjector::new(&manager);
        let manager_listen = manager.clone();
        let listener = thread::spawn(move || manager_listen.listen());

        injector
            .wait_for_listen(Duration::from_secs(1))
            .expect("listener should start");

        // SIGUSR2 is unsupported on Windows (`handle` returns a noop). Use a
        // catalog-supported number so the late registration is actually stored.
        #[cfg(windows)]
        let signal = sigterm_number();
        #[cfg(not(windows))]
        let signal = get_signal_number("SIGUSR2").unwrap_or(SIGUSR2);
        assert!(
            manager.supports(signal),
            "late-registration test requires a supported signal"
        );

        let hits = Arc::new(AtomicUsize::new(0));
        let hits_c = Arc::clone(&hits);
        let _guard = manager
            .handle(signal, move || {
                hits_c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .expect("register after listen");

        injector.inject(signal).expect("inject");

        let deadline = Instant::now() + Duration::from_secs(1);
        while hits.load(Ordering::SeqCst) == 0 && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }

        manager.stop();
        listener
            .join()
            .expect("listener thread should join")
            .expect("listener should exit cleanly");

        assert_eq!(
            hits.load(Ordering::SeqCst),
            1,
            "injected signal must reach a handler registered after listen()"
        );
    }

    #[test]
    fn catalog_timeout_seconds_term_int_zero_and_absent() {
        assert_eq!(
            catalog_shutdown_grace(sigterm_number()),
            Duration::from_secs(30)
        );
        assert_eq!(
            catalog_shutdown_grace(sigint_number()),
            Duration::from_secs(5)
        );
        let quit = get_signal_number("SIGQUIT").unwrap_or(SIGQUIT);
        assert_eq!(catalog_shutdown_grace(quit), Duration::ZERO);
        let usr1 = get_signal_number("SIGUSR1").unwrap_or(SIGUSR1);
        assert_eq!(
            catalog_shutdown_grace(usr1),
            DEFAULT_SHUTDOWN_GRACE,
            "absent timeout_seconds uses the SIGTERM default"
        );
    }

    #[test]
    fn manager_grace_override_none_restores_catalog() {
        let manager = SignalManager::new();
        assert_eq!(
            manager.shutdown_grace_for(sigterm_number()),
            Duration::from_secs(30)
        );
        manager.set_shutdown_grace(Some(Duration::from_millis(10)));
        assert_eq!(
            manager.shutdown_grace_for(sigterm_number()),
            Duration::from_millis(10)
        );
        manager.set_shutdown_grace(None);
        assert_eq!(
            manager.shutdown_grace_for(sigterm_number()),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn shutdown_hook_completes_within_grace() {
        let manager = SignalManager::new();
        manager.set_shutdown_grace(Some(Duration::from_secs(2)));
        let forced = Arc::new(AtomicUsize::new(0));
        let forced_c = Arc::clone(&forced);
        manager.set_force_exit(move |_| {
            forced_c.fetch_add(1, Ordering::SeqCst);
        });

        let hits = Arc::new(AtomicUsize::new(0));
        let hits_c = Arc::clone(&hits);
        manager.on_shutdown(move || {
            hits_c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        manager.dispatch_signal(sigterm_number(), &mut None);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
        assert_eq!(forced.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn shutdown_hook_exceeding_grace_takes_force_exit() {
        let manager = SignalManager::new();
        manager.set_shutdown_grace(Some(Duration::from_millis(40)));
        manager.set_runtime_logger(|_| {});
        let forced = Arc::new(AtomicUsize::new(0));
        let forced_code = Arc::new(Mutex::new(None::<i32>));
        let forced_c = Arc::clone(&forced);
        let code_c = Arc::clone(&forced_code);
        manager.set_force_exit(move |code| {
            forced_c.fetch_add(1, Ordering::SeqCst);
            *code_c.lock().expect("code") = Some(code);
        });

        manager.on_shutdown(|| {
            thread::sleep(Duration::from_millis(250));
            Ok(())
        });

        manager.dispatch_signal(sigterm_number(), &mut None);
        assert_eq!(forced.load(Ordering::SeqCst), 1);
        assert_eq!(
            *forced_code.lock().expect("code"),
            Some(EXIT_SIGTERM),
            "force-exit should use catalog SIGTERM exit code"
        );
    }

    #[test]
    fn shutdown_grace_zero_takes_force_exit_without_hooks() {
        let manager = SignalManager::new();
        manager.set_shutdown_grace(Some(Duration::ZERO));
        manager.set_runtime_logger(|_| {});
        let forced = Arc::new(AtomicUsize::new(0));
        let forced_c = Arc::clone(&forced);
        manager.set_force_exit(move |_| {
            forced_c.fetch_add(1, Ordering::SeqCst);
        });

        let hits = Arc::new(AtomicUsize::new(0));
        let hits_c = Arc::clone(&hits);
        manager.on_shutdown(move || {
            hits_c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        manager.dispatch_signal(sigterm_number(), &mut None);
        assert_eq!(forced.load(Ordering::SeqCst), 1);
        assert_eq!(
            hits.load(Ordering::SeqCst),
            0,
            "zero grace skips hooks (catalog immediate-exit semantics)"
        );
    }

    #[test]
    fn shutdown_grace_expiry_is_logged() {
        let manager = SignalManager::new();
        manager.set_shutdown_grace(Some(Duration::ZERO));
        manager.set_force_exit(|_| {});
        let events = Arc::new(Mutex::new(Vec::<SignalRuntimeLogEvent>::new()));
        let captured = Arc::clone(&events);
        manager.set_runtime_logger(move |event| {
            captured.lock().expect("events").push(event.clone());
        });

        manager.dispatch_signal(sigterm_number(), &mut None);
        let events = events.lock().expect("events");
        assert!(events
            .iter()
            .any(|e| e.kind == SignalRuntimeLogKind::ShutdownGraceExpired));
    }
}
