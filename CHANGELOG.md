# Changelog

## Unreleased (YYYY-MM-DD)

* feat: Better optimisation for keyword `type` if it is array
* doc: Update all_rules.md table order to provide a potentially nicer rendering and ensure that a specifc format is preserved
* perf: Reduce amount of clones and reduce amount of string comparisons
* fix: Maximum|Minimum related keyworkds should consider the allowed PrimitiveTypes
* fix: `allOf`, `anyOf`, `not`, `oneOf` are valid keywords for all kewords
* feat: Add rule processing logging
* feat: Enhance helpers functionalities (especially related to `type` handling)
* feat: Loop over `update_schema` for continuos optimisation
* feat: Add tests against JSON-Test-Schema-Suite to preserve and ensure correctness

## 0.1.0 (2020-06-07)

Initial release.

* Handling of extraneous keys with respect to `type` keyword
* Suppression of empty `additionalProperties` and `required`
* Suppression of impossible schemas caused by `min`/`max` related keywords
