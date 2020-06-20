use json_schema_test_suite::{json_schema_test_suite, TestCase};

use jsonschema::{Draft, JSONSchema};
use jsonschema_equivalent::jsonschema_equivalent;

#[json_schema_test_suite(
    "JSON-Schema-Test-Suite", "draft4",
    {"optional_bignum_0_0", "optional_bignum_2_0"}
)]
#[json_schema_test_suite("JSON-Schema-Test-Suite", "draft6")]
#[json_schema_test_suite(
    "JSON-Schema-Test-Suite", "draft7", 
    {"optional_format_idn_hostname_0_11", "optional_format_idn_hostname_0_6", "optional_format_idn_hostname_0_7"}
)]
fn draft_test(_server_address: &str, test_case: TestCase) {
    let draft_version = match test_case.draft_version.as_ref() {
        "draft4" => Draft::Draft4,
        "draft6" => Draft::Draft6,
        "draft7" => Draft::Draft7,
        _ => panic!("Unsupported draft"),
    };

    let original_is_valid = JSONSchema::compile(&test_case.schema, Some(draft_version))
        .unwrap()
        .is_valid(&test_case.instance);

    if original_is_valid != test_case.is_valid {
        panic!(
            "`jsonschema` crate does not correctly validate the instance ({}) against the schema ({})",
            test_case.instance, test_case.schema
        );
    }

    let optimised_schema = jsonschema_equivalent(test_case.schema.clone());
    let optimised_is_valid = JSONSchema::compile(&optimised_schema, Some(draft_version))
        .unwrap_or_else(|_| {
            panic!(
                "Optimisation of schema resulted into an invalid schema. jsonschema_equivalent({}) = {}",
                test_case.schema,
                optimised_schema
            );
        })
        .is_valid(&test_case.instance);

    assert_eq!(
        original_is_valid, optimised_is_valid,
        "Optimisation of schema changes the schema constraints. jsonschema_equivalent({}) = {}",
        test_case.schema, optimised_schema
    );
}
