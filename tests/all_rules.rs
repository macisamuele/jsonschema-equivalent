use jsonschema_equivalent::jsonschema_equivalent_ref;
use pathsep::{join_path, path_separator};
use serde_json::{from_str, Value};
use std::str::FromStr;

#[derive(Debug)]
struct Rule {
    description: String,
    input_json_schema: Value,
    optimised_json_schema: Value,
}

impl FromStr for Rule {
    type Err = (String, &'static str);
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        // A correct `line` should look like
        // "| DESCRIPTION | `value` | `value` |"
        let mut line_parts = line.split('|');

        if Some("") != line_parts.next() {
            return Err((
                line.to_string(),
                "Not expected characters before the first column",
            ));
        }

        let description = line_parts
            .next()
            .expect("First column, description, should be present")
            .trim();

        let input_json_schema_str = line_parts
            .next()
            .expect("Second column, JSON Schema, should be present")
            .trim();
        if !input_json_schema_str.starts_with('`') || !input_json_schema_str.ends_with('`') {
            return Err((
                line.to_string(),
                "Second column, JSON Schema, is not wrapped by backquote (`)",
            ));
        }
        let input_json_schema: Value = if let Ok(value) =
            from_str(&input_json_schema_str[1..input_json_schema_str.len() - 1])
        {
            value
        } else {
            return Err((
                line.to_string(),
                "Second column, JSON Schema, does not contain valid JSON",
            ));
        };

        let optimised_json_schema_str = line_parts
            .next()
            .expect("Thrid column, Optimised JSON Schema, should be present")
            .trim();
        if !optimised_json_schema_str.starts_with('`') || !optimised_json_schema_str.ends_with('`')
        {
            return Err((
                line.to_string(),
                "Thrid column, Optimised JSON Schema, is not wrapped by backquote (`)",
            ));
        }
        let optimised_json_schema: Value = if let Ok(value) =
            from_str(&optimised_json_schema_str[1..optimised_json_schema_str.len() - 1])
        {
            value
        } else {
            return Err((
                line.to_string(),
                "Thrid column, Optimised JSON Schema, does not contain valid JSON",
            ));
        };

        if Some("") != line_parts.next() {
            return Err((
                line.to_string(),
                "Not expected characters after the thrid column",
            ));
        }
        if None != line_parts.next() {
            return Err((line.to_string(), "Not expected columns after the third"));
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
    let mut errors = Vec::<(String, &'static str)>::new();
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
    let mut rules = load_rules();
    let mut errors = Vec::<(&str, &Value, &Value)>::new();
    for rule in rules.iter_mut() {
        let optimised_schema = jsonschema_equivalent_ref(&mut rule.input_json_schema);
        if optimised_schema != &rule.optimised_json_schema {
            errors.push((
                &rule.description,
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
                    |(index, (description, expected_optimised_schema, optimised_schema))| {
                        format!(
                            "{:3}) {}\n     Expected Optimised Schema: {}\n     Optimised Schema: {}",
                            index + 1,
                            description,
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
