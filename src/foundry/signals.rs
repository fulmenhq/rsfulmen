//! Signal Handling Catalog
//!
//! Provides signal definitions, behaviors, and platform support information
//! from the Crucible SSOT catalog.
//!
//! ## Overview
//!
//! This module exposes the Foundry-layer signal catalog data. It does NOT
//! provide handler registration or runtime signal handling - those are
//! Module-layer concerns for future implementation.
//!
//! ## Example
//!
//! ```rust
//! use rsfulmen::foundry::signals::{lookup_signal, get_signal_number, SIGTERM, EXIT_SIGTERM};
//!
//! // Look up signal by name
//! let term = lookup_signal("SIGTERM").unwrap();
//! assert_eq!(term.unix_number, 15);
//! assert_eq!(term.exit_code, 143);
//!
//! // Get platform-specific signal number
//! let num = get_signal_number("SIGTERM").unwrap();
//! assert_eq!(num, 15);
//!
//! // Use constants
//! assert_eq!(SIGTERM, 15);
//! assert_eq!(EXIT_SIGTERM, 143);
//! ```
//!
//! ## Platform Support
//!
//! Signal numbers vary by platform:
//! - SIGUSR1: 10 (Linux), 30 (macOS/FreeBSD)
//! - SIGUSR2: 12 (Linux), 31 (macOS/FreeBSD)
//!
//! Use [`get_signal_number`] for correct platform-specific values.

use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

use super::{FoundryError, FoundryResult};

// ============================================================================
// Signal Number Constants (Linux defaults)
// ============================================================================

/// SIGTERM signal number (15)
pub const SIGTERM: i32 = 15;
/// SIGINT signal number (2)
pub const SIGINT: i32 = 2;
/// SIGHUP signal number (1)
pub const SIGHUP: i32 = 1;
/// SIGQUIT signal number (3)
pub const SIGQUIT: i32 = 3;
/// SIGPIPE signal number (13)
pub const SIGPIPE: i32 = 13;
/// SIGALRM signal number (14)
pub const SIGALRM: i32 = 14;
/// SIGUSR1 signal number (10 on Linux, 30 on macOS/FreeBSD)
pub const SIGUSR1: i32 = 10;
/// SIGUSR2 signal number (12 on Linux, 31 on macOS/FreeBSD)
pub const SIGUSR2: i32 = 12;
/// SIGKILL signal number (9)
pub const SIGKILL: i32 = 9;

// ============================================================================
// Exit Code Constants (128+N pattern, Linux defaults)
// ============================================================================
//
// Note: Exit codes follow the POSIX 128+N pattern where N is the signal number.
// For signals with platform-specific numbers (SIGUSR1/SIGUSR2), these constants
// use Linux signal numbers. Use `get_exit_code()` for platform-aware values.

/// Exit code for SIGTERM (143 = 128 + 15)
pub const EXIT_SIGTERM: i32 = 143;
/// Exit code for SIGINT (130 = 128 + 2)
pub const EXIT_SIGINT: i32 = 130;
/// Exit code for SIGHUP (129 = 128 + 1)
pub const EXIT_SIGHUP: i32 = 129;
/// Exit code for SIGQUIT (131 = 128 + 3)
pub const EXIT_SIGQUIT: i32 = 131;
/// Exit code for SIGPIPE (141 = 128 + 13)
pub const EXIT_SIGPIPE: i32 = 141;
/// Exit code for SIGALRM (142 = 128 + 14)
pub const EXIT_SIGALRM: i32 = 142;
/// Exit code for SIGUSR1 (138 = 128 + 10 on Linux; 158 = 128 + 30 on macOS/FreeBSD)
pub const EXIT_SIGUSR1: i32 = 138;
/// Exit code for SIGUSR2 (140 = 128 + 12 on Linux; 159 = 128 + 31 on macOS/FreeBSD)
pub const EXIT_SIGUSR2: i32 = 140;
/// Exit code for SIGKILL (137 = 128 + 9)
pub const EXIT_SIGKILL: i32 = 137;

// ============================================================================
// Public Data Structures
// ============================================================================

/// Signal behavior type.
///
/// Defines how a signal should be handled by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalBehavior {
    /// Clean shutdown with timeout and cleanup chain
    GracefulShutdown,
    /// Graceful shutdown with force-quit option (Ctrl+C pattern)
    GracefulShutdownWithDoubleTap,
    /// Config reload via restart with mandatory schema validation
    ReloadViaRestart,
    /// Emergency shutdown without cleanup
    ImmediateExit,
    /// Application-defined handler
    Custom,
    /// Log and continue (no exit)
    ObserveOnly,
}

impl SignalBehavior {
    /// Get the behavior ID as used in the YAML catalog.
    pub fn id(&self) -> &'static str {
        match self {
            SignalBehavior::GracefulShutdown => "graceful_shutdown",
            SignalBehavior::GracefulShutdownWithDoubleTap => "graceful_shutdown_with_double_tap",
            SignalBehavior::ReloadViaRestart => "reload_via_restart",
            SignalBehavior::ImmediateExit => "immediate_exit",
            SignalBehavior::Custom => "custom",
            SignalBehavior::ObserveOnly => "observe_only",
        }
    }

    /// Get a human-readable name for the behavior.
    pub fn name(&self) -> &'static str {
        match self {
            SignalBehavior::GracefulShutdown => "Graceful Shutdown",
            SignalBehavior::GracefulShutdownWithDoubleTap => "Graceful Shutdown with Double-Tap",
            SignalBehavior::ReloadViaRestart => "Configuration Reload via Restart",
            SignalBehavior::ImmediateExit => "Immediate Exit",
            SignalBehavior::Custom => "Custom Handler",
            SignalBehavior::ObserveOnly => "Observe Only",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "graceful_shutdown" => Some(SignalBehavior::GracefulShutdown),
            "graceful_shutdown_with_double_tap" => {
                Some(SignalBehavior::GracefulShutdownWithDoubleTap)
            }
            "reload_via_restart" => Some(SignalBehavior::ReloadViaRestart),
            "immediate_exit" => Some(SignalBehavior::ImmediateExit),
            "custom" => Some(SignalBehavior::Custom),
            "observe_only" => Some(SignalBehavior::ObserveOnly),
            _ => None,
        }
    }
}

