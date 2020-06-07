//! The reference of the JSON Schema specifications are available on
//! <https://tools.ietf.org/html/draft-handrews-json-schema-validation-01>
mod additional_properties;
mod min_max;
mod required;
mod type_;

use serde_json::{Map, Value};
use std::{collections::HashSet, convert::TryFrom};

lazy_static::lazy_static! {
    /// All keywords of Draft4, Draft6 and Draft7
    pub(crate) static ref KEYWORDS: HashSet<&'static str> = [
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
    ].iter().cloned().collect();

    /// Keywords which contains valid JSON Schema
    ///
    /// This contains the list of keywords defined by the JSON Schema specifications as
    /// * > The value of "..." MUST be a valid JSON Schema.
    /// * > The value of "..." MUST be an object. Each value of this object MUST be a valid JSON Schema.
    /// * > This keyword's value MUST be a non-empty array.  Each item of the array MUST be a valid JSON Schema.
    /// * > The value of "..." MUST be either a valid JSON Schema or an array of valid JSON Schemas.
    static ref KEYWORDS_WITH_SUBSCHEMAS: HashSet<&'static str> = [
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
    ].iter().cloned().collect();

    /// Keywords value MUST be a valid JSON Schema
    ///
    /// This contains the list of keywords defined by the JSON Schema specifications as
    /// > The value of "..." MUST be a valid JSON Schema.
    static ref KEYWORDS_WITH_DIRECT_SUBSCHEMAS: HashSet<&'static str> = [
        "additionalItems",
        "additionalProperties",
        "contains",
        "else",
        "if",
        "not",
        "propertyNames",
        "then",
    ].iter().cloned().collect();
}

/// Order of the methods used to update the schema
///
/// NOTE: The order might be important for the capability/quality of the
/// library so please be mindfull before modifying the order (and if you
/// do so please motivate it in the pull request description)
static UPDATE_SCHEMA_METHODS: &[fn(&mut Value) -> &mut Value] = &[
    min_max::update_min_max_related_keywords,
    type_::optimise_keyword_type_if_array,
    type_::remove_extraneous_keys_keyword_type,
    additional_properties::remove_empty_additional_properties,
    required::remove_empty_required,
];

/// Enum representation of the 7 primitive types recognized by JSON Schema.
///
/// The usage of the enum allows to have a faster processing (less string comparisons)
/// as well as smaller memory footprint as the enum instance uses 2 bytes.
#[derive(Debug, PartialEq)]
enum PrimitiveType {
    Array,
    Boolean,
    Integer,
    Null,
    Number,
    Object,
    String,
}
impl TryFrom<&str> for PrimitiveType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "array" => Ok(Self::Array),
            "boolean" => Ok(Self::Boolean),
            "integer" => Ok(Self::Integer),
            "null" => Ok(Self::Null),
            "number" => Ok(Self::Number),
            "object" => Ok(Self::Object),
            "string" => Ok(Self::String),
            _ => Err(format!(r#""{}" is not a recognized primitive type"#, value)),
        }
    }
}
impl TryFrom<&Value> for PrimitiveType {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        if let Some(value_str) = value.as_str() {
            Self::try_from(value_str)
        } else {
            Err(format!("Expected Value::String(...), found {:?}", value))
        }
    }
}
impl ToString for PrimitiveType {
    fn to_string(&self) -> String {
        match self {
            Self::Array => "array".to_string(),
            Self::Boolean => "boolean".to_string(),
            Self::Integer => "integer".to_string(),
            Self::Null => "null".to_string(),
            Self::Number => "number".to_string(),
            Self::Object => "object".to_string(),
            Self::String => "string".to_string(),
        }
    }
}

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
                if KEYWORDS_WITH_SUBSCHEMAS.contains(&key.as_ref()) {
                    match subschema {
                        Value::Object(subschema_object) => {
                            if KEYWORDS_WITH_DIRECT_SUBSCHEMAS.contains(&key.as_ref()) {
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
    use super::{keywords_to_remove, preserve_keys, update_schema, PrimitiveType, KEYWORDS};
    use serde_json::{json, Value};
    use std::{collections::HashSet, convert::TryFrom};
    use test_case::test_case;

    macro_rules! hash_set {
        ($($elem:expr),* $(,)*) => {
            vec![$($elem),*].iter().cloned().collect::<HashSet<_>>()
        };
    }

    #[test_case("array" => Ok(PrimitiveType::Array))]
    #[test_case("boolean" => Ok(PrimitiveType::Boolean))]
    #[test_case("integer" => Ok(PrimitiveType::Integer))]
    #[test_case("null" => Ok(PrimitiveType::Null))]
    #[test_case("number" => Ok(PrimitiveType::Number))]
    #[test_case("object" => Ok(PrimitiveType::Object))]
    #[test_case("string" => Ok(PrimitiveType::String))]
    #[test_case("something" => Err(r#""something" is not a recognized primitive type"#.to_string()))]
    fn test_from_str_to_primitive_type(value: &str) -> Result<PrimitiveType, String> {
        PrimitiveType::try_from(value)
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
