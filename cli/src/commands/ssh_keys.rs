use anyhow::{Context, Result};
use atty;
use colored::Colorize;
use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::utils::{pretty_print_table, ColumnConfig};

#[derive(Debug, Deserialize)]
pub struct KeysResponse {
    pub version: String,
    pub keys: Vec<SSHKey>,
}

#[derive(Debug, Deserialize)]
pub struct SSHKey {
    pub key: String,
    pub user: String,
    pub name: String,
    pub tags: Vec<String>,
}

/// Function to pretty print the SSH keys with formatted columns and colors
pub fn pretty_print_ssh_keys(keys_response: &KeysResponse) {
    // Find the maximum width for each column for better formatting
    let max_name_len = keys_response
        .keys
        .iter()
        .map(|k| k.name.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let max_user_len = keys_response
        .keys
        .iter()
        .map(|k| k.user.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let max_tags_len = keys_response
        .keys
        .iter()
        .map(|k| k.tags.join(", ").len())
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
            header: "USER".to_string(),
            color: |s| s.blue(),
            width: max_user_len,
        },
        ColumnConfig {
            header: "TAGS".to_string(),
            color: |s| s.yellow(),
            width: max_tags_len,
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
        .map(|key| {
            vec![
                key.name.clone(),
                key.user.clone(),
                key.tags.join(", "),
                key.key.clone(),
            ]
        })
        .collect();

    // Use the generic pretty print function
    pretty_print_table(
        "Keys Server Version:",
        &keys_response.version,
        columns,
        rows,
        "No SSH keys found matching the criteria.",
    );
}

pub fn fetch_ssh_keys(server_url: &str) -> Result<()> {
    let url = format!("{server_url}/keys");

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

    let keys_response: KeysResponse = response.json().context("Failed to parse JSON response")?;

    // Check if the output is being piped (not connected to a terminal)
    // Use raw/minimal output when piped to another command
    if !atty::is(atty::Stream::Stdout) {
        for key in &keys_response.keys {
            println!("{}", key.key);
        }
        return Ok(());
    }

    // Use the pretty print function for interactive terminal output
    pretty_print_ssh_keys(&keys_response);

    Ok(())
}
