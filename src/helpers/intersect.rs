use crate::helpers::{
    common_values_and_deduplicate, join_and_deduplicate, replace, types::PrimitiveTypesBitMap,
};
use serde_json::{map::Entry, Map, Value};
use std::ops::Deref;
use std::ops::DerefMut;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum IntersectStatus<'s> {
    /// The updated `schema` fully includes the JSON Schema limitations imposed by `other_schema`
    Complete {
        schema: &'s mut Value,
        updated_schema: bool,
    },
    /// The updated `schema` partially includes the JSON Schema limitations imposed by `other_schema`
    /// This means that `other_schema` cannot be removed without altering the JSON Schema itself
    Partial {
        schema: &'s mut Value,
        updated_schema: bool,
    },
}

impl Deref for IntersectStatus<'_> {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Complete { schema, .. } | Self::Partial { schema, .. } => schema,
        }
    }
}

impl DerefMut for IntersectStatus<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Complete { schema, .. } | Self::Partial { schema, .. } => schema,
        }
    }
}

impl IntersectStatus<'_> {
    fn is_schema_updated(&self) -> bool {
        match self {
            Self::Complete { updated_schema, .. } | Self::Partial { updated_schema, .. } => {
                *updated_schema
            }
        }
    }
}

static DEFFERRED_KEYWORDS: &[&str] = &[
    "additionalItems",
    "additionalProperties",
    "items",
    "patternProperties",
    "properties",
];

/// Handle the intersection of schemas focusing only on `items` and `additionalItems` keywords
fn handle_items_related_keywords(
    _schema_object: &mut Map<String, Value>,
    _other_schema: &Map<String, Value>,
) -> bool {
    false
}

/// Handle the intersection of schemas focusing only on `properties`, `additionalProperties` and `patternProperties` keywords
fn handle_properties_related_keywords(
    _schema_object: &mut Map<String, Value>,
    _other_schema: &Map<String, Value>,
) -> bool {
    false
}

