use crate::helpers::{types::get_primitive_types, is, replace};
use serde_json::Value;

/// Simplify `anyOf` keyword by removing it if the union of the listed schemas are equivalent to a `true` schema
/// or replacing the whole schema with a `false` schema if the union of the listed schemas are equivalent to a
/// `false` schema.
#[rule_processor_logger::log_processing]
pub(crate) fn simplify_any_of(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };

    let schema_primitive_types = get_primitive_types(schema_object.get("type"));
    if let Some(Value::Array(items)) = schema_object.get_mut("anyOf") {
        let indexes_to_remove: Vec<_> = items
            .iter()
            .enumerate()
            .filter_map(|(index, subschema)| {
                if is::false_schema(subschema) {
                    Some(index)
                } else if schema_primitive_types.intersection(&get_primitive_types(subschema.get("type"))).next().is_some() {
                    None
                } else {
                    // index has to be removed as the any_of item has incompatible type with schema, so it will never be valid
                    Some(index)
                }
            })
            .collect();

        for index_to_remove in indexes_to_remove.iter().rev() {
            let _ = items.remove(*index_to_remove);
        }

        if items.is_empty() {
            if !indexes_to_remove.is_empty() {
                // `anyOf` was initially composed only by false schemas, so it's is a false schema
                return replace::with_false_schema(schema);
            }
        } else if items.iter().any(is::true_schema) {
            // if there is a `true` schema in `anyOf` than `anyOf` is not adding schema restrictions
            // so the overall schema is equivalent to the schema without `anyOf`
            let _ = schema_object.remove("anyOf");
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::simplify_any_of;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({"anyOf": [{"type": "string"}]}) => json!({"anyOf": [{"type": "string"}]}))]
    #[test_case(json!({"anyOf": []}) => json!({"anyOf": []}))]
    #[test_case(json!({"type": "object", "anyOf": [{}]}) => json!({"type": "object"}))]
    #[test_case(json!({"anyOf": [true]}) => json!({}))]
    #[test_case(json!({"anyOf": [false]}) => json!(false))]
    #[test_case(json!({"anyOf": [{"type": ["integer", "string"]}]}) => json!({"anyOf": [{"type": ["integer", "string"]}]}))]
    #[test_case(json!({"anyOf": [{"type": "string"}, {"type": "number"}]}) => json!({"anyOf": [{"type": "string"}, {"type": "number"}]}))]
    #[test_case(json!({"anyOf": [{"type": "boolean"}, {"type": "number"}], "type": "number"}) => json!({"anyOf": [{"type": "number"}], "type": "number"}))]
    #[test_case(json!({"anyOf":[{"type":"integer"}], "type": "boolean"}) => json!(false))]
    fn test_simplify_any_of(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = simplify_any_of(&mut schema);
        schema
    }
}
