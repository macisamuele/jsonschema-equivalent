use crate::helpers::is;
use jsonschema_equivalent_rule_processor_logger::log_processing;
use serde_json::Value;

/// Simplify `if` keyword group by embedding the content of `then` or `else` schema
/// into `allOf` in case of deterministic validation or removing it if `then` or `else` are missing.
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn simplify_if(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };

    if let Some(if_schema) = schema_object.get("if") {
        if is::false_schema(if_schema) {
            // In case of a `false` schema we know that the `then` schema will never be considered
            // so we can simplify the schema by ensuring that the `else` schema has to be verified
            let _ = schema_object.remove("if");
            let _ = schema_object.remove("then");
            if let Some(else_schema) = schema_object.remove("else") {
                if let Value::Array(mut all_of_items) = schema_object
                    .remove("allOf")
                    .unwrap_or_else(|| Value::Array(Vec::with_capacity(1)))
                {
                    all_of_items.push(else_schema);
                    let _ = schema_object.insert("allOf".to_string(), Value::Array(all_of_items));
                }
            }
            true
        } else if is::true_schema(if_schema) {
            // In case of a `true` schema we know that the `then` schema will never be considered
            // so we can simplify the schema by ensuring that the `then` schema has to be verified
            let _ = schema_object.remove("if");
            let _ = schema_object.remove("else");
            if let Some(else_schema) = schema_object.remove("then") {
                if let Value::Array(mut all_of_items) = schema_object
                    .remove("allOf")
                    .unwrap_or_else(|| Value::Array(Vec::with_capacity(1)))
                {
                    all_of_items.push(else_schema);
                    let _ = schema_object.insert("allOf".to_string(), Value::Array(all_of_items));
                }
            }
            true
        } else if !schema_object.contains_key("else") && !schema_object.contains_key("then") {
            let _ = schema_object.remove("if");
            true
        } else {
            false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::simplify_if;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({}) => json!({}))]
    #[test_case(&json!({"if": false, "then": {"minLength": 0}, "else": {"maxLength": 0}}) => json!({"allOf": [{"maxLength": 0}]}))]
    #[test_case(&json!({"if": true, "then": {"minLength": 0}, "else": {"maxLength": 0}}) => json!({"allOf": [{"minLength": 0}]}))]
    #[test_case(&json!({"if": {"type": "string"}, "then": {"minLength": 0}, "else": {"maxLength": 0}}) => json!({"if": {"type": "string"}, "then": {"minLength": 0}, "else": {"maxLength": 0}}))]
    #[test_case(&json!({"if": false}) => json!({}))]
    #[test_case(&json!({"if": true}) => json!({}))]
    #[test_case(&json!({"if": {"type": "string"}}) => json!({}))]
    fn test_simplify_if(schema: &Value) -> Value {
        crate::base_test_keyword_processor(&simplify_if, schema)
    }
}
