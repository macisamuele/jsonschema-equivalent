use crate::helpers::intersect::{intersection_schema, IntersectStatus};
use crate::helpers::{is, replace, types::PrimitiveTypesBitMap};
use jsonschema_equivalent_rule_processor_logger::log_processing;
use serde_json::Value;

/// Simplify `allOf` keyword by removing it if the union of the listed schemas are equivalent to a `true` schema
/// or replacing the whole schema with a `false` schema if the union of the listed schemas are equivalent to a
/// `false` schema.
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn simplify_all_of(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };
    let schema_primitive_types = PrimitiveTypesBitMap::from_schema_value(schema_object.get("type"));
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
                .map(|all_of_schema| {
                    PrimitiveTypesBitMap::from_schema_value(all_of_schema.get("type"))
                })
                .collect();

            let mut defined_types_in_all_of = false;
            let mut common_all_of_primitive_types = schema_primitive_types;

            for primitive_types in &all_of_primitive_types {
                common_all_of_primitive_types = common_all_of_primitive_types & *primitive_types;

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
                for (all_of_item, all_of_primitive_types) in
                    items.iter_mut().zip(all_of_primitive_types)
                {
                    if all_of_primitive_types != common_all_of_primitive_types {
                        if let Value::Object(all_of_item_schema) = all_of_item {
                            updated_schema |= replace::type_with(
                                all_of_item_schema,
                                common_all_of_primitive_types,
                            )
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

/// Flatten all the possible keywords of the `allOf` schemas into the main schema.
///
/// The flattening process does remove the common keywords from the `allOf` schemas
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn flatten_all_of(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };

    // The clone is not nice but I found no way around the borrow checker that allows to
    // hold a mutable and immutable  reference to the same object.
    // Using `unsafe` might be an approach, but for now I'm focusing on the functionality
    // rather than performance
    let mut schema_clone = Value::Object(schema_object.clone());
    let mut schema_mut_ref = &mut schema_clone;

    if let Some(Value::Array(all_of_items)) = schema_object.get_mut("allOf") {
        let mut updated_schema = false;
        let mut all_of_indexes_to_remove = Vec::<usize>::new();

        for (index, all_of_item) in all_of_items.iter().enumerate() {
            // TODO: intersection_schema should provide info around modifications happened to schema
            match intersection_schema(schema_mut_ref, all_of_item) {
                IntersectStatus::Complete {
                    schema,
                    updated_schema: updated_schema_by_intersection,
                } => {
                    updated_schema |= updated_schema_by_intersection;
                    all_of_indexes_to_remove.push(index);
                    schema_mut_ref = schema;
                }
                IntersectStatus::Partial {
                    schema,
                    updated_schema: updated_schema_by_intersection,
                } => {
                    updated_schema |= updated_schema_by_intersection;
                    schema_mut_ref = schema;
                }
            };
        }

        let updated_schema = if all_of_indexes_to_remove.len() == all_of_items.len() {
            let _ = std::mem::replace(schema, schema_clone);
            if let Value::Object(schema_object) = schema {
                let _ = schema_object.remove("allOf");
            }
            return true;
        } else if all_of_indexes_to_remove.is_empty() {
            updated_schema
        } else {
            for index_to_remove in all_of_indexes_to_remove.iter().rev() {
                let _ = all_of_items.remove(*index_to_remove);
            }
            true
        };
        if updated_schema {
            let _ = std::mem::replace(schema, schema_clone);
        }
        updated_schema
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{flatten_all_of, simplify_all_of};
    use crate::keywords::update_schema;
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

    #[test_case(json!({"allOf": [{"type": "string"}]}) => json!({"type": "string"}))]
    #[test_case(json!({"allOf": [{"type": "string"}, {"minLength": 1}]}) => json!({"type": "string", "minLength": 1}))]
    // #[test_case(json!({"allOf": [{"type": "string"}, {"allOf": [{"type": "string"}]}]}) => json!({"type": "string", "minLength": 1}))]
    fn test_flatten_all_of(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = flatten_all_of(&mut schema);
        schema
    }

    #[test_case(json!({"type": "string", "minLength": 2, "allOf": [false]}) => json!(false))]
    // #[test_case(json!({"type": "string", "minLength": 2, "allOf": [{"maxLength": 1}]}) => json!(false))]
    // #[test_case(json!({"type": "string", "minLength": 2, "allOf": [{"minLength": 3}]}) => json!({"type": "string", "minLength": 3}))]
    // #[test_case(json!({"type": "string", "minLength": 2, "allOf": [{"maxLength": 3}]}) => json!({"type": "string", "minLength": 2, "maxLength": 3}))]
    fn test_update_schema(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = update_schema(&mut schema);
        schema
    }
}
