use crate::{
    helpers::{preserve_keys, replace, types::get_primitive_types},
    primitive_type::PrimitiveType,
};
use serde_json::Value;
use std::collections::HashSet;

lazy_static::lazy_static! {
    static ref KEYWORDS_TYPE_ARRAY: HashSet<&'static str> = [
        "allOf",
        "anyOf",
        "additionalItems",
        "contains",
        "const",
        "enum",
        "items",
        "maxItems",
        "minItems",
        "not",
        "oneOf",
        "type",
        "uniqueItems",
    ].iter().cloned().collect();
    static ref KEYWORDS_TYPE_BOOLEAN: HashSet<&'static str> = [
        "allOf",
        "anyOf",
        "const",
        "enum",
        "type",
        "not",
        "oneOf",
    ].iter().cloned().collect();
    static ref KEYWORDS_TYPE_NULL: HashSet<&'static str> = KEYWORDS_TYPE_BOOLEAN.iter().cloned().collect();
    static ref KEYWORDS_TYPE_INTEGER: HashSet<&'static str> = [
        "allOf",
        "anyOf",
        "const",
        "enum",
        "exclusiveMaximum",
        "exclusiveMinimum",
        "format",
        "maximum",
        "minimum",
        "multipleOf",
        "not",
        "oneOf",
        "type",
    ].iter().cloned().collect();
    static ref KEYWORDS_TYPE_NUMBER: HashSet<&'static str> = KEYWORDS_TYPE_INTEGER.iter().cloned().collect();
    static ref KEYWORDS_TYPE_OBJECT: HashSet<&'static str> = [
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
    ].iter().cloned().collect();
    static ref KEYWORDS_TYPE_STRING: HashSet<&'static str> = [
        "allOf",
        "anyOf",
        "contentMediaType",
        "contentEncoding",
        "const",
        "enum",
        "format",
        "maxLength",
        "minLength",
        "not",
        "oneOf",
        "pattern",
        "type",
    ].iter().cloned().collect();
}

/// Removes duplicated types, avoid not need of list and remove the keyword if all the types are included
#[rule_processor_logger::log_processing]
pub(crate) fn optimise_keyword_type(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };

    replace::type_with(
        schema_object,
        &get_primitive_types(schema_object.get("type")),
    )
}

/// Removes all the schema keywords that are irrelevant/incongruent with the presence
/// of a specific `type` keyword
#[rule_processor_logger::log_processing]
pub(crate) fn remove_extraneous_keys_keyword_type(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };

    let primitive_types = get_primitive_types(schema_object.get("type"));
    if primitive_types.is_empty() {
        false
    } else {
        let mut keys_to_reserve = HashSet::<&'static str>::new();
        for primtive_type in &primitive_types {
            match primtive_type {
                PrimitiveType::Array => KEYWORDS_TYPE_ARRAY.iter().for_each(|key| {
                    let _ = keys_to_reserve.insert(key);
                }),
                PrimitiveType::Boolean => KEYWORDS_TYPE_BOOLEAN.iter().for_each(|key| {
                    let _ = keys_to_reserve.insert(key);
                }),
                PrimitiveType::Integer => KEYWORDS_TYPE_INTEGER.iter().for_each(|key| {
                    let _ = keys_to_reserve.insert(key);
                }),
                PrimitiveType::Null => KEYWORDS_TYPE_NULL.iter().for_each(|key| {
                    let _ = keys_to_reserve.insert(key);
                }),
                PrimitiveType::Number => KEYWORDS_TYPE_NUMBER.iter().for_each(|key| {
                    let _ = keys_to_reserve.insert(key);
                }),
                PrimitiveType::Object => KEYWORDS_TYPE_OBJECT.iter().for_each(|key| {
                    let _ = keys_to_reserve.insert(key);
                }),
                PrimitiveType::String => KEYWORDS_TYPE_STRING.iter().for_each(|key| {
                    let _ = keys_to_reserve.insert(key);
                }),
            }
        }

        let removed_keys = preserve_keys(schema_object, &keys_to_reserve);

        replace::type_with(schema_object, &primitive_types) || removed_keys
    }
}

#[cfg(test)]
mod tests {
    use super::{optimise_keyword_type, remove_extraneous_keys_keyword_type};
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
        crate::init_logger();
        let mut cloned_schema = schema.clone();
        let _ = remove_extraneous_keys_keyword_type(&mut cloned_schema);
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
        crate::init_logger();
        let _ = remove_extraneous_keys_keyword_type(&mut schema);
        schema
    }

    #[test_case(json!({"type": []}) => json!({}))]
    #[test_case(json!({"type": ["string"]}) => json!({"type": "string"}))]
    #[test_case(json!({"type": ["integer", "number"]}) => json!({"type": "number"}))]
    #[test_case(json!({"type": ["integer", "number", "string"]}) => json!({"type": ["number", "string"]}))]
    fn test_optimise_keyword_type(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = optimise_keyword_type(&mut schema);
        schema
    }

    #[test_case(json!({"type": ["number", "integer"], "minLength": 1}) => json!({"type": "number"}))]
    fn test_keywords_elided_with_with_correct_order(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = update_schema(&mut schema);
        schema
    }
}