/// Windows fallback behavior type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FallbackBehavior {
    /// Use HTTP /admin/signal endpoint
    HttpAdminEndpoint,
    /// Handle via exception/error handling
    ExceptionHandling,
    /// Use native timer APIs
    TimerApi,
}

impl FallbackBehavior {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "http_admin_endpoint" => Some(FallbackBehavior::HttpAdminEndpoint),
            "exception_handling" => Some(FallbackBehavior::ExceptionHandling),
            "timer_api" => Some(FallbackBehavior::TimerApi),
            _ => None,
        }
    }
}

/// Platform support level for Unix-like systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportLevel {
    /// Signal is natively supported
    Native,
    /// Signal is not supported on this platform
    Unsupported,
}

impl SupportLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "native" => Some(SupportLevel::Native),
            "unsupported" => Some(SupportLevel::Unsupported),
            _ => None,
        }
    }
}

/// Platform support level for Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowsSupportLevel {
    /// Signal has a native Windows equivalent
    Native,
    /// Signal is mapped to a Windows console event
    Mapped,
    /// Signal is not supported on Windows
    Unsupported,
}

impl WindowsSupportLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "native" => Some(WindowsSupportLevel::Native),
            "mapped" => Some(WindowsSupportLevel::Mapped),
            "unsupported" => Some(WindowsSupportLevel::Unsupported),
            _ => None,
        }
    }
}

/// Windows fallback configuration for unsupported signals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsFallback {
    /// The fallback behavior to use
    pub fallback_behavior: FallbackBehavior,
    /// Log level for the fallback message
    pub log_level: String,
    /// Human-readable log message
    pub log_message: String,
    /// Structured log template
    pub log_template: String,
    /// Operational hint for users
    pub operation_hint: String,
    /// Telemetry event name
    pub telemetry_event: String,
    /// Telemetry tags
    pub telemetry_tags: HashMap<String, String>,
}

/// Platform-specific signal number overrides.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlatformOverrides {
    /// Signal number on macOS (Darwin)
    pub darwin: Option<i32>,
    /// Signal number on FreeBSD
    pub freebsd: Option<i32>,
}

/// Signal definition from the Crucible catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    /// Short identifier (e.g., "term", "int", "hup")
    pub id: String,
    /// Signal name (e.g., "SIGTERM", "SIGINT")
    pub name: String,
    /// Unix signal number (Linux default)
    pub unix_number: i32,
    /// Windows console event name, if mapped
    pub windows_event: Option<String>,
    /// Human-readable description
    pub description: String,
    /// Default behavior for this signal
    pub default_behavior: SignalBehavior,
    /// Exit code when process terminates due to this signal (128+N)
    pub exit_code: i32,
    /// Timeout in seconds for graceful operations
    pub timeout_seconds: Option<u32>,
    /// Cleanup actions to perform
    pub cleanup_actions: Vec<String>,
    /// Usage notes and guidance
    pub usage_notes: Option<String>,
    /// Double-tap window in seconds (SIGINT only)
    pub double_tap_window_seconds: Option<u32>,
    /// Double-tap hint message (SIGINT only)
    pub double_tap_message: Option<String>,
    /// Reload strategy (SIGHUP only)
    pub reload_strategy: Option<String>,
    /// Whether validation is required (SIGHUP only)
    pub validation_required: Option<bool>,
    /// Windows fallback configuration
    pub windows_fallback: Option<WindowsFallback>,
    /// Platform-specific signal number overrides
    pub platform_overrides: Option<PlatformOverrides>,
}

/// A phase within a behavior definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviorPhase {
    /// Phase name
    pub name: String,
    /// Phase description
    pub description: String,
}

/// Behavior definition with phases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Behavior {
    /// Behavior identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of the behavior
    pub description: String,
    /// Ordered phases of the behavior
    pub phases: Vec<BehaviorPhase>,
}

/// Platform support entry from the support matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformSupport {
    /// Signal name
    pub signal: String,
    /// Linux support level
    pub linux: SupportLevel,
    /// macOS support level
    pub macos: SupportLevel,
    /// FreeBSD support level
    pub freebsd: SupportLevel,
    /// Windows support level
    pub windows: WindowsSupportLevel,
    /// Fallback behavior for unsupported platforms
    pub fallback: Option<FallbackBehavior>,
    /// Additional notes
    pub notes: String,
}

// ============================================================================
// Raw YAML Types (for serde deserialization)
// ============================================================================

