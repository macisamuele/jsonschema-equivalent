use crate::helpers::types::PrimitiveTypesBitMap;

use serde_json::{map::Entry, Map, Value};

use std::mem::replace;

/// Replace the `schema` with `false`.
/// The method returns true if a schema modification occurred. // FIXME
///
/// The objective of the method is to limit as much as possible the exposure
/// to more memory involved memory-related APIs.
/// Using `std::mem::replace` ensures that the value stored in `schema` is dropped
/// once leaving the scope of the method
#[inline]
pub(crate) fn with_false_schema(schema: &mut Value) -> bool {
    if schema == &Value::Bool(false) {
        false
    } else {
        let _ = replace(schema, Value::Bool(false));
        true
    }
}

/// Replace the `schema` with `true`.
/// The method returns true if a schema modification occurred. // FIXME
///
/// The objective of the method is to limit as much as possible the exposure
/// to more memory involved memory-related APIs.
/// Using `std::mem::replace` ensures that the value stored in `schema` is dropped
/// once leaving the scope of the method
#[inline]
pub(crate) fn with_true_schema(schema: &mut Value) -> bool {
    if schema == &Value::Bool(true) {
        false
    } else {
        let _ = replace(schema, Value::Bool(true));
        true
    }
}

/// Replace/Define the `type` keyword into the `schema`
/// The method returns true if a schema modification occurred.
///
/// The method is intended to simplify `type` keyword editing (the most commonly
/// performed) operation without duplicating a lot of logic
#[inline]
pub(crate) fn type_with(
    schema_object: &mut Map<String, Value>,
    primitive_types: PrimitiveTypesBitMap,
) -> bool {
    match schema_object.entry("type") {
        Entry::Vacant(entry) => {
            if let Some(json_primitive_types) = primitive_types.to_schema_value() {
                let _ = entry.insert(json_primitive_types);
                true
            } else {
                false
            }
        }
        Entry::Occupied(mut entry) => {
            if let Some(json_primitive_types) = primitive_types.to_schema_value() {
                let previous_value = entry.insert(json_primitive_types.clone());
                previous_value != json_primitive_types
            } else {
                let _ = entry.remove();
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{type_with, with_false_schema, with_true_schema};
    use crate::helpers::types::PrimitiveTypesBitMap;
    use crate::primitive_type::PrimitiveType;
    use serde_json::{json, Value};

    use test_case::test_case;

    macro_rules! bit_map {
        ($($pt: expr),* $(,)*) => {{
            let mut primitive_type_bit_map: PrimitiveTypesBitMap = PrimitiveTypesBitMap::default();
            $(
                primitive_type_bit_map |= $pt;
            )*
            primitive_type_bit_map
        }};
    }

    #[test_case(json!({}))]
    #[test_case(json!(null))]
    #[test_case(json!(false))]
    #[test_case(json!(true))]
    fn test_with_false_schema(mut schema: Value) {
        let was_a_false_schema = schema == Value::Bool(false);
        assert_eq!(with_false_schema(&mut schema), !was_a_false_schema);
        assert_eq!(schema, Value::Bool(false));
    }

    #[test_case(json!({}))]
    #[test_case(json!(null))]
    #[test_case(json!(false))]
    #[test_case(json!(true))]
    fn test_with_true_schema(mut schema: Value) {
        let was_simplest_true_schema = schema == Value::Bool(true);
        assert_eq!(with_true_schema(&mut schema), !was_simplest_true_schema);
        assert_eq!(schema, Value::Bool(true));
    }

    #[test_case(json!({}), bit_map!(PrimitiveType::Boolean), true => json!({"type": "boolean"}))]
    #[test_case(json!({"type": "null"}), bit_map!(PrimitiveType::Null), false => json!({"type": "null"}))]
    #[test_case(json!({"type": "object"}), bit_map!(PrimitiveType::Null, PrimitiveType::Object), true => json!({"type": ["null", "object"]}))]
    #[test_case(json!({"type": ["string", "object"]}), bit_map!(PrimitiveType::Object, PrimitiveType::String), true => json!({"type": ["object", "string"]}))]
    #[test_case(json!({"type": "number"}), bit_map!(PrimitiveType::Integer), true => json!({"type": "integer"}))]
    // All primitive types case
    #[test_case(json!({}), bit_map!(PrimitiveType::Array, PrimitiveType::Boolean, PrimitiveType::Integer, PrimitiveType::Null, PrimitiveType::Number, PrimitiveType::Object, PrimitiveType::String), false => json!({}))]
    #[test_case(json!({}), bit_map!(PrimitiveType::Array, PrimitiveType::Boolean, PrimitiveType::Null, PrimitiveType::Number, PrimitiveType::Object, PrimitiveType::String), false => json!({}))]
    #[test_case(json!({"type": "string"}), bit_map!(PrimitiveType::Array, PrimitiveType::Boolean, PrimitiveType::Null, PrimitiveType::Number, PrimitiveType::Object, PrimitiveType::String), true => json!({}))]
    fn test_type_with(
        mut schema: Value,
        primitive_types: PrimitiveTypesBitMap,
        is_modified: bool,
    ) -> Value {
        assert_eq!(
            type_with(
                &mut schema.as_object_mut().expect("It should be there"),
                primitive_types
            ),
            is_modified
        );
        schema
    }
}
