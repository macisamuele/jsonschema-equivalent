use crate::{
    helpers::{replace, types::PrimitiveTypesBitMap},
    primitive_type::PrimitiveType,
};
use serde_json::Value;

/// Simplify a schema containing `const` keywords.
/// The simplifications include:
/// * removing types that are not in sync with the type of the `const` value
/// * if no types are left after previous removal, then the `schema` is a `false` schema
#[rule_processor_logger::log_processing]
pub(crate) fn simple_const_cleanup(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };

    if let Some(const_value) = schema_object.get("const") {
        let schema_primitive_types =
            PrimitiveTypesBitMap::from_schema_value(schema_object.get("type"));
        if schema_primitive_types.is_empty() {
            // As we don't have `type` keyword defined we cannot really do simplifications
            return false;
        }

        let const_primitive_type = dbg![PrimitiveType::from_serde_value(dbg![const_value])];
        if schema_primitive_types.contains(const_primitive_type) {
            replace::type_with(
                schema_object,
                PrimitiveTypesBitMap::from_primitive_type(const_primitive_type),
            )
        } else if const_primitive_type == PrimitiveType::Number
            && schema_primitive_types.contains(PrimitiveType::Integer)
        {
            // This additional case is needed because `PrimitiveType::from_serde_value` does not report `PrimitiveType::Integer`. Check the method doc for more info
            replace::type_with(
                schema_object,
                PrimitiveTypesBitMap::from_primitive_type(PrimitiveType::Integer),
            )
        } else {
            replace::with_false_schema(schema)
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::simple_const_cleanup;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({}) => json!({}))]
    #[test_case(json!({"const": []}) => json!({"const": [], "type": "array"}))]
    #[test_case(json!({"const": 1}) => json!({"const": 1, "type": "number"}))]
    #[test_case(json!({"const": true, "type": "boolean"}) => json!({"const": true, "type": "boolean"}))]
    #[test_case(json!({"const": "string", "type": "boolean"}) => json!(false))]
    #[test_case(json!({"const": "some-text", "type": ["boolean", "string"]}) => json!({"const": "some-text", "type": "string"}))]
    #[test_case(json!({"const": 1, "type": "integer"}) => json!({"const": 1, "type": "integer"}))]
    #[test_case(json!({"const": 1, "type": "number"}) => json!({"const": 1, "type": "number"}))]
    #[test_case(json!({"const": 1, "type": ["array", "integer"]}) => json!({"const": 1, "type": "integer"}))]
    #[test_case(json!({"const": 1, "type": ["array", "number"]}) => json!({"const": 1, "type": "number"}))]
    fn test_simple_const_cleanup(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = simple_const_cleanup(&mut schema);
        schema
    }
}
