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
| `1` | `1` | A non schema passses untouched |
| `true` | `true` | A boolean schema passes untouched (1) |
| `false` | `false` | A boolean schema passes untouched (2) |
| `{"minimum": 1, "type": "array"}` | `{"type": "array"}` | `minimum` keyword has no effect on schema with `type` array |
| `{"minimum": 1, "type": "boolean"}` | `{"type": "boolean"}` | `minimum` keyword has no effect on schema with `type` boolean |
| `{"minimum": 1, "type": "null"}` | `{"type": "null"}` | `minimum` keyword has no effect on schema with `type` null |
| `{"minLength": 1, "type": "integer"}` | `{"type": "integer"}` | `minLength` keyword has no effect on schema with `type` integer |
| `{"minLength": 1, "type": "number"}` | `{"type": "number"}` | `minLength` keyword has no effect on schema with `type` number |
| `{"minimum": 1, "type": "object"}` | `{"type": "object"}` | `minimum` keyword has no effect on schema with `type` object |
| `{"minimum": 1, "type": "string"}` | `{"type": "string"}` | `minimum` keyword has no effect on schema with `type` string |
| `{"type": ["number", "integer"]}` | `{"type": "number"}` | `type` keyword containing `number` and `integer` is as effective as only containing number |
| `{"minItems": 3, "minLength": 1, "minimum": 2, "type": ["number", "string"]}` | `{"minLength": 1, "minimum": 2, "type": ["number", "string"]}` | `minItems` keyword has no effect on schema with `type` string or number |
| `{"additionalProperties": true}` | `{}` | `additionalProperties` keyword has no effect on `true` schema |
| `{"additionalProperties": {}}` | `{}` | `additionalProperties` keyword has no effect on empty schema |
| `{"required": []}` | `{}` | `required` keyword has no effect on empty list |
| `{"exclusiveMaximum": 1, "exclusiveMinimum": 2, "type": "number"}` | `false` | `exclusiveMaximum` keyword lower than `exclusiveMinimum` keyword results into a `false` schema |
| `{"maxItems": 1, "minItems": 2, "type": "array"}` | `false` | `maxItems` keyword lower than `minItems` keyword results into a `false` schema |
| `{"maxLength": 1, "minLength": 2, "type": "string"}` | `false` | `maxLength` keyword lower than `minLength` keyword results into a `false` schema |
| `{"maxProperties": 1, "minProperties": 2, "type": "object"}` | `false` | `maxProperties` keyword lower than `minProperties` keyword results into a `false` schema |
| `{"maxItems": 2, "minItems": 1, "type": "integer"}` | `{"type": "integer"}` | Extraneous `*max*`, `*min*` keywords are removed (if not matching with type) |
| `{"maxItems": 1, "minItems": 2, "type": ["integer", "array"]}` | `{"type": "integer"}` | If `*max*`, `*min*` keywords are creating an impossible range then the corresponding `type` is removed |
<!-- TABLE END -->
