use crate::keywords::preserve_keys;
use serde_json::Value;
use std::collections::HashSet;

static KEYWORDS_TYPE_ARRAY: &[&str] = &[
    "additionalItems",
    "contains",
    "const",
    "enum",
    "items",
    "maxItems",
    "minItems",
    "type",
    "uniqueItems",
];
static KEYWORDS_TYPE_BOOLEAN: &[&str] = &["const", "enum", "type"];
static KEYWORDS_TYPE_NULL: &[&str] = KEYWORDS_TYPE_BOOLEAN;
static KEYWORDS_TYPE_INTEGER: &[&str] = &[
    "const",
    "enum",
    "exclusiveMaximum",
    "exclusiveMinimum",
    "format",
    "maximum",
    "minimum",
    "multipleOf",
    "type",
];
static KEYWORDS_TYPE_NUMBER: &[&str] = KEYWORDS_TYPE_INTEGER;
static KEYWORDS_TYPE_OBJECT: &[&str] = &[
    "additionalProperties",
    "allOf",
    "anyOf",
    "dependencies",
    "const",
    "enum",
    "maxProperties",
    "minProperties",
    "not",
    "oneOf",
    "patternProperties",
    "properties",
    "propertyNames",
    "required",
    "type",
];
static KEYWORDS_TYPE_STRING: &[&str] = &[
    "contentMediaType",
    "contentEncoding",
    "const",
    "enum",
    "format",
    "maxLength",
    "minLength",
    "pattern",
    "type",
];

pub(crate) fn optimise_keyword_type_if_array(schema: &mut Value) -> &mut Value {
    let type_ = if let Some(value) = schema.get("type") {
        value.clone()
    } else {
        return schema;
    };

    let schema_object = schema
        .as_object_mut()
        .expect("As a property exist we're sure that we're dealing with an object");

    if let Value::Array(mut items) = type_ {
        if items.is_empty() {
            schema_object.remove("type");
            return schema;
        } else if items.len() == 1 {
            schema_object.remove("type");
            schema_object.insert("type".to_string(), items[0].clone());
        } else if items.contains(&Value::String("integer".to_string()))
            && items.contains(&Value::String("number".to_string()))
        {
            if items.len() == 2 {
                schema_object.insert("type".to_string(), Value::String("number".to_string()));
            } else {
                let index = items.iter().enumerate().find_map(|(index, item)| {
                    if item.as_str() == Some("integer") {
                        Some(index)
                    } else {
                        None
                    }
                });
                items.remove(index.expect(
                    "Index will be present as we know that `integer` in contained in items",
                ));
                schema_object.insert("type".to_string(), Value::Array(items));
            }
        };
    }
    schema
}

/// Removes all the schema keywords that are irrelevant/incongruent with the presence
/// of a specific `type` keyword
pub(crate) fn remove_extraneous_keys_keyword_type(schema: &mut Value) -> &mut Value {
    let type_ = if let Some(value) = schema.get("type") {
        value.clone()
    } else {
        return schema;
    };

    let schema_object = schema
        .as_object_mut()
        .expect("As a property exist we're sure that we're dealing with an object");

    match type_.as_str() {
        Some("array") => preserve_keys(schema_object, KEYWORDS_TYPE_ARRAY),
        Some("boolean") => preserve_keys(schema_object, KEYWORDS_TYPE_BOOLEAN),
        Some("integer") => preserve_keys(schema_object, KEYWORDS_TYPE_INTEGER),
        Some("null") => preserve_keys(schema_object, KEYWORDS_TYPE_NULL),
        Some("number") => preserve_keys(schema_object, KEYWORDS_TYPE_NUMBER),
        Some("object") => preserve_keys(schema_object, KEYWORDS_TYPE_OBJECT),
        Some("string") => preserve_keys(schema_object, KEYWORDS_TYPE_STRING),
        _ => {}
    };
    if let Some(items) = type_.as_array() {
        let mut keys_to_reserve = HashSet::<&'static str>::new();
        for item in items {
            match item.as_str() {
                Some("array") => KEYWORDS_TYPE_ARRAY.iter().for_each(|key| {
                    keys_to_reserve.insert(key);
                }),
                Some("boolean") => KEYWORDS_TYPE_BOOLEAN.iter().for_each(|key| {
                    keys_to_reserve.insert(key);
                }),
                Some("integer") => KEYWORDS_TYPE_INTEGER.iter().for_each(|key| {
                    keys_to_reserve.insert(key);
                }),
                Some("null") => KEYWORDS_TYPE_NULL.iter().for_each(|key| {
                    keys_to_reserve.insert(key);
                }),
                Some("number") => KEYWORDS_TYPE_NUMBER.iter().for_each(|key| {
                    keys_to_reserve.insert(key);
                }),
                Some("object") => KEYWORDS_TYPE_OBJECT.iter().for_each(|key| {
                    keys_to_reserve.insert(key);
                }),
                Some("string") => KEYWORDS_TYPE_STRING.iter().for_each(|key| {
                    keys_to_reserve.insert(key);
                }),
                _ => {}
            }
        }

        preserve_keys(
            schema_object,
            &keys_to_reserve.iter().cloned().collect::<Vec<_>>(),
        );
    }
    schema
}

