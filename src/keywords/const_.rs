use crate::{
    helpers::{replace, types::get_primitive_types},
    primitive_type::PrimitiveType,
};
use serde_json::Value;
use std::collections::BTreeSet;

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
        let schema_primitive_types = if let Some(value) = schema_object.get("type") {
            get_primitive_types(Some(value))
        } else {
            // As we don't have `type` keyword defined we cannot really do simplifications
            return false;
        };

        let const_primitive_type = PrimitiveType::from_serde_value(const_value);
        if schema_primitive_types.contains(&const_primitive_type) {
            let mut final_primitive_types = BTreeSet::new();
            let _ = final_primitive_types.insert(const_primitive_type);
            replace::type_with(schema_object, &final_primitive_types)
        } else if const_primitive_type == PrimitiveType::Number
            && schema_primitive_types.contains(&PrimitiveType::Integer)
        {
            // This additional case is needed because `PrimitiveType::from_serde_value` does not report `PrimitiveType::Integer`. Check the method doc for more info
            let mut final_primitive_types = BTreeSet::new();
            let _ = final_primitive_types.insert(PrimitiveType::Integer);
            replace::type_with(schema_object, &final_primitive_types)
        } else {
            replace::with_false_schema(schema);
            true
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
    #[test_case(json!({"const": []}) => json!({"const": []}))]
    #[test_case(json!({"const": true, "type": "boolean"}) => json!({"const": true, "type": "boolean"}))]
    #[test_case(json!({"const": "string", "type": "boolean"}) => json!(false))]
    #[test_case(json!({"const": "some-text", "type": ["boolean", "string"]}) => json!({"const": "some-text", "type": "string"}))]
    #[test_case(json!({"const": 1, "type": "integer"}) => json!({"const": 1, "type": "integer"}))]
    #[test_case(json!({"const": 1, "type": "number"}) => json!({"const": 1, "type": "number"}))]
    #[test_case(json!({"const": 1, "type": ["array", "integer"]}) => json!({"const": 1, "type": "integer"}))]
    #[test_case(json!({"const": 1, "type": ["array", "number"]}) => json!({"const": 1, "type": "number"}))]
    fn test_remove_extraneous_keys_keyword_type_does_remove_keys(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = simple_const_cleanup(&mut schema);
        schema
    }
}
