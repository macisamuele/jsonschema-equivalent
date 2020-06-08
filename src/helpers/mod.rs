pub(crate) mod intersect;
pub(crate) mod is;
pub(crate) mod replace;
pub(crate) mod types;

use crate::constants::KEYWORDS;
use serde_json::{map::Entry, Map, Value};
use std::collections::HashSet;

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
) -> bool {
    let remove_keywords: HashSet<&str> = keywords_to_remove(keys_to_preserve);
    let keys_to_remove: Vec<String> = map
        .keys()
        .filter(|key| remove_keywords.contains(key.as_str()))
        .cloned()
        .collect();

    keys_to_remove
        .iter()
        .filter(|key_to_remove| {
            if let Entry::Occupied(entry) = map.entry(*key_to_remove) {
                let _ = entry.remove();
                true
            } else {
                false
            }
        })
        .count()
        != 0
}

/// Join the list of `Value`s by appending all the items in `other` not present in `schema` at the end of `schema`
///
/// WARNING: Possible duplicates in `schema` are not removed
///
/// NOTE: For every item in other we do run a lineary search in `schema` items. This is generally acceptable if `schema`
/// length is "small" and we can eventualy make this assumption. An alternative would be to convert all the items to
/// something ordinable/hashable (like via `ToString::to_string`) but it would require more memory and still a
/// linear loop for the set creation.
#[allow(dead_code)]
fn join_and_deduplicate(schema: &mut Vec<Value>, other: &[Value]) {
    for other_value in other {
        if !schema
            .iter()
            .any(|schema_value| schema_value == other_value)
        {
            schema.push(other_value.clone());
        }
    }
}

/// Intersect the list of `Value`s with the items present in `other`
///
/// WARNING: Possible duplicates in `schema` are not removed
///
/// NOTE: For every item in other we do run a lineary search in `schema` items. This is generally acceptable if `schema`
/// length is "small" and we can eventualy make this assumption. An alternative would be to convert all the items to
/// something ordinable/hashable (like via `ToString::to_string`) but it would require more memory and still a
/// linear loop for the set creation.
#[allow(dead_code)]
fn common_values_and_deduplicate(schema: &mut Vec<Value>, other: &[Value]) {
    let schema_indexes_to_remove: Vec<usize> = schema
        .iter()
        .enumerate()
        .filter_map(|(index, schema_value)| {
            if other.contains(schema_value) {
                None
            } else {
                Some(index)
            }
        })
        .collect();
    for index_to_remove in schema_indexes_to_remove.iter().rev() {
        let _ = schema.remove(*index_to_remove);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        common_values_and_deduplicate, join_and_deduplicate, keywords_to_remove, preserve_keys,
        KEYWORDS,
    };
    use serde_json::{json, Value};
    use std::collections::HashSet;
    use test_case::test_case;

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
        let _ = preserve_keys(
            map.as_object_mut().expect("It should be there"),
            keywords_to_remove,
        );
        map
    }

    #[test_case(json!([1, 3, 5]), &json!([]) => json!([1, 3, 5]))]
    #[test_case(json!([1, 1, 3, 5]), &json!([]) => json!([1, 1, 3, 5]))]
    #[test_case(json!([1, 3, 5]), &json!([3, 2, 1, 5]) => json!([1, 3, 5, 2]))]
    #[test_case(json!([]), &json!([3, 2, 1, 5]) => json!([3, 2, 1, 5]))]
    fn test_join_and_deduplicate(mut schema: Value, other: &Value) -> Value {
        join_and_deduplicate(
            schema.as_array_mut().expect("It should be there"),
            other.as_array().expect("It should be there"),
        );
        schema
    }

    #[test_case(json!([1, 3, 5]), &json!([]) => json!([]))]
    #[test_case(json!([1, 1, 3, 5]), &json!([]) => json!([]))]
    #[test_case(json!([1, 3, 5]), &json!([3, 2, 1, 5]) => json!([1, 3, 5]))]
    #[test_case(json!([1, 1, 3, 5]), &json!([3, 2, 1, 5]) => json!([1, 1, 3, 5]))]
    #[test_case(json!([]), &json!([3, 2, 1, 5]) => json!([]))]
    fn test_common_values_and_deduplicate(mut schema: Value, other: &Value) -> Value {
        common_values_and_deduplicate(
            schema.as_array_mut().expect("It should be there"),
            other.as_array().expect("It should be there"),
        );
        schema
    }
}
