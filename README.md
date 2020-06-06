# jsonschema-equivalent

[![ci](https://github.com/macisamuele/jsonschema-equivalent/workflows/ci/badge.svg)](https://github.com/macisamuele/jsonschema-equivalent/actions)
[![codecov](https://codecov.io/gh/macisamuele/jsonschema-equivalent/branch/master/graph/badge.svg)](https://codecov.io/gh/macisamuele/jsonschema-equivalent)
[![Crates.io](https://img.shields.io/crates/v/jsonschema-equivalent.svg)](https://crates.io/crates/jsonschema-equivalent)
[![docs.rs](https://docs.rs/jsonschema-equivalent/badge.svg)](https://docs.rs/jsonschema-equivalent/)

A JSON Schema optimiser library.

The main idea is to flatten the input schema and to remove keywords that are not providing any restriction on the schema.
Possible examples are

* `{"type": "string", "minimum": 0}` is equivalent to `{"type": "string"}` as `minimum` keyword applies only to numberic types
* `{"allOf": [{"type": "integer"}, {"type": "number"}]}` is equivalent to `{"type": "number"}` as `integer` is included in `number`
* `{"allOf": [{"type": "integer"}, {"type": "string"}]}` is equivalent to `{"type": ["integer", "string"]}`
* and many more (the complete list is visible on [all rules page](all_rules.md)).

By flattening and removing extraneous/incongruent keywords we are able to provide a smaller and equivalent schema. Thanks to this, JSON validators can spend CPU cycles on verifying the components that are actually providing restriction on the schema instead of verifying conditions that we know a-priori not been applicable to certain contexts.

## How to use

```toml
# Cargo.toml
jsonschema-equivalent = "0"
```

To validate documents against some schema and get validation errors (if any):

```rust
use jsonschema_equivalent::jsonschema_equivalent;
use serde_json::json;

fn main() {
    let schema = json!({"type": "string", "minimum": 42});
    println!("Original schema: {}", schema);
    let equivalent_schema = jsonschema_equivalent(schema);
    println!("Equivalent schema: {}", equivalent_schema);
}
```

**NOTE**. This library is in early development, so it might not be covering all the possible schema-reductions pattern.
If you idenify new ways to optimise the schema feel free to open an issue describing the approach (with an example) or providing a pull request as well.
Contribution is welcome.