#[derive(Debug, Deserialize)]
struct RawWindowsFallback {
    fallback_behavior: String,
    log_level: String,
    log_message: String,
    log_template: String,
    operation_hint: String,
    telemetry_event: String,
    telemetry_tags: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct RawPlatformOverrides {
    darwin: Option<i32>,
    freebsd: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct RawSignal {
    id: String,
    name: String,
    unix_number: i32,
    windows_event: Option<String>,
    description: String,
    default_behavior: String,
    exit_code: i32,
    timeout_seconds: Option<u32>,
    cleanup_actions: Option<Vec<String>>,
    usage_notes: Option<String>,
    double_tap_window_seconds: Option<u32>,
    double_tap_message: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    double_tap_behavior: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    double_tap_exit_code: Option<i32>,
    reload_strategy: Option<String>,
    validation_required: Option<bool>,
    windows_fallback: Option<RawWindowsFallback>,
    platform_overrides: Option<RawPlatformOverrides>,
}

#[derive(Debug, Deserialize)]
struct RawBehaviorPhase {
    name: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct RawBehavior {
    id: String,
    name: String,
    description: String,
    phases: Vec<RawBehaviorPhase>,
}

#[derive(Debug, Deserialize)]
struct RawPlatformSupport {
    signal: String,
    linux: String,
    macos: String,
    freebsd: String,
    windows: String,
    fallback: Option<String>,
    notes: String,
}

#[derive(Debug, Deserialize)]
struct RawSignalCatalog {
    #[allow(dead_code)]
    description: String,
    #[allow(dead_code)]
    version: String,
    signals: Vec<RawSignal>,
    behaviors: Vec<RawBehavior>,
    #[allow(dead_code)]
    os_mappings: serde_yaml::Value, // We don't need to parse this in detail
    platform_support: Vec<RawPlatformSupport>,
    #[allow(dead_code)]
    exit_codes: serde_yaml::Value, // We have exit_codes module for this
}

// ============================================================================
// Catalog Loading and Indexing
// ============================================================================

/// Embedded signals YAML from Crucible.
const SIGNALS_YAML: &str = include_str!("../../config/crucible-rs/library/foundry/signals.yaml");

/// Pre-computed indexes for fast lookups.
struct SignalIndexes {
    /// All signals in catalog order
    signals: Vec<Signal>,
    /// Index by name (SIGTERM, etc.)
    by_name: HashMap<String, usize>,
    /// Index by id (term, int, etc.)
    by_id: HashMap<String, usize>,
    /// Index by Unix number
    by_number: HashMap<i32, usize>,
    /// All behaviors
    behaviors: Vec<Behavior>,
    /// Behaviors by id
    behaviors_by_id: HashMap<String, usize>,
    /// Platform support entries
    platform_support: Vec<PlatformSupport>,
    /// Platform support by signal name
    platform_support_by_signal: HashMap<String, usize>,
}

impl SignalIndexes {
    fn load() -> FoundryResult<Self> {
        let catalog: RawSignalCatalog = serde_yaml::from_str(SIGNALS_YAML)
            .map_err(|e| FoundryError::LoadError(format!("Failed to parse signals.yaml: {}", e)))?;

        // Parse signals
        let mut signals = Vec::with_capacity(catalog.signals.len());
        let mut by_name = HashMap::new();
        let mut by_id = HashMap::new();
        let mut by_number = HashMap::new();

        for raw in catalog.signals {
            let default_behavior =
                SignalBehavior::from_str(&raw.default_behavior).ok_or_else(|| {
                    FoundryError::LoadError(format!(
                        "Unknown behavior '{}' for signal {}",
                        raw.default_behavior, raw.name
                    ))
                })?;

            let windows_fallback = raw
                .windows_fallback
                .map(|wf| {
                    Ok::<_, FoundryError>(WindowsFallback {
                        fallback_behavior: FallbackBehavior::from_str(&wf.fallback_behavior)
                            .ok_or_else(|| {
                                FoundryError::LoadError(format!(
                                    "Unknown fallback behavior '{}'",
                                    wf.fallback_behavior
                                ))
                            })?,
                        log_level: wf.log_level,
                        log_message: wf.log_message,
                        log_template: wf.log_template,
                        operation_hint: wf.operation_hint,
                        telemetry_event: wf.telemetry_event,
                        telemetry_tags: wf.telemetry_tags,
                    })
                })
                .transpose()?;

            let platform_overrides = raw.platform_overrides.map(|po| PlatformOverrides {
                darwin: po.darwin,
                freebsd: po.freebsd,
            });

            let index = signals.len();
            let signal = Signal {
                id: raw.id.clone(),
                name: raw.name.clone(),
                unix_number: raw.unix_number,
                windows_event: raw.windows_event,
                description: raw.description,
                default_behavior,
                exit_code: raw.exit_code,
                timeout_seconds: raw.timeout_seconds,
                cleanup_actions: raw.cleanup_actions.unwrap_or_default(),
                usage_notes: raw.usage_notes,
                double_tap_window_seconds: raw.double_tap_window_seconds,
                double_tap_message: raw.double_tap_message,
                reload_strategy: raw.reload_strategy,
                validation_required: raw.validation_required,
                windows_fallback,
                platform_overrides,
            };

            by_name.insert(signal.name.clone(), index);
            by_id.insert(signal.id.clone(), index);
            by_number.insert(signal.unix_number, index);

            signals.push(signal);
        }

        // Parse behaviors
        let mut behaviors = Vec::with_capacity(catalog.behaviors.len());
        let mut behaviors_by_id = HashMap::new();

        for raw in catalog.behaviors {
            let index = behaviors.len();
            let behavior = Behavior {
                id: raw.id.clone(),
                name: raw.name,
                description: raw.description,
                phases: raw
                    .phases
                    .into_iter()
                    .map(|p| BehaviorPhase {
                        name: p.name,
                        description: p.description,
                    })
                    .collect(),
            };
            behaviors_by_id.insert(behavior.id.clone(), index);
            behaviors.push(behavior);
        }

        // Parse platform support
        let mut platform_support = Vec::with_capacity(catalog.platform_support.len());
        let mut platform_support_by_signal = HashMap::new();

        for raw in catalog.platform_support {
            let index = platform_support.len();
            let support = PlatformSupport {
                signal: raw.signal.clone(),
                linux: SupportLevel::from_str(&raw.linux).ok_or_else(|| {
                    FoundryError::LoadError(format!(
                        "Unknown Linux support level '{}' for signal {}",
                        raw.linux, raw.signal
                    ))
                })?,
                macos: SupportLevel::from_str(&raw.macos).ok_or_else(|| {
                    FoundryError::LoadError(format!(
                        "Unknown macOS support level '{}' for signal {}",
                        raw.macos, raw.signal
                    ))
                })?,
                freebsd: SupportLevel::from_str(&raw.freebsd).ok_or_else(|| {
                    FoundryError::LoadError(format!(
                        "Unknown FreeBSD support level '{}' for signal {}",
                        raw.freebsd, raw.signal
                    ))
                })?,
                windows: WindowsSupportLevel::from_str(&raw.windows).ok_or_else(|| {
                    FoundryError::LoadError(format!(
                        "Unknown Windows support level '{}' for signal {}",
                        raw.windows, raw.signal
                    ))
                })?,
                fallback: raw
                    .fallback
                    .map(|f| {
                        FallbackBehavior::from_str(&f).ok_or_else(|| {
                            FoundryError::LoadError(format!(
                                "Unknown fallback behavior '{}' for signal {}",
                                f, raw.signal
                            ))
                        })
                    })
                    .transpose()?,
                notes: raw.notes,
            };
            platform_support_by_signal.insert(support.signal.clone(), index);
            platform_support.push(support);
        }

        Ok(Self {
            signals,
            by_name,
            by_id,
            by_number,
            behaviors,
            behaviors_by_id,
            platform_support,
            platform_support_by_signal,
        })
    }
}

/// Lazily initialized signal indexes.
static CATALOG: Lazy<SignalIndexes> =
    Lazy::new(|| SignalIndexes::load().expect("Failed to load embedded signals catalog"));

// ============================================================================
// Public Lookup Functions
// ============================================================================

/// Look up a signal by name (e.g., "SIGTERM").
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::signals::lookup_signal;
///
/// let term = lookup_signal("SIGTERM").unwrap();
/// assert_eq!(term.unix_number, 15);
/// assert_eq!(term.exit_code, 143);
/// ```
pub fn lookup_signal(name: &str) -> Option<&'static Signal> {
    CATALOG.by_name.get(name).map(|&i| &CATALOG.signals[i])
}

/// Look up a signal by its short ID (e.g., "term", "int").
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::signals::lookup_signal_by_id;
///
/// let term = lookup_signal_by_id("term").unwrap();
/// assert_eq!(term.name, "SIGTERM");
/// ```
pub fn lookup_signal_by_id(id: &str) -> Option<&'static Signal> {
    CATALOG.by_id.get(id).map(|&i| &CATALOG.signals[i])
}

/// Look up a signal by Unix signal number.
///
/// Note: For SIGUSR1/SIGUSR2, this uses the Linux signal numbers (10/12).
/// Use [`get_signal_number`] for platform-specific lookups.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::signals::lookup_signal_by_number;
///
/// let term = lookup_signal_by_number(15).unwrap();
/// assert_eq!(term.name, "SIGTERM");
/// ```
pub fn lookup_signal_by_number(number: i32) -> Option<&'static Signal> {
    CATALOG.by_number.get(&number).map(|&i| &CATALOG.signals[i])
}