#[cfg(test)]
mod tests {
    use super::{optimise_keyword_type_if_array, remove_extraneous_keys_keyword_type};
    use crate::keywords::update_schema;
    use serde_json::{json, Value};
    use test_case::test_case;

    // Eventully add test cases for all the keywords to remove
    #[test_case(json!({}); "do nothing if type keyword is not present")]
    // {"type": "array", ...}
    #[test_case(json!({"type": "array"}))]
    #[test_case(json!({"type": "array", "additionalItems": true}))]
    #[test_case(json!({"type": "array", "contains": [1]}))]
    #[test_case(json!({"type": "array", "const": ["value"]}))]
    #[test_case(json!({"type": "array", "enum": [["item"]]}))]
    #[test_case(json!({"type": "array", "items": {}}))]
    #[test_case(json!({"type": "array", "maxItems": 1}))]
    #[test_case(json!({"type": "array", "minItems": 1}))]
    #[test_case(json!({"type": "array", "uniqueItems": true}))]
    // {"type": "boolean", ...}
    #[test_case(json!({"type": "boolean"}))]
    #[test_case(json!({"type": "boolean", "const": [true]}))]
    #[test_case(json!({"type": "boolean", "enum": [true]}))]
    // {"type": "integer", ...}
    #[test_case(json!({"type": "integer"}))]
    #[test_case(json!({"type": "integer", "const": 1}))]
    #[test_case(json!({"type": "integer", "enum": [1, 2]}))]
    #[test_case(json!({"type": "integer", "exclusiveMaximum": 1}))]
    #[test_case(json!({"type": "integer", "exclusiveMinimum": 1}))]
    #[test_case(json!({"type": "integer", "format": "int32"}))]
    #[test_case(json!({"type": "integer", "maximum": 1}))]
    #[test_case(json!({"type": "integer", "minimum": 1}))]
    #[test_case(json!({"type": "integer", "multipleOf": 1}))]
    // {"type": "null", ...}
    #[test_case(json!({"type": "null"}))]
    #[test_case(json!({"type": "null", "const": [null]}))]
    #[test_case(json!({"type": "null", "enum": [null]}))]
    // {"type": "number", ...}
    #[test_case(json!({"type": "number"}))]
    #[test_case(json!({"type": "number", "const": 1}))]
    #[test_case(json!({"type": "number", "enum": [1, 2]}))]
    #[test_case(json!({"type": "number", "exclusiveMaximum": 1}))]
    #[test_case(json!({"type": "number", "exclusiveMinimum": 1}))]
    #[test_case(json!({"type": "number", "format": "int32"}))]
    #[test_case(json!({"type": "number", "maximum": 1}))]
    #[test_case(json!({"type": "number", "minimum": 1}))]
    #[test_case(json!({"type": "number", "multipleOf": 1}))]
    // {"type": "object", ...}
    #[test_case(json!({"type": "object"}))]
    #[test_case(json!({"type": "object", "additionalProperties": {}}))]
    #[test_case(json!({"type": "object", "allOf": []}))]
    #[test_case(json!({"type": "object", "anyOf": []}))]
    #[test_case(json!({"type": "object", "dependencies": []}))]
    #[test_case(json!({"type": "object", "const": {"key": "value"}}))]
    #[test_case(json!({"type": "object", "enum": [{"key": "value"}]}))]
    #[test_case(json!({"type": "object", "maxProperties": 1}))]
    #[test_case(json!({"type": "object", "minProperties": 1}))]
    #[test_case(json!({"type": "object", "not": {}}))]
    #[test_case(json!({"type": "object", "oneOf": []}))]
    #[test_case(json!({"type": "object", "patternProperties": {}}))]
    #[test_case(json!({"type": "object", "properties": {}}))]
    #[test_case(json!({"type": "object", "propertyNames": {}}))]
    #[test_case(json!({"type": "object", "required": []}))]
    // {"type": "string", ...}
    #[test_case(json!({"type": "string"}))]
    #[test_case(json!({"type": "string", "contentMediaType": "application/json"}))]
    #[test_case(json!({"type": "string", "contentEncoding": "base64"}))]
    #[test_case(json!({"type": "string", "const": ["key"]}))]
    #[test_case(json!({"type": "string", "enum": ["value"]}))]
    #[test_case(json!({"type": "string", "format": "date"}))]
    #[test_case(json!({"type": "string", "maxLength": 1}))]
    #[test_case(json!({"type": "string", "minLength": 1}))]
    #[test_case(json!({"type": "string", "pattern": "key[0-9]+"}))]
    #[allow(clippy::needless_pass_by_value)]
    fn test_remove_extraneous_keys_keyword_type_does_not_remove_keys(schema: Value) {
        let mut cloned_schema = schema.clone();
        remove_extraneous_keys_keyword_type(&mut cloned_schema);
        assert_eq!(schema, cloned_schema);
    }

