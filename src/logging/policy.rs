//! Organizational logging policy (Crucible `logging-policy.schema.json`).
//!
//! Evaluated at logger initialization, not per event. Strict mode is the
//! caller's choice (`enforce_or_log`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::Profile;

/// Profiles named in the Crucible policy schema (includes ENTERPRISE/CUSTOM
/// even though [`Profile`] currently exposes Simple/Structured only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyProfile {
    /// CLI / simple console.
    Simple,
    /// JSON structured sinks.
    Structured,
    /// Enterprise (middleware + throttle + policy). Not a [`Profile`] variant yet.
    Enterprise,
    /// Escape hatch.
    Custom,
}

impl PolicyProfile {
    /// Map the runtime [`Profile`] onto a policy token.
    pub fn from_runtime(profile: Profile) -> Self {
        match profile {
            Profile::Simple => Self::Simple,
            Profile::Structured => Self::Structured,
        }
    }
}

impl std::fmt::Display for PolicyProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Simple => "SIMPLE",
            Self::Structured => "STRUCTURED",
            Self::Enterprise => "ENTERPRISE",
            Self::Custom => "CUSTOM",
        })
    }
}

/// Optional context for [`LoggingPolicy::validate`].
#[derive(Debug, Clone, Default)]
pub struct PolicyValidationOptions {
    /// Application class (`workhorse`, `cli`, …).
    pub app_type: Option<String>,
    /// Deployment environment (`production`, `staging`, …).
    pub environment: Option<String>,
}

/// Parsed logging policy document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingPolicy {
    /// Policy document version.
    #[serde(default)]
    pub version: Option<String>,
    /// Profiles the organization permits.
    pub allowed_profiles: Vec<PolicyProfile>,
    /// Required profiles by application type.
    #[serde(default)]
    pub required_profiles: BTreeMap<String, Vec<PolicyProfile>>,
    /// Allowed profiles by environment.
    #[serde(default)]
    pub environment_rules: BTreeMap<String, Vec<PolicyProfile>>,
}

/// Policy load or validation failure.
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    /// File could not be read.
    #[error("policy file not found or unreadable: {0}")]
    Io(PathBuf),
    /// YAML was invalid.
    #[error("invalid policy YAML in {path}: {message}")]
    Parse {
        /// Policy path.
        path: PathBuf,
        /// Parser message.
        message: String,
    },
    /// Document missing required fields.
    #[error("invalid policy {path}: {message}")]
    Invalid {
        /// Policy path.
        path: PathBuf,
        /// Why the document is invalid.
        message: String,
    },
    /// Profile rejected by the policy.
    #[error("{0}")]
    Denied(String),
}

impl LoggingPolicy {
    /// Load and parse a YAML policy file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, PolicyError> {
        let path = path.as_ref();
        let text =
            std::fs::read_to_string(path).map_err(|_| PolicyError::Io(path.to_path_buf()))?;
        let policy: Self = serde_yaml::from_str(&text).map_err(|err| PolicyError::Parse {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?;
        if policy.allowed_profiles.is_empty() {
            return Err(PolicyError::Invalid {
                path: path.to_path_buf(),
                message: "allowedProfiles must be a non-empty array".to_string(),
            });
        }
        Ok(policy)
    }

    /// Return `Ok` when `profile` is allowed for the optional context.
    pub fn validate(
        &self,
        profile: PolicyProfile,
        options: &PolicyValidationOptions,
    ) -> Result<(), PolicyError> {
        if !self.allowed_profiles.contains(&profile) {
            return Err(PolicyError::Denied(format!(
                "profile {profile} is not in allowedProfiles {:?}",
                self.allowed_profiles
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )));
        }
        if let Some(env) = options.environment.as_deref() {
            if let Some(allowed) = self.environment_rules.get(env) {
                if !allowed.contains(&profile) {
                    return Err(PolicyError::Denied(format!(
                        "profile {profile} is not allowed in environment {env}"
                    )));
                }
            }
        }
        if let Some(app) = options.app_type.as_deref() {
            if let Some(required) = self.required_profiles.get(app) {
                if !required.contains(&profile) {
                    return Err(PolicyError::Denied(format!(
                        "app type {app} requires one of {:?}, not {profile}",
                        required.iter().map(ToString::to_string).collect::<Vec<_>>()
                    )));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_and_validate_policy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("logging-policy.yaml");
        let mut file = std::fs::File::create(&path).unwrap();
        write!(
            file,
            "allowedProfiles: [SIMPLE, STRUCTURED]\n\
             environmentRules:\n\
               production: [STRUCTURED]\n\
             requiredProfiles:\n\
               workhorse: [STRUCTURED]\n"
        )
        .unwrap();

        let policy = LoggingPolicy::load(&path).unwrap();
        policy
            .validate(
                PolicyProfile::Structured,
                &PolicyValidationOptions {
                    app_type: Some("workhorse".into()),
                    environment: Some("production".into()),
                },
            )
            .unwrap();
        assert!(policy
            .validate(PolicyProfile::Simple, &PolicyValidationOptions::default())
            .is_ok());
        assert!(policy
            .validate(
                PolicyProfile::Simple,
                &PolicyValidationOptions {
                    app_type: None,
                    environment: Some("production".into()),
                }
            )
            .is_err());
    }

    #[test]
    fn missing_file_is_io_error() {
        let err = LoggingPolicy::load("/no/such/logging-policy.yaml").unwrap_err();
        assert!(matches!(err, PolicyError::Io(_)));
    }
}
