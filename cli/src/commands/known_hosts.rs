use std::io::IsTerminal;

use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::utils::{ColumnConfig, backup_existing_file, pretty_print_table};

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

/// Private function to fetch known hosts from the server
///
/// This function handles the HTTP request to the known hosts server,
/// validates the response, and parses the JSON into a KnownHostsResponse.
///
/// # Arguments
/// * `server_url` - The base URL of the keys server
///
/// # Returns
/// * `Result<KnownHostsResponse>` - The parsed known hosts response or an error
fn fetch_known_hosts_from_server(server_url: &str) -> Result<KnownHostsResponse> {
    let url = format!("{server_url}/known_hosts");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")?;
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

    let known_hosts_response = response
        .json::<KnownHostsResponse>()
        .context("Failed to parse JSON response")?;
    validate_known_hosts_response(&known_hosts_response)?;
    Ok(known_hosts_response)
}

/// Reject server entries that would produce malformed or forged known_hosts
/// lines. Host patterns, key types, and keys are interpolated directly into a
/// space-delimited line, so whitespace (including newlines) in any of them
/// would break the format or inject additional entries; a newline in a comment
/// could do the same past the `#`.
fn validate_known_hosts_response(response: &KnownHostsResponse) -> Result<()> {
    for host in &response.hosts {
        if host.hosts.is_empty() {
            return Err(anyhow::anyhow!(
                "Server returned a known_hosts entry with no host patterns"
            ));
        }

        for pattern in &host.hosts {
            if pattern.is_empty() || pattern.contains(char::is_whitespace) {
                return Err(anyhow::anyhow!(
                    "Server known_hosts entry has an invalid host pattern: {pattern:?}"
                ));
            }
        }

        for key in &host.keys {
            if key.key_type.is_empty() || key.key_type.contains(char::is_whitespace) {
                return Err(anyhow::anyhow!(
                    "Server known_hosts entry has an invalid key type: {:?}",
                    key.key_type
                ));
            }

            if key.key.is_empty() || key.key.contains(char::is_whitespace) {
                return Err(anyhow::anyhow!(
                    "Server known_hosts entry has an invalid key value: {:?}",
                    key.key
                ));
            }

            if let Some(comment) = &key.comment
                && (comment.contains('\n') || comment.contains('\r'))
            {
                return Err(anyhow::anyhow!(
                    "Server known_hosts entry has a comment with an illegal line break: {comment:?}"
                ));
            }
        }
    }

    Ok(())
}

pub fn fetch_known_hosts(server_url: &str) -> Result<()> {
    let known_hosts_response = fetch_known_hosts_from_server(server_url)?;

    // Check if the output is being piped (not connected to a terminal)
    // Use raw/minimal output when piped to another command
    if !std::io::stdout().is_terminal() {
        for host in &known_hosts_response.hosts {
            for key in &host.keys {
                // Reuse the shared formatter so the piped output matches what
                // `--write` produces, including single-marker handling.
                println!("{}", format_known_hosts_line(host, key));
            }
        }
        return Ok(());
    }

    // Use the pretty print function for interactive terminal output
    pretty_print_known_hosts(&known_hosts_response);

    Ok(())
}

/// Helper function to format a host entry in known_hosts format
fn format_known_hosts_line(host: &KnownHost, key: &HostKey) -> String {
    let hosts_str = host.hosts.join(",");
    let key_type = &key.key_type;
    let key_value = &key.key;

    // Add a marker if present. A known_hosts line carries at most one marker;
    // `@revoked` and `@cert-authority` are mutually exclusive, so prefer
    // `@revoked` when the server marks a key as both (a revoked CA must not be
    // trusted).
    let marker = marker_for(key);

    // Format comment if present
    let comment_str = if let Some(comment) = &key.comment {
        format!(" # {comment}")
    } else {
        String::new()
    };

    // Output in OpenSSH known_hosts format with marker and optional comment
    match marker {
        Some(marker) => format!("{marker} {hosts_str} {key_type} {key_value}{comment_str}"),
        None => format!("{hosts_str} {key_type} {key_value}{comment_str}"),
    }
}

/// The single OpenSSH marker for a key, or `None` if it carries neither.
/// `@revoked` and `@cert-authority` are mutually exclusive; `@revoked` wins.
fn marker_for(key: &HostKey) -> Option<&'static str> {
    if key.revoked.unwrap_or(false) {
        Some("@revoked")
    } else if key.cert_authority.unwrap_or(false) {
        Some("@cert-authority")
    } else {
        None
    }
}

/// Identity of a server-provided known_hosts entry: the `hosts key_type key`
/// triple, ignoring marker flags and comments. Used to detect whether an entry
/// is already present locally.
fn server_entry_identity(host: &KnownHost, key: &HostKey) -> String {
    format!("{} {} {}", host.hosts.join(","), key.key_type, key.key)
}