/// List all signals in the catalog.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::signals::list_signals;
///
/// let signals = list_signals();
/// assert_eq!(signals.len(), 9);
/// ```
pub fn list_signals() -> &'static [Signal] {
    &CATALOG.signals
}

/// Get the number of signals in the catalog.
pub fn signal_count() -> usize {
    CATALOG.signals.len()
}

/// Look up a behavior definition by ID.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::signals::lookup_behavior;
///
/// let graceful = lookup_behavior("graceful_shutdown").unwrap();
/// assert!(graceful.phases.len() > 0);
/// ```
pub fn lookup_behavior(id: &str) -> Option<&'static Behavior> {
    CATALOG
        .behaviors_by_id
        .get(id)
        .map(|&i| &CATALOG.behaviors[i])
}

/// List all behavior definitions.
pub fn list_behaviors() -> &'static [Behavior] {
    &CATALOG.behaviors
}

/// Get platform support information for a signal.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::signals::{get_platform_support, SupportLevel, WindowsSupportLevel};
///
/// let support = get_platform_support("SIGTERM").unwrap();
/// assert_eq!(support.linux, SupportLevel::Native);
/// assert_eq!(support.windows, WindowsSupportLevel::Mapped);
/// ```
pub fn get_platform_support(signal_name: &str) -> Option<&'static PlatformSupport> {
    CATALOG
        .platform_support_by_signal
        .get(signal_name)
        .map(|&i| &CATALOG.platform_support[i])
}

/// Check if a signal is supported on the current platform.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::signals::is_signal_supported;
///
/// // SIGTERM is supported on all platforms
/// assert!(is_signal_supported("SIGTERM"));
/// ```
#[allow(clippy::if_same_then_else)]
pub fn is_signal_supported(signal_name: &str) -> bool {
    let support = match get_platform_support(signal_name) {
        Some(s) => s,
        None => return false,
    };

    #[cfg(target_os = "linux")]
    {
        support.linux == SupportLevel::Native
    }

    #[cfg(target_os = "macos")]
    {
        support.macos == SupportLevel::Native
    }

    #[cfg(target_os = "freebsd")]
    {
        support.freebsd == SupportLevel::Native
    }

    #[cfg(target_os = "windows")]
    {
        matches!(
            support.windows,
            WindowsSupportLevel::Native | WindowsSupportLevel::Mapped
        )
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "windows"
    )))]
    {
        // Default to Linux behavior for other Unix-like systems
        support.linux == SupportLevel::Native
    }
}

/// List all platform support entries.
pub fn list_platform_support() -> &'static [PlatformSupport] {
    &CATALOG.platform_support
}

