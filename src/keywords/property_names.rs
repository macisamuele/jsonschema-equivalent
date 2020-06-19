use crate::helpers::{is, replace, types::get_primitive_types};
use crate::primitive_type::PrimitiveType;
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
#[rule_processor_logger::log_processing]
pub(crate) fn optimise_property_names(schema: &mut Value) -> bool {
    let schema_object = if let Some(value) = schema.as_object_mut() {
        value
    } else {
        return false;
    };
    let mut schema_primitive_types = get_primitive_types(schema_object.get("type"));
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

    if schema_primitive_types.contains(&PrimitiveType::Object) {
        let property_names_types = get_primitive_types(property_names_schema.get("type"));
        if schema_min_properties > 0.0
            && (
                // and we know that any possible property won't be valid against `propertyNames` schema
                is::false_schema(property_names_schema)
                    || !property_names_types.contains(&PrimitiveType::String)
            )
        {
            let _ = schema_primitive_types.remove(&PrimitiveType::Object);
            if replace::type_with(schema_object, &schema_primitive_types) {
                if schema_object.get("type") == None {
                    // If the only supported type was object then the schema is just a `false` schema
                    let _ = replace::with_false_schema(schema);
                }
                // We were able to modify the schema on `replace::type_with`
                true
            } else {
                false
            }
        } else if property_names_types.len() == 1 {
            if !property_names_types.contains(&PrimitiveType::String) {
                // No properties can be accepted, so we can simplify the schema by still accepting `type` object but ensuring
                // that no properties can be defined
                let _ = schema_object.insert("maxProperties".to_string(), Value::from(0));
                let _ = schema_object.remove("propertyNames");
                true
            } else if property_names_schema
                .as_object()
                .map_or(false, |value| value.len() == 1)
            {
                // `propertyNames` schema is equivalent to `{"type": "string"}`. We know that because `type` keyword must be defined in order
                // to have only one primitive type (if not defined all the primitive types would be present) and the schema has only 1 property
                // In this case `propertyNames` is only saying that the type of a JSON property is a string, and this is guaranteed by JSON already
                // so the `keyword` is not adding any restriction
                let _ = schema_object.remove("propertyNames");
                true
            } else {
                false
            }
        } else {
            if property_names_types.contains(&PrimitiveType::String) {
                // More than a type is supported in `propertyNames` schema and one is String. They only string is supported
                if let Value::Object(property_names_schema_object) = property_names_schema {
                    let _ = property_names_schema_object.insert(
                        "type".to_string(),
                        Value::String(PrimitiveType::String.to_string()),
                    );
                }
            } else {
                // No properties can be accepted, so we can simplify the schema by still accepting `type` object but ensuring
                // that no properties can be defined
                let _ = schema_object.insert("maxProperties".to_string(), Value::from(0));
                let _ = schema_object.remove("propertyNames");
            }
            true
        }
    } else {
        let _ = schema_object.remove("propertyNames");
        true
    }
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
