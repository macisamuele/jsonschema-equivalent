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
<!-- TABLE END -->
