use anyhow::{Context, Result};
use atty;
use colored::Colorize;
use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::utils::{ColumnConfig, pretty_print_table};

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

    response
        .json::<KnownHostsResponse>()
        .context("Failed to parse JSON response")
}

pub fn fetch_known_hosts(server_url: &str) -> Result<()> {
    let known_hosts_response = fetch_known_hosts_from_server(server_url)?;

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

/// Helper function to format a host entry in known_hosts format
fn format_known_hosts_line(host: &KnownHost, key: &HostKey) -> String {
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
        format!("{hosts_str} {key_type} {key_value}{comment_str}")
    } else {
        format!(
            "{} {} {} {}{}",
            flags.join(" "),
            hosts_str,
            key_type,
            key_value,
            comment_str
        )
    }
}

pub fn write_known_hosts(server_url: &str, file_path: &str) -> Result<()> {
    // Fetch known hosts from the server
    let known_hosts_response = fetch_known_hosts_from_server(server_url)?;

    // Expand ~ to home directory if present
    let expanded_path = shellexpand::tilde(file_path);
    let path = std::path::Path::new(expanded_path.as_ref());

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }

    // Generate the file content (always replace completely)
    let mut lines = Vec::new();
    for host in &known_hosts_response.hosts {
        for key in &host.keys {
            lines.push(format_known_hosts_line(host, key));
        }
    }

    let file_content = lines.join("\n");
    if !file_content.is_empty() {
        let file_content = format!("{}\n", file_content);
        std::fs::write(path, file_content)
            .with_context(|| format!("Failed to write to file: {}", path.display()))?;
    } else {
        // Write empty file if no hosts
        std::fs::write(path, "")
            .with_context(|| format!("Failed to write to file: {}", path.display()))?;
    }

    // Count the total number of entries written
    let total_entries: usize = known_hosts_response
        .hosts
        .iter()
        .map(|h| h.keys.len())
        .sum();

    println!(
        "✅ Wrote {} known host entries to {}",
        total_entries,
        path.display()
    );

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
}
