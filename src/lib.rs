//! inih (INI Not Invented Here) - Simple .INI file parser written in Rust
//!
//! This is a Rust port of the popular C library inih, designed to be small, simple,
//! and suitable for embedded systems. It provides both a low-level callback-based
//! API similar to the C version and a high-level reader API for easy access to
//! INI file values.
//!
//! ## Features
//!
//! - Parse INI files with sections, name=value pairs, and comments
//! - Support for multi-line entries (like Python's ConfigParser)
//! - UTF-8 BOM support
//! - Inline and start-of-line comments
//! - Configurable parsing options
//! - Both callback-based and reader-based APIs
//! - Memory efficient (no unnecessary allocations)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use inih::{IniReader, IniParseError};
//!
//! fn main() -> Result<(), IniParseError> {
//!     let reader = IniReader::from_file("config.ini")?;
//!     
//!     let version = reader.get_integer("protocol", "version", -1);
//!     let name = reader.get_string("user", "name", "UNKNOWN");
//!     let email = reader.get_string("user", "email", "UNKNOWN");
//!     
//!     println!("Config: version={}, name={}, email={}", version, name, email);
//!     Ok(())
//! }
//! ```
//!
//! ## Low-level API
//!
//! ```rust,no_run
//! use inih::{ini_parse, IniHandler};
//!
//! struct Config {
//!     version: i32,
//!     name: String,
//! }
//!
//! impl IniHandler for Config {
//!     fn handle(&mut self, section: &str, name: &str, value: &str) -> Result<(), String> {
//!         match (section, name) {
//!             ("protocol", "version") => {
//!                 self.version = value.parse().map_err(|_| "Invalid version".to_string())?;
//!             }
//!             ("user", "name") => {
//!                 self.name = value.to_string();
//!             }
//!             _ => return Err(format!("Unknown key: {}.{}", section, name)),
//!         }
//!         Ok(())
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut config = Config { version: 0, name: String::new() };
//!     ini_parse("config.ini", &mut config)?;
//!     println!("Version: {}, Name: {}", config.version, config.name);
//!     Ok(())
//! }
//! ```

pub mod parser;
pub mod reader;
pub mod error;

pub use parser::{ini_parse, ini_parse_string, ini_parse_string_with_options, ini_parse_file, IniHandler, ParseOptions};
pub use reader::IniReader;
pub use error::IniParseError;

/// Re-export commonly used types
pub type Result<T> = std::result::Result<T, IniParseError>;
