//! Public configuration merge and deep-copy primitives.
//!
//! Semantics match the Enterprise Three-Layer Configuration Standard:
//!
//! - **Mappings**: last-writer-wins per key; nested mappings merge recursively
//! - **Sequences**: replaced entirely (never concatenated or element-merged)
//! - **Null**: an explicit `null` overlay value deletes that key from a mapping
//! - **Scalars / type changes**: overlay replaces the base value
//!
//! Both inputs are borrowed. The result is a newly owned tree; mutating it
//! cannot alias back into `base` or `overlay`.

use serde_yaml::{Mapping, Value};

/// Deep-merge `overlay` onto `base` without mutating either input.
///
/// Root-level `null` is a last-writer-wins overwrite (the result is `null`).
/// `null` deletes a key only when it appears as a mapping value.
///
/// # Examples
///
/// ```
/// use rsfulmen::config::merge;
/// use serde_yaml::Value;
///
/// let base: Value = serde_yaml::from_str("a: 1\nnested:\n  x: 1\n  y: 2\n").unwrap();
/// let overlay: Value = serde_yaml::from_str("nested:\n  y: 9\n  z: 3\nb: null\n").unwrap();
/// let merged = merge(&base, &overlay);
///
/// let expected: Value = serde_yaml::from_str("a: 1\nnested:\n  x: 1\n  y: 9\n  z: 3\n").unwrap();
/// assert_eq!(merged, expected);
/// ```
pub fn merge(base: &Value, overlay: &Value) -> Value {
    match (base, overlay) {
        (Value::Mapping(base_map), Value::Mapping(overlay_map)) => {
            Value::Mapping(merge_mappings(base_map, overlay_map))
        }
        (_, overlay) => deep_copy(overlay),
    }
}

/// Return a deep copy of `value`.
///
/// `serde_yaml::Value` is an owned tree, so this is equivalent to cloning each
/// node. The function exists as the public contract matching gofulmen's
/// `DeepCopyMap`: callers can rely on the copy not aliasing the source.
///
/// # Examples
///
/// ```
/// use rsfulmen::config::deep_copy;
/// use serde_yaml::Value;
///
/// let src: Value = serde_yaml::from_str("nested:\n  k: original\n").unwrap();
/// let mut copy = deep_copy(&src);
/// if let Value::Mapping(map) = &mut copy {
///     map.insert(Value::String("extra".into()), Value::Bool(true));
/// }
/// let expected: Value = serde_yaml::from_str("nested:\n  k: original\n").unwrap();
/// assert_eq!(src, expected);
/// ```
pub fn deep_copy(value: &Value) -> Value {
    match value {
        Value::Mapping(map) => Value::Mapping(deep_copy_mapping(map)),
        Value::Sequence(seq) => Value::Sequence(seq.iter().map(deep_copy).collect()),
        Value::Tagged(tagged) => Value::Tagged(Box::new(serde_yaml::value::TaggedValue {
            tag: tagged.tag.clone(),
            value: deep_copy(&tagged.value),
        })),
        other => other.clone(),
    }
}

fn merge_mappings(base: &Mapping, overlay: &Mapping) -> Mapping {
    let mut result = deep_copy_mapping(base);
    for (key, overlay_value) in overlay {
        if overlay_value.is_null() {
            result.remove(key);
            continue;
        }

        match (result.get(key), overlay_value) {
            (Some(Value::Mapping(base_child)), Value::Mapping(overlay_child)) => {
                let merged = merge_mappings(base_child, overlay_child);
                result.insert(key.clone(), Value::Mapping(merged));
            }
            _ => {
                result.insert(key.clone(), deep_copy(overlay_value));
            }
        }
    }
    result
}

