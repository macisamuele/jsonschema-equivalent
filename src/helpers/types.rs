use crate::primitive_type::PrimitiveType;
use serde_json::Value;
use std::collections::BTreeSet;
use std::convert::TryFrom;
use std::fmt;
use std::ops::{BitAnd, BitOrAssign};

/// Bitmap representing primitive types. Conceptually this is equivalent to a `BTreeSet<PrimitiveType>` or `HashSet<PrimitiveType>`
/// but allows for a better memory and temporal characteristics.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct PrimitiveTypesBitMap(u8);

impl fmt::Debug for PrimitiveTypesBitMap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "PrimitiveTypesBitMap(")?;
        formatter
            .debug_set()
            .entries(PrimitiveType::from_bit_representation(self.0))
            .finish()?;
        write!(formatter, ")")
    }
}

impl BitAnd<PrimitiveType> for PrimitiveTypesBitMap {
    type Output = Self;

    fn bitand(self, value: PrimitiveType) -> Self::Output {
        Self(self.0 & value.to_bit_representation())
    }
}

impl BitAnd<PrimitiveTypesBitMap> for PrimitiveTypesBitMap {
    type Output = Self;

    fn bitand(self, value: PrimitiveTypesBitMap) -> Self::Output {
        Self(self.0 & value.0)
    }
}

impl BitOrAssign<PrimitiveType> for PrimitiveTypesBitMap {
    fn bitor_assign(&mut self, value: PrimitiveType) {
        self.0 |= value.to_bit_representation();
    }
}

lazy_static::lazy_static! {
    static ref PRIMITIVE_TYPES_BIT_MAP_ALL_TYPES: u8 =
        PrimitiveType::Array.to_bit_representation() |
        PrimitiveType::Boolean.to_bit_representation() |
        PrimitiveType::Integer.to_bit_representation() |
        PrimitiveType::Null.to_bit_representation() |
        PrimitiveType::Number.to_bit_representation() |
        PrimitiveType::Object.to_bit_representation() |
        PrimitiveType::String.to_bit_representation()
    ;
}

impl From<PrimitiveType> for PrimitiveTypesBitMap {
    fn from(primitive_type: PrimitiveType) -> Self {
        Self(primitive_type.to_bit_representation())
    }
}

impl<I: IntoIterator<Item = &'static PrimitiveType>> From<I> for PrimitiveTypesBitMap {
    fn from(primitive_types: I) -> Self {
        let mut result = Self(0);
        for primitive_type in primitive_types {
            result |= *primitive_type;
        }
        result
    }
}

impl PrimitiveTypesBitMap {
    pub(crate) fn from_schema_value(schema_value: Option<&Value>) -> Self {
        match schema_value {
            Some(Value::String(value)) => {
                if let Ok(primitive_type) = PrimitiveType::try_from(value.as_str()) {
                    Self::from(primitive_type)
                } else {
                    // This should not be possible on a valid schema
                    Self(0)
                }
            }
            Some(Value::Array(types)) => {
                let mut result = Self(0);
                for type_ in types {
                    if let Value::String(value) = type_ {
                        if let Ok(primitive_type) = PrimitiveType::try_from(value.as_str()) {
                            result |= primitive_type;
                        }
                    }
                }
                result
            }
            None => Self(*PRIMITIVE_TYPES_BIT_MAP_ALL_TYPES),
            _ => Self(0), // This is not possible, except if the schema is not valid.
        }
    }

    pub(crate) fn from_schema(schema: &Value) -> Self {
        match schema {
            Value::Bool(true) => Self(*PRIMITIVE_TYPES_BIT_MAP_ALL_TYPES),
            Value::Object(schema_object) => Self::from_schema_value(schema_object.get("type")),
            _ => Self(0), // This is not possible, except if the schema is not valid.
        }
    }

    pub(crate) fn contains(self, primitive_type: PrimitiveType) -> bool {
        (self & primitive_type).0 != 0
            && (primitive_type != PrimitiveType::Number
                || (self.0
                    & PrimitiveType::Number.to_bit_representation()
                    & !PrimitiveType::Integer.to_bit_representation())
                    != 0)
    }

    pub(crate) fn remove(&mut self, primitive_type: PrimitiveType) {
        self.0 &= !(primitive_type.to_bit_representation());
    }

    pub(crate) fn remove_all(&mut self, primitive_types: Self) {
        self.0 &= !(primitive_types.0);
    }

    pub(crate) fn to_schema_value(mut self) -> Option<Value> {
        if self.0 == 0 || self.0 == *PRIMITIVE_TYPES_BIT_MAP_ALL_TYPES {
            None
        } else {
            if self.contains(PrimitiveType::Integer) && self.contains(PrimitiveType::Number) {
                // Remove Integer as redundant considering that it is a subtype of Number
                self.remove(PrimitiveType::Integer);
            }
            let primitive_types_vec = PrimitiveType::from_bit_representation(self.0);
            if primitive_types_vec.len() == 1 {
                Some(Value::String(primitive_types_vec[0].to_string()))
            } else {
                Some(Value::Array(
                    primitive_types_vec
                        .iter()
                        .map(|primitive_type| Value::String(primitive_type.to_string()))
                        .collect(),
                ))
            }
        }
    }

