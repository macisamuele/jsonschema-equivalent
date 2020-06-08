use std::collections::HashSet;

lazy_static::lazy_static! {
    /// All keywords of Draft4, Draft6 and Draft7
    pub(crate) static ref KEYWORDS: HashSet<&'static str> = [
        "additionalItems",
        "additionalProperties",
        "allOf",
        "anyOf",
        "const",
        "contains",
        "contentEncoding",
        "contentMediaType",
        "dependencies",
        "else",
        "enum",
        "exclusiveMaximum",
        "exclusiveMinimum",
        "format",
        "if",
        "items",
        "maxItems",
        "maxLength",
        "maxProperties",
        "maximum",
        "minItems",
        "minLength",
        "minProperties",
        "minimum",
        "multipleOf",
        "not",
        "oneOf",
        "pattern",
        "patternProperties",
        "properties",
        "propertyNames",
        "required",
        "then",
        "type",
        "uniqueItems",
    ].iter().cloned().collect();

    /// Keywords which contains valid JSON Schema
    ///
    /// This contains the list of keywords defined by the JSON Schema specifications as
    /// * > The value of "..." MUST be a valid JSON Schema.
    /// * > The value of "..." MUST be an object. Each value of this object MUST be a valid JSON Schema.
    /// * > This keyword's value MUST be a non-empty array.  Each item of the array MUST be a valid JSON Schema.
    /// * > The value of "..." MUST be either a valid JSON Schema or an array of valid JSON Schemas.
    pub(crate) static ref KEYWORDS_WITH_SUBSCHEMAS: HashSet<&'static str> = [
        "additionalItems",
        "additionalProperties",
        "allOf",
        "anyOf",
        "const",
        "contains",
        "contentEncoding",
        "contentMediaType",
        "dependencies",
        "else",
        "enum",
        "exclusiveMaximum",
        "exclusiveMinimum",
        "format",
        "if",
        "items",
        "maxItems",
        "maxLength",
        "maxProperties",
        "maximum",
        "minItems",
        "minLength",
        "minProperties",
        "minimum",
        "multipleOf",
        "not",
        "oneOf",
        "pattern",
        "patternProperties",
        "properties",
        "propertyNames",
        "required",
        "then",
        "type",
        "uniqueItems",
    ].iter().cloned().collect();

    /// Keywords value MUST be a valid JSON Schema
    ///
    /// This contains the list of keywords defined by the JSON Schema specifications as
    /// > The value of "..." MUST be a valid JSON Schema.
    pub(crate) static ref KEYWORDS_WITH_DIRECT_SUBSCHEMAS: HashSet<&'static str> = [
        "additionalItems",
        "additionalProperties",
        "contains",
        "else",
        "if",
        "not",
        "propertyNames",
        "then",
    ].iter().cloned().collect();
}
