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
| Desciption | JSON Schema | Optimised JSON Schema |
|-|:-:|:-:|
| A non schema passses untouched | `1` | `1` |
| A boolean schema passes untouched (1) | `true` | `true` |
| A boolean schema passes untouched (2) | `false` | `false` |
| `minimum` keyword has no effect on schema with `type` array | `{"minimum": 1, "type": "array"}` | `{"type": "array"}` |
| `minimum` keyword has no effect on schema with `type` boolean | `{"minimum": 1, "type": "boolean"}` | `{"type": "boolean"}` |
| `minimum` keyword has no effect on schema with `type` null | `{"minimum": 1, "type": "null"}` | `{"type": "null"}` |
| `minLength` keyword has no effect on schema with `type` integer | `{"minLength": 1, "type": "integer"}` | `{"type": "integer"}` |
| `minLength` keyword has no effect on schema with `type` number | `{"minLength": 1, "type": "number"}` | `{"type": "number"}` |
| `minimum` keyword has no effect on schema with `type` object | `{"minimum": 1, "type": "object"}` | `{"type": "object"}` |
| `minimum` keyword has no effect on schema with `type` string | `{"minimum": 1, "type": "string"}` | `{"type": "string"}` |
| `additionalProperties` keyword has no effect on `true` schema | `{"additionalProperties": true}` | `{}` |
| `additionalProperties` keyword has no effect on empty schema | `{"additionalProperties": {}}` | `{}` |
| `required` keyword has no effect on empty list | `{"required": []}` | `{}` |
| `exclusiveMaximum` keyword lower than `exclusiveMinimum` keyword results into a `false` schema | `{"type": "number", "exclusiveMaximum": 1, "exclusiveMinimum": 2}` | `false` |
| `maxItems` keyword lower than `minItems` keyword results into a `false` schema | `{"type": "array", "maxItems": 1, "minItems": 2}` | `false` |
| `maxLength` keyword lower than `minLength` keyword results into a `false` schema | `{"type": "string", "maxLength": 1, "minLength": 2}` | `false` |
| `maxProperties` keyword lower than `minProperties` keyword results into a `false` schema | `{"type": "object", "maxProperties": 1, "minProperties": 2}` | `false` |
| `maximum` keyword lower than `minimum` keyword results into a `false` schema | `{"type": "number", "maximum": 1, "minimum": 2}` | `false` |
<!-- TABLE END -->
