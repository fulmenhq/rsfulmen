//! The foundry-core catalogs load with yamlprims. These tests confirm that
//! yamlprims and serde_yaml produce the same data for every embedded catalog.
#![cfg(feature = "foundry-core")]

const CATALOGS: &[(&str, &str)] = &[
    (
        "exit-codes",
        include_str!("../config/crucible-rs/library/foundry/exit-codes.yaml"),
    ),
    (
        "country-codes",
        include_str!("../config/crucible-rs/library/foundry/country-codes.yaml"),
    ),
    (
        "http-statuses",
        include_str!("../config/crucible-rs/library/foundry/http-statuses.yaml"),
    ),
    (
        "signals",
        include_str!("../config/crucible-rs/library/foundry/signals.yaml"),
    ),
];

#[test]
fn foundry_core_catalogs_match_serde_yaml() {
    for (name, source) in CATALOGS {
        let expected: serde_json::Value =
            serde_yaml::from_str(source).unwrap_or_else(|e| panic!("{name}: serde_yaml: {e}"));
        let actual: serde_json::Value =
            yamlprims::from_str(source).unwrap_or_else(|e| panic!("{name}: yamlprims: {e}"));
        assert_eq!(
            actual, expected,
            "{name}: catalog data differs between parsers"
        );
    }
}
