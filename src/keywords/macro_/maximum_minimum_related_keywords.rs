use crate::helpers::{replace, types::PrimitiveTypesBitMap};
use crate::primitive_type::PrimitiveType;
use jsonschema_equivalent_rule_processor_logger::log_processing;
use serde_json::Value;

/// This helper method allows to centralise the logic responsible for the update of the schema
/// after the successful identification of incongruent keywords.
/// NOTE: This should only be used by `update_*(&mut Value, &BTreeSet<PrimitiveType>)` methods
/// after they have verified and decided that incongruent keywords are found
fn cleanup_incongruent_keywords(
    schema: &mut Value,
    schema_primitive_types: &mut PrimitiveTypesBitMap,
    primitive_types_to_remove: PrimitiveTypesBitMap,
    keywords_to_remove: &[&str],
) -> bool {
    schema_primitive_types.remove_all(primitive_types_to_remove);
    if schema_primitive_types.is_empty() {
        replace::with_false_schema(schema)
    } else if let Value::Object(schema_object) = schema {
        let mut updated_schema = false;
        for keyword_to_remove in keywords_to_remove {
            updated_schema |= schema_object.remove(*keyword_to_remove).is_some();
        }
        updated_schema
    } else {
        false
    }
}
/// Update schema with incongruent `exclusiveMaximum` and `exclusiveMinimum`.
/// Replaces the schema with `false` schema if `exclusiveMaximum`
/// is smaller than `exclusiveMinimum`
#[log_processing(cfg(feature = "logging"))]
fn update_exclusive_maximum_minimum(
    schema: &mut Value,
    schema_primitive_types: &mut PrimitiveTypesBitMap,
) -> bool {
    // Checking for PrimitiveType::Integer only as PrimitiveType::Number will include integer as well
    if schema_primitive_types.contains(PrimitiveType::Integer) {
        match (
            schema.get("exclusiveMaximum").and_then(Value::as_f64),
            schema.get("exclusiveMinimum").and_then(Value::as_f64),
        ) {
            (Some(max_), Some(min_)) if max_ < min_ => cleanup_incongruent_keywords(
                schema,
                schema_primitive_types,
                PrimitiveTypesBitMap::from(&[PrimitiveType::Integer, PrimitiveType::Number]),
                &["exclusiveMaximum", "exclusiveMinimum"],
            ),
            _ => false,
        }
    } else {
        false
    }
}