/// Intesection of `schema` with `other_schema`.
/// The method is currently not covering all the possible cases, but the one that are covered are _hopefully_ correct and tested.
///
/// WARNINGs:
///     * The operation is done in-place within `schema`, so if you need `schema` to be left untouched ensure to pass a clone ;)
///     * The method might not be able to merge in `schema` all the restrictions imposed by `other_schema`.
///       This might happen because the logic has not been fully implemented yet as well as it is just not possible (`oneOf` for example cannot be merged).
///       For this reason you should check if the result is `IntersectStatus::Complete` or `IntersectStatus::Partial`.
#[allow(dead_code)]
// The method body is very long, but I do argue that it is very simple to follow and creating helper methods would make understanding even harder
#[allow(clippy::too_many_lines)]
pub(crate) fn intersection_schema<'s>(
    schema: &'s mut Value,
    other_schema: &Value,
) -> IntersectStatus<'s> {
    let other_schema_object = match other_schema {
        Value::Object(map) => map,
        Value::Bool(false) => {
            // if `other_schema` is a `false` schema then regrdless of `schema` all the values will be invalid. So the resulting schema is a `false` schema
            let _ = replace::with_false_schema(schema);
            return IntersectStatus::Complete {
                schema,
                updated_schema: true,
            };
        }
        _ => {
            // if `other_schema` is a `true` schema then only `schema` will contribute to validation constraints
            // otherwise `other_schema` is not really a schema, so nothing to be intersected
            return IntersectStatus::Complete {
                schema,
                updated_schema: false,
            };
        }
    };
    let schema_object = match schema {
        Value::Object(map) => map,
        Value::Bool(true) => {
            // if `self` is a `true` schema then only `other_schema` will contribute to validation constraints
            let _ = std::mem::replace(schema, other_schema.clone());
            return IntersectStatus::Complete {
                schema,
                updated_schema: true,
            };
        }
        _ => {
            // if `self` is a `false` schema then regrdless of `other` all the values will be invalid
            // otherwise `schema` is not really a schema, so nothing to be intersected
            return IntersectStatus::Complete {
                schema,
                updated_schema: false,
            };
        }
    };

    let mut is_complete_intersection = true;
    let mut updated_schema = false;

    for (key, other_value) in other_schema_object {
        if DEFFERRED_KEYWORDS.contains(&key.as_str()) {
            continue;
        }
        match schema_object.entry(key) {
            Entry::Vacant(entry) => {
                let _ = entry.insert(other_value.clone());
                updated_schema = true;
            }
            Entry::Occupied(mut entry) => {
                let schema_value = entry.get_mut();
                if schema_value != other_value {
                    // Schema had the key, so we need to decide how to "merge" `schema_value` with `other_value`
                    // NOTE: We might decide to not merge in certain keys!
                    match key.as_ref() {
                        "allOf" | "required" => {
                            if let (Value::Array(schema_items), Value::Array(other_items)) =
                                (schema_value, other_value)
                            {
                                updated_schema |= join_and_deduplicate(schema_items, other_items);
                            }
                        }
                        "const" | "contentEncoding" | "contentMediaType" | "format" => {
                            if schema_value != other_value {
                                let _ = replace::with_false_schema(schema);
                                return IntersectStatus::Complete {
                                    schema,
                                    updated_schema: true,
                                };
                            }
                        }
                        "contains" | "propertyNames" => {
                            updated_schema |=
                                intersection_schema(schema_value, other_value).is_schema_updated();
                        }
                        "enum" => {
                            if let (Value::Array(schema_items), Value::Array(other_items)) =
                                (schema_value, other_value)
                            {
                                updated_schema |=
                                    common_values_and_deduplicate(schema_items, other_items);
                                if schema_items.is_empty() {
                                    let _ = replace::with_false_schema(schema);
                                    return IntersectStatus::Complete {
                                        schema,
                                        updated_schema: true,
                                    };
                                }
                            };
                        }
                        "exclusiveMaximum" | "maxItems" | "maxLength" | "maxProperties"
                        | "maximum" => {
                            if other_value.as_f64() < schema_value.as_f64() {
                                let _ = entry.insert(other_value.clone());
                                updated_schema |= true;
                            }
                        }
                        "exclusiveMinimum" | "minItems" | "minLength" | "minProperties"
                        | "minimum" => {
                            if other_value.as_f64() > schema_value.as_f64() {
                                let _ = entry.insert(other_value.clone());
                                updated_schema |= true;
                            }
                        }
                        "type" => {
                            let schema_primitive_types =
                                PrimitiveTypesBitMap::from_schema_value(schema_object.get("type"));
                            let other_primitive_types = PrimitiveTypesBitMap::from_schema_value(
                                other_schema_object.get("type"),
                            );

                            let final_primiive_types =
                                schema_primitive_types & other_primitive_types;
                            if schema_primitive_types != final_primiive_types {
                                updated_schema = true;
                                if !replace::type_with(schema_object, final_primiive_types)
                                    || schema_object.get("type") == None
                                {
                                    let _ = replace::with_false_schema(schema);
                                    return IntersectStatus::Complete {
                                        schema,
                                        updated_schema: true,
                                    };
                                }
                            }
                        }
                        "uniqueItems" => {
                            if &Value::Bool(true) == other_value {
                                let old_value = entry.insert(Value::Bool(true));
                                updated_schema |= old_value != Value::Bool(true);
                            }
                        }

                        "additionalItems"
                        | "additionalProperties"
                        | "items"
                        | "patternProperties"
                        | "properties " => {
                            // Deferred to `handle_items_related_keywords` or `handle_properties_related_keywords`
                        }

                        // Keywords for which we have not tried to implement the intersection logic
                        "anyOf" | "dependencies" | "else" | "if" | "multipleOf" | "not"
                        | "oneOf" | "pattern" | "then" => {
                            is_complete_intersection = false;
                        }

                        // TODO: Propose implementation for properties
                        //  properties could eventually be "merged" but depends on the presence/absence of additionalProperties/patternProperties
                        //  * if no additionalProperties/patternProperties are defined (or is true schema) then just merge properties
                        //  * if additionalProperties are defined and the intersection of the schemas is not a `false` schema and no patternProperties are defined
                        _ => {
                            log::debug!("Unrecognized keyword: {}", key);
                            is_complete_intersection = false;
                        }
                    }
                }
            }
        };
    }

    updated_schema |= handle_items_related_keywords(schema_object, other_schema_object);
    updated_schema |= handle_properties_related_keywords(schema_object, other_schema_object);

    if is_complete_intersection {
        IntersectStatus::Complete {
            schema,
            updated_schema,
        }
    } else {
        IntersectStatus::Partial {
            schema,
            updated_schema,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::intersection_schema;
    use serde_json::{json, Value};
    use test_case::test_case;

    fn test<I1, I2>(
        schema: &Value,
        other: &Value,
        optimise_schema_fn: &dyn Fn(&mut Value, &Value) -> bool,
        expected_schema: &Value,
        valid: I1,
        invalid: I2,
    ) where
        I1: Into<Option<Value>>,
        I2: Into<Option<Value>>,
    {
        let valid: Option<_> = valid.into();
        let invalid: Option<_> = invalid.into();
        let schema_all_of = json!({"allOf": [schema, other]});
        if let Some(valid_instance) = &valid {
            assert!(
                jsonschema::is_valid(&schema_all_of, valid_instance),
                "{} is not valid against {} [before intersection]",
                valid_instance,
                schema_all_of
            );
        }
        if let Some(invalid_instance) = &invalid {
            assert!(
                !jsonschema::is_valid(&schema_all_of, invalid_instance),
                "{} is valid against {} [before intersection]",
                invalid_instance,
                schema_all_of
            );
        }

        let mut optimised_schema = schema.clone();
        let is_schema_updated = optimise_schema_fn(&mut optimised_schema, other);
        assert_eq!(&optimised_schema, expected_schema);

        if is_schema_updated {
            assert_ne!(
                &optimised_schema, schema,
                "The schema seems not updated, but it was. Original schema = {}, Final schema = {}",
                schema, optimised_schema
            );
        } else {
            assert_eq!(
                &optimised_schema, schema,
                "The schema seems updated, but it was not supposed to be. Original schema = {}, Final schema = {}",
                schema, optimised_schema
            );
        }

        if let Some(valid_instance) = &valid {
            assert!(
                jsonschema::is_valid(&optimised_schema, valid_instance),
                "{} is not valid against {} [after intersection]",
                valid_instance,
                optimised_schema
            );
        }
        if let Some(invalid_instance) = &invalid {
            assert!(
                !jsonschema::is_valid(&optimised_schema, invalid_instance),
                "{} is valid against {} [after intersection]",
                invalid_instance,
                optimised_schema
            );
        }
    }

    // Empty, true or false schema handling
    #[test_case(
        &json!({}),
        &json!({}),
        &json!({}),
        json!(null),
        None
    )]
    #[test_case(
        &json!("not-a-schema"),
        &json!({}),
        &json!("not-a-schema"),
        None,
        None
    )]
    #[test_case(
        &json!({}),
        &json!("not-a-schema"),
        &json!({}),
        None,
        None
    )]
    #[test_case(
        &json!({"type": "string"}),
        &json!(true),
        &json!({"type": "string"}),
        json!("string"),
        json!(false)
    )]
    #[test_case(
        &json!(true),
        &json!({"type": "string"}),
        &json!({"type": "string"}),
        json!("string"),
        json!(false)
    )]
    #[test_case(
        &json!({"type": "string"}),
        &json!(false),
        &json!(false),
        None,
        json!("whatever")
    )]
    #[test_case(
        &json!(false),
        &json!({"type": "string"}),
        &json!(false),
        None,
        json!("whatever")
    )]
    #[test_case(
        &json!({}),
        &json!({"minimum": 1}),
        &json!({"minimum": 1}),
        json!(2),
        json!(0)
    )]
    // Merge of not duplicated keywords
    #[test_case(
        &json!({"maximum": 2}),
        &json!({"minimum": 1}),
        &json!({"maximum": 2, "minimum": 1}),
        json!(1.5),
        json!(0)
    )]
    // Single keyworkds tests. NOTE: Some cases might be "impossible" as other optimisations would remove them (as joining {"type":"string"} with {"type":"number"})
    #[test_case(
        &json!({"allOf": [true]}),
        &json!({"allOf": [true]}),
        &json!({"allOf": [true]}),
        json!(null),
        None
    )]
    #[test_case(
        &json!({"allOf": [{"type": "string"}]}),
        &json!({"allOf": [{"type": "number"}]}),
        &json!({"allOf": [{"type": "string"}, {"type": "number"}]}),
        None,
        json!(1)
    )]
    #[test_case(
        &json!({"allOf": [{"type": "string"}]}),
        &json!({"allOf": [{"type": "number"}, {"type": "string"}]}),
        &json!({"allOf": [{"type": "string"}, {"type": "number"}]}),
        None,
        json!("string")
    )]
    #[test_case(
        &json!({"const": true}),
        &json!({"const": true}),
        &json!({"const": true}),
        json!(true),
        None
    )]
    #[test_case(
        &json!({"const": false}),
        &json!({"const": true}),
        &json!(false),
        None,
        json!(null)
    )]
    #[test_case(
        &json!({"contentEncoding": "base64"}),
        &json!({"contentEncoding": "base64"}),
        &json!({"contentEncoding": "base64"}),
        json!("c29tZXRoaW5nCg=="),  // `echo "something" | base64 -` == "c29tZXRoaW5nCg==""
        json!("something")
    )]
    #[test_case(
        &json!({"contentEncoding": "base64"}),
        &json!({"contentEncoding": "7bit"}),
        &json!(false),
        None,
        json!("something")
    )]
    #[test_case(
        &json!({"contentMediaType": "application/json"}),
        &json!({"contentMediaType": "application/json"}),
        &json!({"contentMediaType": "application/json"}),
        json!(r#"{"key": "value"}"#),
        json!("something")
    )]
    #[test_case(
        &json!({"contentMediaType": "application/json"}),
        &json!({"contentMediaType": "application/png"}),
        &json!(false),
        None,
        json!("something")
    )]
    #[test_case(
        &json!({"contains": {"type": "string"}}),
        &json!({}),
        &json!({"contains": {"type": "string"}}),
        json!(["string"]),
        json!([1])
    )]
    #[test_case(
        &json!({"contains": {"type": "string"}}),
        &json!({"contains": {"minLength": 1}}),
        &json!({"contains": {"type": "string", "minLength": 1}}),
        json!(["string"]),
        json!([1])
    )]
    #[test_case(
        &json!({"enum": [1, 2, 3]}),
        &json!({"enum": [1, 3, 5]}),
        &json!({"enum": [1, 3]}),
        json!(1),
        json!(2)
    )]
    #[test_case(
        &json!({"enum": [1, 2, 3]}),
        &json!({"enum": [4, 5, 6]}),
        &json!(false),
        None,
        json!(1)
    )]
    #[test_case(
        &json!({"exclusiveMaximum": 1}),
        &json!({"exclusiveMaximum": 2}),
        &json!({"exclusiveMaximum": 1}),
        json!(0.5),
        json!(1.5)
    )]
    #[test_case(
        &json!({"exclusiveMinimum": 1}),
        &json!({"exclusiveMinimum": 2}),
        &json!({"exclusiveMinimum": 2}),
        json!(2.5),
        json!(1.5)
    )]
    #[test_case(
        &json!({"format": "date"}),
        &json!({"format": "date"}),
        &json!({"format": "date"}),
        json!("1970-01-01"),
        json!("19700101")
    )]
    #[test_case(
        &json!({"format": "date"}),
        &json!({"format": "date-time"}),
        &json!(false),
        None,
        json!("1970-01-01")
    )]
    #[test_case(
        &json!({"maximum": 1}),
        &json!({"maximum": 2}),
        &json!({"maximum": 1}),
        json!(0.5),
        json!(1.5)
    )]
    #[test_case(
        &json!({"maxItems": 1}),
        &json!({"maxItems": 2}),
        &json!({"maxItems": 1}),
        json!([1]),
        json!([1, 2])
    )]
    #[test_case(
        &json!({"maxLength": 1}),
        &json!({"maxLength": 2}),
        &json!({"maxLength": 1}),
        json!("s"),
        json!("st")
    )]
    #[test_case(
        &json!({"maxProperties": 1}),
        &json!({"maxProperties": 2}),
        &json!({"maxProperties": 1}),
        json!({"p1": null}),
        json!({"p1" :null, "p2":null})
    )]
    #[test_case(
        &json!({"minimum": 1}),
        &json!({"minimum": 2}),
        &json!({"minimum": 2}),
        json!(2.5),
        json!(1.5)
    )]
    #[test_case(
        &json!({"minItems": 1}),
        &json!({"minItems": 2}),
        &json!({"minItems": 2}),
        json!([1,2]),
        json!([1])
    )]
    #[test_case(
        &json!({"minLength": 1}),
        &json!({"minLength": 2}),
        &json!({"minLength": 2}),
        json!("st"),
        json!("s")
    )]
    #[test_case(
        &json!({"minProperties": 1}),
        &json!({"minProperties": 2}),
        &json!({"minProperties": 2}),
        json!({"p1": null, "p2": null}),
        json!({"p1": null})
    )]
    #[test_case(
        &json!({"propertyNames": true}),
        &json!({"propertyNames": {"type": "number"}}),
        &json!({"propertyNames": {"type": "number"}}),
        None,
        None
    )]
    #[test_case(
        &json!({"propertyNames": {"type": "number"}}),
        &json!({"propertyNames": true}),
        &json!({"propertyNames": {"type": "number"}}),
        None,
        None
    )]
    #[test_case(
        &json!({"propertyNames": false}),
        &json!({"propertyNames": {"type": "number"}}),
        &json!({"propertyNames": false}),
        None,
        None
    )]
    #[test_case(
        &json!({"propertyNames": {"type": "number"}}),
        &json!({"propertyNames": false}),
        &json!({"propertyNames": false}),
        None,
        None
    )]
    #[test_case(
        &json!({"propertyNames": {"type": "string"}}),
        &json!({"propertyNames": {"type": "number"}}),
        &json!({"propertyNames": false}),
        None,
        None
    )]
    #[test_case(
        &json!({"required": []}),
        &json!({"required": ["p1"]}),
        &json!({"required": ["p1"]}),
        json!({"p1": 1}),
        json!({})
    )]
    #[test_case(
        &json!({"required": ["p1"]}),
        &json!({"required": ["p2"]}),
        &json!({"required": ["p1", "p2"]}),
        json!({"p1": 1, "p2": 2}),
        json!({"p1": 1})
    )]
    #[test_case(
        &json!({"required": ["p1"]}),
        &json!({"required": ["p1"]}),
        &json!({"required": ["p1"]}),
        json!({"p1": 1}),
        json!({})
    )]
    #[test_case(
        &json!({"type": "integer"}),
        &json!({"type": "number"}),
        &json!({"type": "integer"}),
        json!(1),
        json!(1.5)
    )]
    #[test_case(
        &json!({"type": ["integer", "object", "string"]}),
        &json!({"type": ["integer", "string"]}),
        &json!({"type": ["integer", "string"]}),
        json!("string"),
        json!({})
    )]
    #[test_case(
        &json!({"type": ["integer", "object", "string"]}),
        &json!({"type": "number"}),
        &json!({"type": "integer"}),
        json!(1),
        json!(2.3)
    )]
    #[test_case(
        &json!({"type": "object"}),
        &json!({"type": "string"}),
        &json!(false),
        None,
        json!(null)
    )]
    #[test_case(
        &json!({"uniqueItems": false}),
        &json!({"uniqueItems": true}),
        &json!({"uniqueItems": true}),
        json!([1,2]),
        json!([1,1])
    )]
    fn test_intersection_schema<I1, I2>(
        schema: &Value,
        other: &Value,
        expected_schema: &Value,
        valid: I1,
        invalid: I2,
    ) where
        I1: Into<Option<Value>>,
        I2: Into<Option<Value>>,
    {
        test(
            schema,
            other,
            &|mut schema, other| {
                let intersect_status = intersection_schema(&mut schema, other);
                assert_eq!(&*intersect_status, expected_schema);
                intersect_status.is_schema_updated()
            },
            expected_schema,
            valid,
            invalid,
        )
    }
}
