//! [![ci](https://github.com/macisamuele/jsonschema-equivalent/workflows/ci/badge.svg)](https://github.com/macisamuele/jsonschema-equivalent/actions)
//! [![codecov](https://codecov.io/gh/macisamuele/jsonschema-equivalent/branch/master/graph/badge.svg)](https://codecov.io/gh/macisamuele/jsonschema-equivalent)
//! [![Crates.io](https://img.shields.io/crates/v/jsonschema-equivalent.svg)](https://crates.io/crates/jsonschema-equivalent)
//! [![docs.rs](https://docs.rs/jsonschema-equivalent/badge.svg)](https://docs.rs/jsonschema-equivalent/)
//!
//! A JSON Schema optimiser library.
//!
//! The main idea is to flatten the input schema and to remove keywords that are not providing any restriction on the schema.
//! Possible examples are
//!
//! * `{"type": "string", "minimum": 0}` is equivalent to `{"type": "string"}` as `minimum` keyword applies only to numberic types
//! * `{"allOf": [{"type": "integer"}, {"type": "number"}]}` is equivalent to `{"type": "number"}` as `integer` is included in `number`
//! * `{"allOf": [{"type": "integer"}, {"type": "string"}]}` is equivalent to `{"type": ["integer", "string"]}`
//! * ...
//!
//! By flattening and removing extraneous/incongruent keywords we are able to provide a smaller and equivalent schema. Thanks to this, JSON validators can spend CPU cycles on verifying the components that are actually providing restriction on the schema instead of verifying conditions that we know a-priori not been applicable to certain contexts.
//!
//! # How to use
//!
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
    clippy::cast_possible_truncation,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::integer_arithmetic,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::needless_pass_by_value,
    clippy::option_map_unwrap_or,
    clippy::option_map_unwrap_or_else,
    clippy::option_unwrap_used,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::redundant_closure,
    clippy::result_map_unwrap_or_else,
    clippy::result_unwrap_used,
    clippy::trivially_copy_pass_by_ref,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    unreachable_pub,
    unsafe_code,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]

pub(crate) mod constants;
pub(crate) mod helpers;
mod keywords;
pub(crate) mod primitive_type;
use serde_json::Value;

/// Maximum number of allowed rounds to update the schema. This is needed to prevent, unlikely but possible, infinite loop
static MAX_UPDATE_SCHEMA_ITERATIONS: usize = 100;

/// Optimise input schema by removing extraneous/incongruent keys replacing equivalent
/// schemas with more performant ones to be validates against.
#[must_use]
#[inline]
pub fn jsonschema_equivalent_ref(schema: &mut Value) -> &mut Value {
    for _ in 0..MAX_UPDATE_SCHEMA_ITERATIONS {
        if !keywords::update_schema(schema) {
            return schema;
        }
    }
    log::info!(
        "Optimisation, after {} rounds, is not complete for schema={}",
        MAX_UPDATE_SCHEMA_ITERATIONS,
        schema
    );
    schema
}

/// Generate an equivalent schema to the schema provided as input
/// ```rust
/// use jsonschema_equivalent::jsonschema_equivalent;
/// use serde_json::json;
///
/// let equivalent_schema = jsonschema_equivalent(json!(
///     {"type": "string", "minimum": 42}
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
pub(crate) fn init_logger() {
    use std::io::Write;
    let _ = env_logger::builder()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .is_test(true)
        .try_init();
}

#[cfg(test)]
pub(crate) fn base_test_keyword_processor(
    keyword_processing_method: &dyn Fn(&mut Value) -> bool,
    schema: &Value,
) -> Value {
    init_logger();
    let mut processed_schema: Value = schema.clone();
    let is_schema_updated = keyword_processing_method(&mut processed_schema);
    assert_eq!(
        is_schema_updated,
        schema != &processed_schema,
        "is_schema_updated={} but {} != {}",
        is_schema_updated,
        schema,
        processed_schema
    );
    processed_schema
}

#[cfg(test)]
mod tests {
    use super::{jsonschema_equivalent, jsonschema_equivalent_ref};
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!(null) => json!(null))]
    fn test_jsonschema_equivalent_ref(mut schema: Value) -> Value {
        crate::init_logger();
        let _ = jsonschema_equivalent_ref(&mut schema);
        schema
    }

    #[test_case(json!(null) => json!(null))]
    fn test_jsonschema_equivalent(schema: Value) -> Value {
        crate::init_logger();
        jsonschema_equivalent(schema)
    }
}
