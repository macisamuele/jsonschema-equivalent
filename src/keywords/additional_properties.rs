use serde_json::Value;

/// Removes empty `additionalProperties` schemas.
#[rule_processor_logger::log_processing]
pub(crate) fn remove_empty_additional_properties(schema: &mut Value) -> &mut Value {
    match schema.get("additionalProperties") {
        Some(Value::Bool(true)) => {
            let _ = schema
                .as_object_mut()
                .expect("As a property exist we're sure that we're dealing with an object")
                .remove("additionalProperties");
        }
        Some(Value::Object(obj)) if obj.is_empty() => {
            let _ = schema
                .as_object_mut()
                .expect("As a property exist we're sure that we're dealing with an object")
                .remove("additionalProperties");
        }
        _ => {}
    };
    schema
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
        remove_empty_additional_properties(&mut schema);
        schema
    }
}
