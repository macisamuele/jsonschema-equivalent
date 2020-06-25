use jsonschema_equivalent_rule_processor_logger::log_processing;
use serde_json::Value;

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

#[cfg(test)]
mod tests {
    use super::remove_keywords_in_must_ignore_groups;
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
}
