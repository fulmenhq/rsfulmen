//! Runtime signal handling plus foundry signal catalog re-exports.
//!
//! This module preserves the catalog API from `rsfulmen::foundry::signals` and
//! adds a runtime manager for handler dispatch, shutdown/reload chains, and
//! deterministic test injection.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

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
            self.run_shutdown_chain();
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
            let _ = (entry.handler)();
        }
    }

    fn run_shutdown_chain(&self) {
        let hooks = {
            let mut chain = self
                .inner
                .shutdown_chain
                .lock()
                .expect("shutdown mutex poisoned unexpectedly");
            std::mem::take(&mut *chain)
        };

        for hook in hooks.into_iter().rev() {
            let _ = hook();
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
            let _ = hook();
        }
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
    fn injector_basic_shutdown_dispatch() {
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
}