    // Eventully add test cases for all the keywords to remove
    #[test_case(json!({}) => json!({}); "do nothing if type keyword is not present")]
    // {"type": "array", ...}
    #[test_case(json!({"type": "array", "minItems": 1}) => json!({"type": "array", "minItems": 1}))]
    #[test_case(json!({"type": "array", "minimum": 1}) => json!({"type": "array"}))]
    // {"type": "boolean", ...}
    #[test_case(json!({"type": "boolean", "enum": [true]}) => json!({"type": "boolean", "enum": [true]}))]
    #[test_case(json!({"type": "boolean", "minimum": 1}) => json!({"type": "boolean"}))]
    // {"type": "integer", ...}
    #[test_case(json!({"type": "integer", "minimum": 1}) => json!({"type": "integer", "minimum": 1}))]
    #[test_case(json!({"type": "integer", "minLength": 1}) => json!({"type": "integer"}))]
    // {"type": "null", ...}
    #[test_case(json!({"type": "null", "enum": [null]}) => json!({"type": "null", "enum": [null]}))]
    #[test_case(json!({"type": "null", "minimum": 1}) => json!({"type": "null"}))]
    // {"type": "number", ...}
    #[test_case(json!({"type": "number", "minimum": 1}) => json!({"type": "number", "minimum": 1}))]
    #[test_case(json!({"type": "number", "minLength": 1}) => json!({"type": "number"}))]
    // {"type": "object", ...}
    #[test_case(json!({"type": "object", "minProperties": 1}) => json!({"type": "object", "minProperties": 1}))]
    #[test_case(json!({"type": "object", "minimum": 1}) => json!({"type": "object"}))]
    // {"type": "string", ...}
    #[test_case(json!({"type": "string", "minLength": 1}) => json!({"type": "string", "minLength": 1}))]
    #[test_case(json!({"type": "string", "minItems": 1}) => json!({"type": "string"}))]
    // {"type": [...], ...}
    #[test_case(json!({"type": ["number", "string"], "minLength": 1}) => json!({"type": ["number", "string"], "minLength": 1}))]
    #[test_case(json!({"type": ["number", "string"], "minLength": 1, "minItems": 1}) => json!({"type": ["number", "string"], "minLength": 1}))]
    fn test_remove_extraneous_keys_keyword_type_does_remove_keys(mut schema: Value) -> Value {
        let _ = remove_extraneous_keys_keyword_type(&mut schema);
        schema
    }

    #[test_case(json!({"type": []}) => json!({}))]
    #[test_case(json!({"type": ["string"]}) => json!({"type": "string"}))]
    #[test_case(json!({"type": ["integer", "number"]}) => json!({"type": "number"}))]
    #[test_case(json!({"type": ["string", "integer", "number"]}) => json!({"type": ["string", "number"]}))]
    fn test_optimise_keyword_type_if_array(mut schema: Value) -> Value {
        let _ = optimise_keyword_type_if_array(&mut schema);
        schema
    }

    #[test_case(json!({"type": ["number", "integer"], "minLength": 1}) => json!({"type": "number"}))]
    fn test_keywords_elided_with_with_correct_order(mut schema: Value) -> Value {
        let _ = update_schema(&mut schema);
        schema
    }
}