/// Update schema with incongruent `maxItems` and `minItems`.
/// Replaces the schema with `false` schema if `maxItems`
/// is smaller than `minItems`
#[log_processing(cfg(feature = "logging"))]
fn update_max_min_items(
    schema: &mut Value,
    schema_primitive_types: &mut PrimitiveTypesBitMap,
) -> bool {
    if schema_primitive_types.contains(PrimitiveType::Array) {
        match (
            schema.get("maxItems").and_then(Value::as_f64),
            schema.get("minItems").and_then(Value::as_f64),
        ) {
            (Some(max_), Some(min_)) if max_ < min_ => cleanup_incongruent_keywords(
                schema,
                schema_primitive_types,
                PrimitiveTypesBitMap::from(PrimitiveType::Array),
                &["maxItems", "minItems"],
            ),
            (_, Some(min_)) if min_ <= 0. => {
                if let Value::Object(schema_object) = schema {
                    let _ = schema_object.remove("minItems");
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    } else {
        false
    }
}

/// Update schema with incongruent `maxLength` and `minLength`.
/// Replaces the schema with `false` schema if `maxLength`
/// is smaller than `minLength`
#[log_processing(cfg(feature = "logging"))]
fn update_max_min_length(
    schema: &mut Value,
    schema_primitive_types: &mut PrimitiveTypesBitMap,
) -> bool {
    if schema_primitive_types.contains(PrimitiveType::String) {
        match (
            schema.get("maxLength").and_then(Value::as_f64),
            schema.get("minLength").and_then(Value::as_f64),
        ) {
            (Some(max_), Some(min_)) if max_ < min_ => cleanup_incongruent_keywords(
                schema,
                schema_primitive_types,
                PrimitiveTypesBitMap::from(PrimitiveType::String),
                &["maxLength", "minLength"],
            ),
            (_, Some(min_)) if min_ <= 0. => {
                if let Value::Object(schema_object) = schema {
                    let _ = schema_object.remove("minLength");
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    } else {
        false
    }
}

/// Update schema with incongruent `maxProperties` and `minProperties`.
/// Replaces the schema with `false` schema if `maxProperties`
/// is smaller than `minProperties`
#[log_processing(cfg(feature = "logging"))]
fn update_max_min_properties(
    schema: &mut Value,
    schema_primitive_types: &mut PrimitiveTypesBitMap,
) -> bool {
    if schema_primitive_types.contains(PrimitiveType::Object) {
        match (
            schema.get("maxProperties").and_then(Value::as_f64),
            schema.get("minProperties").and_then(Value::as_f64),
        ) {
            (Some(max_), Some(min_)) if max_ < min_ => cleanup_incongruent_keywords(
                schema,
                schema_primitive_types,
                PrimitiveType::Object.into(),
                &["maxProperties", "minProperties"],
            ),
            (_, Some(min_)) if min_ <= 0. => {
                if let Value::Object(schema_object) = schema {
                    let _ = schema_object.remove("minProperties");
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    } else {
        false
    }
}

/// Update schema with incongruent `maximum` and `minimum`.
/// Replaces the schema with `false` schema if `maximum`
/// is smaller than `minimum`
#[log_processing(cfg(feature = "logging"))]
fn update_maximum_minimum(
    schema: &mut Value,
    schema_primitive_types: &mut PrimitiveTypesBitMap,
) -> bool {
    // Checking for PrimitiveType::Integer only as PrimitiveType::Number will include integer as well
    if schema_primitive_types.contains(PrimitiveType::Integer) {
        match (
            schema.get("maximum").and_then(Value::as_f64),
            schema.get("minimum").and_then(Value::as_f64),
        ) {
            (Some(max_), Some(min_)) if max_ < min_ => cleanup_incongruent_keywords(
                schema,
                schema_primitive_types,
                PrimitiveTypesBitMap::from(&[PrimitiveType::Integer, PrimitiveType::Number]),
                &["maximum", "minimum"],
            ),
            _ => false,
        }
    } else {
        false
    }
}

/// Update the schema by ensuring that (max-min) relations are satisfiable.
/// If this is not possible then the schema is replaced with a `false` schema.
/// The method interacts with `exclusiveMaximum`, `exclusiveMinimum`, `maxItems`,
/// `maxLength`, `maxProperties`, `maximum`, `minItems`, `minLength`, `minProperties`,
/// `minimum` keywords
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn update_max_min_related_keywords(schema: &mut Value) -> bool {
    let mut updated_schema = false;
    let mut schema_primitive_types = PrimitiveTypesBitMap::from_schema(schema);

    for method in &[
        update_max_min_items,
        update_max_min_length,
        update_max_min_properties,
        update_exclusive_maximum_minimum,
        update_maximum_minimum,
    ] {
        updated_schema |= method(schema, &mut schema_primitive_types);
    }

    if updated_schema {
        if let Value::Object(schema_object) = schema {
            let _ = replace::type_with(schema_object, schema_primitive_types);
        }
    }
    updated_schema
}

#[cfg(test)]
mod tests {
    use super::{
        update_exclusive_maximum_minimum, update_max_min_items, update_max_min_length,
        update_max_min_properties, update_max_min_related_keywords, update_maximum_minimum,
    };
    use crate::helpers::{replace, types::PrimitiveTypesBitMap};

    use serde_json::{json, Value};
    use test_case::test_case;

    fn test(
        keyword_update_logic: fn(&mut Value, &mut PrimitiveTypesBitMap) -> bool,
        schema: &Value,
    ) -> Value {
        crate::base_test_keyword_processor(
            &|schema| {
                let mut schema_primitive_types = PrimitiveTypesBitMap::from_schema(schema);
                let r = keyword_update_logic(schema, &mut schema_primitive_types);
                if let Value::Object(schema_object) = schema {
                    // Do it in the test as the tested methods are only a part of the overall
                    // `update_max_min_related_keywords` and it would perform this operation.
                    let _ = replace::type_with(schema_object, schema_primitive_types);
                }
                r
            },
            schema,
        )
    }

    #[test_case(&json!({"type": "integer", "exclusiveMaximum": 2, "exclusiveMinimum": 1}) => json!({"type": "integer", "exclusiveMaximum": 2, "exclusiveMinimum": 1}))]
    #[test_case(&json!({"type": "integer", "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    #[test_case(&json!({"type": "null", "exclusiveMaximum": 2, "exclusiveMinimum": 1}) => json!({"type": "null", "exclusiveMaximum": 2, "exclusiveMinimum": 1}))]
    #[test_case(&json!({"type": "null", "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!({"type": "null", "exclusiveMaximum": 1, "exclusiveMinimum": 2}))]
    #[test_case(&json!({"type": "number", "exclusiveMaximum": 2, "exclusiveMinimum": 1}) => json!({"type": "number", "exclusiveMaximum": 2, "exclusiveMinimum": 1}))]
    #[test_case(&json!({"type": "number", "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    #[test_case(&json!({"type": ["integer", "null"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": ["null", "number"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": ["integer", "null", "number"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": ["integer", "number"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    fn test_update_exclusive_maximum_minimum(schema: &Value) -> Value {
        test(update_exclusive_maximum_minimum, schema)
    }

    #[test_case(&json!({"type": "array", "maxItems": 2, "minItems": 1}) => json!({"type": "array", "maxItems": 2, "minItems": 1}))]
    #[test_case(&json!({"type": "array", "maxItems": 1, "minItems": 2}) => json!(false))]
    #[test_case(&json!({"type": "null", "maxItems": 2, "minItems": 1}) => json!({"type": "null", "maxItems": 2, "minItems": 1}))]
    #[test_case(&json!({"type": "null", "maxItems": 1, "minItems": 2}) => json!({"type": "null", "maxItems": 1, "minItems": 2}))]
    #[test_case(&json!({"type": ["array", "null"], "maxItems": 1, "minItems": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": "array", "minItems": 0}) => json!({"type": "array"}))]
    #[test_case(&json!({"minItems": 0}) => json!({}))]
    fn test_update_max_min_items(schema: &Value) -> Value {
        test(update_max_min_items, schema)
    }

    #[test_case(&json!({"type": "null", "maxLength": 2, "minLength": 1}) => json!({"type": "null", "maxLength": 2, "minLength": 1}))]
    #[test_case(&json!({"type": "null", "maxLength": 1, "minLength": 2}) => json!({"type": "null", "maxLength": 1, "minLength": 2}))]
    #[test_case(&json!({"type": "string", "maxLength": 2, "minLength": 1}) => json!({"type": "string", "maxLength": 2, "minLength": 1}))]
    #[test_case(&json!({"type": "string", "maxLength": 1, "minLength": 2}) => json!(false))]
    #[test_case(&json!({"type": ["null", "string"], "maxLength": 1, "minLength": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": "string", "minLength": 0}) => json!({"type": "string"}))]
    #[test_case(&json!({"minLength": 0}) => json!({}))]
    fn test_update_max_min_length(schema: &Value) -> Value {
        test(update_max_min_length, schema)
    }

    #[test_case(&json!({"type": "null", "maxProperties": 2, "minProperties": 1}) => json!({"type": "null", "maxProperties": 2, "minProperties": 1}))]
    #[test_case(&json!({"type": "null", "maxProperties": 1, "minProperties": 2}) => json!({"type": "null", "maxProperties": 1, "minProperties": 2}))]
    #[test_case(&json!({"type": "object", "maxProperties": 2, "minProperties": 1}) => json!({"type": "object", "maxProperties": 2, "minProperties": 1}))]
    #[test_case(&json!({"type": "object", "maxProperties": 1, "minProperties": 2}) => json!(false))]
    #[test_case(&json!({"type": ["null", "object"], "maxProperties": 1, "minProperties": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": "object", "minProperties": 0}) => json!({"type": "object"}))]
    #[test_case(&json!({"minProperties": 0}) => json!({}))]
    fn test_update_max_min_properties(schema: &Value) -> Value {
        test(update_max_min_properties, schema)
    }

    #[test_case(&json!({"type": "integer", "maximum": 2, "minimum": 1}) => json!({"type": "integer", "maximum": 2, "minimum": 1}))]
    #[test_case(&json!({"type": "integer", "maximum": 1, "minimum": 2}) => json!(false))]
    #[test_case(&json!({"type": "null", "maximum": 2, "minimum": 1}) => json!({"type": "null", "maximum": 2, "minimum": 1}))]
    #[test_case(&json!({"type": "null", "maximum": 1, "minimum": 2}) => json!({"type": "null", "maximum": 1, "minimum": 2}))]
    #[test_case(&json!({"type": "number", "maximum": 2, "minimum": 1}) => json!({"type": "number", "maximum": 2, "minimum": 1}))]
    #[test_case(&json!({"type": "number", "maximum": 1, "minimum": 2}) => json!(false))]
    #[test_case(&json!({"type": ["integer", "null"], "maximum": 1, "minimum": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": ["null", "number"], "maximum": 1, "minimum": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": ["integer", "null", "number"], "maximum": 1, "minimum": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": ["integer", "number"], "maximum": 1, "minimum": 2}) => json!(false))]
    fn test_update_maximum_minimum(schema: &Value) -> Value {
        test(update_maximum_minimum, schema)
    }

    // Ensure that impossible schemas are not modified if type is not defined
    #[test_case(&json!(false))]
    #[test_case(&json!(null))]
    #[test_case(&json!(true))]
    #[test_case(&json!({}))]
    #[test_case(&json!({"exclusiveMaximum": 1, "exclusiveMinimum": 2}))]
    #[test_case(&json!({"exclusiveMaximum": 2, "exclusiveMinimum": 1}))]
    #[test_case(&json!({"maxItems": 1, "minItems": 2}))]
    #[test_case(&json!({"maxItems": 2, "minItems": 1}))]
    #[test_case(&json!({"maxLength": 1, "minLength": 2}))]
    #[test_case(&json!({"maxLength": 2, "minLength": 1}))]
    #[test_case(&json!({"maxProperties": 1, "minProperties": 2}))]
    #[test_case(&json!({"maxProperties": 2, "minProperties": 1}))]
    #[test_case(&json!({"maximum": 1, "minimum": 2}))]
    #[test_case(&json!({"maximum": 2, "minimum": 1}))]
    // Ensure that incongruent keywords are not modified if not associated to available types
    #[test_case(&json!({"type": "null", "exclusiveMaximum": 1, "exclusiveMinimum": 2}))]
    #[test_case(&json!({"type": "null", "exclusiveMaximum": 2, "exclusiveMinimum": 1}))]
    #[test_case(&json!({"type": "null", "maxItems": 1, "minItems": 2}))]
    #[test_case(&json!({"type": "null", "maxItems": 2, "minItems": 1}))]
    #[test_case(&json!({"type": "null", "maxLength": 1, "minLength": 2}))]
    #[test_case(&json!({"type": "null", "maxLength": 2, "minLength": 1}))]
    #[test_case(&json!({"type": "null", "maxProperties": 1, "minProperties": 2}))]
    #[test_case(&json!({"type": "null", "maxProperties": 2, "minProperties": 1}))]
    #[test_case(&json!({"type": "null", "maximum": 1, "minimum": 2}))]
    #[test_case(&json!({"type": "null", "maximum": 2, "minimum": 1}))]
    fn test_update_max_min_related_keywords_does_not_perform_modifications_if_missing_or_incongruent_type(
        schema: &Value,
    ) {
        let _ = crate::base_test_keyword_processor(&update_max_min_related_keywords, schema);
    }

    // Become a false schema as only the incongruent type is allowed
    #[test_case(&json!({"type": "integer", "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    #[test_case(&json!({"type": "number", "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    #[test_case(&json!({"type": ["integer", "number"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    #[test_case(&json!({"type": "array", "maxItems": 1, "minItems": 2}) => json!(false))]
    #[test_case(&json!({"type": "string", "maxLength": 1, "minLength": 2}) => json!(false))]
    #[test_case(&json!({"type": "object", "maxProperties": 1, "minProperties": 2}) => json!(false))]
    #[test_case(&json!({"type": "integer", "maximum": 1, "minimum": 2}) => json!(false))]
    #[test_case(&json!({"type": "number", "maximum": 1, "minimum": 2}) => json!(false))]
    #[test_case(&json!({"type": ["integer", "number"], "maximum": 1, "minimum": 2}) => json!(false))]
    // The incongruent primitive type is removed)
    #[test_case(&json!({"type": ["integer", "null"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": ["null", "number"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": ["array", "null"], "maxItems": 1, "minItems": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": ["null", "string"], "maxLength": 1, "minLength": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": ["null", "object"], "maxProperties": 1, "minProperties": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": ["integer", "null"], "maximum": 1, "minimum": 2}) => json!({"type": "null"}))]
    #[test_case(&json!({"type": ["null", "number"], "maximum": 1, "minimum": 2}) => json!({"type": "null"}))]
    fn test_update_max_min_related_keywords_does_performs_modifications(schema: &Value) -> Value {
        crate::base_test_keyword_processor(&update_max_min_related_keywords, schema)
    }
}
