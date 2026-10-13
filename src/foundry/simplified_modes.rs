//! Simplified exit-code mappings from the Crucible v0.4.19 catalog.
//!
//! Workhorses may collapse the full Fulmen exit-code space for novice CLIs
//! or limited environments. Mapping is **opt-in**: callers must request a
//! mode. Libraries MUST NOT remap codes implicitly.

use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

use super::{FoundryError, FoundryResult};

/// Embedded exit-codes catalog (same SSOT as [`crate::foundry::exit_codes`]).
const EXIT_CODES_YAML: &str =
    include_str!("../../config/crucible-rs/library/foundry/exit-codes.yaml");

/// Simplified mapping mode identifiers from the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SimplifiedMode {
    /// 0 success / 1 error / 2 usage error.
    Basic,
    /// 0 success plus severity 1–7.
    Severity,
}

impl SimplifiedMode {
    /// Catalog `id` (`basic` / `severity`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Severity => "severity",
        }
    }

    /// Parse a catalog mode id.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "basic" => Some(Self::Basic),
            "severity" => Some(Self::Severity),
            _ => None,
        }
    }
}

/// One catalog-defined simplified output code.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SimplifiedCode {
    /// Numeric simplified code.
    pub code: i32,
    /// Catalog `simplified_name`.
    pub name: String,
}

/// Metadata for one simplified mode.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SimplifiedModeInfo {
    /// Catalog id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Distinct simplified codes defined for this mode.
    pub codes: Vec<SimplifiedCode>,
}

#[derive(Debug, Deserialize)]
struct RawCatalog {
    simplified_modes: Vec<RawMode>,
}

#[derive(Debug, Deserialize)]
struct RawMode {
    id: String,
    name: String,
    description: String,
    mappings: Vec<RawMapping>,
}

#[derive(Debug, Deserialize)]
struct RawMapping {
    simplified_code: i32,
    simplified_name: String,
    maps_from: Vec<i32>,
}

struct Indexes {
    basic: HashMap<i32, i32>,
    severity: HashMap<i32, i32>,
    infos: Vec<SimplifiedModeInfo>,
}

static INDEXES: Lazy<Indexes> =
    Lazy::new(|| Indexes::load().expect("Failed to load simplified_modes from exit-codes catalog"));

impl Indexes {
    fn load() -> FoundryResult<Self> {
        let catalog: RawCatalog = serde_yaml::from_str(EXIT_CODES_YAML).map_err(|e| {
            FoundryError::LoadError(format!("Failed to parse simplified_modes: {e}"))
        })?;

        let mut basic = HashMap::new();
        let mut severity = HashMap::new();
        let mut infos = Vec::new();

        for mode in catalog.simplified_modes {
            let target = match mode.id.as_str() {
                "basic" => Some(&mut basic),
                "severity" => Some(&mut severity),
                _ => None,
            };
            if let Some(target) = target {
                for mapping in &mode.mappings {
                    for from in &mapping.maps_from {
                        if target.insert(*from, mapping.simplified_code).is_some() {
                            return Err(FoundryError::LoadError(format!(
                                "duplicate maps_from {from} in mode {}",
                                mode.id
                            )));
                        }
                    }
                }
            }
            let mut codes = Vec::new();
            for mapping in &mode.mappings {
                if !codes
                    .iter()
                    .any(|c: &SimplifiedCode| c.code == mapping.simplified_code)
                {
                    codes.push(SimplifiedCode {
                        code: mapping.simplified_code,
                        name: mapping.simplified_name.clone(),
                    });
                }
            }
            infos.push(SimplifiedModeInfo {
                id: mode.id,
                name: mode.name,
                description: mode.description,
                codes,
            });
        }

        Ok(Self {
            basic,
            severity,
            infos,
        })
    }

    fn map_for(&self, mode: SimplifiedMode) -> &HashMap<i32, i32> {
        match mode {
            SimplifiedMode::Basic => &self.basic,
            SimplifiedMode::Severity => &self.severity,
        }
    }
}

/// Map a Fulmen exit code into a simplified mode.
///
/// Returns `None` when the code is not in that mode's `maps_from` list
/// (unknown or unmapped). Does not default to 0 or 1.
///
/// # Examples
///
/// ```
/// use rsfulmen::foundry::simplified_modes::{map_to_simplified, SimplifiedMode};
///
/// assert_eq!(map_to_simplified(0, SimplifiedMode::Basic), Some(0));
/// assert_eq!(map_to_simplified(20, SimplifiedMode::Basic), Some(1));
/// assert_eq!(map_to_simplified(64, SimplifiedMode::Basic), Some(2));
/// assert_eq!(map_to_simplified(64, SimplifiedMode::Severity), Some(1));
/// assert_eq!(map_to_simplified(999, SimplifiedMode::Basic), None);
/// ```
pub fn map_to_simplified(code: i32, mode: SimplifiedMode) -> Option<i32> {
    INDEXES.map_for(mode).get(&code).copied()
}

