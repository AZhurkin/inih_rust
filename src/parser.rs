//! Low-level INI parser with callback-based API

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use crate::error::IniParseError;

/// Trait for handling INI parsing events
pub trait IniHandler {
    /// Called for each name=value pair found in the INI file
    /// 
    /// # Arguments
    /// * `section` - The section name (empty string if no section)
    /// * `name` - The key name
    /// * `value` - The value (empty string if no value)
    /// 
    /// # Returns
    /// * `Ok(())` - Continue parsing
    /// * `Err(String)` - Stop parsing with error message
    fn handle(&mut self, section: &str, name: &str, value: &str) -> Result<(), String>;
}

/// Configuration options for INI parsing
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// Allow multi-line entries (like Python's ConfigParser)
    pub allow_multiline: bool,
    /// Allow UTF-8 BOM at start of file
    pub allow_bom: bool,
    /// Allow inline comments (with whitespace before comment char)
    pub allow_inline_comments: bool,
    /// Characters that start inline comments
    pub inline_comment_prefixes: String,
    /// Characters that start line comments
    pub start_comment_prefixes: String,
    /// Stop parsing on first error
    pub stop_on_first_error: bool,
    /// Call handler on new section (with name and value as empty)
    pub call_handler_on_new_section: bool,
    /// Allow names without values
    pub allow_no_value: bool,
    /// Maximum line length
    pub max_line: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            allow_multiline: false,
            allow_bom: true,
            allow_inline_comments: true,
            inline_comment_prefixes: ";".to_string(),
            start_comment_prefixes: ";#".to_string(),
            stop_on_first_error: false,
            call_handler_on_new_section: false,
            allow_no_value: false,
            max_line: 200,
        }
    }
}

/// Parse an INI file from a file path
pub fn ini_parse<P: AsRef<Path>>(path: P, handler: &mut dyn IniHandler) -> Result<(), IniParseError> {
    ini_parse_with_options(path, handler, &ParseOptions::default())
}

/// Parse an INI file from a file path with custom options
pub fn ini_parse_with_options<P: AsRef<Path>>(
    path: P,
    handler: &mut dyn IniHandler,
    options: &ParseOptions,
) -> Result<(), IniParseError> {
    let file = File::open(path.as_ref())
        .map_err(|e| IniParseError::FileOpen(format!("{}: {}", path.as_ref().display(), e)))?;
    ini_parse_file_with_options(file, handler, options)
}

/// Parse an INI file from a File object
pub fn ini_parse_file<R: Read>(file: R, handler: &mut dyn IniHandler) -> Result<(), IniParseError> {
    ini_parse_file_with_options(file, handler, &ParseOptions::default())
}

/// Parse an INI file from a File object with custom options
pub fn ini_parse_file_with_options<R: Read>(
    file: R,
    handler: &mut dyn IniHandler,
    options: &ParseOptions,
) -> Result<(), IniParseError> {
    let reader = BufReader::new(file);
    ini_parse_reader_with_options(reader, handler, options)
}

/// Parse INI data from a string
pub fn ini_parse_string(data: &str, handler: &mut dyn IniHandler) -> Result<(), IniParseError> {
    ini_parse_string_with_options(data, handler, &ParseOptions::default())
}

/// Parse INI data from a string with custom options
pub fn ini_parse_string_with_options(
    data: &str,
    handler: &mut dyn IniHandler,
    options: &ParseOptions,
) -> Result<(), IniParseError> {
    let lines = data.lines().map(|s| s.to_string()).collect::<Vec<_>>();
    ini_parse_lines_with_options(&lines, handler, options)
}

/// Parse INI data from a BufRead object
pub fn ini_parse_reader_with_options<R: BufRead>(
    mut reader: R,
    handler: &mut dyn IniHandler,
    options: &ParseOptions,
) -> Result<(), IniParseError> {
    let mut lines = Vec::new();
    let mut line = String::new();
    
    while reader.read_line(&mut line).map_err(|e| IniParseError::FileOpen(e.to_string()))? > 0 {
        lines.push(line.trim_end().to_string());
        line.clear();
    }
    
    ini_parse_lines_with_options(&lines, handler, options)
}

