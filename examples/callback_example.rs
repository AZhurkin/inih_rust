//! Callback-based example using the low-level parser API
//! 
//! This example demonstrates the low-level IniHandler trait for custom parsing logic.

use inih::{ini_parse_string, IniHandler, IniParseError};

#[derive(Debug, Default)]
struct Config {
    version: i32,
    name: String,
    email: String,
    active: bool,
    pi: f64,
    trillion: i64,
}

impl IniHandler for Config {
    fn handle(&mut self, section: &str, name: &str, value: &str) -> Result<(), String> {
        match (section, name) {
            ("protocol", "version") => {
                self.version = value.parse()
                    .map_err(|_| format!("Invalid version '{}'", value))?;
            }
            ("user", "name") => {
                self.name = value.to_string();
            }
            ("user", "email") => {
                self.email = value.to_string();
            }
            ("user", "active") => {
                self.active = match value.to_lowercase().as_str() {
                    "true" | "yes" | "on" | "1" => true,
                    "false" | "no" | "off" | "0" => false,
                    _ => return Err(format!("Invalid boolean value '{}'", value)),
                };
            }
            ("user", "pi") => {
                self.pi = value.parse()
                    .map_err(|_| format!("Invalid float '{}'", value))?;
            }
            ("user", "trillion") => {
                self.trillion = value.parse()
                    .map_err(|_| format!("Invalid integer '{}'", value))?;
            }
            _ => {
                return Err(format!("Unknown key '{}.{}'", section, name));
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a sample INI file content
    let ini_content = r#"
; Test config file for inih Rust library

[protocol]               ; Protocol configuration
version=6                ; IPv6

[user]
name = Bob Smith         ; Spaces around '=' are stripped
email = bob@smith.com    ; And comments (like this) ignored
active = true            ; Test a boolean
pi = 3.14159             ; Test a floating point number
trillion = 1000000000000 ; Test 64-bit integers
"#;

    // Parse using callback-based API
    let mut config = Config::default();
    ini_parse_string(ini_content, &mut config)?;
    
    // Print the results
    println!("Config loaded using callback API:");
    println!("  Protocol version: {}", config.version);
    println!("  User name: {}", config.name);
    println!("  User email: {}", config.email);
    println!("  User active: {}", config.active);
    println!("  Pi value: {}", config.pi);
    println!("  Trillion: {}", config.trillion);
    
    Ok(())
}
