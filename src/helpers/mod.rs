pub(crate) mod is;
pub(crate) mod replace;

use crate::{constants::KEYWORDS, primitive_type::PrimitiveType};
use serde_json::{Map, Value};
use std::collections::{BTreeSet, HashSet};
use std::convert::TryFrom;

/// Extract a set of primitive types contained by the input `type` keyword. (`maybe_type` should be the result of `schema.get("type")`)
///
/// NOTE: A `BTreeSet` is returned in order to preserve order-predictability while testing
pub(crate) fn get_primitive_types(maybe_type: Option<&Value>) -> BTreeSet<PrimitiveType> {
    let mut set = BTreeSet::default();
    if let Some(type_) = maybe_type {
        match type_ {
            Value::String(type_s) => {
                if let Ok(pt) = PrimitiveType::try_from(type_s.as_str()) {
                    let _ = set.insert(pt);
                }
            }
            Value::Array(types_) => {
                for type_s in types_ {
                    if let Ok(pt) = PrimitiveType::try_from(type_s) {
                        let _ = set.insert(pt);
                    }
                }
            }
            _ => {}
        }
    }
    set
}

pub(crate) fn to_json_schema_primitive_types(
    primitive_types: &mut BTreeSet<PrimitiveType>,
) -> Option<Value> {
    match primitive_types.len() {
        0 => None,
        1 => Some(Value::String(
            primitive_types
                .iter()
                .next()
                .expect("Not empty iterator, so there is at least one element")
                .to_string(),
        )),
        _ if primitive_types.contains(&PrimitiveType::Integer)
            && primitive_types.contains(&PrimitiveType::Number) =>
        {
            let _ = primitive_types.remove(&PrimitiveType::Integer);
            to_json_schema_primitive_types(primitive_types)
        }
        _ => Some(Value::Array(
            primitive_types
                .iter()
                .map(|primitive_type| Value::String(primitive_type.to_string()))
                .collect(),
        )),
    }
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
        let _ = map.remove(&key_to_remove.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::{
        get_primitive_types, keywords_to_remove, preserve_keys, to_json_schema_primitive_types,
        KEYWORDS,
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

    #[test_case(&json!({}) => btree_set!())]
    #[test_case(&json!({"type": "array"}) => btree_set!(PrimitiveType::Array))]
    #[test_case(&json!({"type": ["boolean"]}) => btree_set!(PrimitiveType::Boolean))]
    #[test_case(&json!({"type": ["integer", "null"]}) => btree_set!(PrimitiveType::Integer, PrimitiveType::Null))]
    #[test_case(&json!({"type": ["a-wrong-type"]}) => btree_set!())]
    fn test_get_primitive_types(schema: &Value) -> BTreeSet<PrimitiveType> {
        get_primitive_types(schema.get("type"))
    }

    #[test_case(btree_set!() => None)]
    #[test_case(btree_set!(PrimitiveType::Array) => Some(json!("array")))]
    #[test_case(btree_set!(PrimitiveType::Boolean, PrimitiveType::Null) => Some(json!(["boolean", "null"])))]
    #[test_case(btree_set!(PrimitiveType::Integer, PrimitiveType::Number) => Some(json!("number")))]
    #[test_case(btree_set!(PrimitiveType::Integer, PrimitiveType::Number, PrimitiveType::String) => Some(json!(["number", "string"])))]
    fn test_to_json_schema_primitive_types(
        mut primitive_types: BTreeSet<PrimitiveType>,
    ) -> Option<Value> {
        to_json_schema_primitive_types(&mut primitive_types)
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
        preserve_keys(
            map.as_object_mut().expect("It should be there"),
            keywords_to_remove,
        );
        map
    }
}
