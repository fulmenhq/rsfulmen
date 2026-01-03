//! Module Registry (Compliance)
//!
//! Provides read-only access to the Crucible module registries:
//!
//! - Platform modules: `config/taxonomy/library/platform-modules/.../modules.yaml`
//! - Foundry catalogs: `config/taxonomy/library/foundry-catalogs/.../catalogs.yaml`
//!
//! This module is primarily used to self-audit rsfulmen’s module coverage
//! against the SSOT registries.

use std::collections::{BTreeMap, HashSet};

use once_cell::sync::Lazy;
use serde::Deserialize;

/// Platform modules registry path within embedded Crucible config.
pub const PLATFORM_MODULES_REGISTRY_PATH: &str =
    "taxonomy/library/platform-modules/v1.1.0/modules.yaml";

/// Foundry catalogs registry path within embedded Crucible config.
pub const FOUNDRY_CATALOGS_REGISTRY_PATH: &str =
    "taxonomy/library/foundry-catalogs/v1.1.0/catalogs.yaml";

/// Errors returned by registry loading/validation.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ModuleRegistryError {
    /// A required embedded config asset was not found.
    #[error("embedded config asset not found: {0}")]
    AssetNotFound(String),

    /// YAML parsing failed.
    #[error("failed to parse registry yaml at {path}: {message}")]
    InvalidYaml {
        /// Registry path.
        path: String,
        /// Error details.
        message: String,
    },
}

/// Platform module tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleTier {
    /// Core tier.
    Core,
    /// Common tier.
    Common,
    /// Specialized tier.
    Specialized,
}

/// Module weight classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleWeight {
    /// Lightweight (serde-class only).
    Light,
    /// Heavy (brings heavier deps).
    Heavy,
}

/// Per-language implementation metadata.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LanguageEntry {
    /// Availability status (available/planned/etc).
    pub status: String,
    /// Package identifier for the language ecosystem.
    #[serde(default)]
    pub package: Option<String>,
    /// Version string.
    #[serde(default)]
    pub version: Option<String>,
    /// Brief implementation notes.
    #[serde(default)]
    pub implementation: Option<String>,
    /// Optional language-specific notes.
    #[serde(default)]
    pub notes: Option<String>,
}

/// A platform module registry entry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlatformModule {
    /// Module identifier.
    pub module_name: String,
    /// Tier.
    pub tier: ModuleTier,
    /// Dependency weight.
    pub weight: ModuleWeight,
    /// Default inclusion policy.
    pub default_inclusion: bool,
    /// Description.
    pub description: String,
    /// Status.
    pub status: String,
    /// Per-language entries.
    #[serde(default)]
    pub languages: BTreeMap<String, LanguageEntry>,
}

/// A foundry catalog registry entry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FoundryCatalog {
    /// Catalog identifier.
    pub catalog_name: String,
    /// Description.
    pub description: String,
    /// Dependency weight.
    pub weight: ModuleWeight,
    /// Default inclusion policy.
    pub default_inclusion: bool,
    /// Feature group mapping (foundry-core/foundry-patterns/etc).
    pub feature_group: String,
    /// Status.
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PlatformModulesRegistry {
    #[allow(dead_code)]
    version: String,
    modules: Vec<PlatformModule>,
}

#[derive(Debug, Clone, Deserialize)]
struct FoundryCatalogsRegistry {
    #[allow(dead_code)]
    version: String,
    catalogs: Vec<FoundryCatalog>,
}

static PLATFORM_REGISTRY: Lazy<Result<PlatformModulesRegistry, ModuleRegistryError>> =
    Lazy::new(load_platform_registry);

static FOUNDRY_REGISTRY: Lazy<Result<FoundryCatalogsRegistry, ModuleRegistryError>> =
    Lazy::new(load_foundry_registry);

fn load_platform_registry() -> Result<PlatformModulesRegistry, ModuleRegistryError> {
    let bytes =
        crate::crucible::open_config_bytes(PLATFORM_MODULES_REGISTRY_PATH).ok_or_else(|| {
            ModuleRegistryError::AssetNotFound(PLATFORM_MODULES_REGISTRY_PATH.to_string())
        })?;

    serde_yaml::from_slice(bytes).map_err(|e| ModuleRegistryError::InvalidYaml {
        path: PLATFORM_MODULES_REGISTRY_PATH.to_string(),
        message: e.to_string(),
    })
}

fn load_foundry_registry() -> Result<FoundryCatalogsRegistry, ModuleRegistryError> {
    let bytes =
        crate::crucible::open_config_bytes(FOUNDRY_CATALOGS_REGISTRY_PATH).ok_or_else(|| {
            ModuleRegistryError::AssetNotFound(FOUNDRY_CATALOGS_REGISTRY_PATH.to_string())
        })?;

    serde_yaml::from_slice(bytes).map_err(|e| ModuleRegistryError::InvalidYaml {
        path: FOUNDRY_CATALOGS_REGISTRY_PATH.to_string(),
        message: e.to_string(),
    })
}

