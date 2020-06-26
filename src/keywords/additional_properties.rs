use crate::helpers::is;
use jsonschema_equivalent_rule_processor_logger::log_processing;
use serde_json::Value;

/// Simplify `additionalProperties` keyword by:
///  * removing the keyword if the schema is a `true` schema
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn simplify_additional_properties(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };
    if schema_object
        .get("additionalProperties")
        .map_or(false, is::true_schema)
    {
        let _ = schema_object.remove("additionalProperties");
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::simplify_additional_properties;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({}) => json!({}))]
    #[test_case(&json!({"additionalProperties": true}) => json!({}))]
    #[test_case(&json!({"additionalProperties": {}}) => json!({}))]
    #[test_case(&json!({"additionalProperties": false}) => json!({"additionalProperties": false}))]
    fn test_simplify_additional_properties(schema: &Value) -> Value {
        crate::base_test_keyword_processor(&simplify_additional_properties, schema)
    }
}
