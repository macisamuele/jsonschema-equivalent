use crate::{
    helpers::{replace, types::PrimitiveTypesBitMap},
    primitive_type::PrimitiveType,
};
use serde_json::Value;

/// Simplify a schema containing `enum` keywords.
/// The simplifications include:
/// * Removal of enum values which are not compliant with the `schema` allowed types
/// * Enum of a single value are equivalent to `const` keyword (after removal stage)
/// * Enum with no possible variants (after removal stage) are requivalent to a `false` schema
#[jsonschema_equivalent_rule_processor_logger::log_processing(cfg(feature = "logging"))]
pub(crate) fn simple_enum_cleanup(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };

    let schema_primitive_types = PrimitiveTypesBitMap::from_schema_value(schema_object.get("type"));
    if schema_primitive_types.is_empty() {
        return false;
    };

    if let Some(Value::Array(enum_values)) = schema_object.get_mut("enum") {
        if enum_values.is_empty() {
            // This should not be a valid schema, so let's avoid touching it
            false
        } else {
            let enum_indexes_to_remove: Vec<usize> = enum_values
                .iter()
                .enumerate()
                .filter_map(|(index, enum_value)| {
                    let enum_value_primitive_type = PrimitiveType::from_serde_value(enum_value);
                    if schema_primitive_types.contains(enum_value_primitive_type)
                        || (
                            // This additional case is needed because `PrimitiveType::from_serde_value` does not report `PrimitiveType::Integer`. Check the method doc for more info
                            enum_value_primitive_type == PrimitiveType::Number
                                && schema_primitive_types.contains(PrimitiveType::Integer)
                        )
                    {
                        None
                    } else {
                        Some(index)
                    }
                })
                .collect();

            if enum_indexes_to_remove.is_empty() {
                false
            } else if enum_indexes_to_remove.len() == enum_values.len() {
                replace::with_false_schema(schema)
            } else {
                for index_to_remove in enum_indexes_to_remove.iter().rev() {
                    let _ = enum_values.remove(*index_to_remove);
                }
                true
            }
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::simple_enum_cleanup;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({}) => json!({}))]
    #[test_case(json!({"enum": []}) => json!({"enum": []}))]
    #[test_case(json!({"enum": [1], "type": "string"}) => json!(false))]
    #[test_case(json!({"enum": ["0", "1", 2], "type": "string"}) => json!({"enum": ["0", "1"], "type": "string"}))]
    #[test_case(json!({"enum": [3, 4, 5], "type": "string"}) => json!(false))]
    fn test_remove_extraneous_keys_keyword_type_does_remove_keys(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = simple_enum_cleanup(&mut schema);
        schema
    }
}
