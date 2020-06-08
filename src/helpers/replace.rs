use crate::helpers::types::{get_primitive_types, to_json_schema_primitive_types};
use crate::primitive_type::PrimitiveType;
use serde_json::{map::Entry, Map, Value};
use std::collections::BTreeSet;
use std::mem::replace;

/// Replace the `schema` with `false`.
/// The method returns true if a schema modification occurred. // FIXME
///
/// The objective of the method is to limit as much as possible the exposure
/// to more memory involved memory-related APIs.
/// Using `std::mem::replace` ensures that the value stored in `schema` is dropped
/// once leaving the scope of the method
#[inline]
pub(crate) fn with_false_schema(schema: &mut Value) {
    let _ = replace(schema, Value::Bool(false));
}

/// Replace the `schema` with `true`.
/// The method returns true if a schema modification occurred. // FIXME
///
/// The objective of the method is to limit as much as possible the exposure
/// to more memory involved memory-related APIs.
/// Using `std::mem::replace` ensures that the value stored in `schema` is dropped
/// once leaving the scope of the method
#[allow(dead_code)]
#[inline]
pub(crate) fn with_true_schema(schema: &mut Value) {
    let _ = replace(schema, Value::Bool(true));
}

/// Replace/Define the `type` keyword into the `schema`
/// The method returns true if a schema modification occurred.
///
/// The method is intended to simplify `type` keyword editing (the most commonly
/// performed) operation without duplicating a lot of logic
#[inline]
pub(crate) fn type_with(
    schema_object: &mut Map<String, Value>,
    primitive_types: &BTreeSet<PrimitiveType>,
) -> bool {
    match schema_object.entry("type") {
        Entry::Vacant(entry) => {
            if let Some(json_primitive_types) = to_json_schema_primitive_types(primitive_types) {
                let _ = entry.insert(json_primitive_types);
                true
            } else {
                false
            }
        }
        Entry::Occupied(mut entry) => {
            if let Some(json_primitive_types) = to_json_schema_primitive_types(primitive_types) {
                let previous_value = entry.insert(json_primitive_types);
                primitive_types != &get_primitive_types(Some(&previous_value))
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
    use crate::primitive_type::PrimitiveType;
    use serde_json::{json, Value};
    use std::collections::BTreeSet;
    use test_case::test_case;

    #[test_case(json!({}))]
    #[test_case(json!(null))]
    #[test_case(json!(true))]
    fn test_with_false_schema(mut schema: Value) {
        with_false_schema(&mut schema);
        assert_eq!(schema, Value::Bool(false));
    }

    #[test_case(json!({}))]
    #[test_case(json!(null))]
    #[test_case(json!(true))]
    fn test_with_true_schema(mut schema: Value) {
        with_true_schema(&mut schema);
        assert_eq!(schema, Value::Bool(true));
    }

    #[test_case(json!({}), &btree_set!(PrimitiveType::Boolean), true => json!({"type": "boolean"}))]
    #[test_case(json!({"type": "null"}), &btree_set!(PrimitiveType::Null), false => json!({"type": "null"}))]
    #[test_case(json!({"type": "object"}), &btree_set!(PrimitiveType::Null, PrimitiveType::Object), true => json!({"type": ["null", "object"]}))]
    #[test_case(json!({"type": ["string", "object"]}), &btree_set!(PrimitiveType::Object, PrimitiveType::String), false => json!({"type": ["object", "string"]}))]
    #[test_case(json!({"type": "number"}), &btree_set!(PrimitiveType::Integer), true => json!({"type": "integer"}))]
    // All primitive types case
    #[test_case(json!({}), &btree_set!(PrimitiveType::Array, PrimitiveType::Boolean, PrimitiveType::Integer, PrimitiveType::Null, PrimitiveType::Number, PrimitiveType::Object, PrimitiveType::String), false => json!({}))]
    #[test_case(json!({}), &btree_set!(PrimitiveType::Array, PrimitiveType::Boolean, PrimitiveType::Null, PrimitiveType::Number, PrimitiveType::Object, PrimitiveType::String), false => json!({}))]
    #[test_case(json!({"type": "string"}), &btree_set!(PrimitiveType::Array, PrimitiveType::Boolean, PrimitiveType::Null, PrimitiveType::Number, PrimitiveType::Object, PrimitiveType::String), true => json!({}))]
    fn test_type_with(
        mut schema: Value,
        primitive_types: &BTreeSet<PrimitiveType>,
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
