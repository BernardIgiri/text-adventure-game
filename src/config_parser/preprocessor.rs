use std::{fs, path::Path};

use ini::Ini;

use crate::error;

const STR_DELIMITER: &str = r#"""""#;
const ESCAPED_NEWLINE: &str = "\\n";

pub fn preprocess(input: &str) -> String {
    let mut output = String::new();
    let mut in_block = false;
    let mut current_key = String::new();
    let mut current_line = String::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if !in_block {
            if let Some(pos) = trimmed.find('=') {
                let key = trimmed[..pos].trim();
                let value = trimmed[pos + 1..].trim();

                if value == STR_DELIMITER {
                    in_block = true;
                    current_key = key.to_string();
                    current_line.clear();
                    continue;
                }
            }
        }
        if in_block {
            if trimmed == STR_DELIMITER {
                let escaped = current_line.trim_end().replace('\n', ESCAPED_NEWLINE);
                output.push_str(&format!("{}={}\n", current_key, escaped));
                in_block = false;
            } else {
                current_line.push_str(line);
                current_line.push('\n');
            }
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    output
}

pub fn preprocess_to_ini_from_file(file: &Path) -> Result<Ini, error::Application> {
    let input = fs::read_to_string(file).map_err(|e| error::CouldNotLoadFile(e.to_string()))?;
    preprocess_to_ini(input.as_str())
}

pub fn preprocess_to_ini(input: &str) -> Result<Ini, error::Application> {
    let preprocessed = preprocess(input);
    Ini::load_from_str(&preprocessed).map_err(|e| error::CouldNotLoadFile(e.to_string()))
}

// Allowed in tests
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiline_preprocessing() {
        let raw = r#"
[Room:LivingRoom]
description="""
An old man sits by the fire.
He looks up as you enter.
"""
exits=east:Basement
        "#;

        let expected_output = "\
[Room:LivingRoom]
description=An old man sits by the fire.\\nHe looks up as you enter.
exits=east:Basement
        ";

        let result = preprocess(raw);
        assert_eq!(result.trim(), expected_output.trim());
    }

    #[test]
    fn test_preprocess_to_ini() {
        let raw = r#"
[Dialogue:Intro]
text="""
Hello there.
What brings you here?
"""
responses=ask_name,ask_place
        "#;

        let ini = preprocess_to_ini(raw).unwrap();
        let section = ini.section(Some("Dialogue:Intro")).unwrap();
        let text = section.get("text").unwrap();

        assert_eq!(text, "Hello there.\nWhat brings you here?");
        assert_eq!(section.get("responses").unwrap(), "ask_name,ask_place");
    }
}
