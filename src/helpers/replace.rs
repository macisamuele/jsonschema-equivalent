use serde_json::Value;
use std::mem::replace;

/// Replace the `schema` with `false`.
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

#[cfg(test)]
mod tests {
    use super::{with_false_schema, with_true_schema};
    use serde_json::{json, Value};
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
}
