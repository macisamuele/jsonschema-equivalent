use crate::helpers::is;
use jsonschema_equivalent_rule_processor_logger::log_processing;
use serde_json::Value;
use std::collections::HashMap;

/// Removes keywords that have meaning only if a "parent" keyword is defined
/// Examples are:
/// * `then` or `else` keywords have no meaning if `if` keyword is not defined
/// * `additionalItems` keyword have meaning only if `items` keyword is defined
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn remove_keywords_in_must_ignore_groups(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };

    let mut updated_schema = false;

    macro_rules! define_parent_child_relation {
        ($($parent: literal => $child: literal),*,) => {
            $(if schema_object.contains_key($child) && !schema_object.contains_key($parent) {
                let _ = schema_object.remove($child);
                #[allow(clippy::useless_let_if_seq)]  // Clippy override needed to allow the definition of a simple macro, not defining
                {                                     // the override would require considering the first case as an exception
                    updated_schema = true;
                }
            })*
        };
    }
    define_parent_child_relation!(
        "if" => "else",
        "if" => "then",
        "items" => "additionalItems",
    );

    updated_schema
}

/// Check if the input value is a number and if it is equal to zero
#[inline]
fn value_is_zero(value: &Value) -> bool {
    value.as_f64().map_or(false, |number| number == 0.0)
}

/// Check if the input value is a an empty array
#[inline]
fn value_is_empty_array(value: &Value) -> bool {
    if let Value::Array(array) = value {
        array.is_empty()
    } else {
        false
    }
}

/// Check if the input value is a an empty object
#[inline]
fn value_is_empty_object(value: &Value) -> bool {
    if let Value::Object(object) = value {
        object.is_empty()
    } else {
        false
    }
}

