use crate::helpers::{is, replace, types::PrimitiveTypesBitMap};
use crate::primitive_type::PrimitiveType;
use jsonschema_equivalent_rule_processor_logger::log_processing;
use serde_json::Value;

/// `propertyNames` should contain a schema that will be used to validate the properties
/// of the JSON Object to validate. If a different JSON value is passed for validation then
/// it will be valid schema.
///
/// This assumptions are made explict as:
///  * if a JSON Object would not be a valid instance (maybe because `type` keyword allows only `strings`)
///     then removing `propertyNames` would not alter the filtering capability of the schema
///  * if a JSON Object would be a valid instance, then we know that the attributes MUST be strings so we
///     can simplify the `propertyNames` schema allowing only `type` string instances
#[log_processing(cfg(feature = "logging"))]
pub(crate) fn optimise_property_names(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };
    let mut schema_primitive_types =
        PrimitiveTypesBitMap::from_schema_value(schema_object.get("type"));
    let schema_min_properties = schema_object
        .get("minProperties")
        .map_or(0.0, |value| value.as_f64().unwrap_or(0.0));
    let property_names_schema = if let Some(value) = schema_object.get_mut("propertyNames") {
        value
    } else {
        return false;
    };

    if is::true_schema(property_names_schema) {
        let _ = schema_object.remove("propertyNames");
        return true;
    }

    let mut updated_schema = false;
    let property_names_types = PrimitiveTypesBitMap::from_schema(property_names_schema);
    if property_names_types.contains(PrimitiveType::String)
        && property_names_types.has_other_primitive_types_other_than(PrimitiveType::String)
    {
        // We know that `propertyNames` is an object as we have types in the bitmap.
        // A `false` schema does not have types in the bitmap
        if let Value::Object(property_names_schema_object) = property_names_schema {
            updated_schema |= replace::type_with(
                property_names_schema_object,
                PrimitiveTypesBitMap::from_primitive_type(PrimitiveType::String),
            );
        }
    }

    updated_schema |= if schema_primitive_types.contains(PrimitiveType::Object) {
        if !property_names_types.contains(PrimitiveType::String) {
            // No properties can be accepted, so we need to decide if we can still consider JSON Object as valid type or not
            // The determination relies on the requirement of having at least one property to consider the schema valid
            // If one property is required then we cannot accept `type=object`, otherwise we need to restrict the maximum
            // number of properties to 0 (so only empty objects can be passed in)

            if schema_min_properties > 0.0 {
                schema_primitive_types.remove(PrimitiveType::Object);
                if replace::type_with(schema_object, schema_primitive_types) {
                    if schema_object.get("type") == None {
                        // If the only supported type was object then the schema is just a `false` schema
                        let _ = replace::with_false_schema(schema);
                    }
                    // We were able to modify the schema on `replace::type_with`
                    true
                } else {
                    false
                }
            } else {
                let _ = schema_object.insert("maxProperties".to_string(), Value::from(0));
                let _ = schema_object.remove("propertyNames");
                true
            }
        } else if let Value::Object(property_names_schema_object) = property_names_schema {
            if property_names_schema_object.len() == 1
                && property_names_schema_object.contains_key("type")
            {
                // `propertyNames` schema is equivalent to `{"type": "string"}`. We know that because `type` keyword must be defined in order
                // to have only one primitive type (if not defined all the primitive types would be present) and the schema has only 1 property
                // In this case `propertyNames` is only saying that the type of a JSON property is a string, and this is guaranteed by JSON already
                // so the `keyword` is not adding any restriction
                let _ = schema_object.remove("propertyNames");
                true
            } else {
                replace::type_with(
                    property_names_schema_object,
                    PrimitiveTypesBitMap::from_primitive_type(PrimitiveType::String),
                )
            }
        } else {
            // This is impossible because we know that type string is included, so (1) it cannot be a false schema,
            // (2) it cannot be a true schema as it has been checked around the begin of the method
            false
        }
    } else {
        let _ = schema_object.remove("propertyNames");
        true
    };

    updated_schema
}

#[cfg(test)]
mod tests {
    use super::optimise_property_names;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!({"propertyNames": {}}) => json!({}))]
    #[test_case(json!({"type": "integer", "propertyNames": {"type": "string", "minLength": 1}}) => json!({"type": "integer"}))]
    #[test_case(json!({"type": "object", "minProperties": 1, "propertyNames": false}) => json!(false))]
    #[test_case(json!({"type": "object", "minProperties": 1, "propertyNames": {"type": "number"}}) => json!(false))]
    // NOTE: The extraneous properties (after removing `type` object) would be remove by an the handler of `type` keyword
    #[test_case(json!({"type": ["number", "object"], "minProperties": 1, "propertyNames": false}) => json!({"type": "number", "minProperties": 1, "propertyNames": false}))]
    #[test_case(json!({"type": ["number", "object"], "minProperties": 1, "propertyNames": {"type": "number"}}) => json!({"type": "number", "minProperties": 1, "propertyNames": {"type": "number"}}))]
    #[test_case(json!({"type": "object", "propertyNames": {"type": "string"}}) => json!({"type": "object"}))]
    #[test_case(json!({"type": "object", "propertyNames": {"type": "number"}}) => json!({"type": "object", "maxProperties": 0}))]
    #[test_case(json!({"type": "object", "propertyNames": {"minLength": 1}}) => json!({"type": "object", "propertyNames": {"minLength": 1, "type": "string"}}))]
    fn test_optimise_property_names(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = optimise_property_names(&mut schema);
        schema
    }
}
