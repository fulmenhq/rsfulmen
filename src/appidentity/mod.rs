//! Application identity discovery and loading.
//!
//! This module provides a canonical way to load application metadata from
//! `.fulmen/app.yaml`.
//!
//! Discovery behavior:
//! - If `FULMEN_APP_IDENTITY_FILE` is set, load that file directly.
//! - Otherwise, search upward from current dir (`load`) or a provided dir (`load_from`).
//!
//! The environment override is recommended for production binaries and CI jobs where
//! current working directory is not guaranteed to be inside the repository tree.

use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const APP_IDENTITY_ENV: &str = "FULMEN_APP_IDENTITY_FILE";
const APP_IDENTITY_RELATIVE_PATH: &str = ".fulmen/app.yaml";

/// Application identity metadata loaded from `.fulmen/app.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Identity {
    /// Application name (required, e.g., "myservice").
    pub name: String,
    /// Application version (optional, e.g., "1.2.3").
    pub version: Option<String>,
    /// Organization or team (optional, e.g., "platform-team").
    pub org: Option<String>,
    /// Repository URL (optional).
    pub repo: Option<String>,
    /// Description (optional).
    pub description: Option<String>,
    /// Additional fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct RawIdentity {
    pub name: Option<String>,
    pub version: Option<String>,
    pub org: Option<String>,
    pub repo: Option<String>,
    pub description: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Errors that can occur when loading app identity.
#[derive(Debug, thiserror::Error)]
pub enum AppIdentityError {
    /// `.fulmen/app.yaml` not found in search path.
    #[error("app identity not found: searched from {search_root}")]
    NotFound {
        /// Root directory where upward search started.
        search_root: PathBuf,
    },

    /// File found but contains invalid YAML.
    #[error("malformed app identity at {path}: {source}")]
    Malformed {
        /// Path to malformed file.
        path: PathBuf,
        /// YAML parsing error source.
        source: serde_yaml::Error,
    },

    /// File parsed but fails semantic validation.
    #[error("invalid app identity at {path}: {reason}")]
    Invalid {
        /// Path to invalid file.
        path: PathBuf,
        /// Human-readable validation reason.
        reason: String,
    },

    /// I/O error reading the file.
    #[error("failed to read app identity at {path}: {source}")]
    IoError {
        /// Path that failed to read.
        path: PathBuf,
        /// I/O error source.
        source: std::io::Error,
    },

    /// Could not determine current directory for discovery.
    #[error("failed to determine current directory: {source}")]
    CurrentDir {
        /// I/O error source.
        source: std::io::Error,
    },
}

/// Load app identity by searching upward from current directory.
///
/// If `FULMEN_APP_IDENTITY_FILE` is set, that file is used directly and
/// discovery is skipped.
///
/// # Examples
///
/// ```rust,no_run
/// use rsfulmen::appidentity::load;
///
/// let identity = load()?;
/// println!("app={}", identity.name);
/// # Ok::<(), rsfulmen::appidentity::AppIdentityError>(())
/// ```
pub fn load() -> Result<Identity, AppIdentityError> {
    if let Some(path) = env_override_path() {
        return load_file(&path);
    }

    let cwd = std::env::current_dir().map_err(|source| AppIdentityError::CurrentDir { source })?;
    load_from(&cwd)
}

/// Load app identity by searching upward from a specific directory.
///
/// If `FULMEN_APP_IDENTITY_FILE` is set, that file is used directly and
/// discovery is skipped.
///
/// # Examples
///
/// ```rust,no_run
/// use rsfulmen::appidentity::load_from;
/// use std::path::Path;
///
/// let identity = load_from(Path::new("."))?;
/// println!("app={}", identity.name);
/// # Ok::<(), rsfulmen::appidentity::AppIdentityError>(())
/// ```
pub fn load_from(start_dir: &Path) -> Result<Identity, AppIdentityError> {
    if let Some(path) = env_override_path() {
        return load_file(&path);
    }

    let start = if start_dir.is_absolute() {
        start_dir.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|source| AppIdentityError::CurrentDir { source })?
            .join(start_dir)
    };

    let mut current = start.clone();
    loop {
        let candidate = current.join(APP_IDENTITY_RELATIVE_PATH);
        if candidate.is_file() {
            return load_file(&candidate);
        }

        let Some(parent) = current.parent() else {
            break;
        };

        if parent == current {
            break;
        }
        current = parent.to_path_buf();
    }

    Err(AppIdentityError::NotFound { search_root: start })
}

