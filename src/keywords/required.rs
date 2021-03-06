use jsonschema_equivalent_rule_processor_logger::log_processing;
use serde_json::Value;

/// Removes empty `required` schemas.
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn remove_empty_required(schema: &mut Value) -> bool {
    match schema.get("required") {
        Some(Value::Array(array)) if array.is_empty() => {
            let _ = schema
                .as_object_mut()
                .expect("As a property exist we're sure that we're dealing with an object")
                .remove("required");
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::remove_empty_required;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({}) => json!({}))]
    #[test_case(&json!({"required": []}) => json!({}))]
    #[test_case(&json!({"required": ["key"]}) => json!({"required": ["key"]}))]
    fn test_remove_empty_required(schema: &Value) -> Value {
        crate::base_test_keyword_processor(&remove_empty_required, schema)
    }
}
