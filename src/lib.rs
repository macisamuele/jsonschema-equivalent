//! # jsonschema-equivalent
//!
//! A JSON Schema optimiser library.
//!
//! The main idea is to flatten the input schema and to remove keywords that are not providing any restriction on the schema.
//! Possible examples are
//! * `{"type": "string", "minimum": 0}` is equivalent to `{"type": "string"}` as `minimum` keyword applies only to numberic types
//! * `{"allOf": [{"type": "integer"}, {"type": "number"}]}` is equivalent to `{"type": "number"}` as `integer` is included in `number`
//! * `{"allOf": [{"type": "integer"}, {"type": "string"}]}` is equivalent to `{"type": ["integer", "string"]}`
//! * ...
//!
//! By flattening and removing extraneous/incongruent keywords we are able to provide a smaller and equivalent schema. Thanks to this, JSON validators can spend CPU cycles on verifying the components that are actually providing restriction on the schema instead of verifying conditions that we know a-priori not been applicable to certain contexts.
//!
//! ## How to use
//! ```toml
//! # Cargo.toml
//! jsonschema-equivalent = "0"
//! ```
//!
//! To validate documents against some schema and get validation errors (if any):
//!
//! ```rust
//! use jsonschema_equivalent::jsonschema_equivalent;
//! use serde_json::json;
//!
//! let schema = json!({"type": "string", "minimum": 42});
//! println!("Original schema: {}", schema);
//! let equivalent_schema = jsonschema_equivalent(schema);
//! println!("Equivalent schema: {}", equivalent_schema);
//! ```
#![warn(
    clippy::pedantic,
    clippy::doc_markdown,
    clippy::redundant_closure,
    clippy::explicit_iter_loop,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::print_stdout,
    clippy::integer_arithmetic,
    clippy::cast_possible_truncation,
    clippy::result_unwrap_used,
    clippy::result_map_unwrap_or_else,
    clippy::option_unwrap_used,
    clippy::option_map_unwrap_or_else,
    clippy::option_map_unwrap_or,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]

// TODO: deal with schema elision.
// An example is: `{"type": "number", "minimum": 2, "maximum": 1}`
// the schema is impossible and so it would be equivalent to `false`
// This means that a schema can be migrated from JSON object to JSON boolean

mod keywords;
use serde_json::Value;

/// Optimise input schema by removing extraneous/incongruent keys replacing equivalent
/// schemas with more performant ones to be validates against.
#[must_use]
#[inline]
pub fn jsonschema_equivalent_ref(schema: &mut Value) -> &mut Value {
    keywords::update_schema(schema)
}

/// Generate an equivalent schema to the schema provided as input
/// ```rust
/// use jsonschema_equivalent::jsonschema_equivalent;
/// use serde_json::json;
///
/// let equivalent_schema = jsonschema_equivalent(json!(
///     {"type": "string"}
/// ));
/// assert_eq!(equivalent_schema, json!({"type": "string"}))
/// ```
#[must_use]
#[inline]
pub fn jsonschema_equivalent(mut schema: Value) -> Value {
    let _ = jsonschema_equivalent_ref(&mut schema);
    schema
}

#[cfg(test)]
mod tests {
    use super::{jsonschema_equivalent, jsonschema_equivalent_ref};
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!(null) => json!(null))]
    fn test_jsonschema_equivalent_ref(mut schema: Value) -> Value {
        let _ = jsonschema_equivalent_ref(&mut schema);
        schema
    }

    #[test_case(json!(null) => json!(null))]
    fn test_jsonschema_equivalent(schema: Value) -> Value {
        jsonschema_equivalent(schema)
    }
}
