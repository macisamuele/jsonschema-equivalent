//! The reference of the JSON Schema specifications are available on
//! <https://tools.ietf.org/html/draft-handrews-json-schema-validation-01>
mod additional_items;
mod additional_properties;
mod all_of;
mod const_;
mod enum_;
mod if_;
mod items;
mod macro_;
mod property_names;
mod required;
mod type_;

use crate::{
    constants::{KEYWORDS_WITH_DIRECT_SUBSCHEMAS, KEYWORDS_WITH_SUBSCHEMAS},
    helpers::{is, replace},
};
use serde_json::Value;

/// Order of the methods used to update the schema
///
/// NOTE: The order might be important for the capability/quality of the
/// library so please be mindfull before modifying the order (and if you
/// do so please motivate it in the pull request description)
static UPDATE_SCHEMA_METHODS: &[fn(&mut Value) -> bool] = &[
    // `remove_extraneous_keys_keyword_type` and `remove_keywords_in_must_ignore_groups`
    // is added first as it quickly reduces the amount of keywords to process
    type_::remove_extraneous_keys_keyword_type,
    macro_::ignore_keywords::remove_keywords_in_must_ignore_groups,
    // All others, currently no special ordering is defined
    additional_items::simplify_additional_items,
    additional_properties::simplify_additional_properties,
    const_::simple_const_cleanup,
    enum_::simple_enum_cleanup,
    if_::simplify_if,
    items::simplify_items,
    macro_::maximum_minimum_related_keywords::update_max_min_related_keywords,
    property_names::optimise_property_names,
    required::remove_empty_required,
    type_::optimise_keyword_type,
    // Mutli schema handling/merges needs to be done at the end
    all_of::flatten_all_of,
    all_of::simplify_all_of,
];

/// Perform the schema optimisaton without descending the schema
fn update_schema_no_recursive(schema: &mut Value) -> bool {
    let mut updated_schema = false;
    for method in UPDATE_SCHEMA_METHODS {
        if method(schema) {
            updated_schema = true;
        }
        if &Value::Bool(true) == schema {
            // If the schema is a `true` or `false` schema
            // we know that we cannot optimise it even more
            return true;
        }
    }
    updated_schema
}

/// Discend the schema and optimise it.
/// Return true if schema modifications have been performed
pub(crate) fn update_schema(schema: &mut Value) -> bool {
    let mut updated_schema = false;
    if is::true_schema(schema) {
        return replace::with_true_schema(schema);
    } else if let Value::Object(schema_object) = schema {
        for (key, subschema) in schema_object {
            if KEYWORDS_WITH_SUBSCHEMAS.contains(&key.as_ref()) {
                match subschema {
                    Value::Object(subschema_object) => {
                        if KEYWORDS_WITH_DIRECT_SUBSCHEMAS.contains(&key.as_ref()) {
                            // In case of schemas where the keyword value MUST be a valid JSON Schema
                            // ie. `{"additionalProperties": {"type": "string"}}`
                            updated_schema |= update_schema(subschema);
                        } else {
                            // In case of schemas where the keyword holds a JSON Object and its
                            // values MUST be a valid JSON Schema
                            // ie. `{"properties": {"property" {"type": "string"}}}`
                            for subschema_value in subschema_object.values_mut() {
                                updated_schema |= update_schema(subschema_value);
                            }
                        }
                    }
                    Value::Array(subschema_array) => {
                        // In case of schemas where the keyword holds a JSON Array and its
                        // values MUST be a valid JSON Schema
                        // ie. `{"allOf": [{"type": "string"}]}`
                        for subschema_value in subschema_array {
                            updated_schema |= update_schema(subschema_value);
                        }
                    }
                    _ => {}
                }
                updated_schema |= update_schema(subschema);
            }
        }

        updated_schema |= update_schema_no_recursive(schema);
    }
    updated_schema
}

#[cfg(test)]
mod tests {
    use super::update_schema;
    use serde_json::{json, Value};

    use test_case::test_case;

    #[test_case(&json!({}) => json!(true))]
    #[test_case(&json!({"properties": {"prop": {"type": "string", "minimum": 1}}}) => json!({"properties": {"prop": {"type": "string"}}}))]
    #[test_case(&json!({"allOf": [{"type": "string", "minimum": 1}]}) => json!({"type": "string"}))]
    #[test_case(
        &json!({"allOf": [{"properties": {"bar": {"type": "integer"}}, "required": ["bar"]}, {"properties": {"foo": {"type": "string"}}, "required": ["foo"]}]})
        => json!({"allOf": [{"properties": {"bar": {"type":"integer"}}, "required": ["bar"]}, {"properties": {"foo": {"type": "string"}}, "required": ["foo"]}], "required": ["bar", "foo"]})
    )]
    fn test_update_schema_descend_schema(schema: &Value) -> Value {
        crate::base_test_keyword_processor(&update_schema, schema)
    }
}
