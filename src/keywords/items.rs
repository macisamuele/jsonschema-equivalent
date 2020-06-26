use jsonschema_equivalent_rule_processor_logger::log_processing;
use serde_json::Value;

/// Simplify `additionalImtems` keyword by
///  * shrinking `items` keyword if defined as array and longer than `maxItems` keyword
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn simplify_items(schema: &mut Value) -> bool {
    let max_items_len = schema
        .get("maxItems")
        .and_then(Value::as_u64)
        .map_or(usize::MAX, |v| {
            if v >= usize::MAX as u64 {
                usize::MAX
            } else {
                // This is safe because we know that v will never be bigger than usize::MAX
                #[allow(clippy::cast_possible_truncation)]
                {
                    v as usize
                }
            }
        });

    if let Some(Value::Array(items)) = schema.get_mut("items") {
        if items.len() > max_items_len {
            items.truncate(max_items_len);
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
    use super::simplify_items;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({}) => json!({}))]
    #[test_case(&json!({"items": true}) => json!({"items": true}))]
    #[test_case(&json!({"items": [{"type": "boolean"}]}) => json!({"items": [{"type": "boolean"}]}))]
    #[test_case(
        &json!({"items": [{"type": "array"}, {"type": "boolean"}, {"type": "integer"}], "maxItems": 2}) =>
        json!({"items": [{"type": "array"}, {"type": "boolean"}], "maxItems": 2})
    )]
    fn test_simplify_items(schema: &Value) -> Value {
        crate::base_test_keyword_processor(&simplify_items, schema)
    }
}
