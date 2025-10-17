//! Basic example of using inih library
//! 
//! This example demonstrates the high-level IniReader API for easy access to INI values.

use inih::IniReader;

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

    // Parse the INI content
    let reader = IniReader::from_string(ini_content)?;
    
    // Check for parse errors
    if let Some(error) = reader.parse_error() {
        eprintln!("Parse error: {}", error);
        return Err(error.clone().into());
    }
    
    // Read values with defaults
    let version = reader.get_integer("protocol", "version", -1);
    let name = reader.get_string("user", "name", "UNKNOWN");
    let email = reader.get_string("user", "email", "UNKNOWN");
    let active = reader.get_boolean("user", "active", false);
    let pi = reader.get_real("user", "pi", 0.0);
    let trillion = reader.get_integer64("user", "trillion", 0);
    
    // Print the results
    println!("Config loaded:");
    println!("  Protocol version: {}", version);
    println!("  User name: {}", name);
    println!("  User email: {}", email);
    println!("  User active: {}", active);
    println!("  Pi value: {}", pi);
    println!("  Trillion: {}", trillion);
    
    // List all sections
    println!("\nSections found:");
    for section in reader.sections() {
        println!("  - {}", section);
    }
    
    // List keys in user section
    println!("\nKeys in 'user' section:");
    for key in reader.keys("user") {
        println!("  - {}", key);
    }
    
    Ok(())
}
