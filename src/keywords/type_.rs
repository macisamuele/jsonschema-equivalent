use crate::keywords::preserve_keys;
use serde_json::Value;

/// Removes all the schema keywords that are irrelevant/incongruent with the presence
/// of a specific `type` keyword
pub(crate) fn remove_extraneous_keys_keyword_type(schema: &mut Value) -> &mut Value {
    match schema.get("type").and_then(Value::as_str) {
        Some("array") => preserve_keys(
            schema
                .as_object_mut()
                .expect("As a property exist we're sure that we're dealing with an object"),
            &[
                "additionalItems",
                "contains",
                "const",
                "enum",
                "items",
                "maxItems",
                "minItems",
                "type",
                "uniqueItems",
            ],
        ),
        Some("boolean") | Some("null") => preserve_keys(
            schema
                .as_object_mut()
                .expect("As a property exist we're sure that we're dealing with an object"),
            &["const", "enum", "type"],
        ),
        Some("integer") | Some("number") => preserve_keys(
            schema
                .as_object_mut()
                .expect("As a property exist we're sure that we're dealing with an object"),
            &[
                "const",
                "enum",
                "exclusiveMaximum",
                "exclusiveMinimum",
                "format",
                "maximum",
                "minimum",
                "multipleOf",
                "type",
            ],
        ),
        Some("object") => preserve_keys(
            schema
                .as_object_mut()
                .expect("As a property exist we're sure that we're dealing with an object"),
            &[
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
            ],
        ),
        Some("string") => preserve_keys(
            schema
                .as_object_mut()
                .expect("As a property exist we're sure that we're dealing with an object"),
            &[
                "contentMediaType",
                "contentEncoding",
                "const",
                "enum",
                "format",
                "maxLength",
                "minLength",
                "pattern",
                "type",
            ],
        ),
        _ => {}
    };
    schema
}

#[cfg(test)]
mod tests {
    use super::remove_extraneous_keys_keyword_type;
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
    // {"type": "boolean", ...}
    #[test_case(json!({"type": "boolean", "enum": [true]}) => json!({"type": "boolean", "enum": [true]}))]
    // {"type": "integer", ...}
    #[test_case(json!({"type": "integer", "minimum": 1}) => json!({"type": "integer", "minimum": 1}))]
    // {"type": "null", ...}
    #[test_case(json!({"type": "null", "enum": [null]}) => json!({"type": "null", "enum": [null]}))]
    // {"type": "number", ...}
    #[test_case(json!({"type": "number", "minimum": 1}) => json!({"type": "number", "minimum": 1}))]
    // {"type": "object", ...}
    #[test_case(json!({"type": "object", "minProperties": 1}) => json!({"type": "object", "minProperties": 1}))]
    // {"type": "string", ...}
    #[test_case(json!({"type": "string", "minLength": 1}) => json!({"type": "string", "minLength": 1}))]
    fn test_remove_extraneous_keys_keyword_type_does_remove_keys(mut schema: Value) -> Value {
        let _ = remove_extraneous_keys_keyword_type(&mut schema);
        schema
    }
}
