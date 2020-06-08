//! The reference of the JSON Schema specifications are available on
//! <https://tools.ietf.org/html/draft-handrews-json-schema-validation-01>
mod additional_properties;
mod min_max;
mod required;
mod type_;

use crate::constants::{KEYWORDS_WITH_DIRECT_SUBSCHEMAS, KEYWORDS_WITH_SUBSCHEMAS};
use serde_json::Value;

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
    use super::update_schema;
    use serde_json::{json, Value};

    use test_case::test_case;

    #[test_case(json!({}) => json!({}))]
    #[test_case(json!({"properties": {"prop": {"type": "string", "minimum": 1}}}) => json!({"properties": {"prop": {"type": "string"}}}))]
    #[test_case(json!({"allOf": [{"type": "string", "minimum": 1}]}) => json!({"allOf": [{"type": "string"}]}))]
    fn test_update_schema_descend_schema(mut schema: Value) -> Value {
        update_schema(&mut schema);
        schema
    }
}
