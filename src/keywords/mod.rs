//! The reference of the JSON Schema specifications are available on
//! <https://tools.ietf.org/html/draft-handrews-json-schema-validation-01>
mod additional_properties;
mod type_;

use serde_json::{Map, Value};
use std::collections::HashSet;

/// Equivalent to `&[T]::contains` API. The API takes into account that
/// the input slice is sorted and so binary search can be used
#[inline]
fn contains<T: Ord>(sorted_slice: &[T], value: &T) -> bool {
    sorted_slice.binary_search(value).is_ok()
}

/// All keywords of Draft4, Draft6 and Draft7
pub(crate) static KEYWORDS: &[&str] = &[
    "additionalItems",
    "additionalProperties",
    "allOf",
    "anyOf",
    "const",
    "contains",
    "contentEncoding",
    "contentMediaType",
    "dependencies",
    "else",
    "enum",
    "exclusiveMaximum",
    "exclusiveMinimum",
    "format",
    "if",
    "items",
    "maxItems",
    "maxLength",
    "maxProperties",
    "maximum",
    "minItems",
    "minLength",
    "minProperties",
    "minimum",
    "multipleOf",
    "not",
    "oneOf",
    "pattern",
    "patternProperties",
    "properties",
    "propertyNames",
    "required",
    "then",
    "type",
    "uniqueItems",
];

/// Keywords which contains valid JSON Schema
///
/// This contains the list of keywords defined by the JSON Schema specifications as
/// * > The value of "..." MUST be a valid JSON Schema.
/// * > The value of "..." MUST be an object. Each value of this object MUST be a valid JSON Schema.
/// * > This keyword's value MUST be a non-empty array.  Each item of the array MUST be a valid JSON Schema.
/// * > The value of "..." MUST be either a valid JSON Schema or an array of valid JSON Schemas.
static KEYWORDS_WITH_SUBSCHEMAS: &[&str] = &[
    "additionalItems",
    "additionalProperties",
    "allOf",
    "anyOf",
    "const",
    "contains",
    "contentEncoding",
    "contentMediaType",
    "dependencies",
    "else",
    "enum",
    "exclusiveMaximum",
    "exclusiveMinimum",
    "format",
    "if",
    "items",
    "maxItems",
    "maxLength",
    "maxProperties",
    "maximum",
    "minItems",
    "minLength",
    "minProperties",
    "minimum",
    "multipleOf",
    "not",
    "oneOf",
    "pattern",
    "patternProperties",
    "properties",
    "propertyNames",
    "required",
    "then",
    "type",
    "uniqueItems",
];

/// Keywords value MUST be a valid JSON Schema
///
/// This contains the list of keywords defined by the JSON Schema specifications as
/// > The value of "..." MUST be a valid JSON Schema.
static KEYWORDS_WITH_DIRECT_SUBSCHEMAS: &[&str] = &[
    "additionalItems",
    "additionalProperties",
    "contains",
    "else",
    "if",
    "not",
    "propertyNames",
    "then",
];

/// Order of the methods used to update the schema
///
/// NOTE: The order might be important for the capability/quality of the
/// library so please be mindfull before modifying the order (and if you
/// do so please motivate it in the pull request description)
static UPDATE_SCHEMA_METHODS: &[fn(&mut Value) -> &mut Value] = &[
    type_::remove_extraneous_keys_keyword_type,
    additional_properties::remove_empty_additional_properties,
];

/// Replace the `schema` with `false`.
///
/// The objective of the method is to limit as much as possible the exposure
/// to more memory involved memory-related APIs.
/// Using `std::mem::replace` ensures that the value stored in `schema` is dropped
/// once leaving the scope of the method
#[allow(dead_code)]
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