// ============================================================================
// Exit Code Helpers
// ============================================================================

/// Get the exit code for a signal on the current platform.
///
/// Exit codes follow the POSIX 128+N pattern where N is the platform-specific
/// signal number. For SIGUSR1/SIGUSR2, this returns different values on
/// macOS/FreeBSD (158/159) vs Linux (138/140).
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::signals::get_exit_code;
///
/// assert_eq!(get_exit_code("SIGTERM"), Some(143));
/// assert_eq!(get_exit_code("SIGINT"), Some(130));
///
/// // SIGUSR1 exit code varies by platform
/// let usr1_exit = get_exit_code("SIGUSR1").unwrap();
/// #[cfg(target_os = "linux")]
/// assert_eq!(usr1_exit, 138); // 128 + 10
/// #[cfg(target_os = "macos")]
/// assert_eq!(usr1_exit, 158); // 128 + 30
/// ```
pub fn get_exit_code(signal_name: &str) -> Option<i32> {
    get_signal_number(signal_name).map(|n| 128 + n)
}

/// Get the exit code for a signal on a specific platform.
///
/// Valid platform values: "linux", "darwin", "freebsd", "windows"
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::signals::get_exit_code_for_platform;
///
/// assert_eq!(get_exit_code_for_platform("SIGUSR1", "linux"), Some(138));
/// assert_eq!(get_exit_code_for_platform("SIGUSR1", "darwin"), Some(158));
/// assert_eq!(get_exit_code_for_platform("SIGUSR1", "freebsd"), Some(158));
/// assert_eq!(get_exit_code_for_platform("SIGTERM", "linux"), Some(143));
/// ```
pub fn get_exit_code_for_platform(signal_name: &str, platform: &str) -> Option<i32> {
    get_signal_number_for_platform(signal_name, platform).map(|n| 128 + n)
}

/// Find a signal from its exit code (using catalog values).
///
/// Note: This uses the catalog's exit_code field (Linux defaults for SIGUSR1/2).
/// For platform-specific exit code matching, use [`signal_from_exit_code_for_platform`].
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::signals::signal_from_exit_code;
///
/// let sig = signal_from_exit_code(143).unwrap();
/// assert_eq!(sig.name, "SIGTERM");
/// ```
pub fn signal_from_exit_code(exit_code: i32) -> Option<&'static Signal> {
    // For standard signals, the catalog value matches
    if let Some(sig) = CATALOG.signals.iter().find(|s| s.exit_code == exit_code) {
        return Some(sig);
    }
    // Check for macOS/FreeBSD SIGUSR1/2 exit codes
    if exit_code == 158 {
        return lookup_signal("SIGUSR1");
    }
    if exit_code == 159 {
        return lookup_signal("SIGUSR2");
    }
    None
}

// ============================================================================
// Platform-Specific Signal Number Resolution
// ============================================================================

/// Get the signal number for the current platform.
///
/// This handles platform differences for SIGUSR1/SIGUSR2:
/// - Linux: SIGUSR1=10, SIGUSR2=12
/// - macOS/FreeBSD: SIGUSR1=30, SIGUSR2=31
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::signals::get_signal_number;
///
/// // SIGTERM is 15 on all platforms
/// assert_eq!(get_signal_number("SIGTERM"), Some(15));
///
/// // SIGUSR1 varies: 10 on Linux, 30 on macOS/FreeBSD
/// let usr1 = get_signal_number("SIGUSR1").unwrap();
/// #[cfg(target_os = "linux")]
/// assert_eq!(usr1, 10);
/// #[cfg(target_os = "macos")]
/// assert_eq!(usr1, 30);
/// ```
pub fn get_signal_number(signal_name: &str) -> Option<i32> {
    let signal = lookup_signal(signal_name)?;

    #[cfg(target_os = "macos")]
    if let Some(ref overrides) = signal.platform_overrides {
        if let Some(darwin) = overrides.darwin {
            return Some(darwin);
        }
    }

    #[cfg(target_os = "freebsd")]
    if let Some(ref overrides) = signal.platform_overrides {
        if let Some(freebsd) = overrides.freebsd {
            return Some(freebsd);
        }
    }

    Some(signal.unix_number)
}

