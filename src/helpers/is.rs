use serde_json::Value;

/// Checks if the input schema is a `false` schema
#[inline]
pub(crate) fn false_schema(schema: &Value) -> bool {
    match schema {
        Value::Bool(false) => true,
        _ => false,
    }
}

/// Checks if the input schema is a `true` schema
#[inline]
pub(crate) fn true_schema(schema: &Value) -> bool {
    match schema {
        Value::Bool(true) => true,
        Value::Object(obj) if obj.is_empty() => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{false_schema, true_schema};
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({}) => false)]
    #[test_case(&json!({"type": "string"}) => false)]
    #[test_case(&json!(false) => true)]
    #[test_case(&json!(true) => false)]
    fn test_false_schema(schema: &Value) -> bool {
        false_schema(schema)
    }

    #[test_case(&json!({}) => true)]
    #[test_case(&json!({"type": "string"}) => false)]
    #[test_case(&json!(false) => false)]
    #[test_case(&json!(true) => true)]
    fn test_true_schema(schema: &Value) -> bool {
        true_schema(schema)
    }
}
