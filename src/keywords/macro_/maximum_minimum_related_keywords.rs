use crate::helpers::{replace, types::PrimitiveTypesBitMap};
use crate::primitive_type::PrimitiveType;
use serde_json::Value;

/// This helper method allows to centralise the logic responsible for the update of the schema
/// after the successful identification of incongruent keywords.
/// NOTE: This should only be used by `update_*(&mut Value, &BTreeSet<PrimitiveType>)` methods
/// after they have verified and decided that incongruent keywords are found
fn cleanup_incongruent_keywords(
    schema: &mut Value,
    schema_primitive_types: PrimitiveTypesBitMap,
    primitive_type_to_remove: PrimitiveType,
    keywords_to_remove: &[&str],
) -> bool {
    if schema_primitive_types.has_other_primitive_types_other_than(primitive_type_to_remove) {
        let mut final_primitive_types = schema_primitive_types;
        final_primitive_types.remove(primitive_type_to_remove);
        if let Value::Object(schema_object) = schema {
            if replace::type_with(schema_object, final_primitive_types) {
                for keyword_to_remove in keywords_to_remove {
                    let _ = schema_object.remove(*keyword_to_remove);
                }
                true
            } else {
                false
            }
        } else {
            // This is impossible as we know that `schema` is a JSON Object because we were able to extract keywords
            false
        }
    } else {
        replace::with_false_schema(schema)
    }
}
/// Update schema with incongruent `exclusiveMaximum` and `exclusiveMinimum`.
/// Replaces the schema with `false` schema if `exclusiveMaximum`
/// is smaller than `exclusiveMinimum`
#[rule_processor_logger::log_processing]
fn update_exclusive_maximum_minimum(
    schema: &mut Value,
    schema_primitive_types: &PrimitiveTypesBitMap,
) -> bool {
    // Checking for PrimitiveType::Integer only as PrimitiveType::Number will include integer as well
    if schema_primitive_types.contains(PrimitiveType::Integer) {
        match (
            schema.get("exclusiveMaximum").map(Value::as_f64),
            schema.get("exclusiveMinimum").map(Value::as_f64),
        ) {
            (Some(max_), Some(min_)) if max_ < min_ => cleanup_incongruent_keywords(
                schema,
                *schema_primitive_types,
                PrimitiveType::Number,
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
#[rule_processor_logger::log_processing]
fn update_max_min_items(schema: &mut Value, schema_primitive_types: &PrimitiveTypesBitMap) -> bool {
    if schema_primitive_types.contains(PrimitiveType::Array) {
        match (
            schema.get("maxItems").map(Value::as_f64),
            schema.get("minItems").map(Value::as_f64),
        ) {
            (Some(max_), Some(min_)) if max_ < min_ => cleanup_incongruent_keywords(
                schema,
                *schema_primitive_types,
                PrimitiveType::Array,
                &["maxItems", "minItems"],
            ),
            _ => false,
        }
    } else {
        false
    }
}

/// Update schema with incongruent `maxLength` and `minLength`.
/// Replaces the schema with `false` schema if `maxLength`
/// is smaller than `minLength`
#[rule_processor_logger::log_processing]
fn update_max_min_length(
    schema: &mut Value,
    schema_primitive_types: &PrimitiveTypesBitMap,
) -> bool {
    if schema_primitive_types.contains(PrimitiveType::String) {
        match (
            schema.get("maxLength").map(Value::as_f64),
            schema.get("minLength").map(Value::as_f64),
        ) {
            (Some(max_), Some(min_)) if max_ < min_ => cleanup_incongruent_keywords(
                schema,
                *schema_primitive_types,
                PrimitiveType::String,
                &["maxLength", "minLength"],
            ),
            _ => false,
        }
    } else {
        false
    }
}

/// Update schema with incongruent `maxProperties` and `minProperties`.
/// Replaces the schema with `false` schema if `maxProperties`
/// is smaller than `minProperties`
#[rule_processor_logger::log_processing]
fn update_max_min_properties(
    schema: &mut Value,
    schema_primitive_types: &PrimitiveTypesBitMap,
) -> bool {
    if schema_primitive_types.contains(PrimitiveType::Object) {
        match (
            schema.get("maxProperties").map(Value::as_f64),
            schema.get("minProperties").map(Value::as_f64),
        ) {
            (Some(max_), Some(min_)) if max_ < min_ => cleanup_incongruent_keywords(
                schema,
                *schema_primitive_types,
                PrimitiveType::Object,
                &["maxProperties", "minProperties"],
            ),
            _ => false,
        }
    } else {
        false
    }
}

/// Update schema with incongruent `maximum` and `minimum`.
/// Replaces the schema with `false` schema if `maximum`
/// is smaller than `minimum`
#[rule_processor_logger::log_processing]
fn update_maximum_minimum(
    schema: &mut Value,
    schema_primitive_types: &PrimitiveTypesBitMap,
) -> bool {
    // Checking for PrimitiveType::Integer only as PrimitiveType::Number will include integer as well
    if schema_primitive_types.contains(PrimitiveType::Integer) {
        match (
            schema.get("maximum").map(Value::as_f64),
            schema.get("minimum").map(Value::as_f64),
        ) {
            (Some(max_), Some(min_)) if max_ < min_ => cleanup_incongruent_keywords(
                schema,
                *schema_primitive_types,
                PrimitiveType::Number,
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
#[rule_processor_logger::log_processing]
pub(crate) fn update_max_min_related_keywords(schema: &mut Value) -> bool {
    let mut updated_schema = false;
    // We're applying the keyword updates only if `type` keyword is present because
    // `[1, 2, 3]` would be valid against a schema like
    // `{"maximum": 1, "minimum": 2, "minItems": 1}`
    // NOTE: By ensuring the presence of type we're also ensuring that schema is a JSON object as well
    if schema.get("type").is_some() {
        let schema_primitive_types = PrimitiveTypesBitMap::from_schema(schema);

        for method in &[
            update_max_min_items,
            update_max_min_length,
            update_max_min_properties,
            update_exclusive_maximum_minimum,
            update_maximum_minimum,
        ] {
            updated_schema |= method(schema, &schema_primitive_types);
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
    use crate::helpers::types::PrimitiveTypesBitMap;

    use serde_json::{json, Value};
    use test_case::test_case;

    fn test(
        keyword_update_logic: fn(&mut Value, &PrimitiveTypesBitMap) -> bool,
        mut schema: Value,
    ) -> Value {
        crate::init_logger();
        let schema_primitive_types = PrimitiveTypesBitMap::from_schema(&schema);
        let _ = keyword_update_logic(&mut schema, &schema_primitive_types);
        schema
    }

    #[test_case(json!({"type": "integer", "exclusiveMaximum": 2, "exclusiveMinimum": 1}) => json!({"type": "integer", "exclusiveMaximum": 2, "exclusiveMinimum": 1}))]
    #[test_case(json!({"type": "integer", "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    #[test_case(json!({"type": "null", "exclusiveMaximum": 2, "exclusiveMinimum": 1}) => json!({"type": "null", "exclusiveMaximum": 2, "exclusiveMinimum": 1}))]
    #[test_case(json!({"type": "null", "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!({"type": "null", "exclusiveMaximum": 1, "exclusiveMinimum": 2}))]
    #[test_case(json!({"type": "number", "exclusiveMaximum": 2, "exclusiveMinimum": 1}) => json!({"type": "number", "exclusiveMaximum": 2, "exclusiveMinimum": 1}))]
    #[test_case(json!({"type": "number", "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    #[test_case(json!({"type": ["integer", "null"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!({"type": "null"}))]
    #[test_case(json!({"type": ["null", "number"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!({"type": "null"}))]
    #[test_case(json!({"type": ["integer", "null", "number"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!({"type": "null"}))]
    #[test_case(json!({"type": ["integer", "number"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    fn test_update_exclusive_maximum_minimum(value: Value) -> Value {
        test(update_exclusive_maximum_minimum, value)
    }

    #[test_case(json!({"type": "array", "maxItems": 2, "minItems": 1}) => json!({"type": "array", "maxItems": 2, "minItems": 1}))]
    #[test_case(json!({"type": "array", "maxItems": 1, "minItems": 2}) => json!(false))]
    #[test_case(json!({"type": "null", "maxItems": 2, "minItems": 1}) => json!({"type": "null", "maxItems": 2, "minItems": 1}))]
    #[test_case(json!({"type": "null", "maxItems": 1, "minItems": 2}) => json!({"type": "null", "maxItems": 1, "minItems": 2}))]
    #[test_case(json!({"type": ["array", "null"], "maxItems": 1, "minItems": 2}) => json!({"type": "null"}))]
    fn test_update_max_min_items(value: Value) -> Value {
        test(update_max_min_items, value)
    }

    #[test_case(json!({"type": "null", "maxLength": 2, "minLength": 1}) => json!({"type": "null", "maxLength": 2, "minLength": 1}))]
    #[test_case(json!({"type": "null", "maxLength": 1, "minLength": 2}) => json!({"type": "null", "maxLength": 1, "minLength": 2}))]
    #[test_case(json!({"type": "string", "maxLength": 2, "minLength": 1}) => json!({"type": "string", "maxLength": 2, "minLength": 1}))]
    #[test_case(json!({"type": "string", "maxLength": 1, "minLength": 2}) => json!(false))]
    #[test_case(json!({"type": ["null", "string"], "maxLength": 1, "minLength": 2}) => json!({"type": "null"}))]
    fn test_update_max_min_length(value: Value) -> Value {
        test(update_max_min_length, value)
    }

    #[test_case(json!({"type": "null", "maxProperties": 2, "minProperties": 1}) => json!({"type": "null", "maxProperties": 2, "minProperties": 1}))]
    #[test_case(json!({"type": "null", "maxProperties": 1, "minProperties": 2}) => json!({"type": "null", "maxProperties": 1, "minProperties": 2}))]
    #[test_case(json!({"type": "object", "maxProperties": 2, "minProperties": 1}) => json!({"type": "object", "maxProperties": 2, "minProperties": 1}))]
    #[test_case(json!({"type": "object", "maxProperties": 1, "minProperties": 2}) => json!(false))]
    #[test_case(json!({"type": ["null", "object"], "maxProperties": 1, "minProperties": 2}) => json!({"type": "null"}))]
    fn test_update_max_min_properties(value: Value) -> Value {
        test(update_max_min_properties, value)
    }

    #[test_case(json!({"type": "integer", "maximum": 2, "minimum": 1}) => json!({"type": "integer", "maximum": 2, "minimum": 1}))]
    #[test_case(json!({"type": "integer", "maximum": 1, "minimum": 2}) => json!(false))]
    #[test_case(json!({"type": "null", "maximum": 2, "minimum": 1}) => json!({"type": "null", "maximum": 2, "minimum": 1}))]
    #[test_case(json!({"type": "null", "maximum": 1, "minimum": 2}) => json!({"type": "null", "maximum": 1, "minimum": 2}))]
    #[test_case(json!({"type": "number", "maximum": 2, "minimum": 1}) => json!({"type": "number", "maximum": 2, "minimum": 1}))]
    #[test_case(json!({"type": "number", "maximum": 1, "minimum": 2}) => json!(false))]
    #[test_case(json!({"type": ["integer", "null"], "maximum": 1, "minimum": 2}) => json!({"type": "null"}))]
    #[test_case(json!({"type": ["null", "number"], "maximum": 1, "minimum": 2}) => json!({"type": "null"}))]
    #[test_case(json!({"type": ["integer", "null", "number"], "maximum": 1, "minimum": 2}) => json!({"type": "null"}))]
    #[test_case(json!({"type": ["integer", "number"], "maximum": 1, "minimum": 2}) => json!(false))]
    fn test_update_maximum_minimum(value: Value) -> Value {
        test(update_maximum_minimum, value)
    }

    // Ensure that impossible schemas are not modified if type is not defined
    #[test_case(json!(false))]
    #[test_case(json!(null))]
    #[test_case(json!(true))]
    #[test_case(json!({}))]
    #[test_case(json!({"exclusiveMaximum": 1, "exclusiveMinimum": 2}))]
    #[test_case(json!({"exclusiveMaximum": 2, "exclusiveMinimum": 1}))]
    #[test_case(json!({"maxItems": 1, "minItems": 2}))]
    #[test_case(json!({"maxItems": 2, "minItems": 1}))]
    #[test_case(json!({"maxLength": 1, "minLength": 2}))]
    #[test_case(json!({"maxLength": 2, "minLength": 1}))]
    #[test_case(json!({"maxProperties": 1, "minProperties": 2}))]
    #[test_case(json!({"maxProperties": 2, "minProperties": 1}))]
    #[test_case(json!({"maximum": 1, "minimum": 2}))]
    #[test_case(json!({"maximum": 2, "minimum": 1}))]
    // Ensure that incongruent keywords are not modified if not associated to available types
    #[test_case(json!({"type": "null", "exclusiveMaximum": 1, "exclusiveMinimum": 2}))]
    #[test_case(json!({"type": "null", "exclusiveMaximum": 2, "exclusiveMinimum": 1}))]
    #[test_case(json!({"type": "null", "maxItems": 1, "minItems": 2}))]
    #[test_case(json!({"type": "null", "maxItems": 2, "minItems": 1}))]
    #[test_case(json!({"type": "null", "maxLength": 1, "minLength": 2}))]
    #[test_case(json!({"type": "null", "maxLength": 2, "minLength": 1}))]
    #[test_case(json!({"type": "null", "maxProperties": 1, "minProperties": 2}))]
    #[test_case(json!({"type": "null", "maxProperties": 2, "minProperties": 1}))]
    #[test_case(json!({"type": "null", "maximum": 1, "minimum": 2}))]
    #[test_case(json!({"type": "null", "maximum": 2, "minimum": 1}))]
    fn test_update_max_min_related_keywords_does_not_perform_modifications_if_missing_or_incongruent_type(
        mut value: Value,
    ) {
        crate::init_logger();
        let initial_value = value.clone();
        assert!(!update_max_min_related_keywords(&mut value));
        assert_eq!(initial_value, value);
    }

    // Become a false schema as only the incongruent type is allowed
    #[test_case(json!({"type": "integer", "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    #[test_case(json!({"type": "number", "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    #[test_case(json!({"type": ["integer", "number"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!(false))]
    #[test_case(json!({"type": "array", "maxItems": 1, "minItems": 2}) => json!(false))]
    #[test_case(json!({"type": "string", "maxLength": 1, "minLength": 2}) => json!(false))]
    #[test_case(json!({"type": "object", "maxProperties": 1, "minProperties": 2}) => json!(false))]
    #[test_case(json!({"type": "integer", "maximum": 1, "minimum": 2}) => json!(false))]
    #[test_case(json!({"type": "number", "maximum": 1, "minimum": 2}) => json!(false))]
    #[test_case(json!({"type": ["integer", "number"], "maximum": 1, "minimum": 2}) => json!(false))]
    // // The incongruent primitive type is removed)
    #[test_case(json!({"type": ["integer", "null"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!({"type": "null"}))]
    #[test_case(json!({"type": ["null", "number"], "exclusiveMaximum": 1, "exclusiveMinimum": 2}) => json!({"type": "null"}))]
    #[test_case(json!({"type": ["array", "null"], "maxItems": 1, "minItems": 2}) => json!({"type": "null"}))]
    #[test_case(json!({"type": ["null", "string"], "maxLength": 1, "minLength": 2}) => json!({"type": "null"}))]
    #[test_case(json!({"type": ["null", "object"], "maxProperties": 1, "minProperties": 2}) => json!({"type": "null"}))]
    #[test_case(json!({"type": ["integer", "null"], "maximum": 1, "minimum": 2}) => json!({"type": "null"}))]
    #[test_case(json!({"type": ["null", "number"], "maximum": 1, "minimum": 2}) => json!({"type": "null"}))]
    fn test_update_max_min_related_keywords_does_performs_modifications(mut value: Value) -> Value {
        crate::init_logger();
        assert!(update_max_min_related_keywords(&mut value));
        value
    }
}