fn deep_copy_mapping(src: &Mapping) -> Mapping {
    let mut out = Mapping::with_capacity(src.len());
    for (key, value) in src {
        out.insert(key.clone(), deep_copy(value));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yaml(s: &str) -> Value {
        serde_yaml::from_str(s).expect("test yaml should parse")
    }

    fn mapping(value: &Value) -> &Mapping {
        value.as_mapping().expect("expected mapping")
    }

    fn mapping_mut(value: &mut Value) -> &mut Mapping {
        value.as_mapping_mut().expect("expected mapping")
    }

    #[test]
    fn merge_scalars_last_writer_wins() {
        assert_eq!(merge(&yaml("1"), &yaml("2")), yaml("2"));
        assert_eq!(merge(&yaml("true"), &yaml("false")), yaml("false"));
        assert_eq!(merge(&yaml("keep"), &yaml("take")), yaml("take"));
    }

    #[test]
    fn merge_nested_objects_recursively() {
        let base = yaml(
            r#"
keep: base
nested:
  x: 1
  y: 2
"#,
        );
        let overlay = yaml(
            r#"
nested:
  y: 9
  z: 3
added: yes
"#,
        );
        let merged = merge(&base, &overlay);
        assert_eq!(
            merged,
            yaml(
                r#"
keep: base
nested:
  x: 1
  y: 9
  z: 3
added: yes
"#
            )
        );
    }

    #[test]
    fn merge_sequences_replace_not_concat() {
        let base = yaml("list: [1, 2, {k: base}]\n");
        let overlay = yaml("list: [9]\n");
        let merged = merge(&base, &overlay);
        assert_eq!(merged, yaml("list: [9]\n"));
        // Overlay sequence of mappings is not element-merged.
        let base = yaml("items: [{a: 1, b: 2}, {c: 3}]\n");
        let overlay = yaml("items: [{a: 9}]\n");
        assert_eq!(merge(&base, &overlay), yaml("items: [{a: 9}]\n"));
    }

    #[test]
    fn merge_null_deletes_mapping_key() {
        let base = yaml("a: 1\nb: {nested: keep}\nc: 3\n");
        let overlay = yaml("b: null\n");
        assert_eq!(merge(&base, &overlay), yaml("a: 1\nc: 3\n"));
    }

    #[test]
    fn merge_nested_null_deletes_only_that_key() {
        let base = yaml("object: {remove: value, keep: value}\n");
        let overlay = yaml("object: {remove: null, added: value}\n");
        assert_eq!(
            merge(&base, &overlay),
            yaml("object: {keep: value, added: value}\n")
        );
    }

    #[test]
    fn merge_root_null_overwrites() {
        let base = yaml("a: 1\n");
        assert_eq!(merge(&base, &Value::Null), Value::Null);
        assert_eq!(merge(&Value::Null, &yaml("a: 1\n")), yaml("a: 1\n"));
        assert_eq!(merge(&Value::Null, &Value::Null), Value::Null);
    }

    #[test]
    fn merge_empty_overlay_mapping_is_identity_copy() {
        let base = yaml("a: 1\nnested: {x: 2}\n");
        let merged = merge(&base, &Value::Mapping(Mapping::new()));
        assert_eq!(merged, base);
    }

    #[test]
    fn merge_type_change_replaces() {
        let map = yaml("k: {inner: 1}\n");
        let scalar = yaml("k: 7\n");
        assert_eq!(merge(&map, &scalar), yaml("k: 7\n"));
        assert_eq!(merge(&scalar, &map), yaml("k: {inner: 1}\n"));
        assert_eq!(merge(&yaml("[1, 2]"), &yaml("{a: 1}")), yaml("{a: 1}"));
    }

    #[test]
    fn merge_does_not_alias_inputs() {
        let base = yaml(
            r#"
keep: base
object:
  base: kept
  shared:
    - source: base
array:
  - base
"#,
        );
        let overlay = yaml(
            r#"
object:
  overlay:
    - source: overlay
array:
  - overlay
"#,
        );
        let mut merged = merge(&base, &overlay);

        let object = mapping_mut(mapping_mut(&mut merged).get_mut("object").unwrap());
        object.insert(Value::String("base".into()), yaml("changed"));
        let shared = object.get_mut("shared").unwrap().as_sequence_mut().unwrap();
        *shared[0]
            .as_mapping_mut()
            .unwrap()
            .get_mut("source")
            .unwrap() = yaml("changed");
        let overlay_seq = object
            .get_mut("overlay")
            .unwrap()
            .as_sequence_mut()
            .unwrap();
        *overlay_seq[0]
            .as_mapping_mut()
            .unwrap()
            .get_mut("source")
            .unwrap() = yaml("changed");
        mapping_mut(&mut merged)
            .get_mut("array")
            .unwrap()
            .as_sequence_mut()
            .unwrap()[0] = yaml("changed");

        assert_eq!(
            mapping(&base)["object"]["base"],
            yaml("kept"),
            "base object must stay intact"
        );
        assert_eq!(
            mapping(&base)["object"]["shared"][0]["source"],
            yaml("base"),
            "base nested value must stay intact"
        );
        assert_eq!(
            mapping(&overlay)["object"]["overlay"][0]["source"],
            yaml("overlay"),
            "overlay nested value must stay intact"
        );
        assert_eq!(
            mapping(&overlay)["array"][0],
            yaml("overlay"),
            "overlay array must stay intact"
        );
    }

    #[test]
    fn merge_non_string_mapping_keys() {
        let mut base = Mapping::new();
        base.insert(Value::Number(1.into()), yaml("one"));
        base.insert(Value::Bool(true), yaml("truth"));
        let mut overlay = Mapping::new();
        overlay.insert(Value::Number(1.into()), yaml("uno"));
        overlay.insert(Value::Bool(true), Value::Null);
        overlay.insert(Value::Number(2.into()), yaml("two"));

        let merged = merge(&Value::Mapping(base), &Value::Mapping(overlay));
        let map = mapping(&merged);
        assert_eq!(map.get(Value::Number(1.into())).unwrap(), &yaml("uno"));
        assert!(
            map.get(Value::Bool(true)).is_none(),
            "null deletes bool key"
        );
        assert_eq!(map.get(Value::Number(2.into())).unwrap(), &yaml("two"));
    }

    #[test]
    fn deep_copy_is_independent() {
        let src = yaml("map: {slice: [{value: original}]}\n");
        let mut copy = deep_copy(&src);
        mapping_mut(&mut copy)
            .get_mut("map")
            .unwrap()
            .as_mapping_mut()
            .unwrap()
            .get_mut("slice")
            .unwrap()
            .as_sequence_mut()
            .unwrap()[0]
            .as_mapping_mut()
            .unwrap()
            .insert(Value::String("value".into()), yaml("changed"));

        assert_eq!(
            mapping(&src)["map"]["slice"][0]["value"],
            yaml("original"),
            "source must not see copy mutations"
        );
    }

    #[test]
    fn deep_copy_preserves_null_and_empty_containers() {
        assert_eq!(deep_copy(&Value::Null), Value::Null);
        assert_eq!(
            deep_copy(&Value::Sequence(Vec::new())),
            Value::Sequence(Vec::new())
        );
        assert_eq!(
            deep_copy(&Value::Mapping(Mapping::new())),
            Value::Mapping(Mapping::new())
        );
    }

    #[test]
    fn deep_copy_tagged_value_is_independent() {
        let src = Value::Tagged(Box::new(serde_yaml::value::TaggedValue {
            tag: serde_yaml::value::Tag::new("!custom"),
            value: yaml("nested: {k: original}"),
        }));
        let mut copy = deep_copy(&src);
        if let Value::Tagged(tagged) = &mut copy {
            *mapping_mut(&mut tagged.value)
                .get_mut("nested")
                .unwrap()
                .as_mapping_mut()
                .unwrap()
                .get_mut("k")
                .unwrap() = yaml("changed");
        } else {
            panic!("expected tagged copy");
        }
        if let Value::Tagged(tagged) = &src {
            assert_eq!(mapping(&tagged.value)["nested"]["k"], yaml("original"));
        }
    }

    fn merge_layers(layers: &[Value]) -> Value {
        let (first, rest) = layers.split_first().expect("at least one layer");
        rest.iter()
            .fold(deep_copy(first), |acc, layer| merge(&acc, layer))
    }

    #[derive(serde::Deserialize)]
    struct GofulmenParityDoc {
        cases: Vec<GofulmenParityCase>,
    }

    #[derive(serde::Deserialize)]
    struct GofulmenParityCase {
        name: String,
        base: Value,
        overlay: Value,
        expected: Value,
    }

    #[test]
    fn gofulmen_merge_parity_fixtures() {
        // Ported from https://github.com/fulmenhq/gofulmen/blob/main/config/primitives_test.go
        let doc: GofulmenParityDoc =
            serde_yaml::from_str(include_str!("testdata/gofulmen-merge-parity.yaml"))
                .expect("parity fixture yaml should parse");
        assert!(
            !doc.cases.is_empty(),
            "parity fixture file must contain cases"
        );

        for case in &doc.cases {
            let got = merge(&case.base, &case.overlay);
            assert_eq!(got, case.expected, "parity case `{}` failed", case.name);
        }
    }

    #[test]
    fn merge_defaults_user_env_null_delete_then_restore() {
        let defaults = yaml(
            r#"
keep: from-defaults
deleted_then_restored: from-defaults
stays_deleted: from-defaults
nested:
  x: 1
  y: 2
"#,
        );
        let user = yaml(
            r#"
deleted_then_restored: null
stays_deleted: null
nested:
  y: null
  z: 3
"#,
        );
        let env = yaml(
            r#"
deleted_then_restored: from-env
nested:
  y: from-env
"#,
        );

        let merged = merge_layers(&[defaults, user, env]);
        assert_eq!(
            merged,
            yaml(
                r#"
keep: from-defaults
deleted_then_restored: from-env
nested:
  x: 1
  z: 3
  y: from-env
"#
            )
        );
        let map = mapping(&merged);
        assert!(
            map.get("stays_deleted").is_none(),
            "null in user with no later restore must omit the key"
        );
    }

    #[test]
    fn merge_null_missing_key_null_in_sequence_and_map_over_null_base() {
        let missing = merge(&yaml("a: 1\n"), &yaml("absent: null\n"));
        assert_eq!(missing, yaml("a: 1\n"));
        assert!(
            mapping(&missing).get("absent").is_none(),
            "null for a key that was never present is a no-op, not an inserted null"
        );

        let with_seq = merge(
            &yaml("list: [1, 2]\nkeep: yes\n"),
            &yaml("list: [9, null, 8]\n"),
        );
        assert_eq!(with_seq, yaml("keep: yes\nlist: [9, null, 8]\n"));
        let seq = mapping(&with_seq)["list"]
            .as_sequence()
            .expect("list must remain a sequence");
        assert_eq!(seq[1], Value::Null, "null inside a sequence is a value");

        let over_null = merge(
            &yaml("k: null\nkeep: 1\nnested:\n  inner: null\n"),
            &yaml("k:\n  child: 2\nnested:\n  inner:\n    rebuilt: true\n"),
        );
        assert_eq!(
            over_null,
            yaml("k:\n  child: 2\nkeep: 1\nnested:\n  inner:\n    rebuilt: true\n")
        );
    }

    #[cfg(feature = "three-layer-config")]
    #[test]
    fn three_layer_agrees_with_public_merge() {
        use crate::config::three_layer::{merge_with_provenance, ConfigLayer, Provenance};

        let cases: Vec<Vec<Value>> = vec![
            vec![
                yaml("a: 1\nnested:\n  x: 1\n  list: [1, 2]\n"),
                yaml("a: 2\nnested:\n  y: 2\n  list: [9]\n"),
                yaml("a: 3\nnested:\n  x: 10\n"),
            ],
            vec![
                yaml("keep: from-defaults\ndeleted_then_restored: from-defaults\nstays_deleted: from-defaults\n"),
                yaml("deleted_then_restored: null\nstays_deleted: null\n"),
                yaml("deleted_then_restored: from-env\n"),
            ],
            vec![yaml("k: null\n"), yaml("k:\n  child: 2\n")],
            vec![yaml("list: [1, 2]\n"), yaml("list: [9, null, 8]\n")],
            vec![yaml("a: 1\n"), yaml("absent: null\n")],
            vec![yaml("{a: 1}"), yaml("null"), yaml("{a: 2}")],
        ];

        for layers in cases {
            let via_public = merge_layers(&layers);
            let (first, rest) = layers.split_first().unwrap();
            let mut provenance = Provenance::default();
            let via_three = rest.iter().fold(deep_copy(first), |acc, overlay| {
                merge_with_provenance(
                    acc,
                    deep_copy(overlay),
                    ConfigLayer::Runtime,
                    &mut provenance,
                    "",
                )
            });
            assert_eq!(
                via_public, via_three,
                "three_layer must use the same value merge as config::merge"
            );
        }
    }
}
