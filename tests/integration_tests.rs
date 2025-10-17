//! Integration tests for inih library

use inih::{IniReader, ini_parse_string, IniHandler, ParseOptions, IniParseError};

#[derive(Debug, Default)]
struct TestHandler {
    values: Vec<(String, String, String)>,
}

impl IniHandler for TestHandler {
    fn handle(&mut self, section: &str, name: &str, value: &str) -> Result<(), String> {
        self.values.push((section.to_string(), name.to_string(), value.to_string()));
        Ok(())
    }
}

#[test]
fn test_basic_parsing() {
    let data = r#"
[section1]
key1=value1
key2 = value2
key3: value3

[section2]
key4=value4
"#;

    let reader = IniReader::from_string(data).unwrap();
    
    assert_eq!(reader.get_string("section1", "key1", ""), "value1");
    assert_eq!(reader.get_string("section1", "key2", ""), "value2");
    assert_eq!(reader.get_string("section1", "key3", ""), "value3");
    assert_eq!(reader.get_string("section2", "key4", ""), "value4");
}

#[test]
fn test_comments() {
    let data = r#"
; This is a comment
[section1]  ; section comment
key1=value1  ; inline comment
# Another comment
key2=value2
"#;

    let reader = IniReader::from_string(data).unwrap();
    
    assert_eq!(reader.get_string("section1", "key1", ""), "value1");
    assert_eq!(reader.get_string("section1", "key2", ""), "value2");
}

#[test]
fn test_multiline() {
    let data = r#"
[section1]
key1=line1
    line2
    line3
key2=value2
"#;

    let mut options = ParseOptions::default();
    options.allow_multiline = true;
    let reader = IniReader::from_string_with_options(data, &options).unwrap();
    
    assert_eq!(reader.get_string("section1", "key1", ""), "line1\n    line2\n    line3");
    assert_eq!(reader.get_string("section1", "key2", ""), "value2");
}

#[test]
fn test_empty_sections() {
    let data = r#"
[section1]
key1=value1

[empty_section]

[section2]
key2=value2
"#;

    let reader = IniReader::from_string(data).unwrap();
    
    assert_eq!(reader.get_string("section1", "key1", ""), "value1");
    assert_eq!(reader.get_string("section2", "key2", ""), "value2");
    assert!(reader.has_section("empty_section"));
}

#[test]
fn test_no_section() {
    let data = r#"
key1=value1
key2=value2

[section1]
key3=value3
"#;

    let reader = IniReader::from_string(data).unwrap();
    
    assert_eq!(reader.get_string("", "key1", ""), "value1");
    assert_eq!(reader.get_string("", "key2", ""), "value2");
    assert_eq!(reader.get_string("section1", "key3", ""), "value3");
}

#[test]
fn test_types() {
    let data = r#"
[types]
integer = 42
negative = -123
float = 3.14159
boolean_true = true
boolean_false = false
boolean_yes = yes
boolean_no = no
boolean_on = on
boolean_off = off
boolean_1 = 1
boolean_0 = 0
hex = 0x1A
"#;

    let reader = IniReader::from_string(data).unwrap();
    
    assert_eq!(reader.get_integer("types", "integer", 0), 42);
    assert_eq!(reader.get_integer("types", "negative", 0), -123);
    assert_eq!(reader.get_real("types", "float", 0.0), 3.14159);
    assert_eq!(reader.get_boolean("types", "boolean_true", false), true);
    assert_eq!(reader.get_boolean("types", "boolean_false", true), false);
    assert_eq!(reader.get_boolean("types", "boolean_yes", false), true);
    assert_eq!(reader.get_boolean("types", "boolean_no", true), false);
    assert_eq!(reader.get_boolean("types", "boolean_on", false), true);
    assert_eq!(reader.get_boolean("types", "boolean_off", true), false);
    assert_eq!(reader.get_boolean("types", "boolean_1", false), true);
    assert_eq!(reader.get_boolean("types", "boolean_0", true), false);
    assert_eq!(reader.get_integer("types", "hex", 0), 0x1A);
}

#[test]
fn test_sections_and_keys() {
    let data = r#"
[section1]
key1=value1
key2=value2

[section2]
key3=value3

[section1]
key4=value4
"#;

    let reader = IniReader::from_string(data).unwrap();
    
    let sections = reader.sections();
    assert_eq!(sections.len(), 2);
    assert!(sections.contains(&"section1".to_string()));
    assert!(sections.contains(&"section2".to_string()));
    
    let keys1 = reader.keys("section1");
    assert_eq!(keys1.len(), 3);
    assert!(keys1.contains(&"key1".to_string()));
    assert!(keys1.contains(&"key2".to_string()));
    assert!(keys1.contains(&"key4".to_string()));
    
    let keys2 = reader.keys("section2");
    assert_eq!(keys2.len(), 1);
    assert!(keys2.contains(&"key3".to_string()));
    
    assert!(reader.has_section("section1"));
    assert!(reader.has_section("section2"));
    assert!(!reader.has_section("section3"));
    
    assert!(reader.has_value("section1", "key1"));
    assert!(reader.has_value("section2", "key3"));
    assert!(!reader.has_value("section1", "key3"));
}

#[test]
fn test_callback_api() {
    let data = r#"
[section1]
key1=value1
key2=value2

[section2]
key3=value3
"#;

    let mut handler = TestHandler::default();
    ini_parse_string(data, &mut handler).unwrap();
    
    // Filter out section-only calls (empty name and value)
    let key_value_calls: Vec<_> = handler.values.iter()
        .filter(|(_, name, value)| !name.is_empty() || !value.is_empty())
        .collect();
    
    assert_eq!(key_value_calls.len(), 3);
    assert_eq!(key_value_calls[0], &("section1".to_string(), "key1".to_string(), "value1".to_string()));
    assert_eq!(key_value_calls[1], &("section1".to_string(), "key2".to_string(), "value2".to_string()));
    assert_eq!(key_value_calls[2], &("section2".to_string(), "key3".to_string(), "value3".to_string()));
}

#[test]
fn test_parse_error() {
    let data = r#"
[section1]
key1=value1
[unclosed_section
key2=value2
"#;

    let result = IniReader::from_string(data);
    assert!(result.is_err());
    
    if let Err(IniParseError::ParseError { line, .. }) = result {
        assert_eq!(line, 4); // Line with unclosed section
    } else {
        panic!("Expected ParseError");
    }
}

#[test]
fn test_custom_options() {
    let data = r#"
[section1]
key1=value1
    continuation
key2=value2
"#;

    let mut options = ParseOptions::default();
    options.allow_multiline = true;
    
    let reader = IniReader::from_string_with_options(data, &options).unwrap();
    assert_eq!(reader.get_string("section1", "key1", ""), "value1\n    continuation");
}

#[test]
fn test_utf8_bom() {
    let data = "\u{FEFF}[section1]\nkey1=value1\n";
    
    let reader = IniReader::from_string(data).unwrap();
    assert_eq!(reader.get_string("section1", "key1", ""), "value1");
}

#[test]
fn test_case_insensitive() {
    let data = r#"
[Section1]
Key1=value1
KEY2=value2
"#;

    let reader = IniReader::from_string(data).unwrap();
    
    // Should be case-insensitive
    assert_eq!(reader.get_string("section1", "key1", ""), "value1");
    assert_eq!(reader.get_string("Section1", "Key1", ""), "value1");
    assert_eq!(reader.get_string("SECTION1", "KEY1", ""), "value1");
    assert_eq!(reader.get_string("section1", "key2", ""), "value2");
}
