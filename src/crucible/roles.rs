//! Typed Role Catalog — Agentic Role Loading
//!
//! Provides typed access to Crucible agentic role definitions embedded from
//! `config/crucible-rs/agentic/roles/*.yaml`. All fields match the
//! `role-prompt.schema.json` specification.
//!
//! ## Example
//!
//! ```rust
//! use rsfulmen::crucible::roles::{load_role, list_role_slugs, load_role_catalog, RoleStatus};
//!
//! // Load a single role by slug
//! let role = load_role("devlead").expect("devlead should exist");
//! assert_eq!(role.name, "Development Lead");
//! assert_eq!(role.status, RoleStatus::Approved);
//!
//! // List all available slugs (sorted, no README)
//! let slugs = list_role_slugs();
//! assert!(slugs.iter().any(|s| s == "devlead"));
//!
//! // Load the full catalog keyed by slug
//! let catalog = load_role_catalog();
//! assert!(catalog.contains_key("devlead"));
//! ```

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Typed representation of an agentic role prompt.
///
/// Deserialized from YAML files under `config/crucible-rs/agentic/roles/`.
/// All fields match the `role-prompt.schema.json` specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePrompt {
    // --- Required fields ---
    /// Role identifier (lowercase alphanumeric, no hyphens).
    pub slug: String,
    /// Human-readable role name.
    pub name: String,
    /// One-line description of the role's purpose.
    pub description: String,
    /// Semantic version for role prompt evolution.
    pub version: String,
    /// Role prompt lifecycle status.
    pub status: RoleStatus,
    /// What this role covers — boundaries of responsibility.
    pub scope: Vec<String>,
    /// Specific tasks and duties.
    pub responsibilities: Vec<String>,
    /// When and to whom to escalate.
    pub escalates_to: Vec<RoleEscalation>,
    /// Explicit exclusions and out-of-scope items.
    pub does_not: Vec<String>,

    // --- Optional fields ---
    /// Role prompt author (role slug or human identifier).
    #[serde(default)]
    pub author: Option<String>,
    /// Role category for organization.
    #[serde(default)]
    pub category: Option<RoleCategory>,
    /// URL to base role this extends.
    #[serde(default)]
    pub extends: Option<String>,
    /// Process domains this role operates in (1–3).
    #[serde(default)]
    pub domains: Option<Vec<String>>,
    /// Searchable tags for the role.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    /// When to use this role, distinct from similar roles.
    #[serde(default)]
    pub context: Option<String>,
    /// Context engineering — shapes how the agent approaches problems.
    #[serde(default)]
    pub mindset: Option<RoleMindset>,
    /// Concrete examples of role application.
    #[serde(default)]
    pub examples: Option<Vec<RoleExample>>,
    /// Named checklists for common tasks.
    #[serde(default)]
    pub checklists: Option<HashMap<String, Vec<String>>>,
    /// Checklist of validations to perform before pushing changes.
    #[serde(default)]
    pub pre_push_checklist: Option<Vec<String>>,
    /// Documents that must be read before starting work in this role.
    #[serde(default)]
    pub required_reading: Option<RequiredReading>,
    /// Guidance on coordination with other roles.
    #[serde(default)]
    pub cross_role_note: Option<String>,
}

/// Role prompt lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleStatus {
    /// Not yet reviewed.
    Draft,
    /// Under review.
    Review,
    /// Accepted for use.
    Approved,
    /// No longer recommended.
    Deprecated,
}

/// Role category for organization.
///
/// Includes all values defined in `role-prompt.schema.json`. The `Unknown`
/// variant ensures forward compatibility — if crucible adds a category before
/// rsfulmen updates, deserialization succeeds rather than failing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleCategory {
    /// Agent-driven implementation roles.
    Agentic,
    /// Data and analytics roles.
    Analytics,
    /// CI/CD and pipeline automation roles.
    Automation,
    /// Advisory and consulting roles.
    Consulting,
    /// Governance and oversight roles.
    Governance,
    /// Marketing and messaging roles.
    Marketing,
    /// Code review and audit roles.
    Review,
    /// Forward-compatible catch-all for categories not yet in this enum.
    #[serde(other)]
    Unknown,
}

/// Escalation rule — when and to whom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleEscalation {
    /// Who to escalate to (role slug, "human maintainers", etc.).
    pub target: String,
    /// Conditions that trigger escalation.
    pub when: String,
}

