use crate::{constants::KEYWORDS, primitive_type::PrimitiveType};
use serde_json::{Map, Value};
use std::collections::{BTreeSet, HashSet};
use std::convert::TryFrom;

/// Replace the `schema` with `false`.
///
/// The objective of the method is to limit as much as possible the exposure
/// to more memory involved memory-related APIs.
/// Using `std::mem::replace` ensures that the value stored in `schema` is dropped
/// once leaving the scope of the method
#[inline]
pub(crate) fn replace_schema_with_false_schema(schema: &mut Value) {
    std::mem::replace(schema, Value::Bool(false));
}
/// Replace the `schema` with `true`.
///
/// The objective of the method is to limit as much as possible the exposure
/// to more memory involved memory-related APIs.
/// Using `std::mem::replace` ensures that the value stored in `schema` is dropped
/// once leaving the scope of the method
#[allow(dead_code)]
#[inline]
pub(crate) fn replace_schema_with_true_schema(schema: &mut Value) {
    std::mem::replace(schema, Value::Bool(true));
}

/// Checks if the input schema is a `false` schema
#[allow(dead_code)]
#[inline]
pub(crate) fn is_false_schema(schema: &Value) -> bool {
    match schema {
        Value::Bool(false) => true,
        _ => false,
    }
}

/// Checks if the input schema is a `true` schema
#[inline]
pub(crate) fn is_true_schema(schema: &Value) -> bool {
    match schema {
        Value::Bool(true) => true,
        Value::Object(obj) if obj.is_empty() => true,
        _ => false,
    }
}

/// Extract a set of primitive types contained by the schema in input
///
/// NOTE: A `BTreeSet` is returned in order to preserve order-predictability while testing
pub(crate) fn get_primitive_types(schema: &Map<String, Value>) -> BTreeSet<PrimitiveType> {
    let mut set = BTreeSet::default();
    if let Some(type_) = schema.get("type") {
        match type_ {
            Value::String(type_s) => {
                if let Ok(pt) = PrimitiveType::try_from(type_s.as_str()) {
                    set.insert(pt);
                }
            }
            Value::Array(types_) => {
                for type_s in types_ {
                    if let Ok(pt) = PrimitiveType::try_from(type_s) {
                        set.insert(pt);
                    }
                }
            }
            _ => {}
        }
    }
    set
}

/// Build the list of keywords to remove starting from the keywords to preserve
/// This is done in order to avoid removing keywords added in future Draft versions
#[inline]
fn keywords_to_remove(keywords_to_preserve: &HashSet<&'static str>) -> HashSet<&'static str> {
    KEYWORDS.difference(keywords_to_preserve).cloned().collect()
}

/// Removes all the keys present in map which are not present in `keys_to_preserve`
pub(crate) fn preserve_keys(
    map: &mut Map<String, Value>,
    keys_to_preserve: &HashSet<&'static str>,
) {
    let remove_keywords: HashSet<&str> = keywords_to_remove(keys_to_preserve);
    let keys_to_remove: Vec<String> = map
        .keys()
        .filter(|key| remove_keywords.contains(key.as_str()))
        .cloned()
        .collect();
    for key_to_remove in keys_to_remove {
        map.remove(&key_to_remove.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::{
        get_primitive_types, is_false_schema, is_true_schema, keywords_to_remove, preserve_keys,
        replace_schema_with_false_schema, replace_schema_with_true_schema, KEYWORDS,
    };
    use crate::primitive_type::PrimitiveType;
    use serde_json::{json, Value};
    use std::collections::{BTreeSet, HashSet};
    use test_case::test_case;

    macro_rules! hash_set {
        ($($elem: expr),* $(,)*) => {
            vec![$($elem),*].iter().cloned().collect::<HashSet<_>>()
        };
    }

    macro_rules! btree_set {
        ($($elem: expr),* $(,)*) => {
            vec![$($elem),*].iter().cloned().collect::<BTreeSet<_>>()
        };
    }

    #[test_case(json!({}))]
    #[test_case(json!(null))]
    #[test_case(json!(true))]
    fn test_replace_schema_with_false_schema(mut schema: Value) {
        replace_schema_with_false_schema(&mut schema);
        assert_eq!(schema, Value::Bool(false));
    }

    #[test_case(json!({}))]
    #[test_case(json!(null))]
    #[test_case(json!(true))]
    fn test_replace_schema_with_true_schema(mut schema: Value) {
        replace_schema_with_true_schema(&mut schema);
        assert_eq!(schema, Value::Bool(true));
    }

    #[test_case(json!({}) => false)]
    #[test_case(json!({"type": "string"}) => false)]
    #[test_case(json!(false) => true)]
    #[test_case(json!(true) => false)]
    fn test_is_false_schema(schema: Value) -> bool {
        is_false_schema(&schema)
    }

    #[test_case(json!({}) => true)]
    #[test_case(json!({"type": "string"}) => false)]
    #[test_case(json!(false) => false)]
    #[test_case(json!(true) => true)]
    fn test_is_true_schema(schema: Value) -> bool {
        is_true_schema(&schema)
    }

    #[test_case(json!({}) => btree_set!())]
    #[test_case(json!({"type": "array"}) => btree_set!(PrimitiveType::Array))]
    #[test_case(json!({"type": ["boolean"]}) => btree_set!(PrimitiveType::Boolean))]
    #[test_case(json!({"type": ["integer", "null"]}) => btree_set!(PrimitiveType::Integer, PrimitiveType::Null))]
    #[test_case(json!({"type": ["a-wrong-type"]}) => btree_set!())]
    fn test_get_primitive_types(schema: Value) -> BTreeSet<PrimitiveType> {
        get_primitive_types(schema.as_object().unwrap())
    }

    #[test]
    fn test_keywords_to_remove_remove_not_existing_keyword() {
        assert_eq!(
            keywords_to_remove(&hash_set!["not-exitsting"]),
            KEYWORDS.iter().cloned().collect()
        );
    }

    #[test]
    fn test_keywords_to_remove_remove_existing_keyword() {
        assert_eq!(
            keywords_to_remove(&hash_set!["type"]),
            KEYWORDS
                .iter()
                .cloned()
                .filter(|key| key != &"type")
                .collect()
        );

        assert_eq!(
            keywords_to_remove(&hash_set!["minimum", "type"]),
            KEYWORDS
                .iter()
                .cloned()
                .filter(|key| key != &"minimum" && key != &"type")
                .collect()
        );
    }

    #[test_case(
        json!({}), &hash_set!["not-existing-key"] => json!({});
        "not fail if key does not exist"
    )]
    #[test_case(
        json!({"non-jsonschema-keyword": 1}), &hash_set![] => json!({"non-jsonschema-keyword": 1});
        "not remove non jsonschema keywords (even if requested)"
    )]
    #[test_case(
        json!({"type": 1}), &hash_set![] => json!({});
        "remove jsonschema keywords (if requested)"
    )]
    fn test_preserve_keys_remove_key_not_present(
        mut map: Value,
        keywords_to_remove: &HashSet<&'static str>,
    ) -> Value {
        preserve_keys(map.as_object_mut().unwrap(), keywords_to_remove);
        map
    }
}
