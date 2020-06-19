use serde_json::Value;
use std::convert::TryFrom;

/// Enum representation of the 7 primitive types recognized by JSON Schema.
///
/// The usage of the enum allows to have a faster processing (less string comparisons)
/// as well as smaller memory footprint as the enum instance uses 2 bytes.
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum PrimitiveType {
    Array,
    Boolean,
    Integer,
    Null,
    Number,
    Object,
    String,
}
impl TryFrom<&str> for PrimitiveType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "array" => Ok(Self::Array),
            "boolean" => Ok(Self::Boolean),
            "integer" => Ok(Self::Integer),
            "null" => Ok(Self::Null),
            "number" => Ok(Self::Number),
            "object" => Ok(Self::Object),
            "string" => Ok(Self::String),
            _ => Err(format!(r#""{}" is not a recognized primitive type"#, value)),
        }
    }
}
impl TryFrom<&Value> for PrimitiveType {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        if let Some(value_str) = value.as_str() {
            Self::try_from(value_str)
        } else {
            Err(format!("Expected Value::String(...), found {:?}", value))
        }
    }
}
impl ToString for PrimitiveType {
    fn to_string(&self) -> String {
        match self {
            Self::Array => "array".to_string(),
            Self::Boolean => "boolean".to_string(),
            Self::Integer => "integer".to_string(),
            Self::Null => "null".to_string(),
            Self::Number => "number".to_string(),
            Self::Object => "object".to_string(),
            Self::String => "string".to_string(),
        }
    }
}
impl PrimitiveType {
    #[allow(dead_code)]
    pub(crate) fn from_serde_value(value: &Value) -> Self {
        match value {
            Value::Array(_) => Self::Array,
            Value::Bool(_) => Self::Boolean,
            Value::Null => Self::Null,
            // In order to make the tool less binded to Draft versions
            // we're not trying (at least yet) to detect the correct
            // numeric type. This because `1.0` is not a valid `integer`
            // for Draft4 but it is for Draft7
            Value::Number(_) => Self::Number,
            Value::Object(_) => Self::Object,
            Value::String(_) => Self::String,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PrimitiveType;
    use serde_json::{json, Value};
    use std::convert::TryFrom;
    use test_case::test_case;

    #[test_case("array" => Ok(PrimitiveType::Array))]
    #[test_case("boolean" => Ok(PrimitiveType::Boolean))]
    #[test_case("integer" => Ok(PrimitiveType::Integer))]
    #[test_case("null" => Ok(PrimitiveType::Null))]
    #[test_case("number" => Ok(PrimitiveType::Number))]
    #[test_case("object" => Ok(PrimitiveType::Object))]
    #[test_case("string" => Ok(PrimitiveType::String))]
    #[test_case("something" => Err(r#""something" is not a recognized primitive type"#.to_string()))]
    fn test_from_str_to_primitive_type(value: &str) -> Result<PrimitiveType, String> {
        PrimitiveType::try_from(value)
    }

    #[test_case(&json!([]) => PrimitiveType::Array)]
    #[test_case(&json!(true) => PrimitiveType::Boolean)]
    #[test_case(&json!(null) => PrimitiveType::Null)]
    #[test_case(&json!(1) => PrimitiveType::Number)]
    #[test_case(&json!({}) => PrimitiveType::Object)]
    #[test_case(&json!("") => PrimitiveType::String)]
    fn test_from_serde_value(value: &Value) -> PrimitiveType {
        PrimitiveType::from_serde_value(value)
    }
}
