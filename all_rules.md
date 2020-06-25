# All Rules

:information_source: The reported examples are tested by [`tests/all_rules.rs`](https://github.com/macisamuele/jsonschema-equivalent/blob/master/tests/all_rules.rs)

<!--
    Please do not change the strucutre of the table and/or introduce pipes (`|`) in the table rows.
    Pipes are allowed/required only to read the line.

    This is required as we're verifying the examples via the test.

    Few extra notes:
    * Do not modify/remove the TABLE START and END comment lines
    * Each example should be reported on a single line
      (as the test process the input line by line)
    * Ensure that the JSON Schema are wrapped by backquotes (`)
      due to syntax highlighting
-->
<!-- TABLE START -->
| JSON Schema | Optimised JSON Schema | Desciption |
|-|:-:|:-:|
| `{"additionalItems": {"type": "boolean"}, "items": [{"type": "string"}, {"type": "string"}], "maxItems": 1}` | `{"items": [{"type": "string"}], "maxItems": 1}` | `additionalItems` is meaningless if `maxLength` is at most the length of `items` schemas |
| `{"additionalItems": false, "items": [{"type": "string"}, {"type": "string"}]}` | `{"items": [{"type": "string"}, {"type": "string"}], "maxItems": 2}` | `additionalItems` can be replaced with `maxItems`, which is easier to validate, if `additionalItems` is a false schema |
| `{"additionalItems": false, "items": {"type": "string"}}` | `{"items": {"type": "string"}}` | `additionalItems` is meaningless if `items` is not having an array of schemas |
| `{"additionalProperties": {}}` | `true` | `additionalProperties` keyword has no effect on empty schema |
| `{"additionalProperties": true}` | `true` | `additionalProperties` keyword has no effect on `true` schema |
| `{"allOf": [{"type": "boolean"}, {"type": "number"}]}` | `false` | `allOf` without common types results into a `false` schema |
| `{"allOf": [{"type": "integer"}, {"type": "number"}]}` | `{"type": "integer"}` | only common types survive on `allOf` |
| `{"allOf": [{"type": "integer"}], "type": "boolean"}` | `false` | `allOf` without common types (considering the parent-schema types) results into a `false` schema |
| `{"allOf": [{"type": ["boolean", "integer"]}, {"type": "number"}]}` | `{"type": "integer"}` | only common types survive on `allOf` |
| `{"allOf": [false], "type": "object"}` | `false` | `false` schema in `allOf` keyword results into a `false` schema |
| `{"allOf": [true], "type": "object"}` | `{"type": "object"}` | `true` schema in `allOf` does not add restrictions, so it can be removed |
| `{"const": "some-text", "type": "array"}` | `false` | Incongruent types between `const` value and defined type make the schema a `false` schema |
| `{"enum": ["some-text", 1], "type": "string"}` | `{"enum": ["some-text"], "type": "string"}` | Enum values that cannot be valid according to the schema are elided |
| `{"enum": [1], "type": "string"}` | `false` | No `enum` value can be valid against the schema, so it results into a `false` schema |
| `{"exclusiveMaximum": 1, "exclusiveMinimum": 2, "type": "number"}` | `false` | `exclusiveMaximum` keyword lower than `exclusiveMinimum` keyword results into a `false` schema |
| `{"items": [{"type": "string"}, {"type": "string"}], "maxItems": 1}` | `{"items": [{"type": "string"}], "maxItems": 1}` | `items` defined as array with length longer than `maxItems` is equivalent to not have the `items` schemas, after `maxItems` not defined |
| `{"maxItems": 1, "minItems": 2, "type": "array"}` | `false` | `maxItems` keyword lower than `minItems` keyword results into a `false` schema |
| `{"maxItems": 1, "minItems": 2, "type": ["integer", "array"]}` | `{"type": "integer"}` | If `*max*`, `*min*` keywords are creating an impossible range then the corresponding `type` is removed |
| `{"maxItems": 2, "minItems": 1, "type": "integer"}` | `{"type": "integer"}` | Extraneous `*max*`, `*min*` keywords are removed (if not matching with type) |
| `{"maxLength": 1, "minLength": 2, "type": "string"}` | `false` | `maxLength` keyword lower than `minLength` keyword results into a `false` schema |
| `{"maxProperties": 1, "minProperties": 2, "type": "object"}` | `false` | `maxProperties` keyword lower than `minProperties` keyword results into a `false` schema |
| `{"minimum": 1, "type": "array"}` | `{"type": "array"}` | `minimum` keyword has no effect on schema with `type` array |
| `{"minimum": 1, "type": "boolean"}` | `{"type": "boolean"}` | `minimum` keyword has no effect on schema with `type` boolean |
| `{"minimum": 1, "type": "null"}` | `{"type": "null"}` | `minimum` keyword has no effect on schema with `type` null |
| `{"minimum": 1, "type": "object"}` | `{"type": "object"}` | `minimum` keyword has no effect on schema with `type` object |
| `{"minimum": 1, "type": "string"}` | `{"type": "string"}` | `minimum` keyword has no effect on schema with `type` string |
| `{"minItems": 0, "type": "array"}` | `{"type": "array"}` | `minItems` set to 0 has the same effect of not having the keyword defined |
| `{"minItems": 3, "minLength": 1, "minimum": 2, "type": ["number", "string"]}` | `{"minLength": 1, "minimum": 2, "type": ["number", "string"]}` | `minItems` keyword has no effect on schema with `type` string or number |
| `{"minLength": 0, "type": "string"}` | `{"type": "string"}` | `minLength` set to 0 has the same effect of not having the keyword defined |
| `{"minLength": 1, "type": "integer"}` | `{"type": "integer"}` | `minLength` keyword has no effect on schema with `type` integer |
| `{"minLength": 1, "type": "number"}` | `{"type": "number"}` | `minLength` keyword has no effect on schema with `type` number |
| `{"minProperties": 0, "type": "object"}` | `{"type": "object"}` | `minProperties` set to 0 has the same effect of not having the keyword defined |
| `{"minProperties": 1, "propertyNames": false, "type": ["number", "object"]}` | `{"type": "number"}` | `propertyNames` as `false` schema, with the requirement of a property defined in case of `type` object prevents a JSON object to ever be valid |
<!-- TABLE END -->
