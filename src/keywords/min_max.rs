use crate::helpers::replace;
use serde_json::Value;

/// Update schema with incongruent `exclusiveMaximum` and `exclusiveMinimum`.
/// Replaces the schema with `false` schema if `exclusiveMaximum`
/// is smaller than `exclusiveMinimum`
#[rule_processor_logger::log_processing]
fn update_exclusive_maximum_minimum(schema: &mut Value) -> bool {
    match (
        schema.get("exclusiveMaximum").map(Value::as_f64),
        schema.get("exclusiveMinimum").map(Value::as_f64),
    ) {
        (Some(max_), Some(min_)) if max_ < min_ => {
            replace::with_false_schema(schema);
            true
        }
        _ => false
    }
}

/// Update schema with incongruent `maxItems` and `minItems`.
/// Replaces the schema with `false` schema if `maxItems`
/// is smaller than `minItems`
#[rule_processor_logger::log_processing]
fn update_max_min_items(schema: &mut Value) -> bool {
    match (
        schema.get("maxItems").map(Value::as_f64),
        schema.get("minItems").map(Value::as_f64),
    ) {
        (Some(max_), Some(min_)) if max_ < min_ => {
            replace::with_false_schema(schema);
            true
        }
        _ => false
    }
}

/// Update schema with incongruent `maxLength` and `minLength`.
/// Replaces the schema with `false` schema if `maxLength`
/// is smaller than `minLength`
#[rule_processor_logger::log_processing]
fn update_max_min_length(schema: &mut Value) -> bool {
    match (
        schema.get("maxLength").map(Value::as_f64),
        schema.get("minLength").map(Value::as_f64),
    ) {
        (Some(max_), Some(min_)) if max_ < min_ => {
            replace::with_false_schema(schema);
            true
        }
        _ => false
    }
}

/// Update schema with incongruent `maxProperties` and `minProperties`.
/// Replaces the schema with `false` schema if `maxProperties`
/// is smaller than `minProperties`
#[rule_processor_logger::log_processing]
fn update_max_min_properties(schema: &mut Value) -> bool {
    match (
        schema.get("maxProperties").map(Value::as_f64),
        schema.get("minProperties").map(Value::as_f64),
    ) {
        (Some(max_), Some(min_)) if max_ < min_ => {
            replace::with_false_schema(schema);
            true
        }
        _ => false
    }
}

/// Update schema with incongruent `maximum` and `minimum`.
/// Replaces the schema with `false` schema if `maximum`
/// is smaller than `minimum`
#[rule_processor_logger::log_processing]
fn update_maximum_minimum(schema: &mut Value) -> bool {
    match (
        schema.get("maximum").map(Value::as_f64),
        schema.get("minimum").map(Value::as_f64),
    ) {
        (Some(max_), Some(min_)) if max_ < min_ => {
            replace::with_false_schema(schema);
            true
        }
        _ => false
    }
}

/// Update the schema by ensuring that (max-min) relations are satisfiable.
/// If this is not possible then the schema is replaced with a `false` schema.
/// The method interacts with `exclusiveMaximum`, `exclusiveMinimum`, `maxItems`,
/// `maxLength`, `maxProperties`, `maximum`, `minItems`, `minLength`, `minProperties`,
/// `minimum` keywords
#[rule_processor_logger::log_processing]
pub(crate) fn update_min_max_related_keywords(schema: &mut Value) -> bool {
    if schema.get("type").is_some() {
        // We're applying the keyword updates only if `type` keyword is present because
        // `[1, 2, 3]` would be valid against a schema like
        // `{"maximum": 1, "minimum": 2, "minItems": 1}`
        let mut schema_updated = false;
        for method in &[
            update_max_min_items,
            update_max_min_length,
            update_max_min_properties,
            update_exclusive_maximum_minimum,
            update_maximum_minimum,
        ] {
            schema_updated |= method(schema);
        }
        schema_updated
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{
        update_exclusive_maximum_minimum, update_max_min_items, update_max_min_length,
        update_max_min_properties, update_maximum_minimum, update_min_max_related_keywords,
    };
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({}) => json!({}))]
    #[test_case(json!({"exclusiveMaximum": 2, "exclusiveMinimum": 1}) => json!({"exclusiveMaximum": 2, "exclusiveMinimum": 1}))]
    #[test_case(json!({"exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    fn test_update_exclusive_maximum_minimum(mut value: Value) -> Value {
        crate::init_logger();
        let _ = update_exclusive_maximum_minimum(&mut value);
        value
    }

    #[test_case(json!({}) => json!({}))]
    #[test_case(json!({"maxItems": 2, "minItems": 1}) => json!({"maxItems": 2, "minItems": 1}))]
    #[test_case(json!({"maxItems": 1, "minItems": 2}) => json!(false))]
    fn test_update_max_min_items(mut value: Value) -> Value {
        crate::init_logger();
        let _ = update_max_min_items(&mut value);
        value
    }

    #[test_case(json!({}) => json!({}))]
    #[test_case(json!({"maxLength": 2, "minLength": 1}) => json!({"maxLength": 2, "minLength": 1}))]
    #[test_case(json!({"maxLength": 1, "minLength": 2}) => json!(false))]
    fn test_update_max_min_length(mut value: Value) -> Value {
        crate::init_logger();
        let _ = update_max_min_length(&mut value);
        value
    }

    #[test_case(json!({}) => json!({}))]
    #[test_case(json!({"maxProperties": 2, "minProperties": 1}) => json!({"maxProperties": 2, "minProperties": 1}))]
    #[test_case(json!({"maxProperties": 1, "minProperties": 2}) => json!(false))]
    fn test_update_max_min_properties(mut value: Value) -> Value {
        crate::init_logger();
        let _ = update_max_min_properties(&mut value);
        value
    }

    #[test_case(json!({}) => json!({}))]
    #[test_case(json!({"maximum": 2, "minimum": 1}) => json!({"maximum": 2, "minimum": 1}))]
    #[test_case(json!({"maximum": 1, "minimum": 2}) => json!(false))]
    fn test_update_maximum_minimum(mut value: Value) -> Value {
        crate::init_logger();
        let _ = update_maximum_minimum(&mut value);
        value
    }

    // No changes into max-min relations if `type` keyword is not defined
    #[test_case(json!({"maximum": 1, "minimum": 2}) => json!({"maximum": 1, "minimum": 2}))]
    // The schema is impossible and so equivalent to `false` schema
    #[test_case(json!({"type": "number", "maximum": 1, "minimum": 2}) => json!(false))]
    fn test_update_min_max_related_keywords(mut value: Value) -> Value {
        crate::init_logger();
        let _ = update_min_max_related_keywords(&mut value);
        value
    }
}
