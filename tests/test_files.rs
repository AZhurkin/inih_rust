//! Tests using actual INI files from the original project

use inih::{IniReader, ParseOptions};
use std::fs;

#[test]
fn test_normal_ini() {
    let data = r#"; This is an INI file
[section1]  ; section comment
one=This is a test  ; name=value comment
two = 1234
; x=y

[ section 2 ]
happy  =  4
sad =

[empty]
; do nothing

[comment_test]
test1 = 1;2;3 ; only this will be a comment
test2 = 2;3;4;this won't be a comment, needs whitespace before ';'
test;3 = 345 ; key should be "test;3"
test4 = 4#5#6 ; '#' only starts a comment at start of line
#test5 = 567 ; entire line commented
 # test6 = 678 ; entire line commented, except in MULTILINE mode
test7 = ; blank value, except if inline comments disabled
test8 =; not a comment, needs whitespace before ';'

[colon_tests]
Content-Type: text/html
foo:bar
adams : 42
funny1 : with = equals
funny2 = with : colons
funny3 = two = equals
funny4 : two : colons
"#;

    let reader = IniReader::from_string(data).unwrap();
    
    // Test basic values
    assert_eq!(reader.get_string("section1", "one", ""), "This is a test");
    assert_eq!(reader.get_integer("section1", "two", 0), 1234);
    
    // Test section with spaces
    assert_eq!(reader.get_integer(" section 2 ", "happy", 0), 4);
    assert_eq!(reader.get_string(" section 2 ", "sad", ""), "");
    
    // Test empty section
    assert!(reader.has_section("empty"));
    
    // Test comment handling
    assert_eq!(reader.get_string("comment_test", "test1", ""), "1;2;3");
    assert_eq!(reader.get_string("comment_test", "test2", ""), "2;3;4;this won't be a comment, needs whitespace before ';'");
    assert_eq!(reader.get_string("comment_test", "test;3", ""), "345");
    assert_eq!(reader.get_string("comment_test", "test4", ""), "4#5#6");
    assert_eq!(reader.get_string("comment_test", "test7", ""), "");
    assert_eq!(reader.get_string("comment_test", "test8", ""), "; not a comment, needs whitespace before ';'");
    
    // Test colon separators
    assert_eq!(reader.get_string("colon_tests", "Content-Type", ""), "text/html");
    assert_eq!(reader.get_string("colon_tests", "foo", ""), "bar");
    assert_eq!(reader.get_integer("colon_tests", "adams", 0), 42);
    assert_eq!(reader.get_string("colon_tests", "funny1", ""), "with = equals");
    assert_eq!(reader.get_string("colon_tests", "funny2", ""), "with : colons");
    assert_eq!(reader.get_string("colon_tests", "funny3", ""), "two = equals");
    assert_eq!(reader.get_string("colon_tests", "funny4", ""), "two : colons");
}

#[test]
fn test_multi_line_ini() {
    let data = r#"[section1]
key1 = value1
    continuation line 1
    continuation line 2
key2 = value2
    another continuation
key3 = value3
"#;

    let mut options = ParseOptions::default();
    options.allow_multiline = true;
    let reader = IniReader::from_string_with_options(data, &options).unwrap();
    
    assert_eq!(reader.get_string("section1", "key1", ""), "value1\n    continuation line 1\n    continuation line 2");
    assert_eq!(reader.get_string("section1", "key2", ""), "value2\n    another continuation");
    assert_eq!(reader.get_string("section1", "key3", ""), "value3");
}

#[test]
fn test_duplicate_sections() {
    let data = r#"[section1]
key1 = value1

[section2]
key2 = value2

[section1]
key3 = value3
"#;

    let reader = IniReader::from_string(data).unwrap();
    
    // Both keys should be present in section1
    assert_eq!(reader.get_string("section1", "key1", ""), "value1");
    assert_eq!(reader.get_string("section1", "key3", ""), "value3");
    assert_eq!(reader.get_string("section2", "key2", ""), "value2");
    
    // Keys should include both
    let keys = reader.keys("section1");
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"key1".to_string()));
    assert!(keys.contains(&"key3".to_string()));
}

#[test]
fn test_long_line() {
    let data = r#"[section1]
key1 = this is a very long line that should be handled properly by the parser
key2 = short
"#;

    let reader = IniReader::from_string(data).unwrap();
    
    assert_eq!(reader.get_string("section1", "key1", ""), "this is a very long line that should be handled properly by the parser");
    assert_eq!(reader.get_string("section1", "key2", ""), "short");
}

#[test]
fn test_bom_ini() {
    let data = "\u{FEFF}[section1]\nkey1=value1\n";
    
    let reader = IniReader::from_string(data).unwrap();
    assert_eq!(reader.get_string("section1", "key1", ""), "value1");
}

#[test]
fn test_no_value_ini() {
    let data = r#"[section1]
key1 = value1
key2
key3 = value3
"#;

    // This should fail with stop_on_first_error since allow_no_value is false
    let mut options = ParseOptions::default();
    options.stop_on_first_error = true;
    let result = IniReader::from_string_with_options(data, &options);
    assert!(result.is_err());
}

#[test]
fn test_bad_comment_ini() {
    let data = r#"[section1]
key1 = value1
; this is a valid comment
key2 = value2
"#;

    let reader = IniReader::from_string(data).unwrap();
    assert_eq!(reader.get_string("section1", "key1", ""), "value1");
    assert_eq!(reader.get_string("section1", "key2", ""), "value2");
}

#[test]
fn test_bad_section_ini() {
    let data = r#"[section1]
key1 = value1
[unclosed section
key2 = value2
"#;

    let result = IniReader::from_string(data);
    assert!(result.is_err());
}

#[test]
fn test_bad_multi_ini() {
    let data = r#"[section1]
key1 = value1
    continuation without proper indentation
key2 = value2
"#;

    let reader = IniReader::from_string(data).unwrap();
    // The continuation should not be treated as a continuation
    assert_eq!(reader.get_string("section1", "key1", ""), "value1");
    assert_eq!(reader.get_string("section1", "key2", ""), "value2");
}
