use crate::helpers::is;
use jsonschema_equivalent_rule_processor_logger::log_processing;
use serde_json::Value;

/// Removes empty `additionalProperties` schemas.
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn remove_empty_additional_properties(schema: &mut Value) -> bool {
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
    use super::remove_empty_additional_properties;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({}) => json!({}))]
    #[test_case(json!({"additionalProperties": true}) => json!({}))]
    #[test_case(json!({"additionalProperties": {}}) => json!({}))]
    #[test_case(json!({"additionalProperties": false}) => json!({"additionalProperties": false}))]
    fn test_remove_empty_additional_properties(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = remove_empty_additional_properties(&mut schema);
        schema
    }
}