/// Build the list of keywords to remove starting from the keywords to preserve
/// This is done in order to avoid removing keywords added in future Draft versions
fn keywords_to_remove(keywords_to_preserve: &[&'static str]) -> HashSet<&'static str> {
    let mut keywords: HashSet<&str> = KEYWORDS.iter().cloned().collect();
    for keyword in keywords_to_preserve {
        keywords.remove(keyword);
    }
    keywords
}

/// Removes all the keys present in map which are not present in `keys_to_preserve`
pub(crate) fn preserve_keys(map: &mut Map<String, Value>, keys_to_preserve: &[&'static str]) {
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

/// Perform the schema optimisaton without descending the schema
fn update_schema_no_recursive(schema: &mut Value) -> &mut Value {
    let mut result_schema = schema;
    for method in UPDATE_SCHEMA_METHODS {
        result_schema = method(result_schema);
        if let Value::Bool(_) = result_schema {
            // If the schema is a `true` or `false` schema
            // we know that we cannot optimise it even more
            return result_schema;
        }
    }
    result_schema
}

/// Discend the schema and optimise it.
pub(crate) fn update_schema(schema: &mut Value) -> &mut Value {
    match schema {
        Value::Object(schema_object) => {
            for (key, subschema) in schema_object {
                if contains(KEYWORDS_WITH_SUBSCHEMAS, &key.as_ref()) {
                    match subschema {
                        Value::Object(subschema_object) => {
                            if contains(KEYWORDS_WITH_DIRECT_SUBSCHEMAS, &key.as_ref()) {
                                // In case of schemas where the keyword value MUST be a valid JSON Schema
                                // ie. `{"additionalProperties": {"type": "string"}}`
                                update_schema(subschema);
                            } else {
                                // In case of schemas where the keyword holds a JSON Object and its
                                // values MUST be a valid JSON Schema
                                // ie. `{"properties": {"property" {"type": "string"}}}`
                                for subschema_value in subschema_object.values_mut() {
                                    update_schema(subschema_value);
                                }
                            }
                        }
                        Value::Array(subschema_array) => {
                            // In case of schemas where the keyword holds a JSON Array and its
                            // values MUST be a valid JSON Schema
                            // ie. `{"allOf": [{"type": "string"}]}`
                            for subschema_value in subschema_array {
                                update_schema(subschema_value);
                            }
                        }
                        _ => {}
                    }
                    update_schema(subschema);
                }
            }

            update_schema_no_recursive(schema)
        }
        _ => schema,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        keywords_to_remove, preserve_keys, update_schema, KEYWORDS,
        KEYWORDS_WITH_DIRECT_SUBSCHEMAS, KEYWORDS_WITH_SUBSCHEMAS,
    };
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test]
    fn test_ensure_sorted() {
        fn is_sorted<T: Ord>(value: &[T]) -> bool {
            value.windows(2).all(|values| values[0] <= values[1])
        }
        // Smoke test to ensure that `is_sorted` does actually check if is sorted
        assert!(is_sorted(&["a", "b", "c"]));
        assert!(!is_sorted(&["b", "a", "c"]));

        // The code assumes that the static slices are sorted.
        // Running a quick test to ensure that this is true.
        assert!(is_sorted(KEYWORDS));
        assert!(is_sorted(KEYWORDS_WITH_SUBSCHEMAS));
        assert!(is_sorted(KEYWORDS_WITH_DIRECT_SUBSCHEMAS));
    }

    #[test]
    fn test_keywords_to_remove_remove_not_existing_keyword() {
        assert_eq!(
            keywords_to_remove(&["not-exitsting"]),
            KEYWORDS.iter().cloned().collect()
        );
    }

    #[test]
    fn test_keywords_to_remove_remove_existing_keyword() {
        assert_eq!(
            keywords_to_remove(&["type"]),
            KEYWORDS
                .iter()
                .cloned()
                .filter(|key| key != &"type")
                .collect()
        );

        assert_eq!(
            keywords_to_remove(&["minimum", "type"]),
            KEYWORDS
                .iter()
                .cloned()
                .filter(|key| key != &"minimum" && key != &"type")
                .collect()
        );
    }

    #[test_case(
        json!({}), &["not-existing-key"] => json!({});
        "not fail if key does not exist"
    )]
    #[test_case(
        json!({"non-jsonschema-keyword": 1}), &[] => json!({"non-jsonschema-keyword": 1});
        "not remove non jsonschema keywords (even if requested)"
    )]
    #[test_case(
        json!({"type": 1}), &[] => json!({});
        "remove jsonschema keywords (if requested)"
    )]
    fn test_preserve_keys_remove_key_not_present(
        mut map: Value,
        keywords_to_remove: &[&'static str],
    ) -> Value {
        #[allow(clippy::option_unwrap_used)]
        preserve_keys(map.as_object_mut().unwrap(), keywords_to_remove);
        map
    }

    #[test_case(json!({}) => json!({}))]
    #[test_case(json!({"properties": {"prop": {"type": "string", "minimum": 1}}}) => json!({"properties": {"prop": {"type": "string"}}}))]
    #[test_case(json!({"allOf": [{"type": "string", "minimum": 1}]}) => json!({"allOf": [{"type": "string"}]}))]
    fn test_update_schema_descend_schema(mut schema: Value) -> Value {
        update_schema(&mut schema);
        schema
    }
}
