use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use reqwest::header::ACCEPT;
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

#[derive(Debug, Deserialize)]
struct PGPKeysResponse {
    version: String,
    keys: Vec<PGPKey>,
}

#[derive(Debug, Deserialize)]
struct PGPKey {
    name: String,
    key: String,
}

#[derive(Debug, Deserialize)]
struct KnownHostsResponse {
    version: String,
    #[serde(rename = "knownHosts")]
    hosts: Vec<KnownHost>,
}

#[derive(Debug, Deserialize)]
struct KnownHost {
    name: Option<String>,
    hosts: Vec<String>,
    keys: Vec<HostKey>,
}

#[derive(Debug, Deserialize)]
struct HostKey {
    #[serde(rename = "type")]
    key_type: String,
    key: String,
    comment: Option<String>,
    revoked: Option<bool>,
    #[serde(rename = "cert-authority")]
    cert_authority: Option<bool>,
}

/// Struct to represent a column configuration for pretty printing
struct ColumnConfig {
    header: String,
    color: fn(&str) -> colored::ColoredString,
    width: usize,
}

/// Generic function to pretty print tabular data with formatted columns and colors
fn pretty_print_table(
    title: &str,
    version: &str,
    columns: Vec<ColumnConfig>,
    rows: Vec<Vec<String>>,
    empty_message: &str,
) {
    // Print the version information
    println!("{} {}", title.purple().bold(), version);
    println!();

    if rows.is_empty() {
        println!("{}", empty_message.yellow().italic());
        return;
    }

    let column_spacing = 3;

    // Print header
    let mut header_str = String::new();
    let mut divider_len = 0;

    for (i, col) in columns.iter().enumerate() {
        if i < columns.len() - 1 {
            header_str.push_str(&format!(
                "{:width$}",
                col.header.green().bold(),
                width = col.width + column_spacing
            ));
        } else {
            // Last column doesn't need padding
            header_str.push_str(&col.header.green().bold().to_string());
        }
        divider_len += col.width;
    }

    // Add spacing between columns to divider length
    divider_len += (columns.len() - 1) * column_spacing;
    // Add extra padding for better visual appearance
    divider_len += 30;

    println!("{header_str}");
    println!("{}", "-".repeat(divider_len));

    // Print each row with the specified colors
    for row in rows {
        let mut row_str = String::new();

        for (i, (value, col)) in row.iter().zip(columns.iter()).enumerate() {
            if i < columns.len() - 1 {
                let padded = pad_string(value, col.width);
                row_str.push_str(&format!(
                    "{:width$}",
                    (col.color)(&padded),
                    width = col.width + column_spacing
                ));
            } else {
                // Last column doesn't need padding
                row_str.push_str(&(col.color)(value).to_string());
            }
        }

        println!("{row_str}");
    }
}

/// Function to pretty print the SSH keys with formatted columns and colors
fn pretty_print_ssh_keys(keys_response: &KeysResponse) {
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

/// Function to pretty print the PGP keys with formatted columns and colors
fn pretty_print_pgp_keys(keys_response: &PGPKeysResponse) {
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

/// Function to pretty print the known hosts with formatted columns and colors
fn pretty_print_known_hosts(response: &KnownHostsResponse) {
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The server URL
    #[arg(short, long, default_value = "http://localhost:8000")]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Fetch SSH keys from the server
    Keys {},

    /// Fetch PGP keys from the server
    PgpKeys {},

    /// Fetch known hosts from the server
    KnownHosts {},
}

fn fetch_ssh_keys(server_url: &str) -> Result<()> {
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

fn fetch_pgp_keys(server_url: &str) -> Result<()> {
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

fn fetch_known_hosts(server_url: &str) -> Result<()> {
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Keys {} => {
            fetch_ssh_keys(&cli.server)?;
        }
        Commands::PgpKeys {} => {
            fetch_pgp_keys(&cli.server)?;
        }
        Commands::KnownHosts {} => {
            fetch_known_hosts(&cli.server)?;
        }
    }

    Ok(())
}
