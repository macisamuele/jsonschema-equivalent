use crate::{
    helpers::{preserve_keys, replace, types::PrimitiveTypesBitMap},
    primitive_type::PrimitiveType,
};
use jsonschema_equivalent_rule_processor_logger::log_processing;
use serde_json::Value;
use std::collections::HashSet;

lazy_static::lazy_static! {
    static ref KEYWORDS_TYPE_ARRAY: HashSet<&'static str> = [
        "additionalItems",
        "allOf",
        "anyOf",
        "const",
        "contains",
        "else",
        "enum",
        "if",
        "items",
        "maxItems",
        "minItems",
        "not",
        "oneOf",
        "then",
        "type",
        "uniqueItems",
    ].iter().cloned().collect();
    static ref KEYWORDS_TYPE_BOOLEAN: HashSet<&'static str> = [
        "allOf",
        "anyOf",
        "const",
        "else",
        "enum",
        "if",
        "not",
        "oneOf",
        "then",
        "type",
    ].iter().cloned().collect();
    static ref KEYWORDS_TYPE_NULL: HashSet<&'static str> = KEYWORDS_TYPE_BOOLEAN.iter().cloned().collect();
    static ref KEYWORDS_TYPE_INTEGER: HashSet<&'static str> = [
        "allOf",
        "anyOf",
        "const",
        "else",
        "enum",
        "exclusiveMaximum",
        "exclusiveMinimum",
        "format",
        "if",
        "maximum",
        "minimum",
        "multipleOf",
        "not",
        "oneOf",
        "then",
        "type",
    ].iter().cloned().collect();
    static ref KEYWORDS_TYPE_NUMBER: HashSet<&'static str> = KEYWORDS_TYPE_INTEGER.iter().cloned().collect();
    static ref KEYWORDS_TYPE_OBJECT: HashSet<&'static str> = [
        "additionalProperties",
        "allOf",
        "anyOf",
        "const",
        "dependencies",
        "else",
        "enum",
        "if",
        "maxProperties",
        "minProperties",
        "not",
        "oneOf",
        "patternProperties",
        "properties",
        "propertyNames",
        "required",
        "then",
        "type",
    ].iter().cloned().collect();
    static ref KEYWORDS_TYPE_STRING: HashSet<&'static str> = [
        "allOf",
        "anyOf",
        "const",
        "contentEncoding",
        "contentMediaType",
        "else",
        "enum",
        "format",
        "if",
        "maxLength",
        "minLength",
        "not",
        "oneOf",
        "pattern",
        "then",
        "type",
    ].iter().cloned().collect();
}

/// Removes duplicated types, avoid not need of list and remove the keyword if all the types are included
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn optimise_keyword_type(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };

    replace::type_with(
        schema_object,
        PrimitiveTypesBitMap::from_schema_value(schema_object.get("type")),
    )
}

/// Removes all the schema keywords that are irrelevant/incongruent with the presence
/// of a specific `type` keyword
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn remove_extraneous_keys_keyword_type(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };

    let primitive_types = PrimitiveTypesBitMap::from_schema_value(schema_object.get("type"));
    if primitive_types.is_empty() {
        false
    } else {
        let mut keys_to_reserve = HashSet::new();
        if primitive_types.contains(PrimitiveType::Array) {
            keys_to_reserve.extend(KEYWORDS_TYPE_ARRAY.iter());
        }
        if primitive_types.contains(PrimitiveType::Boolean) {
            keys_to_reserve.extend(KEYWORDS_TYPE_BOOLEAN.iter());
        }
        if primitive_types.contains(PrimitiveType::Integer) {
            keys_to_reserve.extend(KEYWORDS_TYPE_INTEGER.iter());
        }
        if primitive_types.contains(PrimitiveType::Null) {
            keys_to_reserve.extend(KEYWORDS_TYPE_NULL.iter());
        }
        if primitive_types.contains(PrimitiveType::Number) {
            keys_to_reserve.extend(KEYWORDS_TYPE_NUMBER.iter());
        }
        if primitive_types.contains(PrimitiveType::Object) {
            keys_to_reserve.extend(KEYWORDS_TYPE_OBJECT.iter());
        }
        if primitive_types.contains(PrimitiveType::String) {
            keys_to_reserve.extend(KEYWORDS_TYPE_STRING.iter());
        }

        let removed_keys = preserve_keys(schema_object, &keys_to_reserve);

        replace::type_with(schema_object, primitive_types) || removed_keys
    }
}

#[cfg(test)]
mod tests {
    use super::{optimise_keyword_type, remove_extraneous_keys_keyword_type};
    use super::{
        KEYWORDS_TYPE_ARRAY, KEYWORDS_TYPE_BOOLEAN, KEYWORDS_TYPE_INTEGER, KEYWORDS_TYPE_NULL,
        KEYWORDS_TYPE_NUMBER, KEYWORDS_TYPE_OBJECT, KEYWORDS_TYPE_STRING,
    };
    use crate::constants::KEYWORDS;
    use crate::keywords::update_schema;
    use serde_json::{json, Value};
    use std::collections::HashSet;
    use test_case::test_case;

