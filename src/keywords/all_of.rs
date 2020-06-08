use crate::{
    helpers::{types::get_primitive_types, is, replace},
    primitive_type::PrimitiveType,
};
use serde_json::Value;
use std::collections::BTreeSet;

/// Simplify `allOf` keyword by removing it if the union of the listed schemas are equivalent to a `true` schema
/// or replacing the whole schema with a `false` schema if the union of the listed schemas are equivalent to a
/// `false` schema.
#[rule_processor_logger::log_processing]
pub(crate) fn simplify_all_of(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };
    let schema_primitive_types = get_primitive_types(schema_object.get("type"));
    if let Some(Value::Array(items)) = schema_object.get_mut("allOf") {
        let mut updated_schema = false;

        let indexes_to_remove: Vec<_> = items
            .iter()
            .enumerate()
            .filter_map(|(index, subschema)| {
                if is::true_schema(subschema) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();

        for index_to_remove in indexes_to_remove.iter().rev() {
            let _ = items.remove(*index_to_remove);
        }

        if items.is_empty() {
            if !indexes_to_remove.is_empty() {
                // `allOf` was initially not empty, but we removed some schemas
                let _ = schema_object.remove("allOf");
                return true;
            }
        } else if items.iter().any(is::false_schema) {
            // if there is a `false` schema in `allOf` than is impossible to have any value that would be valid
            // so the overall schema is a `false` schema
            return replace::with_false_schema(schema);
        } else {
            let all_of_primitive_types: Vec<_> = items
                .iter()
                .map(|all_of_schema| get_primitive_types(all_of_schema.get("type")))
                .collect();

            let mut defined_types_in_all_of = false;
            let mut common_all_of_primitive_types = schema_primitive_types;

            for primitive_types in &all_of_primitive_types {
                // TODO: Look if there is a better way to compute the intersection (even linear-search might provide ok-ish performances as the size is 7 at max)
                let tmp: BTreeSet<PrimitiveType> = common_all_of_primitive_types
                    .intersection(primitive_types)
                    .cloned()
                    .collect();
                common_all_of_primitive_types = tmp;

                if common_all_of_primitive_types.is_empty() {
                    return replace::with_false_schema(schema);
                }

                if !primitive_types.is_empty() {
                    defined_types_in_all_of = true;
                }
            }

            if !defined_types_in_all_of {
                // Do nothing as no types were defined
            } else if common_all_of_primitive_types.is_empty() {
                // We have types defined, but no types are in common
                // So no value can ever be considered valid, hence a `false` schema as result
                return replace::with_false_schema(schema);
            } else {
                // Update all the types in the `allOf` schemas to contain only the common items
                // This is need to allow next stages (ie. `type` optimisation to remove not needed keyworkds)
                for (all_of_item, all_of_primitive_types) in items.iter_mut().zip(all_of_primitive_types) {
                    if all_of_primitive_types != common_all_of_primitive_types {
                        if let Value::Object(all_of_item_schema) = all_of_item {
                            updated_schema |= replace::type_with(all_of_item_schema, &common_all_of_primitive_types)
                        }
                    }
                }
            }
        }
        updated_schema
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::simplify_all_of;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({"allOf": [{"type": "string"}]}) => json!({"allOf": [{"type": "string"}]}))]
    #[test_case(json!({"allOf": []}) => json!({"allOf": []}))]
    #[test_case(json!({"type": "object", "allOf": [{}]}) => json!({"type": "object"}))]
    #[test_case(json!({"allOf": [false]}) => json!(false))]
    #[test_case(json!({"allOf": [{"type": ["integer", "string"]}]}) => json!({"allOf": [{"type": ["integer", "string"]}]}))]
    #[test_case(json!({"allOf": [{"type": "integer"}, {"type": "number"}]}) => json!({"allOf": [{"type": "integer"}, {"type": "integer"}]}))]
    #[test_case(json!({"allOf": [{"type": ["integer", "string"]}, {"type": "number"}]}) => json!({"allOf": [{"type": "integer"}, {"type": "integer"}]}))]
    #[test_case(json!({"allOf": [{"type": "string"}, {"type": "number"}]}) => json!(false))]
    #[test_case(json!({"allOf":[{"type":"integer"}], "type": "boolean"}) => json!(false))]
    fn test_simplify_all_of(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = simplify_all_of(&mut schema);
        schema
    }
}