/// Mindset context for role approach.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleMindset {
    /// Key questions to ask, thinking patterns.
    #[serde(default)]
    pub focus: Vec<String>,
    /// Guiding principles for this role.
    #[serde(default)]
    pub principles: Vec<String>,
}

/// Concrete example of role application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleExample {
    /// Example type classification.
    #[serde(rename = "type")]
    pub example_type: ExampleType,
    /// Example title.
    #[serde(default)]
    pub title: Option<String>,
    /// Example content (may be multi-line).
    pub content: String,
}

/// Example type classification.
///
/// The `Unknown` variant provides forward compatibility if crucible adds
/// example types beyond the current schema enum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExampleType {
    /// Commit message example.
    Commit,
    /// Code review example.
    Review,
    /// Checklist example.
    Checklist,
    /// Other example type.
    Other,
    /// Forward-compatible catch-all.
    #[serde(other)]
    Unknown,
}

/// Required reading specification for a role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredReading {
    /// Explanation of what should be read and why.
    #[serde(default)]
    pub description: Option<String>,
    /// Guidance on where to find required reading (project-specific).
    #[serde(default)]
    pub pattern: Option<String>,
    /// Specific files required for reading.
    #[serde(default)]
    pub files: Option<Vec<RequiredReadingFile>>,
}

/// A specific file required for reading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredReadingFile {
    /// Path to the file (relative to project root).
    pub path: String,
    /// Reason this file must be read.
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Catalog
// ---------------------------------------------------------------------------

/// Internal catalog holding the parsed roles and sorted slug list.
struct RoleCatalog {
    roles: HashMap<String, RolePrompt>,
    slugs: Vec<String>,
}

/// Schema-aligned slug regex: `^[a-z][a-z0-9]*$`.
fn is_valid_slug(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

static CATALOG: Lazy<RoleCatalog> = Lazy::new(|| {
    let mut roles = HashMap::new();

    for asset in super::list_config() {
        // Match agentic/roles/*.yaml, skip README.md and subdirectories.
        if !asset.path.starts_with("agentic/roles/") {
            continue;
        }
        if !asset.path.ends_with(".yaml") {
            continue;
        }

        let bytes = match super::open_config_bytes(asset.path) {
            Some(b) => b,
            None => continue,
        };

        let role: RolePrompt = match serde_yaml::from_slice(bytes) {
            Ok(r) => r,
            Err(e) => {
                // Log but don't panic — a malformed role should not crash the
                // process. In practice this should never happen since the YAMLs
                // are synced from a validated SSOT.
                eprintln!(
                    "rsfulmen: warning: failed to parse role YAML {}: {}",
                    asset.path, e
                );
                continue;
            }
        };

        roles.insert(role.slug.clone(), role);
    }

    let mut slugs: Vec<String> = roles.keys().cloned().collect();
    slugs.sort();

    RoleCatalog { roles, slugs }
});

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Load a single role by slug.
///
/// Returns `None` if the slug is invalid (does not match `^[a-z][a-z0-9]*$`)
/// or no role with that slug exists in the embedded catalog.
///
/// # Example
///
/// ```rust
/// use rsfulmen::crucible::roles::load_role;
///
/// let role = load_role("devlead").expect("devlead should exist");
/// assert_eq!(role.name, "Development Lead");
/// ```
pub fn load_role(slug: &str) -> Option<&'static RolePrompt> {
    if !is_valid_slug(slug) {
        return None;
    }
    CATALOG.roles.get(slug)
}

/// List all available role slugs (sorted alphabetically).
///
/// The returned list excludes non-YAML entries (e.g. `README.md`).
///
/// # Example
///
/// ```rust
/// use rsfulmen::crucible::roles::list_role_slugs;
///
/// let slugs = list_role_slugs();
/// assert!(slugs.contains(&&"devlead".to_string()));
/// ```
pub fn list_role_slugs() -> &'static [String] {
    &CATALOG.slugs
}

