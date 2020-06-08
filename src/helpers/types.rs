use crate::primitive_type::PrimitiveType;
use serde_json::Value;
use std::collections::BTreeSet;
use std::convert::TryFrom;

/// Extract a set of primitive types contained by the input `type` keyword. (`maybe_type` should be the result of `schema.get("type")`)
///
/// NOTE: A `BTreeSet` is returned in order to preserve order-predictability while testing
pub(crate) fn get_primitive_types(maybe_type: Option<&Value>) -> BTreeSet<PrimitiveType> {
    if let Some(type_) = maybe_type {
        let mut set = BTreeSet::default();
        match type_ {
            Value::String(type_s) => {
                if let Ok(pt) = PrimitiveType::try_from(type_s.as_str()) {
                    let _ = set.insert(pt);
                    if pt == PrimitiveType::Number {
                        // "integer" is a subtype of "number"
                        // so if number is present then also integer is an included type
                        // NOTE: `to_json_schema_primitive_types` takes care of removing
                        // the redundancy if present
                        let _ = set.insert(PrimitiveType::Integer);
                    }
                }
            }
            Value::Array(types_) => {
                for type_s in types_ {
                    if let Ok(pt) = PrimitiveType::try_from(type_s) {
                        let _ = set.insert(pt);
                    }
                }
            }
            _ => {}
        }
        set
    } else {
        let mut set = BTreeSet::new();
        let _ = set.insert(PrimitiveType::Array);
        let _ = set.insert(PrimitiveType::Boolean);
        let _ = set.insert(PrimitiveType::Integer);
        let _ = set.insert(PrimitiveType::Null);
        let _ = set.insert(PrimitiveType::Number);
        let _ = set.insert(PrimitiveType::Object);
        let _ = set.insert(PrimitiveType::String);
        set
    }
}

/// Provide the most-efficient JSON representation of the input `primitive_types`.
///
/// The process includes removing `integer` if `number` is in or not having a representation if all the possible types are included
pub(crate) fn to_json_schema_primitive_types(
    primitive_types: &BTreeSet<PrimitiveType>,
) -> Option<Value> {
    match primitive_types.len() {
        0 => None,
        1 => Some(Value::String(
            primitive_types
                .iter()
                .next()
                .expect("Not empty iterator, so there is at least one element")
                .to_string(),
        )),
        6 if !primitive_types.contains(&PrimitiveType::Integer) => None,
        7 => None,
        _ => {
            let contains_number = primitive_types.contains(&PrimitiveType::Number);
            let json_primitive_types: Vec<_> = primitive_types
                .iter()
                .filter_map(|primitive_type| {
                    if contains_number && primitive_type == &PrimitiveType::Integer {
                        None
                    } else {
                        Some(Value::String(primitive_type.to_string()))
                    }
                })
                .collect();
            if json_primitive_types.len() == 1 {
                Some(json_primitive_types[0].clone())
            } else {
                Some(Value::Array(json_primitive_types))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{get_primitive_types, to_json_schema_primitive_types};
    use crate::primitive_type::PrimitiveType;
    use serde_json::{json, Value};
    use std::collections::BTreeSet;
    use test_case::test_case;

    #[test_case(&json!({}) => btree_set!(PrimitiveType::Array, PrimitiveType::Boolean, PrimitiveType::Integer, PrimitiveType::Null, PrimitiveType::Number, PrimitiveType::Object, PrimitiveType::String))]
    #[test_case(&json!({"type": "array"}) => btree_set!(PrimitiveType::Array))]
    #[test_case(&json!({"type": ["boolean"]}) => btree_set!(PrimitiveType::Boolean))]
    #[test_case(&json!({"type": ["integer", "null"]}) => btree_set!(PrimitiveType::Integer, PrimitiveType::Null))]
    #[test_case(&json!({"type": ["a-wrong-type"]}) => btree_set!())]
    fn test_get_primitive_types(schema: &Value) -> BTreeSet<PrimitiveType> {
        get_primitive_types(schema.get("type"))
    }

    #[test_case(&btree_set!() => None)]
    #[test_case(&btree_set!(PrimitiveType::Array) => Some(json!("array")))]
    #[test_case(&btree_set!(PrimitiveType::Boolean, PrimitiveType::Null) => Some(json!(["boolean", "null"])))]
    #[test_case(&btree_set!(PrimitiveType::Integer, PrimitiveType::Number) => Some(json!("number")))]
    #[test_case(&btree_set!(PrimitiveType::Integer, PrimitiveType::Number, PrimitiveType::String) => Some(json!(["number", "string"])))]
    #[test_case(&btree_set!(PrimitiveType::Array, PrimitiveType::Boolean, PrimitiveType::Null, PrimitiveType::Number, PrimitiveType::Object, PrimitiveType::String) => None)]
    #[test_case(&btree_set!(PrimitiveType::Array, PrimitiveType::Boolean, PrimitiveType::Integer, PrimitiveType::Null, PrimitiveType::Number, PrimitiveType::Object, PrimitiveType::String) => None)]
    fn test_to_json_schema_primitive_types(
        primitive_types: &BTreeSet<PrimitiveType>,
    ) -> Option<Value> {
        to_json_schema_primitive_types(primitive_types)
    }
}
