//! High-level INI reader with easy-to-use API

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::IniParseError;
use crate::parser::{ini_parse_file_with_options, IniHandler, ParseOptions};

/// High-level INI reader that stores all values in memory for easy access
pub struct IniReader {
    values: HashMap<String, String>,
    sections: std::collections::HashSet<String>,
    error: Option<IniParseError>,
}

impl IniReader {
    /// Create a new INI reader from a file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, IniParseError> {
        let file = File::open(path)?;
        Self::from_reader(file)
    }

    /// Create a new INI reader from a Read object
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, IniParseError> {
        let mut ini_reader = Self {
            values: HashMap::new(),
            sections: std::collections::HashSet::new(),
            error: None,
        };
        
        let options = ParseOptions::default();
        match ini_parse_file_with_options(reader, &mut ini_reader, &options) {
            Ok(()) => Ok(ini_reader),
            Err(e) => {
                ini_reader.error = Some(e.clone());
                Err(e)
            }
        }
    }

    /// Create a new INI reader from a string
    pub fn from_string(data: &str) -> Result<Self, IniParseError> {
        Self::from_string_with_options(data, &ParseOptions::default())
    }
    
    /// Create a new INI reader from a string with custom options
    pub fn from_string_with_options(data: &str, options: &ParseOptions) -> Result<Self, IniParseError> {
        let mut ini_reader = Self {
            values: HashMap::new(),
            sections: std::collections::HashSet::new(),
            error: None,
        };
        
        match crate::parser::ini_parse_string_with_options(data, &mut ini_reader, options) {
            Ok(()) => Ok(ini_reader),
            Err(e) => {
                ini_reader.error = Some(e.clone());
                Err(e)
            }
        }
    }

    /// Get the parse error if one occurred
    pub fn parse_error(&self) -> Option<&IniParseError> {
        self.error.as_ref()
    }

    /// Get a string value, returning the default if not found
    pub fn get(&self, section: &str, name: &str, default_value: &str) -> String {
        let key = Self::make_key(section, name);
        self.values.get(&key).cloned().unwrap_or_else(|| default_value.to_string())
    }

    /// Get a string value, returning the default if not found or empty
    pub fn get_string(&self, section: &str, name: &str, default_value: &str) -> String {
        let value = self.get(section, name, "");
        if value.is_empty() { default_value.to_string() } else { value }
    }

    /// Get an integer value, returning the default if not found or invalid
    pub fn get_integer(&self, section: &str, name: &str, default_value: i64) -> i64 {
        let value = self.get(section, name, "");
        
        // Handle hexadecimal numbers
        if value.starts_with("0x") || value.starts_with("0X") {
            if let Ok(hex_value) = i64::from_str_radix(&value[2..], 16) {
                return hex_value;
            }
        }
        
        value.parse().unwrap_or(default_value)
    }

    /// Get a 64-bit integer value, returning the default if not found or invalid
    pub fn get_integer64(&self, section: &str, name: &str, default_value: i64) -> i64 {
        self.get_integer(section, name, default_value)
    }

    /// Get an unsigned integer value, returning the default if not found or invalid
    pub fn get_unsigned(&self, section: &str, name: &str, default_value: u64) -> u64 {
        let value = self.get(section, name, "");
        value.parse().unwrap_or(default_value)
    }

    /// Get a 64-bit unsigned integer value, returning the default if not found or invalid
    pub fn get_unsigned64(&self, section: &str, name: &str, default_value: u64) -> u64 {
        self.get_unsigned(section, name, default_value)
    }

    /// Get a floating-point value, returning the default if not found or invalid
    pub fn get_real(&self, section: &str, name: &str, default_value: f64) -> f64 {
        let value = self.get(section, name, "");
        value.parse().unwrap_or(default_value)
    }

    /// Get a boolean value, returning the default if not found or invalid
    /// Valid true values: "true", "yes", "on", "1"
    /// Valid false values: "false", "no", "off", "0"
    pub fn get_boolean(&self, section: &str, name: &str, default_value: bool) -> bool {
        let value = self.get(section, name, "").to_lowercase();
        match value.as_str() {
            "true" | "yes" | "on" | "1" => true,
            "false" | "no" | "off" | "0" => false,
            _ => default_value,
        }
    }

    /// Get all section names
    pub fn sections(&self) -> Vec<String> {
        let mut sections: Vec<String> = self.sections.iter().cloned().collect();
        sections.sort();
        sections
    }

    /// Get all keys in a section
    pub fn keys(&self, section: &str) -> Vec<String> {
        let prefix = format!("{}=", section.to_lowercase());
        let mut keys = Vec::new();
        
        for key in self.values.keys() {
            if key.starts_with(&prefix) {
                keys.push(key[prefix.len()..].to_string());
            }
        }
        keys.sort();
        keys
    }

    /// Check if a section exists
    pub fn has_section(&self, section: &str) -> bool {
        self.sections.contains(section)
    }

    /// Check if a value exists
    pub fn has_value(&self, section: &str, name: &str) -> bool {
        let key = Self::make_key(section, name);
        self.values.contains_key(&key)
    }

    /// Create a key from section and name (case-insensitive)
    fn make_key(section: &str, name: &str) -> String {
        format!("{}={}", section.to_lowercase(), name.to_lowercase())
    }
}

impl IniHandler for IniReader {
    fn handle(&mut self, section: &str, name: &str, value: &str) -> Result<(), String> {
        // Register section
        if !section.is_empty() {
            self.sections.insert(section.to_string());
        }
        
        if name.is_empty() {
            // This happens when INI_CALL_HANDLER_ON_NEW_SECTION is enabled
            return Ok(());
        }
        
        let key = Self::make_key(section, name);
        
        // Handle multi-line values by concatenating with newlines
        if let Some(existing_value) = self.values.get_mut(&key) {
            existing_value.push('\n');
            existing_value.push_str(value);
        } else {
            self.values.insert(key, value.to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let data = r#"
[protocol]
version=6

[user]
name = Bob Smith
email = bob@smith.com
active = true
pi = 3.14159
"#;
        
        let reader = IniReader::from_string(data).unwrap();
        
        assert_eq!(reader.get_integer("protocol", "version", -1), 6);
        assert_eq!(reader.get_string("user", "name", "UNKNOWN"), "Bob Smith");
        assert_eq!(reader.get_string("user", "email", "UNKNOWN"), "bob@smith.com");
        assert_eq!(reader.get_boolean("user", "active", false), true);
        assert_eq!(reader.get_real("user", "pi", 0.0), 3.14159);
    }

    #[test]
    fn test_sections_and_keys() {
        let data = r#"
[section1]
key1=value1
key2=value2

[section2]
key3=value3
"#;
        
        let reader = IniReader::from_string(data).unwrap();
        
        let sections = reader.sections();
        assert_eq!(sections.len(), 2);
        assert!(sections.contains(&"section1".to_string()));
        assert!(sections.contains(&"section2".to_string()));
        
        let keys1 = reader.keys("section1");
        assert_eq!(keys1.len(), 2);
        assert!(keys1.contains(&"key1".to_string()));
        assert!(keys1.contains(&"key2".to_string()));
        
        assert!(reader.has_section("section1"));
        assert!(reader.has_value("section1", "key1"));
        assert!(!reader.has_value("section1", "key3"));
    }
}

impl fmt::Debug for IniReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IniReader")
            .field("values", &self.values)
            .field("sections", &self.sections)
            .field("error", &self.error)
            .finish()
    }
}
