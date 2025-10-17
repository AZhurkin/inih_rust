//! Error types for INI parsing

use std::fmt;

/// Errors that can occur during INI parsing
#[derive(Debug, Clone, PartialEq)]
pub enum IniParseError {
    /// File could not be opened
    FileOpen(String),
    /// Parse error at specific line number
    ParseError { line: usize, message: String },
    /// Memory allocation error
    MemoryError,
    /// Custom error from handler
    HandlerError(String),
}

impl fmt::Display for IniParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IniParseError::FileOpen(path) => write!(f, "Unable to open file: {}", path),
            IniParseError::ParseError { line, message } => {
                write!(f, "Parse error on line {}: {}", line, message)
            }
            IniParseError::MemoryError => write!(f, "Memory allocation error"),
            IniParseError::HandlerError(msg) => write!(f, "Handler error: {}", msg),
        }
    }
}

impl std::error::Error for IniParseError {}

impl From<std::io::Error> for IniParseError {
    fn from(err: std::io::Error) -> Self {
        IniParseError::FileOpen(err.to_string())
    }
}