/// Load the full role catalog keyed by slug.
///
/// # Example
///
/// ```rust
/// use rsfulmen::crucible::roles::load_role_catalog;
///
/// let catalog = load_role_catalog();
/// assert!(catalog.contains_key("devlead"));
/// ```
pub fn load_role_catalog() -> &'static HashMap<String, RolePrompt> {
    &CATALOG.roles
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Core slugs that must be present in any valid Crucible role catalog.
    /// These are the original 8 roles from Crucible v0.4.4 and are stable.
    const CORE_SLUGS: &[&str] = &[
        "cicd", "dataeng", "devlead", "devrev", "entarch", "infoarch", "prodmktg", "secrev",
    ];

    #[test]
    fn test_load_role_devlead() {
        let role = load_role("devlead").expect("devlead should exist");
        assert_eq!(role.slug, "devlead");
        assert_eq!(role.name, "Development Lead");
        assert_eq!(role.status, RoleStatus::Approved);
        assert!(!role.responsibilities.is_empty());
        assert!(!role.escalates_to.is_empty());
        assert!(!role.does_not.is_empty());
    }

    #[test]
    fn test_list_role_slugs_excludes_readme() {
        let slugs = list_role_slugs();
        for s in slugs {
            assert_ne!(s, "README", "README must not appear as a slug");
            assert_ne!(s, "README.md", "README.md must not appear as a slug");
        }
    }

    #[test]
    fn test_list_role_slugs_contains_core_slugs() {
        let slugs = list_role_slugs();
        for core in CORE_SLUGS {
            assert!(
                slugs.iter().any(|s| s == core),
                "core slug '{}' missing from catalog",
                core
            );
        }
    }

    #[test]
    fn test_list_role_slugs_is_sorted() {
        let slugs = list_role_slugs();
        let mut sorted = slugs.to_vec();
        sorted.sort();
        assert_eq!(slugs, &sorted[..]);
    }

    #[test]
    fn test_load_role_catalog_has_core_roles() {
        let catalog = load_role_catalog();
        assert!(
            catalog.len() >= CORE_SLUGS.len(),
            "catalog should have at least {} roles, got {}",
            CORE_SLUGS.len(),
            catalog.len()
        );
        for core in CORE_SLUGS {
            assert!(
                catalog.contains_key(*core),
                "core role '{}' missing from catalog",
                core
            );
        }
    }

    #[test]
    fn test_load_role_unknown_returns_none() {
        assert!(load_role("nonexistent").is_none());
    }

    #[test]
    fn test_load_role_invalid_slug_returns_none() {
        assert!(load_role("INVALID").is_none());
        assert!(load_role("has-hyphen").is_none());
        assert!(load_role("1startsdigit").is_none());
        assert!(load_role("under_score").is_none());
        assert!(load_role("").is_none());
    }

    #[test]
    fn test_draft_roles_included_when_present() {
        // Draft roles are included — consumers filter by status field.
        let catalog = load_role_catalog();
        for role in catalog.values() {
            // Every role must have a valid status — the enum guarantees this
            // at deserialization time.
            let _ = &role.status;
        }
    }

    #[test]
    fn test_releng_exercises_new_schema_fields() {
        // releng.yaml is the canonical exerciser for pre_push_checklist,
        // required_reading (incl. files), and cross_role_note.
        if let Some(role) = load_role("releng") {
            assert!(
                role.pre_push_checklist.is_some(),
                "releng should have pre_push_checklist"
            );
            let rr = role
                .required_reading
                .as_ref()
                .expect("releng should have required_reading");
            assert!(
                rr.description.is_some(),
                "releng required_reading should have description"
            );
            assert!(
                rr.files.as_ref().is_some_and(|f| !f.is_empty()),
                "releng required_reading should have non-empty files list"
            );
            assert!(
                role.cross_role_note.is_some(),
                "releng should have cross_role_note"
            );
        }
        // releng may not exist if synced from older Crucible — skip gracefully.
    }

    #[test]
    fn test_slug_keyed_by_yaml_field() {
        let catalog = load_role_catalog();
        for (key, role) in catalog.iter() {
            assert_eq!(key, &role.slug, "catalog key must match slug field");
        }
    }

    #[test]
    fn test_all_slugs_match_schema_regex() {
        for slug in list_role_slugs() {
            assert!(
                is_valid_slug(slug),
                "slug '{}' does not match schema regex ^[a-z][a-z0-9]*$",
                slug
            );
        }
    }

    #[test]
    fn test_is_valid_slug() {
        assert!(is_valid_slug("devlead"));
        assert!(is_valid_slug("cicd"));
        assert!(is_valid_slug("qa"));

        assert!(!is_valid_slug(""));
        assert!(!is_valid_slug("A"));
        assert!(!is_valid_slug("1abc"));
        assert!(!is_valid_slug("has-hyphen"));
        assert!(!is_valid_slug("under_score"));
        assert!(!is_valid_slug("UPPER"));
    }
}
