use anyhow::{Context, Result};
use atty;
use colored::Colorize;
use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::utils::{pretty_print_table, ColumnConfig};

#[derive(Debug, Deserialize)]
pub struct KnownHostsResponse {
    pub version: String,
    #[serde(rename = "knownHosts")]
    pub hosts: Vec<KnownHost>,
}

#[derive(Debug, Deserialize)]
pub struct KnownHost {
    pub name: Option<String>,
    pub hosts: Vec<String>,
    pub keys: Vec<HostKey>,
}

#[derive(Debug, Deserialize)]
pub struct HostKey {
    #[serde(rename = "type")]
    pub key_type: String,
    pub key: String,
    pub comment: Option<String>,
    pub revoked: Option<bool>,
    #[serde(rename = "cert-authority")]
    pub cert_authority: Option<bool>,
}

/// Function to pretty print the known hosts with formatted columns and colors
pub fn pretty_print_known_hosts(response: &KnownHostsResponse) {
    // Find the maximum width for name and hosts columns for better formatting
    let max_name_len = response
        .hosts
        .iter()
        .filter_map(|h| h.name.as_ref())
        .map(|name| name.len())
        .max()
        .unwrap_or(4)
        .max(4);

    let max_hosts_len = response
        .hosts
        .iter()
        .map(|h| h.hosts.join(",").len())
        .max()
        .unwrap_or(5)
        .max(5);

    let max_type_len = response
        .hosts
        .iter()
        .flat_map(|h| h.keys.iter())
        .map(|k| k.key_type.len())
        .max()
        .unwrap_or(4)
        .max(4);

    let max_comment_len = response
        .hosts
        .iter()
        .flat_map(|h| h.keys.iter())
        .filter_map(|k| k.comment.as_ref())
        .map(|c| c.len())
        .max()
        .unwrap_or(7)
        .max(7); // "COMMENT" header is 7 chars

    // Define the columns
    let columns = vec![
        ColumnConfig {
            header: "NAME".to_string(),
            color: |s| s.green(),
            width: max_name_len,
        },
        ColumnConfig {
            header: "HOSTS".to_string(),
            color: |s| s.cyan(),
            width: max_hosts_len,
        },
        ColumnConfig {
            header: "TYPE".to_string(),
            color: |s| s.blue(),
            width: max_type_len,
        },
        ColumnConfig {
            header: "FLAGS".to_string(),
            color: |s| s.yellow(),
            width: 10,
        },
        ColumnConfig {
            header: "COMMENT".to_string(),
            color: |s| s.magenta(),
            width: max_comment_len,
        },
        ColumnConfig {
            header: "KEY".to_string(),
            color: |s| s.red(),
            width: 50, // Key is typically long, so use a reasonable default width
        },
    ];

    // Prepare the rows - flattening the nested structure
    let mut rows: Vec<Vec<String>> = Vec::new();

    for host in &response.hosts {
        let name = host.name.clone().unwrap_or_default();
        let hosts_str = host.hosts.join(",");

        for key in &host.keys {
            // Create flags string based on boolean values
            let mut flags = Vec::new();
            if key.revoked.unwrap_or(false) {
                flags.push("REVOKED");
            }
            if key.cert_authority.unwrap_or(false) {
                flags.push("CA");
            }
            let flags_str = flags.join(",");

            // Get comment or empty string
            let comment = key.comment.clone().unwrap_or_default();

            rows.push(vec![
                name.clone(),
                hosts_str.clone(),
                key.key_type.clone(),
                flags_str,
                comment,
                key.key.clone(),
            ]);
        }
    }

    // Use the generic pretty print function
    pretty_print_table(
        "Known Hosts Server Version:",
        &response.version,
        columns,
        rows,
        "No known hosts found.",
    );
}

pub fn fetch_known_hosts(server_url: &str) -> Result<()> {
    let url = format!("{server_url}/known_hosts");

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header(ACCEPT, "application/json")
        .send()
        .context("Failed to send request to known hosts server")?;

    let status = response.status();

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "Server returned error code: {} - {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("Unknown")
        ));
    }

    let known_hosts_response: KnownHostsResponse =
        response.json().context("Failed to parse JSON response")?;

    // Check if the output is being piped (not connected to a terminal)
    // Use raw/minimal output when piped to another command
    if !atty::is(atty::Stream::Stdout) {
        for host in &known_hosts_response.hosts {
            for key in &host.keys {
                let hosts_str = host.hosts.join(",");
                let key_type = &key.key_type;
                let key_value = &key.key;

                // Add flags if present
                let mut flags = Vec::new();
                if key.revoked.unwrap_or(false) {
                    flags.push("@revoked");
                }
                if key.cert_authority.unwrap_or(false) {
                    flags.push("@cert-authority");
                }

                // Format comment if present
                let comment_str = if let Some(comment) = &key.comment {
                    format!(" # {comment}")
                } else {
                    String::new()
                };

                // Output in OpenSSH known_hosts format with flags and optional comment
                if flags.is_empty() {
                    println!("{hosts_str} {key_type} {key_value}{comment_str}");
                } else {
                    println!(
                        "{} {} {} {}{}",
                        flags.join(" "),
                        hosts_str,
                        key_type,
                        key_value,
                        comment_str
                    );
                }
            }
        }
        return Ok(());
    }

    // Use the pretty print function for interactive terminal output
    pretty_print_known_hosts(&known_hosts_response);

    Ok(())
}