/// List platform modules from the embedded registry.
pub fn list_platform_modules() -> Result<&'static [PlatformModule], ModuleRegistryError> {
    let registry = PLATFORM_REGISTRY.as_ref().map_err(Clone::clone)?;
    Ok(&registry.modules)
}

/// Look up a single platform module by name.
pub fn lookup_platform_module(
    name: &str,
) -> Result<Option<&'static PlatformModule>, ModuleRegistryError> {
    let modules = list_platform_modules()?;
    Ok(modules.iter().find(|m| m.module_name == name))
}

/// List foundry catalogs from the embedded registry.
pub fn list_foundry_catalogs() -> Result<&'static [FoundryCatalog], ModuleRegistryError> {
    let registry = FOUNDRY_REGISTRY.as_ref().map_err(Clone::clone)?;
    Ok(&registry.catalogs)
}

/// Look up a single foundry catalog by name.
pub fn lookup_foundry_catalog(
    name: &str,
) -> Result<Option<&'static FoundryCatalog>, ModuleRegistryError> {
    let catalogs = list_foundry_catalogs()?;
    Ok(catalogs.iter().find(|c| c.catalog_name == name))
}

/// Result of a registry validation pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryValidation {
    /// Platform modules required by the registry but missing in rsfulmen.
    pub missing_platform_modules: Vec<String>,
    /// Foundry catalogs required by the registry but missing in rsfulmen.
    pub missing_foundry_catalogs: Vec<String>,
}

impl RegistryValidation {
    fn ok() -> Self {
        Self {
            missing_platform_modules: Vec::new(),
            missing_foundry_catalogs: Vec::new(),
        }
    }

    /// True when no gaps were found.
    pub fn is_ok(&self) -> bool {
        self.missing_platform_modules.is_empty() && self.missing_foundry_catalogs.is_empty()
    }
}

/// Validate rsfulmen’s module coverage against the embedded registries.
///
/// For v0.1.0 we validate:
/// - Platform modules with `languages.rust.status == "available"`.
/// - Foundry catalogs with `default_inclusion: true`.
pub fn validate_registry() -> Result<RegistryValidation, ModuleRegistryError> {
    let mut out = RegistryValidation::ok();

    let required_platform = required_platform_modules_for_rust()?;
    let implemented_platform: HashSet<&'static str> =
        rsfulmen_platform_modules().into_iter().collect();

    for module_name in required_platform {
        if !implemented_platform.contains(module_name.as_str()) {
            out.missing_platform_modules.push(module_name);
        }
    }

    let required_catalogs = required_foundry_catalogs()?;
    let implemented_catalogs: HashSet<&'static str> =
        rsfulmen_foundry_catalogs().into_iter().collect();

    for catalog_name in required_catalogs {
        if !implemented_catalogs.contains(catalog_name.as_str()) {
            out.missing_foundry_catalogs.push(catalog_name);
        }
    }

    out.missing_platform_modules.sort();
    out.missing_foundry_catalogs.sort();

    Ok(out)
}

fn required_platform_modules_for_rust() -> Result<Vec<String>, ModuleRegistryError> {
    let modules = list_platform_modules()?;

    let mut required = Vec::new();
    for m in modules {
        if m.status != "active" {
            continue;
        }

        if m.tier != ModuleTier::Core && m.tier != ModuleTier::Common {
            continue;
        }

        let rust = m.languages.get("rust");
        if let Some(rust) = rust {
            if rust.status == "available" {
                required.push(m.module_name.clone());
            }
        }
    }

    Ok(required)
}

fn required_foundry_catalogs() -> Result<Vec<String>, ModuleRegistryError> {
    let catalogs = list_foundry_catalogs()?;

    let mut required = Vec::new();
    for c in catalogs {
        if c.status != "active" {
            continue;
        }

        if c.default_inclusion {
            required.push(c.catalog_name.clone());
        }
    }

    Ok(required)
}

// rsfulmen’s implementation mapping.
//
// These names must match the SSOT registries.
fn rsfulmen_platform_modules() -> Vec<&'static str> {
    vec!["schema", "foundry", "similarity", "signals"]
}

fn rsfulmen_foundry_catalogs() -> Vec<&'static str> {
    vec![
        "countries",
        "http-statuses",
        "mime-types",
        "exit-codes",
        "signals",
        // Optional/opt-in:
        "patterns",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_registry_has_no_gaps_for_v0_1_0() {
        let report = validate_registry().expect("registry validation should run");
        assert!(
            report.is_ok(),
            "registry gaps: platform={:?} catalogs={:?}",
            report.missing_platform_modules,
            report.missing_foundry_catalogs
        );
    }
}