/// Extract the identity (`hosts key_type key`) from an existing known_hosts
/// line, skipping any leading marker flags (`@revoked`, `@cert-authority`) and
/// the trailing `# comment`. Returns `None` if the line is too short to parse.
fn extract_known_hosts_identity(line: &str) -> Option<String> {
    let without_comment = line.split('#').next().unwrap_or(line);
    let mut tokens = without_comment
        .split_whitespace()
        .skip_while(|t| t.starts_with('@'));
    let hosts = tokens.next()?;
    let key_type = tokens.next()?;
    let key = tokens.next()?;
    Some(format!("{hosts} {key_type} {key}"))
}

pub fn write_known_hosts(server_url: &str, file_path: &str, force: bool) -> Result<()> {
    // Fetch known hosts from the server
    let known_hosts_response = fetch_known_hosts_from_server(server_url)?;

    // Expand ~ to home directory if present
    let expanded_path = shellexpand::tilde(file_path);
    let path = std::path::Path::new(expanded_path.as_ref());

    // Read existing entries if the file exists, skipping blank and comment-only
    // lines so they don't get treated as host entries.
    let existing_lines = if path.exists() {
        std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read existing file: {}", path.display()))?
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }

    // Flatten server entries into (identity, formatted line) pairs.
    let server_entries: Vec<(String, String)> = known_hosts_response
        .hosts
        .iter()
        .flat_map(|host| {
            host.keys.iter().map(move |key| {
                (
                    server_entry_identity(host, key),
                    format_known_hosts_line(host, key),
                )
            })
        })
        .collect();
    let num_server_entries = server_entries.len();

    // Identity of each existing line (None if it can't be parsed).
    let existing_identities: Vec<Option<String>> = existing_lines
        .iter()
        .map(|line| extract_known_hosts_identity(line))
        .collect();

    let is_on_server = |identity: &str| server_entries.iter().any(|(sid, _)| sid == identity);
    let is_present_locally = |identity: &str| {
        existing_identities
            .iter()
            .any(|id| id.as_deref() == Some(identity))
    };

    // Entries present locally but absent from the server (reported in safe mode).
    let num_local_only = existing_identities
        .iter()
        .filter(|id| id.as_deref().map(|i| !is_on_server(i)).unwrap_or(true))
        .count();

    let mut updated_count = 0;

    let file_content = if force {
        // Force mode: replace the file with exactly the server entries.
        server_entries
            .iter()
            .map(|(_, line)| line.clone())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        // Safe mode: keep existing entries, refreshing those that match a server
        // entry, then append server entries that aren't already present.
        let mut result_lines = Vec::new();

        for (existing_line, identity) in existing_lines.iter().zip(existing_identities.iter()) {
            if let Some(identity) = identity
                && let Some((_, server_line)) =
                    server_entries.iter().find(|(sid, _)| sid == identity)
            {
                if server_line != existing_line {
                    updated_count += 1;
                }
                result_lines.push(server_line.clone());
            } else {
                result_lines.push(existing_line.clone());
            }
        }

        for (identity, server_line) in &server_entries {
            if !is_present_locally(identity) {
                result_lines.push(server_line.clone());
            }
        }

        result_lines.join("\n")
    };

    // Write to file, ensuring a trailing newline when there is content.
    let file_content = if file_content.is_empty() {
        String::new()
    } else {
        format!("{file_content}\n")
    };

    // Back up the existing file before overwriting it, so a bad merge or a
    // surprising server response can be recovered from.
    if let Some(backup) = backup_existing_file(path)? {
        println!("📦 Backed up existing file to {}", backup.display());
    }

    std::fs::write(path, &file_content)
        .with_context(|| format!("Failed to write to file: {}", path.display()))?;

    // Report what happened.
    let num_existing = existing_lines.len();
    if force {
        println!(
            "✅ Wrote {} known host entries to {} (overwriting {} existing entries)",
            num_server_entries,
            path.display(),
            num_existing
        );
    } else {
        let num_added = server_entries
            .iter()
            .filter(|(identity, _)| !is_present_locally(identity))
            .count();

        if num_added > 0 {
            let mut message = format!(
                "✅ Added {} new known host entries to {}",
                num_added,
                path.display()
            );
            if updated_count > 0 {
                message.push_str(&format!(" and updated {updated_count} existing entries"));
            }
            println!("{message}");
        } else {
            let mut message = format!(
                "✅ Server known host entries are already present at {}",
                path.display()
            );
            if updated_count > 0 {
                message.push_str(&format!(" (updated {updated_count} entries)"));
            }
            println!("{message}");
        }

        if num_local_only > 0 {
            println!(
                "{}  {} local entries were not removed (use {} to remove)",
                "⚠️".yellow().bold(),
                num_local_only.to_string().yellow().bold(),
                "--force".yellow().bold()
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    // Helper function to create a mock server
    fn setup_mock_server(response_body: &str) -> (String, mockito::ServerGuard) {
        let mut mock_server = mockito::Server::new();

        mock_server
            .mock("GET", "/known_hosts")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create();

        (mock_server.url(), mock_server)
    }

    // Helper function to create a mock server with error response
    fn setup_mock_server_with_error(
        status_code: usize,
        response_body: &str,
    ) -> (String, mockito::ServerGuard) {
        let mut mock_server = mockito::Server::new();

        mock_server
            .mock("GET", "/known_hosts")
            .with_status(status_code)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create();

        (mock_server.url(), mock_server)
    }

    #[test]
    fn test_fetch_known_hosts_success() {
        // Setup mock server with known hosts response
        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "name": "GitHub",
                    "hosts": ["github.com", "*.github.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7",
                            "comment": "GitHub RSA key",
                            "revoked": false,
                            "cert-authority": false
                        }
                    ]
                }
            ]
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Call function
        let result = fetch_known_hosts(&server_url);
        assert!(
            result.is_ok(),
            "fetch_known_hosts failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_fetch_known_hosts_server_error() {
        // Setup mock server with error response
        let (server_url, _server) =
            setup_mock_server_with_error(500, r#"{"error": "Internal server error"}"#);

        // Call function
        let result = fetch_known_hosts(&server_url);

        // Should return an error
        assert!(result.is_err());
        let error_msg = result.err().unwrap().to_string();
        assert!(error_msg.contains("Server returned error code: 500"));
    }

    #[test]
    fn test_fetch_known_hosts_malformed_response() {
        // Setup mock server with malformed JSON
        let (server_url, _server) =
            setup_mock_server(r#"{"version": "1.0.0", "knownHosts": [{"incomplete": true}]}"#);

        // Call function
        let result = fetch_known_hosts(&server_url);

        // Should return an error due to missing required fields
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_known_hosts_empty_response() {
        // Setup mock server with empty known hosts array
        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": []
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Call function
        let result = fetch_known_hosts(&server_url);
        assert!(
            result.is_ok(),
            "fetch_known_hosts failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_fetch_known_hosts_multiple_hosts_and_keys() {
        // Setup mock server with multiple hosts and keys
        let mock_response = r#"
        {
            "version": "2.1.0",
            "knownHosts": [
                {
                    "name": "GitHub",
                    "hosts": ["github.com", "*.github.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7GitHub1",
                            "comment": "GitHub RSA key"
                        },
                        {
                            "type": "ssh-ed25519",
                            "key": "AAAAC3NzaC1lZDI1NTE5AAAAIGitHub2",
                            "comment": "GitHub Ed25519 key"
                        }
                    ]
                },
                {
                    "name": "GitLab",
                    "hosts": ["gitlab.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7GitLab1",
                            "comment": "GitLab RSA key",
                            "revoked": true
                        }
                    ]
                },
                {
                    "hosts": ["example.com"],
                    "keys": [
                        {
                            "type": "ssh-ed25519",
                            "key": "AAAAC3NzaC1lZDI1NTE5AAAAIExample1",
                            "cert-authority": true
                        }
                    ]
                }
            ]
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Call function
        let result = fetch_known_hosts(&server_url);
        assert!(
            result.is_ok(),
            "fetch_known_hosts failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_fetch_known_hosts_with_flags() {
        // Setup mock server with keys that have revoked and cert-authority flags
        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "name": "Test Host",
                    "hosts": ["test.example.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7Revoked",
                            "comment": "Revoked key",
                            "revoked": true,
                            "cert-authority": false
                        },
                        {
                            "type": "ssh-ed25519",
                            "key": "AAAAC3NzaC1lZDI1NTE5AAAAICertAuth",
                            "comment": "CA key",
                            "revoked": false,
                            "cert-authority": true
                        },
                        {
                            "type": "ssh-rsa",
                            "key": "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7Both",
                            "comment": "Both flags",
                            "revoked": true,
                            "cert-authority": true
                        }
                    ]
                }
            ]
        }
        "#;

        let (server_url, _server) = setup_mock_server(mock_response);

        // Call function
        let result = fetch_known_hosts(&server_url);
        assert!(
            result.is_ok(),
            "fetch_known_hosts failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_pretty_print_known_hosts() {
        // Create a test response with various host and key data
        let known_hosts_response = KnownHostsResponse {
            version: "1.0.0".to_string(),
            hosts: vec![
                KnownHost {
                    name: Some("GitHub".to_string()),
                    hosts: vec!["github.com".to_string(), "*.github.com".to_string()],
                    keys: vec![
                        HostKey {
                            key_type: "ssh-rsa".to_string(),
                            key: "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7GitHub".to_string(),
                            comment: Some("GitHub RSA key".to_string()),
                            revoked: Some(false),
                            cert_authority: Some(false),
                        },
                        HostKey {
                            key_type: "ssh-ed25519".to_string(),
                            key: "AAAAC3NzaC1lZDI1NTE5AAAAIGitHub2".to_string(),
                            comment: None,
                            revoked: None,
                            cert_authority: Some(true),
                        },
                    ],
                },
                KnownHost {
                    name: None,
                    hosts: vec!["example.com".to_string()],
                    keys: vec![HostKey {
                        key_type: "ssh-rsa".to_string(),
                        key: "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7Example".to_string(),
                        comment: Some("Example key".to_string()),
                        revoked: Some(true),
                        cert_authority: Some(false),
                    }],
                },
            ],
        };

        // This test primarily verifies the function doesn't panic and handles the data correctly
        // Since pretty_print_known_hosts outputs to stdout, we can't easily capture and verify output
        // in this test environment, but we can verify it completes without errors
        pretty_print_known_hosts(&known_hosts_response);
    }

    #[test]
    fn test_pretty_print_known_hosts_empty() {
        // Test with empty hosts list
        let known_hosts_response = KnownHostsResponse {
            version: "1.0.0".to_string(),
            hosts: vec![],
        };

        // Should handle empty hosts gracefully
        pretty_print_known_hosts(&known_hosts_response);
    }

    #[test]
    fn test_pretty_print_known_hosts_no_names() {
        // Test with hosts that have no names
        let known_hosts_response = KnownHostsResponse {
            version: "1.0.0".to_string(),
            hosts: vec![
                KnownHost {
                    name: None,
                    hosts: vec!["host1.example.com".to_string()],
                    keys: vec![HostKey {
                        key_type: "ssh-rsa".to_string(),
                        key: "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7Host1".to_string(),
                        comment: None,
                        revoked: None,
                        cert_authority: None,
                    }],
                },
                KnownHost {
                    name: None,
                    hosts: vec!["host2.example.com".to_string()],
                    keys: vec![HostKey {
                        key_type: "ssh-ed25519".to_string(),
                        key: "AAAAC3NzaC1lZDI1NTE5AAAAIHost2".to_string(),
                        comment: None,
                        revoked: None,
                        cert_authority: None,
                    }],
                },
            ],
        };

        // Should handle missing names gracefully
        pretty_print_known_hosts(&known_hosts_response);
    }

    #[test]
    fn test_pretty_print_known_hosts_long_data() {
        // Test with very long host names, comments, and multiple hosts per entry
        let known_hosts_response = KnownHostsResponse {
            version: "1.0.0".to_string(),
            hosts: vec![
                KnownHost {
                    name: Some("Very Long Service Name That Should Test Column Width Calculations".to_string()),
                    hosts: vec![
                        "very-long-hostname-that-tests-column-width.example.com".to_string(),
                        "another-very-long-hostname.example.com".to_string(),
                        "third-hostname.example.com".to_string()
                    ],
                    keys: vec![
                        HostKey {
                            key_type: "ssh-rsa".to_string(),
                            key: "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7VeryLongKeyDataThatShouldTestTheKeyColumnWidthHandling".to_string(),
                            comment: Some("This is a very long comment that should test the comment column width handling and make sure everything aligns properly".to_string()),
                            revoked: Some(true),
                            cert_authority: Some(true),
                        }
                    ],
                }
            ],
        };

        // Should handle long data gracefully
        pretty_print_known_hosts(&known_hosts_response);
    }

    #[test]
    fn test_deserialize_known_hosts_response() {
        // Test JSON deserialization
        let json_data = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "name": "Test",
                    "hosts": ["test.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7Test",
                            "comment": "Test comment",
                            "revoked": true,
                            "cert-authority": false
                        }
                    ]
                }
            ]
        }
        "#;

        let result: Result<KnownHostsResponse, _> = serde_json::from_str(json_data);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.version, "1.0.0");
        assert_eq!(response.hosts.len(), 1);
        assert_eq!(response.hosts[0].name, Some("Test".to_string()));
        assert_eq!(response.hosts[0].hosts, vec!["test.com"]);
        assert_eq!(response.hosts[0].keys.len(), 1);
        assert_eq!(response.hosts[0].keys[0].key_type, "ssh-rsa");
        assert_eq!(
            response.hosts[0].keys[0].key,
            "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7Test"
        );
        assert_eq!(
            response.hosts[0].keys[0].comment,
            Some("Test comment".to_string())
        );
        assert_eq!(response.hosts[0].keys[0].revoked, Some(true));
        assert_eq!(response.hosts[0].keys[0].cert_authority, Some(false));
    }

    #[test]
    fn test_deserialize_known_hosts_response_minimal() {
        // Test JSON deserialization with minimal required fields
        let json_data = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "hosts": ["minimal.com"],
                    "keys": [
                        {
                            "type": "ssh-ed25519",
                            "key": "AAAAC3NzaC1lZDI1NTE5AAAAIMinimal"
                        }
                    ]
                }
            ]
        }
        "#;

        let result: Result<KnownHostsResponse, _> = serde_json::from_str(json_data);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.version, "1.0.0");
        assert_eq!(response.hosts.len(), 1);
        assert_eq!(response.hosts[0].name, None);
        assert_eq!(response.hosts[0].hosts, vec!["minimal.com"]);
        assert_eq!(response.hosts[0].keys.len(), 1);
        assert_eq!(response.hosts[0].keys[0].key_type, "ssh-ed25519");
        assert_eq!(
            response.hosts[0].keys[0].key,
            "AAAAC3NzaC1lZDI1NTE5AAAAIMinimal"
        );
        assert_eq!(response.hosts[0].keys[0].comment, None);
        assert_eq!(response.hosts[0].keys[0].revoked, None);
        assert_eq!(response.hosts[0].keys[0].cert_authority, None);
    }

    #[test]
    fn test_deserialize_known_hosts_response_invalid() {
        // Test JSON deserialization with missing required fields
        let json_data = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "name": "Invalid",
                    "keys": [
                        {
                            "type": "ssh-rsa"
                        }
                    ]
                }
            ]
        }
        "#;

        let result: Result<KnownHostsResponse, _> = serde_json::from_str(json_data);
        assert!(result.is_err()); // Should fail due to missing required fields
    }

    #[test]
    fn test_fetch_known_hosts_network_error() {
        // Test with invalid URL to simulate network error
        let result = fetch_known_hosts("http://invalid-url-that-does-not-exist.local");
        assert!(result.is_err());
    }

    // ==================== format_known_hosts_line tests ====================

    #[test]
    fn test_format_known_hosts_line_basic() {
        let host = KnownHost {
            name: Some("GitHub".to_string()),
            hosts: vec!["github.com".to_string()],
            keys: vec![HostKey {
                key_type: "ssh-rsa".to_string(),
                key: "AAAAB3NzaC1yc2EAAAADAQABAAAB".to_string(),
                comment: None,
                revoked: None,
                cert_authority: None,
            }],
        };

        let result = format_known_hosts_line(&host, &host.keys[0]);
        assert_eq!(result, "github.com ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAB");
    }

    #[test]
    fn test_format_known_hosts_line_multiple_hosts() {
        let host = KnownHost {
            name: Some("GitHub".to_string()),
            hosts: vec![
                "github.com".to_string(),
                "*.github.com".to_string(),
                "192.30.255.112".to_string(),
            ],
            keys: vec![HostKey {
                key_type: "ssh-ed25519".to_string(),
                key: "AAAAC3NzaC1lZDI1NTE5".to_string(),
                comment: None,
                revoked: None,
                cert_authority: None,
            }],
        };

        let result = format_known_hosts_line(&host, &host.keys[0]);
        assert_eq!(
            result,
            "github.com,*.github.com,192.30.255.112 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5"
        );
    }

    #[test]
    fn test_format_known_hosts_line_with_comment() {
        let host = KnownHost {
            name: Some("Test".to_string()),
            hosts: vec!["example.com".to_string()],
            keys: vec![HostKey {
                key_type: "ssh-rsa".to_string(),
                key: "AAAAB3NzaC1yc2ETEST".to_string(),
                comment: Some("Example RSA key".to_string()),
                revoked: None,
                cert_authority: None,
            }],
        };

        let result = format_known_hosts_line(&host, &host.keys[0]);
        assert_eq!(
            result,
            "example.com ssh-rsa AAAAB3NzaC1yc2ETEST # Example RSA key"
        );
    }

    #[test]
    fn test_format_known_hosts_line_with_revoked_flag() {
        let host = KnownHost {
            name: None,
            hosts: vec!["revoked.example.com".to_string()],
            keys: vec![HostKey {
                key_type: "ssh-rsa".to_string(),
                key: "AAAAB3NzaC1yc2EREVOKED".to_string(),
                comment: None,
                revoked: Some(true),
                cert_authority: None,
            }],
        };

        let result = format_known_hosts_line(&host, &host.keys[0]);
        assert_eq!(
            result,
            "@revoked revoked.example.com ssh-rsa AAAAB3NzaC1yc2EREVOKED"
        );
    }

    #[test]
    fn test_format_known_hosts_line_with_cert_authority_flag() {
        let host = KnownHost {
            name: None,
            hosts: vec!["ca.example.com".to_string()],
            keys: vec![HostKey {
                key_type: "ssh-ed25519".to_string(),
                key: "AAAAC3NzaC1lZDI1NTE5CA".to_string(),
                comment: None,
                revoked: None,
                cert_authority: Some(true),
            }],
        };

        let result = format_known_hosts_line(&host, &host.keys[0]);
        assert_eq!(
            result,
            "@cert-authority ca.example.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5CA"
        );
    }

    #[test]
    fn test_format_known_hosts_line_with_both_flags() {
        let host = KnownHost {
            name: None,
            hosts: vec!["both.example.com".to_string()],
            keys: vec![HostKey {
                key_type: "ssh-rsa".to_string(),
                key: "AAAAB3NzaC1yc2EBOTH".to_string(),
                comment: None,
                revoked: Some(true),
                cert_authority: Some(true),
            }],
        };

        // `@revoked` and `@cert-authority` are mutually exclusive; when the
        // server sets both, only `@revoked` is emitted.
        let result = format_known_hosts_line(&host, &host.keys[0]);
        assert_eq!(
            result,
            "@revoked both.example.com ssh-rsa AAAAB3NzaC1yc2EBOTH"
        );
        assert!(!result.contains("@cert-authority"));
    }

    #[test]
    fn test_format_known_hosts_line_with_flags_and_comment() {
        let host = KnownHost {
            name: Some("Full example".to_string()),
            hosts: vec!["full.example.com".to_string(), "192.168.1.1".to_string()],
            keys: vec![HostKey {
                key_type: "ssh-rsa".to_string(),
                key: "AAAAB3NzaC1yc2EFULL".to_string(),
                comment: Some("Full example with all options".to_string()),
                revoked: Some(true),
                cert_authority: Some(true),
            }],
        };

        // Both flags set on the server collapse to the single `@revoked` marker.
        let result = format_known_hosts_line(&host, &host.keys[0]);
        assert_eq!(
            result,
            "@revoked full.example.com,192.168.1.1 ssh-rsa AAAAB3NzaC1yc2EFULL # Full example with all options"
        );
    }

    #[test]
    fn test_format_known_hosts_line_flags_false_not_included() {
        let host = KnownHost {
            name: None,
            hosts: vec!["noflags.example.com".to_string()],
            keys: vec![HostKey {
                key_type: "ssh-rsa".to_string(),
                key: "AAAAB3NzaC1yc2ENOFLAGS".to_string(),
                comment: None,
                revoked: Some(false),
                cert_authority: Some(false),
            }],
        };

        let result = format_known_hosts_line(&host, &host.keys[0]);
        // When flags are explicitly false, they should not be included
        assert_eq!(result, "noflags.example.com ssh-rsa AAAAB3NzaC1yc2ENOFLAGS");
        assert!(!result.contains("@revoked"));
        assert!(!result.contains("@cert-authority"));
    }

    // ==================== write_known_hosts tests ====================

    #[test]
    fn test_write_known_hosts_creates_file() {
        use std::fs;
        use tempfile::tempdir;

        // Setup mock server
        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "name": "GitHub",
                    "hosts": ["github.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "AAAAB3NzaC1yc2EAAAADAQABAAABgQC7",
                            "comment": "GitHub RSA key"
                        }
                    ]
                }
            ]
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        // Create temp directory
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");

        // Call function
        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), false);
        assert!(
            result.is_ok(),
            "write_known_hosts failed: {:?}",
            result.err()
        );

        // Verify file was created
        assert!(file_path.exists());

        // Verify file contents
        let contents = fs::read_to_string(&file_path).unwrap();
        assert!(contents.contains("github.com ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7"));
        assert!(contents.contains("# GitHub RSA key"));
    }

    #[test]
    fn test_write_known_hosts_creates_parent_directories() {
        use std::fs;
        use tempfile::tempdir;

        // Setup mock server
        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "hosts": ["example.com"],
                    "keys": [
                        {
                            "type": "ssh-ed25519",
                            "key": "AAAAC3NzaC1lZDI1NTE5"
                        }
                    ]
                }
            ]
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        // Create temp directory with nested path
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("known_hosts");

        // Parent directories don't exist yet
        assert!(!file_path.parent().unwrap().exists());

        // Call function
        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), false);
        assert!(
            result.is_ok(),
            "write_known_hosts failed: {:?}",
            result.err()
        );

        // Verify file and parent directories were created
        assert!(file_path.exists());
        let contents = fs::read_to_string(&file_path).unwrap();
        assert!(contents.contains("example.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5"));
    }

    #[test]
    fn test_write_known_hosts_multiple_hosts_and_keys() {
        use std::fs;
        use tempfile::tempdir;

        // Setup mock server with multiple hosts and keys
        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "name": "GitHub",
                    "hosts": ["github.com", "*.github.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "AAAAB3NzaC1yc2EGitHub1"
                        },
                        {
                            "type": "ssh-ed25519",
                            "key": "AAAAC3NzaC1lZDI1NTE5GitHub3"
                        }
                    ]
                },
                {
                    "name": "GitLab",
                    "hosts": ["gitlab.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "AAAAB3NzaC1yc2EGitLab1"
                        }
                    ]
                }
            ]
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");

        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_ok());

        let contents = fs::read_to_string(&file_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();

        // Should have 3 lines (2 keys for GitHub, 1 for GitLab)
        assert_eq!(lines.len(), 3);
        assert!(contents.contains("github.com,*.github.com ssh-rsa AAAAB3NzaC1yc2EGitHub1"));
        assert!(
            contents.contains("github.com,*.github.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5GitHub3")
        );
        assert!(contents.contains("gitlab.com ssh-rsa AAAAB3NzaC1yc2EGitLab1"));
    }

    #[test]
    fn test_write_known_hosts_with_flags() {
        use std::fs;
        use tempfile::tempdir;

        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "hosts": ["revoked.example.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "REVOKED_KEY",
                            "revoked": true
                        }
                    ]
                },
                {
                    "hosts": ["ca.example.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "CA_KEY",
                            "cert-authority": true
                        }
                    ]
                }
            ]
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");

        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_ok());

        let contents = fs::read_to_string(&file_path).unwrap();
        assert!(contents.contains("@revoked revoked.example.com ssh-rsa REVOKED_KEY"));
        assert!(contents.contains("@cert-authority ca.example.com ssh-rsa CA_KEY"));
    }

    #[test]
    fn test_write_known_hosts_empty_response() {
        use std::fs;
        use tempfile::tempdir;

        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": []
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");

        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_ok());

        // File should exist but be empty
        assert!(file_path.exists());
        let contents = fs::read_to_string(&file_path).unwrap();
        assert!(contents.is_empty());
    }

    #[test]
    fn test_write_known_hosts_force_overwrites_existing_file() {
        use std::fs;
        use tempfile::tempdir;

        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "hosts": ["new.example.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "NEW_KEY"
                        }
                    ]
                }
            ]
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");

        // Create existing file with different content
        fs::write(&file_path, "old.example.com ssh-rsa OLD_KEY\n").unwrap();

        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), true);
        assert!(result.is_ok());

        let contents = fs::read_to_string(&file_path).unwrap();
        // With --force the old content should be gone
        assert!(!contents.contains("old.example.com"));
        assert!(!contents.contains("OLD_KEY"));
        // New content should be present
        assert!(contents.contains("new.example.com ssh-rsa NEW_KEY"));
    }

    #[test]
    fn test_write_known_hosts_backs_up_existing_file() {
        use std::fs;
        use tempfile::tempdir;

        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "hosts": ["new.example.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "NEW_KEY"
                        }
                    ]
                }
            ]
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");

        let existing_content = "old.example.com ssh-rsa OLD_KEY\n";
        fs::write(&file_path, existing_content).unwrap();

        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), true);
        assert!(result.is_ok());

        // A `.bak` with the pre-write contents sits alongside the file.
        let backup_path = file_path.with_file_name("known_hosts.bak");
        assert!(backup_path.exists(), "backup was not created");
        assert_eq!(fs::read_to_string(&backup_path).unwrap(), existing_content);
    }

    #[test]
    fn test_validate_known_hosts_response_rejects_bad_fields() {
        // Empty host list.
        let empty_hosts = KnownHostsResponse {
            version: "1.0.0".to_string(),
            hosts: vec![KnownHost {
                name: None,
                hosts: vec![],
                keys: vec![HostKey {
                    key_type: "ssh-rsa".to_string(),
                    key: "AAAA".to_string(),
                    comment: None,
                    revoked: None,
                    cert_authority: None,
                }],
            }],
        };
        assert!(validate_known_hosts_response(&empty_hosts).is_err());

        // Whitespace in a host pattern would split the space-delimited line.
        let spaced_host = KnownHostsResponse {
            version: "1.0.0".to_string(),
            hosts: vec![KnownHost {
                name: None,
                hosts: vec!["exa mple.com".to_string()],
                keys: vec![HostKey {
                    key_type: "ssh-rsa".to_string(),
                    key: "AAAA".to_string(),
                    comment: None,
                    revoked: None,
                    cert_authority: None,
                }],
            }],
        };
        assert!(validate_known_hosts_response(&spaced_host).is_err());

        // Newline in a comment could inject another entry.
        let bad_comment = KnownHostsResponse {
            version: "1.0.0".to_string(),
            hosts: vec![KnownHost {
                name: None,
                hosts: vec!["example.com".to_string()],
                keys: vec![HostKey {
                    key_type: "ssh-rsa".to_string(),
                    key: "AAAA".to_string(),
                    comment: Some("ok\nevil.com ssh-rsa INJECTED".to_string()),
                    revoked: None,
                    cert_authority: None,
                }],
            }],
        };
        assert!(validate_known_hosts_response(&bad_comment).is_err());
    }

    #[test]
    fn test_validate_known_hosts_response_accepts_valid() {
        let valid = KnownHostsResponse {
            version: "1.0.0".to_string(),
            hosts: vec![KnownHost {
                name: Some("GitHub".to_string()),
                hosts: vec!["github.com".to_string(), "*.github.com".to_string()],
                keys: vec![HostKey {
                    key_type: "ssh-rsa".to_string(),
                    key: "AAAAB3NzaC1yc2E".to_string(),
                    comment: Some("a normal comment".to_string()),
                    revoked: None,
                    cert_authority: None,
                }],
            }],
        };
        assert!(validate_known_hosts_response(&valid).is_ok());
    }

    #[test]
    fn test_write_known_hosts_rejects_malformed_server_response() {
        use std::fs;
        use tempfile::tempdir;

        // Whitespace in the key value must abort the write.
        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "hosts": ["example.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "AAAA INJECTED"
                        }
                    ]
                }
            ]
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");
        fs::write(&file_path, "old.example.com ssh-rsa OLD_KEY\n").unwrap();

        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), true);
        assert!(result.is_err());

        // Existing file untouched, no backup created.
        assert_eq!(
            fs::read_to_string(&file_path).unwrap(),
            "old.example.com ssh-rsa OLD_KEY\n"
        );
        assert!(!file_path.with_file_name("known_hosts.bak").exists());
    }

    #[test]
    fn test_write_known_hosts_no_backup_for_new_file() {
        use tempfile::tempdir;

        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "hosts": ["new.example.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "NEW_KEY"
                        }
                    ]
                }
            ]
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");

        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_ok());

        assert!(
            !file_path.with_file_name("known_hosts.bak").exists(),
            "no backup should be created when the file did not previously exist"
        );
    }

    #[test]
    fn test_write_known_hosts_additive_preserves_local_entries() {
        use std::fs;
        use tempfile::tempdir;

        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "hosts": ["new.example.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "NEW_KEY"
                        }
                    ]
                }
            ]
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");

        // Existing file has a local-only entry not present on the server
        fs::write(&file_path, "old.example.com ssh-rsa OLD_KEY\n").unwrap();

        // Default (additive) mode
        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_ok());

        let contents = fs::read_to_string(&file_path).unwrap();
        // Local-only entry is preserved
        assert!(contents.contains("old.example.com ssh-rsa OLD_KEY"));
        // Server entry is added
        assert!(contents.contains("new.example.com ssh-rsa NEW_KEY"));
    }

    #[test]
    fn test_write_known_hosts_additive_does_not_duplicate() {
        use std::fs;
        use tempfile::tempdir;

        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "hosts": ["github.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "SHARED_KEY"
                        }
                    ]
                }
            ]
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");

        // Existing file already contains the same entry (no comment locally)
        fs::write(&file_path, "github.com ssh-rsa SHARED_KEY\n").unwrap();

        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_ok());

        let contents = fs::read_to_string(&file_path).unwrap();
        // The entry should appear exactly once, not duplicated
        let occurrences = contents.matches("github.com ssh-rsa SHARED_KEY").count();
        assert_eq!(occurrences, 1);
    }

    #[test]
    fn test_write_known_hosts_additive_refreshes_comment() {
        use std::fs;
        use tempfile::tempdir;

        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "hosts": ["github.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "SHARED_KEY",
                            "comment": "GitHub RSA key"
                        }
                    ]
                }
            ]
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");

        // Existing entry matches the server key but lacks the comment
        fs::write(&file_path, "github.com ssh-rsa SHARED_KEY\n").unwrap();

        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_ok());

        let contents = fs::read_to_string(&file_path).unwrap();
        // The matching entry is refreshed with the server's comment, not duplicated
        assert_eq!(contents.matches("github.com ssh-rsa SHARED_KEY").count(), 1);
        assert!(contents.contains("# GitHub RSA key"));
    }

    #[test]
    fn test_write_known_hosts_server_error() {
        use tempfile::tempdir;

        let (server_url, _server) =
            setup_mock_server_with_error(500, r#"{"error": "Internal server error"}"#);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");

        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_err());

        // File should not be created on error
        assert!(!file_path.exists());
    }

    #[test]
    fn test_write_known_hosts_file_ends_with_newline() {
        use std::fs;
        use tempfile::tempdir;

        let mock_response = r#"
        {
            "version": "1.0.0",
            "knownHosts": [
                {
                    "hosts": ["example.com"],
                    "keys": [
                        {
                            "type": "ssh-rsa",
                            "key": "TEST_KEY"
                        }
                    ]
                }
            ]
        }
        "#;
        let (server_url, _server) = setup_mock_server(mock_response);

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("known_hosts");

        let result = write_known_hosts(&server_url, file_path.to_str().unwrap(), false);
        assert!(result.is_ok());

        let contents = fs::read_to_string(&file_path).unwrap();
        // File should end with a newline (POSIX standard)
        assert!(contents.ends_with('\n'));
    }
}