    #[test]
    fn test_ensure_that_all_keywords_are_included_into_keyword_specific_types() {
        assert_eq!(
            &*KEYWORDS,
            &[].iter()
                .chain(KEYWORDS_TYPE_ARRAY.iter())
                .chain(KEYWORDS_TYPE_BOOLEAN.iter())
                .chain(KEYWORDS_TYPE_INTEGER.iter())
                .chain(KEYWORDS_TYPE_NULL.iter())
                .chain(KEYWORDS_TYPE_NUMBER.iter())
                .chain(KEYWORDS_TYPE_OBJECT.iter())
                .chain(KEYWORDS_TYPE_STRING.iter())
                .cloned()
                .collect::<HashSet<_>>()
        );
    }

    // Eventully add test cases for all the keywords to remove
    #[test_case(&json!({}); "do nothing if type keyword is not present")]
    // {"type": "array", ...}
    #[test_case(&json!({"type": "array"}))]
    #[test_case(&json!({"type": "array", "additionalItems": true}))]
    #[test_case(&json!({"type": "array", "const": ["value"]}))]
    #[test_case(&json!({"type": "array", "contains": [1]}))]
    #[test_case(&json!({"type": "array", "else": true}))]
    #[test_case(&json!({"type": "array", "enum": [["item"]]}))]
    #[test_case(&json!({"type": "array", "if": true}))]
    #[test_case(&json!({"type": "array", "items": {}}))]
    #[test_case(&json!({"type": "array", "maxItems": 1}))]
    #[test_case(&json!({"type": "array", "minItems": 1}))]
    #[test_case(&json!({"type": "array", "then": true}))]
    #[test_case(&json!({"type": "array", "uniqueItems": true}))]
    // {"type": "boolean", ...}
    #[test_case(&json!({"type": "boolean"}))]
    #[test_case(&json!({"type": "boolean", "const": [true]}))]
    #[test_case(&json!({"type": "boolean", "else": true}))]
    #[test_case(&json!({"type": "boolean", "enum": [true]}))]
    #[test_case(&json!({"type": "boolean", "if": true}))]
    #[test_case(&json!({"type": "boolean", "then": true}))]
    // {"type": "integer", ...}
    #[test_case(&json!({"type": "integer"}))]
    #[test_case(&json!({"type": "integer", "const": 1}))]
    #[test_case(&json!({"type": "integer", "else": true}))]
    #[test_case(&json!({"type": "integer", "enum": [1, 2]}))]
    #[test_case(&json!({"type": "integer", "exclusiveMaximum": 1}))]
    #[test_case(&json!({"type": "integer", "exclusiveMinimum": 1}))]
    #[test_case(&json!({"type": "integer", "format": "int32"}))]
    #[test_case(&json!({"type": "integer", "if": true}))]
    #[test_case(&json!({"type": "integer", "maximum": 1}))]
    #[test_case(&json!({"type": "integer", "minimum": 1}))]
    #[test_case(&json!({"type": "integer", "multipleOf": 1}))]
    #[test_case(&json!({"type": "integer", "then": true}))]
    // {"type": "null", ...}
    #[test_case(&json!({"type": "null"}))]
    #[test_case(&json!({"type": "null", "const": [null]}))]
    #[test_case(&json!({"type": "null", "else": true}))]
    #[test_case(&json!({"type": "null", "enum": [null]}))]
    #[test_case(&json!({"type": "null", "if": true}))]
    #[test_case(&json!({"type": "null", "then": true}))]
    // {"type": "number", ...}
    #[test_case(&json!({"type": "number"}))]
    #[test_case(&json!({"type": "number", "const": 1}))]
    #[test_case(&json!({"type": "number", "else": true}))]
    #[test_case(&json!({"type": "number", "enum": [1, 2]}))]
    #[test_case(&json!({"type": "number", "exclusiveMaximum": 1}))]
    #[test_case(&json!({"type": "number", "exclusiveMinimum": 1}))]
    #[test_case(&json!({"type": "number", "format": "int32"}))]
    #[test_case(&json!({"type": "number", "if": true}))]
    #[test_case(&json!({"type": "number", "maximum": 1}))]
    #[test_case(&json!({"type": "number", "minimum": 1}))]
    #[test_case(&json!({"type": "number", "multipleOf": 1}))]
    #[test_case(&json!({"type": "number", "then": true}))]
    // {"type": "object", ...}
    #[test_case(&json!({"type": "object"}))]
    #[test_case(&json!({"type": "object", "additionalProperties": {}}))]
    #[test_case(&json!({"type": "object", "allOf": []}))]
    #[test_case(&json!({"type": "object", "anyOf": []}))]
    #[test_case(&json!({"type": "object", "const": {"key": "value"}}))]
    #[test_case(&json!({"type": "object", "dependencies": []}))]
    #[test_case(&json!({"type": "object", "else": true}))]
    #[test_case(&json!({"type": "object", "enum": [{"key": "value"}]}))]
    #[test_case(&json!({"type": "object", "if": true}))]
    #[test_case(&json!({"type": "object", "maxProperties": 1}))]
    #[test_case(&json!({"type": "object", "minProperties": 1}))]
    #[test_case(&json!({"type": "object", "not": {}}))]
    #[test_case(&json!({"type": "object", "oneOf": []}))]
    #[test_case(&json!({"type": "object", "patternProperties": {}}))]
    #[test_case(&json!({"type": "object", "properties": {}}))]
    #[test_case(&json!({"type": "object", "propertyNames": {}}))]
    #[test_case(&json!({"type": "object", "required": []}))]
    #[test_case(&json!({"type": "object", "then": true}))]
    // {"type": "string", ...}
    #[test_case(&json!({"type": "string"}))]
    #[test_case(&json!({"type": "string", "const": ["key"]}))]
    #[test_case(&json!({"type": "string", "contentEncoding": "base64"}))]
    #[test_case(&json!({"type": "string", "contentMediaType": "application/json"}))]
    #[test_case(&json!({"type": "string", "else": true}))]
    #[test_case(&json!({"type": "string", "enum": ["value"]}))]
    #[test_case(&json!({"type": "string", "format": "date"}))]
    #[test_case(&json!({"type": "string", "if": true}))]
    #[test_case(&json!({"type": "string", "maxLength": 1}))]
    #[test_case(&json!({"type": "string", "minLength": 1}))]
    #[test_case(&json!({"type": "string", "pattern": "key[0-9]+"}))]
    #[test_case(&json!({"type": "string", "then": true}))]
    fn test_remove_extraneous_keys_keyword_type_does_not_remove_keys(schema: &Value) {
        let _ = crate::base_test_keyword_processor(&remove_extraneous_keys_keyword_type, schema);
    }

