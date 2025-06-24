use anyhow::{Context, Result};
use atty;
use colored::Colorize;
use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::utils::{pretty_print_table, ColumnConfig};

#[derive(Debug, Deserialize)]
pub struct PGPKeysResponse {
    pub version: String,
    pub keys: Vec<PGPKey>,
}

#[derive(Debug, Deserialize)]
pub struct PGPKey {
    pub name: String,
    pub key: String,
}

/// Function to pretty print the PGP keys with formatted columns and colors
pub fn pretty_print_pgp_keys(keys_response: &PGPKeysResponse) {
    // Find the maximum width for name column for better formatting
    let max_name_len = keys_response
        .keys
        .iter()
        .map(|k| k.name.len())
        .max()
        .unwrap_or(4)
        .max(4);

    // Define the columns
    let columns = vec![
        ColumnConfig {
            header: "NAME".to_string(),
            color: |s| s.green(),
            width: max_name_len,
        },
        ColumnConfig {
            header: "KEY".to_string(),
            color: |s| s.red(),
            width: 50, // Key is typically long, so use a reasonable default width
        },
    ];

    // Prepare the rows
    let rows: Vec<Vec<String>> = keys_response
        .keys
        .iter()
        .map(|key| vec![key.name.clone(), key.key.clone()])
        .collect();

    // Use the generic pretty print function
    pretty_print_table(
        "PGP Keys Server Version:",
        &keys_response.version,
        columns,
        rows,
        "No PGP keys found matching the criteria.",
    );
}

pub fn fetch_pgp_keys(server_url: &str) -> Result<()> {
    let url = format!("{server_url}/pgp");

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header(ACCEPT, "application/json")
        .send()
        .context("Failed to send request to PGP keys server")?;

    let status = response.status();

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "Server returned error code: {} - {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("Unknown")
        ));
    }

    let keys_response: PGPKeysResponse =
        response.json().context("Failed to parse JSON response")?;

    // Check if the output is being piped (not connected to a terminal)
    // Use raw/minimal output when piped to another command
    if !atty::is(atty::Stream::Stdout) {
        for key in &keys_response.keys {
            println!("{}", key.key);
        }
        return Ok(());
    }

    // Use the pretty print function for interactive terminal output
    pretty_print_pgp_keys(&keys_response);

    Ok(())
}
