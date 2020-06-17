use serde_json::Value;

/// Removes empty `required` schemas.
#[rule_processor_logger::log_processing]
pub(crate) fn remove_empty_required(schema: &mut Value) -> &mut Value {
    match schema.get("required") {
        Some(Value::Array(array)) if array.is_empty() => {
            let _ = schema
                .as_object_mut()
                .expect("As a property exist we're sure that we're dealing with an object")
                .remove("required");
        }
        _ => {}
    };
    schema
}

#[cfg(test)]
mod tests {
    use super::remove_empty_required;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({}) => json!({}))]
    #[test_case(json!({"required": []}) => json!({}))]
    #[test_case(json!({"required": ["key"]}) => json!({"required": ["key"]}))]
    fn test_remove_empty_required(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = remove_empty_required(&mut schema);
        schema
    }
}