/// Load app identity from an explicit file path (no discovery).
///
/// # Examples
///
/// ```rust,no_run
/// use rsfulmen::appidentity::load_file;
/// use std::path::Path;
///
/// let identity = load_file(Path::new(".fulmen/app.yaml"))?;
/// println!("app={}", identity.name);
/// # Ok::<(), rsfulmen::appidentity::AppIdentityError>(())
/// ```
pub fn load_file(path: &Path) -> Result<Identity, AppIdentityError> {
    let content = fs::read_to_string(path).map_err(|source| AppIdentityError::IoError {
        path: path.to_path_buf(),
        source,
    })?;

    let raw: RawIdentity =
        serde_yaml::from_str(&content).map_err(|source| AppIdentityError::Malformed {
            path: path.to_path_buf(),
            source,
        })?;
    let identity = Identity {
        name: raw.name.unwrap_or_default(),
        version: raw.version,
        org: raw.org,
        repo: raw.repo,
        description: raw.description,
        extra: raw.extra,
    };

    validate_with_path(&identity, path)?;
    Ok(identity)
}

/// Validate an [`Identity`] instance.
///
/// Rules:
/// - `name` must be non-empty
/// - `name` must match `[a-z][a-z0-9-]*`
///
/// # Examples
///
/// ```rust
/// use rsfulmen::appidentity::{validate, Identity};
/// use std::collections::BTreeMap;
///
/// let identity = Identity {
///     name: "my-service".to_string(),
///     version: None,
///     org: None,
///     repo: None,
///     description: None,
///     extra: BTreeMap::new(),
/// };
///
/// validate(&identity)?;
/// # Ok::<(), rsfulmen::appidentity::AppIdentityError>(())
/// ```
pub fn validate(identity: &Identity) -> Result<(), AppIdentityError> {
    validate_with_path(identity, Path::new(APP_IDENTITY_RELATIVE_PATH))
}

fn validate_with_path(identity: &Identity, path: &Path) -> Result<(), AppIdentityError> {
    if identity.name.trim().is_empty() {
        return Err(AppIdentityError::Invalid {
            path: path.to_path_buf(),
            reason: "missing required field: name".to_string(),
        });
    }

    if !is_valid_name(&identity.name) {
        return Err(AppIdentityError::Invalid {
            path: path.to_path_buf(),
            reason: format!(
                "invalid name '{}': must match [a-z][a-z0-9-]*",
                identity.name
            ),
        });
    }

    Ok(())
}

fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_lowercase() {
        return false;
    }

    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn env_override_path() -> Option<PathBuf> {
    let raw = std::env::var_os(APP_IDENTITY_ENV)?;
    let path = PathBuf::from(raw);
    if path.as_os_str().is_empty() {
        return None;
    }
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let mut path = std::env::temp_dir();
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be after epoch")
                .as_nanos();
            path.push(format!(
                "rsfulmen-appidentity-{}-{}",
                std::process::id(),
                nanos
            ));
            fs::create_dir_all(&path).expect("failed to create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &Path) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = std::env::var_os(key);
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn write_identity(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent dir");
        }
        fs::write(path, content).expect("failed to write identity file");
    }

    #[test]
    fn test_load_file_valid() {
        let tmp = TestDir::new();
        let file = tmp.path().join(".fulmen/app.yaml");
        write_identity(
            &file,
            "name: myservice\nversion: 1.2.3\norg: platform\nrepo: https://example.com/repo\ndescription: sample\n",
        );

        let id = load_file(&file).expect("should load identity");
        assert_eq!(id.name, "myservice");
        assert_eq!(id.version.as_deref(), Some("1.2.3"));
        assert_eq!(id.org.as_deref(), Some("platform"));
        assert_eq!(id.repo.as_deref(), Some("https://example.com/repo"));
        assert_eq!(id.description.as_deref(), Some("sample"));
    }

    #[test]
    fn test_load_file_minimal() {
        let tmp = TestDir::new();
        let file = tmp.path().join(".fulmen/app.yaml");
        write_identity(&file, "name: myapp\n");

        let id = load_file(&file).expect("should load minimal identity");
        assert_eq!(id.name, "myapp");
        assert!(id.version.is_none());
        assert!(id.org.is_none());
        assert!(id.repo.is_none());
        assert!(id.description.is_none());
    }

    #[test]
    fn test_load_file_with_extra_fields() {
        let tmp = TestDir::new();
        let file = tmp.path().join(".fulmen/app.yaml");
        write_identity(&file, "name: myapp\nteam: core\nlabels:\n  env: dev\n");

        let id = load_file(&file).expect("should load with extra fields");
        assert_eq!(id.name, "myapp");
        assert_eq!(
            id.extra.get("team"),
            Some(&Value::String("core".to_string()))
        );
        assert!(id.extra.contains_key("labels"));
    }

    #[test]
    fn test_load_from_discovers_in_current_dir() {
        let _lock = env_lock().lock().expect("env lock should not be poisoned");
        let _env = EnvVarGuard::unset(APP_IDENTITY_ENV);
        let tmp = TestDir::new();
        let file = tmp.path().join(".fulmen/app.yaml");
        write_identity(&file, "name: current-app\n");

        let id = load_from(tmp.path()).expect("should discover in current dir");
        assert_eq!(id.name, "current-app");
    }

    #[test]
    fn test_load_from_discovers_in_parent_dir() {
        let _lock = env_lock().lock().expect("env lock should not be poisoned");
        let _env = EnvVarGuard::unset(APP_IDENTITY_ENV);
        let tmp = TestDir::new();
        let file = tmp.path().join(".fulmen/app.yaml");
        write_identity(&file, "name: parent-app\n");

        let nested = tmp.path().join("child/grandchild");
        fs::create_dir_all(&nested).expect("failed to create nested dir");

        let id = load_from(&nested).expect("should discover in parent dir");
        assert_eq!(id.name, "parent-app");
    }

    #[test]
    fn test_load_from_not_found() {
        let _lock = env_lock().lock().expect("env lock should not be poisoned");
        let _env = EnvVarGuard::unset(APP_IDENTITY_ENV);
        let tmp = TestDir::new();

        let err = load_from(tmp.path()).expect_err("should return not found");
        match err {
            AppIdentityError::NotFound { search_root } => {
                assert_eq!(search_root, tmp.path().to_path_buf())
            }
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn test_load_file_malformed_yaml() {
        let tmp = TestDir::new();
        let file = tmp.path().join(".fulmen/app.yaml");
        write_identity(&file, "name: [bad\n");

        let err = load_file(&file).expect_err("should return malformed");
        assert!(matches!(err, AppIdentityError::Malformed { .. }));
    }

    #[test]
    fn test_load_file_missing_name() {
        let tmp = TestDir::new();
        let file = tmp.path().join(".fulmen/app.yaml");
        write_identity(&file, "version: 1.0.0\n");

        let err = load_file(&file).expect_err("should return invalid");
        match err {
            AppIdentityError::Invalid { reason, .. } => {
                assert!(reason.contains("missing required field: name"))
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn test_validate_good_names() {
        for name in ["myapp", "my-service", "a123"] {
            let id = Identity {
                name: name.to_string(),
                version: None,
                org: None,
                repo: None,
                description: None,
                extra: BTreeMap::new(),
            };
            validate(&id).expect("valid name should pass");
        }
    }

    #[test]
    fn test_validate_bad_names() {
        for name in ["", "123abc", "My-App", "my_app"] {
            let id = Identity {
                name: name.to_string(),
                version: None,
                org: None,
                repo: None,
                description: None,
                extra: BTreeMap::new(),
            };
            let err = validate(&id).expect_err("invalid name should fail");
            assert!(matches!(err, AppIdentityError::Invalid { .. }));
        }
    }

    #[test]
    fn test_identity_roundtrip_yaml() {
        let mut extra = BTreeMap::new();
        extra.insert("team".to_string(), Value::String("core".to_string()));

        let original = Identity {
            name: "my-service".to_string(),
            version: Some("1.2.3".to_string()),
            org: Some("platform".to_string()),
            repo: None,
            description: Some("desc".to_string()),
            extra,
        };

        let encoded = serde_yaml::to_string(&original).expect("yaml encode should succeed");
        let decoded: Identity = serde_yaml::from_str(&encoded).expect("yaml decode should succeed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_identity_roundtrip_json() {
        let mut extra = BTreeMap::new();
        extra.insert("team".to_string(), Value::String("core".to_string()));

        let original = Identity {
            name: "my-service".to_string(),
            version: Some("1.2.3".to_string()),
            org: Some("platform".to_string()),
            repo: None,
            description: Some("desc".to_string()),
            extra,
        };

        let encoded = serde_json::to_string(&original).expect("json encode should succeed");
        let decoded: Identity = serde_json::from_str(&encoded).expect("json decode should succeed");
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_env_override_used_by_load_from() {
        let _lock = env_lock().lock().expect("env lock should not be poisoned");
        let tmp = TestDir::new();
        let override_file = tmp.path().join("override.yaml");
        write_identity(&override_file, "name: env-override\n");
        let _env = EnvVarGuard::set(APP_IDENTITY_ENV, &override_file);

        let start = tmp.path().join("nested/path");
        fs::create_dir_all(&start).expect("failed to create nested start");

        let id = load_from(&start).expect("override should be used");
        assert_eq!(id.name, "env-override");
    }

    #[test]
    fn test_env_override_invalid_path_returns_io_error() {
        let _lock = env_lock().lock().expect("env lock should not be poisoned");
        let tmp = TestDir::new();
        let missing = tmp.path().join("missing.yaml");
        let _env = EnvVarGuard::set(APP_IDENTITY_ENV, &missing);

        let err = load().expect_err("missing override path should fail");
        assert!(matches!(err, AppIdentityError::IoError { .. }));
    }
}
