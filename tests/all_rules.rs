use jsonschema_equivalent::jsonschema_equivalent_ref;
use pathsep::{join_path, path_separator};
use serde_json::{from_str, Value};
use std::io::Write;
use std::str::FromStr;

/// This method does expose the one-liner pretty-print value of a given JSON value
/// Respect the default `Value::to_string` method this ensures that the separators (`:` and `,`) have a space after
/// NOTE: The code is far from being good looking or performing, but this is mostly used to esure that all_rules.md has
/// considently formatted JSON fields
fn pretty_format_json_value(value: &Value) -> String {
    value.to_string().replace(":", ": ").replace(",", ", ")
}

#[derive(Debug)]
struct Rule {
    description: String,
    input_json_schema: Value,
    optimised_json_schema: Value,
}

impl FromStr for Rule {
    type Err = (String, String);
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        // A correct `line` should look like
        // "| DESCRIPTION | `value` | `value` |"
        let mut line_parts = line.split('|');

        if Some("") != line_parts.next() {
            return Err((
                line.to_string(),
                "Not expected characters before the first column".to_string(),
            ));
        }

        let input_json_schema_str = line_parts
            .next()
            .expect("First column, JSON Schema, should be present")
            .trim();
        if !input_json_schema_str.starts_with('`') || !input_json_schema_str.ends_with('`') {
            return Err((
                line.to_string(),
                "First column, JSON Schema, is not wrapped by backquote (`)".to_string(),
            ));
        }
        let input_json_schema: Value = if let Ok(value) =
            from_str(&input_json_schema_str[1..input_json_schema_str.len() - 1])
        {
            let formatted_json = pretty_format_json_value(&value);
            if formatted_json != input_json_schema_str[1..input_json_schema_str.len() - 1] {
                return Err((line.to_string(), format!("First column, JSON Schema, does not contain pretty-formatted JSON. Expected: {}", formatted_json)));
            }
            value
        } else {
            return Err((
                line.to_string(),
                "First column, JSON Schema, does not contain valid JSON".to_string(),
            ));
        };

        let optimised_json_schema_str = line_parts
            .next()
            .expect("Second column, Optimised JSON Schema, should be present")
            .trim();
        if !optimised_json_schema_str.starts_with('`') || !optimised_json_schema_str.ends_with('`')
        {
            return Err((
                line.to_string(),
                "Second column, Optimised JSON Schema, is not wrapped by backquote (`)".to_string(),
            ));
        }
        let optimised_json_schema: Value = if let Ok(value) =
            from_str(&optimised_json_schema_str[1..optimised_json_schema_str.len() - 1])
        {
            let formatted_json = pretty_format_json_value(&value);
            if formatted_json != optimised_json_schema_str[1..optimised_json_schema_str.len() - 1] {
                return Err((line.to_string(), format!("Second column, Optimised JSON Schema, does not contain pretty-formatted JSON. Expected: {}", formatted_json)));
            }
            value
        } else {
            return Err((
                line.to_string(),
                "Second column, Optimised JSON Schema, does not contain valid JSON".to_string(),
            ));
        };

        let description = line_parts
            .next()
            .expect("Third column, description, should be present")
            .trim();

        if Some("") != line_parts.next() {
            return Err((
                line.to_string(),
                "Not expected characters after the Second column".to_string(),
            ));
        }
        if None != line_parts.next() {
            return Err((
                line.to_string(),
                "Not expected columns after the third".to_string(),
            ));
        }

        Ok(Self {
            description: description.to_string(),
            input_json_schema,
            optimised_json_schema,
        })
    }
}

fn load_rules() -> Vec<Rule> {
    let all_rules_file: &str = include_str!(join_path!(env!("CARGO_MANIFEST_DIR"), "all_rules.md"));

    let maybe_rules_iterator = all_rules_file
        .lines()
        .skip_while(|line| line != &"<!-- TABLE START -->")
        .skip(3) // "<!-- TABLE START -->", Table header, Table line separator
        .take_while(|line| line != &"<!-- TABLE END -->")
        .filter_map(|line| {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                None
            } else {
                Some(trimmed_line.parse::<Rule>())
            }
        });

    let mut rules = Vec::<Rule>::new();
    let mut errors = Vec::<(String, String)>::new();
    for maybe_rule in maybe_rules_iterator {
        match maybe_rule {
            Ok(rule) => rules.push(rule),
            Err(error) => errors.push(error),
        }
    }

    if errors.is_empty() {
        rules
    } else {
        panic!(
            "Parsing all_rules.md has failed.\n{}",
            errors
                .iter()
                .enumerate()
                .map(|(index, (line, error))| {
                    format!("{:3}) {}\n     {}", index + 1, line, error)
                })
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

#[test]
fn test_all_rules() {
    let _ = env_logger::builder()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .is_test(true)
        .try_init();

    let mut rules = load_rules();
    let mut errors = Vec::<(&str, Value, &Value, &Value)>::new();
    for rule in &mut rules {
        let input_schema = rule.input_json_schema.clone();
        let optimised_schema = jsonschema_equivalent_ref(&mut rule.input_json_schema);
        if optimised_schema != &rule.optimised_json_schema {
            errors.push((
                &rule.description,
                input_schema,
                &rule.optimised_json_schema,
                optimised_schema,
            ));
        }
    }

    if !errors.is_empty() {
        panic!(
            "Failed to validate {} rules defined in all_rules.md.\n{}",
            errors.len(),
            errors
                .iter()
                .enumerate()
                .map(
                    |(index, (description, input_schema, expected_optimised_schema, optimised_schema))| {
                        format!(
                            "{:3}) {}\n     Input Schema: {}\n     Expected Optimised Schema: {}\n     Optimised Schema: {}",
                            index + 1,
                            description,
                            input_schema,
                            expected_optimised_schema,
                            optimised_schema,
                        )
                    }
                )
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