/// Get the signal number for a specific platform.
///
/// Valid platform values: "linux", "darwin", "macos", "freebsd", "windows"
///
/// Returns `None` for unknown platforms to fail fast on invalid input.
///
/// # Example
///
/// ```rust
/// use rsfulmen::foundry::signals::get_signal_number_for_platform;
///
/// assert_eq!(get_signal_number_for_platform("SIGUSR1", "linux"), Some(10));
/// assert_eq!(get_signal_number_for_platform("SIGUSR1", "darwin"), Some(30));
/// assert_eq!(get_signal_number_for_platform("SIGUSR1", "unknown"), None);
/// ```
pub fn get_signal_number_for_platform(signal_name: &str, platform: &str) -> Option<i32> {
    let signal = lookup_signal(signal_name)?;

    match platform {
        "darwin" | "macos" => {
            if let Some(ref overrides) = signal.platform_overrides {
                if let Some(darwin) = overrides.darwin {
                    return Some(darwin);
                }
            }
            Some(signal.unix_number)
        }
        "freebsd" => {
            if let Some(ref overrides) = signal.platform_overrides {
                if let Some(freebsd) = overrides.freebsd {
                    return Some(freebsd);
                }
            }
            Some(signal.unix_number)
        }
        "linux" => Some(signal.unix_number),
        "windows" => {
            // Windows doesn't use signal numbers, but we return the Unix number for reference
            Some(signal.unix_number)
        }
        _ => None, // Unknown platform - fail fast
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Catalog Loading Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_catalog_loads_all_signals() {
        let signals = list_signals();
        assert_eq!(signals.len(), 9, "Expected 9 signals in catalog");
    }

    #[test]
    fn test_catalog_loads_all_behaviors() {
        let behaviors = list_behaviors();
        assert_eq!(behaviors.len(), 6, "Expected 6 behaviors in catalog");
    }

    #[test]
    fn test_catalog_loads_platform_support() {
        let support = list_platform_support();
        assert_eq!(support.len(), 9, "Expected 9 platform support entries");
    }

    #[test]
    fn test_signal_count() {
        assert_eq!(signal_count(), 9);
        assert_eq!(signal_count(), list_signals().len());
    }

    // -------------------------------------------------------------------------
    // Signal Lookup Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_lookup_signal_by_name() {
        let term = lookup_signal("SIGTERM").expect("SIGTERM should exist");
        assert_eq!(term.name, "SIGTERM");
        assert_eq!(term.id, "term");
        assert_eq!(term.unix_number, 15);
        assert_eq!(term.exit_code, 143);
        assert_eq!(term.default_behavior, SignalBehavior::GracefulShutdown);
    }

    #[test]
    fn test_lookup_signal_by_id() {
        let int = lookup_signal_by_id("int").expect("int should exist");
        assert_eq!(int.name, "SIGINT");
        assert_eq!(int.unix_number, 2);
        assert_eq!(int.exit_code, 130);
        assert_eq!(
            int.default_behavior,
            SignalBehavior::GracefulShutdownWithDoubleTap
        );
    }

    #[test]
    fn test_lookup_signal_by_number() {
        let hup = lookup_signal_by_number(1).expect("Signal 1 should exist");
        assert_eq!(hup.name, "SIGHUP");
        assert_eq!(hup.id, "hup");
    }

    #[test]
    fn test_lookup_signal_not_found() {
        assert!(lookup_signal("SIGFOO").is_none());
        assert!(lookup_signal_by_id("foo").is_none());
        assert!(lookup_signal_by_number(999).is_none());
    }

    #[test]
    fn test_all_signals_accessible() {
        let expected = [
            ("SIGTERM", "term", 15, 143),
            ("SIGINT", "int", 2, 130),
            ("SIGHUP", "hup", 1, 129),
            ("SIGQUIT", "quit", 3, 131),
            ("SIGPIPE", "pipe", 13, 141),
            ("SIGALRM", "alrm", 14, 142),
            ("SIGUSR1", "usr1", 10, 138),
            ("SIGUSR2", "usr2", 12, 140),
            ("SIGKILL", "kill", 9, 137),
        ];

        for (name, id, number, exit_code) in expected {
            let by_name = lookup_signal(name).unwrap_or_else(|| panic!("{} should exist", name));
            let by_id = lookup_signal_by_id(id).unwrap_or_else(|| panic!("id {} should exist", id));
            let by_num = lookup_signal_by_number(number)
                .unwrap_or_else(|| panic!("number {} should exist", number));

            assert_eq!(by_name.name, name);
            assert_eq!(by_id.name, name);
            assert_eq!(by_num.name, name);
            assert_eq!(by_name.exit_code, exit_code);
        }
    }

    // -------------------------------------------------------------------------
    // Behavior Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_lookup_behavior() {
        let graceful =
            lookup_behavior("graceful_shutdown").expect("graceful_shutdown should exist");
        assert_eq!(graceful.name, "Graceful Shutdown");
        assert!(!graceful.phases.is_empty());
    }

    #[test]
    fn test_all_behaviors_accessible() {
        let expected_ids = [
            "graceful_shutdown",
            "graceful_shutdown_with_double_tap",
            "reload_via_restart",
            "immediate_exit",
            "custom",
            "observe_only",
        ];

        for id in expected_ids {
            assert!(
                lookup_behavior(id).is_some(),
                "Behavior {} should exist",
                id
            );
        }
    }

    #[test]
    fn test_lookup_behavior_not_found() {
        assert!(lookup_behavior("unknown_behavior").is_none());
        assert!(lookup_behavior("").is_none());
        assert!(lookup_behavior("graceful").is_none()); // Partial match shouldn't work
    }

    #[test]
    fn test_behavior_phases() {
        let double_tap = lookup_behavior("graceful_shutdown_with_double_tap")
            .expect("double_tap behavior should exist");
        assert!(
            double_tap.phases.len() >= 5,
            "Double-tap should have multiple phases"
        );

        // Verify phase structure
        let phase_names: Vec<&str> = double_tap.phases.iter().map(|p| p.name.as_str()).collect();
        assert!(phase_names.contains(&"first_signal"));
        assert!(phase_names.contains(&"second_signal_check"));
    }

    // -------------------------------------------------------------------------
    // Platform Support Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_platform_support_sigterm() {
        let support = get_platform_support("SIGTERM").expect("SIGTERM support should exist");
        assert_eq!(support.linux, SupportLevel::Native);
        assert_eq!(support.macos, SupportLevel::Native);
        assert_eq!(support.freebsd, SupportLevel::Native);
        assert_eq!(support.windows, WindowsSupportLevel::Mapped);
    }

    #[test]
    fn test_platform_support_sighup() {
        let support = get_platform_support("SIGHUP").expect("SIGHUP support should exist");
        assert_eq!(support.linux, SupportLevel::Native);
        assert_eq!(support.windows, WindowsSupportLevel::Unsupported);
        assert_eq!(support.fallback, Some(FallbackBehavior::HttpAdminEndpoint));
    }

    #[test]
    fn test_is_signal_supported() {
        // SIGTERM should be supported on all platforms
        assert!(is_signal_supported("SIGTERM"));
        assert!(is_signal_supported("SIGINT"));

        // Unknown signals should not be supported
        assert!(!is_signal_supported("SIGFOO"));
    }

    #[test]
    fn test_list_platform_support() {
        let support = list_platform_support();
        assert_eq!(support.len(), 9);

        // Verify all expected signals are present
        let signal_names: Vec<&str> = support.iter().map(|s| s.signal.as_str()).collect();
        assert!(signal_names.contains(&"SIGTERM"));
        assert!(signal_names.contains(&"SIGINT"));
        assert!(signal_names.contains(&"SIGHUP"));
        assert!(signal_names.contains(&"SIGQUIT"));
        assert!(signal_names.contains(&"SIGPIPE"));
        assert!(signal_names.contains(&"SIGALRM"));
        assert!(signal_names.contains(&"SIGUSR1"));
        assert!(signal_names.contains(&"SIGUSR2"));
        assert!(signal_names.contains(&"SIGKILL"));
    }

    #[test]
    fn test_platform_support_not_found() {
        assert!(get_platform_support("SIGFOO").is_none());
        assert!(get_platform_support("").is_none());
    }

    // -------------------------------------------------------------------------
    // Exit Code Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_get_exit_code() {
        // Standard signals have consistent exit codes across platforms
        assert_eq!(get_exit_code("SIGTERM"), Some(143));
        assert_eq!(get_exit_code("SIGINT"), Some(130));
        assert_eq!(get_exit_code("SIGHUP"), Some(129));
        assert_eq!(get_exit_code("SIGQUIT"), Some(131));
        assert_eq!(get_exit_code("SIGFOO"), None);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_get_exit_code_sigusr_linux() {
        assert_eq!(get_exit_code("SIGUSR1"), Some(138)); // 128 + 10
        assert_eq!(get_exit_code("SIGUSR2"), Some(140)); // 128 + 12
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_get_exit_code_sigusr_macos() {
        assert_eq!(get_exit_code("SIGUSR1"), Some(158)); // 128 + 30
        assert_eq!(get_exit_code("SIGUSR2"), Some(159)); // 128 + 31
    }

    #[test]
    fn test_get_exit_code_for_platform() {
        // Standard signals - same across platforms
        assert_eq!(get_exit_code_for_platform("SIGTERM", "linux"), Some(143));
        assert_eq!(get_exit_code_for_platform("SIGTERM", "darwin"), Some(143));
        assert_eq!(get_exit_code_for_platform("SIGTERM", "freebsd"), Some(143));

        // SIGUSR1/2 - platform-specific
        assert_eq!(get_exit_code_for_platform("SIGUSR1", "linux"), Some(138));
        assert_eq!(get_exit_code_for_platform("SIGUSR1", "darwin"), Some(158));
        assert_eq!(get_exit_code_for_platform("SIGUSR1", "freebsd"), Some(158));

        assert_eq!(get_exit_code_for_platform("SIGUSR2", "linux"), Some(140));
        assert_eq!(get_exit_code_for_platform("SIGUSR2", "darwin"), Some(159));
        assert_eq!(get_exit_code_for_platform("SIGUSR2", "freebsd"), Some(159));

        // Unknown platform returns None
        assert_eq!(get_exit_code_for_platform("SIGTERM", "unknown"), None);
    }

    #[test]
    fn test_exit_code_128_plus_n_pattern() {
        // Test using get_exit_code_for_platform for Linux (catalog defaults)
        for signal in list_signals() {
            let expected = 128 + signal.unix_number;
            let actual = get_exit_code_for_platform(&signal.name, "linux").unwrap();
            assert_eq!(
                actual, expected,
                "Exit code for {} on Linux should be 128+{}={}",
                signal.name, signal.unix_number, expected
            );
        }
    }

    #[test]
    fn test_signal_from_exit_code() {
        // Standard signals
        let term = signal_from_exit_code(143).expect("143 should map to SIGTERM");
        assert_eq!(term.name, "SIGTERM");

        let int = signal_from_exit_code(130).expect("130 should map to SIGINT");
        assert_eq!(int.name, "SIGINT");

        // Linux SIGUSR1/2 exit codes
        let usr1_linux = signal_from_exit_code(138).expect("138 should map to SIGUSR1");
        assert_eq!(usr1_linux.name, "SIGUSR1");

        // macOS/FreeBSD SIGUSR1/2 exit codes
        let usr1_macos = signal_from_exit_code(158).expect("158 should map to SIGUSR1");
        assert_eq!(usr1_macos.name, "SIGUSR1");

        let usr2_macos = signal_from_exit_code(159).expect("159 should map to SIGUSR2");
        assert_eq!(usr2_macos.name, "SIGUSR2");

        assert!(signal_from_exit_code(999).is_none());
    }

    // -------------------------------------------------------------------------
    // Platform-Specific Signal Number Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_get_signal_number_sigterm() {
        // SIGTERM is 15 on all platforms
        assert_eq!(get_signal_number("SIGTERM"), Some(15));
    }

    #[test]
    fn test_get_signal_number_for_platform() {
        // SIGUSR1: 10 on Linux, 30 on macOS/FreeBSD
        assert_eq!(get_signal_number_for_platform("SIGUSR1", "linux"), Some(10));
        assert_eq!(
            get_signal_number_for_platform("SIGUSR1", "darwin"),
            Some(30)
        );
        assert_eq!(
            get_signal_number_for_platform("SIGUSR1", "freebsd"),
            Some(30)
        );

        // SIGUSR2: 12 on Linux, 31 on macOS/FreeBSD
        assert_eq!(get_signal_number_for_platform("SIGUSR2", "linux"), Some(12));
        assert_eq!(
            get_signal_number_for_platform("SIGUSR2", "darwin"),
            Some(31)
        );
        assert_eq!(
            get_signal_number_for_platform("SIGUSR2", "freebsd"),
            Some(31)
        );

        // "macos" alias works
        assert_eq!(get_signal_number_for_platform("SIGUSR1", "macos"), Some(30));

        // Windows returns Unix number for reference
        assert_eq!(
            get_signal_number_for_platform("SIGTERM", "windows"),
            Some(15)
        );
    }

    #[test]
    fn test_get_signal_number_unknown_platform() {
        // Unknown platforms return None (fail fast)
        assert_eq!(get_signal_number_for_platform("SIGTERM", "unknown"), None);
        assert_eq!(get_signal_number_for_platform("SIGTERM", ""), None);
        assert_eq!(get_signal_number_for_platform("SIGTERM", "solaris"), None);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_sigusr1_linux() {
        assert_eq!(get_signal_number("SIGUSR1"), Some(10));
        assert_eq!(get_signal_number("SIGUSR2"), Some(12));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_sigusr1_macos() {
        assert_eq!(get_signal_number("SIGUSR1"), Some(30));
        assert_eq!(get_signal_number("SIGUSR2"), Some(31));
    }

    // -------------------------------------------------------------------------
    // Constants Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_signal_constants() {
        assert_eq!(SIGTERM, 15);
        assert_eq!(SIGINT, 2);
        assert_eq!(SIGHUP, 1);
        assert_eq!(SIGQUIT, 3);
        assert_eq!(SIGPIPE, 13);
        assert_eq!(SIGALRM, 14);
        assert_eq!(SIGUSR1, 10);
        assert_eq!(SIGUSR2, 12);
    }

    #[test]
    fn test_exit_code_constants() {
        assert_eq!(EXIT_SIGTERM, 143);
        assert_eq!(EXIT_SIGINT, 130);
        assert_eq!(EXIT_SIGHUP, 129);
        assert_eq!(EXIT_SIGQUIT, 131);
        assert_eq!(EXIT_SIGPIPE, 141);
        assert_eq!(EXIT_SIGALRM, 142);
        assert_eq!(EXIT_SIGUSR1, 138);
        assert_eq!(EXIT_SIGUSR2, 140);
    }

    #[test]
    fn test_constants_match_catalog() {
        assert_eq!(lookup_signal("SIGTERM").unwrap().unix_number, SIGTERM);
        assert_eq!(lookup_signal("SIGINT").unwrap().unix_number, SIGINT);
        assert_eq!(lookup_signal("SIGHUP").unwrap().unix_number, SIGHUP);

        assert_eq!(lookup_signal("SIGTERM").unwrap().exit_code, EXIT_SIGTERM);
        assert_eq!(lookup_signal("SIGINT").unwrap().exit_code, EXIT_SIGINT);
        assert_eq!(lookup_signal("SIGHUP").unwrap().exit_code, EXIT_SIGHUP);
    }

    // -------------------------------------------------------------------------
    // SIGINT Double-Tap Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sigint_double_tap_fields() {
        let int = lookup_signal("SIGINT").expect("SIGINT should exist");
        assert_eq!(int.double_tap_window_seconds, Some(2));
        assert!(int.double_tap_message.is_some());
        assert!(int.double_tap_message.as_ref().unwrap().contains("2s"));
    }

    // -------------------------------------------------------------------------
    // SIGHUP Reload Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sighup_reload_fields() {
        let hup = lookup_signal("SIGHUP").expect("SIGHUP should exist");
        assert_eq!(hup.reload_strategy, Some("restart_based".to_string()));
        assert_eq!(hup.validation_required, Some(true));
    }

    // -------------------------------------------------------------------------
    // Windows Fallback Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_windows_fallback_sighup() {
        let hup = lookup_signal("SIGHUP").expect("SIGHUP should exist");
        let fallback = hup
            .windows_fallback
            .as_ref()
            .expect("SIGHUP should have fallback");
        assert_eq!(
            fallback.fallback_behavior,
            FallbackBehavior::HttpAdminEndpoint
        );
        assert!(fallback.operation_hint.contains("/admin/signal"));
    }

    #[test]
    fn test_windows_fallback_sigpipe() {
        let pipe = lookup_signal("SIGPIPE").expect("SIGPIPE should exist");
        let fallback = pipe
            .windows_fallback
            .as_ref()
            .expect("SIGPIPE should have fallback");
        assert_eq!(
            fallback.fallback_behavior,
            FallbackBehavior::ExceptionHandling
        );
    }

    #[test]
    fn test_windows_fallback_sigalrm() {
        let alrm = lookup_signal("SIGALRM").expect("SIGALRM should exist");
        let fallback = alrm
            .windows_fallback
            .as_ref()
            .expect("SIGALRM should have fallback");
        assert_eq!(fallback.fallback_behavior, FallbackBehavior::TimerApi);
    }

    // -------------------------------------------------------------------------
    // SignalBehavior Enum Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_signal_behavior_id() {
        assert_eq!(SignalBehavior::GracefulShutdown.id(), "graceful_shutdown");
        assert_eq!(
            SignalBehavior::GracefulShutdownWithDoubleTap.id(),
            "graceful_shutdown_with_double_tap"
        );
        assert_eq!(SignalBehavior::ReloadViaRestart.id(), "reload_via_restart");
        assert_eq!(SignalBehavior::ImmediateExit.id(), "immediate_exit");
        assert_eq!(SignalBehavior::Custom.id(), "custom");
        assert_eq!(SignalBehavior::ObserveOnly.id(), "observe_only");
    }

    #[test]
    fn test_signal_behavior_name() {
        assert_eq!(SignalBehavior::GracefulShutdown.name(), "Graceful Shutdown");
        assert_eq!(SignalBehavior::ImmediateExit.name(), "Immediate Exit");
    }
}