lazy_static::lazy_static! {
    static ref KEYWORD_TO_OMIT_CHECK: HashMap<&'static str, for<'r> fn(&'r Value) -> bool> = {
        let mut res: HashMap<&'static str, for<'r> fn(&'r Value) -> bool> = HashMap::new();
        let _ = res.insert("additionalItems", is::true_schema);
        let _ = res.insert("additionalProperties", is::true_schema);
        let _ = res.insert("dependencies", value_is_empty_object); // If schema is valid it would be equivalent to `is::true_schema`, but we don't want to make assumptions
        let _ = res.insert("else", is::true_schema);
        let _ = res.insert("items", is::true_schema);
        let _ = res.insert("minItems", value_is_zero);
        let _ = res.insert("minLength", value_is_zero);
        let _ = res.insert("minProperties", value_is_zero);
        let _ = res.insert("patternProperties", value_is_empty_object); // If schema is valid it would be equivalent to `is::true_schema`, but we don't want to make assumptions
        let _ = res.insert("properties", value_is_empty_object); // If schema is valid it would be equivalent to `is::true_schema`, but we don't want to make assumptions
        let _ = res.insert("propertyNames", value_is_empty_object); // If schema is valid it would be equivalent to `is::true_schema`, but we don't want to make assumptions
        let _ = res.insert("required", value_is_empty_array);
        let _ = res.insert("then", is::true_schema);
        let _ = res.insert("uniqueItems", is::false_schema);
        res
    };
}

/// Remove keywords whose definition does not alter the schema respect not having the
/// keywords defined.
/// Examples are:
///  * `additionalItems`, `additionalProperties`, `then`, `else`, set to a `true` schema
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn omit_keywords_that_do_not_alter_schema_selectivity(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };

    let keywords_to_remove: Vec<&&str> = KEYWORD_TO_OMIT_CHECK
        .iter()
        .filter_map(|(property, omit_check)| {
            if let Some(subschema) = schema_object.get(*property) {
                if omit_check(subschema) {
                    Some(property)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    for keyword_to_remove in &keywords_to_remove {
        let _ = schema_object.remove(**keyword_to_remove);
    }
    !keywords_to_remove.is_empty()
}

#[cfg(test)]
mod tests {
    use super::{
        omit_keywords_that_do_not_alter_schema_selectivity, remove_keywords_in_must_ignore_groups,
    };
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({}) => json!({}))]
    #[test_case(&json!({"additionalItems": true, "items": true}) => json!({"additionalItems": true, "items": true}))]
    #[test_case(&json!({"additionalItems": true}) => json!({}))]
    #[test_case(&json!({"else": true, "if": true}) => json!({"else": true, "if": true}))]
    #[test_case(&json!({"else": true}) => json!({}))]
    #[test_case(&json!({"then": true, "if": true}) => json!({"then": true, "if": true}))]
    #[test_case(&json!({"then": true}) => json!({}))]
    fn test_remove_keywords_in_must_ignore_groups(value: &Value) -> Value {
        crate::base_test_keyword_processor(&remove_keywords_in_must_ignore_groups, value)
    }

    #[test_case(&json!({"additionalItems": {"type": "string"}}) => json!({"additionalItems": {"type": "string"}}))]
    #[test_case(&json!({"additionalItems": {}}) => json!({}))]
    #[test_case(&json!({"additionalItems": true}) => json!({}))]
    #[test_case(&json!({"additionalProperties": {"type": "string"}}) => json!({"additionalProperties": {"type": "string"}}))]
    #[test_case(&json!({"additionalProperties": {}}) => json!({}))]
    #[test_case(&json!({"additionalProperties": true}) => json!({}))]
    #[test_case(&json!({"dependencies": {"p1": ["p2", "p3"]}}) => json!({"dependencies": {"p1": ["p2", "p3"]}}))]
    #[test_case(&json!({"dependencies": {}}) => json!({}))]
    #[test_case(&json!({"else": {"type": "string"}}) => json!({"else": {"type": "string"}}))]
    #[test_case(&json!({"else": {}}) => json!({}))]
    #[test_case(&json!({"else": true}) => json!({}))]
    #[test_case(&json!({"items": {"type": "string"}}) => json!({"items": {"type": "string"}}))]
    #[test_case(&json!({"items": []}) => json!({"items": []}))]
    #[test_case(&json!({"items": {}}) => json!({}))]
    #[test_case(&json!({"items": true}) => json!({}))]
    #[test_case(&json!({"minItems": 0}) => json!({}))]
    #[test_case(&json!({"minItems": 1}) => json!({"minItems": 1}))]
    #[test_case(&json!({"minLength": 0}) => json!({}))]
    #[test_case(&json!({"minLength": 1}) => json!({"minLength": 1}))]
    #[test_case(&json!({"minProperties": 0}) => json!({}))]
    #[test_case(&json!({"minProperties": 1}) => json!({"minProperties": 1}))]
    #[test_case(&json!({"patternProperties": {"p1": {"type": "string"}}}) => json!({"patternProperties": {"p1": {"type": "string"}}}))]
    #[test_case(&json!({"patternProperties": {}}) => json!({}))]
    #[test_case(&json!({"properties": {"p1": {"type": "string"}}}) => json!({"properties": {"p1": {"type": "string"}}}))]
    #[test_case(&json!({"properties": {}}) => json!({}))]
    #[test_case(&json!({"propertyNames": {"minLength": 1}}) => json!({"propertyNames": {"minLength": 1}}))]
    #[test_case(&json!({"propertyNames": {}}) => json!({}))]
    #[test_case(&json!({"required": ["p1"]}) => json!({"required": ["p1"]}))]
    #[test_case(&json!({"required": []}) => json!({}))]
    #[test_case(&json!({"then": {"type": "string"}}) => json!({"then": {"type": "string"}}))]
    #[test_case(&json!({"then": {}}) => json!({}))]
    #[test_case(&json!({"then": true}) => json!({}))]
    #[test_case(&json!({"uniqueItems": false}) => json!({}))]
    #[test_case(&json!({"uniqueItems": true}) => json!({"uniqueItems": true}))]
    fn test_omit_keywords_that_do_not_alter_schema_selectivity(value: &Value) -> Value {
        crate::base_test_keyword_processor(
            &omit_keywords_that_do_not_alter_schema_selectivity,
            value,
        )
    }
}