/// Catalog identifiers for the available simplified modes.
pub fn list_simplified_modes() -> &'static [SimplifiedModeInfo] {
    &INDEXES.infos
}

/// Look up metadata for a simplified mode id.
pub fn get_simplified_mode_info(id: &str) -> Option<&'static SimplifiedModeInfo> {
    INDEXES.infos.iter().find(|info| info.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::{BTreeMap, BTreeSet};

    #[derive(Deserialize, PartialEq, Debug)]
    struct Snapshot {
        version: String,
        modes: BTreeMap<String, BTreeMap<String, Vec<i32>>>,
    }

    const SNAPSHOT: &str =
        include_str!("../../config/crucible-rs/library/foundry/simplified-modes.snapshot.json");

    #[test]
    fn maps_catalog_examples() {
        assert_eq!(map_to_simplified(0, SimplifiedMode::Basic), Some(0));
        assert_eq!(map_to_simplified(1, SimplifiedMode::Basic), Some(1));
        assert_eq!(map_to_simplified(64, SimplifiedMode::Basic), Some(2));
        assert_eq!(map_to_simplified(20, SimplifiedMode::Severity), Some(2));
        assert_eq!(map_to_simplified(64, SimplifiedMode::Severity), Some(1));
        assert_eq!(map_to_simplified(80, SimplifiedMode::Severity), Some(7));
    }

    #[test]
    fn unmapped_and_unknown_are_none() {
        assert_eq!(map_to_simplified(999, SimplifiedMode::Basic), None);
        assert_eq!(map_to_simplified(2, SimplifiedMode::Basic), None);
        assert_eq!(map_to_simplified(-1, SimplifiedMode::Severity), None);
        assert!(SimplifiedMode::from_id("disabled").is_none());
    }

    #[test]
    fn snapshot_parity_every_listed_code_maps() {
        let snap: Snapshot = serde_yaml::from_str(SNAPSHOT).expect("snapshot json");
        assert_eq!(snap.version, "v1.0.0");

        let mut computed: BTreeMap<String, BTreeMap<String, Vec<i32>>> = BTreeMap::new();
        for mode in [SimplifiedMode::Basic, SimplifiedMode::Severity] {
            let mut table: BTreeMap<i32, BTreeSet<i32>> = BTreeMap::new();
            for code in 0..=255 {
                if let Some(simplified) = map_to_simplified(code, mode) {
                    table.entry(simplified).or_default().insert(code);
                }
            }
            let rendered: BTreeMap<String, Vec<i32>> = table
                .into_iter()
                .map(|(simplified, sources)| {
                    (simplified.to_string(), sources.into_iter().collect())
                })
                .collect();
            computed.insert(mode.as_str().to_string(), rendered);
        }

        assert_eq!(
            computed.keys().cloned().collect::<BTreeSet<_>>(),
            snap.modes.keys().cloned().collect::<BTreeSet<_>>(),
            "mode set must match snapshot"
        );
        assert_eq!(
            computed, snap.modes,
            "per-mode mapping must match snapshot exactly"
        );
    }

    #[test]
    fn list_modes_matches_catalog_ids() {
        let ids: Vec<_> = list_simplified_modes()
            .iter()
            .map(|m| m.id.as_str())
            .collect();
        assert_eq!(ids, ["basic", "severity"]);
        assert!(get_simplified_mode_info("basic").is_some());
        assert!(get_simplified_mode_info("nope").is_none());
    }

    #[test]
    fn simplified_code_metadata_matches_catalog() {
        let catalog: RawCatalog =
            serde_yaml::from_str(EXIT_CODES_YAML).expect("simplified_modes catalog");
        for mode in catalog.simplified_modes {
            let info = get_simplified_mode_info(&mode.id).expect(&mode.id);
            assert_eq!(info.name, mode.name);
            assert_eq!(info.description, mode.description);
            let expected: Vec<(i32, String)> = mode
                .mappings
                .iter()
                .map(|m| (m.simplified_code, m.simplified_name.clone()))
                .fold(Vec::new(), |mut acc, pair| {
                    if !acc.iter().any(|(c, _)| *c == pair.0) {
                        acc.push(pair);
                    }
                    acc
                });
            let got: Vec<(i32, String)> = info
                .codes
                .iter()
                .map(|c| (c.code, c.name.clone()))
                .collect();
            assert_eq!(got, expected, "mode {}", mode.id);
        }
    }
}
