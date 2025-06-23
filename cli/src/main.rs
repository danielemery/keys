use clap::Parser;
use colored::Colorize;
use reqwest::header::ACCEPT;
use anyhow::{Context, Result};
use serde::Deserialize;

/// Helper function to pad a string to a specific width
fn pad_string(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}

#[derive(Debug, Deserialize)]
struct KeysResponse {
    version: String,
    keys: Vec<SSHKey>,
}

#[derive(Debug, Deserialize)]
struct SSHKey {
    key: String,
    user: String,
    name: String,
    tags: Vec<String>,
}

/// Function to pretty print the keys with formatted columns and colors
fn pretty_print_keys(keys_response: &KeysResponse) {
    // Print the version information
    println!("{} {}", "Keys Server Version:".purple().bold(), keys_response.version);
    println!();
    
    if keys_response.keys.is_empty() {
        println!("{}", "No keys found matching the criteria.".yellow().italic());
        return;
    }
    
    // Find the maximum width for each column for better formatting
    let max_name_len = keys_response.keys.iter().map(|k| k.name.len()).max().unwrap_or(4).max(4);
    let max_user_len = keys_response.keys.iter().map(|k| k.user.len()).max().unwrap_or(4).max(4);
    let max_tags_len = keys_response.keys.iter()
        .map(|k| k.tags.join(", ").len())
        .max().unwrap_or(4).max(4);
    
    let column_spacing = 3;
    
    // Print header
    println!("{:width1$}{:width2$}{:width3$}{}",
        "NAME".green().bold(),
        "USER".blue().bold(),
        "TAGS".yellow().bold(),
        "KEY".red().bold(),
        width1 = max_name_len + column_spacing,
        width2 = max_user_len + column_spacing,
        width3 = max_tags_len + column_spacing
    );
    
    // Print divider
    println!("{}", "-".repeat(max_name_len + max_user_len + max_tags_len + 60));
    
    // Print each key in columns with different colors
    for key in &keys_response.keys {
        let tags_str = key.tags.join(", ");
        
        // First apply padding to each field (without color)
        let name_padded = pad_string(&key.name, max_name_len);
        let user_padded = pad_string(&key.user, max_user_len);
        let tags_padded = pad_string(&tags_str, max_tags_len);
        
        // Then apply color to the padded strings
        println!("{:width1$}{:width2$}{:width3$}{}",
            name_padded.green(),
            user_padded.blue(),
            tags_padded.yellow(),
            key.key.red(),
            width1 = max_name_len + column_spacing,
            width2 = max_user_len + column_spacing,
            width3 = max_tags_len + column_spacing
        );
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The server URL
    #[arg(short, long, default_value = "http://localhost:8000")]
    server: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let url = format!("{}/keys", args.server);
    
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header(ACCEPT, "application/json")
        .send()
        .context("Failed to send request to keys server")?;
    
    let status = response.status();
    
    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "Server returned error code: {} - {}", 
            status.as_u16(), 
            status.canonical_reason().unwrap_or("Unknown")
        ));
    }
    
    let keys_response: KeysResponse = response
        .json()
        .context("Failed to parse JSON response")?;
    
    // Check if the output is being piped (not connected to a terminal)
    // Use raw/minimal output when piped to another command
    if !atty::is(atty::Stream::Stdout) {
        for key in &keys_response.keys {
            println!("{}", key.key);
        }
        return Ok(());
    }

    // Use the pretty print function for interactive terminal output
    pretty_print_keys(&keys_response);
    
    Ok(())
}