/// Parse INI data from a vector of lines
fn ini_parse_lines_with_options(
    lines: &[String],
    handler: &mut dyn IniHandler,
    options: &ParseOptions,
) -> Result<(), IniParseError> {
    let mut section = String::new();
    let mut prev_name = String::new();
    let mut line_number = 0;
    let mut first_error: Option<IniParseError> = None;

    for line in lines {
        line_number += 1;
        
        if line.len() > options.max_line {
            let error = IniParseError::ParseError {
                line: line_number,
                message: "Line too long".to_string(),
            };
            if options.stop_on_first_error {
                return Err(error);
            }
            if first_error.is_none() {
                first_error = Some(error);
            }
            continue;
        }

        let result = parse_line(line, &mut section, &mut prev_name, handler, options, line_number);
        
        match result {
            Ok(()) => {}
            Err(error) => {
                if options.stop_on_first_error {
                    return Err(error);
                }
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }

    if let Some(error) = first_error {
        Err(error)
    } else {
        Ok(())
    }
}

/// Parse a single line of INI data
fn parse_line(
    line: &str,
    section: &mut String,
    prev_name: &mut String,
    handler: &mut dyn IniHandler,
    options: &ParseOptions,
    line_number: usize,
) -> Result<(), IniParseError> {
    let mut line = line.to_string();
    
    // Handle UTF-8 BOM
    if line_number == 1 && options.allow_bom && line.starts_with('\u{FEFF}') {
        line = line[3..].to_string();
    }
    
    // Trim whitespace
    let trimmed = line.trim();
    
    // Skip empty lines
    if trimmed.is_empty() {
        return Ok(());
    }
    
    // Check for start-of-line comments
    if options.start_comment_prefixes.chars().any(|c| trimmed.starts_with(c)) {
        return Ok(());
    }
    
    // Handle multi-line continuation
    if options.allow_multiline && !prev_name.is_empty() && !trimmed.is_empty() && line.starts_with(char::is_whitespace) {
        let value = if options.allow_inline_comments {
            // For inline comments, we need to process the trimmed version but preserve indentation
            let comment_removed = remove_inline_comment(trimmed, &options.inline_comment_prefixes);
            // Reconstruct with original indentation
            let indent_len = line.len() - line.trim_start().len();
            format!("{}{}", &line[..indent_len], comment_removed)
        } else {
            line.to_string() // Use original line to preserve indentation
        };
        
        return handler.handle(section, prev_name, &value)
            .map_err(|msg| IniParseError::HandlerError(msg));
    }
    
    // Handle section headers
    if trimmed.starts_with('[') {
        if let Some(end_pos) = find_char_or_comment(trimmed, ']', &options.inline_comment_prefixes, options.allow_inline_comments) {
            if end_pos > 1 {
                let section_name = trimmed[1..end_pos].to_string();
                *section = section_name;
                *prev_name = String::new();
                
                // Always call handler for new sections to register them
                return handler.handle(section, "", "")
                    .map_err(|msg| IniParseError::HandlerError(msg));
            }
        }
        return Err(IniParseError::ParseError {
            line: line_number,
            message: "Missing ']' in section header".to_string(),
        });
    }
    
    // Handle name=value and name:value pairs
    // Find the first separator (= or :)
    let eq_pos = find_char_or_comment(trimmed, '=', &options.inline_comment_prefixes, options.allow_inline_comments);
    let colon_pos = find_char_or_comment(trimmed, ':', &options.inline_comment_prefixes, options.allow_inline_comments);
    
    let sep_pos = match (eq_pos, colon_pos) {
        (Some(e), Some(c)) => Some(e.min(c)),
        (Some(e), None) => Some(e),
        (None, Some(c)) => Some(c),
        (None, None) => None,
    };
    
    if let Some(sep_pos) = sep_pos {
        let name = trimmed[..sep_pos].trim().to_string();
        let value = if sep_pos + 1 < trimmed.len() {
            let value_part = &trimmed[sep_pos + 1..];
            if options.allow_inline_comments {
                remove_inline_comment(value_part, &options.inline_comment_prefixes)
            } else {
                value_part.trim().to_string()
            }
        } else {
            String::new()
        };
        
        *prev_name = name.clone();
        
        return handler.handle(section, &name, &value)
            .map_err(|msg| IniParseError::HandlerError(msg));
    }
    
    // Handle names without values
    if options.allow_no_value && !trimmed.is_empty() {
        let name = if options.allow_inline_comments {
            remove_inline_comment(trimmed, &options.inline_comment_prefixes)
        } else {
            trimmed.to_string()
        };
        
        *prev_name = name.clone();
        
        return handler.handle(section, &name, "")
            .map_err(|msg| IniParseError::HandlerError(msg));
    }
    
    // If we get here and the line is not empty, it's an invalid line
    // But if it's empty or just whitespace, we should ignore it
    if !trimmed.is_empty() {
        if options.stop_on_first_error {
            Err(IniParseError::ParseError {
                line: line_number,
                message: "Invalid line format".to_string(),
            })
        } else {
            // For invalid lines, we just ignore them instead of erroring
            Ok(())
        }
    } else {
        Ok(())
    }
}

/// Find a character or comment in a string
fn find_char_or_comment(
    s: &str,
    target: char,
    comment_prefixes: &str,
    allow_inline_comments: bool,
) -> Option<usize> {
    let mut was_space = false;
    
    for (i, ch) in s.char_indices() {
        if ch == target {
            return Some(i);
        }
        
        if allow_inline_comments && was_space && comment_prefixes.contains(ch) {
            return Some(i);
        }
        
        was_space = ch.is_whitespace();
    }
    
    None
}

/// Remove inline comment from a string
fn remove_inline_comment(s: &str, comment_prefixes: &str) -> String {
    let mut was_space = false;
    
    for (i, ch) in s.char_indices() {
        if was_space && comment_prefixes.contains(ch) {
            return s[..i].trim().to_string();
        }
        was_space = ch.is_whitespace();
    }
    
    s.trim().to_string()
}
