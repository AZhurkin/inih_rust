//! Example of reading INI data from a file
//! 
//! This example demonstrates reading INI data from an actual file.

use inih::IniReader;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a sample INI file
    let ini_content = r#"
; Sample configuration file

[database]
host = localhost
port = 5432
name = myapp
user = admin
password = secret123

[server]
host = 0.0.0.0
port = 8080
debug = true
max_connections = 100

[cache]
enabled = true
ttl = 3600
max_size = 1024
"#;
    
    // Write the content to a temporary file
    let temp_file = "temp_config.ini";
    std::fs::write(temp_file, ini_content)?;
    
    // Read and parse the INI file
    let reader = IniReader::from_file(temp_file)?;
    
    // Check for parse errors
    if let Some(error) = reader.parse_error() {
        eprintln!("Parse error: {}", error);
        return Err(error.clone().into());
    }
    
    // Read database configuration
    println!("Database Configuration:");
    println!("  Host: {}", reader.get_string("database", "host", "localhost"));
    println!("  Port: {}", reader.get_integer("database", "port", 5432));
    println!("  Name: {}", reader.get_string("database", "name", ""));
    println!("  User: {}", reader.get_string("database", "user", ""));
    println!("  Password: {}", reader.get_string("database", "password", ""));
    
    // Read server configuration
    println!("\nServer Configuration:");
    println!("  Host: {}", reader.get_string("server", "host", "127.0.0.1"));
    println!("  Port: {}", reader.get_integer("server", "port", 3000));
    println!("  Debug: {}", reader.get_boolean("server", "debug", false));
    println!("  Max Connections: {}", reader.get_integer("server", "max_connections", 10));
    
    // Read cache configuration
    println!("\nCache Configuration:");
    println!("  Enabled: {}", reader.get_boolean("cache", "enabled", false));
    println!("  TTL: {}", reader.get_integer("cache", "ttl", 300));
    println!("  Max Size: {}", reader.get_integer("cache", "max_size", 100));
    
    // List all sections
    println!("\nAll sections:");
    for section in reader.sections() {
        println!("  - {}", section);
    }
    
    // Clean up the temporary file
    std::fs::remove_file(temp_file)?;
    
    Ok(())
}
