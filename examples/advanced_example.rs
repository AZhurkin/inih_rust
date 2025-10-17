//! Advanced example showing custom parsing options and error handling
//! 
//! This example demonstrates using custom parsing options and handling various error cases.

use inih::{ini_parse_string_with_options, IniHandler, ParseOptions, IniParseError};

#[derive(Debug, Default)]
struct AdvancedConfig {
    values: Vec<(String, String, String)>, // (section, name, value)
    errors: Vec<String>,
}

impl IniHandler for AdvancedConfig {
    fn handle(&mut self, section: &str, name: &str, value: &str) -> Result<(), String> {
        // Store all values for analysis
        self.values.push((section.to_string(), name.to_string(), value.to_string()));
        
        // Example: reject certain values
        if name == "forbidden" {
            return Err(format!("Forbidden key '{}' found in section '{}'", name, section));
        }
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test INI content with various features
    let ini_content = r#"
; Test file with various INI features

# This is a comment with # symbol
[section1]
key1 = value1
key2 = value2 with spaces
key3=value3_without_spaces
key4: value4_with_colon
key5 = value5 ; inline comment
key6 = value6 # inline comment with #

[section2]
multiline_key = line1
    line2
    line3
forbidden = this should cause an error
empty_key = 
no_value_key

[section3]
# Another comment
key7 = final value
"#;

    // Test with default options
    println!("=== Testing with default options ===");
    let mut config = AdvancedConfig::default();
    match ini_parse_string_with_options(ini_content, &mut config, &ParseOptions::default()) {
        Ok(()) => {
            println!("Parsing successful!");
            for (section, name, value) in &config.values {
                println!("  {}: {} = '{}'", section, name, value);
            }
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }

    // Test with custom options
    println!("\n=== Testing with custom options ===");
    let mut custom_options = ParseOptions::default();
    custom_options.allow_multiline = true;
    custom_options.allow_inline_comments = true;
    custom_options.inline_comment_prefixes = ";#".to_string();
    custom_options.start_comment_prefixes = ";#".to_string();
    custom_options.allow_no_value = true;
    custom_options.stop_on_first_error = false;

    let mut config2 = AdvancedConfig::default();
    match ini_parse_string_with_options(ini_content, &mut config2, &custom_options) {
        Ok(()) => {
            println!("Parsing successful!");
            for (section, name, value) in &config2.values {
                println!("  {}: {} = '{}'", section, name, value);
            }
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }

    // Test error handling
    println!("\n=== Testing error handling ===");
    let error_content = r#"
[section1]
key1 = value1
forbidden = this will cause an error
key2 = value2
"#;

    let mut config3 = AdvancedConfig::default();
    match ini_parse_string_with_options(error_content, &mut config3, &ParseOptions::default()) {
        Ok(()) => {
            println!("Unexpected success!");
        }
        Err(e) => {
            println!("Expected error: {}", e);
        }
    }

    // Test with stop_on_first_error = true
    println!("\n=== Testing with stop_on_first_error = true ===");
    let mut strict_options = ParseOptions::default();
    strict_options.stop_on_first_error = true;

    let mut config4 = AdvancedConfig::default();
    match ini_parse_string_with_options(error_content, &mut config4, &strict_options) {
        Ok(()) => {
            println!("Unexpected success!");
        }
        Err(e) => {
            println!("Error (stopped on first): {}", e);
        }
    }

    Ok(())
}