    pub(crate) fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(crate) fn is_complete(self) -> bool {
        self.0 == *PRIMITIVE_TYPES_BIT_MAP_ALL_TYPES
    }

    pub(crate) fn has_other_primitive_types_other_than(
        self,
        primitive_type: PrimitiveType,
    ) -> bool {
        (self.0 & !primitive_type.to_bit_representation()) != 0
    }
}

impl Default for PrimitiveTypesBitMap {
    fn default() -> Self {
        Self::from_schema(&Value::Bool(false))
    }
}

impl From<PrimitiveTypesBitMap> for BTreeSet<PrimitiveType> {
    fn from(value: PrimitiveTypesBitMap) -> Self {
        PrimitiveType::from_bit_representation(value.0)
            .iter()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::PrimitiveTypesBitMap;
    use crate::primitive_type::PrimitiveType;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(
        &json!(true) => PrimitiveTypesBitMap(
            PrimitiveType::Array.to_bit_representation() |
            PrimitiveType::Boolean.to_bit_representation() |
            PrimitiveType::Integer.to_bit_representation() |
            PrimitiveType::Null.to_bit_representation() |
            PrimitiveType::Number.to_bit_representation() |
            PrimitiveType::Object.to_bit_representation() |
            PrimitiveType::String.to_bit_representation()
        )
    )]
    #[test_case(
        &json!({}) => PrimitiveTypesBitMap(
            PrimitiveType::Array.to_bit_representation() |
            PrimitiveType::Boolean.to_bit_representation() |
            PrimitiveType::Integer.to_bit_representation() |
            PrimitiveType::Null.to_bit_representation() |
            PrimitiveType::Number.to_bit_representation() |
            PrimitiveType::Object.to_bit_representation() |
            PrimitiveType::String.to_bit_representation()
        )
    )]
    #[test_case(&json!(false) => PrimitiveTypesBitMap(0))]
    #[test_case(&json!({"type": "array"}) => PrimitiveTypesBitMap(PrimitiveType::Array.to_bit_representation()))]
    #[test_case(&json!({"type": ["array", "boolean"]}) => PrimitiveTypesBitMap(PrimitiveType::Array.to_bit_representation() | PrimitiveType::Boolean.to_bit_representation()))]
    #[test_case(&json!({"type": "integer"}) => PrimitiveTypesBitMap(PrimitiveType::Integer.to_bit_representation()))]
    #[test_case(&json!({"type": "number"}) => PrimitiveTypesBitMap(PrimitiveType::Integer.to_bit_representation() | PrimitiveType::Number.to_bit_representation()))]
    fn test_primitive_types_bit_map_from_schema(schema: &Value) -> PrimitiveTypesBitMap {
        PrimitiveTypesBitMap::from_schema(schema)
    }

    #[test_case(&json!({"type": ["null"]}) => Some(json!("null")))]
    #[test_case(&json!({"type": ["array", "boolean"]}) => Some(json!(["array", "boolean"])))]
    #[test_case(&json!({"type": "integer"}) => Some(json!("integer")))]
    #[test_case(&json!({"type": "number"}) => Some(json!("number")))]
    fn test_primitive_types_bit_map_to_schema_value(schema: &Value) -> Option<Value> {
        PrimitiveTypesBitMap::from_schema(schema).to_schema_value()
    }

    #[test_case(&json!(true) => false)]
    #[test_case(&json!(false) => true)]
    #[test_case(&json!({}) => false)]
    #[test_case(&json!({"type": []}) => true)]
    #[test_case(&json!({"type": ["null"]}) => false)]
    #[test_case(&json!({"type": ["array", "boolean"]}) => false)]
    #[test_case(&json!({"type": "integer"}) => false)]
    #[test_case(&json!({"type": "number"}) => false)]
    fn test_primitive_types_bit_map_is_empty(schema: &Value) -> bool {
        PrimitiveTypesBitMap::from_schema(schema).is_empty()
    }

    #[test_case(&json!({"type": ["null"]}), PrimitiveType::Null => false)]
    #[test_case(&json!({"type": ["null"]}), PrimitiveType::String => true)]
    #[test_case(&json!({"type": ["array", "boolean"]}), PrimitiveType::Array => true)]
    #[test_case(&json!({"type": ["array", "boolean"]}), PrimitiveType::Integer => true)]
    #[test_case(&json!({"type": "integer"}), PrimitiveType::Integer => false)]
    #[test_case(&json!({"type": "number"}), PrimitiveType::Integer => true)]
    #[test_case(&json!({"type": "number"}), PrimitiveType::Number => false)]
    fn test_primitive_types_bit_map_has_other_primitive_types_other_than(
        schema: &Value,
        primitive_type: PrimitiveType,
    ) -> bool {
        PrimitiveTypesBitMap::from_schema(schema)
            .has_other_primitive_types_other_than(primitive_type)
    }
}
