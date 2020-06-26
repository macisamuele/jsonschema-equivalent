use crate::helpers::is;
use jsonschema_equivalent_rule_processor_logger::log_processing;
use serde_json::{map::Entry, Value};

/// Simplify `additionalItems` keyword by
///  * removing the keyword if the schema is a `true` schema
///  * removing the keyword if `items` is not an array (as `additionalItems` should be ignored in such case)
///  * removing the keyword if `items` is an array whose length is not smaller (>=) than `maxItems`
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn simplify_additional_items(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };
    if let Some(additional_items) = schema_object.get("additionalItems") {
        if is::true_schema(additional_items) {
            let _ = schema_object.remove("additionalItems");
            true
        } else {
            match schema_object.get("items") {
                Some(Value::Object(_)) => {
                    let _ = schema_object.remove("additionalItems");
                    true
                }
                Some(Value::Array(items)) => {
                    let max_items_len = schema_object
                        .get("maxItems")
                        .and_then(Value::as_u64)
                        .unwrap_or(u64::MAX);
                    if is::false_schema(additional_items) {
                        // We know that we can never have additional items, as no value
                        // can be validated correctly. This means that it is equivalent to
                        // have `maxItems` defined to maximum the length of items
                        let items_len = items.len();
                        let _ = schema_object.remove("additionalItems");
                        match schema_object.entry("maxItems") {
                            Entry::Vacant(entry) => {
                                let _ = entry.insert(items_len.into());
                            }
                            Entry::Occupied(mut entry) => {
                                if max_items_len > items_len as u64 {
                                    let _ = entry.insert(items_len.into());
                                }
                            }
                        }
                        true
                    } else if max_items_len <= items.len() as u64 {
                        let _ = schema_object.remove("additionalItems");
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::simplify_additional_items;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&json!({}) => json!({}))]
    #[test_case(&json!({"additionalItems": true}) => json!({}))]
    #[test_case(&json!({"additionalItems": {"type": "string"}, "items": {"type": "boolean"}}) => json!({"items": {"type": "boolean"}}))]
    #[test_case(&json!({"additionalItems": {}}) => json!({}))]
    #[test_case(&json!({"additionalItems": false, "items": [true, true, true]}) => json!({"items": [true, true, true], "maxItems": 3}))]
    #[test_case(&json!({"additionalItems": false, "items": [true, true, true], "maxItems": 4}) => json!({"items": [true, true, true], "maxItems": 3}))]
    #[test_case(&json!({"additionalItems": false, "items": [true, true, true], "maxItems": 2}) => json!({"items": [true, true, true], "maxItems": 2}))]
    #[test_case(&json!({"additionalItems": {"type": "boolean"}, "items": [true, true, true], "maxItems": 2}) => json!({"items": [true, true, true], "maxItems": 2}))]
    #[test_case(&json!({"additionalItems": {"type": "boolean"}, "items": [true, true, true], "maxItems": 3}) => json!({"items": [true, true, true], "maxItems": 3}))]
    #[test_case(&json!({"additionalItems": {"type": "boolean"}, "items": [true, true, true], "maxItems": 4}) => json!({"additionalItems": {"type": "boolean"}, "items": [true, true, true], "maxItems": 4}))]
    fn test_simplify_additional_items(schema: &Value) -> Value {
        crate::base_test_keyword_processor(&simplify_additional_items, schema)
    }
}