    // Eventully add test cases for all the keywords to remove
    #[test_case(&json!({}) => json!({}); "do nothing if type keyword is not present")]
    // {"type": "array", ...}
    #[test_case(&json!({"type": "array", "minItems": 1}) => json!({"type": "array", "minItems": 1}))]
    #[test_case(&json!({"type": "array", "minimum": 1}) => json!({"type": "array"}))]
    // {"type": "boolean", ...}
    #[test_case(&json!({"type": "boolean", "enum": [true]}) => json!({"type": "boolean", "enum": [true]}))]
    #[test_case(&json!({"type": "boolean", "minimum": 1}) => json!({"type": "boolean"}))]
    // {"type": "integer", ...}
    #[test_case(&json!({"type": "integer", "minimum": 1}) => json!({"type": "integer", "minimum": 1}))]
    #[test_case(&json!({"type": "integer", "minLength": 1}) => json!({"type": "integer"}))]
    // {"type": "null", ...}
    #[test_case(&json!({"type": "null", "enum": [null]}) => json!({"type": "null", "enum": [null]}))]
    #[test_case(&json!({"type": "null", "minimum": 1}) => json!({"type": "null"}))]
    // {"type": "number", ...}
    #[test_case(&json!({"type": "number", "minimum": 1}) => json!({"type": "number", "minimum": 1}))]
    #[test_case(&json!({"type": "number", "minLength": 1}) => json!({"type": "number"}))]
    // {"type": "object", ...}
    #[test_case(&json!({"type": "object", "minProperties": 1}) => json!({"type": "object", "minProperties": 1}))]
    #[test_case(&json!({"type": "object", "minimum": 1}) => json!({"type": "object"}))]
    // {"type": "string", ...}
    #[test_case(&json!({"type": "string", "minLength": 1}) => json!({"type": "string", "minLength": 1}))]
    #[test_case(&json!({"type": "string", "minItems": 1}) => json!({"type": "string"}))]
    // {"type": [...], ...}
    #[test_case(&json!({"type": ["number", "string"], "minLength": 1}) => json!({"type": ["number", "string"], "minLength": 1}))]
    #[test_case(&json!({"type": ["number", "string"], "minLength": 1, "minItems": 1}) => json!({"type": ["number", "string"], "minLength": 1}))]
    fn test_remove_extraneous_keys_keyword_type_does_remove_keys(schema: &Value) -> Value {
        crate::base_test_keyword_processor(&remove_extraneous_keys_keyword_type, schema)
    }

    #[test_case(&json!({"type": []}) => json!({}))]
    #[test_case(&json!({"type": ["string"]}) => json!({"type": "string"}))]
    #[test_case(&json!({"type": ["integer", "number"]}) => json!({"type": "number"}))]
    #[test_case(&json!({"type": ["integer", "number", "string"]}) => json!({"type": ["number", "string"]}))]
    fn test_optimise_keyword_type(schema: &Value) -> Value {
        crate::base_test_keyword_processor(&optimise_keyword_type, schema)
    }

    #[test_case(&json!({"type": ["number", "integer"], "minLength": 1}) => json!({"type": "number"}))]
    fn test_keywords_elided_with_with_correct_order(schema: &Value) -> Value {
        crate::base_test_keyword_processor(&update_schema, schema)
    }
}
